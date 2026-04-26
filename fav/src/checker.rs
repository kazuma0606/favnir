// Favnir Type Checker
// Tasks: 4-1..4-20

use std::collections::HashMap;
use crate::ast::*;
use crate::lexer::Span;

// ── Type (4-1) ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Bool,
    Int,
    Float,
    String,
    Unit,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    /// `A -> B`  (single-param function type in type expressions)
    Arrow(Box<Type>, Box<Type>),
    /// Named function definition with parameter list
    Fn(Vec<Type>, Box<Type>),
    /// `trf` definition: input, output, effect
    Trf(Box<Type>, Box<Type>, Option<Effect>),
    /// User-defined named type (after lookup)
    Named(String, Vec<Type>),
    /// Type is not yet known (monomorphic placeholder / built-in generic)
    Unknown,
    /// Error recovery — suppress cascading errors
    Error,
}

impl Type {
    /// Two types are compatible if they are equal, or if either side is Unknown/Error.
    /// Recurses into wrapper types (Option, Result, List, etc.) so that
    /// `Option<Unknown>` is compatible with `Option<Int>`.
    pub fn is_compatible(&self, other: &Type) -> bool {
        if matches!(self, Type::Unknown | Type::Error)
            || matches!(other, Type::Unknown | Type::Error)
        {
            return true;
        }
        match (self, other) {
            (Type::Option(a), Type::Option(b)) => a.is_compatible(b),
            (Type::List(a),   Type::List(b))   => a.is_compatible(b),
            (Type::Result(a1, a2), Type::Result(b1, b2)) => {
                a1.is_compatible(b1) && a2.is_compatible(b2)
            }
            (Type::Arrow(ai, ao), Type::Arrow(bi, bo)) => {
                ai.is_compatible(bi) && ao.is_compatible(bo)
            }
            _ => self == other,
        }
    }

    pub fn display(&self) -> String {
        match self {
            Type::Bool   => "Bool".into(),
            Type::Int    => "Int".into(),
            Type::Float  => "Float".into(),
            Type::String => "String".into(),
            Type::Unit   => "Unit".into(),
            Type::List(t)    => format!("List<{}>", t.display()),
            Type::Map(k, v)  => format!("Map<{}, {}>", k.display(), v.display()),
            Type::Option(t)  => format!("{}?", t.display()),
            Type::Result(t, e) => format!("Result<{}, {}>", t.display(), e.display()),
            Type::Arrow(a, b) => format!("{} -> {}", a.display(), b.display()),
            Type::Fn(params, ret) => {
                let ps: Vec<_> = params.iter().map(|p| p.display()).collect();
                format!("({}) -> {}", ps.join(", "), ret.display())
            }
            Type::Trf(i, o, fx) => {
                let eff = fx.as_ref().map(|e| format!(" !{:?}", e)).unwrap_or_default();
                format!("Trf<{}, {}{}>", i.display(), o.display(), eff)
            }
            Type::Named(n, args) if args.is_empty() => n.clone(),
            Type::Named(n, args) => {
                let s: Vec<_> = args.iter().map(|a| a.display()).collect();
                format!("{}<{}>", n, s.join(", "))
            }
            Type::Unknown => "_".into(),
            Type::Error   => "?".into(),
        }
    }

    /// If this type is a callable (Trf / Arrow / Fn), return (input, output).
    /// For Fn, returns the first param as "input" and return type as "output"
    /// (used in pipeline position where trfs are single-input).
    pub fn as_callable(&self) -> Option<(&Type, &Type)> {
        match self {
            Type::Trf(i, o, _) => Some((i, o)),
            Type::Arrow(i, o)  => Some((i, o)),
            Type::Fn(params, ret) if !params.is_empty() => Some((&params[0], ret)),
            _ => None,
        }
    }
}

// ── Effect composition (4-2) ──────────────────────────────────────────────────

/// Compose two effects: Pure is identity, Io + Io = Io.
pub fn compose_effects(a: Option<Effect>, b: Option<Effect>) -> Option<Effect> {
    match (a, b) {
        (None, e) | (e, None) => e,
        (Some(Effect::Pure), e) | (e, Some(Effect::Pure)) => e,
        (Some(Effect::Io), Some(Effect::Io)) => Some(Effect::Io),
    }
}

// ── TyEnv (4-3) ───────────────────────────────────────────────────────────────

pub struct TyEnv {
    scopes: Vec<HashMap<String, Type>>,
}

impl TyEnv {
    pub fn new() -> Self {
        TyEnv { scopes: vec![HashMap::new()] }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }
}

// ── TypeError (4-19) ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct TypeError {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

