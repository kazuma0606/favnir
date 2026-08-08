# v65.0.0 タスクリスト

Status: COMPLETE
Version: 65.0.0
Base tests: 3449
Target tests: 3453

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3449 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"64.0.0"` であることを確認（`"65.0.0"` への更新対象）
- [x] `MILESTONE.md` に `"Performance 1.0"` が含まれないことを確認（新規追加対象）
- [x] `README.md` に `"Performance 1.0"` が含まれないことを確認（追記対象）
- [x] `driver.rs` に `v64900_tests` が存在することを確認（`v65000_tests` の挿入位置）
- [x] `driver.rs` に `v65000_tests` が存在しないことを確認（新規追加）

---

## T1: バージョン更新・ドキュメント追加

- [x] `fav/Cargo.toml` の `version = "64.0.0"` を `version = "65.0.0"` に変更
- [x] `MILESTONE.md` 先頭（`## v64.0.0` の前）に v65.0.0 宣言エントリを挿入
  - [x] `"Performance 1.0"` を含む宣言文
  - [x] v64.1〜v64.9 の達成内容一覧（9 バージョン）
  - [x] `テスト数: 3453` の記載
- [x] `README.md` の `**v64.0.0 — Incremental & Scale` 行の直前に v65.0.0 宣言文を追加
  - [x] `"Performance 1.0"` または `"v65.0"` を含む（`readme_mentions_performance1` 通過条件）
- [x] `CHANGELOG.md` 先頭に v65.0.0 エントリを追加

---

## T2: `driver.rs` — `v65000_tests` 追加

- [x] `// -- v64900_tests` コメント行の直前に `v65000_tests` を挿入
  - [x] `cargo_toml_version_is_65_0_0`（`"version = \"65.0.0\""` を含む）
  - [x] `changelog_has_v65_0_0`（`"v65.0.0"` を含む）
  - [x] `milestone_has_performance1`（`"Performance 1.0"` を含む）
  - [x] `readme_mentions_performance1`（`"Performance 1.0"` または `"v65.0"` を含む）
  - [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし
- [x] 旧 `cargo_toml_version_is_*` テスト 15 件の `"64.0.0"` 誤参照を `"65.0.0"` に一括修正

---

## T3: ビルド・テスト（クリーンアップ前）

- [x] `cargo test --bin fav v65000_tests` で 4 件 PASS
  - [x] `cargo_toml_version_is_65_0_0` PASS
  - [x] `changelog_has_v65_0_0` PASS
  - [x] `milestone_has_performance1` PASS
  - [x] `readme_mentions_performance1` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3453 tests passed, 0 failed を確認

---

## T4: ★クリーンアップ

- [x] `cargo clean` 実行（12903 files、10.5 GiB 削除）
- [x] `fav/tmp/hello.fav` が存在することを確認（削除されず、復元不要）
- [x] `cargo test -j 8 -- --test-threads=8` で 3453 tests passed, 0 failed を確認（クリーンビルド後）

---

## T5: ドキュメント更新

- [x] `CHANGELOG.md` 先頭に v65.0.0 エントリを追加（T1 で実施済み）
- [x] `versions/roadmap/roadmap-v64.1-v65.0.md` v65.0 セクションに実績追記（3453 tests、★クリーンアップ完了）
- [x] `versions/current.md` の「進行中」を v65.0.0（3453 tests）に更新
- [x] `versions/current.md` の「次に切る版」を v66.0.0 に更新
- [x] `MILESTONE.md` は T1 で更新済み
- [x] tasks.md を COMPLETE に更新（本ファイル）

## 実装中の追加対応

- 旧マイルストーンテスト（v56300〜v64000）の `cargo_toml_version_is_*` が `"64.0.0"` をチェックする既知バグにより、バージョン更新後に 13 件が FAILED → `"65.0.0"` に一括置換して解消

## コードレビュー対応（code-reviewer 指摘）

- [MED] エラーメッセージ文字列 11 件に "64.0.0" が残存 → "65.0.0" に一括修正（3 パターン）
- [MED] v62000/v64000 の `cargo_toml_version_is_*` が "65.0.0" を検査（本来のバージョンではない）→ 既存の systemic 設計問題、次回 v66.0.0 更新時に同様の一括置換が必要（known issue として記録）
- [LOW] `current.md` 最終更新日が "2026-07-30 (v60.0.0)" のまま → "2026-08-04 (v65.0.0)" に更新
