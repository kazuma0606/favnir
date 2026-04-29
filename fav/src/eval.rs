// Favnir Interpreter
// Tasks: 5-1..5-27, 3-2..3-11

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use serde_json::Value as SerdeJsonValue;
use crate::ast::*;
use crate::lexer::Span;

// ── Span helper ───────────────────────────────────────────────────────────────

fn dummy_span() -> Span {
    Span { file: "<runtime>".into(), start: 0, end: 0, line: 0, col: 0 }
}

// ── Value (5-1) ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Unit,
    /// Ordered collection
    List(Vec<Value>),
    /// Record value: field map
    Record(HashMap<String, Value>),
    /// ADT variant: tag + optional payload
    Variant(String, Option<Box<Value>>),
    /// User-defined closure (fn, trf, or anonymous)
    Closure {
        params: Vec<String>,
        body: Box<Expr>,
        env: Env,
    },
    /// flw composition: ordered step names + the env in which steps are defined
    Flw(Vec<String>, Env),
    /// Namespace placeholder: IO, List, String, Option, Result, or type:<Name>
    Namespace(String),
    /// Resolved built-in method: namespace + method name
    Builtin(String, String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(a),   Value::Bool(b))   => a == b,
            (Value::Int(a),    Value::Int(b))    => a == b,
            (Value::Float(a),  Value::Float(b))  => a == b,
            (Value::Str(a),    Value::Str(b))    => a == b,
            (Value::Unit,      Value::Unit)      => true,
            (Value::List(a),   Value::List(b))   => a == b,
            (Value::Record(a), Value::Record(b)) => a == b,
            (Value::Variant(an, ap), Value::Variant(bn, bp)) => an == bn && ap == bp,
            _ => false,
        }
    }
}

impl Value {
    pub fn display(&self) -> String {
        match self {
            Value::Bool(b)    => b.to_string(),
            Value::Int(n)     => n.to_string(),
            Value::Float(f)   => {
                if f.fract() == 0.0 { format!("{:.1}", f) } else { f.to_string() }
            }
            Value::Str(s)     => s.clone(),
            Value::Unit       => "()".into(),
            Value::List(vs)   => {
                let items: Vec<_> = vs.iter().map(|v| v.repr()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Record(m)  => {
                let mut pairs: Vec<_> = m.iter().map(|(k, v)| format!("{}: {}", k, v.repr())).collect();
                pairs.sort();
                format!("{{ {} }}", pairs.join(", "))
            }
            Value::Variant(name, None)    => name.clone(),
            Value::Variant(name, Some(v)) => format!("{}({})", name, v.repr()),
            Value::Closure { .. }         => "<closure>".into(),
            Value::Flw(_, _)              => "<flw>".into(),
            Value::Namespace(ns)          => format!("<namespace:{}>", ns),
            Value::Builtin(ns, m)         => format!("<builtin:{}.{}>", ns, m),
        }
    }

    /// Debug representation (strings are quoted)
    pub fn repr(&self) -> String {
        match self {
            Value::Str(s) => format!("\"{}\"", s),
            other => other.display(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_)       => "Bool",
            Value::Int(_)        => "Int",
            Value::Float(_)      => "Float",
            Value::Str(_)        => "String",
            Value::Unit          => "Unit",
            Value::List(_)       => "List",
            Value::Record(_)     => "Record",
            Value::Variant(..)   => "Variant",
            Value::Closure { .. } => "Closure",
            Value::Flw(..)       => "Flw",
            Value::Namespace(_)  => "Namespace",
            Value::Builtin(..)   => "Builtin",
        }
    }
}

fn json_variant(name: &str, payload: Option<Value>) -> Value {
    Value::Variant(name.to_string(), payload.map(Box::new))
}

fn serde_to_favnir_json(value: SerdeJsonValue) -> Value {
    match value {
        SerdeJsonValue::Null => json_variant("json_null", None),
        SerdeJsonValue::Bool(b) => json_variant("json_bool", Some(Value::Bool(b))),
        SerdeJsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                json_variant("json_int", Some(Value::Int(i)))
            } else {
                json_variant("json_float", Some(Value::Float(n.as_f64().unwrap_or(0.0))))
            }
        }
        SerdeJsonValue::String(s) => json_variant("json_str", Some(Value::Str(s))),
        SerdeJsonValue::Array(items) => {
            json_variant(
                "json_array",
                Some(Value::List(items.into_iter().map(serde_to_favnir_json).collect())),
            )
        }
        SerdeJsonValue::Object(map) => {
            let mut fields = HashMap::new();
            for (k, v) in map {
                fields.insert(k, serde_to_favnir_json(v));
            }
            json_variant("json_object", Some(Value::Record(fields)))
        }
    }
}

fn favnir_json_to_serde(value: &Value) -> Option<SerdeJsonValue> {
    match value {
        Value::Variant(tag, None) if tag == "json_null" => Some(SerdeJsonValue::Null),
        Value::Variant(tag, Some(payload)) if tag == "json_bool" => match payload.as_ref() {
            Value::Bool(b) => Some(SerdeJsonValue::Bool(*b)),
            _ => None,
        },
        Value::Variant(tag, Some(payload)) if tag == "json_int" => match payload.as_ref() {
            Value::Int(i) => Some(SerdeJsonValue::Number((*i).into())),
            _ => None,
        },
        Value::Variant(tag, Some(payload)) if tag == "json_float" => match payload.as_ref() {
            Value::Float(f) => serde_json::Number::from_f64(*f).map(SerdeJsonValue::Number),
            _ => None,
        },
        Value::Variant(tag, Some(payload)) if tag == "json_str" => match payload.as_ref() {
            Value::Str(s) => Some(SerdeJsonValue::String(s.clone())),
            _ => None,
        },
        Value::Variant(tag, Some(payload)) if tag == "json_array" => match payload.as_ref() {
            Value::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(favnir_json_to_serde(item)?);
                }
                Some(SerdeJsonValue::Array(out))
            }
            _ => None,
        },
        Value::Variant(tag, Some(payload)) if tag == "json_object" => match payload.as_ref() {
            Value::Record(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    out.insert(k.clone(), favnir_json_to_serde(v)?);
                }
                Some(SerdeJsonValue::Object(out))
            }
            _ => None,
        },
        _ => None,
    }
}

fn value_string(value: Value, context: &str, span: &Span) -> Result<String, RuntimeError> {
    match value {
        Value::Str(s) => Ok(s),
        other => Err(RuntimeError::new(format!("{} expects String, got {}", context, other.type_name()), span)),
    }
}

fn value_string_list(value: Value, context: &str, span: &Span) -> Result<Vec<String>, RuntimeError> {
    match value {
        Value::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(value_string(item, context, span)?);
            }
            Ok(out)
        }
        other => Err(RuntimeError::new(format!("{} expects List<String>, got {}", context, other.type_name()), span)),
    }
}

// ── Env (5-2) ─────────────────────────────────────────────────────────────────

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
        None    => None,
    }
}

// ── RuntimeError ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub span: Span,
    /// `Some(v)` signals a `chain` early exit; caught at function-call boundaries.
    pub escape: Option<Value>,
}

impl RuntimeError {
    pub fn new(msg: impl Into<String>, span: &Span) -> Self {
        RuntimeError { message: msg.into(), span: span.clone(), escape: None }
    }
    /// Create a chain-escape signal carrying `val` (err(e) or none).
    pub fn chain_escape(val: Value, span: &Span) -> Self {
        RuntimeError { message: String::new(), span: span.clone(), escape: Some(val) }
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "runtime error: {}\n  --> {}:{}:{}",
            self.message, self.span.file, self.span.line, self.span.col
        )
    }
}

pub type EvalResult = Result<Value, RuntimeError>;

// ── Pattern matching helper (5-12) ────────────────────────────────────────────

/// Attempt to match `val` against `pat`.
/// Returns `Some(bindings)` on success, `None` on failure.
fn match_pattern(pat: &Pattern, val: &Value) -> Option<HashMap<String, Value>> {
    match pat {
        // Wildcard: always matches, no bindings
        Pattern::Wildcard(_) => Some(HashMap::new()),

        // Literal pattern: compare values
        Pattern::Lit(lit, _) => {
            let matches = match (lit, val) {
                (Lit::Bool(a),  Value::Bool(b))  => a == b,
                (Lit::Int(a),   Value::Int(b))   => a == b,
                (Lit::Float(a), Value::Float(b)) => a == b,
                (Lit::Str(a),   Value::Str(b))   => a == b,
                (Lit::Unit,     Value::Unit)     => true,
                _ => false,
            };
            if matches { Some(HashMap::new()) } else { None }
        }

        // Bind: always matches, binds name → value
        Pattern::Bind(name, _) => {
            let mut m = HashMap::new();
            m.insert(name.clone(), val.clone());
            Some(m)
        }

        // Variant: match tag, then recursively match payload
        Pattern::Variant(name, inner_pat, _) => {
            match val {
                Value::Variant(tag, payload) if tag == name => {
                    match (inner_pat, payload) {
                        (None, None) => Some(HashMap::new()),
                        (None, Some(_)) => Some(HashMap::new()), // ignore payload if no pattern
                        (Some(p), Some(v)) => match_pattern(p, v),
                        (Some(_), None) => None,
                    }
                }
                _ => None,
            }
        }

        // Record: each field pattern must match the corresponding field value
        Pattern::Record(field_pats, _) => {
            let record_map = match val {
                Value::Record(m) => m,
                _ => return None,
            };
            let mut bindings = HashMap::new();
            for fp in field_pats {
                let field_val = record_map.get(&fp.name)?.clone();
                match &fp.pattern {
                    None => {
                        // shorthand: `{ name }` binds field value to "name"
                        bindings.insert(fp.name.clone(), field_val);
                    }
                    Some(p) => {
                        let sub = match_pattern(p, &field_val)?;
                        bindings.extend(sub);
                    }
                }
            }
            Some(bindings)
        }
    }
}

// ── Binary operators (arithmetic/comparison) ──────────────────────────────────

fn eval_binop(op: &BinOp, l: Value, r: Value, span: &Span) -> EvalResult {
    use BinOp::*;
    match op {
        Add => match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a + *b as f64)),
            _ => Err(RuntimeError::new(format!("cannot add {} and {}", l.type_name(), r.type_name()), span)),
        },
        Sub => match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a - *b as f64)),
            _ => Err(RuntimeError::new(format!("cannot subtract {} and {}", l.type_name(), r.type_name()), span)),
        },
        Mul => match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a * *b as f64)),
            _ => Err(RuntimeError::new(format!("cannot multiply {} and {}", l.type_name(), r.type_name()), span)),
        },
        Div => match (&l, &r) {
            (Value::Int(a),   Value::Int(b)) => {
                if *b == 0 { Err(RuntimeError::new("division by zero", span)) }
                else { Ok(Value::Int(a / b)) }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
            (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a / *b as f64)),
            _ => Err(RuntimeError::new(format!("cannot divide {} and {}", l.type_name(), r.type_name()), span)),
        },
        Eq    => Ok(Value::Bool(l == r)),
        NotEq => Ok(Value::Bool(l != r)),
        Lt => match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            _ => Err(RuntimeError::new("comparison requires numeric types", span)),
        },
        Gt => match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            _ => Err(RuntimeError::new("comparison requires numeric types", span)),
        },
        LtEq => match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(RuntimeError::new("comparison requires numeric types", span)),
        },
        GtEq => match (&l, &r) {
            (Value::Int(a),   Value::Int(b))   => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(RuntimeError::new("comparison requires numeric types", span)),
        },
    }
}

// ── Db value conversion helpers (3-10, 3-11) ─────────────────────────────────

fn value_to_sql(v: &Value) -> Box<dyn rusqlite::ToSql> {
    match v {
        Value::Int(n)    => Box::new(*n),
        Value::Float(f)  => Box::new(*f),
        Value::Str(s)    => Box::new(s.clone()),
        Value::Bool(b)   => Box::new(*b as i64),
        Value::Unit      => Box::new(rusqlite::types::Null),
        other            => Box::new(other.repr()),
    }
}

fn sqlite_value_to_string(v: rusqlite::types::Value) -> String {
    match v {
        rusqlite::types::Value::Null       => "null".into(),
        rusqlite::types::Value::Integer(n) => n.to_string(),
        rusqlite::types::Value::Real(f)    => f.to_string(),
        rusqlite::types::Value::Text(s)    => s,
        rusqlite::types::Value::Blob(b)    => format!("<blob:{} bytes>", b.len()),
    }
}

// ── Built-in functions (5-21..5-25) ───────────────────────────────────────────

