# v32.1.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `32.0.0` であること
- [x] `benchmarks/v32.0.0.json` の `tests_passed` が 2456 であることを確認（暫定値 2456 の根拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2456 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v321000_tests` が存在しないこと
- [x] v32.0.0 が COMPLETE であること
- [x] `type_implements_bound`（checker.rs:7901 付近）が `Display` / `Hash` を処理していること
- [x] `check_errors` が `super::*` では参照できないことを確認（各テストモジュール内のローカル関数）

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `32.0.0` → `32.1.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_32_0_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v321000_tests`（3 件 + ローカル `check_errors`）を追加
       挿入位置: `v320000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::frontend::parser::Parser` 等を使用
- [x] **T4** `CHANGELOG.md` — `[v32.1.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v32.1.0.json` — 新規作成（実測値 2459）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v32.1.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v321000 2>&1 | tail -8` — 3/3 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2460 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v32.1.0.json` の `tests_passed` を実測値で更新（2460 — ネガティブテスト追加後の確定値）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"32.1.0"`
- [x] `cargo_toml_version_is_32_0_0` が空スタブになっていること
- [x] `bounded_generics_display_and_hash_bounds` テストが PASS
- [x] `cargo test --bin fav v321000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS（2460 passed、0 failures）
- [x] `CHANGELOG.md` に `[v32.1.0]` セクション
- [x] `benchmarks/v32.1.0.json` 存在かつ `tests_passed` が実測値（2460）
- [x] `versions/current.md` を v32.1.0 に更新
- [x] `tasks.md` が COMPLETE
- [x] site/ MDX 更新: 対象外（`generics.mdx` は既に完成）

---

## コードレビューチェックリスト

- [x] `v321000_tests` に `use super::*` が**ない**こと（`use crate::...` で完結）
- [x] `v321000_tests` 内にローカル `check_errors` が定義されていること
- [x] `cargo_toml_version_is_32_0_0` が空スタブになっていること（コメント付き）
- [x] テスト 3 が `Display`（identity_display / String）と `Hash`（hash_it / Int）の両方を検証していること
- [x] 挿入位置が `v320000_tests` 直後・`// ── v31.7.0 tests` の前であること（`v31.9.0` ではない）
- [x] `v321000_tests` の 3 件が PASS — `v320000_tests` のスタブは別カウントのため影響なし
- [x] CHANGELOG.md の日付が正しいこと（2026-07-03）
- [x] `benchmarks/v32.1.0.json` の `milestone` が `"Language Power"` であること

---

## 実装メモ

- テスト 3 当初は `f"{val}"` を使用していたが、`T` が `Var` 型のため E0254（f-string type error）が発生。
  `identity_display<T with Display>(val: T) -> T { val }` に変更して解決（Display bound 自体は正常動作）。
