# Favnir v3.3.0 Tasks

## Phase 0: Version Bump

- [x] `fav/Cargo.toml`: `version = "3.3.0"`
- [x] `postgres = { version = "0.19", optional = true }` + `[features] postgres_integration`
- [x] `cargo build` 成功、`env!(\"CARGO_PKG_VERSION\")` 伝播
- [x] `fav --version` で `favnir 3.3.0` を確認

## Phase 1: 型定義 + エフェクト追加

### 1-A: `DbError` 型 + namespace 登録

- [x] `checker.rs`: `\"DbError\"` を stdlib 型として登録
  - フィールド: `code: String`, `message: String`
- [x] `checker.rs`: `\"DbHandle\"` / `\"TxHandle\"` を環境に登録（不透明型）
- [x] `checker.rs`: `\"DB\"` namespace を stdlib グローバル登録ループに追加
- [x] `checker.rs`: `\"Env\"` namespace を同様に追加
- [x] `compiler.rs`: `\"DB\"` / `\"Env\"` を2箇所のグローバル登録ループに追加

### 1-B: `DbHandle` / `TxHandle` 不透明型

- [x] `backend/vm.rs`: `VMValue::DbHandle(u64)` 追加
- [x] `backend/vm.rs`: `VMValue::TxHandle(u64)` 追加
- [x] `backend/vm.rs`: `DbConnWrapper { conn: rusqlite::Connection, in_tx: bool }` 定義
- [x] `backend/vm.rs`: thread-local `DB_CONNECTIONS: RefCell<HashMap<u64, DbConnWrapper>>` 追加
- [x] `backend/vm.rs`: thread-local `DB_NEXT_ID: Cell<u64>` 追加
- [x] `backend/vm.rs`: `vmvalue_type_name` に `\"DbHandle\"` / `\"TxHandle\"` 追加
- [x] `backend/vm.rs`: `vmvalue_repr` に DbHandle/TxHandle 追加
- [x] `backend/vm.rs`: `PartialEq` に DbHandle/TxHandle 追加
- [x] `backend/vm.rs`: `From<VMValue> for Value` に DbHandle/TxHandle 追加

### 1-C: `effect Db` (既存確認)

- [x] `middle/checker.rs`: `\"Db\"` は既存の `BUILTIN_EFFECTS` に含まれていた
- [x] `middle/checker.rs`: `DB.*` 呼び出し時にエフェクト `Db` を要求するチェックを追加

## Phase 2: SQLite VM プリミティブ

- [x] `backend/vm.rs`: `db_error_vm` ヘルパー追加
- [x] `backend/vm.rs`: `sqlite_query_raw` ヘルパー追加
- [x] `backend/vm.rs`: `sqlite_query_raw_params` ヘルパー追加
- [x] `backend/vm.rs`: `DB.connect(conn_str)` ビルトイン追加
  - `\"sqlite::memory:\"` → `rusqlite::Connection::open_in_memory()`
  - `\"sqlite:path\"` → `rusqlite::Connection::open(path)`
  - `\"postgres://...\"` → Phase 3: optional feature でゲート; なければ E0605
  - → `VMValue::Ok(VMValue::DbHandle(id))`
- [x] `backend/vm.rs`: `DB.close(handle)` ビルトイン追加
- [x] `backend/vm.rs`: `DB.query_raw(handle, sql)` ビルトイン追加
- [x] `backend/vm.rs`: `DB.execute_raw(handle, sql)` ビルトイン追加
- [x] `backend/vm.rs`: `DB.query_raw_params(handle, sql, params)` ビルトイン追加
- [x] `backend/vm.rs`: `DB.execute_raw_params(handle, sql, params)` ビルトイン追加
- [x] `backend/vm.rs`: `DB.begin_tx(handle)` ビルトイン追加（raw `BEGIN` SQL）
- [x] `backend/vm.rs`: `DB.commit_tx(tx)` ビルトイン追加（raw `COMMIT` SQL）
- [x] `backend/vm.rs`: `DB.rollback_tx(tx)` ビルトイン追加（raw `ROLLBACK` SQL）
- [x] `backend/vm.rs`: `DB.query_in_tx(tx, sql)` ビルトイン追加
- [x] `backend/vm.rs`: `DB.execute_in_tx(tx, sql)` ビルトイン追加
- [x] `checker.rs`: 上記11関数の型シグネチャを登録
- [x] Test: `db_sqlite_connect_and_close`
- [x] Test: `db_sqlite_create_and_insert`
- [x] Test: `db_sqlite_query_returns_rows`
- [x] Test: `db_sqlite_query_params_bind`
- [x] Test: `db_sqlite_execute_returns_affected_rows`
- [x] Test: `db_sqlite_transaction_commit`
- [x] Test: `db_sqlite_transaction_rollback`

