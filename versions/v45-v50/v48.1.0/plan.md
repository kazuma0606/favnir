# Plan: v48.1.0 — import 構文刷新 AST + parser（パッケージ）

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `ImportKind` enum 追加・`ImportDecl` に `kind` フィールド追加 |
| `fav/src/frontend/parser.rs` | bare ident import → `ImportKind::Package` + 文字列 import → `ImportKind::Legacy` |
| `fav/src/driver.rs` | `v481000_tests` モジュール追加（2テスト）|
| `fav/Cargo.toml` | version → `"48.1.0"` |
| `CHANGELOG.md` | v48.1.0 エントリ追加 |
| `versions/current.md` | v48.1.0 に更新、進行中 v48.2.0 |
| `versions/v45-v50/v48.1.0/tasks.md` | COMPLETE に更新 |

---

## 変更詳細

### Step 1: `ast.rs` — `ImportKind` 追加 + `ImportDecl` 更新

`ImportDecl` の定義を探し（現在 `is_rune: bool` フィールドがある箇所）、`kind: ImportKind` を追加する。

**`ImportKind` enum 追加（`ImportDecl` の直前に挿入）:**

```rust
/// import 構文の種別（v48.1.0）
#[derive(Debug, Clone, PartialEq)]
pub enum ImportKind {
    /// fav.toml [runes] に宣言されたパッケージ: `import kafka`
    Package,
    /// ./ から始まる相対パス: `import "./src/helpers"` (v48.2.0 以降)
    Local,
    /// 従来の文字列パス: `import rune "kafka"` / `import "models/user"`
    Legacy,
}
```

**`ImportDecl` バリアント更新（`is_public` の後に追加）:**

```rust
    ImportDecl {
        path: String,
        alias: Option<String>,
        is_rune: bool,
        is_public: bool,
        kind: ImportKind,   // ← 追加
        span: Span,
    },
```

### Step 2: `parser.rs` — bare ident → Package、文字列 → Legacy

**変更箇所 1: bare Ident ブランチ（1091〜1108 付近）**

変更前: bare Ident（スラッシュなし）で ParseError を返す
変更後: bare Ident をパッケージ名として受け入れ、`kind = ImportKind::Package` を設定

```rust
TokenKind::Ident(first_seg) => {
    self.advance();
    if self.peek() == &TokenKind::Slash {
        self.advance();
        let (second_seg, _) = self.expect_ident()?;
        if first_seg == "runes" {
            second_seg
        } else {
            format!("{}/{}", first_seg, second_seg)
        }
    } else {
        // v48.1.0: bare ident = Package import
        first_seg
    }
}
```

**変更箇所 2: `kind` フィールド計算ロジック**

`path` を決定する `match` ブランチ内で `kind` も同時に決定する。
`let mut kind = ImportKind::Legacy;` を `match` 前に宣言し、各ブランチで設定する:

- `Str(_)` ブランチ（文字列リテラル）→ `kind = ImportKind::Legacy`（デフォルト、変更不要）
- `Ident` + スラッシュなしブランチ（bare ident）→ `kind = ImportKind::Package`
- `Ident` + スラッシュあり（`runes/X`）→ `kind = ImportKind::Legacy`（デフォルト、変更不要）

```rust
let mut kind = ImportKind::Legacy;
let path = match self.peek().clone() {
    TokenKind::Str(path) => { self.advance(); path }
    TokenKind::Ident(first_seg) => {
        self.advance();
        if self.peek() == &TokenKind::Slash {
            // runes/X → Legacy
            self.advance();
            let (second_seg, _) = self.expect_ident()?;
            if first_seg == "runes" { second_seg } else { format!("{}/{}", first_seg, second_seg) }
        } else {
            // bare ident → Package (v48.1.0)
            kind = ImportKind::Package;
            first_seg
        }
    }
    other => return Err(ParseError::new(
        format!("expected string literal import path, got {:?}", other),
        self.peek_span().clone(),
    )),
};

**変更箇所 3: `ImportDecl` 構築に `kind` を追加**

```rust
Ok(Item::ImportDecl {
    path,
    alias,
    is_rune,
    is_public,
    kind,   // ← 追加
    span: self.span_from(&start),
})
```

### Step 3: `ImportDecl` を参照する全パターンマッチに `kind: _` 追記

`cargo build` のコンパイルエラーを見て対象ファイルを特定し、全 `ImportDecl { .. }` パターンに `kind: _` を追記する。主な対象:
- `parser.rs` のテスト `parse_simple_import`（`is_rune` パターンに `kind: _` 追記）
- `compiler.rs` / `checker.rs` / `middle/compiler.rs` など

### Step 4: `driver.rs` — `v481000_tests` 追加

挿入位置: `v48000_tests` の直前。

```rust
// -- v481000_tests (v48.1.0) -- import 構文刷新: Package import パース確認 --
#[cfg(test)]
mod v481000_tests {
    use crate::ast::{Item, ImportKind};
    use crate::frontend::parser::Parser;

    #[test]
    fn import_package_parses() {
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
        let src = "import postgres as db\nfn main() -> Bool { true }";
        let program = Parser::parse_str(src, "test.fav").expect("parse");
        let found = program.items.iter().any(|i| matches!(
            i,
            Item::ImportDecl { kind: ImportKind::Package, alias: Some(a), .. } if a == "db"
        ));
        assert!(found, "import postgres as db should parse as Package with alias 'db'");
    }
}
```

### Step 5: `Cargo.toml` version → `"48.1.0"`

### Step 6: `CHANGELOG.md` 更新

```markdown
## [v48.1.0] — 2026-07-18 — import 構文刷新 AST + parser（パッケージ）

### Added
- `ast.rs`: `ImportKind` enum 追加（`Package` / `Local` / `Legacy`）
- `ast.rs`: `ImportDecl` に `kind: ImportKind` フィールド追加
- `parser.rs`: bare ident `import kafka` を `ImportKind::Package` として解析
- `driver.rs`: `v481000_tests` 追加（`import_package_parses` / `import_package_with_alias` 2テスト）

### Changed
- `Cargo.toml` version: `48.0.0` → `48.1.0`
```

---

## 実装順序

1. `ast.rs` に `ImportKind` enum 追加 + `ImportDecl` に `kind` フィールド追加
2. `parser.rs` の bare ident ブランチを `ImportKind::Package` 対応に変更
3. `cargo build` でコンパイルエラーを確認し、全パターンマッチに `kind: _` を追記
4. `driver.rs` に `v481000_tests` を `v48000_tests` 直前に追加
5. `Cargo.toml` version → `"48.1.0"`
6. `CHANGELOG.md` v48.1.0 エントリ追加
7. `cargo test` で 3047 passed, 0 failed を確認
8. `cargo clippy -- -D warnings` クリーン確認
9. `versions/current.md` 更新（v48.1.0、次 v48.2.0）
10. `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.1.0 完了条件テスト数（3047）を実績で確認
11. `tasks.md` COMPLETE に更新

---

## 注意事項

- `ImportKind` は `ast.rs` の `ImportDecl` バリアント定義の**直前**に追加する。
- Step 3 は `cargo build 2>&1 | grep "missing field\|error\[E"` で対象を特定してから一括修正する。
- `parse_simple_import` テスト（parser.rs 内）は `kind: _` または `kind: ImportKind::Legacy` を追記して既存テストを修正する。
- `roadmap-v48.1-v49.0.md` の推定テスト数（3042）はベース（3045）が後で判明したため実績は 3047 になる。
