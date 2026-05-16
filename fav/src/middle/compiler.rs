use std::cell::Cell;
use std::collections::{HashMap, HashSet};

thread_local! {
    static COVERAGE_MODE: Cell<bool> = Cell::new(false);
}

/// Enable coverage-tracking IR emission for the current thread.
pub fn set_coverage_mode(enabled: bool) {
    COVERAGE_MODE.with(|c| c.set(enabled));
}

use super::checker::Type;
use super::ir::{
    FieldMeta, IRArm, IRExpr, IRFnDef, IRGlobal, IRGlobalKind, IRPattern, IRProgram, IRStmt,
    TypeMeta,
};
use crate::ast::{
    AbstractFlwDef, AbstractTrfDef, BindStmt, Block, Expr, FStringPart, FlwBindingDef, FlwDef,
    FlwSlot, ImplDef, InterfaceDecl, InterfaceImplDecl, Item, Lit, MatchArm, Pattern, PatternField,
    Program, Stmt, TypeBody, TypeExpr,
};

fn collect_abstract_flw_defs(program: &Program) -> HashMap<String, AbstractFlwDef> {
    let mut out = HashMap::new();
    for item in &program.items {
        if let Item::AbstractFlwDef(def) = item {
            out.insert(def.name.clone(), def.clone());
        }
    }
    out
}

fn fully_bound_flw_info<'a>(
    fd: &'a FlwBindingDef,
    templates: &'a HashMap<String, AbstractFlwDef>,
) -> Option<&'a AbstractFlwDef> {
    let template = templates.get(&fd.template)?;
    let bound: HashSet<&str> = fd.bindings.iter().map(|(slot, _)| slot.as_str()).collect();
    let all_known = fd
        .bindings
        .iter()
        .all(|(slot, _)| template.slots.iter().any(|s| s.name == *slot));
    if all_known && bound.len() == template.slots.len() {
        Some(template)
    } else {
        None
    }
}

