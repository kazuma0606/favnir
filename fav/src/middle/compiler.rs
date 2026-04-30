use std::collections::{HashMap, HashSet};

use crate::ast::{
    Block, Expr, FieldPattern, FlwDef, Item, Lit, MatchArm, Pattern, Program, Stmt, TypeBody,
    TypeDef, TypeExpr,
};
use super::checker::Type;
use super::ir::{IRArm, IRExpr, IRFnDef, IRGlobal, IRGlobalKind, IRPattern, IRProgram, IRStmt};

#[derive(Debug, Default)]
pub struct CompileCtx {
    pub locals: Vec<HashMap<String, u16>>,
    pub globals: HashMap<String, u16>,
    pub next_slot: u16,
    pub next_global_idx: u16,
    pub next_fn_idx: usize,
    pub lifted_globals: Vec<IRGlobal>,
    pub lifted_fns: Vec<IRFnDef>,
    anon_counter: u16,
    closure_counter: u16,
}

impl CompileCtx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.locals.pop();
    }

    pub fn define_local(&mut self, name: impl Into<String>) -> u16 {
        let slot = self.next_slot;
        self.next_slot = self.next_slot.saturating_add(1);
        if self.locals.is_empty() {
            self.push_scope();
        }
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.into(), slot);
        }
        slot
    }

    pub fn resolve_local(&self, name: &str) -> Option<u16> {
        for scope in self.locals.iter().rev() {
            if let Some(slot) = scope.get(name) {
                return Some(*slot);
            }
        }
        None
    }

    pub fn resolve_global(&self, name: &str) -> Option<u16> {
        self.globals.get(name).copied()
    }

    fn define_pattern_slot(&mut self) -> u16 {
        let name = format!("$pat{}", self.anon_counter);
        self.anon_counter = self.anon_counter.saturating_add(1);
        self.define_local(name)
    }
}

pub fn compile_program(program: &Program) -> IRProgram {
    let mut ctx = CompileCtx::new();
    let mut globals = Vec::new();
    let mut fns = Vec::new();
    let mut next_fn_idx = 0usize;

    for item in &program.items {
        match item {
            Item::FnDef(fd) => {
                let idx = globals.len() as u16;
                ctx.globals.insert(fd.name.clone(), idx);
                globals.push(IRGlobal {
                    name: fd.name.clone(),
                    kind: IRGlobalKind::Fn(next_fn_idx),
                });
                next_fn_idx += 1;
            }
            Item::TrfDef(td) => {
                let idx = globals.len() as u16;
                ctx.globals.insert(td.name.clone(), idx);
                globals.push(IRGlobal {
                    name: td.name.clone(),
                    kind: IRGlobalKind::Fn(next_fn_idx),
                });
                next_fn_idx += 1;
            }
            Item::FlwDef(FlwDef { name, .. }) => {
                let idx = globals.len() as u16;
                ctx.globals.insert(name.clone(), idx);
                globals.push(IRGlobal {
                    name: name.clone(),
                    kind: IRGlobalKind::Fn(next_fn_idx),
                });
                next_fn_idx += 1;
            }
            Item::TypeDef(TypeDef { body: TypeBody::Sum(variants), .. }) => {
                for variant in variants {
                    let idx = globals.len() as u16;
                    let name = variant.name().to_string();
                    ctx.globals.insert(name.clone(), idx);
                    globals.push(IRGlobal {
                        name,
                        kind: IRGlobalKind::VariantCtor,
                    });
                }
            }
            // TestDef: reserve a function index so closures inside compile correctly.
            // Test functions are not added to globals (user code can't call them).
            Item::TestDef(_) => {
                next_fn_idx += 1;
            }
            _ => {}
        }
    }

    // 組み込み名前空間をグローバルテーブルに登録する（ユーザー定義名と衝突しない場合のみ）
    for ns in &[
        "IO", "Debug", "Result", "Option",
        "Int", "Float", "Bool", "String",
        "List", "Map", "Trace", "Emit", "File", "Json", "Csv", "Db", "Http",
        // test assertion builtins (callable without namespace prefix)
        "assert", "assert_eq", "assert_ne",
    ] {
        if !ctx.globals.contains_key(*ns) {
            let idx = globals.len() as u16;
            ctx.globals.insert(ns.to_string(), idx);
            globals.push(IRGlobal { name: ns.to_string(), kind: IRGlobalKind::Builtin });
        }
    }

    ctx.next_global_idx = globals.len() as u16;
    ctx.next_fn_idx = next_fn_idx;

    for item in &program.items {
        match item {
            Item::FnDef(fd) => fns.push(compile_fn_def(
                &fd.name,
                &fd.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
                &fd.effects,
                &fd.return_ty,
                &fd.body,
                &mut ctx,
            )),
            Item::TrfDef(td) => fns.push(compile_fn_def(
                &td.name,
                &td.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
                &td.effects,
                &td.output_ty,
                &td.body,
                &mut ctx,
            )),
            Item::FlwDef(fd) => fns.push(compile_flw_def(fd, &mut ctx)),
            Item::TestDef(td) => {
                let unit_ty = crate::ast::TypeExpr::Named(
                    "Unit".into(), vec![], crate::frontend::lexer::Span::dummy());
                fns.push(compile_fn_def(
                    &format!("$test:{}", td.name),
                    &[],
                    &[],
                    &unit_ty,
                    &td.body,
                    &mut ctx,
                ))
            }
            _ => {}
        }
    }

    globals.extend(ctx.lifted_globals.clone());
    fns.extend(ctx.lifted_fns.clone());

    IRProgram { globals, fns }
}

