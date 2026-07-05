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
    /// Like ChainCheck but passes through non-Result values unchanged.
    /// Used by `bind` in `--legacy` mode (v12.3.0).
    LegacyBindCheck = 0x35,
    /// seq pipeline fail-fast: unwrap Ok, short-circuit on Err with stage context (v12.4.0).
    /// Layout: opcode(1) + name_str_idx(2) + stage_idx(1) + total(1) + escape_offset(2)
    SeqStageCheck = 0x36,
    /// seq stage enter trace for --verbose (v12.5.0).
    /// Layout: opcode(1) + name_str_idx(2) = 3 bytes total
    SeqStageEnter = 0x5B,
    GetField = 0x40,
    BuildRecord = 0x41,
    MakeClosure = 0x42,
    GetVariantPayload = 0x43,
    CollectBegin = 0x50,
    CollectEnd = 0x51,
    YieldValue = 0x52,
    EmitEvent = 0x53,
    TrackLine = 0x54,
    /// Swap top two stack items.  Used by emit_match to remove the scrutinee
    /// after the arm body has been evaluated.
    Swap = 0x55,
    /// Call a function by name stored in the per-function constants pool.
    /// Operands: name_const_idx (u16 LE) + argc (u16 LE).
    /// Used by the Favnir self-hosted compiler output.
    CallNamed = 0x56,
    /// Jump-if-not-variant using per-function constant pool.
    /// Operands: const_idx (u16 LE) + offset (u16 LE).
    /// Pops top of stack; if its variant tag matches constants[const_idx] pushes back and
    /// continues; otherwise pushes back and jumps forward by offset bytes.
    /// Used by the Favnir self-hosted compiler's match codegen.
    JumpIfNotVariantC = 0x57,
    /// Like GetField but reads field name from per-function constants pool (CVName).
    GetFieldC = 0x58,
    /// Like BuildRecord but reads N field names from constants[base_idx..base_idx+N] (CVName entries).
    BuildRecordC = 0x59,
    /// Like MakeClosure but looks up function by name (CVName) in artifact globals.
    MakeClosureN = 0x5A,
    /// Merge base record with N override fields.
    /// Layout: opcode(1) + n_overrides(2) + names_idx(2) = 5 bytes
    /// Stack (bottom->top): base_record, val_0, ..., val_{n-1}
    /// str_table[names_idx]: override field names joined by 
    MergeRecord = 0x5C,
    /// Pops a list, pushes its length as Int. (v17.2.0)
    ListLen = 0x60,
    /// Pops (list, index: Int), pushes element at that index. (v17.2.0)
    ListGet = 0x61,
    /// Pops (list, n: Int), pushes list with first n elements dropped. (v17.2.0)
    ListDrop = 0x62,
    /// Refinement assertion (v18.3.0).
    /// Layout: opcode(1) + name_str_idx(2) = 3 bytes.
    /// Pops a bool from the stack; if false, panics with a refinement error message.
    /// The name_str_idx points to the param name in str_table.
    RefinementAssert = 0x63,
    // ── Superinstructions (v20.2.0) — IR-level fused opcodes ────────────────
    /// Layout: opcode(1) + slot_a(u16) + slot_b(u16) = 5 bytes
    /// stack[base+a] + stack[base+b] → push
    AddLL = 0xA0,
    /// stack[base+a] - stack[base+b] → push
    SubLL = 0xA1,
    /// stack[base+a] * stack[base+b] → push
    MulLL = 0xA2,
    /// stack[base+a] + constants[k] → push (k = Constant::Int)
    AddLC = 0xA3,
    /// stack[base+a] - constants[k] → push
    SubLC = 0xA4,
    /// stack[base+a] <= constants[k] → push Bool
    LeLC = 0xA5,
    /// stack[base+a] < constants[k] → push Bool
    LtLC = 0xA6,
    /// stack[base+a] == constants[k] → push Bool
    EqLC = 0xA7,
    /// stack[base+a].field[str_table[f]] → push (str remap applies to operand_b)
    GetFieldL = 0xA8,
    /// stack[base+src] → stack[base+dst] (copy, no stack push/pop)
    MoveLocal = 0xA9,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

    /// Emit a SeqStageCheck opcode with stage metadata.
    /// Layout: SeqStageCheck(1) + name_str_idx(2) + stage_idx(1) + total(1) + escape_offset(2)
    /// Returns the position of the escape_offset placeholder for later patching.
    pub fn emit_seq_stage_jump(&mut self, name_str_idx: u16, stage_idx: u8, total: u8) -> usize {
        self.emit_opcode(Opcode::SeqStageCheck);
        self.emit_u16(name_str_idx);
        self.emit_u8(stage_idx);
        self.emit_u8(total);
        let pos = self.code.len();
        self.emit_u16(0); // escape_offset placeholder
        pos
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
            if let IRExpr::Local(a, _) = obj.as_ref() {
                let f_idx = cg.intern_str(field);
                cg.emit_opcode(Opcode::GetFieldL);
                cg.emit_u16(*a);
                cg.emit_u16(f_idx);
            } else {
                emit_expr(obj, cg);
                let idx = cg.intern_str(field);
                cg.emit_opcode(Opcode::GetField);
                cg.emit_u16(idx);
            }
        }
        IRExpr::BinOp(op, left, right, _) => {
            // Superinstruction fusion: Local(a) op Local(b)
            if let (IRExpr::Local(a, _), IRExpr::Local(b, _)) = (left.as_ref(), right.as_ref()) {
                let super_op = match op {
                    BinOp::Add => Some(Opcode::AddLL),
                    BinOp::Sub => Some(Opcode::SubLL),
                    BinOp::Mul => Some(Opcode::MulLL),
                    _ => None,
                };
                if let Some(sop) = super_op {
                    cg.emit_opcode(sop);
                    cg.emit_u16(*a);
                    cg.emit_u16(*b);
                    return;
                }
            }
            // Superinstruction fusion: Local(a) op Lit(Int(k))
            if let (IRExpr::Local(a, _), IRExpr::Lit(Lit::Int(k), _)) =
                (left.as_ref(), right.as_ref())
            {
                let super_op = match op {
                    BinOp::Add => Some(Opcode::AddLC),
                    BinOp::Sub => Some(Opcode::SubLC),
                    BinOp::LtEq => Some(Opcode::LeLC),
                    BinOp::Lt => Some(Opcode::LtLC),
                    BinOp::Eq => Some(Opcode::EqLC),
                    _ => None,
                };
                if let Some(sop) = super_op {
                    let k_idx = cg.const_idx(Constant::Int(*k));
                    cg.emit_opcode(sop);
                    cg.emit_u16(*a);
                    cg.emit_u16(k_idx);
                    return;
                }
            }
            // fallback: generic codegen
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
        IRExpr::RecordSpread(base, updates, _) => {
            // 1. emit base record
            emit_expr(base, cg);
            // 2. emit override values (left to right)
            let mut names: Vec<&str> = Vec::with_capacity(updates.len());
            for (name, value) in updates.iter() {
                emit_expr(value, cg);
                names.push(name.as_str());
            }
            // 3. store override field names in str_table (joined by \x1f)
            let sep = "\u{1f}";
            let joined = names.join(sep);
            let names_idx = cg.intern_str(&joined);
            // 4. emit MergeRecord opcode
            cg.emit_opcode(Opcode::MergeRecord);
            cg.emit_u16(updates.len() as u16);
            cg.emit_u16(names_idx);
        }
    }
}

