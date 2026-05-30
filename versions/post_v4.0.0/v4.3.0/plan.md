# Favnir v4.3.0 実装計画 — DuckDB Rune

作成日: 2026-05-16

---

## Phase 0: バージョン更新 + duckdb クレート追加

- `fav/Cargo.toml` の version を `"4.3.0"` に変更
- `fav/Cargo.toml` に `duckdb = { version = "0.10", features = ["bundled"] }` を追加
- `fav/src/main.rs` のヘルプ文字列・バージョン表示を更新

> `bundled` feature が DuckDB ネイティブライブラリをビルドに同梱するため、
> 初回ビルドは時間がかかる（5〜10 分）。以降はキャッシュが効く。

---

## Phase 1: VM プリミティブ追加（Rust）

**変更ファイル**: `fav/src/backend/vm.rs`

### 1-A: DuckDB 接続管理

グローバル接続マップを追加:

```rust
static DUCKDB_CONNS: Lazy<Mutex<HashMap<String, duckdb::Connection>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
```

接続 ID は `format!("duckdb:{}", next_handle_id())` で生成し、
既存 SQLite ハンドルと名前空間を区別する。

### 1-B: `DuckDb.open_raw(path)`

- `duckdb::Connection::open(path)` で接続
- `DUCKDB_CONNS` に格納
- `DbHandle { id: "duckdb:N" }` を `VMValue::Record` として返す
- エラー時は `err_vm(db_error_vm("OPEN_ERROR", msg))`

### 1-C: `DuckDb.query_raw(conn, sql)`

- ハンドル ID から `DUCKDB_CONNS` を検索
- `conn.prepare(sql)` → `stmt.query_map` でフェッチ
- 各行を `HashMap<String, VMValue>` → `VMValue::Record("Map", ...)` に変換
  - カラム名を key、値を `VMValue::Str(...)` として格納
- `VMValue::List(rows)` を `ok_vm(...)` でラップして返す
- エラー時は `err_vm(db_error_vm("QUERY_ERROR", msg))`

### 1-D: `DuckDb.execute_raw(conn, sql)`

- `conn.execute(sql, [])` で実行
- 影響行数を `VMValue::Int(n)` → `ok_vm(...)` で返す
- エラー時は `err_vm(db_error_vm("EXECUTE_ERROR", msg))`

### 1-E: `DuckDb.close_raw(conn)`

- `DUCKDB_CONNS` から接続を取り出して drop
- `VMValue::Unit` を返す

---

## Phase 2: checker.rs へのシグネチャ登録

**変更ファイル**: `fav/src/middle/checker.rs`

`check_builtin_apply` の `("DuckDb", method)` アームを追加（既存 `("DB", _)` の直後あたり）:

| メソッド | 戻り値型 |
|---------|---------|
| `open_raw` | `Result<DbHandle, DbError>` |
| `query_raw` | `Result<List<Map<String, String>>, DbError>` |
| `execute_raw` | `Result<Int, DbError>` |
| `close_raw` | `Unit` |
| `_`（その他） | `Unknown` |

全メソッドに `self.require_db_effect(span)` を付与する（`!Db` エフェクト必須）。

---

## Phase 3: Favnir rune ファイル作成

**変更ファイル**: `runes/duckdb/`

### 3-A: `runes/duckdb/query.fav`（新規）

関数一覧:

| 関数 | シグネチャ |
|------|----------|
| `open(path)` | `String -> Result<DbHandle, DbError> !Db` |
| `close(conn)` | `DbHandle -> Unit !Db` |
| `query(conn, sql)` | `DbHandle -> String -> Result<List<Map<String, String>>, DbError> !Db` |
| `query_one(conn, sql)` | `DbHandle -> String -> Result<Map<String, String>, DbError> !Db` |
| `execute(conn, sql)` | `DbHandle -> String -> Result<Int, DbError> !Db` |
| `explain(conn, sql)` | `DbHandle -> String -> Result<List<Map<String, String>>, DbError> !Db` |

`query_one` は `match DuckDb.query_raw(...) { Ok(rows) => match List.first(rows) { Some(row) => Result.ok(row) None => Result.err(...) } Err(e) => Result.err(e) }` で実装。
`explain` は `String.concat("EXPLAIN ", sql)` を `DuckDb.query_raw` に渡す。

### 3-B: `runes/duckdb/io.fav`（新規）

関数一覧:

| 関数 | 概要 |
|------|------|
| `read_parquet(conn, path)` | `FROM read_parquet('path')` |
| `read_csv(conn, path)` | `FROM read_csv_auto('path')` |
| `write_parquet(conn, sql, path)` | `COPY (sql) TO 'path' (FORMAT PARQUET)` |
| `write_csv(conn, sql, path)` | `COPY (sql) TO 'path' (FORMAT CSV, HEADER TRUE)` |

全て SQL 文字列を組み立てて `DuckDb.query_raw` / `DuckDb.execute_raw` に委譲するだけ。
Favnir の制約（1 `let` / ブロック）に注意し、`String.concat` 連鎖で組み立てる。

### 3-C: `runes/duckdb/duckdb.fav`（新規、barrel）

```favnir
// runes/duckdb/duckdb.fav — DuckDB Rune public API (v4.3.0)
use query.{ open, close, query, query_one, execute, explain }
use io.{ read_parquet, read_csv, write_parquet, write_csv }
```

### 3-D: `runes/duckdb/duckdb.test.fav`（新規）

テスト 9 件（詳細は tasks.md 参照）。
インメモリ DuckDB `:memory:` を使用するため CI で外部依存なし。

---

## Phase 4: テスト追加

### 4-A: `vm_stdlib_tests.rs` 追加（4 件）

```
duckdb_open_memory_succeeds
duckdb_execute_create_table_succeeds
duckdb_query_returns_inserted_row
duckdb_query_bad_sql_returns_err
```

各テストは `eval(source)` を使ったインライン Favnir ソース方式。

### 4-B: `driver.rs` 統合テスト追加（3 件）

```
duckdb_rune_test_file_passes   — run_fav_test_file_with_runes("runes/duckdb/duckdb.test.fav")
duckdb_open_in_favnir_source   — exec_project_main_source_with_runes でオープン確認
duckdb_parquet_roundtrip_in_source — write_parquet → read_parquet の往復
```

`duckdb_rune_test_file_passes` が全テストを網羅するため、他 2 件は補完的な位置づけ。

---

## Phase 5: examples 追加

### 5-A: `examples/duckdb_demo/` 新規作成

```
examples/duckdb_demo/
  fav.toml
  src/
    main.fav
```

デモ内容:
1. `:memory:` で DuckDB を開く
2. CSV ライクなデータを INSERT して GROUP BY 集計
3. `write_parquet` → `read_parquet` の往復
4. `explain` でクエリプランを表示

---

## 実装順序と依存関係

```
Phase 0 (バージョン + クレート追加)
  ↓
Phase 1 (VM プリミティブ) → Phase 2 (checker.rs)
  ↓
Phase 3 (Favnir rune ファイル)
  ↓
Phase 4 (テスト)
  ↓
Phase 5 (examples)
```

Phase 1 と Phase 2 は並列実施可能。
Phase 3 は Phase 1 + 2 完了後（VM プリミティブがないとテストが動かない）。

---

## リスクと対策

| リスク | 影響 | 対策 |
|--------|------|------|
| `duckdb = "0.10"` のビルドが遅い | 初回ビルドに 10 分前後かかる | `bundled` は一度ビルドされればキャッシュされる。CI では事前ビルドキャッシュ設定 |
| DuckDB の Windows 対応 | MSVC ビルドが必要 | `duckdb` crate は Windows MSVC ビルド対応済み。`features = ["bundled"]` で cmake が自動実行される |
| `DUCKDB_CONNS` のスレッド安全性 | テスト並列実行時に競合 | `Mutex<HashMap<...>>` でロックする（`DB_CONNS` と同じパターン） |
| Parquet ファイルのテスト環境 | CI に Parquet ファイルが必要 | テスト内で DuckDB の `COPY ... TO '...' (FORMAT PARQUET)` で生成してから読み込む（外部ファイル不要） |
| 既存 Parquet rune との競合 | 両方が `read_parquet` 関数名を持つ | rune の名前空間（`parquet.read_parquet` vs `duckdb.read_parquet`）で区別。競合なし |

---

## 完了条件チェックリスト

- [ ] `cargo build` が通る（duckdb クレート込み）
- [ ] 既存 819 件が全て pass
- [ ] 新規テスト 10 件以上が pass
- [ ] `duckdb.open(":memory:") + duckdb.query(...)` が動く
- [ ] `duckdb.write_parquet` → `duckdb.read_parquet` の往復が動く
- [ ] `duckdb.read_csv` → `duckdb.write_parquet` で CSV→Parquet 変換が動く
- [ ] `examples/duckdb_demo/` が `fav run` で動く
