# v32.7.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `32.6.0` であること
- [x] `benchmarks/v32.6.0.json` の `tests_passed` が 2480 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2480 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v327000_tests` が存在しないこと
- [x] v32.6.0 が COMPLETE であること
- [x] `GenericParam.is_const` / `const_ty` / `const_constraint` が ast.rs に存在すること（v18.7.0 実装済み）
- [x] E0335 が checker.rs に存在すること（"E0335" 文字列の確認）
- [x] `cargo_toml_version_is_32_6_0` が v326000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v326000` が 4/4 PASS であること（前バージョンテスト動作確認）
- [x] `cargo test --bin fav v187000` が PASS であること（`const_generic_violation`・`const_generic_valid` 含む 4 件アクティブ PASS を確認）
- [x] テスト名が v187000_tests（`const_generic_parses` / `const_generic_constraint_parses` / `const_generic_violation` / `const_generic_valid`）と重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `32.6.0` → `32.7.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_32_6_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v327000_tests`（4 件 + ローカル `check_errors`）を追加
       挿入位置: `v326000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::frontend::parser::Parser` 等を使用
- [x] **T4** `CHANGELOG.md` — `[v32.7.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v32.7.0.json` — 新規作成（暫定値 2484、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v32.7.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v327000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2484 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v32.7.0.json` の `tests_passed` を実測値で更新（2484 — 暫定値と一致）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"32.7.0"`
- [x] `cargo_toml_version_is_32_6_0` が空スタブになっていること
- [x] `const_gen_chunk_size_valid` テストが PASS（E0335 なし）
- [x] `const_gen_chunk_size_zero_e0335` テストが PASS（E0335 確認）
- [x] `cargo test --bin fav v327000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2484 件、0 failures）
- [x] `CHANGELOG.md` に `[v32.7.0]` セクション
- [x] `benchmarks/v32.7.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v32.7.0.json` の `milestone` フィールドが `"Language Power"` であること
- [x] `versions/current.md` を v32.7.0 に更新
- [x] `tasks.md` が COMPLETE
- [x] site/ MDX 更新: 対象外（const generics は v18.7.0 で完成済み）

---

## コードレビューチェックリスト

- [x] `v327000_tests` に `use super::*` が**ない**こと（`use crate::...` で完結）
- [x] `v327000_tests` 内にローカル `check_errors` が定義されていること
- [x] `cargo_toml_version_is_32_6_0` が空スタブになっていること（コメント付き）
- [x] `const_gen_chunk_size_valid` が v187000_tests のテスト名と異なること
- [x] `const_gen_chunk_size_zero_e0335` が v187000_tests のテスト名と異なること
- [x] テスト 3 が E0335 なしのポジティブケースを検証していること
- [x] テスト 4 が E0335 のネガティブケースを検証していること
- [x] 挿入位置が `v326000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
- [x] `benchmarks/v32.7.0.json` の `milestone` が `"Language Power"` であること
- [x] `versions/current.md` が v32.7.0 に更新されていること
