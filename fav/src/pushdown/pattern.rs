// pushdown/pattern.rs — AST pattern matchers for pushdown detection
// Recognizes List.filter / map / group_by / sum_by / length calls that can be
// pushed down to DuckDB.

use crate::ast::{BinOp, Expr, Lit};
use super::{CmpOp, FilterExpr, SqlLiteral};

// ---------------------------------------------------------------------------
// Public pattern analyzers (called from mod.rs detect_pushdown)
// ---------------------------------------------------------------------------

/// Detect `List.filter(param, |r| cond)` and return the filter expression tree.
pub fn analyze_filter(expr: &Expr, param: &str) -> Option<FilterExpr> {
    // Expect: Apply(FieldAccess(Ident("List"), "filter"), [Ident(param), Closure(...)])
    let (func, args) = extract_list_call(expr, "filter")?;
    let _ = func;
    if args.len() != 2 {
        return None;
    }
    if !is_param_first(&args[0], param) {
        return None;
    }
    // Second arg must be a closure: Closure([lp], body, _)
    let (lp, body) = extract_closure(&args[1])?;
    analyze_filter_expr(body, &lp)
}

/// Detect `List.map(param, |r| { f1: r.f1, ... })` and return field name list.
pub fn analyze_project(expr: &Expr, param: &str) -> Option<Vec<String>> {
    let (_, args) = extract_list_call(expr, "map")?;
    if args.len() != 2 {
        return None;
    }
    if !is_param_first(&args[0], param) {
        return None;
    }
    let (_, body) = extract_closure(&args[1])?;
    extract_projection_fields(body)
}

/// Detect `List.group_by(param, |r| r.key)` and return the key field name.
pub fn analyze_group_by(expr: &Expr, param: &str) -> Option<String> {
    let (_, args) = extract_list_call(expr, "group_by")?;
    if args.len() != 2 {
        return None;
    }
    if !is_param_first(&args[0], param) {
        return None;
    }
    let (lp, body) = extract_closure(&args[1])?;
    extract_field_access(body, &lp)
}

/// Detect `List.sum_by(param, |r| r.val)` and return the value field name.
pub fn analyze_sum_by(expr: &Expr, param: &str) -> Option<String> {
    let (_, args) = extract_list_call(expr, "sum_by")?;
    if args.len() != 2 {
        return None;
    }
    if !is_param_first(&args[0], param) {
        return None;
    }
    let (lp, body) = extract_closure(&args[1])?;
    extract_field_access(body, &lp)
}

