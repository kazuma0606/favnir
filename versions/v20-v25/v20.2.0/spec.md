# v20.2.0 Spec — スーパー命令（Superinstruction）

## 概要

v20.2.0 は VM のディスパッチコストを削減する **スーパー命令（Superinstruction）** を実装する。

現状の VM はバイトコードを1命令ずつ dispatch loop で処理する。
タイトループや頻繁なフィールドアクセスでは同じ短いパターンが繰り返す:

```
LoadLocal(0) + LoadLocal(1) + Add    // 7 bytes, 3 dispatch
LoadLocal(0) + Const(1) + Sub        // 7 bytes, 3 dispatch
LoadLocal(0) + GetField("amount")    // 6 bytes, 2 dispatch
```

これらを単一の命令（スーパー命令）に融合することで、dispatch ループの反復回数を削減し
VM のスループットを改善する。

**テーマ**: Runtime Excellence シリーズ第2弾 — 計測に基づく最初の最適化

---

## 動機と期待効果

v20.1.0 で整備したベンチマーク基盤に基づく改善。
v20.0.0 比での期待改善（v20.1.0 = v20.0.0 と同一ベースライン。NaN-boxing 等との組み合わせで v21.0 目標 < 30ms を目指す）:

| ベンチマーク | v20.0.0 基準 | 期待改善 |
|---|---|---|
| `tight_loop_10m_iter_ms` | ~85ms | **+20〜30%** |
| `record_transform_1m_ms` | ~210ms | **+10〜15%** |

---

## 設計アーキテクチャ

### アプローチ: IR レベルでの融合（コンパイル時）

後処理でバイトコードを書き換えるのではなく、**codegen の emit_expr / emit_stmt で
パターンを検出して直接スーパー命令を出力する**。

利点:
- ジャンプターゲットのオフセット調整が不要（バイトコード変更なし）
- 実装がシンプルで既存テストを壊すリスクが低い
- バイト数削減（最大 7 → 5 bytes = -29%）

### 融合対象パターン

`emit_expr` が `IRExpr::BinOp` / `IRExpr::FieldAccess` を処理する際、
以下のサブパターンを検出してスーパー命令を出力する。

`emit_stmt` が `IRStmt::Bind(dst, expr)` を処理する際、
`expr = IRExpr::Local(src)` のパターンを検出して `MoveLocal` を出力する。

---

## スーパー命令一覧（10件）

| 命令 | 値 | 置き換え対象 IR パターン | バイト数 | str remap |
|---|---|---|---|---|
| `AddLL` | 0xA0 | `BinOp(Add, Local(a), Local(b))` | 5 | no |
| `SubLL` | 0xA1 | `BinOp(Sub, Local(a), Local(b))` | 5 | no |
| `MulLL` | 0xA2 | `BinOp(Mul, Local(a), Local(b))` | 5 | no |
| `AddLC` | 0xA3 | `BinOp(Add, Local(a), Lit(Int(k)))` | 5 | no |
| `SubLC` | 0xA4 | `BinOp(Sub, Local(a), Lit(Int(k)))` | 5 | no |
| `LeLC` | 0xA5 | `BinOp(LtEq, Local(a), Lit(Int(k)))` | 5 | no |
| `LtLC` | 0xA6 | `BinOp(Lt, Local(a), Lit(Int(k)))` | 5 | no |
| `EqLC` | 0xA7 | `BinOp(Eq, Local(a), Lit(Int(k)))` | 5 | no |
| `GetFieldL` | 0xA8 | `FieldAccess(Local(a), field)` | 5 | YES (f_idx at ip+3) |
| `MoveLocal` | 0xA9 | `Bind(dst, Local(src))` | 5 | no |

**旧バイト数 → 新バイト数:**
- `Local(a) + Local(b) + BinOp`: LoadLocal(3) + LoadLocal(3) + op(1) = **7 → 5**
- `Local(a) + Lit(Int(k)) + BinOp`: LoadLocal(3) + Const(3) + op(1) = **7 → 5**
- `FieldAccess(Local(a), f)`: LoadLocal(3) + GetField(3) = **6 → 5**
- `Bind(dst, Local(src))`: LoadLocal(3) + StoreLocal(3) = **6 → 5**

---

## バイトコードエンコーディング

各スーパー命令は `opcode(1) + operand_a(u16 LE) + operand_b(u16 LE)` = **5 bytes**。

