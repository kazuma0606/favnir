# v18.4.0 — Schema Types タスク

## ステータス: **完了** ✅

---

## タスク一覧

### T1: `fav/src/ast.rs` — `TypeExpr::Schema` 追加

- [x] `TypeExpr` enum に `Schema(String, Span)` variant を追加
- [x] `TypeExpr::span()` の match に `Schema(_, s) => s` を追加

### T2: 波及ファイル更新（`TypeExpr::Schema` 追加による exhaustive match 修正）

- [x] `fav/src/middle/checker.rs` — `resolve_type_expr_with_subst` / `resolve_type_expr_with_self` / `validate_type_expr_arity` に `Schema` case 追加
- [x] `fav/src/middle/compiler.rs` — `lower_type_expr*` / `substitute_self_in_type_expr` に `Schema(_, _) => Type::Unknown` 追加
- [x] `fav/src/middle/ast_lower_checker.rs` — `lower_te` / `te_to_string` に `Schema` case 追加
- [x] `fav/src/emit_python.rs` — `Schema(_, _) => "Any"` 追加
- [x] `fav/src/fmt.rs` — `Schema(uri, _) => format!("schema \"{}\"", uri)` 追加
- [x] `fav/src/driver.rs` — `favnir_type_display` / `format_type_expr` / `graphql_type_from_type_expr_nonnull` / `proto_type_from_type_expr_nonwrapper` / `favnir_type_to_sql_from_expr` に `Schema` case 追加
- [x] `cargo build` でコンパイルエラーが 0 になることを確認

### T3: `fav/src/frontend/parser.rs` — `schema "..."` 解析

- [x] `parse_base_type` 内で `TokenKind::Ident(s)` かつ `s == "schema"` を検出
- [x] `schema` を consume した後、`TokenKind::Str` を expect して URI を取得
- [x] `TypeExpr::Schema(uri, span)` を返す

### T4: `fav/src/middle/schema_loader.rs` — 新規作成

- [x] `SchemaSource` enum（`File(PathBuf)` / `BigQuery` / `Postgres` / `Snowflake`）
- [x] `SchemaField { name: String, ty: String }` struct
- [x] `parse_schema_uri(uri: &str) -> Result<SchemaSource, String>` 実装
- [x] `cache_key(source: &SchemaSource) -> String` 実装
- [x] `load_schema_uri(uri, refresh)` / `load_schema(source, cache_dir, refresh)` 実装
- [x] JSON Schema → `SchemaField` 変換（`integer`→`Int` / `number`→`Float` / `boolean`→`Bool` / `array`→`List<T>` / nullable→`Option<T>`）
- [x] キャッシュ読み書き（`.fav/schema-cache/{cache_key}.json`）
- [x] `schema_field_type_to_type(ty_str) -> Type` 変換関数
- [x] `fav/src/middle/mod.rs` に `pub mod schema_loader;` を追加

### T5: `fav/src/middle/checker.rs` — スキーマ解決

- [x] `schema_types: HashMap<String, Type>` / `schema_refresh: bool` フィールドを `Checker` struct に追加
- [x] `register_schema_types(&mut self, program)` — `type X = schema "..."` を前パスで解決し `record_fields` / `schema_types` / `env` に登録
- [x] `resolve_schema(&self, uri) -> Type` — `schema_types` を参照して型を返す
- [x] `TypeExpr::Schema(uri, _)` → `resolve_schema(uri)` に委譲
- [x] `check_program` / `check_with_self` / `check_program_and_export` で `register_schema_types` を呼び出し（`register_item_signatures` の前）
- [x] `type X = schema "..."` は `TypeDef(TypeBody::Alias(TypeExpr::Schema(...)))` としてパースされることを確認し、checker 側も `TypeDef` を参照するように修正

### T6: `fav/src/driver.rs` / `fav/src/main.rs` — `--refresh-schemas` フラグ

- [x] `cmd_check` の引数に `refresh_schemas: bool` を追加
- [x] `check_single_file_legacy` の引数に `refresh_schemas: bool` を追加し `checker.schema_refresh` に設定
- [x] `check_single_file` の引数に `refresh_schemas: bool` を追加
- [x] `main.rs` で `--refresh-schemas` フラグを認識して `cmd_check` に渡す
- [x] 既存の `check_single_file_legacy(&path, false)` / `check_single_file(&path, false, false)` を全箇所修正

### T7: `fav/src/driver.rs` — `v184000_tests` 追加

- [x] `v183000_tests::version_is_18_3_0` を `#[ignore]` に変更
- [x] `v184000_tests` モジュール追加（5件）:
  - [x] `version_is_18_4_0` — Cargo.toml に "18.4.0" が含まれる
  - [x] `schema_uri_parses` — `parse_schema_uri("file:...")` が `SchemaSource::File` を返す
  - [x] `schema_type_syntax_parses` — `type X = schema "..."` が `TypeDef(TypeBody::Alias(Schema(...)))` にパースされる
  - [x] `schema_cache_key_is_stable` — 同 URI で同一キャッシュキーが生成される
  - [x] `schema_file_missing_gives_error` — 存在しないファイルで E0338 のみ（他エラーなし）

### T8: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `18.3.0` → `18.4.0` に更新

### T9: `site/content/docs/language/schema-types.mdx` 作成

- [x] `type X = schema "file:path.json"` の基本構文を記載
- [x] JSON Schema からの型マッピング表を記載
- [x] スキーマキャッシュの説明（`.fav/schema-cache/`）を記載
- [x] `fav check --refresh-schemas` の使い方を記載
- [x] E0338 / E0339 エラーの説明を記載

---

## テスト結果（v184000_tests、5/5 PASS）

| テスト名 | 結果 |
|---|---|
| `version_is_18_4_0` | ✅ PASS |
| `schema_uri_parses` | ✅ PASS |
| `schema_type_syntax_parses` | ✅ PASS |
| `schema_cache_key_is_stable` | ✅ PASS |
| `schema_file_missing_gives_error` | ✅ PASS |

**全体テスト: 1675 passed / 0 failed / 38 ignored**

---

## 完了条件チェックリスト

- [x] `fav/Cargo.toml` のバージョンが `18.4.0`
- [x] `TypeExpr::Schema(String, Span)` が `ast.rs` に存在する
- [x] `type X = schema "file:..."` がパースされる（`TypeDef(TypeBody::Alias(Schema(...)))`）
- [x] `schema_loader.rs` が存在し `load_schema_uri` が `Vec<SchemaField>` を返す
- [x] `file:` スキーマから生成されたフィールドが `record_fields` / `schema_types` に登録される
- [x] `--refresh-schemas` フラグが動作する（`checker.schema_refresh` に伝達）
- [x] `site/content/docs/language/schema-types.mdx` が存在する
- [x] `cargo test v184000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（1675 tests pass）

---

## 実装上の注意点（後続バージョン向けメモ）

- `type X = schema "..."` は `alias X = schema "..."` ではなく `TypeDef(TypeBody::Alias(...))` として解析される。`AliasDecl` を期待すると失敗する。
- `register_schema_types` は `TypeDef` と `AliasDecl` の両方を確認する（両方対応済み）。
- `schema_refresh` は `check_single_file_legacy` 経由でのみ Rust Checker に到達する（Favnir checker path では現時点で未使用）。
- BigQuery / Postgres / Snowflake ソースは URI パースのみ実装、フィールド取得は v18.x 後続で実装予定。
