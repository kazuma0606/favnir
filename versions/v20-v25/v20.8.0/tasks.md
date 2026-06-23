# v20.8.0 — DB コネクションプール タスク

## ステータス: DONE

---

## タスク一覧

### T1: `fav/src/backend/pg_pool.rs` — `PgPoolInner` + `PgPoolStats` 新規実装

- [x] **事前確認**: `grep -n "pg_conn_str_from_env\|pg_execute\|pg_query" fav/src/backend/vm.rs | head -5` で既存 Postgres helper のパターンを確認
- [x] `fav/src/backend/pg_pool.rs` を新規作成（`fav/src/backend/mod.rs` に `#[cfg(not(target_arch = "wasm32"))] pub mod pg_pool;` も追加）
- [x] `PgPoolStats` struct を実装（`Debug`, `Default`, `Clone` derive）:
  - [x] `borrow_count: usize` — pool hit 回数
  - [x] `miss_count: usize` — pool miss（新規接続）回数
  - [x] `return_count: usize` — 接続を pool に返却した回数
  - [x] `error_count: usize` — 接続・クエリエラー回数
  - [x] `idle_count: usize` — 現在の idle 接続数
- [x] `PgPoolInner` struct を実装:
  - [x] `conn_str: String`
  - [x] `pool_size: usize`
  - [x] `idle: Mutex<Vec<tokio_postgres::Client>>`
  - [x] `stats: Mutex<PgPoolStats>`
- [x] `PgPoolInner::new(conn_str: &str, pool_size: usize) -> Arc<Self>` — 空プール作成（lazy 初期化、接続を事前確立しない）
- [x] `PgPoolInner::acquire(&self) -> Result<tokio_postgres::Client, String>`:
  - [x] idle あり → pop して `borrow_count` +1、`idle_count` 更新、Ok(client) を返す
  - [x] idle なし → `crate::backend::vm::pg_pool_runtime().block_on` で `tokio_postgres::connect` を呼ぶ（NoTls）
  - [x] 接続成功 → `tokio::spawn(async move { connection.await.ok(); })` で background task を spawn
  - [x] 接続失敗 → `error_count` +1、Err を返す
  - [x] `miss_count` +1（新規接続試行）
- [x] `PgPoolInner::release(&self, client: tokio_postgres::Client)`:
  - [x] `idle.len() < pool_size` → push して `return_count` +1、`idle_count` 更新
  - [x] pool 満杯 → client をドロップ（接続が閉じる）
- [x] `PgPoolInner::stats_snapshot(&self) -> PgPoolStats` — stats を clone して返す
- [x] `cargo check` でコンパイルエラー 0

---

### T2: `fav/src/backend/heap_val.rs` — `HeapVal::PgPool` 追加

- [x] **事前確認**: `grep -n "ArrowBatch" fav/src/backend/heap_val.rs` でパターンを確認
- [x] `HeapVal` enum に `PgPool(u64),` を追加（`ArrowBatch(u64)` の直後）
- [x] `PartialEq` impl に `(HeapVal::PgPool(a), HeapVal::PgPool(b)) => a == b,` を追加
- [x] `cargo check` でコンパイルエラー 0

---

### T3: `fav/src/backend/nan_val.rs` — PgPool 変換アーム追加

- [x] **事前確認**: `grep -n "ArrowBatch" fav/src/backend/nan_val.rs` で変換箇所を確認
- [x] `VMValue → NanVal` 変換に `VMValue::PgPool(id) => NanVal::from_heap(HeapVal::PgPool(id)),` を追加
- [x] `HeapVal → VMValue` 変換に `HeapVal::PgPool(id) => VMValue::PgPool(*id),` を追加
- [x] `cargo check` でコンパイルエラー 0

---

### T4: `fav/src/backend/vm.rs` — VMValue + グローバルストア + Primitives

#### 4-1. VMValue enum + PartialEq

- [x] `grep -n "ArrowBatch\|DbHandle" fav/src/backend/vm.rs | head -10` で既存パターンを確認
- [x] `VMValue` enum に `PgPool(u64),` を追加（`ArrowBatch(u64)` の直後）
- [x] `PartialEq` impl に `(VMValue::PgPool(a), VMValue::PgPool(b)) => a == b,` を追加

