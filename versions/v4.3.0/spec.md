# Favnir v4.3.0 仕様書 — DuckDB Rune（組み込み OLAP + Parquet/CSV 統合）

作成日: 2026-05-16

## 概要

データエンジニアの日常作業（Parquet 分析・集計・変換・CSV → Parquet ETL）を
SQL で直接書けるようにする組み込み型 OLAP エンジン rune を実装する。

DuckDB はサーバー不要の組み込み型データベースで、Parquet / CSV / JSON ファイルを
テーブルとして直接 SQL でクエリできる。db rune（SQLite / PostgreSQL）が OLTP を担うのに対し、
duckdb rune は OLAP（分析・集計・変換）を担当する。

---

## 1. 設計方針

### 1.1 既存型との関係

| 概念 | 型 | 備考 |
|------|----|------|
| 接続ハンドル | `DbHandle` | db rune と共用（新型は作らない） |
| エラー | `DbError` | db rune と共用 |
| エフェクト | `!Db` | db rune と共用 |

DuckDB 接続と SQLite 接続は Favnir 型システム上では同じ `DbHandle` として扱う。
VM 内部では接続 ID のプレフィクス（`"duckdb:"`）で区別する。

### 1.2 VM プリミティブ（最小セット）

```
DuckDb.open_raw(path)       -> Result<DbHandle, DbError>
DuckDb.query_raw(conn, sql) -> Result<List<Map<String, String>>, DbError>
DuckDb.execute_raw(conn, sql) -> Result<Int, DbError>
DuckDb.close_raw(conn)      -> Unit
```

この 4 つだけを Rust で実装する。それ以上のロジック（Parquet 読み込み・ページネーション等）は
すべて Favnir の rune コードとして実装する。

### 1.3 db rune との使い分け

| 用途 | 使う rune |
|------|----------|
| INSERT / UPDATE / DELETE（OLTP） | db rune |
| Parquet / CSV を SQL でクエリ | **duckdb rune** |
| GROUP BY / Window 関数 / 重い集計 | **duckdb rune** |
| トランザクション管理 | db rune |
| 大量データの ETL（CSV → Parquet） | **duckdb rune** |

---

## 2. Cargo 依存追加

```toml
[dependencies]
duckdb = { version = "0.10", features = ["bundled"] }
```

`bundled` feature により DuckDB のネイティブバイナリが Rust ビルドに同梱される。
外部インストール不要で CI でもそのまま動く。

---

## 3. rune ファイル構成

```
runes/duckdb/
  duckdb.fav      ← public API（barrel file）
  query.fav       ← open / close / query / query_one / execute / explain
  io.fav          ← read_parquet / read_csv / write_parquet / write_csv
  duckdb.test.fav ← テスト
```

S3 統合（`s3_scan` / `s3_query`）は AWS SDK（v4.11.0）と合わせて実装するため v4.3.0 では対象外。

---

## 4. API 仕様

### 4.1 `query.fav`

```favnir
// open: DuckDB データベースを開く（":memory:" でインメモリ）
public fn open(path: String) -> Result<DbHandle, DbError> !Db {
    DuckDb.open_raw(path)
}

// close: 接続を閉じる
public fn close(conn: DbHandle) -> Unit !Db {
    DuckDb.close_raw(conn)
}

// query: SQL を実行して全行を返す
public fn query(conn: DbHandle, sql: String) -> Result<List<Map<String, String>>, DbError> !Db {
    DuckDb.query_raw(conn, sql)
}

// query_one: 最初の 1 行だけを返す（0 行は DbError）
public fn query_one(conn: DbHandle, sql: String) -> Result<Map<String, String>, DbError> !Db {
    match DuckDb.query_raw(conn, sql) {
        Ok(rows) => match List.first(rows) {
            Some(row) => Result.ok(row)
            None      => Result.err(DbError { code: "NOT_FOUND" message: "query returned no rows" })
        }
        Err(e) => Result.err(e)
    }
}

// execute: DDL / DML を実行して影響行数を返す
public fn execute(conn: DbHandle, sql: String) -> Result<Int, DbError> !Db {
    DuckDb.execute_raw(conn, sql)
}

// explain: クエリ実行計画を返す
public fn explain(conn: DbHandle, sql: String) -> Result<List<Map<String, String>>, DbError> !Db {
    DuckDb.query_raw(conn, String.concat("EXPLAIN ", sql))
}
```