/// Detect `List.length(param)` and return true.
pub fn analyze_count(expr: &Expr, param: &str) -> bool {
    if let Some((_, args)) = extract_list_call(expr, "length") {
        if args.len() == 1 && is_param_first(&args[0], param) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Filter expression analysis
// ---------------------------------------------------------------------------

fn analyze_filter_expr(expr: &Expr, lp: &str) -> Option<FilterExpr> {
    match expr {
        Expr::BinOp(op, left, right, _) => match op {
            BinOp::And => {
                let l = analyze_filter_expr(left, lp)?;
                let r = analyze_filter_expr(right, lp)?;
                Some(FilterExpr::And(Box::new(l), Box::new(r)))
            }
            BinOp::Or => {
                let l = analyze_filter_expr(left, lp)?;
                let r = analyze_filter_expr(right, lp)?;
                Some(FilterExpr::Or(Box::new(l), Box::new(r)))
            }
            BinOp::Gt | BinOp::GtEq | BinOp::Lt | BinOp::LtEq | BinOp::Eq | BinOp::NotEq => {
                let cmp_op = binop_to_cmp(op)?;
                // Either side can be the field access; the other must be a literal.
                if let (Some(field), Some(lit)) =
                    (try_field_access(left, lp), try_literal(right))
                {
                    Some(FilterExpr::FieldCmp { field, op: cmp_op, literal: lit })
                } else if let (Some(field), Some(lit)) =
                    (try_field_access(right, lp), try_literal(left))
                {
                    // Swap operands: flip the comparison direction
                    let flipped = flip_cmp_op(cmp_op);
                    Some(FilterExpr::FieldCmp { field, op: flipped, literal: lit })
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Helper: extract `List.method(...)` call
// Returns (func_expr, args) where func_expr is `FieldAccess(Ident("List"), method)`.
// ---------------------------------------------------------------------------

fn extract_list_call<'a>(expr: &'a Expr, method: &str) -> Option<(&'a Expr, &'a [Expr])> {
    if let Expr::Apply(func, args, _) = expr {
        if let Expr::FieldAccess(base, name, _) = func.as_ref() {
            if let Expr::Ident(ns, _) = base.as_ref() {
                if ns == "List" && name == method {
                    return Some((func.as_ref(), args.as_slice()));
                }
            }
        }
    }
    None
}

/// Extract `|lp| body` closure. Returns (param_name, body_expr).
fn extract_closure(expr: &Expr) -> Option<(String, &Expr)> {
    if let Expr::Closure(params, body, _) = expr {
        if params.len() == 1 {
            return Some((params[0].clone(), body.as_ref()));
        }
    }
    None
}

/// Check that expr is `Ident(param)`.
fn is_param_first(expr: &Expr, param: &str) -> bool {
    matches!(expr, Expr::Ident(name, _) if name == param)
}

/// Try to extract a field access `lp.field` → Some("field").
fn try_field_access(expr: &Expr, lp: &str) -> Option<String> {
    if let Expr::FieldAccess(base, field, _) = expr {
        if let Expr::Ident(name, _) = base.as_ref() {
            if name == lp {
                return Some(field.clone());
            }
        }
    }
    None
}

/// Extract `lp.field` where lp matches `param`. Used in closure bodies.
fn extract_field_access(expr: &Expr, lp: &str) -> Option<String> {
    try_field_access(expr, lp)
}

/// Try to extract a literal from an expression.
fn try_literal(expr: &Expr) -> Option<SqlLiteral> {
    extract_literal(expr)
}

fn extract_literal(expr: &Expr) -> Option<SqlLiteral> {
    if let Expr::Lit(lit, _) = expr {
        match lit {
            Lit::Int(n) => return Some(SqlLiteral::Int(*n)),
            Lit::Float(f) => return Some(SqlLiteral::Float(*f)),
            Lit::Str(s) => return Some(SqlLiteral::Str(s.clone())),
            Lit::Bool(b) => return Some(SqlLiteral::Bool(*b)),
            Lit::Unit => {}
        }
    }
    None
}

/// Extract field names from a record construction expression.
/// Supports `TypeName { f1: expr, ... }` (RecordConstruct).
fn extract_projection_fields(expr: &Expr) -> Option<Vec<String>> {
    if let Expr::RecordConstruct(_, fields, _) = expr {
        let names: Vec<String> = fields.iter().map(|(name, _)| name.clone()).collect();
        if names.is_empty() {
            return None;
        }
        Some(names)
    } else {
        None
    }
}

fn binop_to_cmp(op: &BinOp) -> Option<CmpOp> {
    match op {
        BinOp::Gt => Some(CmpOp::Gt),
        BinOp::GtEq => Some(CmpOp::Ge),
        BinOp::Lt => Some(CmpOp::Lt),
        BinOp::LtEq => Some(CmpOp::Le),
        BinOp::Eq => Some(CmpOp::Eq),
        BinOp::NotEq => Some(CmpOp::Ne),
        _ => None,
    }
}

fn flip_cmp_op(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::Gt => CmpOp::Lt,
        CmpOp::Ge => CmpOp::Le,
        CmpOp::Lt => CmpOp::Gt,
        CmpOp::Le => CmpOp::Ge,
        CmpOp::Eq => CmpOp::Eq,
        CmpOp::Ne => CmpOp::Ne,
    }
}

// ---------------------------------------------------------------------------
// Test helper — exposed for v204000_tests
// ---------------------------------------------------------------------------

/// Build a simple `FilterExpr::FieldCmp` for use in unit tests.
pub(crate) fn make_filter_expr() -> FilterExpr {
    FilterExpr::FieldCmp {
        field: "amount".to_string(),
        op: CmpOp::Gt,
        literal: SqlLiteral::Float(1000.0),
    }
}
