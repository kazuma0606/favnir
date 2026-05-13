#![allow(dead_code)]
use std::collections::HashMap;
use std::fmt;

use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, DataSection, ElementSection, Elements, EntityType,
    ExportKind, ExportSection, Function, FunctionSection, GlobalSection, GlobalType, ImportSection,
    Instruction, MemArg, MemorySection, MemoryType, Module, RefType, TableSection, TableType,
    TypeSection, ValType,
};

use crate::ast::Effect;
use crate::middle::checker::Type;
use crate::middle::ir::{IRExpr, IRGlobalKind, IRProgram, IRStmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmCodegenError {
    UnsupportedType(String),
    UnsupportedExpr(String),
    UnsupportedMainSignature,
}

impl WasmCodegenError {
    pub fn code(&self) -> &str {
        match self {
            WasmCodegenError::UnsupportedType(..) => "W001",
            WasmCodegenError::UnsupportedExpr(..) => "W002",
            WasmCodegenError::UnsupportedMainSignature => "W003",
        }
    }
}

impl fmt::Display for WasmCodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmCodegenError::UnsupportedType(message) => {
                write!(f, "error[W001]: WASM codegen unsupported type: {message}")
            }
            WasmCodegenError::UnsupportedExpr(message) => {
                write!(
                    f,
                    "error[W002]: WASM codegen unsupported expression: {message}"
                )
            }
            WasmCodegenError::UnsupportedMainSignature => {
                write!(
                    f,
                    "error[W003]: WASM codegen requires `public fn main() -> Unit !Io`"
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostImport {
    IoPrintln,
    IoPrint,
    IoPrintlnInt,
    IoPrintlnFloat,
    IoPrintlnBool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmLocal {
    Single(u32),
    StringPtrLen(u32, u32),
    /// Closure value stored as (fn_table_idx: i32, env_ptr: i32) pair.
    FnTableEnv {
        fn_idx_local: u32,
        env_ptr_local: u32,
        wrapper_type_idx: u32,
    },
}

#[derive(Debug)]
pub struct WasmCodegenCtx<'a> {
    pub fn_to_wasm_idx: HashMap<usize, u32>,
    pub builtin_to_wasm_idx: HashMap<String, u32>,
    pub bump_alloc_fn_idx: u32,
    pub str_to_offset: HashMap<String, u32>,
    pub globals: &'a [crate::middle::ir::IRGlobal],
    pub fns: &'a [crate::middle::ir::IRFnDef],
    /// Maps closure global_idx → table element index.
    pub closure_table_idx: HashMap<u16, u32>,
    /// Maps closure global_idx → wrapper function type index.
    pub closure_wrapper_type_idx: HashMap<u16, u32>,
}

const HEAP_PTR_INITIAL: i32 = 65536;
const HEAP_PTR_GLOBAL_IDX: u32 = 0;

fn build_heap_ptr_global_section() -> GlobalSection {
    let mut section = GlobalSection::new();
    section.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(HEAP_PTR_INITIAL),
    );
    section
}

fn build_bump_alloc_function() -> Function {
    let mut func = Function::new(vec![]);
    func.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_IDX));
    func.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_IDX));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_IDX));
    func.instruction(&Instruction::End);
    func
}

/// Collect info about all closures in the program: global_idx → (fn_idx, captures_count).
fn collect_closure_info(ir: &IRProgram) -> HashMap<u16, (usize, usize)> {
    let mut map = HashMap::new();
    for fn_def in &ir.fns {
        walk_closures_in_expr(&fn_def.body, ir, &mut map);
    }
    map
}

fn walk_closures_in_expr(expr: &IRExpr, ir: &IRProgram, map: &mut HashMap<u16, (usize, usize)>) {
    match expr {
        IRExpr::Closure(global_idx, captures, _) => {
            if let Some(global) = ir.globals.get(*global_idx as usize) {
                if let IRGlobalKind::Fn(fn_idx) = global.kind {
                    map.entry(*global_idx).or_insert((fn_idx, captures.len()));
                }
            }
            for cap in captures {
                walk_closures_in_expr(cap, ir, map);
            }
        }
        IRExpr::Lit(_, _) | IRExpr::Local(_, _) | IRExpr::Global(_, _) | IRExpr::TrfRef(_, _) => {}
        IRExpr::CallTrfLocal { arg, .. } => {
            walk_closures_in_expr(arg, ir, map);
        }
        IRExpr::Call(callee, args, _) => {
            walk_closures_in_expr(callee, ir, map);
            for arg in args {
                walk_closures_in_expr(arg, ir, map);
            }
        }
        IRExpr::Collect(inner, _) | IRExpr::Emit(inner, _) | IRExpr::FieldAccess(inner, _, _) => {
            walk_closures_in_expr(inner, ir, map);
        }
        IRExpr::Block(stmts, final_expr, _) => {
            for stmt in stmts {
                match stmt {
                    IRStmt::Bind(_, e)
                    | IRStmt::Chain(_, e)
                    | IRStmt::Yield(e)
                    | IRStmt::Expr(e) => walk_closures_in_expr(e, ir, map),
                    IRStmt::TrackLine(_) => {}
                }
            }
            walk_closures_in_expr(final_expr, ir, map);
        }
        IRExpr::If(cond, then_expr, else_expr, _) => {
            walk_closures_in_expr(cond, ir, map);
            walk_closures_in_expr(then_expr, ir, map);
            walk_closures_in_expr(else_expr, ir, map);
        }
        IRExpr::Match(subject, arms, _) => {
            walk_closures_in_expr(subject, ir, map);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    walk_closures_in_expr(guard, ir, map);
                }
                walk_closures_in_expr(&arm.body, ir, map);
            }
        }
        IRExpr::BinOp(_, lhs, rhs, _) => {
            walk_closures_in_expr(lhs, ir, map);
            walk_closures_in_expr(rhs, ir, map);
        }
        IRExpr::RecordConstruct(fields, _) => {
            for (_, v) in fields {
                walk_closures_in_expr(v, ir, map);
            }
        }
    }
}

/// Find all `Bind(slot, Closure(global_idx, ...))` in a function body.
/// Returns slot → global_idx.
fn scan_closure_bound_slots(expr: &IRExpr) -> HashMap<u16, u16> {
    let mut map = HashMap::new();
    scan_closure_bound_slots_walk(expr, &mut map);
    map
}

fn scan_closure_bound_slots_walk(expr: &IRExpr, map: &mut HashMap<u16, u16>) {
    match expr {
        IRExpr::Block(stmts, final_expr, _) => {
            for stmt in stmts {
                if let IRStmt::Bind(slot, IRExpr::Closure(global_idx, _, _)) = stmt {
                    map.insert(*slot, *global_idx);
                }
                match stmt {
                    IRStmt::Bind(_, e)
                    | IRStmt::Chain(_, e)
                    | IRStmt::Yield(e)
                    | IRStmt::Expr(e) => scan_closure_bound_slots_walk(e, map),
                    IRStmt::TrackLine(_) => {}
                }
            }
            scan_closure_bound_slots_walk(final_expr, map);
        }
        IRExpr::If(cond, then_expr, else_expr, _) => {
            scan_closure_bound_slots_walk(cond, map);
            scan_closure_bound_slots_walk(then_expr, map);
            scan_closure_bound_slots_walk(else_expr, map);
        }
        _ => {}
    }
}

/// Build a closure wrapper function.
///
/// The wrapper has signature `(env_ptr: i32, actual_params: i64...) -> return_ty`.
/// It loads captures from linear memory at `env_ptr + i*8` and then calls the
/// original lifted closure function.
fn build_closure_wrapper_function(
    fn_def: &crate::middle::ir::IRFnDef,
    captures_count: usize,
    original_wasm_fn_idx: u32,
) -> Function {
    let actual_params_count = fn_def.param_count.saturating_sub(captures_count);
    let mut func = Function::new(vec![]);

    // Load each capture from env at offset i*8 (env_ptr is local 0)
    for i in 0..captures_count {
        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::I64Load(MemArg {
            offset: (i * 8) as u64,
            align: 3,
            memory_index: 0,
        }));
    }

    // Push actual params (locals 1..=actual_params_count, each i64)
    for i in 0..actual_params_count {
        func.instruction(&Instruction::LocalGet((i + 1) as u32));
    }

    func.instruction(&Instruction::Call(original_wasm_fn_idx));
    func.instruction(&Instruction::End);
    func
}

fn single_wasm_valtype(ty: &Type) -> Result<Option<ValType>, WasmCodegenError> {
    let vals = favnir_type_to_wasm_results(ty)?;
    match vals.as_slice() {
        [] => Ok(None),
        [only] => Ok(Some(*only)),
        _ => Err(WasmCodegenError::UnsupportedType(format!(
            "multi-value lowering not supported for local type: {ty:?}"
        ))),
    }
}

fn block_type_for(ty: &Type) -> Result<BlockType, WasmCodegenError> {
    let vals = favnir_type_to_wasm_results(ty)?;
    match vals.as_slice() {
        [] => Ok(BlockType::Empty),
        [only] => Ok(BlockType::Result(*only)),
        _ => Err(WasmCodegenError::UnsupportedExpr(format!(
            "multi-value block result in wasm MVP: {ty:?}"
        ))),
    }
}

