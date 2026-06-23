# v20.8.0 Spec — DB コネクションプール統合

## 概要

v20.8.0 はデータベースを使うパイプラインの「接続確立コスト」を根本的に排除する。

**問題**: 現在の `Postgres.execute_raw` / `Postgres.query_raw` は呼び出しごとに
新規接続を確立・切断する（`pg_execute` / `pg_query` がそれぞれ独立した tokio runtime と
TCP 接続を作成）。接続確立コストは ~50ms / 回 であり、
多数の stage が DB を使うパイプラインでは累積コストが無視できない。

**解決**: `PgPool` — 接続を事前に確立し、再利用するプールを導入する。
プールは `VMValue::PgPool(u64)` のオペーク handle として Favnir に公開し、
`Postgres.Pool.query` / `Postgres.Pool.execute` でプール経由の操作を提供する。

**テーマ**: Runtime Excellence シリーズ第8弾 — DB コネクションプール統合

---

## 動機と期待効果

### 現状の問題

```rust
// vm.rs — Postgres.execute_raw（現状）
pub fn pg_execute(conn_str: &str, sql: &str, params_json: &str) -> Result<(), String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // ❌ 毎回 TCP handshake + TLS negotiation + Postgres auth
            let (client, connection) = tokio_postgres::connect(conn_str, NoTls).await?;
            tokio::spawn(async move { connection.await.ok(); });
            client.execute(sql, &params).await?;
        })
}
```

10 stage × DB アクセス = 10 回接続確立 × ~50ms = **~500ms のオーバーヘッド**

### 最適化後（PgPool）

```favnir
-- プールを一度作成して stage に渡す
let pool = Postgres.Pool.create(10)  -- pool_size: 10 connections

-- プールを使って DB 操作（接続確立コストゼロ）
bind result <- Postgres.Pool.query(pool, "SELECT * FROM users WHERE id = $1", [user_id])
```

### 期待改善（v20.7.0 比）

| ベンチマーク | v20.7.0 基準 | 期待改善 |
|---|---|---|
| `pg_stage_first_call_ms` | ~50ms（接続確立含む） | **-45ms**（プール再利用時） |
| `pg_pipeline_10stage_ms` | ~520ms（10回接続） | **-450ms（+5〜10x）** |
| `pg_pool_reuse_rate_pct` | 0%（再利用なし） | **>95%**（プール hit） |

> **注**: 上記の改善値は実 PostgreSQL 環境での計測を前提とした期待値。
> ローカル接続（`127.0.0.1`）では接続確立コストが ~5ms 以下になる場合があり、
> 実測値はネットワーク構成・Postgres バージョンにより乖離する可能性がある。

---

## アーキテクチャ

### グローバル プール runtime

既存の `pg_execute` / `pg_query` は呼び出しごとに
`tokio::runtime::Builder::new_current_thread()` でランタイムを作成するため、
接続 background task が呼び出し完了と同時に消える。

プールのために **長寿命の専用 tokio runtime** を導入する:

```rust
// vm.rs — プール専用グローバル runtime（一度だけ初期化）
fn pg_pool_runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("fav-pgpool")
            .build()
            .unwrap()
    })
}
```

### PgPoolInner 構造体（新規ファイル `src/backend/pg_pool.rs`）

```rust
#[derive(Debug, Default, Clone)]
pub struct PgPoolStats {
    pub borrow_count:  usize,  // プールから接続を借りた回数（pool hit）
    pub miss_count:    usize,  // プールが空で新規接続した回数（pool miss）
    pub return_count:  usize,  // 接続をプールに返却した回数
    pub error_count:   usize,  // 接続/クエリエラー回数
    pub idle_count:    usize,  // 現在の idle 接続数
}

pub(crate) struct PgPoolInner {
    pub(crate) conn_str:  String,
    pub(crate) pool_size: usize,
    pub(crate) idle:      Mutex<Vec<tokio_postgres::Client>>,
    pub(crate) stats:     Mutex<PgPoolStats>,
}

impl PgPoolInner {
    pub(crate) fn new(conn_str: &str, pool_size: usize) -> Arc<Self> { ... }
    pub(crate) fn acquire(&self) -> Result<tokio_postgres::Client, String> { ... }
    pub(crate) fn release(&self, client: tokio_postgres::Client) { ... }
    pub(crate) fn stats_snapshot(&self) -> PgPoolStats { ... }
}
```

`acquire()` の動作:
- `idle` mutex を lock してクライアントを pop
- idle あり → `borrow_count` +1、クライアントを返す（pool hit）
- idle なし → `pg_pool_runtime().block_on` で新規接続作成、`miss_count` +1

`release()` の動作:
- `idle.len() < pool_size` → push して `return_count` +1、`idle_count` 更新
- それ以外 → ドロップ（プールが満杯）

### グローバル プールストア（vm.rs）

ArrowBatch / DbHandle パターンに倣う:

