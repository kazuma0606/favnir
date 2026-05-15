use super::artifact::{FvcArtifact, FvcFunction, FvcGlobal, FvcWriter};
use crate::ast::{BinOp, Lit};
use crate::middle::ir::{IRArm, IRExpr, IRPattern, IRStmt};
use crate::middle::ir::{IRGlobalKind, IRProgram};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Const = 0x01,
    ConstUnit = 0x02,
    ConstTrue = 0x03,
    ConstFalse = 0x04,
    LoadLocal = 0x10,
    StoreLocal = 0x11,
    LoadGlobal = 0x12,
    Pop = 0x13,
    Dup = 0x14,
    Call = 0x15,
    Return = 0x16,
    Add = 0x20,
    Sub = 0x21,
    Mul = 0x22,
    Div = 0x23,
    Eq = 0x24,
    Ne = 0x25,
    Lt = 0x26,
    Le = 0x27,
    Gt = 0x28,
    Ge = 0x29,
    And = 0x2A,
    Or = 0x2B,
    Jump = 0x30,
    JumpIfFalse = 0x31,
    MatchFail = 0x32,
    ChainCheck = 0x33,
    JumpIfNotVariant = 0x34,
    GetField = 0x40,
    BuildRecord = 0x41,
    MakeClosure = 0x42,
    GetVariantPayload = 0x43,
    CollectBegin = 0x50,
    CollectEnd = 0x51,
    YieldValue = 0x52,
    EmitEvent = 0x53,
    TrackLine = 0x54,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Str(String),
    Name(String),
}

#[derive(Debug, Default, Clone)]
pub struct Codegen {
    pub code: Vec<u8>,
    pub constants: Vec<Constant>,
    pub str_table: Vec<String>,
    /// Positions of ChainCheck operand placeholders that must be patched to RETURN.
    pub chain_escapes: Vec<usize>,
}

impl Codegen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit_u8(&mut self, value: u8) {
        self.code.push(value);
    }

    pub fn emit_u16(&mut self, value: u16) {
        self.code.extend_from_slice(&value.to_le_bytes());
    }

    #[cfg(test)]
    pub fn emit_i16(&mut self, value: i16) {
        self.code.extend_from_slice(&value.to_le_bytes());
    }

    pub fn emit_opcode(&mut self, opcode: Opcode) {
        self.emit_u8(opcode as u8);
    }

    pub fn emit_jump(&mut self, opcode: Opcode) -> usize {
        self.emit_opcode(opcode);
        let pos = self.code.len();
        self.emit_u16(0);
        pos
    }

    pub fn emit_variant_jump(&mut self, name_idx: u16) -> usize {
        self.emit_opcode(Opcode::JumpIfNotVariant);
        self.emit_u16(name_idx);
        let pos = self.code.len();
        self.emit_u16(0);
        pos
    }

    pub fn patch_jump(&mut self, pos: usize) {
        let jump_target = self.code.len();
        let offset_pos = pos;
        let after_jump = offset_pos + 2;
        let offset = jump_target
            .checked_sub(after_jump)
            .expect("jump target before placeholder");
        let offset = u16::try_from(offset).expect("jump offset overflow");
        let [lo, hi] = offset.to_le_bytes();
        self.code[offset_pos] = lo;
        self.code[offset_pos + 1] = hi;
    }

    pub fn const_idx(&mut self, constant: Constant) -> u16 {
        if let Some(idx) = self.constants.iter().position(|c| c == &constant) {
            return idx as u16;
        }
        let idx = self.constants.len() as u16;
        self.constants.push(constant);
        idx
    }

    pub fn intern_str(&mut self, value: &str) -> u16 {
        if let Some(idx) = self.str_table.iter().position(|s| s == value) {
            return idx as u16;
        }
        let idx = self.str_table.len() as u16;
        self.str_table.push(value.to_string());
        idx
    }
}

