# Favnir v2.6.0 実装計画

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

`Cargo.toml` を `version = "2.6.0"` に変更。
`src/main.rs` の HELP テキストを `v2.6.0` に更新。

---

## Phase 1 — レキサー拡張

### `src/frontend/lexer.rs`

`import` キーワードを追加する。

```rust
// TokenKind に追加
Import,  // "import"

// keywords map に追加
"import" => TokenKind::Import,
```

既存の `Use` トークンはそのまま残す（後方互換）。

---

## Phase 2 — AST 拡張

### `src/ast.rs`

`Item` に `ImportDecl` バリアントを追加する。

```rust
// Item に追加
ImportDecl {
    path: String,            // "models/user" など（クォート除去済み）
    alias: Option<String>,   // `as foo` の foo
    is_rune: bool,           // import rune "..." かどうか
    is_public: bool,         // public import "..." かどうか
    span: Span,
},
```

---

## Phase 3 — パーサー拡張

### `src/frontend/parser.rs`

`parse_import_decl` 関数を追加する。

```rust
fn parse_import_decl(&mut self, is_public: bool) -> Result<Item, ParseError> {
    self.expect(&TokenKind::Import)?;

    // rune フラグ
    let is_rune = if self.peek_is(&TokenKind::Ident) && self.peek_text() == "rune" {
        self.advance();
        true
    } else {
        false
    };

    // パス文字列リテラル
    let path_token = self.expect(&TokenKind::StringLit)?;
    let path = unquote(&path_token.text); // "models/user" → models/user

    // as alias
    let alias = if self.peek_is(&TokenKind::As) {
        self.advance();
        let name_token = self.expect(&TokenKind::Ident)?;
        Some(name_token.text.clone())
    } else {
        None
    };

    Ok(Item::ImportDecl { path, alias, is_rune, is_public, span: path_token.span })
}
```

`parse_item` で `TokenKind::Import` または `public import` を検出したら `parse_import_decl` を呼ぶ。

```rust
// parse_item に追加（public ... の後で Import を検出した場合も含む）
TokenKind::Import => self.parse_import_decl(false),
// public アームの中:
TokenKind::Import => self.parse_import_decl(true),
```

---

## Phase 4 — チェッカー拡張（import 処理）

### `src/middle/checker.rs`

#### 4-1: namespace テーブルの追加

```rust
// Checker 構造体に追加
pub imported_namespaces: HashMap<String, ModuleScope>,
// namespace 名 → そのモジュールの公開シンボル
```

#### 4-2: `process_import_decl` の実装

```rust
fn process_import_decl(
    &mut self,
    path: &str,
    alias: Option<&str>,
    is_rune: bool,
    is_public: bool,
    span: Span,
) -> Result<(), TypeError> {
    // namespace 名を決定（alias があれば alias、なければ末尾セグメント）
    let ns_name = alias.unwrap_or_else(|| path.split('/').last().unwrap_or(path));

    // E081: namespace 名が既に登録されていたら競合エラー
    if self.imported_namespaces.contains_key(ns_name) {
        return Err(TypeError::NamespaceConflict { name: ns_name.to_string(), path: path.to_string(), span });
    }

    // ファイルパスを解決してモジュールをロード
    let file_path = if is_rune {
        self.resolve_rune_path(path)
    } else {
        self.resolve_local_path(path)
    };

    // E080: 循環 import 検出（既存の Resolver.loading HashSet を活用）
    if self.resolver.is_loading(&file_path) {
        return Err(TypeError::CircularImport { path: path.to_string(), span });
    }

    // モジュールをロードしてパース・チェック
    let scope = self.load_and_check_module(&file_path, path)?;

    // is_public なら re-export マップにも登録
    if is_public {
        self.reexport_namespaces.insert(ns_name.to_string(), scope.clone());
    }

    self.imported_namespaces.insert(ns_name.to_string(), scope);
    Ok(())
}
```

#### 4-3: `namespace.Symbol` の型解決

```rust
// check_expr の FieldAccess アームまたは専用アームで処理
Expr::NamespacedIdent { namespace, name, span } => {
    if let Some(scope) = self.imported_namespaces.get(namespace) {
        if let Some(ty) = scope.get_public(name) {
            self.record_type(span, ty);
            return ty.clone();
        }
        return Err(TypeError::UnresolvedSymbol { namespace, name, span });
    }
    Err(TypeError::UnknownNamespace { namespace, span })
}
```

AST 上は `Expr::FieldAccess(Expr::Ident("user"), "ParseUser", span)` として表現される可能性が高い。
その場合 `check_field_access` で namespace テーブルを参照するよう既存コードを拡張する。

#### 4-4: E080 / E081 エラーコードの追加

```rust
// TypeError に追加
CircularImport { path: String, span: Span },   // E080
NamespaceConflict {                             // E081
    name: String,
    path1: String,
    path2: String,
    span: Span,
},
```

エラー表示（`fmt_error` 等）に以下を追加：

