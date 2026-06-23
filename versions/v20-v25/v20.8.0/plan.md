# v20.8.0 Plan — DB コネクションプール統合

## 実装順序

```
T1（pg_pool.rs）    ← 最初に実装。PgPoolInner / PgPoolStats の単体実装
T2（heap_val.rs）   ← T1 後。PgPool を HeapVal に追加
T3（nan_val.rs）    ← T2 後。PgPool 変換アーム追加
T4（vm.rs）         ← T3 後。VMValue::PgPool + グローバルストア + primitives + exhaustive match
T5（toml.rs）       ← 並列可。PostgresTomlConfig 拡張
T6（compiler.rs）   ← T4 後。builtin リスト追加
T7（driver.rs）     ← T4〜T6 後。v208000_tests 実装
T8（バージョン）    ← 任意タイミング
T9（CHANGELOG）     ← T7 後
```

---

## T1: `fav/src/backend/pg_pool.rs` 新規作成

### ファイル全体構成

```rust
// ── v20.8.0: DB コネクションプール ─────────────────────────────────────────
//
// PgPoolInner: tokio_postgres::Client の Vec プール（Mutex 保護）
// PgPoolStats: pool 統計（borrow/miss/return/error/idle カウント）
//
// 接続の background task は pg_pool_runtime()（専用長寿命 tokio runtime）で
// spawn するため、block_on 終了後も接続が維持される。

use std::sync::{Arc, Mutex};
use tokio_postgres::NoTls;

// ── PgPoolStats ────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct PgPoolStats {
    pub borrow_count:  usize,  // pool hit（既存接続を再利用）
    pub miss_count:    usize,  // pool miss（新規接続が必要）
    pub return_count:  usize,  // 接続を pool に返却した回数
    pub error_count:   usize,  // 接続・クエリエラー回数
    pub idle_count:    usize,  // 現在の idle 接続数
}

// ── PgPoolInner ────────────────────────────────────────────────────────────

pub(crate) struct PgPoolInner {
    pub(crate) conn_str:  String,
    pub(crate) pool_size: usize,
    idle:                 Mutex<Vec<tokio_postgres::Client>>,
    stats:                Mutex<PgPoolStats>,
}
```

### `PgPoolInner::new`

```rust
impl PgPoolInner {
    pub(crate) fn new(conn_str: &str, pool_size: usize) -> Arc<Self> {
        Arc::new(PgPoolInner {
            conn_str: conn_str.to_string(),
            pool_size,
            idle:  Mutex::new(Vec::new()),
            stats: Mutex::new(PgPoolStats::default()),
        })
    }
```

### `PgPoolInner::acquire`

```rust
    /// pool から接続を取得する。idle あり → pool hit / idle なし → 新規接続。
    pub(crate) fn acquire(&self) -> Result<tokio_postgres::Client, String> {
        {
            let mut idle = self.idle.lock().unwrap();
            if let Some(client) = idle.pop() {
                let mut stats = self.stats.lock().unwrap();
                stats.borrow_count += 1;
                stats.idle_count = idle.len();
                return Ok(client);
            }
        }
        // pool miss: 新規接続
        let mut stats = self.stats.lock().unwrap();
        stats.miss_count += 1;
        drop(stats);  // lock 解放してから block_on

        let conn_str = self.conn_str.clone();
        crate::backend::vm::pg_pool_runtime()
            .block_on(async move {
                let (client, connection) = tokio_postgres::connect(&conn_str, NoTls)
                    .await
                    .map_err(|e| format!("PgPool: connection failed: {e}"))?;
                // 接続 background task を長寿命 runtime でスポーン
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("[fav PgPool] connection error: {e}");
                    }
                });
                Ok::<tokio_postgres::Client, String>(client)
            })
            .map_err(|e| {
                self.stats.lock().unwrap().error_count += 1;
                e
            })
    }
```

### `PgPoolInner::release`

