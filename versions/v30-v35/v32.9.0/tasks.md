# v32.9.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `32.8.0` であること
- [x] `benchmarks/v32.8.0.json` の `tests_passed` が 2488 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2488 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v329000_tests` が存在しないこと
- [x] v32.8.0 が COMPLETE であること
- [x] `pub type EffectSet` / `pub fn infer_effects_fn` / `pub fn infer_effects_for_program` が checker.rs に存在すること（v18.1.0 実装済み）
- [x] `Effect::Io` が ast.rs の Effect enum に存在すること
- [x] `cargo_toml_version_is_32_8_0` が v328000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v328000` が 4/4 PASS であること（前バージョン 4 件アクティブ PASS を確認）
- [x] `cargo test --bin fav v181000` が PASS であること（`effect_inference_db`・`effect_inference_pure`・`effect_inference_transitive` 含む 4 件 PASS を確認）
- [x] テスト名が v181000_tests（`effect_inference_db` / `effect_inference_multi` / `effect_inference_pure` / `effect_inference_transitive`）と重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `32.8.0` → `32.9.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_32_8_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v329000_tests`（4 件）を追加
       挿入位置: `v328000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::ast::{Effect, Item}; use crate::middle::checker::infer_effects_fn; use crate::frontend::parser::Parser;` を使用
- [x] **T4** `CHANGELOG.md` — `[v32.9.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v32.9.0.json` — 新規作成（暫定値 2492、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v32.9.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v329000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2492 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v32.9.0.json` の `tests_passed` を実測値で更新（2492 — 暫定値と一致）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"32.9.0"`
- [x] `cargo_toml_version_is_32_8_0` が空スタブになっていること
- [x] `effect_infer_io_println` テストが PASS（`!Io` 推論）
- [x] `effect_infer_pure_mul_no_effects` テストが PASS（エフェクトなし）
- [x] `cargo test --bin fav v329000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2492 件、0 failures）
- [x] `CHANGELOG.md` に `[v32.9.0]` セクション
- [x] `benchmarks/v32.9.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v32.9.0.json` の `milestone` フィールドが `"Language Power"` であること
- [x] `versions/current.md` を v32.9.0 に更新
- [x] `tasks.md` が COMPLETE
- [x] site/ MDX 更新: 対象外（エフェクト推論は v18.1.0 で完成済み）

---

## コードレビューチェックリスト

- [x] `v329000_tests` に `use super::*` が**ない**こと（`use crate::...` で完結）
- [x] `cargo_toml_version_is_32_8_0` が空スタブになっていること（コメント付き）
- [x] `effect_infer_io_println` が v181000_tests のテスト名と異なること
- [x] `effect_infer_pure_mul_no_effects` が v181000_tests のテスト名と異なること
- [x] テスト 3 が `IO.println` → `!Io` 推論を検証していること（v181000_tests は Postgres.query_raw → !Postgres）
- [x] テスト 4 が 純粋関数 `mul` → エフェクトなしを検証していること（v181000_tests は `add`）
- [x] 挿入位置が `v328000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
- [x] `benchmarks/v32.9.0.json` の `milestone` が `"Language Power"` であること
- [x] `versions/current.md` が v32.9.0 に更新されていること
