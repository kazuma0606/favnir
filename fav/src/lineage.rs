/// Static lineage analysis for Favnir programs.
///
/// Extracted from driver.rs (v7.1.0) into a standalone module so that
/// both the `fav_core` library crate and the `fav` binary crate can
/// include `backend/vm.rs` without a circular dependency on `driver.rs`.
///
/// This module depends only on `crate::ast` — no VM, no I/O.
use serde::Serialize;

use crate::ast;

// ── structs ──────────────────────────────────────────────────────────────────

/// Per-stage/fn entry in a lineage report.
#[derive(Debug, Clone, Serialize)]
pub struct LineageEntry {
    pub name: String,
    /// Capability-based classification: "read" | "write" | "transform" | "sink" | "io"
    /// (v13.9.0+; previously "stage" | "fn")
    pub kind: String,
    /// The primary capability interface of this entry, e.g. "DbRead", "DbWrite", "StorageWrite"
    pub capability: Option<String>,
    pub effects: Vec<String>,
    pub sources: Vec<String>, // tables read
    pub sinks: Vec<String>,   // tables written
    /// v46.7.0: true = トップレベルに Stmt::Return が存在する（早期脱出パスあり）。
    /// 注意: dead code の厳密な検出ではなく「early-return を持つ」フラグ。
    /// `if cond { return x } y` のように分岐後にコードがある場合も true になる。
    pub is_dead: bool,
    /// v52.3.0: first assert_schema<T> type name found in body, if any.
    pub schema: Option<String>,
}

/// A seq pipeline chain.
#[derive(Debug, Clone, Serialize)]
pub struct PipelineLineage {
    pub name: String,
    pub steps: Vec<String>,
    pub sources: Vec<String>,
    pub sinks: Vec<String>,
}