#### 4-2. exhaustive match 更新（6 箇所）

- [x] `nanval_type_name` 関数（`HeapVal` type_name match）に `HeapVal::PgPool(_) => "PgPool",` を追加
- [x] `vmvalue_type_name` 関数に `VMValue::PgPool(_) => "PgPool",` を追加
- [x] `vm_value_to_json` に `VMValue::PgPool(id) => Value::Str(format!("<pgpool:{id}>"))` を追加（`ArrowBatch` アームの直後）
- [x] `From<VMValue> for Value` impl（行 5071 付近）に `VMValue::PgPool(id) => Value::Str(format!("<pgpool:{id}>"))` を追加
- [x] `vmvalue_repr` 関数に `VMValue::PgPool(id) => format!("<pgpool:{}>", id),` を追加
- [x] `display_vmvalue` 関数に `VMValue::PgPool(_) => "<pgpool>".to_string(),` を追加

#### 4-3. グローバルストア + pg_pool_runtime

- [x] `// ── v20.8.0: PgPool グローバルストア` セクションを追加:
  - [x] `type PgPoolMap = HashMap<u64, Arc<crate::backend::pg_pool::PgPoolInner>>;`
  - [x] `static PG_POOLS: OnceLock<Mutex<PgPoolMap>> = OnceLock::new();`
  - [x] `static PG_POOL_NEXT_ID: AtomicU64 = AtomicU64::new(1);`
  - [x] `fn pg_pool_store() -> MutexGuard<'static, PgPoolMap>`
  - [x] `fn pg_pool_alloc(inner: Arc<PgPoolInner>) -> u64`
  - [x] `pub(crate) fn pg_pool_runtime() -> &'static tokio::runtime::Runtime`（`OnceLock` で 1 度だけ初期化、`new_multi_thread()`, `worker_threads(2)`, `thread_name("fav-pgpool")`）

#### 4-4. `Postgres.Pool.*` Primitives（vm_call_builtin に追加）

> `Postgres.Pool.*` は `self.chunk_arena` 等の VM フィールドにアクセスしないため
> `vm_call_builtin`（自由関数）に追加する（`call_builtin` ではない）。

- [x] `"Postgres.Pool.create"` ハンドラ:
  - [x] 引数 `pool_size: Int`（デフォルト 5）
  - [x] `pg_conn_str_from_env()` で接続文字列取得
  - [x] `PgPoolInner::new(&conn_str, pool_size)` で Arc を作成
  - [x] `pg_pool_alloc(inner)` で id を取得
  - [x] `ok_vm(VMValue::PgPool(id))` を返す
- [x] `"Postgres.Pool.query"` ハンドラ:
  - [x] 引数: `pool: PgPool, sql: Str, params: List`
  - [x] `pg_pool_store().get(&id).cloned()` でプールを取得
  - [x] `pool.acquire()` → クライアント取得（Err なら err_vm）
  - [x] `pg_pool_runtime().block_on(async { ... client.query(&sql, &params).await ... })` でクエリ実行
  - [x] `pool.release(client)` で返却
  - [x] 成功: `ok_vm(VMValue::List(FavList::new(rows)))` を返す
- [x] `"Postgres.Pool.execute"` ハンドラ:
  - [x] 引数: `pool: PgPool, sql: Str, params: List`
  - [x] `pool.acquire()` → クライアント取得
  - [x] `client.execute(&sql, &params).await` — `rows_affected: u64`
  - [x] `pool.release(client)` で返却
  - [x] 成功: `ok_vm(VMValue::Int(rows_affected as i64))` を返す
- [x] `"Postgres.Pool.stats"` ハンドラ:
  - [x] 引数: `pool: PgPool`
  - [x] `pool.stats_snapshot()` から `HashMap<String, VMValue>` を構築（5 フィールド）
  - [x] `ok_vm(VMValue::Record(map))` を返す
- [x] `"Postgres.Pool.close"` ハンドラ:
  - [x] 引数: `pool: PgPool`
  - [x] `pg_pool_store().remove(&id)` で Arc を削除（全 Client が drop されて接続が閉じる）
  - [x] `ok_vm(VMValue::Unit)` を返す
