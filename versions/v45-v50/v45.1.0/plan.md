# Plan: v45.1.0 — `return` 構文 AST + parser

---

## Step 0 — 事前確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

期待: `test result: ok. 2966 passed; 0 failed`

---

## Step 1 — `lexer.rs`: `Return` TokenKind 追加

`fav/src/frontend/lexer.rs` の `TokenKind` enum に `Return` variant を追加する。
追加位置: `Yield` の直後あたり（キーワード群のブロック内）。

キーワードマッピングに `"return" => TokenKind::Return` を追加する。
追加位置: `"yield" => TokenKind::Yield,` の直後。

また、`is_keyword` / reserved words リスト（lexer.rs 末尾の配列）があれば `"return"` を追加。

---

## Step 2 — `ast.rs`: `ReturnStmt` + `Stmt::Return` 追加

### 2a. `ReturnStmt` 構造体を追加

`YieldStmt` の定義の直後に追加する:

```rust
// ── ReturnStmt (v45.1.0) ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub expr: Expr,
    pub span: Span,
}
```

### 2b. `Stmt` enum に `Return` variant を追加

```rust
/// `return expr`  early exit from fn/stage body (v45.1.0)
Return(ReturnStmt),
```

追加位置: `Yield(YieldStmt)` の直後。

### 2c. `Stmt::span()` に `Return` アームを追加

`Stmt::Yield(y) => &y.span,` の直後:

```rust
Stmt::Return(r) => &r.span,
```

---

## Step 3 — `parser.rs`: `return` 構文の解析

### 3a. `parse_return_stmt()` 関数を追加

`parse_yield_stmt()` の直後あたりに追加:

```rust
fn parse_return_stmt(&mut self) -> Result<ReturnStmt, ParseError> {
    let start = self.peek_span().clone();
    self.expect(&TokenKind::Return)?;
    let expr = self.parse_expr()?;
    Ok(ReturnStmt {
        expr,
        span: self.span_from(&start),
    })
}
```

### 3b. `parse_block()` に `return` 分岐を追加

`yield` の分岐の直後に追加:

```rust
// return statement (v45.1.0, requires ;)
if self.peek() == &TokenKind::Return {
    let r = self.parse_return_stmt()?;
    stmts.push(Stmt::Return(r));
    if self.peek() == &TokenKind::Semicolon {
        self.advance();
    }
    continue;
}
```

---

## Step 3c — `parse_return_stmt()` の span 構築

`span_from` ヘルパーは既存の `parse_yield_stmt()` でも使われている（parser.rs:2633）ため、
同パターンで実装する。

---

## Step 4 — exhaustive match 対応（ビルド維持）

`Stmt::Return` variant 追加により、以下の各ファイルの `match stmt` 式に
`Stmt::Return(_) => {}` アームを追加する（本バージョンは stub のみ、実処理は v45.2/v45.3）。

対象ファイルと対応方針:

| ファイル | 対応 |
|---|---|
| `fav/src/fmt.rs` | `Stmt::Return(r) => { write!(f, "return {}", r.expr)? }` または stub |
| `fav/src/emit_python.rs` | `Stmt::Return(r) => { /* TODO v45.3 */ }` |
| `fav/src/lineage.rs` | `Stmt::Return(r) => { self.visit_expr(&r.expr); }` （lineage は式を辿る） |
| `fav/src/lint.rs` | `Stmt::Return(r) => { self.check_expr(&r.expr); }` （lint は式を辿る） |
| `fav/src/lsp/references.rs` | `Stmt::Return(r) => { self.visit_expr(&r.expr); }` |
| `fav/src/middle/checker.rs` | `Stmt::Return(_) => { /* TODO v45.2 */ }` |
| `fav/src/middle/compiler.rs` | `Stmt::Return(_) => { /* TODO v45.3 */ }` |

各ファイルは `Stmt::Yield` のアームを参考に追加する。

---

## Step 5 — `driver.rs`: テストモジュール追加 + バージョン更新

### 5a. Cargo.toml: バージョン更新

```toml
version = "45.1.0"
```

### 5b. `v45000_tests::cargo_toml_version_is_45_0_0` をスタブ化

```rust
fn cargo_toml_version_is_45_0_0() {
    // Stubbed: version bumped to 45.1.0 in v45.1.0.
}
```

### 5c. `v451000_tests` モジュールを `v45000_tests` の直前に追加

```rust
// -- v451000_tests (v45.1.0) -- return 構文 AST + parser --
#[cfg(test)]
mod v451000_tests {
    use super::*;

    #[test]
    fn return_stmt_parses() {
        let src = r#"
fn clamp(v: Float, lo: Float, hi: Float) -> Float {
  if v < lo { return lo }
  if v > hi { return hi }
  v
}
"#;
        let prog = parse_program(src).expect("should parse");
        // Find the fn definition and verify body has Return stmts
        let fn_def = prog.items.iter().find_map(|i| {
            if let crate::ast::Item::FnDef(f) = i { Some(f) } else { None }
        }).expect("fn clamp should exist");
        // The block stmts should include Return nodes
        let has_return = fn_def.body.stmts.iter().any(|s| {
            matches!(s, crate::ast::Stmt::Return(_))
        });
        assert!(has_return, "fn body should contain a Return stmt");
    }

    #[test]
    fn single_expr_body_no_return_needed() {
        let src = r#"fn add(a: Int, b: Int) -> Int { a + b }"#;
        let prog = parse_program(src).expect("should parse");
        let fn_def = prog.items.iter().find_map(|i| {
            if let crate::ast::Item::FnDef(f) = i { Some(f) } else { None }
        }).expect("fn add should exist");
        assert!(fn_def.body.stmts.is_empty(), "single-expr body has no stmts");
    }
}
```

---

## Step 6 — ビルド＆テスト

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -20
```

期待: `test result: ok. 2968 passed; 0 failed`

```bash
cargo clippy --locked -D warnings 2>&1 | grep -E "^error" | head -20
```

CHANGELOG.md に v45.1.0 エントリを追加する（`return` 構文 AST + parser）。