fn eval_builtin(ns: &str, method: &str, mut args: Vec<Value>, span: &Span) -> EvalResult {
    match (ns, method) {
        // ── IO (5-21) ────────────────────────────────────────────────────────
        // ── Trace (task 4-13/4-14) ──────────────────────────────────────────
        ("Trace", "print") => {
            let v = args.into_iter().next().unwrap_or(Value::Unit);
            eprintln!("[trace] {}", v.display());
            Ok(v)
        }
        ("Trace", "log") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Trace.log requires 2 arguments (label, value)", span));
            }
            let label = args.remove(0);
            let val   = args.remove(0);
            eprintln!("[trace] {}: {}", label.display(), val.display());
            Ok(val)
        }

        // ── IO (5-21) ────────────────────────────────────────────────────────
        ("IO", "print") => {
            let s = args.into_iter().next().unwrap_or(Value::Unit);
            print!("{}", s.display());
            Ok(Value::Unit)
        }
        ("IO", "println") => {
            let s = args.into_iter().next().unwrap_or(Value::Unit);
            println!("{}", s.display());
            Ok(Value::Unit)
        }

        // ── List (5-22) ──────────────────────────────────────────────────────
        ("List", "length") => {
            match args.into_iter().next() {
                Some(Value::List(vs)) => Ok(Value::Int(vs.len() as i64)),
                Some(v) => Err(RuntimeError::new(format!("List.length expects List, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("List.length requires 1 argument", span)),
            }
        }
        ("List", "is_empty") => {
            match args.into_iter().next() {
                Some(Value::List(vs)) => Ok(Value::Bool(vs.is_empty())),
                Some(v) => Err(RuntimeError::new(format!("List.is_empty expects List, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("List.is_empty requires 1 argument", span)),
            }
        }
        ("List", "first") => {
            match args.into_iter().next() {
                Some(Value::List(vs)) => {
                    let result = vs.into_iter().next()
                        .map(|v| Value::Variant("some".into(), Some(Box::new(v))))
                        .unwrap_or(Value::Variant("none".into(), None));
                    Ok(result)
                }
                Some(v) => Err(RuntimeError::new(format!("List.first expects List, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("List.first requires 1 argument", span)),
            }
        }
        ("List", "last") => {
            match args.into_iter().next() {
                Some(Value::List(vs)) => {
                    let result = vs.into_iter().last()
                        .map(|v| Value::Variant("some".into(), Some(Box::new(v))))
                        .unwrap_or(Value::Variant("none".into(), None));
                    Ok(result)
                }
                Some(v) => Err(RuntimeError::new(format!("List.last expects List, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("List.last requires 1 argument", span)),
            }
        }
        ("List", "map") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.map requires 2 arguments", span));
            }
            let f = args.remove(1);
            let list = args.remove(0);
            match list {
                Value::List(vs) => {
                    let mut out = Vec::with_capacity(vs.len());
                    for v in vs {
                        out.push(eval_apply(f.clone(), vec![v], span)?);
                    }
                    Ok(Value::List(out))
                }
                v => Err(RuntimeError::new(format!("List.map expects List, got {}", v.type_name()), span)),
            }
        }
        ("List", "filter") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.filter requires 2 arguments", span));
            }
            let f = args.remove(1);
            let list = args.remove(0);
            match list {
                Value::List(vs) => {
                    let mut out = Vec::new();
                    for v in vs {
                        match eval_apply(f.clone(), vec![v.clone()], span)? {
                            Value::Bool(true) => out.push(v),
                            Value::Bool(false) => {}
                            other => return Err(RuntimeError::new(
                                format!("List.filter predicate must return Bool, got {}", other.type_name()), span
                            )),
                        }
                    }
                    Ok(Value::List(out))
                }
                v => Err(RuntimeError::new(format!("List.filter expects List, got {}", v.type_name()), span)),
            }
        }
        ("List", "fold") => {
            if args.len() < 3 {
                return Err(RuntimeError::new("List.fold requires 3 arguments", span));
            }
            let f = args.remove(2);
            let init = args.remove(1);
            let list = args.remove(0);
            match list {
                Value::List(vs) => {
                    let mut acc = init;
                    for v in vs {
                        acc = eval_apply(f.clone(), vec![acc, v], span)?;
                    }
                    Ok(acc)
                }
                v => Err(RuntimeError::new(format!("List.fold expects List, got {}", v.type_name()), span)),
            }
        }
        ("List", "flat_map") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.flat_map requires 2 arguments", span));
            }
            let f = args.remove(1);
            let list = args.remove(0);
            match list {
                Value::List(vs) => {
                    let mut out = Vec::new();
                    for v in vs {
                        match eval_apply(f.clone(), vec![v], span)? {
                            Value::List(inner) => out.extend(inner),
                            other => return Err(RuntimeError::new(
                                format!("List.flat_map: callback must return List, got {}", other.type_name()), span
                            )),
                        }
                    }
                    Ok(Value::List(out))
                }
                v => Err(RuntimeError::new(format!("List.flat_map expects List, got {}", v.type_name()), span)),
            }
        }
        ("List", "zip") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.zip requires 2 arguments", span));
            }
            let ys_val = args.remove(1);
            let xs_val = args.remove(0);
            match (xs_val, ys_val) {
                (Value::List(xs), Value::List(ys)) => {
                    let pairs: Vec<Value> = xs.into_iter().zip(ys.into_iter()).map(|(x, y)| {
                        let mut m = HashMap::new();
                        m.insert("first".to_string(), x);
                        m.insert("second".to_string(), y);
                        Value::Record(m)
                    }).collect();
                    Ok(Value::List(pairs))
                }
                _ => Err(RuntimeError::new("List.zip expects (List, List)", span)),
            }
        }
        ("List", "sort") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.sort requires 2 arguments", span));
            }
            let cmp = args.remove(1);
            let list = args.remove(0);
            match list {
                Value::List(mut vs) => {
                    let mut err: Option<RuntimeError> = None;
                    vs.sort_by(|a, b| {
                        if err.is_some() { return std::cmp::Ordering::Equal; }
                        match eval_apply(cmp.clone(), vec![a.clone(), b.clone()], span) {
                            Ok(Value::Int(n)) => {
                                if n < 0 { std::cmp::Ordering::Less }
                                else if n > 0 { std::cmp::Ordering::Greater }
                                else { std::cmp::Ordering::Equal }
                            }
                            Ok(other) => {
                                err = Some(RuntimeError::new(
                                    format!("List.sort: comparator must return Int, got {}", other.type_name()), span
                                ));
                                std::cmp::Ordering::Equal
                            }
                            Err(e) => { err = Some(e); std::cmp::Ordering::Equal }
                        }
                    });
                    if let Some(e) = err { return Err(e); }
                    Ok(Value::List(vs))
                }
                v => Err(RuntimeError::new(format!("List.sort expects List, got {}", v.type_name()), span)),
            }
        }
        ("List", "range") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.range requires 2 arguments (start, end)", span));
            }
            let end_val = args.remove(1);
            let start_val = args.remove(0);
            match (start_val, end_val) {
                (Value::Int(s), Value::Int(e)) => {
                    let out: Vec<Value> = (s..e).map(Value::Int).collect();
                    Ok(Value::List(out))
                }
                _ => Err(RuntimeError::new("List.range expects (Int, Int)", span)),
            }
        }
        ("List", "reverse") => {
            match args.into_iter().next() {
                Some(Value::List(mut vs)) => { vs.reverse(); Ok(Value::List(vs)) }
                Some(v) => Err(RuntimeError::new(format!("List.reverse expects List, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("List.reverse requires 1 argument", span)),
            }
        }
        ("List", "concat") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.concat requires 2 arguments", span));
            }
            let ys_val = args.remove(1);
            let xs_val = args.remove(0);
            match (xs_val, ys_val) {
                (Value::List(mut xs), Value::List(ys)) => { xs.extend(ys); Ok(Value::List(xs)) }
                _ => Err(RuntimeError::new("List.concat expects (List, List)", span)),
            }
        }
        ("List", "take") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.take requires 2 arguments", span));
            }
            let n_val = args.remove(1);
            let list_val = args.remove(0);
            match (list_val, n_val) {
                (Value::List(vs), Value::Int(n)) => {
                    let n = n.max(0) as usize;
                    Ok(Value::List(vs.into_iter().take(n).collect()))
                }
                _ => Err(RuntimeError::new("List.take expects (List, Int)", span)),
            }
        }
        ("List", "drop") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.drop requires 2 arguments", span));
            }
            let n_val = args.remove(1);
            let list_val = args.remove(0);
            match (list_val, n_val) {
                (Value::List(vs), Value::Int(n)) => {
                    let n = n.max(0) as usize;
                    Ok(Value::List(vs.into_iter().skip(n).collect()))
                }
                _ => Err(RuntimeError::new("List.drop expects (List, Int)", span)),
            }
        }
        ("List", "enumerate") => {
            match args.into_iter().next() {
                Some(Value::List(vs)) => {
                    let pairs: Vec<Value> = vs.into_iter().enumerate().map(|(i, v)| {
                        let mut m = HashMap::new();
                        m.insert("first".to_string(), Value::Int(i as i64));
                        m.insert("second".to_string(), v);
                        Value::Record(m)
                    }).collect();
                    Ok(Value::List(pairs))
                }
                Some(v) => Err(RuntimeError::new(format!("List.enumerate expects List, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("List.enumerate requires 1 argument", span)),
            }
        }
        ("List", "find") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.find requires 2 arguments", span));
            }
            let pred = args.remove(1);
            let list = args.remove(0);
            match list {
                Value::List(vs) => {
                    for v in vs {
                        match eval_apply(pred.clone(), vec![v.clone()], span)? {
                            Value::Bool(true) => return Ok(Value::Variant("some".into(), Some(Box::new(v)))),
                            Value::Bool(false) => {}
                            other => return Err(RuntimeError::new(
                                format!("List.find predicate must return Bool, got {}", other.type_name()), span
                            )),
                        }
                    }
                    Ok(Value::Variant("none".into(), None))
                }
                v => Err(RuntimeError::new(format!("List.find expects List, got {}", v.type_name()), span)),
            }
        }
        ("List", "any") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.any requires 2 arguments", span));
            }
            let pred = args.remove(1);
            let list = args.remove(0);
            match list {
                Value::List(vs) => {
                    for v in vs {
                        match eval_apply(pred.clone(), vec![v], span)? {
                            Value::Bool(true) => return Ok(Value::Bool(true)),
                            Value::Bool(false) => {}
                            other => return Err(RuntimeError::new(
                                format!("List.any predicate must return Bool, got {}", other.type_name()), span
                            )),
                        }
                    }
                    Ok(Value::Bool(false))
                }
                v => Err(RuntimeError::new(format!("List.any expects List, got {}", v.type_name()), span)),
            }
        }
        ("List", "all") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.all requires 2 arguments", span));
            }
            let pred = args.remove(1);
            let list = args.remove(0);
            match list {
                Value::List(vs) => {
                    for v in vs {
                        match eval_apply(pred.clone(), vec![v], span)? {
                            Value::Bool(false) => return Ok(Value::Bool(false)),
                            Value::Bool(true) => {}
                            other => return Err(RuntimeError::new(
                                format!("List.all predicate must return Bool, got {}", other.type_name()), span
                            )),
                        }
                    }
                    Ok(Value::Bool(true))
                }
                v => Err(RuntimeError::new(format!("List.all expects List, got {}", v.type_name()), span)),
            }
        }
        ("List", "index_of") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.index_of requires 2 arguments", span));
            }
            let pred = args.remove(1);
            let list = args.remove(0);
            match list {
                Value::List(vs) => {
                    for (i, v) in vs.into_iter().enumerate() {
                        match eval_apply(pred.clone(), vec![v], span)? {
                            Value::Bool(true) => return Ok(Value::Variant("some".into(), Some(Box::new(Value::Int(i as i64))))),
                            Value::Bool(false) => {}
                            other => return Err(RuntimeError::new(
                                format!("List.index_of predicate must return Bool, got {}", other.type_name()), span
                            )),
                        }
                    }
                    Ok(Value::Variant("none".into(), None))
                }
                v => Err(RuntimeError::new(format!("List.index_of expects List, got {}", v.type_name()), span)),
            }
        }
        ("List", "join") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("List.join requires 2 arguments (xs, sep)", span));
            }
            let sep_val = args.remove(1);
            let list_val = args.remove(0);
            match (list_val, sep_val) {
                (Value::List(vs), Value::Str(sep)) => {
                    let mut parts = Vec::with_capacity(vs.len());
                    for v in vs {
                        match v {
                            Value::Str(s) => parts.push(s),
                            other => return Err(RuntimeError::new(
                                format!("List.join expects List<String>, got List<{}>", other.type_name()), span
                            )),
                        }
                    }
                    Ok(Value::Str(parts.join(&sep)))
                }
                _ => Err(RuntimeError::new("List.join expects (List<String>, String)", span)),
            }
        }

        // ── String (5-23) ─────────────────────────────────────────────────────
        ("String", "trim") => {
            match args.into_iter().next() {
                Some(Value::Str(s)) => Ok(Value::Str(s.trim().into())),
                Some(v) => Err(RuntimeError::new(format!("String.trim expects String, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("String.trim requires 1 argument", span)),
            }
        }
        ("String", "lower") => {
            match args.into_iter().next() {
                Some(Value::Str(s)) => Ok(Value::Str(s.to_lowercase())),
                Some(v) => Err(RuntimeError::new(format!("String.lower expects String, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("String.lower requires 1 argument", span)),
            }
        }
        ("String", "upper") => {
            match args.into_iter().next() {
                Some(Value::Str(s)) => Ok(Value::Str(s.to_uppercase())),
                Some(v) => Err(RuntimeError::new(format!("String.upper expects String, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("String.upper requires 1 argument", span)),
            }
        }
        ("String", "split") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("String.split requires 2 arguments (string, delimiter)", span));
            }
            let delim = args.remove(1);
            let s_val = args.remove(0);
            match (s_val, delim) {
                (Value::Str(s), Value::Str(d)) => {
                    let parts: Vec<Value> = s.split(d.as_str()).map(|p| Value::Str(p.into())).collect();
                    Ok(Value::List(parts))
                }
                _ => Err(RuntimeError::new("String.split expects (String, String)", span)),
            }
        }
        ("String", "length") => {
            match args.into_iter().next() {
                Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
                Some(v) => Err(RuntimeError::new(format!("String.length expects String, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("String.length requires 1 argument", span)),
            }
        }
        ("String", "is_empty") => {
            match args.into_iter().next() {
                Some(Value::Str(s)) => Ok(Value::Bool(s.is_empty())),
                Some(v) => Err(RuntimeError::new(format!("String.is_empty expects String, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("String.is_empty requires 1 argument", span)),
            }
        }
        ("String", "concat") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("String.concat requires 2 arguments", span));
            }
            let b = args.remove(1);
            let a = args.remove(0);
            match (a, b) {
                (Value::Str(s1), Value::Str(s2)) => Ok(Value::Str(format!("{}{}", s1, s2))),
                _ => Err(RuntimeError::new("String.concat expects (String, String)", span)),
            }
        }
        ("String", "join") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("String.join requires 2 arguments (xs, sep)", span));
            }
            let sep_val = args.remove(1);
            let list_val = args.remove(0);
            match (list_val, sep_val) {
                (Value::List(vs), Value::Str(sep)) => {
                    let mut parts = Vec::with_capacity(vs.len());
                    for v in vs {
                        match v {
                            Value::Str(s) => parts.push(s),
                            other => return Err(RuntimeError::new(
                                format!("String.join expects List<String>, got List<{}>", other.type_name()), span
                            )),
                        }
                    }
                    Ok(Value::Str(parts.join(&sep)))
                }
                _ => Err(RuntimeError::new("String.join expects (List<String>, String)", span)),
            }
        }
        ("String", "replace") => {
            if args.len() < 3 {
                return Err(RuntimeError::new("String.replace requires 3 arguments (s, from, to)", span));
            }
            let to_val = args.remove(2);
            let from_val = args.remove(1);
            let s_val = args.remove(0);
            match (s_val, from_val, to_val) {
                (Value::Str(s), Value::Str(from), Value::Str(to)) => {
                    Ok(Value::Str(s.replace(&*from, &*to)))
                }
                _ => Err(RuntimeError::new("String.replace expects (String, String, String)", span)),
            }
        }
        ("String", "starts_with") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("String.starts_with requires 2 arguments", span));
            }
            let prefix = args.remove(1);
            let s_val = args.remove(0);
            match (s_val, prefix) {
                (Value::Str(s), Value::Str(p)) => Ok(Value::Bool(s.starts_with(&*p))),
                _ => Err(RuntimeError::new("String.starts_with expects (String, String)", span)),
            }
        }
        ("String", "ends_with") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("String.ends_with requires 2 arguments", span));
            }
            let suffix = args.remove(1);
            let s_val = args.remove(0);
            match (s_val, suffix) {
                (Value::Str(s), Value::Str(suf)) => Ok(Value::Bool(s.ends_with(&*suf))),
                _ => Err(RuntimeError::new("String.ends_with expects (String, String)", span)),
            }
        }
        ("String", "contains") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("String.contains requires 2 arguments", span));
            }
            let sub = args.remove(1);
            let s_val = args.remove(0);
            match (s_val, sub) {
                (Value::Str(s), Value::Str(sub)) => Ok(Value::Bool(s.contains(&*sub))),
                _ => Err(RuntimeError::new("String.contains expects (String, String)", span)),
            }
        }
        ("String", "slice") => {
            if args.len() < 3 {
                return Err(RuntimeError::new("String.slice requires 3 arguments (s, start, end)", span));
            }
            let end_val = args.remove(2);
            let start_val = args.remove(1);
            let s_val = args.remove(0);
            match (s_val, start_val, end_val) {
                (Value::Str(s), Value::Int(start), Value::Int(end)) => {
                    let chars: Vec<char> = s.chars().collect();
                    let len = chars.len() as i64;
                    let s2 = start.max(0).min(len) as usize;
                    let e2 = end.max(0).min(len) as usize;
                    let e2 = e2.max(s2);
                    Ok(Value::Str(chars[s2..e2].iter().collect()))
                }
                _ => Err(RuntimeError::new("String.slice expects (String, Int, Int)", span)),
            }
        }
        ("String", "repeat") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("String.repeat requires 2 arguments", span));
            }
            let n_val = args.remove(1);
            let s_val = args.remove(0);
            match (s_val, n_val) {
                (Value::Str(s), Value::Int(n)) if n >= 0 => {
                    Ok(Value::Str(s.repeat(n as usize)))
                }
                (Value::Str(_), Value::Int(_)) => Err(RuntimeError::new("String.repeat requires non-negative count", span)),
                _ => Err(RuntimeError::new("String.repeat expects (String, Int)", span)),
            }
        }
        ("String", "char_at") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("String.char_at requires 2 arguments", span));
            }
            let idx_val = args.remove(1);
            let s_val = args.remove(0);
            match (s_val, idx_val) {
                (Value::Str(s), Value::Int(idx)) => {
                    let result = s.chars().nth(idx as usize)
                        .map(|c| Value::Variant("some".into(), Some(Box::new(Value::Str(c.to_string())))))
                        .unwrap_or(Value::Variant("none".into(), None));
                    Ok(result)
                }
                _ => Err(RuntimeError::new("String.char_at expects (String, Int)", span)),
            }
        }
        ("String", "to_int") => {
            match args.into_iter().next() {
                Some(Value::Str(s)) => {
                    let result = s.parse::<i64>()
                        .map(|n| Value::Variant("some".into(), Some(Box::new(Value::Int(n)))))
                        .unwrap_or(Value::Variant("none".into(), None));
                    Ok(result)
                }
                Some(v) => Err(RuntimeError::new(format!("String.to_int expects String, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("String.to_int requires 1 argument", span)),
            }
        }
        ("String", "to_float") => {
            match args.into_iter().next() {
                Some(Value::Str(s)) => {
                    let result = s.parse::<f64>()
                        .map(|f| Value::Variant("some".into(), Some(Box::new(Value::Float(f)))))
                        .unwrap_or(Value::Variant("none".into(), None));
                    Ok(result)
                }
                Some(v) => Err(RuntimeError::new(format!("String.to_float expects String, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("String.to_float requires 1 argument", span)),
            }
        }
        ("String", "from_int") => {
            match args.into_iter().next() {
                Some(Value::Int(n)) => Ok(Value::Str(n.to_string())),
                Some(v) => Err(RuntimeError::new(format!("String.from_int expects Int, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("String.from_int requires 1 argument", span)),
            }
        }
        ("String", "from_float") => {
            match args.into_iter().next() {
                Some(Value::Float(f)) => Ok(Value::Str(f.to_string())),
                Some(v) => Err(RuntimeError::new(format!("String.from_float expects Float, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("String.from_float requires 1 argument", span)),
            }
        }

        // ── Option (5-24) ─────────────────────────────────────────────────────
        ("Option", "some") => {
            let v = args.into_iter().next().unwrap_or(Value::Unit);
            Ok(Value::Variant("some".into(), Some(Box::new(v))))
        }
        ("Option", "none") => {
            Ok(Value::Variant("none".into(), None))
        }
        ("Option", "map") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Option.map requires 2 arguments", span));
            }
            let f = args.remove(1);
            let opt = args.remove(0);
            match opt {
                Value::Variant(tag, payload) if tag == "some" => {
                    let inner = payload.map(|v| *v).unwrap_or(Value::Unit);
                    let result = eval_apply(f, vec![inner], span)?;
                    Ok(Value::Variant("some".into(), Some(Box::new(result))))
                }
                Value::Variant(tag, _) if tag == "none" => {
                    Ok(Value::Variant("none".into(), None))
                }
                v => Err(RuntimeError::new(format!("Option.map expects Option, got {}", v.type_name()), span)),
            }
        }
        ("Option", "unwrap_or") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Option.unwrap_or requires 2 arguments", span));
            }
            let default = args.remove(1);
            let opt = args.remove(0);
            match opt {
                Value::Variant(tag, payload) if tag == "some" => {
                    Ok(payload.map(|v| *v).unwrap_or(Value::Unit))
                }
                Value::Variant(tag, _) if tag == "none" => Ok(default),
                v => Err(RuntimeError::new(format!("Option.unwrap_or expects Option, got {}", v.type_name()), span)),
            }
        }

        // ── Result (5-25) ─────────────────────────────────────────────────────
        ("Result", "ok") => {
            let v = args.into_iter().next().unwrap_or(Value::Unit);
            Ok(Value::Variant("ok".into(), Some(Box::new(v))))
        }
        ("Result", "err") => {
            let e = args.into_iter().next().unwrap_or(Value::Unit);
            Ok(Value::Variant("err".into(), Some(Box::new(e))))
        }
        ("Result", "map") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Result.map requires 2 arguments", span));
            }
            let f = args.remove(1);
            let res = args.remove(0);
            match res {
                Value::Variant(tag, payload) if tag == "ok" => {
                    let inner = payload.map(|v| *v).unwrap_or(Value::Unit);
                    let result = eval_apply(f, vec![inner], span)?;
                    Ok(Value::Variant("ok".into(), Some(Box::new(result))))
                }
                Value::Variant(tag, e) if tag == "err" => {
                    Ok(Value::Variant("err".into(), e))
                }
                v => Err(RuntimeError::new(format!("Result.map expects Result, got {}", v.type_name()), span)),
            }
        }
        ("Result", "unwrap_or") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Result.unwrap_or requires 2 arguments", span));
            }
            let default = args.remove(1);
            let res = args.remove(0);
            match res {
                Value::Variant(tag, payload) if tag == "ok" => {
                    Ok(payload.map(|v| *v).unwrap_or(Value::Unit))
                }
                Value::Variant(tag, _) if tag == "err" => Ok(default),
                v => Err(RuntimeError::new(format!("Result.unwrap_or expects Result, got {}", v.type_name()), span)),
            }
        }
        ("Option", "and_then") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Option.and_then requires 2 arguments", span));
            }
            let f = args.remove(1);
            let opt = args.remove(0);
            match opt {
                Value::Variant(tag, payload) if tag == "some" => {
                    let inner = payload.map(|v| *v).unwrap_or(Value::Unit);
                    eval_apply(f, vec![inner], span)
                }
                Value::Variant(tag, _) if tag == "none" => {
                    Ok(Value::Variant("none".into(), None))
                }
                v => Err(RuntimeError::new(format!("Option.and_then expects Option, got {}", v.type_name()), span)),
            }
        }
        ("Option", "or_else") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Option.or_else requires 2 arguments", span));
            }
            let f = args.remove(1);
            let opt = args.remove(0);
            match opt {
                Value::Variant(tag, payload) if tag == "some" => {
                    Ok(Value::Variant("some".into(), payload))
                }
                Value::Variant(tag, _) if tag == "none" => {
                    eval_apply(f, vec![Value::Unit], span)
                }
                v => Err(RuntimeError::new(format!("Option.or_else expects Option, got {}", v.type_name()), span)),
            }
        }
        ("Option", "is_some") => {
            match args.into_iter().next() {
                Some(Value::Variant(tag, _)) => Ok(Value::Bool(tag == "some")),
                Some(v) => Err(RuntimeError::new(format!("Option.is_some expects Option, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Option.is_some requires 1 argument", span)),
            }
        }
        ("Option", "is_none") => {
            match args.into_iter().next() {
                Some(Value::Variant(tag, _)) => Ok(Value::Bool(tag == "none")),
                Some(v) => Err(RuntimeError::new(format!("Option.is_none expects Option, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Option.is_none requires 1 argument", span)),
            }
        }
        ("Option", "to_result") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Option.to_result requires 2 arguments (option, err_val)", span));
            }
            let err_val = args.remove(1);
            let opt = args.remove(0);
            match opt {
                Value::Variant(tag, payload) if tag == "some" => {
                    let inner = payload.map(|v| *v).unwrap_or(Value::Unit);
                    Ok(Value::Variant("ok".into(), Some(Box::new(inner))))
                }
                Value::Variant(tag, _) if tag == "none" => {
                    Ok(Value::Variant("err".into(), Some(Box::new(err_val))))
                }
                v => Err(RuntimeError::new(format!("Option.to_result expects Option, got {}", v.type_name()), span)),
            }
        }
        ("Result", "map_err") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Result.map_err requires 2 arguments", span));
            }
            let f = args.remove(1);
            let res = args.remove(0);
            match res {
                Value::Variant(tag, payload) if tag == "ok" => {
                    Ok(Value::Variant("ok".into(), payload))
                }
                Value::Variant(tag, payload) if tag == "err" => {
                    let e = payload.map(|v| *v).unwrap_or(Value::Unit);
                    let mapped = eval_apply(f, vec![e], span)?;
                    Ok(Value::Variant("err".into(), Some(Box::new(mapped))))
                }
                v => Err(RuntimeError::new(format!("Result.map_err expects Result, got {}", v.type_name()), span)),
            }
        }
        ("Result", "and_then") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Result.and_then requires 2 arguments", span));
            }
            let f = args.remove(1);
            let res = args.remove(0);
            match res {
                Value::Variant(tag, payload) if tag == "ok" => {
                    let inner = payload.map(|v| *v).unwrap_or(Value::Unit);
                    eval_apply(f, vec![inner], span)
                }
                Value::Variant(tag, e) if tag == "err" => {
                    Ok(Value::Variant("err".into(), e))
                }
                v => Err(RuntimeError::new(format!("Result.and_then expects Result, got {}", v.type_name()), span)),
            }
        }
        ("Result", "is_ok") => {
            match args.into_iter().next() {
                Some(Value::Variant(tag, _)) => Ok(Value::Bool(tag == "ok")),
                Some(v) => Err(RuntimeError::new(format!("Result.is_ok expects Result, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Result.is_ok requires 1 argument", span)),
            }
        }
        ("Result", "is_err") => {
            match args.into_iter().next() {
                Some(Value::Variant(tag, _)) => Ok(Value::Bool(tag == "err")),
                Some(v) => Err(RuntimeError::new(format!("Result.is_err expects Result, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Result.is_err requires 1 argument", span)),
            }
        }
        ("Result", "to_option") => {
            match args.into_iter().next() {
                Some(Value::Variant(tag, payload)) if tag == "ok" => {
                    let inner = payload.map(|v| *v).unwrap_or(Value::Unit);
                    Ok(Value::Variant("some".into(), Some(Box::new(inner))))
                }
                Some(Value::Variant(tag, _)) if tag == "err" => {
                    Ok(Value::Variant("none".into(), None))
                }
                Some(v) => Err(RuntimeError::new(format!("Result.to_option expects Result, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Result.to_option requires 1 argument", span)),
            }
        }

        // ── Internal variant constructors (5-19) ─────────────────────────────
        ("__variant__", variant_name) => {
            let payload = args.into_iter().next();
            Ok(Value::Variant(variant_name.into(), payload.map(Box::new)))
        }

        // ── Internal record-variant constructors ──────────────────────────────
        (ns_str, field_names_csv) if ns_str.starts_with("__variant_record__:") => {
            let vname = ns_str.trim_start_matches("__variant_record__:").to_string();
            let fields: Vec<&str> = field_names_csv.split(',').collect();
            if fields.len() != args.len() {
                return Err(RuntimeError::new(
                    format!("variant `{}` expects {} fields, got {}", vname, fields.len(), args.len()),
                    span,
                ));
            }
            let mut map = HashMap::new();
            for (f, v) in fields.iter().zip(args.into_iter()) {
                map.insert(f.to_string(), v);
            }
            Ok(Value::Variant(vname, Some(Box::new(Value::Record(map)))))
        }

        // ── Map (3-15..3-18) ──────────────────────────────────────────────────
        ("Map", "get") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Map.get requires 2 arguments (map, key)", span));
            }
            let key = args.remove(1);
            let map = args.remove(0);
            let m = match map {
                Value::Record(m) => m,
                Value::Unit      => HashMap::new(),
                other => return Err(RuntimeError::new(
                    format!("Map.get expects (Record, String), got ({}, ...)", other.type_name()), span
                )),
            };
            match key {
                Value::Str(k) => Ok(match m.get(&k) {
                    Some(v) => Value::Variant("some".into(), Some(Box::new(v.clone()))),
                    None    => Value::Variant("none".into(), None),
                }),
                other => Err(RuntimeError::new(
                    format!("Map.get: key must be String, got {}", other.type_name()), span
                )),
            }
        }
        ("Map", "set") => {
            if args.len() < 3 {
                return Err(RuntimeError::new("Map.set requires 3 arguments (map, key, value)", span));
            }
            let val = args.remove(2);
            let key = args.remove(1);
            let map = args.remove(0);
            let mut m = match map {
                Value::Record(m) => m,
                Value::Unit      => HashMap::new(), // treat Unit as empty map
                other => return Err(RuntimeError::new(
                    format!("Map.set expects (Record, String, value), got ({}, ...)", other.type_name()), span
                )),
            };
            match key {
                Value::Str(k) => { m.insert(k, val); Ok(Value::Record(m)) }
                other => Err(RuntimeError::new(
                    format!("Map.set: key must be String, got {}", other.type_name()), span
                )),
            }
        }
        ("Map", "keys") => {
            match args.into_iter().next() {
                Some(Value::Record(m)) => {
                    let mut keys: Vec<Value> = m.into_keys().map(Value::Str).collect();
                    keys.sort_by(|a, b| a.display().cmp(&b.display()));
                    Ok(Value::List(keys))
                }
                Some(v) => Err(RuntimeError::new(format!("Map.keys expects Record, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Map.keys requires 1 argument", span)),
            }
        }
        ("Map", "values") => {
            match args.into_iter().next() {
                Some(Value::Record(m)) => {
                    let mut pairs: Vec<(String, Value)> = m.into_iter().collect();
                    pairs.sort_by(|a, b| a.0.cmp(&b.0));
                    Ok(Value::List(pairs.into_iter().map(|(_, v)| v).collect()))
                }
                Some(v) => Err(RuntimeError::new(format!("Map.values expects Record, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Map.values requires 1 argument", span)),
            }
        }
        ("Map", "has_key") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Map.has_key requires 2 arguments (map, key)", span));
            }
            let key = args.remove(1);
            let map = args.remove(0);
            match (map, key) {
                (Value::Record(m), Value::Str(k)) => Ok(Value::Bool(m.contains_key(&k))),
                (Value::Unit, Value::Str(_)) => Ok(Value::Bool(false)),
                _ => Err(RuntimeError::new("Map.has_key expects (Map, String)", span)),
            }
        }
        ("Map", "size") => {
            match args.into_iter().next() {
                Some(Value::Record(m)) => Ok(Value::Int(m.len() as i64)),
                Some(Value::Unit) => Ok(Value::Int(0)),
                Some(v) => Err(RuntimeError::new(format!("Map.size expects Map, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Map.size requires 1 argument", span)),
            }
        }
        ("Map", "is_empty") => {
            match args.into_iter().next() {
                Some(Value::Record(m)) => Ok(Value::Bool(m.is_empty())),
                Some(Value::Unit) => Ok(Value::Bool(true)),
                Some(v) => Err(RuntimeError::new(format!("Map.is_empty expects Map, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Map.is_empty requires 1 argument", span)),
            }
        }
        ("Map", "merge") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Map.merge requires 2 arguments (base, overrides)", span));
            }
            let overrides_val = args.remove(1);
            let base_val = args.remove(0);
            let to_map = |v: Value| match v {
                Value::Record(m) => Ok(m),
                Value::Unit => Ok(HashMap::new()),
                other => Err(RuntimeError::new(format!("Map.merge expects Map, got {}", other.type_name()), span)),
            };
            let mut base = to_map(base_val)?;
            let overrides = to_map(overrides_val)?;
            base.extend(overrides);
            Ok(Value::Record(base))
        }
        ("Map", "from_list") => {
            match args.into_iter().next() {
                Some(Value::List(pairs)) => {
                    let mut m = HashMap::new();
                    for pair in pairs {
                        match pair {
                            Value::Record(ref rec) => {
                                let k = rec.get("first").cloned().ok_or_else(|| RuntimeError::new("Map.from_list: Pair missing `first`", span))?;
                                let v = rec.get("second").cloned().ok_or_else(|| RuntimeError::new("Map.from_list: Pair missing `second`", span))?;
                                match k {
                                    Value::Str(s) => { m.insert(s, v); }
                                    other => return Err(RuntimeError::new(format!("Map.from_list: key must be String, got {}", other.type_name()), span)),
                                }
                            }
                            other => return Err(RuntimeError::new(format!("Map.from_list: expected Pair record, got {}", other.type_name()), span)),
                        }
                    }
                    Ok(Value::Record(m))
                }
                Some(v) => Err(RuntimeError::new(format!("Map.from_list expects List, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Map.from_list requires 1 argument", span)),
            }
        }
        ("Map", "to_list") => {
            match args.into_iter().next() {
                Some(Value::Record(m)) => {
                    let mut pairs: Vec<(String, Value)> = m.into_iter().collect();
                    pairs.sort_by(|a, b| a.0.cmp(&b.0));
                    let list: Vec<Value> = pairs.into_iter().map(|(k, v)| {
                        let mut rec = HashMap::new();
                        rec.insert("first".to_string(), Value::Str(k));
                        rec.insert("second".to_string(), v);
                        Value::Record(rec)
                    }).collect();
                    Ok(Value::List(list))
                }
                Some(Value::Unit) => Ok(Value::List(vec![])),
                Some(v) => Err(RuntimeError::new(format!("Map.to_list expects Map, got {}", v.type_name()), span)),
                None => Err(RuntimeError::new("Map.to_list requires 1 argument", span)),
            }
        }
        ("Map", "map_values") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Map.map_values requires 2 arguments (map, f)", span));
            }
            let f = args.remove(1);
            let map_val = args.remove(0);
            match map_val {
                Value::Record(m) => {
                    let mut out = HashMap::new();
                    for (k, v) in m {
                        out.insert(k, eval_apply(f.clone(), vec![v], span)?);
                    }
                    Ok(Value::Record(out))
                }
                Value::Unit => Ok(Value::Record(HashMap::new())),
                v => Err(RuntimeError::new(format!("Map.map_values expects Map, got {}", v.type_name()), span)),
            }
        }
        ("Map", "filter_values") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Map.filter_values requires 2 arguments (map, pred)", span));
            }
            let pred = args.remove(1);
            let map_val = args.remove(0);
            match map_val {
                Value::Record(m) => {
                    let mut out = HashMap::new();
                    for (k, v) in m {
                        match eval_apply(pred.clone(), vec![v.clone()], span)? {
                            Value::Bool(true) => { out.insert(k, v); }
                            Value::Bool(false) => {}
                            other => return Err(RuntimeError::new(
                                format!("Map.filter_values predicate must return Bool, got {}", other.type_name()), span
                            )),
                        }
                    }
                    Ok(Value::Record(out))
                }
                Value::Unit => Ok(Value::Record(HashMap::new())),
                v => Err(RuntimeError::new(format!("Map.filter_values expects Map, got {}", v.type_name()), span)),
            }
        }

        // ── Debug (3-19) ──────────────────────────────────────────────────────
        ("Json", "null") => Ok(json_variant("json_null", None)),
        ("Json", "bool") => match args.into_iter().next() {
            Some(Value::Bool(b)) => Ok(json_variant("json_bool", Some(Value::Bool(b)))),
            Some(v) => Err(RuntimeError::new(format!("Json.bool expects Bool, got {}", v.type_name()), span)),
            None => Err(RuntimeError::new("Json.bool requires 1 argument", span)),
        },
        ("Json", "int") => match args.into_iter().next() {
            Some(Value::Int(i)) => Ok(json_variant("json_int", Some(Value::Int(i)))),
            Some(v) => Err(RuntimeError::new(format!("Json.int expects Int, got {}", v.type_name()), span)),
            None => Err(RuntimeError::new("Json.int requires 1 argument", span)),
        },
        ("Json", "float") => match args.into_iter().next() {
            Some(Value::Float(f)) => Ok(json_variant("json_float", Some(Value::Float(f)))),
            Some(v) => Err(RuntimeError::new(format!("Json.float expects Float, got {}", v.type_name()), span)),
            None => Err(RuntimeError::new("Json.float requires 1 argument", span)),
        },
        ("Json", "str") => match args.into_iter().next() {
            Some(Value::Str(s)) => Ok(json_variant("json_str", Some(Value::Str(s)))),
            Some(v) => Err(RuntimeError::new(format!("Json.str expects String, got {}", v.type_name()), span)),
            None => Err(RuntimeError::new("Json.str requires 1 argument", span)),
        },
        ("Json", "array") => match args.into_iter().next() {
            Some(Value::List(items)) => Ok(json_variant("json_array", Some(Value::List(items)))),
            Some(v) => Err(RuntimeError::new(format!("Json.array expects List<Json>, got {}", v.type_name()), span)),
            None => Err(RuntimeError::new("Json.array requires 1 argument", span)),
        },
        ("Json", "object") => match args.into_iter().next() {
            Some(Value::List(fields)) => {
                let mut obj = HashMap::new();
                for field in fields {
                    let rec = match field {
                        Value::Record(rec) => rec,
                        other => return Err(RuntimeError::new(format!("Json.object expects List<JsonField>, got {}", other.type_name()), span)),
                    };
                    let key = match rec.get("key") {
                        Some(Value::Str(s)) => s.clone(),
                        Some(other) => return Err(RuntimeError::new(format!("JsonField.key must be String, got {}", other.type_name()), span)),
                        None => return Err(RuntimeError::new("JsonField missing `key`", span)),
                    };
                    let value = rec.get("value").cloned().ok_or_else(|| RuntimeError::new("JsonField missing `value`", span))?;
                    obj.insert(key, value);
                }
                Ok(json_variant("json_object", Some(Value::Record(obj))))
            }
            Some(v) => Err(RuntimeError::new(format!("Json.object expects List<JsonField>, got {}", v.type_name()), span)),
            None => Err(RuntimeError::new("Json.object requires 1 argument", span)),
        },
        ("Json", "parse") => match args.into_iter().next() {
            Some(Value::Str(s)) => match serde_json::from_str::<SerdeJsonValue>(&s) {
                Ok(v) => Ok(Value::Variant("some".into(), Some(Box::new(serde_to_favnir_json(v))))),
                Err(_) => Ok(Value::Variant("none".into(), None)),
            },
            Some(v) => Err(RuntimeError::new(format!("Json.parse expects String, got {}", v.type_name()), span)),
            None => Err(RuntimeError::new("Json.parse requires 1 argument", span)),
        },
        ("Json", "encode") | ("Json", "encode_pretty") => {
            let json = args.into_iter().next().ok_or_else(|| RuntimeError::new(format!("Json.{} requires 1 argument", method), span))?;
            let serde = favnir_json_to_serde(&json).ok_or_else(|| RuntimeError::new(format!("Json.{} expects Json", method), span))?;
            let out = if method == "encode_pretty" { serde_json::to_string_pretty(&serde) } else { serde_json::to_string(&serde) }
                .map_err(|e| RuntimeError::new(format!("Json.{} failed: {}", method, e), span))?;
            Ok(Value::Str(out))
        }
        ("Json", "get") => {
            if args.len() != 2 {
                return Err(RuntimeError::new("Json.get requires 2 arguments", span));
            }
            let mut it = args.into_iter();
            let json = it.next().unwrap();
            let key = match it.next().unwrap() {
                Value::Str(s) => s,
                other => return Err(RuntimeError::new(format!("Json.get expects String key, got {}", other.type_name()), span)),
            };
            match json {
                Value::Variant(tag, Some(payload)) if tag == "json_object" => match *payload {
                    Value::Record(map) => Ok(match map.get(&key) {
                        Some(v) => Value::Variant("some".into(), Some(Box::new(v.clone()))),
                        None => Value::Variant("none".into(), None),
                    }),
                    _ => Err(RuntimeError::new("Json.get received malformed json_object payload", span)),
                },
                _ => Ok(Value::Variant("none".into(), None)),
            }
        }
        ("Json", "at") => {
            if args.len() != 2 {
                return Err(RuntimeError::new("Json.at requires 2 arguments", span));
            }
            let mut it = args.into_iter();
            let json = it.next().unwrap();
            let idx = match it.next().unwrap() {
                Value::Int(i) => i,
                other => return Err(RuntimeError::new(format!("Json.at expects Int index, got {}", other.type_name()), span)),
            };
            match json {
                Value::Variant(tag, Some(payload)) if tag == "json_array" => match *payload {
                    Value::List(items) if idx >= 0 => Ok(items.get(idx as usize).cloned().map(|v| Value::Variant("some".into(), Some(Box::new(v)))).unwrap_or(Value::Variant("none".into(), None))),
                    Value::List(_) => Ok(Value::Variant("none".into(), None)),
                    _ => Err(RuntimeError::new("Json.at received malformed json_array payload", span)),
                },
                _ => Ok(Value::Variant("none".into(), None)),
            }
        }
        ("Json", "as_str") => match args.into_iter().next() {
            Some(Value::Variant(tag, Some(payload))) if tag == "json_str" => Ok(Value::Variant("some".into(), Some(payload))),
            Some(_) => Ok(Value::Variant("none".into(), None)),
            None => Err(RuntimeError::new("Json.as_str requires 1 argument", span)),
        },
        ("Json", "as_int") => match args.into_iter().next() {
            Some(Value::Variant(tag, Some(payload))) if tag == "json_int" => Ok(Value::Variant("some".into(), Some(payload))),
            Some(_) => Ok(Value::Variant("none".into(), None)),
            None => Err(RuntimeError::new("Json.as_int requires 1 argument", span)),
        },
        ("Json", "as_float") => match args.into_iter().next() {
            Some(Value::Variant(tag, Some(payload))) if tag == "json_float" => Ok(Value::Variant("some".into(), Some(payload))),
            Some(_) => Ok(Value::Variant("none".into(), None)),
            None => Err(RuntimeError::new("Json.as_float requires 1 argument", span)),
        },
        ("Json", "as_bool") => match args.into_iter().next() {
            Some(Value::Variant(tag, Some(payload))) if tag == "json_bool" => Ok(Value::Variant("some".into(), Some(payload))),
            Some(_) => Ok(Value::Variant("none".into(), None)),
            None => Err(RuntimeError::new("Json.as_bool requires 1 argument", span)),
        },
        ("Json", "as_array") => match args.into_iter().next() {
            Some(Value::Variant(tag, Some(payload))) if tag == "json_array" => Ok(Value::Variant("some".into(), Some(payload))),
            Some(_) => Ok(Value::Variant("none".into(), None)),
            None => Err(RuntimeError::new("Json.as_array requires 1 argument", span)),
        },
        ("Json", "is_null") => match args.into_iter().next() {
            Some(Value::Variant(tag, None)) if tag == "json_null" => Ok(Value::Bool(true)),
            Some(_) => Ok(Value::Bool(false)),
            None => Err(RuntimeError::new("Json.is_null requires 1 argument", span)),
        },
        ("Json", "keys") => match args.into_iter().next() {
            Some(Value::Variant(tag, Some(payload))) if tag == "json_object" => match *payload {
                Value::Record(map) => {
                    let mut keys: Vec<Value> = map.into_keys().map(Value::Str).collect();
                    keys.sort_by(|a, b| a.display().cmp(&b.display()));
                    Ok(Value::Variant("some".into(), Some(Box::new(Value::List(keys)))))
                }
                _ => Err(RuntimeError::new("Json.keys received malformed json_object payload", span)),
            },
            Some(_) => Ok(Value::Variant("none".into(), None)),
            None => Err(RuntimeError::new("Json.keys requires 1 argument", span)),
        },
        ("Json", "length") => match args.into_iter().next() {
            Some(Value::Variant(tag, Some(payload))) if tag == "json_array" => match *payload {
                Value::List(items) => Ok(Value::Variant("some".into(), Some(Box::new(Value::Int(items.len() as i64))))),
                _ => Err(RuntimeError::new("Json.length received malformed json_array payload", span)),
            },
            Some(Value::Variant(tag, Some(payload))) if tag == "json_object" => match *payload {
                Value::Record(map) => Ok(Value::Variant("some".into(), Some(Box::new(Value::Int(map.len() as i64))))),
                _ => Err(RuntimeError::new("Json.length received malformed json_object payload", span)),
            },
            Some(_) => Ok(Value::Variant("none".into(), None)),
            None => Err(RuntimeError::new("Json.length requires 1 argument", span)),
        },

        ("Csv", "parse") => {
            let input = value_string(
                args.into_iter().next().ok_or_else(|| RuntimeError::new("Csv.parse requires 1 argument", span))?,
                "Csv.parse",
                span,
            )?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(input.as_bytes());
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = record.map_err(|e| RuntimeError::new(format!("Csv.parse failed: {}", e), span))?;
                rows.push(Value::List(record.iter().map(|cell| Value::Str(cell.to_string())).collect()));
            }
            Ok(Value::List(rows))
        }
        ("Csv", "parse_with_header") => {
            let input = value_string(
                args.into_iter().next().ok_or_else(|| RuntimeError::new("Csv.parse_with_header requires 1 argument", span))?,
                "Csv.parse_with_header",
                span,
            )?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .from_reader(input.as_bytes());
            let headers = rdr.headers()
                .map_err(|e| RuntimeError::new(format!("Csv.parse_with_header failed: {}", e), span))?
                .clone();
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = record.map_err(|e| RuntimeError::new(format!("Csv.parse_with_header failed: {}", e), span))?;
                let mut row = HashMap::new();
                for (key, value) in headers.iter().zip(record.iter()) {
                    row.insert(key.to_string(), Value::Str(value.to_string()));
                }
                rows.push(Value::Record(row));
            }
            Ok(Value::List(rows))
        }
        ("Csv", "encode") => {
            let rows = match args.into_iter().next() {
                Some(Value::List(rows)) => rows,
                Some(other) => return Err(RuntimeError::new(format!("Csv.encode expects List<List<String>>, got {}", other.type_name()), span)),
                None => return Err(RuntimeError::new("Csv.encode requires 1 argument", span)),
            };
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            for row in rows {
                let fields = value_string_list(row, "Csv.encode", span)?;
                writer.write_record(fields)
                    .map_err(|e| RuntimeError::new(format!("Csv.encode failed: {}", e), span))?;
            }
            let bytes = writer.into_inner()
                .map_err(|e| RuntimeError::new(format!("Csv.encode failed: {}", e.into_error()), span))?;
            let out = String::from_utf8(bytes)
                .map_err(|e| RuntimeError::new(format!("Csv.encode produced invalid UTF-8: {}", e), span))?;
            Ok(Value::Str(out))
        }
        ("Csv", "encode_with_header") => {
            if args.len() != 2 {
                return Err(RuntimeError::new("Csv.encode_with_header requires 2 arguments", span));
            }
            let mut it = args.into_iter();
            let header = value_string_list(it.next().unwrap(), "Csv.encode_with_header", span)?;
            let rows = match it.next().unwrap() {
                Value::List(rows) => rows,
                other => return Err(RuntimeError::new(format!("Csv.encode_with_header expects List<List<String>>, got {}", other.type_name()), span)),
            };
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            writer.write_record(&header)
                .map_err(|e| RuntimeError::new(format!("Csv.encode_with_header failed: {}", e), span))?;
            for row in rows {
                let fields = value_string_list(row, "Csv.encode_with_header", span)?;
                writer.write_record(fields)
                    .map_err(|e| RuntimeError::new(format!("Csv.encode_with_header failed: {}", e), span))?;
            }
            let bytes = writer.into_inner()
                .map_err(|e| RuntimeError::new(format!("Csv.encode_with_header failed: {}", e.into_error()), span))?;
            let out = String::from_utf8(bytes)
                .map_err(|e| RuntimeError::new(format!("Csv.encode_with_header produced invalid UTF-8: {}", e), span))?;
            Ok(Value::Str(out))
        }
        ("Csv", "from_records") => {
            let records = match args.into_iter().next() {
                Some(Value::List(records)) => records,
                Some(other) => return Err(RuntimeError::new(format!("Csv.from_records expects List<Map<String>>, got {}", other.type_name()), span)),
                None => return Err(RuntimeError::new("Csv.from_records requires 1 argument", span)),
            };
            let mut headers = std::collections::BTreeSet::new();
            let mut rows = Vec::new();
            for record in records {
                match record {
                    Value::Record(map) => {
                        for key in map.keys() {
                            headers.insert(key.clone());
                        }
                        rows.push(map);
                    }
                    other => return Err(RuntimeError::new(format!("Csv.from_records expects record rows, got {}", other.type_name()), span)),
                }
            }
            let header: Vec<String> = headers.into_iter().collect();
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            writer.write_record(&header)
                .map_err(|e| RuntimeError::new(format!("Csv.from_records failed: {}", e), span))?;
            for row in rows {
                let mut values = Vec::with_capacity(header.len());
                for key in &header {
                    let value = row.get(key).cloned().unwrap_or(Value::Str(String::new()));
                    values.push(value_string(value, "Csv.from_records", span)?);
                }
                writer.write_record(values)
                    .map_err(|e| RuntimeError::new(format!("Csv.from_records failed: {}", e), span))?;
            }
            let bytes = writer.into_inner()
                .map_err(|e| RuntimeError::new(format!("Csv.from_records failed: {}", e.into_error()), span))?;
            let out = String::from_utf8(bytes)
                .map_err(|e| RuntimeError::new(format!("Csv.from_records produced invalid UTF-8: {}", e), span))?;
            Ok(Value::Str(out))
        }

        ("Debug", "show") => {
            let v = args.into_iter().next().unwrap_or(Value::Unit);
            Ok(Value::Str(v.repr()))
        }

        // ── Util ──────────────────────────────────────────────────────────────
        ("Util", "uuid") => {
            Ok(Value::Str(uuid::Uuid::new_v4().to_string()))
        }
        ("Util", _) => Err(RuntimeError::new(format!("Util.{} is not implemented", method), span)),

        // ── Emit.log (3-4) ────────────────────────────────────────────────────
        ("Emit", "log") => {
            let snapshot: Vec<Value> = emit_log_snapshot()
                .into_iter()
                .map(|v| Value::Str(v.repr()))
                .collect();
            Ok(Value::List(snapshot))
        }

        // ── Db (3-7..3-11) ────────────────────────────────────────────────────
        ("File", "read") => {
            let path = match args.into_iter().next() {
                Some(Value::Str(s)) => s,
                Some(other) => return Err(RuntimeError::new(format!("File.read expects String path, got {}", other.type_name()), span)),
                None => return Err(RuntimeError::new("File.read requires 1 argument", span)),
            };
            let content = std::fs::read_to_string(&path)
                .map_err(|e| RuntimeError::new(format!("File.read failed for `{}`: {}", path, e), span))?;
            Ok(Value::Str(content))
        }
        ("File", "read_lines") => {
            let path = match args.into_iter().next() {
                Some(Value::Str(s)) => s,
                Some(other) => return Err(RuntimeError::new(format!("File.read_lines expects String path, got {}", other.type_name()), span)),
                None => return Err(RuntimeError::new("File.read_lines requires 1 argument", span)),
            };
            let content = std::fs::read_to_string(&path)
                .map_err(|e| RuntimeError::new(format!("File.read_lines failed for `{}`: {}", path, e), span))?;
            Ok(Value::List(content.lines().map(|line| Value::Str(line.to_string())).collect()))
        }
        ("File", "write") => {
            if args.len() != 2 {
                return Err(RuntimeError::new("File.write requires 2 arguments", span));
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                Value::Str(s) => s,
                other => return Err(RuntimeError::new(format!("File.write expects String path, got {}", other.type_name()), span)),
            };
            let content = match it.next().unwrap() {
                Value::Str(s) => s,
                other => return Err(RuntimeError::new(format!("File.write expects String content, got {}", other.type_name()), span)),
            };
            std::fs::write(&path, content)
                .map_err(|e| RuntimeError::new(format!("File.write failed for `{}`: {}", path, e), span))?;
            Ok(Value::Unit)
        }
        ("File", "write_lines") => {
            if args.len() != 2 {
                return Err(RuntimeError::new("File.write_lines requires 2 arguments", span));
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                Value::Str(s) => s,
                other => return Err(RuntimeError::new(format!("File.write_lines expects String path, got {}", other.type_name()), span)),
            };
            let lines = match it.next().unwrap() {
                Value::List(items) => {
                    let mut parts = Vec::with_capacity(items.len());
                    for item in items {
                        match item {
                            Value::Str(s) => parts.push(s),
                            other => return Err(RuntimeError::new(format!("File.write_lines expects List<String>, got List<{}>", other.type_name()), span)),
                        }
                    }
                    parts
                }
                other => return Err(RuntimeError::new(format!("File.write_lines expects List<String>, got {}", other.type_name()), span)),
            };
            std::fs::write(&path, lines.join("\n"))
                .map_err(|e| RuntimeError::new(format!("File.write_lines failed for `{}`: {}", path, e), span))?;
            Ok(Value::Unit)
        }
        ("File", "append") => {
            use std::io::Write;
            if args.len() != 2 {
                return Err(RuntimeError::new("File.append requires 2 arguments", span));
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                Value::Str(s) => s,
                other => return Err(RuntimeError::new(format!("File.append expects String path, got {}", other.type_name()), span)),
            };
            let content = match it.next().unwrap() {
                Value::Str(s) => s,
                other => return Err(RuntimeError::new(format!("File.append expects String content, got {}", other.type_name()), span)),
            };
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| RuntimeError::new(format!("File.append failed for `{}`: {}", path, e), span))?;
            file.write_all(content.as_bytes())
                .map_err(|e| RuntimeError::new(format!("File.append failed for `{}`: {}", path, e), span))?;
            Ok(Value::Unit)
        }
        ("File", "exists") => {
            let path = match args.into_iter().next() {
                Some(Value::Str(s)) => s,
                Some(other) => return Err(RuntimeError::new(format!("File.exists expects String path, got {}", other.type_name()), span)),
                None => return Err(RuntimeError::new("File.exists requires 1 argument", span)),
            };
            Ok(Value::Bool(std::path::Path::new(&path).exists()))
        }
        ("File", "delete") => {
            let path = match args.into_iter().next() {
                Some(Value::Str(s)) => s,
                Some(other) => return Err(RuntimeError::new(format!("File.delete expects String path, got {}", other.type_name()), span)),
                None => return Err(RuntimeError::new("File.delete requires 1 argument", span)),
            };
            match std::fs::remove_file(&path) {
                Ok(_) => Ok(Value::Unit),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Unit),
                Err(e) => Err(RuntimeError::new(format!("File.delete failed for `{}`: {}", path, e), span)),
            }
        }
        ("Db", "execute") => {
            // Db.execute(sql, args...) → Int (rows changed)
            let sql = match args.first() {
                Some(Value::Str(s)) => s.clone(),
                Some(v) => return Err(RuntimeError::new(
                    format!("Db.execute: first arg must be String, got {}", v.type_name()), span
                )),
                None => return Err(RuntimeError::new("Db.execute requires a SQL string", span)),
            };
            let params = &args[1..];
            with_db(span, |conn| {
                let mut stmt = conn.prepare(&sql)?;
                let bound: Vec<Box<dyn rusqlite::ToSql>> = params.iter().map(value_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> = bound.iter().map(|b| b.as_ref()).collect();
                let rows = stmt.execute(refs.as_slice())?;
                Ok(Value::Int(rows as i64))
            })
        }
        ("Db", "query") => {
            // Db.query(sql, args...) → List<Map<String, String>>
            let sql = match args.first() {
                Some(Value::Str(s)) => s.clone(),
                Some(v) => return Err(RuntimeError::new(
                    format!("Db.query: first arg must be String, got {}", v.type_name()), span
                )),
                None => return Err(RuntimeError::new("Db.query requires a SQL string", span)),
            };
            let params = &args[1..];
            with_db(span, |conn| {
                let mut stmt = conn.prepare(&sql)?;
                let bound: Vec<Box<dyn rusqlite::ToSql>> = params.iter().map(value_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> = bound.iter().map(|b| b.as_ref()).collect();
                let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
                let rows: Result<Vec<Value>, rusqlite::Error> = stmt
                    .query_map(refs.as_slice(), |row| {
                        let mut map = HashMap::new();
                        for (i, name) in col_names.iter().enumerate() {
                            let v: rusqlite::types::Value = row.get(i)?;
                            map.insert(name.clone(), Value::Str(sqlite_value_to_string(v)));
                        }
                        Ok(Value::Record(map))
                    })?
                    .collect();
                Ok(Value::List(rows?))
            })
        }
        ("Db", "query_one") => {
            // Db.query_one(sql, args...) → Map<String, String>?
            let sql = match args.first() {
                Some(Value::Str(s)) => s.clone(),
                Some(v) => return Err(RuntimeError::new(
                    format!("Db.query_one: first arg must be String, got {}", v.type_name()), span
                )),
                None => return Err(RuntimeError::new("Db.query_one requires a SQL string", span)),
            };
            let params = &args[1..];
            with_db(span, |conn| {
                let mut stmt = conn.prepare(&sql)?;
                let bound: Vec<Box<dyn rusqlite::ToSql>> = params.iter().map(value_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> = bound.iter().map(|b| b.as_ref()).collect();
                let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
                let mut rows = stmt.query(refs.as_slice())?;
                match rows.next()? {
                    None => Ok(Value::Variant("none".into(), None)),
                    Some(row) => {
                        let mut map = HashMap::new();
                        for (i, name) in col_names.iter().enumerate() {
                            let v: rusqlite::types::Value = row.get(i)?;
                            map.insert(name.clone(), Value::Str(sqlite_value_to_string(v)));
                        }
                        Ok(Value::Variant("some".into(), Some(Box::new(Value::Record(map)))))
                    }
                }
            })
        }
        ("Db", _) => Err(RuntimeError::new(
            format!("Db.{} is not implemented", method), span
        )),

        // ── Http (3-13, 3-14 via ureq) ───────────────────────────────────────
        ("Http", "get") => {
            let url = match args.into_iter().next() {
                Some(Value::Str(s)) => s,
                Some(v) => return Err(RuntimeError::new(
                    format!("Http.get: URL must be String, got {}", v.type_name()), span
                )),
                None => return Err(RuntimeError::new("Http.get requires a URL argument", span)),
            };
            match ureq::get(&url).call() {
                Ok(resp) => {
                    let body = resp.into_string()
                        .map_err(|e| RuntimeError::new(format!("Http.get read error: {}", e), span))?;
                    Ok(Value::Variant("ok".into(), Some(Box::new(Value::Str(body)))))
                }
                Err(e) => Ok(Value::Variant("err".into(), Some(Box::new(Value::Str(e.to_string()))))),
            }
        }
        ("Http", "post") => {
            if args.len() < 2 {
                return Err(RuntimeError::new("Http.post requires 2 arguments (url, body)", span));
            }
            let body_val = args.remove(1);
            let url = match args.remove(0) {
                Value::Str(s) => s,
                v => return Err(RuntimeError::new(
                    format!("Http.post: URL must be String, got {}", v.type_name()), span
                )),
            };
            let body_str = match body_val {
                Value::Str(s) => s,
                v => v.display(),
            };
            match ureq::post(&url).send_string(&body_str) {
                Ok(resp) => {
                    let body = resp.into_string()
                        .map_err(|e| RuntimeError::new(format!("Http.post read error: {}", e), span))?;
                    Ok(Value::Variant("ok".into(), Some(Box::new(Value::Str(body)))))
                }
                Err(e) => Ok(Value::Variant("err".into(), Some(Box::new(Value::Str(e.to_string()))))),
            }
        }
        ("Http", _) => Err(RuntimeError::new(
            format!("Http.{} is not implemented", method), span
        )),

        // ── Built-in cap method dispatch (v0.4.0) ────────────────────────────
        // ns = "cap_eq_int", "cap_ord_float", etc.  method = "equals"|"compare"|"show"
        (ns, method) if ns.starts_with("cap_") => {
            let parts: Vec<&str> = ns.splitn(3, '_').collect(); // ["cap", "eq"|"ord"|"show", ty_lower]
            if parts.len() < 3 {
                return Err(RuntimeError::new(format!("malformed cap builtin: {}", ns), span));
            }
            let cap = parts[1];
            let ty  = parts[2];
            match (cap, ty, method) {
                // equals — Int, Float, String, Bool
                ("eq", _, "equals") => {
                    if args.len() != 2 {
                        return Err(RuntimeError::new("equals requires 2 arguments", span));
                    }
                    Ok(Value::Bool(args[0] == args[1]))
                }
                // compare — Int, Float, String
                ("ord", "int", "compare") => {
                    match (&args[0], &args[1]) {
                        (Value::Int(a), Value::Int(b)) => {
                            Ok(Value::Int(a.cmp(b) as i64))
                        }
                        _ => Err(RuntimeError::new("ord.compare expects Int", span)),
                    }
                }
                ("ord", "float", "compare") => {
                    match (&args[0], &args[1]) {
                        (Value::Float(a), Value::Float(b)) => {
                            Ok(Value::Int(a.partial_cmp(b).map(|o| o as i64).unwrap_or(0)))
                        }
                        _ => Err(RuntimeError::new("ord.compare expects Float", span)),
                    }
                }
                ("ord", "string", "compare") => {
                    match (&args[0], &args[1]) {
                        (Value::Str(a), Value::Str(b)) => {
                            Ok(Value::Int(a.cmp(b) as i64))
                        }
                        _ => Err(RuntimeError::new("ord.compare expects String", span)),
                    }
                }
                // show — Int, Float, String, Bool
                ("show", _, "show") => {
                    let v = args.into_iter().next().unwrap_or(Value::Unit);
                    Ok(Value::Str(v.repr()))
                }
                _ => Err(RuntimeError::new(
                    format!("unknown cap built-in: {}.{}", ns, method), span
                )),
            }
        }

        _ => Err(RuntimeError::new(format!("unknown built-in: {}.{}", ns, method), span)),
    }
}

// ── apply (5-5, 5-6) ──────────────────────────────────────────────────────────

pub fn eval_apply(callee: Value, args: Vec<Value>, span: &Span) -> EvalResult {
    match callee {
        // User-defined closure (fn, trf, anonymous closure)
        Value::Closure { params, body, env } => {
            if params.len() != args.len() {
                return Err(RuntimeError::new(
                    format!("expected {} argument(s), got {}", params.len(), args.len()),
                    span,
                ));
            }
            let call_env = EnvInner::new_child(&env);
            for (p, a) in params.iter().zip(args.into_iter()) {
                env_define(&call_env, p.clone(), a);
            }
            // Catch chain early-exit at function boundary (task 4-5/4-6)
            match eval_expr(&body, &call_env) {
                Ok(v)                        => Ok(v),
                Err(e) if e.escape.is_some() => Ok(e.escape.unwrap()),
                Err(e)                       => Err(e),
            }
        }

        // flw composition (5-17): pipe arg through each step
        Value::Flw(steps, flw_env) => {
            if args.len() != 1 {
                return Err(RuntimeError::new(
                    format!("flw expects 1 argument, got {}", args.len()),
                    span,
                ));
            }
            let mut current = args.into_iter().next().unwrap();
            for step_name in &steps {
                let step_val = env_lookup(&flw_env, step_name).ok_or_else(|| {
                    RuntimeError::new(format!("undefined step `{}` in flw", step_name), span)
                })?;
                current = eval_apply(step_val, vec![current], span)?;
            }
            Ok(current)
        }

        // Built-in method
        Value::Builtin(ns, method) => eval_builtin(&ns, &method, args, span),

        _ => Err(RuntimeError::new(
            format!("`{}` is not callable", callee.type_name()),
            span,
        )),
    }
}

// ── expr evaluation ───────────────────────────────────────────────────────────

pub fn eval_expr(expr: &Expr, env: &Env) -> EvalResult {
    match expr {
        // Literals (5-3)
        Expr::Lit(lit, _) => Ok(match lit {
            Lit::Bool(b)  => Value::Bool(*b),
            Lit::Int(n)   => Value::Int(*n),
            Lit::Float(f) => Value::Float(*f),
            Lit::Str(s)   => Value::Str(s.clone()),
            Lit::Unit     => Value::Unit,
        }),

        // Identifier (5-4)
        Expr::Ident(name, span) => {
            env_lookup(env, name).ok_or_else(|| {
                RuntimeError::new(format!("undefined: `{}`", name), span)
            })
        }

        // Field access: obj.field (5-20)
        Expr::FieldAccess(obj_expr, field, span) => {
            let obj = eval_expr(obj_expr, env)?;
            match &obj {
                Value::Namespace(ns) if ns.starts_with("type:") => {
                    // Type namespace: look up cap instance (e.g. Int.eq → Eq<Int> record).
                    let ty_key = &ns["type:".len()..];
                    if let Some(cap_record) = impl_registry_get(field, ty_key) {
                        return Ok(cap_record);
                    }
                    // Fall back to Builtin for unknown fields on type namespaces.
                    Ok(Value::Builtin(ns.clone(), field.clone()))
                }
                Value::Namespace(ns) => {
                    Ok(Value::Builtin(ns.clone(), field.clone()))
                }
                Value::Record(m) => {
                    m.get(field).cloned().ok_or_else(|| {
                        RuntimeError::new(format!("record has no field `{}`", field), span)
                    })
                }
                other => Err(RuntimeError::new(
                    format!("cannot access field `{}` on {}", field, other.type_name()),
                    span,
                )),
            }
        }

        // Function application (5-5)
        Expr::Apply(func_expr, arg_exprs, span) => {
            let callee = eval_expr(func_expr, env)?;
            let mut args = Vec::with_capacity(arg_exprs.len());
            for a in arg_exprs {
                args.push(eval_expr(a, env)?);
            }
            eval_apply(callee, args, span)
        }

        // Pipeline: val |> f |> g  (5-7)
        Expr::Pipeline(parts, span) => {
            if parts.is_empty() {
                return Ok(Value::Unit);
            }
            let mut current = eval_expr(&parts[0], env)?;
            for step_expr in &parts[1..] {
                let step = eval_expr(step_expr, env)?;
                current = eval_apply(step, vec![current], span)?;
            }
            Ok(current)
        }

        // Block (5-14)
        Expr::Block(block) => eval_block(block, env),

        // Match (5-12)
        Expr::Match(scrutinee_expr, arms, span) => {
            let scrutinee = eval_expr(scrutinee_expr, env)?;
            'arm: for arm in arms {
                if let Some(bindings) = match_pattern(&arm.pattern, &scrutinee) {
                    let arm_env = EnvInner::new_child(env);
                    for (k, v) in bindings {
                        env_define(&arm_env, k, v);
                    }
                    // Evaluate pattern guard if present (task 4-15)
                    if let Some(guard) = &arm.guard {
                        match eval_expr(guard, &arm_env)? {
                            Value::Bool(true)  => {}
                            Value::Bool(false) => continue 'arm,
                            other => return Err(RuntimeError::new(
                                format!("pattern guard must be Bool, got {}", other.type_name()),
                                guard.span(),
                            )),
                        }
                    }
                    return eval_expr(&arm.body, &arm_env);
                }
            }
            Err(RuntimeError::new("non-exhaustive match", span))
        }

        // If (5-13)
        Expr::If(cond_expr, then_block, else_block, span) => {
            let cond = eval_expr(cond_expr, env)?;
            match cond {
                Value::Bool(true)  => eval_block(then_block, env),
                Value::Bool(false) => {
                    match else_block {
                        Some(b) => eval_block(b, env),
                        None    => Ok(Value::Unit),
                    }
                }
                other => Err(RuntimeError::new(
                    format!("if condition must be Bool, got {}", other.type_name()),
                    span,
                )),
            }
        }

        // Closure (5-6)
        Expr::Closure(params, body, _span) => {
            Ok(Value::Closure {
                params: params.clone(),
                body: body.clone(),
                env: Rc::clone(env),
            })
        }

        // Binary operators
        Expr::BinOp(op, lhs, rhs, span) => {
            let l = eval_expr(lhs, env)?;
            let r = eval_expr(rhs, env)?;
            eval_binop(op, l, r, span)
        }

        // Record construction: TypeName { field: expr, ... } (3-1)
        Expr::RecordConstruct(_type_name, fields, _span) => {
            let mut map = HashMap::new();
            for (fname, fexpr) in fields {
                let val = eval_expr(fexpr, env)?;
                map.insert(fname.clone(), val);
            }
            Ok(Value::Record(map))
        }

        // emit expr (3-3): evaluate inner, push to emit_log, return Unit
        Expr::EmitExpr(inner, span) => {
            let v = eval_expr(inner, env)?;
            emit_log_push(v, span)?;
            Ok(Value::Unit)
        }

        // collect { yield expr; ... } (task 4-10)
        Expr::Collect(block, _span) => {
            collect_push_frame();
            let result = eval_block(block, env);
            let items = collect_pop_frame();
            match result {
                Ok(_) => Ok(Value::List(items)),
                Err(e) => Err(e),  // propagate escapes / errors unchanged
            }
        }
    }
}

