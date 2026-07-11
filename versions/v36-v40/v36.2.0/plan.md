# v36.2.0 実装計画 — `expect` ブロック

## 実装順序

| ステップ | 対象 | 内容 |
|---|---|---|
| S1 | `CHANGELOG.md` | `## [v36.2.0]` エントリを追加（`## [v36.1.0]` の直後） |
| S2 | `fav/src/ast.rs` | `ExpectStmt` 構造体 + `Stmt::Expect` + `span()` アーム追加 |
| S3 | `fav/src/frontend/parser.rs` | `parse_expect_stmt` 追加 + `parse_block` に分岐追加 |
| S4 | 各ファイルの match 文 | `Stmt::Expect(_) => {}` no-op アームを追加してコンパイルエラー解消 |
| S5 | `fav/src/driver.rs` | `v36100_tests::cargo_toml_version_is_36_1_0` をスタブ化 |
| S6 | `fav/src/driver.rs` | `v36200_tests` モジュール（3 件）を追加 |
| S7 | `fav/Cargo.toml` | バージョンを `36.1.0` → `36.2.0` に更新（必ず **S4・S5・S6 すべて完了後**） |
| S8 | `cargo test` | 全通過確認（≥ 2665 件） |

## 各ステップの詳細

### S1: CHANGELOG.md

`## [v36.1.0]` の `---` セパレータの直後（`## [35.3.0]` の前）に挿入（実装当日の日付を記入）:

```markdown
## [v36.2.0] — 2026-07-08

### Added
- `expect <target> { <rules> }` ブロック構文（Data Quality ルール宣言）
- `ast::ExpectStmt` 構造体・`Stmt::Expect` variant（v36.2.0）
- `parse_expect_stmt` — expect ブロックパーサー

---
```

### S2: ast.rs — ExpectStmt 追加

`SchemaDef` 定義（v36.1.0）の後に追加:

```rust
// ── ExpectStmt (v36.2.0) ─────────────────────────────────────────────────────

/// `expect <target> { <rules> }` — データ品質ルール宣言（v36.2.0）
#[derive(Debug, Clone)]
pub struct ExpectStmt {
    pub target: Box<Expr>,
    pub rules: Vec<Expr>,
    pub span: Span,
}
```

`Stmt` enum の末尾（`Forall` の後）に追加:
```rust
/// `expect rows { not_empty; all(|r| r.amount >= 0.0) }` — データ品質ルール（v36.2.0）
Expect(ExpectStmt),
```

`impl Stmt` の `span()` match に追加:
```rust
Stmt::Expect(e) => &e.span,
```

### S3: parser.rs — parse_expect_stmt と parse_block 分岐追加

`parse_block` の forall 分岐の後（行 2363 付近）に追加:
```rust
// expect <expr> { <rules> }  (v36.2.0)
if matches!(self.peek(), TokenKind::Ident(n) if n == "expect") {
    let e = self.parse_expect_stmt()?;
    stmts.push(Stmt::Expect(e));
    if self.peek() == &TokenKind::Semicolon { self.advance(); }
    continue;
}
```

`parse_expect_stmt` 関数（`parse_schema_def` の後などに配置）:
```rust
/// Parse `expect <expr> { <rule_expr>* }` (v36.2.0)
fn parse_expect_stmt(&mut self) -> Result<ExpectStmt, ParseError> {
    let start = self.peek_span().clone();
    self.advance(); // consume `expect`
    let target = self.parse_expr()?;
    self.expect(&TokenKind::LBrace)?;
    let mut rules = vec![];
    while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
        let rule = self.parse_expr()?;
        rules.push(rule);
        if self.peek() == &TokenKind::Semicolon { self.advance(); }
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(ExpectStmt {
        target: Box::new(target),
        rules,
        span: self.span_from(&start),
    })
}
```

### S4: match 文への no-op アーム追加

`cargo build 2>&1 | grep "error\[E0004\]"` で対象を特定し、各 match 文に追加する。
`Stmt` は `Item` より多くの場所でマッチされるため、対象ファイルが多い。

主な対象（build エラー確認後に確定）:
- `middle/checker.rs` — `collect_helpers_in_stmt`・`check_stmt`・`scan_expr_for_pipeline_calls` 等
- `middle/compiler.rs` — `compile_stmt`・`find_bind_in_stmt` 等
- `middle/ast_lower_checker.rs` — stmt lowering
- `lint.rs` — stmt lint チェック
- `lineage.rs` — lineage 収集
- `fmt.rs` — フォーマット

no-op アームの形式は返り値型によって異なる:
```rust
Stmt::Expect(_) => {}          // Unit を返す場所
Stmt::Expect(_) => None,       // Option<T> を返す場所
Stmt::Expect(_) => { /* noop */ }  // block を必要とする場所
```

### S5: driver.rs — v36100_tests スタブ化

```rust
fn cargo_toml_version_is_36_1_0() {
    // stubbed: version bumped to 36.2.0
}
```

### S6: driver.rs — v36200_tests モジュール追加

`v36100_tests` の閉じ `}` の後（ファイル末尾）に追加:

```rust
// ── v36200_tests (v36.2.0) — expect ブロック ─────────────────────────────────
#[cfg(test)]
mod v36200_tests {
    #[test]
    fn cargo_toml_version_is_36_2_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("36.2.0"), "Cargo.toml must contain version 36.2.0");
    }
    #[test]
    fn changelog_has_v36_2_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v36.2.0]"), "CHANGELOG.md must contain [v36.2.0]");
    }
    #[test]
    fn expect_stmt_in_ast() {
        let src = include_str!("ast.rs");
        assert!(
            src.contains("ExpectStmt")
                && (src.contains("Stmt::Expect") || src.contains("Expect(ExpectStmt)")),
            "ast.rs must contain ExpectStmt struct and Stmt::Expect variant"
        );
    }
}
```

### S7: Cargo.toml バージョン更新

**必ず S4（コンパイルエラー解消）・S5（スタブ化）・S6（v36200_tests 追加）すべて完了後に実行すること**。

`version = "36.1.0"` → `version = "36.2.0"`

### S8: cargo test

期待値: 2662（現在）+ 3（v36200_tests）+ 1（parse_expect_stmt_basic）= **2666 件** pass、0 failures
