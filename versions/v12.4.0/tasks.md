# Favnir v12.4.0 Tasks

Date: 2026-06-07
Theme: `seq` pipeline fail-fast（`SeqStageCheck` opcode）

---

## Phase A — 現状把握

- [x] A-1: `fav/src/middle/compiler.rs` の `compile_flw_def`（行 580〜663）を精読
  - `IRExpr::Block(Vec<IRStmt>, Box<IRExpr>, Type)` が既存であることを確認
  - 現在の body = nested `IRExpr::Call` 構造を確認
- [x] A-2: `backend/codegen.rs` の `emit_expr` で `IRExpr::Block` の codegen を確認
- [x] A-3: `backend/vm.rs` の `LegacyBindCheck`（0x35）ハンドラと `chain_escapes` パッチ機構を確認

---

## Phase B — IRStmt::SeqChain 追加

- [x] B-1: `middle/ir.rs` に `IRStmt::SeqChain { slot: u16, expr: IRExpr, stage_name: String, stage_idx: u8, total: u8 }` 追加
- [x] B-2: `middle/ir.rs::collect_stmt_deps` に `SeqChain` ケース追加

---

## Phase C — compile_flw_def を Block スタイルに修正

- [x] C-1: `middle/compiler.rs::compile_flw_def` を修正
  - 2 ステージ以上: `IRExpr::Block(SeqChain stmts, final_call)` を生成
  - 単一ステージ: 従来通り `IRExpr::Call` のまま（変更なし）
  - `flw_step_name` / `build_step_call` ヘルパー関数追加

---

## Phase D — Opcode::SeqStageCheck 追加 + emit_stmt

- [x] D-1: `backend/codegen.rs` に `Opcode::SeqStageCheck = 0x36` 追加
- [x] D-2: `emit_seq_stage_jump(name_str_idx, stage_idx, total)` メソッド追加
- [x] D-3: `emit_stmt` に `IRStmt::SeqChain` ケース追加（chain_escapes に escape_offset を push）
- [x] D-4: `remap_string_operands` に `SeqStageCheck → remap_u16_at(ip+1); ip+=7` 追加
         + `LegacyBindCheck → ip+=3` バグ修正（v12.3.0 の抜け）

---

## Phase E — VM: SeqStageCheck ハンドラ追加

- [x] E-1: `backend/vm.rs` に `SeqStageCheck` ハンドラ追加
  - `ok(v)` / `some(v)` → unwrap して push、続行
  - `err(e)` / `none` → `"pipeline stopped at stage I/N 'name': e"` に wrap → push → escape
  - その他 → pass-through（そのまま push、続行）

---

## Phase F — 全 IRStmt パターンマッチ更新

- [x] F-1: `driver.rs::collect_tracklines_in_expr` に `SeqChain` 追加（expr を再帰）
- [x] F-2: `driver.rs::remap_ir_stmt` に `SeqChain` 追加（expr を remap）
- [x] F-3: `driver.rs::opcode_info` に `SeqStageCheck => ("SeqStageCheck", 7)` 追加
- [x] F-4: `backend/wasm_codegen.rs` — 全 5 箇所に `SeqChain` 追加（plain Bind に降格）
- [x] F-5: `driver.rs::legacy_transform_stmt` — `SeqChain` を追加（expr のみ再帰変換）
- [x] F-6: `cargo build` でコンパイルエラーがないことを確認

---

## Phase G — v12400_tests モジュール追加

- [x] G-1: 正常系テスト（2件）
  - [x] `seq_passes_ok_through` — `ok(v)` を返す 2 ステージ seq → v が unwrap されて最終結果
  - [x] `seq_plain_value_passes_through` — plain String を返す 2 ステージ seq → pass-through 動作確認
- [x] G-2: 短絡系テスト（3件）
  - [x] `seq_stops_on_stage_err` — Stage 1 が `err("fail")` → Stage 2 は実行されない
  - [x] `seq_error_includes_stage_name` — `err("db error")` → `Err("pipeline stopped at stage 1/2 'LoadData': db error")`
  - [x] `seq_error_at_middle_stage` — 3 ステージ中 Stage 2 が Err → Stage 3 未実行、`stage 2/3` がエラーに含まれる
- [x] G-3: 後方互換確認（1件）
  - [x] `seq_legacy_mode_fail_fast` — `--legacy` モードでも seq fail-fast が有効なこと
- [x] G-4: バージョン確認（1件）
  - [x] `version_is_12_4_0` — `CARGO_PKG_VERSION == "12.4.0"`
- [x] G-5: `v12300_tests` モジュールから `version_is_12_3_0` テストを削除
- [x] G-6: `cargo test v12400 -- --nocapture` — 7 件通過確認

---

## Phase H — 全テスト通過確認

- [x] H-1: `cargo test` — 1376 件全通過

---

## Phase I — バージョン更新 + コミット

- [x] I-1: `fav/Cargo.toml` version → `"12.4.0"`
- [x] I-2: `cargo build` で `Cargo.lock` 更新
- [x] I-3: `git commit -m "feat: v12.4.0 — seq pipeline fail-fast (SeqStageCheck opcode)"`
- [x] I-4: `git push`

---

## Phase J — CI 修正（rune-registry Lambda）

- [x] J-1: Lambda ログで `RuntimeError: non-exhaustive match` を確認
- [x] J-2: 原因特定：v12.3.0 `--legacy` monadic bind 変更により `main.fav` の
         `bind x_r <- Result_fn(); match x_r { ... }` パターンが全滅
- [x] J-3: `rune-registry/src/main.fav` を修正
         — `bind x_r <- fn(); match x_r { ... }` → `match fn() { ... }` にインライン化（全箇所）
         — `bind x_r <- fn(); bind x <- match x_r { Err(_) => d Ok(v) => v }` → `bind x <- match fn() { ... }`
- [ ] J-4: `git commit -m "fix: rune-registry main.fav — monadic bind compatibility (v12.3.0)"` + push
- [ ] J-5: deploy-registry → CI Site build 通過確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `seq A \|> B` で A が `err` → B は実行されない | ✅ |
| `seq A \|> B` で A が `ok(v)` → B が `v` を受け取る（unwrap） | ✅ |
| `seq A \|> B` で A が plain String → B がそのまま受け取る（pass-through） | ✅ |
| エラーに `stage N/M 'StageName'` が含まれる | ✅ |
| `cargo test v12400` 7 件通過 | ✅ |
| `cargo test` 全通過（1376） | ✅ |
| CI Site build 通過 | ⏳ J-4/J-5 待ち |
