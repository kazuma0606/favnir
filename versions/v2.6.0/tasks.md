# Favnir v2.6.0 タスクリスト

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "2.6.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v2.6.0` に更新

---

## Phase 1 — レキサー拡張

### `src/frontend/lexer.rs`

- [x] `TokenKind::Import` を追加
- [x] キーワードマップに `"import" => TokenKind::Import` を追加

---

## Phase 2 — AST 拡張

### `src/ast.rs`

- [x] `Item::ImportDecl` バリアントを追加
  - [x] `path: String`（クォート除去済みパス文字列）
  - [x] `alias: Option<String>`（`as foo` の foo）
  - [x] `is_rune: bool`（`import rune "..."` かどうか）
  - [x] `is_public: bool`（`public import "..."` かどうか）
  - [x] `span: Span`

---

## Phase 3 — パーサー拡張

### `src/frontend/parser.rs`

- [x] `parse_import_decl(is_public: bool) -> Result<Item, ParseError>` を実装
  - [x] `import` キーワードを消費
  - [x] `rune` 識別子があれば `is_rune = true` としてスキップ
  - [x] 文字列リテラルをパスとして取得（クォート除去）
  - [x] `as <ident>` があれば alias として取得
  - [x] `Item::ImportDecl { ... }` を返す
- [x] `parse_item` に `TokenKind::Import` アームを追加（`parse_import_decl(false)` を呼ぶ）
- [x] `parse_item` の `public` アームに `Import` ケースを追加（`parse_import_decl(true)` を呼ぶ）

---

## Phase 4 — チェッカー拡張

### `src/middle/checker.rs`

- [x] `Checker` 構造体に新フィールドを追加
  - [x] `pub imported_namespaces: HashMap<String, ModuleScope>` — namespace 名 → モジュールスコープ
  - [x] `reexport_namespaces: HashMap<String, ModuleScope>` — public import の re-export 用（内部）
  - [x] `imported_namespace_paths: HashMap<String, String>` — E081 重複検出用（内部）
- [x] E080/E081 エラーを文字列コードとして `type_error` 呼び出しで実装
  - [x] E080: circular import detected（`process_import_decl` 内で `begin_loading` 戻り値を判定）
  - [x] E081: namespace conflict（`imported_namespace_paths` への重複挿入で検出）
- [x] エラー表示に E080/E081 のフォーマットを追加
  - [x] E080: `circular import detected\n  "A" imports "B" which imports "A"`
  - [x] E081: `namespace conflict: 'ns' is imported from both "A" and "B"\n  hint: use \`as\` to resolve:`
- [x] `process_import_decl` を実装
  - [x] namespace 名を決定（alias 優先、なければ末尾セグメント）
  - [x] E081: 既存 namespace 名との重複チェック
  - [x] `is_rune` によるパス解決の分岐（Resolver の `resolve_rune_import_file` を使用）
  - [x] E080: 循環 import 検出（Resolver の `begin_loading`/`finish_loading` を活用）
  - [x] モジュールファイルをロード・パース・チェック
  - [x] `is_public` なら `reexport_namespaces` にも登録
  - [x] `imported_namespaces` に登録
- [x] `process_imports` を `check_program` の first_pass で呼び出す
- [x] `check_field_access`（`check_expr` の `FieldAccess` アーム）で namespace 参照を解決
  - [x] `Expr::FieldAccess(Expr::Ident(ns), sym)` の形式を検出
  - [x] `imported_namespaces[ns]` にシンボルが存在すればその型を返す
  - [x] 存在しなければ既存の型エラー処理にフォールスルー

---

## Phase 5 — ドライバー拡張

### `src/driver.rs`

- [x] `cmd_check_dir(dir: &str)` を実装
  - [x] `collect_fav_files_recursive(dir)` で `.fav` ファイルを再帰収集
  - [x] 各ファイルを `Checker::new_with_resolver` + `check_with_self` でチェック
  - [x] 全エラーを `format_diagnostic` + `eprintln!` 出力
  - [x] エラーがあれば `std::process::exit(1)`

### `src/main.rs`

- [x] `check --dir <dir>` の引数パターンを追加
- [x] `driver::cmd_check_dir(dir)` にルーティング
- [x] HELP テキストに `fav check --dir <dir>` の説明を追加

---

## Phase 6 — テスト追加

### `src/frontend/lexer.rs`

- [x] `import_keyword_is_tokenized`: `"import"` → `TokenKind::Import`

### `src/frontend/parser.rs`

- [x] `parse_simple_import`: `import "models/user"` → `ImportDecl { path: "models/user", alias: None, is_rune: false, is_public: false }`
- [x] `parse_import_with_alias`: `import "models/user" as u` → `alias: Some("u")`
- [x] `parse_rune_import`: `import rune "validate"` → `is_rune: true`
- [x] `parse_public_import`: `public import "models/user"` → `is_public: true`

### `src/middle/checker.rs`

- [x] `import_resolves_public_symbol`: `import "models/user"; user.ParseUser(...)` が型解決される
- [x] `import_e080_circular_import`: A が B を import し B が A を import → E080 が報告される
- [x] `import_e081_namespace_conflict`: 同じ namespace 名が 2 つ import される → E081 が報告される
- [x] `import_with_alias_resolves`: `import "models/user" as m; m.ParseUser(...)` が解決される

### `src/driver.rs`

- [x] `check_dir_finds_errors_in_all_files`: ディレクトリ以下の複数 `.fav` のエラーを一括報告
- [x] `check_dir_exits_0_for_clean_dir`: エラーのないディレクトリは正常終了（exit code 0）

---

## Phase 7 — ドキュメント・最終確認

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（v2.5.0 の 595 → 607）

### ドキュメント作成

- [x] `versions/v2.6.0/langspec.md` を作成
  - [x] `import "path"` / `import rune "..."` / `public import "..."` の構文説明
  - [x] namespace 参照（`ns.Symbol`）の説明
  - [x] E080 / E081 エラーの説明
  - [x] `fav check --dir <dir>` の動作説明
  - [x] 既存 `use` との共存・互換性（`use` は引き続き動作する）
  - [x] `fav.toml` の `[runes] path` 設定説明

---

## 完了条件チェック

- [x] `import "models/user"` で `user.Symbol` が参照できる
- [x] `import "models/user" as u` で `u.Symbol` が参照できる
- [x] `import rune "validate"` で `validate.Required` 等が参照できる
- [x] `public import "models/user"` でバレルファイルの re-export が機能する
- [x] E080: 循環 import を検出してエラーを報告する
- [x] E081: namespace 名の重複を検出して `as` ヒント付きでエラーを報告する
- [x] `fav check --dir src/` でディレクトリ一括チェックが動作する
- [x] `cargo test` 全テスト通過
- [x] `cargo build` 警告ゼロ
- [x] `Cargo.toml` バージョンが `"2.6.0"`
- [x] `versions/v2.6.0/langspec.md` 作成済み