fn infer_binop_type(op: &crate::ast::BinOp, lhs: &Type, rhs: &Type, result: &Type) -> Type {
    for candidate in [lhs, rhs, result] {
        if !matches!(candidate, Type::Unknown) {
            return candidate.clone();
        }
    }

    match op {
        crate::ast::BinOp::Add
        | crate::ast::BinOp::Sub
        | crate::ast::BinOp::Mul
        | crate::ast::BinOp::Div
        | crate::ast::BinOp::Lt
        | crate::ast::BinOp::Gt
        | crate::ast::BinOp::LtEq
        | crate::ast::BinOp::GtEq => Type::Int,
        crate::ast::BinOp::Eq
        | crate::ast::BinOp::NotEq
        | crate::ast::BinOp::And
        | crate::ast::BinOp::Or => Type::Bool,
        crate::ast::BinOp::NullCoalesce => unreachable!("?? desugared before IR"),
    }
}

fn first_known_type<'a>(types: &[&'a Type]) -> &'a Type {
    types
        .iter()
        .copied()
        .find(|ty| !matches!(ty, Type::Unknown))
        .unwrap_or(types[0])
}

fn resolved_expr_type(expr: &IRExpr, ctx: &WasmCodegenCtx<'_>) -> Type {
    match expr {
        IRExpr::CallTrfLocal { ty, .. } if matches!(ty, Type::Unknown) => ty.clone(),
        IRExpr::Call(callee, _, ty) if matches!(ty, Type::Unknown) => {
            if let Some(builtin) = builtin_call_name(callee, ctx.globals) {
                return match builtin.as_str() {
                    "IO.println" | "IO.print" | "IO.println_int" | "IO.println_float"
                    | "IO.println_bool" => Type::Unit,
                    _ => ty.clone(),
                };
            }

            if let IRExpr::Global(fn_global_idx, _) = callee.as_ref() {
                if let Some(global) = ctx.globals.get(*fn_global_idx as usize) {
                    if let IRGlobalKind::Fn(fn_idx) = global.kind {
                        if let Some(fn_def) = ctx.fns.get(fn_idx) {
                            return fn_def.return_ty.clone();
                        }
                    }
                }
            }

            ty.clone()
        }
        IRExpr::Block(_, final_expr, ty) if matches!(ty, Type::Unknown) => {
            resolved_expr_type(final_expr, ctx)
        }
        IRExpr::If(_, then_expr, else_expr, ty) if matches!(ty, Type::Unknown) => {
            let then_ty = resolved_expr_type(then_expr, ctx);
            let else_ty = resolved_expr_type(else_expr, ctx);
            first_known_type(&[ty, &then_ty, &else_ty]).clone()
        }
        IRExpr::BinOp(op, lhs, rhs, ty) if matches!(ty, Type::Unknown) => {
            let lhs_ty = resolved_expr_type(lhs, ctx);
            let rhs_ty = resolved_expr_type(rhs, ctx);
            let inferred = infer_binop_type(op, &lhs_ty, &rhs_ty, ty);
            inferred
        }
        _ => expr.ty().clone(),
    }
}

fn host_import_signature(import: HostImport) -> (Vec<ValType>, Vec<ValType>) {
    match import {
        HostImport::IoPrintln | HostImport::IoPrint => (vec![ValType::I32, ValType::I32], vec![]),
        HostImport::IoPrintlnInt => (vec![ValType::I64], vec![]),
        HostImport::IoPrintlnFloat => (vec![ValType::F64], vec![]),
        HostImport::IoPrintlnBool => (vec![ValType::I32], vec![]),
    }
}

pub fn ensure_supported_main_signature(ir: &IRProgram) -> Result<(), WasmCodegenError> {
    let Some(main_idx) = ir
        .globals
        .iter()
        .find(|g| g.name == "main")
        .and_then(|g| match g.kind {
            IRGlobalKind::Fn(idx) => Some(idx),
            _ => None,
        })
    else {
        return Ok(());
    };
    let Some(main_fn) = ir.fns.get(main_idx) else {
        return Err(WasmCodegenError::UnsupportedMainSignature);
    };
    let io_only = main_fn.effects.len() == 1 && matches!(main_fn.effects[0], Effect::Io);
    if main_fn.param_count == 0 && main_fn.return_ty == Type::Unit && io_only {
        Ok(())
    } else {
        Err(WasmCodegenError::UnsupportedMainSignature)
    }
}

pub fn build_type_section(
    ir: &IRProgram,
    imports: &[HostImport],
) -> Result<(TypeSection, HashMap<usize, u32>, u32), WasmCodegenError> {
    ensure_supported_main_signature(ir)?;

    let mut section = TypeSection::new();
    let mut next_type_idx = 0u32;

    for import in imports {
        let (params, results) = host_import_signature(*import);
        section.ty().function(params, results);
        next_type_idx += 1;
    }

    let mut fn_to_type_idx = HashMap::new();
    for (fn_idx, fn_def) in ir.fns.iter().enumerate() {
        let mut params = Vec::new();
        for ty in &fn_def.param_tys {
            params.extend(favnir_type_to_wasm_params(ty)?);
        }
        let results = favnir_type_to_wasm_results(&fn_def.return_ty).map_err(|err| match err {
            WasmCodegenError::UnsupportedType(message) => {
                WasmCodegenError::UnsupportedType(format!("{} (in fn {})", message, fn_def.name))
            }
            other => other,
        })?;
        section.ty().function(params, results);
        fn_to_type_idx.insert(fn_idx, next_type_idx);
        next_type_idx += 1;
    }

    let bump_alloc_type_idx = next_type_idx;
    section.ty().function([ValType::I32], [ValType::I32]);

    Ok((section, fn_to_type_idx, bump_alloc_type_idx))
}

pub fn favnir_type_to_wasm_results(ty: &Type) -> Result<Vec<ValType>, WasmCodegenError> {
    match ty {
        Type::Unit => Ok(vec![]),
        Type::Int => Ok(vec![ValType::I64]),
        Type::Float => Ok(vec![ValType::F64]),
        Type::Bool => Ok(vec![ValType::I32]),
        Type::String => Ok(vec![ValType::I32, ValType::I32]),
        Type::Unknown => Ok(vec![ValType::I64]),
        Type::Error => Err(WasmCodegenError::UnsupportedType(format!(
            "unknown return type: {ty:?}"
        ))),
        other => Err(WasmCodegenError::UnsupportedType(format!(
            "unsupported return type: {other:?}"
        ))),
    }
}

pub fn favnir_type_to_wasm_params(ty: &Type) -> Result<Vec<ValType>, WasmCodegenError> {
    match ty {
        Type::Unit => Ok(vec![]),
        Type::Int => Ok(vec![ValType::I64]),
        Type::Float => Ok(vec![ValType::F64]),
        Type::Bool => Ok(vec![ValType::I32]),
        Type::String => Ok(vec![ValType::I32, ValType::I32]),
        Type::Unknown => Ok(vec![ValType::I64]),
        Type::Error => Err(WasmCodegenError::UnsupportedType(format!(
            "unknown parameter type: {ty:?}"
        ))),
        other => Err(WasmCodegenError::UnsupportedType(format!(
            "unsupported parameter type: {other:?}"
        ))),
    }
}

pub fn collect_local_types(expr: &IRExpr, map: &mut HashMap<u16, Type>) {
    match expr {
        IRExpr::Lit(_, _) | IRExpr::Global(_, _) | IRExpr::TrfRef(_, _) => {}
        IRExpr::Local(idx, ty) => {
            map.entry(*idx).or_insert_with(|| ty.clone());
        }
        IRExpr::CallTrfLocal { arg, .. } => {
            collect_local_types(arg, map);
        }
        IRExpr::Call(callee, args, _) => {
            collect_local_types(callee, map);
            for arg in args {
                collect_local_types(arg, map);
            }
        }
        IRExpr::Collect(callee, _)
        | IRExpr::Emit(callee, _)
        | IRExpr::FieldAccess(callee, _, _) => {
            collect_local_types(callee, map);
        }
        IRExpr::Block(stmts, final_expr, _) => {
            for stmt in stmts {
                collect_local_types_stmt(stmt, map);
            }
            collect_local_types(final_expr, map);
        }
        IRExpr::If(cond, then_expr, else_expr, _) => {
            collect_local_types(cond, map);
            collect_local_types(then_expr, map);
            collect_local_types(else_expr, map);
        }
        IRExpr::Match(subject, arms, _) => {
            collect_local_types(subject, map);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_local_types(guard, map);
                }
                collect_local_types(&arm.body, map);
            }
        }
        IRExpr::BinOp(_, lhs, rhs, _) => {
            collect_local_types(lhs, map);
            collect_local_types(rhs, map);
        }
        IRExpr::Closure(_, captures, _) => {
            for capture in captures {
                collect_local_types(capture, map);
            }
        }
        IRExpr::RecordConstruct(fields, _) => {
            for (_, value) in fields {
                collect_local_types(value, map);
            }
        }
    }
}