```
E080: circular import detected
  "models/user" imports "models/post" which imports "models/user"

E081: namespace conflict: 'user' is imported from both "models/user" and "auth/user"
  hint: use `as` to resolve:
    import "models/user" as model_user
    import "auth/user"   as auth_user
```

---

## Phase 5 — ドライバー拡張（fav check --dir）

### `src/driver.rs`

```rust
pub fn cmd_check_dir(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // dir 以下の *.fav を再帰収集（既存の collect_fav_files_recursive を流用）
    let files = collect_fav_files_recursive(dir);

    // import グラフを構築してトポロジカルソート
    let sorted = topological_sort_imports(&files)?;

    let mut all_errors: Vec<(String, Vec<TypeError>)> = vec![];
    let mut any_error = false;

    for file in sorted {
        let source = std::fs::read_to_string(&file)?;
        let errors = check_source(&source, Some(&file));
        if !errors.is_empty() {
            any_error = true;
            all_errors.push((file, errors));
        }
    }

    // 全エラーをまとめて出力
    for (file, errors) in &all_errors {
        for e in errors {
            eprintln!("[{}] {}", file, format_type_error(e));
        }
    }

    if any_error {
        std::process::exit(1);
    }
    Ok(())
}
```

### `src/main.rs`

```rust
// check サブコマンドに --dir フラグを追加
["check", "--dir", dir] => driver::cmd_check_dir(dir),
// ヘルプテキストに追加:
// "  fav check --dir <dir>   Check all .fav files under directory"
```

---

## Phase 6 — テスト追加

### `src/frontend/lexer.rs`

```rust
#[test]
fn import_keyword_is_tokenized() {
    // "import" → TokenKind::Import
}
```

### `src/frontend/parser.rs`

```rust
#[test]
fn parse_simple_import() {
    // import "models/user" → ImportDecl { path: "models/user", alias: None, is_rune: false, is_public: false }
}

#[test]
fn parse_import_with_alias() {
    // import "models/user" as u → alias: Some("u")
}

#[test]
fn parse_rune_import() {
    // import rune "validate" → is_rune: true
}

#[test]
fn parse_public_import() {
    // public import "models/user" → is_public: true
}
```

### `src/middle/checker.rs`

```rust
#[test]
fn import_resolves_public_symbol() {
    // import "models/user"; user.ParseUser(...) が型解決される
}

#[test]
fn import_e080_circular_import() {
    // A が B を import、B が A を import → E080
}

#[test]
fn import_e081_namespace_conflict() {
    // import "models/user"; import "auth/user" → E081（同じ namespace "user"）
}

#[test]
fn import_with_alias_resolves() {
    // import "models/user" as m; m.ParseUser(...) が解決される
}
```

### `src/driver.rs`

```rust
#[test]
fn check_dir_finds_errors_in_all_files() {
    // ディレクトリ以下の複数 .fav ファイルのエラーを一括報告
}

#[test]
fn check_dir_exits_0_for_clean_dir() {
    // エラーのないディレクトリは正常終了
}
```

---

## Phase 7 — ドキュメント・最終確認

- `versions/v2.6.0/langspec.md` を作成
  - `import "path"` / `import rune "..."` / `public import "..."` の構文説明
  - namespace 参照の記法（`ns.Symbol`）
  - E080 / E081 エラーの説明
  - `fav check --dir` の説明
  - 既存 `use` との共存・互換性
- `cargo build` で警告ゼロを確認
- `cargo test` で全テスト通過を確認（v2.5.0 の 595 → 目標 607 程度）

---

## テスト数の見込み

v2.5.0 ベースライン: 595

- lexer テスト: +1
- parser テスト: +4
- checker テスト: +4
- driver テスト: +2
- 余裕を含めた目標: **607**（+12 程度）

---

## 注意点

### 既存 `Resolver` との統合

既存の `Resolver` は `use dotted.path` 形式を処理する。
`import "path"` の実装では Resolver の `load_module` / `is_loading` を活用して循環検出を再利用できる。
ただし `import` 専用の namespace テーブル（`imported_namespaces`）は Checker 側に持つ。

### `namespace.Symbol` の AST 表現

パーサー後の AST では `user.ParseUser` は `Expr::FieldAccess(Expr::Ident("user"), "ParseUser")` として表現される。
チェッカーの `check_field_access` が `Expr::Ident` の名前を `imported_namespaces` に照合し、
一致した場合はモジュールスコープからシンボルを解決する分岐を追加する。

### rune パス解決

`import rune "validate"` → `runes/validate/validate.fav` を探す。
`fav.toml` に `[runes] path = "..."` があればそのパスを起点にする。
なければカレントディレクトリの `runes/` を使う。

### `public import` の re-export

`public import "models/user"` はバレルファイルで使う。
バレルファイル自体が import されたとき、その re-export シンボルも namespace として公開される。
実装は `reexport_namespaces: HashMap<String, ModuleScope>` で管理し、
バレルを import した側がアクセスできるようにする。