```rust
// vm.rs — グローバル PgPool ストア
type PgPoolMap = HashMap<u64, Arc<PgPoolInner>>;
static PG_POOLS: OnceLock<Mutex<PgPoolMap>> = OnceLock::new();
static PG_POOL_NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn pg_pool_store() -> MutexGuard<'static, PgPoolMap> {
    PG_POOLS.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap()
}
```

### VMValue::PgPool(u64) — 新バリアント

`DbHandle(u64)` / `ArrowBatch(u64)` と同じパターン:

```rust
// vm.rs — VMValue enum に追加
pub(crate) enum VMValue {
    // ... 既存バリアント ...
    DbHandle(u64),
    TxHandle(u64),
    ArrowBatch(u64),
    PgPool(u64),  // ← v20.8.0 追加
}
```

**更新が必要な箇所（exhaustive match）:**

| ファイル | 箇所 | 追加内容 |
|---|---|---|
| `heap_val.rs` | `HeapVal` enum | `PgPool(u64)` |
| `heap_val.rs` | `PartialEq` impl | `(HeapVal::PgPool(a), HeapVal::PgPool(b)) => a == b` |
| `nan_val.rs` | `VMValue → NanVal` 変換 | `VMValue::PgPool(id) => NanVal::from_heap(HeapVal::PgPool(id))` |
| `nan_val.rs` | `HeapVal → VMValue` 変換 | `HeapVal::PgPool(id) => VMValue::PgPool(*id)` |
| `vm.rs` | `VMValue` enum | `PgPool(u64)` |
| `vm.rs` | `PartialEq` impl | `(VMValue::PgPool(a), VMValue::PgPool(b)) => a == b` |
| `vm.rs` | `HeapVal` type_name match | `HeapVal::PgPool(_) => "PgPool"` |
| `vm.rs` | `vmvalue_type_name` | `VMValue::PgPool(_) => "PgPool"` |
| `vm.rs` | `vm_value_to_json` | `VMValue::PgPool(id) => Value::Str(format!("<pgpool:{id}>"))` |
| `vm.rs` | `From<VMValue> for Value` impl（行 5071 付近） | `VMValue::PgPool(id) => Value::Str(format!("<pgpool:{id}>"))` |
| `vm.rs` | `vmvalue_repr` | `VMValue::PgPool(id) => format!("<pgpool:{}>", id)` |
| `vm.rs` | `display_vmvalue` | `VMValue::PgPool(_) => "<pgpool>".to_string()` |

### VM Primitives（`vm_call_builtin` 自由関数に追加）

`PgPool` は `self.chunk_arena` 等の VM フィールドにアクセスしないため
`vm_call_builtin`（自由関数）に追加する。

| Primitive | 引数 | 返値 |
|---|---|---|
| `Postgres.Pool.create` | `pool_size: Int` | `Result<PgPool>` |
| `Postgres.Pool.query` | `pool: PgPool, sql: Str, params: List` | `Result<List<Record>>` |
| `Postgres.Pool.execute` | `pool: PgPool, sql: Str, params: List` | `Result<Int>` |
| `Postgres.Pool.stats` | `pool: PgPool` | `Result<Record>` |
| `Postgres.Pool.close` | `pool: PgPool` | `Result<Unit>` |

環境変数 `POSTGRES_URL`（または `fav.toml` の `[postgres]` 設定）から接続文字列を取得する。

### `fav.toml` `[postgres]` セクション拡張

```toml
[postgres]
host     = "localhost"
port     = 5432
dbname   = "mydb"
user     = "admin"
password = "${POSTGRES_PASSWORD}"
pool_size = 10   # ← v20.8.0 追加（デフォルト: 5）
min_idle  = 2    # ← v20.8.0 追加（デフォルト: 1）
```

```rust
// toml.rs — PostgresTomlConfig に追加
pub struct PostgresTomlConfig {
    // 既存フィールド
    pub host:     Option<String>,
    pub port:     Option<u16>,
    pub dbname:   Option<String>,
    pub user:     Option<String>,
    pub password: Option<String>,
    pub sslmode:  Option<String>,
    // v20.8.0 追加
    pub pool_size: Option<usize>,
    pub min_idle:  Option<usize>,
}
```

---

## 使用例

