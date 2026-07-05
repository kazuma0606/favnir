# v33.9.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `33.8.0` であること
- [x] `benchmarks/v33.8.0.json` の `tests_passed` が 2528 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2528 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v339000_tests` が存在しないこと
- [x] v33.8.0 が COMPLETE であること
- [x] `cargo_toml_version_is_33_8_0` が v338000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v338000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] `fav/src/parallel/compiler.rs` に `compile_parallel` が存在すること（v19.4.0 実装確認）
- [x] `fav/src/parallel/topo.rs` に `topo_layers` が存在すること（v19.4.0 実装確認）
- [x] `v194000_tests` のテスト名（`parallel_compile_same_output` / `parallel_compile_faster` / `parallel_dep_order_respected` / `parallel_compile_thread_count`）と v339000_tests のテスト名が重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `33.8.0` → `33.9.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_33_8_0` をスタブ化（v338000_tests は `#[cfg(not(target_arch = "wasm32"))]` ゲートを持つ。スタブ化してもゲートは外さないこと）
- [x] **T3** `fav/src/driver.rs` — `v339000_tests`（4 件）を追加
       挿入位置: `v338000_tests` 直後・`// ── v31.7.0 tests` の前
       `#[cfg(not(target_arch = "wasm32"))]` → `#[cfg(test)]` の順で付与（`parallel` モジュールは `lib.rs` で WASM ゲート済み）
       `use super::*` なし、`use crate::parallel::{compiler::compile_parallel, topo::topo_layers};` / `use crate::incremental::dep_graph::DepGraph;` を明示 import
- [x] **T4** `CHANGELOG.md` — `[v33.9.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v33.9.0.json` — 新規作成（暫定値 2532、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v33.9.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v339000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2532 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v33.9.0.json` の `tests_passed` を実測値で更新（2532 確定）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"33.9.0"`
- [x] `cargo_toml_version_is_33_8_0` が空スタブになっていること
- [x] `cargo test --bin fav v339000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2532 件、0 failures）
- [x] `CHANGELOG.md` に `[v33.9.0]` セクション
- [x] `benchmarks/v33.9.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v33.9.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- [x] `versions/current.md` を v33.9.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v339000_tests` に `use super::*` が**ない**こと
- [x] `#[cfg(not(target_arch = "wasm32"))]` → `#[cfg(test)]` の順で `mod v339000_tests` の上に付与されていること
- [x] `cargo_toml_version_is_33_8_0` が空スタブになっていること（コメント付き）
- [x] `parallel_topo_cyclic_dep_returns_err` で `add_dep("a","b")` + `add_dep("b","a")` を設定し、`result.is_err()` かつ `msg.contains("circular")` を assert していること
- [x] `parallel_compile_empty_sources` で `compile_parallel(vec![], 1)` が `Ok` を返し `ir.fns.len() == 0` を assert していること
- [x] v194000_tests のテスト名と重複しないこと
- [x] 挿入位置が `v338000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v33.9.0.json` の `milestone` が `"Performance & Tooling"` であること
- [x] `versions/current.md` が v33.9.0 に更新されていること