#[derive(Debug, Default)]
pub struct CompileCtx {
    pub locals: Vec<HashMap<String, u16>>,
    pub local_tys: Vec<HashMap<String, Type>>,
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
        self.local_tys.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.locals.pop();
        self.local_tys.pop();
    }

    pub fn define_local(&mut self, name: impl Into<String>) -> u16 {
        self.define_local_with_ty(name, Type::Unknown)
    }

    pub fn define_local_with_ty(&mut self, name: impl Into<String>, ty: Type) -> u16 {
        let name = name.into();
        let slot = self.next_slot;
        self.next_slot = self.next_slot.saturating_add(1);
        if self.locals.is_empty() {
            self.push_scope();
        }
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.clone(), slot);
        }
        if let Some(scope) = self.local_tys.last_mut() {
            scope.insert(name, ty);
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

    pub fn resolve_local_ty(&self, name: &str) -> Option<Type> {
        for scope in self.local_tys.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
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
    let mut type_metas = HashMap::new();
    let mut next_fn_idx = 0usize;
    let interface_method_types = collect_interface_method_types(program);
    let abstract_flw_defs = collect_abstract_flw_defs(program);
    let include_std_states = program
        .uses
        .iter()
        .any(|path| path.len() >= 2 && path[0] == "std" && path[1] == "states");
    let std_state_defs = if include_std_states {
        crate::std_states::parsed_type_defs()
    } else {
        Vec::new()
    };

    for ns in &[
        "IO",
        "Debug",
        "Result",
        "Option",
        "Math",
        "Int",
        "Float",
        "Bool",
        "String",
        "List",
        "Map",
        "Trace",
        "Emit",
        "File",
        "Json",
        "Csv",
        "Schema",
        "Checkpoint",
        "Db",
        "DB",
        "Env",
        "Http",
        "Grpc",
        "Parquet",
        "Task",
        "Random",
        "Stream",
        "Gen",
        "Validate",
        "assert",
        "assert_eq",
        "assert_ne",
        "IO.println_int",
        "IO.println_float",
        "IO.println_bool",
        "IO.print",
    ] {
        if !ctx.globals.contains_key(*ns) {
            let idx = globals.len() as u16;
            ctx.globals.insert(ns.to_string(), idx);
            globals.push(IRGlobal {
                name: ns.to_string(),
                kind: IRGlobalKind::Builtin,
            });
        }
    }

    for td in &std_state_defs {
        if !ctx.globals.contains_key(&td.name) {
            let idx = globals.len() as u16;
            ctx.globals.insert(td.name.clone(), idx);
            globals.push(IRGlobal {
                name: td.name.clone(),
                kind: IRGlobalKind::Builtin,
            });
        }
        let ctor_name = format!("{}.new", td.name);
        let idx = globals.len() as u16;
        ctx.globals.insert(ctor_name.clone(), idx);
        globals.push(IRGlobal {
            name: ctor_name,
            kind: IRGlobalKind::Fn(next_fn_idx),
        });
        next_fn_idx += 1;
    }

    for td in &std_state_defs {
        fns.push(compile_type_def_constructor(td, &mut ctx));
    }

    for ty_name in &[
        "CheckpointMeta",
        "HttpResponse",
        "HttpError",
        "RpcError",
        "RpcRequest",
        "ParquetError",
        "ValidationError",
    ] {
        if !ctx.globals.contains_key(*ty_name) {
            let idx = globals.len() as u16;
            ctx.globals.insert((*ty_name).into(), idx);
            globals.push(IRGlobal {
                name: (*ty_name).into(),
                kind: IRGlobalKind::Builtin,
            });
        }
    }

    for item in &program.items {
        match item {
            Item::EffectDef(..) => {}
            Item::ImportDecl { .. } => {}
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
            Item::AbstractTrfDef(AbstractTrfDef { name, .. }) => {
                let idx = globals.len() as u16;
                ctx.globals.insert(name.clone(), idx);
                globals.push(IRGlobal {
                    name: name.clone(),
                    kind: IRGlobalKind::Builtin,
                });
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
            Item::AbstractFlwDef(AbstractFlwDef { name, .. }) => {
                let idx = globals.len() as u16;
                ctx.globals.insert(name.clone(), idx);
                globals.push(IRGlobal {
                    name: name.clone(),
                    kind: IRGlobalKind::Builtin,
                });
            }
            Item::FlwBindingDef(FlwBindingDef { name, .. }) => {
                let idx = globals.len() as u16;
                ctx.globals.insert(name.clone(), idx);
                globals.push(IRGlobal {
                    name: name.clone(),
                    kind: match program.items.iter().find_map(|item| match item {
                        Item::FlwBindingDef(fd) if fd.name == *name => Some(fd),
                        _ => None,
                    }) {
                        Some(fd) if fully_bound_flw_info(fd, &abstract_flw_defs).is_some() => {
                            let kind = IRGlobalKind::Fn(next_fn_idx);
                            next_fn_idx += 1;
                            kind
                        }
                        _ => IRGlobalKind::Builtin,
                    },
                });
            }
            Item::TypeDef(td) => {
                if let Some(meta) = build_type_meta(td) {
                    type_metas.insert(td.name.clone(), meta);
                }
                if !ctx.globals.contains_key(&td.name) {
                    let idx = globals.len() as u16;
                    ctx.globals.insert(td.name.clone(), idx);
                    globals.push(IRGlobal {
                        name: td.name.clone(),
                        kind: IRGlobalKind::Builtin,
                    });
                }
                if let TypeBody::Sum(variants) = &td.body {
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
                if !td.invariants.is_empty() && matches!(td.body, TypeBody::Record(_)) {
                    let ctor_name = format!("{}.new", td.name);
                    let idx = globals.len() as u16;
                    ctx.globals.insert(ctor_name.clone(), idx);
                    globals.push(IRGlobal {
                        name: ctor_name,
                        kind: IRGlobalKind::Fn(next_fn_idx),
                    });
                    next_fn_idx += 1;
                }
            }
            Item::ImplDef(id) => {
                if let Some(first_arg) = id.type_args.first() {
                    let type_key = lower_type_expr(first_arg).display();
                    let cap_ns = id.cap_name.to_ascii_lowercase();
                    for method in &id.methods {
                        let global_name = format!("{}.{}.{}", type_key, cap_ns, method.name);
                        let idx = globals.len() as u16;
                        ctx.globals.insert(global_name.clone(), idx);
                        globals.push(IRGlobal {
                            name: global_name,
                            kind: IRGlobalKind::Fn(next_fn_idx),
                        });
                        next_fn_idx += 1;
                    }
                }
            }
            Item::InterfaceImplDecl(id) => {
                if !id.is_auto {
                    for interface_name in &id.interface_names {
                        let iface_ns = interface_name.to_ascii_lowercase();
                        for (method_name, _) in &id.methods {
                            let global_name =
                                format!("{}.{}.{}", id.type_name, iface_ns, method_name);
                            let idx = globals.len() as u16;
                            ctx.globals.insert(global_name.clone(), idx);
                            globals.push(IRGlobal {
                                name: global_name,
                                kind: IRGlobalKind::Fn(next_fn_idx),
                            });
                            next_fn_idx += 1;
                        }
                    }
                }
            }
            Item::TestDef(_) => {
                next_fn_idx += 1;
            }
            Item::BenchDef(_) => {
                next_fn_idx += 1;
            }
            Item::InterfaceDecl(_) => {}
            _ => {}
        }
    }

    // 組み込み名前空間をグローバルテーブルに登録する（ユーザー定義名と衝突しない場合のみ）
    for ns in &[
        "IO",
        "Debug",
        "Result",
        "Option",
        "Int",
        "Float",
        "Bool",
        "String",
        "List",
        "Map",
        "Trace",
        "Emit",
        "File",
        "Json",
        "Csv",
        "Schema",
        "Db",
        "DB",
        "Env",
        "Http",
        "Parquet",
        "Random",
        "Stream",
        "Gen",
        "Validate",
        // test assertion builtins (callable without namespace prefix)
        "assert",
        "assert_eq",
        "assert_ne",
        "IO.println_int",
        "IO.println_float",
        "IO.println_bool",
        "IO.print",
    ] {
        if !ctx.globals.contains_key(*ns) {
            let idx = globals.len() as u16;
            ctx.globals.insert(ns.to_string(), idx);
            globals.push(IRGlobal {
                name: ns.to_string(),
                kind: IRGlobalKind::Builtin,
            });
        }
    }

    ctx.next_global_idx = globals.len() as u16;
    ctx.next_fn_idx = next_fn_idx;

    for item in &program.items {
        match item {
            Item::EffectDef(..) => {}
            Item::ImportDecl { .. } => {}
            Item::TypeDef(td) => {
                if !td.invariants.is_empty() && matches!(td.body, TypeBody::Record(_)) {
                    fns.push(compile_type_def_constructor(td, &mut ctx));
                }
            }
            Item::FnDef(fd) => fns.push(compile_fn_def(
                &fd.name,
                &fd.type_params,
                &fd.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
                &fd.params
                    .iter()
                    .map(|p| lower_type_expr(&p.ty))
                    .collect::<Vec<_>>(),
                &fd.effects,
                fd.return_ty.as_ref(),
                &fd.body,
                &mut ctx,
            )),
            Item::TrfDef(td) => fns.push(compile_fn_def(
                &td.name,
                &td.type_params,
                &td.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
                &td.params
                    .iter()
                    .map(|p| lower_type_expr(&p.ty))
                    .collect::<Vec<_>>(),
                &td.effects,
                Some(&td.output_ty),
                &td.body,
                &mut ctx,
            )),
            Item::FlwDef(fd) => fns.push(compile_flw_def(fd, &mut ctx)),
            Item::AbstractTrfDef(_) => {}
            Item::AbstractFlwDef(_) => {}
            Item::FlwBindingDef(fd) => {
                if let Some(template) = fully_bound_flw_info(fd, &abstract_flw_defs) {
                    fns.push(compile_flw_binding_def(fd, template, &mut ctx));
                }
            }
            Item::ImplDef(id) => {
                fns.extend(compile_impl_def(id, &mut ctx));
            }
            Item::InterfaceImplDecl(id) => {
                fns.extend(compile_interface_impl_decl(
                    id,
                    &interface_method_types,
                    &mut ctx,
                ));
            }
            Item::TestDef(td) => {
                let unit_ty = crate::ast::TypeExpr::Named(
                    "Unit".into(),
                    vec![],
                    crate::frontend::lexer::Span::dummy(),
                );
                fns.push(compile_fn_def(
                    &format!("$test:{}", td.name),
                    &[],
                    &[],
                    &[],
                    &[],
                    Some(&unit_ty),
                    &td.body,
                    &mut ctx,
                ))
            }
            Item::BenchDef(bd) => {
                // Bench bodies are compiled with a generated function name for runner use.
                let unit_ty = crate::ast::TypeExpr::Named(
                    "Unit".into(),
                    vec![],
                    crate::frontend::lexer::Span::dummy(),
                );
                fns.push(compile_fn_def(
                    &format!("$bench:{}", bd.description),
                    &[],
                    &[],
                    &[],
                    &[],
                    Some(&unit_ty),
                    &bd.body,
                    &mut ctx,
                ))
            }
            Item::InterfaceDecl(_) => {}
            _ => {}
        }
    }

    globals.extend(ctx.lifted_globals.clone());
    fns.extend(ctx.lifted_fns.clone());

    IRProgram {
        globals,
        fns,
        type_metas,
    }
}

fn build_type_meta(td: &crate::ast::TypeDef) -> Option<TypeMeta> {
    let crate::ast::TypeBody::Record(fields) = &td.body else {
        return None;
    };
    Some(TypeMeta {
        type_name: td.name.clone(),
        fields: fields
            .iter()
            .map(|field| FieldMeta {
                name: field.name.clone(),
                ty: lower_type_expr(&field.ty).display(),
                col_index: field
                    .attrs
                    .iter()
                    .find(|attr| attr.name == "col")
                    .and_then(|attr| attr.arg.as_ref())
                    .and_then(|arg| arg.parse::<usize>().ok()),
            })
            .collect(),
    })
}

fn compile_flw_def(fd: &FlwDef, ctx: &mut CompileCtx) -> IRFnDef {
    let saved_next = ctx.next_slot;
    let saved_anon = ctx.anon_counter;
    let saved_locals = std::mem::take(&mut ctx.locals);
    let saved_local_tys = std::mem::take(&mut ctx.local_tys);

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
    ctx.local_tys = saved_local_tys;
    ctx.next_slot = saved_next;
    ctx.anon_counter = saved_anon;

    IRFnDef {
        name: fd.name.clone(),
        param_count: 1,
        param_tys: vec![Type::Unknown],
        local_count,
        effects: Vec::new(),
        return_ty: Type::Unknown,
        body: current,
    }
}

fn lower_type_expr_with_subst(ty: &TypeExpr, subst: &HashMap<String, Type>) -> Type {
    match ty {
        TypeExpr::Named(name, args, _) if args.is_empty() => subst
            .get(name)
            .cloned()
            .unwrap_or_else(|| lower_type_expr(ty)),
        TypeExpr::Named(name, args, _) => {
            let resolved_args: Vec<Type> = args
                .iter()
                .map(|arg| lower_type_expr_with_subst(arg, subst))
                .collect();
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
        TypeExpr::Optional(inner, _) => {
            Type::Option(Box::new(lower_type_expr_with_subst(inner, subst)))
        }
        TypeExpr::Fallible(inner, _) => Type::Result(
            Box::new(lower_type_expr_with_subst(inner, subst)),
            Box::new(Type::String),
        ),
        TypeExpr::Arrow(input, output, _) => Type::Arrow(
            Box::new(lower_type_expr_with_subst(input, subst)),
            Box::new(lower_type_expr_with_subst(output, subst)),
        ),
        TypeExpr::TrfFn {
            input,
            output,
            effects,
            ..
        } => Type::Trf(
            Box::new(lower_type_expr_with_subst(input, subst)),
            Box::new(lower_type_expr_with_subst(output, subst)),
            effects.clone(),
        ),
    }
}

fn infer_flw_slot_effects(slots: &[FlwSlot]) -> Vec<crate::ast::Effect> {
    let mut out = Vec::new();
    for slot in slots {
        let slot_effects: Vec<crate::ast::Effect> = if let Some(slot_ty) = &slot.abstract_trf_ty {
            match lower_type_expr(slot_ty) {
                Type::AbstractTrf { effects, .. } | Type::Trf(_, _, effects) => effects,
                _ => slot.effects.clone(),
            }
        } else {
            slot.effects.clone()
        };
        for effect in &slot_effects {
            if !out.iter().any(|existing| existing == effect) {
                out.push(effect.clone());
            }
        }
    }
    out
}

fn lower_flw_slot_signature_with_subst(
    slot: &FlwSlot,
    subst: &HashMap<String, Type>,
) -> (Type, Type, Vec<crate::ast::Effect>) {
    if let Some(slot_ty) = &slot.abstract_trf_ty {
        match lower_type_expr_with_subst(slot_ty, subst) {
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
        lower_type_expr_with_subst(&slot.input_ty, subst),
        lower_type_expr_with_subst(&slot.output_ty, subst),
        slot.effects.clone(),
    )
}

fn compile_flw_binding_def(
    fd: &FlwBindingDef,
    template: &AbstractFlwDef,
    ctx: &mut CompileCtx,
) -> IRFnDef {
    let saved_next = ctx.next_slot;
    let saved_anon = ctx.anon_counter;
    let saved_locals = std::mem::take(&mut ctx.locals);
    let saved_local_tys = std::mem::take(&mut ctx.local_tys);
    let mut saved_local_slots = HashMap::new();
    for scope in &saved_locals {
        for (name, slot) in scope {
            saved_local_slots.insert(name.clone(), *slot);
        }
    }

    ctx.next_slot = 0;
    ctx.anon_counter = 0;
    ctx.push_scope();
    let input_slot = ctx.define_local("$input");

    let type_subst: HashMap<String, Type> = template
        .type_params
        .iter()
        .cloned()
        .zip(fd.type_args.iter().map(lower_type_expr))
        .collect();
    let binding_map: HashMap<&str, &crate::ast::SlotImpl> = fd
        .bindings
        .iter()
        .map(|(slot, impl_name)| (slot.as_str(), impl_name))
        .collect();

    let mut current = IRExpr::Local(input_slot, Type::Unknown);
    for slot in &template.slots {
        let impl_name = binding_map
            .get(slot.name.as_str())
            .copied()
            .expect("fully bound flw binding must have all slots");
        current = match impl_name {
            crate::ast::SlotImpl::Global(name) => {
                let callee = if let Some(global_idx) = ctx.resolve_global(name) {
                    IRExpr::TrfRef(global_idx, Type::Unknown)
                } else {
                    IRExpr::TrfRef(u16::MAX, Type::Unknown)
                };
                IRExpr::Call(Box::new(callee), vec![current], Type::Unknown)
            }
            crate::ast::SlotImpl::Local(name) => {
                let local = saved_local_slots.get(name).copied().unwrap_or(u16::MAX);
                IRExpr::CallTrfLocal {
                    local,
                    arg: Box::new(current),
                    ty: Type::Unknown,
                }
            }
        };
    }

    let local_count = ctx.next_slot as usize;
    ctx.locals = saved_locals;
    ctx.local_tys = saved_local_tys;
    ctx.next_slot = saved_next;
    ctx.anon_counter = saved_anon;

    let first_slot = template
        .slots
        .first()
        .expect("fully bound flw template has at least one slot");
    let last_slot = template.slots.last().unwrap();
    let (input_ty, _, _) = lower_flw_slot_signature_with_subst(first_slot, &type_subst);
    let (_, return_ty, _) = lower_flw_slot_signature_with_subst(last_slot, &type_subst);
    IRFnDef {
        name: fd.name.clone(),
        param_count: 1,
        param_tys: vec![input_ty],
        local_count,
        effects: infer_flw_slot_effects(&template.slots),
        return_ty,
        body: current,
    }
}

fn compile_type_def_constructor(td: &crate::ast::TypeDef, ctx: &mut CompileCtx) -> IRFnDef {
    let TypeBody::Record(fields) = &td.body else {
        unreachable!("constructor only generated for record state types");
    };

    let saved_next = ctx.next_slot;
    let saved_anon = ctx.anon_counter;
    let saved_locals = std::mem::take(&mut ctx.locals);
    let saved_local_tys = std::mem::take(&mut ctx.local_tys);

    ctx.next_slot = 0;
    ctx.anon_counter = 0;
    ctx.push_scope();
    for field in fields {
        ctx.define_local_with_ty(field.name.clone(), lower_type_expr(&field.ty));
    }

    let body = build_constructor_body(td, fields, ctx);
    let local_count = ctx.next_slot as usize;

    ctx.locals = saved_locals;
    ctx.local_tys = saved_local_tys;
    ctx.next_slot = saved_next;
    ctx.anon_counter = saved_anon;

    IRFnDef {
        name: format!("{}.new", td.name),
        param_count: fields.len(),
        param_tys: fields.iter().map(|f| lower_type_expr(&f.ty)).collect(),
        local_count,
        effects: Vec::new(),
        return_ty: Type::Result(
            Box::new(Type::Named(td.name.clone(), vec![])),
            Box::new(Type::String),
        ),
        body,
    }
}

fn build_constructor_body(
    td: &crate::ast::TypeDef,
    fields: &[crate::ast::Field],
    ctx: &mut CompileCtx,
) -> IRExpr {
    let record_expr = IRExpr::RecordConstruct(
        fields
            .iter()
            .map(|field| {
                let slot = ctx
                    .resolve_local(&field.name)
                    .expect("constructor field local must exist");
                (
                    field.name.clone(),
                    IRExpr::Local(slot, lower_type_expr(&field.ty)),
                )
            })
            .collect(),
        Type::Named(td.name.clone(), vec![]),
    );

    let ok_expr = make_result_ctor_call(ctx, "ok", record_expr, td.name.clone());
    if td.invariants.is_empty() {
        return ok_expr;
    }

    let cond = build_invariant_condition(&td.invariants, ctx);
    let err_expr = make_result_ctor_call(
        ctx,
        "err",
        IRExpr::Lit(
            Lit::Str(format!("InvariantViolation: {}", td.name)),
            Type::String,
        ),
        td.name.clone(),
    );
    IRExpr::If(
        Box::new(cond),
        Box::new(ok_expr),
        Box::new(err_expr),
        Type::Result(
            Box::new(Type::Named(td.name.clone(), vec![])),
            Box::new(Type::String),
        ),
    )
}

fn build_invariant_condition(invariants: &[Expr], ctx: &mut CompileCtx) -> IRExpr {
    let mut iter = invariants.iter();
    let first = compile_expr(iter.next().expect("at least one invariant"), ctx);
    iter.fold(first, |acc, invariant| {
        let next = compile_expr(invariant, ctx);
        IRExpr::If(
            Box::new(acc),
            Box::new(next),
            Box::new(IRExpr::Lit(Lit::Bool(false), Type::Bool)),
            Type::Bool,
        )
    })
}

fn make_result_ctor_call(
    ctx: &CompileCtx,
    method: &str,
    payload: IRExpr,
    type_name: String,
) -> IRExpr {
    let result_idx = ctx
        .resolve_global("Result")
        .expect("Result namespace must be registered");
    let result_ty = Type::Result(
        Box::new(Type::Named(type_name, vec![])),
        Box::new(Type::String),
    );
    IRExpr::Call(
        Box::new(IRExpr::FieldAccess(
            Box::new(IRExpr::Global(result_idx, Type::Unknown)),
            method.to_string(),
            Type::Unknown,
        )),
        vec![payload],
        result_ty,
    )
}

fn compile_impl_def(id: &ImplDef, ctx: &mut CompileCtx) -> Vec<IRFnDef> {
    let Some(first_arg) = id.type_args.first() else {
        return Vec::new();
    };
    let type_key = lower_type_expr(first_arg).display();
    let cap_ns = id.cap_name.to_ascii_lowercase();
    id.methods
        .iter()
        .map(|method| {
            let global_name = format!("{}.{}.{}", type_key, cap_ns, method.name);
            compile_fn_def(
                &global_name,
                &method.type_params,
                &method
                    .params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>(),
                &method
                    .params
                    .iter()
                    .map(|p| lower_type_expr(&p.ty))
                    .collect::<Vec<_>>(),
                &method.effects,
                method.return_ty.as_ref(),
                &method.body,
                ctx,
            )
        })
        .collect()
}

fn compile_interface_impl_decl(
    id: &InterfaceImplDecl,
    interface_method_types: &HashMap<String, HashMap<String, TypeExpr>>,
    ctx: &mut CompileCtx,
) -> Vec<IRFnDef> {
    if id.is_auto {
        return Vec::new();
    }

    let mut out = Vec::new();
    for interface_name in &id.interface_names {
        let iface_ns = interface_name.to_ascii_lowercase();
        for (method_name, body_expr) in &id.methods {
            let global_name = format!("{}.{}.{}", id.type_name, iface_ns, method_name);
            let method_ty = interface_method_types
                .get(interface_name)
                .and_then(|m| m.get(method_name))
                .cloned()
                .or_else(|| builtin_interface_method_type(interface_name, method_name))
                .unwrap_or_else(|| {
                    TypeExpr::Named(
                        "Unknown".into(),
                        vec![],
                        crate::frontend::lexer::Span::dummy(),
                    )
                });

            let method_ty = substitute_self_in_type_expr(&method_ty, &id.type_name);
            let (param_tys, return_ty) = split_arrow_type(&method_ty);

            match body_expr {
                Expr::Closure(params, body, span) => {
                    let body_block = Block {
                        stmts: vec![],
                        expr: body.clone(),
                        span: span.clone(),
                    };
                    out.push(compile_fn_def(
                        &global_name,
                        &[],
                        params,
                        &param_tys.iter().map(lower_type_expr).collect::<Vec<_>>(),
                        &[],
                        Some(&return_ty),
                        &body_block,
                        ctx,
                    ));
                }
                expr => {
                    let body_block = Block {
                        stmts: vec![],
                        expr: Box::new(expr.clone()),
                        span: expr.span().clone(),
                    };
                    out.push(compile_fn_def(
                        &global_name,
                        &[],
                        &[],
                        &[],
                        &[],
                        Some(&return_ty),
                        &body_block,
                        ctx,
                    ));
                }
            }
        }
    }
    out
}

fn collect_interface_method_types(program: &Program) -> HashMap<String, HashMap<String, TypeExpr>> {
    let mut out = HashMap::new();
    for item in &program.items {
        if let Item::InterfaceDecl(InterfaceDecl { name, methods, .. }) = item {
            let mut map = HashMap::new();
            for method in methods {
                map.insert(method.name.clone(), method.ty.clone());
            }
            out.insert(name.clone(), map);
        }
    }
    out
}

fn substitute_self_in_type_expr(ty: &TypeExpr, type_name: &str) -> TypeExpr {
    match ty {
        TypeExpr::Named(name, args, span) if name == "Self" => {
            TypeExpr::Named(type_name.to_string(), vec![], span.clone())
        }
        TypeExpr::Named(name, args, span) => TypeExpr::Named(
            name.clone(),
            args.iter()
                .map(|a| substitute_self_in_type_expr(a, type_name))
                .collect(),
            span.clone(),
        ),
        TypeExpr::Optional(inner, span) => TypeExpr::Optional(
            Box::new(substitute_self_in_type_expr(inner, type_name)),
            span.clone(),
        ),
        TypeExpr::Fallible(inner, span) => TypeExpr::Fallible(
            Box::new(substitute_self_in_type_expr(inner, type_name)),
            span.clone(),
        ),
        TypeExpr::Arrow(a, b, span) => TypeExpr::Arrow(
            Box::new(substitute_self_in_type_expr(a, type_name)),
            Box::new(substitute_self_in_type_expr(b, type_name)),
            span.clone(),
        ),
        TypeExpr::TrfFn {
            input,
            output,
            effects,
            span,
        } => TypeExpr::TrfFn {
            input: Box::new(substitute_self_in_type_expr(input, type_name)),
            output: Box::new(substitute_self_in_type_expr(output, type_name)),
            effects: effects.clone(),
            span: span.clone(),
        },
    }
}

fn split_arrow_type(ty: &TypeExpr) -> (Vec<TypeExpr>, TypeExpr) {
    let mut params = Vec::new();
    let mut current = ty;
    loop {
        match current {
            TypeExpr::Arrow(a, b, _) => {
                params.push((**a).clone());
                current = b;
            }
            other => return (params, other.clone()),
        }
    }
}

fn builtin_interface_method_type(interface_name: &str, method_name: &str) -> Option<TypeExpr> {
    let span = crate::frontend::lexer::Span::dummy();
    let self_ty = || TypeExpr::Named("Self".into(), vec![], span.clone());
    let unit_ty = || TypeExpr::Named("Unit".into(), vec![], span.clone());
    let int_ty = || TypeExpr::Named("Int".into(), vec![], span.clone());
    let bool_ty = || TypeExpr::Named("Bool".into(), vec![], span.clone());
    let string_ty = || TypeExpr::Named("String".into(), vec![], span.clone());
    let error_ty = || TypeExpr::Named("Error".into(), vec![], span.clone());
    let result_ty =
        |ok: TypeExpr, err: TypeExpr| TypeExpr::Named("Result".into(), vec![ok, err], span.clone());
    let arrow = |a: TypeExpr, b: TypeExpr| TypeExpr::Arrow(Box::new(a), Box::new(b), span.clone());

    match (interface_name, method_name) {
        ("Show", "show") => Some(arrow(self_ty(), string_ty())),
        ("Eq", "eq") => Some(arrow(self_ty(), arrow(self_ty(), bool_ty()))),
        ("Ord", "compare") => Some(arrow(self_ty(), arrow(self_ty(), int_ty()))),
        ("Gen", "gen") => Some(arrow(
            TypeExpr::Optional(Box::new(int_ty()), span.clone()),
            self_ty(),
        )),
        ("Semigroup", "combine") => Some(arrow(self_ty(), arrow(self_ty(), self_ty()))),
        ("Monoid", "empty") => Some(arrow(unit_ty(), self_ty())),
        ("Group", "inverse") => Some(arrow(self_ty(), self_ty())),
        ("Ring", "multiply") => Some(arrow(self_ty(), arrow(self_ty(), self_ty()))),
        ("Field", "divide") => Some(arrow(
            self_ty(),
            arrow(self_ty(), result_ty(self_ty(), error_ty())),
        )),
        _ => None,
    }
}

fn compile_fn_def(
    name: &str,
    type_params: &[String],
    params: &[String],
    param_tys: &[Type],
    effects: &[crate::ast::Effect],
    return_ty: Option<&TypeExpr>,
    body: &Block,
    ctx: &mut CompileCtx,
) -> IRFnDef {
    let saved_next = ctx.next_slot;
    let saved_anon = ctx.anon_counter;
    let saved_locals = std::mem::take(&mut ctx.locals);
    let saved_local_tys = std::mem::take(&mut ctx.local_tys);

    ctx.next_slot = 0;
    ctx.anon_counter = 0;
    ctx.push_scope();
    for (idx, param) in params.iter().enumerate() {
        let ty = param_tys.get(idx).cloned().unwrap_or(Type::Unknown);
        ctx.define_local_with_ty(param.clone(), ty);
    }
    for type_param in type_params {
        ctx.define_local_with_ty(format!("$type_{}", type_param), Type::String);
    }
    let body_ir = compile_block(body, ctx);
    let local_count = ctx.next_slot as usize;

    ctx.locals = saved_locals;
    ctx.local_tys = saved_local_tys;
    ctx.next_slot = saved_next;
    ctx.anon_counter = saved_anon;

    IRFnDef {
        name: name.to_string(),
        param_count: params.len() + type_params.len(),
        param_tys: param_tys
            .iter()
            .cloned()
            .chain(std::iter::repeat_n(Type::String, type_params.len()))
            .collect(),
        local_count,
        effects: effects.to_vec(),
        return_ty: return_ty
            .map(lower_type_expr)
            .unwrap_or_else(|| body_ir.ty().clone()),
        body: body_ir,
    }
}

fn compile_block(block: &Block, ctx: &mut CompileCtx) -> IRExpr {
    ctx.push_scope();
    let mut stmts = Vec::new();
    let cov = COVERAGE_MODE.with(|c| c.get());
    for stmt in &block.stmts {
        if cov {
            let line = stmt.span().line;
            stmts.push(IRStmt::TrackLine(line));
        }
        compile_stmt_into(stmt, ctx, &mut stmts);
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
                match field {
                    PatternField::Pun(name, _) => {
                        out.insert(name.clone());
                    }
                    PatternField::Alias(_, inner, _) => pattern_binds(inner, out),
                    PatternField::Wildcard(_) => {}
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
        Expr::TypeApply(expr, _, _) => collect_free_vars_expr(expr, bound, free),
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
        Expr::FString(parts, _) => {
            for part in parts {
                if let FStringPart::Expr(expr) = part {
                    collect_free_vars_expr(expr, bound, free);
                }
            }
        }
        Expr::AssertMatches(expr, _, _) => collect_free_vars_expr(expr, bound, free),
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
            Stmt::ForIn(f) => {
                collect_free_vars_expr(&f.iter, &mut local_bound, free);
                let mut inner_bound = local_bound.clone();
                inner_bound.insert(f.var.clone());
                collect_free_vars_block(&f.body, &mut inner_bound, free);
            }
        }
    }
    collect_free_vars_expr(&block.expr, &mut local_bound, free);
}

pub fn compile_expr(expr: &Expr, ctx: &mut CompileCtx) -> IRExpr {
    match expr {
        Expr::Lit(lit, _) => IRExpr::Lit(lit.clone(), lit_type(lit)),
        Expr::Ident(name, _) => {
            if let Some(slot) = ctx.resolve_local(name) {
                IRExpr::Local(slot, ctx.resolve_local_ty(name).unwrap_or(Type::Unknown))
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
        Expr::Apply(callee, args, _) => {
            if let Expr::TypeApply(inner, type_args, _) = callee.as_ref() {
                if let Expr::Ident(name, _) = inner.as_ref() {
                    if name == "type_name_of" && args.is_empty() {
                        if let Some(TypeExpr::Named(type_name, _, _)) = type_args.first() {
                            if let Some(slot) = ctx.resolve_local(&format!("$type_{}", type_name)) {
                                return IRExpr::Local(slot, Type::String);
                            }
                        }
                        let ty_name = type_args
                            .first()
                            .map(|ty| lower_type_expr(ty).display())
                            .unwrap_or_default();
                        return IRExpr::Lit(Lit::Str(ty_name), Type::String);
                    }
                }
            }
            let callee_ir = match callee.as_ref() {
                Expr::TypeApply(inner, _, _) => compile_expr(inner, ctx),
                _ => compile_expr(callee, ctx),
            };
            let mut compiled_args: Vec<IRExpr> =
                args.iter().map(|a| compile_expr(a, ctx)).collect();
            if let Expr::TypeApply(_, type_args, _) = callee.as_ref() {
                compiled_args.extend(
                    type_args.iter().map(|ty| {
                        IRExpr::Lit(Lit::Str(lower_type_expr(ty).display()), Type::String)
                    }),
                );
            }
            IRExpr::Call(Box::new(callee_ir), compiled_args, Type::Unknown)
        }
        Expr::TypeApply(expr, _, _) => compile_expr(expr, ctx),
        Expr::FieldAccess(obj, field, _) => {
            if let Expr::Ident(namespace, _) = obj.as_ref() {
                let namespace_is_bound = ctx.resolve_local(namespace).is_some()
                    || ctx.resolve_global(namespace).is_some();
                if !namespace_is_bound {
                    if let Some(idx) = ctx.resolve_global(field) {
                        return IRExpr::Global(idx, Type::Unknown);
                    }
                }
            }
            IRExpr::FieldAccess(
                Box::new(compile_expr(obj, ctx)),
                field.clone(),
                Type::Unknown,
            )
        }
        Expr::Block(block) => compile_block(block, ctx),
        Expr::Match(subject, arms, _) => {
            let subject = compile_expr(subject, ctx);
            let arms = arms.iter().map(|a| compile_arm(a, ctx)).collect();
            IRExpr::Match(Box::new(subject), arms, Type::Unknown)
        }
        Expr::AssertMatches(expr, pattern, _) => {
            let subject = compile_expr(expr, ctx);
            let assert_idx = ctx
                .resolve_global("assert")
                .expect("assert builtin must be registered");
            let ok_arm = IRArm {
                pattern: compile_pattern(pattern, ctx),
                guard: None,
                body: IRExpr::Lit(Lit::Unit, Type::Unit),
            };
            let fail_arm = IRArm {
                pattern: IRPattern::Wildcard,
                guard: None,
                body: IRExpr::Call(
                    Box::new(IRExpr::Global(assert_idx, Type::Unknown)),
                    vec![IRExpr::Lit(Lit::Bool(false), Type::Bool)],
                    Type::Unit,
                ),
            };
            IRExpr::Match(Box::new(subject), vec![ok_arm, fail_arm], Type::Unit)
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
            let saved_local_tys = std::mem::take(&mut ctx.local_tys);

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
            ctx.local_tys = saved_local_tys;
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
                param_tys: vec![Type::Unknown; captures.len() + params.len()],
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
            if matches!(op, crate::ast::BinOp::NullCoalesce) {
                // Desugar: lhs ?? rhs  →  Option.unwrap_or(lhs, rhs)
                let lhs_ir = compile_expr(left, ctx);
                let rhs_ir = compile_expr(right, ctx);
                let opt_idx = ctx.resolve_global("Option").unwrap_or(u16::MAX);
                let unwrap_or = IRExpr::FieldAccess(
                    Box::new(IRExpr::Global(opt_idx, Type::Unknown)),
                    "unwrap_or".to_string(),
                    Type::Unknown,
                );
                return IRExpr::Call(Box::new(unwrap_or), vec![lhs_ir, rhs_ir], Type::Unknown);
            }
            let left = compile_expr(left, ctx);
            let right = compile_expr(right, ctx);
            let ty = match op {
                crate::ast::BinOp::Eq
                | crate::ast::BinOp::NotEq
                | crate::ast::BinOp::Lt
                | crate::ast::BinOp::Gt
                | crate::ast::BinOp::LtEq
                | crate::ast::BinOp::GtEq
                | crate::ast::BinOp::And
                | crate::ast::BinOp::Or => Type::Bool,
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
        Expr::FString(parts, _) => compile_fstring(parts, ctx),
        Expr::EmitExpr(inner, _) => IRExpr::Emit(Box::new(compile_expr(inner, ctx)), Type::Unit),
    }
}

fn compile_fstring(parts: &[FStringPart], ctx: &mut CompileCtx) -> IRExpr {
    let string_idx = ctx.resolve_global("String").unwrap_or(u16::MAX);
    let debug_idx = ctx.resolve_global("Debug").unwrap_or(u16::MAX);
    let mut acc: Option<IRExpr> = None;
    for part in parts {
        let next = match part {
            FStringPart::Lit(s) => IRExpr::Lit(Lit::Str(s.clone()), Type::String),
            FStringPart::Expr(expr) => {
                let inner = compile_expr(expr, ctx);
                if matches!(inner.ty(), Type::String) {
                    inner
                } else {
                    IRExpr::Call(
                        Box::new(IRExpr::FieldAccess(
                            Box::new(IRExpr::Global(debug_idx, Type::Unknown)),
                            "show".into(),
                            Type::Unknown,
                        )),
                        vec![inner],
                        Type::String,
                    )
                }
            }
        };
        acc = Some(match acc {
            None => next,
            Some(prev) => IRExpr::Call(
                Box::new(IRExpr::FieldAccess(
                    Box::new(IRExpr::Global(string_idx, Type::Unknown)),
                    "concat".into(),
                    Type::Unknown,
                )),
                vec![prev, next],
                Type::String,
            ),
        });
    }
    acc.unwrap_or_else(|| IRExpr::Lit(Lit::Str(String::new()), Type::String))
}

fn compile_stmt_into(stmt: &Stmt, ctx: &mut CompileCtx, out: &mut Vec<IRStmt>) {
    match stmt {
        Stmt::Bind(bind) => match &bind.pattern {
            Pattern::Record(fields, _) => {
                let expr_ir = compile_expr(&bind.expr, ctx);
                let tmp_name = format!("$pat{}", ctx.anon_counter);
                ctx.anon_counter = ctx.anon_counter.saturating_add(1);
                let tmp_slot = ctx.define_local_with_ty(tmp_name.clone(), expr_ir.ty().clone());
                out.push(IRStmt::Bind(tmp_slot, expr_ir));
                for field in fields {
                    match field {
                        PatternField::Pun(name, span) => {
                            let field_expr = Expr::FieldAccess(
                                Box::new(Expr::Ident(tmp_name.clone(), span.clone())),
                                name.clone(),
                                span.clone(),
                            );
                            let nested = BindStmt {
                                pattern: Pattern::Bind(name.clone(), span.clone()),
                                annotated_ty: None,
                                expr: field_expr,
                                span: span.clone(),
                            };
                            compile_stmt_into(&Stmt::Bind(nested), ctx, out);
                        }
                        PatternField::Alias(field_name, pattern, span) => {
                            let field_expr = Expr::FieldAccess(
                                Box::new(Expr::Ident(tmp_name.clone(), span.clone())),
                                field_name.clone(),
                                span.clone(),
                            );
                            let nested = BindStmt {
                                pattern: (**pattern).clone(),
                                annotated_ty: None,
                                expr: field_expr,
                                span: span.clone(),
                            };
                            compile_stmt_into(&Stmt::Bind(nested), ctx, out);
                        }
                        PatternField::Wildcard(_) => {}
                    }
                }
            }
            Pattern::Variant(name, Some(inner), span) => {
                let expr_ir = compile_expr(&bind.expr, ctx);
                let tmp_name = format!("$pat{}", ctx.anon_counter);
                ctx.anon_counter = ctx.anon_counter.saturating_add(1);
                let tmp_slot = ctx.define_local_with_ty(tmp_name.clone(), expr_ir.ty().clone());
                out.push(IRStmt::Bind(tmp_slot, expr_ir));

                let payload_name = format!("$pat{}", ctx.anon_counter);
                ctx.anon_counter = ctx.anon_counter.saturating_add(1);
                let match_expr = Expr::Match(
                    Box::new(Expr::Ident(tmp_name.clone(), span.clone())),
                    vec![
                        MatchArm {
                            pattern: Pattern::Variant(
                                name.clone(),
                                Some(Box::new(Pattern::Bind(payload_name.clone(), span.clone()))),
                                span.clone(),
                            ),
                            guard: None,
                            body: Expr::Ident(payload_name.clone(), span.clone()),
                            span: span.clone(),
                        },
                        MatchArm {
                            pattern: Pattern::Wildcard(span.clone()),
                            guard: None,
                            body: Expr::Lit(Lit::Unit, span.clone()),
                            span: span.clone(),
                        },
                    ],
                    span.clone(),
                );
                let nested = BindStmt {
                    pattern: (**inner).clone(),
                    annotated_ty: None,
                    expr: match_expr,
                    span: span.clone(),
                };
                compile_stmt_into(&Stmt::Bind(nested), ctx, out);
            }
            Pattern::Bind(name, _) => {
                let expr_ir = compile_expr(&bind.expr, ctx);
                let slot = ctx.define_local_with_ty(name.clone(), expr_ir.ty().clone());
                out.push(IRStmt::Bind(slot, expr_ir));
            }
            _ => {
                let expr_ir = compile_expr(&bind.expr, ctx);
                let slot = ctx.define_pattern_slot();
                out.push(IRStmt::Bind(slot, expr_ir));
            }
        },
        Stmt::Chain(chain) => {
            let expr_ir = compile_expr(&chain.expr, ctx);
            let slot = if let Some(slot) = ctx.resolve_local(&chain.name) {
                slot
            } else {
                ctx.define_local_with_ty(chain.name.clone(), expr_ir.ty().clone())
            };
            out.push(IRStmt::Chain(slot, expr_ir));
        }
        Stmt::Yield(yield_stmt) => out.push(IRStmt::Yield(compile_expr(&yield_stmt.expr, ctx))),
        Stmt::Expr(expr) => out.push(IRStmt::Expr(compile_expr(expr, ctx))),
        Stmt::ForIn(f) => {
            // Desugar: `for x in iter { body }` → `List.fold(iter, Unit, |$acc, x| { body; Unit })`
            let closure_name = format!("$for_closure{}", ctx.closure_counter);
            ctx.closure_counter = ctx.closure_counter.saturating_add(1);
            let params = vec!["$acc".to_string(), f.var.clone()];
            let mut bound: HashSet<String> = params.iter().cloned().collect();
            let mut free = HashSet::new();
            collect_free_vars_block(&f.body, &mut bound, &mut free);
            let mut captures: Vec<(String, u16)> = free
                .into_iter()
                .filter_map(|name| ctx.resolve_local(&name).map(|slot| (name, slot)))
                .collect();
            captures.sort_by(|a, b| a.0.cmp(&b.0));
            let saved_next = ctx.next_slot;
            let saved_anon = ctx.anon_counter;
            let saved_locals = std::mem::take(&mut ctx.locals);
            let saved_local_tys = std::mem::take(&mut ctx.local_tys);
            ctx.next_slot = 0;
            ctx.anon_counter = 0;
            ctx.push_scope();
            for (name, _) in &captures {
                ctx.define_local(name.clone());
            }
            for param in &params {
                ctx.define_local(param.clone());
            }
            let body_ir = compile_block(&f.body, ctx);
            let local_count = ctx.next_slot as usize;
            ctx.locals = saved_locals;
            ctx.local_tys = saved_local_tys;
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
                param_tys: vec![Type::Unknown; captures.len() + params.len()],
                local_count,
                effects: Vec::new(),
                return_ty: body_ir.ty().clone(),
                body: body_ir,
            });
            let closure_ir = IRExpr::Closure(
                global_idx,
                captures
                    .into_iter()
                    .map(|(_, slot)| IRExpr::Local(slot, Type::Unknown))
                    .collect(),
                Type::Unknown,
            );
            let iter_ir = compile_expr(&f.iter, ctx);
            let list_idx = ctx.resolve_global("List").unwrap_or(u16::MAX);
            let fold_access = IRExpr::FieldAccess(
                Box::new(IRExpr::Global(list_idx, Type::Unknown)),
                "fold".to_string(),
                Type::Unknown,
            );
            out.push(IRStmt::Expr(IRExpr::Call(
                Box::new(fold_access),
                vec![iter_ir, IRExpr::Lit(Lit::Unit, Type::Unit), closure_ir],
                Type::Unit,
            )));
        }
    }
}

fn compile_arm(arm: &MatchArm, ctx: &mut CompileCtx) -> IRArm {
    ctx.push_scope();
    let pattern = compile_pattern(&arm.pattern, ctx);
    let guard = arm.guard.as_ref().map(|g| compile_expr(g, ctx));
    let body = compile_expr(&arm.body, ctx);
    ctx.pop_scope();
    IRArm {
        pattern,
        guard,
        body,
    }
}

pub fn compile_pattern(pat: &Pattern, ctx: &mut CompileCtx) -> IRPattern {
    match pat {
        Pattern::Wildcard(_) => IRPattern::Wildcard,
        Pattern::Lit(lit, _) => IRPattern::Lit(lit.clone()),
        Pattern::Bind(name, _) => IRPattern::Bind(ctx.define_local(name.clone())),
        Pattern::Variant(name, inner, _) => {
            let normalized = match name.as_str() {
                "Ok" => "ok",
                "Err" => "err",
                "Some" => "some",
                "None" => "none",
                other => other,
            };
            IRPattern::Variant(
                normalized.to_string(),
                inner.as_ref().map(|p| Box::new(compile_pattern(p, ctx))),
            )
        }
        Pattern::Record(fields, _) => IRPattern::Record(
            fields
                .iter()
                .filter_map(|field| match field {
                    PatternField::Pun(name, _) => Some((
                        name.clone(),
                        IRPattern::Bind(ctx.define_local(name.clone())),
                    )),
                    PatternField::Alias(name, pattern, _) => {
                        Some((name.clone(), compile_pattern(pattern, ctx)))
                    }
                    PatternField::Wildcard(_) => None,
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
            ("Map", [k, v]) => {
                Type::Map(Box::new(lower_type_expr(k)), Box::new(lower_type_expr(v)))
            }
            ("Option", [inner]) => Type::Option(Box::new(lower_type_expr(inner))),
            ("Result", [ok, err]) => Type::Result(
                Box::new(lower_type_expr(ok)),
                Box::new(lower_type_expr(err)),
            ),
            _ if args.is_empty() => match name.as_str() {
                "Bool" => Type::Bool,
                "Int" => Type::Int,
                "Float" => Type::Float,
                "String" => Type::String,
                "Unit" => Type::Unit,
                _ => {
                    if name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        Type::Var(name.clone())
                    } else {
                        Type::Named(name.clone(), Vec::new())
                    }
                }
            },
            _ => Type::Named(name.clone(), args.iter().map(lower_type_expr).collect()),
        },
        TypeExpr::Optional(inner, _) => Type::Option(Box::new(lower_type_expr(inner))),
        TypeExpr::Fallible(inner, _) => {
            Type::Result(Box::new(lower_type_expr(inner)), Box::new(Type::String))
        }
        TypeExpr::Arrow(input, output, _) => Type::Arrow(
            Box::new(lower_type_expr(input)),
            Box::new(lower_type_expr(output)),
        ),
        TypeExpr::TrfFn {
            input,
            output,
            effects,
            ..
        } => Type::Trf(
            Box::new(lower_type_expr(input)),
            Box::new(lower_type_expr(output)),
            effects.clone(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AbstractFlwDef, CompileCtx, FlwBindingDef, FlwSlot, compile_flw_binding_def,
        compile_program,
    };
    use crate::ast::TypeExpr;
    use crate::frontend::lexer::Lexer;
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Type;
    use crate::middle::ir::{IRExpr, IRGlobalKind, IRStmt};

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

stage Inc: Int -> Int = |x| { x + 1 }
seq Bump = Inc |> Inc

public fn main() -> Int {
    bind result <- 1 |> Bump
    result
}
"#,
        );

        assert_eq!(ir.fns.len(), 3);
        assert!(
            ir.globals
                .iter()
                .any(|g| g.name == "Inc" && matches!(g.kind, IRGlobalKind::Fn(_)))
        );
        assert!(
            ir.globals
                .iter()
                .any(|g| g.name == "main" && matches!(g.kind, IRGlobalKind::Fn(_)))
        );
        assert!(
            ir.globals
                .iter()
                .any(|g| g.name == "Bump" && matches!(g.kind, IRGlobalKind::Fn(_)))
        );
        assert!(
            ir.globals
                .iter()
                .any(|g| g.name == "North" && matches!(g.kind, IRGlobalKind::VariantCtor))
        );
        assert!(
            ir.globals
                .iter()
                .any(|g| g.name == "South" && matches!(g.kind, IRGlobalKind::VariantCtor))
        );
    }

    #[test]
    fn compile_pipeline_lowers_to_nested_calls() {
        let ir = compile_source(
            r#"
stage Inc: Int -> Int = |x| { x + 1 }

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
        assert!(
            ir.globals
                .iter()
                .any(|g| g.name == "main" && matches!(g.kind, IRGlobalKind::Fn(0)))
        );
        assert!(
            ir.globals
                .iter()
                .any(|g| g.name.starts_with("$closure") && matches!(g.kind, IRGlobalKind::Fn(1)))
        );
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

        let closure_fn = ir
            .fns
            .iter()
            .find(|f| f.name.starts_with("$closure"))
            .expect("closure fn");
        assert_eq!(closure_fn.param_count, 2);
    }

    #[test]
    fn compile_flw_binding_exec_ok() {
        let ir = compile_source(
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

        assert!(
            ir.globals
                .iter()
                .any(|g| g.name == "UserImport" && matches!(g.kind, IRGlobalKind::Fn(_)))
        );
        let flw_fn = ir
            .fns
            .iter()
            .find(|f| f.name == "UserImport")
            .expect("UserImport fn");
        assert_eq!(flw_fn.param_count, 1);
        assert!(matches!(flw_fn.param_tys.as_slice(), [Type::String]));
        assert!(matches!(flw_fn.return_ty, Type::Int));
        assert!(flw_fn.effects.contains(&crate::ast::Effect::Db));
    }

    #[test]
    fn compile_flw_binding_local_uses_call_trf_local() {
        let span = crate::frontend::lexer::Span::dummy();
        let template = AbstractFlwDef {
            visibility: None,
            name: "SavePipeline".into(),
            type_params: vec!["Row".into()],
            slots: vec![FlwSlot {
                name: "save".into(),
                abstract_trf_ty: None,
                input_ty: TypeExpr::Named("Row".into(), vec![], span.clone()),
                output_ty: TypeExpr::Named("Int".into(), vec![], span.clone()),
                effects: vec![crate::ast::Effect::Db],
                span: span.clone(),
            }],
            span: span.clone(),
        };
        let fd = FlwBindingDef {
            visibility: None,
            name: "Injected".into(),
            template: "SavePipeline".into(),
            type_args: vec![TypeExpr::Named("UserRow".into(), vec![], span.clone())],
            bindings: vec![("save".into(), crate::ast::SlotImpl::Local("save".into()))],
            span,
        };
        let mut ctx = CompileCtx::new();
        ctx.push_scope();
        let slot = ctx.define_local("save");
        let compiled = compile_flw_binding_def(&fd, &template, &mut ctx);
        match compiled.body {
            IRExpr::CallTrfLocal { local, .. } => assert_eq!(local, slot),
            other => panic!("expected CallTrfLocal, got {:?}", other),
        }
    }

    #[test]
    fn compile_flw_binding_partial_skips_fn_codegen() {
        let ir = compile_source(
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
        );

        assert!(
            ir.globals
                .iter()
                .any(|g| g.name == "PartialImport" && matches!(g.kind, IRGlobalKind::Builtin))
        );
        assert!(!ir.fns.iter().any(|f| f.name == "PartialImport"));
    }
}