pub fn collect_local_types_stmt(stmt: &IRStmt, map: &mut HashMap<u16, Type>) {
    match stmt {
        IRStmt::Bind(slot, expr) | IRStmt::Chain(slot, expr) => {
            map.entry(*slot).or_insert_with(|| expr.ty().clone());
            collect_local_types(expr, map);
        }
        IRStmt::Yield(expr) | IRStmt::Expr(expr) => collect_local_types(expr, map),
        IRStmt::TrackLine(_) => {}
    }
}

fn host_imports() -> &'static [(&'static str, HostImport)] {
    &[
        ("IO.println", HostImport::IoPrintln),
        ("IO.print", HostImport::IoPrint),
        ("IO.println_int", HostImport::IoPrintlnInt),
        ("IO.println_float", HostImport::IoPrintlnFloat),
        ("IO.println_bool", HostImport::IoPrintlnBool),
    ]
}

fn host_import_symbol(import: HostImport) -> &'static str {
    match import {
        HostImport::IoPrintln => "io_println",
        HostImport::IoPrint => "io_print",
        HostImport::IoPrintlnInt => "io_println_int",
        HostImport::IoPrintlnFloat => "io_println_float",
        HostImport::IoPrintlnBool => "io_println_bool",
    }
}

fn builtin_call_name(expr: &IRExpr, globals: &[crate::middle::ir::IRGlobal]) -> Option<String> {
    let IRExpr::FieldAccess(obj, field, _) = expr else {
        return None;
    };
    let IRExpr::Global(idx, _) = obj.as_ref() else {
        return None;
    };
    let global = globals.get(*idx as usize)?;
    matches!(global.kind, IRGlobalKind::Builtin).then(|| format!("{}.{}", global.name, field))
}

fn collect_expr_string_literals(expr: &IRExpr, ordered: &mut Vec<String>) {
    match expr {
        IRExpr::Lit(crate::ast::Lit::Str(value), _) => {
            if !ordered.iter().any(|s| s == value) {
                ordered.push(value.clone());
            }
        }
        IRExpr::Lit(_, _) | IRExpr::Local(_, _) | IRExpr::Global(_, _) | IRExpr::TrfRef(_, _) => {}
        IRExpr::CallTrfLocal { arg, .. } => {
            collect_expr_string_literals(arg, ordered);
        }
        IRExpr::Call(callee, args, _) => {
            collect_expr_string_literals(callee, ordered);
            for arg in args {
                collect_expr_string_literals(arg, ordered);
            }
        }
        IRExpr::Collect(inner, _) | IRExpr::Emit(inner, _) | IRExpr::FieldAccess(inner, _, _) => {
            collect_expr_string_literals(inner, ordered);
        }
        IRExpr::Block(stmts, final_expr, _) => {
            for stmt in stmts {
                collect_stmt_string_literals(stmt, ordered);
            }
            collect_expr_string_literals(final_expr, ordered);
        }
        IRExpr::If(cond, then_expr, else_expr, _) => {
            collect_expr_string_literals(cond, ordered);
            collect_expr_string_literals(then_expr, ordered);
            collect_expr_string_literals(else_expr, ordered);
        }
        IRExpr::Match(subject, arms, _) => {
            collect_expr_string_literals(subject, ordered);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_expr_string_literals(guard, ordered);
                }
                collect_expr_string_literals(&arm.body, ordered);
            }
        }
        IRExpr::BinOp(_, lhs, rhs, _) => {
            collect_expr_string_literals(lhs, ordered);
            collect_expr_string_literals(rhs, ordered);
        }
        IRExpr::Closure(_, captures, _) => {
            for capture in captures {
                collect_expr_string_literals(capture, ordered);
            }
        }
        IRExpr::RecordConstruct(fields, _) => {
            for (_, value) in fields {
                collect_expr_string_literals(value, ordered);
            }
        }
    }
}

fn collect_stmt_string_literals(stmt: &IRStmt, ordered: &mut Vec<String>) {
    match stmt {
        IRStmt::Bind(_, expr)
        | IRStmt::Chain(_, expr)
        | IRStmt::Yield(expr)
        | IRStmt::Expr(expr) => {
            collect_expr_string_literals(expr, ordered);
        }
        IRStmt::TrackLine(_) => {}
    }
}

pub fn collect_string_literals(ir: &IRProgram) -> (Vec<u8>, HashMap<String, u32>) {
    let mut ordered = Vec::new();
    for fn_def in &ir.fns {
        collect_expr_string_literals(&fn_def.body, &mut ordered);
    }

    let mut bytes = Vec::new();
    let mut map = HashMap::new();
    for value in ordered {
        let offset = bytes.len() as u32;
        bytes.extend_from_slice(value.as_bytes());
        map.insert(value, offset);
    }
    (bytes, map)
}

pub fn collect_used_builtins(ir: &IRProgram) -> std::collections::HashSet<String> {
    fn walk_expr(
        expr: &IRExpr,
        globals: &[crate::middle::ir::IRGlobal],
        used: &mut std::collections::HashSet<String>,
    ) {
        if let Some(name) = builtin_call_name(expr, globals) {
            used.insert(name);
            return;
        }
        match expr {
            IRExpr::Lit(_, _)
            | IRExpr::Local(_, _)
            | IRExpr::Global(_, _)
            | IRExpr::TrfRef(_, _) => {}
            IRExpr::CallTrfLocal { arg, .. } => {
                walk_expr(arg, globals, used);
            }
            IRExpr::Call(callee, args, _) => {
                walk_expr(callee, globals, used);
                for arg in args {
                    walk_expr(arg, globals, used);
                }
            }
            IRExpr::Collect(inner, _)
            | IRExpr::Emit(inner, _)
            | IRExpr::FieldAccess(inner, _, _) => {
                walk_expr(inner, globals, used);
            }
            IRExpr::Block(stmts, final_expr, _) => {
                for stmt in stmts {
                    match stmt {
                        IRStmt::Bind(_, expr)
                        | IRStmt::Chain(_, expr)
                        | IRStmt::Yield(expr)
                        | IRStmt::Expr(expr) => walk_expr(expr, globals, used),
                        IRStmt::TrackLine(_) => {}
                    }
                }
                walk_expr(final_expr, globals, used);
            }
            IRExpr::If(cond, then_expr, else_expr, _) => {
                walk_expr(cond, globals, used);
                walk_expr(then_expr, globals, used);
                walk_expr(else_expr, globals, used);
            }
            IRExpr::Match(subject, arms, _) => {
                walk_expr(subject, globals, used);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        walk_expr(guard, globals, used);
                    }
                    walk_expr(&arm.body, globals, used);
                }
            }
            IRExpr::BinOp(_, lhs, rhs, _) => {
                walk_expr(lhs, globals, used);
                walk_expr(rhs, globals, used);
            }
            IRExpr::Closure(_, captures, _) => {
                for capture in captures {
                    walk_expr(capture, globals, used);
                }
            }
            IRExpr::RecordConstruct(fields, _) => {
                for (_, value) in fields {
                    walk_expr(value, globals, used);
                }
            }
        }
    }

    let mut used = std::collections::HashSet::new();
    for fn_def in &ir.fns {
        walk_expr(&fn_def.body, &ir.globals, &mut used);
    }
    used
}

fn wasm_local_for_type(ty: &Type, next_local_idx: &mut u32) -> Result<WasmLocal, WasmCodegenError> {
    match favnir_type_to_wasm_params(ty)?.as_slice() {
        [] => Err(WasmCodegenError::UnsupportedType(
            "unit locals are not supported in wasm".into(),
        )),
        [_] => {
            let idx = *next_local_idx;
            *next_local_idx += 1;
            Ok(WasmLocal::Single(idx))
        }
        [_, _] if matches!(ty, Type::String) => {
            let ptr = *next_local_idx;
            let len = *next_local_idx + 1;
            *next_local_idx += 2;
            Ok(WasmLocal::StringPtrLen(ptr, len))
        }
        _ => Err(WasmCodegenError::UnsupportedType(format!(
            "unsupported multi-value local type: {ty:?}"
        ))),
    }
}

