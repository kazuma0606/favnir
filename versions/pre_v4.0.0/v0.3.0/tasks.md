# Favnir v0.3.0 タスク一覧

更新日: 2026-04-28 (全タスク完了)

タスクが完了したら `[ ]` を `[x]` に変える。

---

## Phase 1: Lexer / Parser

### Lexer

- [x] 1-1: `namespace` キーワードを `TokenKind::Namespace` として追加する
- [x] 1-2: `use` キーワードを `TokenKind::Use` として追加する
- [x] 1-3: Lexer の単体テストを更新する (`test_keywords` に namespace / use を追加)

### AST

- [x] 1-4: `Program` 構造体に `namespace: Option<String>` フィールドを追加する
- [x] 1-5: `Program` 構造体に `uses: Vec<Vec<String>>` フィールドを追加する
- [x] 1-6: `Item` 列挙体に `NamespaceDecl(String, Span)` を追加する (内部表現用)
- [x] 1-7: `Item` 列挙体に `UseDecl(Vec<String>, Span)` を追加する (内部表現用)

### Parser

- [x] 1-8: `parse_module_path()` を実装する (`IDENT ("." IDENT)*` → `Vec<String>`)
- [x] 1-9: `parse_namespace_decl()` を実装する (`namespace` + module_path)
- [x] 1-10: `parse_use_decl()` を実装する (`use` + module_path)
- [x] 1-11: `parse_program()` の先頭で namespace / use を収集するよう変更する
- [x] 1-12: Parser の単体テストを追加する
  - `test_parse_namespace` — namespace 宣言のパース
  - `test_parse_use` — use 宣言のパース
  - `test_parse_namespace_and_use` — 両方の組み合わせ

---

## Phase 2: fav.toml パーサ

- [x] 2-1: `src/toml.rs` を新規作成する
- [x] 2-2: `FavToml` 構造体を定義する (`name`, `version`, `src` フィールド)
- [x] 2-3: `FavToml::load(project_root: &Path) -> Option<Self>` を実装する
- [x] 2-4: `FavToml::find_root(start: &Path) -> Option<PathBuf>` を実装する (上位ディレクトリを探索)
- [x] 2-5: `parse_fav_toml(content: &str) -> FavToml` を実装する (最小ラインパーサ)
  - `[rune]` セクションを認識する
  - `name = "..."`, `version = "..."`, `src = "..."` を読む
  - `#` コメント行をスキップする
  - `src` の省略時デフォルトは `"."` とする
- [x] 2-6: `toml.rs` の単体テストを書く
  - 全フィールドあり / src 省略 / コメント行 の各ケース

---

## Phase 3: Module Resolver

- [x] 3-1: `src/resolver.rs` を新規作成する
- [x] 3-2: `ModuleScope` 構造体を定義する (`symbols: HashMap<String, (Type, Visibility)>`)
- [x] 3-3: `Resolver` 構造体を定義する
  - `toml: Option<FavToml>`
  - `root: Option<PathBuf>`
  - `modules: HashMap<String, ModuleScope>`
  - `loading: HashSet<String>` (循環検出)
- [x] 3-4: `mod_path_to_file(root, src, mod_path) -> PathBuf` を実装する
- [x] 3-5: `Resolver::load_module(mod_path, errors) -> Option<&ModuleScope>` を実装する
  - 循環検出: `loading` に既にあれば E012 を報告して `None` を返す
  - キャッシュ: `modules` にあれば返す
  - ファイルを読んでパース・型チェックして `ModuleScope` に収録する
- [x] 3-6: `extract_public_scope(program, checker) -> ModuleScope` を実装する
  - checker.rs の `collect_exports` として実装。全 visibility を収録し resolve_use 側で private を弾く
- [x] 3-7: Resolver の単体テストを書く (tempfile クレートで一時ディレクトリを使用)

---

## Phase 4: 型チェックへの統合

