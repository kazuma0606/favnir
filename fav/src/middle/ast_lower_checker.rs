/// AST → VMValue lowering for checker.fav
///
/// Converts a Rust `ast::Program` into a `crate::value::Value` tree that
/// matches the type definitions in `fav/self/checker.fav`.  The result is
/// passed directly to `VM::run` as the single argument to `check`.
use crate::ast;
use crate::value::Value;
use std::collections::HashMap;

// ── helpers ───────────────────────────────────────────────────────────────────

/// 0-arg variant
#[inline]
fn v0(tag: &str) -> Value {
    Value::Variant(tag.to_string(), None)
}

/// 1-arg variant
#[inline]
fn v1(tag: &str, a: Value) -> Value {
    Value::Variant(tag.to_string(), Some(Box::new(a)))
}

/// 2-arg variant — payload is `{_0, _1}` record
#[inline]
fn v2(tag: &str, a: Value, b: Value) -> Value {
    let mut map = HashMap::new();
    map.insert("_0".to_string(), a);
    map.insert("_1".to_string(), b);
    Value::Variant(tag.to_string(), Some(Box::new(Value::Record(map))))
}

/// 3-arg variant — payload is `{_0, _1, _2}` record
#[inline]
fn v3(tag: &str, a: Value, b: Value, c: Value) -> Value {
    let mut map = HashMap::new();
    map.insert("_0".to_string(), a);
    map.insert("_1".to_string(), b);
    map.insert("_2".to_string(), c);
    Value::Variant(tag.to_string(), Some(Box::new(Value::Record(map))))
}

#[inline]
fn sv(s: &str) -> Value {
    Value::Str(s.to_string())
}

fn vm_record(fields: Vec<(&str, Value)>) -> Value {
    let map: HashMap<String, Value> = fields
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    Value::Record(map)
}

fn vm_list(items: Vec<Value>) -> Value {
    Value::List(items)
}

// ── Lit ───────────────────────────────────────────────────────────────────────

pub fn lower_lit(lit: &ast::Lit) -> Value {
    match lit {
        ast::Lit::Int(n) => v1("LInt", Value::Int(*n)),
        ast::Lit::Float(f) => v1("LFloat", Value::Float(*f)),
        ast::Lit::Str(s) => v1("LStr", sv(s)),
        ast::Lit::Bool(b) => v1("LBool", Value::Bool(*b)),
        ast::Lit::Unit => v0("LUnit"),
    }
}

// ── BinOp ─────────────────────────────────────────────────────────────────────

pub fn lower_binop(op: &ast::BinOp) -> Value {
    match op {
        ast::BinOp::Add => v0("OpAdd"),
        ast::BinOp::Sub => v0("OpSub"),
        ast::BinOp::Mul => v0("OpMul"),
        ast::BinOp::Div => v0("OpDiv"),
        ast::BinOp::Eq => v0("OpEq"),
        ast::BinOp::NotEq => v0("OpNeq"),
        ast::BinOp::Lt => v0("OpLt"),
        ast::BinOp::Gt => v0("OpGt"),
        ast::BinOp::LtEq => v0("OpLtEq"),
        ast::BinOp::GtEq => v0("OpGtEq"),
        ast::BinOp::And => v0("OpAnd"),
        ast::BinOp::Or => v0("OpOr"),
        ast::BinOp::NullCoalesce => v0("OpNullCoalesce"),
    }
}

// ── Pattern ───────────────────────────────────────────────────────────────────

pub fn lower_pat(pat: &ast::Pattern) -> Value {
    match pat {
        ast::Pattern::Wildcard(_) => v0("PWild"),
        ast::Pattern::Bind(name, _) => v1("PVar", sv(name)),
        ast::Pattern::Lit(lit, _) => match lit {
            ast::Lit::Int(n) => v1("PInt", Value::Int(*n)),
            ast::Lit::Float(f) => v1("PFloat", Value::Float(*f)),
            ast::Lit::Str(s) => v1("PStr", sv(s)),
            ast::Lit::Bool(b) => v1("PBool", Value::Bool(*b)),
            ast::Lit::Unit => v0("PUnit"),
        },
        ast::Pattern::Variant(name, None, _) => v1("PVariant", sv(name)),
        ast::Pattern::Variant(name, Some(inner), _) => {
            v2("PVariantP", sv(name), lower_pat(inner))
        }
        ast::Pattern::Record(_, _) => v0("PWild"),
    }
}