/// Full lineage report for a file.
#[derive(Debug, Clone, Serialize, Default)]
pub struct LineageReport {
    pub transformations: Vec<LineageEntry>,
    pub pipelines: Vec<PipelineLineage>,
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// v46.7.0: fn/stage ボディのトップレベル stmts に Stmt::Return が存在するかを判定。
/// Phase 1 スコープ: ネストした if/match/for 内は対象外。
fn has_early_return(stmts: &[ast::Stmt]) -> bool {
    stmts.iter().any(|s| matches!(s, ast::Stmt::Return(_)))
}

/// Classify a function/stage by capability kind based on parameter types.
/// Returns (kind, capability): e.g. ("read", Some("DbRead")), ("transform", None).
fn classify_capability_kind(
    params: &[ast::Param],
) -> (String, Option<String>) {
    // Check parameter type names for ctx-based capabilities (v13.x design)
    for p in params {
        if let ast::TypeExpr::Named(name, _, _) = &p.ty {
            match name.as_str() {
                "DbWrite" | "WriteCtx" | "MigrateCtx" => {
                    return ("write".into(), Some("DbWrite".into()))
                }
                "StorageWrite" => return ("sink".into(), Some("StorageWrite".into())),
                "DbRead" | "LoadCtx" => return ("read".into(), Some("DbRead".into())),
                "AppCtx" => return ("read".into(), Some("DbRead".into())),
                "Io" | "CommonCtx" => return ("io".into(), Some("Io".into())),
                _ => {}
            }
        }
    }
    ("transform".into(), None)
}

fn strip_sql_ident(s: &str) -> String {
    let s = s.trim_matches(|c: char| c == ',' || c == '(' || c == ')' || c == ';');
    let s = s.trim_matches(|c: char| c == '"' || c == '`' || c == '\'');
    s.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect()
}

// ── SQL literal extraction ────────────────────────────────────────────────────

/// Extract read-tables (FROM / JOIN) and write-tables (INSERT INTO / UPDATE / DELETE FROM)
/// from a SQL string literal using simple regex-free pattern matching.
pub fn extract_tables_from_sql(sql: &str) -> (Vec<String>, Vec<String>) {
    let upper = sql.to_uppercase();
    let tokens: Vec<&str> = upper.split_whitespace().collect();
    let mut reads: Vec<String> = Vec::new();
    let mut writes: Vec<String> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            "FROM" | "JOIN" => {
                if let Some(t) = tokens.get(i + 1) {
                    let name = strip_sql_ident(t);
                    if !name.is_empty() && !reads.contains(&name) {
                        reads.push(name);
                    }
                }
                i += 1;
            }
            "INSERT" => {
                if tokens.get(i + 1) == Some(&"INTO") {
                    if let Some(t) = tokens.get(i + 2) {
                        let name = strip_sql_ident(t);
                        if !name.is_empty() && !writes.contains(&name) {
                            writes.push(name);
                        }
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "UPDATE" => {
                if let Some(t) = tokens.get(i + 1) {
                    let name = strip_sql_ident(t);
                    if !name.is_empty() && !writes.contains(&name) {
                        writes.push(name);
                    }
                }
                i += 1;
            }
            "DELETE" => {
                if tokens.get(i + 1) == Some(&"FROM") {
                    if let Some(t) = tokens.get(i + 2) {
                        let name = strip_sql_ident(t);
                        if !name.is_empty() && !writes.contains(&name) {
                            writes.push(name);
                        }
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    (reads, writes)
}

/// Recursively walk an expression and collect string literal arguments
/// that appear as the first arg to DB call expressions (`DB.*`).
pub fn collect_sql_literals(expr: &ast::Expr) -> Vec<String> {
    let mut result = Vec::new();
    collect_sql_literals_inner(expr, &mut result);
    result
}

fn collect_sql_literals_inner(expr: &ast::Expr, out: &mut Vec<String>) {
    match expr {
        ast::Expr::Apply(func, args, _) => {
            let is_db_call = match func.as_ref() {
                ast::Expr::FieldAccess(obj, _, _) => {
                    matches!(obj.as_ref(), ast::Expr::Ident(name, _) if name == "DB" || name == "Db")
                }
                _ => false,
            };
            if is_db_call {
                if let Some(ast::Expr::Lit(ast::Lit::Str(sql), _)) = args.first() {
                    out.push(sql.clone());
                }
            }
            for a in args {
                collect_sql_literals_inner(a, out);
            }
            collect_sql_literals_inner(func, out);
        }
        ast::Expr::Pipeline(exprs, _) => {
            for e in exprs {
                collect_sql_literals_inner(e, out);
            }
        }
        ast::Expr::Block(block) => {
            for s in &block.stmts {
                collect_sql_literals_stmt(s, out);
            }
            collect_sql_literals_inner(&block.expr, out);
        }
        ast::Expr::If(cond, then_blk, else_blk, _) => {
            collect_sql_literals_inner(cond, out);
            collect_sql_literals_block(then_blk, out);
            if let Some(b) = else_blk {
                collect_sql_literals_block(b, out);
            }
        }
        ast::Expr::Match(scrutinee, arms, _) => {
            collect_sql_literals_inner(scrutinee, out);
            for arm in arms {
                collect_sql_literals_inner(&arm.body, out);
            }
        }
        ast::Expr::BinOp(_, l, r, _) => {
            collect_sql_literals_inner(l, out);
            collect_sql_literals_inner(r, out);
        }
        ast::Expr::FieldAccess(obj, _, _) => {
            collect_sql_literals_inner(obj, out);
        }
        ast::Expr::TypeApply(e, _, _) => {
            collect_sql_literals_inner(e, out);
        }
        ast::Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields {
                collect_sql_literals_inner(v, out);
            }
        }
        ast::Expr::Closure(_, body, _) => {
            collect_sql_literals_inner(body, out);
        }
        ast::Expr::Collect(blk, _) => {
            collect_sql_literals_block(blk, out);
        }
        ast::Expr::EmitExpr(e, _) => {
            collect_sql_literals_inner(e, out);
        }
        ast::Expr::AssertMatches(e, _, _) => {
            collect_sql_literals_inner(e, out);
        }
        ast::Expr::AssertSchema { arg, .. } => {
            collect_sql_literals_inner(arg, out);
        }
        ast::Expr::Question(e, _) => {
            collect_sql_literals_inner(e, out);
        }
        ast::Expr::RecordSpread(base, updates, _) => {
            collect_sql_literals_inner(base, out);
            for (_, v) in updates {
                collect_sql_literals_inner(v, out);
            }
        }
        ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _) => {}
        ast::Expr::ListComp { expr, clauses, .. }
        | ast::Expr::ResultComp { expr, clauses, .. } => {
            collect_sql_literals_inner(expr, out);
            for c in clauses {
                match c {
                    ast::CompClause::For { src, .. } => collect_sql_literals_inner(src, out),
                    ast::CompClause::Guard(g) => collect_sql_literals_inner(g, out),
                }
            }
        }
    }
}

fn collect_sql_literals_block(block: &ast::Block, out: &mut Vec<String>) {
    for s in &block.stmts {
        collect_sql_literals_stmt(s, out);
    }
    collect_sql_literals_inner(&block.expr, out);
}

fn collect_sql_literals_stmt(stmt: &ast::Stmt, out: &mut Vec<String>) {
    match stmt {
        ast::Stmt::Bind(b) => collect_sql_literals_inner(&b.expr, out),
        ast::Stmt::Expr(e) => collect_sql_literals_inner(e, out),
        ast::Stmt::Chain(c) => collect_sql_literals_inner(&c.expr, out),
        ast::Stmt::Yield(y) => collect_sql_literals_inner(&y.expr, out),
        ast::Stmt::Return(r) => collect_sql_literals_inner(&r.expr, out),
        ast::Stmt::ForIn(f) => {
            collect_sql_literals_inner(&f.iter, out);
            collect_sql_literals_block(&f.body, out);
        }
        ast::Stmt::Forall(f) => {
            if let Some(g) = &f.guard { collect_sql_literals_inner(g, out); }
            collect_sql_literals_block(&f.body, out);
        }
        ast::Stmt::Expect(_) => {} // v36.2.0 — 実行は v36.3 以降
    }
}

// ── assert_schema name collection (v52.3.0) ──────────────────────────────────

/// Collect the first `assert_schema<T>` type name from an expression tree.
/// Returns the `ty_name` of the first `Expr::AssertSchema` found, or `None`.
pub fn collect_assert_schema_name(expr: &ast::Expr) -> Option<String> {
    match expr {
        ast::Expr::AssertSchema { ty_name, .. } => Some(ty_name.clone()),
        ast::Expr::Block(block) => collect_assert_schema_name_block(block),
        ast::Expr::Collect(block, _) => collect_assert_schema_name_block(block),
        ast::Expr::Pipeline(exprs, _) => exprs.iter().find_map(collect_assert_schema_name),
        ast::Expr::Apply(func, args, _) => {
            // `assert_schema<T>(value)` is parsed as Apply(TypeApply(Ident("assert_schema"), [T]), [value]).
            // The compiler rewrites this to Expr::AssertSchema, but lineage analysis runs on the
            // pre-compilation parsed AST, so we detect the call pattern directly here.
            if let ast::Expr::TypeApply(inner, type_args, _) = func.as_ref() {
                if let ast::Expr::Ident(name, _) = inner.as_ref() {
                    if name == "assert_schema" {
                        if let Some(ast::TypeExpr::Named(ty_name, _, _)) = type_args.first() {
                            return Some(ty_name.clone());
                        }
                    }
                }
            }
            collect_assert_schema_name(func)
                .or_else(|| args.iter().find_map(collect_assert_schema_name))
        }
        ast::Expr::TypeApply(inner, _, _) => collect_assert_schema_name(inner),
        ast::Expr::FieldAccess(inner, _, _) => collect_assert_schema_name(inner),
        ast::Expr::If(cond, then_blk, else_blk, _) => {
            collect_assert_schema_name(cond)
                .or_else(|| collect_assert_schema_name_block(then_blk))
                .or_else(|| else_blk.as_ref().and_then(|b| collect_assert_schema_name_block(b)))
        }
        ast::Expr::Match(scrutinee, arms, _) => {
            collect_assert_schema_name(scrutinee)
                .or_else(|| arms.iter().find_map(|a| collect_assert_schema_name(&a.body)))
        }
        ast::Expr::Closure(_, body, _) => collect_assert_schema_name(body),
        ast::Expr::BinOp(_, lhs, rhs, _) => {
            collect_assert_schema_name(lhs)
                .or_else(|| collect_assert_schema_name(rhs))
        }
        ast::Expr::RecordConstruct(_, fields, _) => {
            fields.iter().find_map(|(_, v)| collect_assert_schema_name(v))
        }
        ast::Expr::RecordSpread(base, fields, _) => {
            collect_assert_schema_name(base)
                .or_else(|| fields.iter().find_map(|(_, v)| collect_assert_schema_name(v)))
        }
        ast::Expr::EmitExpr(inner, _) => collect_assert_schema_name(inner),
        ast::Expr::Question(inner, _) => collect_assert_schema_name(inner),
        ast::Expr::AssertMatches(inner, _, _) => collect_assert_schema_name(inner),
        ast::Expr::ListComp { expr, clauses, .. } => {
            collect_assert_schema_name(expr)
                .or_else(|| clauses.iter().find_map(collect_assert_schema_name_comp_clause))
        }
        ast::Expr::ResultComp { expr, clauses, .. } => {
            collect_assert_schema_name(expr)
                .or_else(|| clauses.iter().find_map(collect_assert_schema_name_comp_clause))
        }
        // Leaf nodes — no sub-expressions to recurse into
        ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _) => None,
    }
}

fn collect_assert_schema_name_comp_clause(clause: &ast::CompClause) -> Option<String> {
    match clause {
        ast::CompClause::For { src, .. } => collect_assert_schema_name(src),
        ast::CompClause::Guard(expr) => collect_assert_schema_name(expr),
    }
}

fn collect_assert_schema_name_stmt(s: &ast::Stmt) -> Option<String> {
    match s {
        ast::Stmt::Bind(b) => collect_assert_schema_name(&b.expr),
        ast::Stmt::Expr(e) => collect_assert_schema_name(e),
        ast::Stmt::Chain(c) => collect_assert_schema_name(&c.expr),
        ast::Stmt::Yield(y) => collect_assert_schema_name(&y.expr),
        ast::Stmt::Return(r) => collect_assert_schema_name(&r.expr),
        ast::Stmt::ForIn(f) => {
            collect_assert_schema_name(&f.iter)
                .or_else(|| collect_assert_schema_name_block(&f.body))
        }
        ast::Stmt::Forall(f) => {
            f.guard.as_ref().and_then(collect_assert_schema_name)
                .or_else(|| collect_assert_schema_name_block(&f.body))
        }
        ast::Stmt::Expect(_) => None,
    }
}

fn collect_assert_schema_name_block(b: &ast::Block) -> Option<String> {
    b.stmts.iter().find_map(collect_assert_schema_name_stmt)
        .or_else(|| collect_assert_schema_name(&b.expr))
}

// ── AzureDb read/write classification (v14.1.0) ───────────────────────────────

fn is_postgres_read_method(name: &str) -> bool {
    name == "query_raw" || name == "query"
}

fn is_postgres_write_method(name: &str) -> bool {
    name == "execute_raw" || name == "execute"
}

/// Walk an expression tree and return `(has_read, has_write)` for Postgres calls.
/// - `Postgres.query_raw(...)`   → has_read
/// - `Postgres.execute_raw(...)` → has_write
pub fn collect_postgres_call_kinds(expr: &ast::Expr) -> (bool, bool) {
    let mut has_read = false;
    let mut has_write = false;
    collect_pg_kinds_inner(expr, &mut has_read, &mut has_write);
    (has_read, has_write)
}

fn collect_pg_kinds_inner(expr: &ast::Expr, r: &mut bool, w: &mut bool) {
    match expr {
        ast::Expr::Apply(func, args, _) => {
            if let ast::Expr::FieldAccess(obj, method, _) = func.as_ref() {
                let is_pg = matches!(
                    obj.as_ref(),
                    ast::Expr::Ident(n, _) if n == "Postgres" || n == "postgres"
                );
                if is_pg {
                    if is_postgres_read_method(method)  { *r = true; }
                    if is_postgres_write_method(method) { *w = true; }
                }
            }
            for a in args { collect_pg_kinds_inner(a, r, w); }
            collect_pg_kinds_inner(func, r, w);
        }
        ast::Expr::Block(blk) => {
            for s in &blk.stmts { collect_pg_kinds_stmt(s, r, w); }
            collect_pg_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::If(cond, then_blk, else_blk, _) => {
            collect_pg_kinds_inner(cond, r, w);
            for s in &then_blk.stmts { collect_pg_kinds_stmt(s, r, w); }
            collect_pg_kinds_inner(&then_blk.expr, r, w);
            if let Some(b) = else_blk {
                for s in &b.stmts { collect_pg_kinds_stmt(s, r, w); }
                collect_pg_kinds_inner(&b.expr, r, w);
            }
        }
        ast::Expr::Match(scrutinee, arms, _) => {
            collect_pg_kinds_inner(scrutinee, r, w);
            for arm in arms { collect_pg_kinds_inner(&arm.body, r, w); }
        }
        ast::Expr::Pipeline(exprs, _) => {
            for e in exprs { collect_pg_kinds_inner(e, r, w); }
        }
        ast::Expr::Closure(_, body, _) => { collect_pg_kinds_inner(body, r, w); }
        ast::Expr::Collect(blk, _) => {
            for s in &blk.stmts { collect_pg_kinds_stmt(s, r, w); }
            collect_pg_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::BinOp(_, l, r2, _) => {
            collect_pg_kinds_inner(l, r, w);
            collect_pg_kinds_inner(r2, r, w);
        }
        ast::Expr::FieldAccess(obj, _, _) | ast::Expr::TypeApply(obj, _, _) => {
            collect_pg_kinds_inner(obj, r, w);
        }
        _ => {}
    }
}

fn collect_pg_kinds_stmt(stmt: &ast::Stmt, r: &mut bool, w: &mut bool) {
    match stmt {
        ast::Stmt::Bind(b)  => collect_pg_kinds_inner(&b.expr, r, w),
        ast::Stmt::Chain(c) => collect_pg_kinds_inner(&c.expr, r, w),
        ast::Stmt::Expr(e)  => collect_pg_kinds_inner(e, r, w),
        ast::Stmt::Yield(y) => collect_pg_kinds_inner(&y.expr, r, w),
        ast::Stmt::Return(r2) => collect_pg_kinds_inner(&r2.expr, r, w),
        ast::Stmt::ForIn(f) => {
            collect_pg_kinds_inner(&f.iter, r, w);
            for s in &f.body.stmts { collect_pg_kinds_stmt(s, r, w); }
            collect_pg_kinds_inner(&f.body.expr, r, w);
        }
        ast::Stmt::Forall(f) => {
            if let Some(g) = &f.guard { collect_pg_kinds_inner(g, r, w); }
            for s in &f.body.stmts { collect_pg_kinds_stmt(s, r, w); }
            collect_pg_kinds_inner(&f.body.expr, r, w);
        }
        ast::Stmt::Expect(_) => {} // v36.2.0 — 実行は v36.3 以降
    }
}

fn is_azure_db_read_method(name: &str) -> bool {
    name == "query_raw"
}

fn is_azure_db_write_method(name: &str) -> bool {
    name == "execute_raw"
}

/// Walk an expression tree and return `(has_read, has_write)` for AzurePostgres calls.
/// - `AzurePostgres.query_raw(...)`   → has_read
/// - `AzurePostgres.execute_raw(...)` → has_write
pub fn collect_azure_call_kinds(expr: &ast::Expr) -> (bool, bool) {
    let mut has_read = false;
    let mut has_write = false;
    collect_azure_kinds_inner(expr, &mut has_read, &mut has_write);
    (has_read, has_write)
}

fn collect_azure_kinds_inner(expr: &ast::Expr, r: &mut bool, w: &mut bool) {
    match expr {
        ast::Expr::Apply(func, args, _) => {
            if let ast::Expr::FieldAccess(obj, method, _) = func.as_ref() {
                let is_azure = matches!(
                    obj.as_ref(),
                    ast::Expr::Ident(n, _) if n == "AzurePostgres"
                );
                if is_azure {
                    if is_azure_db_read_method(method) {
                        *r = true;
                    }
                    if is_azure_db_write_method(method) {
                        *w = true;
                    }
                }
            }
            for a in args {
                collect_azure_kinds_inner(a, r, w);
            }
            collect_azure_kinds_inner(func, r, w);
        }
        ast::Expr::Block(blk) => {
            for s in &blk.stmts {
                collect_azure_kinds_stmt(s, r, w);
            }
            collect_azure_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::If(cond, then_blk, else_blk, _) => {
            collect_azure_kinds_inner(cond, r, w);
            for s in &then_blk.stmts {
                collect_azure_kinds_stmt(s, r, w);
            }
            collect_azure_kinds_inner(&then_blk.expr, r, w);
            if let Some(b) = else_blk {
                for s in &b.stmts {
                    collect_azure_kinds_stmt(s, r, w);
                }
                collect_azure_kinds_inner(&b.expr, r, w);
            }
        }
        ast::Expr::Match(scrutinee, arms, _) => {
            collect_azure_kinds_inner(scrutinee, r, w);
            for arm in arms {
                collect_azure_kinds_inner(&arm.body, r, w);
            }
        }
        ast::Expr::Pipeline(exprs, _) => {
            for e in exprs {
                collect_azure_kinds_inner(e, r, w);
            }
        }
        ast::Expr::Closure(_, body, _) => {
            collect_azure_kinds_inner(body, r, w);
        }
        ast::Expr::Collect(blk, _) => {
            for s in &blk.stmts {
                collect_azure_kinds_stmt(s, r, w);
            }
            collect_azure_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::BinOp(_, l, r2, _) => {
            collect_azure_kinds_inner(l, r, w);
            collect_azure_kinds_inner(r2, r, w);
        }
        ast::Expr::FieldAccess(obj, _, _) | ast::Expr::TypeApply(obj, _, _) => {
            collect_azure_kinds_inner(obj, r, w);
        }
        ast::Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields {
                collect_azure_kinds_inner(v, r, w);
            }
        }
        ast::Expr::EmitExpr(e, _)
        | ast::Expr::AssertMatches(e, _, _)
        | ast::Expr::Question(e, _) => {
            collect_azure_kinds_inner(e, r, w);
        }
        ast::Expr::AssertSchema { arg, .. } => collect_azure_kinds_inner(arg, r, w),
        ast::Expr::RecordSpread(base, updates, _) => {
            collect_azure_kinds_inner(base, r, w);
            for (_, v) in updates {
                collect_azure_kinds_inner(v, r, w);
            }
        }
        ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _) => {}
        ast::Expr::ListComp { expr, clauses, .. }
        | ast::Expr::ResultComp { expr, clauses, .. } => {
            collect_azure_kinds_inner(expr, r, w);
            for c in clauses {
                if let ast::CompClause::For { src, .. } = c {
                    collect_azure_kinds_inner(src, r, w);
                }
            }
        }
    }
}

fn collect_azure_kinds_stmt(stmt: &ast::Stmt, r: &mut bool, w: &mut bool) {
    match stmt {
        ast::Stmt::Bind(b) => collect_azure_kinds_inner(&b.expr, r, w),
        ast::Stmt::Expr(e) => collect_azure_kinds_inner(e, r, w),
        ast::Stmt::Chain(c) => collect_azure_kinds_inner(&c.expr, r, w),
        ast::Stmt::Yield(y) => collect_azure_kinds_inner(&y.expr, r, w),
        ast::Stmt::Return(r2) => collect_azure_kinds_inner(&r2.expr, r, w),
        ast::Stmt::ForIn(f) => {
            collect_azure_kinds_inner(&f.iter, r, w);
            for s in &f.body.stmts {
                collect_azure_kinds_stmt(s, r, w);
            }
            collect_azure_kinds_inner(&f.body.expr, r, w);
        }
        ast::Stmt::Forall(f) => {
            if let Some(g) = &f.guard { collect_azure_kinds_inner(g, r, w); }
            for s in &f.body.stmts { collect_azure_kinds_stmt(s, r, w); }
            collect_azure_kinds_inner(&f.body.expr, r, w);
        }
        ast::Stmt::Expect(_) => {} // v36.2.0 — 実行は v36.3 以降
    }
}

// ── AzureBlob read/write classification (v14.3.0) ─────────────────────────────

fn is_azure_blob_read_method(method: &str) -> bool {
    matches!(method, "get_raw" | "list_raw")
}

fn is_azure_blob_write_method(method: &str) -> bool {
    matches!(method, "put_raw" | "delete_raw")
}

/// Walk an expression tree and return `(has_read, has_write)` for AzureBlob calls.
/// - `AzureBlob.get_raw(...)` / `AzureBlob.list_raw(...)` → has_read
/// - `AzureBlob.put_raw(...)` / `AzureBlob.delete_raw(...)` → has_write
pub fn collect_azure_blob_call_kinds(expr: &ast::Expr) -> (bool, bool) {
    let mut has_read = false;
    let mut has_write = false;
    collect_azure_blob_kinds_inner(expr, &mut has_read, &mut has_write);
    (has_read, has_write)
}

fn collect_azure_blob_kinds_inner(expr: &ast::Expr, r: &mut bool, w: &mut bool) {
    match expr {
        ast::Expr::Apply(func, args, _) => {
            if let ast::Expr::FieldAccess(obj, method, _) = func.as_ref() {
                if matches!(obj.as_ref(), ast::Expr::Ident(n, _) if n == "AzureBlob") {
                    if is_azure_blob_read_method(method) { *r = true; }
                    if is_azure_blob_write_method(method) { *w = true; }
                }
            }
            for a in args { collect_azure_blob_kinds_inner(a, r, w); }
            collect_azure_blob_kinds_inner(func, r, w);
        }
        ast::Expr::Block(blk) => {
            for s in &blk.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
            collect_azure_blob_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::If(cond, then_blk, else_blk, _) => {
            collect_azure_blob_kinds_inner(cond, r, w);
            for s in &then_blk.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
            collect_azure_blob_kinds_inner(&then_blk.expr, r, w);
            if let Some(b) = else_blk {
                for s in &b.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
                collect_azure_blob_kinds_inner(&b.expr, r, w);
            }
        }
        ast::Expr::Match(scrutinee, arms, _) => {
            collect_azure_blob_kinds_inner(scrutinee, r, w);
            for arm in arms { collect_azure_blob_kinds_inner(&arm.body, r, w); }
        }
        ast::Expr::Pipeline(exprs, _) => {
            for e in exprs { collect_azure_blob_kinds_inner(e, r, w); }
        }
        ast::Expr::Closure(_, body, _) => { collect_azure_blob_kinds_inner(body, r, w); }
        ast::Expr::Collect(blk, _) => {
            for s in &blk.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
            collect_azure_blob_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::BinOp(_, l, r2, _) => {
            collect_azure_blob_kinds_inner(l, r, w);
            collect_azure_blob_kinds_inner(r2, r, w);
        }
        ast::Expr::FieldAccess(obj, _, _) | ast::Expr::TypeApply(obj, _, _) => {
            collect_azure_blob_kinds_inner(obj, r, w);
        }
        ast::Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields { collect_azure_blob_kinds_inner(v, r, w); }
        }
        ast::Expr::EmitExpr(e, _)
        | ast::Expr::AssertMatches(e, _, _)
        | ast::Expr::Question(e, _) => {
            collect_azure_blob_kinds_inner(e, r, w);
        }
        ast::Expr::AssertSchema { arg, .. } => collect_azure_blob_kinds_inner(arg, r, w),
        ast::Expr::RecordSpread(base, updates, _) => {
            collect_azure_blob_kinds_inner(base, r, w);
            for (_, v) in updates {
                collect_azure_blob_kinds_inner(v, r, w);
            }
        }
        ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _) => {}
        ast::Expr::ListComp { expr, clauses, .. }
        | ast::Expr::ResultComp { expr, clauses, .. } => {
            collect_azure_blob_kinds_inner(expr, r, w);
            for c in clauses {
                if let ast::CompClause::For { src, .. } = c {
                    collect_azure_blob_kinds_inner(src, r, w);
                }
            }
        }
    }
}

fn collect_azure_blob_kinds_stmt(stmt: &ast::Stmt, r: &mut bool, w: &mut bool) {
    match stmt {
        ast::Stmt::Bind(b) => collect_azure_blob_kinds_inner(&b.expr, r, w),
        ast::Stmt::Expr(e) => collect_azure_blob_kinds_inner(e, r, w),
        ast::Stmt::Chain(c) => collect_azure_blob_kinds_inner(&c.expr, r, w),
        ast::Stmt::Yield(y) => collect_azure_blob_kinds_inner(&y.expr, r, w),
        ast::Stmt::Return(r2) => collect_azure_blob_kinds_inner(&r2.expr, r, w),
        ast::Stmt::ForIn(f) => {
            collect_azure_blob_kinds_inner(&f.iter, r, w);
            for s in &f.body.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
            collect_azure_blob_kinds_inner(&f.body.expr, r, w);
        }
        ast::Stmt::Forall(f) => {
            if let Some(g) = &f.guard { collect_azure_blob_kinds_inner(g, r, w); }
            for s in &f.body.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
            collect_azure_blob_kinds_inner(&f.body.expr, r, w);
        }
        ast::Stmt::Expect(_) => {} // v36.2.0 — 実行は v36.3 以降
    }
}


// ── Snowflake read/write classification ───────────────────────────────────────

fn is_snowflake_read_method(name: &str) -> bool {
    name == "query" || name == "query_raw"
}

fn is_snowflake_write_method(name: &str) -> bool {
    name == "execute" || name == "execute_raw"
}

/// Walk an expression tree and return `(has_read, has_write)` for Snowflake calls.
/// - `snowflake.query(...)` / `snowflake.query_raw(...)`   → has_read
/// - `snowflake.execute(...)` / `snowflake.execute_raw(...)` → has_write
pub fn collect_snowflake_call_kinds(expr: &ast::Expr) -> (bool, bool) {
    let mut has_read = false;
    let mut has_write = false;
    collect_sf_kinds_inner(expr, &mut has_read, &mut has_write);
    (has_read, has_write)
}

fn collect_sf_kinds_inner(expr: &ast::Expr, r: &mut bool, w: &mut bool) {
    match expr {
        ast::Expr::Apply(func, args, _) => {
            if let ast::Expr::FieldAccess(obj, method, _) = func.as_ref() {
                let is_sf = matches!(
                    obj.as_ref(),
                    ast::Expr::Ident(n, _) if n == "snowflake" || n == "Snowflake"
                );
                if is_sf {
                    if is_snowflake_read_method(method) {
                        *r = true;
                    }
                    if is_snowflake_write_method(method) {
                        *w = true;
                    }
                }
            }
            for a in args {
                collect_sf_kinds_inner(a, r, w);
            }
            collect_sf_kinds_inner(func, r, w);
        }
        ast::Expr::Block(blk) => {
            for s in &blk.stmts {
                collect_sf_kinds_stmt(s, r, w);
            }
            collect_sf_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::If(cond, then_blk, else_blk, _) => {
            collect_sf_kinds_inner(cond, r, w);
            for s in &then_blk.stmts {
                collect_sf_kinds_stmt(s, r, w);
            }
            collect_sf_kinds_inner(&then_blk.expr, r, w);
            if let Some(b) = else_blk {
                for s in &b.stmts {
                    collect_sf_kinds_stmt(s, r, w);
                }
                collect_sf_kinds_inner(&b.expr, r, w);
            }
        }
        ast::Expr::Match(scrutinee, arms, _) => {
            collect_sf_kinds_inner(scrutinee, r, w);
            for arm in arms {
                collect_sf_kinds_inner(&arm.body, r, w);
            }
        }
        ast::Expr::Pipeline(exprs, _) => {
            for e in exprs {
                collect_sf_kinds_inner(e, r, w);
            }
        }
        ast::Expr::Closure(_, body, _) => {
            collect_sf_kinds_inner(body, r, w);
        }
        ast::Expr::Collect(blk, _) => {
            for s in &blk.stmts {
                collect_sf_kinds_stmt(s, r, w);
            }
            collect_sf_kinds_inner(&blk.expr, r, w);
        }
        ast::Expr::BinOp(_, l, r2, _) => {
            collect_sf_kinds_inner(l, r, w);
            collect_sf_kinds_inner(r2, r, w);
        }
        ast::Expr::FieldAccess(obj, _, _) | ast::Expr::TypeApply(obj, _, _) => {
            collect_sf_kinds_inner(obj, r, w);
        }
        ast::Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields {
                collect_sf_kinds_inner(v, r, w);
            }
        }
        ast::Expr::EmitExpr(e, _)
        | ast::Expr::AssertMatches(e, _, _)
        | ast::Expr::Question(e, _) => {
            collect_sf_kinds_inner(e, r, w);
        }
        ast::Expr::AssertSchema { arg, .. } => collect_sf_kinds_inner(arg, r, w),
        ast::Expr::RecordSpread(base, updates, _) => {
            collect_sf_kinds_inner(base, r, w);
            for (_, v) in updates {
                collect_sf_kinds_inner(v, r, w);
            }
        }
        ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _) => {}
        ast::Expr::ListComp { expr, clauses, .. }
        | ast::Expr::ResultComp { expr, clauses, .. } => {
            collect_sf_kinds_inner(expr, r, w);
            for c in clauses {
                if let ast::CompClause::For { src, .. } = c {
                    collect_sf_kinds_inner(src, r, w);
                }
            }
        }
    }
}

fn collect_sf_kinds_stmt(stmt: &ast::Stmt, r: &mut bool, w: &mut bool) {
    match stmt {
        ast::Stmt::Bind(b) => collect_sf_kinds_inner(&b.expr, r, w),
        ast::Stmt::Expr(e) => collect_sf_kinds_inner(e, r, w),
        ast::Stmt::Chain(c) => collect_sf_kinds_inner(&c.expr, r, w),
        ast::Stmt::Yield(y) => collect_sf_kinds_inner(&y.expr, r, w),
        ast::Stmt::Return(r2) => collect_sf_kinds_inner(&r2.expr, r, w),
        ast::Stmt::ForIn(f) => {
            collect_sf_kinds_inner(&f.iter, r, w);
            for s in &f.body.stmts {
                collect_sf_kinds_stmt(s, r, w);
            }
            collect_sf_kinds_inner(&f.body.expr, r, w);
        }
        ast::Stmt::Forall(f) => {
            if let Some(g) = &f.guard { collect_sf_kinds_inner(g, r, w); }
            for s in &f.body.stmts { collect_sf_kinds_stmt(s, r, w); }
            collect_sf_kinds_inner(&f.body.expr, r, w);
        }
        ast::Stmt::Expect(_) => {} // v36.2.0 — 実行は v36.3 以降
    }
}

/// Build the inferred effects list for a stage/fn from body call analysis.
/// v35.4.0: !Effect annotations are gone; effects are inferred from body calls only.
fn infer_effects_from_calls(
    sf_read: bool, sf_write: bool,
    az_read: bool, az_write: bool,
    az_blob_read: bool, az_blob_write: bool,
) -> Vec<String> {
    let mut result = Vec::new();
    if sf_read  { result.push("!Snowflake(read)".to_string()); }
    if sf_write { result.push("!Snowflake(write)".to_string()); }
    if az_read  { result.push("!AzureDb(read)".to_string()); }
    if az_write { result.push("!AzureDb(write)".to_string()); }
    if az_blob_read  { result.push("!AzureStorage(read)".to_string()); }
    if az_blob_write { result.push("!AzureStorage(write)".to_string()); }
    result
}

// ── public API ────────────────────────────────────────────────────────────────

/// Analyse a parsed program and build a `LineageReport`.
pub fn lineage_analysis(program: &ast::Program) -> LineageReport {
    let mut transformations: Vec<LineageEntry> = Vec::new();

    for item in &program.items {
        if let ast::Item::TrfDef(trf) = item {
            let sqls = collect_sql_literals(&ast::Expr::Block(Box::new(trf.body.clone())));
            let mut sources: Vec<String> = Vec::new();
            let mut sinks: Vec<String> = Vec::new();
            for sql in &sqls {
                let (reads, writes) = extract_tables_from_sql(sql);
                for r in reads {
                    if !sources.contains(&r) {
                        sources.push(r);
                    }
                }
                for w in writes {
                    if !sinks.contains(&w) {
                        sinks.push(w);
                    }
                }
            }
            // Snowflake read/write classification — inferred from body calls
            let (sf_read, sf_write) = collect_snowflake_call_kinds(&ast::Expr::Block(Box::new(trf.body.clone())));
            if sf_read {
                sources.push(format!("({}:snowflake-read)", trf.name));
            }
            if sf_write {
                sinks.push(format!("({}:snowflake-write)", trf.name));
            }
            // AzureDb read/write classification (v14.1.0)
            let (az_read, az_write) = collect_azure_call_kinds(&ast::Expr::Block(Box::new(trf.body.clone())));
            if az_read {
                sources.push(format!("({}:azure-db-read)", trf.name));
            }
            if az_write {
                sinks.push(format!("({}:azure-db-write)", trf.name));
            }
            // AzureBlob read/write classification (v14.3.0)
            let (az_blob_read, az_blob_write) = collect_azure_blob_call_kinds(&ast::Expr::Block(Box::new(trf.body.clone())));
            if az_blob_read {
                sources.push(format!("({}:azure-blob-read)", trf.name));
            }
            if az_blob_write {
                sinks.push(format!("({}:azure-blob-write)", trf.name));
            }
            // Postgres read/write classification (v35.6.0)
            let (pg_read, pg_write) = collect_postgres_call_kinds(&ast::Expr::Block(Box::new(trf.body.clone())));
            if pg_read {
                sources.push(format!("({}:postgres-read)", trf.name));
            }
            if pg_write {
                sinks.push(format!("({}:postgres-write)", trf.name));
            }
            let (mut cap_kind, mut cap_name) = classify_capability_kind(&trf.params);
            // Fallback: if no ctx param matched, infer from Snowflake/Azure/Postgres call kinds
            if cap_kind == "transform" {
                if sf_read || az_read || az_blob_read || pg_read {
                    cap_kind = "read".into();
                    cap_name = Some("DbRead".into());
                } else if sf_write || az_write || az_blob_write || pg_write {
                    cap_kind = "write".into();
                    cap_name = Some("DbWrite".into());
                }
            }
            let mut effects = infer_effects_from_calls(sf_read, sf_write, az_read, az_write, az_blob_read, az_blob_write);
            if pg_read  { effects.push("!Postgres(read)".to_string()); }
            if pg_write { effects.push("!Postgres(write)".to_string()); }
            let schema = collect_assert_schema_name_block(&trf.body);
            transformations.push(LineageEntry {
                name: trf.name.clone(),
                kind: cap_kind,
                capability: cap_name,
                effects,
                sources,
                sinks,
                is_dead: has_early_return(&trf.body.stmts),
                schema,
            });
        } else if let ast::Item::FnDef(fndef) = item {
            let sqls = collect_sql_literals(&ast::Expr::Block(Box::new(fndef.body.clone())));
            let mut sources: Vec<String> = Vec::new();
            let mut sinks: Vec<String> = Vec::new();
            for sql in &sqls {
                let (reads, writes) = extract_tables_from_sql(sql);
                for r in reads {
                    if !sources.contains(&r) {
                        sources.push(r);
                    }
                }
                for w in writes {
                    if !sinks.contains(&w) {
                        sinks.push(w);
                    }
                }
            }
            // Snowflake read/write classification — inferred from body calls
            let (sf_read, sf_write) = collect_snowflake_call_kinds(&ast::Expr::Block(Box::new(fndef.body.clone())));
            if sf_read {
                sources.push(format!("({}:snowflake-read)", fndef.name));
            }
            if sf_write {
                sinks.push(format!("({}:snowflake-write)", fndef.name));
            }
            // AzureDb read/write classification (v14.1.0)
            let (az_read, az_write) = collect_azure_call_kinds(&ast::Expr::Block(Box::new(fndef.body.clone())));
            if az_read {
                sources.push(format!("({}:azure-db-read)", fndef.name));
            }
            if az_write {
                sinks.push(format!("({}:azure-db-write)", fndef.name));
            }
            // AzureBlob read/write classification (v14.3.0)
            let (az_blob_read, az_blob_write) = collect_azure_blob_call_kinds(&ast::Expr::Block(Box::new(fndef.body.clone())));
            if az_blob_read {
                sources.push(format!("({}:azure-blob-read)", fndef.name));
            }
            if az_blob_write {
                sinks.push(format!("({}:azure-blob-write)", fndef.name));
            }
            // Emit entry for ctx-based functions or functions with inferred effects
            let has_ctx_param = fndef.params.iter().any(|p| {
                matches!(&p.ty, ast::TypeExpr::Named(n, _, _) if
                    matches!(n.as_str(), "AppCtx"|"CommonCtx"|"LoadCtx"|"WriteCtx"|"MigrateCtx"|"MockCtx"|"DbCtx"|"IoCtx"|"HttpCtx"|"StreamCtx"))
            });
            let (pg_read, pg_write) = collect_postgres_call_kinds(&ast::Expr::Block(Box::new(fndef.body.clone())));
            if pg_read {
                sources.push(format!("({}:postgres-read)", fndef.name));
            }
            if pg_write {
                sinks.push(format!("({}:postgres-write)", fndef.name));
            }
            if has_ctx_param || sf_read || sf_write || az_read || az_write || pg_read || pg_write {
                let (cap_kind, cap_name) = classify_capability_kind(&fndef.params);
                let mut effects = infer_effects_from_calls(sf_read, sf_write, az_read, az_write, az_blob_read, az_blob_write);
                if pg_read  { effects.push("!Postgres(read)".to_string()); }
                if pg_write { effects.push("!Postgres(write)".to_string()); }
                transformations.push(LineageEntry {
                    name: fndef.name.clone(),
                    kind: cap_kind,
                    capability: cap_name,
                    effects,
                    sources,
                    sinks,
                    is_dead: has_early_return(&fndef.body.stmts),
                    // fn は assert_schema<T> の主要な使用場所でないため収集しない（設計上の選択）
                    schema: None,
                });
            }
        }
    }

    let mut entry_map: std::collections::HashMap<String, (Vec<String>, Vec<String>)> =
        std::collections::HashMap::new();
    for e in &transformations {
        entry_map.insert(e.name.clone(), (e.sources.clone(), e.sinks.clone()));
    }

    let mut pipelines: Vec<PipelineLineage> = Vec::new();
    for item in &program.items {
        if let ast::Item::FlwDef(flw) = item {
            let mut all_sources: Vec<String> = Vec::new();
            let mut all_sinks: Vec<String> = Vec::new();
            for step in &flw.steps {
                for stage_name in step.stage_names() {
                    if let Some((srcs, snks)) = entry_map.get(stage_name) {
                        for s in srcs {
                            if !all_sources.contains(s) {
                                all_sources.push(s.clone());
                            }
                        }
                        for s in snks {
                            if !all_sinks.contains(s) {
                                all_sinks.push(s.clone());
                            }
                        }
                    }
                }
            }
            let step_strs: Vec<String> = flw.steps.iter().map(|s| s.display_str()).collect();
            pipelines.push(PipelineLineage {
                name: flw.name.clone(),
                steps: step_strs,
                sources: all_sources,
                sinks: all_sinks,
            });
        }
    }

    LineageReport {
        transformations,
        pipelines,
    }
}

/// Render lineage as human-readable text.
pub fn render_lineage_text(report: &LineageReport, filename: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("Lineage: {}\n", filename));
    out.push('\n');

    let mut all_sources: Vec<String> = Vec::new();
    let mut all_sinks: Vec<String> = Vec::new();
    for e in &report.transformations {
        for s in &e.sources {
            if !all_sources.contains(s) {
                all_sources.push(s.clone());
            }
        }
        for s in &e.sinks {
            if !all_sinks.contains(s) {
                all_sinks.push(s.clone());
            }
        }
    }
    for p in &report.pipelines {
        for s in &p.sources {
            if !all_sources.contains(s) {
                all_sources.push(s.clone());
            }
        }
        for s in &p.sinks {
            if !all_sinks.contains(s) {
                all_sinks.push(s.clone());
            }
        }
    }

    out.push_str("Sources:\n");
    if all_sources.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for s in &all_sources {
            out.push_str(&format!("  - {}\n", s));
        }
    }
    out.push('\n');

    out.push_str("Sinks:\n");
    if all_sinks.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for s in &all_sinks {
            out.push_str(&format!("  - {}\n", s));
        }
    }
    out.push('\n');

    out.push_str("Transformations:\n");
    if report.transformations.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for e in &report.transformations {
            let cap_str = e.capability.as_deref().unwrap_or("(pure)");
            out.push_str(&format!(
                "  {:12} [{}]  {}",
                e.name,
                e.kind,
                cap_str,
            ));
            if !e.sources.is_empty() || !e.sinks.is_empty() {
                out.push_str(&format!(
                    "  sources=[{}] sinks=[{}]",
                    e.sources.join(", "),
                    e.sinks.join(", ")
                ));
            }
            out.push('\n');
        }
    }
    out.push('\n');

    // CrossCloud Flow: emit when both an AWS-side DB effect and coexist (v14.3.0)
    let has_aws_db = report.transformations.iter().any(|e| {
        e.effects.iter().any(|eff| {
            eff.contains("!Postgres") || eff.contains("!Db") || eff.contains("!Snowflake")
        })
    });
    let has_azure_db = report.transformations.iter().any(|e| {
        e.effects.iter().any(|eff| eff.contains("!AzureDb"))
    });
    if has_aws_db && has_azure_db {
        out.push_str("CrossCloud Flow:\n");
        let stages: Vec<String> = if !report.pipelines.is_empty() {
            report.pipelines[0].steps.clone()
        } else {
            report.transformations.iter().map(|e| e.name.clone()).collect()
        };
        out.push_str(&format!("  [AWS RDS] → {} → [Azure Postgres]\n", stages.join(" → ")));
        out.push('\n');
    }

    out.push_str("Pipelines:\n");
    if report.pipelines.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for p in &report.pipelines {
            out.push_str(&format!("  seq {} = {}\n", p.name, p.steps.join(" |> ")));
            if !p.sources.is_empty() {
                out.push_str(&format!("    sources: {}\n", p.sources.join(", ")));
            }
            if !p.sinks.is_empty() {
                out.push_str(&format!("    sinks:   {}\n", p.sinks.join(", ")));
            }
        }
    }