fn compile_flw_def(fd: &FlwDef, ctx: &mut CompileCtx) -> IRFnDef {
    let saved_next = ctx.next_slot;
    let saved_anon = ctx.anon_counter;
    let saved_locals = std::mem::take(&mut ctx.locals);

    ctx.next_slot = 0;
    ctx.anon_counter = 0;
    ctx.push_scope();
    let input_slot = ctx.define_local("$input");

    let mut current = IRExpr::Local(input_slot, Type::Unknown);
    for step in &fd.steps {
        let callee = if let Some(global_idx) = ctx.resolve_global(step) {
            IRExpr::Global(global_idx, Type::Unknown)
        } else {
            IRExpr::Global(u16::MAX, Type::Unknown)
        };
        current = IRExpr::Call(Box::new(callee), vec![current], Type::Unknown);
    }

    let local_count = ctx.next_slot as usize;
    ctx.locals = saved_locals;
    ctx.next_slot = saved_next;
    ctx.anon_counter = saved_anon;

    IRFnDef {
        name: fd.name.clone(),
        param_count: 1,
        local_count,
        effects: Vec::new(),
        return_ty: Type::Unknown,
        body: current,
    }
}

fn compile_fn_def(
    name: &str,
    params: &[String],
    effects: &[crate::ast::Effect],
    return_ty: &TypeExpr,
    body: &Block,
    ctx: &mut CompileCtx,
) -> IRFnDef {
    let saved_next = ctx.next_slot;
    let saved_anon = ctx.anon_counter;
    let saved_locals = std::mem::take(&mut ctx.locals);

    ctx.next_slot = 0;
    ctx.anon_counter = 0;
    ctx.push_scope();
    for param in params {
        ctx.define_local(param.clone());
    }
    let body_ir = compile_block(body, ctx);
    let local_count = ctx.next_slot as usize;

    ctx.locals = saved_locals;
    ctx.next_slot = saved_next;
    ctx.anon_counter = saved_anon;

    IRFnDef {
        name: name.to_string(),
        param_count: params.len(),
        local_count,
        effects: effects.to_vec(),
        return_ty: lower_type_expr(return_ty),
        body: body_ir,
    }
}

fn compile_block(block: &Block, ctx: &mut CompileCtx) -> IRExpr {
    ctx.push_scope();
    let mut stmts = Vec::new();
    for stmt in &block.stmts {
        stmts.push(compile_stmt(stmt, ctx));
    }
    let tail = compile_expr(&block.expr, ctx);
    ctx.pop_scope();
    let ty = tail.ty().clone();
    IRExpr::Block(stmts, Box::new(tail), ty)
}

fn pattern_binds(pattern: &Pattern, out: &mut HashSet<String>) {
    match pattern {
        Pattern::Bind(name, _) => {
            out.insert(name.clone());
        }
        Pattern::Variant(_, Some(inner), _) => pattern_binds(inner, out),
        Pattern::Variant(_, None, _) | Pattern::Wildcard(_) | Pattern::Lit(_, _) => {}
        Pattern::Record(fields, _) => {
            for field in fields {
                if let Some(inner) = &field.pattern {
                    pattern_binds(inner, out);
                } else {
                    out.insert(field.name.clone());
                }
            }
        }
    }
}

