# v20.2.0 実装計画 — スーパー命令（Superinstruction）

## 実装順序

```
T1: codegen.rs — Opcode 追加                        ← 最初（他すべてが依存）
T2: codegen.rs — emit_expr/emit_stmt 融合パターン   ← T1 完了後
T3: codegen.rs — remap_string_operands 更新         ← T1 完了後（T2 と並列可）
T4: vm.rs — resume ループに dispatch 追加            ← T1 完了後（T2/T3 と並列可）
T5: driver.rs — v202000_tests 追加                   ← T1〜T4 完了後
T6: Cargo.toml バージョン更新                        ← 任意のタイミング
```

**変更ファイル一覧:**
- `fav/src/backend/codegen.rs`（T1/T2/T3）
- `fav/src/backend/vm.rs`（T4）
- `fav/src/driver.rs`（T5）
- `fav/Cargo.toml`（T6）

---

## T1: `codegen.rs` — Opcode enum に 10 variants 追加

`RefinementAssert = 0x63,` の直後に追加する。

```rust
    // ── Superinstructions (v20.2.0) — IR-level fused opcodes ────────────────────
    /// AddLL: stack[base+a] + stack[base+b] → push.
    /// Layout: opcode(1) + slot_a(u16 LE) + slot_b(u16 LE) = 5 bytes.
    AddLL = 0xA0,
    /// SubLL: stack[base+a] - stack[base+b] → push.
    SubLL = 0xA1,
    /// MulLL: stack[base+a] * stack[base+b] → push.
    MulLL = 0xA2,
    /// AddLC: stack[base+a] + constants[k] → push.  k must be Constant::Int or Float.
    AddLC = 0xA3,
    /// SubLC: stack[base+a] - constants[k] → push.
    SubLC = 0xA4,
    /// LeLC: (stack[base+a] <= constants[k]) → push Bool.
    LeLC = 0xA5,
    /// LtLC: (stack[base+a] < constants[k]) → push Bool.
    LtLC = 0xA6,
    /// EqLC: (stack[base+a] == constants[k]) → push Bool.
    EqLC = 0xA7,
    /// GetFieldL: stack[base+a].field[str_table[f]] → push.
    /// str_table idx is subject to remap_string_operands.
    /// Layout: opcode(1) + slot_a(u16 LE) + str_idx_f(u16 LE) = 5 bytes.
    GetFieldL = 0xA8,
    /// MoveLocal: stack[base+dst] = stack[base+src].  No stack push/pop.
    /// Layout: opcode(1) + src_slot(u16 LE) + dst_slot(u16 LE) = 5 bytes.
    MoveLocal = 0xA9,
```

---

## T2: `codegen.rs` — emit_expr / emit_stmt 融合パターン追加

### emit_expr::IRExpr::BinOp の書き換え

現状（line ~262）:
```rust
IRExpr::BinOp(op, left, right, _) => {
    emit_expr(left, cg);
    emit_expr(right, cg);
    cg.emit_opcode(match op { ... });
}
```