// ── TypeExpr ──────────────────────────────────────────────────────────────────

pub fn lower_te(te: &ast::TypeExpr) -> Value {
    match te {
        ast::TypeExpr::Named(name, args, _) => match (name.as_str(), args.as_slice()) {
            ("List", [t]) => v1("TeList", lower_te(t)),
            ("Option", [t]) => v1("TeOption", lower_te(t)),
            ("Result", [ok, err]) => v2("TeResult", lower_te(ok), lower_te(err)),
            ("Map", [k, v]) => v2("TeMap", lower_te(k), lower_te(v)),
            (name, []) => v1("TeSimple", sv(name)),
            // Named with unknown args — use the base name
            (name, _) => v1("TeSimple", sv(name)),
        },
        ast::TypeExpr::Optional(inner, _) => v1("TeOption", lower_te(inner)),
        ast::TypeExpr::Fallible(inner, _) => {
            v2("TeResult", lower_te(inner), v1("TeSimple", sv("String")))
        }
        ast::TypeExpr::Arrow(a, b, _) => v2("TeFn", lower_te(a), lower_te(b)),
        ast::TypeExpr::TrfFn { input, output, .. } => v2("TeFn", lower_te(input), lower_te(output)),
    }
}

// ── TypeExpr → String (for stage scheme registration) ─────────────────────────

fn te_to_string(te: &ast::TypeExpr) -> String {
    match te {
        ast::TypeExpr::Named(name, args, _) => match (name.as_str(), args.as_slice()) {
            ("List", [t]) => format!("List<{}>", te_to_string(t)),
            ("Option", [t]) => format!("Option<{}>", te_to_string(t)),
            ("Result", [ok, err]) => format!("Result<{}, {}>", te_to_string(ok), te_to_string(err)),
            ("Map", [k, v]) => format!("Map<{}, {}>", te_to_string(k), te_to_string(v)),
            (name, []) => name.to_string(),
            (name, args) => {
                let args_str: Vec<String> = args.iter().map(te_to_string).collect();
                format!("{}<{}>", name, args_str.join(", "))
            }
        },
        ast::TypeExpr::Optional(inner, _) => format!("Option<{}>", te_to_string(inner)),
        ast::TypeExpr::Fallible(inner, _) => format!("Result<{}, String>", te_to_string(inner)),
        ast::TypeExpr::Arrow(a, b, _) => format!("{} -> {}", te_to_string(a), te_to_string(b)),
        ast::TypeExpr::TrfFn { input, output, .. } => {
            format!("{} -> {}", te_to_string(input), te_to_string(output))
        }
    }
}

// ── TrfDef (stage) → IStage ────────────────────────────────────────────────────

fn lower_trf_def(td: &ast::TrfDef) -> Value {
    let params: Vec<Value> = td.params.iter().map(lower_param).collect();
    let body = lower_block(&td.body);
    vm_record(vec![
        ("name", sv(&td.name)),
        ("input_ty_str", sv(&te_to_string(&td.input_ty))),
        ("output_ty_str", sv(&te_to_string(&td.output_ty))),
        ("params", vm_list(params)),
        ("body", body),
    ])
}

// ── FlwDef → ISeq ──────────────────────────────────────────────────────────────

fn lower_flw_step(step: &ast::FlwStep) -> Value {
    match step {
        ast::FlwStep::Stage(name) => v1("SStage", sv(name)),
        ast::FlwStep::Par(names) => {
            let ns: Vec<Value> = names.iter().map(|n| sv(n)).collect();
            v1("SPar", vm_list(ns))
        }
    }
}