- [x] 4-1: `Checker` に `current_file: Option<PathBuf>` フィールドを追加する
- [x] 4-2: `Checker` に `current_rune_root: Option<PathBuf>` フィールドを追加する (resolver 経由でアクセス可能; W001 実装で resolver.root を直接参照するため独立フィールドは不要と判断)
- [x] 4-3: `Checker` に `imported: HashMap<String, (Type, Visibility, PathBuf)>` フィールドを追加する
- [x] 4-4: `Checker::new_with_resolver(resolver)` コンストラクタを追加する
- [x] 4-5: `check_program` の先頭で `use` 宣言を解決するよう変更する (`resolve_uses` メソッド)
- [x] 4-6: `check_use_decl(use_path: &[String])` を実装する (`resolve_uses` 内で処理)
  - モジュールパスとシンボル名を分離する
  - `Resolver::load_module` を呼ぶ
  - E013 (シンボルが見つからない) を報告する
  - E014 (private シンボルを import しようとした) を報告する
  - 解決成功時は `env` と `imported` に追加する
- [x] 4-7: `check_symbol_visibility(name, span)` を実装する
  - `imported` に存在し `private` なら E015 を報告する
  - `imported` に存在し `internal` で別 rune なら E016 を報告する (v0.3.0 は single-rune なので未発動)
- [x] 4-8: `Checker::check_ident_expr` で `check_symbol_visibility` を呼ぶ
- [x] 4-9: `Span::dummy()` を `lexer.rs` に追加する (line: 0, col: 0 のゼロスパン)
- [x] 4-10: `namespace` 宣言とファイルパスの不一致を W001 として報告する (`check_namespace_match` メソッド)
- [x] 4-11: 型チェックの単体テストを追加する
  - `test_use_public_fn` — public fn を別ファイルから use できる
  - `test_use_private_fn_error` — private fn を use すると E014
  - `test_use_missing_symbol_error` — 存在しないシンボルは E013
  - `test_circular_import_error` — resolver レベルの循環検出と E012 の伝播を確認

---

## Phase 5: CLI 変更

- [x] 5-1: `cmd_run` の引数を `file: Option<&str>` に変更する
- [x] 5-2: `file` が `None` のとき `fav.toml` を探してエントリポイント (`src/main.fav`) を自動検出する
- [x] 5-3: `cmd_check` の引数を `file: Option<&str>` に変更する
- [x] 5-4: `file` が `None` のとき `fav.toml` を探して `src` 配下の全 `.fav` をチェックする
- [x] 5-5: `cmd_explain` の引数を `file: Option<&str>` に変更する
- [x] 5-6: CLI の引数パースを更新する (`fav run`, `fav check`, `fav explain` の file を省略可能に)
- [x] 5-7: `fav help` のヘルプテキストを更新する (プロジェクトモードの説明を追加)

---

## Phase 6: サンプルと動作確認

- [x] 6-1: `examples/multi_file/fav.toml` を作成する
- [x] 6-2: `examples/multi_file/src/data/users.fav` を作成する
  - `namespace data.users` 宣言あり
  - `private` / `internal` / `public` の各 fn を含む
  - `User` 型を `public type` として定義する
- [x] 6-3: `examples/multi_file/src/main.fav` を作成する
  - `use data.users.create` など複数の use を含む
  - `public fn main()` でエントリポイントを定義する
- [x] 6-4: `fav check` (引数なし、`examples/multi_file` ディレクトリで実行) が型エラーなく通ることを確認する
- [x] 6-5: `fav run --db :memory:` が動くことを確認する
- [x] 6-6: `fav explain` で全ファイルの定義が VIS 列付きで表示されることを確認する
- [x] 6-7: private なシンボルを別ファイルから参照したとき E014 が出ることを確認する (`examples/visibility_errors/` プロジェクトを作成)

---

## ドキュメント

- [x] 7-1: `README.md` に v0.3.0 の使い方 (`namespace`, `use`, `fav.toml`) を追記する
- [x] 7-2: `examples/multi_file/` のコードにコメントを書く (users.fav に visibility 解説コメント記載済み)
- [x] 7-3: `versions/roadmap.md` の v0.3.0 完了日を記録する (2026-04-28)
