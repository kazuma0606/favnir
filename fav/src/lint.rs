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
        Expr::Question(inner, _) => lint_expr_l008(inner, errors),
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
                    for name in step.stage_names() {
                        if top_level_names.contains(name) {
                            uses.insert(name.to_string());
                        }
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
        Expr::Question(inner, _) => collect_expr_calls(inner, names, uses),
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
        Expr::Question(inner, _) => lint_expr_sub_blocks(inner, errors),
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
        Expr::Question(inner, _) => expr_references(inner, name),
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

// ── W008: ambient effect detection (v13.1.0) ──────────────────────────────────

/// Namespaces whose direct calls constitute ambient effects.
const AMBIENT_NAMESPACES: &[&str] = &[
    "IO", "Postgres", "AWS", "Snowflake", "Http", "Grpc",
    "Llm", "Queue", "Cache", "Slack", "Email",
];

/// Gen functions that have side effects (randomness).
const AMBIENT_GEN_FNS: &[&str] = &["uuid_raw", "uuid_v7_raw", "nano_id"];

/// Standard check (v13.8.0) — detect ambient effect calls and return E0023 errors.
/// Used by standard `fav check` in non-legacy mode. In legacy mode, use `check_ambient_effects`.
pub fn check_ambient_errors(program: &Program) -> Vec<LintError> {
    collect_ambient(program, "E0023")
}

/// `fav check --ambient` — detect ambient effect calls (ctx-less NS.fn(...) calls).
/// Returns W008 warnings. In non-legacy mode use `check_ambient_errors` for E0023.
pub fn check_ambient_effects(program: &Program) -> Vec<LintError> {
    collect_ambient(program, "W008")
}

fn has_ctx_param(fd: &crate::ast::FnDef) -> bool {
    fd.params.first().map(|p| p.name == "ctx").unwrap_or(false)
}

fn collect_ambient(program: &Program, code: &'static str) -> Vec<LintError> {
    let mut errors = Vec::new();
    for item in &program.items {
        match item {
            // E0023 exempts functions that receive a ctx parameter — they use capability-context
            // threading instead of ambient effects, and may call Gen/Snowflake builtins internally.
            Item::FnDef(fd) if code == "E0023" && has_ctx_param(fd) => {}
            Item::FnDef(fd) => collect_ambient_in_block(&fd.body, &mut errors, code),
            Item::TrfDef(td) => collect_ambient_in_block(&td.body, &mut errors, code),
            Item::FlwDef(_) => {}
            Item::ImplDef(id) => {
                for m in &id.methods {
                    collect_ambient_in_block(&m.body, &mut errors, code);
                }
            }
            Item::TestDef(td) => collect_ambient_in_block(&td.body, &mut errors, code),
            _ => {}
        }
    }
    errors
}

fn collect_ambient_in_block(block: &Block, errors: &mut Vec<LintError>, code: &'static str) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Bind(b) => collect_ambient_in_expr(&b.expr, errors, code),
            Stmt::Chain(c) => collect_ambient_in_expr(&c.expr, errors, code),
            Stmt::Expr(e) => collect_ambient_in_expr(e, errors, code),
            Stmt::Yield(y) => collect_ambient_in_expr(&y.expr, errors, code),
            Stmt::ForIn(f) => {
                collect_ambient_in_expr(&f.iter, errors, code);
                collect_ambient_in_block(&f.body, errors, code);
            }
        }
    }
    collect_ambient_in_expr(&block.expr, errors, code);
}