fn lower_flw_def(fd: &ast::FlwDef) -> Value {
    let steps: Vec<Value> = fd.steps.iter().map(lower_flw_step).collect();
    let ctx_param = match &fd.ctx_param {
        Some(p) => Value::Str(p.clone()),
        None => Value::Str(String::new()),
    };
    vm_record(vec![
        ("name", sv(&fd.name)),
        ("stages", vm_list(steps)),
        ("ctx_param", ctx_param),
    ])
}

// ── EArgList chain ────────────────────────────────────────────────────────────

fn lower_arg_list(args: &[ast::Expr]) -> Value {
    args.iter()
        .rev()
        .fold(v0("EArgNil"), |acc, arg| v2("EArgList", lower_expr(arg), acc))
}

// ── EArm chain ────────────────────────────────────────────────────────────────

fn lower_arms(arms: &[ast::MatchArm]) -> Value {
    arms.iter()
        .rev()
        .fold(v0("EArmNil"), |acc, arm| {
            v3("EArm", lower_pat(&arm.pattern), lower_expr(&arm.body), acc)
        })
}

// ── EField chain ──────────────────────────────────────────────────────────────

fn lower_field_list(fields: &[(String, ast::Expr)]) -> Value {
    fields.iter().rev().fold(v0("EFieldNil"), |acc, (name, expr)| {
        v3("EField", sv(name), lower_expr(expr), acc)
    })
}

// ── Block / stmts ─────────────────────────────────────────────────────────────

pub fn lower_block(block: &ast::Block) -> Value {
    lower_stmts_and_tail(&block.stmts, &block.expr)
}

fn lower_stmts_and_tail(stmts: &[ast::Stmt], tail: &ast::Expr) -> Value {
    // Iterative build: process stmts right-to-left, wrapping around acc.
    let mut acc = lower_expr(tail);
    for stmt in stmts.iter().rev() {
        acc = match stmt {
            ast::Stmt::Bind(b) => {
                let name = match &b.pattern {
                    ast::Pattern::Bind(n, _) => n.clone(),
                    _ => "_".to_string(),
                };
                v3("EBind", sv(&name), lower_expr(&b.expr), acc)
            }
            ast::Stmt::Expr(e) => v2("EBlock", lower_expr(e), acc),
            ast::Stmt::Chain(c) => v3("EChain", sv(&c.name), lower_expr(&c.expr), acc),
            // Yield / ForIn — skip
            _ => acc,
        };
    }
    acc
}

// ── Apply → ECall ─────────────────────────────────────────────────────────────

fn lower_apply(func: &ast::Expr, args: &[ast::Expr]) -> Value {
    match func {
        ast::Expr::Ident(name, _) => {
            if let Some(dot) = name.find('.') {
                let ns = &name[..dot];
                let fname = &name[dot + 1..];
                v3("ECall", sv(ns), sv(fname), lower_arg_list(args))
            } else {
                v3("ECall", sv(""), sv(name), lower_arg_list(args))
            }
        }
        ast::Expr::FieldAccess(inner, field, _) => match inner.as_ref() {
            ast::Expr::Ident(ns, _) => v3("ECall", sv(ns), sv(field), lower_arg_list(args)),
            ast::Expr::FieldAccess(_, cap_name, _) => {
                // ctx.cap.method(args) → ECall("AppCtx.{cap_name}", method, args)
                let ns = format!("AppCtx.{}", cap_name);
                v3("ECall", sv(&ns), sv(field), lower_arg_list(args))
            }
            _ => v3("ECall", sv(""), sv(field), lower_arg_list(args)),
        },
        _ => v3("ECall", sv(""), sv("_unsupported_"), lower_arg_list(args)),
    }
}

// ── Pipeline ──────────────────────────────────────────────────────────────────