### 4.2 `io.fav`

```favnir
// read_parquet: Parquet ファイルを SQL で読む
public fn read_parquet(conn: DbHandle, path: String) -> Result<List<Map<String, String>>, DbError> !Db {
    DuckDb.query_raw(conn,
        String.concat("SELECT * FROM read_parquet('", String.concat(path, "')")))
}

// read_csv: CSV ファイルを自動スキーマ検出で読む
public fn read_csv(conn: DbHandle, path: String) -> Result<List<Map<String, String>>, DbError> !Db {
    DuckDb.query_raw(conn,
        String.concat("SELECT * FROM read_csv_auto('", String.concat(path, "')")))
}

// write_parquet: SQL の結果を Parquet に書き出す
public fn write_parquet(conn: DbHandle, sql: String, path: String) -> Result<Int, DbError> !Db {
    DuckDb.execute_raw(conn,
        String.concat(
            String.concat("COPY (", String.concat(sql, ") TO '")),
            String.concat(path, "' (FORMAT PARQUET)")))
}

// write_csv: SQL の結果を CSV に書き出す
public fn write_csv(conn: DbHandle, sql: String, path: String) -> Result<Int, DbError> !Db {
    DuckDb.execute_raw(conn,
        String.concat(
            String.concat("COPY (", String.concat(sql, ") TO '")),
            String.concat(path, "' (FORMAT CSV, HEADER TRUE)")))
}
```

### 4.3 `duckdb.fav`（barrel）

```favnir
use query.{ open, close, query, query_one, execute, explain }
use io.{ read_parquet, read_csv, write_parquet, write_csv }
```

---

## 5. VM プリミティブ実装詳細

### 5.1 `DuckDb.open_raw`

```rust
"DuckDb.open_raw" => {
    let path = /* args[0] として String を取り出す */;
    let conn = duckdb::Connection::open(&path)
        .map_err(|e| format!("DuckDB open error: {}", e))?;
    let handle_id = format!("duckdb:{}", next_handle_id());
    DUCKDB_CONNS.lock().insert(handle_id.clone(), conn);
    ok_vm(VMValue::Record { name: "DbHandle", fields: [("id", VMValue::Str(handle_id))] })
}
```

DuckDB 接続は既存 SQLite 接続（`DB_CONNS`）とは別のグローバルマップ `DUCKDB_CONNS` で管理する。
`DbHandle` の `id` フィールドが `"duckdb:"` で始まる場合に DuckDB として扱う。

### 5.2 `DuckDb.query_raw`

```rust
"DuckDb.query_raw" => {
    let handle_id = /* DbHandle.id を取り出す */;
    let sql = /* args[1] を String として取り出す */;
    let guard = DUCKDB_CONNS.lock();
    let conn = guard.get(&handle_id).ok_or("invalid DuckDb handle")?;
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| { /* フィールドを Map<String, String> に変換 */ })?;
    ok_vm(VMValue::List(rows.collect()))
}
```

### 5.3 `DuckDb.execute_raw`

```rust
"DuckDb.execute_raw" => {
    let conn = /* handle から取り出す */;
    let affected = conn.execute(&sql, [])?;
    ok_vm(VMValue::Int(affected as i64))
}
```

### 5.4 checker.rs 登録

```rust
("DuckDb", "open_raw") => {
    self.require_db_effect(span);
    Some(Type::Result(
        Box::new(Type::Named("DbHandle".into(), vec![])),
        Box::new(Type::Named("DbError".into(), vec![])),
    ))
}
("DuckDb", "query_raw") => {
    self.require_db_effect(span);
    Some(Type::Result(
        Box::new(Type::List(Box::new(Type::Map(
            Box::new(Type::String),
            Box::new(Type::String),
        )))),
        Box::new(Type::Named("DbError".into(), vec![])),
    ))
}
("DuckDb", "execute_raw") => {
    self.require_db_effect(span);
    Some(Type::Result(
        Box::new(Type::Int),
        Box::new(Type::Named("DbError".into(), vec![])),
    ))
}
("DuckDb", "close_raw") => {
    self.require_db_effect(span);
    Some(Type::Unit)
}
("DuckDb", _) => {
    self.require_db_effect(span);
    Some(Type::Unknown)
}
```

---

## 6. 使用例

### 6.1 Parquet ファイルの集計