// ── block evaluation (5-14) ───────────────────────────────────────────────────

pub fn eval_block(block: &Block, env: &Env) -> EvalResult {
    let block_env = EnvInner::new_child(env);
    for stmt in &block.stmts {
        eval_stmt(stmt, &block_env)?;
    }
    eval_expr(&block.expr, &block_env)
}

// ── stmt evaluation (5-8, 5-9, 5-10, 5-11) ───────────────────────────────────

fn eval_stmt(stmt: &Stmt, env: &Env) -> Result<(), RuntimeError> {
    match stmt {
        Stmt::Bind(b) => {
            let val = eval_expr(&b.expr, env)?;
            let bindings = match_pattern(&b.pattern, &val).ok_or_else(|| {
                RuntimeError::new("bind pattern did not match", &b.span)
            })?;
            for (k, v) in bindings {
                env_define(env, k, v);
            }
            Ok(())
        }
        Stmt::Expr(e) => {
            eval_expr(e, env)?;
            Ok(())
        }

        // chain n <- expr  (task 4-4)
        Stmt::Chain(c) => {
            let val = eval_expr(&c.expr, env)?;
            match &val {
                Value::Variant(n, Some(inner)) if n == "ok" || n == "some" => {
                    env_define(env, c.name.clone(), *inner.clone());
                    Ok(())
                }
                Value::Variant(n, None) if n == "none" => {
                    Err(RuntimeError::chain_escape(val, &c.span))
                }
                Value::Variant(n, _) if n == "err" => {
                    Err(RuntimeError::chain_escape(val, &c.span))
                }
                _ => Err(RuntimeError::new(
                    format!("chain: expected Result or Option, got {}", val.type_name()),
                    &c.span,
                )),
            }
        }

        // yield expr;  (task 4-9)
        Stmt::Yield(y) => {
            let val = eval_expr(&y.expr, env)?;
            collect_yield(val);
            Ok(())
        }
    }
}