fn lower_pipeline(steps: &[ast::Expr]) -> Value {
    if steps.is_empty() {
        return v1("EVar", sv("_unsupported_"));
    }
    steps[1..].iter().fold(lower_expr(&steps[0]), |acc, step| {
        match step {
            ast::Expr::Ident(name, _) => {
                v3("ECall", sv(""), sv(name), v2("EArgList", acc, v0("EArgNil")))
            }
            ast::Expr::Apply(func, extra_args, _) => {
                // f(b, c) in pipeline becomes f(acc, b, c)
                // Build: EArgList(acc, lower_arg_list(extra_args))
                let rest = lower_arg_list(extra_args);
                let arglist = v2("EArgList", acc, rest);
                match func.as_ref() {
                    ast::Expr::Ident(name, _) => v3("ECall", sv(""), sv(name), arglist),
                    ast::Expr::FieldAccess(obj, fname, _) => match obj.as_ref() {
                        ast::Expr::Ident(ns, _) => v3("ECall", sv(ns), sv(fname), arglist),
                        _ => v3("ECall", sv(""), sv(fname), arglist),
                    },
                    _ => v1("EVar", sv("_unsupported_")),
                }
            }
            _ => v1("EVar", sv("_unsupported_")),
        }
    })
}

// ── Expr ──────────────────────────────────────────────────────────────────────

pub fn lower_expr(expr: &ast::Expr) -> Value {
    match expr {
        ast::Expr::Lit(lit, _) => v1("ELit", lower_lit(lit)),
        ast::Expr::Ident(name, _) => v1("EVar", sv(name)),
        ast::Expr::BinOp(op, l, r, _) => {
            v3("EBinOp", lower_binop(op), lower_expr(l), lower_expr(r))
        }
        ast::Expr::FieldAccess(inner, field, _) => {
            v2("EAccess", lower_expr(inner), sv(field))
        }
        ast::Expr::Closure(params, body, _) => {
            // Curry multi-param closures: |x, y| body → ELambda("x", ELambda("y", body))
            let lowered_body = lower_expr(body);
            if params.len() <= 1 {
                let param = params.first().map(|s| s.as_str()).unwrap_or("_");
                v2("ELambda", sv(param), lowered_body)
            } else {
                // fold from the right: last param wraps body first
                let mut result = lowered_body;
                for param in params.iter().rev() {
                    result = v2("ELambda", sv(param.as_str()), result);
                }
                result
            }
        }
        ast::Expr::RecordConstruct(name, fields, _) => {
            v2("ERecordLit", sv(name), lower_field_list(fields))
        }
        ast::Expr::If(cond, then_block, else_opt, _) => {
            let then_val = lower_block(then_block);
            let else_val = match else_opt {
                Some(b) => lower_block(b),
                None => v1("ELit", lower_lit(&ast::Lit::Unit)),
            };
            v3("EIf", lower_expr(cond), then_val, else_val)
        }
        ast::Expr::Match(scrutinee, arms, _) => {
            v2("EMatch", lower_expr(scrutinee), lower_arms(arms))
        }
        ast::Expr::Block(block) => lower_block(block),
        ast::Expr::Apply(func, args, _) => lower_apply(func, args),
        ast::Expr::Pipeline(steps, _) => lower_pipeline(steps),
        ast::Expr::Question(inner, _) => v1("EQuestion", lower_expr(inner)),
        // v13.3.0: collect { body } → ECollect(body); infer_hm returns "Unknown"
        ast::Expr::Collect(block, _) => v1("ECollect", lower_block(block)),
        ast::Expr::FString(parts, _) => lower_fstring(parts),
        // Fallbacks for TypeApply, AssertMatches, EmitExpr
        _ => v1("EVar", sv("_unsupported_")),
    }
}

fn lower_fstring_part(part: &ast::FStringPart) -> Value {
    match part {
        ast::FStringPart::Lit(s) => v1("ELit", v1("LStr", sv(s))),
        ast::FStringPart::Expr(expr) => lower_expr(expr),
    }
}