pub fn emit_expr(expr: &IRExpr, cg: &mut Codegen) {
    match expr {
        IRExpr::Lit(lit, _) => match lit {
            Lit::Int(v) => {
                let idx = cg.const_idx(Constant::Int(*v));
                cg.emit_opcode(Opcode::Const);
                cg.emit_u16(idx);
            }
            Lit::Float(v) => {
                let idx = cg.const_idx(Constant::Float(*v));
                cg.emit_opcode(Opcode::Const);
                cg.emit_u16(idx);
            }
            Lit::Str(v) => {
                let idx = cg.const_idx(Constant::Str(v.clone()));
                cg.emit_opcode(Opcode::Const);
                cg.emit_u16(idx);
            }
            Lit::Bool(true) => cg.emit_opcode(Opcode::ConstTrue),
            Lit::Bool(false) => cg.emit_opcode(Opcode::ConstFalse),
            Lit::Unit => cg.emit_opcode(Opcode::ConstUnit),
        },
        IRExpr::Local(slot, _) => {
            cg.emit_opcode(Opcode::LoadLocal);
            cg.emit_u16(*slot);
        }
        IRExpr::Global(idx, _) => {
            cg.emit_opcode(Opcode::LoadGlobal);
            cg.emit_u16(*idx);
        }
        IRExpr::TrfRef(idx, _) => {
            cg.emit_opcode(Opcode::LoadGlobal);
            cg.emit_u16(*idx);
        }
        IRExpr::CallTrfLocal { local, arg, .. } => {
            cg.emit_opcode(Opcode::LoadLocal);
            cg.emit_u16(*local);
            emit_expr(arg, cg);
            cg.emit_opcode(Opcode::Call);
            cg.emit_u16(1);
        }
        IRExpr::Call(callee, args, _) => {
            emit_expr(callee, cg);
            for arg in args {
                emit_expr(arg, cg);
            }
            cg.emit_opcode(Opcode::Call);
            cg.emit_u16(args.len() as u16);
        }
        IRExpr::Block(stmts, tail, _) => {
            for stmt in stmts {
                emit_stmt(stmt, cg);
            }
            emit_expr(tail, cg);
        }
        IRExpr::If(cond, then_expr, else_expr, _) => {
            emit_expr(cond, cg);
            let false_jump = cg.emit_jump(Opcode::JumpIfFalse);
            emit_expr(then_expr, cg);
            let end_jump = cg.emit_jump(Opcode::Jump);
            cg.patch_jump(false_jump);
            emit_expr(else_expr, cg);
            cg.patch_jump(end_jump);
        }
        IRExpr::Match(scrutinee, arms, _) => emit_match(scrutinee, arms, cg),
        IRExpr::FieldAccess(obj, field, _) => {
            emit_expr(obj, cg);
            let idx = cg.intern_str(field);
            cg.emit_opcode(Opcode::GetField);
            cg.emit_u16(idx);
        }
        IRExpr::BinOp(op, left, right, _) => {
            emit_expr(left, cg);
            emit_expr(right, cg);
            cg.emit_opcode(match op {
                BinOp::Add => Opcode::Add,
                BinOp::Sub => Opcode::Sub,
                BinOp::Mul => Opcode::Mul,
                BinOp::Div => Opcode::Div,
                BinOp::Eq => Opcode::Eq,
                BinOp::NotEq => Opcode::Ne,
                BinOp::Lt => Opcode::Lt,
                BinOp::LtEq => Opcode::Le,
                BinOp::Gt => Opcode::Gt,
                BinOp::GtEq => Opcode::Ge,
                BinOp::And => Opcode::And,
                BinOp::Or => Opcode::Or,
                // NullCoalesce is desugared in the checker; should not appear in IR BinOp
                BinOp::NullCoalesce => unreachable!("?? desugared before codegen"),
            });
        }
        IRExpr::Closure(global_idx, captures, _) => {
            for capture in captures {
                emit_expr(capture, cg);
            }
            cg.emit_opcode(Opcode::MakeClosure);
            cg.emit_u16(*global_idx);
            cg.emit_u16(captures.len() as u16);
        }
        IRExpr::Collect(body, _) => {
            cg.emit_opcode(Opcode::CollectBegin);
            emit_expr(body, cg);
            cg.emit_opcode(Opcode::CollectEnd);
        }
        IRExpr::Emit(val, _) => {
            emit_expr(val, cg);
            cg.emit_opcode(Opcode::EmitEvent);
        }
        IRExpr::RecordConstruct(fields, _) => {
            let mut names = Vec::with_capacity(fields.len());
            for (name, value) in fields {
                emit_expr(value, cg);
                names.push(name.as_str());
            }
            let joined = names.join("\u{1f}");
            let names_idx = cg.intern_str(&joined);
            cg.emit_opcode(Opcode::BuildRecord);
            cg.emit_u16(fields.len() as u16);
            cg.emit_u16(names_idx);
        }
    }
}