- [x] ヘルパー関数を実装（vm.rs 末尾付近）:
  - [x] `async fn pg_query_with_client(client: &tokio_postgres::Client, sql: &str, params_json: &str) -> Result<String, String>` — params をパースしてクエリ実行、行を JSON 文字列で返す
  - [x] `fn pg_json_to_vm_list(json_str: &str) -> Vec<VMValue>` — JSON 文字列を `VMValue::Record` の Vec に変換
- [x] `cargo check` でコンパイルエラー 0

---

### T5: `fav/src/toml.rs` — PostgresTomlConfig 拡張

- [x] **事前確認**: `grep -n "PostgresTomlConfig\|pool_size\|min_idle" fav/src/toml.rs` で既存構造を確認
- [x] **事前確認**: `grep -n "unwrap_or(PostgresTomlConfig\|PostgresTomlConfig {" fav/src/toml.rs` でデフォルト初期化箇所を特定
- [x] `PostgresTomlConfig` に `pub pool_size: Option<usize>,` と `pub min_idle: Option<usize>,` を追加
- [x] パース処理（`"postgres"` セクション内）に追加:
  - [x] `"pool_size" => current.pool_size = val.parse::<usize>().ok(),`
  - [x] `"min_idle"  => current.min_idle  = val.parse::<usize>().ok(),`
- [x] `PostgresTomlConfig` のデフォルト生成箇所（`unwrap_or(PostgresTomlConfig { ... })`）に `pool_size: None, min_idle: None` を追加
- [x] `cargo check` でコンパイルエラー 0

---

### T6: `fav/src/middle/compiler.rs` — builtin リスト追加

- [x] **事前確認**: `grep -n "ArrowBatch\|__duckdb_push\|Arena.stats" fav/src/middle/compiler.rs | head -5` で追加位置を確認
- [x] `"Arena.stats"` の直後に追加:
  ```rust
  // v20.8.0 Postgres Pool
  "Postgres.Pool.create",
  "Postgres.Pool.query",
  "Postgres.Pool.execute",
  "Postgres.Pool.stats",
  "Postgres.Pool.close",
  ```
- [x] `cargo check` でコンパイルエラー 0

---

### T7: `fav/src/driver.rs` — `v208000_tests`

- [x] `driver.rs` 末尾に `#[cfg(test)] mod v208000_tests { ... }` を追加
- [x] テスト 1: `version_is_20_8_0` — `include_str!("../Cargo.toml")` に `"20.8.0"` が含まれる
- [x] テスト 2: `pg_pool_stats_default_zero` — `PgPoolStats::default()` の全 5 フィールドが 0
- [x] テスト 3: `pg_pool_inner_pool_size` — `PgPoolInner::new("...", 7)` で `pool_size == 7`、`idle_count == 0`
- [x] テスト 4: `pg_pool_toml_pool_size_min_idle` — `[postgres]\npool_size = 8\nmin_idle = 3` が正しくパースされる
- [x] テスト 5: `pg_pool_acquire_increments_miss_on_failure` — `"postgresql://127.0.0.1:19999/noexist?connect_timeout=1"` で acquire が Err、`miss_count == 1`、`error_count == 1`（`?connect_timeout=1` でタイムアウトを 1s に制限）
- [x] 各テストで `#[cfg(not(target_arch = "wasm32"))]` ガードを付与
- [x] `cargo test v208000` — 5/5 PASS を確認

---

### T8: `fav/Cargo.toml` バージョン更新

- [x] `version = "20.7.0"` → `"20.8.0"` に変更
- [x] 既存の `version_is_20_7_0` テストに `#[ignore]` を追加

---

### T9: `CHANGELOG.md` + `benchmarks/v20.8.0.json` + サイトドキュメント

- [x] `CHANGELOG.md` の先頭に v20.8.0 エントリを追加:
  - [x] `### Added` — `PgPool`、`PgPoolStats`、`pg_pool_runtime`、`VMValue::PgPool`、5 primitives、`pool_size`/`min_idle` fav.toml 追加
  - [x] `### Performance` — `pg_pipeline_10stage_ms` +5〜10x（期待値）
