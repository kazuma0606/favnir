use std::collections::HashMap;
use std::sync::Mutex;
use rusqlite::Connection;
use serde_json::Value as SerdeJsonValue;

use super::artifact::FvcArtifact;
use super::codegen::{Constant, Opcode};
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct VMError {
    pub message: String,
    pub fn_name: String,
    pub ip: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallFrame {
    pub fn_idx: usize,
    pub ip: usize,
    pub base: usize,
    pub n_locals: usize,
}

#[derive(Debug, Clone)]
pub struct VM {
    globals: Vec<VMValue>,
    stack: Vec<VMValue>,
    frames: Vec<CallFrame>,
    collect_frames: Vec<Vec<VMValue>>,
    emit_log: Vec<VMValue>,
    db_path: Option<String>,
}

static SHARED_DBS: Mutex<Vec<(String, Connection)>> = Mutex::new(Vec::new());

#[derive(Debug, Clone, PartialEq)]
enum VMValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Unit,
    List(Vec<VMValue>),
    Record(HashMap<String, VMValue>),
    Variant(String, Option<Box<VMValue>>),
    VariantCtor(String),
    CompiledFn(usize),
    Closure(usize, Vec<VMValue>),
    Builtin(String),
}

impl VM {
    #[allow(dead_code)]
    pub fn new(artifact: &FvcArtifact) -> VM {
        Self::new_with_db_path(artifact, None)
    }

    pub fn new_with_db_path(artifact: &FvcArtifact, db_path: Option<String>) -> VM {
        let globals = artifact
            .globals
            .iter()
            .map(|g| match g.kind {
                0 => VMValue::CompiledFn(g.fn_idx as usize),
                1 => {
                    let name = artifact
                        .str_table
                        .get(g.name_idx as usize)
                        .cloned()
                        .unwrap_or_else(|| "<builtin>".to_string());
                    VMValue::Builtin(name)
                }
                2 => {
                    let name = artifact
                        .str_table
                        .get(g.name_idx as usize)
                        .cloned()
                        .unwrap_or_else(|| "<variant>".to_string());
                    VMValue::VariantCtor(name)
                }
                _ => VMValue::Unit,
            })
            .collect();
        VM {
            globals,
            stack: Vec::new(),
            frames: Vec::new(),
            collect_frames: Vec::new(),
            emit_log: Vec::new(),
            db_path,
        }
    }

    #[allow(dead_code)]
    pub fn run(artifact: &FvcArtifact, fn_idx: usize, args: Vec<Value>) -> Result<Value, VMError> {
        Self::run_with_db_path(artifact, fn_idx, args, None).map(|(value, _)| value)
    }

    pub fn run_with_db_path(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
        db_path: Option<&str>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        Self::run_with_emits_and_db_path(artifact, fn_idx, args, db_path)
    }

    #[allow(dead_code)]
    pub fn run_with_emits(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        Self::run_with_emits_and_db_path(artifact, fn_idx, args, None)
    }

    pub fn run_with_emits_and_db_path(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<Value>,
        db_path: Option<&str>,
    ) -> Result<(Value, Vec<Value>), VMError> {
        let (value, emits) =
            Self::run_with_vmvalues(
                artifact,
                fn_idx,
                args.into_iter().map(VMValue::from).collect(),
                db_path.map(|s| s.to_string()),
            )?;
        Ok((
            Value::from(value),
            emits.into_iter().map(Value::from).collect(),
        ))
    }

