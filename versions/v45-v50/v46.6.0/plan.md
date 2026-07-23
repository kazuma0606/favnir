# Plan: v46.6.0 — `fav explain` 2.0 Phase 1

Date: 2026-07-17

---

## ステップ

### Step 1 — `driver.rs`: `scan_returns` + `render_pipeline_mermaid_v2` 追加

`render_graph_mermaid_with_opts` の近辺（`sanitize_mermaid_id` の前後）に追加。

**`scan_returns`**:

```rust
/// fn ボディの stmts を走査して (has_any_return, has_err_return) を返す。
/// Stmt::Return の expr が Apply(Ident("Err"), ...) なら has_err_return = true。
fn scan_returns(stmts: &[crate::ast::Stmt]) -> (bool, bool) {
    let mut has_any = false;
    let mut has_err = false;
    for stmt in stmts {
        if let crate::ast::Stmt::Return(r) = stmt {
            has_any = true;
            if is_err_call(&r.expr) {
                has_err = true;
            }
        }
    }
    (has_any, has_err)
}

/// Expr::Apply(Expr::Ident("Err", _), _, _) パターンを判定
fn is_err_call(expr: &crate::ast::Expr) -> bool {
    if let crate::ast::Expr::Apply(func, _, _) = expr {
        if let crate::ast::Expr::Ident(name, _) = func.as_ref() {
            return name == "Err";
        }
    }
    false
}
```

**`render_pipeline_mermaid_v2`**:

```rust
/// v46.6.0: `fav explain` 2.0 — return dead path + Err error path。
/// pub(crate): v466000_tests から直接呼び出すため。
/// コマンド統合（fav explain --format mermaid への差し替え）は v46.7.0 以降。
pub(crate) fn render_pipeline_mermaid_v2(program: &crate::ast::Program) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(out, "flowchart LR");
    let _ = writeln!(out, "    classDef deadPath stroke-dasharray: 5 5");
    let _ = writeln!(out, "    classDef errPath fill:#ffcccc,stroke:#cc0000");

    for item in &program.items {
        if let crate::ast::Item::FnDef(fd) = item {
            if fd.name.starts_with('$') { continue; }
            let fn_id = sanitize_mermaid_id(&fd.name);
            let _ = writeln!(out, "    {}[\"fn {}\"]", fn_id, fd.name);

            let (has_ret, has_err) = scan_returns(&fd.body.stmts);
            if has_ret {
                let ret_id = format!("{}_dead_return", fn_id);
                let label = if has_err { "return Err" } else { "return" };
                let _ = writeln!(out, "    {}([\"{}\"])", ret_id, label);
                let _ = writeln!(out, "    {} -.-> {}", fn_id, ret_id);
                let class = if has_err { "errPath" } else { "deadPath" };
                let _ = writeln!(out, "    class {} {}", ret_id, class);
            }
        }
    }
    out
}
```

---

### Step 2 — `driver.rs`: `v466000_tests`

`v465000_tests` の後（`v455000_tests` の前）に追加:

```rust
mod v466000_tests {
    use crate::frontend::parser::Parser;
    use super::render_pipeline_mermaid_v2;

    #[test]
    fn explain_mermaid_includes_dead_path() {
        // return を含む fn → dead path (dotted -.->)
        let src = "fn process(x: Int) -> Int { return x }";
        let program = Parser::parse_str(src, "test.fav").expect("parse");
        let rendered = render_pipeline_mermaid_v2(&program);
        assert!(
            rendered.contains("-.->"),
            "expected dotted arrow for return path, got:\n{}",
            rendered
        );
        assert!(
            rendered.contains("deadPath"),
            "expected deadPath class, got:\n{}",
            rendered
        );
    }

    #[test]
    fn explain_pipeline_v2() {
        // return Err(...) を含む fn → errPath (赤)
        let src = "fn validate(x: Int) -> Int { return Err(x) }";
        let program = Parser::parse_str(src, "test.fav").expect("parse");
        let rendered = render_pipeline_mermaid_v2(&program);
        assert!(
            rendered.contains("flowchart"),
            "expected flowchart header"
        );
        assert!(
            rendered.contains("errPath"),
            "expected errPath class for Err return, got:\n{}",
            rendered
        );
        assert!(
            rendered.contains("-.->"),
            "expected dotted arrow for error path, got:\n{}",
            rendered
        );
    }
}
```

---

### Step 3 — バージョン更新

- `fav/Cargo.toml`: `46.6.0`
- `CHANGELOG.md`: v46.6.0 エントリ
- `versions/current.md`: v46.6.0（3005 tests）