    // v37.9.0: サマリー行
    out.push('\n');
    out.push_str(&format!(
        "Total: {} stage(s), {} pipeline(s)\n",
        report.transformations.len(),
        report.pipelines.len(),
    ));

    out
}

/// Render lineage as JSON.
pub fn render_lineage_json(report: &LineageReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into())
}

/// LineageReport を Mermaid flowchart LR 形式にレンダリングする。
pub fn render_lineage_mermaid(report: &LineageReport) -> String {
    render_lineage_mermaid_with_opts(report, false)
}

/// v52.3.0: `--with-schema` オプション付き mermaid レンダリング。
/// `with_schema = true` のとき、スキーマ名が存在するノードラベルに `<br/>schema:<Name>` を追加。
pub fn render_lineage_mermaid_with_schema(
    report: &LineageReport,
    show_dead: bool,
    with_schema: bool,
) -> String {
    let mut out = String::from("flowchart LR\n");
    if show_dead && report.transformations.iter().any(|e| e.is_dead) {
        out.push_str("    classDef deadEntry stroke-dasharray:5 5\n");
    }

    for entry in &report.transformations {
        let effects = if entry.effects.is_empty() {
            "Pure".to_string()
        } else {
            entry.effects.iter()
                .map(|e| format!("!{}", e.trim_start_matches('!')))
                .collect::<Vec<_>>()
                .join("+")
        };
        let id = sanitize_mermaid_id(&entry.name);
        let schema_label = if with_schema {
            entry.schema.as_ref()
                .map(|s| format!("<br/>schema:{}", s))
                .unwrap_or_default()
        } else {
            String::new()
        };
        out.push_str(&format!(
            "  {}[\"{}<br/>{}{}\"]\n",
            id, entry.name, effects, schema_label
        ));
        if show_dead && entry.is_dead {
            out.push_str(&format!("  class {} deadEntry\n", id));
        }
    }

    for pipeline in &report.pipelines {
        let steps = &pipeline.steps;
        for i in 0..steps.len().saturating_sub(1) {
            let from = sanitize_mermaid_id(&steps[i]);
            let to   = sanitize_mermaid_id(&steps[i + 1]);
            out.push_str(&format!("  {} --> {}\n", from, to));
        }
    }

    out
}

