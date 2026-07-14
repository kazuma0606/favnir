# v43.12.0 Plan — W031〜W033 lint

## 前提

- 現行バージョン: `43.11.0`（2932 tests）
- 追加テスト数: 3 件
- 目標テスト数: 2935

---

## 重要な AST 確認事項

- `Block.expr` は `Box<Expr>`（非 Option）— `*fd.body.expr` で参照する
- `Expr::Var` は存在しない — 変数参照は `Expr::Ident(String, Span)` を使用
- W031〜W033 は既存パターン通り **lint.rs にインライン定義**（error_catalog.rs への追加なし）

---

## ステップ

### Step 1: lint.rs — check_w031 追加

`lint_program()` に呼び出しを追加し、ヘルパー関数を定義:

```rust
fn check_w031(program: &Program, warnings: &mut Vec<LintError>) {
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            if fd.return_ty.is_some() && fd.body.stmts.is_empty() {
                let is_simple = matches!(
                    &*fd.body.expr,
                    Expr::Lit(_, _) | Expr::Ident(_, _)
                );
                if is_simple {
                    warnings.push(LintError {
                        code: "W031",
                        message: "return type annotation is redundant; type can be inferred"
                            .to_string(),
                        span: fd.span.clone(),
                    });
                }
            }
        }
    }
}
```

`lint_program()` の末尾に追加:

```rust
check_w031(program, &mut warnings);
check_w032(program, &mut warnings);
// W033: 将来版（AST 拡張後に実装 — Expr::Closure はパラメータ型を保持しない）
```

### Step 2: lint.rs — check_w032 追加

```rust
fn check_w032(program: &Program, warnings: &mut Vec<LintError>) {
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            // stmts を走査
            for stmt in &fd.body.stmts {
                check_w032_in_stmt(stmt, warnings);
            }
            // 末尾式を走査
            check_w032_in_expr(&fd.body.expr, warnings);
        }
    }
}

fn check_w032_in_expr(expr: &Expr, warnings: &mut Vec<LintError>) {
    match expr {
        Expr::TypeApply(_, _, span) => {
            warnings.push(LintError {
                code: "W032",
                message: "explicit generic type argument is redundant; type can be inferred from argument"
                    .to_string(),
                span: span.clone(),
            });
        }
        // 再帰ケース（必要に応じて追加）
        Expr::Call(f, args, _) => {
            check_w032_in_expr(f, warnings);
            for arg in args {
                check_w032_in_expr(arg, warnings);
            }
        }
        Expr::Block(b, _) => {
            for stmt in &b.stmts {
                check_w032_in_stmt(stmt, warnings);
            }
            check_w032_in_expr(&b.expr, warnings);
        }
        _ => {}
    }
}

fn check_w032_in_stmt(stmt: &Stmt, warnings: &mut Vec<LintError>) {
    match stmt {
        Stmt::Bind(_, expr, _) => check_w032_in_expr(expr, warnings),
        Stmt::Expr(expr, _) => check_w032_in_expr(expr, warnings),
        _ => {}
    }
}
```

### Step 3: driver.rs — v431200_tests 追加 / スタブ化 / Cargo.toml

`v431100_tests` の直前に挿入:

```rust
mod v431200_tests {
    use crate::frontend::parser::Parser;

    #[test]
    fn cargo_toml_version_is_43_12_0() {
        let toml = include_str!("../Cargo.toml");
        assert!(toml.contains("version = \"43.12.0\""), "Cargo.toml version mismatch");
    }

    #[test]
    fn w031_warns_on_redundant_return_annotation() {
        let src = "fn answer() -> Int { 42 }";
        let prog = Parser::parse_str(src, "v431200_test.fav").expect("parse");
        let warnings = crate::lint::lint_program(&prog);
        assert!(
            warnings.iter().any(|w| w.code == "W031"),
            "expected W031 warning, got: {:?}", warnings
        );
    }

    #[test]
    fn w032_warns_on_explicit_generic_type_arg() {
        // identity::<Int>(42) — TypeApply ノードが生成される
        let src = "fn f() { identity::<Int>(42) }";
        let prog = Parser::parse_str(src, "v431200_test.fav").expect("parse");
        let warnings = crate::lint::lint_program(&prog);
        assert!(
            warnings.iter().any(|w| w.code == "W032"),
            "expected W032 warning, got: {:?}", warnings
        );
    }
}
```

スタブ化（`v431100_tests::cargo_toml_version_is_43_11_0`）:

```rust
// Stubbed: version bumped to 43.12.0 in v43.12.0.
```

### Step 4: Cargo.toml version bump 43.11.0 → 43.12.0

### Step 5: CHANGELOG.md に v43.12.0 エントリ追加

### Step 6: テスト実行（2935 passed; 0 failed）

### Step 7: バージョン管理ドキュメント更新

---

## 注意事項

- W031〜W033 は **lint.rs にインライン定義**（error_catalog.rs への追加なし）
- `fd.body.expr` は `Box<Expr>` — `&*fd.body.expr` でデリファレンスして `&Expr` として渡す
- `Expr::Var` は存在しない — `Expr::Ident(String, Span)` を使うこと
- `check_w032_in_expr` / `check_w032_in_stmt` の Stmt バリアント名は ast.rs で確認してから実装すること
- `Stmt::Bind` / `Stmt::Expr` 等の正確なバリアント名は ast.rs で要確認
