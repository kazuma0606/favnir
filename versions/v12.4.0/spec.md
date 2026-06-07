# Favnir v12.4.0 Spec — `seq` Pipeline Fail-Fast

Date: 2026-06-07
Theme: `seq` パイプラインが途中で失敗したら後続 stage を止める

---

## 背景・動機

`seq Pipeline = LoadAndInsert |> Aggregate |> SaveResult` において、
`LoadAndInsert` ステージが `Err(...)` を返しても、`Aggregate` が `Err` 文字列を
引数として受け取り実行を続けていた。結果として ECS タスクが exit 0 で終了し、
S3 に `[ERROR] Aggregate: ...` という文字列が保存されるだけという状況が発生した。

v12.3.0 で `--legacy` モードの `bind` が monadic bind になったことで、
ステージ内部の `bind _ <- Postgres.execute_raw(...)` は Err で短絡するようになった。
しかし **seq パイプライン自体の fail-fast** は未実装であり、
「前ステージが Err を返したら後続ステージを止める」機能が必要。

---

## 設計目標

1. `seq` パイプラインの中間 stage が `Err(...)` または `none` を返したら、後続 stage を実行せず pipeline 全体を即座に失敗させる
2. `ok(v)` を返した場合は `v` を unwrap して次の stage に渡す
3. `some(v)` を返した場合は `v` を unwrap して次の stage に渡す
4. `none` を返した場合はそのまま escape（pipeline 全体が `none` を返す）
5. **非 Result 値**（plain String など）はそのまま次の stage に渡す（pass-through）— `LegacyBindCheck` と同じ挙動
6. エラーには stage 名・連番・総ステージ数を付与:
   ```
   "pipeline stopped at stage 1/3 'LoadAndInsert': db error: SSL required"
   ```

---

## 現状分析

### `compile_flw_def` の現在の IR 出力

```rust
// seq Pipeline = A |> B |> C
// → body = IRExpr::Call(C, [IRExpr::Call(B, [IRExpr::Call(A, [Local(0)])])])
```

A が `Err(e)` を返すと、B が `Err(e)` を引数として受け取り実行を続ける。
ステージ間に ChainCheck 相当の opcode がないため fail-fast にならない。

### v12.3.0 で追加した `LegacyBindCheck` opcode

`LegacyBindCheck = 0x35` — 非 Result は pass-through、Err/none で escape、Ok/some で unwrap。
この挙動はそのまま seq pipeline の中間チェックに使える。

ただし `LegacyBindCheck` は stage 名をエラーメッセージに含めない。
v12.4.0 では新たに **`SeqStageCheck = 0x36`** を追加し、
Err 発生時に stage コンテキスト文字列でラップする。

---

## 実装アプローチ

### Phase B — IRStmt::SeqChain 追加

```rust
// middle/ir.rs
IRStmt::SeqChain {
    slot: u16,
    expr: IRExpr,
    stage_name: String,
    stage_idx: u8,   // 0-based
    total: u8,
}
```

`compile_flw_def` を修正して、非最終ステージの結果を `IRStmt::SeqChain` に変換:

```rust
// before: nested IRExpr::Call
// after: IRExpr::Block(stmts, final_call)
// where stmts = [SeqChain { slot=1, expr=A(input), name="A", idx=0, total=3 },
//                SeqChain { slot=2, expr=B(Local(1)), name="B", idx=1, total=3 }]
// final_call = C(Local(2))
```

### Phase C — Opcode::SeqStageCheck 追加

**幅: 5 bytes** = opcode(1) + escape_offset(2) + stage_idx(1) + total(1)

ステージ名は `CodegenFn.seq_stage_names: Vec<String>` で保持し、
VM は `function.seq_stage_names[stage_idx]` で参照する。

```rust
// backend/codegen.rs
Opcode::SeqStageCheck = 0x36,
```

emit_stmt での SeqChain の codegen:
```rust
IRStmt::SeqChain { slot, expr, stage_name, stage_idx, total } => {
    // push stage_name into cg.seq_stage_names, get name_idx
    emit_expr(expr, cg);
    let escape = cg.emit_jump(Opcode::SeqStageCheck);
    cg.emit_u8(*stage_idx);
    cg.emit_u8(*total);
    cg.emit_opcode(Opcode::StoreLocal);
    cg.emit_u16(*slot);
    cg.chain_escapes.push(escape);
}
```

### Phase D — VM SeqStageCheck ハンドラ