fn lower_fstring(parts: &[ast::FStringPart]) -> Value {
    if parts.is_empty() {
        return v1("ELit", v1("LStr", sv("")));
    }
    let lowered: Vec<Value> = parts.iter().map(lower_fstring_part).collect();
    lowered
        .into_iter()
        .reduce(|acc, next| {
            v3(
                "ECall",
                sv("String"),
                sv("concat"),
                v2("EArgList", acc, v2("EArgList", next, v0("EArgNil"))),
            )
        })
        .unwrap()
}

// ── Param ─────────────────────────────────────────────────────────────────────

fn lower_param(p: &ast::Param) -> Value {
    vm_record(vec![("name", sv(&p.name)), ("ty", lower_te(&p.ty))])
}

// ── FnDef ─────────────────────────────────────────────────────────────────────

fn effect_to_str(e: &ast::Effect) -> String {
    match e {
        ast::Effect::Pure => String::new(),
        ast::Effect::Io => "IO".to_string(),
        ast::Effect::Db | ast::Effect::DbRead | ast::Effect::DbWrite | ast::Effect::DbAdmin => {
            "DB".to_string()
        }
        ast::Effect::Network => "Network".to_string(),
        ast::Effect::Http => "Http".to_string(),
        ast::Effect::Llm => "Llm".to_string(),
        ast::Effect::Snowflake => "Snowflake".to_string(),
        ast::Effect::Gcp => "Gcp".to_string(),
        ast::Effect::Stream => "Stream".to_string(),
        ast::Effect::Postgres => "Postgres".to_string(),
        ast::Effect::AzureDb => "AzureDb".to_string(),
        ast::Effect::AzureStorage => "AzureStorage".to_string(),
        ast::Effect::Rpc => "Rpc".to_string(),
        ast::Effect::File => "File".to_string(),
        ast::Effect::Checkpoint => "Checkpoint".to_string(),
        ast::Effect::Trace => "Trace".to_string(),
        ast::Effect::Emit(s) => format!("Emit<{}>", s),
        ast::Effect::EmitUnion(vs) => format!("Emit<{}>", vs.join("|")),
        ast::Effect::Unknown(s) => s.clone(),
    }
}

fn lower_fn_def(fd: &ast::FnDef) -> Value {
    let is_public = matches!(fd.visibility, Some(ast::Visibility::Public));
    let effects: Vec<Value> = fd
        .effects
        .iter()
        .filter(|e| !matches!(e, ast::Effect::Pure))
        .map(|e| sv(&effect_to_str(e)))
        .collect();
    let params: Vec<Value> = fd.params.iter().map(lower_param).collect();
    let ret = fd
        .return_ty
        .as_ref()
        .map(lower_te)
        .unwrap_or_else(|| v1("TeSimple", sv("Unit")));
    let body = lower_block(&fd.body);
    vm_record(vec![
        ("is_public", Value::Bool(is_public)),
        ("name", sv(&fd.name)),
        ("effects", vm_list(effects)),
        ("params", vm_list(params)),
        ("ret", ret),
        ("body", body),
    ])
}

// ── TypeDef ───────────────────────────────────────────────────────────────────

fn lower_type_def(td: &ast::TypeDef) -> Value {
    let (is_record, fields, variants) = match &td.body {
        ast::TypeBody::Record(record_fields) => {
            let fields: Vec<Value> = record_fields
                .iter()
                .map(|f| vm_record(vec![("name", sv(&f.name)), ("ty", lower_te(&f.ty))]))
                .collect();
            (true, fields, vec![])
        }
        ast::TypeBody::Sum(variants) => {
            let variant_vals: Vec<Value> = variants
                .iter()
                .map(|v| {
                    let payload = match v {
                        ast::Variant::Unit(_, _) => v0("none"),
                        ast::Variant::Tuple(_, tys, _) => {
                            if tys.len() <= 1 {
                                let te = tys.first().map(lower_te).unwrap_or_else(|| v1("TeSimple", sv("Unit")));
                                v1("some", te)
                            } else {
                                // Multi-field variant: encode all field types as a semicolon-separated
                                // params string so that infer_call_user counts arity correctly.
                                let params: Vec<String> = tys.iter().map(te_to_string).collect();
                                v1("some", v1("TeSimple", sv(&params.join(";"))))
                            }
                        }
                        ast::Variant::Record(_, fields, _) => {
                            // Encode record variant payload as a record TypeExpr (best-effort)
                            let _ = fields;
                            v0("none")
                        }
                    };
                    vm_record(vec![("name", sv(v.name())), ("payload", payload)])
                })
                .collect();
            (false, vec![], variant_vals)
        }
        ast::TypeBody::Alias(_te) => {
            // Type alias — treat as opaque record-like type
            (true, vec![], vec![])
        }
        ast::TypeBody::Wrapper(_inner) => {
            // Handled separately via lower_item → IWrapper
            (false, vec![], vec![])
        }
    };
    let type_params: Vec<Value> = td.type_params.iter().map(|s| sv(s)).collect();
    vm_record(vec![
        ("name", sv(&td.name)),
        ("is_record", Value::Bool(is_record)),
        ("type_params", vm_list(type_params)),
        ("variants", vm_list(variants)),
        ("fields", vm_list(fields)),
    ])
}

