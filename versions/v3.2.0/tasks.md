# Favnir v3.2.0 Tasks

## Phase 0: Version Bump

- [x] `fav/Cargo.toml`: `version = "3.2.0"`
- [x] `cargo build` 成功、`env!("CARGO_PKG_VERSION")` 伝播
- [x] `fav --version` で `favnir 3.2.0` を確認

## Phase 1: `SchemaError` 型 + `FieldAttr` AST 拡張

### 1-A: `SchemaError` 型

- [x] `checker.rs`: `"SchemaError"` を stdlib 型として登録
  - フィールド: `field: String`, `expected: String`, `got: String`
- [x] `"Schema"` namespace を checker の stdlib グローバル登録ループに追加
- [x] `compiler.rs`: `"Schema"` を2箇所のグローバル登録ループに追加

### 1-B: `FieldAttr` AST 拡張

- [x] `ast.rs`: `FieldAttr { name: String, arg: Option<String>, span: Span }` 追加
- [x] `ast.rs`: `Field` に `attrs: Vec<FieldAttr>` フィールド追加
- [x] `frontend/parser.rs`: フィールド前の `#[attr(arg)]` を解析するパーサー追加
- [x] `frontend/parser.rs`: アノテーションなしフィールドは `attrs: vec![]`
- [x] Test: `parse_field_with_col_attr` — `#[col(0)] id: Int` のパース確認
- [x] Test: `parse_type_with_multiple_col_attrs` — 複数フィールドの `#[col(n)]`

## Phase 2: 型メタデータシステム

### 2-A: `TypeMeta` IR ノード

- [x] `src/middle/ir.rs`: `FieldMeta { name, ty, col_index: Option<usize> }` 追加
- [x] `src/middle/ir.rs`: `TypeMeta { type_name: String, fields: Vec<FieldMeta> }` 追加
- [x] `IRProgram` に `type_metas: HashMap<String, TypeMeta>` 追加
- [x] `middle/compiler.rs`: 型定義コンパイル時に `TypeMeta` を収集して格納
- [x] `#[col(n)]` アノテーションがあるとき `FieldMeta.col_index` に格納

### 2-B: `type_name_of<T>()` 組み込み

- [x] `middle/checker.rs`: `type_name_of<T>()` を特殊形式として認識、戻り型 `String`
- [x] `middle/compiler.rs`: `type_name_of<ConcreteType>()` を型名文字列リテラルに置換
- [x] Test: `type_meta_is_embedded_in_artifact`

## Phase 3: CSV VM プリミティブ

- [x] `backend/vm.rs`: `Csv.parse_raw(text, delimiter, has_header)` ビルトイン追加
  - `csv` クレート使用；ヘッダーあり/なし両対応
  - パースエラー → `VMValue::Err(SchemaError { ... })`
- [x] `backend/vm.rs`: `Csv.write_raw(rows, delimiter)` ビルトイン追加
- [x] `backend/vm.rs`: `Schema.adapt(rows, type_name)` ビルトイン追加
  - `type_metas` から `TypeMeta` を参照し各フィールドを型変換
  - `#[col(n)]` がある型は位置ベースマッピングを使用
- [x] `backend/vm.rs`: `Schema.to_csv(rows, type_name)` ビルトイン追加
- [x] `checker.rs`: 上記4関数の型シグネチャを登録
- [x] Test: `csv_parse_raw_header`
- [x] Test: `csv_parse_raw_no_header`
- [x] Test: `csv_write_raw_produces_correct_text`
- [x] Test: `schema_adapt_int_field`
- [x] Test: `schema_adapt_float_field`
- [x] Test: `schema_adapt_bool_field`
- [x] Test: `schema_adapt_option_field_none`
- [x] Test: `schema_adapt_option_field_some`
- [x] Test: `schema_adapt_type_mismatch_returns_err`

## Phase 4: JSON VM プリミティブ

- [x] `backend/vm.rs`: `Json.parse_raw(text)` ビルトイン追加
- [x] `backend/vm.rs`: `Json.parse_array_raw(text)` ビルトイン追加
- [x] `backend/vm.rs`: `Json.write_raw(map)` ビルトイン追加
- [x] `backend/vm.rs`: `Json.write_array_raw(rows)` ビルトイン追加
- [x] `backend/vm.rs`: `Schema.adapt_one(map, type_name)` ビルトイン追加
- [x] `backend/vm.rs`: `Schema.to_json(value, type_name)` ビルトイン追加
- [x] `backend/vm.rs`: `Schema.to_json_array(rows, type_name)` ビルトイン追加
- [x] `checker.rs`: 上記7関数の型シグネチャを登録
- [x] Test: `json_parse_raw_basic_object`
- [x] Test: `json_parse_array_raw_basic`
- [x] Test: `json_write_raw_produces_object`
- [x] Test: `json_write_array_raw_produces_array`
- [x] Test: `schema_adapt_one_from_json`