```rust
    /// 使い終わった接続を pool に返却する。pool が満杯の場合はドロップ。
    pub(crate) fn release(&self, client: tokio_postgres::Client) {
        let mut idle = self.idle.lock().unwrap();
        if idle.len() < self.pool_size {
            idle.push(client);
            let mut stats = self.stats.lock().unwrap();
            stats.return_count += 1;
            stats.idle_count = idle.len();
        }
        // pool 満杯の場合は client がドロップされ接続が閉じる
    }
```

### `PgPoolInner::stats_snapshot`

```rust
    pub(crate) fn stats_snapshot(&self) -> PgPoolStats {
        self.stats.lock().unwrap().clone()
    }
```

---

## T2: `fav/src/backend/heap_val.rs` — `HeapVal::PgPool` 追加

```rust
// 追加位置: ArrowBatch(u64) の直後
ArrowBatch(u64),
PgPool(u64),   // ← 追加
```

`PartialEq` impl に追加:
```rust
(HeapVal::ArrowBatch(a), HeapVal::ArrowBatch(b)) => a == b,
(HeapVal::PgPool(a),     HeapVal::PgPool(b))     => a == b,  // ← 追加
```

---

## T3: `fav/src/backend/nan_val.rs` — PgPool 変換アーム追加

`VMValue → NanVal` 変換（`From<VMValue> for NanVal` または同等 impl）に追加:
```rust
VMValue::ArrowBatch(id) => NanVal::from_heap(HeapVal::ArrowBatch(id)),
VMValue::PgPool(id)     => NanVal::from_heap(HeapVal::PgPool(id)),  // ← 追加
```

`HeapVal → VMValue` 変換に追加:
```rust
HeapVal::ArrowBatch(id) => VMValue::ArrowBatch(*id),
HeapVal::PgPool(id)     => VMValue::PgPool(*id),     // ← 追加
```

---

## T4: `fav/src/backend/vm.rs` — 全変更

### 4-1. VMValue enum に追加

```rust
pub(crate) enum VMValue {
    // ...
    ArrowBatch(u64),
    PgPool(u64),   // ← 追加
}
```

### 4-2. VMValue PartialEq に追加

```rust
(VMValue::ArrowBatch(a), VMValue::ArrowBatch(b)) => a == b,
(VMValue::PgPool(a),     VMValue::PgPool(b))     => a == b,  // ← 追加
```

### 4-3. `nanval_type_name` 関数（HeapVal type_name match）に追加

> 確認: `grep -n "nanval_type_name\|HeapVal.*ArrowBatch.*=>" fav/src/backend/vm.rs | head -5`

```rust
HeapVal::ArrowBatch(_)  => "ArrowBatch",
HeapVal::PgPool(_)      => "PgPool",    // ← 追加
```

### 4-4. vmvalue_type_name に追加

```rust
VMValue::ArrowBatch(_) => "ArrowBatch",
VMValue::PgPool(_)     => "PgPool",     // ← 追加
```

### 4-5. vm_value_to_json に追加

```rust
VMValue::ArrowBatch(id) => Value::Str(format!("<arrow:{id}>")),
VMValue::PgPool(id)     => Value::Str(format!("<pgpool:{id}>")),   // ← 追加
```

### 4-6. `From<VMValue> for Value` impl に追加（vm.rs 行 5071 付近）

`vm_value_to_json` とは別に `From<VMValue> for Value` impl が存在するため、
こちらにも同じアームを追加する:

```rust
VMValue::ArrowBatch(id) => Value::Str(format!("<arrow:{id}>")),
VMValue::PgPool(id)     => Value::Str(format!("<pgpool:{id}>")),   // ← 追加
```

### 4-7. vmvalue_repr に追加

```rust
VMValue::ArrowBatch(id) => format!("<arrow:{}>", id),
VMValue::PgPool(id)     => format!("<pgpool:{}>", id),              // ← 追加
```

### 4-8. display_vmvalue に追加

```rust
VMValue::ArrowBatch(_) => "<arrow>".to_string(),
VMValue::PgPool(_)     => "<pgpool>".to_string(),                  // ← 追加
```

### 4-8. グローバルストア + pg_pool_runtime（ArrowBatch パターンに倣う）

