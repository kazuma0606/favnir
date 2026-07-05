# v30.6.0 — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `30.5.0` であること
- [x] `cargo test 2>&1 | grep "test result"` が `2406 passed` を含むこと
- [x] `driver.rs` に `mod v306000_tests` が存在しないこと
- [x] v30.5.0 が COMPLETE であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `30.5.0` → `30.6.0` に更新
- [x] **T2** `fav/src/driver.rs` — `cargo_toml_version_is_30_5_0` をスタブ化
- [x] **T3** `fav/src/driver.rs` — `cmd_test` の引数なし時に `tests/` も走査するよう修正
- [x] **T4** 手動検証 — `examples/csv-to-postgres/` で `fav test` が 3 件を検出すること
- [x] **T4b** 手動検証 — `fav test --filter validate` でフィルタ動作すること（ロードマップ完了条件）
- [x] **T5** `fav/src/driver.rs` — `v306000_tests`（3 件）を追加
- [x] **T6** `CHANGELOG.md` — `[v30.6.0]` セクションを先頭に追記
- [x] **T7** `benchmarks/v30.6.0.json` — 新規作成
- [x] **T8** `versions/current.md` — 最新安定版を v30.6.0 に更新

---

## テスト確認

- [x] **T9** `cargo test v306000 2>&1 | tail -5` — 3/3 PASS
- [x] **T10** `cargo test 2>&1 | grep "test result"` — 全件 PASS（0 failures）

---

## 完了処理

- [x] **T11** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = "30.6.0"
- [x] `cmd_test` 引数なし時に `tests/` ディレクトリも走査する
- [x] `fav test`（`examples/csv-to-postgres/` で実行）が pipeline_test.fav の 3 件を検出
- [x] `fav test --filter validate`（`examples/csv-to-postgres/` で実行）がフィルタ動作する
- [x] `cargo test v306000` — 3/3 PASS
- [x] `cargo test` — 全件 PASS
- [x] `CHANGELOG.md` に `[v30.6.0]` セクション
- [x] `benchmarks/v30.6.0.json` 存在
- [x] `versions/current.md` を v30.6.0 に更新
- [x] tasks.md を COMPLETE に更新

---

## コードレビューチェックリスト

- [x] `tests_dir.is_dir()` チェックがあること（`tests/` が存在しない場合に安全）
- [x] `test_files.sort(); test_files.dedup();` が `extend` の後に呼ばれていること（重複排除）
- [x] `v306000_tests` に `use super::*;` がないこと（include_str! のみ使用、不要インポートなし）
- [x] `v306000_tests` に `benchmark_v30_6_0_exists` テストがあること

---

## コードレビュー指摘・対応記録

- `[project]` セクションを `fav.toml` パーサーが認識しないため、`src_dir = "."` になりプロジェクト全体を走査していた。`tests/` ファイルが `src_dir` スキャンで重複検出される問題を発見。
- 対応: `tests/` ファイルを canonical path で先に収集し、`src_dir` スキャン時に同一ファイルをスキップする方式で重複排除を実装。
- `tests/` 側のファイルは `load_all_items` で import 解決、`src/` 側はパースのみ（rune import のある stages.fav が load_all_items でエラーになるため）。