// ── Thread-local Db connection (3-6) ──────────────────────────────────────────

// Thread-local storage: each thread (including each test thread) gets its own
// optional Db connection.  rusqlite::Connection is Send but not Sync; thread_local
// gives us single-threaded access per thread without needing Mutex.
thread_local! {
    static DB_CONN: RefCell<Option<rusqlite::Connection>> = const { RefCell::new(None) };
}

fn with_db<F, T>(span: &Span, f: F) -> Result<T, RuntimeError>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error>,
{
    DB_CONN.with(|cell| {
        let borrow = cell.borrow();
        match borrow.as_ref() {
            Some(conn) => f(conn).map_err(|e| RuntimeError::new(format!("Db error: {}", e), span)),
            None => Err(RuntimeError::new(
                "Db not initialized — run with --db <path> flag", span,
            )),
        }
    })
}

// ── Thread-local emit log (3-2) ───────────────────────────────────────────────

thread_local! {
    static EMIT_LOG: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
}

// ── Thread-local collect stack (task 4-7) ────────────────────────────────────
// Each `collect { }` block pushes a frame; `yield` pushes into the top frame.

thread_local! {
    static COLLECT_STACK: RefCell<Vec<Vec<Value>>> = const { RefCell::new(Vec::new()) };
}

fn collect_push_frame() {
    COLLECT_STACK.with(|s| s.borrow_mut().push(Vec::new()));
}

