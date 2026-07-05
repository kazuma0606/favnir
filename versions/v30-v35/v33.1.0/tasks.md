# v33.1.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `33.0.0` であること
- [x] `benchmarks/v33.0.0.json` の `tests_passed` が 2496 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2496 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v331000_tests` が存在しないこと
- [x] v33.0.0 が COMPLETE であること
- [x] `cargo_toml_version_is_33_0_0` が v330000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v330000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] `fav/src/backend/cranelift_aot.rs` に `CraneliftBackend` が存在すること
- [x] `v192000_tests` のテスト名（`build_target_native_produces_binary` / `native_binary_executes` / `native_vs_vm_same_output` / `build_target_vm_still_works`）と v331000_tests のテスト名が重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `33.0.0` → `33.1.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_33_0_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v331000_tests`（4 件）を追加
       挿入位置: `v330000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use super::cmd_build_native;` を明示 import
- [x] **T4** `CHANGELOG.md` — `[v33.1.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v33.1.0.json` — 新規作成（暫定値 2500、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v33.1.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v331000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2500 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v33.1.0.json` の `tests_passed` を実測値（2500）で更新
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"33.1.0"`
- [x] `cargo_toml_version_is_33_0_0` が空スタブになっていること
- [x] `cargo test --bin fav v331000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2500 件、0 failures）
- [x] `CHANGELOG.md` に `[v33.1.0]` セクション
- [x] `benchmarks/v33.1.0.json` 存在かつ `tests_passed` が実測値（2500）
- [x] `benchmarks/v33.1.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- [x] `versions/current.md` を v33.1.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v331000_tests` に `use super::*` が**ない**こと（`use super::cmd_build_native;` のみ）
- [x] `cargo_toml_version_is_33_0_0` が空スタブになっていること（コメント付き）
- [x] `aot_if_branch_selects_true_arm` / `aot_bool_comparison_native` が v192000_tests のテスト名と異なること
- [x] cc 非インストール環境で `if !cc_available() { return; }` によりスキップされること
- [x] 挿入位置が `v330000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v33.1.0.json` の `milestone` が `"Performance & Tooling"` であること
- [x] `versions/current.md` が v33.1.0 に更新されていること
