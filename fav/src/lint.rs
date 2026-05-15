// src/lint.rs — Favnir static linter (Phase 5, v0.8.0)
//
// Entry point: `lint_program(prog: &Program) -> Vec<LintError>`
//
// Lint codes:
//   L001  pub fn is missing an explicit return type
//   L002  unused bind binding (name not referenced in subsequent stmts/expr)
//   L003  fn name is not snake_case
//   L004  type name is not PascalCase
//   L005  unused private trf/flw-like top-level item
//   L006  trf name is not PascalCase
//   L007  effect name is not PascalCase
//   L008  hardcoded db credential in DB.connect string literal

use crate::ast::*;
use crate::frontend::lexer::Span;
use std::collections::HashSet;

// ── LintError ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LintError {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

impl LintError {
    fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        LintError {
            code,
            message: message.into(),
            span,
        }
    }
}

impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "lint[{}]: {}\n  --> {}:{}:{}",
            self.code, self.message, self.span.file, self.span.line, self.span.col
        )
    }
}

// ── public API ────────────────────────────────────────────────────────────────

pub fn lint_program(program: &Program) -> Vec<LintError> {
    let mut errors = Vec::new();
    let uses = collect_trf_flw_uses(program);
    for item in &program.items {
        match item {
            Item::FnDef(fd) => {
                lint_fn_def(fd, &mut errors);
                lint_block_l008(&fd.body, &mut errors);
            }
            Item::TrfDef(td) => lint_trf_def(td, &uses, &mut errors),
            Item::TypeDef(td) => lint_type_def(td, &mut errors),
            Item::InterfaceDecl(_) | Item::InterfaceImplDecl(_) => {}
            Item::EffectDef(ed) => lint_effect_def(ed, &mut errors),
            Item::FlwDef(fd) => lint_flw_like(None, &fd.name, &fd.span, &uses, &mut errors),
            Item::AbstractTrfDef(td) => lint_flw_like(
                td.visibility.as_ref(),
                &td.name,
                &td.span,
                &uses,
                &mut errors,
            ),
            Item::AbstractFlwDef(fd) => lint_flw_like(
                fd.visibility.as_ref(),
                &fd.name,
                &fd.span,
                &uses,
                &mut errors,
            ),
            Item::FlwBindingDef(fd) => lint_flw_like(
                fd.visibility.as_ref(),
                &fd.name,
                &fd.span,
                &uses,
                &mut errors,
            ),
            Item::ImplDef(id) => {
                for m in &id.methods {
                    lint_fn_def(m, &mut errors);
                    lint_block_l008(&m.body, &mut errors);
                }
            }
            Item::TestDef(td) => lint_block_unused_binds(&td.body, &mut errors),
            Item::BenchDef(bd) => lint_block_unused_binds(&bd.body, &mut errors),
            Item::ImportDecl { .. } => {}
            _ => {}
        }
    }
    errors
}

fn lint_trf_def(td: &TrfDef, uses: &HashSet<String>, errors: &mut Vec<LintError>) {
    lint_flw_like(td.visibility.as_ref(), &td.name, &td.span, uses, errors);
    if !is_pascal_case(&td.name) {
        errors.push(LintError::new(
            "L006",
            format!("stage name `{}` should be PascalCase", td.name),
            td.span.clone(),
        ));
    }
}

fn lint_effect_def(ed: &EffectDef, errors: &mut Vec<LintError>) {
    if !is_pascal_case(&ed.name) {
        errors.push(LintError::new(
            "L007",
            format!("effect name `{}` should be PascalCase", ed.name),
            ed.span.clone(),
        ));
    }
}

// ── L008: hardcoded db credential ────────────────────────────────────────────

fn lint_block_l008(block: &Block, errors: &mut Vec<LintError>) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Bind(b) => lint_expr_l008(&b.expr, errors),
            Stmt::Expr(e) => lint_expr_l008(e, errors),
            Stmt::Chain(c) => lint_expr_l008(&c.expr, errors),
            Stmt::Yield(y) => lint_expr_l008(&y.expr, errors),
            Stmt::ForIn(f) => {
                lint_expr_l008(&f.iter, errors);
                lint_block_l008(&f.body, errors);
            }
        }
    }
    lint_expr_l008(&block.expr, errors);
}