    fn run_with_vmvalues(
        artifact: &FvcArtifact,
        fn_idx: usize,
        args: Vec<VMValue>,
        db_path: Option<String>,
    ) -> Result<(VMValue, Vec<VMValue>), VMError> {
        let mut vm = VM::new_with_db_path(artifact, db_path);
        let function = artifact.functions.get(fn_idx).ok_or_else(|| VMError {
            message: format!("unknown function index: {fn_idx}"),
            fn_name: "<invalid>".to_string(),
            ip: 0,
        })?;

        let base = vm.stack.len();
        vm.stack.extend(args);
        let required = function.local_count as usize;
        while vm.stack.len() < base + required {
            vm.stack.push(VMValue::Unit);
        }
        vm.frames.push(CallFrame {
            fn_idx,
            ip: 0,
            base,
            n_locals: required,
        });

        loop {
            let Some(frame) = vm.frames.last_mut() else {
                return Ok((VMValue::Unit, vm.emit_log));
            };
            let function = &artifact.functions[frame.fn_idx];
            if frame.ip >= function.code.len() {
                return Err(vm.error(artifact, "instruction pointer out of bounds"));
            }
            let opcode = function.code[frame.ip];
            frame.ip += 1;

            match opcode {
                x if x == Opcode::Const as u8 => {
                    let idx = Self::read_u16(function, frame)? as usize;
                    let constant = function.constants.get(idx).ok_or_else(|| vm.error(artifact, "constant index out of bounds"))?;
                    vm.stack.push(constant_to_value(constant.clone()));
                }
                x if x == Opcode::ConstUnit as u8 => vm.stack.push(VMValue::Unit),
                x if x == Opcode::ConstTrue as u8 => vm.stack.push(VMValue::Bool(true)),
                x if x == Opcode::ConstFalse as u8 => vm.stack.push(VMValue::Bool(false)),
                x if x == Opcode::LoadLocal as u8 => {
                    let slot = Self::read_u16(function, frame)? as usize;
                    let idx = frame.base + slot;
                    let value = vm.stack.get(idx).cloned().ok_or_else(|| vm.error(artifact, "local slot out of bounds"))?;
                    vm.stack.push(value);
                }
                x if x == Opcode::StoreLocal as u8 => {
                    let slot = Self::read_u16(function, frame)? as usize;
                    let idx = frame.base + slot;
                    let value = vm.stack.pop().ok_or_else(|| vm.error(artifact, "stack underflow on store"))?;
                    if idx >= vm.stack.len() {
                        vm.stack.resize(idx + 1, VMValue::Unit);
                    }
                    vm.stack[idx] = value;
                }
                x if x == Opcode::LoadGlobal as u8 => {
                    let idx = Self::read_u16(function, frame)? as usize;
                    let value = vm.globals.get(idx).cloned().ok_or_else(|| vm.error(artifact, "global index out of bounds"))?;
                    vm.stack.push(value);
                }
                x if x == Opcode::Pop as u8 => {
                    vm.stack.pop().ok_or_else(|| vm.error(artifact, "stack underflow on pop"))?;
                }
                x if x == Opcode::Dup as u8 => {
                    let value = vm.stack.last().cloned().ok_or_else(|| vm.error(artifact, "stack underflow on dup"))?;
                    vm.stack.push(value);
                }
                x if x == Opcode::Jump as u8 => {
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(next_ip) = frame.ip.checked_add(offset) else {
                        return Err(vm.error(artifact, "jump overflow"));
                    };
                    frame.ip = next_ip;
                }
                x if x == Opcode::JumpIfFalse as u8 => {
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(cond) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on conditional jump"));
                    };
                    match cond {
                        VMValue::Bool(false) => {
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        VMValue::Bool(true) => {}
                        _ => return Err(vm.error(artifact, "conditional jump requires a Bool")),
                    }
                }
                x if x == Opcode::MatchFail as u8 => {
                    return Err(vm.error(artifact, "non-exhaustive match"));
                }
                x if x == Opcode::ChainCheck as u8 => {
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on chain_check"));
                    };
                    match value {
                        VMValue::Variant(tag, payload) if tag == "ok" || tag == "some" => {
                            let unwrapped = payload
                                .map(|inner| *inner)
                                .ok_or_else(|| vm.error(artifact, "chain_check expected payload"))?;
                            vm.stack.push(unwrapped);
                        }
                        VMValue::Variant(tag, payload) if tag == "err" => {
                            vm.stack.push(VMValue::Variant(tag, payload));
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        VMValue::Variant(tag, None) if tag == "none" => {
                            vm.stack.push(VMValue::Variant(tag, None));
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                        other => {
                            return Err(vm.error(
                                artifact,
                                &format!("chain_check requires ok/some/err/none variant, got {other:?}"),
                            ));
                        }
                    }
                }
                x if x == Opcode::JumpIfNotVariant as u8 => {
                    let name_idx = Self::read_u16(function, frame)? as usize;
                    let offset = Self::read_u16(function, frame)? as usize;
                    let Some(expected) = artifact.str_table.get(name_idx).cloned() else {
                        return Err(vm.error(artifact, "variant name index out of bounds"));
                    };
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on variant check"));
                    };
                    match value {
                        VMValue::Variant(tag, payload) if tag == expected => {
                            vm.stack.push(VMValue::Variant(tag, payload));
                        }
                        other => {
                            vm.stack.push(other);
                            let Some(next_ip) = frame.ip.checked_add(offset) else {
                                return Err(vm.error(artifact, "jump overflow"));
                            };
                            frame.ip = next_ip;
                        }
                    }
                }
                x if x == Opcode::GetField as u8 => {
                    let idx = Self::read_u16(function, frame)? as usize;
                    let field_name = artifact
                        .str_table
                        .get(idx)
                        .cloned()
                        .ok_or_else(|| vm.error(artifact, "field name index out of bounds"))?;
                    let value = vm.stack.pop().ok_or_else(|| vm.error(artifact, "stack underflow on get_field"))?;
                    match value {
                        VMValue::Record(map) => {
                            let field = map.get(&field_name).cloned().ok_or_else(|| {
                                vm.error(artifact, &format!("missing record field `{field_name}`"))
                            })?;
                            vm.stack.push(field);
                        }
                        VMValue::Builtin(ns) => {
                            vm.stack.push(VMValue::Builtin(format!("{}.{}", ns, field_name)));
                        }
                        _ => return Err(vm.error(artifact, "get_field requires a record value")),
                    }
                }
                x if x == Opcode::BuildRecord as u8 => {
                    let field_count = Self::read_u16(function, frame)? as usize;
                    let names_idx = Self::read_u16(function, frame)? as usize;
                    let names = artifact
                        .str_table
                        .get(names_idx)
                        .cloned()
                        .ok_or_else(|| vm.error(artifact, "record field names index out of bounds"))?;
                    let field_names: Vec<&str> = if names.is_empty() {
                        Vec::new()
                    } else {
                        names.split('\u{1f}').collect()
                    };
                    if field_names.len() != field_count {
                        return Err(vm.error(artifact, "record field name count mismatch"));
                    }
                    let mut values = Vec::with_capacity(field_count);
                    for _ in 0..field_count {
                        values.push(vm.stack.pop().ok_or_else(|| vm.error(artifact, "stack underflow on build_record"))?);
                    }
                    values.reverse();
                    let mut map = HashMap::with_capacity(field_count);
                    for (name, value) in field_names.into_iter().zip(values.into_iter()) {
                        map.insert(name.to_string(), value);
                    }
                    vm.stack.push(VMValue::Record(map));
                }
                x if x == Opcode::MakeClosure as u8 => {
                    let global_idx = Self::read_u16(function, frame)? as usize;
                    let capture_count = Self::read_u16(function, frame)? as usize;
                    let mut captures = Vec::with_capacity(capture_count);
                    for _ in 0..capture_count {
                        captures.push(
                            vm.stack
                                .pop()
                                .ok_or_else(|| vm.error(artifact, "stack underflow on make_closure"))?,
                        );
                    }
                    captures.reverse();
                    let target = vm
                        .globals
                        .get(global_idx)
                        .cloned()
                        .ok_or_else(|| vm.error(artifact, "closure global index out of bounds"))?;
                    match target {
                        VMValue::CompiledFn(fn_idx) => vm.stack.push(VMValue::Closure(fn_idx, captures)),
                        _ => {
                            return Err(vm.error(
                                artifact,
                                "make_closure requires a function global target",
                            ))
                        }
                    }
                }
                x if x == Opcode::GetVariantPayload as u8 => {
                    let value = vm
                        .stack
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "stack underflow on get_variant_payload"))?;
                    match value {
                        VMValue::Variant(_, Some(payload)) => vm.stack.push(*payload),
                        VMValue::Variant(_, None) => {
                            return Err(vm.error(artifact, "variant has no payload"));
                        }
                        _ => return Err(vm.error(artifact, "get_variant_payload requires a variant")),
                    }
                }
                x if x == Opcode::CollectBegin as u8 => {
                    vm.collect_frames.push(Vec::new());
                }
                x if x == Opcode::CollectEnd as u8 => {
                    let values = vm
                        .collect_frames
                        .pop()
                        .ok_or_else(|| vm.error(artifact, "collect_end without collect_begin"))?;
                    vm.stack.push(VMValue::List(values));
                }
                x if x == Opcode::YieldValue as u8 => {
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on yield"));
                    };
                    let Some(collect_frame) = vm.collect_frames.last_mut() else {
                        return Err(vm.error(artifact, "yield outside collect"));
                    };
                    collect_frame.push(value);
                }
                x if x == Opcode::EmitEvent as u8 => {
                    let Some(value) = vm.stack.pop() else {
                        return Err(vm.error(artifact, "stack underflow on emit"));
                    };
                    vm.emit_log.push(value);
                    vm.stack.push(VMValue::Unit);
                }
                x if x == Opcode::Call as u8 => {
                    let arg_count = Self::read_u16(function, frame)? as usize;
                    let callee_pos = vm
                        .stack
                        .len()
                        .checked_sub(arg_count + 1)
                        .ok_or_else(|| vm.error(artifact, "stack underflow on call"))?;
                    let callee = vm.stack[callee_pos].clone();
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(vm.stack.pop().ok_or_else(|| {
                            vm.error(artifact, "stack underflow on call")
                        })?);
                    }
                    args.reverse();
                    vm.stack.remove(callee_pos);
                    let result = vm.call_value(artifact, callee, args)?;
                    vm.stack.push(result);
                }
                x if x == Opcode::Add as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(apply_numeric_binop(left, right, |a, b| a + b, |a, b| a + b, "add", artifact, &vm.frames)?);
                }
                x if x == Opcode::Sub as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(apply_numeric_binop(left, right, |a, b| a - b, |a, b| a - b, "sub", artifact, &vm.frames)?);
                }
                x if x == Opcode::Mul as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(apply_numeric_binop(left, right, |a, b| a * b, |a, b| a * b, "mul", artifact, &vm.frames)?);
                }
                x if x == Opcode::Div as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(apply_numeric_binop(left, right, |a, b| a / b, |a, b| a / b, "div", artifact, &vm.frames)?);
                }
                x if x == Opcode::Eq as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(VMValue::Bool(left == right));
                }
                x if x == Opcode::Ne as u8 => {
                    let (left, right) = vm.pop_pair(artifact)?;
                    vm.stack.push(VMValue::Bool(left != right));
                }
                x if x == Opcode::Lt as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack.push(compare_pair(pair, |a, b| a < b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Le as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack.push(compare_pair(pair, |a, b| a <= b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Gt as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack.push(compare_pair(pair, |a, b| a > b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Ge as u8 => {
                    let pair = vm.pop_pair(artifact)?;
                    vm.stack.push(compare_pair(pair, |a, b| a >= b, artifact, &vm.frames)?);
                }
                x if x == Opcode::Return as u8 => {
                    let ret = vm.stack.pop().unwrap_or(VMValue::Unit);
                    let frame = vm.frames.pop().expect("frame exists");
                    vm.stack.truncate(frame.base);
                    if vm.frames.is_empty() {
                        return Ok((ret, vm.emit_log));
                    }
                    vm.stack.push(ret);
                }
                other => {
                    return Err(vm.error(artifact, &format!("unsupported opcode: 0x{other:02x}")));
                }
            }
        }
    }

    fn read_u16(function: &crate::backend::artifact::FvcFunction, frame: &mut CallFrame) -> Result<u16, VMError> {
        if frame.ip + 1 >= function.code.len() {
            return Err(VMError {
                message: "unexpected end of bytecode".to_string(),
                fn_name: "<decode>".to_string(),
                ip: frame.ip,
            });
        }
        let lo = function.code[frame.ip];
        let hi = function.code[frame.ip + 1];
        frame.ip += 2;
        Ok(u16::from_le_bytes([lo, hi]))
    }

    fn error(&self, artifact: &FvcArtifact, message: &str) -> VMError {
        if let Some(frame) = self.frames.last() {
            let function = &artifact.functions[frame.fn_idx];
            let fn_name = artifact
                .str_table
                .get(function.name_idx as usize)
                .cloned()
                .unwrap_or_else(|| "<unknown>".to_string());
            VMError {
                message: message.to_string(),
                fn_name,
                ip: frame.ip,
            }
        } else {
            VMError {
                message: message.to_string(),
                fn_name: "<none>".to_string(),
                ip: 0,
            }
        }
    }

    fn call_value(
        &mut self,
        artifact: &FvcArtifact,
        callee: VMValue,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        match callee {
            VMValue::CompiledFn(target_idx) => {
                let (value, emits) =
                    Self::run_with_vmvalues(artifact, target_idx, args, self.db_path.clone())?;
                self.emit_log.extend(emits);
                Ok(value)
            }
            VMValue::Closure(target_idx, captures) => {
                let mut full_args = captures;
                full_args.extend(args);
                let (value, emits) = Self::run_with_vmvalues(
                    artifact,
                    target_idx,
                    full_args,
                    self.db_path.clone(),
                )?;
                self.emit_log.extend(emits);
                Ok(value)
            }
            VMValue::VariantCtor(name) => {
                let payload = match args.len() {
                    0 => None,
                    1 => Some(Box::new(args.into_iter().next().expect("single payload"))),
                    _ => {
                        return Err(self.error(
                            artifact,
                            "variant constructor call expects 0 or 1 argument",
                        ))
                    }
                };
                Ok(VMValue::Variant(name, payload))
            }
            VMValue::Builtin(name) => self.call_builtin(artifact, &name, args),
            _ => Err(self.error(artifact, "attempted to call a non-function value")),
        }
    }

    fn call_builtin(
        &mut self,
        artifact: &FvcArtifact,
        name: &str,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        match name {
            "List.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(xs) => {
                        let mut out = Vec::with_capacity(xs.len());
                        for x in xs {
                            out.push(self.call_value(artifact, func.clone(), vec![x])?);
                        }
                        Ok(VMValue::List(out))
                    }
                    _ => Err(self.error(artifact, "List.map requires a List as first argument")),
                }
            }
            "List.filter" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.filter requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(xs) => {
                        let mut out = Vec::new();
                        for x in xs {
                            let keep = self.call_value(artifact, func.clone(), vec![x.clone()])?;
                            match keep {
                                VMValue::Bool(true) => out.push(x),
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "List.filter predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ))
                                }
                            }
                        }
                        Ok(VMValue::List(out))
                    }
                    _ => Err(self.error(artifact, "List.filter requires a List as first argument")),
                }
            }
            "List.fold" => {
                if args.len() != 3 {
                    return Err(self.error(artifact, "List.fold requires 3 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let mut acc = it.next().expect("init");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(xs) => {
                        for x in xs {
                            acc = self.call_value(artifact, func.clone(), vec![acc, x])?;
                        }
                        Ok(acc)
                    }
                    _ => Err(self.error(artifact, "List.fold requires a List as first argument")),
                }
            }
            "List.flat_map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.flat_map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let func = it.next().expect("func");
                match list {
                    VMValue::List(xs) => {
                        let mut out: Vec<VMValue> = Vec::new();
                        for x in xs {
                            match self.call_value(artifact, func.clone(), vec![x])? {
                                VMValue::List(inner) => out.extend(inner),
                                other => return Err(self.error(artifact, &format!(
                                    "List.flat_map: callback must return List, got {}", vmvalue_type_name(&other)
                                ))),
                            }
                        }
                        Ok(VMValue::List(out))
                    }
                    _ => Err(self.error(artifact, "List.flat_map requires a List as first argument")),
                }
            }
            "List.sort" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.sort requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let cmp = it.next().expect("cmp");
                match list {
                    VMValue::List(mut xs) => {
                        let mut sort_err: Option<VMError> = None;
                        xs.sort_by(|a, b| {
                            if sort_err.is_some() { return std::cmp::Ordering::Equal; }
                            match self.call_value(artifact, cmp.clone(), vec![a.clone(), b.clone()]) {
                                Ok(VMValue::Int(n)) => {
                                    if n < 0 { std::cmp::Ordering::Less }
                                    else if n > 0 { std::cmp::Ordering::Greater }
                                    else { std::cmp::Ordering::Equal }
                                }
                                Ok(other) => {
                                    sort_err = Some(self.error(artifact, &format!(
                                        "List.sort: comparator must return Int, got {}", vmvalue_type_name(&other)
                                    )));
                                    std::cmp::Ordering::Equal
                                }
                                Err(e) => { sort_err = Some(e); std::cmp::Ordering::Equal }
                            }
                        });
                        if let Some(e) = sort_err { return Err(e); }
                        Ok(VMValue::List(xs))
                    }
                    _ => Err(self.error(artifact, "List.sort requires a List as first argument")),
                }
            }
            "List.find" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.find requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(xs) => {
                        for x in xs {
                            match self.call_value(artifact, pred.clone(), vec![x.clone()])? {
                                VMValue::Bool(true) => return Ok(VMValue::Variant("some".into(), Some(Box::new(x)))),
                                VMValue::Bool(false) => {}
                                other => return Err(self.error(artifact, &format!(
                                    "List.find predicate must return Bool, got {}", vmvalue_type_name(&other)
                                ))),
                            }
                        }
                        Ok(VMValue::Variant("none".into(), None))
                    }
                    _ => Err(self.error(artifact, "List.find requires a List as first argument")),
                }
            }
            "List.any" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.any requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(xs) => {
                        for x in xs {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(true) => return Ok(VMValue::Bool(true)),
                                VMValue::Bool(false) => {}
                                other => return Err(self.error(artifact, &format!(
                                    "List.any predicate must return Bool, got {}", vmvalue_type_name(&other)
                                ))),
                            }
                        }
                        Ok(VMValue::Bool(false))
                    }
                    _ => Err(self.error(artifact, "List.any requires a List as first argument")),
                }
            }
            "List.all" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.all requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(xs) => {
                        for x in xs {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(false) => return Ok(VMValue::Bool(false)),
                                VMValue::Bool(true) => {}
                                other => return Err(self.error(artifact, &format!(
                                    "List.all predicate must return Bool, got {}", vmvalue_type_name(&other)
                                ))),
                            }
                        }
                        Ok(VMValue::Bool(true))
                    }
                    _ => Err(self.error(artifact, "List.all requires a List as first argument")),
                }
            }
            "List.index_of" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "List.index_of requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let list = it.next().expect("list");
                let pred = it.next().expect("pred");
                match list {
                    VMValue::List(xs) => {
                        for (i, x) in xs.into_iter().enumerate() {
                            match self.call_value(artifact, pred.clone(), vec![x])? {
                                VMValue::Bool(true) => return Ok(VMValue::Variant("some".into(), Some(Box::new(VMValue::Int(i as i64))))),
                                VMValue::Bool(false) => {}
                                other => return Err(self.error(artifact, &format!(
                                    "List.index_of predicate must return Bool, got {}", vmvalue_type_name(&other)
                                ))),
                            }
                        }
                        Ok(VMValue::Variant("none".into(), None))
                    }
                    _ => Err(self.error(artifact, "List.index_of requires a List as first argument")),
                }
            }
            "Map.map_values" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Map.map_values requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let map = it.next().expect("map");
                let func = it.next().expect("func");
                match map {
                    VMValue::Record(m) => {
                        let mut out = HashMap::with_capacity(m.len());
                        for (k, v) in m {
                            let mapped = self.call_value(artifact, func.clone(), vec![v])?;
                            out.insert(k, mapped);
                        }
                        Ok(VMValue::Record(out))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Map.map_values requires a Map as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Map.filter_values" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Map.filter_values requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let map = it.next().expect("map");
                let func = it.next().expect("func");
                match map {
                    VMValue::Record(m) => {
                        let mut out = HashMap::new();
                        for (k, v) in m {
                            let keep = self.call_value(artifact, func.clone(), vec![v.clone()])?;
                            match keep {
                                VMValue::Bool(true) => {
                                    out.insert(k, v);
                                }
                                VMValue::Bool(false) => {}
                                other => {
                                    return Err(self.error(
                                        artifact,
                                        &format!(
                                            "Map.filter_values predicate must return Bool, got {}",
                                            vmvalue_type_name(&other)
                                        ),
                                    ))
                                }
                            }
                        }
                        Ok(VMValue::Record(out))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Map.filter_values requires a Map as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let func = it.next().expect("func");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        let inner = payload
                            .map(|value| *value)
                            .ok_or_else(|| self.error(artifact, "Option.map expected payload for some"))?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        Ok(VMValue::Variant("some".to_string(), Some(Box::new(mapped))))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        Ok(VMValue::Variant("none".to_string(), None))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!("Option.map requires an Option as first argument, got {}", vmvalue_type_name(&other)),
                    )),
                }
            }
            "Option.and_then" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.and_then requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let func = it.next().expect("func");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        let inner = payload
                            .map(|value| *value)
                            .ok_or_else(|| self.error(artifact, "Option.and_then expected payload for some"))?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        match mapped {
                            VMValue::Variant(tag, payload) if tag == "some" || tag == "none" => {
                                Ok(VMValue::Variant(tag, payload))
                            }
                            other => Err(self.error(
                                artifact,
                                &format!(
                                    "Option.and_then callback must return Option, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            )),
                        }
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        Ok(VMValue::Variant("none".to_string(), None))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.and_then requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.unwrap_or" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.unwrap_or requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let default = it.next().expect("default");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => payload
                        .map(|value| *value)
                        .ok_or_else(|| self.error(artifact, "Option.unwrap_or expected payload for some")),
                    VMValue::Variant(tag, None) if tag == "none" => Ok(default),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.unwrap_or requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.or_else" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.or_else requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let func = it.next().expect("func");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        Ok(VMValue::Variant(tag, payload))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        let mapped = self.call_value(artifact, func, vec![])?;
                        match mapped {
                            VMValue::Variant(tag, payload) if tag == "some" || tag == "none" => {
                                Ok(VMValue::Variant(tag, payload))
                            }
                            other => Err(self.error(
                                artifact,
                                &format!(
                                    "Option.or_else callback must return Option, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            )),
                        }
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.or_else requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.is_some" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Option.is_some requires 1 argument"));
                }
                match args.into_iter().next().expect("option") {
                    VMValue::Variant(tag, payload) if tag == "some" => Ok(VMValue::Bool(payload.is_some())),
                    VMValue::Variant(tag, None) if tag == "none" => Ok(VMValue::Bool(false)),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.is_some requires an Option argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.is_none" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Option.is_none requires 1 argument"));
                }
                match args.into_iter().next().expect("option") {
                    VMValue::Variant(tag, payload) if tag == "some" => Ok(VMValue::Bool(payload.is_none())),
                    VMValue::Variant(tag, None) if tag == "none" => Ok(VMValue::Bool(true)),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.is_none requires an Option argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Option.to_result" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Option.to_result requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let option = it.next().expect("option");
                let err = it.next().expect("err");
                match option {
                    VMValue::Variant(tag, payload) if tag == "some" => {
                        let inner = payload
                            .map(|value| *value)
                            .ok_or_else(|| self.error(artifact, "Option.to_result expected payload for some"))?;
                        Ok(VMValue::Variant("ok".to_string(), Some(Box::new(inner))))
                    }
                    VMValue::Variant(tag, None) if tag == "none" => {
                        Ok(VMValue::Variant("err".to_string(), Some(Box::new(err))))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Option.to_result requires an Option as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.map" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.map requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let func = it.next().expect("func");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        let inner = payload
                            .map(|value| *value)
                            .ok_or_else(|| self.error(artifact, "Result.map expected payload for ok"))?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        Ok(VMValue::Variant("ok".to_string(), Some(Box::new(mapped))))
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        Ok(VMValue::Variant(tag, payload))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.map requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.map_err" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.map_err requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let func = it.next().expect("func");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => Ok(VMValue::Variant(tag, payload)),
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        let inner = payload
                            .map(|value| *value)
                            .ok_or_else(|| self.error(artifact, "Result.map_err expected payload for err"))?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        Ok(VMValue::Variant("err".to_string(), Some(Box::new(mapped))))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.map_err requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.and_then" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.and_then requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let func = it.next().expect("func");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => {
                        let inner = payload
                            .map(|value| *value)
                            .ok_or_else(|| self.error(artifact, "Result.and_then expected payload for ok"))?;
                        let mapped = self.call_value(artifact, func, vec![inner])?;
                        match mapped {
                            VMValue::Variant(tag, payload) if tag == "ok" || tag == "err" => {
                                Ok(VMValue::Variant(tag, payload))
                            }
                            other => Err(self.error(
                                artifact,
                                &format!(
                                    "Result.and_then callback must return Result, got {}",
                                    vmvalue_type_name(&other)
                                ),
                            )),
                        }
                    }
                    VMValue::Variant(tag, payload) if tag == "err" => Ok(VMValue::Variant(tag, payload)),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.and_then requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.unwrap_or" => {
                if args.len() != 2 {
                    return Err(self.error(artifact, "Result.unwrap_or requires 2 arguments"));
                }
                let mut it = args.into_iter();
                let result = it.next().expect("result");
                let default = it.next().expect("default");
                match result {
                    VMValue::Variant(tag, payload) if tag == "ok" => payload
                        .map(|value| *value)
                        .ok_or_else(|| self.error(artifact, "Result.unwrap_or expected payload for ok")),
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        let _ = payload
                            .map(|value| *value)
                            .ok_or_else(|| self.error(artifact, "Result.unwrap_or expected payload for err"))?;
                        Ok(default)
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.unwrap_or requires a Result as first argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.is_ok" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Result.is_ok requires 1 argument"));
                }
                match args.into_iter().next().expect("result") {
                    VMValue::Variant(tag, payload) if tag == "ok" => Ok(VMValue::Bool(payload.is_some())),
                    VMValue::Variant(tag, payload) if tag == "err" => Ok(VMValue::Bool(false && payload.is_some())),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.is_ok requires a Result argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.is_err" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Result.is_err requires 1 argument"));
                }
                match args.into_iter().next().expect("result") {
                    VMValue::Variant(tag, payload) if tag == "ok" => Ok(VMValue::Bool(false && payload.is_some())),
                    VMValue::Variant(tag, payload) if tag == "err" => Ok(VMValue::Bool(payload.is_some())),
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.is_err requires a Result argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            "Result.to_option" => {
                if args.len() != 1 {
                    return Err(self.error(artifact, "Result.to_option requires 1 argument"));
                }
                match args.into_iter().next().expect("result") {
                    VMValue::Variant(tag, payload) if tag == "ok" => Ok(VMValue::Variant("some".to_string(), payload)),
                    VMValue::Variant(tag, payload) if tag == "err" => {
                        let _ = payload
                            .map(|value| *value)
                            .ok_or_else(|| self.error(artifact, "Result.to_option expected payload for err"))?;
                        Ok(VMValue::Variant("none".to_string(), None))
                    }
                    other => Err(self.error(
                        artifact,
                        &format!(
                            "Result.to_option requires a Result argument, got {}",
                            vmvalue_type_name(&other)
                        ),
                    )),
                }
            }
            _ => {
                if let Some(target_idx) = artifact
                    .globals
                    .iter()
                    .position(|g| g.kind == 0 && artifact.str_table.get(g.name_idx as usize).is_some_and(|n| n == name))
                {
                    return self.call_value(artifact, VMValue::CompiledFn(artifact.globals[target_idx].fn_idx as usize), args);
                }
                vm_call_builtin(name, args, &mut self.emit_log, self.db_path.as_deref())
                    .map_err(|e| self.error(artifact, &e))
            }
        }
    }

    fn pop_pair(&mut self, artifact: &FvcArtifact) -> Result<(VMValue, VMValue), VMError> {
        let right = self.stack.pop().ok_or_else(|| self.error(artifact, "stack underflow"))?;
        let left = self.stack.pop().ok_or_else(|| self.error(artifact, "stack underflow"))?;
        Ok((left, right))
    }
}