impl TypeError {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        TypeError { code, message: message.into(), span }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error[{}]: {}\n  --> {}:{}:{}",
            self.code, self.message,
            self.span.file, self.span.line, self.span.col
        )
    }
}

// ── Checker ───────────────────────────────────────────────────────────────────

pub struct Checker {
    env: TyEnv,
    pub errors: Vec<TypeError>,
    /// User-defined type bodies, for field and variant lookup.
    type_defs: HashMap<String, TypeBody>,
}

impl Checker {
    fn new() -> Self {
        Checker {
            env: TyEnv::new(),
            errors: Vec::new(),
            type_defs: HashMap::new(),
        }
    }

    pub fn check_program(program: &Program) -> Vec<TypeError> {
        let mut c = Checker::new();
        c.register_builtins();
        c.register_item_signatures(program);
        for item in &program.items {
            c.check_item(item);
        }
        c.errors
    }

    fn type_error(&mut self, code: &'static str, msg: impl Into<String>, span: &Span) {
        self.errors.push(TypeError::new(code, msg, span.clone()));
    }

    // ── built-in registration (4-4, 4-5) ─────────────────────────────────────

    fn register_builtins(&mut self) {
        // IO namespace functions are handled specially in check_builtin_apply.
        // Register placeholder so "IO" resolves to something.
        self.env.define("IO".into(), Type::Named("IO".into(), vec![]));

        // List, String, Option, Result namespace placeholders.
        for ns in &["List", "String", "Option", "Result"] {
            self.env.define(ns.to_string(), Type::Named(ns.to_string(), vec![]));
        }
    }

    // ── first-pass: register top-level names (4-6..4-9) ─────────────────────

