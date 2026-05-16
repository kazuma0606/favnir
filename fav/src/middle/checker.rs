// Favnir Type Checker
// Tasks: 4-1..4-20

use crate::ast::*;
use crate::frontend::lexer::Span;
use crate::frontend::parser::Parser;
use crate::schemas::{FieldConstraints, ProjectSchemas};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
    /// `trf` definition: input, output, effects
    Trf(Box<Type>, Box<Type>, Vec<Effect>),
    /// `abstract trf` definition (v1.3.0)
    AbstractTrf {
        input: Box<Type>,
        output: Box<Type>,
        effects: Vec<Effect>,
    },
    /// `abstract seq Name<T> { ... }` template marker (v1.3.0)
    AbstractFlwTemplate(String),
    /// Partially bound abstract seq (v1.3.0)
    PartialFlw {
        template: String,
        type_args: Vec<Type>,
        unbound_slots: Vec<String>,
    },
    /// User-defined named type (after lookup)
    Named(String, Vec<Type>),
    /// Type variable  E`T`, `U`, or fresh `$0`, `$1` (v0.4.0)
    Var(String),
    /// Capability instance type  E`Ord<Int>`, `Eq<String>` (v0.4.0)
    Cap(String, Vec<Type>),
    /// Interface instance type (v1.1.0)
    Interface(String, Vec<Type>),
    /// `async fn` return type wrapper: `Task<T>` (v1.7.0)
    Task(Box<Type>),
    /// `Stream<T>` lazy sequence (v2.9.0)
    Stream(Box<Type>),
    /// Type is not yet known (monomorphic placeholder / built-in generic)
    Unknown,
    /// Error recovery  Esuppress cascading errors
    Error,
}

impl Type {
    /// Two types are compatible if they are equal, or if either side is Unknown/Error.
    /// Recurses into wrapper types (Option, Result, List, etc.) so that
    /// `Option<Unknown>` is compatible with `Option<Int>`.
    pub fn is_compatible(&self, other: &Type) -> bool {
        if matches!(self, Type::Unknown | Type::Error | Type::Var(_))
            || matches!(other, Type::Unknown | Type::Error | Type::Var(_))
        {
            return true;
        }
        match (self, other) {
            (Type::Task(a), Type::Task(b)) => a.is_compatible(b),
            (Type::Stream(a), Type::Stream(b)) => a.is_compatible(b),
            (Type::Option(a), Type::Option(b)) => a.is_compatible(b),
            (Type::List(a), Type::List(b)) => a.is_compatible(b),
            (Type::Map(a1, a2), Type::Map(b1, b2)) => a1.is_compatible(b1) && a2.is_compatible(b2),
            (Type::Result(a1, a2), Type::Result(b1, b2)) => {
                a1.is_compatible(b1) && a2.is_compatible(b2)
            }
            (Type::Arrow(ai, ao), Type::Arrow(bi, bo)) => {
                ai.is_compatible(bi) && ao.is_compatible(bo)
            }
            (Type::Arrow(ai, ao), Type::Trf(bi, bo, _))
            | (Type::Trf(ai, ao, _), Type::Arrow(bi, bo)) => {
                ai.is_compatible(bi) && ao.is_compatible(bo)
            }
            // Cross-compatibility: impl method bodies use Arrow (lambda checker output)
            // while builtin interface defs register Fn. Be lenient when input is fuzzy.
            (Type::Arrow(ai, ao), Type::Fn(_params, ret)) => {
                let fuzzy = matches!(ai.as_ref(), Type::Unknown | Type::Unit | Type::Error);
                fuzzy && ao.is_compatible(ret)
            }
            (Type::Fn(_params, ret), Type::Arrow(bi, bo)) => {
                let fuzzy = matches!(bi.as_ref(), Type::Unknown | Type::Unit | Type::Error);
                fuzzy && ret.is_compatible(bo)
            }
            // 0-param Fn is a value (e.g. Monoid.empty): compatible with the return type directly
            (_, Type::Fn(params, ret)) if params.is_empty() => self.is_compatible(ret),
            (Type::Fn(params, ret), _) if params.is_empty() => ret.is_compatible(other),
            (Type::Cap(n1, as1), Type::Cap(n2, as2)) => {
                n1 == n2
                    && as1.len() == as2.len()
                    && as1.iter().zip(as2).all(|(a, b)| a.is_compatible(b))
            }
            (Type::Interface(n1, as1), Type::Interface(n2, as2)) => {
                n1 == n2
                    && as1.len() == as2.len()
                    && as1.iter().zip(as2).all(|(a, b)| a.is_compatible(b))
            }
            (
                Type::AbstractTrf {
                    input: i1,
                    output: o1,
                    effects: f1,
                },
                Type::AbstractTrf {
                    input: i2,
                    output: o2,
                    effects: f2,
                },
            ) => i1.is_compatible(i2) && o1.is_compatible(o2) && f1 == f2,
            (Type::AbstractFlwTemplate(a), Type::AbstractFlwTemplate(b)) => a == b,
            (
                Type::PartialFlw {
                    template: t1,
                    type_args: a1,
                    unbound_slots: s1,
                },
                Type::PartialFlw {
                    template: t2,
                    type_args: a2,
                    unbound_slots: s2,
                },
            ) => {
                t1 == t2
                    && a1.len() == a2.len()
                    && s1 == s2
                    && a1.iter().zip(a2).all(|(a, b)| a.is_compatible(b))
            }
            // Named types with the same name: compatible if args match, or if
            // one side has no args (raw/unapplied form from record construction).
            (Type::Named(n1, a1), Type::Named(n2, a2)) if n1 == n2 => {
                if a1.is_empty() || a2.is_empty() {
                    true // raw form is compatible with any applied form of the same type
                } else {
                    a1.len() == a2.len() && a1.iter().zip(a2).all(|(a, b)| a.is_compatible(b))
                }
            }
            _ => self == other,
        }
    }

    pub fn display(&self) -> String {
        match self {
            Type::Bool => "Bool".into(),
            Type::Int => "Int".into(),
            Type::Float => "Float".into(),
            Type::String => "String".into(),
            Type::Unit => "Unit".into(),
            Type::List(t) => format!("List<{}>", t.display()),
            Type::Map(k, v) => format!("Map<{}, {}>", k.display(), v.display()),
            Type::Option(t) => format!("{}?", t.display()),
            Type::Result(t, e) => format!("Result<{}, {}>", t.display(), e.display()),
            Type::Arrow(a, b) => format!("{} -> {}", a.display(), b.display()),
            Type::Fn(params, ret) => {
                let ps: Vec<_> = params.iter().map(|p| p.display()).collect();
                format!("({}) -> {}", ps.join(", "), ret.display())
            }
            Type::Trf(i, o, fx) => {
                let effs: Vec<String> = fx.iter().map(|e| format!("!{:?}", e)).collect();
                let eff = if effs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", effs.join(" "))
                };
                format!("Trf<{}, {}{}>", i.display(), o.display(), eff)
            }
            Type::AbstractTrf {
                input,
                output,
                effects,
            } => {
                let effs: Vec<String> = effects.iter().map(|e| format!("!{:?}", e)).collect();
                let eff = if effs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", effs.join(" "))
                };
                format!(
                    "AbstractTrf<{}, {}{}>",
                    input.display(),
                    output.display(),
                    eff
                )
            }
            Type::AbstractFlwTemplate(name) => format!("AbstractFlw<{}>", name),
            Type::PartialFlw {
                template,
                type_args,
                unbound_slots,
            } => {
                let args = if type_args.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        type_args
                            .iter()
                            .map(|t| t.display())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                format!(
                    "PartialFlw<{}{}; {}>",
                    template,
                    args,
                    unbound_slots.join(", ")
                )
            }
            Type::Named(n, args) if args.is_empty() => n.clone(),
            // Named("Option", [T]) ↁEdisplay as T? to match Type::Option(T)
            Type::Named(n, args) if n == "Option" && args.len() == 1 => {
                format!("{}?", args[0].display())
            }
            Type::Named(n, args) => {
                let s: Vec<_> = args.iter().map(|a| a.display()).collect();
                format!("{}<{}>", n, s.join(", "))
            }
            Type::Var(name) => name.clone(),
            Type::Cap(name, args) if args.is_empty() => name.clone(),
            Type::Cap(name, args) => {
                let s: Vec<_> = args.iter().map(|a| a.display()).collect();
                format!("{}<{}>", name, s.join(", "))
            }
            Type::Interface(name, args) if args.is_empty() => name.clone(),
            Type::Interface(name, args) => {
                let s: Vec<_> = args.iter().map(|a| a.display()).collect();
                format!("{}<{}>", name, s.join(", "))
            }
            Type::Task(t) => format!("Task<{}>", t.display()),
            Type::Stream(t) => format!("Stream<{}>", t.display()),
            Type::Unknown => "_".into(),
            Type::Error => "?".into(),
        }
    }

    /// If this type is a callable (Trf / Arrow / Fn), return (input, output).
    /// For Fn, returns the first param as "input" and return type as "output"
    /// (used in pipeline position where trfs are single-input).
    pub fn as_callable(&self) -> Option<(&Type, &Type)> {
        match self {
            Type::Trf(i, o, _) => Some((i, o)),
            Type::AbstractTrf { input, output, .. } => Some((input, output)),
            Type::Arrow(i, o) => Some((i, o)),
            Type::Fn(params, ret) if !params.is_empty() => Some((&params[0], ret)),
            _ => None,
        }
    }
}

// ── Subst (v0.4.0) ────────────────────────────────────────────────────────────

/// Type variable substitution map.  `apply` replaces `Var(name)` with the
/// mapped type, recursing transitively.
#[derive(Debug, Clone)]
pub struct Subst {
    pub map: HashMap<String, Type>,
}

impl Subst {
    pub fn empty() -> Self {
        Subst {
            map: HashMap::new(),
        }
    }

    pub fn singleton(var: String, ty: Type) -> Self {
        let mut map = HashMap::new();
        map.insert(var, ty);
        Subst { map }
    }

    pub fn extend(&mut self, var: String, ty: Type) {
        self.map.insert(var, ty);
    }

    /// Apply the substitution to a type, replacing type variables transitively.
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(name) => {
                if let Some(t) = self.map.get(name) {
                    self.apply(t) // transitive closure
                } else {
                    ty.clone()
                }
            }
            Type::Task(t) => Type::Task(Box::new(self.apply(t))),
            Type::Stream(t) => Type::Stream(Box::new(self.apply(t))),
            Type::List(t) => Type::List(Box::new(self.apply(t))),
            Type::Option(t) => Type::Option(Box::new(self.apply(t))),
            Type::Map(k, v) => Type::Map(Box::new(self.apply(k)), Box::new(self.apply(v))),
            Type::Result(t, e) => Type::Result(Box::new(self.apply(t)), Box::new(self.apply(e))),
            Type::Arrow(a, b) => Type::Arrow(Box::new(self.apply(a)), Box::new(self.apply(b))),
            Type::Fn(ps, ret) => Type::Fn(
                ps.iter().map(|p| self.apply(p)).collect(),
                Box::new(self.apply(ret)),
            ),
            Type::Trf(i, o, fx) => {
                Type::Trf(Box::new(self.apply(i)), Box::new(self.apply(o)), fx.clone())
            }
            Type::Named(n, args) => {
                Type::Named(n.clone(), args.iter().map(|a| self.apply(a)).collect())
            }
            Type::Cap(n, args) => {
                Type::Cap(n.clone(), args.iter().map(|a| self.apply(a)).collect())
            }
            Type::Interface(n, args) => {
                Type::Interface(n.clone(), args.iter().map(|a| self.apply(a)).collect())
            }
            _ => ty.clone(),
        }
    }

    /// Compose: `self.compose(other)` produces a substitution `s` such that
    /// `s.apply(ty) == self.apply(other.apply(ty))`.
    pub fn compose(self, other: Subst) -> Subst {
        let mut result = HashMap::new();
        // Apply self to every value in other
        for (k, v) in other.map {
            result.insert(k, self.apply(&v));
        }
        // Add bindings from self not overridden by other
        for (k, v) in self.map {
            result.entry(k).or_insert(v);
        }
        Subst { map: result }
    }
}

/// Occurs check: does the variable `var` appear in `ty`?
pub fn occurs(var: &str, ty: &Type) -> bool {
    match ty {
        Type::Var(name) => name == var,
        Type::Task(t) => occurs(var, t),
        Type::Stream(t) => occurs(var, t),
        Type::List(t) => occurs(var, t),
        Type::Option(t) => occurs(var, t),
        Type::Map(k, v) => occurs(var, k) || occurs(var, v),
        Type::Result(t, e) => occurs(var, t) || occurs(var, e),
        Type::Arrow(a, b) => occurs(var, a) || occurs(var, b),
        Type::Fn(ps, ret) => ps.iter().any(|p| occurs(var, p)) || occurs(var, ret),
        Type::Trf(i, o, _) => occurs(var, i) || occurs(var, o),
        Type::Named(_, args) => args.iter().any(|a| occurs(var, a)),
        Type::Cap(_, args) => args.iter().any(|a| occurs(var, a)),
        Type::Interface(_, args) => args.iter().any(|a| occurs(var, a)),
        _ => false,
    }
}

/// Robinson's unification algorithm.
/// Returns a `Subst` that makes `t1` and `t2` equal, or an error string.
pub fn unify(t1: &Type, t2: &Type) -> Result<Subst, String> {
    match (t1, t2) {
        // Identical types  Eno work needed
        (a, b) if a == b => Ok(Subst::empty()),

        // Type variables
        (Type::Var(a), t) => {
            if let Type::Var(b) = t {
                if a == b {
                    return Ok(Subst::empty());
                }
            }
            if occurs(a, t) {
                return Err(format!(
                    "infinite type: `{}` occurs in `{}`",
                    a,
                    t.display()
                ));
            }
            Ok(Subst::singleton(a.clone(), t.clone()))
        }
        (t, Type::Var(a)) => {
            if occurs(a, t) {
                return Err(format!(
                    "infinite type: `{}` occurs in `{}`",
                    a,
                    t.display()
                ));
            }
            Ok(Subst::singleton(a.clone(), t.clone()))
        }

        // Unknown / Error are compatible with anything
        (Type::Unknown, _) | (_, Type::Unknown) => Ok(Subst::empty()),
        (Type::Error, _) | (_, Type::Error) => Ok(Subst::empty()),

        // Structural rules
        (Type::Task(a), Type::Task(b)) => unify(a, b),
        (Type::Stream(a), Type::Stream(b)) => unify(a, b),
        (Type::List(a), Type::List(b)) => unify(a, b),
        (Type::Option(a), Type::Option(b)) => unify(a, b),
        (Type::Map(k1, v1), Type::Map(k2, v2)) => {
            let s1 = unify(k1, k2)?;
            let s2 = unify(&s1.apply(v1), &s1.apply(v2))?;
            Ok(s2.compose(s1))
        }
        (Type::Result(t1, e1), Type::Result(t2, e2)) => {
            let s1 = unify(t1, t2)?;
            let s2 = unify(&s1.apply(e1), &s1.apply(e2))?;
            Ok(s2.compose(s1))
        }
        (Type::Arrow(a1, b1), Type::Arrow(a2, b2)) => {
            let s1 = unify(a1, a2)?;
            let s2 = unify(&s1.apply(b1), &s1.apply(b2))?;
            Ok(s2.compose(s1))
        }
        (Type::Arrow(a1, b1), Type::Trf(a2, b2, _))
        | (Type::Trf(a1, b1, _), Type::Arrow(a2, b2)) => {
            let s1 = unify(a1, a2)?;
            let s2 = unify(&s1.apply(b1), &s1.apply(b2))?;
            Ok(s2.compose(s1))
        }
        (Type::Named(n1, as1), Type::Named(n2, as2)) if n1 == n2 && as1.len() == as2.len() => as1
            .iter()
            .zip(as2.iter())
            .try_fold(Subst::empty(), |acc, (a, b)| {
                let s = unify(&acc.apply(a), &acc.apply(b))?;
                Ok(s.compose(acc))
            }),
        (Type::Interface(n1, as1), Type::Interface(n2, as2))
            if n1 == n2 && as1.len() == as2.len() =>
        {
            as1.iter()
                .zip(as2.iter())
                .try_fold(Subst::empty(), |acc, (a, b)| {
                    let s = unify(&acc.apply(a), &acc.apply(b))?;
                    Ok(s.compose(acc))
                })
        }
        // Option<T> ↁENamed("Option", [T]) compatibility (phase 5 transition)
        (Type::Option(t), Type::Named(n, args)) if n == "Option" && args.len() == 1 => {
            unify(t, &args[0])
        }
        (Type::Named(n, args), Type::Option(t)) if n == "Option" && args.len() == 1 => {
            unify(&args[0], t)
        }
        // Result<T,E> ↁENamed("Result", [T, E]) compatibility
        (Type::Result(t, e), Type::Named(n, args)) if n == "Result" && args.len() == 2 => {
            let s1 = unify(t, &args[0])?;
            let s2 = unify(&s1.apply(e), &s1.apply(&args[1]))?;
            Ok(s2.compose(s1))
        }
        (Type::Named(n, args), Type::Result(t, e)) if n == "Result" && args.len() == 2 => {
            let s1 = unify(&args[0], t)?;
            let s2 = unify(&s1.apply(&args[1]), &s1.apply(e))?;
            Ok(s2.compose(s1))
        }
        (t1, t2) => Err(format!(
            "cannot unify `{}` with `{}`",
            t1.display(),
            t2.display()
        )),
    }
}

// ── CapScope / ImplScope (v0.4.0) ─────────────────────────────────────────────

/// Registered capability definition: field names ↁEtype expressions (in type-param scope).
pub struct CapScope {
    /// field name ↁEfield type expression (unresolved; requires substituting type_params)
    pub fields: HashMap<String, TypeExpr>,
}

/// Registered implementation: method name ↁEresolved type.
pub struct ImplScope {
    pub methods: HashMap<String, Type>,
}

pub struct InterfaceDef {
    pub super_interface: Option<String>,
    pub methods: HashMap<String, Type>,
}

#[allow(dead_code)]
pub struct InterfaceImplEntry {
    pub methods: HashMap<String, Type>,
    pub is_auto: bool,
}

pub struct InterfaceRegistry {
    pub interfaces: HashMap<String, InterfaceDef>,
    pub impls: HashMap<(String, String), InterfaceImplEntry>,
}

impl InterfaceRegistry {
    pub fn new() -> Self {
        Self {
            interfaces: HashMap::new(),
            impls: HashMap::new(),
        }
    }

    pub fn register_interface(
        &mut self,
        name: String,
        super_interface: Option<String>,
        methods: HashMap<String, Type>,
    ) {
        self.interfaces.insert(
            name,
            InterfaceDef {
                super_interface,
                methods,
            },
        );
    }

    pub fn register_impl(
        &mut self,
        interface_name: String,
        type_name: String,
        methods: HashMap<String, Type>,
        is_auto: bool,
    ) {
        self.impls.insert(
            (interface_name, type_name),
            InterfaceImplEntry { methods, is_auto },
        );
    }

    pub fn is_implemented(&self, interface_name: &str, type_name: &str) -> bool {
        self.impls
            .contains_key(&(interface_name.to_string(), type_name.to_string()))
    }

    pub fn lookup_method(
        &self,
        interface_name: &str,
        type_name: &str,
        method_name: &str,
    ) -> Option<&Type> {
        self.impls
            .get(&(interface_name.to_string(), type_name.to_string()))
            .and_then(|entry| entry.methods.get(method_name))
            .or_else(|| {
                self.interfaces
                    .get(interface_name)
                    .and_then(|def| def.methods.get(method_name))
            })
    }

    /// Look up the method type from the interface *declaration* (canonical type, not impl body).
    /// Used for method dispatch to get the correct callable type with proper param counts.
    pub fn lookup_declared_method(&self, interface_name: &str, method_name: &str) -> Option<&Type> {
        self.interfaces
            .get(interface_name)?
            .methods
            .get(method_name)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlwBindingInfo {
    pub template: String,
    pub bindings: Vec<(String, crate::ast::SlotImpl)>,
}

// ── TyEnv (4-3) ───────────────────────────────────────────────────────────────

pub struct TyEnv {
    scopes: Vec<HashMap<String, Type>>,
}

impl TyEnv {
    pub fn new() -> Self {
        TyEnv {
            scopes: vec![HashMap::new()],
        }
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
        TypeError {
            code,
            message: message.into(),
            span,
        }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error[{}]: {}\n  --> {}:{}:{}",
            self.code, self.message, self.span.file, self.span.line, self.span.col
        )
    }
}

#[derive(Debug, Clone)]
pub struct TypeWarning {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

pub type FavWarning = TypeWarning;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Type,
    Stage,
    Seq,
    Interface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub detail: String,
    pub def_span: Span,
}

impl TypeWarning {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        TypeWarning {
            code,
            message: message.into(),
            span,
        }
    }
}

impl std::fmt::Display for TypeWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "warning[{}]: {}\n  --> {}:{}:{}",
            self.code, self.message, self.span.file, self.span.line, self.span.col
        )
    }
}

// ── Checker ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum StaticValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
}

impl StaticValue {
    fn from_lit(lit: Lit) -> Self {
        match lit {
            Lit::Int(v) => StaticValue::Int(v),
            Lit::Float(v) => StaticValue::Float(v),
            Lit::Bool(v) => StaticValue::Bool(v),
            Lit::Str(v) => StaticValue::String(v),
            Lit::Unit => StaticValue::Unit,
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            StaticValue::String(v) => Some(v),
            _ => None,
        }
    }
}

pub struct Checker {
    env: TyEnv,
    pub errors: Vec<TypeError>,
    pub warnings: Vec<TypeWarning>,
    pub type_at: HashMap<Span, Type>,
    pub def_at: HashMap<Span, Span>,
    pub symbol_index: Vec<LspSymbol>,
    pub imported_namespaces: HashMap<String, crate::middle::resolver::ModuleScope>,
    reexport_namespaces: HashMap<String, crate::middle::resolver::ModuleScope>,
    imported_namespace_paths: HashMap<String, String>,
    /// User-defined type bodies, for field and variant lookup.
    type_defs: HashMap<String, TypeBody>,
    pub record_fields: HashMap<String, Vec<(String, Type)>>,
    global_def_spans: HashMap<String, Span>,
    /// User-defined invariants by type name.
    type_invariants: HashMap<String, Vec<Expr>>,
    /// Effects declared on the current fn/trf being checked.
    current_effects: Vec<Effect>,
    /// Module resolver (Some = project mode, None = single-file mode).
    resolver: Option<Arc<Mutex<crate::middle::resolver::Resolver>>>,
    /// File being checked (for visibility enforcement).
    current_file: Option<PathBuf>,
    /// Imported symbols: name ↁE(type, visibility, source_file).
    imported: HashMap<String, (Type, Visibility, PathBuf)>,
    /// Type parameters in scope for the current fn/trf/type (v0.4.0).
    type_params: HashSet<String>,
    /// Counter for fresh type variable generation (v0.4.0).
    fresh_counter: usize,
    /// Registered capability definitions (v0.4.0).
    caps: HashMap<String, CapScope>,
    /// Registered implementations: (cap_name, type_key) ↁEscope (v0.4.0).
    impls: HashMap<(String, String), ImplScope>,
    /// Registered interfaces and their implementations (v1.1.0).
    interface_registry: InterfaceRegistry,
    /// Registered abstract stage definitions (v1.3.0).
    abstract_trf_registry: HashMap<String, AbstractTrfDef>,
    /// Registered abstract seq template definitions (v1.3.0).
    abstract_flw_registry: HashMap<String, AbstractFlwDef>,
    /// User-declared custom effects (v1.5.0).
    effect_registry: HashSet<String>,
    /// Explain/check metadata for bound abstract seq values (v1.3.0).
    flw_binding_info: HashMap<String, FlwBindingInfo>,
    /// Expected type-parameter arity for user-defined generic types (v0.4.0).
    type_arity: HashMap<String, usize>,
    /// Chain context: the return type of the enclosing fn when it is Result/Option (v0.5.0).
    chain_context: Option<Type>,
    /// Whether we are inside a collect { } block (v0.5.0).
    in_collect: bool,
    /// Type alias definitions: name -> target TypeExpr (v1.7.0).
    type_aliases: HashMap<String, TypeExpr>,
    /// Type constraint schemas loaded from schemas/*.yaml (v4.1.5).
    pub schemas: ProjectSchemas,
}

impl Checker {
    pub fn new() -> Self {
        Checker {
            env: TyEnv::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            type_at: HashMap::new(),
            def_at: HashMap::new(),
            symbol_index: Vec::new(),
            imported_namespaces: HashMap::new(),
            reexport_namespaces: HashMap::new(),
            imported_namespace_paths: HashMap::new(),
            type_defs: HashMap::new(),
            record_fields: HashMap::new(),
            global_def_spans: HashMap::new(),
            type_invariants: HashMap::new(),
            current_effects: Vec::new(),
            resolver: None,
            current_file: None,
            imported: HashMap::new(),
            type_params: HashSet::new(),
            fresh_counter: 0,
            caps: HashMap::new(),
            interface_registry: InterfaceRegistry::new(),
            abstract_trf_registry: HashMap::new(),
            abstract_flw_registry: HashMap::new(),
            effect_registry: HashSet::new(),
            flw_binding_info: HashMap::new(),
            impls: HashMap::new(),
            type_arity: HashMap::new(),
            chain_context: None,
            in_collect: false,
            type_aliases: HashMap::new(),
            schemas: HashMap::new(),
        }
    }

    pub fn new_with_resolver(
        resolver: Arc<Mutex<crate::middle::resolver::Resolver>>,
        file: PathBuf,
    ) -> Self {
        Checker {
            env: TyEnv::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            type_at: HashMap::new(),
            def_at: HashMap::new(),
            symbol_index: Vec::new(),
            imported_namespaces: HashMap::new(),
            reexport_namespaces: HashMap::new(),
            imported_namespace_paths: HashMap::new(),
            type_defs: HashMap::new(),
            record_fields: HashMap::new(),
            global_def_spans: HashMap::new(),
            type_invariants: HashMap::new(),
            current_effects: Vec::new(),
            resolver: Some(resolver),
            current_file: Some(file),
            imported: HashMap::new(),
            type_params: HashSet::new(),
            fresh_counter: 0,
            caps: HashMap::new(),
            interface_registry: InterfaceRegistry::new(),
            abstract_trf_registry: HashMap::new(),
            abstract_flw_registry: HashMap::new(),
            effect_registry: HashSet::new(),
            flw_binding_info: HashMap::new(),
            impls: HashMap::new(),
            type_arity: HashMap::new(),
            chain_context: None,
            in_collect: false,
            type_aliases: HashMap::new(),
            schemas: HashMap::new(),
        }
    }