pub fn emit_stmt(stmt: &IRStmt, cg: &mut Codegen) {
    match stmt {
        IRStmt::Bind(slot, expr) => {
            emit_expr(expr, cg);
            cg.emit_opcode(Opcode::StoreLocal);
            cg.emit_u16(*slot);
        }
        IRStmt::Chain(slot, expr) => {
            emit_expr(expr, cg);
            let escape = cg.emit_jump(Opcode::ChainCheck);
            cg.emit_opcode(Opcode::StoreLocal);
            cg.emit_u16(*slot);
            // Defer patching: escape must jump to the function's RETURN,
            // which isn't known yet. Collect for later patching.
            cg.chain_escapes.push(escape);
        }
        IRStmt::Yield(expr) => {
            emit_expr(expr, cg);
            cg.emit_opcode(Opcode::YieldValue);
        }
        IRStmt::Expr(expr) => {
            emit_expr(expr, cg);
            cg.emit_opcode(Opcode::Pop);
        }
        IRStmt::TrackLine(line) => {
            cg.emit_opcode(Opcode::TrackLine);
            // Emit line as u32 (4 bytes LE)
            cg.code.extend_from_slice(&line.to_le_bytes());
        }
    }
}

fn emit_match(scrutinee: &IRExpr, arms: &[IRArm], cg: &mut Codegen) {
    emit_expr(scrutinee, cg);
    let mut end_jumps = Vec::new();

    for arm in arms {
        cg.emit_opcode(Opcode::Dup);
        let mut fail_jumps = Vec::new();
        emit_pattern_test(&arm.pattern, &mut fail_jumps, cg);

        if let Some(guard) = &arm.guard {
            emit_expr(guard, cg);
            fail_jumps.push(cg.emit_jump(Opcode::JumpIfFalse));
        }

        emit_expr(&arm.body, cg);
        end_jumps.push(cg.emit_jump(Opcode::Jump));

        for jump in fail_jumps {
            cg.patch_jump(jump);
        }
    }

    cg.emit_opcode(Opcode::MatchFail);
    for jump in end_jumps {
        cg.patch_jump(jump);
    }
}

fn emit_pattern_test(pattern: &IRPattern, fail_jumps: &mut Vec<usize>, cg: &mut Codegen) {
    match pattern {
        IRPattern::Wildcard => {}
        IRPattern::Lit(lit) => {
            cg.emit_opcode(Opcode::Dup);
            emit_expr(
                &IRExpr::Lit(lit.clone(), crate::middle::checker::Type::Unknown),
                cg,
            );
            cg.emit_opcode(Opcode::Eq);
            fail_jumps.push(cg.emit_jump(Opcode::JumpIfFalse));
        }
        IRPattern::Bind(slot) => {
            cg.emit_opcode(Opcode::Dup);
            cg.emit_opcode(Opcode::StoreLocal);
            cg.emit_u16(*slot);
        }
        IRPattern::Variant(name, inner) => {
            let idx = cg.intern_str(name);
            cg.emit_opcode(Opcode::Dup);
            fail_jumps.push(cg.emit_variant_jump(idx));
            if let Some(inner) = inner {
                cg.emit_opcode(Opcode::GetVariantPayload);
                emit_pattern_test(inner, fail_jumps, cg);
            }
        }
        IRPattern::Record(fields) => {
            for (name, inner) in fields {
                let idx = cg.intern_str(name);
                cg.emit_opcode(Opcode::Dup);
                cg.emit_opcode(Opcode::GetField);
                cg.emit_u16(idx);
                emit_pattern_test(inner, fail_jumps, cg);
                cg.emit_opcode(Opcode::Pop);
            }
        }
    }
}