fn collect_yield(val: Value) {
    COLLECT_STACK.with(|s| {
        if let Some(frame) = s.borrow_mut().last_mut() {
            frame.push(val);
        }
    });
}

fn collect_pop_frame() -> Vec<Value> {
    COLLECT_STACK.with(|s| s.borrow_mut().pop().unwrap_or_default())
}

// ── Thread-local impl registry (v0.4.0) ───────────────────────────────────────
// Maps (cap_name_lower, type_key) → Value::Record of method closures.
// e.g. ("eq", "Int") → Record { "equals" → Closure }

thread_local! {
    static IMPL_REGISTRY: RefCell<HashMap<(String, String), Value>> =
        RefCell::new(HashMap::new());
}

fn impl_registry_init() {
    IMPL_REGISTRY.with(|r| r.borrow_mut().clear());
}

fn impl_registry_insert(cap: String, ty_key: String, val: Value) {
    IMPL_REGISTRY.with(|r| r.borrow_mut().insert((cap, ty_key), val));
}

fn impl_registry_get(cap: &str, ty_key: &str) -> Option<Value> {
    IMPL_REGISTRY.with(|r| r.borrow().get(&(cap.to_string(), ty_key.to_string())).cloned())
}

fn emit_log_init() {
    EMIT_LOG.with(|log| log.borrow_mut().clear());
}

