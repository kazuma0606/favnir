# v17.4.0 — `let` バインディング タスク

## ステータス: 未着手

---

## タスク一覧

### T1: Lexer — `TokenKind::Let` 追加

- [ ] `fav/src/frontend/lexer.rs` の `TokenKind` enum に `Let` variant を追加
- [ ] `keyword_or_ident`（またはキーワードマッチ関数）に `"let" => TokenKind::Let` を追加
- [ ] 既存テストが壊れないことを確認（`let` を識別子として使っているケースがないか grep）

### T2: AST — `Stmt::Let` 追加

- [ ] `fav/src/ast.rs` の `Stmt` enum に `Let { name: String, expr: Box<Expr>, span: Span }` を追加
- [ ] `Stmt::span()` メソッドに `Stmt::Let { span, .. } => span` を追加
- [ ] `Stmt::Bind` の定義の直後に配置（意味的に近い位置）

### T3: パーサー — `let name = expr` 解析

- [ ] `fav/src/frontend/parser.rs` の `parse_stmt` に `TokenKind::Let` ブランチを追加
  ```
  advance()            // 'let' 消費
  expect_ident()       // 変数名
  expect(Eq)           // '='
  parse_expr()         // 右辺
  → Stmt::Let { name, expr, span }
  ```
- [ ] `bind x <- expr` と `let x = expr` の構文を並べてパーサーで正しく分岐できることを確認

### T4: 型チェッカー — E0326 チェック

- [ ] `fav/src/middle/checker.rs` の `check_stmt` に `Stmt::Let` を追加
  - `expr` の型を推論
  - `Type::Result(_, _)` なら E0326 を発出
  - そうでなければ `name` を推論型でスコープに追加
- [ ] E0326 エラーメッセージ: `"let cannot be used with Result values — use 'bind name <- expr' instead"`

### T5: コンパイラ — `Stmt::Let` の StoreLocal

- [ ] `fav/src/middle/compiler.rs` の `compile_stmt` に `Stmt::Let` を追加
  - `compile_expr(expr, ctx)` で右辺をコンパイル
  - `ctx.next_slot` でスロット確保
  - `ctx.locals.insert(name, slot)`
  - Result チェック opcode（`LegacyBindCheck` / `SeqStageCheck`）は不要
- [ ] `Stmt::Bind` との違いが明確になるようコメントを添える

### T6: Exhaustive match 対応

- [ ] `fav/src/fmt.rs` — `Stmt::Let { name, expr, .. }` ブランチ追加
  ```rust
  Stmt::Let { name, expr, .. } => format!("let {} = {}", name, fmt_expr(expr))
  ```
- [ ] `fav/src/emit_python.rs` — `Stmt::Let { name, expr, .. }` ブランチ追加
  ```rust
  Stmt::Let { name, expr, .. } => format!("{} = {}", name, emit_expr(expr))
  ```
- [ ] `fav/src/lineage.rs` — `Stmt::Let { expr, .. }` で子ノードを再帰処理
- [ ] `fav/src/lint.rs` — `Stmt::Let { name, expr, .. }` で lint チェック（未使用変数等）
- [ ] `fav/src/driver.rs` — `format_stmt_compact` 等の match に追加

### T7: Self-hosted 対応（`self/compiler.fav` / `self/checker.fav`）

- [ ] `self/compiler.fav` の `parse_stmt` 相当処理に `"let"` トークンのケースを追加
  - `parse_ident` → `expect "="` → `parse_expr` → `Stmt.Let(name, expr)` を生成
- [ ] `self/checker.fav` の `check_stmt` 相当処理に `Stmt.Let` ケースを追加
  - `check_expr(ctx, expr)` → 型が Result なら E0326 → そうでなければ `define(ctx, name, ty)`

### T8: テスト（`fav/src/driver.rs`）

- [ ] `v174000_tests` モジュールを `driver.rs` に追加

```rust
#[cfg(test)]
mod v174000_tests {
    use super::{build_artifact, exec_artifact_main, check_single_file};
    use crate::frontend::parser::Parser;
    use crate::value::Value;

    fn run(src: &str) -> Value {
        let program = Parser::parse_str(src, "v174000_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        exec_artifact_main(&artifact, None).expect("exec")
    }

    #[test]
    fn version_is_17_4_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"17.4.0\""), "Cargo.toml should have version 17.4.0");
    }

    #[test]
    fn let_binding_basic() {
        // let x = 42 で束縛し、加算に使える
        let result = run(r#"
fn main() -> Int {
    let x = 42
    let y = 8
    x + y
}
"#);
        assert_eq!(result, Value::Int(50));
    }

    #[test]
    fn let_binding_string() {
        // let name = String.trim(s) が動作する
        let result = run(r#"
fn main() -> Int {
    let name = String.trim("  hello  ")
    String.length(name)
}
"#);
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn let_with_bind_mix() {
        // let と bind の混在が動作する
        let result = run(r#"
fn safe_add(a: Int, b: Int) -> Result<Int, String> {
    Result.ok(a + b)
}
fn main() -> Int {
    let x = 10
    let y = 20
    bind sum <- safe_add(x, y)
    sum
}
"#);
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn let_with_list_comp() {
        // let で list comprehension 結果を束縛
        let result = run(r#"
fn main() -> Int {
    bind ns <- Result.ok(List.push(List.push(List.singleton(1), 2), 3))
    let doubled = [x * 2 | x <- ns]
    List.length(doubled)
}
"#);
        assert_eq!(result, Value::Int(3));
    }
}
```

- [ ] `let_result_type_error` は E0326 チェッカーテストとして別途追加（checker 経由で確認）
- [ ] `cargo test v174000` — 4+/5 PASS（E0326 テストは checker テストとして）
- [ ] `cargo test` — リグレッションなし

### T9: ドキュメント

- [ ] `site/content/docs/language/let-binding.mdx` を新規作成
  - `let` の用途・使用例
  - `bind` との使い分け表
  - E0326 の説明

### T10: バージョン更新

- [ ] `fav/Cargo.toml` のバージョンを `17.4.0` に更新
- [ ] `fav/Cargo.lock` を `cargo build` で更新

---

## 完了条件チェックリスト

- [ ] `let x = 42` で変数を束縛し、後続の式で使えること
- [ ] `let name = String.trim(s)` のような stdlib 呼び出しが動作すること
- [ ] `let` の後に `bind` を続けて混在できること
- [ ] `let` で list comprehension 結果を束縛できること
- [ ] `let r = result_fn()` で E0326 が発出されること
- [ ] `cargo test v174000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし

---

## 優先度

T1（Lexer）→ T2（AST）→ T3（Parser）→ T4（Checker）→ T5（Compiler）→ T6（exhaustive match）→ T7（self-hosted）→ T8（テスト）→ T9（doc）→ T10（version）

T6 は T2 直後に行うと clippy -D warnings を早期に解消できる。
T7（self-hosted）は後回し可能（既存の `fav run` が Rust checker にフォールバックするため）。
