use super::checker::Type;
use crate::ast::{BinOp, Lit};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub globals: Vec<IRGlobal>,
    pub fns: Vec<IRFnDef>,
}

#[derive(Debug, Clone)]
pub enum IRGlobalKind {
    Fn(usize),
    Builtin, // 組み込み（実行時に解決）
    VariantCtor,
}

#[derive(Debug, Clone)]
pub struct IRGlobal {
    pub name: String,
    pub kind: IRGlobalKind,
}

#[derive(Debug, Clone)]
pub struct IRFnDef {
    pub name: String,
    pub param_count: usize,
    pub param_tys: Vec<Type>,
    pub local_count: usize,
    pub effects: Vec<crate::ast::Effect>,
    pub return_ty: Type,
    pub body: IRExpr,
}

#[derive(Debug, Clone)]
pub enum IRExpr {
    Lit(Lit, Type),
    Local(u16, Type),
    Global(u16, Type),
    TrfRef(u16, Type),
    CallTrfLocal {
        local: u16,
        arg: Box<IRExpr>,
        ty: Type,
    },
    Call(Box<IRExpr>, Vec<IRExpr>, Type),
    Block(Vec<IRStmt>, Box<IRExpr>, Type),
    If(Box<IRExpr>, Box<IRExpr>, Box<IRExpr>, Type),
    Match(Box<IRExpr>, Vec<IRArm>, Type),
    FieldAccess(Box<IRExpr>, String, Type),
    BinOp(BinOp, Box<IRExpr>, Box<IRExpr>, Type),
    Closure(u16, Vec<IRExpr>, Type),
    Collect(Box<IRExpr>, Type),
    Emit(Box<IRExpr>, Type),
    RecordConstruct(Vec<(String, IRExpr)>, Type),
}

