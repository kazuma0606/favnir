# v32.3.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `32.2.0` であること
- [x] `benchmarks/v32.2.0.json` の `tests_passed` が 2464 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2464 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v323000_tests` が存在しないこと
- [x] v32.2.0 が COMPLETE であること
- [x] `fn_refinement_registry`（checker.rs:982 付近）が関数の `where` 制約を登録していること
- [x] E0331 が checker.rs:4820 付近（実発行は 4838）でリテラル違反時に発行されること
- [x] `cargo_toml_version_is_32_2_0` が v322000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v322000` が 4/4 PASS であること（`check_errors` パターンの動作確認）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `32.2.0` → `32.3.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_32_2_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v323000_tests`（4 件 + ローカル `check_errors`）を追加
       挿入位置: `v322000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::frontend::parser::Parser` 等を使用
- [x] **T4** `CHANGELOG.md` — `[v32.3.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v32.3.0.json` — 新規作成（暫定値 2468、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v32.3.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v323000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2468 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v32.3.0.json` の `tests_passed` を実測値で更新（2468 — 暫定値と一致）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"32.3.0"`
- [x] `cargo_toml_version_is_32_2_0` が空スタブになっていること
- [x] `where_constraint_literal_pass` テストが PASS
- [x] `where_constraint_literal_fail_e0331` テストが PASS（E0331 確認）
- [x] `cargo test --bin fav v323000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `CHANGELOG.md` に `[v32.3.0]` セクション
- [x] `benchmarks/v32.3.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v32.3.0.json` の `milestone` フィールドが `"Language Power"` であること
- [x] `versions/current.md` を v32.3.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v323000_tests` に `use super::*` が**ない**こと（`use crate::...` で完結）
- [x] `v323000_tests` 内にローカル `check_errors` が定義されていること
- [x] `cargo_toml_version_is_32_2_0` が空スタブになっていること（コメント付き）
- [x] テスト 3 が `b=2` で `b != 0` 制約の PASS ケースを検証していること
- [x] テスト 4 が `b=0` で E0331 のネガティブケースを検証していること
- [x] 挿入位置が `v322000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
- [x] `benchmarks/v32.3.0.json` の `milestone` が `"Language Power"` であること