pub fn emit_stmt(stmt: &IRStmt, cg: &mut Codegen) {
    match stmt {
        IRStmt::Bind(slot, expr) => {
            if let IRExpr::Local(src, _) = expr {
                cg.emit_opcode(Opcode::MoveLocal);
                cg.emit_u16(*src);
                cg.emit_u16(*slot);
            } else {
                emit_expr(expr, cg);
                cg.emit_opcode(Opcode::StoreLocal);
                cg.emit_u16(*slot);
            }
        }
        IRStmt::LegacyBind(slot, expr) => {
            emit_expr(expr, cg);
            let escape = cg.emit_jump(Opcode::LegacyBindCheck);
            cg.emit_opcode(Opcode::StoreLocal);
            cg.emit_u16(*slot);
            cg.chain_escapes.push(escape);
        }
        IRStmt::SeqChain { slot, expr, stage_name, stage_idx, total } => {
            let name_str_idx = cg.intern_str(stage_name);
            // Emit stage enter trace opcode before calling the stage (v12.5.0)
            cg.emit_opcode(Opcode::SeqStageEnter);
            cg.emit_u16(name_str_idx);
            emit_expr(expr, cg);
            let escape = cg.emit_seq_stage_jump(name_str_idx, *stage_idx, *total);
            cg.emit_opcode(Opcode::StoreLocal);
            cg.emit_u16(*slot);
            cg.chain_escapes.push(escape);
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
        IRStmt::RefinementAssert { param, expr } => {
            emit_expr(expr, cg);
            cg.emit_opcode(Opcode::RefinementAssert);
            let str_idx = cg.intern_str(param);
            cg.emit_u16(str_idx);
        }
    }
}

fn emit_match(scrutinee: &IRExpr, arms: &[IRArm], cg: &mut Codegen) {
    emit_expr(scrutinee, cg);
    // Stack: [..., scrutinee]
    let mut end_jumps = Vec::new();

    for arm in arms {
        cg.emit_opcode(Opcode::Dup);
        // Stack: [..., scrutinee, arm_copy]
        // fail_jumps: (jump_instruction_pos, stack_depth_above_scrutinee_at_fail)
        // depth=1 because arm_copy is on the stack above the scrutinee.
        let mut fail_jumps: Vec<(usize, usize)> = Vec::new();
        let depth_after = emit_pattern_test(&arm.pattern, &mut fail_jumps, cg, 1);

        // SUCCESS PATH: pop all pattern-test items (arm_copy + test copies + payloads)
        // so the stack is clean — only scrutinee remains — before evaluating the body.
        // Any Bind patterns have already stored their values in local slots via StoreLocal.
        for _ in 0..depth_after {
            cg.emit_opcode(Opcode::Pop);
        }
        // Stack: [..., scrutinee]

        if let Some(guard) = &arm.guard {
            emit_expr(guard, cg);
            // Stack: [..., scrutinee, guard_bool]
            // JumpIfFalse pops the bool; on fail the stack is [..., scrutinee] (excess=0).
            fail_jumps.push((cg.emit_jump(Opcode::JumpIfFalse), 0));
        }

        emit_expr(&arm.body, cg);
        // Stack: [..., scrutinee, body_result]
        // Remove the scrutinee cleanly via Swap+Pop so the match expression produces
        // exactly one value at a consistent stack depth.
        cg.emit_opcode(Opcode::Swap);
        // Stack: [..., body_result, scrutinee]
        cg.emit_opcode(Opcode::Pop);
        // Stack: [..., body_result]
        end_jumps.push(cg.emit_jump(Opcode::Jump));

        // FAIL PATH: for each fail_jump, emit a cleanup block that pops `excess` items
        // to restore the scrutinee to the stack top, then jumps to the next arm.
        let mut cleanup_end_jumps: Vec<usize> = Vec::new();
        for (fail_jump, excess) in &fail_jumps {
            cg.patch_jump(*fail_jump);
            for _ in 0..*excess {
                cg.emit_opcode(Opcode::Pop);
            }
            // Stack: [..., scrutinee]
            cleanup_end_jumps.push(cg.emit_jump(Opcode::Jump));
        }
        // All cleanup jumps land here = start of the next arm (or MatchFail).
        for j in cleanup_end_jumps {
            cg.patch_jump(j);
        }
    }

    cg.emit_opcode(Opcode::MatchFail);
    for jump in end_jumps {
        cg.patch_jump(jump);
    }
    // After end_jumps: stack = [..., body_result].
    // The scrutinee was already removed by Swap+Pop in the winning arm.
}

fn emit_pattern_test(
    pattern: &IRPattern,
    fail_jumps: &mut Vec<(usize, usize)>,
    cg: &mut Codegen,
    depth: usize,
) -> usize {
    match pattern {
        IRPattern::Wildcard => depth,
        IRPattern::Lit(lit) => {
            cg.emit_opcode(Opcode::Dup);
            emit_expr(
                &IRExpr::Lit(lit.clone(), crate::middle::checker::Type::Unknown),
                cg,
            );
            cg.emit_opcode(Opcode::Eq);
            // JumpIfFalse pops the bool; stack depth restored to `depth` on fail
            fail_jumps.push((cg.emit_jump(Opcode::JumpIfFalse), depth));
            depth
        }
        IRPattern::Bind(slot) => {
            cg.emit_opcode(Opcode::Dup);
            cg.emit_opcode(Opcode::StoreLocal);
            cg.emit_u16(*slot);
            depth
        }
        IRPattern::Variant(name, inner) => {
            let idx = cg.intern_str(name);
            cg.emit_opcode(Opcode::Dup);
            // On fail: the Dup'd copy stays on the stack → depth+1
            fail_jumps.push((cg.emit_variant_jump(idx), depth + 1));
            if let Some(inner) = inner {
                cg.emit_opcode(Opcode::GetVariantPayload);
                // GetVariantPayload replaces the Dup'd copy with payload; net depth unchanged
                emit_pattern_test(inner, fail_jumps, cg, depth + 1)
            } else {
                depth + 1
            }
        }
        IRPattern::Record(fields) => {
            let mut d = depth;
            for (name, inner) in fields {
                let idx = cg.intern_str(name);
                cg.emit_opcode(Opcode::Dup);
                cg.emit_opcode(Opcode::GetField);
                cg.emit_u16(idx);
                // Dup+GetField: net +1 (record copy consumed, field value pushed)
                d = emit_pattern_test(inner, fail_jumps, cg, d + 1);
                cg.emit_opcode(Opcode::Pop);
                d -= 1;
            }
            d
        }
        // ── or-pattern (v17.2.0) ──────────────────────────────────────────────
        IRPattern::Or(pats) => {
            let mut or_success_jumps: Vec<usize> = Vec::new();
            for (i, pat) in pats.iter().enumerate() {
                let is_last = i == pats.len() - 1;
                // Dup the arm_copy for this sub-pattern test
                cg.emit_opcode(Opcode::Dup);
                let mut inner_fail: Vec<(usize, usize)> = Vec::new();
                let inner_depth = emit_pattern_test(pat, &mut inner_fail, cg, depth + 1);
                // SUCCESS PATH: pop extras back to depth
                for _ in (depth + 1)..=inner_depth {
                    cg.emit_opcode(Opcode::Pop);
                }
                if is_last {
                    // Fall through to or_success; propagate fails to outer
                    for item in inner_fail {
                        fail_jumps.push(item);
                    }
                } else {
                    // Jump to or_success on this alternative's success
                    or_success_jumps.push(cg.emit_jump(Opcode::Jump));
                    // FAIL PATH: pop Dup'd copy, continue to next alternative
                    for (fail_jump, excess) in inner_fail {
                        cg.patch_jump(fail_jump);
                        for _ in 0..excess.saturating_sub(depth) {
                            cg.emit_opcode(Opcode::Pop);
                        }
                    }
                }
            }
            // All or_success_jumps land here
            for j in or_success_jumps {
                cg.patch_jump(j);
            }
            depth
        }
        // ── list-pattern (v17.2.0) ────────────────────────────────────────────
        IRPattern::List { head, tail } => {
            let head_len = head.len();
            // Step 1: length check
            cg.emit_opcode(Opcode::Dup);          // arm_copy_dup
            cg.emit_opcode(Opcode::ListLen);       // → Int(len)
            let count_idx = cg.const_idx(Constant::Int(head_len as i64));
            cg.emit_opcode(Opcode::Const);
            cg.emit_u16(count_idx);
            if tail.is_none() {
                // Exact length: len == head_len
                cg.emit_opcode(Opcode::Eq);
            } else {
                // At least: head_len <= len  ↔  Const(head_len) Le len
                // Stack: ..., len, head_len  — swap so Le computes head_len <= len
                cg.emit_opcode(Opcode::Swap);
                cg.emit_opcode(Opcode::Le);
            }
            fail_jumps.push((cg.emit_jump(Opcode::JumpIfFalse), depth));
            // Step 2: test each head element
            for (i, head_pat) in head.iter().enumerate() {
                cg.emit_opcode(Opcode::Dup);       // arm_copy_dup
                let idx_c = cg.const_idx(Constant::Int(i as i64));
                cg.emit_opcode(Opcode::Const);
                cg.emit_u16(idx_c);
                cg.emit_opcode(Opcode::ListGet);   // element[i]
                let inner_depth = emit_pattern_test(head_pat, fail_jumps, cg, depth + 1);
                for _ in (depth + 1)..=inner_depth {
                    cg.emit_opcode(Opcode::Pop);
                }
            }
            // Step 3: bind tail if any
            if let Some(tail_slot) = tail {
                cg.emit_opcode(Opcode::Dup);
                let n_idx = cg.const_idx(Constant::Int(head_len as i64));
                cg.emit_opcode(Opcode::Const);
                cg.emit_u16(n_idx);
                cg.emit_opcode(Opcode::ListDrop);  // tail slice
                cg.emit_opcode(Opcode::StoreLocal);
                cg.emit_u16(*tail_slot);
            }
            depth
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
        writer.intern("[]");
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
        let effect_str_idx = writer.intern("[]");
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
        meta: None,
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
                || x == Opcode::ChainCheck as u8
                || x == Opcode::LegacyBindCheck as u8 =>
            {
                ip += 3;
            }
            x if x == Opcode::SeqStageCheck as u8 => {
                // layout: opcode(1) + name_str_idx(2) + stage_idx(1) + total(1) + escape_offset(2)
                remap_u16_at(code, ip + 1, str_remap); // remap name_str_idx
                ip += 7;
            }
            x if x == Opcode::SeqStageEnter as u8 => {
                // layout: opcode(1) + name_str_idx(2)
                remap_u16_at(code, ip + 1, str_remap);
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
                || x == Opcode::EmitEvent as u8
                || x == Opcode::Swap as u8
                || x == Opcode::ListLen as u8
                || x == Opcode::ListGet as u8
                || x == Opcode::ListDrop as u8 =>
            {
                ip += 1;
            }
            x if x == Opcode::MakeClosure as u8 => {
                ip += 5;
            }
            x if x == Opcode::TrackLine as u8 => {
                ip += 5; // 1-byte opcode + 4-byte u32 line number
            }
            x if x == Opcode::CallNamed as u8 => {
                ip += 5; // 1-byte opcode + 2-byte name_const_idx + 2-byte argc
            }
            x if x == Opcode::JumpIfNotVariantC as u8 => {
                ip += 5; // 1-byte opcode + 2-byte const_idx + 2-byte offset (no remap)
            }
            x if x == Opcode::GetFieldC as u8 => {
                ip += 3; // 1-byte opcode + 2-byte const_idx (no remap needed)
            }
            x if x == Opcode::BuildRecordC as u8 => {
                ip += 5; // 1-byte opcode + 2-byte n + 2-byte base_const_idx (no remap)
            }
            x if x == Opcode::MakeClosureN as u8 => {
                ip += 5; // 1-byte opcode + 2-byte name_const_idx + 2-byte capture_count (no remap)
            }
            x if x == Opcode::MergeRecord as u8 => {
                // Layout: opcode(1) + n_overrides(2) + names_idx(2)
                remap_u16_at(code, ip + 3, str_remap); // remap names_idx
                ip += 5;
            }
            x if x == Opcode::RefinementAssert as u8 => {
                // Layout: opcode(1) + name_str_idx(2)
                remap_u16_at(code, ip + 1, str_remap);
                ip += 3;
            }
            // Superinstructions (v20.2.0): opcode(1) + a(2) + b(2) = 5 bytes, no str remap
            x if x == Opcode::AddLL as u8
                || x == Opcode::SubLL as u8
                || x == Opcode::MulLL as u8
                || x == Opcode::AddLC as u8
                || x == Opcode::SubLC as u8
                || x == Opcode::LeLC as u8
                || x == Opcode::LtLC as u8
                || x == Opcode::EqLC as u8
                || x == Opcode::MoveLocal as u8 =>
            {
                ip += 5;
            }
            // GetFieldL: opcode(1) + slot_a(2) + f_idx(2) — f_idx IS a str_table index
            x if x == Opcode::GetFieldL as u8 => {
                remap_u16_at(code, ip + 3, str_remap);
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