/// v46.7.0: `fav explain --lineage --show-dead` 用。
/// `show_dead = true` のとき dead エントリに `classDef deadEntry` + `class <id> deadEntry` を付与。
/// dead エントリが存在しない場合は `classDef` を出力しない（Mermaid ノイズ抑制）。
pub fn render_lineage_mermaid_with_opts(report: &LineageReport, show_dead: bool) -> String {
    let mut out = String::from("flowchart LR\n");
    // classDef は dead エントリが 1 件以上ある場合のみ出力する
    if show_dead && report.transformations.iter().any(|e| e.is_dead) {
        out.push_str("    classDef deadEntry stroke-dasharray:5 5\n");
    }

    // ノード定義: stage / fn ごとに1ノード
    for entry in &report.transformations {
        let effects = if entry.effects.is_empty() {
            "Pure".to_string()
        } else {
            entry.effects.iter()
                .map(|e| format!("!{}", e.trim_start_matches('!')))
                .collect::<Vec<_>>()
                .join("+")
        };
        let id = sanitize_mermaid_id(&entry.name);
        out.push_str(&format!("  {}[\"{}<br/>{}\"]\n", id, entry.name, effects));
        if show_dead && entry.is_dead {
            out.push_str(&format!("  class {} deadEntry\n", id));
        }
    }

    // エッジ定義: pipeline の steps を順に接続
    for pipeline in &report.pipelines {
        let steps = &pipeline.steps;
        for i in 0..steps.len().saturating_sub(1) {
            let from = sanitize_mermaid_id(&steps[i]);
            let to   = sanitize_mermaid_id(&steps[i + 1]);
            out.push_str(&format!("  {} --> {}\n", from, to));
        }
    }

    out
}

