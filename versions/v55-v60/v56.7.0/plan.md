# Plan — v56.7.0 — モジュール名前空間（qualified imports）

## 実装順序

```
Cargo.toml → ast.rs → parser.rs → checker.rs → fmt.rs → lint.rs → driver.rs
```

依存関係:
- `ast.rs` が先（`ImportDecl` に `is_wildcard` フィールド追加）
- `parser.rs` は `ast.rs` の `ImportDecl` 変更後
- `checker.rs` は `ast.rs` の `ImportDecl` 変更後（`process_imports` destructure 修正）
- `fmt.rs` は `ast.rs` の `ImportDecl` 変更後（`is_wildcard` フィールド参照）
- `lint.rs` は `ast.rs` の `ImportDecl` 変更後（`is_wildcard` フィールド参照）
- `driver.rs` は全変更後（テスト追加）

---

## Step 1: `fav/Cargo.toml` — バージョン更新

```toml
version = "56.7.0"
```

---

## Step 2: `fav/src/ast.rs` — `is_wildcard` フィールド追加

`ImportDecl` 列挙体バリアントに `is_wildcard: bool` を追加。

**変更箇所**: `ImportDecl { ... }` の `kind: ImportKind` の後、`span: Span` の前に挿入:

```rust
ImportDecl {
    path: String,
    alias: Option<String>,
    is_rune: bool,
    is_public: bool,
    kind: ImportKind,
    is_wildcard: bool,  // v56.7.0: `import "path" as alias.*`
    span: Span,
}
```

**影響確認**: `..` を使う箇所は自動的に対応（`compiler_fav_runner.rs`, `lint.rs`, `driver.rs`）。
`..` を使わない `checker.rs` の `process_imports` は別途修正が必要（Step 4）。
`fmt.rs` は `is_wildcard` を明示的にバインドして出力形式を変更するため Step 5 で手動更新する。

---

## Step 3: `fav/src/frontend/parser.rs` — `as alias.*` パース

`parse_import_decl` 関数の alias 解析直後（`let alias = ... };` の後）に追加:

```rust
// v56.7.0: wildcard suffix `.*` — `import "path" as alias.*`
let is_wildcard = if alias.is_some()
    && self.peek() == &TokenKind::Dot
    && self.peek2() == Some(&TokenKind::Star)
{
    self.advance(); // consume '.'
    self.advance(); // consume '*'
    true
} else {
    false
};
```

`Ok(Item::ImportDecl { ... })` に `is_wildcard,` を追加。

**注意点**: `peek()` は現在位置、`peek2()` は `self.tokens.get(self.pos + 1).map(|t| &t.kind)` で
`Option<&TokenKind>` を返す既存ヘルパー。alias 解析後の位置で `Dot` と `Star` が連続する場合のみ `.*` と認識する。

---

## Step 4: `fav/src/middle/checker.rs` — `process_imports` 修正

`process_imports` の `ImportDecl` destructure に `is_wildcard: _` を追加:

```rust
let Item::ImportDecl {
    path,
    alias,
    is_rune,
    is_public,
    kind: _,
    is_wildcard: _,  // NEW: not used yet
    span,
} = item else { continue; };
```

---

## Step 5: `fav/src/fmt.rs` — ワイルドカードフォーマット

`Item::ImportDecl` アームで `is_wildcard` を参照し、alias を `as alias.*` 形式に変更:

```rust
Item::ImportDecl {
    path,
    alias,
    is_public,
    is_wildcard,  // NEW
    kind,
    ..
} => {
    let public = if *is_public { "public " } else { "" };
    let alias = match alias {
        Some(a) if *is_wildcard => format!(" as {}.*", a),
        Some(a) => format!(" as {}", a),
        None => String::new(),
    };
    match kind {
        crate::ast::ImportKind::Package => format!("{public}import {path}{alias}"),
        _ => format!(r#"{public}import "{path}"{alias}"#),
    }
}
```

