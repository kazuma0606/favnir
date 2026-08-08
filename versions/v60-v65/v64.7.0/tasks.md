# v64.7.0 タスクリスト

Status: COMPLETE
Version: 64.7.0
Base tests: 3443
Target tests: 3445

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3443 tests passed, 0 failed を確認
- [x] `driver.rs` に `v64700_tests` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `v64600_tests` が存在することを確認（`v64700_tests` の挿入位置）
- [x] `driver.rs` に `cmd_build_wasm` が存在しないことを確認（新規追加）
- [x] `driver.rs` に `cmd_build_ci` が存在することを確認（`cmd_build_wasm` の挿入位置）
- [x] `Cargo.toml` に `wasm-encoder` が登録済みであることを確認
- [x] `backend/wasm_codegen.rs` に `wasm_codegen_program(ir: &IRProgram) -> Result<Vec<u8>, WasmCodegenError>` が存在することを確認
- [x] `WasmCodegenError` の `Display` impl を確認（`{e}` で文字列化）

**スコープ注記**: `cmd_build` への `"wasm32"` アーム統合と site/ MDX 追記は後送り（v64.9 以降）

---

## T1: `driver.rs` — `cmd_build_wasm` 追加

- [x] `cmd_build_ci` の直後に `cmd_build_wasm(src: &str, out: &str) -> String` を追加
  - [x] 完全修飾パス `crate::backend::wasm_codegen::wasm_codegen_program` を使用（`use` 宣言不要）
  - [x] `Parser::parse_str` でパース（失敗時 `"wasm: error: parse error: ..."` を返す）
  - [x] `compile_program(&program)` で IR 生成
  - [x] `wasm_codegen_program(&ir)` で WASM バイト列生成
  - [x] `bytes.is_empty()` ガード（`"wasm: error: empty output"`）
  - [x] 成功: `"Compiling (target: wasm32)...\nOutput: {out} (WASM module, {N} bytes)"` を返す
  - [x] codegen エラー: `"wasm: error: codegen error: {e}"` を返す

---

## T2: `driver.rs` — `v64700_tests` 追加

- [x] `// -- v64600_tests` コメント行の直前に `v64700_tests` を挿入
  - [x] `build_wasm_target_outputs_wasm`（`"wasm: error:"` で始まらない・"wasm32"/"WASM"/"wasm" を含む）
  - [x] `wasm_build_compat_check`（`"wasm: error:"` で始まらない・"bytes"/"WASM" を含む）
  - [x] テストソースに `public fn main() -> Unit { ... }` を含める（`wasm_codegen_program` 要件）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v64700_tests` で 2 件 PASS
  - [x] `build_wasm_target_outputs_wasm` PASS
  - [x] `wasm_build_compat_check` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3445 tests passed, 0 failed を確認

---

## T4: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v64.7.0 エントリを追加
- [x] `versions/roadmap/roadmap-v64.1-v65.0.md` v64.7.0 セクションに実績追記（3445 tests、`cmd_build` 統合は後送り（v64.9 以降）を明記）
- [x] `versions/current.md` の「進行中」を v64.7.0（3445 tests）に更新
- [x] `MILESTONE.md` は v65.0 で更新（本バージョンでは不要）
- [x] tasks.md を COMPLETE に更新（本ファイル）

**注記**: site/ MDX への `--target wasm32` 説明追記は `cmd_build` 統合完了後（v64.9 以降）に対応

## コードレビュー対応

- [MED] CLI dispatch 未接続 → CHANGELOG に「main.rs dispatch 未接続・v64.9 で有効化」を明記
- [MED] 型チェックスキップ → cmd_build_ci と同一パターンであるためコメントで明示（変更なし）
- [LOW] テストアサーション弱い → `contains("Compiling (target: wasm32)")` / `contains("WASM module,")` に強化
- [LOW] ファイル書き出しなし → cmd_build_wasm のコメントに「バイト列書き出しは v64.9 以降」を明記