```favnir
-- プールを作成（pool_size=5、fav.toml の [postgres] 設定を使用）
let pool = Postgres.Pool.create(5)

-- プール経由でクエリ（接続確立コストゼロ）
bind users <- Postgres.Pool.query(pool, "SELECT id, name FROM users", [])

-- プール経由で書き込み
bind _ <- Postgres.Pool.execute(pool,
  "INSERT INTO logs (msg, ts) VALUES ($1, NOW())",
  ["pipeline started"])

-- 統計を確認
let stats = Postgres.Pool.stats(pool)
-- { borrow_count: 2, miss_count: 1, return_count: 2, error_count: 0, idle_count: 1 }

-- プールを閉じる（オプション — プロセス終了時に自動クローズ）
Postgres.Pool.close(pool)
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `20.7.0` → `20.8.0` |
| `fav/src/backend/pg_pool.rs` | `PgPoolInner` / `PgPoolStats` 新規実装（新規ファイル） |
| `fav/src/backend/mod.rs` | `pub mod pg_pool;`（`#[cfg(not(target_arch = "wasm32"))]` ガード付き）追加 |
| `fav/src/backend/heap_val.rs` | `HeapVal::PgPool(u64)` 追加 |
| `fav/src/backend/nan_val.rs` | `PgPool` 変換アーム追加 |
| `fav/src/backend/vm.rs` | `VMValue::PgPool(u64)`、グローバルストア、`pg_pool_runtime`、primitives、exhaustive match 更新 |
| `fav/src/toml.rs` | `PostgresTomlConfig` に `pool_size` / `min_idle` 追加 |
| `fav/src/middle/compiler.rs` | builtin リストに `Postgres.Pool.*` 追加 |
| `fav/src/driver.rs` | `v208000_tests`（5 件）、`version_is_20_7_0` に `#[ignore]` 追加 |
| `CHANGELOG.md` | v20.8.0 エントリ追加 |
| `benchmarks/v20.8.0.json` | 実測ベンチマーク結果 |
| `site/content/docs/runes/postgres.mdx` | `Postgres.Pool.*` ドキュメント追加 |

---

## テスト（v208000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_8_0` | `Cargo.toml` に `"20.8.0"` が含まれる |
| `pg_pool_stats_default_zero` | `PgPoolStats::default()` の全フィールドが 0 |
| `pg_pool_inner_pool_size` | `PgPoolInner::new("conn", 7)` で `pool_size == 7`、`idle_count == 0` |
| `pg_pool_toml_pool_size_min_idle` | `fav.toml` の `pool_size = 8` / `min_idle = 3` が正しくパースされる |
| `pg_pool_acquire_increments_miss_on_failure` | `PgPoolInner` を接続不可 URL（`?connect_timeout=1`）で acquire すると `miss_count` / `error_count` が増加する（実 DB 不要） |

> テストは実際の Postgres 接続を必要としない設計とする（`PgPoolInner::new` は lazy 初期化）。
> `pg_pool_acquire_increments_miss` は接続エラーを期待値とし、
> `miss_count` が増加かつ `error_count` が増加することを確認する。

---

## 完了条件

- [ ] `PgPoolInner::acquire` / `release` が接続を pool から再利用する
- [ ] `VMValue::PgPool(u64)` が追加されており、exhaustive match が全箇所で更新済み
- [ ] `Postgres.Pool.create` / `query` / `execute` / `stats` / `close` が `vm_call_builtin` に追加
- [ ] `fav.toml` の `[postgres]` セクションで `pool_size` / `min_idle` がパースできる
- [ ] `pg_pool_runtime()` が専用長寿命 tokio runtime を返す
- [ ] `cargo test v208000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `fav/Cargo.toml` version が `20.8.0`
- [ ] `CHANGELOG.md` に v20.8.0 エントリが追加されている
- [ ] `benchmarks/v20.8.0.json` が生成されている
- [ ] `pg_pipeline_10stage_ms` が v20.7.0 比 +5x 以上改善（実 DB 環境での計測）
- [ ] `grep -n "Effect::Postgres\|Postgres.*effect" fav/src/middle/checker.rs` でエフェクトチェックに `PgPool` 操作が漏れていないことを確認

---

## 技術ノート

### なぜ専用 runtime が必要か

`tokio_postgres::connect()` は `(Client, Connection)` を返す。
`Connection` は async task として spawn しないと TCP 接続が機能しない。
既存の `pg_execute` は `new_current_thread().block_on()` で接続を作り、
block_on が終わると runtime が drop されて Connection task も消える。

プールでは Connection task を生き続けさせるため、
**長寿命の multi-thread runtime（`pg_pool_runtime`）** でタスクを spawn し、
Client をプールに格納する必要がある。

### TLS サポート

v20.8 では NoTls のみ対応（`sslmode = "disable"` 相当）。
既存の `tokio_postgres_rustls` を使った TLS 対応は v20.9 以降で追加予定。

### `is_known_builtin_namespace` への追加不要

`Postgres.Pool.create` のような `Postgres.Pool.*` 形式の名前は、
`is_known_builtin_namespace` が `split('.').next()` で `"Postgres"` を取得するため、
既存の `"Postgres"` エントリで自動的に認識される。追加不要。

### `min_idle` の実装について

v20.8 では `min_idle` はパース・保存するが、実際の prewarming（接続の事前確立）は
`Postgres.Pool.create` 呼び出し時に行わない（lazy 初期化）。
`min_idle` は将来の background warmup（v20.9）のために値を保存するのみ。