| 命令 | operand_a | operand_b |
|---|---|---|
| AddLL / SubLL / MulLL | local_slot_a (u16) | local_slot_b (u16) |
| AddLC / SubLC / LeLC / LtLC / EqLC | local_slot_a (u16) | const_pool_idx_k (u16) |
| GetFieldL | local_slot_a (u16) | str_table_idx_f (u16) |
| MoveLocal | src_slot (u16) | dst_slot (u16) |

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/backend/codegen.rs` | ①`Opcode` enum に 10 variants 追加（0xA0〜0xA9）<br>②`emit_expr::IRExpr::BinOp` にパターンマッチ追加<br>③`emit_expr::IRExpr::FieldAccess` にパターンマッチ追加<br>④`emit_stmt::IRStmt::Bind` にパターンマッチ追加<br>⑤`remap_string_operands` に新 opcodes の stride 追加（GetFieldL のみ str remap） |
| `fav/src/backend/vm.rs` | `resume` ループに 10 opcode の dispatch ケース追加 |
| `fav/src/driver.rs` | `v202000_tests` モジュール追加（5件） |
| `fav/Cargo.toml` | version `20.1.0` → `20.2.0` |

---

## codegen.rs 詳細仕様

### Opcode enum への追加

```rust
// Superinstructions (v20.2.0) — IR-level fused opcodes
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
/// stack[base+a].field[str_table[f]] → push (str remap applies)
GetFieldL = 0xA8,
/// stack[base+src] → stack[base+dst] (copy, no stack push/pop)
MoveLocal = 0xA9,
```

### emit_expr::BinOp の変更

```rust
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
            return; // ← early return (emit_expr は &mut Codegen を受け取る)
        }
    }
    // Superinstruction fusion: Local(a) op Lit(Int(k))
    // Note: Gt/GtEq は省略（ループ終了条件での出現頻度が低いため Phase 1 スコープ外）
    if let (IRExpr::Local(a, _), IRExpr::Lit(Lit::Int(k), _)) = (left.as_ref(), right.as_ref()) {
        let super_op = match op {
            BinOp::Add   => Some(Opcode::AddLC),
            BinOp::Sub   => Some(Opcode::SubLC),
            BinOp::LtEq  => Some(Opcode::LeLC),
            BinOp::Lt    => Some(Opcode::LtLC),
            BinOp::Eq    => Some(Opcode::EqLC),
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
    cg.emit_opcode(match op { ... });
}
```

> 注意: `emit_expr` は現在 `fn emit_expr(expr: &IRExpr, cg: &mut Codegen)` のシグネチャで
> `return` ではなく `match` arm の末尾で書くのが慣例。
> 実際には `match` 全体を再構成し、パターンを arm の最初に置く。

### emit_expr::FieldAccess の変更

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

### emit_stmt::Bind の変更

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

### remap_string_operands への追加

```rust
// AddLL/SubLL/MulLL: op(1)+a(2)+b(2) = 5 bytes, no str remap
x if x == Opcode::AddLL as u8
    || x == Opcode::SubLL as u8
    || x == Opcode::MulLL as u8
    || x == Opcode::AddLC as u8
    || x == Opcode::SubLC as u8
    || x == Opcode::LeLC as u8
    || x == Opcode::LtLC as u8
    || x == Opcode::EqLC as u8
    || x == Opcode::MoveLocal as u8 => { ip += 5; }

// GetFieldL: op(1)+a(2)+f_idx(2) — f_idx IS a str_table index
x if x == Opcode::GetFieldL as u8 => {
    remap_u16_at(code, ip + 3, str_remap);
    ip += 5;
}
```

---

## vm.rs 詳細仕様

`resume` ループに追加する dispatch ケース:

```rust
// ─── Superinstructions (v20.2.0) ─────────────────────────────────
x if x == Opcode::AddLL as u8 => {
    let a = Self::read_u16(function, frame)? as usize;
    let b = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "AddLL: slot a out of bounds"))?;
    let vb = vm.stack.get(frame.base + b).cloned()
        .ok_or_else(|| vm.error(artifact, "AddLL: slot b out of bounds"))?;
    vm.stack.push(apply_numeric_binop(va, vb, |x, y| x + y, |x, y| x + y, "add", artifact, &vm.frames)?);
}
x if x == Opcode::SubLL as u8 => { /* 同様に sub */ }
x if x == Opcode::MulLL as u8 => { /* 同様に mul */ }

x if x == Opcode::AddLC as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let k_idx = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "AddLC: slot out of bounds"))?;
    let vk = function.constants.get(k_idx).cloned()
        .map(constant_to_value)
        .ok_or_else(|| vm.error(artifact, "AddLC: constant out of bounds"))?;
    vm.stack.push(apply_numeric_binop(va, vk, |x, y| x + y, |x, y| x + y, "add", artifact, &vm.frames)?);
}
x if x == Opcode::SubLC as u8 => { /* 同様に sub */ }

x if x == Opcode::LeLC as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let k_idx = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "LeLC: slot out of bounds"))?;
    let vk = function.constants.get(k_idx).cloned()
        .map(constant_to_value)
        .ok_or_else(|| vm.error(artifact, "LeLC: constant out of bounds"))?;
    // compare_pair: handles Int/Float mixed types (same as Le opcode)
    vm.stack.push(compare_pair((va, vk), |a, b| a <= b, artifact, &vm.frames)?);
}
x if x == Opcode::LtLC as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let k_idx = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "LtLC: slot out of bounds"))?;
    let vk = function.constants.get(k_idx).cloned()
        .map(constant_to_value)
        .ok_or_else(|| vm.error(artifact, "LtLC: constant out of bounds"))?;
    vm.stack.push(compare_pair((va, vk), |a, b| a < b, artifact, &vm.frames)?);
}
x if x == Opcode::EqLC as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let k_idx = Self::read_u16(function, frame)? as usize;
    let va = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "EqLC: slot out of bounds"))?;
    let vk = function.constants.get(k_idx).cloned()
        .map(constant_to_value)
        .ok_or_else(|| vm.error(artifact, "EqLC: constant out of bounds"))?;
    // VMValue implements PartialEq (same as Eq opcode: `left == right`)
    vm.stack.push(VMValue::Bool(va == vk));
}