## Phase 3: PostgreSQL VM プリミティブ

- [x] `fav/Cargo.toml`: `postgres = { version = "0.19", optional = true }` 追加
- [x] `fav/Cargo.toml`: `[features] postgres_integration = ["dep:postgres"]` 追加
- [x] `backend/vm.rs`: `DB.connect` に PostgreSQL 分岐（`postgres://` → E0605 stub）
  - 注: `postgres_integration` feature なしでは E0605 を返す
- [x] PostgreSQL 統合テストは `#[cfg(feature = "postgres_integration")]` でゲート（未実装）

## Phase 4: `Env.get` / `Env.get_or` VM プリミティブ

- [x] `backend/vm.rs`: `Env.get(name)` ビルトイン追加
  - `std::env::var(name)` → `Result<String, DbError相当>`
- [x] `backend/vm.rs`: `Env.get_or(name, default)` ビルトイン追加
  - `std::env::var(name).unwrap_or(default)` → `String`
- [x] `checker.rs`: `Env.get` / `Env.get_or` の型シグネチャを登録
- [x] Test: `env_get_or_returns_default_when_missing`
- [x] Test: `env_get_or_returns_value_when_set`

## Phase 5: rune ファイル作成

> 配置場所: `<repo_root>/runes/db/`

- [x] `runes/db/db.fav` 作成（6関数、全て `!Db` effect 付き）
  - `public fn connect(conn_str: String) -> Result<DbHandle, DbError> !Db`
  - `public fn query(handle, sql) -> Result<List<Map<String,String>>, DbError> !Db`
  - `public fn query_params(handle, sql, params) -> Result<List<Map<String,String>>, DbError> !Db`
  - `public fn execute(handle, sql) -> Result<Int, DbError> !Db`
  - `public fn execute_params(handle, sql, params) -> Result<Int, DbError> !Db`
  - `public fn close(handle) -> Unit !Db`
- [x] `runes/db/db.test.fav` 作成（8 テスト、SQLite インメモリ使用）
  - `test_connect_sqlite_memory`
  - `test_create_table_and_insert`
  - `test_query_returns_typed_rows`
  - `test_query_params_bind`
  - `test_execute_returns_affected_rows`
  - `test_transaction_commit`
  - `test_transaction_rollback_on_error`
  - `test_schema_mismatch_returns_err`

## Phase 6: checker 統合 + L008 リンタ

- [x] `error_catalog.rs`: E0601〜E0605 を追加
  - E0601: db connection failed
  - E0602: db query failed
  - E0603: db transaction failed
  - E0604: db schema mismatch
  - E0605: db driver unsupported
- [x] `middle/checker.rs`: `DB.*` 関数の戻り型 `Result<T, DbError>` 登録
- [x] `middle/checker.rs`: `DB.*` 呼び出し時のエフェクト `Db` チェック
- [x] `middle/checker.rs`: `Env.get` の戻り型 `Result<String, DbError>` 登録
- [x] `middle/checker.rs`: `Env.get_or` の戻り型 `String` 登録（エフェクトなし）
- [x] `lint.rs`: L008 `hardcoded db credential` 警告を追加
  - 検出: `DB.connect` / `db.connect` の文字列リテラル引数に `://` と `@` が両方含まれる場合
  - ヒント: `Env.get(\"DB_URL\")` を使うよう誘導
- [x] Test: `lint_l008_postgres_url_with_password`
- [x] Test: `lint_l008_sqlite_no_warning`

## Phase 7: サンプル + 統合テスト

### サンプル

- [x] `examples/db_demo/fav.toml` 作成
- [x] `examples/db_demo/src/main.fav` 作成（SQLite CRUD + トランザクション）

### driver.rs 統合テスト（`migrate_tests` モジュール内）

注: `exec_project_main_source_with_runes` / `run_fav_test_file_with_runes` を
`pub(super)` にし `migrate_tests` から参照

- [x] Test: `db_rune_connect_and_query`
- [x] Test: `db_rune_query_params_bind`
- [x] Test: `db_rune_transaction_commit`
- [x] Test: `db_rune_transaction_rollback`
- [x] Test: `db_rune_schema_mismatch_returns_err`
- [x] Test: `db_rune_test_file_passes` — `runes/db/db.test.fav` の全テスト実行
- [x] Test: `env_get_or_in_favnir_source`
- [x] 既存の全テストが通ること確認 (`cargo test`) — 706 passed

## Phase 8: ドキュメント

- [x] `versions/v3.3.0/langspec.md` 作成
- [x] `versions/v3.3.0/migration-guide.md` 作成（破壊的変更なし）
- [x] `versions/v3.3.0/progress.md` を全 Phase `[x]` に更新