fn emit_stmt(
    stmt: &IRStmt,
    ctx: &WasmCodegenCtx<'_>,
    slot_map: &HashMap<u16, WasmLocal>,
    func: &mut Function,
) -> Result<(), WasmCodegenError> {
    match stmt {
        IRStmt::Bind(slot, expr) => {
            // Closure binding: allocate env and set (fn_idx, env_ptr) locals.
            if let IRExpr::Closure(global_idx, captures, _) = expr {
                let Some(local) = slot_map.get(slot) else {
                    return Err(WasmCodegenError::UnsupportedExpr(format!(
                        "missing wasm local mapping for closure bind slot {slot}"
                    )));
                };
                let WasmLocal::FnTableEnv {
                    fn_idx_local,
                    env_ptr_local,
                    wrapper_type_idx: _,
                } = *local
                else {
                    return Err(WasmCodegenError::UnsupportedExpr(
                        "closure bind slot must be FnTableEnv".into(),
                    ));
                };

                // Allocate env: captures.len() * 8 bytes
                let env_size = (captures.len() * 8) as i32;
                func.instruction(&Instruction::I32Const(env_size));
                func.instruction(&Instruction::Call(ctx.bump_alloc_fn_idx));
                func.instruction(&Instruction::LocalSet(env_ptr_local));

                // Store each capture into env at offset i*8
                for (i, capture) in captures.iter().enumerate() {
                    func.instruction(&Instruction::LocalGet(env_ptr_local));
                    emit_expr(capture, ctx, slot_map, func)?;
                    func.instruction(&Instruction::I64Store(MemArg {
                        offset: (i * 8) as u64,
                        align: 3,
                        memory_index: 0,
                    }));
                }

                // Set fn_idx_local to the table element index
                let Some(&table_elem_idx) = ctx.closure_table_idx.get(global_idx) else {
                    return Err(WasmCodegenError::UnsupportedExpr(format!(
                        "missing table index for closure global {global_idx}"
                    )));
                };
                func.instruction(&Instruction::I32Const(table_elem_idx as i32));
                func.instruction(&Instruction::LocalSet(fn_idx_local));
                return Ok(());
            }

            emit_expr(expr, ctx, slot_map, func)?;
            let Some(local) = slot_map.get(slot) else {
                return Err(WasmCodegenError::UnsupportedExpr(format!(
                    "missing wasm local mapping for bind slot {slot}"
                )));
            };
            match local {
                WasmLocal::Single(local_idx) => {
                    if single_wasm_valtype(expr.ty())?.is_none() {
                        return Err(WasmCodegenError::UnsupportedExpr(format!(
                            "cannot bind unit expression into local slot {slot}"
                        )));
                    }
                    func.instruction(&Instruction::LocalSet(*local_idx));
                }
                WasmLocal::StringPtrLen(ptr, len) => {
                    func.instruction(&Instruction::LocalSet(*len));
                    func.instruction(&Instruction::LocalSet(*ptr));
                }
                WasmLocal::FnTableEnv { .. } => {
                    return Err(WasmCodegenError::UnsupportedExpr(
                        "FnTableEnv local used for non-closure binding".into(),
                    ));
                }
            }
            Ok(())
        }
        IRStmt::Expr(expr) => {
            emit_expr(expr, ctx, slot_map, func)?;
            if single_wasm_valtype(&resolved_expr_type(expr, ctx))?.is_some() {
                func.instruction(&Instruction::Drop);
            }
            Ok(())
        }
        IRStmt::Chain(_, _) => Err(WasmCodegenError::UnsupportedExpr(
            "chain statement in wasm MVP".into(),
        )),
        IRStmt::Yield(_) => Err(WasmCodegenError::UnsupportedExpr(
            "yield statement in wasm MVP".into(),
        )),
        IRStmt::TrackLine(_) => Ok(()), // no-op in WASM
    }
}

fn emit_expr(
    expr: &IRExpr,
    ctx: &WasmCodegenCtx<'_>,
    slot_map: &HashMap<u16, WasmLocal>,
    func: &mut Function,
) -> Result<(), WasmCodegenError> {
    match expr {
        IRExpr::Lit(lit, _) => match lit {
            crate::ast::Lit::Int(value) => {
                func.instruction(&Instruction::I64Const(*value));
                Ok(())
            }
            crate::ast::Lit::Float(value) => {
                func.instruction(&Instruction::F64Const(*value));
                Ok(())
            }
            crate::ast::Lit::Bool(value) => {
                func.instruction(&Instruction::I32Const(if *value { 1 } else { 0 }));
                Ok(())
            }
            crate::ast::Lit::Unit => Ok(()),
            crate::ast::Lit::Str(value) => {
                let Some(offset) = ctx.str_to_offset.get(value) else {
                    return Err(WasmCodegenError::UnsupportedExpr(format!(
                        "missing data-section offset for string literal: {value}"
                    )));
                };
                func.instruction(&Instruction::I32Const(*offset as i32));
                func.instruction(&Instruction::I32Const(value.len() as i32));
                Ok(())
            }
        },
        IRExpr::Local(slot, _) => {
            let Some(local) = slot_map.get(slot) else {
                return Err(WasmCodegenError::UnsupportedExpr(format!(
                    "missing wasm local mapping for slot {slot}"
                )));
            };
            match local {
                WasmLocal::Single(local_idx) => {
                    func.instruction(&Instruction::LocalGet(*local_idx));
                }
                WasmLocal::StringPtrLen(ptr, len) => {
                    func.instruction(&Instruction::LocalGet(*ptr));
                    func.instruction(&Instruction::LocalGet(*len));
                }
                WasmLocal::FnTableEnv { .. } => {
                    return Err(WasmCodegenError::UnsupportedExpr(
                        "closure local cannot be used as a bare value; use in call position only"
                            .into(),
                    ));
                }
            }
            Ok(())
        }
        IRExpr::TrfRef(_, _) => Err(WasmCodegenError::UnsupportedExpr(
            "trf reference value in wasm MVP".into(),
        )),
        IRExpr::CallTrfLocal { .. } => Err(WasmCodegenError::UnsupportedExpr(
            "local slot injected call in wasm MVP".into(),
        )),
        IRExpr::Global(_, _) => Err(WasmCodegenError::UnsupportedExpr(
            "bare global value expression in wasm MVP".into(),
        )),
        IRExpr::Call(callee, args, _) => {
            if let Some(builtin) = builtin_call_name(callee, ctx.globals) {
                for arg in args {
                    emit_expr(arg, ctx, slot_map, func)?;
                }
                let Some(target) = ctx.builtin_to_wasm_idx.get(&builtin) else {
                    return Err(WasmCodegenError::UnsupportedExpr(format!(
                        "unsupported wasm builtin call: {builtin}"
                    )));
                };
                func.instruction(&Instruction::Call(*target));
                return Ok(());
            }

            // Closure call via call_indirect
            if let IRExpr::Local(slot, _) = callee.as_ref() {
                if let Some(WasmLocal::FnTableEnv {
                    fn_idx_local,
                    env_ptr_local,
                    wrapper_type_idx,
                }) = slot_map.get(slot).copied()
                {
                    func.instruction(&Instruction::LocalGet(env_ptr_local));
                    for arg in args {
                        emit_expr(arg, ctx, slot_map, func)?;
                    }
                    func.instruction(&Instruction::LocalGet(fn_idx_local));
                    func.instruction(&Instruction::CallIndirect {
                        type_index: wrapper_type_idx,
                        table_index: 0,
                    });
                    return Ok(());
                }
            }

            let IRExpr::Global(fn_global_idx, _) = callee.as_ref() else {
                return Err(WasmCodegenError::UnsupportedExpr(
                    "only direct global function calls or closure calls are supported in wasm MVP"
                        .into(),
                ));
            };
            let Some(global) = ctx.globals.get(*fn_global_idx as usize) else {
                return Err(WasmCodegenError::UnsupportedExpr(format!(
                    "global index out of range: {fn_global_idx}"
                )));
            };
            let IRGlobalKind::Fn(fn_idx) = global.kind else {
                return Err(WasmCodegenError::UnsupportedExpr(format!(
                    "non-function global call: {}",
                    global.name
                )));
            };
            for arg in args {
                emit_expr(arg, ctx, slot_map, func)?;
            }
            let Some(target) = ctx.fn_to_wasm_idx.get(&fn_idx) else {
                return Err(WasmCodegenError::UnsupportedExpr(format!(
                    "missing wasm function index for {}",
                    global.name
                )));
            };
            func.instruction(&Instruction::Call(*target));
            Ok(())
        }
        IRExpr::Block(stmts, final_expr, _) => {
            for stmt in stmts {
                emit_stmt(stmt, ctx, slot_map, func)?;
            }
            emit_expr(final_expr, ctx, slot_map, func)
        }
        IRExpr::If(cond, then_expr, else_expr, ty) => {
            emit_expr(cond, ctx, slot_map, func)?;
            let then_ty = resolved_expr_type(then_expr, ctx);
            let else_ty = resolved_expr_type(else_expr, ctx);
            let merge_ty = first_known_type(&[ty, &then_ty, &else_ty]);
            func.instruction(&Instruction::If(block_type_for(merge_ty)?));
            emit_expr(then_expr, ctx, slot_map, func)?;
            func.instruction(&Instruction::Else);
            emit_expr(else_expr, ctx, slot_map, func)?;
            func.instruction(&Instruction::End);
            Ok(())
        }
        IRExpr::BinOp(op, lhs, rhs, _) => {
            let op_ty = infer_binop_type(op, lhs.ty(), rhs.ty(), expr.ty());
            emit_expr(lhs, ctx, slot_map, func)?;
            emit_expr(rhs, ctx, slot_map, func)?;
            match (op, &op_ty) {
                (crate::ast::BinOp::Add, Type::Int) => func.instruction(&Instruction::I64Add),
                (crate::ast::BinOp::Sub, Type::Int) => func.instruction(&Instruction::I64Sub),
                (crate::ast::BinOp::Mul, Type::Int) => func.instruction(&Instruction::I64Mul),
                (crate::ast::BinOp::Div, Type::Int) => func.instruction(&Instruction::I64DivS),
                (crate::ast::BinOp::Eq, Type::Int) => func.instruction(&Instruction::I64Eq),
                (crate::ast::BinOp::NotEq, Type::Int) => func.instruction(&Instruction::I64Ne),
                (crate::ast::BinOp::Lt, Type::Int) => func.instruction(&Instruction::I64LtS),
                (crate::ast::BinOp::Gt, Type::Int) => func.instruction(&Instruction::I64GtS),
                (crate::ast::BinOp::LtEq, Type::Int) => func.instruction(&Instruction::I64LeS),
                (crate::ast::BinOp::GtEq, Type::Int) => func.instruction(&Instruction::I64GeS),
                (crate::ast::BinOp::Add, Type::Float) => func.instruction(&Instruction::F64Add),
                (crate::ast::BinOp::Sub, Type::Float) => func.instruction(&Instruction::F64Sub),
                (crate::ast::BinOp::Mul, Type::Float) => func.instruction(&Instruction::F64Mul),
                (crate::ast::BinOp::Div, Type::Float) => func.instruction(&Instruction::F64Div),
                (crate::ast::BinOp::Eq, Type::Float) => func.instruction(&Instruction::F64Eq),
                (crate::ast::BinOp::NotEq, Type::Float) => func.instruction(&Instruction::F64Ne),
                (crate::ast::BinOp::Lt, Type::Float) => func.instruction(&Instruction::F64Lt),
                (crate::ast::BinOp::Gt, Type::Float) => func.instruction(&Instruction::F64Gt),
                (crate::ast::BinOp::LtEq, Type::Float) => func.instruction(&Instruction::F64Le),
                (crate::ast::BinOp::GtEq, Type::Float) => func.instruction(&Instruction::F64Ge),
                (crate::ast::BinOp::Eq, Type::Bool) => func.instruction(&Instruction::I32Eq),
                (crate::ast::BinOp::NotEq, Type::Bool) => func.instruction(&Instruction::I32Ne),
                (other, ty) => {
                    return Err(WasmCodegenError::UnsupportedExpr(format!(
                        "unsupported wasm binary op {other:?} for type {ty:?}"
                    )));
                }
            };
            Ok(())
        }
        IRExpr::FieldAccess(_, _, _) => Err(WasmCodegenError::UnsupportedExpr(
            "field access outside direct builtin calls".into(),
        )),
        IRExpr::Match(_, _, _) => Err(WasmCodegenError::UnsupportedExpr(
            "match expression in wasm MVP".into(),
        )),
        IRExpr::Closure(_, _, _) => Err(WasmCodegenError::UnsupportedExpr(
            "closure expression in wasm MVP".into(),
        )),
        IRExpr::Collect(_, _) => Err(WasmCodegenError::UnsupportedExpr(
            "collect expression in wasm MVP".into(),
        )),
        IRExpr::Emit(_, _) => Err(WasmCodegenError::UnsupportedExpr(
            "emit expression in wasm MVP".into(),
        )),
        IRExpr::RecordConstruct(_, _) => Err(WasmCodegenError::UnsupportedExpr(
            "record construction in wasm MVP".into(),
        )),
    }
}

