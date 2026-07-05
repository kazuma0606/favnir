# v32.5.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `32.4.0` であること
- [x] `benchmarks/v32.4.0.json` の `tests_passed` が 2472 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2472 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v325000_tests` が存在しないこと
- [x] v32.4.0 が COMPLETE であること
- [x] `TokenKind::LinearArrow` が lexer に存在すること（v18.5.0 実装済み）
- [x] E0332 / E0333 が checker に存在すること（checker.rs に "E0332"/"E0333" の文字列）
- [x] `cargo_toml_version_is_32_4_0` が v324000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v324000` が 4/4 PASS であること（前バージョンテスト動作確認）
- [x] `cargo test --bin fav v185000` が PASS であること（線形型チェックが動作中であることの事前確認）
- [x] テスト名が v185000_tests と重複しないこと（`linear_type_double_use_e0332` / `linear_type_unused_var_e0333`）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `32.4.0` → `32.5.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_32_4_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v325000_tests`（4 件 + ローカル `check_errors`）を追加
       挿入位置: `v324000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::frontend::parser::Parser` 等を使用
- [x] **T4** `CHANGELOG.md` — `[v32.5.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v32.5.0.json` — 新規作成（暫定値 2476、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v32.5.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v325000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2476 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v32.5.0.json` の `tests_passed` を実測値で更新（2476 — 暫定値と一致）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"32.5.0"`
- [x] `cargo_toml_version_is_32_4_0` が空スタブになっていること
- [x] `linear_type_double_use_e0332` テストが PASS（E0332 確認）
- [x] `linear_type_unused_var_e0333` テストが PASS（E0333 確認）
- [x] `cargo test --bin fav v325000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `CHANGELOG.md` に `[v32.5.0]` セクション
- [x] `benchmarks/v32.5.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v32.5.0.json` の `milestone` フィールドが `"Language Power"` であること
- [x] `versions/current.md` を v32.5.0 に更新
- [x] `tasks.md` が COMPLETE
- [x] site/ MDX 更新: 対象外（`linear-types.mdx` は v18.5.0 で完成済み）

---

## コードレビューチェックリスト

- [x] `v325000_tests` に `use super::*` が**ない**こと（`use crate::...` で完結）
- [x] `v325000_tests` 内にローカル `check_errors` が定義されていること
- [x] `cargo_toml_version_is_32_4_0` が空スタブになっていること（コメント付き）
- [x] `linear_type_double_use_e0332` が v185000_tests の `linear_double_use_is_e0332` と名前が異なること
- [x] `linear_type_unused_var_e0333` が v185000_tests の `linear_unused_is_e0333` と名前が異なること
- [x] テスト 3 が E0332 のネガティブ（エラーが出ること）を検証していること
- [x] テスト 4 が E0333 のネガティブ（エラーが出ること）を検証していること
- [x] 挿入位置が `v324000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
- [x] `benchmarks/v32.5.0.json` の `milestone` が `"Language Power"` であること
- [x] `versions/current.md` が v32.5.0 に更新されていること
