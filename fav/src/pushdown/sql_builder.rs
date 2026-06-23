// pushdown/sql_builder.rs — SQL string generation for pushdown plans
// Produces SQL templates using `?pushdown_table?` as the table placeholder.
// At runtime the VM replaces `?pushdown_table?` with `_batch_{id}`.

use super::{CmpOp, FilterExpr, PushdownOp, SqlLiteral};

const TABLE_PLACEHOLDER: &str = "?pushdown_table?";

// ---------------------------------------------------------------------------
// Top-level builder
// ---------------------------------------------------------------------------

/// Generate a SQL template for the given pushdown operation.
pub fn build_sql(op: &PushdownOp) -> String {
    match op {
        PushdownOp::Count => {
            format!("SELECT COUNT(*) FROM {TABLE_PLACEHOLDER}")
        }
        PushdownOp::SumBy(field) => {
            format!("SELECT SUM({}) FROM {TABLE_PLACEHOLDER}", quote_ident(field))
        }
        PushdownOp::GroupBy(key) => {
            format!("SELECT DISTINCT {} FROM {TABLE_PLACEHOLDER}", quote_ident(key))
        }
        PushdownOp::Project(fields) => {
            let cols = fields.iter().map(|f| quote_ident(f)).collect::<Vec<_>>().join(", ");
            format!("SELECT {cols} FROM {TABLE_PLACEHOLDER}")
        }
        PushdownOp::Filter(expr) => {
            let where_clause = build_filter_where(expr);
            format!("SELECT * FROM {TABLE_PLACEHOLDER} WHERE {where_clause}")
        }
    }
}

// ---------------------------------------------------------------------------
// Filter WHERE clause builder
// ---------------------------------------------------------------------------

pub fn build_filter_where(expr: &FilterExpr) -> String {
    match expr {
        FilterExpr::FieldCmp { field, op, literal } => {
            format!("{} {} {}", quote_ident(field), build_cmp_op(op), build_literal(literal))
        }
        FilterExpr::And(left, right) => {
            format!("({}) AND ({})", build_filter_where(left), build_filter_where(right))
        }
        FilterExpr::Or(left, right) => {
            format!("({}) OR ({})", build_filter_where(left), build_filter_where(right))
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn build_cmp_op(op: &CmpOp) -> &'static str {
    match op {
        CmpOp::Gt => ">",
        CmpOp::Ge => ">=",
        CmpOp::Lt => "<",
        CmpOp::Le => "<=",
        CmpOp::Eq => "=",
        CmpOp::Ne => "<>",
    }
}

pub fn build_literal(lit: &SqlLiteral) -> String {
    match lit {
        SqlLiteral::Int(n) => n.to_string(),
        SqlLiteral::Float(f) => {
            // Use Rust's Display which produces a decimal point for floats
            if f.fract() == 0.0 {
                format!("{f}.0")
            } else {
                f.to_string()
            }
        }
        SqlLiteral::Str(s) => format!("'{}'", escape_sql_str(s)),
        SqlLiteral::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
    }
}

/// Escape single quotes in SQL string literals by doubling them.
fn escape_sql_str(s: &str) -> String {
    s.replace('\'', "''")
}

/// Quote a SQL identifier with double quotes, escaping any embedded double quotes.
/// DuckDB uses `"identifier"` syntax for quoted identifiers.
fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
