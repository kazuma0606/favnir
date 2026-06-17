# v19.6.0 — WASM バイナリ最適化 タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `fav/src/backend/wasm_dce.rs`（新規）— Dead Code Elimination

- [x] `collect_reachable_fns(ir: &IRProgram, entry: &str) -> HashSet<usize>` を実装:
  - `entry` 名（通常 `"main"`）から `ir.globals` を走査して fn インデックスを解決
  - BFS でコールグラフを辿り、到達可能な fn インデックスを収集
  - `IRExpr::Global(idx)` / `IRExpr::TrfRef(idx)` / `IRExpr::Closure { global_idx }` を処理

- [x] `apply_dce(ir: &mut IRProgram, reachable: &HashSet<usize>) -> DceReport` を実装:
  - 到達可能な fn のみを含む新しい `ir.fns` を構築
  - `old_idx → new_idx` の remap map を作成
  - `ir.globals` の `IRGlobalKind::Fn(idx)` を remap
  - 到達不可能な Fn グローバルを `ir.globals` から除去
  - `DceReport { removed: usize, remaining: usize }` を返す

- [x] `DceReport` struct を定義（`#[derive(Debug, Clone)]`）

- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T2: `fav/src/backend/wasm_opt_pass.rs`（新規）— wasm-opt 統合・サイズ計測

- [x] `WasmOptLevel` enum を定義（`O0 / O1 / O2 / O3`）
- [x] `WasmSizeReport { before: usize, after: usize }` を定義
- [x] `WasmOptError { NotInstalled, ExitNonZero(i32), Io(String) }` enum を定義
- [x] `run_wasm_opt(bytes, level, strip_debug)` を実装
- [x] `try_wasm_opt(bytes, level, strip_debug)` を実装（graceful fallback）
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T3: `fav/src/backend/mod.rs` — モジュール追加

- [x] `pub mod wasm_dce; pub mod wasm_opt_pass;` 追加
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T4: `fav/src/backend/wasm_codegen.rs` — `wasm_codegen_program_wasi` 追加

- [x] `wasm_codegen_program_wasi(ir)` 実装（`_start` エクスポート付き）
- [x] `add_wasi_start_export(bytes, main_wasm_idx)` — バイナリパッチ実装
- [x] `leb128_write_u32` / `leb128_read_u32` ヘルパー追加
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T5: `fav/src/driver.rs` — `WasmBuildConfig` / `build_wasm_artifact_with_config` 追加

- [x] `WasmTarget` enum 追加（`Wasm32 / Wasm32Wasi`）
- [x] `WasmBuildConfig` struct 追加（target/opt_level/strip_debug/size_report/dce）
- [x] `build_wasm_artifact_with_config` 実装（DCE + wasm-opt + target 統合）
- [x] `cargo build` でコンパイルエラーが 0 であることを確認

---

### T6: `fav/src/driver.rs` — `v196000_tests` 追加（5件）

- [x] `v196000_tests` モジュール追加（5テスト）
- [x] `v195000_tests::version_is_19_5_0` に `#[ignore]` 追加
- [x] `cargo test v196000` — 5/5 PASS 確認

---

### T7: `fav/Cargo.toml` — バージョン更新

- [x] `version = "19.5.0"` → `"19.6.0"` に変更

---

### T8: `site/content/docs/tools/wasm-opt.mdx`（新規）

- [x] WASM 最適化の概要（DCE + wasm-opt）
- [x] `fav build --target wasm --wasm-opt=O3` の使い方
- [x] `fav build --target wasm32-wasi` と `wasmtime` での実行方法
- [x] wasm-opt のインストール（`brew install binaryen` / `apt install binaryen`）
- [x] サイズレポートの読み方

---

## テスト（v196000_tests、5件）

| テスト名 | 内容 | 結果 |
|---|---|---|
| `version_is_19_6_0` | Cargo.toml に `"19.6.0"` が含まれる | PASS |
| `wasm_dce_reduces_fn_count` | 未使用関数を含む IR に DCE を適用し、`ir.fns.len()` が減少する | PASS |
| `wasm_size_report_computes` | `WasmSizeReport { before: 1000, after: 600 }.reduction_pct()` == 40.0 | PASS |
| `wasm_output_correct` | DCE 付き WASM ビルド → `wasm_exec_main()` が成功する | PASS |
| `wasm_wasi_target_builds` | `WasmTarget::Wasm32Wasi` で `_start` エクスポート付き WASM が生成される | PASS |

---

## 完了条件チェックリスト

- [x] `src/backend/wasm_dce.rs` が存在し、`collect_reachable_fns` / `apply_dce` が動作する
- [x] `src/backend/wasm_opt_pass.rs` が存在し、`WasmSizeReport::reduction_pct` が正確
- [x] `wasm_codegen_program_wasi` が `_start` エクスポートを含む WASM を生成する
- [x] `build_wasm_artifact_with_config` が DCE + wasm-opt + target を統合して動作する
- [x] `wasm-opt` 未インストール環境でも全テスト PASS（graceful fallback）
- [x] `cargo test v196000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし（既存 WASM テストも PASS）
- [x] `site/content/docs/tools/wasm-opt.mdx` が存在する
