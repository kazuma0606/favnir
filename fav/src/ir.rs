use crate::ast::{BinOp, Lit};
use crate::checker::Type;

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub globals: Vec<IRGlobal>,
    pub fns: Vec<IRFnDef>,
}

#[derive(Debug, Clone)]
pub enum IRGlobalKind {
    Fn(usize),
    Builtin,        // 組み込み（実行時に解決）
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