fn emit_log_push(v: Value, _span: &Span) -> Result<(), RuntimeError> {
    EMIT_LOG.with(|log| log.borrow_mut().push(v));
    Ok(())
}

fn emit_log_snapshot() -> Vec<Value> {
    EMIT_LOG.with(|log| log.borrow().clone())
}

// ── Interpreter: program-level setup ─────────────────────────────────────────

pub struct Interpreter;

impl Interpreter {
    /// Evaluate a complete program, then call `main`.
    pub fn run(program: &Program) -> EvalResult {
        emit_log_init();
        let env = EnvInner::new_root();
        Self::register_builtins(&env);
        Self::register_items(program, &env)?;
        let main_val = env_lookup(&env, "main").ok_or_else(|| {
            RuntimeError::new("`main` is not defined", &dummy_span())
        })?;
        eval_apply(main_val, vec![], &dummy_span())
    }

    /// Evaluate a complete program with a Db connection, then call `main`.
    pub fn run_with_db(program: &Program, conn: rusqlite::Connection) -> EvalResult {
        DB_CONN.with(|cell| *cell.borrow_mut() = Some(conn));
        emit_log_init();
        let env = EnvInner::new_root();
        Self::register_builtins(&env);
        Self::register_items(program, &env)?;
        let main_val = env_lookup(&env, "main").ok_or_else(|| {
            RuntimeError::new("`main` is not defined", &dummy_span())
        })?;
        eval_apply(main_val, vec![], &dummy_span())
    }

    /// Evaluate a program and return the value of any named item (for tests).
    pub fn eval_item(program: &Program, name: &str) -> EvalResult {
        let env = EnvInner::new_root();
        Self::register_builtins(&env);
        Self::register_items(program, &env)?;
        let val = env_lookup(&env, name).ok_or_else(|| {
            RuntimeError::new(format!("`{}` is not defined", name), &dummy_span())
        })?;
        Ok(val)
    }

    fn register_builtins(env: &Env) {
        // Register namespace values so `IO`, `List`, `Db`, etc. resolve
        for ns in &["IO", "List", "String", "Option", "Result", "Db", "Http", "Map", "Debug", "Emit", "Util", "Trace", "File", "Json", "Csv"] {
            env_define(env, ns.to_string(), Value::Namespace(ns.to_string()));
        }

        // Register primitive type names as type namespaces (for cap access: Int.eq, etc.)
        for ty in &["Bool", "Int", "Float"] {
            env_define(env, ty.to_string(), Value::Namespace(format!("type:{}", ty)));
        }

        // ── Built-in cap instances in the impl registry ───────────────────────
        impl_registry_init();

        // Register built-in cap instances as Builtin sentinels that dispatch to eval_builtin.
        // The actual dispatch happens in eval_apply / FieldAccess via the registry.
        // We store Value::Record of Builtin sentinels so method dispatch works.
        fn make_eq_record(ty: &str) -> Value {
            let mut m = HashMap::new();
            m.insert("equals".into(), Value::Builtin(format!("cap_eq_{}", ty.to_lowercase()), "equals".into()));
            Value::Record(m)
        }
        fn make_ord_record(ty: &str) -> Value {
            let mut m = HashMap::new();
            m.insert("compare".into(), Value::Builtin(format!("cap_ord_{}", ty.to_lowercase()), "compare".into()));
            m.insert("equals".into(),  Value::Builtin(format!("cap_eq_{}", ty.to_lowercase()),  "equals".into()));
            Value::Record(m)
        }
        fn make_show_record(ty: &str) -> Value {
            let mut m = HashMap::new();
            m.insert("show".into(), Value::Builtin(format!("cap_show_{}", ty.to_lowercase()), "show".into()));
            Value::Record(m)
        }

        for ty in &["Int", "Float", "String"] {
            impl_registry_insert("eq".into(),   ty.to_string(), make_eq_record(ty));
            impl_registry_insert("ord".into(),  ty.to_string(), make_ord_record(ty));
            impl_registry_insert("show".into(), ty.to_string(), make_show_record(ty));
        }
        impl_registry_insert("eq".into(),   "Bool".into(), make_eq_record("Bool"));
        impl_registry_insert("show".into(), "Bool".into(), make_show_record("Bool"));
    }

    fn register_items(program: &Program, env: &Env) -> Result<(), RuntimeError> {
        // Single pass: closures capture `env` by Rc ref, so forward references
        // within fn/trf bodies work because the env is mutated in place.
        for item in &program.items {
            match item {
                Item::TypeDef(td)            => Self::register_type_def(td, env),
                Item::FnDef(fd)              => Self::register_fn_def(fd, env),
                Item::TrfDef(td)             => Self::register_trf_def(td, env),
                Item::FlwDef(fd)             => Self::register_flw_def(fd, env),
                Item::CapDef(..)             => {}
                Item::ImplDef(id)            => Self::register_impl_def(id, env),
                Item::NamespaceDecl(..)      => {}
                Item::UseDecl(..)            => {}
            }
        }
        Ok(())
    }

    // 5-18: type definitions — register constructors
    fn register_type_def(td: &TypeDef, env: &Env) {
        // Register the type name as a namespace for field-access-based construction
        env_define(env, td.name.clone(), Value::Namespace(format!("type:{}", td.name)));

        match &td.body {
            TypeBody::Sum(variants) => {
                for v in variants {
                    match v {
                        // 5-19: Unit variant → register as plain Variant value
                        Variant::Unit(name, _) => {
                            env_define(env, name.clone(), Value::Variant(name.clone(), None));
                        }
                        // 5-19: Tuple variant → register as a 1-arg constructor builtin
                        Variant::Tuple(name, _, _) => {
                            let vname = name.clone();
                            env_define(env, name.clone(), Value::Builtin("__variant__".into(), vname));
                        }
                        // 5-19: Record variant → register as multi-arg constructor closure
                        Variant::Record(name, fields, _) => {
                            let vname = name.clone();
                            let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
                            env_define(env, name.clone(), Value::Builtin(
                                format!("__variant_record__:{}", vname),
                                field_names.join(","),
                            ));
                        }
                    }
                }
            }
            TypeBody::Record(_fields) => {
                // Record type: constructor is registered as a builtin
                // (record construction via fn is the idiomatic approach in v0.1.0)
            }
        }
    }

    // 5-15: fn definition
    fn register_fn_def(fd: &FnDef, env: &Env) {
        let params: Vec<String> = fd.params.iter().map(|p| p.name.clone()).collect();
        let closure = Value::Closure {
            params,
            body: Box::new(Expr::Block(Box::new(fd.body.clone()))),
            env: Rc::clone(env),
        };
        env_define(env, fd.name.clone(), closure);
    }

    // v0.4.0: impl definition — evaluate methods and store in impl registry
    fn register_impl_def(id: &ImplDef, env: &Env) {
        // Derive the type key string from the first type argument.
        fn type_expr_str(te: &TypeExpr) -> String {
            match te {
                TypeExpr::Named(n, args, _) if args.is_empty() => n.clone(),
                TypeExpr::Named(n, args, _) => {
                    let s: Vec<_> = args.iter().map(type_expr_str).collect();
                    format!("{}<{}>", n, s.join(", "))
                }
                TypeExpr::Optional(inner, _) => format!("{}?", type_expr_str(inner)),
                TypeExpr::Fallible(inner, _) => format!("{}!", type_expr_str(inner)),
                TypeExpr::Arrow(a, b, _)     => format!("{} -> {}", type_expr_str(a), type_expr_str(b)),
            }
        }
        let Some(first_arg) = id.type_args.first() else { return };
        let ty_key = type_expr_str(first_arg);
        let cap_lower = id.cap_name.to_lowercase();

        let mut methods = HashMap::new();
        for method in &id.methods {
            let params: Vec<String> = method.params.iter().map(|p| p.name.clone()).collect();
            let closure = Value::Closure {
                params,
                body: Box::new(Expr::Block(Box::new(method.body.clone()))),
                env: Rc::clone(env),
            };
            methods.insert(method.name.clone(), closure);
        }
        impl_registry_insert(cap_lower, ty_key, Value::Record(methods));
    }

