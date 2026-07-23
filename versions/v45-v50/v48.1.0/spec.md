# Spec: v48.1.0 — import 構文刷新 AST + parser（パッケージ）

## 概要

`import kafka`（引用符なし bare ident）構文を新しいパッケージ import として解析できるようにする。
`ast.rs` に `ImportKind` enum を追加し、`ImportDecl` に `kind` フィールドを追加。
`parser.rs` の bare ident ブランチを ParseError から `ImportKind::Package` に変更する。
既存の `import rune "kafka"` / `import "path"` 構文は `ImportKind::Legacy` として共存維持。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `ImportKind` enum 追加（`Package` / `Local` / `Legacy`）、`ImportDecl` に `kind: ImportKind` フィールド追加 |
| `fav/src/frontend/parser.rs` | bare Ident ブランチを `ImportKind::Package` に変更（ParseError を除去）、文字列パスは `ImportKind::Legacy` |
| `fav/src/driver.rs` | `v481000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version → `"48.1.0"` |
| `CHANGELOG.md` | v48.1.0 エントリ追加 |

---

## AST 変更詳細

### `ImportKind` enum（新規）

```rust
/// import 構文の種別（v48.1.0）
#[derive(Debug, Clone, PartialEq)]
pub enum ImportKind {
    /// fav.toml [runes] に宣言されたパッケージ: `import kafka`
    Package,
    /// ./ から始まる相対パス: `import "./src/helpers" as helpers`（v48.2.0 で追加）
    Local,
    /// 従来の文字列パス: `import rune "kafka"` / `import "models/user"`
    Legacy,
}
```

### `ImportDecl` への `kind` フィールド追加

```rust
Item::ImportDecl {
    path: String,
    alias: Option<String>,
    is_rune: bool,
    is_public: bool,
    kind: ImportKind,   // v48.1.0 新規追加
    span: Span,
}
```

---

## Parser 変更詳細

### 変更前（parser.rs 1091〜1108 付近）

bare `Ident`（スラッシュなし）は ParseError を返す:
```
TokenKind::Ident(first_seg) if peek != Slash
  → ParseError("expected string literal import path")
```

### 変更後

bare `Ident` を `ImportKind::Package` として受け入れる:
```
TokenKind::Ident(first_seg) if peek != Slash
  → path = first_seg, kind = ImportKind::Package
```

文字列パス（`import "kafka"` / `import rune "kafka"`）は `ImportKind::Legacy` として継続。

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `import_package_parses` | `import kafka` が `ImportKind::Package`、`path == "kafka"` でパースされる |
| `import_package_with_alias` | `import postgres as db` が `ImportKind::Package`、`alias == Some("db")` でパースされる |

```rust
#[test]
fn import_package_parses() {
    use crate::ast::{Item, ImportKind};
    use crate::frontend::parser::Parser;
    let src = "import kafka\nfn main() -> Bool { true }";
    let program = Parser::parse_str(src, "test.fav").expect("parse");
    let found = program.items.iter().any(|i| matches!(
        i,
        Item::ImportDecl { kind: ImportKind::Package, path, .. } if path == "kafka"
    ));
    assert!(found, "import kafka should parse as ImportKind::Package with path 'kafka'");
}

#[test]
fn import_package_with_alias() {
    use crate::ast::{Item, ImportKind};
    use crate::frontend::parser::Parser;
    let src = "import postgres as db\nfn main() -> Bool { true }";
    let program = Parser::parse_str(src, "test.fav").expect("parse");
    let found = program.items.iter().any(|i| matches!(
        i,
        Item::ImportDecl { kind: ImportKind::Package, alias: Some(a), .. } if a == "db"
    ));
    assert!(found, "import postgres as db should parse as Package with alias 'db'");
}
```

テスト数: 3045 → **3047**（+2）

---

## 注意事項

- `kind` フィールド追加により `ImportDecl` を参照する全 `match` / `if let` に `kind: _` を追記（exhaust 抑制）。
- 既存テスト `parse_simple_import`（`import "models/user"`）は `kind: ImportKind::Legacy` になるよう更新が必要。
- `compiler.rs` / `checker.rs` / `lineage.rs` など `ImportDecl` をパターンマッチしているファイルも `kind: _` 追記が必要。

---

## 完了条件

- `cargo test` 3047 passed, 0 failed（3045 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"48.1.0"`
- `CHANGELOG.md` に v48.1.0 エントリ追加
- `versions/current.md` を v48.1.0（3047 tests）に更新、進行中バージョンを `v48.2.0` に更新
- `tasks.md` を COMPLETE に更新（T0〜T2 全 `[x]`）
