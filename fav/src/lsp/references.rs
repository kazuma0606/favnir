use crate::ast::{Block, Expr, Item, Program, Stmt, TypeExpr};
use crate::frontend::lexer::Span;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::hover::position_to_char_offset;
use crate::lsp::protocol::{Location, Position, Range};

// ── Public entry points ───────────────────────────────────────────────────────

pub fn handle_references(store: &DocumentStore, uri: &str, pos: Position) -> Vec<Location> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return Vec::new(),
    };
    let program = match &doc.program {
        Some(p) => p,
        None => return Vec::new(),
    };
    let offset = match position_to_char_offset(&doc.source, pos) {
        Some(o) => o,
        None => return Vec::new(),
    };
    let name = match word_at_offset(&doc.source, offset) {
        Some(n) => n,
        None => return Vec::new(),
    };

    collect_symbol_occurrences(program, &name)
        .into_iter()
        .map(|span| {
            let range = span_to_range(&span);
            Location {
                uri: uri.to_string(),
                range,
            }
        })
        .collect()
}

pub fn collect_symbol_occurrences(program: &Program, name: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    for item in &program.items {
        collect_in_item(item, name, &mut spans);
    }
    spans
}

// ── Span → Range ─────────────────────────────────────────────────────────────

pub fn span_to_range(span: &Span) -> Range {
    let start_line = span.line.saturating_sub(1);
    let start_char = span.col.saturating_sub(1);
    let len = (span.end - span.start) as u32;
    Range {
        start: Position { line: start_line, character: start_char },
        end:   Position { line: start_line, character: start_char + len },
    }
}

// ── Word at cursor ────────────────────────────────────────────────────────────

pub fn word_at_offset(src: &str, offset: usize) -> Option<String> {
    let bytes = src.as_bytes();
    if offset > bytes.len() {
        return None;
    }
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';

    // find start of word
    let mut start = offset;
    while start > 0 && is_ident(bytes[start - 1]) {
        start -= 1;
    }
    // find end of word
    let mut end = offset;
    while end < bytes.len() && is_ident(bytes[end]) {
        end += 1;
    }
    if start == end {
        return None;
    }
    Some(src[start..end].to_string())
}

// ── AST traversal ────────────────────────────────────────────────────────────

fn collect_in_item(item: &Item, name: &str, spans: &mut Vec<Span>) {
    match item {
        Item::FnDef(fd) => {
            if fd.name == name {
                spans.push(fd.span.clone());
            }
            for param in &fd.params {
                collect_in_type_expr(&param.ty, name, spans);
            }
            if let Some(ret) = &fd.return_ty { collect_in_type_expr(ret, name, spans); }
            collect_in_block(&fd.body, name, spans);
        }
        Item::TrfDef(td) => {
            if td.name == name {
                spans.push(td.span.clone());
            }
            collect_in_type_expr(&td.input_ty, name, spans);
            collect_in_type_expr(&td.output_ty, name, spans);
            collect_in_block(&td.body, name, spans);
        }
        Item::TypeDef(td) => {
            if td.name == name {
                spans.push(td.span.clone());
            }
        }
        Item::TestDef(td) => {
            collect_in_block(&td.body, name, spans);
        }
        Item::TestGroup { tests, .. } => {
            for td in tests {
                collect_in_block(&td.body, name, spans);
            }
        }
        _ => {}
    }
}

fn collect_in_block(block: &Block, name: &str, spans: &mut Vec<Span>) {
    for stmt in &block.stmts {
        collect_in_stmt(stmt, name, spans);
    }
    collect_in_expr(&block.expr, name, spans);
}

fn collect_in_stmt(stmt: &Stmt, name: &str, spans: &mut Vec<Span>) {
    match stmt {
        Stmt::Bind(b) => collect_in_expr(&b.expr, name, spans),
        Stmt::Expr(e) => collect_in_expr(e, name, spans),
        Stmt::Chain(c) => collect_in_expr(&c.expr, name, spans),
        Stmt::Yield(y) => collect_in_expr(&y.expr, name, spans),
        Stmt::Return(r) => collect_in_expr(&r.expr, name, spans),
        Stmt::ForIn(f) => {
            collect_in_expr(&f.iter, name, spans);
            collect_in_block(&f.body, name, spans);
        }
        Stmt::Forall(f) => {
            if let Some(g) = &f.guard { collect_in_expr(g, name, spans); }
            collect_in_block(&f.body, name, spans);
        }
        Stmt::Expect(_) => {} // v36.2.0 — 実行は v36.3 以降
    }
}