fn constant_to_value(constant: Constant) -> VMValue {
    match constant {
        Constant::Int(v) => VMValue::Int(v),
        Constant::Float(v) => VMValue::Float(v),
        Constant::Str(v) => VMValue::Str(v),
        Constant::Name(v) => VMValue::Str(v),
    }
}

impl From<Value> for VMValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(v) => VMValue::Bool(v),
            Value::Int(v) => VMValue::Int(v),
            Value::Float(v) => VMValue::Float(v),
            Value::Str(v) => VMValue::Str(v),
            Value::Unit => VMValue::Unit,
            Value::List(values) => VMValue::List(values.into_iter().map(VMValue::from).collect()),
            Value::Record(map) => VMValue::Record(
                map.into_iter().map(|(k, v)| (k, VMValue::from(v))).collect(),
            ),
            Value::Variant(tag, payload) => VMValue::Variant(
                tag,
                payload.map(|inner| Box::new(VMValue::from(*inner))),
            ),
            other => panic!("unsupported VM argument value: {other:?}"),
        }
    }
}

impl From<VMValue> for Value {
    fn from(value: VMValue) -> Self {
        match value {
            VMValue::Bool(v) => Value::Bool(v),
            VMValue::Int(v) => Value::Int(v),
            VMValue::Float(v) => Value::Float(v),
            VMValue::Str(v) => Value::Str(v),
            VMValue::Unit => Value::Unit,
            VMValue::List(values) => Value::List(values.into_iter().map(Value::from).collect()),
            VMValue::Record(map) => Value::Record(
                map.into_iter().map(|(k, v)| (k, Value::from(v))).collect(),
            ),
            VMValue::Variant(tag, payload) => Value::Variant(
                tag,
                payload.map(|inner| Box::new(Value::from(*inner))),
            ),
            VMValue::VariantCtor(name) => Value::Variant(name, None),
            VMValue::CompiledFn(idx) => Value::Str(format!("<fn:{idx}>")),
            VMValue::Closure(idx, captures) => {
                Value::Str(format!("<closure:{idx};captures={}>", captures.len()))
            }
            VMValue::Builtin(name) => Value::Str(format!("<builtin:{name}>")),
        }
    }
}