fn build_wasm_function(
    fn_def: &crate::middle::ir::IRFnDef,
    ctx: &WasmCodegenCtx<'_>,
) -> Result<Function, WasmCodegenError> {
    let (slot_map, local_decls) = plan_wasm_locals(fn_def, ctx)?;
    let mut func = Function::new(local_decls);
    emit_expr(&fn_def.body, ctx, &slot_map, &mut func)?;
    func.instruction(&Instruction::End);
    Ok(func)
}

fn plan_wasm_locals(
    fn_def: &crate::middle::ir::IRFnDef,
    ctx: &WasmCodegenCtx<'_>,
) -> Result<(HashMap<u16, WasmLocal>, Vec<(u32, ValType)>), WasmCodegenError> {
    let closure_slots = scan_closure_bound_slots(&fn_def.body);

    let mut slot_map = HashMap::new();
    let mut next_local_idx = 0u32;
    for (slot, ty) in fn_def.param_tys.iter().enumerate() {
        let local = wasm_local_for_type(ty, &mut next_local_idx)?;
        slot_map.insert(slot as u16, local);
    }

    let mut local_types = HashMap::new();
    collect_local_types(&fn_def.body, &mut local_types);
    let mut local_slots = local_types.keys().copied().collect::<Vec<_>>();
    local_slots.sort_unstable();

    let mut local_decls = Vec::new();
    for slot in local_slots {
        if (slot as usize) < fn_def.param_count {
            continue;
        }

        if let Some(&global_idx) = closure_slots.get(&slot) {
            let wrapper_type_idx = ctx
                .closure_wrapper_type_idx
                .get(&global_idx)
                .copied()
                .unwrap_or(0);
            let fn_idx_local = next_local_idx;
            let env_ptr_local = next_local_idx + 1;
            next_local_idx += 2;
            let local = WasmLocal::FnTableEnv {
                fn_idx_local,
                env_ptr_local,
                wrapper_type_idx,
            };
            slot_map.insert(slot, local);
            local_decls.push((1, ValType::I32));
            local_decls.push((1, ValType::I32));
            continue;
        }

        let ty = local_types.get(&slot).expect("slot from key set");
        let local = wasm_local_for_type(ty, &mut next_local_idx)?;
        slot_map.insert(slot, local);
        append_local_decls(local, ty, &mut local_decls)?;
    }

    Ok((slot_map, local_decls))
}

fn append_local_decls(
    local: WasmLocal,
    ty: &Type,
    local_decls: &mut Vec<(u32, ValType)>,
) -> Result<(), WasmCodegenError> {
    match local {
        WasmLocal::Single(_) => {
            let wasm_ty = single_wasm_valtype(ty)?.expect("single local must have value type");
            local_decls.push((1, wasm_ty));
        }
        WasmLocal::StringPtrLen(_, _) => {
            local_decls.push((1, ValType::I32));
            local_decls.push((1, ValType::I32));
        }
        WasmLocal::FnTableEnv { .. } => {
            local_decls.push((1, ValType::I32)); // fn_idx
            local_decls.push((1, ValType::I32)); // env_ptr
        }
    }
    Ok(())
}