---

## Step 6: `fav/src/lint.rs` — W038 追加

ファイル末尾付近（W037/`check_unreachable_patterns` の後）に追加:

```rust
// ── W038: wildcard import collision (v56.7.0) ────────────────────────────────

fn check_w038_wildcard_import_collision(program: &Program, errors: &mut Vec<LintError>) {
    let wildcards: Vec<(&String, &Span)> = program.items.iter().filter_map(|item| {
        if let Item::ImportDecl { is_wildcard: true, path, span, .. } = item {
            Some((path, span))
        } else {
            None
        }
    }).collect();

    if wildcards.len() >= 2 {
        for (path, span) in &wildcards[1..] {
            errors.push(LintError::new(
                "W038",
                format!(
                    "wildcard import `as .*` from \"{}\" may cause name collisions with other wildcard imports; consider using qualified access instead",
                    path
                ),
                (*span).clone(),
            ));
        }
    }
}
```

`lint_program` の W037 呼び出し後に追加:

```rust
// v56.7.0: W038
check_w038_wildcard_import_collision(program, &mut errors);
```

---

## Step 7: `fav/src/driver.rs` — テスト追加

`v56600_tests` モジュールの直前に `v56700_tests` を追加:

```rust
// -- v56700_tests (v56.7.0) -- モジュール名前空間 (qualified imports / wildcard) --
#[cfg(test)]
mod v56700_tests {
    use crate::ast;
    use crate::frontend::parser::Parser;

    #[test]
    fn qualified_import_deep_access() {
        let src = r#"
import "./stages" as stages
fn run_order(order: String) -> Bool {
    stages.validate.run(order)
}
public fn main() -> Bool { true }
"#;
        let program = Parser::parse_str(src, "v56700_test.fav").expect("parse should succeed");
        let found = program.items.iter().any(|item| {
            matches!(item, ast::Item::ImportDecl { alias: Some(a), is_wildcard: false, .. } if a == "stages")
        });
        assert!(found, "should have non-wildcard alias import 'stages'");
    }

    #[test]
    fn wildcard_import_expands() {
        let src = r#"
import "./validate" as v.*
public fn main() -> Bool { true }
"#;
        let program = Parser::parse_str(src, "v56700_test.fav").expect("parse should succeed");
        let found = program.items.iter().any(|item| {
            matches!(item, ast::Item::ImportDecl { is_wildcard: true, .. })
        });
        assert!(found, "should have wildcard import (is_wildcard: true)");
    }
}
```

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.6.0"` → `"56.7.0"` に更新。

**テスト 3 — `w038_wildcard_import_collision_warning`** も同モジュールに追加:

```rust
#[test]
fn w038_wildcard_import_collision_warning() {
    use crate::frontend::parser::Parser;
    use crate::lint::lint_program;
    let src = r#"
import "./validate" as v.*
import "./transform" as t.*
public fn main() -> Bool { true }
"#;
    let program = Parser::parse_str(src, "v56700_test.fav").expect("parse");
    let errors = lint_program(&program);
    let codes: Vec<&str> = errors.iter().map(|e| e.code).collect();
    assert!(
        codes.contains(&"W038"),
        "two wildcard imports should trigger W038, got: {:?}", codes
    );
}
```

---

## リスク・注意点

| リスク | 対策 |
|---|---|
| `ImportDecl` に `..` を使わずデストラクト → コンパイルエラー | `checker.rs:process_imports` に `is_wildcard: _` を追加（Step 4） |
| `peek2()` が EOF を返す（alias なしの末尾 import） | `alias.is_some()` ガードで短絡評価（alias がないなら `.*` チェックしない） |
| `./` prefix のない `import "validate" as v.*` の is_wildcard | 動作: `is_wildcard: true` になる（パス解決方法とは独立） |
| `fmt.rs` ラウンドトリップ | `is_wildcard: true` 時は `as alias.*` を出力するため往復変換が保証される |