/// LineageReport を D2 diagram 形式にレンダリングする。
pub fn render_lineage_d2(report: &LineageReport) -> String {
    let mut out = String::new();

    // ノード定義
    for entry in &report.transformations {
        let effects = if entry.effects.is_empty() {
            "Pure".to_string()
        } else {
            entry.effects.iter()
                .map(|e| format!("!{}", e.trim_start_matches('!')))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let id = sanitize_mermaid_id(&entry.name);
        out.push_str(&format!("{}: \"{} ({})\"\n", id, entry.name, effects));
    }

    // エッジ定義
    for pipeline in &report.pipelines {
        let steps = &pipeline.steps;
        for i in 0..steps.len().saturating_sub(1) {
            let from = sanitize_mermaid_id(&steps[i]);
            let to   = sanitize_mermaid_id(&steps[i + 1]);
            out.push_str(&format!("{} -> {}\n", from, to));
        }
    }

    out
}

/// LineageReport を Graphviz DOT 形式にレンダリングする。
pub fn render_lineage_dot(report: &LineageReport) -> String {
    let mut out = String::from("digraph lineage {\n    rankdir=LR;\n    node [shape=box style=filled fillcolor=\"#eef6f9\"];\n");

    for entry in &report.transformations {
        let id    = sanitize_mermaid_id(&entry.name);
        let label = format!("{}\\n{}", entry.name, entry.kind);
        out.push_str(&format!("    {} [label=\"{}\"];\n", id, label));
    }

    for pipeline in &report.pipelines {
        let steps = &pipeline.steps;
        for i in 0..steps.len().saturating_sub(1) {
            let from = sanitize_mermaid_id(&steps[i]);
            let to   = sanitize_mermaid_id(&steps[i + 1]);
            out.push_str(&format!("    {} -> {};\n", from, to));
        }
    }

    out.push('}');
    out
}

/// v52.3.0: `--with-schema` オプション付き DOT レンダリング。
/// `with_schema = true` のとき、スキーマ名が存在するノードラベルに `\nschema:<Name>` を追加。
pub fn render_lineage_dot_with_schema(report: &LineageReport, with_schema: bool) -> String {
    let mut out = String::from(
        "digraph lineage {\n    rankdir=LR;\n    node [shape=box style=filled fillcolor=\"#eef6f9\"];\n"
    );

    for entry in &report.transformations {
        let id = sanitize_mermaid_id(&entry.name);
        let schema_part = if with_schema {
            entry.schema.as_ref()
                .map(|s| format!("\\nschema:{}", s))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let label = format!("{}\\n{}{}", entry.name, entry.kind, schema_part);
        out.push_str(&format!("    {} [label=\"{}\"];\n", id, label));
    }

    for pipeline in &report.pipelines {
        let steps = &pipeline.steps;
        for i in 0..steps.len().saturating_sub(1) {
            let from = sanitize_mermaid_id(&steps[i]);
            let to   = sanitize_mermaid_id(&steps[i + 1]);
            out.push_str(&format!("    {} -> {};\n", from, to));
        }
    }

    out.push('}');
    out
}

/// LineageReport を外部ツール不要のインライン SVG にレンダリングする。
/// 各ノードを 160×40px の矩形として横に並べ、矢印で接続する。
pub fn render_lineage_svg(report: &LineageReport) -> String {
    let nodes: Vec<&LineageEntry> = report.transformations.iter().collect();
    let n      = nodes.len();
    let width  = (n * 200 + 40).max(200);
    let height = 140_usize;

    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\n",
        width, height
    );
    out.push_str("  <defs><marker id=\"arr\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" orient=\"auto\">\n");
    out.push_str("    <polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#555\"/>\n");
    out.push_str("  </marker></defs>\n");

    for (i, entry) in nodes.iter().enumerate() {
        let x = i * 200 + 20;
        let y = 60_usize;
        out.push_str(&format!(
            "  <rect x=\"{}\" y=\"{}\" width=\"160\" height=\"40\" rx=\"4\" fill=\"#eef6f9\" stroke=\"#555\"/>\n",
            x, y
        ));
        out.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"12\" fill=\"#222\">{}</text>\n",
            x + 80, y + 16, entry.name
        ));
        out.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#666\">{}</text>\n",
            x + 80, y + 30, entry.kind
        ));
    }

    let name_to_idx: std::collections::HashMap<&str, usize> =
        nodes.iter().enumerate().map(|(i, e)| (e.name.as_str(), i)).collect();

    for pipeline in &report.pipelines {
        let steps = &pipeline.steps;
        for i in 0..steps.len().saturating_sub(1) {
            let from_name = steps[i].as_str();
            let to_name   = steps[i + 1].as_str();
            if let (Some(&fi), Some(&ti)) = (name_to_idx.get(from_name), name_to_idx.get(to_name)) {
                let x1 = fi * 200 + 180;
                let x2 = ti * 200 + 20;
                out.push_str(&format!(
                    "  <line x1=\"{}\" y1=\"80\" x2=\"{}\" y2=\"80\" stroke=\"#555\" stroke-width=\"1.5\" marker-end=\"url(#arr)\"/>\n",
                    x1, x2
                ));
            }
        }
    }

    out.push_str("</svg>");
    out
}