pub fn codegen_program(ir: &IRProgram) -> FvcArtifact {
    let mut writer = FvcWriter::new();

    for global in &ir.globals {
        writer.intern(&global.name);
    }

    for f in &ir.fns {
        writer.intern(&f.name);
        writer.intern(&format!("{:?}", f.return_ty));
        writer.intern(&format!("{:?}", f.effects));
    }

    for global in &ir.globals {
        let name_idx = writer.intern(&global.name);
        let (kind, fn_idx) = match global.kind {
            IRGlobalKind::Fn(idx) => (0u8, idx as u32),
            IRGlobalKind::Builtin => (1u8, u32::MAX),
            IRGlobalKind::VariantCtor => (2u8, u32::MAX),
        };
        writer.add_global(FvcGlobal {
            name_idx,
            kind,
            fn_idx,
        });
    }

    for f in &ir.fns {
        let mut cg = Codegen::new();
        emit_expr(&f.body, &mut cg);
        // Patch all chain escapes to jump to the upcoming RETURN instruction.
        for escape in cg.chain_escapes.clone() {
            cg.patch_jump(escape);
        }
        cg.emit_opcode(Opcode::Return);
        let str_remap: Vec<u16> = cg
            .str_table
            .iter()
            .map(|value| {
                let idx = writer.intern(value);
                u16::try_from(idx).expect("artifact string table overflow")
            })
            .collect();
        remap_string_operands(&mut cg.code, &str_remap);

        let name_idx = writer.intern(&f.name);
        let return_ty_str_idx = writer.intern(&format!("{:?}", f.return_ty));
        let effect_str_idx = writer.intern(&format!("{:?}", f.effects));
        writer.add_function(FvcFunction {
            name_idx,
            param_count: f.param_count as u32,
            local_count: f.local_count as u32,
            source_line: 0,
            return_ty_str_idx,
            effect_str_idx,
            constants: cg.constants,
            code: cg.code,
        });
    }

    FvcArtifact {
        str_table: writer.str_table,
        globals: writer.globals,
        functions: writer.functions,
        type_metas: ir.type_metas.clone(),
        explain_json: None,
    }
}

fn remap_string_operands(code: &mut [u8], str_remap: &[u16]) {
    let mut ip = 0usize;
    while ip < code.len() {
        match code[ip] {
            x if x == Opcode::Const as u8
                || x == Opcode::LoadLocal as u8
                || x == Opcode::StoreLocal as u8
                || x == Opcode::LoadGlobal as u8
                || x == Opcode::Call as u8
                || x == Opcode::Jump as u8
                || x == Opcode::JumpIfFalse as u8
                || x == Opcode::ChainCheck as u8 =>
            {
                ip += 3;
            }
            x if x == Opcode::JumpIfNotVariant as u8 => {
                remap_u16_at(code, ip + 1, str_remap);
                ip += 5;
            }
            x if x == Opcode::GetField as u8 => {
                remap_u16_at(code, ip + 1, str_remap);
                ip += 3;
            }
            x if x == Opcode::BuildRecord as u8 => {
                remap_u16_at(code, ip + 3, str_remap);
                ip += 5;
            }
            x if x == Opcode::ConstUnit as u8
                || x == Opcode::ConstTrue as u8
                || x == Opcode::ConstFalse as u8
                || x == Opcode::Pop as u8
                || x == Opcode::Dup as u8
                || x == Opcode::Return as u8
                || x == Opcode::Add as u8
                || x == Opcode::Sub as u8
                || x == Opcode::Mul as u8
                || x == Opcode::Div as u8
                || x == Opcode::Eq as u8
                || x == Opcode::Ne as u8
                || x == Opcode::Lt as u8
                || x == Opcode::Le as u8
                || x == Opcode::Gt as u8
                || x == Opcode::Ge as u8
                || x == Opcode::And as u8
                || x == Opcode::Or as u8
                || x == Opcode::MatchFail as u8
                || x == Opcode::GetVariantPayload as u8
                || x == Opcode::CollectBegin as u8
                || x == Opcode::CollectEnd as u8
                || x == Opcode::YieldValue as u8
                || x == Opcode::EmitEvent as u8 =>
            {
                ip += 1;
            }
            x if x == Opcode::MakeClosure as u8 => {
                ip += 5;
            }
            _ => break,
        }
    }
}