// ── WrapperDef ────────────────────────────────────────────────────────────────

fn lower_wrapper_def(td: &ast::TypeDef, inner: &ast::TypeExpr) -> Value {
    let inner_str = match inner {
        ast::TypeExpr::Named(name, _, _) => name.clone(),
        ast::TypeExpr::Optional(inner, _) => {
            format!("Option<{}>", match inner.as_ref() {
                ast::TypeExpr::Named(n, _, _) => n.clone(),
                _ => "Unknown".to_string(),
            })
        }
        _ => "Unknown".to_string(),
    };
    let has_where = !td.invariants.is_empty();
    let with_impls: Vec<Value> = td.with_interfaces.iter().map(|s| sv(s)).collect();
    vm_record(vec![
        ("name", sv(&td.name)),
        ("inner", sv(&inner_str)),
        ("has_where", Value::Bool(has_where)),
        ("with_impls", vm_list(with_impls)),
    ])
}

// ── TestDef ───────────────────────────────────────────────────────────────────

fn lower_test_def(td: &ast::TestDef) -> Value {
    vm_record(vec![("name", sv(&td.name)), ("body", lower_block(&td.body))])
}

// ── Item ──────────────────────────────────────────────────────────────────────

fn lower_interface_decl(decl: &ast::InterfaceDecl) -> Value {
    vm_record(vec![("name", sv(&decl.name))])
}

fn lower_impl_decl(decl: &ast::InterfaceImplDecl) -> Value {
    let iface_names: Vec<Value> = decl.interface_names.iter().map(|s| sv(s)).collect();
    vm_record(vec![
        ("interface_names", vm_list(iface_names)),
        ("type_name", sv(&decl.type_name)),
    ])
}

fn lower_item(item: &ast::Item) -> Option<Value> {
    match item {
        ast::Item::FnDef(fd) => Some(v1("IFn", lower_fn_def(fd))),
        ast::Item::TypeDef(td) => {
            match &td.body {
                ast::TypeBody::Wrapper(inner) => Some(v1("IWrapper", lower_wrapper_def(td, inner))),
                _ => Some(v1("IType", lower_type_def(td))),
            }
        }
        ast::Item::TestDef(td) => Some(v1("ITest", lower_test_def(td))),
        ast::Item::InterfaceDecl(d) => Some(v1("IInterface", lower_interface_decl(d))),
        ast::Item::InterfaceImplDecl(d) => Some(v1("IImpl", lower_impl_decl(d))),
        ast::Item::TrfDef(td) => Some(v1("IStage", lower_trf_def(td))),
        ast::Item::FlwDef(fd) => Some(v1("ISeq", lower_flw_def(fd))),
        _ => None,
    }
}

// ── Program ───────────────────────────────────────────────────────────────────

pub fn lower_program(prog: &ast::Program) -> Value {
    let items: Vec<Value> = prog.items.iter().filter_map(lower_item).collect();
    vm_record(vec![("items", vm_list(items))])
}