/// LineageReport をインタラクティブな自己完結型 HTML にレンダリングする。
/// クリック可能な SVG ノードと JS による詳細パネルを含む。外部ライブラリ不要。
pub fn render_lineage_html(report: &LineageReport) -> String {
    let nodes: Vec<&LineageEntry> = report.transformations.iter().collect();
    let n = nodes.len();
    let svg_width = (n * 200 + 40).max(200);

    // ── SVG ──────────────────────────────────────────────────────────────────
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"160\">\n",
        svg_width
    );
    svg.push_str("  <defs><marker id=\"arr\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" orient=\"auto\">\n");
    svg.push_str("    <polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#555\"/>\n");
    svg.push_str("  </marker></defs>\n");

    for (i, entry) in nodes.iter().enumerate() {
        let x = i * 200 + 20;
        // For text nodes: escape &, <, > only
        let safe_text = entry.name.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        // For attribute values: additionally escape "
        let safe_attr = safe_text.replace('"', "&quot;");
        let safe_kind = entry.kind.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        svg.push_str(&format!(
            "  <g class=\"node\" onclick=\"showDetail(&quot;{}&quot;)\">\n",
            safe_attr
        ));
        svg.push_str(&format!(
            "    <rect x=\"{}\" y=\"70\" width=\"160\" height=\"40\" rx=\"4\"/>\n",
            x
        ));
        svg.push_str(&format!(
            "    <text x=\"{}\" y=\"86\" text-anchor=\"middle\" font-size=\"12\" fill=\"#222\">{}</text>\n",
            x + 80, safe_text
        ));
        svg.push_str(&format!(
            "    <text x=\"{}\" y=\"100\" text-anchor=\"middle\" font-size=\"10\" fill=\"#666\">{}</text>\n",
            x + 80, safe_kind
        ));
        svg.push_str("  </g>\n");
    }

    let name_to_idx: std::collections::HashMap<&str, usize> =
        nodes.iter().enumerate().map(|(i, e)| (e.name.as_str(), i)).collect();
    for pipeline in &report.pipelines {
        for i in 0..pipeline.steps.len().saturating_sub(1) {
            if let (Some(&fi), Some(&ti)) = (
                name_to_idx.get(pipeline.steps[i].as_str()),
                name_to_idx.get(pipeline.steps[i + 1].as_str()),
            ) {
                let x1 = fi * 200 + 180;
                let x2 = ti * 200 + 20;
                svg.push_str(&format!(
                    "  <line x1=\"{}\" y1=\"90\" x2=\"{}\" y2=\"90\" stroke=\"#555\" stroke-width=\"1.5\" marker-end=\"url(#arr)\"/>\n",
                    x1, x2
                ));
            }
        }
    }
    svg.push_str("</svg>\n");

    // ── JS stages data ─────────────────────────────────────────────────────
    let mut js_entries = String::new();
    for entry in &nodes {
        let effects = if entry.effects.is_empty() {
            "Pure".to_string()
        } else {
            entry
                .effects
                .iter()
                .map(|e| format!("!{}", e.trim_start_matches('!')))
                .collect::<Vec<_>>()
                .join("+")
        };
        let schema  = entry.schema.as_deref().unwrap_or("").replace('"', "\\\"");
        let sources = entry.sources.join(", ").replace('"', "\\\"");
        let sinks   = entry.sinks.join(", ").replace('"', "\\\"");
        let key     = entry.name.replace('"', "\\\"");
        js_entries.push_str(&format!(
            "  \"{}\": {{\"kind\":\"{}\",\"effects\":\"{}\",\"schema\":\"{}\",\"sources\":\"{}\",\"sinks\":\"{}\"}},\n",
            key, entry.kind, effects, schema, sources, sinks
        ));
    }

    // ── HTML 組み立て ──────────────────────────────────────────────────────
    format!(
        "<!DOCTYPE html>\n\
<html lang=\"en\">\n\
<head>\n\
<meta charset=\"utf-8\">\n\
<title>Favnir Lineage Report</title>\n\
<style>\n\
body{{font-family:sans-serif;margin:20px;background:#fff}}\n\
.node rect{{fill:#eef6f9;stroke:#555;cursor:pointer}}\n\
.node rect:hover{{fill:#d0e8f0}}\n\
.node text{{pointer-events:none}}\n\
#detail{{margin-top:20px;padding:12px;background:#f9f9f9;border:1px solid #ddd;border-radius:4px;min-height:60px}}\n\
table{{border-collapse:collapse}}\n\
td{{padding:4px 8px;border-bottom:1px solid #eee;vertical-align:top}}\n\
td:first-child{{font-weight:bold;color:#555;width:100px}}\n\
</style>\n\
</head>\n\
<body>\n\
<h1>Favnir Lineage Report</h1>\n\
{}\
<div id=\"detail\"><em>Click a stage node to see details.</em></div>\n\
<script>\n\
var stages={{{}}};\n\
function showDetail(name){{\n\
  var s=stages[name];if(!s)return;\n\
  var h='<table>';\n\
  h+='<tr><td>Name</td><td>'+name+'</td></tr>';\n\
  h+='<tr><td>Kind</td><td>'+s.kind+'</td></tr>';\n\
  h+='<tr><td>Effects</td><td>'+s.effects+'</td></tr>';\n\
  if(s.schema)h+='<tr><td>Schema</td><td>'+s.schema+'</td></tr>';\n\
  if(s.sources)h+='<tr><td>Sources</td><td>'+s.sources+'</td></tr>';\n\
  if(s.sinks)h+='<tr><td>Sinks</td><td>'+s.sinks+'</td></tr>';\n\
  h+='</table>';\n\
  document.getElementById('detail').innerHTML=h;\n\
}}\n\
</script>\n\
</body>\n\
</html>",
        svg, js_entries
    )
}