impl IRExpr {
    pub fn ty(&self) -> &Type {
        match self {
            IRExpr::Lit(_, ty)
            | IRExpr::Local(_, ty)
            | IRExpr::Global(_, ty)
            | IRExpr::TrfRef(_, ty)
            | IRExpr::CallTrfLocal { ty, .. }
            | IRExpr::Call(_, _, ty)
            | IRExpr::Block(_, _, ty)
            | IRExpr::If(_, _, _, ty)
            | IRExpr::Match(_, _, ty)
            | IRExpr::FieldAccess(_, _, ty)
            | IRExpr::BinOp(_, _, _, ty)
            | IRExpr::Closure(_, _, ty)
            | IRExpr::Collect(_, ty)
            | IRExpr::Emit(_, ty)
            | IRExpr::RecordConstruct(_, ty) => ty,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IRStmt {
    Bind(u16, IRExpr),
    Chain(u16, IRExpr),
    Yield(IRExpr),
    Expr(IRExpr),
    /// Coverage tracking: record that line N was executed (v1.7.0).
    TrackLine(u32),
}

#[derive(Debug, Clone)]
pub struct IRArm {
    pub pattern: IRPattern,
    pub guard: Option<IRExpr>,
    pub body: IRExpr,
}

#[derive(Debug, Clone)]
pub enum IRPattern {
    Wildcard,
    Lit(Lit),
    Bind(u16),
    Variant(String, Option<Box<IRPattern>>),
    Record(Vec<(String, IRPattern)>),
}

// ── dep collection ────────────────────────────────────────────────────────────

/// Collect all dependencies (called fns + accessed builtins) from a function's body.
/// Returns a sorted, deduplicated list of dependency names.
/// - User functions: `"fn_name"`
/// - Builtin methods: `"IO.println"`, `"List.map"`, etc.
pub fn collect_deps(fn_def: &IRFnDef, globals: &[IRGlobal]) -> Vec<String> {
    let mut deps = BTreeSet::new();
    collect_expr_deps(&fn_def.body, globals, &mut deps);
    deps.into_iter().collect()
}

pub fn collect_calls_in_ir(fn_def: &IRFnDef, globals: &[IRGlobal]) -> Vec<String> {
    collect_deps(fn_def, globals)
}

fn collect_expr_deps(expr: &IRExpr, globals: &[IRGlobal], deps: &mut BTreeSet<String>) {
    match expr {
        IRExpr::Lit(_, _) | IRExpr::Local(_, _) => {}

        IRExpr::Global(idx, _) => {
            if let Some(g) = globals.get(*idx as usize) {
                match &g.kind {
                    // User-defined functions and variant constructors
                    IRGlobalKind::Fn(_) | IRGlobalKind::VariantCtor => {
                        if !g.name.starts_with('$') {
                            deps.insert(g.name.clone());
                        }
                    }
                    // Bare namespace access — only interesting with FieldAccess above
                    IRGlobalKind::Builtin => {}
                }
            }
        }

        IRExpr::TrfRef(idx, _) => {
            if let Some(g) = globals.get(*idx as usize) {
                match &g.kind {
                    IRGlobalKind::Fn(_) | IRGlobalKind::VariantCtor => {
                        if !g.name.starts_with('$') {
                            deps.insert(g.name.clone());
                        }
                    }
                    IRGlobalKind::Builtin => {}
                }
            }
        }

        IRExpr::CallTrfLocal { arg, .. } => {
            collect_expr_deps(arg, globals, deps);
        }

        IRExpr::FieldAccess(obj, field, _) => {
            // Builtin namespace access: Global("IO", Builtin).println → "IO.println"
            if let IRExpr::Global(idx, _) = obj.as_ref() {
                if let Some(g) = globals.get(*idx as usize) {
                    if matches!(&g.kind, IRGlobalKind::Builtin) {
                        deps.insert(format!("{}.{}", g.name, field));
                        return; // don't recurse further into the namespace Global
                    }
                }
            }
            collect_expr_deps(obj, globals, deps);
        }

        IRExpr::Call(f, args, _) => {
            collect_expr_deps(f, globals, deps);
            for a in args {
                collect_expr_deps(a, globals, deps);
            }
        }

        IRExpr::Block(stmts, final_expr, _) => {
            for s in stmts {
                collect_stmt_deps(s, globals, deps);
            }
            collect_expr_deps(final_expr, globals, deps);
        }

        IRExpr::If(cond, then, else_, _) => {
            collect_expr_deps(cond, globals, deps);
            collect_expr_deps(then, globals, deps);
            collect_expr_deps(else_, globals, deps);
        }

        IRExpr::Match(scrutinee, arms, _) => {
            collect_expr_deps(scrutinee, globals, deps);
            for arm in arms {
                if let Some(g) = &arm.guard {
                    collect_expr_deps(g, globals, deps);
                }
                collect_expr_deps(&arm.body, globals, deps);
            }
        }

        IRExpr::BinOp(_, lhs, rhs, _) => {
            collect_expr_deps(lhs, globals, deps);
            collect_expr_deps(rhs, globals, deps);
        }

        IRExpr::Closure(global_idx, captures, _) => {
            if let Some(g) = globals.get(*global_idx as usize) {
                deps.insert(g.name.clone());
            }
            for c in captures {
                collect_expr_deps(c, globals, deps);
            }
        }

        IRExpr::Collect(inner, _) | IRExpr::Emit(inner, _) => {
            collect_expr_deps(inner, globals, deps);
        }

        IRExpr::RecordConstruct(fields, _) => {
            for (_, e) in fields {
                collect_expr_deps(e, globals, deps);
            }
        }
    }
}

fn collect_stmt_deps(stmt: &IRStmt, globals: &[IRGlobal], deps: &mut BTreeSet<String>) {
    match stmt {
        IRStmt::Bind(_, e) | IRStmt::Chain(_, e) | IRStmt::Yield(e) | IRStmt::Expr(e) => {
            collect_expr_deps(e, globals, deps);
        }
        IRStmt::TrackLine(_) => {}
    }
}
