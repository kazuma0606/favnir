# v33.4.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `33.3.0` であること
- [x] `benchmarks/v33.3.0.json` の `tests_passed` が 2508 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2508 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v334000_tests` が存在しないこと
- [x] v33.3.0 が COMPLETE であること
- [x] `cargo_toml_version_is_33_3_0` が v333000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v333000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] `fav/src/ast.rs` に `TrfDef.arrow: bool` および `TrfDef.stateful: bool` フィールドが存在すること
- [x] `v195000_tests` のテスト名（`arrow_batch_from_list` / `arrow_batch_to_list` / `arrow_parquet_roundtrip` / `arrow_stage_executes`）と v334000_tests のテスト名が重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `33.3.0` → `33.4.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_33_3_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v334000_tests`（4 件）を追加
       挿入位置: `v333000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::frontend::parser::Parser;` を明示 import
- [x] **T4** `CHANGELOG.md` — `[v33.4.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v33.4.0.json` — 新規作成（暫定値 2512、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v33.4.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v334000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2512 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v33.4.0.json` の `tests_passed` を実測値で更新（2512 確定）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"33.4.0"`
- [x] `cargo_toml_version_is_33_3_0` が空スタブになっていること
- [x] `cargo test --bin fav v334000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2512 件、0 failures）
- [x] `CHANGELOG.md` に `[v33.4.0]` セクション
- [x] `benchmarks/v33.4.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v33.4.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- [x] `versions/current.md` を v33.4.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v334000_tests` に `use super::*` が**ない**こと（`use crate::frontend::parser::Parser;` のみ）
- [x] `cargo_toml_version_is_33_3_0` が空スタブになっていること（コメント付き）
- [x] `arrow_trf_without_annotation_has_false` / `arrow_trf_arrow_and_stateful_are_independent` が v195000_tests のテスト名と異なること
- [x] `arrow_trf_without_annotation_has_false` が `prog.items.len()` 確認後に `prog.items[0]` アクセスし、`!trf.arrow` を assert していること
- [x] `arrow_trf_arrow_and_stateful_are_independent` が `prog.items.len()` 確認後に `prog.items[0]` アクセスし、`trf.stateful == true` かつ `trf.arrow == false` を assert していること
- [x] 両テストに `assert_eq!(prog.items.len(), 1, "expected 1 item")` が存在すること（境界チェック、spec-reviewer [MED] 教訓）
- [x] 挿入位置が `v333000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v33.4.0.json` の `milestone` が `"Performance & Tooling"` であること
- [x] `versions/current.md` が v33.4.0 に更新されていること
