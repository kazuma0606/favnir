# v25.1.0 実装計画 — postgres Rune 実質化

## 実装ステップ

### Step 0: ブランチ作成 + Cargo.toml bump

```bash
git checkout -b feat/v25.1-postgres-rune
```

`fav/Cargo.toml` の `version = "25.0.0"` を `version = "25.1.0"` に更新する。

### Step 1: `DbConn` interface 定義

**ファイル**: `runes/postgres/db_conn.fav`（新規作成）

```
interface DbConn {
  fn query[T](sql: String, params: List[String]) -> Result[List[T], PgError]
  fn execute(sql: String, params: List[String]) -> Result[Int, PgError]
  fn execute_many(sql: String, rows: List[List[String]]) -> Result[Int, PgError]
  fn transaction[T](fn: (DbConn) -> Result[T, PgError]) -> Result[T, PgError]
}
```

> `fn transaction[T]` の型変数 `[T]` を忘れずに宣言すること（interface メソッドもジェネリック宣言が必要）。

### Step 2: `PgConfig` / `PgConn` 型定義追加

**ファイル**: `runes/postgres/types.fav`（新規作成）

```
type PgConfig = {
  host: String,
  port: Int,
  user: String,
  password: String,
  database: String,
  ssl: Bool,
  connect_timeout_ms: Int,
}

type PgError = {
  code: String,
  message: String,
}

type PgConn = {
  host: String,
  port: Int,
  database: String,
}

type PoolConfig = {
  host: String,
  port: Int,
  user: String,
  password: String,
  database: String,
  max_size: Int,
}

type PgPool = {
  max_size: Int,
}
```

### Step 3: `client.fav` に 6 関数追加

**ファイル**: `runes/postgres/client.fav`（既存ファイルを編集）

追加する関数:
- `fn connect(config: PgConfig) -> Result[PgConn, PgError]`
- `fn execute_many(conn: PgConn, sql: String, rows: List[List[String]]) -> Result[Int, PgError]`
- `fn transaction[T](conn: PgConn, fn: (PgConn) -> Result[T, PgError]) -> Result[T, PgError]`
- `fn Pool.create(config: PoolConfig) -> Result[PgPool, PgError]`
- `fn Pool.get(pool: PgPool) -> Result[PgConn, PgError]`
- `fn Pool.release(pool: PgPool, conn: PgConn) -> Unit`

各関数は以下の VM primitive を呼び出す:
- `connect` → `"Postgres.connect_raw"`（新規追加）
- `execute_many` → `"Postgres.execute_many_raw"`（新規追加）
- `transaction` → `"Postgres.transaction_raw"`（新規追加）
- `Pool.create` → 既存 `"Postgres.Pool.create"` primitive を直接呼び出す（新規 primitive 不要）
- `Pool.get` → `"Postgres.pool_get_raw"`（新規追加）
- `Pool.release` → `"Postgres.pool_release_raw"`（新規追加）

### Step 4: VM Primitive 追加

**ファイル**: `fav/src/backend/vm.rs`（既存ファイルを編集）

追加する primitive（`Postgres.Pool.create` は既存のため除く）:

| primitive 名 | 処理内容 |
|---|---|
| `"Postgres.connect_raw"` | PgConfig Record を受け取り PgConn Record を返す。`ssl` / `connect_timeout_ms` を考慮 |
| `"Postgres.execute_many_raw"` | PgConn + SQL + List[List[String]] を受け取りバッチ実行、影響行数を返す |
| `"Postgres.transaction_raw"` | PgConn + Closure を受け取り BEGIN → call_closure → COMMIT / ROLLBACK |
| `"Postgres.pool_get_raw"` | PgPool から PgConn を取得 |
| `"Postgres.pool_release_raw"` | PgPool に PgConn を返却 |

各 primitive は `!Postgres` エフェクトチェック（**E0315**）を通過した後のみ実行される。

### Step 5: `postgres.fav` エントリポイント更新

**ファイル**: `runes/postgres/postgres.fav`（既存ファイルを編集）

```
use types.{ PgConfig, PgConn, PgError, PoolConfig, PgPool }
use db_conn.{ DbConn }
use client.{ connect, execute, execute_many, query, transaction, Pool }
```

### Step 6: `examples/postgres_etl.fav` 作成

**ファイル**: `examples/postgres_etl.fav`（新規作成）

E2E デモ: connect → execute_many (INSERT) → query[User] (SELECT) → transaction (UPDATE)