fn collect_free_vars_expr(expr: &Expr, bound: &mut HashSet<String>, free: &mut HashSet<String>) {
    match expr {
        Expr::Lit(_, _) => {}
        Expr::Ident(name, _) => {
            if !bound.contains(name) {
                free.insert(name.clone());
            }
        }
        Expr::Pipeline(parts, _) => {
            for part in parts {
                collect_free_vars_expr(part, bound, free);
            }
        }
        Expr::Apply(callee, args, _) => {
            collect_free_vars_expr(callee, bound, free);
            for arg in args {
                collect_free_vars_expr(arg, bound, free);
            }
        }
        Expr::FieldAccess(obj, _, _) => collect_free_vars_expr(obj, bound, free),
        Expr::Block(block) => collect_free_vars_block(block, bound, free),
        Expr::Match(subject, arms, _) => {
            collect_free_vars_expr(subject, bound, free);
            for arm in arms {
                let mut arm_bound = bound.clone();
                pattern_binds(&arm.pattern, &mut arm_bound);
                if let Some(guard) = &arm.guard {
                    collect_free_vars_expr(guard, &mut arm_bound, free);
                }
                collect_free_vars_expr(&arm.body, &mut arm_bound, free);
            }
        }
        Expr::Collect(block, _) => collect_free_vars_block(block, bound, free),
        Expr::If(cond, then_block, else_block, _) => {
            collect_free_vars_expr(cond, bound, free);
            collect_free_vars_block(then_block, bound, free);
            if let Some(else_block) = else_block {
                collect_free_vars_block(else_block, bound, free);
            }
        }
        Expr::Closure(params, body, _) => {
            let mut inner_bound = bound.clone();
            for param in params {
                inner_bound.insert(param.clone());
            }
            collect_free_vars_expr(body, &mut inner_bound, free);
        }
        Expr::BinOp(_, left, right, _) => {
            collect_free_vars_expr(left, bound, free);
            collect_free_vars_expr(right, bound, free);
        }
        Expr::RecordConstruct(_, fields, _) => {
            for (_, expr) in fields {
                collect_free_vars_expr(expr, bound, free);
            }
        }
        Expr::EmitExpr(inner, _) => collect_free_vars_expr(inner, bound, free),
    }
}

fn collect_free_vars_block(block: &Block, bound: &mut HashSet<String>, free: &mut HashSet<String>) {
    let mut local_bound = bound.clone();
    for stmt in &block.stmts {
        match stmt {
            Stmt::Bind(bind) => {
                collect_free_vars_expr(&bind.expr, &mut local_bound, free);
                pattern_binds(&bind.pattern, &mut local_bound);
            }
            Stmt::Chain(chain) => {
                if !local_bound.contains(&chain.name) {
                    free.insert(chain.name.clone());
                }
                collect_free_vars_expr(&chain.expr, &mut local_bound, free);
            }
            Stmt::Yield(yield_stmt) => {
                collect_free_vars_expr(&yield_stmt.expr, &mut local_bound, free);
            }
            Stmt::Expr(expr) => collect_free_vars_expr(expr, &mut local_bound, free),
        }
    }
    collect_free_vars_expr(&block.expr, &mut local_bound, free);
}