fn collect_in_expr(expr: &Expr, name: &str, spans: &mut Vec<Span>) {
    match expr {
        Expr::Ident(n, span) => {
            if n == name {
                spans.push(span.clone());
            }
        }
        Expr::Apply(func, args, _) => {
            collect_in_expr(func, name, spans);
            for a in args { collect_in_expr(a, name, spans); }
        }
        Expr::TypeApply(func, type_args, _) => {
            collect_in_expr(func, name, spans);
            for ty in type_args { collect_in_type_expr(ty, name, spans); }
        }
        Expr::FieldAccess(obj, _, _) => collect_in_expr(obj, name, spans),
        Expr::Pipeline(steps, _) => {
            for s in steps { collect_in_expr(s, name, spans); }
        }
        Expr::Block(b) => collect_in_block(b, name, spans),
        Expr::If(cond, then, else_, _) => {
            collect_in_expr(cond, name, spans);
            collect_in_block(then, name, spans);
            if let Some(eb) = else_ { collect_in_block(eb, name, spans); }
        }
        Expr::Match(scrutinee, arms, _) => {
            collect_in_expr(scrutinee, name, spans);
            for arm in arms { collect_in_expr(&arm.body, name, spans); }
        }
        Expr::Closure(_, body, _) => collect_in_expr(body, name, spans),
        Expr::BinOp(_, l, r, _) => {
            collect_in_expr(l, name, spans);
            collect_in_expr(r, name, spans);
        }
        Expr::RecordConstruct(tname, fields, span) => {
            if tname == name { spans.push(span.clone()); }
            for (_, e) in fields { collect_in_expr(e, name, spans); }
        }
        Expr::RecordSpread(base, fields, _) => {
            collect_in_expr(base, name, spans);
            for (_, e) in fields { collect_in_expr(e, name, spans); }
        }
        Expr::Question(inner, _) => collect_in_expr(inner, name, spans),
        Expr::EmitExpr(inner, _) => collect_in_expr(inner, name, spans),
        Expr::AssertMatches(scrutinee, _, _) => collect_in_expr(scrutinee, name, spans),
        Expr::AssertSchema { arg, .. } => collect_in_expr(arg, name, spans),
        Expr::Collect(b, _) => collect_in_block(b, name, spans),
        Expr::FString(parts, _) => {
            for part in parts {
                if let crate::ast::FStringPart::Expr(e) = part {
                    collect_in_expr(e, name, spans);
                }
            }
        }
        Expr::ListComp { expr, clauses, .. } | Expr::ResultComp { expr, clauses, .. } => {
            collect_in_expr(expr, name, spans);
            for clause in clauses {
                if let crate::ast::CompClause::Guard(g) = clause {
                    collect_in_expr(g, name, spans);
                }
            }
        }
        Expr::Lit(_, _) => {}
    }
}

fn collect_in_type_expr(ty: &TypeExpr, name: &str, spans: &mut Vec<Span>) {
    match ty {
        TypeExpr::Named(n, args, span) => {
            if n == name { spans.push(span.clone()); }
            for a in args { collect_in_type_expr(a, name, spans); }
        }
        TypeExpr::Optional(inner, _) | TypeExpr::Fallible(inner, _) => {
            collect_in_type_expr(inner, name, spans);
        }
        TypeExpr::Arrow(a, b, _) | TypeExpr::LinearArrow(a, b, _) | TypeExpr::Intersection(a, b, _) => {
            collect_in_type_expr(a, name, spans);
            collect_in_type_expr(b, name, spans);
        }
        TypeExpr::TrfFn { input, output, .. } => {
            collect_in_type_expr(input, name, spans);
            collect_in_type_expr(output, name, spans);
        }
        TypeExpr::RecordType(fields, _) => {
            for (_, ty) in fields { collect_in_type_expr(ty, name, spans); }
        }
        _ => {}
    }
}