/// Mermaid / D2 ノード ID として使える文字列に変換する（英数字 + アンダースコアのみ）。
/// 先頭が数字の場合は `n_` プレフィックスを付加する。
fn sanitize_mermaid_id(name: &str) -> String {
    let sanitized: String = name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
        format!("n_{}", sanitized)
    } else if sanitized.is_empty() {
        "_node".to_string()
    } else {
        sanitized
    }
}

// ── v11000_tests (v11.0.0) — Snowflake lineage read/write distinction ─────────
#[cfg(test)]
mod v11000_tests {
    use super::lineage_analysis;
    use crate::frontend::parser::Parser;

    #[test]
    fn lineage_snowflake_write_stage_shows_write_label() {
        let src = r#"
stage Insert: List<String> -> Int = |rows| {
  snowflake.execute("INSERT INTO T VALUES (?)")
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let report = lineage_analysis(&prog);
        let entry = report
            .transformations
            .iter()
            .find(|e| e.name == "Insert")
            .expect("Insert not found");
        assert!(
            entry.effects.contains(&"!Snowflake(write)".to_string()),
            "expected(write) in effects, got: {:?}",
            entry.effects
        );
        assert!(
            entry.sinks.iter().any(|s| s.contains("snowflake-write")),
            "expected snowflake-write in sinks"
        );
    }

    #[test]
    fn lineage_snowflake_read_stage_shows_read_label() {
        let src = r#"
stage Query: String -> List<String> = |sql| {
  snowflake.query(sql)
}
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let report = lineage_analysis(&prog);
        let entry = report
            .transformations
            .iter()
            .find(|e| e.name == "Query")
            .expect("Query not found");
        assert!(
            entry.effects.contains(&"!Snowflake(read)".to_string()),
            "expected(read) in effects, got: {:?}",
            entry.effects
        );
        assert!(
            entry.sources.iter().any(|s| s.contains("snowflake-read")),
            "expected snowflake-read in sources"
        );
    }

    #[test]
    fn lineage_snowflake_undistinguished_falls_back() {
        // v34.8A: !Snowflake annotation removed (E0374); undistinguished fallback is no longer possible.
        // A stage without snowflake calls has no Snowflake effects in lineage.
        let src = r#"
stage Sf: String -> String = |x| { x }
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let report = lineage_analysis(&prog);
        let entry = report
            .transformations
            .iter()
            .find(|e| e.name == "Sf")
            .expect("Sf not found");
        // Without annotation or snowflake calls, effects should be empty
        assert!(
            !entry.effects.iter().any(|e| e.contains("Snowflake")),
            "expected no Snowflake in effects for pure stage, got: {:?}",
            entry.effects
        );
    }
}