`type User = { name: String, age: Int }` を先頭で定義すること（デモコード内の型参照を解決するため）。

### Step 7: `site/content/docs/runes/postgres.mdx` 更新

既存 MDX に以下の API セクションを追記:
- `Postgres.connect(config)` — `PgConfig` フィールド表（ssl / connect_timeout_ms を含む）
- `Postgres.execute_many(conn, sql, rows)`
- `Postgres.transaction(conn, fn)`
- `Postgres.Pool.create(config)` / `Pool.get(pool)` / `Pool.release(pool, conn)`

### Step 8: `CHANGELOG.md` 更新

`[v25.1.0]` エントリを追加:
```
## [v25.1.0] — 2026-06-24

### Added
- `Postgres.connect(config)` — 接続オブジェクト返却（SSL / タイムアウト対応）
- `Postgres.execute_many(conn, sql, rows)` — バッチ実行
- `Postgres.transaction(conn, fn)` — トランザクション管理（自動 ROLLBACK）
- `Postgres.Pool.create(config)` / `Pool.get` / `Pool.release` — コネクションプール
- `DbConn` interface 定義（`runes/postgres/db_conn.fav`）
- `examples/postgres_etl.fav` — E2E デモ
```

### Step 9: `benchmarks/v25.1.0.json` 作成

```json
{
  "version": "25.1.0",
  "timestamp": "2026-06-24T00:00:00Z",
  "metrics": {
    "test_count": 1980,
    "compile_hello_ms": 12,
    "compile_etl_ms": 45
  }
}
```

### Step 10: driver.rs に `v251000_tests` 追加

**ファイル**: `fav/src/driver.rs`（既存ファイルを編集）

```rust
#[cfg(test)]
mod v251000_tests {
    #[test]
    fn postgres_rune_has_connect_fn() {
        let src = include_str!("../../runes/postgres/client.fav");
        assert!(src.contains("fn connect"), "connect not found");
    }
    #[test]
    fn postgres_rune_has_execute_many_fn() {
        let src = include_str!("../../runes/postgres/client.fav");
        assert!(src.contains("fn execute_many"), "execute_many not found");
    }
    #[test]
    fn postgres_rune_has_transaction_fn() {
        let src = include_str!("../../runes/postgres/client.fav");
        assert!(src.contains("fn transaction"), "transaction not found");
    }
    #[test]
    fn postgres_rune_has_pool_create_fn() {
        let src = include_str!("../../runes/postgres/client.fav");
        assert!(src.contains("Pool.create"), "Pool.create not found");
    }
    #[test]
    fn postgres_etl_example_exists() {
        let src = include_str!("../../examples/postgres_etl.fav");
        assert!(src.contains("postgres"), "postgres not referenced");
    }
    #[test]
    fn changelog_has_v25_1_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("v25.1.0"), "v25.1.0 not in CHANGELOG");
    }
}
```

### Step 11: テスト実行・確認

```bash
cd fav && cargo test v251000 -- --test-threads=1
cd fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

---

## 実装順序まとめ

```
Step 0:  ブランチ作成 + Cargo.toml bump (25.0.0 → 25.1.0)
Step 1:  db_conn.fav（DbConn interface、transaction[T] の [T] に注意）
Step 2:  types.fav（PgConfig に ssl / connect_timeout_ms を含む）
Step 3:  client.fav（6 関数追加）
Step 4:  vm.rs（primitive 5 件追加、Pool.create は既存のため除外）
Step 5:  postgres.fav（re-export 更新）
Step 6:  examples/postgres_etl.fav（User 型定義を含む）
Step 7:  postgres.mdx（API ドキュメント追記）
Step 8:  CHANGELOG.md
Step 9:  benchmarks/v25.1.0.json
Step 10: driver.rs（v251000_tests 6 件）
Step 11: テスト実行
```

---

## リスクと対応

| リスク | 対応 |
|---|---|
| VM primitive の型整合性 | `PgConn` を `Val::Record` として表現（既存 Record 変換パターンを踏襲） |
| `Pool.create` の既存 primitive との重複 | 既存 `"Postgres.Pool.create"` をそのまま使用し、新規 primitive は追加しない |
| `transaction` クロージャの VM 実行 | `Val::Closure` を受け取り `vm.call_closure()` で実行（既存パターン） |
| `ssl: Bool` の Rust レベル実装 | `PgConfig.ssl = true` の場合 `Postgres.connect_raw` で `sslmode=require` を接続文字列に付与 |
