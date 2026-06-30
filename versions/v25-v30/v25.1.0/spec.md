# v25.1.0 仕様書 — postgres Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v25.1.0 |
| フェーズ | Rune Foundation（v25.1〜v26.0） |
| テーマ | postgres Rune の「動く Rune」5 条件達成 |
| 依存関係 | v25.0.0（Practical Self-Hosting 宣言） |
| 目標テスト数 | 1980 件（+6 件 ≥ ロードマップ最小 5 件） |

---

## 背景と目的

v25.0.0 時点の postgres Rune は `execute(sql, params)` / `query[T](sql, params)` の 2 関数のみを実装しており、接続オブジェクト・トランザクション・コネクションプールを持たない簡易実装である。

v25.1.0 では postgres Rune が「動く Rune」の 5 条件を満たすよう実質化する。

### 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `Postgres.connect(config)` — SSL / タイムアウト対応、接続オブジェクト返却 |
| 2 | read | `Postgres.query[T](conn, sql, params)` — 接続オブジェクト経由 |
| 3 | write | `Postgres.execute(conn, sql, params)` — 接続オブジェクト経由 |
| 4 | error | `Result[T, PgError]` 統一、E0315 エラーコード対応 |
| 5 | test | `Postgres.Pool.create(config)` + `examples/postgres_etl.fav` E2E デモ |

---

## 機能仕様

### 1. `DbConn` interface 定義

`runes/postgres/db_conn.fav` を新規作成し、接続オブジェクトを表す interface を定義する。

```
interface DbConn {
  fn query[T](sql: String, params: List[String]) -> Result[List[T], PgError]
  fn execute(sql: String, params: List[String]) -> Result[Int, PgError]
  fn execute_many(sql: String, rows: List[List[String]]) -> Result[Int, PgError]
  fn transaction[T](fn: (DbConn) -> Result[T, PgError]) -> Result[T, PgError]
}
```

### 2. `Postgres.connect(config)` 追加

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

fn connect(config: PgConfig) -> Result[PgConn, PgError]
```

- `PgConn` は `DbConn` を実装する具体型
- `ssl: true` で `sslmode=require`（基本 SSL 接続）を有効化
- `connect_timeout_ms: 0` で無制限（デフォルト）
- 環境変数ベースの既存 `execute` / `query[T]` は後方互換として残す

### 3. `Postgres.execute_many(conn, sql, rows)` 追加

```
fn execute_many(conn: PgConn, sql: String, rows: List[List[String]]) -> Result[Int, PgError]
```

- バッチ INSERT / UPDATE 用
- 成功時: 影響行数の合計を返す

### 4. `Postgres.transaction(conn, fn)` 追加

```
fn transaction[T](conn: PgConn, fn: (PgConn) -> Result[T, PgError]) -> Result[T, PgError]
```

- BEGIN / COMMIT / ROLLBACK を自動管理
- `fn` が `Err` を返した場合は ROLLBACK

### 5. `Postgres.Pool.create(config)` / `Pool.get` / `Pool.release` 追加

```
type PoolConfig = {
  host: String,
  port: Int,
  user: String,
  password: String,
  database: String,
  max_size: Int,
}

fn Pool.create(config: PoolConfig) -> Result[PgPool, PgError]
fn Pool.get(pool: PgPool) -> Result[PgConn, PgError]
fn Pool.release(pool: PgPool, conn: PgConn) -> Unit
```

### 6. `examples/postgres_etl.fav` E2E デモ

```
use runes/postgres.{ connect, execute_many, transaction, Pool }

type User = { name: String, age: Int }

fn main() -> Result[Unit, PgError] !Postgres {
  let config = PgConfig {
    host: "localhost",
    port: 5432,
    user: "postgres",
    password: "postgres",
    database: "etl_demo",
    ssl: false,
    connect_timeout_ms: 5000,
  }
  let conn <- connect(config)?
  let rows = [
    ["alice", "30"],
    ["bob", "25"],
  ]
  execute_many(conn, "INSERT INTO users(name, age) VALUES ($1, $2)", rows)?
  let users <- query[User](conn, "SELECT * FROM users", [])?
  Ok(())
}
```

---

## エラーコード

| コード | 名前 | 説明 |
|---|---|---|
| E0315 | EffectMismatch | `!Postgres` エフェクトなしで Postgres Rune を呼び出した場合 |

（既存 E0314 は `!Snowflake` 用として登録済み。E0315 が `!Postgres` 用）

---

## やらないこと（スコープ外）

- 非同期（async/await）接続モデル
- カスタム TLS 証明書設定（自己署名証明書・クライアント証明書の指定）
- `COPY FROM` / `COPY TO` バルク操作
- PostgreSQL 固有の型（JSONB / ARRAY / ENUM 等）の直接マッピング
- mysql Rune への同等機能追加（v25.2.0 以降）

> 基本 SSL 接続（`sslmode=require`）は `PgConfig.ssl: Bool` でサポートする。
> カスタム証明書・証明書検証（`sslcert` / `sslkey` / `sslrootcert`）は本バージョン外。

---

## 完了条件

| # | 条件 |
|---|---|
| 1 | `Postgres.connect` / `execute_many` / `transaction` / `Pool.create` / `Pool.get` / `Pool.release` が `runes/postgres/client.fav` に実装済み |
| 2 | `runes/postgres/db_conn.fav` に `DbConn` interface が定義済み |
| 3 | `examples/postgres_etl.fav` が存在し `User` 型定義を含む |
| 4 | `CHANGELOG.md` に `[v25.1.0]` エントリが存在する |
| 5 | `cargo test` で v251000_tests 6 件すべて PASS（ロードマップ最小 5 件を超過） |
| 6 | 総テスト数 ≥ 1980 件 |
| 7 | `site/content/docs/runes/postgres.mdx` に新規 API（connect / execute_many / transaction / Pool）が追記済み |

---

## 検証コマンド

```bash
cd fav && cargo test v251000 -- --test-threads=1
```