pub fn compile_expr(expr: &Expr, ctx: &mut CompileCtx) -> IRExpr {
    match expr {
        Expr::Lit(lit, _) => IRExpr::Lit(lit.clone(), lit_type(lit)),
        Expr::Ident(name, _) => {
            if let Some(slot) = ctx.resolve_local(name) {
                IRExpr::Local(slot, Type::Unknown)
            } else if let Some(idx) = ctx.resolve_global(name) {
                IRExpr::Global(idx, Type::Unknown)
            } else {
                IRExpr::Global(u16::MAX, Type::Unknown)
            }
        }
        Expr::Pipeline(parts, _) => {
            let mut it = parts.iter();
            let first = compile_expr(it.next().expect("pipeline has at least one part"), ctx);
            it.fold(first, |acc, part| {
                let callee = compile_expr(part, ctx);
                IRExpr::Call(Box::new(callee), vec![acc], Type::Unknown)
            })
        }
        Expr::Apply(callee, args, _) => IRExpr::Call(
            Box::new(compile_expr(callee, ctx)),
            args.iter().map(|a| compile_expr(a, ctx)).collect(),
            Type::Unknown,
        ),
        Expr::FieldAccess(obj, field, _) => IRExpr::FieldAccess(
            Box::new(compile_expr(obj, ctx)),
            field.clone(),
            Type::Unknown,
        ),
        Expr::Block(block) => compile_block(block, ctx),
        Expr::Match(subject, arms, _) => {
            let subject = compile_expr(subject, ctx);
            let arms = arms.iter().map(|a| compile_arm(a, ctx)).collect();
            IRExpr::Match(Box::new(subject), arms, Type::Unknown)
        }
        Expr::Collect(block, _) => {
            let inner = compile_block(block, ctx);
            IRExpr::Collect(Box::new(inner), Type::List(Box::new(Type::Unknown)))
        }
        Expr::If(cond, then_block, else_block, _) => {
            let cond = compile_expr(cond, ctx);
            let then_expr = compile_block(then_block, ctx);
            let else_expr = else_block
                .as_ref()
                .map(|b| compile_block(b, ctx))
                .unwrap_or_else(|| IRExpr::Lit(Lit::Unit, Type::Unit));
            IRExpr::If(
                Box::new(cond),
                Box::new(then_expr),
                Box::new(else_expr),
                Type::Unknown,
            )
        }
        Expr::Closure(params, body, _) => {
            let closure_name = format!("$closure{}", ctx.closure_counter);
            ctx.closure_counter = ctx.closure_counter.saturating_add(1);
            let mut bound = params.iter().cloned().collect::<HashSet<_>>();
            let mut free = HashSet::new();
            collect_free_vars_expr(body, &mut bound, &mut free);
            let mut captures: Vec<(String, u16)> = free
                .into_iter()
                .filter_map(|name| ctx.resolve_local(&name).map(|slot| (name, slot)))
                .collect();
            captures.sort_by(|a, b| a.0.cmp(&b.0));
            let saved_next = ctx.next_slot;
            let saved_anon = ctx.anon_counter;
            let saved_locals = std::mem::take(&mut ctx.locals);

            ctx.next_slot = 0;
            ctx.anon_counter = 0;
            ctx.push_scope();
            for (name, _) in &captures {
                ctx.define_local(name.clone());
            }
            for param in params {
                ctx.define_local(param.clone());
            }
            let body_ir = compile_expr(body, ctx);
            let local_count = ctx.next_slot as usize;

            ctx.locals = saved_locals;
            ctx.next_slot = saved_next;
            ctx.anon_counter = saved_anon;

            let global_idx = ctx.next_global_idx;
            ctx.next_global_idx = ctx.next_global_idx.saturating_add(1);
            let fn_idx = ctx.next_fn_idx;
            ctx.next_fn_idx += 1;
            ctx.globals.insert(closure_name.clone(), global_idx);
            ctx.lifted_globals.push(IRGlobal {
                name: closure_name.clone(),
                kind: IRGlobalKind::Fn(fn_idx),
            });
            ctx.lifted_fns.push(IRFnDef {
                name: closure_name,
                param_count: captures.len() + params.len(),
                local_count,
                effects: Vec::new(),
                return_ty: body_ir.ty().clone(),
                body: body_ir,
            });

            IRExpr::Closure(
                global_idx,
                captures
                    .into_iter()
                    .map(|(_, slot)| IRExpr::Local(slot, Type::Unknown))
                    .collect(),
                Type::Unknown,
            )
        }
        Expr::BinOp(op, left, right, _) => {
            let left = compile_expr(left, ctx);
            let right = compile_expr(right, ctx);
            let ty = match op {
                crate::ast::BinOp::Eq
                | crate::ast::BinOp::NotEq
                | crate::ast::BinOp::Lt
                | crate::ast::BinOp::Gt
                | crate::ast::BinOp::LtEq
                | crate::ast::BinOp::GtEq => Type::Bool,
                _ => left.ty().clone(),
            };
            IRExpr::BinOp(op.clone(), Box::new(left), Box::new(right), ty)
        }
        Expr::RecordConstruct(_, fields, _) => IRExpr::RecordConstruct(
            fields
                .iter()
                .map(|(name, expr)| (name.clone(), compile_expr(expr, ctx)))
                .collect(),
            Type::Unknown,
        ),
        Expr::EmitExpr(inner, _) => {
            IRExpr::Emit(Box::new(compile_expr(inner, ctx)), Type::Unit)
        }
    }
}

