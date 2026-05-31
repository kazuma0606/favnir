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
    pub kind: String, // "stage" | "fn"
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
        ast::Expr::Lit(_, _) | ast::Expr::Ident(_, _) | ast::Expr::FString(_, _) => {}
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
            transformations.push(LineageEntry {
                name: trf.name.clone(),
                kind: "stage".into(),
                effects: trf
                    .effects
                    .iter()
                    .map(|e| format_effects(std::slice::from_ref(e)))
                    .collect(),
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
            if !fndef.effects.is_empty() {
                transformations.push(LineageEntry {
                    name: fndef.name.clone(),
                    kind: "fn".into(),
                    effects: fndef
                        .effects
                        .iter()
                        .map(|e| format_effects(std::slice::from_ref(e)))
                        .collect(),
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
                if let Some((srcs, snks)) = entry_map.get(step.as_str()) {
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
            pipelines.push(PipelineLineage {
                name: flw.name.clone(),
                steps: flw.steps.clone(),
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
            out.push_str(&format!(
                "  {} {} [{}]",
                e.kind,
                e.name,
                e.effects.join(", ")
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
