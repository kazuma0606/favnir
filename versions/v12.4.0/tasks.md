# Favnir v12.4.0 Tasks

Date: 2026-06-07
Theme: `seq` pipeline fail-fast（`SeqStageCheck` opcode）

---

## Phase A — 現状把握

- [ ] A-1: `fav/src/middle/compiler.rs` の `compile_flw_def`（行 580〜663）を精読
  - `IRExpr::Block(Vec<IRStmt>, Box<IRExpr>, Type)` が既存であることを確認
  - 現在の body = nested `IRExpr::Call` 構造を確認
- [ ] A-2: `backend/codegen.rs` の `emit_expr` で `IRExpr::Block` の codegen を確認
- [ ] A-3: `backend/vm.rs` の `LegacyBindCheck`（0x35）ハンドラと `chain_escapes` パッチ機構を確認

---

## Phase B — IRStmt::SeqChain 追加

- [ ] B-1: `middle/ir.rs` に `IRStmt::SeqChain { slot: u16, expr: IRExpr, stage_name: String, stage_idx: u8, total: u8 }` 追加
- [ ] B-2: `middle/ir.rs::collect_stmt_deps` に `SeqChain` ケース追加

---

## Phase C — compile_flw_def を Block スタイルに修正

- [ ] C-1: `middle/compiler.rs::compile_flw_def` を修正
  - 2 ステージ以上: `IRExpr::Block(SeqChain stmts, final_call)` を生成
  - 単一ステージ: 従来通り `IRExpr::Call` のまま（変更なし）
  - `step_name_str` ヘルパー関数追加（`FlwStep::Stage(name) → name`、`FlwStep::Par(names) → "par[A,B]"`）

---

## Phase D — CodegenFn / Artifact に seq_stage_names 追加

- [ ] D-1: `backend/codegen.rs` の `CodegenFn` struct に `pub seq_stage_names: Vec<String>` 追加
- [ ] D-2: `push_seq_stage_name` メソッドを `CodegenCtx` に追加（名前を push して index を返す）
- [ ] D-3: `backend/vm.rs` の関数構造体（Artifact 内）に `seq_stage_names: Vec<String>` 追加
- [ ] D-4: Artifact の serialize / deserialize（bincode）更新 — デフォルト空 Vec で後方互換を維持

---

## Phase E — Opcode::SeqStageCheck 追加 + emit_stmt

- [ ] E-1: `backend/codegen.rs` に `Opcode::SeqStageCheck = 0x36` 追加
- [ ] E-2: `emit_stmt` に `IRStmt::SeqChain` ケース追加
  - `emit_expr(expr)` → `emit_jump(SeqStageCheck)` → `emit_u8(stage_idx)` → `emit_u8(total)` → `StoreLocal` → `emit_u16(slot)`
  - `cg.chain_escapes.push(escape)` で Return 直前にパッチ

---

## Phase F — VM: SeqStageCheck ハンドラ追加

- [ ] F-1: `backend/vm.rs` に `SeqStageCheck` ハンドラ追加
  - `ok(v)` / `some(v)` → unwrap して push、続行
  - `err(e)` / `none` → `"pipeline stopped at stage I/N 'name': e"` に wrap → push → escape
  - その他 → pass-through（そのまま push、続行）

---

## Phase G — 全 IRStmt パターンマッチ更新

- [ ] G-1: `driver.rs::collect_tracklines_in_expr` に `SeqChain` 追加（expr を再帰）
- [ ] G-2: `driver.rs::remap_ir_stmt` に `SeqChain` 追加（expr を remap）
- [ ] G-3: `driver.rs::opcode_info` に `SeqStageCheck => ("SeqStageCheck", 5)` 追加
- [ ] G-4: `backend/wasm_codegen.rs` — 全 5 箇所に `SeqChain` 追加（UnsupportedExpr）
- [ ] G-5: `driver.rs::legacy_transform_stmt` — `SeqChain` を追加（expr のみ再帰変換）
- [ ] G-6: `emit_python.rs` — `SeqChain` 追加（Unsupported または変換なし）
- [ ] G-7: `cargo build` でコンパイルエラーがないことを確認

---

## Phase H — v12400_tests モジュール追加

- [ ] H-1: 正常系テスト（2件）
  - [ ] `seq_passes_ok_through` — `ok(v)` を返す 2 ステージ seq → v が unwrap されて最終結果
  - [ ] `seq_plain_value_passes_through` — plain String を返す 2 ステージ seq → pass-through 動作確認
- [ ] H-2: 短絡系テスト（3件）
  - [ ] `seq_stops_on_stage_err` — Stage 1 が `err("fail")` → Stage 2 は実行されない
  - [ ] `seq_error_includes_stage_name` — `err("db error")` → `Err("pipeline stopped at stage 1/2 'LoadData': db error")`
  - [ ] `seq_error_at_middle_stage` — 3 ステージ中 Stage 2 が Err → Stage 3 未実行、`stage 2/3` がエラーに含まれる
- [ ] H-3: 後方互換確認（1件）
  - [ ] `seq_legacy_mode_fail_fast` — `--legacy` モードでも seq fail-fast が有効なこと
- [ ] H-4: バージョン確認（1件）
  - [ ] `version_is_12_4_0` — `CARGO_PKG_VERSION == "12.4.0"`
- [ ] H-5: `v12300_tests` モジュールから `version_is_12_3_0` テストを削除
- [ ] H-6: `cargo test v12400 -- --nocapture` — 7 件通過確認

---

## Phase I — 全テスト通過確認

- [ ] I-1: `cargo test` — 全通過（1377 件程度）

---

## Phase J — バージョン更新 + コミット

- [ ] J-1: `fav/Cargo.toml` version → `"12.4.0"`
- [ ] J-2: `cargo build` で `Cargo.lock` 更新
- [ ] J-3: `git add fav/Cargo.toml fav/Cargo.lock fav/src/ versions/v12.4.0/ && git commit -m "feat: v12.4.0 — seq pipeline fail-fast (SeqStageCheck opcode)"`
- [ ] J-4: `git push` → CI/CD 通過確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `seq A \|> B` で A が `err` → B は実行されない | |
| `seq A \|> B` で A が `ok(v)` → B が `v` を受け取る（unwrap） | |
| `seq A \|> B` で A が plain String → B がそのまま受け取る（pass-through） | |
| エラーに `stage N/M 'StageName'` が含まれる | |
| `cargo test v12400` 7 件通過 | |
| `cargo test` 全通過 | |

---

## 技術的注意点

1. **`chain_escapes` パッチ機構**: `LegacyBind` と同様、`SeqChain` が push した escape offset は関数末尾の `Return` opcode 位置でパッチされる。`emit_expr(IRExpr::Block)` がこのパッチを正しく行うか確認すること。

2. **`IRExpr::Block` の codegen**: `emit_expr` の `IRExpr::Block` ケースが `stmts` を `emit_stmt` で処理し、`final_expr` を評価後 `Return` を emit することを確認。

3. **opcode 幅**: `SeqStageCheck` は 5 bytes（opcode 1 + escape_offset 2 + stage_idx 1 + total 1）。`opcode_info` で幅 5 を設定すること。

4. **`emit_jump` の幅**: `emit_jump(opcode)` が `opcode(1) + offset(2) = 3 bytes` を emit するなら、追加の `stage_idx(1) + total(1)` は `emit_u8` で別途追加。escape パッチ時の offset 計算がズレないよう注意。