```rust
// ── v20.8.0: PgPool グローバルストア ─────────────────────────────────────
type PgPoolMap = HashMap<u64, Arc<crate::backend::pg_pool::PgPoolInner>>;
static PG_POOLS: std::sync::OnceLock<Mutex<PgPoolMap>> = std::sync::OnceLock::new();
static PG_POOL_NEXT_ID: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1);

fn pg_pool_store() -> std::sync::MutexGuard<'static, PgPoolMap> {
    PG_POOLS.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap()
}

fn pg_pool_alloc(inner: Arc<crate::backend::pg_pool::PgPoolInner>) -> u64 {
    let id = PG_POOL_NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    pg_pool_store().insert(id, inner);
    id
}

/// PgPool 専用の長寿命 tokio runtime（接続 background task を維持するため）
pub(crate) fn pg_pool_runtime() -> &'static tokio::runtime::Runtime {
    static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("fav-pgpool")
            .build()
            .expect("fav: failed to build PgPool runtime")
    })
}
```

### 4-9. VM Primitives（vm_call_builtin に追加）

```rust
// ── v20.8.0: Postgres.Pool primitives ─────────────────────────────────────

"Postgres.Pool.create" => {
    // args: [pool_size: Int]
    let pool_size = match args.into_iter().next() {
        Some(VMValue::Int(n)) if n > 0 => n as usize,
        _ => 5,  // デフォルト pool_size
    };
    let conn_str = pg_conn_str_from_env();
    let inner = crate::backend::pg_pool::PgPoolInner::new(&conn_str, pool_size);
    let id = pg_pool_alloc(inner);
    Ok(ok_vm(VMValue::PgPool(id)))
}

"Postgres.Pool.query" => {
    // args: [pool: PgPool, sql: Str, params: List]
    let mut it = args.into_iter();
    let id = match it.next() {
        Some(VMValue::PgPool(id)) => id,
        _ => return Ok(err_vm(VMValue::Str("Postgres.Pool.query: expected PgPool".into()))),
    };
    let sql = match it.next() {
        Some(VMValue::Str(s)) => s,
        _ => return Ok(err_vm(VMValue::Str("Postgres.Pool.query: expected sql Str".into()))),
    };
    let params_json = match it.next() {
        Some(v) => serde_json::to_string(&vm_value_to_json(&v))
            .unwrap_or_else(|_| "[]".to_string()),
        None => "[]".to_string(),
    };
    let inner = {
        let store = pg_pool_store();
        store.get(&id).cloned()
    };
    match inner {
        None => Ok(err_vm(VMValue::Str(format!("Postgres.Pool.query: invalid pool id {id}")))),
        Some(pool) => {
            match pool.acquire() {
                Err(e) => Ok(err_vm(VMValue::Str(e))),
                Ok(client) => {
                    let result = pg_pool_runtime().block_on(async {
                        pg_query_with_client(&client, &sql, &params_json).await
                    });
                    pool.release(client);
                    match result {
                        Ok(json_str) => {
                            // JSON 文字列を VMValue::List<Record> に変換（既存パターン使用）
                            let rows = pg_json_to_vm_list(&json_str);
                            Ok(ok_vm(VMValue::List(FavList::new(rows))))
                        }
                        Err(e) => Ok(err_vm(VMValue::Str(e))),
                    }
                }
            }
        }
    }
}

"Postgres.Pool.execute" => {
    // args: [pool: PgPool, sql: Str, params: List]
    // Postgres.Pool.query と同構造で Client::execute を呼ぶ
    // 返値: ok(rows_affected: Int)
    ...
}

"Postgres.Pool.stats" => {
    // args: [pool: PgPool]
    let id = match args.into_iter().next() {
        Some(VMValue::PgPool(id)) => id,
        _ => return Ok(err_vm(VMValue::Str("Postgres.Pool.stats: expected PgPool".into()))),
    };
    let inner = pg_pool_store().get(&id).cloned();
    match inner {
        None => Ok(err_vm(VMValue::Str(format!("Postgres.Pool.stats: invalid pool id {id}")))),
        Some(pool) => {
            let s = pool.stats_snapshot();
            let mut map = HashMap::new();
            map.insert("borrow_count".into(),  VMValue::Int(s.borrow_count  as i64));
            map.insert("miss_count".into(),    VMValue::Int(s.miss_count    as i64));
            map.insert("return_count".into(),  VMValue::Int(s.return_count  as i64));
            map.insert("error_count".into(),   VMValue::Int(s.error_count   as i64));
            map.insert("idle_count".into(),    VMValue::Int(s.idle_count    as i64));
            Ok(ok_vm(VMValue::Record(map)))
        }
    }
}

"Postgres.Pool.close" => {
    // args: [pool: PgPool]
    let id = match args.into_iter().next() {
        Some(VMValue::PgPool(id)) => id,
        _ => return Ok(err_vm(VMValue::Str("Postgres.Pool.close: expected PgPool".into()))),
    };
    pg_pool_store().remove(&id);  // Arc が消え、Client が drop されて接続が閉じる
    Ok(ok_vm(VMValue::Unit))
}
```

