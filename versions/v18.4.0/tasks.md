# v18.4.0 — Schema Types タスク

## ステータス: 未着手

---

## タスク一覧

### T1: `fav/src/ast.rs` — `TypeExpr::Schema` 追加

- [ ] `TypeExpr` enum に `Schema(String, Span)` variant を追加
- [ ] `TypeExpr::span()` の match に `Schema(_, s) => s` を追加

### T2: 波及ファイル更新（`TypeExpr::Schema` 追加による exhaustive match 修正）

- [ ] `fav/src/middle/checker.rs` — `resolve_type_expr_with_subst` / `resolve_type_expr_with_self` / `validate_type_expr_arity` に `Schema` case 追加（T3 の本実装前はとりあえず `Type::Unknown`）
- [ ] `fav/src/middle/compiler.rs` — `lower_type_expr*` / `substitute_self_in_type_expr` に `Schema(_, _) => Type::Unknown` 追加
- [ ] `fav/src/middle/ast_lower_checker.rs` — `lower_te` / `te_to_string` に `Schema` case 追加（`v1("TeSimple", "Any")`）
- [ ] `fav/src/emit_python.rs` — `Schema(_, _) => "Any"` 追加
- [ ] `fav/src/fmt.rs` — `Schema(uri, _) => format!("schema \"{}\"", uri)` 追加
- [ ] `fav/src/driver.rs` — `favnir_type_display` / `format_type_expr` / `graphql_type_from_type_expr_nonnull` / `proto_type_from_type_expr_nonwrapper` など `TypeExpr` match 箇所に `Schema` case 追加
- [ ] `cargo build` でコンパイルエラーが 0 になることを確認

### T3: `fav/src/frontend/parser.rs` — `schema "..."` 解析

- [ ] `parse_base_type` 内で `TokenKind::Ident(s)` かつ `s == "schema"` を検出
- [ ] `schema` を consume した後、文字列トークンを expect して URI を取得
- [ ] `TypeExpr::Schema(uri, span)` を返す
- [ ] Lexer の文字列トークン種別を確認（`Str` か `Lit(Lit::Str(...))` か）

### T4: `fav/src/middle/schema_loader.rs` — 新規作成

- [ ] `SchemaSource` enum を定義（`File(PathBuf)` / `BigQuery { ... }` / `Postgres { ... }` / `Snowflake { ... }`）
- [ ] `SchemaField { name: String, ty: String }` struct を定義
- [ ] `parse_schema_uri(uri: &str) -> Result<SchemaSource, String>` を実装
  - [ ] `"file:"` → `SchemaSource::File(PathBuf)`
  - [ ] `"bigquery:"` → `SchemaSource::BigQuery { ... }`
  - [ ] `"postgres:"` → `SchemaSource::Postgres { ... }`
  - [ ] `"snowflake:"` → `SchemaSource::Snowflake { ... }`
- [ ] `cache_key(source: &SchemaSource) -> String` を実装
- [ ] `load_schema(source: &SchemaSource, cache_dir: &Path, refresh: bool) -> Result<Vec<SchemaField>, String>` を実装:
  - [ ] `refresh=false` かつキャッシュあり → キャッシュから `Vec<SchemaField>` を返す
  - [ ] `file:` ソース → JSON ファイルを読み込んで JSON Schema パース
  - [ ] `bigquery/postgres/snowflake` → 空の `Vec` を返す（v18.4.0 では未実装）
  - [ ] キャッシュ書き込み（`.fav/schema-cache/{cache_key}`）
  - [ ] `create_dir_all` でキャッシュディレクトリを自動作成
- [ ] JSON Schema → `SchemaField` 変換ロジック:
  - [ ] `"type": "integer"` → `"Int"`
  - [ ] `"type": "number"` → `"Float"`
  - [ ] `"type": "string"` → `"String"`
  - [ ] `"type": "boolean"` → `"Bool"`
  - [ ] 上記以外 → `"String"`（保守的フォールバック）
- [ ] `fav/src/lib.rs` または `fav/src/middle/mod.rs` に `pub mod schema_loader;` を追加

### T5: `fav/src/middle/checker.rs` — スキーマ解決

