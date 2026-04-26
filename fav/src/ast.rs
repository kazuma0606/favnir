// Favnir AST
// Tasks: 2-1..2-13

use crate::lexer::Span;

// ── Visibility ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}

// ── Effect (2-12) ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Pure,
    Io,
}

// ── TypeExpr (2-2) ────────────────────────────────────────────────────────────

/// A type expression in source code.
///
/// Named("List", [Named("Row", [], ..)], ..)  →  List<Row>
/// Optional(Named("User", [], ..), ..)         →  User?
/// Fallible(Named("User", [], ..), ..)         →  User!
/// Arrow(A, B, ..)                             →  A -> B
#[derive(Debug, Clone)]
pub enum TypeExpr {
    Named(String, Vec<TypeExpr>, Span),
    Optional(Box<TypeExpr>, Span),
    Fallible(Box<TypeExpr>, Span),
    Arrow(Box<TypeExpr>, Box<TypeExpr>, Span),
}

impl TypeExpr {
    pub fn span(&self) -> &Span {
        match self {
            TypeExpr::Named(_, _, s)    => s,
            TypeExpr::Optional(_, s)    => s,
            TypeExpr::Fallible(_, s)    => s,
            TypeExpr::Arrow(_, _, s)    => s,
        }
    }
}

// ── Field ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}

// ── Variant (2-4) ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Variant {
    /// `Guest`
    Unit(String, Span),
    /// `ok(User)`
    Tuple(String, TypeExpr, Span),
    /// `Authenticated { user: User }`
    Record(String, Vec<Field>, Span),
}

impl Variant {
    pub fn name(&self) -> &str {
        match self {
            Variant::Unit(n, _)      => n,
            Variant::Tuple(n, _, _)  => n,
            Variant::Record(n, _, _) => n,
        }
    }

    pub fn span(&self) -> &Span {
        match self {
            Variant::Unit(_, s)      => s,
            Variant::Tuple(_, _, s)  => s,
            Variant::Record(_, _, s) => s,
        }
    }
}

// ── TypeBody / TypeDef (2-3) ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum TypeBody {
    Record(Vec<Field>),
    Sum(Vec<Variant>),
}

/// `type User = { ... }` or `type Session = | Guest | Authenticated { ... }`
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub body: TypeBody,
    pub span: Span,
}

// ── Param ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}

// ── Literal ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Unit,
}

// ── Pattern (2-10) ────────────────────────────────────────────────────────────

/// Field pattern inside `{ ... }`.
/// `{ name }` is shorthand for `{ name: name }` — pattern is None in that case.
#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: Option<Pattern>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    /// `_`
    Wildcard(Span),
    /// `42`, `"hi"`, `true`, `()`
    Lit(Lit, Span),
    /// `user`  (plain name — binds the value)
    Bind(String, Span),
    /// `ok(p)` or `Guest` (no payload)
    Variant(String, Option<Box<Pattern>>, Span),
    /// `{ name, email }` or `{ name: p }`
    Record(Vec<FieldPattern>, Span),
}

impl Pattern {
    pub fn span(&self) -> &Span {
        match self {
            Pattern::Wildcard(s)       => s,
            Pattern::Lit(_, s)         => s,
            Pattern::Bind(_, s)        => s,
            Pattern::Variant(_, _, s)  => s,
            Pattern::Record(_, s)      => s,
        }
    }
}

// ── MatchArm (2-11) ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

// ── Expr (2-9) ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    /// `42`, `"hello"`, `true`, `()`
    Lit(Lit, Span),

    /// `foo`, `ParseCsv`
    Ident(String, Span),

    /// `expr |> expr |> expr`  (always 2 or more elements)
    Pipeline(Vec<Expr>, Span),

    /// `f(a, b)`
    Apply(Box<Expr>, Vec<Expr>, Span),

    /// `expr.field`
    FieldAccess(Box<Expr>, String, Span),

    /// `{ stmt* expr }`
    Block(Box<Block>),

    /// `match expr { arm+ }`
    Match(Box<Expr>, Vec<MatchArm>, Span),

    /// `if cond { ... } else { ... }`
    If(Box<Expr>, Box<Block>, Option<Box<Block>>, Span),

    /// `|x, y| expr`
    Closure(Vec<String>, Box<Expr>, Span),

    /// binary operators: +, -, *, /, ==, !=, <, >, <=, >=
    BinOp(BinOp, Box<Expr>, Box<Expr>, Span),
}

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::Lit(_, s)              => s,
            Expr::Ident(_, s)            => s,
            Expr::Pipeline(_, s)         => s,
            Expr::Apply(_, _, s)         => s,
            Expr::FieldAccess(_, _, s)   => s,
            Expr::Block(b)               => &b.span,
            Expr::Match(_, _, s)         => s,
            Expr::If(_, _, _, s)         => s,
            Expr::Closure(_, _, s)       => s,
            Expr::BinOp(_, _, _, s)      => s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, NotEq,
    Lt, Gt, LtEq, GtEq,
}

// ── Stmt ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    Bind(BindStmt),
    Expr(Expr),
}

// ── BindStmt (2-8) ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BindStmt {
    pub pattern: Pattern,
    pub expr: Expr,
    pub span: Span,
}

// ── Block ─────────────────────────────────────────────────────────────────────

/// `{ stmt* expr }`
/// The final `expr` is the block's value.
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Box<Expr>,
    pub span: Span,
}

// ── FnDef (2-5) ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FnDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: TypeExpr,
    pub effect: Option<Effect>,
    pub body: Block,
    pub span: Span,
}

// ── TrfDef (2-6) ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TrfDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub input_ty: TypeExpr,
    pub output_ty: TypeExpr,
    pub effect: Option<Effect>,
    pub params: Vec<Param>,
    pub body: Block,
    pub span: Span,
}

// ── FlwDef (2-7) ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FlwDef {
    pub name: String,
    /// Ordered list of trf/fn names joined by `|>`.
    pub steps: Vec<String>,
    pub span: Span,
}

// ── Item (2-1) ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Item {
    TypeDef(TypeDef),
    FnDef(FnDef),
    TrfDef(TrfDef),
    FlwDef(FlwDef),
}

impl Item {
    pub fn span(&self) -> &Span {
        match self {
            Item::TypeDef(t) => &t.span,
            Item::FnDef(f)   => &f.span,
            Item::TrfDef(t)  => &t.span,
            Item::FlwDef(f)  => &f.span,
        }
    }
}

// ── Program ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}
