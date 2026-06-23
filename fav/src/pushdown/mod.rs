// pushdown/mod.rs — DuckDB pushdown optimization pass
// Detects AST patterns that can be delegated to DuckDB at runtime.

pub mod pattern;
pub mod sql_builder;

use crate::ast::Expr;
use pattern::{
    analyze_count, analyze_filter, analyze_group_by, analyze_project, analyze_sum_by,
};
use sql_builder::build_sql;

// ---------------------------------------------------------------------------
// Comparison operator
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CmpOp {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

// ---------------------------------------------------------------------------
// SQL literal value
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum SqlLiteral {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

// ---------------------------------------------------------------------------
// Filter expression tree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum FilterExpr {
    FieldCmp {
        field: String,
        op: CmpOp,
        literal: SqlLiteral,
    },
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
}

// ---------------------------------------------------------------------------
// Pushdown operation variants
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum PushdownOp {
    Filter(FilterExpr),
    Project(Vec<String>),
    GroupBy(String),
    SumBy(String),
    Count,
}

// ---------------------------------------------------------------------------
// Pushdown plan — SQL template + which operation it represents
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PushdownPlan {
    /// SQL template with `?pushdown_table?` as placeholder.
    /// At runtime, `?pushdown_table?` is replaced with the actual batch table name.
    pub sql: String,
    pub op: PushdownOp,
}

// ---------------------------------------------------------------------------
// Entry point: detect a pushdown-eligible pattern in a stage body
// ---------------------------------------------------------------------------

/// Attempt to detect a pushdown-eligible pattern in `body`.
/// `param_name` is the first parameter of the stage (the rows argument).
/// Returns `Some(PushdownPlan)` if a single supported pattern is found,
/// `None` otherwise.
pub fn detect_pushdown(body: &Expr, param_name: &str) -> Option<PushdownPlan> {
    // Try each pattern in priority order.

    // 1. Filter — List.filter(param, |r| cond)
    if let Some(filter_expr) = analyze_filter(body, param_name) {
        let op = PushdownOp::Filter(filter_expr);
        let sql = build_sql(&op);
        return Some(PushdownPlan { sql, op });
    }

    // 2. Project — List.map(param, |r| { f1: r.f1, ... })
    if let Some(fields) = analyze_project(body, param_name) {
        let op = PushdownOp::Project(fields);
        let sql = build_sql(&op);
        return Some(PushdownPlan { sql, op });
    }

    // 3. GroupBy — List.group_by(param, |r| r.key)
    if let Some(key) = analyze_group_by(body, param_name) {
        let op = PushdownOp::GroupBy(key);
        let sql = build_sql(&op);
        return Some(PushdownPlan { sql, op });
    }

    // 4. SumBy — List.sum_by(param, |r| r.val)
    if let Some(field) = analyze_sum_by(body, param_name) {
        let op = PushdownOp::SumBy(field);
        let sql = build_sql(&op);
        return Some(PushdownPlan { sql, op });
    }

    // 5. Count — List.length(param)
    if analyze_count(body, param_name) {
        let op = PushdownOp::Count;
        let sql = build_sql(&op);
        return Some(PushdownPlan { sql, op });
    }

    None
}