x if x == Opcode::GetFieldL as u8 => {
    let a     = Self::read_u16(function, frame)? as usize;
    let f_idx = Self::read_u16(function, frame)? as usize;
    let field_name = artifact.str_table.get(f_idx).cloned()
        .ok_or_else(|| vm.error(artifact, "GetFieldL: str_table index out of bounds"))?;
    let value = vm.stack.get(frame.base + a).cloned()
        .ok_or_else(|| vm.error(artifact, "GetFieldL: local slot out of bounds"))?;
    // Same branches as GetField: Record / Builtin / VariantCtor
    match value {
        VMValue::Record(map) => {
            let v = map.get(&field_name).cloned()
                .ok_or_else(|| vm.error(artifact, &format!("GetFieldL: missing field `{field_name}`")))?;
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
        other => return Err(vm.error(artifact,
            &format!("GetFieldL: expected Record/Builtin/VariantCtor, got {}", vmvalue_type_name(&other)))),
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

> **実装ノート**: `LeLC` / `LtLC` は既存の `compare_pair` 自由関数（vm.rs）を再利用する。
> `EqLC` は `VMValue` の `PartialEq`（`va == vk`）を使う（既存 `Eq` opcode と同一）。
> `vmvalue_eq` という名前の関数は存在しないため使用しないこと。

---

## テスト（v202000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_2_0` | Cargo.toml に `"20.2.0"` が含まれる |
| `addll_opcode_value` | `Opcode::AddLL as u8 == 0xA0` |
| `getfieldl_opcode_value` | `Opcode::GetFieldL as u8 == 0xA8` |
| `superinsn_add_local_local` | `fn add(a: Int, b: Int) -> Int { a + b }` → `add(3, 4) == 7` |
| `superinsn_tight_loop` | `tight_loop(100, 0) == 5050`（再帰タイトループの正確性確認） |

tests 4〜5 は `compile_and_run_si` ヘルパー（direct Rust codegen path、plan.md T5 で定義）を使用する。

---

## 完了条件

- [ ] `Opcode` enum に `AddLL`〜`MoveLocal`（0xA0〜0xA9）が追加されている
- [ ] `emit_expr::BinOp` が 8 パターンのスーパー命令を出力する
- [ ] `emit_expr::FieldAccess(Local)` が `GetFieldL` を出力する
- [ ] `emit_stmt::Bind(Local)` が `MoveLocal` を出力する
- [ ] `remap_string_operands` が新オペコードを正しくスキップ（GetFieldL は str remap）
- [ ] `resume` ループが 10 opcode すべてを dispatch する
- [ ] `fav/Cargo.toml` version が `20.2.0`
- [ ] `cargo test v202000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし（全既存テストが PASS）
- [ ] `benchmarks/v20.2.0.json` が生成されている
- [ ] `tight_loop_10m_iter_ms` が v20.0.0 比 +20% 以上改善していることを確認

---

## 技術ノート

### emit_expr の構造について

現状 `emit_expr` は `fn emit_expr(expr: &IRExpr, cg: &mut Codegen)` で戻り値なし。
BinOp arm の中で early return したい場合、arm 全体を新しい `match` 構造に書き直す。
具体的には:

```rust
IRExpr::BinOp(op, left, right, _) => {
    // パターン1: LL 融合
    match (left.as_ref(), right.as_ref()) {
        (IRExpr::Local(a, _), IRExpr::Local(b, _)) => {
            if let Some(sop) = ll_super_op(op) {
                cg.emit_opcode(sop); cg.emit_u16(*a); cg.emit_u16(*b);
                return;   // ← これが使えない → 代わりに else ブランチ
            }
        }
        _ => {}
    }
    // ...フォールバック
}
```

`return` が使えない場合は `if let ... { ... } else { ... }` の入れ子にする。

### MoveLocal の注意点

`MoveLocal(src, dst)` はスタックへの push/pop を行わない（直接 `vm.stack[base+dst]` に書く）。
`try_apply_tco` 内のジャンプ追跡コードが MoveLocal を認識できるよう、`insn_size` 相当の場所で
5 bytes として扱うことを確認する（`remap_string_operands` のループが該当）。

### TCO（末尾呼び出し最適化）との干渉

既存の `try_apply_tco` は `OpCode::Jump` と `Opcode::Return` のみを認識してテール判定する。
新スーパー命令は `Return` の前に現れる可能性があるが、TCO の判定ロジック自体は
「Call の直後に Return（またはJump→Return）がある」パターンを見るため、影響なし。
（スーパー命令は Call の前に現れ、Call → Return の間には新命令は入らない）
