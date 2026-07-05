# v33.8.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `33.7.0` であること
- [x] `benchmarks/v33.7.0.json` の `tests_passed` が 2524 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2524 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v338000_tests` が存在しないこと
- [x] v33.7.0 が COMPLETE であること
- [x] `cargo_toml_version_is_33_7_0` が v337000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v337000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] `fav/src/driver.rs` に profiler モジュール（`profiler::collector::parse_profile_json` / `to_folded_stacks`）が存在すること（v19.8.0 実装確認）
- [x] `v198000_tests` のテスト名（`profile_flamegraph_generates_svg` / `profile_text_output` / `profile_json_output` / `profile_hot_path_detected`）と v338000_tests のテスト名が重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `33.7.0` → `33.8.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_33_7_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v338000_tests`（4 件）を追加
       挿入位置: `v337000_tests` 直後・`// ── v31.7.0 tests` の前
       `#[cfg(test)]` → `#[cfg(not(target_arch = "wasm32"))]` の順で mod の上に付与（v198000_tests スタイル準拠）
       `use super::*` なし、`use crate::profiler::collector::{StageRecord, parse_profile_json, to_folded_stacks};` を明示 import
- [x] **T4** `CHANGELOG.md` — `[v33.8.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v33.8.0.json` — 新規作成（暫定値 2528、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v33.8.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v338000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2528 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v33.8.0.json` の `tests_passed` を実測値で更新（2528 確定）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"33.8.0"`
- [x] `cargo_toml_version_is_33_7_0` が空スタブになっていること
- [x] `cargo test --bin fav v338000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2528 件、0 failures）
- [x] `CHANGELOG.md` に `[v33.8.0]` セクション
- [x] `benchmarks/v33.8.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v33.8.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- [x] `versions/current.md` を v33.8.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v338000_tests` に `use super::*` が**ない**こと
- [x] `#[cfg(test)]` → `#[cfg(not(target_arch = "wasm32"))]` の順で `mod v338000_tests` の上に付与されていること
- [x] `cargo_toml_version_is_33_7_0` が空スタブになっていること（コメント付き）
- [x] `profile_parse_json_valid_records` で JSON キーが `"name"` / `"ms"` であること、`parse_profile_json` の戻り値を `.expect()` せず直接代入していること、`records[0].name == "Load"` / `records[0].elapsed_ms == 10` / `records[1].name == "Transform"` / `records[1].elapsed_ms == 25` を assert していること
- [x] `profile_folded_stacks_has_pipeline_prefix` で `StageRecord { name: ..., elapsed_ms: ... }` を使い、`folded.iter().all(|line| line.starts_with("pipeline;"))` を assert していること
- [x] v198000_tests のテスト名と重複しないこと
- [x] 挿入位置が `v337000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v33.8.0.json` の `milestone` が `"Performance & Tooling"` であること
- [x] `versions/current.md` が v33.8.0 に更新されていること