fn apply_numeric_binop(
    left: VMValue,
    right: VMValue,
    int_op: impl FnOnce(i64, i64) -> i64,
    float_op: impl FnOnce(f64, f64) -> f64,
    op_name: &str,
    artifact: &FvcArtifact,
    frames: &[CallFrame],
) -> Result<VMValue, VMError> {
    match (left, right) {
        (VMValue::Int(a), VMValue::Int(b)) => Ok(VMValue::Int(int_op(a, b))),
        (VMValue::Float(a), VMValue::Float(b)) => Ok(VMValue::Float(float_op(a, b))),
        _ => Err(vm_error_from_frames(
            artifact,
            frames,
            format!("type error in {op_name}: numeric operands required"),
        )),
    }
}

fn compare_pair(
    pair: (VMValue, VMValue),
    cmp: impl FnOnce(f64, f64) -> bool,
    artifact: &FvcArtifact,
    frames: &[CallFrame],
) -> Result<VMValue, VMError> {
    match pair {
        (VMValue::Int(a), VMValue::Int(b)) => Ok(VMValue::Bool(cmp(a as f64, b as f64))),
        (VMValue::Float(a), VMValue::Float(b)) => Ok(VMValue::Bool(cmp(a, b))),
        _ => Err(vm_error_from_frames(
            artifact,
            frames,
            "type error in comparison: numeric operands required".to_string(),
        )),
    }
}

fn vm_error_from_frames(artifact: &FvcArtifact, frames: &[CallFrame], message: String) -> VMError {
    if let Some(frame) = frames.last() {
        let function = &artifact.functions[frame.fn_idx];
        let fn_name = artifact
            .str_table
            .get(function.name_idx as usize)
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string());
        VMError { message, fn_name, ip: frame.ip }
    } else {
        VMError { message, fn_name: "<none>".to_string(), ip: 0 }
    }
}

fn vmvalue_repr(v: &VMValue) -> String {
    match v {
        VMValue::Bool(b) => b.to_string(),
        VMValue::Int(n) => n.to_string(),
        VMValue::Float(f) => {
            if f.fract() == 0.0 { format!("{:.1}", f) } else { f.to_string() }
        }
        VMValue::Str(s) => format!("\"{}\"", s),
        VMValue::Unit => "()".to_string(),
        VMValue::List(vs) => {
            let items: Vec<_> = vs.iter().map(vmvalue_repr).collect();
            format!("[{}]", items.join(", "))
        }
        VMValue::Record(m) => {
            let mut pairs: Vec<_> = m.iter().map(|(k, v)| format!("{}: {}", k, vmvalue_repr(v))).collect();
            pairs.sort();
            format!("{{ {} }}", pairs.join(", "))
        }
        VMValue::Variant(name, None) => name.clone(),
        VMValue::Variant(name, Some(payload)) => format!("{}({})", name, vmvalue_repr(payload)),
        VMValue::CompiledFn(idx) => format!("<fn:{}>", idx),
        VMValue::Closure(idx, caps) => format!("<closure:{};captures={}>", idx, caps.len()),
        VMValue::VariantCtor(name) => format!("<ctor:{}>", name),
        VMValue::Builtin(name) => format!("<builtin:{}>", name),
    }
}

fn vmvalue_type_name(v: &VMValue) -> &'static str {
    match v {
        VMValue::Bool(_) => "Bool",
        VMValue::Int(_) => "Int",
        VMValue::Float(_) => "Float",
        VMValue::Str(_) => "String",
        VMValue::Unit => "Unit",
        VMValue::List(_) => "List",
        VMValue::Record(_) => "Record",
        VMValue::Variant(_, _) => "Variant",
        VMValue::VariantCtor(_) => "VariantCtor",
        VMValue::CompiledFn(_) => "CompiledFn",
        VMValue::Closure(_, _) => "Closure",
        VMValue::Builtin(_) => "Builtin",
    }
}

fn json_variant_vm(name: &str, payload: Option<VMValue>) -> VMValue {
    VMValue::Variant(name.to_string(), payload.map(Box::new))
}

fn serde_to_vm_json(value: SerdeJsonValue) -> VMValue {
    match value {
        SerdeJsonValue::Null => json_variant_vm("json_null", None),
        SerdeJsonValue::Bool(b) => json_variant_vm("json_bool", Some(VMValue::Bool(b))),
        SerdeJsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                json_variant_vm("json_int", Some(VMValue::Int(i)))
            } else {
                json_variant_vm("json_float", Some(VMValue::Float(n.as_f64().unwrap_or(0.0))))
            }
        }
        SerdeJsonValue::String(s) => json_variant_vm("json_str", Some(VMValue::Str(s))),
        SerdeJsonValue::Array(items) => {
            json_variant_vm(
                "json_array",
                Some(VMValue::List(items.into_iter().map(serde_to_vm_json).collect())),
            )
        }
        SerdeJsonValue::Object(map) => {
            let mut fields = HashMap::new();
            for (k, v) in map {
                fields.insert(k, serde_to_vm_json(v));
            }
            json_variant_vm("json_object", Some(VMValue::Record(fields)))
        }
    }
}

fn vm_json_to_serde(value: &VMValue) -> Option<SerdeJsonValue> {
    match value {
        VMValue::Variant(tag, None) if tag == "json_null" => Some(SerdeJsonValue::Null),
        VMValue::Variant(tag, Some(payload)) if tag == "json_bool" => match payload.as_ref() {
            VMValue::Bool(b) => Some(SerdeJsonValue::Bool(*b)),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_int" => match payload.as_ref() {
            VMValue::Int(i) => Some(SerdeJsonValue::Number((*i).into())),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_float" => match payload.as_ref() {
            VMValue::Float(f) => serde_json::Number::from_f64(*f).map(SerdeJsonValue::Number),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_str" => match payload.as_ref() {
            VMValue::Str(s) => Some(SerdeJsonValue::String(s.clone())),
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_array" => match payload.as_ref() {
            VMValue::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(vm_json_to_serde(item)?);
                }
                Some(SerdeJsonValue::Array(out))
            }
            _ => None,
        },
        VMValue::Variant(tag, Some(payload)) if tag == "json_object" => match payload.as_ref() {
            VMValue::Record(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    out.insert(k.clone(), vm_json_to_serde(v)?);
                }
                Some(SerdeJsonValue::Object(out))
            }
            _ => None,
        },
        _ => None,
    }
}

fn vm_string(value: VMValue, context: &str) -> Result<String, String> {
    match value {
        VMValue::Str(s) => Ok(s),
        other => Err(format!("{} expects String, got {}", context, vmvalue_type_name(&other))),
    }
}

fn vm_string_list(value: VMValue, context: &str) -> Result<Vec<String>, String> {
    match value {
        VMValue::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(vm_string(item, context)?);
            }
            Ok(out)
        }
        other => Err(format!("{} expects List<String>, got {}", context, vmvalue_type_name(&other))),
    }
}

fn vmvalue_to_sql(value: &VMValue) -> rusqlite::types::Value {
    match value {
        VMValue::Int(n) => rusqlite::types::Value::Integer(*n),
        VMValue::Float(f) => rusqlite::types::Value::Real(*f),
        VMValue::Str(s) => rusqlite::types::Value::Text(s.clone()),
        VMValue::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
        VMValue::Unit => rusqlite::types::Value::Null,
        other => rusqlite::types::Value::Text(vmvalue_repr(other)),
    }
}

fn sqlite_value_to_string(value: rusqlite::types::Value) -> String {
    match value {
        rusqlite::types::Value::Null => "null".to_string(),
        rusqlite::types::Value::Integer(n) => n.to_string(),
        rusqlite::types::Value::Real(f) => f.to_string(),
        rusqlite::types::Value::Text(s) => s,
        rusqlite::types::Value::Blob(bytes) => format!("<blob:{} bytes>", bytes.len()),
    }
}

fn with_db_path<T, F>(db_path: Option<&str>, f: F) -> Result<T, String>
where
    F: FnOnce(&Connection) -> Result<T, String>,
{
    let path = db_path.ok_or_else(|| "Db not initialized 窶・run with --db <path> flag".to_string())?;
    let mut dbs = SHARED_DBS.lock().map_err(|_| "Db mutex poisoned".to_string())?;
    let entry_idx = if let Some(idx) = dbs.iter().position(|(p, _)| p == path) {
        idx
    } else {
        let conn = if path == ":memory:" {
            Connection::open_in_memory().map_err(|e| format!("Db open failed: {}", e))?
        } else {
            Connection::open(path).map_err(|e| format!("Db open failed for `{}`: {}", path, e))?
        };
        dbs.push((path.to_string(), conn));
        dbs.len() - 1
    };
    let (_, conn) = &dbs[entry_idx];
    f(conn)
}

