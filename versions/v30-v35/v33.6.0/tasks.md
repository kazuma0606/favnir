# v33.6.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `33.5.0` であること
- [x] `benchmarks/v33.5.0.json` の `tests_passed` が 2516 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2516 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v336000_tests` が存在しないこと
- [x] v33.5.0 が COMPLETE であること
- [x] `cargo_toml_version_is_33_5_0` が v335000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v335000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] `fav/src/backend/wasm_dce.rs` に `collect_reachable_fns` / `apply_dce` が存在すること（v19.6.0 実装確認）
- [x] `fav/src/driver.rs` に `WasmBuildConfig` / `WasmTarget` が存在すること
- [x] `fav/src/backend/wasm_opt_pass.rs` に `WasmOptLevel` が存在すること
- [x] `v196000_tests` のテスト名（`wasm_dce_reduces_fn_count` / `wasm_size_report_computes` / `wasm_output_correct` / `wasm_wasi_target_builds`）と v336000_tests のテスト名が重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `33.5.0` → `33.6.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_33_5_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v336000_tests`（4 件）を追加
       挿入位置: `v335000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、必要な import のみ明示
- [x] **T4** `CHANGELOG.md` — `[v33.6.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v33.6.0.json` — 新規作成（暫定値 2520、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v33.6.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v336000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2520 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v33.6.0.json` の `tests_passed` を実測値で更新（2520 確定）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"33.6.0"`
- [x] `cargo_toml_version_is_33_5_0` が空スタブになっていること
- [x] `cargo test --bin fav v336000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2520 件、0 failures）
- [x] `CHANGELOG.md` に `[v33.6.0]` セクション
- [x] `benchmarks/v33.6.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v33.6.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- [x] `versions/current.md` を v33.6.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v336000_tests` に `use super::*` が**ない**こと
- [x] `cargo_toml_version_is_33_5_0` が空スタブになっていること（コメント付き）
- [x] `wasm_dce_keeps_reachable_fn` / `wasm_default_config_is_o0_with_dce` が v196000_tests のテスト名と異なること
- [x] `wasm_dce_keeps_reachable_fn` が DCE 後に `ir.fns.iter().any(|f| f.name.contains("helper"))` で到達可能関数の保持を確認していること
- [x] `wasm_default_config_is_o0_with_dce` が `opt_level: O0` / `dce: true` / `strip_debug: false` / `target: Wasm32` をすべて assert していること
- [x] 挿入位置が `v335000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v33.6.0.json` の `milestone` が `"Performance & Tooling"` であること
- [x] `versions/current.md` が v33.6.0 に更新されていること
