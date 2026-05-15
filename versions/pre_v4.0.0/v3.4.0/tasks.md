# Favnir v3.4.0 Tasks

## Phase 0: Version Bump

- [x] `fav/Cargo.toml`: `version = "3.4.0"`
- [x] `cargo build` 成功、`env!("CARGO_PKG_VERSION")` 伝播
- [x] `fav --version` で `favnir 3.4.0` を確認

## Phase 1: CLI プラミング

- [x] `fav/src/main.rs`: `"infer"` アームを `match args[1]` に追加
  - フラグパース: `--db <conn>`, `--out <path>`, `--name <TypeName>`, positional args
  - `cmd_infer(csv_path, db_conn, table_name, out_path, type_name)` を呼ぶ
- [x] `fav/src/driver.rs`: `pub fn cmd_infer(...)` を追加
- [x] `cargo build` が通ること確認

## Phase 2: 型推論コア

- [x] `fav/src/driver.rs`: `enum InferredType` 定義（`Int / Float / Bool / FavString / Option(Box<_>)`）
- [x] `fav/src/driver.rs`: `struct InferredField { name: String, ty: InferredType }`
- [x] `fav/src/driver.rs`: `struct InferredTypeDef { name: String, fields: Vec<InferredField>, source: String }`
- [x] `fav/src/driver.rs`: `fn is_bool_value(s: &str) -> bool`
- [x] `fav/src/driver.rs`: `fn is_int_value(s: &str) -> bool`
- [x] `fav/src/driver.rs`: `fn is_float_value(s: &str) -> bool`
- [x] `fav/src/driver.rs`: `fn infer_type_from_values(values: &[String]) -> InferredType`
- [x] `fav/src/driver.rs`: `fn format_inferred_type(ty: &InferredType) -> String`
- [x] `fav/src/driver.rs`: `fn table_name_to_type_name(table: &str) -> String`
- [x] `fav/src/driver.rs`: `fn format_type_def(def: &InferredTypeDef) -> String`
- [x] Test: `test_infer_int_values`
- [x] Test: `test_infer_float_values`
- [x] Test: `test_infer_bool_values`
- [x] Test: `test_infer_string_values`
- [x] Test: `test_infer_option_when_empty_present`
- [x] Test: `test_infer_all_empty_is_option_string`
- [x] Test: `test_table_name_to_type_name`
- [x] Test: `test_format_type_def_alignment`

## Phase 3: CSV 推論

- [x] `fav/src/driver.rs`: `fn infer_from_csv(csv_path: &str, type_name: &str) -> Result<InferredTypeDef, String>`
- [x] `fav/src/driver.rs`: `fn write_infer_output(content: &str, out: Option<&str>)`
- [x] `fav/src/driver.rs`: `cmd_infer` の CSV 分岐を実装
- [x] Test: `infer_csv_basic_types`
- [x] Test: `infer_csv_nullable_column`
- [x] Test: `infer_csv_all_empty_column`
- [x] Test: `infer_csv_header_only`
- [x] Test: `infer_csv_custom_name`

## Phase 4: SQLite スキーマ推論

- [x] `fav/src/driver.rs`: `fn sqlite_list_tables(...) -> Result<Vec<String>, String>`
- [x] `fav/src/driver.rs`: `fn sqlite_type_to_inferred(type_str: &str) -> InferredType`
- [x] `fav/src/driver.rs`: `fn infer_from_sqlite_table(...) -> Result<InferredTypeDef, String>`
- [x] `fav/src/driver.rs`: `fn write_infer_multi_output(defs: &[InferredTypeDef], out: Option<&str>)`
- [x] `fav/src/driver.rs`: `cmd_infer` の SQLite 分岐を実装
- [x] Test: `infer_sqlite_single_table`
- [x] Test: `infer_sqlite_nullable_column`
- [x] Test: `infer_sqlite_all_tables`
- [x] Test: `infer_sqlite_table_not_found`
- [x] Test: `sqlite_type_to_inferred_mapping`

## Phase 5: PostgreSQL スキーマ推論

- [x] `fav/src/driver.rs`: `fn postgres_type_to_inferred(pg_type: &str) -> InferredType`
- [x] `fav/src/driver.rs`: `#[cfg(feature = "postgres_integration")] fn infer_from_postgres(...)`
- [x] `fav/src/driver.rs`: `cmd_infer` の PostgreSQL 分岐（feature なし → stderr + exit 1）
- [x] Test: `postgres_type_to_inferred_mapping`
- [x] PostgreSQL 統合テストは `#[cfg(feature = "postgres_integration")]` でゲート（未実装）

## Phase 6: `--out` ディレクトリサポート + エラー整備

- [x] `write_infer_multi_output`: ディレクトリ末尾スラッシュ判定の実装
- [x] `write_infer_multi_output`: `std::fs::create_dir_all` でディレクトリ自動作成
- [x] ファイル名: `<dir>/<type_name_lowercase>.fav`
- [x] エラーメッセージ整備
- [x] Test: `infer_out_file_single_def`
- [x] Test: `infer_out_dir_multi_def`
- [x] Test: `infer_error_csv_not_found`
- [x] Test: `infer_error_table_not_found`
- [x] 既存の全テストが通ること確認 (`cargo test`) — 729 passed

## Phase 7: サンプル + ドキュメント

### サンプル

- [x] `fav/examples/infer_demo/data.csv` 作成（id, name, value, region, notes 5カラム; region/notes は空値あり）
- [x] `fav/examples/infer_demo/schema/row.fav` 作成（`fav infer data.csv` の出力例、静的コピー）
- [x] `fav/examples/infer_demo/src/main.fav` 作成

### ドキュメント

- [x] `versions/v3.4.0/langspec.md` 作成
- [x] `versions/v3.4.0/migration-guide.md` 作成（破壊的変更なし）
- [x] `versions/v3.4.0/progress.md` を全 Phase `[x]` に更新
