// src/lint.rs — Favnir static linter (Phase 5, v0.8.0)
//
// Entry point: `lint_program(prog: &Program) -> Vec<LintError>`
//
// Lint codes:
//   L001  pub fn is missing an explicit return type
//   L002  unused bind binding (name not referenced in subsequent stmts/expr)
//   L003  fn name is not snake_case
//   L004  type name is not PascalCase

use crate::ast::*;
use crate::frontend::lexer::Span;

// ── LintError ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LintError {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

impl LintError {
    fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        LintError { code, message: message.into(), span }
    }
}

impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "lint[{}]: {}\n  --> {}:{}:{}",
            self.code, self.message,
            self.span.file, self.span.line, self.span.col
        )
    }
}

// ── public API ────────────────────────────────────────────────────────────────

pub fn lint_program(program: &Program) -> Vec<LintError> {
    let mut errors = Vec::new();
    for item in &program.items {
        match item {
            Item::FnDef(fd)   => lint_fn_def(fd, &mut errors),
            Item::TrfDef(_)   => {}
            Item::TypeDef(td) => lint_type_def(td, &mut errors),
            Item::InterfaceDecl(_) | Item::InterfaceImplDecl(_) => {}
            Item::ImplDef(id) => {
                for m in &id.methods { lint_fn_def(m, &mut errors); }
            }
            Item::TestDef(td) => lint_block_unused_binds(&td.body, &mut errors),
            _ => {}
        }
    }
    errors
}

// ── L001: pub fn missing return type ─────────────────────────────────────────
// ── L003: fn name not snake_case      ─────────────────────────────────────────

fn lint_fn_def(fd: &FnDef, errors: &mut Vec<LintError>) {
    // L001: public fn must have explicit non-Unit return type annotation
    // We check by seeing if the return type is Named("Unit") with no params
    if fd.visibility == Some(Visibility::Public) {
        if let TypeExpr::Named(name, args, _) = &fd.return_ty {
            if name == "Unit" && args.is_empty() && fd.params.is_empty() {
                // Unit return with no params is allowed for main-like fns;
                // only flag when there are params (non-trivial function)
                // Actually L001 fires when there's no return type specified.
                // Since the parser always requires a return type, we can't
                // distinguish "omitted" from "explicitly Unit". Skip this for now.
            }
        }
        // L001: fire if return type is Named("_infer") (omitted / placeholder)
        if let TypeExpr::Named(name, _, _) = &fd.return_ty {
            if name == "_infer" {
                errors.push(LintError::new(
                    "L001",
                    format!("pub fn `{}` is missing an explicit return type", fd.name),
                    fd.span.clone(),
                ));
            }
        }
    }

    // L003: fn name must be snake_case
    if !is_snake_case(&fd.name) {
        errors.push(LintError::new(
            "L003",
            format!("fn name `{}` should be snake_case", fd.name),
            fd.span.clone(),
        ));
    }

    // Check body for unused bindings
    lint_block_unused_binds(&fd.body, errors);
}

// ── L004: type name not PascalCase ────────────────────────────────────────────

fn lint_type_def(td: &TypeDef, errors: &mut Vec<LintError>) {
    if !is_pascal_case(&td.name) {
        errors.push(LintError::new(
            "L004",
            format!("type name `{}` should be PascalCase", td.name),
            td.span.clone(),
        ));
    }
}

// ── L002: unused bind binding ─────────────────────────────────────────────────

fn lint_block_unused_binds(block: &Block, errors: &mut Vec<LintError>) {
    // Collect all bind names and check if they appear in subsequent stmts/final expr
    let stmts = &block.stmts;
    for (i, stmt) in stmts.iter().enumerate() {
        if let Stmt::Bind(b) = stmt {
            if let Pattern::Bind(name, span) = &b.pattern {
                if name == "_" { continue; } // underscore intentionally ignored
                // Check if `name` is referenced in stmts[i+1..] or block.expr
                let used = stmts[i + 1..].iter().any(|s| stmt_references(s, name))
                    || expr_references(&block.expr, name);
                if !used {
                    errors.push(LintError::new(
                        "L002",
                        format!("binding `{}` is never used", name),
                        span.clone(),
                    ));
                }
            }
        }
    }

    // Recurse into sub-blocks
    for stmt in stmts {
        lint_stmt_sub_blocks(stmt, errors);
    }
    lint_expr_sub_blocks(&block.expr, errors);
}

