#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Expr;

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Unit,
    List(Vec<Value>),
    Record(HashMap<String, Value>),
    Variant(String, Option<Box<Value>>),
    Closure {
        params: Vec<String>,
        body: Box<Expr>,
        env: Env,
    },
    Flw(Vec<String>, Env),
    Namespace(String),
    Builtin(String, String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Record(a), Value::Record(b)) => a == b,
            (Value::Variant(an, ap), Value::Variant(bn, bp)) => an == bn && ap == bp,
            _ => false,
        }
    }
}

impl Value {
    pub fn display(&self) -> String {
        match self {
            Value::Bool(b) => b.to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => {
                if f.fract() == 0.0 {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            }
            Value::Str(s) => s.clone(),
            Value::Unit => "()".into(),
            Value::List(vs) => {
                let items: Vec<_> = vs.iter().map(|v| v.repr()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Record(m) => {
                let mut pairs: Vec<_> = m.iter().map(|(k, v)| format!("{}: {}", k, v.repr())).collect();
                pairs.sort();
                format!("{{ {} }}", pairs.join(", "))
            }
            Value::Variant(name, None) => name.clone(),
            Value::Variant(name, Some(v)) => format!("{}({})", name, v.repr()),
            Value::Closure { .. } => "<closure>".into(),
            Value::Flw(_, _) => "<flw>".into(),
            Value::Namespace(ns) => format!("<namespace:{}>", ns),
            Value::Builtin(ns, m) => format!("<builtin:{}.{}>", ns, m),
        }
    }

    pub fn repr(&self) -> String {
        match self {
            Value::Str(s) => format!("\"{}\"", s),
            other => other.display(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_) => "Bool",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Str(_) => "String",
            Value::Unit => "Unit",
            Value::List(_) => "List",
            Value::Record(_) => "Record",
            Value::Variant(..) => "Variant",
            Value::Closure { .. } => "Closure",
            Value::Flw(..) => "Flw",
            Value::Namespace(_) => "Namespace",
            Value::Builtin(..) => "Builtin",
        }
    }
}

pub type Env = Rc<RefCell<EnvInner>>;

pub struct EnvInner {
    bindings: HashMap<String, Value>,
    parent: Option<Env>,
}

impl std::fmt::Debug for EnvInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<env>")
    }
}

impl EnvInner {
    pub fn new_root() -> Env {
        Rc::new(RefCell::new(EnvInner { bindings: HashMap::new(), parent: None }))
    }

    pub fn new_child(parent: &Env) -> Env {
        Rc::new(RefCell::new(EnvInner {
            bindings: HashMap::new(),
            parent: Some(Rc::clone(parent)),
        }))
    }
}

pub fn env_define(env: &Env, name: String, val: Value) {
    env.borrow_mut().bindings.insert(name, val);
}

pub fn env_lookup(env: &Env, name: &str) -> Option<Value> {
    let inner = env.borrow();
    if let Some(v) = inner.bindings.get(name) {
        return Some(v.clone());
    }
    match &inner.parent {
        Some(p) => env_lookup(p, name),
        None => None,
    }
}