fn vm_call_builtin(
    name: &str,
    args: Vec<VMValue>,
    emit_log: &mut Vec<VMValue>,
    db_path: Option<&str>,
) -> Result<VMValue, String> {
    match name {
        "IO.println" => {
            let s = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(v) => vmvalue_repr(&v),
                None => return Err("IO.println requires 1 argument".to_string()),
            };
            println!("{}", s);
            Ok(VMValue::Unit)
        }
        "IO.println_int" => match args.as_slice() {
            [VMValue::Int(n)] => {
                println!("{}", n);
                Ok(VMValue::Unit)
            }
            [_] => Err("IO.println_int requires an Int argument".to_string()),
            _ => Err("IO.println_int requires 1 argument".to_string()),
        },
        "IO.println_float" => match args.as_slice() {
            [VMValue::Float(n)] => {
                println!("{}", n);
                Ok(VMValue::Unit)
            }
            [_] => Err("IO.println_float requires a Float argument".to_string()),
            _ => Err("IO.println_float requires 1 argument".to_string()),
        },
        "IO.println_bool" => match args.as_slice() {
            [VMValue::Bool(b)] => {
                println!("{}", if *b { "true" } else { "false" });
                Ok(VMValue::Unit)
            }
            [_] => Err("IO.println_bool requires a Bool argument".to_string()),
            _ => Err("IO.println_bool requires 1 argument".to_string()),
        },
        "IO.print" => {
            use std::io::Write;
            let s = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(v) => vmvalue_repr(&v),
                None => return Err("IO.print requires 1 argument".to_string()),
            };
            print!("{}", s);
            std::io::stdout().flush().ok();
            Ok(VMValue::Unit)
        }
        "Debug.show" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Debug.show requires 1 argument".to_string())?;
            Ok(VMValue::Str(vmvalue_repr(&v)))
        }
        "assert" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "assert requires 1 argument".to_string())?;
            match v {
                VMValue::Bool(true)  => Ok(VMValue::Unit),
                VMValue::Bool(false) => Err("assertion failed".to_string()),
                other => Err(format!("assert requires Bool, got {}", vmvalue_type_name(&other))),
            }
        }
        "assert_eq" => {
            let mut it = args.into_iter();
            let a = it.next().ok_or_else(|| "assert_eq requires 2 arguments".to_string())?;
            let b = it.next().ok_or_else(|| "assert_eq requires 2 arguments".to_string())?;
            if vmvalue_repr(&a) == vmvalue_repr(&b) {
                Ok(VMValue::Unit)
            } else {
                Err(format!("assert_eq failed: left={}, right={}", vmvalue_repr(&a), vmvalue_repr(&b)))
            }
        }
        "assert_ne" => {
            let mut it = args.into_iter();
            let a = it.next().ok_or_else(|| "assert_ne requires 2 arguments".to_string())?;
            let b = it.next().ok_or_else(|| "assert_ne requires 2 arguments".to_string())?;
            if vmvalue_repr(&a) != vmvalue_repr(&b) {
                Ok(VMValue::Unit)
            } else {
                Err(format!("assert_ne failed: both equal to {}", vmvalue_repr(&a)))
            }
        }
        "Result.ok" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Result.ok requires 1 argument".to_string())?;
            Ok(VMValue::Variant("ok".to_string(), Some(Box::new(v))))
        }
        "Result.err" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Result.err requires 1 argument".to_string())?;
            Ok(VMValue::Variant("err".to_string(), Some(Box::new(v))))
        }
        "Option.some" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Option.some requires 1 argument".to_string())?;
            Ok(VMValue::Variant("some".to_string(), Some(Box::new(v))))
        }
        "Option.none" => {
            Ok(VMValue::Variant("none".to_string(), None))
        }
        "Int.show.show" | "Float.show.show" => {
            let v = args.into_iter().next()
                .ok_or_else(|| format!("{} requires 1 argument", name))?;
            Ok(VMValue::Str(match v {
                VMValue::Int(n) => n.to_string(),
                VMValue::Float(f) => {
                    if f.fract() == 0.0 { format!("{:.1}", f) } else { f.to_string() }
                }
                other => return Err(format!("{} requires Int/Float, got {:?}", name, other)),
            }))
        }
        "Bool.show.show" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Bool.show.show requires 1 argument".to_string())?;
            Ok(VMValue::Str(match v {
                VMValue::Bool(b) => b.to_string(),
                other => return Err(format!("Bool.show.show requires Bool, got {:?}", other)),
            }))
        }
        "String.show.show" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "String.show.show requires 1 argument".to_string())?;
            Ok(VMValue::Str(match v {
                VMValue::Str(s) => format!("\"{}\"", s),
                other => return Err(format!("String.show.show requires String, got {:?}", other)),
            }))
        }
        "Int.ord.compare" => {
            let mut it = args.into_iter();
            let a = it.next().ok_or_else(|| "Int.ord.compare requires 2 arguments".to_string())?;
            let b = it.next().ok_or_else(|| "Int.ord.compare requires 2 arguments".to_string())?;
            match (a, b) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Int(match x.cmp(&y) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                })),
                _ => Err("Int.ord.compare requires two Int arguments".to_string()),
            }
        }
        "Int.eq.equals" => {
            let mut it = args.into_iter();
            let a = it.next().ok_or_else(|| "Int.eq.equals requires 2 arguments".to_string())?;
            let b = it.next().ok_or_else(|| "Int.eq.equals requires 2 arguments".to_string())?;
            match (a, b) {
                (VMValue::Int(x), VMValue::Int(y)) => Ok(VMValue::Bool(x == y)),
                _ => Err("Int.eq.equals requires two Int arguments".to_string()),
            }
        }
        "String.concat" => {
            let mut it = args.into_iter();
            let a = it.next().ok_or_else(|| "String.concat requires 2 arguments".to_string())?;
            let b = it.next().ok_or_else(|| "String.concat requires 2 arguments".to_string())?;
            match (a, b) {
                (VMValue::Str(x), VMValue::Str(y)) => Ok(VMValue::Str(x + &y)),
                _ => Err("String.concat requires two String arguments".to_string()),
            }
        }
        "String.length" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "String.length requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Int(s.len() as i64)),
                _ => Err("String.length requires a String argument".to_string()),
            }
        }
        "String.is_empty" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "String.is_empty requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Bool(s.is_empty())),
                _ => Err("String.is_empty requires a String argument".to_string()),
            }
        }
        "String.trim" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "String.trim requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.trim().to_string())),
                _ => Err("String.trim requires a String argument".to_string()),
            }
        }
        "String.upper" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "String.upper requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.to_uppercase())),
                _ => Err("String.upper requires a String argument".to_string()),
            }
        }
        "String.lower" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "String.lower requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(VMValue::Str(s.to_lowercase())),
                _ => Err("String.lower requires a String argument".to_string()),
            }
        }
        "String.split" => {
            let mut it = args.into_iter();
            let s = it.next().ok_or_else(|| "String.split requires 2 arguments".to_string())?;
            let d = it.next().ok_or_else(|| "String.split requires 2 arguments".to_string())?;
            match (s, d) {
                (VMValue::Str(s), VMValue::Str(delim)) => Ok(VMValue::List(
                    s.split(&*delim).map(|p| VMValue::Str(p.to_string())).collect(),
                )),
                _ => Err("String.split requires (String, String)".to_string()),
            }
        }
        "String.join" => {
            let mut it = args.into_iter();
            let xs = it.next().ok_or_else(|| "String.join requires 2 arguments".to_string())?;
            let sep = it.next().ok_or_else(|| "String.join requires 2 arguments".to_string())?;
            match (xs, sep) {
                (VMValue::List(values), VMValue::Str(sep)) => {
                    let mut parts = Vec::with_capacity(values.len());
                    for value in values {
                        match value {
                            VMValue::Str(s) => parts.push(s),
                            _ => return Err("String.join requires List<String> as first argument".to_string()),
                        }
                    }
                    Ok(VMValue::Str(parts.join(&sep)))
                }
                _ => Err("String.join requires (List<String>, String)".to_string()),
            }
        }
        "String.replace" => {
            let mut it = args.into_iter();
            let s = it.next().ok_or_else(|| "String.replace requires 3 arguments".to_string())?;
            let from = it.next().ok_or_else(|| "String.replace requires 3 arguments".to_string())?;
            let to = it.next().ok_or_else(|| "String.replace requires 3 arguments".to_string())?;
            match (s, from, to) {
                (VMValue::Str(s), VMValue::Str(from), VMValue::Str(to)) => {
                    Ok(VMValue::Str(s.replace(&from, &to)))
                }
                _ => Err("String.replace requires (String, String, String)".to_string()),
            }
        }
        "String.starts_with" => {
            let mut it = args.into_iter();
            let s = it.next().ok_or_else(|| "String.starts_with requires 2 arguments".to_string())?;
            let prefix = it.next().ok_or_else(|| "String.starts_with requires 2 arguments".to_string())?;
            match (s, prefix) {
                (VMValue::Str(s), VMValue::Str(prefix)) => Ok(VMValue::Bool(s.starts_with(&prefix))),
                _ => Err("String.starts_with requires (String, String)".to_string()),
            }
        }
        "String.ends_with" => {
            let mut it = args.into_iter();
            let s = it.next().ok_or_else(|| "String.ends_with requires 2 arguments".to_string())?;
            let suffix = it.next().ok_or_else(|| "String.ends_with requires 2 arguments".to_string())?;
            match (s, suffix) {
                (VMValue::Str(s), VMValue::Str(suffix)) => Ok(VMValue::Bool(s.ends_with(&suffix))),
                _ => Err("String.ends_with requires (String, String)".to_string()),
            }
        }
        "String.contains" => {
            let mut it = args.into_iter();
            let s = it.next().ok_or_else(|| "String.contains requires 2 arguments".to_string())?;
            let sub = it.next().ok_or_else(|| "String.contains requires 2 arguments".to_string())?;
            match (s, sub) {
                (VMValue::Str(s), VMValue::Str(sub)) => Ok(VMValue::Bool(s.contains(&sub))),
                _ => Err("String.contains requires (String, String)".to_string()),
            }
        }
        "String.slice" => {
            let mut it = args.into_iter();
            let s = it.next().ok_or_else(|| "String.slice requires 3 arguments".to_string())?;
            let start = it.next().ok_or_else(|| "String.slice requires 3 arguments".to_string())?;
            let end = it.next().ok_or_else(|| "String.slice requires 3 arguments".to_string())?;
            match (s, start, end) {
                (VMValue::Str(s), VMValue::Int(start), VMValue::Int(end)) => {
                    if start < 0 || end < start {
                        return Err("String.slice requires 0 <= start <= end".to_string());
                    }
                    let chars: Vec<char> = s.chars().collect();
                    let start = start as usize;
                    let end = end as usize;
                    if end > chars.len() {
                        return Err("String.slice end is out of bounds".to_string());
                    }
                    Ok(VMValue::Str(chars[start..end].iter().collect()))
                }
                _ => Err("String.slice requires (String, Int, Int)".to_string()),
            }
        }
        "String.repeat" => {
            let mut it = args.into_iter();
            let s = it.next().ok_or_else(|| "String.repeat requires 2 arguments".to_string())?;
            let n = it.next().ok_or_else(|| "String.repeat requires 2 arguments".to_string())?;
            match (s, n) {
                (VMValue::Str(s), VMValue::Int(n)) if n >= 0 => Ok(VMValue::Str(s.repeat(n as usize))),
                (VMValue::Str(_), VMValue::Int(_)) => Err("String.repeat requires a non-negative count".to_string()),
                _ => Err("String.repeat requires (String, Int)".to_string()),
            }
        }
        "String.char_at" => {
            let mut it = args.into_iter();
            let s = it.next().ok_or_else(|| "String.char_at requires 2 arguments".to_string())?;
            let idx = it.next().ok_or_else(|| "String.char_at requires 2 arguments".to_string())?;
            match (s, idx) {
                (VMValue::Str(s), VMValue::Int(idx)) => {
                    if idx < 0 {
                        return Ok(VMValue::Variant("none".to_string(), None));
                    }
                    let ch = s.chars().nth(idx as usize);
                    Ok(match ch {
                        Some(ch) => VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Str(ch.to_string())))),
                        None => VMValue::Variant("none".to_string(), None),
                    })
                }
                _ => Err("String.char_at requires (String, Int)".to_string()),
            }
        }
        "String.to_int" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "String.to_int requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(match s.parse::<i64>() {
                    Ok(n) => VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(n)))),
                    Err(_) => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("String.to_int requires a String argument".to_string()),
            }
        }
        "String.to_float" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "String.to_float requires 1 argument".to_string())?;
            match v {
                VMValue::Str(s) => Ok(match s.parse::<f64>() {
                    Ok(n) => VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Float(n)))),
                    Err(_) => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("String.to_float requires a String argument".to_string()),
            }
        }
        "String.from_int" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "String.from_int requires 1 argument".to_string())?;
            match v {
                VMValue::Int(n) => Ok(VMValue::Str(n.to_string())),
                _ => Err("String.from_int requires an Int argument".to_string()),
            }
        }
        "String.from_float" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "String.from_float requires 1 argument".to_string())?;
            match v {
                VMValue::Float(n) => Ok(VMValue::Str(n.to_string())),
                _ => Err("String.from_float requires a Float argument".to_string()),
            }
        }
        "List.length" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "List.length requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => Ok(VMValue::Int(xs.len() as i64)),
                _ => Err("List.length requires a List argument".to_string()),
            }
        }
        "List.is_empty" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "List.is_empty requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => Ok(VMValue::Bool(xs.is_empty())),
                _ => Err("List.is_empty requires a List argument".to_string()),
            }
        }
        "List.first" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "List.first requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => Ok(match xs.into_iter().next() {
                    Some(first) => VMValue::Variant("some".to_string(), Some(Box::new(first))),
                    None => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("List.first requires a List argument".to_string()),
            }
        }
        "List.last" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "List.last requires 1 argument".to_string())?;
            match v {
                VMValue::List(mut xs) => Ok(match xs.pop() {
                    Some(last) => VMValue::Variant("some".to_string(), Some(Box::new(last))),
                    None => VMValue::Variant("none".to_string(), None),
                }),
                _ => Err("List.last requires a List argument".to_string()),
            }
        }
        "List.push" => {
            let mut it = args.into_iter();
            let list = it.next().ok_or_else(|| "List.push requires 2 arguments".to_string())?;
            let item = it.next().ok_or_else(|| "List.push requires 2 arguments".to_string())?;
            match list {
                VMValue::List(mut xs) => { xs.push(item); Ok(VMValue::List(xs)) }
                _ => Err("List.push requires a List as first argument".to_string()),
            }
        }
        "List.zip" => {
            let mut it = args.into_iter();
            let xs = it.next().ok_or_else(|| "List.zip requires 2 arguments".to_string())?;
            let ys = it.next().ok_or_else(|| "List.zip requires 2 arguments".to_string())?;
            match (xs, ys) {
                (VMValue::List(xs), VMValue::List(ys)) => {
                    let pairs: Vec<VMValue> = xs.into_iter().zip(ys.into_iter()).map(|(x, y)| {
                        let mut m = HashMap::new();
                        m.insert("first".to_string(), x);
                        m.insert("second".to_string(), y);
                        VMValue::Record(m)
                    }).collect();
                    Ok(VMValue::List(pairs))
                }
                _ => Err("List.zip expects (List, List)".to_string()),
            }
        }
        "List.range" => {
            let mut it = args.into_iter();
            let start = it.next().ok_or_else(|| "List.range requires 2 arguments".to_string())?;
            let end = it.next().ok_or_else(|| "List.range requires 2 arguments".to_string())?;
            match (start, end) {
                (VMValue::Int(s), VMValue::Int(e)) => {
                    Ok(VMValue::List((s..e).map(VMValue::Int).collect()))
                }
                _ => Err("List.range expects (Int, Int)".to_string()),
            }
        }
        "List.reverse" => {
            match args.into_iter().next() {
                Some(VMValue::List(mut xs)) => { xs.reverse(); Ok(VMValue::List(xs)) }
                _ => Err("List.reverse expects List".to_string()),
            }
        }
        "List.concat" => {
            let mut it = args.into_iter();
            let xs = it.next().ok_or_else(|| "List.concat requires 2 arguments".to_string())?;
            let ys = it.next().ok_or_else(|| "List.concat requires 2 arguments".to_string())?;
            match (xs, ys) {
                (VMValue::List(mut xs), VMValue::List(ys)) => { xs.extend(ys); Ok(VMValue::List(xs)) }
                _ => Err("List.concat expects (List, List)".to_string()),
            }
        }
        "List.take" => {
            let mut it = args.into_iter();
            let list = it.next().ok_or_else(|| "List.take requires 2 arguments".to_string())?;
            let n = it.next().ok_or_else(|| "List.take requires 2 arguments".to_string())?;
            match (list, n) {
                (VMValue::List(xs), VMValue::Int(n)) => {
                    Ok(VMValue::List(xs.into_iter().take(n.max(0) as usize).collect()))
                }
                _ => Err("List.take expects (List, Int)".to_string()),
            }
        }
        "List.drop" => {
            let mut it = args.into_iter();
            let list = it.next().ok_or_else(|| "List.drop requires 2 arguments".to_string())?;
            let n = it.next().ok_or_else(|| "List.drop requires 2 arguments".to_string())?;
            match (list, n) {
                (VMValue::List(xs), VMValue::Int(n)) => {
                    Ok(VMValue::List(xs.into_iter().skip(n.max(0) as usize).collect()))
                }
                _ => Err("List.drop expects (List, Int)".to_string()),
            }
        }
        "List.enumerate" => {
            match args.into_iter().next() {
                Some(VMValue::List(xs)) => {
                    let pairs: Vec<VMValue> = xs.into_iter().enumerate().map(|(i, v)| {
                        let mut m = HashMap::new();
                        m.insert("first".to_string(), VMValue::Int(i as i64));
                        m.insert("second".to_string(), v);
                        VMValue::Record(m)
                    }).collect();
                    Ok(VMValue::List(pairs))
                }
                _ => Err("List.enumerate expects List".to_string()),
            }
        }
        "List.join" => {
            let mut it = args.into_iter();
            let list = it.next().ok_or_else(|| "List.join requires 2 arguments".to_string())?;
            let sep = it.next().ok_or_else(|| "List.join requires 2 arguments".to_string())?;
            match (list, sep) {
                (VMValue::List(xs), VMValue::Str(sep)) => {
                    let mut parts = Vec::with_capacity(xs.len());
                    for v in xs {
                        match v {
                            VMValue::Str(s) => parts.push(s),
                            other => return Err(format!("List.join expects List<String>, got {:?}", other)),
                        }
                    }
                    Ok(VMValue::Str(parts.join(&sep)))
                }
                _ => Err("List.join expects (List<String>, String)".to_string()),
            }
        }
        "Map.set" => {
            let mut it = args.into_iter();
            let map = it.next().ok_or_else(|| "Map.set requires 3 arguments".to_string())?;
            let key = it.next().ok_or_else(|| "Map.set requires 3 arguments".to_string())?;
            let val = it.next().ok_or_else(|| "Map.set requires 3 arguments".to_string())?;
            let mut m = match map {
                VMValue::Record(m) => m,
                VMValue::Unit => HashMap::new(),
                _ => return Err("Map.set requires a Record or Unit as first argument".to_string()),
            };
            let k = match key {
                VMValue::Str(s) => s,
                _ => return Err("Map.set requires a String key".to_string()),
            };
            m.insert(k, val);
            Ok(VMValue::Record(m))
        }
        "Map.get" => {
            let mut it = args.into_iter();
            let map = it.next().ok_or_else(|| "Map.get requires 2 arguments".to_string())?;
            let key = it.next().ok_or_else(|| "Map.get requires 2 arguments".to_string())?;
            let m = match map {
                VMValue::Record(m) => m,
                _ => return Err("Map.get requires a Record as first argument".to_string()),
            };
            let k = match key {
                VMValue::Str(s) => s,
                _ => return Err("Map.get requires a String key".to_string()),
            };
            Ok(match m.get(&k) {
                Some(v) => VMValue::Variant("some".to_string(), Some(Box::new(v.clone()))),
                None => VMValue::Variant("none".to_string(), None),
            })
        }
        "Map.delete" => {
            let mut it = args.into_iter();
            let map = it.next().ok_or_else(|| "Map.delete requires 2 arguments".to_string())?;
            let key = it.next().ok_or_else(|| "Map.delete requires 2 arguments".to_string())?;
            let mut m = match map {
                VMValue::Record(m) => m,
                _ => return Err("Map.delete requires a Record as first argument".to_string()),
            };
            let k = match key {
                VMValue::Str(s) => s,
                _ => return Err("Map.delete requires a String key".to_string()),
            };
            m.remove(&k);
            Ok(VMValue::Record(m))
        }
        "Map.keys" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Map.keys requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => {
                    let mut keys: Vec<VMValue> = m.keys().map(|k| VMValue::Str(k.clone())).collect();
                    keys.sort_by(|a, b| match (a, b) {
                        (VMValue::Str(x), VMValue::Str(y)) => x.cmp(y),
                        _ => std::cmp::Ordering::Equal,
                    });
                    Ok(VMValue::List(keys))
                }
                _ => Err("Map.keys requires a Record (map) argument".to_string()),
            }
        }
        "Map.values" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Map.values requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => {
                    let mut pairs: Vec<_> = m.iter().collect();
                    pairs.sort_by(|a, b| a.0.cmp(b.0));
                    Ok(VMValue::List(pairs.into_iter().map(|(_, v)| v.clone()).collect()))
                }
                _ => Err("Map.values requires a Record (map) argument".to_string()),
            }
        }
        "Map.has_key" => {
            let mut it = args.into_iter();
            let map = it.next().ok_or_else(|| "Map.has_key requires 2 arguments".to_string())?;
            let key = it.next().ok_or_else(|| "Map.has_key requires 2 arguments".to_string())?;
            match (map, key) {
                (VMValue::Record(m), VMValue::Str(k)) => Ok(VMValue::Bool(m.contains_key(&k))),
                _ => Err("Map.has_key requires (Map, String)".to_string()),
            }
        }
        "Map.size" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Map.size requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => Ok(VMValue::Int(m.len() as i64)),
                _ => Err("Map.size requires a Map argument".to_string()),
            }
        }
        "Map.is_empty" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Map.is_empty requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => Ok(VMValue::Bool(m.is_empty())),
                _ => Err("Map.is_empty requires a Map argument".to_string()),
            }
        }
        "Map.merge" => {
            let mut it = args.into_iter();
            let base = it.next().ok_or_else(|| "Map.merge requires 2 arguments".to_string())?;
            let overrides = it.next().ok_or_else(|| "Map.merge requires 2 arguments".to_string())?;
            match (base, overrides) {
                (VMValue::Record(mut base), VMValue::Record(overrides)) => {
                    for (k, v) in overrides {
                        base.insert(k, v);
                    }
                    Ok(VMValue::Record(base))
                }
                _ => Err("Map.merge requires (Map, Map)".to_string()),
            }
        }
        "Map.from_list" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Map.from_list requires 1 argument".to_string())?;
            match v {
                VMValue::List(xs) => {
                    let mut out = HashMap::with_capacity(xs.len());
                    for pair in xs {
                        match pair {
                            VMValue::Record(mut fields) => {
                                let first = fields.remove("first");
                                let second = fields.remove("second");
                                match (first, second) {
                                    (Some(VMValue::Str(k)), Some(v)) => {
                                        out.insert(k, v);
                                    }
                                    _ => {
                                        return Err(
                                            "Map.from_list requires Pair-like records with { first: String second: V }"
                                                .to_string(),
                                        )
                                    }
                                }
                            }
                            _ => {
                                return Err(
                                    "Map.from_list requires List<Pair<String, V>>".to_string(),
                                )
                            }
                        }
                    }
                    Ok(VMValue::Record(out))
                }
                _ => Err("Map.from_list requires a List argument".to_string()),
            }
        }
        "Map.to_list" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Map.to_list requires 1 argument".to_string())?;
            match v {
                VMValue::Record(m) => {
                    let mut pairs: Vec<_> = m.into_iter().collect();
                    pairs.sort_by(|a, b| a.0.cmp(&b.0));
                    Ok(VMValue::List(
                        pairs
                            .into_iter()
                            .map(|(k, v)| {
                                let mut fields = HashMap::new();
                                fields.insert("first".to_string(), VMValue::Str(k));
                                fields.insert("second".to_string(), v);
                                VMValue::Record(fields)
                            })
                            .collect(),
                    ))
                }
                _ => Err("Map.to_list requires a Map argument".to_string()),
            }
        }
        "Json.null" => Ok(json_variant_vm("json_null", None)),
        "Json.bool" => match args.into_iter().next() {
            Some(VMValue::Bool(b)) => Ok(json_variant_vm("json_bool", Some(VMValue::Bool(b)))),
            Some(other) => Err(format!("Json.bool expects Bool, got {}", vmvalue_type_name(&other))),
            None => Err("Json.bool requires 1 argument".to_string()),
        },
        "Json.int" => match args.into_iter().next() {
            Some(VMValue::Int(i)) => Ok(json_variant_vm("json_int", Some(VMValue::Int(i)))),
            Some(other) => Err(format!("Json.int expects Int, got {}", vmvalue_type_name(&other))),
            None => Err("Json.int requires 1 argument".to_string()),
        },
        "Json.float" => match args.into_iter().next() {
            Some(VMValue::Float(f)) => Ok(json_variant_vm("json_float", Some(VMValue::Float(f)))),
            Some(other) => Err(format!("Json.float expects Float, got {}", vmvalue_type_name(&other))),
            None => Err("Json.float requires 1 argument".to_string()),
        },
        "Json.str" => match args.into_iter().next() {
            Some(VMValue::Str(s)) => Ok(json_variant_vm("json_str", Some(VMValue::Str(s)))),
            Some(other) => Err(format!("Json.str expects String, got {}", vmvalue_type_name(&other))),
            None => Err("Json.str requires 1 argument".to_string()),
        },
        "Json.array" => match args.into_iter().next() {
            Some(VMValue::List(items)) => Ok(json_variant_vm("json_array", Some(VMValue::List(items)))),
            Some(other) => Err(format!("Json.array expects List<Json>, got {}", vmvalue_type_name(&other))),
            None => Err("Json.array requires 1 argument".to_string()),
        },
        "Json.object" => match args.into_iter().next() {
            Some(VMValue::List(fields)) => {
                let mut obj = HashMap::new();
                for field in fields {
                    let rec = match field {
                        VMValue::Record(rec) => rec,
                        other => return Err(format!("Json.object expects List<JsonField>, got {}", vmvalue_type_name(&other))),
                    };
                    let key = match rec.get("key") {
                        Some(VMValue::Str(s)) => s.clone(),
                        Some(other) => return Err(format!("JsonField.key must be String, got {}", vmvalue_type_name(other))),
                        None => return Err("JsonField missing `key`".to_string()),
                    };
                    let value = rec.get("value").cloned().ok_or_else(|| "JsonField missing `value`".to_string())?;
                    obj.insert(key, value);
                }
                Ok(json_variant_vm("json_object", Some(VMValue::Record(obj))))
            }
            Some(other) => Err(format!("Json.object expects List<JsonField>, got {}", vmvalue_type_name(&other))),
            None => Err("Json.object requires 1 argument".to_string()),
        },
        "Json.parse" => match args.into_iter().next() {
            Some(VMValue::Str(s)) => match serde_json::from_str::<SerdeJsonValue>(&s) {
                Ok(v) => Ok(VMValue::Variant("some".to_string(), Some(Box::new(serde_to_vm_json(v))))),
                Err(_) => Ok(VMValue::Variant("none".to_string(), None)),
            },
            Some(other) => Err(format!("Json.parse expects String, got {}", vmvalue_type_name(&other))),
            None => Err("Json.parse requires 1 argument".to_string()),
        },
        "Json.encode" | "Json.encode_pretty" => {
            let json = args.into_iter().next().ok_or_else(|| format!("{} requires 1 argument", name))?;
            let serde = vm_json_to_serde(&json).ok_or_else(|| format!("{} expects Json", name))?;
            let out = if name == "Json.encode_pretty" {
                serde_json::to_string_pretty(&serde)
            } else {
                serde_json::to_string(&serde)
            }
            .map_err(|e| format!("{} failed: {}", name, e))?;
            Ok(VMValue::Str(out))
        }
        "Json.get" => {
            if args.len() != 2 {
                return Err("Json.get requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let json = it.next().unwrap();
            let key = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => return Err(format!("Json.get expects String key, got {}", vmvalue_type_name(&other))),
            };
            match json {
                VMValue::Variant(tag, Some(payload)) if tag == "json_object" => match *payload {
                    VMValue::Record(map) => Ok(map.get(&key)
                        .cloned()
                        .map(|v| VMValue::Variant("some".to_string(), Some(Box::new(v))))
                        .unwrap_or(VMValue::Variant("none".to_string(), None))),
                    _ => Err("Json.get received malformed json_object payload".to_string()),
                },
                _ => Ok(VMValue::Variant("none".to_string(), None)),
            }
        }
        "Json.at" => {
            if args.len() != 2 {
                return Err("Json.at requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let json = it.next().unwrap();
            let idx = match it.next().unwrap() {
                VMValue::Int(i) => i,
                other => return Err(format!("Json.at expects Int index, got {}", vmvalue_type_name(&other))),
            };
            match json {
                VMValue::Variant(tag, Some(payload)) if tag == "json_array" => match *payload {
                    VMValue::List(items) if idx >= 0 => Ok(items.get(idx as usize)
                        .cloned()
                        .map(|v| VMValue::Variant("some".to_string(), Some(Box::new(v))))
                        .unwrap_or(VMValue::Variant("none".to_string(), None))),
                    VMValue::List(_) => Ok(VMValue::Variant("none".to_string(), None)),
                    _ => Err("Json.at received malformed json_array payload".to_string()),
                },
                _ => Ok(VMValue::Variant("none".to_string(), None)),
            }
        }
        "Json.as_str" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_str" => Ok(VMValue::Variant("some".to_string(), Some(payload))),
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_str requires 1 argument".to_string()),
        },
        "Json.as_int" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_int" => Ok(VMValue::Variant("some".to_string(), Some(payload))),
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_int requires 1 argument".to_string()),
        },
        "Json.as_float" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_float" => Ok(VMValue::Variant("some".to_string(), Some(payload))),
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_float requires 1 argument".to_string()),
        },
        "Json.as_bool" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_bool" => Ok(VMValue::Variant("some".to_string(), Some(payload))),
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_bool requires 1 argument".to_string()),
        },
        "Json.as_array" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_array" => Ok(VMValue::Variant("some".to_string(), Some(payload))),
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.as_array requires 1 argument".to_string()),
        },
        "Json.is_null" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, None)) if tag == "json_null" => Ok(VMValue::Bool(true)),
            Some(_) => Ok(VMValue::Bool(false)),
            None => Err("Json.is_null requires 1 argument".to_string()),
        },
        "Json.keys" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_object" => match *payload {
                VMValue::Record(map) => {
                    let mut keys: Vec<VMValue> = map.into_keys().map(VMValue::Str).collect();
                    keys.sort_by(|a, b| vmvalue_repr(a).cmp(&vmvalue_repr(b)));
                    Ok(VMValue::Variant("some".to_string(), Some(Box::new(VMValue::List(keys)))))
                }
                _ => Err("Json.keys received malformed json_object payload".to_string()),
            },
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.keys requires 1 argument".to_string()),
        },
        "Json.length" => match args.into_iter().next() {
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_array" => match *payload {
                VMValue::List(items) => Ok(VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(items.len() as i64))))),
                _ => Err("Json.length received malformed json_array payload".to_string()),
            },
            Some(VMValue::Variant(tag, Some(payload))) if tag == "json_object" => match *payload {
                VMValue::Record(map) => Ok(VMValue::Variant("some".to_string(), Some(Box::new(VMValue::Int(map.len() as i64))))),
                _ => Err("Json.length received malformed json_object payload".to_string()),
            },
            Some(_) => Ok(VMValue::Variant("none".to_string(), None)),
            None => Err("Json.length requires 1 argument".to_string()),
        },
        "Csv.parse" => {
            let input = vm_string(
                args.into_iter().next().ok_or_else(|| "Csv.parse requires 1 argument".to_string())?,
                "Csv.parse",
            )?;
            let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(input.as_bytes());
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = record.map_err(|e| format!("Csv.parse failed: {}", e))?;
                rows.push(VMValue::List(record.iter().map(|cell| VMValue::Str(cell.to_string())).collect()));
            }
            Ok(VMValue::List(rows))
        }
        "Csv.parse_with_header" => {
            let input = vm_string(
                args.into_iter().next().ok_or_else(|| "Csv.parse_with_header requires 1 argument".to_string())?,
                "Csv.parse_with_header",
            )?;
            let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(input.as_bytes());
            let headers = rdr.headers().map_err(|e| format!("Csv.parse_with_header failed: {}", e))?.clone();
            let mut rows = Vec::new();
            for record in rdr.records() {
                let record = record.map_err(|e| format!("Csv.parse_with_header failed: {}", e))?;
                let mut row = HashMap::new();
                for (key, value) in headers.iter().zip(record.iter()) {
                    row.insert(key.to_string(), VMValue::Str(value.to_string()));
                }
                rows.push(VMValue::Record(row));
            }
            Ok(VMValue::List(rows))
        }
        "Csv.encode" => {
            let rows = match args.into_iter().next() {
                Some(VMValue::List(rows)) => rows,
                Some(other) => return Err(format!("Csv.encode expects List<List<String>>, got {}", vmvalue_type_name(&other))),
                None => return Err("Csv.encode requires 1 argument".to_string()),
            };
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            for row in rows {
                let fields = vm_string_list(row, "Csv.encode")?;
                writer.write_record(fields).map_err(|e| format!("Csv.encode failed: {}", e))?;
            }
            let bytes = writer.into_inner().map_err(|e| format!("Csv.encode failed: {}", e.into_error()))?;
            let out = String::from_utf8(bytes).map_err(|e| format!("Csv.encode produced invalid UTF-8: {}", e))?;
            Ok(VMValue::Str(out))
        }
        "Csv.encode_with_header" => {
            if args.len() != 2 {
                return Err("Csv.encode_with_header requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let header = vm_string_list(it.next().unwrap(), "Csv.encode_with_header")?;
            let rows = match it.next().unwrap() {
                VMValue::List(rows) => rows,
                other => return Err(format!("Csv.encode_with_header expects List<List<String>>, got {}", vmvalue_type_name(&other))),
            };
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            writer.write_record(&header).map_err(|e| format!("Csv.encode_with_header failed: {}", e))?;
            for row in rows {
                let fields = vm_string_list(row, "Csv.encode_with_header")?;
                writer.write_record(fields).map_err(|e| format!("Csv.encode_with_header failed: {}", e))?;
            }
            let bytes = writer.into_inner().map_err(|e| format!("Csv.encode_with_header failed: {}", e.into_error()))?;
            let out = String::from_utf8(bytes).map_err(|e| format!("Csv.encode_with_header produced invalid UTF-8: {}", e))?;
            Ok(VMValue::Str(out))
        }
        "Csv.from_records" => {
            let records = match args.into_iter().next() {
                Some(VMValue::List(records)) => records,
                Some(other) => return Err(format!("Csv.from_records expects List<Map<String>>, got {}", vmvalue_type_name(&other))),
                None => return Err("Csv.from_records requires 1 argument".to_string()),
            };
            let mut headers = std::collections::BTreeSet::new();
            let mut rows = Vec::new();
            for record in records {
                match record {
                    VMValue::Record(map) => {
                        for key in map.keys() {
                            headers.insert(key.clone());
                        }
                        rows.push(map);
                    }
                    other => return Err(format!("Csv.from_records expects record rows, got {}", vmvalue_type_name(&other))),
                }
            }
            let header: Vec<String> = headers.into_iter().collect();
            let mut writer = csv::WriterBuilder::new().from_writer(vec![]);
            writer.write_record(&header).map_err(|e| format!("Csv.from_records failed: {}", e))?;
            for row in rows {
                let mut values = Vec::with_capacity(header.len());
                for key in &header {
                    let value = row.get(key).cloned().unwrap_or(VMValue::Str(String::new()));
                    values.push(vm_string(value, "Csv.from_records")?);
                }
                writer.write_record(values).map_err(|e| format!("Csv.from_records failed: {}", e))?;
            }
            let bytes = writer.into_inner().map_err(|e| format!("Csv.from_records failed: {}", e.into_error()))?;
            let out = String::from_utf8(bytes).map_err(|e| format!("Csv.from_records produced invalid UTF-8: {}", e))?;
            Ok(VMValue::Str(out))
        }
        "Trace.print" => {
            let v = args.into_iter().next()
                .ok_or_else(|| "Trace.print requires 1 argument".to_string())?;
            let s = match v { VMValue::Str(s) => s, other => vmvalue_repr(&other) };
            eprintln!("[trace] {}", s);
            Ok(VMValue::Unit)
        }
        "Trace.log" => {
            let mut it = args.into_iter();
            let label = it.next().ok_or_else(|| "Trace.log requires 2 arguments".to_string())?;
            let val = it.next().ok_or_else(|| "Trace.log requires 2 arguments".to_string())?;
            let label_s = match label { VMValue::Str(s) => s, other => vmvalue_repr(&other) };
            eprintln!("[trace] {}: {}", label_s, vmvalue_repr(&val));
            Ok(VMValue::Unit)
        }
        "Emit.log" => {
            let log: Vec<VMValue> = emit_log.iter().map(|v| VMValue::Str(vmvalue_repr(v))).collect();
            Ok(VMValue::List(log))
        }
        "Db.execute" => {
            if args.is_empty() {
                return Err("Db.execute requires a SQL string".to_string());
            }
            let mut it = args.into_iter();
            let sql = vm_string(it.next().expect("sql"), "Db.execute")?;
            let params: Vec<VMValue> = it.collect();
            with_db_path(db_path, |conn| {
                let mut stmt = conn.prepare(&sql).map_err(|e| format!("Db error: {}", e))?;
                let bound: Vec<rusqlite::types::Value> = params.iter().map(vmvalue_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> = bound.iter().map(|b| b as &dyn rusqlite::ToSql).collect();
                let rows = stmt.execute(refs.as_slice()).map_err(|e| format!("Db error: {}", e))?;
                Ok(VMValue::Int(rows as i64))
            })
        }
        "Db.query" => {
            if args.is_empty() {
                return Err("Db.query requires a SQL string".to_string());
            }
            let mut it = args.into_iter();
            let sql = vm_string(it.next().expect("sql"), "Db.query")?;
            let params: Vec<VMValue> = it.collect();
            with_db_path(db_path, |conn| {
                let mut stmt = conn.prepare(&sql).map_err(|e| format!("Db error: {}", e))?;
                let bound: Vec<rusqlite::types::Value> = params.iter().map(vmvalue_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> = bound.iter().map(|b| b as &dyn rusqlite::ToSql).collect();
                let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
                let rows = stmt
                    .query_map(refs.as_slice(), |row| {
                        let mut map = HashMap::new();
                        for (i, name) in col_names.iter().enumerate() {
                            let value: rusqlite::types::Value = row.get(i)?;
                            map.insert(name.clone(), VMValue::Str(sqlite_value_to_string(value)));
                        }
                        Ok(VMValue::Record(map))
                    })
                    .map_err(|e| format!("Db error: {}", e))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("Db error: {}", e))?;
                Ok(VMValue::List(rows))
            })
        }
        "Db.query_one" => {
            if args.is_empty() {
                return Err("Db.query_one requires a SQL string".to_string());
            }
            let mut it = args.into_iter();
            let sql = vm_string(it.next().expect("sql"), "Db.query_one")?;
            let params: Vec<VMValue> = it.collect();
            with_db_path(db_path, |conn| {
                let mut stmt = conn.prepare(&sql).map_err(|e| format!("Db error: {}", e))?;
                let bound: Vec<rusqlite::types::Value> = params.iter().map(vmvalue_to_sql).collect();
                let refs: Vec<&dyn rusqlite::ToSql> = bound.iter().map(|b| b as &dyn rusqlite::ToSql).collect();
                let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
                let mut rows = stmt.query(refs.as_slice()).map_err(|e| format!("Db error: {}", e))?;
                match rows.next().map_err(|e| format!("Db error: {}", e))? {
                    None => Ok(VMValue::Variant("none".to_string(), None)),
                    Some(row) => {
                        let mut map = HashMap::new();
                        for (i, name) in col_names.iter().enumerate() {
                            let value: rusqlite::types::Value = row.get(i).map_err(|e| format!("Db error: {}", e))?;
                            map.insert(name.clone(), VMValue::Str(sqlite_value_to_string(value)));
                        }
                        Ok(VMValue::Variant(
                            "some".to_string(),
                            Some(Box::new(VMValue::Record(map))),
                        ))
                    }
                }
            })
        }
        "Http.get" => {
            let url = vm_string(
                args.into_iter().next().ok_or_else(|| "Http.get requires a URL argument".to_string())?,
                "Http.get",
            )?;
            match ureq::get(&url).call() {
                Ok(resp) => {
                    let body = resp.into_string().map_err(|e| format!("Http.get read error: {}", e))?;
                    Ok(VMValue::Variant("ok".to_string(), Some(Box::new(VMValue::Str(body)))))
                }
                Err(e) => Ok(VMValue::Variant("err".to_string(), Some(Box::new(VMValue::Str(e.to_string()))))),
            }
        }
        "Http.post" => {
            if args.len() < 2 {
                return Err("Http.post requires 2 arguments (url, body)".to_string());
            }
            let mut it = args.into_iter();
            let url = vm_string(it.next().expect("url"), "Http.post")?;
            let body = match it.next().expect("body") {
                VMValue::Str(s) => s,
                other => vmvalue_repr(&other),
            };
            match ureq::post(&url).send_string(&body) {
                Ok(resp) => {
                    let body = resp.into_string().map_err(|e| format!("Http.post read error: {}", e))?;
                    Ok(VMValue::Variant("ok".to_string(), Some(Box::new(VMValue::Str(body)))))
                }
                Err(e) => Ok(VMValue::Variant("err".to_string(), Some(Box::new(VMValue::Str(e.to_string()))))),
            }
        }
        "File.read" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => return Err(format!("File.read expects String path, got {}", vmvalue_type_name(&other))),
                None => return Err("File.read requires 1 argument".to_string()),
            };
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("File.read failed for `{}`: {}", path, e))?;
            Ok(VMValue::Str(content))
        }
        "File.read_lines" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => return Err(format!("File.read_lines expects String path, got {}", vmvalue_type_name(&other))),
                None => return Err("File.read_lines requires 1 argument".to_string()),
            };
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("File.read_lines failed for `{}`: {}", path, e))?;
            Ok(VMValue::List(content.lines().map(|line| VMValue::Str(line.to_string())).collect()))
        }
        "File.write" => {
            if args.len() != 2 {
                return Err("File.write requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => return Err(format!("File.write expects String path, got {}", vmvalue_type_name(&other))),
            };
            let content = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => return Err(format!("File.write expects String content, got {}", vmvalue_type_name(&other))),
            };
            std::fs::write(&path, content)
                .map_err(|e| format!("File.write failed for `{}`: {}", path, e))?;
            Ok(VMValue::Unit)
        }
        "File.write_lines" => {
            if args.len() != 2 {
                return Err("File.write_lines requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => return Err(format!("File.write_lines expects String path, got {}", vmvalue_type_name(&other))),
            };
            let lines = match it.next().unwrap() {
                VMValue::List(items) => {
                    let mut parts = Vec::with_capacity(items.len());
                    for item in items {
                        match item {
                            VMValue::Str(s) => parts.push(s),
                            other => return Err(format!("File.write_lines expects List<String>, got List<{}>", vmvalue_type_name(&other))),
                        }
                    }
                    parts
                }
                other => return Err(format!("File.write_lines expects List<String>, got {}", vmvalue_type_name(&other))),
            };
            std::fs::write(&path, lines.join("\n"))
                .map_err(|e| format!("File.write_lines failed for `{}`: {}", path, e))?;
            Ok(VMValue::Unit)
        }
        "File.append" => {
            use std::io::Write;
            if args.len() != 2 {
                return Err("File.append requires 2 arguments".to_string());
            }
            let mut it = args.into_iter();
            let path = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => return Err(format!("File.append expects String path, got {}", vmvalue_type_name(&other))),
            };
            let content = match it.next().unwrap() {
                VMValue::Str(s) => s,
                other => return Err(format!("File.append expects String content, got {}", vmvalue_type_name(&other))),
            };
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| format!("File.append failed for `{}`: {}", path, e))?;
            file.write_all(content.as_bytes())
                .map_err(|e| format!("File.append failed for `{}`: {}", path, e))?;
            Ok(VMValue::Unit)
        }
        "File.exists" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => return Err(format!("File.exists expects String path, got {}", vmvalue_type_name(&other))),
                None => return Err("File.exists requires 1 argument".to_string()),
            };
            Ok(VMValue::Bool(std::path::Path::new(&path).exists()))
        }
        "File.delete" => {
            let path = match args.into_iter().next() {
                Some(VMValue::Str(s)) => s,
                Some(other) => return Err(format!("File.delete expects String path, got {}", vmvalue_type_name(&other))),
                None => return Err("File.delete requires 1 argument".to_string()),
            };
            match std::fs::remove_file(&path) {
                Ok(_) => Ok(VMValue::Unit),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(VMValue::Unit),
                Err(e) => Err(format!("File.delete failed for `{}`: {}", path, e)),
            }
        }
        other => Err(format!("unknown builtin: {}", other)),
    }
}

#[cfg(test)]
#[path = "vm_legacy_coverage_tests.rs"]
mod vm_legacy_coverage_tests;

#[cfg(test)]
#[path = "vm_stdlib_tests.rs"]
mod vm_stdlib_tests;


#[cfg(test)]
mod wasm_phase0_builtin_tests {
    use super::{vm_call_builtin, VMValue};

    #[test]
    fn vm_builtin_io_print_variants_return_unit() {
        let mut emit_log = Vec::new();
        assert_eq!(
            vm_call_builtin("IO.print", vec![VMValue::Str("hello".into())], &mut emit_log, None).unwrap(),
            VMValue::Unit
        );
        assert_eq!(
            vm_call_builtin("IO.println_int", vec![VMValue::Int(42)], &mut emit_log, None).unwrap(),
            VMValue::Unit
        );
        assert_eq!(
            vm_call_builtin("IO.println_float", vec![VMValue::Float(3.5)], &mut emit_log, None).unwrap(),
            VMValue::Unit
        );
        assert_eq!(
            vm_call_builtin("IO.println_bool", vec![VMValue::Bool(true)], &mut emit_log, None).unwrap(),
            VMValue::Unit
        );
    }
}
