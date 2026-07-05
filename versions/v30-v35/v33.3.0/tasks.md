# v33.3.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `33.2.0` であること
- [x] `benchmarks/v33.2.0.json` の `tests_passed` が 2504 であることを確認
- [x] `cargo test 2>&1 | grep "test result"` を実行して通過件数（2504 passed 想定）を実測確認すること
- [x] `driver.rs` に `mod v333000_tests` が存在しないこと
- [x] v33.2.0 が COMPLETE であること
- [x] `cargo_toml_version_is_33_2_0` が v332000_tests 内に存在すること（スタブ化対象）
- [x] `cargo test --bin fav v332000` が 4/4 PASS であること（前バージョン 4 件 PASS を確認）
- [x] `fav/src/ast.rs` に `StreamingAnnotation` 構造体および `FlwDef.streaming` フィールドが存在すること
- [x] `v191000_tests` のテスト名（`streaming_annotation_parses` / `streaming_default_chunk_size_parses` / `streaming_pipeline_executes` / `streaming_stateful_annotation_parses`）と v333000_tests のテスト名が重複しないこと

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `33.2.0` → `33.3.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_33_2_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `v333000_tests`（4 件）を追加
       挿入位置: `v332000_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし、`use crate::frontend::parser::Parser;` を明示 import
- [x] **T4** `CHANGELOG.md` — `[v33.3.0]` セクションを先頭に追記
- [x] **T5** `benchmarks/v33.3.0.json` — 新規作成（暫定値 2508、実測後に確定）
- [x] **T6** `versions/current.md` — 「最新安定版」欄を v33.3.0 に更新

---

## テスト確認

- [x] **T7** `cargo test --bin fav v333000 2>&1 | tail -8` — 4/4 PASS
- [x] **T8** `cargo test 2>&1 | grep "test result"` — 全件 PASS（2508 passed、0 failures）

---

## 完了処理

- [x] **T9** `benchmarks/v33.3.0.json` の `tests_passed` を実測値で更新（2508 確定）
- [x] **T10** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"33.3.0"`
- [x] `cargo_toml_version_is_33_2_0` が空スタブになっていること
- [x] `cargo test --bin fav v333000` — 4/4 PASS
- [x] `cargo test` — 全件 PASS（2508 件、0 failures）
- [x] `CHANGELOG.md` に `[v33.3.0]` セクション
- [x] `benchmarks/v33.3.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v33.3.0.json` の `milestone` フィールドが `"Performance & Tooling"` であること
- [x] `versions/current.md` を v33.3.0 に更新
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v333000_tests` に `use super::*` が**ない**こと（`use crate::frontend::parser::Parser;` のみ）
- [x] `cargo_toml_version_is_33_2_0` が空スタブになっていること（コメント付き）
- [x] `streaming_seq_without_annotation_has_none` / `streaming_chunk_size_boundary_one` が v191000_tests のテスト名と異なること
- [x] `streaming_seq_without_annotation_has_none` が `fd.streaming.is_none()` を assert していること
- [x] `streaming_chunk_size_boundary_one` が `s.chunk_size == Some(1)` を assert していること
- [x] 挿入位置が `v332000_tests` 直後・`// ── v31.7.0 tests` の前であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-04）
- [x] `benchmarks/v33.3.0.json` の `milestone` が `"Performance & Tooling"` であること
- [x] `versions/current.md` が v33.3.0 に更新されていること

---

## コードレビュー指摘と対応

### spec-reviewer [MED]（実装時に対応済み）
- `streaming_seq_without_annotation_has_none` / `streaming_chunk_size_boundary_one` で `prog.items[0]` を直接アクセスする前に長さチェックがなかった
- 対応: 両テストに `assert_eq!(prog.items.len(), 1, "expected 1 item")` を追加