- [ ] `TypeExpr::Schema` の `resolve_type_expr_with_subst` / `resolve_type_expr_with_self` 実装を本実装に更新:
  - [ ] `schema_loader::parse_schema_uri(uri)` で URI をパース
  - [ ] `schema_loader::load_schema(source, cache_dir, refresh)` でフィールドリストを取得
  - [ ] フィールドリストを `record_fields` に登録（型名 `$schema:{uri}` を使用）
  - [ ] `Type::Named(synthetic_name, vec![])` を返す
  - [ ] URI パースエラー → `type_error("E0338", ...)` + `Type::Error`
  - [ ] スキーマ読み込みエラー → `type_error("E0339", ...)` + `Type::Error`
- [ ] E0338 / E0339 エラーを使用している場所でメッセージを定義

### T6: `fav/src/driver.rs` — `--refresh-schemas` フラグ

- [ ] `CheckOpts` struct に `refresh_schemas: bool` フィールドを追加
- [ ] `cmd_check` の引数パース部分で `--refresh-schemas` を認識
- [ ] `SCHEMA_REFRESH: Cell<bool>` thread-local を定義して `schema_loader` から参照できるようにする
  - 代替: `CheckOpts` を `schema_loader::load_schema` まで引き回す
- [ ] `--refresh-schemas` が指定されたとき、キャッシュを無視して再取得することを確認

### T7: `fav/src/driver.rs` — `v184000_tests` 追加

- [ ] `v183000_tests` の `version_is_18_3_0` を `#[ignore]` に変更
- [ ] `v184000_tests` モジュールを追加（5件）:
  - [ ] `version_is_18_4_0`
  - [ ] `schema_type_parses`（`type X = schema "file:..."` がパースされ AST に Schema variant が含まれる）
  - [ ] `schema_cache_creates`（`schema_loader::load_schema` がキャッシュファイルを生成する）
  - [ ] `schema_file_source`（`file:` ソースのスキーマが正しくフィールドに変換される）
  - [ ] `schema_type_checks`（スキーマ型のフィールドアクセスが型チェックを通る）

### T8: バージョン更新

- [ ] `fav/Cargo.toml` のバージョンを `18.3.0` → `18.4.0` に更新
- [ ] `cargo build` で `Cargo.lock` 更新

### T9: `site/content/docs/language/schema-types.mdx` 作成

- [ ] `type X = schema "file:path.json"` の基本構文を記載
- [ ] JSON Schema からの型マッピング表を記載
- [ ] スキーマキャッシュの説明（`.fav/schema-cache/`）を記載
- [ ] `fav check --refresh-schemas` の使い方を記載
- [ ] E0338 / E0339 エラーの説明を記載
- [ ] CI での推奨運用（`--refresh-schemas` を定期実行）を記載

---

## テスト（v184000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_4_0` | Cargo.toml に "18.4.0" が含まれる |
| `schema_type_parses` | `type X = schema "file:schemas/users.json"` が AST として解析される |
| `schema_cache_creates` | `load_schema` 呼び出し後にキャッシュファイルが生成される |
| `schema_file_source` | `file:` URI から `{ id: Int, name: String }` 等のフィールドが生成される |
| `schema_type_checks` | スキーマ型に対するフィールドアクセスが型チェックを通る |

---

## 完了条件チェックリスト

- [ ] `fav/Cargo.toml` のバージョンが `18.4.0`
- [ ] `TypeExpr::Schema(String, Span)` が `ast.rs` に存在する
- [ ] `type X = schema "file:..."` がパースされる
- [ ] `schema_loader.rs` が存在し `load_schema` が `Vec<SchemaField>` を返す
- [ ] `file:` スキーマから生成されたフィールドが `record_fields` に登録される
- [ ] E0338 / E0339 エラーコードが定義される
- [ ] `--refresh-schemas` フラグが動作する
- [ ] `site/content/docs/language/schema-types.mdx` が存在する
- [ ] `cargo test v184000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし

---

## 優先度

T1（ast.rs Schema 追加）                  ← 最初
T2（波及ファイル修正）                     ← T1 完了後すぐ（`cargo build` で確認）
T3（parser.rs schema 解析）               ← T1, T2 完了後
T4（schema_loader.rs 新規）               ← T1 完了後（T2, T3 と並列可）
T5（checker.rs スキーマ解決）              ← T3, T4 完了後
T6（driver.rs --refresh-schemas）          ← T4 完了後
→ T7（v184000_tests）                     ← T5, T6 完了後
T8（バージョン更新）                       ← T7 完了後
T9（ドキュメント）                         ← T8 と並列可

**重要**: T1 の `TypeExpr::Schema` 追加は exhaustive match エラーを多数発生させる。
T2 で全ての波及箇所を修正してから T3 以降に進む。