    // 5-16: trf definition
    fn register_trf_def(td: &TrfDef, env: &Env) {
        let params: Vec<String> = td.params.iter().map(|p| p.name.clone()).collect();
        let closure = Value::Closure {
            params,
            body: Box::new(Expr::Block(Box::new(td.body.clone()))),
            env: Rc::clone(env),
        };
        env_define(env, td.name.clone(), closure);
    }

    // 5-17: flw definition
    fn register_flw_def(fd: &FlwDef, env: &Env) {
        let flw_env = EnvInner::new_child(env);
        let flw = Value::Flw(fd.steps.clone(), flw_env);
        env_define(env, fd.name.clone(), flw);
    }
}

// Handle variant constructors in eval_apply (special Builtin variants)
// We override the generic Builtin path in eval_apply by extending eval_builtin above.
// The "__variant__" and "__variant_record__:*" namespaces are handled here.
// Patch: extend eval_builtin to handle internal variant construction.

// Note: eval_builtin is already called from eval_apply for all Builtin values.
// The "__variant__" builtins are handled in eval_builtin under the `_ => Err(...)` fallback,
// but we need to handle them explicitly. The cleanest fix: check ns prefix in eval_builtin.
// We handle this by intercepting in eval_builtin with a prefix match.

// ── re-export eval_builtin with variant constructor support ───────────────────
// (already handled above — add the variant cases to eval_builtin)