fn remap_u16_at(code: &mut [u8], offset: usize, str_remap: &[u16]) {
    let old = u16::from_le_bytes([code[offset], code[offset + 1]]);
    let new = str_remap
        .get(old as usize)
        .copied()
        .unwrap_or_else(|| panic!("missing string remap for local string index {old}"));
    let [lo, hi] = new.to_le_bytes();
    code[offset] = lo;
    code[offset + 1] = hi;
}

#[cfg(test)]
mod tests {
    use super::{Codegen, Constant, Opcode, codegen_program, emit_expr, emit_stmt};
    use crate::ast::Lit;
    use crate::middle::checker::Type;
    use crate::middle::ir::{IRExpr, IRFnDef, IRGlobal, IRGlobalKind, IRProgram, IRStmt};

    #[test]
    fn const_idx_deduplicates_constants() {
        let mut cg = Codegen::new();
        let a = cg.const_idx(Constant::Int(42));
        let b = cg.const_idx(Constant::Int(42));
        let c = cg.const_idx(Constant::Str("fav".into()));

        assert_eq!(a, 0);
        assert_eq!(b, 0);
        assert_eq!(c, 1);
        assert_eq!(cg.constants.len(), 2);
    }

    #[test]
    fn intern_str_deduplicates_entries() {
        let mut cg = Codegen::new();
        let a = cg.intern_str("main");
        let b = cg.intern_str("main");
        let c = cg.intern_str("emit");

        assert_eq!(a, 0);
        assert_eq!(b, 0);
        assert_eq!(c, 1);
        assert_eq!(cg.str_table, vec!["main".to_string(), "emit".to_string()]);
    }

    #[test]
    fn emit_helpers_write_little_endian_bytes() {
        let mut cg = Codegen::new();
        cg.emit_opcode(Opcode::Const);
        cg.emit_u16(0x1234);
        cg.emit_i16(-2);

        assert_eq!(cg.code, vec![Opcode::Const as u8, 0x34, 0x12, 0xFE, 0xFF]);
    }

    #[test]
    fn emit_jump_and_patch_jump_write_forward_offset() {
        let mut cg = Codegen::new();
        let patch = cg.emit_jump(Opcode::JumpIfFalse);
        cg.emit_opcode(Opcode::ConstUnit);
        cg.patch_jump(patch);

        assert_eq!(
            cg.code,
            vec![
                Opcode::JumpIfFalse as u8,
                0x01,
                0x00,
                Opcode::ConstUnit as u8,
            ]
        );
    }

    #[test]
    fn emit_variant_jump_writes_name_and_patchable_offset() {
        let mut cg = Codegen::new();
        let patch = cg.emit_variant_jump(7);
        cg.emit_opcode(Opcode::ConstUnit);
        cg.patch_jump(patch);

        assert_eq!(
            cg.code,
            vec![
                Opcode::JumpIfNotVariant as u8,
                0x07,
                0x00,
                0x01,
                0x00,
                Opcode::ConstUnit as u8,
            ]
        );
    }

    #[test]
    fn emit_expr_for_call_writes_callee_args_and_call_count() {
        let mut cg = Codegen::new();
        let expr = IRExpr::Call(
            Box::new(IRExpr::Global(2, Type::Unknown)),
            vec![IRExpr::Lit(Lit::Int(7), Type::Int)],
            Type::Unknown,
        );

        emit_expr(&expr, &mut cg);

        assert_eq!(
            cg.code,
            vec![
                Opcode::LoadGlobal as u8,
                0x02,
                0x00,
                Opcode::Const as u8,
                0x00,
                0x00,
                Opcode::Call as u8,
                0x01,
                0x00,
            ]
        );
        assert_eq!(cg.constants, vec![Constant::Int(7)]);
    }

    #[test]
    fn emit_expr_for_call_trf_local_writes_local_arg_and_call_count() {
        let mut cg = Codegen::new();
        let expr = IRExpr::CallTrfLocal {
            local: 3,
            arg: Box::new(IRExpr::Lit(Lit::Int(7), Type::Int)),
            ty: Type::Unknown,
        };

        emit_expr(&expr, &mut cg);

        assert_eq!(
            cg.code,
            vec![
                Opcode::LoadLocal as u8,
                0x03,
                0x00,
                Opcode::Const as u8,
                0x00,
                0x00,
                Opcode::Call as u8,
                0x01,
                0x00,
            ]
        );
        assert_eq!(cg.constants, vec![Constant::Int(7)]);
    }