```favnir
import "duckdb"

type OrderSummary = { customer: String total: String count: String }

public fn main() -> Unit !Io !Db {
    bind conn_result <- duckdb.open(":memory:")
    match conn_result {
        Ok(conn) => {
            bind result <- duckdb.query(conn,
                "SELECT customer, SUM(amount) AS total, COUNT(*) AS count
                 FROM read_parquet('data/orders.parquet')
                 GROUP BY customer ORDER BY total DESC LIMIT 10")
            match result {
                Ok(rows) => IO.println($"Rows: {List.length(rows)}")
                Err(e)   => IO.println($"Error: {e.message}")
            }
            duckdb.close(conn)
        }
        Err(e) => IO.println($"Open error: {e.message}")
    }
}
```

### 6.2 CSV → Parquet 変換（ETL）

```favnir
import "duckdb"

public fn convert(input: String, output: String) -> Result<Int, DbError> !Db {
    bind conn_result <- duckdb.open(":memory:")
    match conn_result {
        Ok(conn) => {
            bind result <- duckdb.write_parquet(conn,
                String.concat("SELECT * FROM read_csv_auto('", String.concat(input, "')")),
                output)
            duckdb.close(conn)
            result
        }
        Err(e) => Result.err(e)
    }
}
```

### 6.3 インメモリ分析

```favnir
import "duckdb"

public fn top_products() -> Result<List<Map<String, String>>, DbError> !Db {
    bind conn_result <- duckdb.open(":memory:")
    match conn_result {
        Ok(conn) => {
            bind _ <- duckdb.execute(conn,
                "CREATE TABLE sales AS SELECT * FROM read_csv_auto('sales.csv')")
            duckdb.query(conn,
                "SELECT product, SUM(qty) AS total FROM sales GROUP BY product ORDER BY total DESC")
        }
        Err(e) => Result.err(e)
    }
}
```

---

## 7. テスト方針

### 7.1 vm_stdlib_tests.rs（Rust レベル）

- `duckdb_open_memory_succeeds` — `:memory:` で開けること
- `duckdb_execute_create_table` — CREATE TABLE が実行できること
- `duckdb_query_returns_rows` — SELECT が行を返すこと
- `duckdb_query_raw_on_bad_sql_returns_err` — 不正 SQL で Err になること

### 7.2 `runes/duckdb/duckdb.test.fav`（Favnir レベル）

- `test_open_memory` — `duckdb.open(":memory:")` が Ok
- `test_execute_create_and_insert` — CREATE TABLE + INSERT
- `test_query_returns_rows` — SELECT で行が返る
- `test_query_one_found` — 1 行を返す
- `test_query_one_not_found` — 0 行で Err
- `test_explain_returns_plan` — EXPLAIN が結果を返す
- `test_write_and_read_parquet` — Parquet 書き込み・読み込みの往復
- `test_write_and_read_csv` — CSV 書き込み・読み込みの往復
- `test_read_csv_auto_schema` — read_csv_auto の列名推定

### 7.3 driver.rs 統合テスト

- `duckdb_rune_test_file_passes` — duckdb.test.fav が全パス
- `duckdb_open_in_favnir_source` — inline Favnir source でオープン確認
- `duckdb_query_in_favnir_source` — inline Favnir source でクエリ確認

---

## 8. 既存 Parquet rune との関係

| 機能 | Parquet rune | DuckDB rune |
|------|-------------|-------------|
| 列型を保持して read/write | ✓（Arrow スキーマ）| ✗（Map<String,String>） |
| SQL でクエリ・集計 | ✗ | ✓ |
| CSV → Parquet 変換 | ✗ | ✓ |
| 大量行のストリーム処理 | ✗（TODO: Gen 2.0） | ✓（DuckDB の最適化に委ねる） |

v4.3.0 では既存 Parquet rune は変更しない。DuckDB rune は SQL 経由の
`Map<String, String>` インターフェースに特化する。

---

## 9. 完了条件

- `duckdb.open(":memory:") + duckdb.query(...)` がローカル Parquet に対して動く
- `duckdb.write_parquet` / `duckdb.read_parquet` の往復が動く
- `duckdb.read_csv` + `duckdb.write_parquet` で CSV → Parquet 変換が動く
- 統合テスト 10 件以上（サーバー不要、CI で完結）
- 既存 819 件のテストがすべて通る（破壊的変更なし）
- `examples/duckdb_demo/` が動く
