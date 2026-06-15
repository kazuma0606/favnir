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
#[derive(Debug, Clone, Serialize)]
pub struct LineageReport {
    pub transformations: Vec<LineageEntry>,
    pub pipelines: Vec<PipelineLineage>,
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn format_effects(effects: &[ast::Effect]) -> String {
    use ast::Effect::*;
    if effects.is_empty() {
        return "Pure".into();
    }
    effects
        .iter()
        .map(|e| match e {
            Pure => "!Pure".into(),
            Io => "!Io".into(),
            Db => "!Db".into(),
            DbRead => "!DbRead".into(),
            DbWrite => "!DbWrite".into(),
            DbAdmin => "!DbAdmin".into(),
            Network => "!Network".into(),
            Http => "!Http".into(),
            Llm => "!Llm".into(),
            Snowflake => "!Snowflake".into(),
            Gcp => "!Gcp".into(),
            Stream => "!Stream".into(),
            Postgres => "!Postgres".into(),
            AzureDb => "!AzureDb".into(),
            AzureStorage => "!AzureStorage".into(),
            Rpc => "!Rpc".into(),
            File => "!File".into(),
            Checkpoint => "!Checkpoint".into(),
            Unknown(name) => format!("!{}", name),
            Emit(ev) => format!("!Emit<{}>", ev),
            EmitUnion(evs) => format!("!Emit<{}>", evs.join("|")),
            Trace => "!Trace".into(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_db_read_effect(e: &ast::Effect) -> bool {
    matches!(e, ast::Effect::Db | ast::Effect::DbRead)
}

fn is_db_write_effect(e: &ast::Effect) -> bool {
    matches!(e, ast::Effect::Db | ast::Effect::DbWrite | ast::Effect::DbAdmin)
}

/// Classify a function/stage by capability kind based on parameter types and effects.
/// Returns (kind, capability): e.g. ("read", Some("DbRead")), ("transform", None).
fn classify_capability_kind(
    params: &[ast::Param],
    effects: &[ast::Effect],
) -> (String, Option<String>) {
    // First: check parameter type names for ctx-based capabilities (v13.x design)
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
    // Fallback: classify by legacy effect annotations
    for e in effects {
        match e {
            ast::Effect::DbWrite | ast::Effect::DbAdmin => {
                return ("write".into(), Some("DbWrite".into()))
            }
            ast::Effect::Db | ast::Effect::DbRead => {
                return ("read".into(), Some("DbRead".into()))
            }
            ast::Effect::Postgres | ast::Effect::Snowflake | ast::Effect::Gcp => {
                return ("read".into(), Some("DbRead".into()))
            }
            ast::Effect::Stream => {
                return ("io".into(), Some("StreamWrite".into()))
            }
            ast::Effect::Network | ast::Effect::Http | ast::Effect::Llm | ast::Effect::Rpc => {
                return ("io".into(), Some("HttpClient".into()))
            }
            ast::Effect::Io | ast::Effect::File => return ("io".into(), Some("Io".into())),
            _ => {}
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
        ast::Stmt::ForIn(f) => {
            collect_sql_literals_inner(&f.iter, out);
            collect_sql_literals_block(&f.body, out);
        }
    }
}

// ── AzureDb read/write classification (v14.1.0) ───────────────────────────────

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
        ast::Stmt::ForIn(f) => {
            collect_azure_kinds_inner(&f.iter, r, w);
            for s in &f.body.stmts {
                collect_azure_kinds_stmt(s, r, w);
            }
            collect_azure_kinds_inner(&f.body.expr, r, w);
        }
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
        ast::Stmt::ForIn(f) => {
            collect_azure_blob_kinds_inner(&f.iter, r, w);
            for s in &f.body.stmts { collect_azure_blob_kinds_stmt(s, r, w); }
            collect_azure_blob_kinds_inner(&f.body.expr, r, w);
        }
    }
}

/// Replace `!AzureDb` with `!AzureDb(read)` / `!AzureDb(write)` where known.
fn azure_db_effects(
    effects: &[ast::Effect],
    az_read: bool,
    az_write: bool,
) -> Vec<String> {
    effects
        .iter()
        .flat_map(|e| {
            if matches!(e, ast::Effect::AzureDb) && (az_read || az_write) {
                let mut v = Vec::new();
                if az_read {
                    v.push("!AzureDb(read)".to_string());
                }
                if az_write {
                    v.push("!AzureDb(write)".to_string());
                }
                v
            } else {
                vec![format_effects(std::slice::from_ref(e))]
            }
        })
        .collect()
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
        ast::Stmt::ForIn(f) => {
            collect_sf_kinds_inner(&f.iter, r, w);
            for s in &f.body.stmts {
                collect_sf_kinds_stmt(s, r, w);
            }
            collect_sf_kinds_inner(&f.body.expr, r, w);
        }
    }
}

/// Given a stage/fn effect list and its body, produce the final effects string list,
/// replacing `!Snowflake` with `!Snowflake(read)` / `!Snowflake(write)` where possible.
fn snowflake_effects(
    effects: &[ast::Effect],
    sf_read: bool,
    sf_write: bool,
) -> Vec<String> {
    effects
        .iter()
        .flat_map(|e| {
            if matches!(e, ast::Effect::Snowflake) && (sf_read || sf_write) {
                let mut v = Vec::new();
                if sf_read {
                    v.push("!Snowflake(read)".to_string());
                }
                if sf_write {
                    v.push("!Snowflake(write)".to_string());
                }
                v
            } else {
                vec![format_effects(std::slice::from_ref(e))]
            }
        })
        .collect()
}

/// Combine Snowflake, AzureDb, and AzureStorage read/write classification in one pass.
fn combined_effects(
    effects: &[ast::Effect],
    sf_read: bool, sf_write: bool,
    az_read: bool, az_write: bool,
    az_blob_read: bool, az_blob_write: bool,
) -> Vec<String> {
    effects
        .iter()
        .flat_map(|e| {
            if matches!(e, ast::Effect::Snowflake) && (sf_read || sf_write) {
                let mut v = Vec::new();
                if sf_read  { v.push("!Snowflake(read)".to_string()); }
                if sf_write { v.push("!Snowflake(write)".to_string()); }
                v
            } else if matches!(e, ast::Effect::AzureDb) && (az_read || az_write) {
                let mut v = Vec::new();
                if az_read  { v.push("!AzureDb(read)".to_string()); }
                if az_write { v.push("!AzureDb(write)".to_string()); }
                v
            } else if matches!(e, ast::Effect::AzureStorage) {
                let mut v = Vec::new();
                if az_blob_read || az_blob_write {
                    if az_blob_read  { v.push("!AzureStorage(read)".to_string()); }
                    if az_blob_write { v.push("!AzureStorage(write)".to_string()); }
                } else {
                    v.push("!AzureStorage".to_string());
                }
                v
            } else {
                vec![format_effects(std::slice::from_ref(e))]
            }
        })
        .collect()
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
            let has_read = trf.effects.iter().any(is_db_read_effect);
            let has_write = trf.effects.iter().any(is_db_write_effect);
            if sources.is_empty() && has_read {
                sources.push(format!("({}:db-read)", trf.name));
            }
            if sinks.is_empty() && has_write {
                sinks.push(format!("({}:db-write)", trf.name));
            }
            // Snowflake read/write classification
            let has_snowflake = trf.effects.iter().any(|e| matches!(e, ast::Effect::Snowflake));
            let (sf_read, sf_write) = if has_snowflake {
                collect_snowflake_call_kinds(&ast::Expr::Block(Box::new(trf.body.clone())))
            } else {
                (false, false)
            };
            if sf_read {
                sources.push(format!("({}:snowflake-read)", trf.name));
            }
            if sf_write {
                sinks.push(format!("({}:snowflake-write)", trf.name));
            }
            // AzureDb read/write classification (v14.1.0)
            let has_azure_db = trf.effects.iter().any(|e| matches!(e, ast::Effect::AzureDb));
            let (az_read, az_write) = if has_azure_db {
                collect_azure_call_kinds(&ast::Expr::Block(Box::new(trf.body.clone())))
            } else {
                (false, false)
            };
            if az_read {
                sources.push(format!("({}:azure-db-read)", trf.name));
            }
            if az_write {
                sinks.push(format!("({}:azure-db-write)", trf.name));
            }
            // AzureBlob read/write classification (v14.3.0)
            let has_azure_blob = trf.effects.iter().any(|e| matches!(e, ast::Effect::AzureStorage));
            let (az_blob_read, az_blob_write) = if has_azure_blob {
                collect_azure_blob_call_kinds(&ast::Expr::Block(Box::new(trf.body.clone())))
            } else {
                (false, false)
            };
            if az_blob_read {
                sources.push(format!("({}:azure-blob-read)", trf.name));
            }
            if az_blob_write {
                sinks.push(format!("({}:azure-blob-write)", trf.name));
            }
            let (cap_kind, cap_name) = classify_capability_kind(&trf.params, &trf.effects);
            transformations.push(LineageEntry {
                name: trf.name.clone(),
                kind: cap_kind,
                capability: cap_name,
                effects: combined_effects(&trf.effects, sf_read, sf_write, az_read, az_write, az_blob_read, az_blob_write),
                sources,
                sinks,
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
            let has_read = fndef.effects.iter().any(is_db_read_effect);
            let has_write = fndef.effects.iter().any(is_db_write_effect);
            if sources.is_empty() && has_read {
                sources.push(format!("({}:db-read)", fndef.name));
            }
            if sinks.is_empty() && has_write {
                sinks.push(format!("({}:db-write)", fndef.name));
            }
            // Snowflake read/write classification
            let has_snowflake = fndef.effects.iter().any(|e| matches!(e, ast::Effect::Snowflake));
            let (sf_read, sf_write) = if has_snowflake {
                collect_snowflake_call_kinds(&ast::Expr::Block(Box::new(fndef.body.clone())))
            } else {
                (false, false)
            };
            if sf_read {
                sources.push(format!("({}:snowflake-read)", fndef.name));
            }
            if sf_write {
                sinks.push(format!("({}:snowflake-write)", fndef.name));
            }
            // AzureDb read/write classification (v14.1.0)
            let has_azure_db = fndef.effects.iter().any(|e| matches!(e, ast::Effect::AzureDb));
            let (az_read, az_write) = if has_azure_db {
                collect_azure_call_kinds(&ast::Expr::Block(Box::new(fndef.body.clone())))
            } else {
                (false, false)
            };
            if az_read {
                sources.push(format!("({}:azure-db-read)", fndef.name));
            }
            if az_write {
                sinks.push(format!("({}:azure-db-write)", fndef.name));
            }
            // AzureBlob read/write classification (v14.3.0)
            let has_azure_blob = fndef.effects.iter().any(|e| matches!(e, ast::Effect::AzureStorage));
            let (az_blob_read, az_blob_write) = if has_azure_blob {
                collect_azure_blob_call_kinds(&ast::Expr::Block(Box::new(fndef.body.clone())))
            } else {
                (false, false)
            };
            if az_blob_read {
                sources.push(format!("({}:azure-blob-read)", fndef.name));
            }
            if az_blob_write {
                sinks.push(format!("({}:azure-blob-write)", fndef.name));
            }
            if !fndef.effects.is_empty() {
                let (cap_kind, cap_name) = classify_capability_kind(&fndef.params, &fndef.effects);
                transformations.push(LineageEntry {
                    name: fndef.name.clone(),
                    kind: cap_kind,
                    capability: cap_name,
                    effects: combined_effects(&fndef.effects, sf_read, sf_write, az_read, az_write, az_blob_read, az_blob_write),
                    sources,
                    sinks,
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

    // CrossCloud Flow: emit when both an AWS-side DB effect and !AzureDb coexist (v14.3.0)
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

    out
}

/// Render lineage as JSON.
pub fn render_lineage_json(report: &LineageReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into())
}

// ── v11000_tests (v11.0.0) — Snowflake lineage read/write distinction ─────────
#[cfg(test)]
mod v11000_tests {
    use super::lineage_analysis;
    use crate::frontend::parser::Parser;

    #[test]
    fn lineage_snowflake_write_stage_shows_write_label() {
        let src = r#"
stage Insert: List<String> -> Int !Snowflake = |rows| {
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
            "expected !Snowflake(write) in effects, got: {:?}",
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
stage Query: String -> List<String> !Snowflake = |sql| {
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
            "expected !Snowflake(read) in effects, got: {:?}",
            entry.effects
        );
        assert!(
            entry.sources.iter().any(|s| s.contains("snowflake-read")),
            "expected snowflake-read in sources"
        );
    }

    #[test]
    fn lineage_snowflake_undistinguished_falls_back() {
        let src = r#"
stage Sf: String -> String !Snowflake = |x| { x }
"#;
        let prog = Parser::parse_str(src, "test.fav").expect("parse failed");
        let report = lineage_analysis(&prog);
        let entry = report
            .transformations
            .iter()
            .find(|e| e.name == "Sf")
            .expect("Sf not found");
        assert!(
            entry.effects.contains(&"!Snowflake".to_string()),
            "expected !Snowflake fallback in effects, got: {:?}",
            entry.effects
        );
    }
}