fn lint_stmt_sub_blocks(stmt: &Stmt, errors: &mut Vec<LintError>) {
    match stmt {
        Stmt::Bind(b) => lint_expr_sub_blocks(&b.expr, errors),
        Stmt::Expr(e) => lint_expr_sub_blocks(e, errors),
        Stmt::Chain(c) => lint_expr_sub_blocks(&c.expr, errors),
        Stmt::Yield(y) => lint_expr_sub_blocks(&y.expr, errors),
    }
}

fn lint_expr_sub_blocks(expr: &Expr, errors: &mut Vec<LintError>) {
    match expr {
        Expr::Block(b) => lint_block_unused_binds(b, errors),
        Expr::If(_, then, else_, _) => {
            lint_block_unused_binds(then, errors);
            if let Some(eb) = else_ { lint_block_unused_binds(eb, errors); }
        }
        Expr::Match(scrutinee, arms, _) => {
            lint_expr_sub_blocks(scrutinee, errors);
            for arm in arms { lint_expr_sub_blocks(&arm.body, errors); }
        }
        Expr::Apply(f, args, _) => {
            lint_expr_sub_blocks(f, errors);
            for a in args { lint_expr_sub_blocks(a, errors); }
        }
        Expr::Pipeline(steps, _) => {
            for s in steps { lint_expr_sub_blocks(s, errors); }
        }
        Expr::FieldAccess(obj, _, _) => lint_expr_sub_blocks(obj, errors),
        Expr::BinOp(_, l, r, _) => {
            lint_expr_sub_blocks(l, errors);
            lint_expr_sub_blocks(r, errors);
        }
        Expr::Closure(_, body, _) => lint_expr_sub_blocks(body, errors),
        Expr::Collect(b, _) => lint_block_unused_binds(b, errors),
        Expr::EmitExpr(inner, _) => lint_expr_sub_blocks(inner, errors),
        Expr::RecordConstruct(_, fields, _) => {
            for (_, e) in fields { lint_expr_sub_blocks(e, errors); }
        }
        Expr::Lit(..) | Expr::Ident(..) => {}
    }
}

// ── reference checking (does expression mention a name?) ─────────────────────

fn expr_references(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Ident(n, _) => n == name,
        Expr::Apply(f, args, _) => {
            expr_references(f, name) || args.iter().any(|a| expr_references(a, name))
        }
        Expr::Pipeline(steps, _) => steps.iter().any(|s| expr_references(s, name)),
        Expr::FieldAccess(obj, _, _) => expr_references(obj, name),
        Expr::BinOp(_, l, r, _) => expr_references(l, name) || expr_references(r, name),
        Expr::Block(b) => block_references(b, name),
        Expr::Match(s, arms, _) => {
            expr_references(s, name)
                || arms.iter().any(|arm| {
                    arm.guard.as_ref().map_or(false, |g| expr_references(g, name))
                        || expr_references(&arm.body, name)
                })
        }
        Expr::If(c, t, e, _) => {
            expr_references(c, name)
                || block_references(t, name)
                || e.as_ref().map_or(false, |eb| block_references(eb, name))
        }
        Expr::Closure(params, body, _) => {
            // If the closure re-binds the name, it shadows it — don't count
            if params.iter().any(|p| p == name) {
                false
            } else {
                expr_references(body, name)
            }
        }
        Expr::Collect(b, _) => block_references(b, name),
        Expr::EmitExpr(inner, _) => expr_references(inner, name),
        Expr::RecordConstruct(_, fields, _) => fields.iter().any(|(_, e)| expr_references(e, name)),
        Expr::Lit(..) => false,
    }
}

