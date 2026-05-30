# Favnir v4.3.0 タスクリスト — DuckDB Rune

作成日: 2026-05-16

---

## Phase 0: バージョン更新 + duckdb クレート追加

- [x] `fav/Cargo.toml` の version を `"4.3.0"` に変更
- [x] `fav/Cargo.toml` に `duckdb = { version = "0.10", features = ["bundled"] }` を追加
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.3.0` に更新

---

## Phase 1: VM プリミティブ追加（`fav/src/backend/vm.rs`）

### 1-A: DuckDB 接続マップの追加

- [x] `use duckdb;` インポートを追加
- [x] `static DUCKDB_CONNS: Lazy<Mutex<HashMap<String, duckdb::Connection>>>` をグローバルに定義
- [x] 接続 ID 生成用ヘルパー（`"duckdb:N"` 形式）を追加

### 1-B: `DuckDb.open_raw` の実装

- [x] `vm_call_builtin` に `"DuckDb.open_raw"` アームを追加
- [x] `duckdb::Connection::open(path)` で接続し `DUCKDB_CONNS` に格納
- [x] 成功時は `ok_vm(db_handle_vm("duckdb:N"))` を返す
- [x] 失敗時は `err_vm(db_error_vm("OPEN_ERROR", msg))` を返す

### 1-C: `DuckDb.query_raw` の実装

- [x] `vm_call_builtin` に `"DuckDb.query_raw"` アームを追加
- [x] ハンドル ID から `DUCKDB_CONNS` を検索
- [x] `conn.prepare(sql)` → `stmt.query_map` でフェッチ
- [x] 各行のカラム名・値を `VMValue::Record` の `Map` 形式に変換
- [x] `ok_vm(VMValue::List(rows))` を返す
- [x] エラー時は `err_vm(db_error_vm("QUERY_ERROR", msg))` を返す

### 1-D: `DuckDb.execute_raw` の実装

- [x] `vm_call_builtin` に `"DuckDb.execute_raw"` アームを追加
- [x] `conn.execute(sql, duckdb::params![])` で実行
- [x] `ok_vm(VMValue::Int(affected as i64))` を返す
- [x] エラー時は `err_vm(db_error_vm("EXECUTE_ERROR", msg))` を返す

### 1-E: `DuckDb.close_raw` の実装

- [x] `vm_call_builtin` に `"DuckDb.close_raw"` アームを追加
- [x] `DUCKDB_CONNS.lock().remove(&handle_id)` で接続を破棄
- [x] `VMValue::Unit` を返す

---

## Phase 2: checker.rs へのシグネチャ登録（`fav/src/middle/checker.rs`）

- [x] `check_builtin_apply` に `("DuckDb", "open_raw")` アームを追加（戻り値: `Result<DbHandle, DbError>`）
- [x] `check_builtin_apply` に `("DuckDb", "query_raw")` アームを追加（戻り値: `Result<List<Map<String, String>>, DbError>`）
- [x] `check_builtin_apply` に `("DuckDb", "execute_raw")` アームを追加（戻り値: `Result<Int, DbError>`）
- [x] `check_builtin_apply` に `("DuckDb", "close_raw")` アームを追加（戻り値: `Unit`）
- [x] `check_builtin_apply` に `("DuckDb", _)` フォールバックアームを追加（`require_db_effect` + `Unknown`）
- [x] 全 `DuckDb.*` アームで `self.require_db_effect(span)` を呼ぶ

---

## Phase 3: Favnir rune ファイル作成

### 3-A: `runes/duckdb/query.fav`（新規作成）

- [x] `open(path: String) -> Result<DbHandle, DbError> !Db` を実装
- [x] `close(conn: DbHandle) -> Unit !Db` を実装
- [x] `query(conn: DbHandle, sql: String) -> Result<List<Map<String, String>>, DbError> !Db` を実装
- [x] `query_one(conn: DbHandle, sql: String) -> Result<Map<String, String>, DbError> !Db` を実装
  - `match DuckDb.query_raw(...) { Ok(rows) => match List.first(rows) { Some(row) => Result.ok(row) None => Result.err(DbError {...}) } Err(e) => Result.err(e) }` パターン
- [x] `execute(conn: DbHandle, sql: String) -> Result<Int, DbError> !Db` を実装
- [x] `explain(conn: DbHandle, sql: String) -> Result<List<Map<String, String>>, DbError> !Db` を実装
  - `String.concat("EXPLAIN ", sql)` を `DuckDb.query_raw` に渡す

### 3-B: `runes/duckdb/io.fav`（新規作成）

- [x] `read_parquet(conn: DbHandle, path: String) -> Result<List<Map<String, String>>, DbError> !Db` を実装
  - `"SELECT * FROM read_parquet('" + path + "')"` を組み立てて `DuckDb.query_raw`
- [x] `read_csv(conn: DbHandle, path: String) -> Result<List<Map<String, String>>, DbError> !Db` を実装
  - `"SELECT * FROM read_csv_auto('" + path + "')"` を組み立てて `DuckDb.query_raw`
- [x] `write_parquet(conn: DbHandle, sql: String, path: String) -> Result<Int, DbError> !Db` を実装
  - `"COPY (" + sql + ") TO '" + path + "' (FORMAT PARQUET)"` を `DuckDb.execute_raw`
- [x] `write_csv(conn: DbHandle, sql: String, path: String) -> Result<Int, DbError> !Db` を実装
  - `"COPY (" + sql + ") TO '" + path + "' (FORMAT CSV, HEADER TRUE)"` を `DuckDb.execute_raw`

### 3-C: `runes/duckdb/duckdb.fav`（新規作成、barrel）

- [x] `use query.{ open, close, query, query_one, execute, explain }` を記述
- [x] `use io.{ read_parquet, read_csv, write_parquet, write_csv }` を記述

### 3-D: `runes/duckdb/duckdb.test.fav`（新規作成）

- [x] `test_open_memory` — `duckdb.open(":memory:")` が Ok を返す
- [x] `test_execute_create_table` — CREATE TABLE が成功する
- [x] `test_query_returns_rows` — INSERT 後に SELECT で行が返る
- [x] `test_query_one_found` — 1 行の場合 Ok(row) を返す
- [x] `test_query_one_not_found` — 0 行の場合 Err を返す
- [x] `test_explain_returns_result` — EXPLAIN が空でない結果を返す
- [x] `test_write_parquet_and_read_back` — write_parquet → read_parquet の往復が一致する
- [x] `test_read_csv_auto` — CSV 文字列をファイルに書いて read_csv が行を返す
- [x] `test_write_csv` — SELECT 結果を CSV に書き出して行数が正しい

---

## Phase 4: テスト追加

### 4-A: `fav/src/backend/vm_stdlib_tests.rs` 追加

- [x] `duckdb_open_memory_succeeds` — `:memory:` で開き Bool(true) を返すテスト
- [x] `duckdb_execute_create_table_succeeds` — DDL が影響行数 0 で Ok になる
- [x] `duckdb_query_returns_inserted_row` — INSERT + SELECT で 1 件が返る
- [x] `duckdb_query_bad_sql_returns_err` — 不正 SQL で Result.is_err(r) が true

### 4-B: `fav/src/driver.rs` 統合テスト追加

- [x] `duckdb_rune_test_file_passes` — `run_fav_test_file_with_runes("runes/duckdb/duckdb.test.fav")` が全 pass
- [x] `duckdb_open_in_favnir_source` — `exec_project_main_source_with_runes` で `duckdb.open(":memory:")` が Ok
- [x] `duckdb_parquet_roundtrip_in_source` — `write_parquet` → `read_parquet` の往復が動く

---

## Phase 5: examples 追加

- [x] `examples/duckdb_demo/fav.toml` を作成
- [x] `examples/duckdb_demo/src/main.fav` を作成
  - `:memory:` オープン
  - CSV ライクなデータを INSERT して GROUP BY 集計
  - `write_parquet` → `read_parquet` の往復
  - `explain` でクエリプランを表示して行数を確認
  - `close` で接続を閉じる

---

## 完了条件

- [x] `cargo build` が通る（duckdb クレート込み）
- [x] 既存 819 件が全て pass
- [x] 新規テスト 10 件以上が pass
- [x] `duckdb.open(":memory:") + duckdb.query(...)` が動く
- [x] `duckdb.write_parquet` → `duckdb.read_parquet` の往復が動く
- [x] `duckdb.read_csv` + `duckdb.write_parquet` で CSV → Parquet 変換が動く
- [x] `examples/duckdb_demo/` が `fav run` で動く