書き換え後:
```rust
IRExpr::BinOp(op, left, right, _) => {
    // Superinstruction: Local(a) op Local(b)
    if let (IRExpr::Local(a, _), IRExpr::Local(b, _)) = (left.as_ref(), right.as_ref()) {
        let sop = match op {
            BinOp::Add => Some(Opcode::AddLL),
            BinOp::Sub => Some(Opcode::SubLL),
            BinOp::Mul => Some(Opcode::MulLL),
            _          => None,
        };
        if let Some(sop) = sop {
            cg.emit_opcode(sop);
            cg.emit_u16(*a);
            cg.emit_u16(*b);
            return;  // emit_expr には return が使えるか確認。使えない場合は else 節に
        }
    }
    // Superinstruction: Local(a) op Lit(Int(k))
    if let (IRExpr::Local(a, _), IRExpr::Lit(Lit::Int(k), _)) = (left.as_ref(), right.as_ref()) {
        let sop = match op {
            BinOp::Add  => Some(Opcode::AddLC),
            BinOp::Sub  => Some(Opcode::SubLC),
            BinOp::LtEq => Some(Opcode::LeLC),
            BinOp::Lt   => Some(Opcode::LtLC),
            BinOp::Eq   => Some(Opcode::EqLC),
            _           => None,
        };
        if let Some(sop) = sop {
            let k_idx = cg.const_idx(Constant::Int(*k));
            cg.emit_opcode(sop);
            cg.emit_u16(*a);
            cg.emit_u16(k_idx);
            return;
        }
    }
    // Fallback: generic codegen
    emit_expr(left, cg);
    emit_expr(right, cg);
    cg.emit_opcode(match op {
        BinOp::Add          => Opcode::Add,
        BinOp::Sub          => Opcode::Sub,
        BinOp::Mul          => Opcode::Mul,
        BinOp::Div          => Opcode::Div,
        BinOp::Eq           => Opcode::Eq,
        BinOp::NotEq        => Opcode::Ne,
        BinOp::Lt           => Opcode::Lt,
        BinOp::LtEq         => Opcode::Le,
        BinOp::Gt           => Opcode::Gt,
        BinOp::GtEq         => Opcode::Ge,
        BinOp::And          => Opcode::And,
        BinOp::Or           => Opcode::Or,
        BinOp::NullCoalesce => unreachable!("?? desugared before codegen"),
    });
}
```

> **注意**: `emit_expr` が自由関数で `return` が使える場合はそのまま使う。
> もし使えない（たとえば match 内の `return` が別のコンテキストを抜ける）場合は、
> 上記を `if-else` のチェーンに書き直すこと。実際には自由関数なので `return` は使える。

### emit_expr::IRExpr::FieldAccess の書き換え

現状（line ~256）:
```rust
IRExpr::FieldAccess(obj, field, _) => {
    emit_expr(obj, cg);
    let idx = cg.intern_str(field);
    cg.emit_opcode(Opcode::GetField);
    cg.emit_u16(idx);
}
```

書き換え後:
```rust
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
```

### emit_stmt::IRStmt::Bind の書き換え

現状（line ~334）:
```rust
IRStmt::Bind(slot, expr) => {
    emit_expr(expr, cg);
    cg.emit_opcode(Opcode::StoreLocal);
    cg.emit_u16(*slot);
}
```

書き換え後:
```rust
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
```

---

## T3: `codegen.rs` — remap_string_operands 更新

`_ => break` の直前に追加する:

```rust
// Superinstructions (v20.2.0) — 5 bytes each
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
x if x == Opcode::GetFieldL as u8 => {
    // op(1) + slot_a(2) + str_idx_f(2): remap str_idx_f at ip+3
    remap_u16_at(code, ip + 3, str_remap);
    ip += 5;
}
```

---

## T4: `vm.rs` — resume ループに dispatch 追加

既存の `Opcode::RefinementAssert` ハンドラの直後（または末尾の `_ => { return Err(...) }` の直前）に追加。