fn collect_ambient_in_expr(expr: &Expr, errors: &mut Vec<LintError>, code: &'static str) {
    match expr {
        // NS.method(args...) — potential ambient call
        Expr::Apply(func, args, span) => {
            if let Expr::FieldAccess(base, method_name, _) = func.as_ref() {
                if let Expr::Ident(ns, _) = base.as_ref() {
                    let is_ambient = AMBIENT_NAMESPACES.contains(&ns.as_str())
                        || (ns == "Gen" && AMBIENT_GEN_FNS.contains(&method_name.as_str()));
                    if is_ambient {
                        errors.push(LintError::new(
                            code,
                            format!(
                                "ambient effect call — `{}.{}` called without ctx argument",
                                ns, method_name
                            ),
                            span.clone(),
                        ));
                    }
                }
            }
            collect_ambient_in_expr(func, errors, code);
            for a in args {
                collect_ambient_in_expr(a, errors, code);
            }
        }
        Expr::Block(b) => collect_ambient_in_block(b, errors, code),
        Expr::If(cond, then, else_, _) => {
            collect_ambient_in_expr(cond, errors, code);
            collect_ambient_in_block(then, errors, code);
            if let Some(eb) = else_ {
                collect_ambient_in_block(eb, errors, code);
            }
        }
        Expr::Match(scrutinee, arms, _) => {
            collect_ambient_in_expr(scrutinee, errors, code);
            for arm in arms {
                collect_ambient_in_expr(&arm.body, errors, code);
            }
        }
        Expr::Pipeline(steps, _) => {
            for s in steps {
                collect_ambient_in_expr(s, errors, code);
            }
        }
        Expr::FieldAccess(obj, _, _) => collect_ambient_in_expr(obj, errors, code),
        Expr::BinOp(_, l, r, _) => {
            collect_ambient_in_expr(l, errors, code);
            collect_ambient_in_expr(r, errors, code);
        }
        Expr::Closure(_, body, _) => collect_ambient_in_expr(body, errors, code),
        Expr::Collect(b, _) => collect_ambient_in_block(b, errors, code),
        Expr::EmitExpr(inner, _) => collect_ambient_in_expr(inner, errors, code),
        Expr::Question(inner, _) => collect_ambient_in_expr(inner, errors, code),
        Expr::AssertMatches(e, _, _) => collect_ambient_in_expr(e, errors, code),
        Expr::TypeApply(f, _, _) => collect_ambient_in_expr(f, errors, code),
        Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields {
                collect_ambient_in_expr(v, errors, code);
            }
        }
        Expr::FString(parts, _) => {
            for part in parts {
                if let FStringPart::Expr(e) = part {
                    collect_ambient_in_expr(e, errors, code);
                }
            }
        }
        Expr::Lit(..) | Expr::Ident(..) => {}
    }
}

/// (namespace, function_name, migration hint) — calls that should use capability ctx instead.
const DEPRECATED_RUNE_CALLS: &[(&str, &str, &str)] = &[
    ("Postgres",  "query_raw",           "ctx.db.query(...)"),
    ("Postgres",  "execute_raw",         "ctx.db.execute(...)"),
    ("AWS",       "s3_get_object_raw",   "ctx.storage.get(...)"),
    ("AWS",       "s3_put_object_raw",   "ctx.storage.put(...)"),
    ("AWS",       "s3_list_objects_raw", "ctx.storage.list(...)"),
    ("AWS",       "s3_delete_object_raw","ctx.storage.delete(...)"),
    ("Snowflake", "query_raw",           "ctx.db.query(...)"),
    ("Snowflake", "execute_raw",         "ctx.db.execute(...)"),
    // v13.3.0: IO / Http direct calls deprecated
    ("IO",        "println",             "ctx.io.println(...)"),
    ("IO",        "print",              "ctx.io.print(...)"),
    ("IO",        "read_line",          "ctx.io.read_line()"),
    ("Http",      "get_raw",            "ctx.http.get(...)"),
    ("Http",      "post_raw",           "ctx.http.post(...)"),
];

/// `fav check --ambient` — detect direct Rune calls that should use capability ctx (W009).
/// NOT part of `lint_program`.
pub fn check_deprecated_rune_calls(program: &Program) -> Vec<LintError> {
    let mut errors = Vec::new();
    for item in &program.items {
        match item {
            Item::FnDef(fd)   => collect_deprecated_in_block(&fd.body, &mut errors),
            Item::TrfDef(td)  => collect_deprecated_in_block(&td.body, &mut errors),
            Item::FlwDef(_)   => {}
            Item::ImplDef(id) => {
                for m in &id.methods {
                    collect_deprecated_in_block(&m.body, &mut errors);
                }
            }
            Item::TestDef(td) => collect_deprecated_in_block(&td.body, &mut errors),
            _ => {}
        }
    }
    errors
}