```rust
x if x == Opcode::SeqStageCheck as u8 => {
    let offset = Self::read_u16(function, frame)?  as usize;
    let stage_idx = function.bytecode[frame.ip] as usize; frame.ip += 1;
    let total     = function.bytecode[frame.ip] as usize; frame.ip += 1;
    let value = vm.stack.pop().ok_or(...)?;
    match value {
        VMValue::Variant(tag, payload) if tag == "ok" || tag == "some" => {
            let inner = payload.map(|b| *b).unwrap_or(VMValue::Unit);
            vm.stack.push(inner);
        }
        VMValue::Variant(tag, payload) if tag == "err" || tag == "none" => {
            // wrap error with stage context
            let stage_name = function.seq_stage_names
                .get(stage_idx)
                .map(|s| s.as_str())
                .unwrap_or("?");
            let msg = match &*payload {
                Some(b) => format!(
                    "pipeline stopped at stage {}/{} '{}': {}",
                    stage_idx + 1, total, stage_name,
                    vm_value_to_string(b)
                ),
                None => format!(
                    "pipeline stopped at stage {}/{} '{}': none",
                    stage_idx + 1, total, stage_name
                ),
            };
            vm.stack.push(VMValue::Variant("err".to_string(), Some(Box::new(VMValue::Str(msg)))));
            frame.ip = frame.ip.checked_add(offset)?;
        }
        other => { vm.stack.push(other); } // non-Result: pass-through
    }
}
```

### Phase E — CodegenFn / Artifact に seq_stage_names 追加

```rust
// backend/codegen.rs
struct CodegenFn {
    // ...existing fields...
    pub seq_stage_names: Vec<String>,  // stage names for SeqStageCheck
}

// backend/vm.rs (CodegenFn / compiled function struct)
pub seq_stage_names: Vec<String>,
```

Artifact の serialize/deserialize も更新する。

### Phase F — 全 IRStmt パターンマッチ更新

LegacyBind 追加時と同様、以下の箇所に `IRStmt::SeqChain` を追加:
- `driver.rs::collect_tracklines_in_expr`
- `driver.rs::remap_ir_stmt`
- `driver.rs::opcode_info` — `SeqStageCheck` の幅 = 5 bytes
- `backend/wasm_codegen.rs` — UnsupportedExpr として
- `middle/ir.rs::collect_stmt_deps`
- `driver.rs::apply_legacy_bind_semantics` の `legacy_transform_stmt` — SeqChain はそのまま通過

---

## テスト設計

### 正常系

```
seq_passes_ok_through
```
- `stage A: String -> String = |s| { ok(s) }` を 2 ステージ連結
- seq pipeline 結果が `ok("input")` になること（Ok を unwrap して渡す）

```
seq_plain_value_passes_through
```
- `stage A: String -> String = |s| { s }` (plain String) を 2 ステージ連結
- 非 Result 値は pass-through で正常動作すること

### 短絡系

```
seq_stops_on_stage_err
```
- Stage 1 が `err("fail")` を返す
- Stage 2 が呼ばれないこと（副作用カウンター等で確認）
- Pipeline 全体が `Err(...)` を返すこと

```
seq_error_includes_stage_name
```
- Stage 1 (`LoadData`) が `err("db error")` を返す
- Pipeline の最終結果が `Err("pipeline stopped at stage 1/2 'LoadData': db error")` であること

```
seq_error_at_middle_stage
```
- 3 ステージ中 Stage 2 が `err("mid fail")` を返す
- Stage 3 が実行されないこと
- エラーに `stage 2/3` が含まれること

### 後方互換確認

```
seq_legacy_mode_fail_fast
```
- `--legacy` モードでも seq fail-fast が有効なこと
- （`apply_legacy_bind_semantics` が SeqChain を変換しないことを確認）

### バージョン確認

```
version_is_12_4_0
```
- `CARGO_PKG_VERSION == "12.4.0"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `seq A |> B` で A が `err` → B は実行されない | |
| `seq A |> B` で A が `ok(v)` → B が `v` を受け取る（unwrap） | |
| `seq A |> B` で A が plain String → B がそのまま受け取る（pass-through） | |
| エラーに `stage N/M 'StageName'` が含まれる | |
| `cargo test v12400` 7 件通過 | |
| `cargo test` 全通過 | |

---

## 非目標（v12.4.0 スコープ外）

- `fav run --verbose` での stage トレース出力（v12.5.0）
- `par [A, B]` の fail-fast（今回は単一ステージのみ対象）
- checker.fav での seq pipeline 型チェック強化