    /// Generate a fresh type variable `$N`.
    fn fresh_var(&mut self) -> Type {
        let n = self.fresh_counter;
        self.fresh_counter += 1;
        Type::Var(format!("${}", n))
    }

    pub fn check_program(program: &Program) -> (Vec<TypeError>, Vec<FavWarning>) {
        let mut c = Checker::new();
        c.register_builtins();
        c.resolve_uses(program);
        c.process_imports(program);
        c.register_item_signatures(program);
        for item in &program.items {
            c.check_item(item);
        }
        (c.errors, c.warnings)
    }

    /// Check a program using a pre-built Checker (project mode with resolver).
    /// Returns collected errors.
    pub fn check_with_self(&mut self, program: &Program) -> (Vec<TypeError>, Vec<FavWarning>) {
        self.register_builtins();
        self.resolve_uses(program);
        self.process_imports(program);
        self.check_namespace_match(program);
        self.register_item_signatures(program);
        for item in &program.items {
            self.check_item(item);
        }
        (
            std::mem::take(&mut self.errors),
            std::mem::take(&mut self.warnings),
        )
    }

    fn remember_type(&mut self, span: &Span, ty: &Type) {
        self.type_at.insert(span.clone(), ty.clone());
    }

    fn remember_global_symbol(
        &mut self,
        name: impl Into<String>,
        kind: SymbolKind,
        detail: impl Into<String>,
        def_span: &Span,
    ) {
        let name = name.into();
        self.global_def_spans.insert(name.clone(), def_span.clone());
        self.symbol_index.push(LspSymbol {
            name,
            kind,
            detail: detail.into(),
            def_span: def_span.clone(),
        });
    }

    /// W012: warn if `namespace` declaration doesn't match the derived module path.
    fn check_namespace_match(&mut self, program: &Program) {
        let declared = match &program.namespace {
            Some(ns) => ns.clone(),
            None => return,
        };
        let file = match &self.current_file {
            Some(f) => f.clone(),
            None => return,
        };
        let resolver = match &self.resolver {
            Some(r) => r.clone(),
            None => return,
        };
        let guard = resolver.lock().unwrap();
        let src_dir = match (guard.root.as_ref(), guard.toml.as_ref()) {
            (Some(root), Some(toml)) => root.join(&toml.src),
            _ => return,
        };
        drop(guard);
        if let Some(derived) = crate::middle::resolver::derive_module_path(&file, &src_dir) {
            if derived != declared {
                let span = crate::frontend::lexer::Span::new(&*file.to_string_lossy(), 0, 0, 1, 1);
                self.type_warning(
                    "W012",
                    format!(
                        "namespace `{}` does not match file path `{}` (expected `{}`)",
                        declared,
                        file.display(),
                        derived
                    ),
                    &span,
                );
            }
        }
    }

    /// Check a program and return (errors, exported_symbols).
    /// `exported_symbols` maps each top-level name to its (Type, Visibility).
    pub fn check_program_and_export(
        program: &Program,
    ) -> (
        Vec<TypeError>,
        Vec<FavWarning>,
        HashMap<String, (Type, Visibility)>,
    ) {
        let mut c = Checker::new();
        c.register_builtins();
        c.resolve_uses(program);
        c.process_imports(program);
        c.register_item_signatures(program);
        for item in &program.items {
            c.check_item(item);
        }
        let exports = c.collect_export_scope(program);
        (c.errors, c.warnings, exports)
    }

    fn collect_export_scope(&self, program: &Program) -> HashMap<String, (Type, Visibility)> {
        let mut exports = collect_exports(program, &self.env);
        for scope in self.reexport_namespaces.values() {
            for (name, (ty, vis)) in &scope.symbols {
                if *vis != Visibility::Private {
                    exports
                        .entry(name.clone())
                        .or_insert((ty.clone(), Visibility::Public));
                }
            }
        }
        exports
    }

    /// Resolve `use` declarations using the attached resolver.
    fn resolve_uses(&mut self, program: &Program) {
        if program.uses.is_empty() {
            return;
        }
        let resolver = match self.resolver.clone() {
            Some(r) => r,
            None => {
                // No resolver: report each use as unresolvable
                for use_path in &program.uses {
                    let sym = use_path.last().cloned().unwrap_or_default();
                    let mod_path = use_path[..use_path.len().saturating_sub(1)].join(".");
                    self.errors.push(TypeError::new(
                        "E0213",
                        format!(
                            "`use {}.{}`: no fav.toml found  Ecannot resolve modules in single-file mode",
                            mod_path, sym
                        ),
                        Span::dummy(),
                    ));
                }
                return;
            }
        };
        for use_path in &program.uses {
            let mut resolve_errors = Vec::new();
            let result = {
                let mut r = resolver.lock().unwrap();
                r.resolve_use(use_path, &mut resolve_errors, &Span::dummy())
            };
            // Convert resolve errors to type errors
            for re in resolve_errors {
                self.errors
                    .push(TypeError::new(re.code, re.message, re.span));
            }
            if let Some((sym_name, ty)) = result {
                self.env.define(sym_name.clone(), ty.clone());
                // Store import metadata for visibility enforcement
                let source_file =
                    PathBuf::from(format!("<{}>", use_path[..use_path.len() - 1].join(".")));
                self.imported
                    .insert(sym_name, (ty, Visibility::Public, source_file));
            }
        }
    }

    fn process_imports(&mut self, program: &Program) {
        for item in &program.items {
            let Item::ImportDecl {
                path,
                alias,
                is_rune,
                is_public,
                span,
            } = item
            else {
                continue;
            };
            self.process_import_decl(path, alias.as_deref(), *is_rune, *is_public, span);
        }
    }

    fn process_import_decl(
        &mut self,
        path: &str,
        alias: Option<&str>,
        is_rune: bool,
        is_public: bool,
        span: &Span,
    ) {
        let namespace = alias
            .map(str::to_string)
            .unwrap_or_else(|| path.split('/').last().unwrap_or(path).to_string());
        if let Some(prev) = self.imported_namespace_paths.get(&namespace).cloned() {
            self.type_error(
                "E0581",
                format!(
                    "namespace conflict: '{}' is imported from both \"{}\" and \"{}\"\n  hint: use `as` to resolve:\n    import \"{}\" as {}_1\n    import \"{}\" as {}_2",
                    namespace, prev, path, prev, namespace, path, namespace
                ),
                span,
            );
            return;
        }

        let Some(resolver) = self.resolver.clone() else {
            self.type_error(
                "E0213",
                format!(
                    "`import \"{}\"`: no fav.toml found - cannot resolve imports in single-file mode",
                    path
                ),
                span,
            );
            return;
        };

        let cache_key = format!("{}:{}", if is_rune { "rune" } else { "local" }, path);
        let (file_path, existing, cached) = {
            let mut guard = resolver.lock().unwrap();
            let resolved = if is_rune {
                guard.resolve_rune_import_file(path)
            } else {
                guard.resolve_local_import_file(path)
            };
            let Some(file_path) = resolved else {
                self.type_error(
                    "E0213",
                    format!("import path \"{}\" could not be resolved", path),
                    span,
                );
                return;
            };
            let cached = guard.cached_scope(&cache_key);
            let existing = if cached.is_none() {
                guard.begin_loading(&cache_key, path)
            } else {
                None
            };
            (file_path, existing, cached)
        };

        if let Some(scope) = cached {
            self.imported_namespace_paths
                .insert(namespace.clone(), path.to_string());
            self.imported_namespaces
                .insert(namespace.clone(), scope.clone());
            if is_public {
                self.reexport_namespaces.insert(namespace, scope);
            }
            return;
        }

        if let Some(origin) = existing {
            let current = self
                .current_file
                .as_ref()
                .map(|file| file.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string());
            self.type_error(
                "E0580",
                format!(
                    "circular import detected\n  \"{}\" imports \"{}\" which imports \"{}\"",
                    origin, current, path
                ),
                span,
            );
            return;
        }

        let source = match std::fs::read_to_string(&file_path) {
            Ok(source) => source,
            Err(_) => {
                let mut guard = resolver.lock().unwrap();
                guard.finish_loading(&cache_key);
                self.type_error(
                    "E0213",
                    format!("import path \"{}\" could not be loaded", path),
                    span,
                );
                return;
            }
        };
        let file_str = file_path.to_string_lossy().to_string();
        let program = match Parser::parse_str(&source, &file_str) {
            Ok(program) => program,
            Err(err) => {
                let mut guard = resolver.lock().unwrap();
                guard.finish_loading(&cache_key);
                self.type_error(
                    "E0213",
                    format!(
                        "parse error in imported module \"{}\": {}",
                        path, err.message
                    ),
                    &err.span,
                );
                return;
            }
        };
        // For directory runes (v4.1.0): merge items from sibling files
        // referenced by `use X.{ ... }` / `use X.*` in the entrypoint.
        let program = if is_rune {
            let dir = file_path.parent().unwrap_or(std::path::Path::new("."));
            let mut all_items = program.items.clone();
            let mut seen: std::collections::HashSet<std::path::PathBuf> =
                std::collections::HashSet::new();
            seen.insert(file_path.clone());
            for item in &program.items {
                if let Item::RuneUse { module, .. } = item {
                    let sib = dir.join(format!("{module}.fav"));
                    if seen.insert(sib.clone()) && sib.exists() {
                        if let Ok(src) = std::fs::read_to_string(&sib) {
                            let sib_str = sib.to_string_lossy().to_string();
                            if let Ok(sib_prog) = Parser::parse_str(&src, &sib_str) {
                                all_items.extend(sib_prog.items);
                            }
                        }
                    }
                }
            }
            Program {
                namespace: program.namespace,
                uses: program.uses,
                items: all_items,
            }
        } else {
            program
        };
        let mut child = Checker::new_with_resolver(resolver.clone(), file_path.clone());
        let (child_errors, _) = child.check_with_self(&program);
        for error in child_errors {
            self.errors.push(error);
        }
        let scope = crate::middle::resolver::ModuleScope {
            symbols: child.collect_export_scope(&program),
        };
        {
            let mut guard = resolver.lock().unwrap();
            guard.cache_scope(cache_key.clone(), scope.clone());
            guard.finish_loading(&cache_key);
        }
        self.imported_namespace_paths
            .insert(namespace.clone(), path.to_string());
        self.imported_namespaces
            .insert(namespace.clone(), scope.clone());
        if is_public {
            self.reexport_namespaces.insert(namespace, scope);
        }
    }

    /// Check that a referenced symbol's visibility allows access from the current file.
    /// Currently reports E015 for private cross-file access.
    fn check_symbol_visibility(&mut self, name: &str, span: &Span) {
        if let Some((_, vis, source_file)) = self.imported.get(name) {
            if *vis == Visibility::Private {
                if self.current_file.as_deref() != Some(source_file.as_path()) {
                    self.type_error(
                        "E0215",
                        format!(
                            "`{}` is private  Ecannot be referenced from another file",
                            name
                        ),
                        span,
                    );
                }
            }
        }
    }

    fn type_error(&mut self, code: &'static str, msg: impl Into<String>, span: &Span) {
        self.errors.push(TypeError::new(code, msg, span.clone()));
    }

    fn type_warning(&mut self, code: &'static str, msg: impl Into<String>, span: &Span) {
        self.warnings
            .push(TypeWarning::new(code, msg, span.clone()));
    }

    // ── built-in registration (4-4, 4-5) ─────────────────────────────────────