    fn register_item_signatures(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::TypeDef(td) => {
                    self.type_defs.insert(td.name.clone(), td.body.clone());
                    self.env.define(td.name.clone(), Type::Named(td.name.clone(), vec![]));
                    // Register variant constructors so they resolve in expressions
                    if let TypeBody::Sum(variants) = &td.body {
                        let parent = Type::Named(td.name.clone(), vec![]);
                        for v in variants {
                            match v {
                                Variant::Unit(name, _) => {
                                    self.env.define(name.clone(), parent.clone());
                                }
                                Variant::Tuple(name, te, _) => {
                                    let payload = self.resolve_type_expr(te);
                                    self.env.define(name.clone(), Type::Fn(vec![payload], Box::new(parent.clone())));
                                }
                                Variant::Record(name, fields, _) => {
                                    let field_tys: Vec<Type> = fields.iter()
                                        .map(|f| self.resolve_type_expr(&f.ty))
                                        .collect();
                                    self.env.define(name.clone(), Type::Fn(field_tys, Box::new(parent.clone())));
                                }
                            }
                        }
                    }
                }
                Item::FnDef(fd) => {
                    let params: Vec<Type> = fd.params.iter()
                        .map(|p| self.resolve_type_expr(&p.ty))
                        .collect();
                    let ret = self.resolve_type_expr(&fd.return_ty);
                    self.env.define(fd.name.clone(), Type::Fn(params, Box::new(ret)));
                }
                Item::TrfDef(td) => {
                    let input  = self.resolve_type_expr(&td.input_ty);
                    let output = self.resolve_type_expr(&td.output_ty);
                    let effect = td.effect.clone();
                    self.env.define(
                        td.name.clone(),
                        Type::Trf(Box::new(input), Box::new(output), effect),
                    );
                }
                Item::FlwDef(fd) => {
                    // Compute flw type from its steps; register Unknown for now,
                    // will be refined during check_flw_def.
                    self.env.define(fd.name.clone(), Type::Unknown);
                }
            }
        }
    }

    // ── item checking ─────────────────────────────────────────────────────────

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::TypeDef(td) => self.check_type_def(td),
            Item::FnDef(fd)   => self.check_fn_def(fd),
            Item::TrfDef(td)  => self.check_trf_def(td),
            Item::FlwDef(fd)  => self.check_flw_def(fd),
        }
    }

    // ── type_def (4-6) ────────────────────────────────────────────────────────

    fn check_type_def(&mut self, _td: &TypeDef) {
        // Type definitions are structurally valid if they parsed correctly.
        // Field types are resolved lazily during use.
    }

    // ── fn_def (4-7) ──────────────────────────────────────────────────────────

    fn check_fn_def(&mut self, fd: &FnDef) {
        self.env.push();

        // Bind parameters
        for p in &fd.params {
            let ty = self.resolve_type_expr(&p.ty);
            self.env.define(p.name.clone(), ty);
        }

        let body_ty    = self.check_block(&fd.body);
        let return_ty  = self.resolve_type_expr(&fd.return_ty);

        if !body_ty.is_compatible(&return_ty) {
            self.type_error(
                "E001",
                format!(
                    "fn `{}`: body type `{}` does not match return type `{}`",
                    fd.name, body_ty.display(), return_ty.display()
                ),
                &fd.body.span,
            );
        }

        self.env.pop();
    }

    // ── trf_def (4-8) ─────────────────────────────────────────────────────────

    fn check_trf_def(&mut self, td: &TrfDef) {
        self.env.push();

        // Bind the closure parameters.  For trfs the first (and usually only)
        // param receives the trf's input type.
        let input_ty = self.resolve_type_expr(&td.input_ty);
        if let Some(p) = td.params.first() {
            self.env.define(p.name.clone(), input_ty);
        }
        for p in td.params.iter().skip(1) {
            let ty = self.resolve_type_expr(&p.ty);
            self.env.define(p.name.clone(), ty);
        }

        let body_ty   = self.check_block(&td.body);
        let output_ty = self.resolve_type_expr(&td.output_ty);

        if !body_ty.is_compatible(&output_ty) {
            self.type_error(
                "E001",
                format!(
                    "trf `{}`: body type `{}` does not match output type `{}`",
                    td.name, body_ty.display(), output_ty.display()
                ),
                &td.body.span,
            );
        }

        self.env.pop();
    }

    // ── flw_def (4-9) ─────────────────────────────────────────────────────────

    fn check_flw_def(&mut self, fd: &FlwDef) {
        if fd.steps.is_empty() {
            return;
        }

        let mut current_output: Option<Type> = None;

        for step_name in &fd.steps {
            match self.env.lookup(step_name).cloned() {
                None => {
                    self.type_error(
                        "E002",
                        format!("undefined: `{}`", step_name),
                        &fd.span,
                    );
                    current_output = Some(Type::Error);
                }
                Some(ty) => {
                    // Verify the connection: previous output must match this step's input.
                    if let Some(prev_out) = &current_output {
                        if let Some((input, _output)) = ty.as_callable() {
                            if !prev_out.is_compatible(input) {
                                self.type_error(
                                    "E003",
                                    format!(
                                        "flw `{}`: `{}` outputs `{}` but `{}` expects `{}`",
                                        fd.name,
                                        // previous step name
                                        fd.steps[fd.steps.iter().position(|s| s == step_name).unwrap().saturating_sub(1)],
                                        prev_out.display(),
                                        step_name,
                                        input.display(),
                                    ),
                                    &fd.span,
                                );
                            }
                        } else {
                            self.type_error(
                                "E003",
                                format!("`{}` is not a trf or fn, cannot be used in flw", step_name),
                                &fd.span,
                            );
                        }
                    }
                    // Advance current output.
                    current_output = ty.as_callable().map(|(_, o)| o.clone())
                        .or(Some(ty));
                }
            }
        }

        // Register the resolved flw type.
        if let Some(last_name) = fd.steps.last() {
            if let Some(last_ty) = self.env.lookup(last_name).cloned() {
                if let Some(first_name) = fd.steps.first() {
                    if let Some(first_ty) = self.env.lookup(first_name).cloned() {
                        let input  = first_ty.as_callable().map(|(i, _)| i.clone()).unwrap_or(Type::Unknown);
                        let output = last_ty.as_callable().map(|(_, o)| o.clone()).unwrap_or(Type::Unknown);
                        self.env.define(
                            fd.name.clone(),
                            Type::Trf(Box::new(input), Box::new(output), None),
                        );
                    }
                }
            }
        }
    }

    // ── block (4-17) ──────────────────────────────────────────────────────────

    fn check_block(&mut self, block: &Block) -> Type {
        self.env.push();
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        let ty = self.check_expr(&block.expr);
        self.env.pop();
        ty
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Bind(b) => {
                let expr_ty = self.check_expr(&b.expr);
                self.check_pattern_bindings(&b.pattern, &expr_ty);
            }
            Stmt::Expr(e) => {
                self.check_expr(e);
            }
        }
    }

    // ── pattern bindings (4-10, 4-11) ────────────────────────────────────────

    /// Walk a pattern and define bindings in the current scope.
    fn check_pattern_bindings(&mut self, pat: &Pattern, ty: &Type) {
        match pat {
            Pattern::Wildcard(_) => {}
            Pattern::Lit(_, _) => {}

            Pattern::Bind(name, _) => {
                self.env.define(name.clone(), ty.clone());
            }

            Pattern::Variant(name, inner, span) => {
                // Look up the variant to find the payload type.
                let payload_ty = self.resolve_variant_payload(name, ty, span);
                if let Some(inner_pat) = inner {
                    self.check_pattern_bindings(inner_pat, &payload_ty);
                }
            }

            Pattern::Record(fields, _) => {
                for fp in fields {
                    let field_ty = self.lookup_field_type(ty, &fp.name);
                    match &fp.pattern {
                        Some(p) => self.check_pattern_bindings(p, &field_ty),
                        None    => self.env.define(fp.name.clone(), field_ty),
                    }
                }
            }
        }
    }

    /// Determine the payload type of a variant pattern.
    fn resolve_variant_payload(&self, variant_name: &str, scrutinee_ty: &Type, _span: &Span) -> Type {
        let type_name = match scrutinee_ty {
            Type::Named(n, _) => n.clone(),
            Type::Option(inner) => {
                // some(x) → inner type; none → Unit
                if variant_name == "some" { return *inner.clone(); }
                if variant_name == "none" { return Type::Unit; }
                return Type::Unknown;
            }
            Type::Result(ok, err) => {
                if variant_name == "ok"  { return *ok.clone(); }
                if variant_name == "err" { return *err.clone(); }
                return Type::Unknown;
            }
            _ => return Type::Unknown,
        };

        if let Some(body) = self.type_defs.get(&type_name) {
            if let TypeBody::Sum(variants) = body {
                for v in variants {
                    if v.name() == variant_name {
                        return match v {
                            Variant::Unit(_, _)         => Type::Unit,
                            Variant::Tuple(_, te, _)    => self.resolve_type_expr(te),
                            Variant::Record(_, fields, _) => {
                                // Record variant payload — keep as Named for field lookup
                                Type::Named(type_name.clone(), vec![])
                            }
                        };
                    }
                }
            }
        }
        Type::Unknown
    }

    /// Look up a field type on a record type.
    fn lookup_field_type(&self, ty: &Type, field: &str) -> Type {
        let type_name = match ty {
            Type::Named(n, _) => n.clone(),
            _ => return Type::Unknown,
        };
        if let Some(body) = self.type_defs.get(&type_name) {
            match body {
                TypeBody::Record(fields) => {
                    for f in fields {
                        if f.name == field {
                            return self.resolve_type_expr(&f.ty);
                        }
                    }
                }
                TypeBody::Sum(variants) => {
                    for v in variants {
                        if let Variant::Record(_, fields, _) = v {
                            for f in fields {
                                if f.name == field {
                                    return self.resolve_type_expr(&f.ty);
                                }
                            }
                        }
                    }
                }
            }
        }
        Type::Unknown
    }

    // ── expr (4-14..4-16) ────────────────────────────────────────────────────

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            // literals (4-15)
            Expr::Lit(lit, _) => match lit {
                Lit::Int(_)   => Type::Int,
                Lit::Float(_) => Type::Float,
                Lit::Str(_)   => Type::String,
                Lit::Bool(_)  => Type::Bool,
                Lit::Unit     => Type::Unit,
            },

            // identifier (4-15)
            Expr::Ident(name, span) => {
                match self.env.lookup(name).cloned() {
                    Some(ty) => ty,
                    None => {
                        self.type_error("E002", format!("undefined: `{}`", name), span);
                        Type::Error
                    }
                }
            }

            // field access: expr.field (4-15)
            Expr::FieldAccess(obj, field, _span) => {
                let obj_ty = self.check_expr(obj);
                self.resolve_field_access(&obj_ty, field)
            }

            // function application (4-15)
            Expr::Apply(func, args, span) => {
                // Check for built-in namespaced calls first.
                if let Some(ty) = self.check_builtin_apply(func, args, span) {
                    return ty;
                }

                let func_ty = self.check_expr(func);
                let arg_tys: Vec<Type> = args.iter().map(|a| self.check_expr(a)).collect();

                match &func_ty {
                    Type::Fn(params, ret) => {
                        if params.len() != arg_tys.len() {
                            self.type_error(
                                "E001",
                                format!(
                                    "expected {} argument(s), got {}",
                                    params.len(), arg_tys.len()
                                ),
                                span,
                            );
                        } else {
                            for (i, (p, a)) in params.iter().zip(arg_tys.iter()).enumerate() {
                                if !a.is_compatible(p) {
                                    self.type_error(
                                        "E001",
                                        format!(
                                            "argument {}: expected `{}`, got `{}`",
                                            i + 1, p.display(), a.display()
                                        ),
                                        span,
                                    );
                                }
                            }
                        }
                        *ret.clone()
                    }
                    Type::Arrow(input, output) => {
                        let arg_ty = arg_tys.first().cloned().unwrap_or(Type::Unit);
                        if !arg_ty.is_compatible(input) {
                            self.type_error(
                                "E001",
                                format!(
                                    "expected `{}`, got `{}`",
                                    input.display(), arg_ty.display()
                                ),
                                span,
                            );
                        }
                        *output.clone()
                    }
                    Type::Trf(input, output, _) => {
                        let arg_ty = arg_tys.first().cloned().unwrap_or(Type::Unit);
                        if !arg_ty.is_compatible(input) {
                            self.type_error(
                                "E001",
                                format!(
                                    "expected `{}`, got `{}`",
                                    input.display(), arg_ty.display()
                                ),
                                span,
                            );
                        }
                        *output.clone()
                    }
                    Type::Unknown | Type::Error => Type::Unknown,
                    other => {
                        self.type_error(
                            "E001",
                            format!("`{}` is not callable", other.display()),
                            span,
                        );
                        Type::Error
                    }
                }
            }

            // pipeline: a |> b |> c  (4-14)
            Expr::Pipeline(parts, span) => {
                if parts.is_empty() {
                    return Type::Unit;
                }
                let mut current = self.check_expr(&parts[0]);
                for step in &parts[1..] {
                    let step_ty = self.check_expr(step);
                    match step_ty.as_callable() {
                        None if matches!(step_ty, Type::Unknown | Type::Error) => {
                            current = Type::Unknown;
                        }
                        None => {
                            self.type_error(
                                "E003",
                                format!("`{}` is not callable in pipeline", step_ty.display()),
                                span,
                            );
                            current = Type::Error;
                        }
                        Some((input, output)) => {
                            if !current.is_compatible(input) {
                                self.type_error(
                                    "E003",
                                    format!(
                                        "pipeline type mismatch: `{}` → `{}` (expected `{}`)",
                                        current.display(), step_ty.display(), input.display()
                                    ),
                                    span,
                                );
                                current = Type::Error;
                            } else {
                                current = output.clone();
                            }
                        }
                    }
                }
                current
            }

            // block
            Expr::Block(block) => self.check_block(block),

            // match (4-12)
            Expr::Match(scrutinee, arms, span) => {
                let scrutinee_ty = self.check_expr(scrutinee);
                self.check_match_arms(arms, &scrutinee_ty, span)
            }

            // if (4-13)
            Expr::If(cond, then_block, else_block, span) => {
                let cond_ty = self.check_expr(cond);
                if !cond_ty.is_compatible(&Type::Bool) {
                    self.type_error(
                        "E001",
                        format!("if condition must be Bool, got `{}`", cond_ty.display()),
                        span,
                    );
                }
                let then_ty = self.check_block(then_block);
                match else_block {
                    Some(else_b) => {
                        let else_ty = self.check_block(else_b);
                        if !then_ty.is_compatible(&else_ty) {
                            self.type_error(
                                "E001",
                                format!(
                                    "if branches have different types: `{}` vs `{}`",
                                    then_ty.display(), else_ty.display()
                                ),
                                span,
                            );
                            Type::Error
                        } else {
                            then_ty
                        }
                    }
                    None => Type::Unit,
                }
            }

            // closure (4-16)
            Expr::Closure(params, body, _span) => {
                self.env.push();
                for p in params {
                    self.env.define(p.clone(), Type::Unknown);
                }
                let body_ty = self.check_expr(body);
                self.env.pop();
                // Represent as Arrow(Unknown, body_ty) for single-param closures
                let input_ty = if params.len() == 1 { Type::Unknown } else { Type::Unit };
                Type::Arrow(Box::new(input_ty), Box::new(body_ty))
            }

            // binary op
            Expr::BinOp(op, lhs, rhs, span) => {
                let l = self.check_expr(lhs);
                let r = self.check_expr(rhs);
                self.check_binop(op, &l, &r, span)
            }
        }
    }

    // ── match arm consistency (4-12) ─────────────────────────────────────────

    fn check_match_arms(&mut self, arms: &[MatchArm], scrutinee_ty: &Type, span: &Span) -> Type {
        if arms.is_empty() {
            return Type::Unit;
        }
        let mut result_ty: Option<Type> = None;
        for arm in arms {
            self.env.push();
            self.check_pattern_bindings(&arm.pattern, scrutinee_ty);
            let arm_ty = self.check_expr(&arm.body);
            self.env.pop();

            match &result_ty {
                None => result_ty = Some(arm_ty),
                Some(prev) => {
                    if !prev.is_compatible(&arm_ty) {
                        self.type_error(
                            "E001",
                            format!(
                                "match arms have inconsistent types: `{}` vs `{}`",
                                prev.display(), arm_ty.display()
                            ),
                            span,
                        );
                        result_ty = Some(Type::Error);
                    }
                }
            }
        }
        result_ty.unwrap_or(Type::Unit)
    }

    // ── binary operators ──────────────────────────────────────────────────────

    fn check_binop(&mut self, op: &BinOp, l: &Type, r: &Type, span: &Span) -> Type {
        use BinOp::*;
        match op {
            Add | Sub | Mul | Div => {
                let numeric = matches!(l, Type::Int | Type::Float | Type::Unknown | Type::Error)
                    && matches!(r, Type::Int | Type::Float | Type::Unknown | Type::Error);
                if !numeric && !l.is_compatible(r) {
                    self.type_error(
                        "E001",
                        format!("arithmetic on non-numeric types `{}` and `{}`", l.display(), r.display()),
                        span,
                    );
                    return Type::Error;
                }
                if matches!(l, Type::Float) || matches!(r, Type::Float) { Type::Float } else { Type::Int }
            }
            Eq | NotEq | Lt | Gt | LtEq | GtEq => {
                if !l.is_compatible(r) {
                    self.type_error(
                        "E001",
                        format!("comparison between incompatible types `{}` and `{}`", l.display(), r.display()),
                        span,
                    );
                }
                Type::Bool
            }
        }
    }

    // ── field access resolution ───────────────────────────────────────────────

    fn resolve_field_access(&self, obj_ty: &Type, field: &str) -> Type {
        match obj_ty {
            // Namespace placeholders — return Unknown; actual type resolved at Apply time.
            Type::Named(n, _) if matches!(n.as_str(), "IO" | "List" | "String" | "Option" | "Result") => {
                Type::Unknown
            }
            // User-defined record
            Type::Named(_, _) => self.lookup_field_type(obj_ty, field),
            _ => Type::Unknown,
        }
    }

    // ── built-in call handling (4-5) ──────────────────────────────────────────

    /// If `func` is a known built-in namespace call (IO.println etc.), type-check
    /// the arguments and return the result type.  Returns None if not a built-in.
    fn check_builtin_apply(
        &mut self,
        func: &Expr,
        args: &[Expr],
        span: &Span,
    ) -> Option<Type> {
        let (ns, method) = match func {
            Expr::FieldAccess(obj, method, _) => {
                if let Expr::Ident(ns, _) = obj.as_ref() {
                    (ns.clone(), method.clone())
                } else {
                    return None;
                }
            }
            _ => return None,
        };

        let arg_tys: Vec<Type> = args.iter().map(|a| self.check_expr(a)).collect();

        match (ns.as_str(), method.as_str()) {
            // IO
            ("IO", "print") | ("IO", "println") => {
                if let Some(ty) = arg_tys.first() {
                    if !ty.is_compatible(&Type::String) {
                        self.type_error("E001", format!("IO.{} expects String, got `{}`", method, ty.display()), span);
                    }
                } else {
                    self.type_error("E001", format!("IO.{} requires one argument", method), span);
                }
                Some(Type::Unit)
            }

            // List
            ("List", "length") | ("List", "is_empty") => {
                let _ = self.expect_list_arg(&arg_tys, 0, span);
                Some(if method == "is_empty" { Type::Bool } else { Type::Int })
            }
            ("List", "first") | ("List", "last") => {
                let elem = self.expect_list_arg(&arg_tys, 0, span);
                Some(Type::Option(Box::new(elem)))
            }
            ("List", "map") => {
                let elem = self.expect_list_arg(&arg_tys, 0, span);
                // arg1 should be a function T -> U; result is List<U>
                let out = if let Some(f_ty) = arg_tys.get(1) {
                    f_ty.as_callable().map(|(_, o)| o.clone()).unwrap_or(Type::Unknown)
                } else {
                    Type::Unknown
                };
                Some(Type::List(Box::new(out)))
            }
            ("List", "filter") => {
                let elem = self.expect_list_arg(&arg_tys, 0, span);
                Some(Type::List(Box::new(elem)))
            }
            ("List", "fold") => {
                // fold(items, init, f) → type of init
                let init_ty = arg_tys.get(1).cloned().unwrap_or(Type::Unknown);
                Some(init_ty)
            }

            // String
            ("String", "trim") | ("String", "lower") | ("String", "upper") => {
                Some(Type::String)
            }
            ("String", "split") => {
                Some(Type::List(Box::new(Type::String)))
            }
            ("String", "length") => Some(Type::Int),
            ("String", "is_empty") => Some(Type::Bool),

            // Option
            ("Option", "some") => {
                let ty = arg_tys.first().cloned().unwrap_or(Type::Unknown);
                Some(Type::Option(Box::new(ty)))
            }
            ("Option", "none") => Some(Type::Option(Box::new(Type::Unknown))),
            ("Option", "map") => {
                let out = arg_tys.get(1)
                    .and_then(|f| f.as_callable().map(|(_, o)| o.clone()))
                    .unwrap_or(Type::Unknown);
                Some(Type::Option(Box::new(out)))
            }
            ("Option", "unwrap_or") => {
                let default_ty = arg_tys.get(1).cloned().unwrap_or(Type::Unknown);
                Some(default_ty)
            }

            // Result
            ("Result", "ok") => {
                let ty = arg_tys.first().cloned().unwrap_or(Type::Unknown);
                Some(Type::Result(Box::new(ty), Box::new(Type::Named("Error".into(), vec![]))))
            }
            ("Result", "err") => {
                Some(Type::Result(
                    Box::new(Type::Unknown),
                    Box::new(Type::Named("Error".into(), vec![])),
                ))
            }
            ("Result", "map") => {
                let out = arg_tys.get(1)
                    .and_then(|f| f.as_callable().map(|(_, o)| o.clone()))
                    .unwrap_or(Type::Unknown);
                Some(Type::Result(
                    Box::new(out),
                    Box::new(Type::Named("Error".into(), vec![])),
                ))
            }
            ("Result", "unwrap_or") => {
                let default_ty = arg_tys.get(1).cloned().unwrap_or(Type::Unknown);
                Some(default_ty)
            }

            _ => None,
        }
    }

    fn expect_list_arg(&mut self, arg_tys: &[Type], idx: usize, span: &Span) -> Type {
        match arg_tys.get(idx) {
            Some(Type::List(elem)) => *elem.clone(),
            Some(Type::Unknown)    => Type::Unknown,
            Some(other) => {
                self.type_error("E001", format!("expected List<_>, got `{}`", other.display()), span);
                Type::Error
            }
            None => {
                self.type_error("E001", "missing List argument", span);
                Type::Error
            }
        }
    }

    // ── type expression resolution (4-18) ────────────────────────────────────

    /// Convert a `TypeExpr` (AST surface) into a `Type` (internal).
    /// `T?` → `Option<T>`, `T!` → `Result<T, Error>`.
    pub fn resolve_type_expr(&self, te: &TypeExpr) -> Type {
        match te {
            TypeExpr::Optional(inner, _) => {
                Type::Option(Box::new(self.resolve_type_expr(inner)))
            }
            TypeExpr::Fallible(inner, _) => {
                Type::Result(
                    Box::new(self.resolve_type_expr(inner)),
                    Box::new(Type::Named("Error".into(), vec![])),
                )
            }
            TypeExpr::Arrow(a, b, _) => {
                Type::Arrow(
                    Box::new(self.resolve_type_expr(a)),
                    Box::new(self.resolve_type_expr(b)),
                )
            }
            TypeExpr::Named(name, args, _) => {
                let resolved_args: Vec<Type> = args.iter().map(|a| self.resolve_type_expr(a)).collect();
                match name.as_str() {
                    "Bool"    => Type::Bool,
                    "Int"     => Type::Int,
                    "Float"   => Type::Float,
                    "String"  => Type::String,
                    "Unit"    => Type::Unit,
                    "List"    => Type::List(Box::new(resolved_args.into_iter().next().unwrap_or(Type::Unknown))),
                    "Map"     => {
                        let mut it = resolved_args.into_iter();
                        let k = it.next().unwrap_or(Type::Unknown);
                        let v = it.next().unwrap_or(Type::Unknown);
                        Type::Map(Box::new(k), Box::new(v))
                    }
                    "Option"  => Type::Option(Box::new(resolved_args.into_iter().next().unwrap_or(Type::Unknown))),
                    "Result"  => {
                        let mut it = resolved_args.into_iter();
                        let t = it.next().unwrap_or(Type::Unknown);
                        let e = it.next().unwrap_or(Type::Named("Error".into(), vec![]));
                        Type::Result(Box::new(t), Box::new(e))
                    }
                    "_infer"  => Type::Unknown,
                    _         => Type::Named(name.clone(), resolved_args),
                }
            }
        }
    }
}

