# v36.2.0 spec — `expect` ブロック

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v36.2.0 |
| テーマ | `expect` ブロック（Data Quality ルール宣言構文） |
| 前提 | v36.1.0 COMPLETE — `schema` インライン定義構文実装済み |
| 完了条件 | `v36200_tests` 全テスト pass・`cargo test` 0 failures（≥ 2666 件）|

## 背景と目的

v36.1.0 で `schema Orders { id: Int, ... }` インライン定義を追加した。
本バージョンはデータ品質ルールを宣言的に記述する `expect` ブロック構文を追加する。

**想定構文（ロードマップより）:**
```favnir
fn validate_orders(rows: List<Orders>) -> Result<List<Orders>, String> {
  expect rows {
    not_empty
    all(|r| r.amount >= 0.0)
    no_nulls([.customer_id, .amount])
    unique([.id])
  }
}
```

## 実装スコープ

### 1. `fav/src/ast.rs` — `ExpectStmt` 構造体と `Stmt::Expect` 追加

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

`Stmt` enum に追加:
```rust
/// `expect rows { not_empty; all(|r| r.amount >= 0.0) }` — データ品質ルール（v36.2.0）
Expect(ExpectStmt),
```

`impl Stmt { fn span() }` に追加:
```rust
Stmt::Expect(e) => &e.span,
```

### 2. `fav/src/frontend/parser.rs` — `parse_block` に `expect` 分岐追加

`expect` は専用トークンを持たない（`TokenKind::Ident("expect")`）。
`parse_block` の `forall` 分岐の後に追加:

```rust
// expect <expr> { <rules> }  (v36.2.0)
if matches!(self.peek(), TokenKind::Ident(n) if n == "expect") {
    let e = self.parse_expect_stmt()?;
    stmts.push(Stmt::Expect(e));
    if self.peek() == &TokenKind::Semicolon { self.advance(); }
    continue;
}
```

`parse_expect_stmt` 関数を追加:

> **注意**: `parse_expr()` でターゲット式を読んだ後、直後に `{` を期待する。
> `parse_expr()` は `{` を block-start として解釈しない（Favnir では `{ }` が Record リテラルではないため）ので、
> `rows`・`rows.filter(...)` 等の一般的な式が `{` の前で正しく停止する。

```rust
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

### 3. `Stmt::Expect` を参照する既存 match 文に no-op アーム追加

`Stmt` は `Item` より多くの場所でマッチされる。
`cargo build` でコンパイルエラーを確認し、すべての match 文に:
```rust
Stmt::Expect(_) => {} // v36.2.0 — 実行は v36.3 以降
```
を追加する（返り値型によっては `None` / `unreachable!()` 等で調整）。

主な対象ファイル（`cargo build` エラーで確定）:
- `middle/checker.rs`（`collect_helpers_in_stmt`・`check_stmt`・`scan_expr_for_pipeline_calls` 等）
- `middle/compiler.rs`（`compile_stmt`・`find_bind_in_stmt` 等）
- `middle/ast_lower_checker.rs`（stmt lowering）
- `emit_python.rs`（`emit_stmt` — exhaustive match、wildcard なし）
- `lsp/references.rs`（`collect_in_stmt` — exhaustive match、wildcard なし）
- `lint.rs` / `lineage.rs` / `fmt.rs`（該当 match があれば）

### 4. `fav/src/driver.rs` — テストモジュール

`v36100_tests::cargo_toml_version_is_36_1_0` をスタブ化し、`v36200_tests` を追加。

## v36200_tests の設計

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_36_2_0` | Cargo.toml に `"36.2.0"` が含まれる |
| `changelog_has_v36_2_0` | `CHANGELOG.md` に `[v36.2.0]` が含まれる |
| `expect_stmt_in_ast` | `ast.rs` に `ExpectStmt` かつ `Stmt::Expect` が含まれる |

### parser.rs ラウンドトリップテスト（`#[cfg(test)] mod tests` に追加）

`parse_expect_stmt_basic` — `expect` ブロックがパースエラーなく通ること:

```rust
#[test]
fn parse_expect_stmt_basic() {
    use crate::frontend::parser::parse;
    let src = r#"
fn validate(rows: List<Row>) -> Bool {
    expect rows {
        not_empty
        all(|r| r.ok)
    }
    true
}"#;
    // パースが成功すること（エラーなし）
    let result = parse(src);
    assert!(!result.items.is_empty(), "expect block should parse without error");
}
```

このテストは `parser.rs` 内の `#[cfg(test)] mod tests { ... }` ブロックに追加する。

## ロードマップとの整合

ロードマップ v36.2.0 完了条件:「`expect` ブロックが型チェックと実行を通る / Rust テスト 3 件」

「型チェックと実行を通る」はコンパイルエラーなしかつ `cargo test` 全通過を意味する（no-op アームで対応）。
実際のルール評価（`not_empty` / `all` / `no_nulls` / `unique`）は `fav validate`（v36.4.0）で実装する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `ast.rs` に `ExpectStmt` 構造体と `Stmt::Expect` が含まれる | `expect_stmt_in_ast` テスト |
| 2 | `CHANGELOG.md` に `[v36.2.0]` が含まれる | `changelog_has_v36_2_0` テスト |
| 3 | `Cargo.toml` バージョンが `36.2.0` | `cargo_toml_version_is_36_2_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2666） | `cargo test` 実行結果 |