pub fn compile_stmt(stmt: &Stmt, ctx: &mut CompileCtx) -> IRStmt {
    match stmt {
        Stmt::Bind(bind) => {
            let slot = match &bind.pattern {
                Pattern::Bind(name, _) => ctx.define_local(name.clone()),
                _ => ctx.define_pattern_slot(),
            };
            IRStmt::Bind(slot, compile_expr(&bind.expr, ctx))
        }
        Stmt::Chain(chain) => {
            let slot = if let Some(slot) = ctx.resolve_local(&chain.name) {
                slot
            } else {
                ctx.define_local(chain.name.clone())
            };
            IRStmt::Chain(slot, compile_expr(&chain.expr, ctx))
        }
        Stmt::Yield(yield_stmt) => IRStmt::Yield(compile_expr(&yield_stmt.expr, ctx)),
        Stmt::Expr(expr) => IRStmt::Expr(compile_expr(expr, ctx)),
    }
}

fn compile_arm(arm: &MatchArm, ctx: &mut CompileCtx) -> IRArm {
    ctx.push_scope();
    let pattern = compile_pattern(&arm.pattern, ctx);
    let guard = arm.guard.as_ref().map(|g| compile_expr(g, ctx));
    let body = compile_expr(&arm.body, ctx);
    ctx.pop_scope();
    IRArm { pattern, guard, body }
}

pub fn compile_pattern(pat: &Pattern, ctx: &mut CompileCtx) -> IRPattern {
    match pat {
        Pattern::Wildcard(_) => IRPattern::Wildcard,
        Pattern::Lit(lit, _) => IRPattern::Lit(lit.clone()),
        Pattern::Bind(name, _) => IRPattern::Bind(ctx.define_local(name.clone())),
        Pattern::Variant(name, inner, _) => IRPattern::Variant(
            name.clone(),
            inner.as_ref().map(|p| Box::new(compile_pattern(p, ctx))),
        ),
        Pattern::Record(fields, _) => IRPattern::Record(
            fields
                .iter()
                .map(|FieldPattern { name, pattern, .. }| {
                    let pat = if let Some(inner) = pattern {
                        compile_pattern(inner, ctx)
                    } else {
                        IRPattern::Bind(ctx.define_local(name.clone()))
                    };
                    (name.clone(), pat)
                })
                .collect(),
        ),
    }
}

fn lit_type(lit: &Lit) -> Type {
    match lit {
        Lit::Bool(_) => Type::Bool,
        Lit::Int(_) => Type::Int,
        Lit::Float(_) => Type::Float,
        Lit::Str(_) => Type::String,
        Lit::Unit => Type::Unit,
    }
}