```rust
// ─── Superinstructions (v20.2.0) ─────────────────────────────────────────
x if x == Opcode::AddLL as u8 => {
    let a = Self::read_u16(function, frame)? as usize;
    let b = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "AddLL: slot a out of bounds"))?;
    let vb = vm.stack.get(frame.base + b).cloned()
        .ok_or_else(|| vm.error(artifact, "AddLL: slot b out of bounds"))?;
    vm.stack.push(apply_numeric_binop(
        va, vb, |x, y| x + y, |x, y| x + y, "add", artifact, &vm.frames,
    )?);
}
x if x == Opcode::SubLL as u8 => {
    let a = Self::read_u16(function, frame)? as usize;
    let b = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "SubLL: slot a out of bounds"))?;
    let vb = vm.stack.get(frame.base + b).cloned()
        .ok_or_else(|| vm.error(artifact, "SubLL: slot b out of bounds"))?;
    vm.stack.push(apply_numeric_binop(
        va, vb, |x, y| x - y, |x, y| x - y, "sub", artifact, &vm.frames,
    )?);
}
x if x == Opcode::MulLL as u8 => {
    let a = Self::read_u16(function, frame)? as usize;
    let b = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "MulLL: slot a out of bounds"))?;
    let vb = vm.stack.get(frame.base + b).cloned()
        .ok_or_else(|| vm.error(artifact, "MulLL: slot b out of bounds"))?;
    vm.stack.push(apply_numeric_binop(
        va, vb, |x, y| x * y, |x, y| x * y, "mul", artifact, &vm.frames,
    )?);
}
x if x == Opcode::AddLC as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let k_idx = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "AddLC: slot out of bounds"))?;
    let vk = function.constants.get(k_idx).cloned().map(constant_to_value)
        .ok_or_else(|| vm.error(artifact, "AddLC: constant out of bounds"))?;
    vm.stack.push(apply_numeric_binop(
        va, vk, |x, y| x + y, |x, y| x + y, "add", artifact, &vm.frames,
    )?);
}
x if x == Opcode::SubLC as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let k_idx = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "SubLC: slot out of bounds"))?;
    let vk = function.constants.get(k_idx).cloned().map(constant_to_value)
        .ok_or_else(|| vm.error(artifact, "SubLC: constant out of bounds"))?;
    vm.stack.push(apply_numeric_binop(
        va, vk, |x, y| x - y, |x, y| x - y, "sub", artifact, &vm.frames,
    )?);
}
x if x == Opcode::LeLC as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let k_idx = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "LeLC: slot out of bounds"))?;
    let vk = function.constants.get(k_idx).cloned().map(constant_to_value)
        .ok_or_else(|| vm.error(artifact, "LeLC: constant out of bounds"))?;
    // compare_pair は Int/Float 混合型を f64 に統一して比較（既存 Le opcode と同一ロジック）
    vm.stack.push(compare_pair((va, vk), |a, b| a <= b, artifact, &vm.frames)?);
}
x if x == Opcode::LtLC as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let k_idx = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "LtLC: slot out of bounds"))?;
    let vk = function.constants.get(k_idx).cloned().map(constant_to_value)
        .ok_or_else(|| vm.error(artifact, "LtLC: constant out of bounds"))?;
    vm.stack.push(compare_pair((va, vk), |a, b| a < b, artifact, &vm.frames)?);
}
x if x == Opcode::EqLC as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let k_idx = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "EqLC: slot out of bounds"))?;
    let vk = function.constants.get(k_idx).cloned().map(constant_to_value)
        .ok_or_else(|| vm.error(artifact, "EqLC: constant out of bounds"))?;
    // VMValue implements PartialEq; same as existing Eq opcode: `left == right`
    vm.stack.push(VMValue::Bool(va == vk));
}
x if x == Opcode::GetFieldL as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let f_idx = Self::read_u16(function, frame)? as usize;
    let field_name = artifact.str_table.get(f_idx).cloned()
        .ok_or_else(|| vm.error(artifact, "GetFieldL: str_table index out of bounds"))?;
    let value = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "GetFieldL: local slot out of bounds"))?;
    // GetField と同じ分岐: Record / Builtin (Math.pi 等) / VariantCtor
    match value {
        VMValue::Record(map) => {
            let v = map.get(&field_name).cloned().ok_or_else(|| {
                vm.error(artifact, &format!("GetFieldL: missing field `{field_name}`"))
            })?;
            vm.stack.push(v);
        }
        VMValue::Builtin(ns) => {
            let full = format!("{}.{}", ns, field_name);
            let v = match full.as_str() {
                "Math.pi" => VMValue::Float(std::f64::consts::PI),
                "Math.e"  => VMValue::Float(std::f64::consts::E),
                _         => VMValue::Builtin(full),
            };
            vm.stack.push(v);
        }
        VMValue::VariantCtor(ns) => {
            vm.stack.push(VMValue::Builtin(format!("{}.{}", ns, field_name)));
        }
        other => return Err(vm.error(
            artifact,
            &format!("GetFieldL: expected Record/Builtin/VariantCtor, got {}", vmvalue_type_name(&other)),
        )),
    }
}
x if x == Opcode::MoveLocal as u8 => {
    let src = Self::read_u16(function, frame)? as usize;
    let dst = Self::read_u16(function, frame)? as usize;
    let value = vm.stack.get(frame.base + src).cloned()
        .ok_or_else(|| vm.error(artifact, "MoveLocal: src slot out of bounds"))?;
    let dst_idx = frame.base + dst;
    if dst_idx >= vm.stack.len() {
        vm.stack.resize(dst_idx + 1, VMValue::Unit);
    }
    vm.stack[dst_idx] = value;
}
```

