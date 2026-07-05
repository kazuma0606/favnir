# v32.2.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `32.1.0` であること
- [x] `benchmarks/v32.1.0.json` の `tests_passed` が 2460 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2460 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v322000_tests` が存在しないこと
- [x] v32.1.0 が COMPLETE であること
- [x] `type_has_field`（checker.rs:7917 付近）が Named 型のフィールド存在チェックを行っていること
- [x] E0337 が `TypeConstraint::HasField` の違反時に発生すること（checker.rs:4805 付近）
- [x] `cargo_toml_version_is_32_1_0` が v321000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v321000` が 4/4 PASS であること（`check_errors` パターンの動作確認）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `32.1.0` → `32.2.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_32_1_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v322000_tests`（4 件 + ローカル `check_errors`）を追加
       挿入位置: `v321000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::frontend::parser::Parser` 等を使用
- [x] **T4** `CHANGELOG.md` — `[v32.2.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v32.2.0.json` — 新規作成（暫定値 2464、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v32.2.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v322000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2464 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v32.2.0.json` の `tests_passed` を実測値で更新（2464 — 暫定値と一致）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"32.2.0"`
- [x] `cargo_toml_version_is_32_1_0` が空スタブになっていること
- [x] `row_poly_field_constraint_pass` テストが PASS
- [x] `row_poly_missing_field_e0337` テストが PASS（E0337 確認）
- [x] `cargo test --bin fav v322000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `CHANGELOG.md` に `[v32.2.0]` セクション
- [x] `benchmarks/v32.2.0.json` 存在かつ `tests_passed` が実測値
- [x] `versions/current.md` を v32.2.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v322000_tests` に `use super::*` が**ない**こと（`use crate::...` で完結）
- [x] `v322000_tests` 内にローカル `check_errors` が定義されていること
- [x] `cargo_toml_version_is_32_1_0` が空スタブになっていること（コメント付き）
- [x] テスト 3 が `with { id: Int }` 制約の PASS ケースを検証していること
- [x] テスト 4 が E0337 のネガティブケースを検証していること
- [x] 挿入位置が `v321000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
- [x] `benchmarks/v32.2.0.json` の `milestone` が `"Language Power"` であること
