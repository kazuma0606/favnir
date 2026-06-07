# Favnir v12.4.0 Plan — `seq` Pipeline Fail-Fast

Date: 2026-06-07

---

## 全体方針

v12.3.0 で追加した `LegacyBindCheck` opcode と同じパターンで
`SeqStageCheck` opcode を追加する。

`compile_flw_def` が現在生成している「ネストした IRExpr::Call」構造を
「`IRExpr::Block` + `IRStmt::SeqChain` の列 + 最終 stage の return expr」に変更する。

---

## Phase A — 現状把握

**目標**: `compile_flw_def` の IR 出力と、bytecode レベルの実行フローを理解する。

1. `fav/src/middle/compiler.rs` の `compile_flw_def`（行 580〜663）を精読
   - 現在の body = nested `IRExpr::Call` 構造を確認
   - `IRExpr::Block(Vec<IRStmt>, Box<IRExpr>, Type)` が既存であることを確認
2. `fav/src/backend/codegen.rs` の `emit_expr` で `IRExpr::Block` がどう codegen されるかを確認
3. `fav/src/backend/vm.rs` の `LegacyBindCheck` ハンドラ（0x35）を確認
   - `chain_escapes` が `Return` opcode 直前の offset をパッチする仕組みを確認

**成果物**: 実装アプローチの確定

---

## Phase B — IRStmt::SeqChain 追加

**ファイル**: `fav/src/middle/ir.rs`

1. `IRStmt` enum に追加:
   ```rust
   IRStmt::SeqChain {
       slot: u16,
       expr: IRExpr,
       stage_name: String,
       stage_idx: u8,
       total: u8,
   }
   ```

2. `collect_stmt_deps` の match に `SeqChain` を追加:
   ```rust
   IRStmt::SeqChain { expr, .. } => collect_expr_deps(expr, globals, deps),
   ```

---

## Phase C — compile_flw_def を Block スタイルに修正

**ファイル**: `fav/src/middle/compiler.rs`

`compile_flw_def` を以下のように書き直す:

```rust
fn compile_flw_def(fd: &FlwDef, ctx: &mut CompileCtx) -> IRFnDef {
    // ... save/restore ctx ...
    let input_slot = ctx.define_local("$input");

    let total = fd.steps.len();
    let mut stmts: Vec<IRStmt> = Vec::new();
    let mut current_slot = input_slot;

    for (idx, step) in fd.steps.iter().enumerate() {
        let is_last = idx == total - 1;
        let call_expr = build_step_call(step, current_slot, ctx); // 既存ロジックを抽出

        if is_last {
            // 最終ステージは return expr
            let body_expr = IRExpr::Block(stmts, Box::new(call_expr), Type::Unknown);
            return IRFnDef { body: body_expr, ... };
        } else {
            let tmp_slot = ctx.alloc_slot();
            let stage_name = step_name_str(step); // "A" or "par[A,B]"
            stmts.push(IRStmt::SeqChain {
                slot: tmp_slot,
                expr: call_expr,
                stage_name,
                stage_idx: idx as u8,
                total: total as u8,
            });
            current_slot = tmp_slot;
        }
    }
    // edge case: 0 or 1 step (no intermediate stmts)
    // ...
}
```

単一ステージの場合は従来通り `body = IRExpr::Call(stage, [input])` でよい。

---

## Phase D — CodegenFn / Artifact に seq_stage_names 追加

**ファイル**: `fav/src/backend/codegen.rs`

1. `CodegenFn` struct に `pub seq_stage_names: Vec<String>` を追加
2. `CodegenCtx` / `begin_fn` 等でこのフィールドを初期化

**ファイル**: `fav/src/backend/vm.rs` のコンパイル済み関数構造体

3. Artifact 内の関数構造体にも `seq_stage_names: Vec<String>` を追加
4. serialize / deserialize (bincode) の更新

---

## Phase E — Opcode::SeqStageCheck 追加 + emit_stmt

**ファイル**: `fav/src/backend/codegen.rs`

1. `Opcode` enum に追加:
   ```rust
   SeqStageCheck = 0x36,
   ```

2. `emit_stmt` に `IRStmt::SeqChain` ケース追加:
   ```rust
   IRStmt::SeqChain { slot, expr, stage_name, stage_idx, total } => {
       // push stage_name into cg.seq_stage_names
       let name_idx = cg.push_seq_stage_name(stage_name.clone());
       emit_expr(expr, cg);
       let escape = cg.emit_jump(Opcode::SeqStageCheck);
       cg.emit_u8(*stage_idx);
       cg.emit_u8(*total);
       cg.emit_opcode(Opcode::StoreLocal);
       cg.emit_u16(*slot);
       cg.chain_escapes.push(escape);
   }
   ```

   emit_jump で SeqStageCheck は 3 bytes (opcode + u16 offset)、
   その後 stage_idx(1) + total(1) = 合計 5 bytes