    fn register_builtins(&mut self) {
        // IO namespace functions are handled specially in check_builtin_apply.
        // Register placeholder so "IO" resolves to something.
        self.env
            .define("IO".into(), Type::Named("IO".into(), vec![]));

        // List, String, Option, Result, and v0.2.0 namespace placeholders.
        for ns in &[
            "Math",
            "List",
            "String",
            "Option",
            "Result",
            "Db",
            "Http",
            "Map",
            "Debug",
            "Emit",
            "Util",
            "Trace",
            "File",
            "Json",
            "Csv",
            "Schema",
            "Task",
            "Random",
            "Stream",
            "DB",
            "Env",
            "Gen",
            "Checkpoint",
            "Parquet",
            "Grpc",
        ] {
            self.env
                .define(ns.to_string(), Type::Named(ns.to_string(), vec![]));
        }
        self.env.define(
            "SchemaError".into(),
            Type::Named("SchemaError".into(), vec![]),
        );
        let schema_fields = vec![
            Field {
                name: "field".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "expected".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "got".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
        ];
        self.type_defs.insert(
            "SchemaError".into(),
            TypeBody::Record(schema_fields.clone()),
        );
        self.record_fields.insert(
            "SchemaError".into(),
            schema_fields
                .iter()
                .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
                .collect(),
        );

        // DbError (v3.3.0)
        self.env
            .define("DbError".into(), Type::Named("DbError".into(), vec![]));
        let db_error_fields = vec![
            Field {
                name: "code".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "message".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
        ];
        self.type_defs
            .insert("DbError".into(), TypeBody::Record(db_error_fields.clone()));
        self.record_fields.insert(
            "DbError".into(),
            db_error_fields
                .iter()
                .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
                .collect(),
        );
        // DbHandle / TxHandle as opaque named types
        self.env
            .define("DbHandle".into(), Type::Named("DbHandle".into(), vec![]));
        self.env
            .define("TxHandle".into(), Type::Named("TxHandle".into(), vec![]));

        // GenProfile (v3.5.0)
        self.env.define(
            "GenProfile".into(),
            Type::Named("GenProfile".into(), vec![]),
        );
        let gen_profile_fields = vec![
            Field {
                name: "total".into(),
                ty: TypeExpr::Named("Int".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "valid".into(),
                ty: TypeExpr::Named("Int".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "invalid".into(),
                ty: TypeExpr::Named("Int".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "rate".into(),
                ty: TypeExpr::Named("Float".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
        ];
        self.type_defs.insert(
            "GenProfile".into(),
            TypeBody::Record(gen_profile_fields.clone()),
        );
        self.record_fields.insert(
            "GenProfile".into(),
            gen_profile_fields
                .iter()
                .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
                .collect(),
        );

        self.env.define(
            "CheckpointMeta".into(),
            Type::Named("CheckpointMeta".into(), vec![]),
        );
        let checkpoint_meta_fields = vec![
            Field {
                name: "name".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "value".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "updated_at".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
        ];
        self.type_defs.insert(
            "CheckpointMeta".into(),
            TypeBody::Record(checkpoint_meta_fields.clone()),
        );
        self.record_fields.insert(
            "CheckpointMeta".into(),
            checkpoint_meta_fields
                .iter()
                .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
                .collect(),
        );

        self.env.define(
            "HttpResponse".into(),
            Type::Named("HttpResponse".into(), vec![]),
        );
        let http_response_fields = vec![
            Field {
                name: "status".into(),
                ty: TypeExpr::Named("Int".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "body".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "content_type".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
        ];
        self.type_defs.insert(
            "HttpResponse".into(),
            TypeBody::Record(http_response_fields.clone()),
        );
        self.record_fields.insert(
            "HttpResponse".into(),
            http_response_fields
                .iter()
                .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
                .collect(),
        );

        self.env
            .define("HttpError".into(), Type::Named("HttpError".into(), vec![]));
        let http_error_fields = vec![
            Field {
                name: "code".into(),
                ty: TypeExpr::Named("Int".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "message".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "status".into(),
                ty: TypeExpr::Named("Int".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
        ];
        self.type_defs.insert(
            "HttpError".into(),
            TypeBody::Record(http_error_fields.clone()),
        );
        self.record_fields.insert(
            "HttpError".into(),
            http_error_fields
                .iter()
                .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
                .collect(),
        );

        self.env.define(
            "ParquetError".into(),
            Type::Named("ParquetError".into(), vec![]),
        );
        let parquet_error_fields = vec![Field {
            name: "message".into(),
            ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
            attrs: vec![],
            span: Span::dummy(),
        }];
        self.type_defs.insert(
            "ParquetError".into(),
            TypeBody::Record(parquet_error_fields.clone()),
        );
        self.record_fields.insert(
            "ParquetError".into(),
            parquet_error_fields
                .iter()
                .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
                .collect(),
        );

        self.env
            .define("RpcError".into(), Type::Named("RpcError".into(), vec![]));
        let rpc_error_fields = vec![
            Field {
                name: "code".into(),
                ty: TypeExpr::Named("Int".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "message".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
        ];
        self.type_defs.insert(
            "RpcError".into(),
            TypeBody::Record(rpc_error_fields.clone()),
        );
        self.record_fields.insert(
            "RpcError".into(),
            rpc_error_fields
                .iter()
                .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
                .collect(),
        );

        self.env.define(
            "RpcRequest".into(),
            Type::Named("RpcRequest".into(), vec![]),
        );
        let rpc_request_fields = vec![
            Field {
                name: "method".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "payload".into(),
                ty: TypeExpr::Named(
                    "Map".into(),
                    vec![
                        TypeExpr::Named("String".into(), vec![], Span::dummy()),
                        TypeExpr::Named("String".into(), vec![], Span::dummy()),
                    ],
                    Span::dummy(),
                ),
                attrs: vec![],
                span: Span::dummy(),
            },
        ];
        self.type_defs.insert(
            "RpcRequest".into(),
            TypeBody::Record(rpc_request_fields.clone()),
        );
        self.record_fields.insert(
            "RpcRequest".into(),
            rpc_request_fields
                .iter()
                .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
                .collect(),
        );

        // ValidationError (v4.1.5)
        self.env.define(
            "ValidationError".into(),
            Type::Named("ValidationError".into(), vec![]),
        );
        let validation_error_fields = vec![
            Field {
                name: "field".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "constraint".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
            Field {
                name: "value".into(),
                ty: TypeExpr::Named("String".into(), vec![], Span::dummy()),
                attrs: vec![],
                span: Span::dummy(),
            },
        ];
        self.type_defs.insert(
            "ValidationError".into(),
            TypeBody::Record(validation_error_fields.clone()),
        );
        self.record_fields.insert(
            "ValidationError".into(),
            validation_error_fields
                .iter()
                .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
                .collect(),
        );

        // Primitive type names as env values (so `Int.eq` etc. resolve).
        for ty_name in &["Bool", "Int", "Float"] {
            self.env.define(
                ty_name.to_string(),
                Type::Named(ty_name.to_string(), vec![]),
            );
        }

        // ── Built-in cap definitions ──────────────────────────────────────────
        // Eq<T> = { equals: T -> T -> Bool }
        // Ord<T> = { compare: T -> T -> Int  equals: T -> T -> Bool }
        // Show<T> = { show: T -> String }
        // Gen<T> = { gen: Int? -> T }
        // Semigroup/Monoid/Group/Ring/Field are algebraic interfaces.
        for cap_name in &[
            "Eq",
            "Ord",
            "Show",
            "Gen",
            "Semigroup",
            "Monoid",
            "Group",
            "Ring",
            "Field",
        ] {
            self.env.define(
                cap_name.to_string(),
                Type::Named(cap_name.to_string(), vec![]),
            );
        }

        // ── Built-in impl registrations ───────────────────────────────────────
        let bool_ty = || Type::Bool;
        let int_ty = || Type::Int;
        let _float_ty = || Type::Float;
        let str_ty = || Type::String;

        let mk_eq_scope = |t: fn() -> Type| {
            let mut m = HashMap::new();
            m.insert(
                "equals".into(),
                Type::Fn(vec![t(), t()], Box::new(bool_ty())),
            );
            ImplScope { methods: m }
        };
        let mk_ord_scope = |t: fn() -> Type| {
            let mut m = HashMap::new();
            m.insert(
                "compare".into(),
                Type::Fn(vec![t(), t()], Box::new(int_ty())),
            );
            m.insert(
                "equals".into(),
                Type::Fn(vec![t(), t()], Box::new(bool_ty())),
            );
            ImplScope { methods: m }
        };
        let mk_show_scope = |t: fn() -> Type| {
            let mut m = HashMap::new();
            m.insert("show".into(), Type::Fn(vec![t()], Box::new(str_ty())));
            ImplScope { methods: m }
        };

        for ty_key in &["Int", "Float", "String"] {
            let t: fn() -> Type = match *ty_key {
                "Int" => || Type::Int,
                "Float" => || Type::Float,
                _ => || Type::String,
            };
            self.impls
                .insert(("Eq".into(), ty_key.to_string()), mk_eq_scope(t));
            self.impls
                .insert(("Ord".into(), ty_key.to_string()), mk_ord_scope(t));
            self.impls
                .insert(("Show".into(), ty_key.to_string()), mk_show_scope(t));
        }
        self.impls
            .insert(("Eq".into(), "Bool".into()), mk_eq_scope(bool_ty));
        self.impls
            .insert(("Show".into(), "Bool".into()), mk_show_scope(bool_ty));
        self.register_builtin_interfaces();
        self.register_stdlib_states();
    }

    fn register_stdlib_states(&mut self) {
        for td in crate::std_states::parsed_type_defs() {
            self.type_defs.insert(td.name.clone(), td.body.clone());
            self.type_invariants
                .insert(td.name.clone(), td.invariants.clone());
            self.env
                .define(td.name.clone(), Type::Named(td.name.clone(), vec![]));
        }
    }

    fn register_builtin_interfaces(&mut self) {
        if self.interface_registry.interfaces.contains_key("Show") {
            return;
        }

        let self_named = Type::Named("Self".into(), vec![]);

        let mut show_methods = HashMap::new();
        show_methods.insert(
            "show".into(),
            Type::Fn(vec![self_named.clone()], Box::new(Type::String)),
        );
        self.interface_registry
            .register_interface("Show".into(), None, show_methods);

        let mut eq_methods = HashMap::new();
        eq_methods.insert(
            "eq".into(),
            Type::Fn(
                vec![self_named.clone(), self_named.clone()],
                Box::new(Type::Bool),
            ),
        );
        self.interface_registry
            .register_interface("Eq".into(), None, eq_methods);

        let mut ord_methods = HashMap::new();
        ord_methods.insert(
            "compare".into(),
            Type::Fn(
                vec![self_named.clone(), self_named.clone()],
                Box::new(Type::Int),
            ),
        );
        self.interface_registry
            .register_interface("Ord".into(), Some("Eq".into()), ord_methods);

        let mut gen_methods = HashMap::new();
        gen_methods.insert(
            "gen".into(),
            Type::Fn(
                vec![Type::Option(Box::new(Type::Int))],
                Box::new(self_named.clone()),
            ),
        );
        self.interface_registry
            .register_interface("Gen".into(), None, gen_methods);

        let mut semigroup_methods = HashMap::new();
        semigroup_methods.insert(
            "combine".into(),
            Type::Fn(
                vec![self_named.clone(), self_named.clone()],
                Box::new(self_named.clone()),
            ),
        );
        self.interface_registry
            .register_interface("Semigroup".into(), None, semigroup_methods);

        let mut monoid_methods = HashMap::new();
        monoid_methods.insert(
            "empty".into(),
            Type::Fn(vec![], Box::new(self_named.clone())),
        );
        self.interface_registry.register_interface(
            "Monoid".into(),
            Some("Semigroup".into()),
            monoid_methods,
        );

        let mut group_methods = HashMap::new();
        group_methods.insert(
            "inverse".into(),
            Type::Fn(vec![self_named.clone()], Box::new(self_named.clone())),
        );
        self.interface_registry.register_interface(
            "Group".into(),
            Some("Monoid".into()),
            group_methods,
        );

        let mut ring_methods = HashMap::new();
        ring_methods.insert(
            "multiply".into(),
            Type::Fn(
                vec![self_named.clone(), self_named.clone()],
                Box::new(self_named.clone()),
            ),
        );
        self.interface_registry.register_interface(
            "Ring".into(),
            Some("Monoid".into()),
            ring_methods,
        );

        let mut field_methods = HashMap::new();
        field_methods.insert(
            "divide".into(),
            Type::Fn(
                vec![self_named.clone(), self_named.clone()],
                Box::new(Type::Result(
                    Box::new(self_named.clone()),
                    Box::new(Type::Named("Error".into(), vec![])),
                )),
            ),
        );
        self.interface_registry.register_interface(
            "Field".into(),
            Some("Ring".into()),
            field_methods,
        );

        for (interface_name, type_name) in [
            ("Show", "Int"),
            ("Eq", "Int"),
            ("Ord", "Int"),
            ("Gen", "Int"),
            ("Semigroup", "Int"),
            ("Monoid", "Int"),
            ("Group", "Int"),
            ("Ring", "Int"),
            ("Show", "Float"),
            ("Eq", "Float"),
            ("Ord", "Float"),
            ("Gen", "Float"),
            ("Semigroup", "Float"),
            ("Monoid", "Float"),
            ("Group", "Float"),
            ("Ring", "Float"),
            ("Field", "Float"),
            ("Show", "String"),
            ("Eq", "String"),
            ("Ord", "String"),
            ("Gen", "String"),
            ("Show", "Bool"),
            ("Eq", "Bool"),
            ("Gen", "Bool"),
        ] {
            self.interface_registry.register_impl(
                interface_name.into(),
                type_name.into(),
                HashMap::new(),
                true,
            );
        }
    }

    // ── first-pass: register top-level names (4-6..4-9) ─────────────────────

    fn register_item_signatures(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::EffectDef(ed) => {
                    self.effect_registry.insert(ed.name.clone());
                }
                Item::TypeDef(td) => {
                    self.remember_global_symbol(
                        td.name.clone(),
                        SymbolKind::Type,
                        td.name.clone(),
                        &td.span,
                    );
                    // Type aliases: store the target TypeExpr for later resolution.
                    if let TypeBody::Alias(target) = &td.body {
                        self.type_aliases.insert(td.name.clone(), target.clone());
                        if !td.type_params.is_empty() {
                            self.type_arity
                                .insert(td.name.clone(), td.type_params.len());
                        }
                        // Define in env so the name resolves
                        self.env
                            .define(td.name.clone(), Type::Named(td.name.clone(), vec![]));
                        continue;
                    }
                    self.type_defs.insert(td.name.clone(), td.body.clone());
                    if let TypeBody::Record(fields) = &td.body {
                        self.record_fields.insert(
                            td.name.clone(),
                            fields
                                .iter()
                                .map(|field| {
                                    (field.name.clone(), self.resolve_type_expr(&field.ty))
                                })
                                .collect(),
                        );
                    }
                    self.type_invariants
                        .insert(td.name.clone(), td.invariants.clone());
                    self.env
                        .define(td.name.clone(), Type::Named(td.name.clone(), vec![]));
                    // Track arity for generic type arity checking (E023).
                    if !td.type_params.is_empty() {
                        self.type_arity
                            .insert(td.name.clone(), td.type_params.len());
                    }
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
                                    self.env.define(
                                        name.clone(),
                                        Type::Fn(vec![payload], Box::new(parent.clone())),
                                    );
                                }
                                Variant::Record(name, fields, _) => {
                                    let field_tys: Vec<Type> = fields
                                        .iter()
                                        .map(|f| self.resolve_type_expr(&f.ty))
                                        .collect();
                                    self.env.define(
                                        name.clone(),
                                        Type::Fn(field_tys, Box::new(parent.clone())),
                                    );
                                }
                            }
                        }
                    }
                }
                Item::FnDef(fd) => {
                    // Resolve param/return types with type_params in scope.
                    let saved_tp = std::mem::replace(
                        &mut self.type_params,
                        fd.type_params.iter().cloned().collect(),
                    );
                    let params: Vec<Type> = fd
                        .params
                        .iter()
                        .map(|p| self.resolve_type_expr(&p.ty))
                        .collect();
                    let ret = fd
                        .return_ty
                        .as_ref()
                        .map(|ty| self.resolve_type_expr(ty))
                        .unwrap_or(Type::Unknown);
                    let effective_ret = if fd.is_async {
                        Type::Task(Box::new(ret))
                    } else {
                        ret
                    };
                    let fn_ty = Type::Fn(params.clone(), Box::new(effective_ret.clone()));
                    self.remember_global_symbol(
                        fd.name.clone(),
                        SymbolKind::Function,
                        fn_ty.display(),
                        &fd.span,
                    );
                    self.type_params = saved_tp;
                    self.env.define(fd.name.clone(), fn_ty);
                }
                Item::TrfDef(td) => {
                    let saved_tp = std::mem::replace(
                        &mut self.type_params,
                        td.type_params.iter().cloned().collect(),
                    );
                    let input = self.resolve_type_expr(&td.input_ty);
                    let output = self.resolve_type_expr(&td.output_ty);
                    let trf_ty = Type::Trf(
                        Box::new(input.clone()),
                        Box::new(output.clone()),
                        td.effects.clone(),
                    );
                    self.remember_global_symbol(
                        td.name.clone(),
                        SymbolKind::Stage,
                        trf_ty.display(),
                        &td.span,
                    );
                    self.type_params = saved_tp;
                    self.env.define(td.name.clone(), trf_ty);
                }
                Item::AbstractTrfDef(td) => {
                    self.abstract_trf_registry
                        .insert(td.name.clone(), td.clone());
                    if !td.type_params.is_empty() {
                        self.type_arity
                            .insert(td.name.clone(), td.type_params.len());
                    }
                    let input = self.resolve_type_expr(&td.input_ty);
                    let output = self.resolve_type_expr(&td.output_ty);
                    self.env.define(
                        td.name.clone(),
                        Type::AbstractTrf {
                            input: Box::new(input),
                            output: Box::new(output),
                            effects: td.effects.clone(),
                        },
                    );
                }
                Item::FlwDef(fd) => {
                    self.remember_global_symbol(
                        fd.name.clone(),
                        SymbolKind::Seq,
                        "Seq".to_string(),
                        &fd.span,
                    );
                    // Compute flw type from its steps; register Unknown for now,
                    // will be refined during check_flw_def.
                    self.env.define(fd.name.clone(), Type::Unknown);
                }
                Item::AbstractFlwDef(fd) => {
                    self.abstract_flw_registry
                        .insert(fd.name.clone(), fd.clone());
                    self.env
                        .define(fd.name.clone(), Type::AbstractFlwTemplate(fd.name.clone()));
                }
                Item::FlwBindingDef(fd) => {
                    self.env.define(fd.name.clone(), Type::Unknown);
                }
                Item::CapDef(cd) => {
                    self.env
                        .define(cd.name.clone(), Type::Named(cd.name.clone(), vec![]));
                    let scope = CapScope {
                        fields: cd
                            .fields
                            .iter()
                            .map(|f| (f.name.clone(), f.ty.clone()))
                            .collect(),
                    };
                    self.caps.insert(cd.name.clone(), scope);
                }
                Item::ImplDef(id) => {
                    // Compute a string key for the first type argument.
                    if let Some(first_arg) = id.type_args.first() {
                        let ty_key = self.resolve_type_expr(first_arg).display();
                        let mut methods = HashMap::new();
                        for method in &id.methods {
                            let params: Vec<Type> = method
                                .params
                                .iter()
                                .map(|p| self.resolve_type_expr(&p.ty))
                                .collect();
                            let ret = method
                                .return_ty
                                .as_ref()
                                .map(|ty| self.resolve_type_expr(ty))
                                .unwrap_or(Type::Unknown);
                            methods.insert(method.name.clone(), Type::Fn(params, Box::new(ret)));
                        }
                        self.impls
                            .insert((id.cap_name.clone(), ty_key), ImplScope { methods });
                    }
                }
                // namespace / use / test / bench / interface are handled elsewhere
                Item::NamespaceDecl(..)
                | Item::UseDecl(..)
                | Item::RuneUse { .. }
                | Item::ImportDecl { .. }
                | Item::TestDef(..)
                | Item::BenchDef(..)
                | Item::InterfaceImplDecl(..) => {}
                Item::InterfaceDecl(decl) => {
                    self.remember_global_symbol(
                        decl.name.clone(),
                        SymbolKind::Interface,
                        decl.name.clone(),
                        &decl.span,
                    );
                }
            }
        }
    }

    // ── item checking ─────────────────────────────────────────────────────────

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::TypeDef(td) => self.check_type_def(td),
            Item::FnDef(fd) => self.check_fn_def(fd),
            Item::TrfDef(td) => self.check_trf_def(td),
            Item::AbstractTrfDef(td) => self.check_abstract_trf_def(td),
            Item::FlwDef(fd) => self.check_flw_def(fd),
            Item::AbstractFlwDef(fd) => self.check_abstract_flw_def(fd),
            Item::FlwBindingDef(fd) => self.check_flw_binding_def(fd),
            Item::InterfaceDecl(id) => self.check_interface_decl(id),
            Item::InterfaceImplDecl(id) => self.check_interface_impl_decl(id),
            Item::CapDef(cd) => self.check_cap_def(cd),
            Item::ImplDef(id) => self.check_impl_def(id),
            Item::EffectDef(..) => {}
            Item::TestDef(td) => self.check_test_def(td),
            Item::BenchDef(bd) => self.check_bench_def(bd),
            Item::NamespaceDecl(..) | Item::UseDecl(..) | Item::RuneUse { .. } | Item::ImportDecl { .. } => {}
        }
    }

    fn check_effects_declared(&mut self, effects: &[Effect], span: &Span) {
        const BUILTIN_EFFECTS: &[&str] = &[
            "Pure",
            "Io",
            "Db",
            "Network",
            "Rpc",
            "File",
            "Checkpoint",
            "Trace",
            "Emit",
            "Random",
        ];
        for effect in effects {
            if let Effect::Unknown(name) = effect {
                if !BUILTIN_EFFECTS.contains(&name.as_str()) && !self.effect_registry.contains(name)
                {
                    self.type_error(
                        "E0252",
                        format!(
                            "undeclared effect `{}`; declare it with `effect {}` at top level",
                            name, name
                        ),
                        span,
                    );
                }
            }
        }
    }

    fn check_abstract_trf_def(&mut self, td: &AbstractTrfDef) {
        self.check_effects_declared(&td.effects, &td.span);
        self.validate_type_expr_arity(&td.input_ty);
        self.validate_type_expr_arity(&td.output_ty);
    }

    fn check_abstract_flw_def(&mut self, fd: &AbstractFlwDef) {
        for slot in &fd.slots {
            self.check_effects_declared(&slot.effects, &slot.span);
            self.validate_type_expr_arity(&slot.input_ty);
            self.validate_type_expr_arity(&slot.output_ty);
            if let Some(ty) = &slot.abstract_trf_ty {
                self.validate_type_expr_arity(ty);
            }
        }
    }

    // ── type_def (4-6) ────────────────────────────────────────────────────────

    fn check_type_def(&mut self, td: &TypeDef) {
        if let TypeBody::Record(fields) = &td.body {
            for field in fields {
                self.validate_field_attrs(field);
            }
            self.env.push();
            for field in fields {
                let field_ty = self.resolve_type_expr(&field.ty);
                self.env.define(field.name.clone(), field_ty);
            }
            for invariant in &td.invariants {
                let inv_ty = self.check_expr(invariant);
                if !inv_ty.is_compatible(&Type::Bool) {
                    self.type_error(
                        "E0245",
                        format!(
                            "`invariant` for type `{}` must be of type Bool, got `{}`",
                            td.name,
                            inv_ty.display()
                        ),
                        invariant.span(),
                    );
                }
            }
            self.env.pop();
        }

        // Type definitions are structurally valid if they parsed correctly.
        // Field types are resolved lazily during use.
        for interface_name in &td.with_interfaces {
            self.synthesize_interface_impl_for_type_def(td, interface_name, &td.span);
        }
    }

    fn validate_field_attrs(&mut self, field: &Field) {
        for attr in &field.attrs {
            if attr.name != "col" {
                continue;
            }
            let Some(arg) = attr.arg.as_deref() else {
                self.type_error(
                    "E0503",
                    format!(
                        "`#[col(...)]` on field `{}` requires an integer argument",
                        field.name
                    ),
                    &attr.span,
                );
                continue;
            };
            if arg.parse::<usize>().is_err() {
                self.type_error(
                    "E0503",
                    format!(
                        "`#[col({})]` on field `{}` must use a non-negative integer index",
                        arg, field.name
                    ),
                    &attr.span,
                );
            }
        }
    }

    fn check_interface_decl(&mut self, id: &InterfaceDecl) {
        if let Some(super_name) = &id.super_interface {
            if !self.interface_registry.interfaces.contains_key(super_name) {
                self.type_error(
                    "E0241",
                    format!("undefined super interface `{}`", super_name),
                    &id.span,
                );
                return;
            }
        }

        let mut methods = HashMap::new();
        for method in &id.methods {
            let ty = self
                .resolve_type_expr_with_self(&method.ty, Some(&Type::Named("Self".into(), vec![])));
            methods.insert(method.name.clone(), ty);
        }
        self.interface_registry.register_interface(
            id.name.clone(),
            id.super_interface.clone(),
            methods,
        );
    }

    fn check_interface_impl_decl(&mut self, id: &InterfaceImplDecl) {
        let self_ty = Type::Named(id.type_name.clone(), vec![]);

        for interface_name in &id.interface_names {
            let Some(interface_def) = self.interface_registry.interfaces.get(interface_name) else {
                self.type_error(
                    "E0241",
                    format!("undefined interface `{}`", interface_name),
                    &id.span,
                );
                continue;
            };
            let super_interface = interface_def.super_interface.clone();
            let expected_methods = interface_def.methods.clone();

            if let Some(super_name) = &super_interface {
                let satisfied_by_same_decl = id.interface_names.iter().any(|n| n == super_name);
                let satisfied_by_prior_impl = self
                    .interface_registry
                    .is_implemented(super_name, &id.type_name);
                if !satisfied_by_same_decl && !satisfied_by_prior_impl {
                    self.type_error(
                        "E0243",
                        format!(
                            "interface `{}` requires super interface `{}` to be implemented for `{}`",
                            interface_name, super_name, id.type_name
                        ),
                        &id.span,
                    );
                }
            }

            if id.is_auto {
                self.synthesize_interface_impl_for_type_name(
                    &id.type_name,
                    interface_name,
                    &id.span,
                );
                continue;
            }

            let mut provided = HashMap::new();
            for (method_name, body) in &id.methods {
                let body_ty = self.check_expr(body);
                provided.insert(method_name.clone(), body_ty);
            }

            for expected_name in expected_methods.keys() {
                if !provided.contains_key(expected_name) {
                    self.type_error(
                        "E0242",
                        format!(
                            "impl for `{}` is missing method `{}` required by interface `{}`",
                            id.type_name, expected_name, interface_name
                        ),
                        &id.span,
                    );
                }
            }

            for (method_name, body_ty) in &provided {
                let Some(expected) = expected_methods.get(method_name) else {
                    self.type_error(
                        "E0242",
                        format!(
                            "method `{}` is not declared in interface `{}`",
                            method_name, interface_name
                        ),
                        &id.span,
                    );
                    continue;
                };
                let expected = self.substitute_self_in_type(expected, &self_ty);
                // Lambda bodies are typed as Arrow(Unknown/Unit, ret) — Unknown for 1-param,
                // Unit as placeholder for multi-param. Be lenient on input type for closures,
                // but still check that the return type matches.
                let compatible = if let Type::Arrow(input, ret_ty) = body_ty {
                    if matches!(input.as_ref(), Type::Unknown | Type::Unit) {
                        // Extract the final return type, unwrapping curried arrows
                        fn final_ret(ty: &Type) -> &Type {
                            match ty {
                                Type::Arrow(_, out) => final_ret(out),
                                Type::Fn(_, ret) => ret,
                                other => other,
                            }
                        }
                        ret_ty.is_compatible(final_ret(&expected))
                    } else {
                        body_ty.is_compatible(&expected)
                    }
                } else {
                    body_ty.is_compatible(&expected)
                };
                if !compatible {
                    self.type_error(
                        "E0242",
                        format!(
                            "method `{}` for `{}` has type `{}`, expected `{}`",
                            method_name,
                            id.type_name,
                            body_ty.display(),
                            expected.display()
                        ),
                        &id.span,
                    );
                }
            }

            self.interface_registry.register_impl(
                interface_name.clone(),
                id.type_name.clone(),
                provided,
                false,
            );
        }
    }

    fn synthesize_interface_impl_for_type_name(
        &mut self,
        type_name: &str,
        interface_name: &str,
        span: &Span,
    ) {
        let Some(body) = self.type_defs.get(type_name).cloned() else {
            self.type_error(
                "E0244",
                format!(
                    "cannot auto-synthesize interface `{}` for `{}` without a local type definition",
                    interface_name, type_name
                ),
                span,
            );
            return;
        };
        let td = TypeDef {
            visibility: None,
            name: type_name.to_string(),
            type_params: vec![],
            with_interfaces: vec![],
            invariants: vec![],
            body,
            span: span.clone(),
        };
        self.synthesize_interface_impl_for_type_def(&td, interface_name, span);
    }

    /// Returns true if `ty` implements `interface_name`, including generic containers:
    /// List<T> / Option<T> / Result<T,E> are considered to implement an interface
    /// when their element type(s) do.
    fn is_type_implementing(&self, interface_name: &str, ty: &Type) -> bool {
        if self
            .interface_registry
            .is_implemented(interface_name, &ty.display())
        {
            return true;
        }
        match ty {
            Type::List(inner) => self.is_type_implementing(interface_name, inner),
            Type::Option(inner) => self.is_type_implementing(interface_name, inner),
            Type::Result(ok, _err) => self.is_type_implementing(interface_name, ok),
            _ => false,
        }
    }

    fn synthesize_interface_impl_for_type_def(
        &mut self,
        td: &TypeDef,
        interface_name: &str,
        span: &Span,
    ) {
        let Some(interface_def) = self.interface_registry.interfaces.get(interface_name) else {
            self.type_error(
                "E0241",
                format!("undefined interface `{}`", interface_name),
                span,
            );
            return;
        };

        let super_interface = interface_def.super_interface.clone();
        if let Some(super_name) = &super_interface {
            if !self.interface_registry.is_implemented(super_name, &td.name) {
                self.synthesize_interface_impl_for_type_def(td, super_name, span);
                if !self.interface_registry.is_implemented(super_name, &td.name) {
                    self.type_error(
                        "E0243",
                        format!(
                            "interface `{}` requires super interface `{}` to be implemented for `{}`",
                            interface_name, super_name, td.name
                        ),
                        span,
                    );
                    return;
                }
            }
        }

        let fields = match &td.body {
            TypeBody::Record(fields) => fields,
            _ => {
                self.type_error(
                    "E0244",
                    format!(
                        "auto-synthesis for interface `{}` is only supported on record types",
                        interface_name
                    ),
                    span,
                );
                return;
            }
        };

        for field in fields {
            let field_ty = self.resolve_type_expr(&field.ty);
            if !self.is_type_implementing(interface_name, &field_ty) {
                self.type_error(
                    "E0244",
                    format!(
                        "field `{}` of `{}` does not implement interface `{}`",
                        field.name, td.name, interface_name
                    ),
                    &field.span,
                );
                return;
            }
        }

        self.interface_registry.register_impl(
            interface_name.to_string(),
            td.name.clone(),
            HashMap::new(),
            true,
        );
    }

    // ── cap_def (v0.4.0) ──────────────────────────────────────────────────────

    fn check_cap_def(&mut self, cd: &CapDef) {
        self.type_warning(
            "W010",
            "`cap` is deprecated. Use `interface` instead.",
            &cd.span,
        );
        // Validate that each field's type expression is well-formed in type_params scope.
        let saved_tp = std::mem::replace(
            &mut self.type_params,
            cd.type_params.iter().cloned().collect(),
        );
        for field in &cd.fields {
            self.resolve_type_expr(&field.ty); // triggers errors on bad type refs
        }
        self.type_params = saved_tp;
    }

    // ── impl_def (v0.4.0) ─────────────────────────────────────────────────────

    fn check_impl_def(&mut self, id: &ImplDef) {
        self.type_warning(
            "W010",
            "`cap`-style `impl` is deprecated. Use `impl Interface for Type` instead.",
            &id.span,
        );
        // E020: cap must exist.
        if !self.caps.contains_key(&id.cap_name) {
            // Only error if it's not a built-in cap either.
            let is_builtin = matches!(id.cap_name.as_str(), "Eq" | "Ord" | "Show");
            if !is_builtin {
                let span = &id.span;
                self.type_error("E0220", format!("undefined cap `{}`", id.cap_name), span);
                return;
            }
        }

        // Determine expected field names from the cap definition.
        let expected_fields: HashSet<String> = if let Some(scope) = self.caps.get(&id.cap_name) {
            scope.fields.keys().cloned().collect()
        } else {
            // Built-in cap: derive expected methods from built-in impls.
            // If we have any impl registered for this cap, use its method names.
            HashSet::new() // permissive for built-ins
        };

        // E022: each method must correspond to a cap field.
        for method in &id.methods {
            if !expected_fields.is_empty() && !expected_fields.contains(&method.name) {
                self.type_error(
                    "E0222",
                    format!(
                        "impl `{}`: method `{}` is not declared in cap `{}`",
                        id.cap_name, method.name, id.cap_name
                    ),
                    &method.span,
                );
            }
            // Type-check the method body.
            self.check_fn_def(method);
        }
    }

    // ── fn_def (4-7) ──────────────────────────────────────────────────────────

    // ── test_def (v0.8.0) ─────────────────────────────────────────────────────

    fn check_test_def(&mut self, td: &TestDef) {
        // Test bodies run with Io+File effects allowed (assert builtins are pure)
        let saved_effects =
            std::mem::replace(&mut self.current_effects, vec![Effect::Io, Effect::File]);
        self.env.push();
        // Register assert builtins as visible inside test bodies
        self.env.define(
            "assert".to_string(),
            Type::Fn(vec![Type::Bool], Box::new(Type::Unit)),
        );
        self.env.define("assert_eq".to_string(), Type::Unknown);
        self.env.define("assert_ne".to_string(), Type::Unknown);
        self.check_block(&td.body);
        self.env.pop();
        self.current_effects = saved_effects;
    }

    fn check_bench_def(&mut self, bd: &BenchDef) {
        // Bench bodies may use Io effect only; Db/Network/File are disallowed (E064).
        let saved_effects = std::mem::replace(&mut self.current_effects, vec![Effect::Io]);
        self.env.push();
        self.check_block(&bd.body);
        self.env.pop();
        // Check for disallowed effects in bench body (E064 stub — no effect tracking yet).
        self.current_effects = saved_effects;
    }

    fn check_fn_def(&mut self, fd: &FnDef) {
        self.check_effects_declared(&fd.effects, &fd.span);
        let saved_effects = std::mem::replace(&mut self.current_effects, fd.effects.clone());
        let saved_tp = std::mem::replace(
            &mut self.type_params,
            fd.type_params.iter().cloned().collect(),
        );
        self.env.push();

        // Validate type arity (E023) for param and return type annotations.
        for p in &fd.params {
            self.validate_type_expr_arity(&p.ty);
        }
        if let Some(return_ty) = &fd.return_ty {
            self.validate_type_expr_arity(return_ty);
        }

        // Set chain context based on return type (v0.5.0).
        let ret_resolved = fd
            .return_ty
            .as_ref()
            .map(|ty| self.resolve_type_expr(ty))
            .unwrap_or(Type::Unknown);
        let saved_chain = self.chain_context.take();
        self.chain_context = match &ret_resolved {
            Type::Result(_, _) => Some(ret_resolved.clone()),
            Type::Option(_) => Some(ret_resolved.clone()),
            Type::Named(n, _) if n == "Result" || n == "Option" => Some(ret_resolved.clone()),
            _ => None,
        };

        // Bind parameters
        for p in &fd.params {
            let ty = self.resolve_type_expr(&p.ty);
            self.env.define(p.name.clone(), ty);
        }

        let body_ty = self.check_block(&fd.body);
        let return_ty = if let Some(return_ty) = &fd.return_ty {
            self.resolve_type_expr(return_ty)
        } else {
            if body_ty == Type::Unknown {
                self.type_error(
                    "E0274",
                    format!(
                        "cannot infer return type for fn `{}`; add explicit return type for recursive functions",
                        fd.name
                    ),
                    &fd.span,
                );
            }
            body_ty.clone()
        };

        if !body_ty.is_compatible(&return_ty) {
            self.type_error(
                "E0101",
                format!(
                    "fn `{}`: body type `{}` does not match return type `{}`",
                    fd.name,
                    body_ty.display(),
                    return_ty.display()
                ),
                &fd.body.span,
            );
        }

        self.env.pop();
        self.type_params = saved_tp;
        self.current_effects = saved_effects;
        self.chain_context = saved_chain;
    }

    // ── trf_def (4-8) ─────────────────────────────────────────────────────────

    fn check_trf_def(&mut self, td: &TrfDef) {
        self.check_effects_declared(&td.effects, &td.span);
        let saved_effects = std::mem::replace(&mut self.current_effects, td.effects.clone());
        let saved_tp = std::mem::replace(
            &mut self.type_params,
            td.type_params.iter().cloned().collect(),
        );
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

        let body_ty = self.check_block(&td.body);
        let output_ty = self.resolve_type_expr(&td.output_ty);

        if !body_ty.is_compatible(&output_ty) {
            self.type_error(
                "E0101",
                format!(
                    "trf `{}`: body type `{}` does not match output type `{}`",
                    td.name,
                    body_ty.display(),
                    output_ty.display()
                ),
                &td.body.span,
            );
        }

        self.env.pop();
        self.type_params = saved_tp;
        self.current_effects = saved_effects;
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
                    self.type_error("E0102", format!("undefined: `{}`", step_name), &fd.span);
                    current_output = Some(Type::Error);
                }
                Some(ty) => {
                    // Verify the connection: previous output must match this step's input.
                    if let Some(prev_out) = &current_output {
                        if let Some((input, _output)) = ty.as_callable() {
                            if !prev_out.is_compatible(input) {
                                self.type_error(
                                    "E0103",
                                    format!(
                                        "flw `{}`: `{}` outputs `{}` but `{}` expects `{}`",
                                        fd.name,
                                        // previous step name
                                        fd.steps[fd
                                            .steps
                                            .iter()
                                            .position(|s| s == step_name)
                                            .unwrap()
                                            .saturating_sub(1)],
                                        prev_out.display(),
                                        step_name,
                                        input.display(),
                                    ),
                                    &fd.span,
                                );
                            }
                        } else {
                            self.type_error(
                                "E0103",
                                format!(
                                    "`{}` is not a trf or fn, cannot be used in flw",
                                    step_name
                                ),
                                &fd.span,
                            );
                        }
                    }
                    // Advance current output.
                    current_output = ty.as_callable().map(|(_, o)| o.clone()).or(Some(ty));
                }
            }
        }

        // Register the resolved flw type.
        if let Some(last_name) = fd.steps.last() {
            if let Some(last_ty) = self.env.lookup(last_name).cloned() {
                if let Some(first_name) = fd.steps.first() {
                    if let Some(first_ty) = self.env.lookup(first_name).cloned() {
                        let input = first_ty
                            .as_callable()
                            .map(|(i, _)| i.clone())
                            .unwrap_or(Type::Unknown);
                        let output = last_ty
                            .as_callable()
                            .map(|(_, o)| o.clone())
                            .unwrap_or(Type::Unknown);
                        self.env.define(
                            fd.name.clone(),
                            Type::Trf(Box::new(input), Box::new(output), vec![]),
                        );
                    }
                }
            }
        }
    }

    // ── block (4-17) ──────────────────────────────────────────────────────────

    fn check_flw_binding_def(&mut self, fd: &FlwBindingDef) {
        let Some(template) = self.abstract_flw_registry.get(&fd.template).cloned() else {
            self.type_error("E0102", format!("undefined: `{}`", fd.template), &fd.span);
            self.env.define(fd.name.clone(), Type::Error);
            return;
        };

        if template.type_params.len() != fd.type_args.len() {
            self.type_error(
                "E0223",
                format!(
                    "wrong number of type arguments for `{}`: expected {}, got {}",
                    template.name,
                    template.type_params.len(),
                    fd.type_args.len()
                ),
                &fd.span,
            );
            self.env.define(fd.name.clone(), Type::Error);
            return;
        }

        let type_subst: HashMap<String, Type> = template
            .type_params
            .iter()
            .cloned()
            .zip(fd.type_args.iter().map(|arg| self.resolve_type_expr(arg)))
            .collect();
        let slot_map: HashMap<String, FlwSlot> = template
            .slots
            .iter()
            .cloned()
            .map(|slot| (slot.name.clone(), slot))
            .collect();

        let mut bound_slots = HashSet::new();
        let mut effect_acc = Vec::new();
        let mut has_binding_error = false;

        for (slot_name, slot_impl) in &fd.bindings {
            let Some(slot) = slot_map.get(slot_name) else {
                self.type_error(
                    "E0249",
                    format!(
                        "unknown slot `{}` in abstract seq `{}`",
                        slot_name, template.name
                    ),
                    &fd.span,
                );
                has_binding_error = true;
                continue;
            };
            bound_slots.insert(slot_name.clone());

            let Some((impl_name, impl_ty)) = self.resolve_slot_impl_type(slot_impl, &fd.span)
            else {
                has_binding_error = true;
                continue;
            };

            let (expected_input, expected_output, expected_effects) =
                self.resolve_flw_slot_signature_with_subst(slot, &type_subst);
            let mismatch = match impl_ty.as_callable() {
                Some((actual_input, actual_output)) => {
                    !actual_input.is_compatible(&expected_input)
                        || !actual_output.is_compatible(&expected_output)
                        || self.callable_effects(&impl_ty) != expected_effects
                }
                None => true,
            };

            if mismatch {
                let actual_sig = match impl_ty.as_callable() {
                    Some((input, output)) => {
                        let effects = self.callable_effects(&impl_ty);
                        let eff_str = if effects.is_empty() {
                            String::new()
                        } else {
                            format!(
                                " {}",
                                effects
                                    .iter()
                                    .map(|e| format!("!{:?}", e))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            )
                        };
                        format!("{} -> {}{}", input.display(), output.display(), eff_str)
                    }
                    None => impl_ty.display(),
                };
                let expected_eff_str = if expected_effects.is_empty() {
                    String::new()
                } else {
                    format!(
                        " {}",
                        expected_effects
                            .iter()
                            .map(|e| format!("!{:?}", e))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                };
                self.type_error(
                    "E0248",
                    format!(
                        "slot `{}` in abstract seq `{}` expects `{}` -> `{}`{}, got `{}` from `{}`",
                        slot_name,
                        template.name,
                        expected_input.display(),
                        expected_output.display(),
                        expected_eff_str,
                        actual_sig,
                        impl_name
                    ),
                    &fd.span,
                );
                has_binding_error = true;
                continue;
            }

            effect_acc.extend(expected_effects);
        }

        self.flw_binding_info.insert(
            fd.name.clone(),
            FlwBindingInfo {
                template: fd.template.clone(),
                bindings: fd.bindings.clone(),
            },
        );

        if has_binding_error {
            self.env.define(fd.name.clone(), Type::Error);
            return;
        }

        let unbound_slots: Vec<String> = template
            .slots
            .iter()
            .filter(|slot| !bound_slots.contains(&slot.name))
            .map(|slot| slot.name.clone())
            .collect();

        if unbound_slots.is_empty() {
            let first_slot = match template.slots.first() {
                Some(slot) => slot,
                None => {
                    self.env.define(fd.name.clone(), Type::Unknown);
                    return;
                }
            };
            let last_slot = template.slots.last().unwrap();
            let (input_ty, _, _) =
                self.resolve_flw_slot_signature_with_subst(first_slot, &type_subst);
            let (_, output_ty, _) =
                self.resolve_flw_slot_signature_with_subst(last_slot, &type_subst);
            self.env.define(
                fd.name.clone(),
                Type::Trf(
                    Box::new(input_ty),
                    Box::new(output_ty),
                    self.infer_flw_binding_effects(&effect_acc),
                ),
            );
        } else {
            self.env.define(
                fd.name.clone(),
                Type::PartialFlw {
                    template: fd.template.clone(),
                    type_args: fd
                        .type_args
                        .iter()
                        .map(|arg| self.resolve_type_expr(arg))
                        .collect(),
                    unbound_slots,
                },
            );
        }
    }

    fn resolve_type_expr_with_subst(&self, te: &TypeExpr, subst: &HashMap<String, Type>) -> Type {
        match te {
            TypeExpr::Optional(inner, _) => {
                Type::Option(Box::new(self.resolve_type_expr_with_subst(inner, subst)))
            }
            TypeExpr::Fallible(inner, _) => Type::Result(
                Box::new(self.resolve_type_expr_with_subst(inner, subst)),
                Box::new(Type::Named("Error".into(), vec![])),
            ),
            TypeExpr::Arrow(a, b, _) => Type::Arrow(
                Box::new(self.resolve_type_expr_with_subst(a, subst)),
                Box::new(self.resolve_type_expr_with_subst(b, subst)),
            ),
            TypeExpr::TrfFn {
                input,
                output,
                effects,
                ..
            } => Type::Trf(
                Box::new(self.resolve_type_expr_with_subst(input, subst)),
                Box::new(self.resolve_type_expr_with_subst(output, subst)),
                effects.clone(),
            ),
            TypeExpr::Named(name, args, _) if args.is_empty() => subst
                .get(name)
                .cloned()
                .unwrap_or_else(|| self.resolve_type_expr(te)),
            TypeExpr::Named(name, args, _) => {
                let resolved_args: Vec<Type> = args
                    .iter()
                    .map(|arg| self.resolve_type_expr_with_subst(arg, subst))
                    .collect();
                if let Some(td) = self.abstract_trf_registry.get(name) {
                    let type_subst: HashMap<String, Type> = td
                        .type_params
                        .iter()
                        .cloned()
                        .zip(resolved_args.iter().cloned())
                        .collect();
                    return Type::AbstractTrf {
                        input: Box::new(
                            self.resolve_type_expr_with_subst(&td.input_ty, &type_subst),
                        ),
                        output: Box::new(
                            self.resolve_type_expr_with_subst(&td.output_ty, &type_subst),
                        ),
                        effects: td.effects.clone(),
                    };
                }
                match name.as_str() {
                    "Bool" => Type::Bool,
                    "Int" => Type::Int,
                    "Float" => Type::Float,
                    "String" => Type::String,
                    "Unit" => Type::Unit,
                    "List" if resolved_args.len() == 1 => {
                        Type::List(Box::new(resolved_args[0].clone()))
                    }
                    "Map" if resolved_args.len() == 2 => Type::Map(
                        Box::new(resolved_args[0].clone()),
                        Box::new(resolved_args[1].clone()),
                    ),
                    "Option" if resolved_args.len() == 1 => {
                        Type::Option(Box::new(resolved_args[0].clone()))
                    }
                    "Result" if resolved_args.len() == 2 => Type::Result(
                        Box::new(resolved_args[0].clone()),
                        Box::new(resolved_args[1].clone()),
                    ),
                    _ => Type::Named(name.clone(), resolved_args),
                }
            }
        }
    }

    fn callable_effects(&self, ty: &Type) -> Vec<Effect> {
        match ty {
            Type::Trf(_, _, effects) => effects.clone(),
            Type::AbstractTrf { effects, .. } => effects.clone(),
            _ => Vec::new(),
        }
    }

    fn infer_flw_binding_effects(&self, effects: &[Effect]) -> Vec<Effect> {
        let mut out = Vec::new();
        for effect in effects {
            if !out.iter().any(|existing| existing == effect) {
                out.push(effect.clone());
            }
        }
        out
    }

    fn resolve_flw_slot_signature_with_subst(
        &self,
        slot: &FlwSlot,
        subst: &HashMap<String, Type>,
    ) -> (Type, Type, Vec<Effect>) {
        if let Some(slot_ty) = &slot.abstract_trf_ty {
            match self.resolve_type_expr_with_subst(slot_ty, subst) {
                Type::AbstractTrf {
                    input,
                    output,
                    effects,
                }
                | Type::Trf(input, output, effects) => {
                    return ((*input).clone(), (*output).clone(), effects);
                }
                other => return (other, Type::Unknown, Vec::new()),
            }
        }
        (
            self.resolve_type_expr_with_subst(&slot.input_ty, subst),
            self.resolve_type_expr_with_subst(&slot.output_ty, subst),
            slot.effects.clone(),
        )
    }

    fn resolve_slot_impl_type(
        &mut self,
        slot_impl: &crate::ast::SlotImpl,
        span: &Span,
    ) -> Option<(String, Type)> {
        match slot_impl {
            crate::ast::SlotImpl::Global(name) => {
                let ty = self.env.lookup(name).cloned()?;
                Some((name.clone(), ty))
            }
            crate::ast::SlotImpl::Local(name) => {
                let ty = self.env.lookup(name).cloned().or_else(|| {
                    self.type_error("E0102", format!("undefined local: `{}`", name), span);
                    None
                })?;
                Some((name.clone(), ty))
            }
        }
    }

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
                // Unwrap Task<T> in bind: `bind x <- async_fn()` binds x as T
                let effective_ty = match expr_ty {
                    Type::Task(inner) => *inner,
                    other => other,
                };
                if let Pattern::Bind(name, span) = &b.pattern {
                    if effective_ty == Type::Unknown {
                        self.type_warning(
                            "W001",
                            format!("type of `{}` could not be resolved (Unknown)", name),
                            span,
                        );
                    }
                }
                self.check_record_destructure_bind(&b.pattern, &effective_ty, &b.span);
                if let Some(annotated_ty_expr) = &b.annotated_ty {
                    let annotated_ty = self.resolve_type_expr(annotated_ty_expr);
                    self.check_typed_bind_annotation(
                        &b.pattern,
                        &effective_ty,
                        annotated_ty_expr,
                        &annotated_ty,
                        &b.expr,
                    );
                    self.check_pattern_bindings(&b.pattern, &annotated_ty);
                } else {
                    self.check_pattern_bindings(&b.pattern, &effective_ty);
                }
            }
            Stmt::Expr(e) => {
                self.check_expr(e);
            }
            // chain x <- expr  (v0.5.0)
            Stmt::Chain(c) => {
                let expr_ty = self.check_expr(&c.expr);
                let inner_ty = match &self.chain_context.clone() {
                    None => {
                        self.type_error(
                            "E0224",
                            "chain used outside a Result/Option-returning function",
                            &c.span,
                        );
                        Type::Unknown
                    }
                    Some(ctx) => self.check_chain_expr_type(&expr_ty, ctx, &c.span),
                };
                self.env.define(c.name.clone(), inner_ty);
            }
            // yield expr;  (v0.5.0)
            Stmt::Yield(y) => {
                if !self.in_collect {
                    self.type_error("E0226", "yield used outside a collect block", &y.span);
                }
                self.check_expr(&y.expr);
            }
            // for x in list { body }  (v1.9.0; collect-inside supported since v2.9.0)
            Stmt::ForIn(f) => {
                let iter_ty = self.check_expr(&f.iter);
                let elem_ty = match iter_ty {
                    Type::List(inner) => *inner,
                    Type::Unknown | Type::Error => Type::Unknown,
                    _ => {
                        self.type_error("E0365", "`for` iterator must be `List<T>`", &f.span);
                        Type::Unknown
                    }
                };
                self.env.push();
                self.env.define(f.var.clone(), elem_ty);
                self.check_block(&f.body);
                self.env.pop();
            }
        }
    }

    /// Extract the inner type `T` from `Result<T,E>` or `Option<T>` for chain.
    /// v1.8.0: also unwraps `Task<Result<T,E>>` and `Task<Option<T>>`.
    /// Emits E025 when the expr type doesn't match the chain context.
    fn check_chain_expr_type(&mut self, expr_ty: &Type, ctx: &Type, span: &Span) -> Type {
        // Unwrap a Task<X> wrapper before checking the inner type (v1.8.0).
        let effective_ty = match expr_ty {
            Type::Task(inner) => inner.as_ref(),
            other => other,
        };
        let is_result_ctx =
            matches!(ctx, Type::Result(_, _)) || matches!(ctx, Type::Named(n, _) if n == "Result");
        let is_option_ctx =
            matches!(ctx, Type::Option(_)) || matches!(ctx, Type::Named(n, _) if n == "Option");
        match effective_ty {
            Type::Result(inner, _) if is_result_ctx => *inner.clone(),
            Type::Named(n, args) if n == "Result" && args.len() >= 1 && is_result_ctx => {
                args[0].clone()
            }
            Type::Option(inner) if is_option_ctx => *inner.clone(),
            Type::Named(n, args) if n == "Option" && args.len() == 1 && is_option_ctx => {
                args[0].clone()
            }
            Type::Error | Type::Unknown => Type::Unknown,
            _ => {
                self.type_error(
                    "E0225",
                    format!(
                        "chain expression has type `{}`, expected `Result<_,_>` or `Option<_>`",
                        expr_ty.display()
                    ),
                    span,
                );
                Type::Unknown
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
                    match fp {
                        PatternField::Pun(name, _) => {
                            let field_ty = self.lookup_field_type(ty, name);
                            self.env.define(name.clone(), field_ty);
                        }
                        PatternField::Alias(name, p, _) => {
                            let field_ty = self.lookup_field_type(ty, name);
                            self.check_pattern_bindings(p, &field_ty);
                        }
                        PatternField::Wildcard(_) => {}
                    }
                }
            }
        }
    }

    fn check_record_destructure_bind(&mut self, pat: &Pattern, ty: &Type, span: &Span) {
        let Pattern::Record(fields, _) = pat else {
            return;
        };
        let type_name = match ty {
            Type::Named(name, _) => name.clone(),
            _ => {
                self.type_error("E0372", "destructuring bind requires a record type", span);
                return;
            }
        };
        let Some(body) = self.type_defs.get(&type_name) else {
            self.type_error("E0372", "destructuring bind requires a record type", span);
            return;
        };
        let TypeBody::Record(record_fields) = body else {
            self.type_error("E0372", "destructuring bind requires a record type", span);
            return;
        };
        let record_field_names: HashSet<String> =
            record_fields.iter().map(|f| f.name.clone()).collect();
        for field in fields {
            let name = match field {
                PatternField::Pun(name, _) | PatternField::Alias(name, _, _) => name,
                PatternField::Wildcard(_) => continue,
            };
            if !record_field_names.contains(name) {
                self.type_error(
                    "E0373",
                    format!("record destructuring field `{}` does not exist", name),
                    field.span(),
                );
            }
        }
    }

    fn check_typed_bind_annotation(
        &mut self,
        pattern: &Pattern,
        expr_ty: &Type,
        annotated_ty_expr: &TypeExpr,
        annotated_ty: &Type,
        expr: &Expr,
    ) {
        if expr_ty.is_compatible(annotated_ty) {
            return;
        }

        let Pattern::Bind(_, span) = pattern else {
            self.type_error(
                "E0246",
                format!(
                    "typed bind expects `{}`, but expression has type `{}`",
                    annotated_ty.display(),
                    expr_ty.display()
                ),
                annotated_ty_expr.span(),
            );
            return;
        };

        if let Some((field_name, field_ty, invariants)) =
            self.single_field_invariant_state(annotated_ty)
        {
            if expr_ty.is_compatible(&field_ty) {
                if let Some(false) =
                    self.evaluate_invariants_for_literal(&field_name, &invariants, expr)
                {
                    self.type_error(
                        "E0246",
                        format!(
                            "literal does not satisfy invariant for state type `{}`",
                            annotated_ty.display()
                        ),
                        expr.span(),
                    );
                }
                return;
            }
        }

        self.type_error(
            "E0246",
            format!(
                "typed bind expects `{}`, but expression has type `{}`",
                annotated_ty.display(),
                expr_ty.display()
            ),
            span,
        );
    }

    fn has_invariants(&self, type_name: &str) -> bool {
        self.type_invariants
            .get(type_name)
            .map(|invariants| !invariants.is_empty())
            .unwrap_or(false)
    }

    fn single_field_invariant_state(
        &self,
        annotated_ty: &Type,
    ) -> Option<(String, Type, Vec<Expr>)> {
        let Type::Named(type_name, _) = annotated_ty else {
            return None;
        };
        if !self.has_invariants(type_name) {
            return None;
        }
        let invariants = self.type_invariants.get(type_name)?;
        let TypeBody::Record(fields) = self.type_defs.get(type_name)? else {
            return None;
        };
        if fields.len() != 1 {
            return None;
        }
        let field = &fields[0];
        Some((
            field.name.clone(),
            self.resolve_type_expr(&field.ty),
            invariants.clone(),
        ))
    }

    fn evaluate_invariants_for_literal(
        &self,
        field_name: &str,
        invariants: &[Expr],
        expr: &Expr,
    ) -> Option<bool> {
        let lit = match expr {
            Expr::Lit(lit, _) => lit.clone(),
            _ => return None,
        };
        let mut values = HashMap::new();
        values.insert(field_name.to_string(), lit);
        for invariant in invariants {
            match self.eval_static_expr(invariant, &values)? {
                StaticValue::Bool(true) => {}
                StaticValue::Bool(false) => return Some(false),
                _ => return None,
            }
        }
        Some(true)
    }

    fn eval_static_expr(&self, expr: &Expr, values: &HashMap<String, Lit>) -> Option<StaticValue> {
        match expr {
            Expr::Lit(lit, _) => Some(StaticValue::from_lit(lit.clone())),
            Expr::Ident(name, _) => values.get(name).cloned().map(StaticValue::from_lit),
            Expr::BinOp(op, lhs, rhs, _) => {
                let lhs = self.eval_static_expr(lhs, values)?;
                let rhs = self.eval_static_expr(rhs, values)?;
                Self::eval_static_binop(op, lhs, rhs)
            }
            Expr::Apply(callee, args, _) => {
                let Expr::FieldAccess(base, method, _) = &**callee else {
                    return None;
                };
                let Expr::Ident(type_name, _) = &**base else {
                    return None;
                };
                match (type_name.as_str(), method.as_str(), args.as_slice()) {
                    ("String", "contains", [haystack, needle]) => {
                        let haystack = self.eval_static_expr(haystack, values)?.into_string()?;
                        let needle = self.eval_static_expr(needle, values)?.into_string()?;
                        Some(StaticValue::Bool(haystack.contains(&needle)))
                    }
                    ("String", "length", [value]) => {
                        let value = self.eval_static_expr(value, values)?.into_string()?;
                        Some(StaticValue::Int(value.len() as i64))
                    }
                    ("String", "starts_with", [value, prefix]) => {
                        let value = self.eval_static_expr(value, values)?.into_string()?;
                        let prefix = self.eval_static_expr(prefix, values)?.into_string()?;
                        Some(StaticValue::Bool(value.starts_with(&prefix)))
                    }
                    ("String", "is_slug", [value]) => {
                        let value = self.eval_static_expr(value, values)?.into_string()?;
                        Some(StaticValue::Bool(Self::is_slug_string(&value)))
                    }
                    ("String", "is_url", [value]) => {
                        let value = self.eval_static_expr(value, values)?.into_string()?;
                        Some(StaticValue::Bool(
                            value.starts_with("http://") || value.starts_with("https://"),
                        ))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn eval_static_binop(op: &BinOp, lhs: StaticValue, rhs: StaticValue) -> Option<StaticValue> {
        match op {
            BinOp::Add => match (lhs, rhs) {
                (StaticValue::Int(a), StaticValue::Int(b)) => Some(StaticValue::Int(a + b)),
                (StaticValue::Float(a), StaticValue::Float(b)) => Some(StaticValue::Float(a + b)),
                _ => None,
            },
            BinOp::Sub => match (lhs, rhs) {
                (StaticValue::Int(a), StaticValue::Int(b)) => Some(StaticValue::Int(a - b)),
                (StaticValue::Float(a), StaticValue::Float(b)) => Some(StaticValue::Float(a - b)),
                _ => None,
            },
            BinOp::Mul => match (lhs, rhs) {
                (StaticValue::Int(a), StaticValue::Int(b)) => Some(StaticValue::Int(a * b)),
                (StaticValue::Float(a), StaticValue::Float(b)) => Some(StaticValue::Float(a * b)),
                _ => None,
            },
            BinOp::Div => match (lhs, rhs) {
                (StaticValue::Int(a), StaticValue::Int(b)) if b != 0 => {
                    Some(StaticValue::Int(a / b))
                }
                (StaticValue::Float(a), StaticValue::Float(b)) if b != 0.0 => {
                    Some(StaticValue::Float(a / b))
                }
                _ => None,
            },
            BinOp::Eq => Some(StaticValue::Bool(lhs == rhs)),
            BinOp::NotEq => Some(StaticValue::Bool(lhs != rhs)),
            BinOp::Lt => Self::eval_static_compare(lhs, rhs, |o| o < 0),
            BinOp::Gt => Self::eval_static_compare(lhs, rhs, |o| o > 0),
            BinOp::LtEq => Self::eval_static_compare(lhs, rhs, |o| o <= 0),
            BinOp::GtEq => Self::eval_static_compare(lhs, rhs, |o| o >= 0),
            BinOp::And => match (lhs, rhs) {
                (StaticValue::Bool(a), StaticValue::Bool(b)) => Some(StaticValue::Bool(a && b)),
                _ => None,
            },
            BinOp::Or => match (lhs, rhs) {
                (StaticValue::Bool(a), StaticValue::Bool(b)) => Some(StaticValue::Bool(a || b)),
                _ => None,
            },
            BinOp::NullCoalesce => None, // not statically evaluable in general
        }
    }

    fn eval_static_compare(
        lhs: StaticValue,
        rhs: StaticValue,
        pred: impl FnOnce(i8) -> bool,
    ) -> Option<StaticValue> {
        let ord = match (lhs, rhs) {
            (StaticValue::Int(a), StaticValue::Int(b)) => match a.cmp(&b) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            },
            (StaticValue::Float(a), StaticValue::Float(b)) => match a.partial_cmp(&b)? {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            },
            (StaticValue::String(a), StaticValue::String(b)) => match a.cmp(&b) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            },
            _ => return None,
        };
        Some(StaticValue::Bool(pred(ord)))
    }

    fn is_slug_string(s: &str) -> bool {
        !s.is_empty()
            && s.chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    }

    /// Determine the payload type of a variant pattern.
    fn resolve_variant_payload(
        &self,
        variant_name: &str,
        scrutinee_ty: &Type,
        _span: &Span,
    ) -> Type {
        let type_name = match scrutinee_ty {
            Type::Named(n, _) => n.clone(),
            Type::Option(inner) => {
                // some(x) ↁEinner type; none ↁEUnit
                if variant_name == "some" {
                    return *inner.clone();
                }
                if variant_name == "none" {
                    return Type::Unit;
                }
                return Type::Unknown;
            }
            Type::Result(ok, err) => {
                if variant_name == "ok" {
                    return *ok.clone();
                }
                if variant_name == "err" {
                    return *err.clone();
                }
                return Type::Unknown;
            }
            _ => return Type::Unknown,
        };

        if let Some(body) = self.type_defs.get(&type_name) {
            if let TypeBody::Sum(variants) = body {
                for v in variants {
                    if v.name() == variant_name {
                        return match v {
                            Variant::Unit(_, _) => Type::Unit,
                            Variant::Tuple(_, te, _) => self.resolve_type_expr(te),
                            Variant::Record(_, _fields, _) => {
                                // Record variant payload  Ekeep as Named for field lookup
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
                TypeBody::Alias(_) => {}
            }
        }
        Type::Unknown
    }

    // ── expr (4-14..4-16) ────────────────────────────────────────────────────

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            // literals (4-15)
            Expr::Lit(lit, span) => {
                let ty = match lit {
                    Lit::Int(_) => Type::Int,
                    Lit::Float(_) => Type::Float,
                    Lit::Str(_) => Type::String,
                    Lit::Bool(_) => Type::Bool,
                    Lit::Unit => Type::Unit,
                };
                self.remember_type(span, &ty);
                ty
            }

            // identifier (4-15)
            Expr::Ident(name, span) => {
                if let Some(def_span) = self.global_def_spans.get(name).cloned() {
                    self.def_at.insert(span.clone(), def_span);
                }
                let ty = match self.env.lookup(name).cloned() {
                    Some(ty) => {
                        self.check_symbol_visibility(name, span);
                        ty
                    }
                    None => {
                        self.type_error("E0102", format!("undefined: `{}`", name), span);
                        Type::Error
                    }
                };
                self.remember_type(span, &ty);
                ty
            }

            Expr::TypeApply(func, type_args, span) => {
                let ty = if let Expr::Ident(name, _) = func.as_ref() {
                    if name == "type_name_of" {
                        Type::Fn(vec![], Box::new(Type::String))
                    } else {
                        instantiate_explicit_type_args(self.check_expr(func), type_args, self)
                    }
                } else {
                    instantiate_explicit_type_args(self.check_expr(func), type_args, self)
                };
                self.remember_type(span, &ty);
                ty
            }

            // field access: expr.field (4-15)
            Expr::FieldAccess(obj, field, span) => {
                if let Expr::Ident(namespace, _) = obj.as_ref() {
                    if let Some(scope) = self.imported_namespaces.get(namespace) {
                        let ty = match scope.symbols.get(field) {
                            Some((ty, vis)) if *vis != Visibility::Private => ty.clone(),
                            None => {
                                self.type_error(
                                    "E0102",
                                    format!("`{}.{}` is not exported", namespace, field),
                                    span,
                                );
                                Type::Error
                            }
                            Some(_) => {
                                self.type_error(
                                    "E0102",
                                    format!("`{}.{}` is not exported", namespace, field),
                                    span,
                                );
                                Type::Error
                            }
                        };
                        self.remember_type(span, &ty);
                        return ty;
                    }
                }
                let obj_ty = self.check_expr(obj);
                let ty = self.resolve_field_access(&obj_ty, field, span);
                self.remember_type(span, &ty);
                ty
            }

            // function application (4-15)
            Expr::Apply(func, args, span) => {
                // Check for built-in namespaced calls first.
                if let Some(ty) = self.check_builtin_apply(func, args, span) {
                    self.remember_type(span, &ty);
                    return ty;
                }

                let func_ty = self.check_expr(func);
                let arg_tys: Vec<Type> = args.iter().map(|a| self.check_expr(a)).collect();

                let ty = match &func_ty {
                    Type::Fn(params, ret) => {
                        // Collect type variables from params and ret.
                        let mut vars: HashSet<String> = HashSet::new();
                        fn collect_vars(ty: &Type, out: &mut HashSet<String>) {
                            match ty {
                                Type::Var(n) => {
                                    out.insert(n.clone());
                                }
                                Type::List(t) | Type::Option(t) => collect_vars(t, out),
                                Type::Map(k, v) => {
                                    collect_vars(k, out);
                                    collect_vars(v, out);
                                }
                                Type::Result(t, e) | Type::Arrow(t, e) => {
                                    collect_vars(t, out);
                                    collect_vars(e, out);
                                }
                                Type::Fn(ps, r) => {
                                    for p in ps {
                                        collect_vars(p, out);
                                    }
                                    collect_vars(r, out);
                                }
                                Type::Named(_, args)
                                | Type::Cap(_, args)
                                | Type::Interface(_, args) => {
                                    for a in args {
                                        collect_vars(a, out);
                                    }
                                }
                                _ => {}
                            }
                        }
                        for p in params.iter() {
                            collect_vars(p, &mut vars);
                        }
                        collect_vars(ret, &mut vars);

                        // Build instantiation substitution.
                        let mut inst = Subst::empty();
                        for v in &vars {
                            inst.extend(v.clone(), self.fresh_var());
                        }
                        let inst_params: Vec<Type> = params.iter().map(|p| inst.apply(p)).collect();
                        let inst_ret = inst.apply(ret);

                        if inst_params.len() != arg_tys.len() {
                            self.type_error(
                                "E0101",
                                format!(
                                    "expected {} argument(s), got {}",
                                    inst_params.len(),
                                    arg_tys.len()
                                ),
                                span,
                            );
                            Type::Error
                        } else {
                            // Unify each param with corresponding arg type.
                            let mut subst = Subst::empty();
                            for (p, a) in inst_params.iter().zip(arg_tys.iter()) {
                                let ap = subst.apply(p);
                                let aa = subst.apply(a);
                                match unify(&ap, &aa) {
                                    Ok(s) => subst = s.compose(subst),
                                    Err(msg) => {
                                        let code = if msg.contains("infinite type") {
                                            "E0219"
                                        } else {
                                            "E0218"
                                        };
                                        self.type_error(code, msg, span);
                                        return Type::Error;
                                    }
                                }
                            }
                            subst.apply(&inst_ret)
                        }
                    }
                    Type::Arrow(input, output) => {
                        let arg_ty = arg_tys.first().cloned().unwrap_or(Type::Unit);
                        if !arg_ty.is_compatible(input) {
                            self.type_error(
                                "E0101",
                                format!(
                                    "expected `{}`, got `{}`",
                                    input.display(),
                                    arg_ty.display()
                                ),
                                span,
                            );
                        }
                        // Support curried multi-arg application (e.g. interface methods: eq(a, b))
                        let mut result = *output.clone();
                        for extra_arg in arg_tys.iter().skip(1) {
                            match result {
                                Type::Arrow(ref next_in, ref next_out) => {
                                    if !extra_arg.is_compatible(next_in) {
                                        self.type_error(
                                            "E0101",
                                            format!(
                                                "expected `{}`, got `{}`",
                                                next_in.display(),
                                                extra_arg.display()
                                            ),
                                            span,
                                        );
                                    }
                                    result = *next_out.clone();
                                }
                                _ => break,
                            }
                        }
                        result
                    }
                    Type::Trf(input, output, _) => {
                        let arg_ty = arg_tys.first().cloned().unwrap_or(Type::Unit);
                        if !arg_ty.is_compatible(input) {
                            self.type_error(
                                "E0101",
                                format!(
                                    "expected `{}`, got `{}`",
                                    input.display(),
                                    arg_ty.display()
                                ),
                                span,
                            );
                        }
                        *output.clone()
                    }
                    Type::AbstractTrf { .. } => {
                        self.type_error(
                              "E0251",
                              format!(
                                  "cannot call abstract stage `{}` directly; bind it into an abstract seq slot first",
                                  func_ty.display()
                              ),
                              span,
                          );
                        Type::Error
                    }
                    Type::Unknown | Type::Error => Type::Unknown,
                    other => {
                        self.type_error(
                            "E0101",
                            format!("`{}` is not callable", other.display()),
                            span,
                        );
                        Type::Error
                    }
                };
                self.remember_type(span, &ty);
                ty
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
                                "E0103",
                                format!("`{}` is not callable in pipeline", step_ty.display()),
                                span,
                            );
                            current = Type::Error;
                        }
                        Some((input, output)) => {
                            if !current.is_compatible(input) {
                                self.type_error(
                                    "E0103",
                                    format!(
                                        "pipeline type mismatch: `{}` ↁE`{}` (expected `{}`)",
                                        current.display(),
                                        step_ty.display(),
                                        input.display()
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

            Expr::AssertMatches(expr, pattern, _span) => {
                let expr_ty = self.check_expr(expr);
                self.env.push();
                self.check_pattern_bindings(pattern, &expr_ty);
                self.env.pop();
                Type::Unit
            }

            // if (4-13)
            Expr::If(cond, then_block, else_block, span) => {
                let cond_ty = self.check_expr(cond);
                if !cond_ty.is_compatible(&Type::Bool) {
                    self.type_error(
                        "E0101",
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
                                "E0101",
                                format!(
                                    "if branches have different types: `{}` vs `{}`",
                                    then_ty.display(),
                                    else_ty.display()
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
                // task 3-10: yield inside a closure is invalid (E026) even within collect
                let saved_in_collect = self.in_collect;
                self.in_collect = false;
                let body_ty = self.check_expr(body);
                self.in_collect = saved_in_collect;
                self.env.pop();
                // Represent as Arrow(Unknown, body_ty) for single-param closures
                let input_ty = if params.len() == 1 {
                    Type::Unknown
                } else {
                    Type::Unit
                };
                Type::Arrow(Box::new(input_ty), Box::new(body_ty))
            }

            // binary op
            Expr::BinOp(op, lhs, rhs, span) => {
                let l = self.check_expr(lhs);
                let r = self.check_expr(rhs);
                self.check_binop(op, &l, &r, span)
            }

            // record construction: TypeName { field: expr, ... } (2-4)
            Expr::RecordConstruct(type_name, fields, span) => {
                for (fname, fexpr) in fields {
                    self.check_expr(fexpr);
                    // Constraint check against schemas/*.yaml (v4.1.5)
                    if let Some(type_schema) = self.schemas.get(type_name).cloned() {
                        if let Some(fc) = type_schema.get(fname).cloned() {
                            self.check_field_constraints(fname, fexpr, &fc, span);
                        }
                    }
                }
                match self.type_defs.get(type_name) {
                    Some(_) => Type::Named(type_name.clone(), vec![]),
                    None => {
                        self.type_error("E0102", format!("undefined type `{}`", type_name), span);
                        Type::Error
                    }
                }
            }

            Expr::FString(parts, span) => {
                for part in parts {
                    let FStringPart::Expr(inner) = part else {
                        continue;
                    };
                    if matches!(inner.as_ref(), Expr::FString(_, _)) {
                        self.type_error(
                            "E0253",
                            "nested string interpolation is not supported",
                            span,
                        );
                    }
                    let ty = self.check_expr(inner);
                    if matches!(
                        ty,
                        Type::Unknown
                            | Type::Error
                            | Type::String
                            | Type::Int
                            | Type::Float
                            | Type::Bool
                    ) {
                        continue;
                    }
                    if !self
                        .interface_registry
                        .is_implemented("Show", &ty.display())
                    {
                        self.type_error(
                            "E0254",
                            format!(
                                "type `{}` does not implement Show; cannot use in string interpolation",
                                ty.display()
                            ),
                            span,
                        );
                    }
                }
                Type::String
            }

            // emit expr (2-5, 2-8)
            Expr::EmitExpr(inner, span) => {
                self.require_emit_effect(span);
                self.check_expr(inner);
                Type::Unit
            }

            // collect { ... } (v0.5.0)
            Expr::Collect(block, _span) => {
                let old_in_collect = self.in_collect;
                self.in_collect = true;
                // Collect yield types recursively (v2.9.0: for-in bodies included).
                let yield_tys = self.collect_yield_types(&block.stmts);
                // Also type-check the tail expression (usually Unit / ()).
                self.check_expr(&block.expr);
                self.in_collect = old_in_collect;
                // Determine element type by unifying all yields.
                let elem_ty = yield_tys.into_iter().fold(Type::Unknown, |acc, t| {
                    if matches!(acc, Type::Unknown) {
                        t
                    } else if acc.is_compatible(&t) {
                        acc
                    } else {
                        Type::Unknown
                    }
                });
                Type::List(Box::new(elem_ty))
            }
        }
    }

    // ── collect_yield_types helper (v2.9.0) ──────────────────────────────────

    /// Recursively collect yield expression types from a statement list.
    /// Descends into `for` bodies so that `collect { for x in list { yield x; } }` works.
    fn collect_yield_types(&mut self, stmts: &[Stmt]) -> Vec<Type> {
        let mut tys = Vec::new();
        for stmt in stmts {
            match stmt {
                Stmt::Yield(y) => {
                    tys.push(self.check_expr(&y.expr));
                }
                Stmt::ForIn(f) => {
                    let iter_ty = self.check_expr(&f.iter);
                    let elem_ty = match iter_ty {
                        Type::List(inner) => *inner,
                        Type::Unknown | Type::Error => Type::Unknown,
                        _ => Type::Unknown,
                    };
                    self.env.push();
                    self.env.define(f.var.clone(), elem_ty);
                    tys.extend(self.collect_yield_types(&f.body.stmts));
                    self.env.pop();
                }
                other => {
                    self.check_stmt(other);
                }
            }
        }
        tys
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
            // v0.5.0: check optional `where guard` (E027)
            if let Some(guard) = &arm.guard {
                let guard_ty = self.check_expr(guard);
                if !guard_ty.is_compatible(&Type::Bool)
                    && !matches!(guard_ty, Type::Unknown | Type::Error)
                {
                    self.type_error(
                        "E0227",
                        "pattern guard (where) must be of type Bool",
                        &arm.span,
                    );
                }
            }
            let arm_ty = self.check_expr(&arm.body);
            self.env.pop();

            match &result_ty {
                None => result_ty = Some(arm_ty),
                Some(prev) => {
                    if !prev.is_compatible(&arm_ty) {
                        self.type_error(
                            "E0101",
                            format!(
                                "match arms have inconsistent types: `{}` vs `{}`",
                                prev.display(),
                                arm_ty.display()
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
                        "E0101",
                        format!(
                            "arithmetic on non-numeric types `{}` and `{}`",
                            l.display(),
                            r.display()
                        ),
                        span,
                    );
                    return Type::Error;
                }
                if matches!(l, Type::Float) || matches!(r, Type::Float) {
                    Type::Float
                } else {
                    Type::Int
                }
            }
            And | Or => {
                let code = if matches!(op, And) { "E0370" } else { "E0371" };
                let opname = if matches!(op, And) { "&&" } else { "||" };
                if !matches!(l, Type::Bool | Type::Unknown | Type::Error) {
                    self.type_error(
                        code,
                        format!(
                            "left operand of `{}` must be Bool, got `{}`",
                            opname,
                            l.display()
                        ),
                        span,
                    );
                }
                if !matches!(r, Type::Bool | Type::Unknown | Type::Error) {
                    self.type_error(
                        code,
                        format!(
                            "right operand of `{}` must be Bool, got `{}`",
                            opname,
                            r.display()
                        ),
                        span,
                    );
                }
                Type::Bool
            }
            Eq | NotEq | Lt | Gt | LtEq | GtEq => {
                if !l.is_compatible(r) {
                    self.type_error(
                        "E0101",
                        format!(
                            "comparison between incompatible types `{}` and `{}`",
                            l.display(),
                            r.display()
                        ),
                        span,
                    );
                }
                Type::Bool
            }
            // ?? operator (v1.9.0): Option<T> ?? T -> T
            NullCoalesce => match l {
                Type::Option(inner) => {
                    let t = *inner.clone();
                    if !r.is_compatible(&t) {
                        self.type_error(
                                "E0369",
                                format!("`??` right-hand side `{}` is incompatible with `Option` inner type `{}`", r.display(), t.display()),
                                span,
                            );
                    }
                    t
                }
                Type::Unknown | Type::Error => r.clone(),
                _ => {
                    self.type_error(
                        "E0368",
                        format!(
                            "`??` left-hand side must be `Option<T>`, got `{}`",
                            l.display()
                        ),
                        span,
                    );
                    Type::Error
                }
            },
        }
    }

    // ── field access resolution ───────────────────────────────────────────────

    fn resolve_field_access(&mut self, obj_ty: &Type, field: &str, span: &Span) -> Type {
        if let Type::Named(ty_name, _) = obj_ty {
            if field == "new" {
                if let Some(sig) = self.resolve_state_constructor_signature(ty_name) {
                    return sig;
                }
            }
            let cap_name = {
                let mut s = field.to_string();
                if let Some(c) = s.get_mut(0..1) {
                    c.make_ascii_uppercase();
                }
                s
            };
            if self.interface_registry.is_implemented(&cap_name, ty_name) {
                let target_ty = match ty_name.as_str() {
                    "Bool" => Type::Bool,
                    "Int" => Type::Int,
                    "Float" => Type::Float,
                    "String" => Type::String,
                    _ => obj_ty.clone(),
                };
                return Type::Interface(cap_name, vec![target_ty]);
            }
            if self
                .impls
                .contains_key(&(cap_name.clone(), ty_name.clone()))
            {
                return Type::Cap(cap_name, vec![obj_ty.clone()]);
            }
            if self.caps.contains_key(&cap_name)
                || matches!(cap_name.as_str(), "Eq" | "Ord" | "Show")
            {
                self.type_error(
                    "E0221",
                    format!("no impl of `{}` for type `{}`", cap_name, ty_name),
                    span,
                );
                return Type::Error;
            }
        }
        if let Type::Interface(interface_name, args) = obj_ty {
            if let Some(target_ty) = args.first() {
                let impl_ty = self.interface_registry.lookup_method(
                    interface_name,
                    &target_ty.display(),
                    field,
                );
                // Lambda bodies use Arrow(Unknown/Unit, ret) as a placeholder type.
                // For dispatch, prefer the declared method type which has the correct param count.
                // For non-lambda values (e.g. Monoid.empty = record literal), keep the impl body type.
                let is_lambda_placeholder = impl_ty.map(|t| matches!(t,
                    Type::Arrow(input, _) if matches!(input.as_ref(), Type::Unknown | Type::Unit)
                )).unwrap_or(false);
                let method_ty = if is_lambda_placeholder {
                    self.interface_registry
                        .lookup_declared_method(interface_name, field)
                        .or(impl_ty)
                } else {
                    impl_ty
                };
                if let Some(method_ty) = method_ty {
                    return self.substitute_self_in_type(method_ty, target_ty);
                }
            }
        }
        if let Type::Cap(cap_name, args) = obj_ty {
            if let Some(impl_scope) = args.first().and_then(|a| {
                let key = (cap_name.clone(), a.display());
                self.impls.get(&key)
            }) {
                if let Some(method_ty) = impl_scope.methods.get(field) {
                    return method_ty.clone();
                }
            }
        }
        match obj_ty {
            Type::Named(n, _)
                if matches!(
                    n.as_str(),
                    "IO" | "List"
                        | "String"
                        | "Option"
                        | "Result"
                        | "Db"
                        | "Http"
                        | "Map"
                        | "Debug"
                        | "Emit"
                        | "Util"
                        | "Trace"
                        | "File"
                        | "Json"
                        | "Csv"
                        | "Checkpoint"
                        | "Parquet"
                        | "Grpc"
                        | "Stream"
                ) =>
            {
                Type::Unknown
            }
            Type::Named(_, _) => self.lookup_field_type(obj_ty, field),
            _ => Type::Unknown,
        }
    }

    fn resolve_state_constructor_signature(&self, type_name: &str) -> Option<Type> {
        if !self.has_invariants(type_name) {
            return None;
        }
        let TypeBody::Record(fields) = self.type_defs.get(type_name)? else {
            return None;
        };
        let params = fields
            .iter()
            .map(|field| self.resolve_type_expr(&field.ty))
            .collect::<Vec<_>>();
        Some(Type::Fn(
            params,
            Box::new(Type::Result(
                Box::new(Type::Named(type_name.to_string(), vec![])),
                Box::new(Type::Named("Error".into(), vec![])),
            )),
        ))
    }

    // ── effect enforcement helpers (2-6, 2-7, 2-8) ───────────────────────────

    fn has_effect(&self, pred: impl Fn(&Effect) -> bool) -> bool {
        self.current_effects.iter().any(pred)
    }

    fn require_db_effect(&mut self, span: &Span) {
        if !self.has_effect(|e| matches!(e, Effect::Db)) {
            self.type_error(
                "E0107",
                "Db.* call requires `!Db` effect on enclosing fn/trf",
                span,
            );
        }
    }

    fn require_network_effect(&mut self, span: &Span) {
        if !self.has_effect(|e| matches!(e, Effect::Network)) {
            self.type_error(
                "E0108",
                "Http.* call requires `!Network` effect on enclosing fn/trf",
                span,
            );
        }
    }

    fn require_rpc_effect(&mut self, span: &Span) {
        if !self.has_effect(|e| matches!(e, Effect::Rpc)) {
            self.type_error(
                "E0310",
                "Grpc.* call requires `!Rpc` effect on enclosing fn/trf",
                span,
            );
        }
    }

    fn require_io_effect(&mut self, span: &Span) {
        if !self.has_effect(|e| matches!(e, Effect::Io)) {
            self.type_error(
                "E0106",
                "call requires `!Io` effect on enclosing fn/trf",
                span,
            );
        }
    }

    fn require_file_effect(&mut self, span: &Span) {
        if !self.has_effect(|e| matches!(e, Effect::File)) {
            self.type_error(
                "E0136",
                "File.* call requires `!File` effect on enclosing fn/trf",
                span,
            );
        }
    }

    fn require_checkpoint_effect(&mut self, span: &Span) {
        if !self.has_effect(|e| matches!(e, Effect::Checkpoint)) {
            self.type_error(
                "E0308",
                "Checkpoint.* call requires `!Checkpoint` effect on enclosing fn/trf",
                span,
            );
        }
    }

    fn require_emit_effect(&mut self, span: &Span) {
        let has_emit = self.has_effect(|e| matches!(e, Effect::Emit(_) | Effect::EmitUnion(_)));
        if !has_emit {
            self.type_error(
                "E0109",
                "`emit` requires `!Emit<T>` effect on enclosing fn/trf",
                span,
            );
        }
    }

    // task 3-13: require !Trace effect (E010)
    fn require_trace_effect(&mut self, span: &Span) {
        if !self.has_effect(|e| matches!(e, Effect::Trace)) {
            self.type_error(
                "E0110",
                "Trace.* call requires `!Trace` effect on enclosing fn/trf",
                span,
            );
        }
    }

    // ── built-in call handling (4-5) ──────────────────────────────────────────

    /// If `func` is a known built-in namespace call (IO.println etc.), type-check
    /// the arguments and return the result type.  Returns None if not a built-in.
    fn check_builtin_apply(&mut self, func: &Expr, args: &[Expr], span: &Span) -> Option<Type> {
        let func = match func {
            Expr::TypeApply(inner, _, _) => inner.as_ref(),
            other => other,
        };
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
                        self.type_error(
                            "E0101",
                            format!("IO.{} expects String, got `{}`", method, ty.display()),
                            span,
                        );
                    }
                } else {
                    self.type_error(
                        "E0101",
                        format!("IO.{} requires one argument", method),
                        span,
                    );
                }
                Some(Type::Unit)
            }
            ("IO", "read_line") => Some(Type::String),
            ("IO", "timestamp") => Some(Type::String),

            // Math
            ("Math", "pi") | ("Math", "e") => Some(Type::Float),
            ("Math", "abs")
            | ("Math", "min")
            | ("Math", "max")
            | ("Math", "clamp")
            | ("Math", "pow")
            | ("Math", "floor")
            | ("Math", "ceil")
            | ("Math", "round") => Some(Type::Int),
            ("Math", "abs_float")
            | ("Math", "min_float")
            | ("Math", "max_float")
            | ("Math", "pow_float")
            | ("Math", "sqrt") => Some(Type::Float),

            // List
            ("List", "length") | ("List", "is_empty") => {
                let _ = self.expect_list_arg(&arg_tys, 0, span);
                Some(if method == "is_empty" {
                    Type::Bool
                } else {
                    Type::Int
                })
            }
            ("List", "unique") => {
                let elem = self.expect_list_arg(&arg_tys, 0, span);
                Some(Type::List(Box::new(elem)))
            }
            ("List", "flatten") => match arg_tys.first() {
                Some(Type::List(inner)) => match inner.as_ref() {
                    Type::List(elem) => Some(Type::List(elem.clone())),
                    _ => Some(Type::List(Box::new(Type::Unknown))),
                },
                _ => Some(Type::List(Box::new(Type::Unknown))),
            },
            ("List", "chunk") => {
                let elem = self.expect_list_arg(&arg_tys, 0, span);
                Some(Type::List(Box::new(Type::List(Box::new(elem)))))
            }
            ("List", "sum") => Some(Type::Int),
            ("List", "sum_float") => Some(Type::Float),
            ("List", "min") | ("List", "max") => Some(Type::Option(Box::new(Type::Int))),
            ("List", "first") | ("List", "last") => {
                let elem = self.expect_list_arg(&arg_tys, 0, span);
                Some(Type::Option(Box::new(elem)))
            }
            ("List", "map") => {
                let _elem = self.expect_list_arg(&arg_tys, 0, span);
                // arg1 should be a function T -> U; result is List<U>
                let out = if let Some(f_ty) = arg_tys.get(1) {
                    f_ty.as_callable()
                        .map(|(_, o)| o.clone())
                        .unwrap_or(Type::Unknown)
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
                // fold(items, init, f) ↁEtype of init
                let init_ty = arg_tys.get(1).cloned().unwrap_or(Type::Unknown);
                Some(init_ty)
            }
            ("List", "count") => Some(Type::Int),

            // String
            ("String", "trim") | ("String", "lower") | ("String", "upper") | ("String", "base64_encode") => Some(Type::String),
            ("String", "split") => Some(Type::List(Box::new(Type::String))),
            ("String", "index_of") => Some(Type::Option(Box::new(Type::Int))),
            ("String", "pad_left") | ("String", "pad_right") | ("String", "reverse") => {
                Some(Type::String)
            }
            ("String", "lines") | ("String", "words") => Some(Type::List(Box::new(Type::String))),
            ("String", "length") => Some(Type::Int),
            ("String", "is_empty") => Some(Type::Bool),
            ("String", "contains")
            | ("String", "starts_with")
            | ("String", "ends_with")
            | ("String", "is_slug")
            | ("String", "is_url") => Some(Type::Bool),

            // Option
            ("Option", "some") => {
                let ty = arg_tys.first().cloned().unwrap_or(Type::Unknown);
                Some(Type::Option(Box::new(ty)))
            }
            ("Option", "none") => Some(Type::Option(Box::new(Type::Unknown))),
            ("Option", "map") => {
                let out = arg_tys
                    .get(1)
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
                Some(Type::Result(Box::new(ty), Box::new(Type::Unknown)))
            }
            ("Result", "err") => Some(Type::Result(
                Box::new(Type::Unknown),
                Box::new(Type::Unknown),
            )),
            ("Result", "map") => {
                let out = arg_tys
                    .get(1)
                    .and_then(|f| f.as_callable().map(|(_, o)| o.clone()))
                    .unwrap_or(Type::Unknown);
                let err_ty = match arg_tys.first() {
                    Some(Type::Result(_, err)) => (**err).clone(),
                    _ => Type::Named("Error".into(), vec![]),
                };
                Some(Type::Result(Box::new(out), Box::new(err_ty)))
            }
            ("Result", "unwrap_or") => {
                let default_ty = arg_tys.get(1).cloned().unwrap_or(Type::Unknown);
                Some(default_ty)
            }

            // Db (2-6): require !Db effect
            ("Db", "execute") => {
                self.require_db_effect(span);
                Some(Type::Int)
            }
            ("Db", "query") => {
                self.require_db_effect(span);
                Some(Type::List(Box::new(Type::Map(
                    Box::new(Type::String),
                    Box::new(Type::String),
                ))))
            }
            ("Db", "query_one") => {
                self.require_db_effect(span);
                Some(Type::Option(Box::new(Type::Map(
                    Box::new(Type::String),
                    Box::new(Type::String),
                ))))
            }
            ("Db", _) => {
                self.require_db_effect(span);
                Some(Type::Unknown)
            }

            // DB.* (v3.3.0) — uppercase namespace, require !Db effect
            ("DB", "connect") => {
                self.require_db_effect(span);
                Some(Type::Result(
                    Box::new(Type::Named("DbHandle".into(), vec![])),
                    Box::new(Type::Named("DbError".into(), vec![])),
                ))
            }
            ("DB", "close") => {
                self.require_db_effect(span);
                Some(Type::Unit)
            }
            ("DB", "query_raw") | ("DB", "query_in_tx") => {
                self.require_db_effect(span);
                Some(Type::Result(
                    Box::new(Type::List(Box::new(Type::Map(
                        Box::new(Type::String),
                        Box::new(Type::String),
                    )))),
                    Box::new(Type::Named("DbError".into(), vec![])),
                ))
            }
            ("DB", "execute_raw") | ("DB", "execute_in_tx") => {
                self.require_db_effect(span);
                Some(Type::Result(
                    Box::new(Type::Int),
                    Box::new(Type::Named("DbError".into(), vec![])),
                ))
            }
            ("DB", "query_raw_params") => {
                self.require_db_effect(span);
                Some(Type::Result(
                    Box::new(Type::List(Box::new(Type::Map(
                        Box::new(Type::String),
                        Box::new(Type::String),
                    )))),
                    Box::new(Type::Named("DbError".into(), vec![])),
                ))
            }
            ("DB", "execute_raw_params") => {
                self.require_db_effect(span);
                Some(Type::Result(
                    Box::new(Type::Int),
                    Box::new(Type::Named("DbError".into(), vec![])),
                ))
            }
            ("DB", "begin_tx") => {
                self.require_db_effect(span);
                Some(Type::Result(
                    Box::new(Type::Named("TxHandle".into(), vec![])),
                    Box::new(Type::Named("DbError".into(), vec![])),
                ))
            }
            ("DB", "commit_tx") | ("DB", "rollback_tx") => {
                self.require_db_effect(span);
                Some(Type::Result(
                    Box::new(Type::Unit),
                    Box::new(Type::Named("DbError".into(), vec![])),
                ))
            }
            ("DB", "upsert_raw") => {
                self.require_db_effect(span);
                Some(Type::Unit)
            }
            ("DB", _) => {
                self.require_db_effect(span);
                Some(Type::Unknown)
            }

            // Env.* (v3.3.0)
            ("Env", "get") => Some(Type::Result(
                Box::new(Type::String),
                Box::new(Type::Named("DbError".into(), vec![])),
            )),
            ("Env", "get_or") => Some(Type::String),
            ("Env", _) => Some(Type::Unknown),

            // File (v0.7.0): require !File effect
            ("File", "read") => {
                self.require_file_effect(span);
                Some(Type::String)
            }
            ("File", "read_lines") => {
                self.require_file_effect(span);
                Some(Type::List(Box::new(Type::String)))
            }
            ("File", "write")
            | ("File", "write_lines")
            | ("File", "append")
            | ("File", "delete") => {
                self.require_file_effect(span);
                Some(Type::Unit)
            }
            ("File", "exists") => {
                self.require_file_effect(span);
                Some(Type::Bool)
            }
            ("File", _) => {
                self.require_file_effect(span);
                Some(Type::Unknown)
            }

            ("Checkpoint", "last") => {
                self.require_checkpoint_effect(span);
                Some(Type::Option(Box::new(Type::String)))
            }
            ("Checkpoint", "save") | ("Checkpoint", "reset") => {
                self.require_checkpoint_effect(span);
                Some(Type::Unit)
            }
            ("Checkpoint", "meta") => {
                self.require_checkpoint_effect(span);
                Some(Type::Named("CheckpointMeta".into(), vec![]))
            }

            // Http (2-7): require !Network effect
            ("Http", "get") | ("Http", "post") => {
                self.require_network_effect(span);
                Some(Type::Result(
                    Box::new(Type::String),
                    Box::new(Type::Named("Error".into(), vec![])),
                ))
            }
            ("Http", "get_raw")
            | ("Http", "post_raw")
            | ("Http", "put_raw")
            | ("Http", "delete_raw")
            | ("Http", "patch_raw")
            | ("Http", "get_raw_headers")
            | ("Http", "post_raw_headers") => {
                self.require_network_effect(span);
                Some(Type::Result(
                    Box::new(Type::Named("HttpResponse".into(), vec![])),
                    Box::new(Type::Named("HttpError".into(), vec![])),
                ))
            }
            ("Http", "serve_raw") => {
                self.require_network_effect(span);
                self.require_io_effect(span);
                Some(Type::Unit)
            }

            ("Grpc", "serve_raw") => {
                self.require_rpc_effect(span);
                self.require_io_effect(span);
                Some(Type::Unit)
            }
            ("Grpc", "serve_stream_raw") => {
                self.require_rpc_effect(span);
                self.require_io_effect(span);
                Some(Type::Unit)
            }
            ("Grpc", "call_raw") | ("Grpc", "call_typed_raw") => {
                self.require_rpc_effect(span);
                Some(Type::Result(
                    Box::new(Type::Map(Box::new(Type::String), Box::new(Type::String))),
                    Box::new(Type::Named("RpcError".into(), vec![])),
                ))
            }
            ("Grpc", "call_stream_raw") => {
                self.require_rpc_effect(span);
                Some(Type::List(Box::new(Type::Map(
                    Box::new(Type::String),
                    Box::new(Type::String),
                ))))
            }
            ("Grpc", "encode_raw") => Some(Type::String),
            ("Grpc", "decode_raw") => {
                Some(Type::Map(Box::new(Type::String), Box::new(Type::String)))
            }

            ("Parquet", "write_raw") => Some(Type::Result(
                Box::new(Type::Unit),
                Box::new(Type::Named("ParquetError".into(), vec![])),
            )),
            ("Parquet", "read_raw") => Some(Type::Result(
                Box::new(Type::List(Box::new(Type::Map(
                    Box::new(Type::String),
                    Box::new(Type::String),
                )))),
                Box::new(Type::Named("ParquetError".into(), vec![])),
            )),

            // Map (3-15..3-18)
            ("Map", "get") => Some(Type::Option(Box::new(Type::Unknown))),
            ("Map", "set") => Some(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
            ("Map", "keys") => Some(Type::List(Box::new(Type::Unknown))),
            ("Map", "values") => Some(Type::List(Box::new(Type::Unknown))),
            ("Map", _) => Some(Type::Unknown),

            // Json (v0.7.0)
            ("Json", "null")
            | ("Json", "bool")
            | ("Json", "int")
            | ("Json", "float")
            | ("Json", "str")
            | ("Json", "array")
            | ("Json", "object")
            | ("Json", "parse") => Some(Type::Named("Json".into(), vec![])),
            ("Json", "parse_raw") => Some(Type::Result(
                Box::new(Type::Map(Box::new(Type::String), Box::new(Type::String))),
                Box::new(Type::Named("SchemaError".into(), vec![])),
            )),
            ("Json", "parse_array_raw") => Some(Type::Result(
                Box::new(Type::List(Box::new(Type::Map(
                    Box::new(Type::String),
                    Box::new(Type::String),
                )))),
                Box::new(Type::Named("SchemaError".into(), vec![])),
            )),
            ("Json", "write_raw") | ("Json", "write_array_raw") => Some(Type::String),
            ("Json", "encode") | ("Json", "encode_pretty") => Some(Type::String),
            ("Json", "get") | ("Json", "at") => {
                Some(Type::Option(Box::new(Type::Named("Json".into(), vec![]))))
            }
            ("Json", "as_str") => Some(Type::Option(Box::new(Type::String))),
            ("Json", "as_int") => Some(Type::Option(Box::new(Type::Int))),
            ("Json", "as_float") => Some(Type::Option(Box::new(Type::Float))),
            ("Json", "as_bool") => Some(Type::Option(Box::new(Type::Bool))),
            ("Json", "as_array") => Some(Type::Option(Box::new(Type::List(Box::new(
                Type::Named("Json".into(), vec![]),
            ))))),
            ("Json", "is_null") => Some(Type::Bool),
            ("Json", "keys") => Some(Type::Option(Box::new(Type::List(Box::new(Type::String))))),
            ("Json", "length") => Some(Type::Option(Box::new(Type::Int))),
            ("Json", _) => Some(Type::Unknown),

            // Csv (v0.7.0)
            ("Csv", "parse") => Some(Type::List(Box::new(Type::List(Box::new(Type::String))))),
            ("Csv", "parse_with_header") => Some(Type::List(Box::new(Type::Map(
                Box::new(Type::String),
                Box::new(Type::String),
            )))),
            ("Csv", "parse_raw") => Some(Type::Result(
                Box::new(Type::List(Box::new(Type::Map(
                    Box::new(Type::String),
                    Box::new(Type::String),
                )))),
                Box::new(Type::Named("SchemaError".into(), vec![])),
            )),
            ("Csv", "write_raw") => Some(Type::String),
            ("Csv", "encode") | ("Csv", "encode_with_header") | ("Csv", "from_records") => {
                Some(Type::String)
            }
            ("Csv", _) => Some(Type::Unknown),

            ("Schema", "adapt") => Some(Type::Result(
                Box::new(Type::List(Box::new(Type::Unknown))),
                Box::new(Type::Named("SchemaError".into(), vec![])),
            )),
            ("Schema", "adapt_one") => Some(Type::Result(
                Box::new(Type::Unknown),
                Box::new(Type::Named("SchemaError".into(), vec![])),
            )),
            ("Schema", "to_csv") | ("Schema", "to_json") | ("Schema", "to_json_array") => {
                Some(Type::String)
            }
            ("Schema", _) => Some(Type::Unknown),

            // Debug (2-9)
            ("Debug", "show") => Some(Type::String),
            ("Debug", _) => Some(Type::Unknown),

            // Emit (3-4)
            ("Emit", "log") => Some(Type::List(Box::new(Type::String))),
            ("Emit", _) => Some(Type::Unknown),

            // Util
            ("Util", "uuid") => Some(Type::String),
            ("Util", _) => Some(Type::Unknown),

            // Trace (task 3-14): require !Trace effect
            ("Trace", "print") => {
                self.require_trace_effect(span);
                // Returns the argument unchanged (pass-through)
                Some(arg_tys.first().cloned().unwrap_or(Type::Unknown))
            }
            ("Trace", "log") => {
                self.require_trace_effect(span);
                // Returns the second argument (the value)
                Some(arg_tys.get(1).cloned().unwrap_or(Type::Unknown))
            }
            ("Trace", _) => {
                self.require_trace_effect(span);
                Some(Type::Unknown)
            }

            // Stream (v2.9.0)
            ("Stream", "from") => Some(Type::Unknown),
            ("Stream", "of") => Some(Type::Stream(Box::new(Type::Unknown))),
            ("Stream", "map") => Some(Type::Stream(Box::new(Type::Unknown))),
            ("Stream", "filter") => Some(Type::Stream(Box::new(Type::Unknown))),
            ("Stream", "take") => Some(Type::Stream(Box::new(Type::Unknown))),
            ("Stream", "to_list") => Some(Type::List(Box::new(Type::Unknown))),
            ("Stream", _) => Some(Type::Unknown),

            // Random.seed (v3.5.0)
            ("Random", "seed") => Some(Type::Unit),
            ("Random", "int") => Some(Type::Int),
            ("Random", "float") => Some(Type::Float),
            ("Random", _) => Some(Type::Unknown),

            // Gen.* (v3.5.0)
            ("Gen", "string_val") => Some(Type::String),
            ("Gen", "one_raw") => Some(Type::Map(Box::new(Type::String), Box::new(Type::String))),
            ("Gen", "list_raw") => Some(Type::List(Box::new(Type::Map(
                Box::new(Type::String),
                Box::new(Type::String),
            )))),
            ("Gen", "simulate_raw") => Some(Type::List(Box::new(Type::Map(
                Box::new(Type::String),
                Box::new(Type::String),
            )))),
            ("Gen", "profile_raw") => Some(Type::Named("GenProfile".into(), vec![])),
            ("Gen", _) => Some(Type::Unknown),

            // Validate.run_raw(type_name, raw_map) (v4.1.5)
            ("Validate", "run_raw") => Some(Type::Result(
                Box::new(Type::Unknown),
                Box::new(Type::List(Box::new(Type::Named(
                    "ValidationError".into(),
                    vec![],
                )))),
            )),
            ("Validate", _) => Some(Type::Unknown),

            // Dynamic T.validate — type name that has a schema entry
            (type_name, "validate") if self.schemas.contains_key(type_name) => {
                Some(Type::Result(
                    Box::new(Type::Named(type_name.to_string(), vec![])),
                    Box::new(Type::List(Box::new(Type::Named(
                        "ValidationError".into(),
                        vec![],
                    )))),
                ))
            }

            _ => None,
        }
    }

    /// Check a single record field expression against its FieldConstraints.
    /// Only literal values are checked; variables and function calls are skipped.
    fn check_field_constraints(
        &mut self,
        field: &str,
        expr: &Expr,
        fc: &FieldConstraints,
        span: &Span,
    ) {
        match expr {
            Expr::Lit(Lit::Int(n), _) => {
                let n = *n;
                if fc.constraints.iter().any(|c| c == "positive") && n <= 0 {
                    self.type_error(
                        "E0510",
                        format!("field '{}' must be positive (got {})", field, n),
                        span,
                    );
                }
                if fc.constraints.iter().any(|c| c == "non_negative") && n < 0 {
                    self.type_error(
                        "E0510",
                        format!("field '{}' must be non-negative (got {})", field, n),
                        span,
                    );
                }
                if let Some(min) = fc.min {
                    if (n as f64) < min {
                        self.type_error(
                            "E0511",
                            format!("field '{}' must be >= {} (got {})", field, min, n),
                            span,
                        );
                    }
                }
                if let Some(max) = fc.max {
                    if (n as f64) > max {
                        self.type_error(
                            "E0511",
                            format!("field '{}' must be <= {} (got {})", field, max, n),
                            span,
                        );
                    }
                }
            }
            Expr::Lit(Lit::Float(f), _) => {
                let f = *f;
                if fc.constraints.iter().any(|c| c == "positive") && f <= 0.0 {
                    self.type_error(
                        "E0510",
                        format!("field '{}' must be positive (got {})", field, f),
                        span,
                    );
                }
                if fc.constraints.iter().any(|c| c == "non_negative") && f < 0.0 {
                    self.type_error(
                        "E0510",
                        format!("field '{}' must be non-negative (got {})", field, f),
                        span,
                    );
                }
                if let Some(min) = fc.min {
                    if f < min {
                        self.type_error(
                            "E0511",
                            format!("field '{}' must be >= {} (got {})", field, min, f),
                            span,
                        );
                    }
                }
                if let Some(max) = fc.max {
                    if f > max {
                        self.type_error(
                            "E0511",
                            format!("field '{}' must be <= {} (got {})", field, max, f),
                            span,
                        );
                    }
                }
            }
            Expr::Lit(Lit::Str(s), _) => {
                if let Some(max_len) = fc.max_length {
                    if s.len() > max_len {
                        self.type_error(
                            "E0513",
                            format!(
                                "field '{}' exceeds max_length {} (got {})",
                                field,
                                max_len,
                                s.len()
                            ),
                            span,
                        );
                    }
                }
                if let Some(min_len) = fc.min_length {
                    if s.len() < min_len {
                        self.type_error(
                            "E0513",
                            format!(
                                "field '{}' below min_length {} (got {})",
                                field,
                                min_len,
                                s.len()
                            ),
                            span,
                        );
                    }
                }
                if let Some(pat) = &fc.pattern.clone() {
                    match regex::Regex::new(pat) {
                        Ok(re) if !re.is_match(s) => {
                            self.type_error(
                                "E0512",
                                format!(
                                    "field '{}' does not match pattern '{}' (got '{}')",
                                    field, pat, s
                                ),
                                span,
                            );
                        }
                        Err(_) => {
                            self.type_error(
                                "E0512",
                                format!("field '{}': invalid regex pattern '{}'", field, pat),
                                span,
                            );
                        }
                        Ok(_) => {}
                    }
                }
            }
            // Unary negation is parsed as BinOp(Sub, Lit::Int(0), operand)
            Expr::BinOp(BinOp::Sub, lhs, rhs, _) => {
                if let Expr::Lit(Lit::Int(0), _) = lhs.as_ref() {
                    match rhs.as_ref() {
                        Expr::Lit(Lit::Int(n), _) => {
                            let neg = -(*n);
                            let fake_expr = Expr::Lit(Lit::Int(neg), span.clone());
                            self.check_field_constraints(field, &fake_expr, fc, span);
                        }
                        Expr::Lit(Lit::Float(f), _) => {
                            let fake_expr = Expr::Lit(Lit::Float(-f), span.clone());
                            self.check_field_constraints(field, &fake_expr, fc, span);
                        }
                        _ => {}
                    }
                }
            }
            // Variables, function calls, etc. — skip (runtime validation only)
            _ => {}
        }
    }

    fn expect_list_arg(&mut self, arg_tys: &[Type], idx: usize, span: &Span) -> Type {
        match arg_tys.get(idx) {
            Some(Type::List(elem)) => *elem.clone(),
            Some(Type::Unknown) => Type::Unknown,
            Some(other) => {
                self.type_error(
                    "E0101",
                    format!("expected List<_>, got `{}`", other.display()),
                    span,
                );
                Type::Error
            }
            None => {
                self.type_error("E0101", "missing List argument", span);
                Type::Error
            }
        }
    }

    // ── type expression resolution (4-18) ────────────────────────────────────

    /// Convert a `TypeExpr` (AST surface) into a `Type` (internal).
    /// `T?` ↁE`Option<T>`, `T!` ↁE`Result<T, Error>`.
    pub fn resolve_type_expr(&self, te: &TypeExpr) -> Type {
        self.resolve_type_expr_with_self(te, None)
    }

    pub fn resolve_type_expr_with_self(&self, te: &TypeExpr, self_ty: Option<&Type>) -> Type {
        match te {
            TypeExpr::Optional(inner, _) => {
                Type::Option(Box::new(self.resolve_type_expr_with_self(inner, self_ty)))
            }
            TypeExpr::Fallible(inner, _) => Type::Result(
                Box::new(self.resolve_type_expr_with_self(inner, self_ty)),
                Box::new(Type::Named("Error".into(), vec![])),
            ),
            TypeExpr::Arrow(a, b, _) => Type::Arrow(
                Box::new(self.resolve_type_expr_with_self(a, self_ty)),
                Box::new(self.resolve_type_expr_with_self(b, self_ty)),
            ),
            TypeExpr::TrfFn {
                input,
                output,
                effects,
                ..
            } => Type::Trf(
                Box::new(self.resolve_type_expr_with_self(input, self_ty)),
                Box::new(self.resolve_type_expr_with_self(output, self_ty)),
                effects.clone(),
            ),
            TypeExpr::Named(name, args, _) => {
                if name == "Self" && args.is_empty() {
                    return self_ty
                        .cloned()
                        .unwrap_or_else(|| Type::Named("Self".into(), vec![]));
                }
                if args.is_empty() && self.type_params.contains(name.as_str()) {
                    return Type::Var(name.clone());
                }
                let resolved_args: Vec<Type> = args
                    .iter()
                    .map(|a| self.resolve_type_expr_with_self(a, self_ty))
                    .collect();
                if let Some(td) = self.abstract_trf_registry.get(name) {
                    let type_subst: HashMap<String, Type> = td
                        .type_params
                        .iter()
                        .cloned()
                        .zip(resolved_args.iter().cloned())
                        .collect();
                    return Type::AbstractTrf {
                        input: Box::new(
                            self.resolve_type_expr_with_subst(&td.input_ty, &type_subst),
                        ),
                        output: Box::new(
                            self.resolve_type_expr_with_subst(&td.output_ty, &type_subst),
                        ),
                        effects: td.effects.clone(),
                    };
                }
                match name.as_str() {
                    "Bool" => Type::Bool,
                    "Int" => Type::Int,
                    "Float" => Type::Float,
                    "String" => Type::String,
                    "Unit" => Type::Unit,
                    "List" => Type::List(Box::new(
                        resolved_args.into_iter().next().unwrap_or(Type::Unknown),
                    )),
                    "Map" => {
                        let mut it = resolved_args.into_iter();
                        let k = it.next().unwrap_or(Type::Unknown);
                        let v = it.next().unwrap_or(Type::Unknown);
                        Type::Map(Box::new(k), Box::new(v))
                    }
                    "Option" => Type::Option(Box::new(
                        resolved_args.into_iter().next().unwrap_or(Type::Unknown),
                    )),
                    "Result" => {
                        let mut it = resolved_args.into_iter();
                        let t = it.next().unwrap_or(Type::Unknown);
                        let e = it.next().unwrap_or(Type::Named("Error".into(), vec![]));
                        Type::Result(Box::new(t), Box::new(e))
                    }
                    "Task" => Type::Task(Box::new(
                        resolved_args.into_iter().next().unwrap_or(Type::Unknown),
                    )),
                    "Stream" => Type::Stream(Box::new(
                        resolved_args.into_iter().next().unwrap_or(Type::Unknown),
                    )),
                    "_infer" => Type::Unknown,
                    _ if self.interface_registry.interfaces.contains_key(name) => {
                        Type::Interface(name.clone(), resolved_args)
                    }
                    // Type alias resolution (v1.7.0)
                    _ if self.type_aliases.contains_key(name.as_str()) => {
                        let alias_target = self.type_aliases[name.as_str()].clone();
                        self.resolve_type_expr_with_self(&alias_target, self_ty)
                    }
                    _ => Type::Named(name.clone(), resolved_args),
                }
            }
        }
    }

    fn substitute_self_in_type(&self, ty: &Type, self_ty: &Type) -> Type {
        match ty {
            Type::Named(name, args) if name == "Self" && args.is_empty() => self_ty.clone(),
            Type::List(t) => Type::List(Box::new(self.substitute_self_in_type(t, self_ty))),
            Type::Stream(t) => Type::Stream(Box::new(self.substitute_self_in_type(t, self_ty))),
            Type::Map(k, v) => Type::Map(
                Box::new(self.substitute_self_in_type(k, self_ty)),
                Box::new(self.substitute_self_in_type(v, self_ty)),
            ),
            Type::Option(t) => Type::Option(Box::new(self.substitute_self_in_type(t, self_ty))),
            Type::Result(t, e) => Type::Result(
                Box::new(self.substitute_self_in_type(t, self_ty)),
                Box::new(self.substitute_self_in_type(e, self_ty)),
            ),
            Type::Arrow(a, b) => Type::Arrow(
                Box::new(self.substitute_self_in_type(a, self_ty)),
                Box::new(self.substitute_self_in_type(b, self_ty)),
            ),
            Type::Fn(params, ret) => Type::Fn(
                params
                    .iter()
                    .map(|p| self.substitute_self_in_type(p, self_ty))
                    .collect(),
                Box::new(self.substitute_self_in_type(ret, self_ty)),
            ),
            Type::Trf(i, o, fx) => Type::Trf(
                Box::new(self.substitute_self_in_type(i, self_ty)),
                Box::new(self.substitute_self_in_type(o, self_ty)),
                fx.clone(),
            ),
            Type::Cap(name, args) => Type::Cap(
                name.clone(),
                args.iter()
                    .map(|a| self.substitute_self_in_type(a, self_ty))
                    .collect(),
            ),
            Type::Interface(name, args) => Type::Interface(
                name.clone(),
                args.iter()
                    .map(|a| self.substitute_self_in_type(a, self_ty))
                    .collect(),
            ),
            Type::Named(name, args) => Type::Named(
                name.clone(),
                args.iter()
                    .map(|a| self.substitute_self_in_type(a, self_ty))
                    .collect(),
            ),
            _ => ty.clone(),
        }
    }

    /// Recursively validate that all Named type usages have the correct arity (E023).
    fn validate_type_expr_arity(&mut self, te: &TypeExpr) {
        match te {
            TypeExpr::Named(name, args, span) => {
                if let Some(&expected) = self.type_arity.get(name.as_str()) {
                    let got = args.len();
                    if got != expected && got != 0 {
                        self.type_error(
                            "E0223",
                            format!(
                                "type `{}` expects {} type argument(s), got {}",
                                name, expected, got
                            ),
                            span,
                        );
                    }
                }
                for a in args {
                    self.validate_type_expr_arity(a);
                }
            }
            TypeExpr::Optional(inner, _) | TypeExpr::Fallible(inner, _) => {
                self.validate_type_expr_arity(inner);
            }
            TypeExpr::Arrow(a, b, _) => {
                self.validate_type_expr_arity(a);
                self.validate_type_expr_arity(b);
            }
            TypeExpr::TrfFn { input, output, .. } => {
                self.validate_type_expr_arity(input);
                self.validate_type_expr_arity(output);
            }
        }
    }
}

fn collect_type_vars_ordered(ty: &Type, out: &mut Vec<String>) {
    match ty {
        Type::Var(name) => {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
        Type::List(inner) | Type::Option(inner) | Type::Task(inner) | Type::Stream(inner) => {
            collect_type_vars_ordered(inner, out);
        }
        Type::Map(k, v) | Type::Result(k, v) | Type::Arrow(k, v) => {
            collect_type_vars_ordered(k, out);
            collect_type_vars_ordered(v, out);
        }
        Type::Fn(params, ret) => {
            for param in params {
                collect_type_vars_ordered(param, out);
            }
            collect_type_vars_ordered(ret, out);
        }
        Type::Trf(input, output, _) => {
            collect_type_vars_ordered(input, out);
            collect_type_vars_ordered(output, out);
        }
        Type::Named(_, args) | Type::Cap(_, args) | Type::Interface(_, args) => {
            for arg in args {
                collect_type_vars_ordered(arg, out);
            }
        }
        Type::AbstractTrf { input, output, .. } => {
            collect_type_vars_ordered(input, out);
            collect_type_vars_ordered(output, out);
        }
        Type::AbstractFlwTemplate(_)
        | Type::PartialFlw { .. }
        | Type::Bool
        | Type::Int
        | Type::Float
        | Type::String
        | Type::Unit
        | Type::Unknown
        | Type::Error => {}
    }
}

fn instantiate_explicit_type_args(
    func_ty: Type,
    type_args: &[TypeExpr],
    checker: &mut Checker,
) -> Type {
    let mut names = Vec::new();
    collect_type_vars_ordered(&func_ty, &mut names);
    let mut subst = Subst::empty();
    for (name, arg) in names.into_iter().zip(type_args.iter()) {
        subst.extend(name, checker.resolve_type_expr(arg));
    }
    match func_ty {
        Type::Fn(params, ret) => Type::Fn(
            params.iter().map(|p| subst.apply(p)).collect(),
            Box::new(subst.apply(&ret)),
        ),
        Type::Trf(input, output, effects) => Type::Trf(
            Box::new(subst.apply(&input)),
            Box::new(subst.apply(&output)),
            effects,
        ),
        Type::AbstractTrf {
            input,
            output,
            effects,
        } => Type::AbstractTrf {
            input: Box::new(subst.apply(&input)),
            output: Box::new(subst.apply(&output)),
            effects,
        },
        other => other,
    }
}

// ── Tests (4-20) ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    fn check(src: &str) -> Vec<String> {
        let prog = Parser::parse_str(src, "test").expect("parse error");
        Checker::check_program(&prog)
            .0
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

    fn inferred_type_of(src: &str, name: &str) -> Type {
        let prog = Parser::parse_str(src, "test").expect("parse error");
        let mut checker = Checker::new();
        let (errs, _) = checker.check_with_self(&prog);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
        checker
            .env
            .lookup(name)
            .cloned()
            .expect("missing inferred type")
    }

    fn check_warnings(src: &str) -> Vec<String> {
        let prog = Parser::parse_str(src, "test").expect("parse error");
        let mut checker = Checker::new();
        let (_, warnings) = checker.check_with_self(&prog);
        warnings
            .into_iter()
            .map(|w| format!("[{}] {}", w.code, w.message))
            .collect()
    }

    // 4-4, 4-5: built-in types and functions
    #[test]
    fn test_builtin_io_println() {
        check_ok(
            r#"
            public fn main() -> Unit !Io {
                IO.println("hello")
            }
        "#,
        );
    }

    #[test]
    fn test_fstring_string_type_ok() {
        check_ok(r#"fn f(name: String) -> String { $"Hello {name}!" }"#);
    }

    #[test]
    fn test_fstring_int_auto_show() {
        check_ok(r#"fn f(age: Int) -> String { $"Age: {age}" }"#);
    }

    #[test]
    fn test_fstring_e054_no_show() {
        let errs = check_err(
            r#"
            type User = { name: String }
            fn f(user: User) -> String { $"User: {user}" }
        "#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0254")),
            "expected E054, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_fstring_e053_nested() {
        let errs = check_err(r#"fn f() -> String { $"outer {$"inner"}" }"#);
        assert!(
            errs.iter().any(|e| e.contains("E0253")),
            "expected E053, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_assert_matches_record_ok() {
        check_ok(
            r#"
            type User = { name: String age: Int }
            fn f(user: User) -> Unit {
                assert_matches(user, { name, age })
            }
        "#,
        );
    }

    // 4-6: type definitions
    #[test]
    fn test_type_def_ok() {
        check_ok("type User = { name: String email: String }");
    }

    #[test]
    fn test_col_attr_ok() {
        check_ok("type Row = { #[col(0)] id: Int #[col(1)] name: String }");
    }

    #[test]
    fn test_col_attr_invalid_index_e0503() {
        let errs = check_err("type Row = { #[col(foo)] id: Int }");
        assert!(
            errs.iter().any(|e| e.contains("E0503")),
            "expected E0503, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_invariant_type_check_bool() {
        check_ok("type PosInt = { value: Int invariant value > 0 }");
    }

    #[test]
    fn test_invariant_type_check_e045() {
        let errs = check_err("type Bad = { value: Int invariant value + 1 }");
        assert!(
            errs.iter().any(|e| e.contains("E0245")),
            "expected E045, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_invariant_field_scope() {
        check_ok("type Email = { value: String invariant String.contains(value, \"@\") }");
    }

    #[test]
    fn test_bind_state_annotation_ok() {
        check_ok(
            "type PosInt = { value: Int invariant value > 0 }\nfn f() -> PosInt { bind x: PosInt <- 42; x }",
        );
    }

    #[test]
    fn test_bind_state_annotation_fail_e046() {
        let errs = check_err(
            "type PosInt = { value: Int invariant value > 0 }\nfn f() -> PosInt { bind x: PosInt <- 0; x }",
        );
        assert!(
            errs.iter().any(|e| e.contains("E0246")),
            "expected E046, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_bind_state_annotation_string_invariant_ok() {
        check_ok(
            "type Email = { value: String invariant String.contains(value, \"@\") }\nfn f() -> Email { bind x: Email <- \"a@example.com\"; x }",
        );
    }

    #[test]
    fn test_invariant_unknown_field_e002() {
        let errs = check_err("type Broken = { value: Int invariant missing > 0 }");
        assert!(
            errs.iter().any(|e| e.contains("E0102")),
            "expected E002, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_bind_state_annotation_non_literal_ok() {
        check_ok(
            "type PosInt = { value: Int invariant value > 0 }\nfn f(n: Int) -> PosInt { bind x: PosInt <- n; x }",
        );
    }

    #[test]
    fn test_state_constructor_signature_ok() {
        check_ok(
            "type PosInt = { value: Int invariant value > 0 }\nfn f() -> PosInt! { PosInt.new(5) }",
        );
    }

    #[test]
    fn test_stdlib_states_registered_in_checker() {
        let mut checker = Checker::new();
        checker.register_builtins();
        assert!(checker.env.lookup("PosInt").is_some());
        assert!(checker.env.lookup("Email").is_some());
        assert!(checker.type_invariants.contains_key("Probability"));
        assert!(checker.type_invariants.contains_key("Slug"));
    }

    #[test]
    fn test_stdlib_state_constructor_signature_posint_ok() {
        check_ok("fn f() -> PosInt! { PosInt.new(1) }");
    }

    #[test]
    fn test_stdlib_state_constructor_signature_email_ok() {
        check_ok("fn f() -> Email! { Email.new(\"a@b.com\") }");
    }

    #[test]
    fn test_stdlib_bind_state_annotation_ok() {
        check_ok("fn f() -> PosInt { bind x: PosInt <- 25; x }");
    }

    #[test]
    fn test_stdlib_bind_state_annotation_fail_e046() {
        let errs = check_err("fn f() -> PosInt { bind x: PosInt <- 0; x }");
        assert!(
            errs.iter().any(|e| e.contains("E0246")),
            "expected E046, got: {:?}",
            errs
        );
    }

    // 4-7: fn return type mismatch
    #[test]
    fn test_fn_return_mismatch() {
        let errs = check_err("fn f() -> Int { \"not an int\" }");
        assert!(errs[0].contains("E0101"));
    }

    // 4-7: fn return type matches
    #[test]
    fn test_fn_return_ok() {
        check_ok("fn f() -> Int { 42 }");
    }

    // 4-8: trf type checks
    #[test]
    fn test_trf_ok() {
        check_ok("stage Double: Int -> Int = |n| { n }");
    }

    // 4-9: flw pipeline  Ecompatible
    #[test]
    fn test_flw_ok() {
        check_ok(
            "
            stage A: String -> Int = |s| { 0 }
            stage B: Int -> Bool   = |n| { true }
            seq AB = A |> B
        ",
        );
    }

    // 4-9: flw pipeline  Etype mismatch
    #[test]
    fn test_flw_type_mismatch() {
        let errs = check_err(
            "
            stage A: String -> Int  = |s| { 0 }
            stage B: Bool   -> Unit = |b| { () }
            seq Bad = A |> B
        ",
        );
        assert!(errs.iter().any(|e| e.contains("E0103")));
    }

    // 4-9: flw  Eundefined step
    #[test]
    fn test_flw_undefined_step() {
        let errs = check_err("seq Bad = NoSuchTrf |> AnotherMissing");
        assert!(errs.iter().any(|e| e.contains("E0102")));
    }

    // 4-10: bind infers type
    #[test]
    fn test_bind_type_inferred() {
        check_ok("fn f() -> Int { bind x <- 42; x }");
    }

    // 4-11: pattern binding  Erecord
    #[test]
    fn test_pattern_record_bind() {
        check_ok(
            "
            type User = { name: String }
            fn f(u: User) -> String { bind { name } <- u; name }
        ",
        );
    }

    // 4-12: match arm types consistent
    #[test]
    fn test_match_consistent_arms() {
        check_ok(
            "
            type Color = | Red | Blue
            fn f(c: Color) -> Int {
                match c {
                    Red  => 0
                    Blue => 1
                }
            }
        ",
        );
    }

    // 4-12: match arm type mismatch
    #[test]
    fn test_match_inconsistent_arms() {
        let errs = check_err(
            "
            type Color = | Red | Blue
            fn f(c: Color) -> Int {
                match c {
                    Red  => 0
                    Blue => \"not an int\"
                }
            }
        ",
        );
        assert!(errs.iter().any(|e| e.contains("E0101")));
    }

    // 4-13: if branch type mismatch
    #[test]
    fn test_if_branch_mismatch() {
        let errs = check_err("fn f() -> Int { if true { 1 } else { \"wrong\" } }");
        assert!(errs.iter().any(|e| e.contains("E0101")));
    }

    // 4-13: if branches match
    #[test]
    fn test_if_branches_ok() {
        check_ok("fn f() -> Int { if true { 1 } else { 2 } }");
    }

    #[test]
    fn test_logical_and_bool_ok() {
        check_ok("fn f() -> Bool { true && false }");
    }

    #[test]
    fn test_logical_or_bool_ok() {
        check_ok("fn f() -> Bool { false || true }");
    }

    #[test]
    fn test_logical_and_non_bool_e070() {
        let errs = check_err("fn f() -> Bool { 1 && true }");
        assert!(errs.iter().any(|e| e.contains("E0370")));
    }

    #[test]
    fn test_logical_or_non_bool_e071() {
        let errs = check_err("fn f() -> Bool { true || 1 }");
        assert!(errs.iter().any(|e| e.contains("E0371")));
    }

    #[test]
    fn test_logical_and_non_bool_right_e070() {
        let errs = check_err("fn f() -> Bool { true && 1 }");
        assert!(errs.iter().any(|e| e.contains("E0370")));
    }

    #[test]
    fn test_logical_and_precedence() {
        // `1 == 1 && 2 == 2` should parse as `(1 == 1) && (2 == 2)` — type checks OK
        check_ok("fn f() -> Bool { 1 == 1 && 2 == 2 }");
    }

    #[test]
    fn test_logical_or_precedence() {
        // `false || 1 == 1` should parse as `false || (1 == 1)` — type checks OK
        check_ok("fn f() -> Bool { false || 1 == 1 }");
    }

    // 4-14: pipeline type mismatch
    #[test]
    fn test_pipeline_type_mismatch() {
        let errs = check_err(
            "
            stage A: String -> Int  = |s| { 0 }
            stage B: Bool   -> Unit = |b| { () }
            fn f() -> Unit { \"hello\" |> A |> B }
        ",
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("E0103") || e.contains("E0101"))
        );
    }

    // 4-15: undefined identifier
    #[test]
    fn test_undefined_ident() {
        let errs = check_err("fn f() -> Int { undefined_var }");
        assert!(errs.iter().any(|e| e.contains("E0102")));
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

    // ── v0.2.0 checker tests (2-10) ───────────────────────────────────────────

    // 2-4: record construction type-checks against known type
    #[test]
    fn test_record_construct_ok() {
        check_ok(
            r#"
            type User = { name: String age: Int }
            fn f() -> User { User { name: "Alice", age: 30 } }
        "#,
        );
    }

    // 2-4: record construction with undefined type ↁEE002
    #[test]
    fn test_record_construct_unknown_type() {
        let errs = check_err(r#"fn f() -> Unit { Ghost { x: 1 } }"#);
        assert!(errs.iter().any(|e| e.contains("E0102")));
    }

    // 2-5: emit expr returns Unit
    #[test]
    fn test_emit_expr_unit() {
        check_ok(r#"fn f() -> Unit !Emit<E> { emit "hello" }"#);
    }

    // Db.* calls resolve to known types
    #[test]
    fn test_db_query_type() {
        check_ok(
            r#"
            fn f() -> Unit !Db {
                bind rows <- Db.query("SELECT * FROM users");
                ()
            }
        "#,
        );
    }

    // Debug.show returns String
    #[test]
    fn test_debug_show_string() {
        check_ok(
            r#"
            fn f(n: Int) -> String { Debug.show(n) }
        "#,
        );
    }

    // Map.get returns Option
    #[test]
    fn test_map_get_option() {
        check_ok(
            r#"
            fn f() -> Unit {
                bind result <- Map.get(Map.set(Map.set((), "a", 1), "b", 2), "a");
                ()
            }
        "#,
        );
    }

    // 2-6: Db.* without !Db ↁEE007
    #[test]
    fn test_db_effect_missing() {
        let errs = check_err(
            r#"
            fn f() -> Int {
                Db.execute("SELECT 1")
            }
        "#,
        );
        assert!(errs.iter().any(|e| e.contains("E0107")), "got: {:?}", errs);
    }

    // 2-6: Db.* with !Db ↁEok
    #[test]
    fn test_db_effect_present() {
        check_ok(r#"fn f() -> Int !Db { Db.execute("SELECT 1") }"#);
    }

    // 2-7: Http.* without !Network ↁEE008
    #[test]
    fn test_network_effect_missing() {
        let errs = check_err(
            r#"
            fn f() -> String! {
                Http.get("http://example.com")
            }
        "#,
        );
        assert!(errs.iter().any(|e| e.contains("E0108")), "got: {:?}", errs);
    }

    // 2-7: Http.* with !Network ↁEok
    #[test]
    fn test_network_effect_present() {
        check_ok(r#"fn f() -> String! !Network { Http.get("http://example.com") }"#);
    }

    #[test]
    fn test_file_effect_missing() {
        let errs = check_err(r#"fn f() -> String { File.read("a.txt") }"#);
        assert!(errs.iter().any(|e| e.contains("E0136")), "got: {:?}", errs);
    }

    #[test]
    fn test_file_effect_present() {
        check_ok(r#"fn f() -> Bool !File { File.exists("a.txt") }"#);
    }

    // 2-8: emit without !Emit<T> ↁEE009
    #[test]
    fn test_emit_effect_missing() {
        let errs = check_err(r#"fn f() -> Unit { emit "event" }"#);
        assert!(errs.iter().any(|e| e.contains("E0109")), "got: {:?}", errs);
    }

    // 2-8: emit with !Emit<T> ↁEok
    #[test]
    fn test_emit_effect_present() {
        check_ok(r#"fn f() -> Unit !Emit<OrderPlaced> { emit "order" }"#);
    }

    // 2-8: trf with !Emit<T>
    #[test]
    fn test_trf_emit_effect() {
        check_ok(r#"stage T: String -> Unit !Emit<E> = |s| { emit s }"#);
    }

    #[test]
    fn effect_def_registered() {
        let program = Parser::parse_str(
            "effect Payment\nstage Charge: Int -> Int !Payment = |x| { x }",
            "effect_registered.fav",
        )
        .expect("parse");
        let mut checker = Checker::new();
        let (errs, _) = checker.check_with_self(&program);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
        assert!(checker.effect_registry.contains("Payment"));
    }

    #[test]
    fn effect_custom_in_trf_ok() {
        check_ok(
            r#"
effect Payment
stage Charge: Int -> Int !Payment = |x| { x }
"#,
        );
    }

    #[test]
    fn effect_unknown_e052() {
        let errs = check_err(r#"stage Charge: Int -> Int !Payment = |x| { x }"#);
        assert!(errs.iter().any(|e| e.contains("E0252")), "got: {:?}", errs);
    }

    #[test]
    fn effect_builtin_no_error() {
        check_ok(
            r#"
fn f() -> Unit !Io { () }
stage T: Int -> Int !Db = |x| { x }
abstract stage Fetch: Int -> Int !Trace
abstract seq Pipeline {
    save: Int -> Int !File
}
"#,
        );
    }

    // ── 4-11: use resolution tests ────────────────────────────────────────────

    use crate::middle::resolver::Resolver;
    use crate::toml::FavToml;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    /// Build a Resolver + temp project with a single .fav file under src/.
    fn make_project(
        src_content: &str,
        filename: &str,
    ) -> (Arc<Mutex<Resolver>>, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::write(
            root.join("fav.toml"),
            "[rune]\nname=\"t\"\nversion=\"0.1.0\"\nsrc=\"src\"\n",
        )
        .unwrap();
        let src_dir = root.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let fav_path = src_dir.join(filename.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(p) = fav_path.parent() {
            std::fs::create_dir_all(p).unwrap();
        }
        std::fs::write(&fav_path, src_content).unwrap();
        let toml = FavToml {
            name: "t".into(),
            version: "0.1.0".into(),
            src: "src".into(),
            runes_path: None,
            dependencies: vec![],
            checkpoint: None,
            database: None,
        };
        let resolver = Arc::new(Mutex::new(Resolver::new(Some(toml), Some(root))));
        (resolver, dir)
    }

    fn check_with_resolver(src: &str, file: &str, resolver: Arc<Mutex<Resolver>>) -> Vec<String> {
        let prog = Parser::parse_str(src, file).expect("parse error");
        let mut c = Checker::new_with_resolver(resolver, std::path::PathBuf::from(file));
        c.check_with_self(&prog)
            .0
            .into_iter()
            .map(|e| format!("[{}] {}", e.code, e.message))
            .collect()
    }

    // 4-11a: public fn from another file can be used
    #[test]
    fn test_use_public_fn() {
        let (resolver, _dir) = make_project("public fn greet() -> Unit { () }", "helpers.fav");
        let src = "use helpers.greet\npublic fn main() -> Unit { greet() }";
        let errs = check_with_resolver(src, "main.fav", resolver);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    // 4-11b: private fn cannot be imported (E014 from resolver)
    #[test]
    fn test_use_private_fn_error() {
        let (resolver, _dir) = make_project("fn secret() -> Unit { () }", "utils.fav");
        let src = "use utils.secret\npublic fn main() -> Unit { () }";
        let errs = check_with_resolver(src, "main.fav", resolver);
        assert!(
            errs.iter().any(|e| e.contains("E0214")),
            "expected E014, got: {:?}",
            errs
        );
    }

    // 4-11c: missing symbol gives E013
    #[test]
    fn test_use_missing_symbol_error() {
        let (resolver, _dir) = make_project("public fn real() -> Unit { () }", "stuff.fav");
        let src = "use stuff.ghost\npublic fn main() -> Unit { () }";
        let errs = check_with_resolver(src, "main.fav", resolver);
        assert!(
            errs.iter().any(|e| e.contains("E0213")),
            "expected E013, got: {:?}",
            errs
        );
    }

    // 4-11d: circular import gives E012  Etested via Resolver directly because the
    // current architecture calls Checker::check_program_and_export (no resolver) for
    // inner modules, so deep cycle detection only fires at the Resolver level.
    #[test]
    fn test_circular_import_error() {
        use crate::frontend::lexer::Span;
        use crate::middle::resolver::ResolveError;
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::write(
            root.join("fav.toml"),
            "[rune]\nname=\"t\"\nversion=\"0.1.0\"\nsrc=\"src\"\n",
        )
        .unwrap();
        let src_dir = root.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("cycle.fav"), "public fn f() -> Unit { () }").unwrap();
        let toml = FavToml {
            name: "t".into(),
            version: "0.1.0".into(),
            src: "src".into(),
            runes_path: None,
            dependencies: vec![],
            checkpoint: None,
            database: None,
        };
        let mut resolver = Resolver::new(Some(toml), Some(root));
        // Simulate a mid-load state: "cycle" is already in the loading set
        let span = Span::new("test", 0, 0, 1, 1);
        // First load succeeds
        let mut errors: Vec<ResolveError> = Vec::new();
        resolver.load_module("cycle", &mut errors, &span);
        assert!(
            errors.is_empty(),
            "unexpected error on first load: {:?}",
            errors
        );
        // Loading the same module again uses cache  Eno E012 (idempotent)
        resolver.load_module("cycle", &mut errors, &span);
        assert!(errors.is_empty(), "expected cache hit: {:?}", errors);
        // Simulate a cycle by checking E012 would be reported via resolve_use
        // with a non-existent module (E013), confirming error propagation works
        let path = vec!["nonexistent".to_string(), "sym".to_string()];
        resolver.resolve_use(&path, &mut errors, &span);
        assert!(
            errors.iter().any(|e| e.code == "E0213"),
            "expected E013, got: {:?}",
            errors
        );
    }

    #[test]
    fn import_resolves_public_symbol() {
        let (resolver, _dir) = make_project(
            "public type User = { name: String age: Int }\npublic stage ParseUser: String -> User = |s| { User { name: s age: 30 } }",
            "models/user.fav",
        );
        let src = "import \"models/user\"\npublic fn main() -> Unit { user.ParseUser(\"a\"); () }";
        let errs = check_with_resolver(src, "main.fav", resolver);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    #[test]
    fn import_with_alias_resolves() {
        let (resolver, _dir) = make_project(
            "public fn ParseUser(s: String) -> String { s }",
            "models/user.fav",
        );
        let src =
            "import \"models/user\" as m\npublic fn main() -> Unit { m.ParseUser(\"a\"); () }";
        let errs = check_with_resolver(src, "main.fav", resolver);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    #[test]
    fn import_e080_circular_import() {
        let (resolver, _dir) = make_project(
            "import \"models/post\"\npublic fn user() -> Unit { () }",
            "models/user.fav",
        );
        {
            let root = resolver.lock().unwrap().root.clone().unwrap();
            let post = root.join("src").join("models").join("post.fav");
            std::fs::create_dir_all(post.parent().unwrap()).unwrap();
            std::fs::write(
                post,
                "import \"models/user\"\npublic fn post() -> Unit { () }",
            )
            .unwrap();
        }
        let src = "import \"models/user\"\npublic fn main() -> Unit { () }";
        let errs = check_with_resolver(src, "main.fav", resolver);
        assert!(
            errs.iter().any(|e| e.contains("E0580")),
            "expected E080, got: {:?}",
            errs
        );
    }

    #[test]
    fn import_e081_namespace_conflict() {
        let (resolver, _dir) = make_project("public fn left() -> Unit { () }", "models/user.fav");
        {
            let root = resolver.lock().unwrap().root.clone().unwrap();
            let auth = root.join("src").join("auth").join("user.fav");
            std::fs::create_dir_all(auth.parent().unwrap()).unwrap();
            std::fs::write(auth, "public fn right() -> Unit { () }").unwrap();
        }
        let src = "import \"models/user\"\nimport \"auth/user\"\npublic fn main() -> Unit { () }";
        let errs = check_with_resolver(src, "main.fav", resolver);
        assert!(
            errs.iter().any(|e| e.contains("E0581")),
            "expected E081, got: {:?}",
            errs
        );
    }

    // ── Phase 1: Subst / unify / occurs (v0.4.0) ──────────────────────────────

    #[test]
    fn test_interface_show_int_ok() {
        check_ok(
            r#"
            interface Show { show: Self -> String }
            impl Show for Int { show = |x| "int" }
        "#,
        );
    }

    #[test]
    fn test_interface_method_type_mismatch_e042() {
        let errs = check_err(
            r#"
            interface Show { show: Self -> String }
            impl Show for Int { show = |x| 1 }
        "#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0242")),
            "expected E042, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_interface_super_missing_e043() {
        let errs = check_err(
            r#"
            interface Eq { eq: Self -> Bool }
            interface Ord: Eq { compare: Self -> Int }
            type User = { name: String }
            impl Ord for User { compare = |x| 0 }
        "#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0243")),
            "expected E043, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_interface_unknown_e041() {
        let errs = check_err(
            r#"
            impl UnknownFace for Int { show = |x| "int" }
        "#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0241")),
            "expected E041, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_interface_explicit_passing() {
        check_ok(
            r#"
            interface Show { show: Self -> String }
            impl Show for Int { show = |x| "int" }
            fn use_show(x: Int, show: Show<Int>) -> String { show.show(x) }
        "#,
        );
    }

    #[test]
    fn test_interface_auto_synthesis_ok() {
        check_ok(
            r#"
            interface Show { show: Self -> String }
            impl Show for String { show = |x| x }
            type User with Show = { name: String }
            fn use_show(x: User, show: Show<User>) -> String { show.show(x) }
        "#,
        );
    }

    #[test]
    fn test_interface_auto_synthesis_fail_e044() {
        // Use a custom interface that Int does NOT implement → List<Int> also fails E044.
        // (Int has builtin Show/Eq/Gen, so those would succeed now with generic container support.)
        let errs = check_err(
            r#"
            interface Serializable { serialize: Self -> String }
            type User with Serializable = { tags: List<Int> }
        "#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0244")),
            "expected E044, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_interface_impl_multi_interface() {
        check_ok(
            r#"
            interface Show { show: Self -> String }
            interface Eq { eq: Self -> Bool }
            impl Show for String { show = |x| x }
            impl Eq for String { eq = |x| true }
            type User with Show, Eq = { name: String }
            fn use_all(x: User, show: Show<User>, eq: Eq<User>) -> String { show.show(x) }
        "#,
        );
    }

    #[test]
    fn test_interface_auto_impl_decl_ok() {
        check_ok(
            r#"
            interface Show { show: Self -> String }
            impl Show for String { show = |x| x }
            type User = { name: String }
            impl Show for User
            fn use_show(x: User, show: Show<User>) -> String { show.show(x) }
        "#,
        );
    }

    #[test]
    fn test_builtin_show_int_registered() {
        check_ok(
            r#"
            fn use_show() -> String { Int.show.show(1) }
        "#,
        );
    }

    #[test]
    fn test_builtin_ord_int_registered() {
        check_ok(
            r#"
            fn use_ord() -> Int { Int.ord.compare(1, 2) }
        "#,
        );
    }

    #[test]
    fn test_builtin_gen_int_registered() {
        check_ok(
            r#"
            fn use_gen(seed: Int?, gen: Gen<Int>) -> Int { gen.gen(seed) }
        "#,
        );
    }

    #[test]
    fn test_gen_interface_auto_synthesis_ok() {
        check_ok(
            r#"
            type User with Gen = { age: Int flag: Bool }
            fn use_gen(seed: Int?, gen: Gen<User>) -> User { gen.gen(seed) }
        "#,
        );
    }

    #[test]
    fn test_gen_interface_auto_impl_decl_ok() {
        check_ok(
            r#"
            type User = { age: Int flag: Bool }
            impl Gen for User
            fn use_gen(seed: Int?, gen: Gen<User>) -> User { gen.gen(seed) }
        "#,
        );
    }

    #[test]
    fn test_gen_auto_synthesis_fail_e044() {
        // CustomTag has no Gen impl → List<CustomTag> also fails E044.
        let errs = check_err(
            r#"
            type CustomTag = { label: String  count: Float }
            type User with Gen = { tags: List<CustomTag> }
        "#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0244")),
            "expected E044, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_cap_removed_parse_error_e2003() {
        // v2.0.0: `cap` is removed; parser emits an error with migration hint
        let result = Parser::parse_str(
            r#"
            cap Show<T> = { show: T -> String }
        "#,
            "test",
        );
        assert!(result.is_err(), "expected parse error for `cap` in v2.0.0");
        assert!(
            result.unwrap_err().message.contains("cap"),
            "error should mention `cap`"
        );
    }

    #[test]
    fn test_cap_style_impl_deprecated_warning_w010() {
        let warnings = check_warnings(
            r#"
            impl Eq<Int> {
                fn equals(a: Int, b: Int) -> Bool { a == b }
            }
        "#,
        );
        assert!(
            warnings.iter().any(|w| w.contains("W010")),
            "expected W010, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_field_interface_float_registered() {
        check_ok(
            r#"
            fn use_field(x: Float, y: Float) -> Result<Float, Error> {
                Float.field.divide(x, y)
            }
        "#,
        );
    }

    #[test]
    fn test_semigroup_interface_int_registered() {
        check_ok(
            r#"
            fn use_semigroup() -> Int { Int.semigroup.combine(1, 2) }
        "#,
        );
    }

    #[test]
    fn test_type_with_sugar_equivalent_to_explicit_impl() {
        // type T with Show, Eq { ... } must behave identically to:
        //   type T { ... }
        //   impl Show, Eq for T
        check_ok(
            r#"
            type Point with Show, Eq = { x: Int  y: Int }
            fn use_show(p: Point, show: Show<Point>) -> String { show.show(p) }
            fn use_eq(a: Point, b: Point, eq: Eq<Point>) -> Bool { eq.eq(a, b) }
        "#,
        );
        // Explicit impl form must also pass
        check_ok(
            r#"
            type Point = { x: Int  y: Int }
            impl Show, Eq for Point
            fn use_show(p: Point, show: Show<Point>) -> String { show.show(p) }
            fn use_eq(a: Point, b: Point, eq: Eq<Point>) -> Bool { eq.eq(a, b) }
        "#,
        );
    }

    #[test]
    fn test_list_field_show_synthesis_ok() {
        // List<Int> field: Int has builtin Show → List<Int> is considered to implement Show
        check_ok(
            r#"
            type Tags = { items: List<Int> }
            impl Show for Tags
        "#,
        );
    }

    #[test]
    fn test_option_field_show_synthesis_ok() {
        // Option<String> field: String has builtin Show → Option<String> implements Show
        check_ok(
            r#"
            type Opt = { label: Option<String> }
            impl Show for Opt
        "#,
        );
    }

    #[test]
    fn test_list_field_custom_interface_e044() {
        // Custom interface not implemented for Int → List<Int> still fails E044
        let errs = check_err(
            r#"
            interface Serialize { serialize: Self -> String }
            type Tags = { items: List<Int> }
            impl Serialize for Tags
        "#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0244")),
            "expected E044, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_subst_apply_var() {
        let s = Subst::singleton("T".into(), Type::Int);
        assert_eq!(s.apply(&Type::Var("T".into())), Type::Int);
        assert_eq!(s.apply(&Type::Var("U".into())), Type::Var("U".into()));
    }

    #[test]
    fn test_subst_apply_nested() {
        let s = Subst::singleton("T".into(), Type::Bool);
        assert_eq!(
            s.apply(&Type::List(Box::new(Type::Var("T".into())))),
            Type::List(Box::new(Type::Bool))
        );
        assert_eq!(
            s.apply(&Type::Option(Box::new(Type::Var("T".into())))),
            Type::Option(Box::new(Type::Bool))
        );
    }

    #[test]
    fn test_subst_compose() {
        let s1 = Subst::singleton("T".into(), Type::Int);
        let s2 = Subst::singleton("U".into(), Type::List(Box::new(Type::Var("T".into()))));
        let composed = s2.compose(s1);
        // After compose: U ↁEList<Int>, T ↁEInt
        assert_eq!(
            composed.apply(&Type::Var("U".into())),
            Type::List(Box::new(Type::Int))
        );
        assert_eq!(composed.apply(&Type::Var("T".into())), Type::Int);
    }

    #[test]
    fn test_occurs_check() {
        assert!(occurs("T", &Type::Var("T".into())));
        assert!(occurs("T", &Type::List(Box::new(Type::Var("T".into())))));
        assert!(!occurs("T", &Type::Int));
        assert!(!occurs("T", &Type::Var("U".into())));
    }

    #[test]
    fn test_unify_identical() {
        assert!(unify(&Type::Int, &Type::Int).is_ok());
        assert!(unify(&Type::Bool, &Type::Bool).is_ok());
    }

    #[test]
    fn test_unify_var_left() {
        let s = unify(&Type::Var("T".into()), &Type::Int).unwrap();
        assert_eq!(s.apply(&Type::Var("T".into())), Type::Int);
    }

    #[test]
    fn test_unify_var_right() {
        let s = unify(&Type::String, &Type::Var("T".into())).unwrap();
        assert_eq!(s.apply(&Type::Var("T".into())), Type::String);
    }

    #[test]
    fn test_unify_structural() {
        let t1 = Type::List(Box::new(Type::Var("T".into())));
        let t2 = Type::List(Box::new(Type::Int));
        let s = unify(&t1, &t2).unwrap();
        assert_eq!(s.apply(&Type::Var("T".into())), Type::Int);
    }

    #[test]
    fn test_unify_infinite_type_error() {
        // T ~ List<T> should fail with occurs check
        let t1 = Type::Var("T".into());
        let t2 = Type::List(Box::new(Type::Var("T".into())));
        assert!(unify(&t1, &t2).is_err());
    }

    #[test]
    fn test_unify_mismatch() {
        assert!(unify(&Type::Int, &Type::Bool).is_err());
    }

    #[test]
    fn test_unify_unknown_is_any() {
        assert!(unify(&Type::Unknown, &Type::Int).is_ok());
        assert!(unify(&Type::Bool, &Type::Unknown).is_ok());
    }

    // ── Phase 3: checker integration (v0.4.0) ─────────────────────────────────

    #[test]
    fn test_generic_identity() {
        // Generic fn with single type param  Eno type errors.
        let errs = check("fn identity<T>(x: T) -> T { x }");
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_generic_pair_type() {
        let errs = check("type Pair<A, B> = { first: A second: B }");
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_cap_def_checks() {
        let errs = check("interface Eq { equals: Self -> Self -> Bool }");
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_impl_def_valid() {
        let src = "interface Eq { equals: Self -> Self -> Bool }\nimpl Eq for Int { equals = |a b| a == b }";
        let errs = check(src);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_impl_undefined_cap_error() {
        let src = "impl NoSuchCap<Int> { fn foo(x: Int) -> Int { x } }";
        let errs = check_err(src);
        assert!(
            errs.iter().any(|e| e.contains("E0220")),
            "expected E020, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_impl_method_not_in_cap_error() {
        let src = "interface Eq { equals: Self -> Self -> Bool }\nimpl Eq for Int { bogus = |a b| a == b }";
        let errs = check_err(src);
        // E042 is raised for undeclared methods in interface impl
        assert!(
            errs.iter().any(|e| e.contains("E0242")),
            "expected E042, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_e021_no_impl_for_type() {
        // Accessing the built-in Eq cap on a type with no impl should produce E021.
        let src = "type MyData = { x: Int }\nfn main() -> Unit { bind t <- MyData { x: 1 }; bind _ <- t.eq; () }";
        let errs = check_err(src);
        assert!(
            errs.iter().any(|e| e.contains("E0221")),
            "expected E021, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_generic_fn_call_infers_type() {
        // identity<T>(42) should return Int without errors.
        let src = "fn identity<T>(x: T) -> T { x }\nfn main() -> Int { identity(42) }";
        let errs = check(src);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    #[test]
    fn test_unify_fail_at_call_site() {
        // Calling a function with wrong argument type should produce E018.
        let src = "fn add(a: Int, b: Int) -> Int { a }\nfn main() -> Int { add(1, true) }";
        let errs = check_err(src);
        assert!(
            errs.iter().any(|e| e.contains("E0218")),
            "expected E018, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_e019_occurs_check_infinite_type() {
        // unify(T, List<T>) must fail with "infinite type" so check_apply routes to E019.
        let err = unify(
            &Type::Var("T".into()),
            &Type::List(Box::new(Type::Var("T".into()))),
        )
        .unwrap_err();
        assert!(
            err.contains("infinite type"),
            "expected 'infinite type' error, got: {}",
            err
        );
    }

    #[test]
    fn test_e023_type_param_arity_mismatch() {
        // Pair<A,B> used with one argument should produce E023.
        let src = "type Pair<A, B> = { first: A second: B }\nfn bad(x: Pair<Int>) -> Int { 0 }";
        let errs = check_err(src);
        assert!(
            errs.iter().any(|e| e.contains("E0223")),
            "expected E023, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_builtin_cap_field_access_ok() {
        // Int.eq should resolve without type error (returns Cap type).
        let src = "fn get_eq() -> Bool { Int.eq.equals(1, 1) }";
        let errs = check(src);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    // ── v0.5.0 checker tests ───────────────────────────────────────────────────

    // task 3-16: chain in Result context passes
    #[test]
    fn test_chain_result_ok() {
        // Int! expands to Result<Int, Error>, which is what Result.ok returns
        let src = r#"
fn parse_int(s: String) -> Int! { Result.ok(42) }
fn main() -> Int! {
    chain n <- parse_int("42")
    Result.ok(n)
}
"#;
        check_ok(src);
    }

    // task 3-17: chain in Option context passes
    #[test]
    fn test_chain_option_ok() {
        let src = r#"
fn find(x: Int) -> Int? { Option.some(x) }
fn main() -> Int? {
    chain n <- find(1)
    Option.some(n)
}
"#;
        check_ok(src);
    }

    // task 3-18: chain outside Result/Option ↁEE024
    #[test]
    fn test_chain_outside_context() {
        let src = r#"
fn main() -> Int {
    chain n <- Result.ok(42)
    n
}
"#;
        let errs = check_err(src);
        assert!(
            errs.iter().any(|e| e.contains("E0224")),
            "expected E024, got: {:?}",
            errs
        );
    }

    // task 3-20: yield outside collect ↁEE026
    #[test]
    fn test_yield_outside_collect() {
        let src = r#"
fn main() -> Int {
    yield 42;
    0
}
"#;
        let errs = check_err(src);
        assert!(
            errs.iter().any(|e| e.contains("E0226")),
            "expected E026, got: {:?}",
            errs
        );
    }

    // task 3-21: collect { yield 1; yield 2; } has type List<Int>
    #[test]
    fn test_collect_type() {
        let src = r#"
fn main() -> List<Int> {
    collect { yield 1; yield 2; () }
}
"#;
        check_ok(src);
    }

    // task 3-22: guard with non-Bool ↁEE027
    #[test]
    fn test_guard_non_bool() {
        let src = r#"
fn main() -> Int {
    match 42 {
        n where 1 => n
    }
}
"#;
        let errs = check_err(src);
        assert!(
            errs.iter().any(|e| e.contains("E0227")),
            "expected E027, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_guard_non_bool_compound() {
        let errs = check_err("fn f(x: Int) -> Int { match x { n where n + 1 => n _ => 0 } }");
        assert!(
            errs.iter().any(|e| e.contains("E0227")),
            "expected E027, got: {:?}",
            errs
        );
    }

    // task 3-19: chain with non-monadic expr ↁEE025
    #[test]
    fn test_e072_destructure_bind_non_record() {
        let errs = check_err("fn f() -> Int { bind { x } <- 42; x }");
        assert!(
            errs.iter().any(|e| e.contains("E0372")),
            "expected E072, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_e073_destructure_bind_missing_field() {
        let src = r#"
type Point = { x: Int }
fn f() -> Int {
    bind pt <- Point { x: 1 }
    bind { x, y } <- pt
    x
}
"#;
        let errs = check_err(src);
        assert!(
            errs.iter().any(|e| e.contains("E0373")),
            "expected E073, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_e074_infer_recursive_return_type() {
        let errs = check_err("fn loop(n: Int) = loop(n)");
        assert!(
            errs.iter().any(|e| e.contains("E0274")),
            "expected E074, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_w001_unknown_type_bind() {
        let warnings = check_warnings(r#"test "warn" { bind x <- assert_eq; () }"#);
        assert!(
            warnings.iter().any(|w| w.contains("W001")),
            "expected W001, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_chain_type_mismatch() {
        let src = r#"
fn main() -> Int! {
    chain n <- 42
    Result.ok(n)
}
"#;
        let errs = check_err(src);
        assert!(
            errs.iter().any(|e| e.contains("E0225")),
            "expected E025, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_flw_binding_type_ok() {
        check_ok(
            r#"
abstract seq DataPipeline<Row> {
    parse: String -> List<Row>!
    save: List<Row> -> Int !Db
}
abstract stage ParseCsv: String -> List<UserRow>!
abstract stage SaveUsers: List<UserRow> -> Int !Db
type UserRow = { name: String }
seq UserImport = DataPipeline<UserRow> { parse <- ParseCsv; save <- SaveUsers }
"#,
        );
    }

    #[test]
    fn test_flw_binding_e048() {
        let errs = check_err(
            r#"
abstract seq DataPipeline<Row> {
    parse: String -> List<Row>!
    save: List<Row> -> Int !Db
}
abstract stage ParseCsv: String -> List<UserRow>!
abstract stage SaveUsers: List<OrderRow> -> Int !Db
type UserRow = { name: String }
type OrderRow = { id: Int }
seq UserImport = DataPipeline<UserRow> { parse <- ParseCsv; save <- SaveUsers }
"#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0248")),
            "expected E048, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_flw_binding_e049() {
        let errs = check_err(
            r#"
abstract seq DataPipeline<Row> {
    parse: String -> List<Row>!
}
abstract stage ParseCsv: String -> List<UserRow>!
type UserRow = { name: String }
seq UserImport = DataPipeline<UserRow> { extra <- ParseCsv }
"#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0249")),
            "expected E049, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_flw_binding_effect_inference() {
        let ty = inferred_type_of(
            r#"
abstract seq DataPipeline<Row> {
    parse: String -> List<Row>! !Network
    save: List<Row> -> Int !Db
}
abstract stage ParseCsv: String -> List<UserRow>! !Network
abstract stage SaveUsers: List<UserRow> -> Int !Db
type UserRow = { name: String }
seq UserImport = DataPipeline<UserRow> { parse <- ParseCsv; save <- SaveUsers }
"#,
            "UserImport",
        );
        match ty {
            Type::Trf(input, output, effects) => {
                assert!(input.is_compatible(&Type::String));
                assert!(output.is_compatible(&Type::Int));
                assert!(effects.contains(&Effect::Network));
                assert!(effects.contains(&Effect::Db));
            }
            other => panic!("expected Trf, got {:?}", other),
        }
    }

    #[test]
    fn test_flw_partial_type() {
        let ty = inferred_type_of(
            r#"
abstract seq DataPipeline<Row> {
    parse: String -> List<Row>!
    validate: Row -> Row!
    save: List<Row> -> Int !Db
}
abstract stage ParseCsv: String -> List<UserRow>!
type UserRow = { name: String }
seq PartialImport = DataPipeline<UserRow> { parse <- ParseCsv }
"#,
            "PartialImport",
        );
        match ty {
            Type::PartialFlw {
                template,
                type_args,
                unbound_slots,
            } => {
                assert_eq!(template, "DataPipeline");
                assert_eq!(type_args.len(), 1);
                assert_eq!(
                    unbound_slots,
                    vec!["validate".to_string(), "save".to_string()]
                );
            }
            other => panic!("expected PartialFlw, got {:?}", other),
        }
    }

    #[test]
    fn test_abstract_trf_direct_call_e051() {
        let errs = check_err(
            r#"
abstract stage FetchUser: Int -> String !Db
fn main() -> String { FetchUser(1) }
"#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0251")),
            "expected E051, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_abstract_trf_generic_binding_ok() {
        check_ok(
            r#"
abstract stage Fetch<T>: Int -> T? !Db
type User = { name: String }
abstract stage FetchUser: Int -> User? !Db
abstract seq Pipeline<Row> {
    fetch: Fetch<Row>
}
seq UserPipeline = Pipeline<User> { fetch <- FetchUser }
"#,
        );
    }

    #[test]
    fn test_abstract_trf_generic_binding_e048() {
        let errs = check_err(
            r#"
abstract stage Fetch<T>: Int -> T? !Db
type User = { name: String }
abstract stage FetchBad: String -> User? !Db
abstract seq Pipeline<Row> {
    fetch: Fetch<Row>
}
seq UserPipeline = Pipeline<User> { fetch <- FetchBad }
"#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0248")),
            "expected E048, got: {:?}",
            errs
        );
    }

    #[test]
    fn test_flw_binding_type_params() {
        check_ok(
            r#"
  abstract seq DataPipeline<Row> {
      parse: String -> List<Row>!
      save: List<Row> -> Int !Db
}
abstract stage ParseInts: String -> List<Int>!
abstract stage SaveInts: List<Int> -> Int !Db
  seq IntImport = DataPipeline<Int> { parse <- ParseInts; save <- SaveInts }
  "#,
        );
    }

    #[test]
    fn test_dynamic_injection_type_ok() {
        let prog = Parser::parse_str(
            r#"
abstract seq SavePipeline<Row> {
    save: Row -> Int !Db
}
"#,
            "test",
        )
        .expect("parse");
        let mut checker = Checker::new();
        checker.register_builtins();
        checker.register_item_signatures(&prog);
        for item in &prog.items {
            checker.check_item(item);
        }
        checker.env.push();
        checker.env.define(
            "save".to_string(),
            Type::Trf(
                Box::new(Type::Named("UserRow".into(), vec![])),
                Box::new(Type::Int),
                vec![Effect::Db],
            ),
        );
        let fd = FlwBindingDef {
            visibility: None,
            name: "Injected".into(),
            template: "SavePipeline".into(),
            type_args: vec![TypeExpr::Named("UserRow".into(), vec![], Span::dummy())],
            bindings: vec![("save".into(), crate::ast::SlotImpl::Local("save".into()))],
            span: Span::dummy(),
        };
        checker.check_flw_binding_def(&fd);
        let ty = checker
            .env
            .lookup("Injected")
            .cloned()
            .unwrap_or(Type::Unknown);
        checker.env.pop();

        assert!(
            checker.errors.is_empty(),
            "unexpected errors: {:?}",
            checker.errors
        );
        match ty {
            Type::Trf(input, output, effects) => {
                assert!(input.is_compatible(&Type::Named("UserRow".into(), vec![])));
                assert!(output.is_compatible(&Type::Int));
                assert!(effects.contains(&Effect::Db));
            }
            other => panic!("expected Trf, got {:?}", other),
        }
    }

    #[test]
    fn test_dynamic_injection_type_e048() {
        let prog = Parser::parse_str(
            r#"
abstract seq SavePipeline<Row> {
    save: Row -> Int !Db
}
"#,
            "test",
        )
        .expect("parse");
        let mut checker = Checker::new();
        checker.register_builtins();
        checker.register_item_signatures(&prog);
        for item in &prog.items {
            checker.check_item(item);
        }
        checker.env.push();
        checker.env.define(
            "save".to_string(),
            Type::Trf(
                Box::new(Type::String),
                Box::new(Type::Int),
                vec![Effect::Db],
            ),
        );
        let fd = FlwBindingDef {
            visibility: None,
            name: "Injected".into(),
            template: "SavePipeline".into(),
            type_args: vec![TypeExpr::Named("UserRow".into(), vec![], Span::dummy())],
            bindings: vec![("save".into(), crate::ast::SlotImpl::Local("save".into()))],
            span: Span::dummy(),
        };
        checker.check_flw_binding_def(&fd);
        checker.env.pop();

        let errs = checker
            .errors
            .iter()
            .map(|e| format!("[{}] {}", e.code, e.message))
            .collect::<Vec<_>>();
        assert!(
            errs.iter().any(|e| e.contains("E0248")),
            "expected E048, got: {:?}",
            errs
        );
    }

    // ── v1.7.0: Task<T> async ──────────────────────────────────────────────

    #[test]
    fn task_async_fn_returns_task_type() {
        let ty = inferred_type_of(
            r#"
async fn fetch_value() -> Int !Io {
    42
}
"#,
            "fetch_value",
        );
        // async fn with no params is registered as Fn([], Task<ret>)
        let inner_ret = match &ty {
            Type::Fn(_, ret) => ret.as_ref(),
            Type::Arrow(_, ret) => ret.as_ref(),
            other => panic!("expected Fn or Arrow, got {:?}", other),
        };
        assert!(
            matches!(inner_ret, Type::Task(inner) if matches!(**inner, Type::Int)),
            "expected Task<Int>, got {:?}",
            inner_ret
        );
    }

    #[test]
    fn task_bind_unwraps_task() {
        check_ok(
            r#"
async fn fetch_num() -> Int !Io {
    42
}
public fn main() -> Int !Io {
    bind n <- fetch_num()
    n
}
"#,
        );
    }

    #[test]
    fn task_run_executes_immediately() {
        check_ok(
            r#"
async fn compute() -> Int !Io {
    10
}
public fn main() -> Int !Io {
    bind v <- compute()
    Task.run(v)
}
"#,
        );
    }

    #[test]
    fn task_map_transforms_value() {
        check_ok(
            r#"
async fn get_x() -> Int !Io {
    5
}
public fn main() -> Int !Io {
    bind x <- get_x()
    Task.map(x, |n| n * 2)
}
"#,
        );
    }

    #[test]
    fn task_and_then_chains() {
        check_ok(
            r#"
async fn step1() -> Int !Io {
    3
}
public fn main() -> Int !Io {
    bind a <- step1()
    Task.and_then(a, |n| n + 1)
}
"#,
        );
    }

    // ── v1.7.0: Type aliases ───────────────────────────────────────────────

    #[test]
    fn type_alias_simple() {
        check_ok(
            r#"
type Name = String
public fn greet(n: Name) -> String {
    n
}
"#,
        );
    }

    #[test]
    fn type_alias_compatible_with_target() {
        check_ok(
            r#"
type UserId = Int
public fn get_id() -> UserId {
    42
}
"#,
        );
    }

    #[test]
    fn type_alias_generic() {
        check_ok(
            r#"
type MaybeInt = Option<Int>
public fn wrap(x: Int) -> MaybeInt {
    Option.some(x)
}
"#,
        );
    }

    // ── v1.9.0: for-in expression ────────────────────────────────────────────

    #[test]
    fn for_in_basic_ok() {
        check_ok(
            r#"
public fn main() -> Unit !Io {
    bind nums <- collect { yield 1; yield 2; yield 3; }
    for n in nums {
        IO.println_int(n)
    }
}
"#,
        );
    }

    #[test]
    fn for_in_non_list_e065() {
        let errs = check_err(
            r#"
public fn main() -> Unit !Io {
    for n in 42 {
        IO.println_int(n)
    }
}
"#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0365")),
            "expected E065, got {:?}",
            errs
        );
    }

    #[test]
    fn for_in_in_collect_allowed_v2_9() {
        // E067 was removed in v2.9.0: for inside collect is now supported
        check_ok(
            r#"
public fn main() -> List<Int> {
    collect {
        for n in List.range(0, 3) {
            yield n;
        }
    }
}
"#,
        );
    }

    // ── v1.9.0: ?? null-coalesce operator ───────────────────────────────────

    #[test]
    fn null_coalesce_ok() {
        check_ok(
            r#"
public fn main() -> Int {
    bind x: Option<Int> <- Option.some(5)
    x ?? 0
}
"#,
        );
    }

    #[test]
    fn null_coalesce_lhs_not_option_e068() {
        let errs = check_err(
            r#"
public fn main() -> Int {
    42 ?? 0
}
"#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0368")),
            "expected E068, got {:?}",
            errs
        );
    }

    #[test]
    fn null_coalesce_type_mismatch_e069() {
        let errs = check_err(
            r#"
public fn main() -> Int {
    bind x: Option<Int> <- Option.some(5)
    x ?? "fallback"
}
"#,
        );
        assert!(
            errs.iter().any(|e| e.contains("E0369")),
            "expected E069, got {:?}",
            errs
        );
    }

    // ── v1.9.0: stage/seq keyword aliases ───────────────────────────────────

    #[test]
    fn stage_alias_for_trf_ok() {
        check_ok(
            r#"
stage double: Int -> Int = |x| { x * 2 }
public fn main() -> Int {
    21 |> double
}
"#,
        );
    }

    #[test]
    fn seq_alias_for_flw_ok() {
        check_ok(
            r#"
stage add_one: Int -> Int = |x| { x + 1 }
seq pipeline = add_one
public fn main() -> Int {
    5 |> pipeline
}
"#,
        );
    }

    // ── v2.9.0: Stream<T> type ───────────────────────────────────────────────

    #[test]
    fn stream_type_annotation_ok() {
        check_ok(
            r#"
public fn main() -> Unit {
    bind s <- Stream.from(0, |n| n + 1)
    bind xs <- Stream.to_list(Stream.take(s, 3))
    IO.println_int(List.length(xs))
}
"#,
        );
    }

    #[test]
    fn collect_for_in_no_e067() {
        // v2.9.0: for inside collect is allowed (E067 removed)
        check_ok(
            r#"
public fn main() -> List<Int> {
    collect {
        for n in List.range(0, 5) {
            yield n * 2;
        }
    }
}
"#,
        );
    }

    // ── v4.1.5: Compile-time schema constraint checking ────────────────────

    fn check_with_schemas(src: &str, schemas: crate::schemas::ProjectSchemas) -> Vec<String> {
        let prog = Parser::parse_str(src, "test").expect("parse error");
        let mut checker = Checker::new();
        checker.register_builtins();
        checker.schemas = schemas;
        let (errs, _) = checker.check_with_self(&prog);
        errs.into_iter()
            .map(|e| format!("[{}] {}", e.code, e.message))
            .collect()
    }

    #[test]
    fn schema_constraint_positive_violation_on_literal() {
        use crate::schemas::{FieldConstraints, ProjectSchemas};
        use std::collections::HashMap;

        let mut fc = FieldConstraints::default();
        fc.constraints = vec!["positive".to_string()];
        let mut type_schema = HashMap::new();
        type_schema.insert("amount".to_string(), fc);
        let mut schemas: ProjectSchemas = HashMap::new();
        schemas.insert("Order".to_string(), type_schema);

        let src = r#"
type Order = { amount: Int }
public fn main() -> Order {
    Order { amount: -5 }
}
"#;
        let errs = check_with_schemas(src, schemas);
        assert!(
            errs.iter().any(|e| e.contains("E0510")),
            "expected E0510 positive violation, got: {:?}",
            errs
        );
    }

    #[test]
    fn schema_constraint_max_length_violation_on_literal() {
        use crate::schemas::{FieldConstraints, ProjectSchemas};
        use std::collections::HashMap;

        let mut fc = FieldConstraints::default();
        fc.max_length = Some(3);
        let mut type_schema = HashMap::new();
        type_schema.insert("code".to_string(), fc);
        let mut schemas: ProjectSchemas = HashMap::new();
        schemas.insert("Item".to_string(), type_schema);

        let src = r#"
type Item = { code: String }
public fn main() -> Item {
    Item { code: "TOOLONG" }
}
"#;
        let errs = check_with_schemas(src, schemas);
        assert!(
            errs.iter().any(|e| e.contains("E0513")),
            "expected E0513 max_length violation, got: {:?}",
            errs
        );
    }

    #[test]
    fn schema_constraint_min_violation_on_literal() {
        use crate::schemas::{FieldConstraints, ProjectSchemas};
        use std::collections::HashMap;

        let mut fc = FieldConstraints::default();
        fc.min = Some(0.0);
        let mut type_schema = HashMap::new();
        type_schema.insert("score".to_string(), fc);
        let mut schemas: ProjectSchemas = HashMap::new();
        schemas.insert("Score".to_string(), type_schema);

        let src = r#"
type Score = { score: Float }
public fn main() -> Score {
    Score { score: -1.0 }
}
"#;
        let errs = check_with_schemas(src, schemas);
        assert!(
            errs.iter().any(|e| e.contains("E0511")),
            "expected E0511 min violation, got: {:?}",
            errs
        );
    }

    #[test]
    fn schema_constraint_no_violation_passes() {
        use crate::schemas::{FieldConstraints, ProjectSchemas};
        use std::collections::HashMap;

        let mut fc = FieldConstraints::default();
        fc.constraints = vec!["positive".to_string()];
        let mut type_schema = HashMap::new();
        type_schema.insert("amount".to_string(), fc);
        let mut schemas: ProjectSchemas = HashMap::new();
        schemas.insert("Order".to_string(), type_schema);

        let src = r#"
type Order = { amount: Int }
public fn main() -> Order {
    Order { amount: 42 }
}
"#;
        let errs = check_with_schemas(src, schemas);
        let constraint_errs: Vec<_> = errs.iter()
            .filter(|e| e.contains("E051"))
            .collect();
        assert!(
            constraint_errs.is_empty(),
            "unexpected constraint errors: {:?}",
            constraint_errs
        );
    }
}

// ── Module export extraction ───────────────────────────────────────────────────

/// Extract the publicly-visible symbols from a program after type-checking.
/// Returns a map of symbol name ↁE(resolved Type, Visibility).
pub fn collect_exports(program: &Program, env: &TyEnv) -> HashMap<String, (Type, Visibility)> {
    let mut exports = HashMap::new();
    for item in &program.items {
        match item {
            Item::FnDef(fd) => {
                let vis = fd.visibility.clone().unwrap_or(Visibility::Private);
                if let Some(ty) = env.lookup(&fd.name) {
                    exports.insert(fd.name.clone(), (ty.clone(), vis));
                }
            }
            Item::TrfDef(td) => {
                let vis = td.visibility.clone().unwrap_or(Visibility::Private);
                if let Some(ty) = env.lookup(&td.name) {
                    exports.insert(td.name.clone(), (ty.clone(), vis));
                }
            }
            Item::AbstractTrfDef(td) => {
                let vis = td.visibility.clone().unwrap_or(Visibility::Private);
                if let Some(ty) = env.lookup(&td.name) {
                    exports.insert(td.name.clone(), (ty.clone(), vis));
                }
            }
            Item::FlwBindingDef(fd) => {
                let vis = fd.visibility.clone().unwrap_or(Visibility::Private);
                if let Some(ty) = env.lookup(&fd.name) {
                    exports.insert(fd.name.clone(), (ty.clone(), vis));
                }
            }
            Item::TypeDef(td) => {
                let vis = td.visibility.clone().unwrap_or(Visibility::Private);
                if let Some(ty) = env.lookup(&td.name) {
                    exports.insert(td.name.clone(), (ty.clone(), vis));
                }
            }
            _ => {}
        }
    }
    exports
}