fn collect_deprecated_in_block(block: &Block, errors: &mut Vec<LintError>) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Bind(b)  => collect_deprecated_in_expr(&b.expr, errors),
            Stmt::Chain(c) => collect_deprecated_in_expr(&c.expr, errors),
            Stmt::Expr(e)  => collect_deprecated_in_expr(e, errors),
            Stmt::Yield(y) => collect_deprecated_in_expr(&y.expr, errors),
            Stmt::ForIn(f) => {
                collect_deprecated_in_expr(&f.iter, errors);
                collect_deprecated_in_block(&f.body, errors);
            }
        }
    }
    collect_deprecated_in_expr(&block.expr, errors);
}

fn collect_deprecated_in_expr(expr: &Expr, errors: &mut Vec<LintError>) {
    match expr {
        Expr::Apply(func, args, span) => {
            if let Expr::FieldAccess(base, method_name, _) = func.as_ref() {
                if let Expr::Ident(ns, _) = base.as_ref() {
                    for (dep_ns, dep_fn, hint) in DEPRECATED_RUNE_CALLS {
                        if ns == dep_ns && method_name == dep_fn {
                            errors.push(LintError::new(
                                "W009",
                                format!(
                                    "direct Rune call `{}.{}` is deprecated — use `{}` instead",
                                    ns, method_name, hint
                                ),
                                span.clone(),
                            ));
                        }
                    }
                }
            }
            collect_deprecated_in_expr(func, errors);
            for a in args {
                collect_deprecated_in_expr(a, errors);
            }
        }
        Expr::Block(b) => collect_deprecated_in_block(b, errors),
        Expr::If(cond, then, else_, _) => {
            collect_deprecated_in_expr(cond, errors);
            collect_deprecated_in_block(then, errors);
            if let Some(eb) = else_ {
                collect_deprecated_in_block(eb, errors);
            }
        }
        Expr::Match(scrutinee, arms, _) => {
            collect_deprecated_in_expr(scrutinee, errors);
            for arm in arms {
                collect_deprecated_in_expr(&arm.body, errors);
            }
        }
        Expr::Pipeline(steps, _) => {
            for s in steps {
                collect_deprecated_in_expr(s, errors);
            }
        }
        Expr::FieldAccess(obj, _, _) => collect_deprecated_in_expr(obj, errors),
        Expr::BinOp(_, l, r, _) => {
            collect_deprecated_in_expr(l, errors);
            collect_deprecated_in_expr(r, errors);
        }
        Expr::Closure(_, body, _) => collect_deprecated_in_expr(body, errors),
        Expr::Collect(b, _) => collect_deprecated_in_block(b, errors),
        Expr::EmitExpr(inner, _) => collect_deprecated_in_expr(inner, errors),
        Expr::Question(inner, _) => collect_deprecated_in_expr(inner, errors),
        Expr::AssertMatches(e, _, _) => collect_deprecated_in_expr(e, errors),
        Expr::TypeApply(f, _, _) => collect_deprecated_in_expr(f, errors),
        Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields {
                collect_deprecated_in_expr(v, errors);
            }
        }
        Expr::FString(parts, _) => {
            for part in parts {
                if let FStringPart::Expr(e) = part {
                    collect_deprecated_in_expr(e, errors);
                }
            }
        }
        Expr::Lit(..) | Expr::Ident(..) => {}
    }
}

// ── E0024: type state mismatch ────────────────────────────────────────────────

const CTX_LIKE_TYPES: &[&str] = &[
    "AppCtx", "LoadCtx", "WriteCtx", "MigrateCtx", "CommonCtx",
];

/// Extract a simple type name (no type args) from a TypeExpr.
fn simple_type_name(ty: &TypeExpr) -> Option<&str> {
    if let TypeExpr::Named(name, args, _) = ty {
        if args.is_empty() {
            return Some(name.as_str());
        }
    }
    None
}

/// Unwrap `Result<T, E>` → T name, or plain `T` → T name.
fn unwrap_output_type_name(ty: &TypeExpr) -> Option<&str> {
    if let TypeExpr::Named(name, args, _) = ty {
        if name == "Result" {
            if let Some(inner) = args.first() {
                return simple_type_name(inner);
            }
        }
        if args.is_empty() {
            return Some(name.as_str());
        }
    }
    None
}