// ── Tests (4-20) ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn check(src: &str) -> Vec<String> {
        let prog = Parser::parse_str(src, "test").expect("parse error");
        Checker::check_program(&prog)
            .into_iter()
            .map(|e| format!("[{}] {}", e.code, e.message))
            .collect()
    }

    fn check_ok(src: &str) {
        let errs = check(src);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    fn check_err(src: &str) -> Vec<String> {
        let errs = check(src);
        assert!(!errs.is_empty(), "expected type errors but got none");
        errs
    }

    // 4-4, 4-5: built-in types and functions
    #[test]
    fn test_builtin_io_println() {
        check_ok(r#"
            public fn main() -> Unit !Io {
                IO.println("hello")
            }
        "#);
    }

    // 4-6: type definitions
    #[test]
    fn test_type_def_ok() {
        check_ok("type User = { name: String email: String }");
    }

    // 4-7: fn return type mismatch
    #[test]
    fn test_fn_return_mismatch() {
        let errs = check_err("fn f() -> Int { \"not an int\" }");
        assert!(errs[0].contains("E001"));
    }

    // 4-7: fn return type matches
    #[test]
    fn test_fn_return_ok() {
        check_ok("fn f() -> Int { 42 }");
    }

    // 4-8: trf type checks
    #[test]
    fn test_trf_ok() {
        check_ok("trf Double: Int -> Int = |n| { n }");
    }

    // 4-9: flw pipeline — compatible
    #[test]
    fn test_flw_ok() {
        check_ok("
            trf A: String -> Int = |s| { 0 }
            trf B: Int -> Bool   = |n| { true }
            flw AB = A |> B
        ");
    }

    // 4-9: flw pipeline — type mismatch
    #[test]
    fn test_flw_type_mismatch() {
        let errs = check_err("
            trf A: String -> Int  = |s| { 0 }
            trf B: Bool   -> Unit = |b| { () }
            flw Bad = A |> B
        ");
        assert!(errs.iter().any(|e| e.contains("E003")));
    }

    // 4-9: flw — undefined step
    #[test]
    fn test_flw_undefined_step() {
        let errs = check_err("flw Bad = NoSuchTrf |> AnotherMissing");
        assert!(errs.iter().any(|e| e.contains("E002")));
    }

    // 4-10: bind infers type
    #[test]
    fn test_bind_type_inferred() {
        check_ok("fn f() -> Int { bind x <- 42; x }");
    }

    // 4-11: pattern binding — record
    #[test]
    fn test_pattern_record_bind() {
        check_ok("
            type User = { name: String }
            fn f(u: User) -> String { bind { name } <- u; name }
        ");
    }

    // 4-12: match arm types consistent
    #[test]
    fn test_match_consistent_arms() {
        check_ok("
            type Color = | Red | Blue
            fn f(c: Color) -> Int {
                match c {
                    Red  => 0
                    Blue => 1
                }
            }
        ");
    }

    // 4-12: match arm type mismatch
    #[test]
    fn test_match_inconsistent_arms() {
        let errs = check_err("
            type Color = | Red | Blue
            fn f(c: Color) -> Int {
                match c {
                    Red  => 0
                    Blue => \"not an int\"
                }
            }
        ");
        assert!(errs.iter().any(|e| e.contains("E001")));
    }

    // 4-13: if branch type mismatch
    #[test]
    fn test_if_branch_mismatch() {
        let errs = check_err("fn f() -> Int { if true { 1 } else { \"wrong\" } }");
        assert!(errs.iter().any(|e| e.contains("E001")));
    }

    // 4-13: if branches match
    #[test]
    fn test_if_branches_ok() {
        check_ok("fn f() -> Int { if true { 1 } else { 2 } }");
    }

    // 4-14: pipeline type mismatch
    #[test]
    fn test_pipeline_type_mismatch() {
        let errs = check_err("
            trf A: String -> Int  = |s| { 0 }
            trf B: Bool   -> Unit = |b| { () }
            fn f() -> Unit { \"hello\" |> A |> B }
        ");
        assert!(errs.iter().any(|e| e.contains("E003") || e.contains("E001")));
    }

    // 4-15: undefined identifier
    #[test]
    fn test_undefined_ident() {
        let errs = check_err("fn f() -> Int { undefined_var }");
        assert!(errs.iter().any(|e| e.contains("E002")));
    }

    // 4-16: closure infers body type
    #[test]
    fn test_closure_type() {
        check_ok("fn f() -> Bool { bind g <- |x| true; g(1) }");
    }

    // 4-17: block returns last expr
    #[test]
    fn test_block_return() {
        check_ok("fn f() -> Int { bind x <- 1; bind y <- 2; x }");
    }

    // 4-18: T? expands to Option<T>
    #[test]
    fn test_optional_type() {
        check_ok("fn f() -> Int? { Option.none() }");
    }

    // 4-18: T! expands to Result<T, Error>
    #[test]
    fn test_fallible_type() {
        check_ok("fn f() -> Int! { Result.ok(42) }");
    }

    // 4-2: effect composition
    #[test]
    fn test_effect_composition() {
        assert_eq!(compose_effects(None, None), None);
        assert_eq!(compose_effects(Some(Effect::Pure), Some(Effect::Io)), Some(Effect::Io));
        assert_eq!(compose_effects(Some(Effect::Io), Some(Effect::Io)), Some(Effect::Io));
        assert_eq!(compose_effects(Some(Effect::Pure), Some(Effect::Pure)), Some(Effect::Pure));
    }
}