    #[test]
    fn emit_stmt_bind_stores_into_local_slot() {
        let mut cg = Codegen::new();
        let stmt = IRStmt::Bind(3, IRExpr::Lit(Lit::Bool(true), Type::Bool));

        emit_stmt(&stmt, &mut cg);

        assert_eq!(
            cg.code,
            vec![
                Opcode::ConstTrue as u8,
                Opcode::StoreLocal as u8,
                0x03,
                0x00,
            ]
        );
    }

    #[test]
    fn codegen_program_emits_artifact_sections() {
        let ir = IRProgram {
            globals: vec![IRGlobal {
                name: "main".to_string(),
                kind: IRGlobalKind::Fn(0),
            }],
            fns: vec![IRFnDef {
                name: "main".to_string(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: Vec::new(),
                return_ty: Type::Unit,
                body: IRExpr::Lit(Lit::Unit, Type::Unit),
            }],
            type_metas: std::collections::HashMap::new(),
        };

        let artifact = codegen_program(&ir);

        assert_eq!(artifact.globals.len(), 1);
        assert_eq!(artifact.functions.len(), 1);
        assert!(artifact.fn_idx_by_name("main").is_some());
        assert_eq!(
            artifact.functions[0].code,
            vec![Opcode::ConstUnit as u8, Opcode::Return as u8]
        );
    }

    #[test]
    fn codegen_program_remaps_record_string_operands_into_artifact_table() {
        let ir = IRProgram {
            globals: vec![IRGlobal {
                name: "main".to_string(),
                kind: IRGlobalKind::Fn(0),
            }],
            fns: vec![IRFnDef {
                name: "main".to_string(),
                param_count: 0,
                param_tys: vec![],
                local_count: 0,
                effects: Vec::new(),
                return_ty: Type::String,
                body: IRExpr::FieldAccess(
                    Box::new(IRExpr::RecordConstruct(
                        vec![(
                            "name".to_string(),
                            IRExpr::Lit(Lit::Str("A".into()), Type::String),
                        )],
                        Type::Unknown,
                    )),
                    "name".to_string(),
                    Type::String,
                ),
            }],
            type_metas: std::collections::HashMap::new(),
        };

        let artifact = codegen_program(&ir);
        let code = &artifact.functions[0].code;
        let names_idx = u16::from_le_bytes([code[6], code[7]]) as usize;
        let field_idx = u16::from_le_bytes([code[9], code[10]]) as usize;

        assert_eq!(artifact.str_table[names_idx], "name");
        assert_eq!(artifact.str_table[field_idx], "name");
    }

    #[test]
    fn codegen_program_remaps_variant_string_operands_into_artifact_table() {
        let ir = IRProgram {
            globals: vec![IRGlobal {
                name: "main".to_string(),
                kind: IRGlobalKind::Fn(0),
            }],
            fns: vec![IRFnDef {
                name: "main".to_string(),
                param_count: 1,
                param_tys: vec![Type::Unknown],
                local_count: 1,
                effects: Vec::new(),
                return_ty: Type::Int,
                body: IRExpr::Match(
                    Box::new(IRExpr::Local(0, Type::Unknown)),
                    vec![crate::middle::ir::IRArm {
                        pattern: crate::middle::ir::IRPattern::Variant(
                            "ok".to_string(),
                            Some(Box::new(crate::middle::ir::IRPattern::Bind(0))),
                        ),
                        guard: None,
                        body: IRExpr::Local(0, Type::Int),
                    }],
                    Type::Int,
                ),
            }],
            type_metas: std::collections::HashMap::new(),
        };

        let artifact = codegen_program(&ir);
        let code = &artifact.functions[0].code;
        let opcode_pos = code
            .iter()
            .position(|b| *b == Opcode::JumpIfNotVariant as u8)
            .expect("variant jump opcode");
        let name_idx = u16::from_le_bytes([code[opcode_pos + 1], code[opcode_pos + 2]]) as usize;
        assert_eq!(artifact.str_table[name_idx], "ok");
    }
}
