// Favnir AST
// Tasks: 2-1..2-13

#![allow(dead_code)]

use crate::frontend::lexer::Span;

// ── Visibility ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Internal,
    Private,
}

// ── Effect (2-12) ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Pure,
    Io,
    Db,
    Network,
    File,
    Trace,
    /// `Emit<EventType>`
    Emit(String),
    /// `Emit<A | B>`  Eresult of composing multiple Emit effects
    EmitUnion(Vec<String>),
}

// ── TypeExpr (2-2) ────────────────────────────────────────────────────────────

/// A type expression in source code.
///
/// Named("List", [Named("Row", [], ..)], ..)  ↁE List<Row>
/// Optional(Named("User", [], ..), ..)         ↁE User?
/// Fallible(Named("User", [], ..), ..)         ↁE User!
/// Arrow(A, B, ..)                             ↁE A -> B
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
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_params: Vec<String>,   // e.g. ["T", "U"] for type Pair<T, U>
    pub with_interfaces: Vec<String>,
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
/// `{ name }` is shorthand for `{ name: name }`  Epattern is None in that case.
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
    /// `user`  (plain name  Ebinds the value)
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
    pub guard: Option<Box<Expr>>,   // v0.5.0: optional `where guard_expr`
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

    /// `collect { stmt* expr }`
    Collect(Box<Block>, Span),

    /// `if cond { ... } else { ... }`
    If(Box<Expr>, Box<Block>, Option<Box<Block>>, Span),

    /// `|x, y| expr`
    Closure(Vec<String>, Box<Expr>, Span),

    /// binary operators: +, -, *, /, ==, !=, <, >, <=, >=
    BinOp(BinOp, Box<Expr>, Box<Expr>, Span),

    /// `TypeName { field: expr, ... }`  Erecord construction
    RecordConstruct(String, Vec<(String, Expr)>, Span),

    /// `emit expr`  Epublish an event
    EmitExpr(Box<Expr>, Span),
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
            Expr::BinOp(_, _, _, s)         => s,
            Expr::RecordConstruct(_, _, s)  => s,
            Expr::EmitExpr(_, s)            => s,
            Expr::Collect(_, s)             => s,
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
    /// `chain x <- expr`  Emonadic bind with early-exit on failure (v0.5.0)
    Chain(ChainStmt),
    /// `yield expr;`  Epush a value into the enclosing collect block (v0.5.0)
    Yield(YieldStmt),
}

// ── BindStmt (2-8) ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BindStmt {
    pub pattern: Pattern,
    pub expr: Expr,
    pub span: Span,
}

// ── ChainStmt / YieldStmt (v0.5.0) ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChainStmt {
    pub name: String,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct YieldStmt {
    pub expr: Expr,
    pub span: Span,
}

impl Stmt {
    pub fn span(&self) -> &Span {
        match self {
            Stmt::Bind(b)  => &b.span,
            Stmt::Expr(e)  => e.span(),
            Stmt::Chain(c) => &c.span,
            Stmt::Yield(y) => &y.span,
        }
    }
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
    pub type_params: Vec<String>,   // e.g. ["T", "U"] for fn f<T, U>(...)
    pub params: Vec<Param>,
    pub return_ty: TypeExpr,
    pub effects: Vec<Effect>,
    pub body: Block,
    pub span: Span,
}

// ── TrfDef (2-6) ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TrfDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_params: Vec<String>,   // e.g. ["T", "U"] for trf F<T, U>: ...
    pub input_ty: TypeExpr,
    pub output_ty: TypeExpr,
    pub effects: Vec<Effect>,
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

// ── CapDef / ImplDef (v0.4.0) ─────────────────────────────────────────────────

/// A single field declaration inside `cap Eq<T> = { equals: T -> T -> Bool }`.
#[derive(Debug, Clone)]
pub struct CapField {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}

/// A single method declaration inside an `interface`.
#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}

/// `interface Show { show: Self -> String }`
#[derive(Debug, Clone)]
pub struct InterfaceDecl {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub super_interface: Option<String>,
    pub methods: Vec<InterfaceMethod>,
    pub span: Span,
}

/// `impl Show, Eq for UserRow { ... }` or auto-synthesized `impl Show for UserRow`.
#[derive(Debug, Clone)]
pub struct InterfaceImplDecl {
    pub interface_names: Vec<String>,
    pub type_name: String,
    pub type_params: Vec<String>,
    pub methods: Vec<(String, Expr)>,
    pub is_auto: bool,
    pub span: Span,
}

/// `cap Eq<T> = { equals: T -> T -> Bool }`  Ecapability definition.
#[derive(Debug, Clone)]
pub struct CapDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_params: Vec<String>,   // ["T"] for cap Eq<T>
    pub fields: Vec<CapField>,
    pub span: Span,
}

/// `impl Eq<Int> { fn equals(a: Int, b: Int) -> Bool { ... } }`  Ecapability implementation.
#[derive(Debug, Clone)]
pub struct ImplDef {
    pub cap_name: String,
    pub type_args: Vec<TypeExpr>,   // [Int] for impl Eq<Int>
    pub methods: Vec<FnDef>,
    pub span: Span,
}

// ── TestDef (v0.8.0) ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TestDef {
    pub name: String,   // description string literal
    pub body: Block,
    pub span: Span,
}

// ── Item (2-1) ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Item {
    TypeDef(TypeDef),
    FnDef(FnDef),
    TrfDef(TrfDef),
    FlwDef(FlwDef),
    NamespaceDecl(String, Span),   // namespace data.users
    UseDecl(Vec<String>, Span),    // use data.users.create ↁE["data","users","create"]
    InterfaceDecl(InterfaceDecl),
    InterfaceImplDecl(InterfaceImplDecl),
    CapDef(CapDef),                // cap Eq<T> = { ... }
    ImplDef(ImplDef),              // impl Eq<Int> { ... }
    TestDef(TestDef),              // test "description" { ... }
}

impl Item {
    pub fn span(&self) -> &Span {
        match self {
            Item::TypeDef(t)          => &t.span,
            Item::FnDef(f)            => &f.span,
            Item::TrfDef(t)           => &t.span,
            Item::FlwDef(f)           => &f.span,
            Item::NamespaceDecl(_, s) => s,
            Item::UseDecl(_, s)       => s,
            Item::InterfaceDecl(d)     => &d.span,
            Item::InterfaceImplDecl(d) => &d.span,
            Item::CapDef(c)           => &c.span,
            Item::ImplDef(i)          => &i.span,
            Item::TestDef(t)          => &t.span,
        }
    }
}

// ── Program ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Program {
    /// Declared module path (`namespace data.users`). None = derived from file path.
    pub namespace: Option<String>,
    /// Import declarations (`use data.users.create`).
    pub uses: Vec<Vec<String>>,
    pub items: Vec<Item>,
}
