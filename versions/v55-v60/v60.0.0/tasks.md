# v60.0.0 Tasks — Enterprise 1.0 宣言 ★クリーンアップ

Date: 2026-07-30
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3326 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"59.9.0"` であることを確認
- [x] `fav/src/driver.rs` に `v60000_tests` がまだ存在しないことを確認
- [x] `grep -c 'Cargo.toml version should be 59\.9\.0' fav/src/driver.rs` が 8 件であることを確認（rolling check failure メッセージ）
- [x] `grep -c 'version = \\"59\.9\.0\\"' fav/src/driver.rs` が 8 件であることを確認（rolling check assert 本体）
- [x] `MILESTONE.md` の先頭が `## v60.0.0（予定）— Enterprise 1.0` であることを確認

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "59.9.0"` → `"60.0.0"`

---

## T2: CHANGELOG.md 更新

- [x] `CHANGELOG.md` 先頭に v60.0.0 エントリを追加

---

## T3: MILESTONE.md 更新

- [x] `## v60.0.0（予定）— Enterprise 1.0` エントリを正式版に置き換える
  - `（予定）` → `（2026-07-30）` に変更（元の `## v60.0.0（予定）— Enterprise 1.0` エントリを削除して置き換えること）
  - 宣言文（引用文）を追加
  - v56〜v59 達成内容のリストを追加
  - `milestone_has_enterprise1` テストは `"Enterprise 1.0"` を検索する（既存でも通るが正式化する）
  - [x] 事後確認: `grep '（予定）' MILESTONE.md` が 0 件であることを確認（「予定」エントリの残存がないこと）

---

## T4: README.md 更新

- [x] 正式リリース文に変更（"予定" → "宣言しました（2026-07-30）"）

---

## T5: driver.rs — v60000_tests 追加

- [x] `v60000_tests` モジュールを `v59900_tests` の直前に挿入（4 件）
  - [x] `cargo_toml_version_is_60_0_0`
  - [x] `changelog_has_v60_0_0`
  - [x] `milestone_has_enterprise1`
  - [x] `readme_mentions_enterprise1`
  - [x] `use super::*;` は不要（`include_str!` のみ使用）

---

## T6: driver.rs — ローリングチェック更新（8 件）

- [x] `version = \"59.9.0\"` → `\"60.0.0\"` に一括更新（8 件）
- [x] failure メッセージ 8 件を `"60.0.0"` に更新
  - 対象: `v59000_tests` / `v58900_tests` / `v58000_tests` / `v57900_tests` / `v57000_tests` / `v56900_tests` / `v56300_tests` / `v59900_tests`
- [x] 事後確認: `grep -c 'version should be 60\.0\.0' fav/src/driver.rs` = 9 件（rolling check 8 件 + v60000_tests 1 件）
- [x] 事後確認: `grep -c 'version = \\"60\.0\.0\\"' fav/src/driver.rs` = 9 件（同上）
- [x] 事後確認: `grep 'version should be 59\.9\.0' fav/src/driver.rs` = 0 件

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v60000_tests::cargo_toml_version_is_60_0_0` pass
- [x] `v60000_tests::changelog_has_v60_0_0` pass
- [x] `v60000_tests::milestone_has_enterprise1` pass
- [x] `v60000_tests::readme_mentions_enterprise1` pass
- [x] 総テスト数 **3330** tests passed, 0 failed を確認

---

## T8: ★クリーンアップ（cargo clean）

- [x] `cargo clean` を実行（28034 files、28.2 GiB 削除）
- [x] `cargo build` でビルドが通ることを確認（fav v60.0.0 Finished）

---

## T9: 事後処理

- [x] `versions/current.md` を v60.0.0 / 3330 tests に更新
- [x] `versions/roadmap/roadmap-v59.1-v60.0.md` の v60.0.0 実績欄を更新（`3330 tests passed, 0 failed（2026-07-30 完了）`）
- [x] `versions/roadmap/roadmap-v55.1-v60.0.md` のテーブル行を更新（`3330 / +4 / 実績値（2026-07-30 COMPLETE）`）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

なし（Enterprise 1.0 宣言バージョン。コード追加なし、ドキュメント・テスト・バージョン更新のみ）

---

Status: COMPLETE