fn block_references(block: &Block, name: &str) -> bool {
    block.stmts.iter().any(|s| stmt_references(s, name))
        || expr_references(&block.expr, name)
}

fn stmt_references(stmt: &Stmt, name: &str) -> bool {
    match stmt {
        Stmt::Bind(b) => {
            // The binding introduces a new name; only the RHS can reference `name`
            // (the pattern itself is a definition, not a reference)
            expr_references(&b.expr, name)
        }
        Stmt::Expr(e)    => expr_references(e, name),
        Stmt::Chain(c)   => expr_references(&c.expr, name),
        Stmt::Yield(y)   => expr_references(&y.expr, name),
    }
}

// ── naming conventions ────────────────────────────────────────────────────────

fn is_snake_case(name: &str) -> bool {
    if name.is_empty() { return true; }
    // snake_case: lowercase letters, digits, underscores; must not start with digit
    // Allow leading underscore for "intentionally unused" convention
    name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !name.starts_with(|c: char| c.is_ascii_digit())
}

fn is_pascal_case(name: &str) -> bool {
    if name.is_empty() { return true; }
    // PascalCase: starts with uppercase, rest alphanumeric (no underscores)
    let mut chars = name.chars();
    chars.next().map_or(false, |c| c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_alphanumeric())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{is_pascal_case, is_snake_case, lint_program};
    use crate::frontend::parser::Parser;

    fn lint(src: &str) -> Vec<String> {
        let prog = Parser::parse_str(src, "lint_test.fav").expect("parse");
        lint_program(&prog).into_iter().map(|e| e.code.to_string()).collect()
    }

    #[test]
    fn lint_snake_case_ok() {
        assert!(is_snake_case("foo_bar"));
        assert!(is_snake_case("main"));
        assert!(is_snake_case("_unused"));
        assert!(is_snake_case("fn2"));
    }

    #[test]
    fn lint_snake_case_fail() {
        assert!(!is_snake_case("FooBar"));
        assert!(!is_snake_case("fooBar"));
        assert!(!is_snake_case("Foo"));
    }

    #[test]
    fn lint_pascal_case_ok() {
        assert!(is_pascal_case("FooBar"));
        assert!(is_pascal_case("User"));
        assert!(is_pascal_case("Direction"));
    }

    #[test]
    fn lint_pascal_case_fail() {
        assert!(!is_pascal_case("foo_bar"));
        assert!(!is_pascal_case("fooBar"));
        assert!(!is_pascal_case("direction"));
    }

    #[test]
    fn lint_l003_non_snake_fn() {
        let codes = lint(r#"
fn FooBar(x: Int) -> Int { x }
"#);
        assert!(codes.contains(&"L003".to_string()), "expected L003, got {:?}", codes);
    }

    #[test]
    fn lint_l004_non_pascal_type() {
        let codes = lint(r#"
type direction = | North | South
"#);
        assert!(codes.contains(&"L004".to_string()), "expected L004, got {:?}", codes);
    }

    #[test]
    fn lint_l002_unused_bind() {
        let codes = lint(r#"
fn foo() -> Int {
    bind x <- 42
    1
}
"#);
        assert!(codes.contains(&"L002".to_string()), "expected L002, got {:?}", codes);
    }

    #[test]
    fn lint_l002_used_bind_no_error() {
        let codes = lint(r#"
fn foo() -> Int {
    bind x <- 42
    x
}
"#);
        assert!(!codes.contains(&"L002".to_string()), "unexpected L002, got {:?}", codes);
    }

    #[test]
    fn lint_clean_file_no_errors() {
        let codes = lint(r#"
fn add(a: Int, b: Int) -> Int {
    a + b
}

public fn main() -> Unit !Io {
    IO.println("hello")
}
"#);
        assert!(codes.is_empty(), "expected no lint errors, got {:?}", codes);
    }
}