fn lower_type_expr(ty: &TypeExpr) -> Type {
    match ty {
        TypeExpr::Named(name, args, _) => match (name.as_str(), args.as_slice()) {
            ("List", [inner]) => Type::List(Box::new(lower_type_expr(inner))),
            ("Map", [k, v]) => Type::Map(Box::new(lower_type_expr(k)), Box::new(lower_type_expr(v))),
            ("Option", [inner]) => Type::Option(Box::new(lower_type_expr(inner))),
            ("Result", [ok, err]) => {
                Type::Result(Box::new(lower_type_expr(ok)), Box::new(lower_type_expr(err)))
            }
            _ if args.is_empty() => match name.as_str() {
                "Bool" => Type::Bool,
                "Int" => Type::Int,
                "Float" => Type::Float,
                "String" => Type::String,
                "Unit" => Type::Unit,
                _ => {
                    if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                        Type::Var(name.clone())
                    } else {
                        Type::Named(name.clone(), Vec::new())
                    }
                }
            },
            _ => Type::Named(name.clone(), args.iter().map(lower_type_expr).collect()),
        },
        TypeExpr::Optional(inner, _) => Type::Option(Box::new(lower_type_expr(inner))),
        TypeExpr::Fallible(inner, _) => Type::Result(
            Box::new(lower_type_expr(inner)),
            Box::new(Type::String),
        ),
        TypeExpr::Arrow(input, output, _) => {
            Type::Arrow(Box::new(lower_type_expr(input)), Box::new(lower_type_expr(output)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compile_program;
    use crate::middle::ir::{IRExpr, IRGlobalKind, IRStmt};
    use crate::frontend::lexer::Lexer;
    use crate::frontend::parser::Parser;

    fn compile_source(source: &str) -> crate::middle::ir::IRProgram {
        let mut lexer = Lexer::new(source, "test.fav");
        let tokens = lexer.tokenize().expect("tokenize");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().expect("parse");
        compile_program(&program)
    }

    #[test]
    fn compile_program_registers_functions_flows_and_variants() {
        let ir = compile_source(
            r#"
type Direction =
    | North
    | South

trf Inc: Int -> Int = |x| { x + 1 }
flw Bump = Inc |> Inc

public fn main() -> Int {
    bind result <- 1 |> Bump
    result
}
"#,
        );

        assert_eq!(ir.fns.len(), 3);
        assert!(ir.globals.iter().any(|g| g.name == "Inc" && matches!(g.kind, IRGlobalKind::Fn(_))));
        assert!(ir.globals.iter().any(|g| g.name == "main" && matches!(g.kind, IRGlobalKind::Fn(_))));
        assert!(ir.globals.iter().any(|g| g.name == "Bump" && matches!(g.kind, IRGlobalKind::Fn(_))));
        assert!(ir.globals.iter().any(|g| g.name == "North" && matches!(g.kind, IRGlobalKind::VariantCtor)));
        assert!(ir.globals.iter().any(|g| g.name == "South" && matches!(g.kind, IRGlobalKind::VariantCtor)));
    }

    #[test]
    fn compile_pipeline_lowers_to_nested_calls() {
        let ir = compile_source(
            r#"
trf Inc: Int -> Int = |x| { x + 1 }

public fn main() -> Int {
    bind result <- 1 |> Inc |> Inc
    result
}
"#,
        );

        let main_fn = ir.fns.iter().find(|f| f.name == "main").expect("main fn");
        let IRExpr::Block(stmts, tail, _) = &main_fn.body else {
            panic!("expected block body");
        };

        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            IRStmt::Bind(_, IRExpr::Call(callee, args, _)) => {
                assert_eq!(args.len(), 1);
                assert!(matches!(callee.as_ref(), IRExpr::Global(_, _)));
                assert!(matches!(args[0], IRExpr::Call(_, _, _)));
            }
            other => panic!("expected pipeline bind call, got {other:?}"),
        }

        assert!(matches!(tail.as_ref(), IRExpr::Local(_, _)));
    }

    #[test]
    fn compile_closure_lifts_to_synthetic_function_global() {
        let ir = compile_source(
            r#"
public fn main() -> Int {
    bind f <- |x| x + 1
    f(10)
}
"#,
        );

        assert_eq!(ir.fns.len(), 2);
        assert!(ir.globals.iter().any(|g| g.name == "main" && matches!(g.kind, IRGlobalKind::Fn(0))));
        assert!(ir.globals.iter().any(|g| g.name.starts_with("$closure") && matches!(g.kind, IRGlobalKind::Fn(1))));
    }

    #[test]
    fn compile_closure_captures_outer_local_slots() {
        let ir = compile_source(
            r#"
public fn main() -> Int {
    bind y <- 2
    bind f <- |x| x + y
    f(10)
}
"#,
        );

        let main_fn = ir.fns.iter().find(|f| f.name == "main").expect("main fn");
        let IRExpr::Block(stmts, _, _) = &main_fn.body else {
            panic!("expected block body");
        };
        let Some(IRStmt::Bind(_, IRExpr::Closure(_, captures, _))) = stmts.get(1) else {
            panic!("expected captured closure bind");
        };
        assert_eq!(captures.len(), 1);
        assert!(matches!(captures[0], IRExpr::Local(_, _)));

        let closure_fn = ir.fns.iter().find(|f| f.name.starts_with("$closure")).expect("closure fn");
        assert_eq!(closure_fn.param_count, 2);
    }
}
