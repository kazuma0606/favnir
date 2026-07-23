# Spec: v45.1.0 — `return` 構文 AST + parser

Date: 2026-07-15
Sprint: Language Refinement (v45.1〜v46.0)

---

## 概要

`return <expr>` 構文を Favnir に追加する。AST ノード `ReturnStmt` の定義と
`parser.rs` での解析が本バージョンのスコープ。型チェック・VM 実行は v45.2〜v45.3 で対応。

## 動機

複数行ボディで早期脱出（guard pattern）が書けるようになる。

```favnir
stage ValidateOrder: Order -> Result<Order> = |order| {
  if order.amount <= 0.0 { return Err("invalid amount") }
  Ok(order)
}

fn clamp(v: Float, lo: Float, hi: Float) -> Float {
  if v < lo { return lo }
  if v > hi { return hi }
  v
}
```

## 適用スコープ

| コンテキスト | 許可 |
|---|---|
| `fn` ボディ | ✅ |
| `stage` ボディ | ✅ |
| `seq` パイプライン本体 | ❌（stage の合成であり return の概念がない） |

単一式ボディの暗黙 return は変更なし。`return` は複数行ボディの途中脱出専用。

**seq ボディでの `return` 禁止の実装方針**: パーサーはコンテキスト問わず `Stmt::Return` を生成する。
`seq` ボディ内の `return` は v45.2 の checker.rs (E0415 型チェック) で検出・拒否する。
（パーサーをコンテキスト非依存に保ち、エラー診断を checker に集約する方針）

## 変更ファイル

`Stmt` enum への `Return` variant 追加により、`Stmt` を `match` する全ファイルに
`Stmt::Return(_) => { /* TODO v45.2 */ }` アームの追加が必要（ビルド維持のため）。

| ファイル | 変更内容 |
|---|---|
| `fav/src/frontend/lexer.rs` | `Return` variant を `TokenKind` に追加、`"return"` キーワードマッピング追加 |
| `fav/src/ast.rs` | `ReturnStmt` 構造体追加、`Stmt::Return(ReturnStmt)` variant 追加、`Stmt::span()` に `Return` アーム追加 |
| `fav/src/frontend/parser.rs` | `parse_block()` に `return` 分岐追加、`parse_return_stmt()` 実装 |
| `fav/src/fmt.rs` | `match Stmt` に `Return` アーム追加（stub） |
| `fav/src/emit_python.rs` | `match Stmt` に `Return` アーム追加（stub） |
| `fav/src/lineage.rs` | `match Stmt` に `Return` アーム追加（stub、複数箇所） |
| `fav/src/lint.rs` | `match Stmt` に `Return` アーム追加（stub、複数箇所） |
| `fav/src/lsp/references.rs` | `match Stmt` に `Return` アーム追加（stub） |
| `fav/src/middle/checker.rs` | `match Stmt` に `Return` アーム追加（stub、型チェックは v45.2） |
| `fav/src/middle/compiler.rs` | `match Stmt` に `Return` アーム追加（stub、opcode emit は v45.3） |
| `fav/src/driver.rs` | `v451000_tests` テストモジュール追加（2件）、`v45000_tests::cargo_toml_version_is_45_0_0` をスタブ化 |
| `fav/Cargo.toml` | version `45.0.0` → `45.1.0` |

## 完了条件

- `cargo test` 全通過（**2968 tests** passed, 0 failed）
- `v451000_tests` の 2 件が pass:
  - `return_stmt_parses`
  - `single_expr_body_no_return_needed`
- `cargo clippy --locked -D warnings` クリーン
- `CHANGELOG.md` に v45.1.0 エントリ追加