fn lint_expr_l008(expr: &Expr, errors: &mut Vec<LintError>) {
    match expr {
        Expr::Apply(callee, args, span) => {
            // Detect DB.connect("postgres://user:pass@host/db")
            if let Expr::FieldAccess(obj, method, _) = callee.as_ref() {
                if let Expr::Ident(ns, _) = obj.as_ref() {
                    if (ns == "DB" || ns == "db") && method == "connect" {
                        if let Some(first_arg) = args.first() {
                            if let Expr::Lit(Lit::Str(s), _) = first_arg {
                                if s.contains("://") && s.contains('@') {
                                    errors.push(LintError::new(
                                        "L008",
                                        "hardcoded db credential: connection string contains password; use Env.get(\"DB_URL\") instead".to_string(),
                                        span.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            lint_expr_l008(callee, errors);
            for arg in args {
                lint_expr_l008(arg, errors);
            }
        }
        Expr::TypeApply(callee, _, _) => lint_expr_l008(callee, errors),
        Expr::Pipeline(steps, _) => {
            for step in steps {
                lint_expr_l008(step, errors);
            }
        }
        Expr::FieldAccess(obj, _, _) => lint_expr_l008(obj, errors),
        Expr::BinOp(_, left, right, _) => {
            lint_expr_l008(left, errors);
            lint_expr_l008(right, errors);
        }
        Expr::Block(block) => lint_block_l008(block, errors),
        Expr::Match(scrutinee, arms, _) => {
            lint_expr_l008(scrutinee, errors);
            for arm in arms {
                lint_expr_l008(&arm.body, errors);
            }
        }
        Expr::If(cond, then_block, else_block, _) => {
            lint_expr_l008(cond, errors);
            lint_block_l008(then_block, errors);
            if let Some(eb) = else_block {
                lint_block_l008(eb, errors);
            }
        }
        Expr::Closure(_, body, _) => lint_expr_l008(body, errors),
        Expr::Collect(block, _) => lint_block_l008(block, errors),
        Expr::EmitExpr(inner, _) => lint_expr_l008(inner, errors),
        Expr::RecordConstruct(_, fields, _) => {
            for (_, e) in fields {
                lint_expr_l008(e, errors);
            }
        }
        Expr::FString(parts, _) => {
            for part in parts {
                if let FStringPart::Expr(e) = part {
                    lint_expr_l008(e, errors);
                }
            }
        }
        Expr::AssertMatches(e, _, _) => lint_expr_l008(e, errors),
        Expr::Ident(..) | Expr::Lit(..) => {}
    }
}

fn lint_flw_like(
    visibility: Option<&Visibility>,
    name: &str,
    span: &Span,
    uses: &HashSet<String>,
    errors: &mut Vec<LintError>,
) {
    if visibility.is_none() && !uses.contains(name) {
        errors.push(LintError::new(
            "L005",
            format!("private item `{}` is never used", name),
            span.clone(),
        ));
    }
}

fn collect_trf_flw_uses(program: &Program) -> HashSet<String> {
    let top_level_names: HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::TrfDef(td) => Some(td.name.clone()),
            Item::AbstractTrfDef(td) => Some(td.name.clone()),
            Item::FlwDef(fd) => Some(fd.name.clone()),
            Item::AbstractFlwDef(fd) => Some(fd.name.clone()),
            Item::FlwBindingDef(fd) => Some(fd.name.clone()),
            _ => None,
        })
        .collect();

    let mut uses = HashSet::new();
    for item in &program.items {
        match item {
            Item::FnDef(fd) => collect_block_calls(&fd.body, &top_level_names, &mut uses),
            Item::TrfDef(td) => collect_block_calls(&td.body, &top_level_names, &mut uses),
            Item::FlwDef(fd) => {
                for step in &fd.steps {
                    if top_level_names.contains(step) {
                        uses.insert(step.clone());
                    }
                }
            }
            Item::FlwBindingDef(fd) => {
                if top_level_names.contains(&fd.template) {
                    uses.insert(fd.template.clone());
                }
                for (_, imp) in &fd.bindings {
                    let name = match imp {
                        SlotImpl::Global(name) | SlotImpl::Local(name) => name,
                    };
                    if top_level_names.contains(name) {
                        uses.insert(name.clone());
                    }
                }
            }
            Item::ImplDef(id) => {
                for m in &id.methods {
                    collect_block_calls(&m.body, &top_level_names, &mut uses);
                }
            }
            Item::TestDef(td) => collect_block_calls(&td.body, &top_level_names, &mut uses),
            Item::BenchDef(bd) => collect_block_calls(&bd.body, &top_level_names, &mut uses),
            Item::InterfaceImplDecl(id) => {
                for (_, expr) in &id.methods {
                    collect_expr_calls(expr, &top_level_names, &mut uses);
                }
            }
            _ => {}
        }
    }
    uses
}

fn collect_block_calls(block: &Block, names: &HashSet<String>, uses: &mut HashSet<String>) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Bind(b) => collect_expr_calls(&b.expr, names, uses),
            Stmt::Expr(e) => collect_expr_calls(e, names, uses),
            Stmt::Chain(c) => collect_expr_calls(&c.expr, names, uses),
            Stmt::Yield(y) => collect_expr_calls(&y.expr, names, uses),
            Stmt::ForIn(f) => {
                collect_expr_calls(&f.iter, names, uses);
                collect_block_calls(&f.body, names, uses);
            }
        }
    }
    collect_expr_calls(&block.expr, names, uses);
}

fn collect_expr_calls(expr: &Expr, names: &HashSet<String>, uses: &mut HashSet<String>) {
    match expr {
        Expr::Ident(name, _) => {
            if names.contains(name) {
                uses.insert(name.clone());
            }
        }
        Expr::Apply(callee, args, _) => {
            collect_expr_calls(callee, names, uses);
            for arg in args {
                collect_expr_calls(arg, names, uses);
            }
        }
        Expr::TypeApply(callee, _, _) => collect_expr_calls(callee, names, uses),
        Expr::Pipeline(steps, _) => {
            for step in steps {
                collect_expr_calls(step, names, uses);
            }
        }
        Expr::FieldAccess(obj, _, _) => collect_expr_calls(obj, names, uses),
        Expr::BinOp(_, left, right, _) => {
            collect_expr_calls(left, names, uses);
            collect_expr_calls(right, names, uses);
        }
        Expr::Block(block) => collect_block_calls(block, names, uses),
        Expr::Match(scrutinee, arms, _) => {
            collect_expr_calls(scrutinee, names, uses);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_expr_calls(guard, names, uses);
                }
                collect_expr_calls(&arm.body, names, uses);
            }
        }
        Expr::AssertMatches(expr, _, _) => collect_expr_calls(expr, names, uses),
        Expr::If(cond, then_block, else_block, _) => {
            collect_expr_calls(cond, names, uses);
            collect_block_calls(then_block, names, uses);
            if let Some(else_block) = else_block {
                collect_block_calls(else_block, names, uses);
            }
        }
        Expr::Closure(_, body, _) => collect_expr_calls(body, names, uses),
        Expr::Collect(block, _) => collect_block_calls(block, names, uses),
        Expr::EmitExpr(inner, _) => collect_expr_calls(inner, names, uses),
        Expr::RecordConstruct(_, fields, _) => {
            for (_, expr) in fields {
                collect_expr_calls(expr, names, uses);
            }
        }
        Expr::FString(parts, _) => {
            for part in parts {
                if let FStringPart::Expr(expr) = part {
                    collect_expr_calls(expr, names, uses);
                }
            }
        }
        Expr::Lit(..) => {}
    }
}

// ── L001: pub fn missing return type ─────────────────────────────────────────
// ── L003: fn name not snake_case      ─────────────────────────────────────────

fn lint_fn_def(fd: &FnDef, errors: &mut Vec<LintError>) {
    // L001: public fn must have explicit non-Unit return type annotation
    // We check by seeing if the return type is Named("Unit") with no params
    if fd.visibility == Some(Visibility::Public) {
        if fd.return_ty.is_none() {
            errors.push(LintError::new(
                "L001",
                format!("pub fn `{}` is missing an explicit return type", fd.name),
                fd.span.clone(),
            ));
        }
        if let Some(TypeExpr::Named(name, args, _)) = &fd.return_ty {
            if name == "Unit" && args.is_empty() && fd.params.is_empty() {
                // Unit return with no params is allowed for main-like fns;
                // only flag when there are params (non-trivial function)
                // Actually L001 fires when there's no return type specified.
                // Since the parser always requires a return type, we can't
                // distinguish "omitted" from "explicitly Unit". Skip this for now.
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
                if name == "_" {
                    continue;
                } // underscore intentionally ignored
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
        Stmt::ForIn(f) => {
            lint_expr_sub_blocks(&f.iter, errors);
            lint_block_unused_binds(&f.body, errors);
        }
    }
}

fn lint_expr_sub_blocks(expr: &Expr, errors: &mut Vec<LintError>) {
    match expr {
        Expr::Block(b) => lint_block_unused_binds(b, errors),
        Expr::If(_, then, else_, _) => {
            lint_block_unused_binds(then, errors);
            if let Some(eb) = else_ {
                lint_block_unused_binds(eb, errors);
            }
        }
        Expr::Match(scrutinee, arms, _) => {
            lint_expr_sub_blocks(scrutinee, errors);
            for arm in arms {
                lint_expr_sub_blocks(&arm.body, errors);
            }
        }
        Expr::AssertMatches(expr, _, _) => lint_expr_sub_blocks(expr, errors),
        Expr::Apply(f, args, _) => {
            lint_expr_sub_blocks(f, errors);
            for a in args {
                lint_expr_sub_blocks(a, errors);
            }
        }
        Expr::TypeApply(f, _, _) => lint_expr_sub_blocks(f, errors),
        Expr::Pipeline(steps, _) => {
            for s in steps {
                lint_expr_sub_blocks(s, errors);
            }
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
            for (_, e) in fields {
                lint_expr_sub_blocks(e, errors);
            }
        }
        Expr::FString(parts, _) => {
            for part in parts {
                if let FStringPart::Expr(expr) = part {
                    lint_expr_sub_blocks(expr, errors);
                }
            }
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
        Expr::TypeApply(f, _, _) => expr_references(f, name),
        Expr::Pipeline(steps, _) => steps.iter().any(|s| expr_references(s, name)),
        Expr::FieldAccess(obj, _, _) => expr_references(obj, name),
        Expr::BinOp(_, l, r, _) => expr_references(l, name) || expr_references(r, name),
        Expr::Block(b) => block_references(b, name),
        Expr::Match(s, arms, _) => {
            expr_references(s, name)
                || arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .map_or(false, |g| expr_references(g, name))
                        || expr_references(&arm.body, name)
                })
        }
        Expr::AssertMatches(expr, _, _) => expr_references(expr, name),
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
        Expr::FString(parts, _) => parts.iter().any(|part| match part {
            FStringPart::Lit(_) => false,
            FStringPart::Expr(expr) => expr_references(expr, name),
        }),
        Expr::Lit(..) => false,
    }
}

fn block_references(block: &Block, name: &str) -> bool {
    block.stmts.iter().any(|s| stmt_references(s, name)) || expr_references(&block.expr, name)
}

fn stmt_references(stmt: &Stmt, name: &str) -> bool {
    match stmt {
        Stmt::Bind(b) => {
            // The binding introduces a new name; only the RHS can reference `name`
            // (the pattern itself is a definition, not a reference)
            expr_references(&b.expr, name)
        }
        Stmt::Expr(e) => expr_references(e, name),
        Stmt::Chain(c) => expr_references(&c.expr, name),
        Stmt::Yield(y) => expr_references(&y.expr, name),
        Stmt::ForIn(f) => expr_references(&f.iter, name) || block_references(&f.body, name),
    }
}

// ── naming conventions ────────────────────────────────────────────────────────

fn is_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    // snake_case: lowercase letters, digits, underscores; must not start with digit
    // Allow leading underscore for "intentionally unused" convention
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !name.starts_with(|c: char| c.is_ascii_digit())
}

fn is_pascal_case(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
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
        lint_program(&prog)
            .into_iter()
            .map(|e| e.code.to_string())
            .collect()
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
        let codes = lint(
            r#"
fn FooBar(x: Int) -> Int { x }
"#,
        );
        assert!(
            codes.contains(&"L003".to_string()),
            "expected L003, got {:?}",
            codes
        );
    }

    #[test]
    fn lint_l004_non_pascal_type() {
        let codes = lint(
            r#"
type direction = | North | South
"#,
        );
        assert!(
            codes.contains(&"L004".to_string()),
            "expected L004, got {:?}",
            codes
        );
    }

    #[test]
    fn lint_l005_unused_trf() {
        let codes = lint(
            r#"
stage ParseCsv: String -> Int = |s| { 1 }
public fn main() -> Int { 0 }
"#,
        );
        assert!(
            codes.contains(&"L005".to_string()),
            "expected L005, got {:?}",
            codes
        );
    }

    #[test]
    fn lint_l005_public_trf_ignored() {
        let codes = lint(
            r#"
public stage ParseCsv: String -> Int = |s| { 1 }
public fn main() -> Int { 0 }
"#,
        );
        assert!(
            !codes.contains(&"L005".to_string()),
            "unexpected L005: {:?}",
            codes
        );
    }

    #[test]
    fn lint_l005_unused_flw() {
        let codes = lint(
            r#"
stage ParseCsv: String -> Int = |s| { 1 }
seq ImportUsers = ParseCsv
public fn main() -> Int { 0 }
"#,
        );
        assert!(
            codes.contains(&"L005".to_string()),
            "expected L005, got {:?}",
            codes
        );
    }

    #[test]
    fn lint_l005_used_trf_no_warning() {
        let codes = lint(
            r#"
stage ParseCsv: String -> Int = |s| { 1 }
public fn main() -> Int { ParseCsv("x") }
"#,
        );
        assert!(
            !codes.contains(&"L005".to_string()),
            "unexpected L005: {:?}",
            codes
        );
    }

    #[test]
    fn lint_l006_trf_not_pascal() {
        let codes = lint(
            r#"
stage parse_csv: String -> Int = |s| { 1 }
public fn main() -> Int { 0 }
"#,
        );
        assert!(
            codes.contains(&"L006".to_string()),
            "expected L006, got {:?}",
            codes
        );
    }

    #[test]
    fn lint_l006_trf_pascal_ok() {
        let codes = lint(
            r#"
stage ParseCsv: String -> Int = |s| { 1 }
public fn main() -> Int { ParseCsv("x") }
"#,
        );
        assert!(
            !codes.contains(&"L006".to_string()),
            "unexpected L006: {:?}",
            codes
        );
    }

    #[test]
    fn lint_l007_effect_not_pascal() {
        let codes = lint(
            r#"
effect payment
public fn main() -> Unit !payment { () }
"#,
        );
        assert!(
            codes.contains(&"L007".to_string()),
            "expected L007, got {:?}",
            codes
        );
    }

    #[test]
    fn lint_l002_unused_bind() {
        let codes = lint(
            r#"
fn foo() -> Int {
    bind x <- 42
    1
}
"#,
        );
        assert!(
            codes.contains(&"L002".to_string()),
            "expected L002, got {:?}",
            codes
        );
    }

    #[test]
    fn lint_l002_used_bind_no_error() {
        let codes = lint(
            r#"
fn foo() -> Int {
    bind x <- 42
    x
}
"#,
        );
        assert!(
            !codes.contains(&"L002".to_string()),
            "unexpected L002, got {:?}",
            codes
        );
    }

    #[test]
    fn lint_clean_file_no_errors() {
        let codes = lint(
            r#"
fn add(a: Int, b: Int) -> Int {
    a + b
}

public fn main() -> Unit !Io {
    IO.println("hello")
}
"#,
        );
        assert!(codes.is_empty(), "expected no lint errors, got {:?}", codes);
    }

    #[test]
    fn lint_l008_postgres_url_with_password() {
        let codes = lint(
            r#"
public fn main() -> Unit {
    bind _ <- DB.connect("postgres://user:secret@localhost:5432/mydb")
}
"#,
        );
        assert!(
            codes.contains(&"L008".to_string()),
            "expected L008, got {:?}",
            codes
        );
    }

    #[test]
    fn lint_l008_sqlite_no_warning() {
        let codes = lint(
            r#"
public fn main() -> Unit {
    bind _ <- DB.connect("sqlite::memory:")
}
"#,
        );
        assert!(
            !codes.contains(&"L008".to_string()),
            "unexpected L008: {:?}",
            codes
        );
    }
}