- [x] `benchmarks/v20.8.0.json` を生成（実測または期待値）
- [x] `site/content/docs/runes/postgres.mdx` を更新（`## Postgres.Pool` セクション追加）:
  - [x] `Postgres.Pool.create(pool_size)` シグネチャ・説明・使用例
  - [x] `Postgres.Pool.query / execute / stats / close` の説明
  - [x] `PgPoolStats` フィールド一覧
  - [x] WASM 非対応の注意書き
  - [x] `fav.toml` の `pool_size` / `min_idle` 設定例

---

## テスト（v208000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_8_0` | `Cargo.toml` に `"20.8.0"` が含まれる |
| `pg_pool_stats_default_zero` | `PgPoolStats::default()` の全フィールドが 0 |
| `pg_pool_inner_pool_size` | `PgPoolInner::new("...", 7)` で `pool_size == 7`、初期 `idle_count == 0` |
| `pg_pool_toml_pool_size_min_idle` | `fav.toml` の `pool_size = 8` / `min_idle = 3` が正しくパースされる |
| `pg_pool_acquire_increments_miss_on_failure` | 接続不可能 URL で Err、`miss_count == 1`、`error_count == 1` |

---

## 完了条件チェックリスト

- [x] `PgPoolInner::new` が lazy（接続を事前確立しない）
- [x] `PgPoolInner::acquire` が pool hit 時に `borrow_count` を増加させ、miss 時に `miss_count` を増加させる
- [x] `PgPoolInner::release` が `pool_size` 未満の場合のみ pool に返却する
- [x] `VMValue::PgPool(u64)` が追加されており、exhaustive match が全 5 箇所で更新済み
- [x] `HeapVal::PgPool(u64)` が追加されており、`nan_val.rs` の変換が双方向で追加済み
- [x] `pg_pool_runtime()` が専用長寿命 tokio runtime を返す（`new_multi_thread`、`worker_threads(2)`）
- [x] `Postgres.Pool.create / query / execute / stats / close` が `vm_call_builtin` に追加されている
- [x] `fav.toml` の `[postgres]` セクションで `pool_size` / `min_idle` がパースできる
- [x] compiler.rs の builtin リストに 5 primitives が追加されている
- [x] `cargo test v208000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
- [x] `fav/Cargo.toml` version が `20.8.0`
- [x] `CHANGELOG.md` に v20.8.0 エントリが追加されている
- [x] `benchmarks/v20.8.0.json` が生成されている
- [x] `grep -n "Effect::Postgres\|Postgres.*effect" fav/src/middle/checker.rs` でエフェクトチェックが `PgPool` 操作を正しくカバーしていることを確認

---

## 優先度

```
T1（pg_pool.rs）    ← 他すべての前提
T2（heap_val.rs）   ← T1 完了後
T3（nan_val.rs）    ← T2 完了後
T4（vm.rs 統合）    ← T3 完了後（最大工数）
T5（toml.rs）       ← T1 と並列可
T6（compiler.rs）   ← T4 完了後
T7（driver.rs）     ← T4〜T6 完了後
T8（バージョン）    ← 任意タイミング
T9（CHANGELOG）     ← T7 完了後
```

---

## 実装リスク と 対策

| リスク | 対策 |
|---|---|
| `pg_pool_runtime().block_on()` が既存 tokio context 内から呼ばれて panic | block_on は runtime 外（driver 層）からのみ呼ぶ設計。vm_call_builtin は tokio context 外で実行される |
| `tokio_postgres::Client` が runtime をまたぐと接続が切断される | background task を pg_pool_runtime で spawn するため、Client はその runtime の context で動く |
| exhaustive match 漏れ（`VMValue::PgPool` を追加したのに一部 match が未更新） | `cargo check` でコンパイルエラーとして検出される。T4 完了後に必ず `cargo check` を実行 |
| テスト 5 の `pg_pool_acquire` が接続 timeout で CI をブロック | `127.0.0.1:19999` は通常 refused（すぐに Err）。timeout が問題になる場合は `connect_timeout` を 1s に設定 |
| `pg_pool_toml_pool_size_min_idle` テストで `parse_fav_toml` の API が不明 | `grep -n "pub fn parse_fav_toml" fav/src/toml.rs` で確認してから使用 |
| `Postgres.Pool.query` の params 変換（VMValue → tokio_postgres params） | 既存 `pg_query` の JSON 経由変換ロジックを再利用（`serde_json::to_string` → JSON 文字列 → `&dyn ToSql`） |