## Phase 5: rune ファイル作成

> **注**: rune ファイルの配置場所は `<repo_root>/runes/` (`fav/` の外) に統一されている。
> `exec_project_main_source_with_runes` は `env!("CARGO_MANIFEST_DIR")/../runes/` を参照する。

### csv rune (`runes/csv/`)

- [x] `runes/csv/csv.fav` 作成
  - `public type CsvOptions = { delimiter: String  has_header: Bool }`
  - `public fn parse<T>`: `chain raw <- Csv.parse_raw(...)` + `Schema.adapt`
  - `public fn parse_positional<T>`: `#[col(n)]` 版
  - `public fn write<T>`: `Schema.to_csv`
  - `public fn parse_with_opts<T>`: `CsvOptions` 付き版
  - 注: `stage` ではなく `fn` で定義（`chain` キーワードを使うため）
- [x] `runes/csv/csv.test.fav` 作成（6 テスト、rune import なしの自己完結形式）
  - `test_parse_header_csv`
  - `test_parse_positional_csv`
  - `test_write_csv`
  - `test_parse_type_mismatch_returns_err`
  - `test_parse_option_field`
  - `test_chain_integration`

### json rune (`runes/json/`)

- [x] `runes/json/json.fav` 作成
  - `public fn parse<T>`: `Json.parse_raw` + `Schema.adapt_one`
  - `public fn parse_list<T>`: `Json.parse_array_raw` + `Schema.adapt`
  - `public fn write<T>`: `Schema.to_json`
  - `public fn write_list<T>`: `Schema.to_json_array`
- [x] `runes/json/json.test.fav` 作成（5 テスト、自己完結形式）
  - `test_parse_object`
  - `test_parse_list`
  - `test_write_object`
  - `test_write_list`
  - `test_parse_error_invalid_json`

## Phase 6: checker / compiler 型チェック統合

- [x] `error_catalog.rs`: E0501〜E0505 を追加
  - E0501: schema field missing
  - E0502: schema type mismatch
  - E0503: invalid col index
  - E0504: json parse error
  - E0505: csv parse error
- [x] `middle/checker.rs`: `#[col(n)]` の `n` が非負整数チェック（コンパイル時 E0503）
- [x] `middle/checker.rs`: `Schema.adapt` の戻り型を `Result<List<T>, SchemaError>` として推論
- [x] `middle/checker.rs`: `Schema.adapt_one` の戻り型を `Result<T, SchemaError>` として推論

## Phase 7: サンプル + 統合テスト

### サンプル

- [x] `examples/csv_demo/users.csv` 作成（サンプルデータ、ヘッダーあり）
- [x] `examples/csv_demo/src/main.fav` 作成（CSV 読み込み → match → 出力）
- [x] `examples/json_demo/config.json` 作成
- [x] `examples/json_demo/src/main.fav` 作成（JSON 読み込み → 操作）

### driver.rs 統合テスト（`tests` モジュール内）

> **注**: csv_json_tests 専用モジュールではなく、既存 `tests` モジュールに追加されている。

- [x] Test: `csv_rune_parse_and_write_roundtrip`
- [x] Test: `csv_rune_schema_error_propagates` (`chain` との連携)
- [x] Test: `col_annotation_maps_by_position`
- [x] Test: `option_field_maps_empty_to_none`
- [x] Test: `csv_rune_test_file_passes` (runes/csv/csv.test.fav の全テストを実行)
- [x] Test: `json_rune_parse_and_write_roundtrip`
- [x] Test: `json_rune_parse_list`
- [x] Test: `json_schema_error_on_type_mismatch`
- [x] Test: `json_rune_write_list`
- [x] Test: `json_rune_test_file_passes` (runes/json/json.test.fav の全テストを実行)
- [x] 既存の全テストが通ること確認 (`cargo test`)

## Phase 8: ドキュメント

- [x] `versions/v3.2.0/langspec.md` 作成
- [x] `versions/v3.2.0/migration-guide.md` 作成（破壊的変更なし）
- [x] `versions/v3.2.0/progress.md` を全 Phase `[x]` に更新