/// Find the "content" parameter type (first non-ctx-like named type).
fn content_param_type_name<'a>(params: &'a [Param]) -> Option<&'a str> {
    for p in params {
        if let Some(name) = simple_type_name(&p.ty) {
            if !CTX_LIKE_TYPES.contains(&name) {
                return Some(name);
            }
        }
    }
    None
}

/// Collect type-state transition functions: fn_name → (input_type, output_type).
/// Only functions where both input and output are declared `type X(...)` in the file.
fn collect_type_state_edges(
    program: &Program,
) -> (
    std::collections::HashMap<String, String>, // fn_name → expected_input
    std::collections::HashMap<String, String>, // fn_name → output_type
    std::collections::HashSet<String>,         // all type state type names
) {
    let type_state_names: std::collections::HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| {
            if let Item::TypeDef(td) = item {
                Some(td.name.clone())
            } else {
                None
            }
        })
        .collect();

    let mut fn_expects = std::collections::HashMap::new();
    let mut fn_output = std::collections::HashMap::new();

    for item in &program.items {
        if let Item::FnDef(fd) = item {
            if let Some(input_name) = content_param_type_name(&fd.params) {
                if type_state_names.contains(input_name) {
                    if let Some(ret_ty) = &fd.return_ty {
                        if let Some(output_name) = unwrap_output_type_name(ret_ty) {
                            if type_state_names.contains(output_name)
                                && input_name != output_name
                            {
                                fn_expects
                                    .insert(fd.name.clone(), input_name.to_string());
                                fn_output
                                    .insert(fd.name.clone(), output_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    (fn_expects, fn_output, type_state_names)
}

/// Returns E0024 errors for type state phase violations in `program`.
pub fn check_type_state_errors(program: &Program) -> Vec<LintError> {
    let (fn_expects, fn_output, type_state_names) = collect_type_state_edges(program);
    if fn_expects.is_empty() {
        return vec![];
    }

    let mut errors = Vec::new();
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            let mut env: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            for p in &fd.params {
                if let Some(name) = simple_type_name(&p.ty) {
                    env.insert(p.name.clone(), name.to_string());
                }
            }
            collect_type_state_in_block(
                &fd.body,
                &fn_expects,
                &fn_output,
                &type_state_names,
                &mut env,
                &mut errors,
            );
        }
    }
    errors
}

fn collect_type_state_in_block(
    block: &Block,
    fn_expects: &std::collections::HashMap<String, String>,
    fn_output: &std::collections::HashMap<String, String>,
    type_state_names: &std::collections::HashSet<String>,
    env: &mut std::collections::HashMap<String, String>,
    errors: &mut Vec<LintError>,
) {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Bind(b) => {
                collect_type_state_in_expr(
                    &b.expr, fn_expects, fn_output, type_state_names, env, errors,
                );
                if let Pattern::Bind(var_name, _) = &b.pattern {
                    if let Some(out_ty) = get_ts_call_output(&b.expr, fn_output) {
                        env.insert(var_name.clone(), out_ty);
                    }
                }
            }
            Stmt::Chain(c) => {
                collect_type_state_in_expr(
                    &c.expr, fn_expects, fn_output, type_state_names, env, errors,
                );
                if let Some(out_ty) = get_ts_call_output(&c.expr, fn_output) {
                    env.insert(c.name.clone(), out_ty);
                }
            }
            Stmt::Expr(e) => {
                collect_type_state_in_expr(e, fn_expects, fn_output, type_state_names, env, errors)
            }
            Stmt::Yield(y) => collect_type_state_in_expr(
                &y.expr, fn_expects, fn_output, type_state_names, env, errors,
            ),
            Stmt::ForIn(f) => {
                collect_type_state_in_expr(
                    &f.iter, fn_expects, fn_output, type_state_names, env, errors,
                );
                collect_type_state_in_block(
                    &f.body, fn_expects, fn_output, type_state_names, env, errors,
                );
            }
        }
    }
    collect_type_state_in_expr(
        &block.expr, fn_expects, fn_output, type_state_names, env, errors,
    );
}

/// If `expr` is `fn_name(args)` and fn_name is a type-state fn, return its output type.
fn get_ts_call_output(
    expr: &Expr,
    fn_output: &std::collections::HashMap<String, String>,
) -> Option<String> {
    if let Expr::Apply(func, _, _) = expr {
        if let Expr::Ident(fn_name, _) = func.as_ref() {
            return fn_output.get(fn_name).cloned();
        }
    }
    None
}

fn collect_type_state_in_expr(
    expr: &Expr,
    fn_expects: &std::collections::HashMap<String, String>,
    fn_output: &std::collections::HashMap<String, String>,
    type_state_names: &std::collections::HashSet<String>,
    env: &mut std::collections::HashMap<String, String>,
    errors: &mut Vec<LintError>,
) {
    match expr {
        Expr::Apply(func, args, span) => {
            if let Expr::Ident(fn_name, _) = func.as_ref() {
                if let Some(expected_ty) = fn_expects.get(fn_name) {
                    // Find the first arg that is an Ident (skip ctx-like args)
                    let first_content_arg = args.iter().find(|a| {
                        if let Expr::Ident(v, _) = a {
                            !CTX_LIKE_TYPES.contains(&v.as_str())
                        } else {
                            false
                        }
                    });
                    if let Some(Expr::Ident(var_name, _)) = first_content_arg {
                        if let Some(actual_ty) = env.get(var_name.as_str()) {
                            if actual_ty != expected_ty
                                && type_state_names.contains(actual_ty.as_str())
                            {
                                errors.push(LintError::new(
                                    "E0024",
                                    format!(
                                        "type state mismatch — `{}` expected `{}`, got `{}`",
                                        fn_name, expected_ty, actual_ty
                                    ),
                                    span.clone(),
                                ));
                            }
                        }
                    }
                }
            }
            collect_type_state_in_expr(func, fn_expects, fn_output, type_state_names, env, errors);
            for a in args {
                collect_type_state_in_expr(a, fn_expects, fn_output, type_state_names, env, errors);
            }
        }
        Expr::Block(b) => {
            collect_type_state_in_block(b, fn_expects, fn_output, type_state_names, env, errors)
        }
        Expr::If(cond, then, else_, _) => {
            collect_type_state_in_expr(cond, fn_expects, fn_output, type_state_names, env, errors);
            collect_type_state_in_block(then, fn_expects, fn_output, type_state_names, env, errors);
            if let Some(eb) = else_ {
                collect_type_state_in_block(
                    eb, fn_expects, fn_output, type_state_names, env, errors,
                );
            }
        }
        Expr::Match(scrutinee, arms, _) => {
            collect_type_state_in_expr(
                scrutinee, fn_expects, fn_output, type_state_names, env, errors,
            );
            for arm in arms {
                collect_type_state_in_expr(
                    &arm.body, fn_expects, fn_output, type_state_names, env, errors,
                );
            }
        }
        Expr::Pipeline(steps, _) => {
            for s in steps {
                collect_type_state_in_expr(
                    s, fn_expects, fn_output, type_state_names, env, errors,
                );
            }
        }
        Expr::FieldAccess(obj, _, _) => {
            collect_type_state_in_expr(obj, fn_expects, fn_output, type_state_names, env, errors)
        }
        Expr::BinOp(_, l, r, _) => {
            collect_type_state_in_expr(l, fn_expects, fn_output, type_state_names, env, errors);
            collect_type_state_in_expr(r, fn_expects, fn_output, type_state_names, env, errors);
        }
        Expr::Closure(_, body, _) => {
            collect_type_state_in_expr(body, fn_expects, fn_output, type_state_names, env, errors)
        }
        Expr::Collect(b, _) => {
            collect_type_state_in_block(b, fn_expects, fn_output, type_state_names, env, errors)
        }
        Expr::EmitExpr(inner, _) => {
            collect_type_state_in_expr(inner, fn_expects, fn_output, type_state_names, env, errors)
        }
        Expr::Question(inner, _) => {
            collect_type_state_in_expr(inner, fn_expects, fn_output, type_state_names, env, errors)
        }
        Expr::AssertMatches(e, _, _) => {
            collect_type_state_in_expr(e, fn_expects, fn_output, type_state_names, env, errors)
        }
        Expr::TypeApply(f, _, _) => {
            collect_type_state_in_expr(f, fn_expects, fn_output, type_state_names, env, errors)
        }
        Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields {
                collect_type_state_in_expr(
                    v, fn_expects, fn_output, type_state_names, env, errors,
                );
            }
        }
        Expr::FString(parts, _) => {
            for part in parts {
                if let FStringPart::Expr(e) = part {
                    collect_type_state_in_expr(
                        e, fn_expects, fn_output, type_state_names, env, errors,
                    );
                }
            }
        }
        Expr::Lit(..) | Expr::Ident(..) => {}
    }
}

// ── E0025: check_bang_notation ────────────────────────────────────────────────

/// Returns a context type hint string for a given set of effects.
/// Conservative: unknown combos → "AppCtx".
fn infer_ctx_hint(effects: &[Effect]) -> &'static str {
    let has_postgres = effects.iter().any(|e| matches!(e, Effect::Postgres | Effect::Db | Effect::DbRead | Effect::DbWrite | Effect::DbAdmin | Effect::Snowflake));
    let has_aws = effects.iter().any(|e| matches!(e, Effect::Unknown(s) if s == "AWS" || s == "S3"));
    let has_io = effects.iter().any(|e| matches!(e, Effect::Io));
    let has_http = effects.iter().any(|e| matches!(e, Effect::Http | Effect::Rpc | Effect::Network));
    let has_llm = effects.iter().any(|e| matches!(e, Effect::Llm));

    // Single-effect cases → specific ctx hint
    if !has_aws && !has_http && !has_llm && has_postgres {
        // Only DB effects
        let db_write = effects.iter().any(|e| matches!(e, Effect::DbWrite | Effect::DbAdmin));
        if db_write {
            return "WriteCtx";
        }
        return "LoadCtx";
    }
    if !has_postgres && !has_aws && !has_http && !has_llm && has_io {
        return "CommonCtx";
    }
    // Multiple effects or unknown → AppCtx
    "AppCtx"
}

/// Returns E0025 errors for functions that still use `!Effect` notation (non-legacy mode).
pub fn check_bang_notation(program: &Program) -> Vec<LintError> {
    let mut errors = Vec::new();
    for item in &program.items {
        match item {
            Item::FnDef(fd) => {
                let meaningful: Vec<&Effect> = fd.effects.iter()
                    .filter(|e| !matches!(e, Effect::Pure))
                    .collect();
                if !meaningful.is_empty() {
                    let effects_str: Vec<String> = meaningful.iter().map(|e| format!("!{}", effect_name(e))).collect();
                    let ctx_hint = infer_ctx_hint(&fd.effects);
                    errors.push(LintError::new(
                        "E0025",
                        format!(
                            "`!` effect notation is no longer supported: {}. Migrate to `fn {}(ctx: {}, ...) -> ...`",
                            effects_str.join(" "),
                            fd.name,
                            ctx_hint
                        ),
                        fd.span.clone(),
                    ));
                }
            }
            Item::TrfDef(td) => {
                let meaningful: Vec<&Effect> = td.effects.iter()
                    .filter(|e| !matches!(e, Effect::Pure))
                    .collect();
                if !meaningful.is_empty() {
                    let effects_str: Vec<String> = meaningful.iter().map(|e| format!("!{}", effect_name(e))).collect();
                    let ctx_hint = infer_ctx_hint(&td.effects);
                    errors.push(LintError::new(
                        "E0025",
                        format!(
                            "`!` effect notation is no longer supported: {}. Migrate stage `{}` to use capability context `{}`",
                            effects_str.join(" "),
                            td.name,
                            ctx_hint
                        ),
                        td.span.clone(),
                    ));
                }
            }
            _ => {}
        }
    }
    errors
}

fn effect_name(e: &Effect) -> &str {
    match e {
        Effect::Pure => "Pure",
        Effect::Io => "Io",
        Effect::Db => "Db",
        Effect::DbRead => "DbRead",
        Effect::DbWrite => "DbWrite",
        Effect::DbAdmin => "DbAdmin",
        Effect::Network => "Network",
        Effect::Http => "Http",
        Effect::Llm => "Llm",
        Effect::Snowflake => "Snowflake",
        Effect::Postgres => "Postgres",
        Effect::Rpc => "Rpc",
        Effect::File => "File",
        Effect::Checkpoint => "Checkpoint",
        Effect::Trace => "Trace",
        Effect::Emit(_) => "Emit",
        Effect::EmitUnion(_) => "EmitUnion",
        Effect::Unknown(s) => s.as_str(),
    }
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