pub fn wasm_codegen_program(ir: &IRProgram) -> Result<Vec<u8>, WasmCodegenError> {
    // Collect closure info: global_idx → (fn_idx, captures_count)
    let closure_info = collect_closure_info(ir);
    let mut sorted_closures: Vec<(u16, (usize, usize))> =
        closure_info.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_closures.sort_by_key(|(k, _)| *k);

    let used_builtin_names = collect_used_builtins(ir);
    let imports = host_imports()
        .iter()
        .copied()
        .filter(|(name, _)| used_builtin_names.contains(*name))
        .collect::<Vec<_>>();
    let import_kinds = imports.iter().map(|(_, kind)| *kind).collect::<Vec<_>>();
    let (mut type_section, fn_to_type_idx, bump_alloc_type_idx) =
        build_type_section(ir, &import_kinds)?;
    let (data_bytes, str_to_offset) = collect_string_literals(ir);

    // Register wrapper types for each closure and build index maps.
    // Wrapper signature: (env_ptr: i32, actual_params: i64...) -> return_ty
    let mut next_type_idx = imports.len() as u32 + ir.fns.len() as u32 + 1; // after bump_alloc type
    let mut closure_wrapper_type_idx: HashMap<u16, u32> = HashMap::new();
    let mut closure_table_idx: HashMap<u16, u32> = HashMap::new();
    for (table_elem_idx, (global_idx, (fn_idx, captures_count))) in
        sorted_closures.iter().enumerate()
    {
        let fn_def = &ir.fns[*fn_idx];
        let actual_params = fn_def.param_count.saturating_sub(*captures_count);
        let mut wrapper_params = vec![ValType::I32]; // env_ptr
        wrapper_params.extend(std::iter::repeat(ValType::I64).take(actual_params));
        let return_results = favnir_type_to_wasm_results(&fn_def.return_ty)?;
        type_section.ty().function(wrapper_params, return_results);
        closure_wrapper_type_idx.insert(*global_idx, next_type_idx);
        closure_table_idx.insert(*global_idx, table_elem_idx as u32);
        next_type_idx += 1;
    }

    let mut import_section = ImportSection::new();
    let mut builtin_to_wasm_idx = HashMap::new();
    for (idx, (name, kind)) in imports.iter().enumerate() {
        let type_idx = idx as u32;
        import_section.import(
            "fav_host",
            host_import_symbol(*kind),
            EntityType::Function(type_idx),
        );
        builtin_to_wasm_idx.insert((*name).to_string(), idx as u32);
    }

    let fn_to_wasm_idx = ir
        .fns
        .iter()
        .enumerate()
        .map(|(idx, _)| (idx, imports.len() as u32 + idx as u32))
        .collect::<HashMap<_, _>>();
    let bump_alloc_fn_idx = imports.len() as u32 + ir.fns.len() as u32;
    // Wrapper functions are placed after bump_alloc in the function index space.
    let wrapper_base_wasm_idx = bump_alloc_fn_idx + 1;

    let ctx = WasmCodegenCtx {
        fn_to_wasm_idx,
        builtin_to_wasm_idx,
        bump_alloc_fn_idx,
        str_to_offset,
        globals: &ir.globals,
        fns: &ir.fns,
        closure_table_idx,
        closure_wrapper_type_idx,
    };

    let mut function_section = FunctionSection::new();
    let mut code_section = CodeSection::new();
    for (fn_idx, fn_def) in ir.fns.iter().enumerate() {
        let type_idx = *fn_to_type_idx.get(&fn_idx).ok_or_else(|| {
            WasmCodegenError::UnsupportedExpr(format!("missing type index for fn {}", fn_def.name))
        })?;
        function_section.function(type_idx);
        code_section.function(&build_wasm_function(fn_def, &ctx)?);
    }
    function_section.function(bump_alloc_type_idx);
    code_section.function(&build_bump_alloc_function());

    // Emit closure wrapper functions.
    for (global_idx, (fn_idx, captures_count)) in &sorted_closures {
        let wrapper_type_idx = *ctx.closure_wrapper_type_idx.get(global_idx).unwrap();
        function_section.function(wrapper_type_idx);
        let fn_def = &ir.fns[*fn_idx];
        let original_wasm_idx = *ctx.fn_to_wasm_idx.get(fn_idx).unwrap();
        code_section.function(&build_closure_wrapper_function(
            fn_def,
            *captures_count,
            original_wasm_idx,
        ));
    }

    // Table and element sections for closures.
    let has_closures = !sorted_closures.is_empty();
    let mut table_section = TableSection::new();
    let mut element_section = ElementSection::new();
    if has_closures {
        table_section.table(TableType {
            element_type: RefType::FUNCREF,
            table64: false,
            minimum: sorted_closures.len() as u64,
            maximum: None,
            shared: false,
        });
        let wrapper_indices: Vec<u32> = (0..sorted_closures.len() as u32)
            .map(|i| wrapper_base_wasm_idx + i)
            .collect();
        element_section.active(
            None, // MVP encoding: table 0
            &ConstExpr::i32_const(0),
            Elements::Functions(wrapper_indices.as_slice().into()),
        );
    }

    let mut export_section = ExportSection::new();
    let mut memory_section = MemorySection::new();
    let global_section = build_heap_ptr_global_section();
    memory_section.memory(MemoryType {
        minimum: 1,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    export_section.export("memory", ExportKind::Memory, 0);
    if let Some(main_fn_idx) =
        ir.globals
            .iter()
            .find(|g| g.name == "main")
            .and_then(|g| match g.kind {
                IRGlobalKind::Fn(idx) => Some(idx),
                _ => None,
            })
    {
        let wasm_idx = *ctx.fn_to_wasm_idx.get(&main_fn_idx).ok_or_else(|| {
            WasmCodegenError::UnsupportedExpr("missing wasm function index for main".into())
        })?;
        export_section.export("main", ExportKind::Func, wasm_idx);
    }

    let has_data = !data_bytes.is_empty();
    let mut data_section = DataSection::new();
    if has_data {
        data_section.active(0, &ConstExpr::i32_const(0), data_bytes);
    }

    // Assemble module following WASM section ordering spec.
    let mut module = Module::new();
    module.section(&type_section);
    module.section(&import_section);
    module.section(&function_section);
    if has_closures {
        module.section(&table_section);
    }
    module.section(&memory_section);
    module.section(&global_section);
    module.section(&export_section);
    if has_closures {
        module.section(&element_section);
    }
    module.section(&code_section);
    if has_data {
        module.section(&data_section);
    }
    Ok(module.finish())
}

#[cfg(test)]
mod tests {
    use super::{
        HostImport, WasmCodegenCtx, WasmCodegenError, WasmLocal, build_bump_alloc_function,
        build_heap_ptr_global_section, build_type_section, build_wasm_function,
        collect_local_types, collect_local_types_stmt, collect_string_literals,
        collect_used_builtins, ensure_supported_main_signature, favnir_type_to_wasm_params,
        favnir_type_to_wasm_results, plan_wasm_locals, wasm_codegen_program, wasm_local_for_type,
    };
    use crate::ast::{Effect, Lit};
    use crate::frontend::parser::Parser;
    use crate::middle::checker::Type;
    use crate::middle::compiler::compile_program;
    use crate::middle::ir::{IRExpr, IRFnDef, IRGlobal, IRGlobalKind, IRProgram, IRStmt};
    use std::collections::HashMap;
    use wasm_encoder::{Encode, Module, ValType};
    use wasmtime::Engine;

    #[test]
    fn wasm_results_for_scalars_and_unit() {
        assert_eq!(favnir_type_to_wasm_results(&Type::Unit).unwrap(), vec![]);
        assert_eq!(
            favnir_type_to_wasm_results(&Type::Int).unwrap(),
            vec![ValType::I64]
        );
        assert_eq!(
            favnir_type_to_wasm_results(&Type::Float).unwrap(),
            vec![ValType::F64]
        );
        assert_eq!(
            favnir_type_to_wasm_results(&Type::Bool).unwrap(),
            vec![ValType::I32]
        );
    }

    #[test]
    fn wasm_string_return_is_ptr_len_pair() {
        assert_eq!(
            favnir_type_to_wasm_results(&Type::String).unwrap(),
            vec![ValType::I32, ValType::I32]
        );
    }

    #[test]
    fn wasm_string_param_is_ptr_len() {
        assert_eq!(
            favnir_type_to_wasm_params(&Type::String).unwrap(),
            vec![ValType::I32, ValType::I32]
        );
    }

    #[test]
    fn wasm_local_for_string_uses_ptr_len_pair() {
        let mut next = 0u32;
        let local = wasm_local_for_type(&Type::String, &mut next).unwrap();
        assert_eq!(local, WasmLocal::StringPtrLen(0, 1));
        assert_eq!(next, 2);
    }

    #[test]
    fn wasm_local_for_int_uses_single_slot() {
        let mut next = 4u32;
        let local = wasm_local_for_type(&Type::Int, &mut next).unwrap();
        assert_eq!(local, WasmLocal::Single(4));
        assert_eq!(next, 5);
    }

    fn make_empty_ctx<'a>(globals: &'a [IRGlobal], fns: &'a [IRFnDef]) -> WasmCodegenCtx<'a> {
        WasmCodegenCtx {
            fn_to_wasm_idx: HashMap::new(),
            builtin_to_wasm_idx: HashMap::new(),
            bump_alloc_fn_idx: 0,
            str_to_offset: HashMap::new(),
            globals,
            fns,
            closure_table_idx: HashMap::new(),
            closure_wrapper_type_idx: HashMap::new(),
        }
    }

    #[test]
    fn plan_wasm_locals_maps_string_param_to_ptr_len_pair() {
        let ctx = make_empty_ctx(&[], &[]);
        let (slot_map, local_decls) = plan_wasm_locals(
            &IRFnDef {
                name: "greet".into(),
                param_count: 1,
                param_tys: vec![Type::String],
                local_count: 1,
                effects: vec![],
                return_ty: Type::Unit,
                body: IRExpr::Lit(Lit::Unit, Type::Unit),
            },
            &ctx,
        )
        .unwrap();
        assert_eq!(slot_map.get(&0), Some(&WasmLocal::StringPtrLen(0, 1)));
        assert!(local_decls.is_empty());
    }

    #[test]
    fn plan_wasm_locals_places_string_local_after_scalar_param() {
        let ctx = make_empty_ctx(&[], &[]);
        let (slot_map, local_decls) = plan_wasm_locals(
            &IRFnDef {
                name: "main".into(),
                param_count: 1,
                param_tys: vec![Type::Int],
                local_count: 2,
                effects: vec![],
                return_ty: Type::Unit,
                body: IRExpr::Block(
                    vec![IRStmt::Bind(
                        1,
                        IRExpr::Lit(Lit::Str("hi".into()), Type::String),
                    )],
                    Box::new(IRExpr::Lit(Lit::Unit, Type::Unit)),
                    Type::Unit,
                ),
            },
            &ctx,
        )
        .unwrap();
        assert_eq!(slot_map.get(&0), Some(&WasmLocal::Single(0)));
        assert_eq!(slot_map.get(&1), Some(&WasmLocal::StringPtrLen(1, 2)));
        assert_eq!(local_decls, vec![(1, ValType::I32), (1, ValType::I32)]);
    }

    #[test]
    fn collect_local_types_tracks_bind_and_nested_reads() {
        let mut map = HashMap::new();
        let expr = IRExpr::Block(
            vec![
                IRStmt::Bind(0, IRExpr::Lit(crate::ast::Lit::Int(1), Type::Int)),
                IRStmt::Bind(
                    1,
                    IRExpr::BinOp(
                        crate::ast::BinOp::Add,
                        Box::new(IRExpr::Local(0, Type::Int)),
                        Box::new(IRExpr::Lit(crate::ast::Lit::Int(2), Type::Int)),
                        Type::Int,
                    ),
                ),
            ],
            Box::new(IRExpr::Local(1, Type::Int)),
            Type::Int,
        );
        collect_local_types(&expr, &mut map);
        assert_eq!(map.get(&0), Some(&Type::Int));
        assert_eq!(map.get(&1), Some(&Type::Int));
    }

    #[test]
    fn collect_local_types_stmt_tracks_chain_slot() {
        let mut map = HashMap::new();
        collect_local_types_stmt(&IRStmt::Chain(7, IRExpr::Local(3, Type::Bool)), &mut map);
        assert_eq!(map.get(&7), Some(&Type::Bool));
        assert_eq!(map.get(&3), Some(&Type::Bool));
    }

    #[test]
    fn wasm_codegen_ctx_holds_expected_maps() {
        let globals = vec![IRGlobal {
            name: "main".into(),
            kind: IRGlobalKind::Fn(0),
        }];
        let mut fn_to_wasm_idx = HashMap::new();
        fn_to_wasm_idx.insert(0usize, 3u32);
        let mut builtin_to_wasm_idx = HashMap::new();
        builtin_to_wasm_idx.insert("IO.println".into(), 1u32);
        let mut str_to_offset = HashMap::new();
        str_to_offset.insert("hello".into(), 0u32);
        let ctx = WasmCodegenCtx {
            fn_to_wasm_idx,
            builtin_to_wasm_idx,
            bump_alloc_fn_idx: 99,
            str_to_offset,
            globals: &globals,
            fns: &[],
            closure_table_idx: HashMap::new(),
            closure_wrapper_type_idx: HashMap::new(),
        };
        assert_eq!(ctx.fn_to_wasm_idx.get(&0), Some(&3));
        assert_eq!(ctx.builtin_to_wasm_idx.get("IO.println"), Some(&1));
        assert_eq!(ctx.bump_alloc_fn_idx, 99);
        assert_eq!(ctx.str_to_offset.get("hello"), Some(&0));
        assert_eq!(ctx.globals.len(), 1);
    }

    #[test]
    fn host_import_is_copyable_marker() {
        let import = HostImport::IoPrintlnInt;
        assert_eq!(import, HostImport::IoPrintlnInt);
    }

    #[test]
    fn ensure_supported_main_signature_accepts_unit_io_main() {
        let ir = IRProgram {
            globals: vec![IRGlobal {
                name: "main".into(),
                kind: IRGlobalKind::Fn(0),
            }],
            fns: vec![IRFnDef {
                name: "main".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![Effect::Io],
                return_ty: Type::Unit,
                body: IRExpr::Lit(crate::ast::Lit::Unit, Type::Unit),
            }],
        };
        ensure_supported_main_signature(&ir).unwrap();
    }

    #[test]
    fn ensure_supported_main_signature_rejects_non_unit_or_non_io_main() {
        let ir = IRProgram {
            globals: vec![IRGlobal {
                name: "main".into(),
                kind: IRGlobalKind::Fn(0),
            }],
            fns: vec![IRFnDef {
                name: "main".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![],
                return_ty: Type::Int,
                body: IRExpr::Lit(crate::ast::Lit::Int(1), Type::Int),
            }],
        };
        assert_eq!(
            ensure_supported_main_signature(&ir).unwrap_err(),
            WasmCodegenError::UnsupportedMainSignature
        );
    }

    #[test]
    fn build_type_section_registers_imports_and_functions() {
        let ir = IRProgram {
            globals: vec![IRGlobal {
                name: "main".into(),
                kind: IRGlobalKind::Fn(0),
            }],
            fns: vec![IRFnDef {
                name: "main".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![Effect::Io],
                return_ty: Type::Unit,
                body: IRExpr::Lit(crate::ast::Lit::Unit, Type::Unit),
            }],
        };
        let (section, fn_map, bump_type_idx) =
            build_type_section(&ir, &[HostImport::IoPrintln, HostImport::IoPrintlnInt]).unwrap();
        let mut module = Module::new();
        module.section(&section);
        assert!(!module.finish().is_empty());
        assert_eq!(fn_map.get(&0), Some(&2));
        assert_eq!(bump_type_idx, 3);
    }

    #[test]
    fn heap_ptr_global_section_encodes_mut_i32_at_64k() {
        let section = build_heap_ptr_global_section();
        let mut module = Module::new();
        module.section(&section);
        let bytes = module.finish();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn bump_alloc_function_encodes() {
        let func = build_bump_alloc_function();
        let mut bytes = Vec::new();
        func.encode(&mut bytes);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn build_wasm_function_emits_simple_int_add() {
        let globals = vec![];
        let ctx = WasmCodegenCtx {
            fn_to_wasm_idx: HashMap::new(),
            builtin_to_wasm_idx: HashMap::new(),
            bump_alloc_fn_idx: 0,
            str_to_offset: HashMap::new(),
            globals: &globals,
            fns: &[],
            closure_table_idx: HashMap::new(),
            closure_wrapper_type_idx: HashMap::new(),
        };
        let func = build_wasm_function(
            &IRFnDef {
                name: "add1".into(),
                param_count: 1,
                param_tys: vec![Type::Int],
                local_count: 1,
                effects: vec![],
                return_ty: Type::Int,
                body: IRExpr::BinOp(
                    crate::ast::BinOp::Add,
                    Box::new(IRExpr::Local(0, Type::Int)),
                    Box::new(IRExpr::Lit(Lit::Int(1), Type::Int)),
                    Type::Int,
                ),
            },
            &ctx,
        )
        .unwrap();

        let mut code = Vec::new();
        func.encode(&mut code);
        assert!(!code.is_empty());
    }

    #[test]
    fn wasm_codegen_program_emits_valid_module_for_unit_io_main() {
        let ir = IRProgram {
            globals: vec![
                IRGlobal {
                    name: "IO".into(),
                    kind: IRGlobalKind::Builtin,
                },
                IRGlobal {
                    name: "main".into(),
                    kind: IRGlobalKind::Fn(0),
                },
            ],
            fns: vec![IRFnDef {
                name: "main".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![Effect::Io],
                return_ty: Type::Unit,
                body: IRExpr::Call(
                    Box::new(IRExpr::FieldAccess(
                        Box::new(IRExpr::Global(0, Type::Unknown)),
                        "println_int".into(),
                        Type::Unknown,
                    )),
                    vec![IRExpr::Lit(Lit::Int(42), Type::Int)],
                    Type::Unit,
                ),
            }],
        };

        let bytes = wasm_codegen_program(&ir).unwrap();
        assert!(bytes.starts_with(&[0x00, 0x61, 0x73, 0x6d]));
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_codegen_program_rejects_unsupported_match_expr() {
        let ir = IRProgram {
            globals: vec![IRGlobal {
                name: "main".into(),
                kind: IRGlobalKind::Fn(0),
            }],
            fns: vec![IRFnDef {
                name: "main".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![Effect::Io],
                return_ty: Type::Unit,
                body: IRExpr::Match(
                    Box::new(IRExpr::Lit(Lit::Int(1), Type::Int)),
                    vec![],
                    Type::Unit,
                ),
            }],
        };
        let err = wasm_codegen_program(&ir).unwrap_err();
        assert_eq!(err.code(), "W002");
    }

    #[test]
    fn collect_string_literals_interns_and_offsets() {
        let ir = IRProgram {
            globals: vec![],
            fns: vec![IRFnDef {
                name: "main".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![Effect::Io],
                return_ty: Type::Unit,
                body: IRExpr::Block(
                    vec![IRStmt::Expr(IRExpr::Lit(
                        Lit::Str("hello".into()),
                        Type::String,
                    ))],
                    Box::new(IRExpr::Lit(Lit::Str("world".into()), Type::String)),
                    Type::String,
                ),
            }],
        };
        let (bytes, offsets) = collect_string_literals(&ir);
        assert_eq!(bytes, b"helloworld");
        assert_eq!(offsets.get("hello"), Some(&0));
        assert_eq!(offsets.get("world"), Some(&5));
    }

    #[test]
    fn collect_used_builtins_detects_io_calls() {
        let ir = IRProgram {
            globals: vec![
                IRGlobal {
                    name: "IO".into(),
                    kind: IRGlobalKind::Builtin,
                },
                IRGlobal {
                    name: "main".into(),
                    kind: IRGlobalKind::Fn(0),
                },
            ],
            fns: vec![IRFnDef {
                name: "main".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![Effect::Io],
                return_ty: Type::Unit,
                body: IRExpr::Call(
                    Box::new(IRExpr::FieldAccess(
                        Box::new(IRExpr::Global(0, Type::Unknown)),
                        "println".into(),
                        Type::Unknown,
                    )),
                    vec![IRExpr::Lit(Lit::Str("hello".into()), Type::String)],
                    Type::Unit,
                ),
            }],
        };
        let used = collect_used_builtins(&ir);
        assert!(used.contains("IO.println"));
        assert_eq!(used.len(), 1);
    }

    #[test]
    fn wasm_codegen_program_emits_valid_module_for_hello_string() {
        let ir = IRProgram {
            globals: vec![
                IRGlobal {
                    name: "IO".into(),
                    kind: IRGlobalKind::Builtin,
                },
                IRGlobal {
                    name: "main".into(),
                    kind: IRGlobalKind::Fn(0),
                },
            ],
            fns: vec![IRFnDef {
                name: "main".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![Effect::Io],
                return_ty: Type::Unit,
                body: IRExpr::Call(
                    Box::new(IRExpr::FieldAccess(
                        Box::new(IRExpr::Global(0, Type::Unknown)),
                        "println".into(),
                        Type::Unknown,
                    )),
                    vec![IRExpr::Lit(Lit::Str("Hello, Favnir!".into()), Type::String)],
                    Type::Unit,
                ),
            }],
        };

        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_codegen_program_emits_valid_module_for_if_returning_int() {
        let ir = IRProgram {
            globals: vec![IRGlobal {
                name: "main".into(),
                kind: IRGlobalKind::Fn(0),
            }],
            fns: vec![IRFnDef {
                name: "main".into(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: vec![Effect::Io],
                return_ty: Type::Unit,
                body: IRExpr::Block(
                    vec![IRStmt::Expr(IRExpr::If(
                        Box::new(IRExpr::Lit(Lit::Bool(true), Type::Bool)),
                        Box::new(IRExpr::Lit(Lit::Int(1), Type::Int)),
                        Box::new(IRExpr::Lit(Lit::Int(2), Type::Int)),
                        Type::Unknown,
                    ))],
                    Box::new(IRExpr::Lit(Lit::Unit, Type::Unit)),
                    Type::Unit,
                ),
            }],
        };

        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_codegen_program_emits_valid_module_for_recursive_factorial_shape() {
        let ir = IRProgram {
            globals: vec![
                IRGlobal {
                    name: "factorial".into(),
                    kind: IRGlobalKind::Fn(0),
                },
                IRGlobal {
                    name: "main".into(),
                    kind: IRGlobalKind::Fn(1),
                },
            ],
            fns: vec![
                IRFnDef {
                    name: "factorial".into(),
                    param_count: 1,
                    param_tys: vec![Type::Int],
                    local_count: 1,
                    effects: vec![],
                    return_ty: Type::Int,
                    body: IRExpr::If(
                        Box::new(IRExpr::BinOp(
                            crate::ast::BinOp::LtEq,
                            Box::new(IRExpr::Local(0, Type::Int)),
                            Box::new(IRExpr::Lit(Lit::Int(1), Type::Int)),
                            Type::Bool,
                        )),
                        Box::new(IRExpr::Lit(Lit::Int(1), Type::Int)),
                        Box::new(IRExpr::BinOp(
                            crate::ast::BinOp::Mul,
                            Box::new(IRExpr::Local(0, Type::Int)),
                            Box::new(IRExpr::Call(
                                Box::new(IRExpr::Global(0, Type::Unknown)),
                                vec![IRExpr::BinOp(
                                    crate::ast::BinOp::Sub,
                                    Box::new(IRExpr::Local(0, Type::Int)),
                                    Box::new(IRExpr::Lit(Lit::Int(1), Type::Int)),
                                    Type::Int,
                                )],
                                Type::Int,
                            )),
                            Type::Int,
                        )),
                        Type::Unknown,
                    ),
                },
                IRFnDef {
                    name: "main".into(),
                    param_count: 0,
                    param_tys: vec![],
                    local_count: 0,
                    effects: vec![Effect::Io],
                    return_ty: Type::Unit,
                    body: IRExpr::Lit(Lit::Unit, Type::Unit),
                },
            ],
        };

        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_codegen_program_emits_valid_module_for_math_wasm_source_shape() {
        let source = r#"
public fn add(a: Int, b: Int) -> Int {
    a + b
}

public fn abs(n: Int) -> Int {
    if n < 0 {
        0 - n
    } else {
        n
    }
}

public fn factorial(n: Int) -> Int {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

public fn main() -> Unit !Io {
    IO.println_int(add(21, 21));
    IO.println_int(abs(-5));
    IO.println_int(factorial(5))
}
"#;
        let program = Parser::parse_str(source, "math_wasm_test.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_hello_world() {
        let source = r#"
public fn main() -> Unit !Io {
    IO.println("Hello")
}
"#;
        let program = Parser::parse_str(source, "wasm_hello_world.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_int_arithmetic() {
        let source = r#"
public fn main() -> Unit !Io {
    IO.println_int(21 + 21)
}
"#;
        let program = Parser::parse_str(source, "wasm_int_arithmetic.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_if_else() {
        let source = r#"
public fn abs(n: Int) -> Int {
    if n < 0 {
        0 - n
    } else {
        n
    }
}

public fn main() -> Unit !Io {
    IO.println_int(abs(-5))
}
"#;
        let program = Parser::parse_str(source, "wasm_if_else.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_recursive_factorial() {
        let source = r#"
public fn factorial(n: Int) -> Int {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

public fn main() -> Unit !Io {
    IO.println_int(factorial(5))
}
"#;
        let program = Parser::parse_str(source, "wasm_recursive_factorial.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_bool_ops() {
        let source = r#"
public fn main() -> Unit !Io {
    IO.println_bool((1 < 2) == true)
}
"#;
        let program = Parser::parse_str(source, "wasm_bool_ops.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_string_return_function_module_is_valid() {
        let source = r#"
public fn greet() -> String {
    "hi"
}

public fn main() -> Unit !Io {
    IO.println("ok")
}
"#;
        let program = Parser::parse_str(source, "wasm_string_return_module.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_string_identity_function_module_is_valid() {
        let source = r#"
public fn id(name: String) -> String {
    name
}

public fn main() -> Unit !Io {
    IO.println(id("Favnir"))
}
"#;
        let program = Parser::parse_str(source, "wasm_string_identity.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_string_bind_and_print_module_is_valid() {
        let source = r#"
public fn main() -> Unit !Io {
    bind s <- "hi";
    IO.println(s)
}
"#;
        let program = Parser::parse_str(source, "wasm_string_bind_print.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_w002_string_if_else_result() {
        let source = r#"
public fn choose(flag: Bool) -> String {
    if flag {
        "a"
    } else {
        "b"
    }
}

public fn main() -> Unit !Io {
    IO.println("ok")
}
"#;
        let program =
            Parser::parse_str(source, "wasm_w002_string_if_else_result.fav").expect("parse");
        let ir = compile_program(&program);
        let err = wasm_codegen_program(&ir).unwrap_err();
        assert_eq!(err.code(), "W002");
        assert!(err.to_string().contains("multi-value block result"));
    }

    #[test]
    fn wasm_w002_debug_show() {
        let source = r#"
public fn main() -> Unit !Io {
    Debug.show(42)
}
"#;
        let program = Parser::parse_str(source, "wasm_w002_debug_show.fav").expect("parse");
        let ir = compile_program(&program);
        let err = wasm_codegen_program(&ir).unwrap_err();
        assert_eq!(err.code(), "W002");
    }

    #[test]
    fn wasm_w003_main_returns_int() {
        let source = r#"
public fn main() -> Int !Io {
    42
}
"#;
        let program = Parser::parse_str(source, "wasm_w003_main_returns_int.fav").expect("parse");
        let ir = compile_program(&program);
        let err = wasm_codegen_program(&ir).unwrap_err();
        assert_eq!(err.code(), "W003");
    }

    // ── Phase 3: closure tests ──────────────────────────────────────────────

    #[test]
    fn wasm_closure_codegen_produces_valid_module() {
        // `bind f <- |x| x + 1; f(5)` — closure with no captures
        let source = r#"
public fn main() -> Unit !Io {
    bind f <- |x| x + 1;
    IO.println_int(f(5))
}
"#;
        let program = Parser::parse_str(source, "closure_direct.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        assert!(bytes.starts_with(&[0x00, 0x61, 0x73, 0x6d]));
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_closure_capture_produces_valid_module() {
        // closure captures an outer variable
        let source = r#"
public fn main() -> Unit !Io {
    bind n <- 10;
    bind add_n <- |x| x + n;
    IO.println_int(add_n(5))
}
"#;
        let program = Parser::parse_str(source, "closure_capture.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();
        assert!(bytes.starts_with(&[0x00, 0x61, 0x73, 0x6d]));
        let engine = Engine::default();
        wasmtime::Module::new(&engine, &bytes).unwrap();
    }

    #[test]
    fn wasm_closure_exec_returns_correct_result() {
        use wasmtime::{Engine, Linker, Store};

        let source = r#"
public fn main() -> Unit !Io {
    bind f <- |x| x + 1;
    IO.println_int(f(5))
}
"#;
        let program = Parser::parse_str(source, "closure_exec.fav").expect("parse");
        let ir = compile_program(&program);
        let bytes = wasm_codegen_program(&ir).unwrap();

        let engine = Engine::default();
        let mut store = Store::new(&engine, ());
        let module = wasmtime::Module::new(&engine, &bytes).unwrap();
        let mut linker = Linker::new(&engine);

        let printed: std::sync::Arc<std::sync::Mutex<Option<i64>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let printed_clone = printed.clone();
        linker
            .func_wrap("fav_host", "io_println_int", move |v: i64| {
                *printed_clone.lock().unwrap() = Some(v);
            })
            .unwrap();
        linker
            .func_wrap("fav_host", "io_println", |_ptr: i32, _len: i32| {})
            .unwrap();

        let instance = linker.instantiate(&mut store, &module).unwrap();
        let main = instance
            .get_typed_func::<(), ()>(&mut store, "main")
            .unwrap();
        main.call(&mut store, ()).unwrap();

        assert_eq!(*printed.lock().unwrap(), Some(6)); // f(5) = 5 + 1 = 6
    }
}