> **`vmvalue_eq` 不存在**: `vmvalue_eq` という関数は vm.rs に存在しない。
> `EqLC` は `VMValue` の `PartialEq`（`va == vk`）を使う。既存 `Eq` opcode と同一実装。
> 誤って `vmvalue_eq(...)` と書かないこと（コンパイルエラーになる）。

---

## T5: `driver.rs` — `v202000_tests` 追加

v201000_tests の直後に追加する:

```rust
// ── v202000_tests (v20.2.0) — スーパー命令 ──────────────────────────────────
#[cfg(test)]
mod v202000_tests {
    use crate::backend::codegen::Opcode;
    use crate::backend::artifact::FvcArtifact;
    use crate::backend::vm::VM;
    use crate::value::Value;

    fn compile_and_run_si(name: &str, src: &str) -> Value {
        let filename = format!("fav_si_{name}.fav");
        let tmp = std::env::temp_dir().join(filename);
        std::fs::write(&tmp, src).expect("write tmp");
        let path = tmp.to_str().expect("utf8 path");
        let bytes = crate::compiler_fav_runner::compile_file_to_bytes(path)
            .unwrap_or_else(|e| panic!("compile error: {e}"));
        let artifact = FvcArtifact::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("artifact error: {e:?}"));
        let fn_idx = artifact.fn_idx_by_name("main").expect("main not found");
        VM::run(&artifact, fn_idx, vec![]).expect("VM run failed")
    }

    #[test]
    fn version_is_20_2_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("20.2.0"), "Cargo.toml should have version 20.2.0");
    }

    #[test]
    fn addll_opcode_value() {
        assert_eq!(Opcode::AddLL as u8, 0xA0_u8, "AddLL should be 0xA0");
    }

    #[test]
    fn getfieldl_opcode_value() {
        assert_eq!(Opcode::GetFieldL as u8, 0xA8_u8, "GetFieldL should be 0xA8");
    }

    #[test]
    fn superinsn_add_local_local() {
        // a + b exercises AddLL superinstruction
        let result = compile_and_run_si(
            "addll",
            r#"fn add(a: Int, b: Int) -> Int { a + b }
               public fn main() -> Int { add(3, 4) }"#,
        );
        assert_eq!(result, Value::Int(7), "3 + 4 should be 7");
    }

    #[test]
    fn superinsn_tight_loop() {
        // tight_loop(100, 0) exercises SubLC (n-1) + AddLL (acc+n) + LeLC (n <= 0)
        let result = compile_and_run_si(
            "tight_loop",
            r#"fn tight_loop(n: Int, acc: Int) -> Int {
                 if n <= 0 { acc }
                 else { tight_loop(n - 1, acc + n) }
               }
               public fn main() -> Int { tight_loop(100, 0) }"#,
        );
        assert_eq!(result, Value::Int(5050), "sum 1..100 should be 5050");
    }
}
```

---

## T6: `fav/Cargo.toml` バージョン更新

`version = "20.1.0"` → `"20.2.0"`

---

## 注意点

### `apply_numeric_binop` の可視性

`apply_numeric_binop` は `vm.rs` のモジュールレベルまたは `VM` impl 内の自由関数。
`resume` ループ内から呼ぶ際は `vm.` プレフィックスなしで呼べることを確認する。

### `vmvalue_eq` の存在確認

既存コードで `Eq` opcode ハンドラがどう実装されているかを確認し、
同等の比較を `EqLC` ハンドラで再現する。自由関数があれば再利用、なければインライン実装。

### `constant_to_value` の可視性

`constant_to_value` は `vm.rs` 内の自由関数として存在する（既存コードの `Const` ハンドラ参照）。
`AddLC` / `SubLC` などのハンドラで使う。

### LegacyBind / SeqStageCheck との干渉なし

`IRStmt::Bind` の書き換えは通常の `Bind` のみ。
`LegacyBind` / `SeqStageBind` / `Chain` は別の arm なので影響なし。
