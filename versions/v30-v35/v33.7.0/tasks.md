# v33.7.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `33.6.0` であること
- [x] `benchmarks/v33.6.0.json` の `tests_passed` が 2520 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2520 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v337000_tests` が存在しないこと
- [x] v33.6.0 が COMPLETE であること
- [x] `cargo_toml_version_is_33_6_0` が v336000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v336000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] `fav/src/driver.rs` に `migrate_effects_in_source` / `resolve_use_effects` が `pub fn` として存在すること（v13.10.0 実装確認）
- [x] `v13100_tests` のテスト名（`e0025_bang_notation_error` / `fmt_migrate_postgres_to_load_ctx` / `fmt_migrate_appctx_with_w010` / `ctx_destructure_sugar_parses` / `ctx_destructure_io_only` / `migrate_tool_scans_directory`）と v337000_tests のテスト名が重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `33.6.0` → `33.7.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_33_6_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v337000_tests`（4 件）を追加
       挿入位置: `v336000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::driver::{migrate_effects_in_source, resolve_use_effects};` を明示 import
- [x] **T4** `CHANGELOG.md` — `[v33.7.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v33.7.0.json` — 新規作成（暫定値 2524、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v33.7.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v337000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2524 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v33.7.0.json` の `tests_passed` を実測値で更新（2524 確定）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"33.7.0"`
- [x] `cargo_toml_version_is_33_6_0` が空スタブになっていること
- [x] `cargo test --bin fav v337000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2524 件、0 failures）
- [x] `CHANGELOG.md` に `[v33.7.0]` セクション
- [x] `benchmarks/v33.7.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v33.7.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- [x] `versions/current.md` を v33.7.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v337000_tests` に `use super::*` が**ない**こと
- [x] `cargo_toml_version_is_33_6_0` が空スタブになっていること（コメント付き）
- [x] `migrate_effects_idempotent` / `resolve_use_effects_from_v13` が v13100_tests のテスト名と異なること
- [x] `migrate_effects_idempotent` で 2 回目の `migrate_effects_in_source` 呼び出し後に `first == second` かつ `w010s.is_empty()` を assert していること
- [x] `resolve_use_effects_from_v13` で `Some("v13")` / `Some("13")` が `true`、`Some("v12")` / `None` が `false` を assert していること
- [x] 挿入位置が `v336000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v33.7.0.json` の `milestone` が `"Performance & Tooling"` であること
- [x] `versions/current.md` が v33.7.0 に更新されていること
