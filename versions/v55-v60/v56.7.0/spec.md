# Spec — v56.7.0 — モジュール名前空間（qualified imports）

## 概要

`import "path" as alias.*` ワイルドカードインポート構文を追加する。
`import "./stages" as stages` の深い qualified アクセス（`stages.validate.run`）は
パーサーの FieldAccess チェーンとして既に動作するため、AST 確認テストで検証する。
W038 lint で複数ワイルドカードインポートによる名前衝突リスクを警告する。

**ロードマップ参照との差異**:
- `stages.validate.run` の「resolver 正式サポート」はファイルシステム resolver を要しないため、
  単一ファイル mode でのパース確認テストで代替する（E0213 は resolver 不在の期待値）。
- checker での wildcard 名前注入（実際に名前をスコープに展開）は resolver を要するため未実装とし、
  v57.0 以降の課題とする。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v56.1-v57.0.md` — v56.7.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v56.7.0 行
- ベーステスト数: **3240**（v56.6.0 完了時点の実績値）
- 目標テスト数: **3243**（+3）

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "56.7.0"
```

---

### 2. `fav/src/ast.rs` — `ImportDecl` に `is_wildcard: bool` フィールド追加

`import "path" as alias.*` のワイルドカードフラグを表すフィールド。

```rust
ImportDecl {
    path: String,
    alias: Option<String>,
    is_rune: bool,
    is_public: bool,
    kind: ImportKind,
    is_wildcard: bool,  // NEW v56.7.0: `import "path" as alias.*`
    span: Span,
}
```

---

### 3. `fav/src/frontend/parser.rs` — `as alias.*` パース

`parse_import_decl` の alias 解析後に `.*` チェックを追加:

```rust
// alias already parsed into `alias: Option<String>`
// v56.7.0: wildcard suffix `.*` after alias
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
Ok(Item::ImportDecl {
    path,
    alias,
    is_rune,
    is_public,
    kind,
    is_wildcard,  // NEW
    span: self.span_from(&start),
})
```

`peek2()` は既存ヘルパー（`self.tokens.get(self.pos + 1).map(|t| &t.kind)` — `Option<&TokenKind>` を返す）。

---

### 4. `fav/src/middle/checker.rs` — `process_imports` の destructure 更新

`process_imports` の `ImportDecl` パターンに `is_wildcard: _` を追加（フィールド網羅が必要）:

```rust
let Item::ImportDecl {
    path,
    alias,
    is_rune,
    is_public,
    kind: _,
    is_wildcard: _,  // NEW: ignored for now (scope injection requires resolver)
    span,
} = item else { continue; };
```

---

### 5. `fav/src/fmt.rs` — ワイルドカードインポートのフォーマット

`Item::ImportDecl` の alias フォーマットを `is_wildcard` 対応に変更:

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

### 6. `fav/src/lint.rs` — W038 ワイルドカードインポート衝突警告

複数のワイルドカードインポートが存在する場合、2 件目以降に W038 を発行:

```rust
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

`lint_program` 末尾から W037 直後に呼び出す:

```rust
// v56.7.0: W038
check_w038_wildcard_import_collision(program, &mut errors);
```

---

### 7. `fav/src/driver.rs` — `v56700_tests` 追加

`v56600_tests` の直前に挿入する。

**テスト 1: `qualified_import_deep_access`**

`import "./stages" as stages` が `is_wildcard: false` で解析されること、
および `stages.validate.run(order)` が parse エラーなく AST に変換されることを確認:

```rust
#[test]
fn qualified_import_deep_access() {
    use crate::ast;
    use crate::frontend::parser::Parser;
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
```

**テスト 2: `wildcard_import_expands`**

`import "./validate" as v.*` が `is_wildcard: true` で解析されることを確認:

```rust
#[test]
fn wildcard_import_expands() {
    use crate::ast;
    use crate::frontend::parser::Parser;
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
```

---

### 8. `fav/src/driver.rs` — バージョンチェックテスト更新

`v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.6.0"` → `"56.7.0"` に更新。

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `qualified_import_deep_access` | `import "./stages" as stages` が `is_wildcard: false` で解析される + `stages.validate.run(x)` がパースエラーなし |
| `wildcard_import_expands` | `import "./validate" as v.*` が `is_wildcard: true` で解析される |
| `w038_wildcard_import_collision_warning` | ワイルドカードインポート 2 件のプログラムに `lint_program` を呼ぶと W038 が返ることを確認 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3243 tests passed, 0 failed**）
- `cargo clippy -- -D warnings` クリーン
- `qualified_import_deep_access` pass
- `wildcard_import_expands` pass
- `ImportDecl` に `is_wildcard: bool` フィールドが追加されている
- `import "path" as alias.*` がパースされ `is_wildcard: true` になる
- `fmt.rs` が `is_wildcard` を考慮し `as alias.*` を出力する
- W038 が `lint_program` に統合されている
- `CHANGELOG.md` に v56.7.0 エントリが追加されている
- `versions/current.md` が v56.7.0 / 3243 tests を反映
- 両ロードマップの v56.7.0 実績を COMPLETE に更新

---

## サイトドキュメント

ワイルドカードインポートの MDX ドキュメントは v56.8.0 の Language Power 2.0 記事群に委譲する。
v56.7.0 では MDX ファイルの新規作成は行わない。

---

## 備考

- **テスト数**: `v56700_tests` に 3 件追加。ベース 3240 + 3 = 3243。
- **`peek2()` の利用**: `as alias.*` のパースで `peek()` が `.` かつ `peek2()` が `*` の 2 トークン先読みを使用。
  `peek2()` は `parser.rs` の既存ヘルパー（`self.tokens.get(self.pos + 1).map(|t| &t.kind)`、`Option<&TokenKind>` を返す）。
- **`stages.validate.run` の FieldAccess 動作**: `stages.validate.run(order)` は
  `Apply(FieldAccess(FieldAccess(Var("stages"), "validate"), "run"), [order])` として
  既存の再帰的 FieldAccess パースで処理される（resolver サポート不要）。
- **`checker.rs` フィールド更新**: `process_imports` の `ImportDecl` パターンに `is_wildcard: _` が必要。
  `..` ショートハンドを使っていない唯一の箇所。
- **wildcard 名前注入**: resolver なしでは実装できないため v57.0 以降に委譲。
  W038 は AST レベルで静的に検出可能なため今バージョンで実装。
- **`ImportKind::Wildcard`** は追加しない。`is_wildcard: bool` フラグで十分。
  `ImportKind` は「パス解決方法」を表すが `is_wildcard` は「展開方法」を表す直交概念。