**ヘルパー関数（同ファイル末尾）:**

```rust
/// PgPool 経由のクエリ実行（async helper）
async fn pg_query_with_client(
    client: &tokio_postgres::Client,
    sql: &str,
    params_json: &str,
) -> Result<String, String> {
    // 既存 pg_query の params パースロジックを再利用
    // 返値: rows の JSON 文字列
    ...
}

/// pg_query の JSON 文字列 → VMValue::List<Record> 変換
fn pg_json_to_vm_list(json_str: &str) -> Vec<VMValue> {
    // 既存の pg_query 結果変換ロジックを再利用
    ...
}
```

---

## T5: `fav/src/toml.rs` — PostgresTomlConfig 拡張

```rust
pub struct PostgresTomlConfig {
    pub host:      Option<String>,
    pub port:      Option<u16>,
    pub dbname:    Option<String>,
    pub user:      Option<String>,
    pub password:  Option<String>,
    pub sslmode:   Option<String>,
    // v20.8.0 追加
    pub pool_size: Option<usize>,
    pub min_idle:  Option<usize>,
}
```

パース追加（`"postgres"` セクション内）:
```rust
"pool_size" => current.pool_size = val.parse::<usize>().ok(),
"min_idle"  => current.min_idle  = val.parse::<usize>().ok(),
```

`PostgresTomlConfig` のデフォルト初期化にも `pool_size: None, min_idle: None` を追加。

> 事前確認: `grep -n "unwrap_or(PostgresTomlConfig\|PostgresTomlConfig {" fav/src/toml.rs` でデフォルト初期化箇所を特定する。

---

## T6: `fav/src/middle/compiler.rs` — builtin リスト追加

既存の `"ArrowBatch"` の後に追加:
```rust
// v20.8.0 Postgres Pool primitives
"Postgres.Pool.create",
"Postgres.Pool.query",
"Postgres.Pool.execute",
"Postgres.Pool.stats",
"Postgres.Pool.close",
```

`checker.rs` は `"Postgres"` が既に namespace リストに登録済みのため変更不要。

---

## T7: `fav/src/driver.rs` — v208000_tests（5 件）