---

## Phase F — VM: SeqStageCheck ハンドラ追加

**ファイル**: `fav/src/backend/vm.rs`

`LegacyBindCheck` ハンドラの直後に `SeqStageCheck` ハンドラを追加:

```rust
x if x == Opcode::SeqStageCheck as u8 => {
    let offset    = Self::read_u16(function, frame)? as usize;
    let stage_idx = function.bytecode[frame.ip] as usize; frame.ip += 1;
    let total     = function.bytecode[frame.ip] as usize; frame.ip += 1;
    let value = vm.stack.pop().ok_or(...)?;
    match value {
        VMValue::Variant(tag, payload) if tag == "ok" || tag == "some" => {
            let inner = payload.map(|b| *b).unwrap_or(VMValue::Unit);
            vm.stack.push(inner);
        }
        VMValue::Variant(tag, payload) if tag == "err" || tag == "none" => {
            let stage_name = function.seq_stage_names
                .get(stage_idx)
                .map(|s| s.as_str())
                .unwrap_or("?");
            let inner_msg = match payload.as_deref() {
                Some(v) => vm_value_to_display_str(v),
                None => "none".to_string(),
            };
            let wrapped = format!(
                "pipeline stopped at stage {}/{} '{}': {}",
                stage_idx + 1, total, stage_name, inner_msg
            );
            vm.stack.push(VMValue::Variant(
                "err".to_string(),
                Some(Box::new(VMValue::Str(wrapped))),
            ));
            frame.ip = frame.ip.checked_add(offset).ok_or(...)?;
        }
        other => { vm.stack.push(other); } // non-Result: pass-through
    }
}
```

---

## Phase G — 全 IRStmt パターンマッチ更新

`LegacyBind` を追加した時と同様、以下を更新:

1. `driver.rs::collect_tracklines_in_expr` — `SeqChain` の expr を再帰
2. `driver.rs::remap_ir_stmt` — `SeqChain` の expr を remap
3. `driver.rs::opcode_info` — `SeqStageCheck => ("SeqStageCheck", 5)` （opcode1 + offset2 + idx1 + total1）
4. `backend/wasm_codegen.rs` — 全 5 箇所に `SeqChain` 追加（UnsupportedExpr）
5. `driver.rs::apply_legacy_bind_semantics::legacy_transform_stmt` — `SeqChain` はそのまま通過（expr を再帰変換のみ）
6. `emit_python.rs` — `SeqChain` 追加（Unsupported として）

---

## Phase H — テスト追加（v12400_tests モジュール）

**ファイル**: `fav/src/driver.rs`

`v12300_tests` モジュールの直後に `v12400_tests` モジュールを追加。

7 件のテスト:
1. `seq_passes_ok_through`
2. `seq_plain_value_passes_through`
3. `seq_stops_on_stage_err`
4. `seq_error_includes_stage_name`
5. `seq_error_at_middle_stage`
6. `seq_legacy_mode_fail_fast`
7. `version_is_12_4_0`

`version_is_12_3_0` テストは削除する（v12300_tests モジュールから）。

---

## Phase I — 全テスト通過確認

```bash
cargo test v12400 -- --nocapture
cargo test
```

期待: v12400 テスト 7 件 + 全テスト通過（1370 + 7 = 1377 件程度）

---

## Phase J — バージョン更新 + コミット

1. `fav/Cargo.toml` version → `"12.4.0"`
2. `cargo build` で `Cargo.lock` 更新
3. `git add -A && git commit -m "feat: v12.4.0 — seq pipeline fail-fast (SeqStageCheck opcode)"`
4. `git push`

---

## リスク・注意点

1. **`chain_escapes` の仕組みを正確に理解する**: LegacyBind と同様、SeqChain が生成した escape offset は関数末尾の `Return` opcode に向けてパッチされる必要がある。compile_flw_def で生成した Block の return expr の後に `Return` が来ることを確認すること。

2. **`IRExpr::Block` 内の stmt の chain_escapes は関数全体の escape リストと共有される**: `emit_expr` の Block ケースで `cg.chain_escapes` に push された escape が、関数の Return 直前でパッチされるかを確認すること。

3. **単一ステージの FlwDef は変更不要**: `total == 1` の場合は SeqChain が 0 件 → 従来通り動作。

4. **`CodegenFn` の seq_stage_names が deserialize で空になる既存 artifact**: `seq_stage_names` がない古い artifact に対するデフォルト値（空 Vec）を設定する。