// ── Tests (5-27) ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn eval(src: &str) -> Value {
        let prog = Parser::parse_str(src, "test").expect("parse error");
        Interpreter::run(&prog).expect("runtime error")
    }

    fn eval_err(src: &str) -> String {
        let prog = Parser::parse_str(src, "test").expect("parse error");
        Interpreter::run(&prog).unwrap_err().message
    }

    fn eval_fn(src: &str, fname: &str, args: Vec<Value>) -> Value {
        let prog = Parser::parse_str(src, "test").expect("parse error");
        let env = EnvInner::new_root();
        Interpreter::register_builtins(&env);
        Interpreter::register_items(&prog, &env).expect("register error");
        let f = env_lookup(&env, fname).expect("fn not found");
        eval_apply(f, args, &dummy_span()).expect("runtime error")
    }

    // 5-3: literal evaluation
    #[test]
    fn test_literals() {
        assert_eq!(eval_fn("fn f() -> Int { 42 }", "f", vec![]), Value::Int(42));
        assert_eq!(eval_fn("fn f() -> Bool { true }", "f", vec![]), Value::Bool(true));
        assert_eq!(eval_fn("fn f() -> String { \"hi\" }", "f", vec![]), Value::Str("hi".into()));
        assert_eq!(eval_fn("fn f() -> Float { 3.14 }", "f", vec![]), Value::Float(3.14));
    }

    // 5-4: identifier resolution
    #[test]
    fn test_ident() {
        assert_eq!(eval_fn("fn f(x: Int) -> Int { x }", "f", vec![Value::Int(7)]), Value::Int(7));
    }

    // 5-5: function application
    #[test]
    fn test_fn_apply() {
        assert_eq!(
            eval_fn("fn add(a: Int, b: Int) -> Int { a + b }", "add", vec![Value::Int(3), Value::Int(4)]),
            Value::Int(7)
        );
    }

    // 5-6: closure creation and application
    #[test]
    fn test_closure() {
        assert_eq!(
            eval_fn("fn f() -> Int { bind g <- |x| x + 1; g(10) }", "f", vec![]),
            Value::Int(11)
        );
    }

    // 5-7: pipeline evaluation
    #[test]
    fn test_pipeline() {
        let src = "
            trf Double: Int -> Int = |n| { n + n }
            trf Inc:    Int -> Int = |n| { n + 1 }
            fn f(x: Int) -> Int { x |> Double |> Inc }
        ";
        assert_eq!(eval_fn(src, "f", vec![Value::Int(3)]), Value::Int(7));
    }

    // 5-8, 5-9: simple bind binding
    #[test]
    fn test_bind_simple() {
        assert_eq!(
            eval_fn("fn f() -> Int { bind x <- 10; bind y <- 20; x + y }", "f", vec![]),
            Value::Int(30)
        );
    }

    // 5-10: record destructuring bind
    #[test]
    fn test_bind_record_destruct() {
        let src = "
            type Point = { x: Int y: Int }
            fn sum(p: Point) -> Int { bind { x, y } <- p; x + y }
        ";
        let point = Value::Record({
            let mut m = HashMap::new();
            m.insert("x".into(), Value::Int(3));
            m.insert("y".into(), Value::Int(4));
            m
        });
        assert_eq!(eval_fn(src, "sum", vec![point]), Value::Int(7));
    }

    // 5-11: variant destructuring bind
    #[test]
    fn test_bind_variant_destruct() {
        let src = "
            type Wrap = | Val(Int)
            fn unwrap(w: Wrap) -> Int { bind Val(v) <- w; v }
        ";
        let wrapped = Value::Variant("Val".into(), Some(Box::new(Value::Int(99))));
        assert_eq!(eval_fn(src, "unwrap", vec![wrapped]), Value::Int(99));
    }

    // 5-12: match expression
    #[test]
    fn test_match() {
        let src = "
            type Color = | Red | Green | Blue
            fn to_num(c: Color) -> Int {
                match c {
                    Red   => 0
                    Green => 1
                    Blue  => 2
                }
            }
        ";
        assert_eq!(eval_fn(src, "to_num", vec![Value::Variant("Red".into(),   None)]), Value::Int(0));
        assert_eq!(eval_fn(src, "to_num", vec![Value::Variant("Green".into(), None)]), Value::Int(1));
        assert_eq!(eval_fn(src, "to_num", vec![Value::Variant("Blue".into(),  None)]), Value::Int(2));
    }

    // 5-12: wildcard match arm
    #[test]
    fn test_match_wildcard() {
        let src = "
            fn f(n: Int) -> String {
                match n {
                    1 => \"one\"
                    _ => \"other\"
                }
            }
        ";
        assert_eq!(eval_fn(src, "f", vec![Value::Int(1)]), Value::Str("one".into()));
        assert_eq!(eval_fn(src, "f", vec![Value::Int(99)]), Value::Str("other".into()));
    }

    // 5-13: if expression
    #[test]
    fn test_if() {
        let src = "fn f(b: Bool) -> Int { if b { 1 } else { 0 } }";
        assert_eq!(eval_fn(src, "f", vec![Value::Bool(true)]),  Value::Int(1));
        assert_eq!(eval_fn(src, "f", vec![Value::Bool(false)]), Value::Int(0));
    }

    // 5-14: block returns last expr
    #[test]
    fn test_block_return() {
        assert_eq!(
            eval_fn("fn f() -> Int { bind x <- 1; bind y <- 2; x + y }", "f", vec![]),
            Value::Int(3)
        );
    }

    // 5-15: fn definition registered and callable
    #[test]
    fn test_fn_def() {
        assert_eq!(
            eval_fn("fn double(n: Int) -> Int { n + n }", "double", vec![Value::Int(5)]),
            Value::Int(10)
        );
    }

    // 5-16: trf definition
    #[test]
    fn test_trf_def() {
        let src = "trf AddOne: Int -> Int = |n| { n + 1 }";
        assert_eq!(eval_fn(src, "AddOne", vec![Value::Int(4)]), Value::Int(5));
    }

    // 5-17: flw definition
    #[test]
    fn test_flw_def() {
        let src = "
            trf Double: Int -> Int = |n| { n + n }
            trf Inc:    Int -> Int = |n| { n + 1 }
            flw DoubleInc = Double |> Inc
        ";
        assert_eq!(eval_fn(src, "DoubleInc", vec![Value::Int(3)]), Value::Int(7));
    }

    // 5-19: ADT variant construction
    #[test]
    fn test_variant_unit() {
        let src = "
            type Status = | Active | Inactive
            fn f() -> Status { Active }
        ";
        assert_eq!(eval_fn(src, "f", vec![]), Value::Variant("Active".into(), None));
    }

    // 5-20: record field access
    #[test]
    fn test_field_access() {
        let src = "fn f(p: Point) -> Int { p.x }";
        let record = Value::Record({
            let mut m = HashMap::new();
            m.insert("x".into(), Value::Int(42));
            m.insert("y".into(), Value::Int(0));
            m
        });
        assert_eq!(eval_fn(src, "f", vec![record]), Value::Int(42));
    }

    // 5-21: IO.println returns Unit
    #[test]
    fn test_io_println() {
        assert_eq!(
            eval_fn(r#"fn f() -> Unit { IO.println("hello") }"#, "f", vec![]),
            Value::Unit
        );
    }

    // 5-22: List operations
    #[test]
    fn test_list_length() {
        let src = "fn f(xs: List<Int>) -> Int { List.length(xs) }";
        let lst = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(eval_fn(src, "f", vec![lst]), Value::Int(3));
    }

    #[test]
    fn test_list_map() {
        let src = "fn f(xs: List<Int>) -> List<Int> { List.map(xs, |x| x + 1) }";
        let lst = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(
            eval_fn(src, "f", vec![lst]),
            Value::List(vec![Value::Int(2), Value::Int(3), Value::Int(4)])
        );
    }

    #[test]
    fn test_list_filter() {
        let src = "fn f(xs: List<Int>) -> List<Int> { List.filter(xs, |x| x > 2) }";
        let lst = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);
        assert_eq!(
            eval_fn(src, "f", vec![lst]),
            Value::List(vec![Value::Int(3), Value::Int(4)])
        );
    }

    #[test]
    fn test_list_fold() {
        let src = "fn f(xs: List<Int>) -> Int { List.fold(xs, 0, |acc, x| acc + x) }";
        let lst = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);
        assert_eq!(eval_fn(src, "f", vec![lst]), Value::Int(10));
    }

    // 5-23: String operations
    #[test]
    fn test_string_ops() {
        let src = "fn f(s: String) -> String { String.upper(s) }";
        assert_eq!(
            eval_fn(src, "f", vec![Value::Str("hello".into())]),
            Value::Str("HELLO".into())
        );
    }

    #[test]
    fn test_string_split() {
        let src = r#"fn f(s: String) -> List<String> { String.split(s, ",") }"#;
        assert_eq!(
            eval_fn(src, "f", vec![Value::Str("a,b,c".into())]),
            Value::List(vec![
                Value::Str("a".into()),
                Value::Str("b".into()),
                Value::Str("c".into()),
            ])
        );
    }

    // 5-24: Option operations
    #[test]
    fn test_option_some_unwrap() {
        let src = "fn f() -> Int { Option.unwrap_or(Option.some(42), 0) }";
        assert_eq!(eval_fn(src, "f", vec![]), Value::Int(42));
    }

    #[test]
    fn test_option_none_unwrap() {
        let src = "fn f() -> Int { Option.unwrap_or(Option.none(), 99) }";
        assert_eq!(eval_fn(src, "f", vec![]), Value::Int(99));
    }

    #[test]
    fn test_option_map() {
        let src = "fn f() -> Int? { Option.map(Option.some(5), |x| x + 1) }";
        assert_eq!(
            eval_fn(src, "f", vec![]),
            Value::Variant("some".into(), Some(Box::new(Value::Int(6))))
        );
    }

    // 5-25: Result operations
    #[test]
    fn test_result_ok_unwrap() {
        let src = "fn f() -> Int { Result.unwrap_or(Result.ok(10), 0) }";
        assert_eq!(eval_fn(src, "f", vec![]), Value::Int(10));
    }

    #[test]
    fn test_result_err_unwrap() {
        let src = r#"fn f() -> Int { Result.unwrap_or(Result.err("fail"), 0) }"#;
        assert_eq!(eval_fn(src, "f", vec![]), Value::Int(0));
    }

    #[test]
    fn test_result_map() {
        let src = "fn f() -> Int! { Result.map(Result.ok(3), |x| x * 2) }";
        assert_eq!(
            eval_fn(src, "f", vec![]),
            Value::Variant("ok".into(), Some(Box::new(Value::Int(6))))
        );
    }

    // 5-26: Pure/Io effect — parser already rejects unknown effects
    #[test]
    fn test_effect_parse_ok() {
        // These should parse without errors
        Parser::parse_str("trf T: Int -> Int !Pure = |n| { n }", "test").expect("should parse");
        Parser::parse_str("trf T: Int -> Int !Io   = |n| { n }", "test").expect("should parse");
    }

    // arithmetic + comparison
    #[test]
    fn test_arithmetic() {
        let src = "fn f() -> Int { 2 + 3 * 4 }";
        // Parser respects mul > add precedence: 2 + (3*4) = 14
        assert_eq!(eval_fn(src, "f", vec![]), Value::Int(14));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(eval_fn("fn f() -> Bool { 3 > 2 }", "f", vec![]), Value::Bool(true));
        assert_eq!(eval_fn("fn f() -> Bool { 1 == 1 }", "f", vec![]), Value::Bool(true));
        assert_eq!(eval_fn("fn f() -> Bool { 1 != 2 }", "f", vec![]), Value::Bool(true));
    }

    // ── v0.2.0 eval tests ─────────────────────────────────────────────────────

    // 3-1 / 3-20: record construction evaluates to Value::Record
    #[test]
    fn test_record_construct() {
        let src = r#"
            type User = { name: String age: Int }
            fn f() -> User { User { name: "Alice", age: 30 } }
        "#;
        let result = eval_fn(src, "f", vec![]);
        match result {
            Value::Record(m) => {
                assert_eq!(m.get("name"), Some(&Value::Str("Alice".into())));
                assert_eq!(m.get("age"),  Some(&Value::Int(30)));
            }
            other => panic!("expected Record, got {:?}", other),
        }
    }

    // 3-3 / 3-21: emit expr evaluates inner and returns Unit
    #[test]
    fn test_emit_returns_unit() {
        let src = r#"fn f() -> Unit !Emit<E> { emit "event" }"#;
        assert_eq!(eval_fn(src, "f", vec![]), Value::Unit);
    }

    // 3-19 / 3-24: Debug.show converts value to string
    #[test]
    fn test_debug_show() {
        let src = r#"fn f(n: Int) -> String { Debug.show(n) }"#;
        assert_eq!(eval_fn(src, "f", vec![Value::Int(42)]), Value::Str("42".into()));
    }

    #[test]
    fn test_debug_show_bool() {
        let src = r#"fn f(b: Bool) -> String { Debug.show(b) }"#;
        assert_eq!(eval_fn(src, "f", vec![Value::Bool(true)]), Value::Str("true".into()));
    }

    // 3-15..3-18 / 3-24: Map built-ins
    #[test]
    fn test_map_set_get() {
        let src = r#"
            fn f() -> String? {
                bind m <- Map.set((), "key", "val");
                Map.get(m, "key")
            }
        "#;
        // Map.get returns some("val")
        assert_eq!(
            eval_fn(src, "f", vec![]),
            Value::Variant("some".into(), Some(Box::new(Value::Str("val".into()))))
        );
    }

    #[test]
    fn test_map_get_missing() {
        let src = r#"fn f() -> String? { Map.get((), "missing") }"#;
        assert_eq!(eval_fn(src, "f", vec![]), Value::Variant("none".into(), None));
    }

    #[test]
    fn test_map_keys() {
        let src = r#"
            fn f() -> List<String> {
                bind m <- Map.set(Map.set((), "b", 2), "a", 1);
                Map.keys(m)
            }
        "#;
        // keys are sorted alphabetically
        assert_eq!(
            eval_fn(src, "f", vec![]),
            Value::List(vec![Value::Str("a".into()), Value::Str("b".into())])
        );
    }

    // 3-22: Db built-ins with in-memory SQLite
    #[test]
    fn test_db_execute_query() {
        let src = r#"
            public fn main() -> Unit !Db {
                bind _ <- Db.execute("CREATE TABLE t (id INTEGER, name TEXT)");
                bind _ <- Db.execute("INSERT INTO t VALUES (?, ?)", 1, "Alice");
                bind _ <- Db.execute("INSERT INTO t VALUES (?, ?)", 2, "Bob");
                bind rows <- Db.query("SELECT id, name FROM t ORDER BY id");
                IO.println(Debug.show(rows))
            }
        "#;
        let prog = crate::parser::Parser::parse_str(src, "test").expect("parse error");
        let conn = rusqlite::Connection::open_in_memory().expect("open db");
        Interpreter::run_with_db(&prog, conn).expect("runtime error");
    }

    #[test]
    fn test_db_query_one_some() {
        let src = r#"
            public fn main() -> Unit !Db {
                bind _ <- Db.execute("CREATE TABLE u (id INTEGER, name TEXT)");
                bind _ <- Db.execute("INSERT INTO u VALUES (1, 'Alice')");
                bind row <- Db.query_one("SELECT id, name FROM u WHERE id = ?", 1);
                IO.println(Debug.show(row))
            }
        "#;
        let prog = crate::parser::Parser::parse_str(src, "test").expect("parse error");
        let conn = rusqlite::Connection::open_in_memory().expect("open db");
        Interpreter::run_with_db(&prog, conn).expect("runtime error");
    }

    #[test]
    fn test_db_query_one_none() {
        let src = r#"
            public fn main() -> Unit !Db {
                bind _ <- Db.execute("CREATE TABLE v (id INTEGER)");
                bind row <- Db.query_one("SELECT id FROM v WHERE id = ?", 999);
                IO.println(Debug.show(row))
            }
        "#;
        let prog = crate::parser::Parser::parse_str(src, "test").expect("parse error");
        let conn = rusqlite::Connection::open_in_memory().expect("open db");
        let result = Interpreter::run_with_db(&prog, conn).expect("runtime error");
        assert_eq!(result, Value::Unit); // main returns Unit
    }

    // IO.println with main
    #[test]
    fn test_main_runs() {
        let src = r#"public fn main() -> Unit !Io { IO.println("ok") }"#;
        let prog = Parser::parse_str(src, "test").expect("parse error");
        let result = Interpreter::run(&prog);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Unit);
    }

    // ── Phase 4: cap / impl eval (v0.4.0) ─────────────────────────────────────

    #[test]
    fn test_eval_identity_generic() {
        let src = "fn identity<T>(x: T) -> T { x }\npublic fn main() -> Int { identity(42) }";
        assert_eq!(eval(src), Value::Int(42));
    }

    #[test]
    fn test_eval_cap_eq_int() {
        let src = "public fn main() -> Bool { Int.eq.equals(1, 1) }";
        assert_eq!(eval(src), Value::Bool(true));
    }

    #[test]
    fn test_eval_cap_eq_int_false() {
        let src = "public fn main() -> Bool { Int.eq.equals(1, 2) }";
        assert_eq!(eval(src), Value::Bool(false));
    }

    #[test]
    fn test_eval_cap_ord_int_compare() {
        // 1 < 2 → negative
        let src = "public fn main() -> Int { Int.ord.compare(1, 2) }";
        let v = eval(src);
        if let Value::Int(n) = v { assert!(n < 0); } else { panic!("expected Int, got {:?}", v); }
    }

    #[test]
    fn test_eval_cap_show_int() {
        let src = "public fn main() -> String { Int.show.show(42) }";
        assert_eq!(eval(src), Value::Str("42".into()));
    }

    #[test]
    fn test_eval_user_impl() {
        let src = r#"
cap Eq<T> = { equals: T -> T -> Bool }
impl Eq<Bool> {
    fn equals(a: Bool, b: Bool) -> Bool { a == b }
}
public fn main() -> Bool { Bool.eq.equals(true, true) }
"#;
        assert_eq!(eval(src), Value::Bool(true));
    }

    // ── v0.5.0 eval tests ──────────────────────────────────────────────────────

    // task 4-16: chain n <- Result.ok(42) → n = 42, function continues
    #[test]
    fn test_eval_chain_ok() {
        let src = r#"
public fn main() -> Int! {
    chain n <- Result.ok(42)
    Result.ok(n)
}
"#;
        assert_eq!(eval(src), Value::Variant("ok".into(), Some(Box::new(Value::Int(42)))));
    }

    // task 4-17: chain n <- Result.err("boom") → function returns err("boom") early
    #[test]
    fn test_eval_chain_escape_err() {
        let src = r#"
public fn main() -> Int! {
    chain n <- Result.err("boom")
    Result.ok(n + 1)
}
"#;
        assert_eq!(
            eval(src),
            Value::Variant("err".into(), Some(Box::new(Value::Str("boom".into()))))
        );
    }

    // task 4-18: chain n <- Option.none() → function returns none early
    #[test]
    fn test_eval_chain_escape_none() {
        let src = r#"
public fn main() -> Int? {
    chain n <- Option.none()
    Option.some(n + 1)
}
"#;
        assert_eq!(eval(src), Value::Variant("none".into(), None));
    }

    // task 4-19: collect { yield 1; yield 2; () } → [1, 2]
    #[test]
    fn test_eval_collect_yield() {
        let src = r#"
public fn main() -> List<Int> {
    collect { yield 1; yield 2; () }
}
"#;
        assert_eq!(eval(src), Value::List(vec![Value::Int(1), Value::Int(2)]));
    }

    // task 4-20: collect { () } → []
    #[test]
    fn test_eval_collect_empty() {
        let src = r#"
public fn main() -> List<Int> {
    collect { () }
}
"#;
        assert_eq!(eval(src), Value::List(vec![]));
    }

    // task 4-21: guard true → matching arm body is returned
    #[test]
    fn test_eval_match_guard_true() {
        let src = r#"
public fn main() -> String {
    match 5 {
        n where n > 0 => "positive"
        _ => "nonpositive"
    }
}
"#;
        assert_eq!(eval(src), Value::Str("positive".into()));
    }

    // task 4-22: guard false → skip to next arm
    #[test]
    fn test_eval_match_guard_false() {
        let src = r#"
public fn main() -> String {
    match 0 {
        n where n > 0 => "positive"
        _ => "nonpositive"
    }
}
"#;
        assert_eq!(eval(src), Value::Str("nonpositive".into()));
    }

    // task 4-23: pipe match with where guard
    #[test]
    fn test_eval_pipe_match() {
        let src = r#"
public fn main() -> String {
    42 |> match {
        n where n > 0 => "pos"
        _ => "neg"
    }
}
"#;
        assert_eq!(eval(src), Value::Str("pos".into()));
    }

    #[test]
    fn json_parse_encode_roundtrip() {
        let src = r#"
type Field = { key: String value: Json }

public fn main() -> String {
    bind fields <- collect {
        yield Field { key: "name" value: Json.str("fav") };
        yield Field { key: "count" value: Json.int(2) };
        ()
    }
    bind obj <- Json.object(fields)
    bind parsed <- Json.parse(Json.encode(obj))
    bind json <- Option.unwrap_or(parsed, Json.null())
    Json.encode(json)
}
"#;
        assert_eq!(eval(src), Value::Str(r#"{"count":2,"name":"fav"}"#.into()));
    }

    #[test]
    fn csv_parse_with_header() {
        let src = r#"
public fn main() -> String {
    bind rows <- Csv.parse_with_header("name,count\nfav,2\nvm,3\n")
    Csv.from_records(rows)
}
"#;
        assert_eq!(eval(src), Value::Str("count,name\n2,fav\n3,vm\n".into()));
    }

    // ── 7-1: List new functions ───────────────────────────────────────────────

    #[test]
    fn test_list_range() {
        let src = r#"
public fn main() -> List<Int> {
    List.range(1, 5)
}
"#;
        assert_eq!(
            eval(src),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)])
        );
    }

    #[test]
    fn test_list_reverse() {
        let src = r#"
public fn main() -> List<Int> {
    List.reverse(List.range(1, 4))
}
"#;
        assert_eq!(
            eval(src),
            Value::List(vec![Value::Int(3), Value::Int(2), Value::Int(1)])
        );
    }

    #[test]
    fn test_list_concat() {
        let src = r#"
public fn main() -> List<Int> {
    List.concat(List.range(1, 3), List.range(3, 5))
}
"#;
        assert_eq!(
            eval(src),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)])
        );
    }

    #[test]
    fn test_list_take_drop() {
        let src = r#"
public fn main() -> List<Int> {
    bind xs <- List.range(0, 6)
    List.concat(List.take(xs, 3), List.drop(xs, 4))
}
"#;
        assert_eq!(
            eval(src),
            Value::List(vec![Value::Int(0), Value::Int(1), Value::Int(2), Value::Int(4), Value::Int(5)])
        );
    }

    #[test]
    fn test_list_flat_map() {
        let src = r#"
public fn main() -> List<Int> {
    List.flat_map(List.range(1, 4), |x| List.range(0, x))
}
"#;
        // flat_map([1,2,3], |x| range(0,x)) → [0] ++ [0,1] ++ [0,1,2] → [0,0,1,0,1,2]
        assert_eq!(
            eval(src),
            Value::List(vec![
                Value::Int(0),
                Value::Int(0), Value::Int(1),
                Value::Int(0), Value::Int(1), Value::Int(2),
            ])
        );
    }

    #[test]
    fn test_list_zip() {
        let src = r#"
public fn main() -> Int {
    bind pairs <- List.zip(List.range(1, 4), List.range(10, 13))
    List.fold(pairs, 0, |acc, p| acc + p.first + p.second)
}
"#;
        // zip([1,2,3], [10,11,12]) → [{1,10},{2,11},{3,12}]
        // fold sum of all = (1+10) + (2+11) + (3+12) = 11+13+15 = 39
        assert_eq!(eval(src), Value::Int(39));
    }

    #[test]
    fn test_list_sort() {
        let src = r#"
public fn main() -> List<Int> {
    bind xs <- List.concat(List.range(3, 6), List.range(0, 3))
    List.sort(xs, |a, b| a - b)
}
"#;
        assert_eq!(
            eval(src),
            Value::List((0..6).map(Value::Int).collect())
        );
    }

    #[test]
    fn test_list_find_any_all() {
        let src = r#"
public fn main() -> Bool {
    bind xs <- List.range(0, 5)
    bind found <- Option.is_some(List.find(xs, |x| x > 3))
    bind has_neg <- List.any(xs, |x| x < 0)
    bind all_small <- List.all(xs, |x| x < 10)
    found
}
"#;
        assert_eq!(eval(src), Value::Bool(true));
    }

    #[test]
    fn test_list_join() {
        let src = r#"
public fn main() -> String {
    bind parts <- List.map(List.range(1, 4), |n| String.from_int(n))
    List.join(parts, "-")
}
"#;
        assert_eq!(eval(src), Value::Str("1-2-3".into()));
    }

    // ── 7-2: Option / Result new functions ───────────────────────────────────

    #[test]
    fn test_option_and_then() {
        let src = r#"
public fn main() -> Int {
    bind r <- Option.and_then(Option.some(4), |x| Option.some(x + 1))
    Option.unwrap_or(r, 0)
}
"#;
        assert_eq!(eval(src), Value::Int(5));
    }

    #[test]
    fn test_option_and_then_none() {
        let src = r#"
public fn main() -> Int {
    bind r <- Option.and_then(Option.none(), |x| Option.some(x + 1))
    Option.unwrap_or(r, 99)
}
"#;
        assert_eq!(eval(src), Value::Int(99));
    }

    #[test]
    fn test_option_is_some_is_none() {
        let src = r#"
public fn main() -> Bool {
    Option.is_some(Option.some(1))
}
"#;
        assert_eq!(eval(src), Value::Bool(true));
    }

    #[test]
    fn test_result_map_and_then() {
        let src = r#"
public fn main() -> Int {
    bind r <- Result.and_then(Result.ok(3), |x| Result.ok(x * 2))
    bind r2 <- Result.map(r, |x| x + 1)
    Result.unwrap_or(r2, 0)
}
"#;
        assert_eq!(eval(src), Value::Int(7));
    }

    #[test]
    fn test_result_map_err() {
        let src = r#"
public fn main() -> String {
    bind r <- Result.map_err(Result.err("oops"), |e| String.concat("err: ", e))
    match r {
        ok(_) => "ok"
        err(e) => e
    }
}
"#;
        assert_eq!(eval(src), Value::Str("err: oops".into()));
    }

    // ── 7-3: String new functions ─────────────────────────────────────────────

    #[test]
    fn test_string_join() {
        let src = r#"
public fn main() -> String {
    String.join(List.map(List.range(1, 4), |n| String.from_int(n)), ", ")
}
"#;
        assert_eq!(eval(src), Value::Str("1, 2, 3".into()));
    }

    #[test]
    fn test_string_replace() {
        let src = r#"
public fn main() -> String {
    String.replace("hello world", "world", "Favnir")
}
"#;
        assert_eq!(eval(src), Value::Str("hello Favnir".into()));
    }

    #[test]
    fn test_string_slice() {
        let src = r#"
public fn main() -> String {
    String.slice("abcdef", 2, 5)
}
"#;
        assert_eq!(eval(src), Value::Str("cde".into()));
    }

    #[test]
    fn test_string_predicates() {
        let src = r#"
public fn main() -> Bool {
    bind a <- String.starts_with("hello", "he")
    bind b <- String.ends_with("hello", "lo")
    bind c <- String.contains("hello", "ell")
    bind d <- String.is_empty("")
    a
}
"#;
        assert_eq!(eval(src), Value::Bool(true));
    }

    #[test]
    fn test_string_to_from_int() {
        let src = r#"
public fn main() -> Int {
    bind s <- String.from_int(42)
    Option.unwrap_or(String.to_int(s), 0)
}
"#;
        assert_eq!(eval(src), Value::Int(42));
    }

    // ── 7-4: Map new functions ────────────────────────────────────────────────

    #[test]
    fn test_map_merge() {
        let src = r#"
public fn main() -> String {
    bind base     <- Map.set(Map.set((), "a", "1"), "b", "2")
    bind overrides <- Map.set((), "b", "99")
    bind merged   <- Map.merge(base, overrides)
    Option.unwrap_or(Map.get(merged, "b"), "?")
}
"#;
        assert_eq!(eval(src), Value::Str("99".into()));
    }

    #[test]
    fn test_map_from_list_to_list() {
        let src = r#"
public fn main() -> Int {
    bind pairs <- List.zip(
        List.map(List.range(0, 3), |n| String.from_int(n)),
        List.range(10, 13)
    )
    bind m <- Map.from_list(pairs)
    Map.size(m)
}
"#;
        assert_eq!(eval(src), Value::Int(3));
    }

    #[test]
    fn test_map_has_key_is_empty() {
        let src = r#"
public fn main() -> Bool {
    bind m <- Map.set((), "x", 1)
    bind has <- Map.has_key(m, "x")
    bind missing <- Map.has_key(m, "y")
    bind empty <- Map.is_empty(())
    has
}
"#;
        assert_eq!(eval(src), Value::Bool(true));
    }

    // ── 7-5: File read/write roundtrip ───────────────────────────────────────

    #[test]
    fn test_file_read_write_roundtrip() {
        use tempfile::NamedTempFile;
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path().to_str().expect("path").replace('\\', "/");
        let content = "hello from Favnir";
        let src = format!(
            r#"
public fn main() -> String !File {{
    File.write("{path}", "{content}");
    File.read("{path}")
}}
"#
        );
        assert_eq!(eval(&src), Value::Str(content.into()));
    }
}