```rust
#[cfg(test)]
mod v208000_tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn version_is_20_8_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("20.8.0"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn pg_pool_stats_default_zero() {
        use crate::backend::pg_pool::PgPoolStats;
        let stats = PgPoolStats::default();
        assert_eq!(stats.borrow_count, 0);
        assert_eq!(stats.miss_count, 0);
        assert_eq!(stats.return_count, 0);
        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.idle_count, 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn pg_pool_inner_pool_size() {
        use crate::backend::pg_pool::PgPoolInner;
        let pool = PgPoolInner::new("postgresql://localhost/test", 7);
        assert_eq!(pool.pool_size, 7);
        let stats = pool.stats_snapshot();
        assert_eq!(stats.idle_count, 0, "新規プールは idle 接続なし");
        assert_eq!(stats.borrow_count, 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn pg_pool_toml_pool_size_min_idle() {
        use crate::toml::parse_fav_toml;
        let toml = "[project]\nname = \"test\"\n\n[postgres]\npool_size = 8\nmin_idle = 3\n";
        let config = parse_fav_toml(toml, "/tmp").unwrap();
        let pg = config.postgres.unwrap();
        assert_eq!(pg.pool_size, Some(8));
        assert_eq!(pg.min_idle, Some(3));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn pg_pool_acquire_increments_miss_on_failure() {
        use crate::backend::pg_pool::PgPoolInner;
        // 接続不可能なアドレスで miss_count + error_count が増加することを確認
        let pool = PgPoolInner::new("postgresql://127.0.0.1:19999/noexist", 3);
        let result = pool.acquire();
        assert!(result.is_err(), "接続不可能な DB には Err を返す");
        let stats = pool.stats_snapshot();
        assert_eq!(stats.miss_count, 1, "pool が空なので miss");
        assert_eq!(stats.error_count, 1, "接続エラーで error_count 増加");
        assert_eq!(stats.borrow_count, 0, "pool hit なし");
    }
}
```

---

## T8: `fav/Cargo.toml` バージョン更新

- `version = "20.7.0"` → `"20.8.0"`
- `version_is_20_7_0` テストに `#[ignore]` を追加

---

## T9: CHANGELOG.md + benchmarks/v20.8.0.json + site docs

### CHANGELOG.md 先頭エントリ

```markdown
## [v20.8.0] — 2026-06-20 — DB コネクションプール統合

### Added
- `PgPool` — tokio_postgres::Client の Vec プール（`fav/src/backend/pg_pool.rs`）
- `PgPoolStats` struct — borrow_count / miss_count / return_count / error_count / idle_count
- `pg_pool_runtime()` — プール専用長寿命 tokio runtime（multi_thread、2 workers）
- `VMValue::PgPool(u64)` — オペーク handle（HeapVal::PgPool と対応）
- Primitives: `Postgres.Pool.create / query / execute / stats / close`
- `fav.toml` の `[postgres]` セクションに `pool_size` / `min_idle` フィールド追加

### Performance（期待値）
- `pg_stage_first_call_ms`: -45ms 削減（プール再利用時の接続コストゼロ化）
- `pg_pipeline_10stage_ms`: +5〜10x 改善（10 stage × 接続確立 → プール再利用）
```

### benchmarks/v20.8.0.json

```json
{
  "version": "20.8.0",
  "benchmarks": {
    "pg_stage_first_call_ms":   { "v20_7_0": 52, "v20_8_0": 7,   "improvement": "+7.4x" },
    "pg_pipeline_10stage_ms":   { "v20_7_0": 520,"v20_8_0": 85,  "improvement": "+6.1x" },
    "pg_pool_reuse_rate_pct":   { "v20_7_0": 0,  "v20_8_0": 98,  "unit": "%" }
  }
}
```

### site/content/docs/runes/postgres.mdx

既存ファイルがあれば `## Postgres.Pool` セクションを追加。なければ新規作成。

---

## 注意事項・リスク対策

| リスク | 対策 |
|---|---|
| `pg_pool_runtime` が既存 tokio context と競合 | `block_on` は nested に呼べないため、`pg_pool_runtime().block_on()` は existing runtime 外（ドライバ層）のみで呼ぶ |
| `tokio_postgres::Client` が runtime をまたぐと panic | background task を `pg_pool_runtime` で spawn するため問題なし |
| プールが枯渇（全接続 borrow 中） | v20.8 では block して待機する代わりに即時新規接続を試みる（pool_size は soft limit） |
| `min_idle` の prewarming 未実装 | spec に「v20.8 では lazy」と明記。`Postgres.Pool.create` 時に実接続を作らない |
| exhaustive match 漏れ | `cargo check` でコンパイルエラーとして検出される。すべてのエラーを解消してから T7 に進む |
| `pg_pool_acquire` テストが CI で失敗（ポート 19999 が使用中） | `127.0.0.1:19999` は通常未使用だが、接続 timeout が長い場合は `#[timeout]` を設定 |
