# v37.0.0 タスクリスト — Data Quality First マイルストーン宣言

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v36.1-v37.0.md` の v37.0.0（「Data Quality First マイルストーン宣言 ★クリーンアップ」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2699（v36.9.0 完了時点の実績値））し、実測値をここに記録: 2699
- [x] Cargo.toml バージョンが `36.9.0` であることを確認
- [x] `v36900_tests::cargo_toml_version_is_36_9_0` がライブアサーション（`assert!(cargo.contains("36.9.0"), ...)`）であることを確認し、行番号を記録: 43092
- [x] `v36900_tests` の他 3 テスト（`changelog_has_v36_9_0` / `w025_message_references_e0380` / `validate_summary_line_added`）はバージョン変更後も pass することを確認（バージョン番号を含まないため影響なし）
- [x] `driver.rs` に `v37000_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] `CHANGELOG.md` に `[v37.0.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `MILESTONE.md` に `"Data Quality First"` が存在しないことを確認（今回追加）
- [x] `MILESTONE.md` の先頭が `# Favnir Milestones` → `## v35.0.0` であり、v36.0.0 先頭セクションが存在しないことを確認（v36.0.0 と v37.0.0 の両セクションを追加する）
- [x] `README.md` に `"Data Quality"` が含まれないことを確認（今回追加）
- [x] `README.md` に `"Deployment Story"` が含まれないことを確認（今回 v36.0 宣言も追加する）
- [x] `fav/tmp/hello.fav` の存在と内容を確認（cargo clean 後も存在・内容正常）
- [x] `v36900_tests` の閉じ `}` の行番号を確認し、ここに記録: 43117
- [x] `versions/current.md` の最新安定版が `v36.9.0`・次バージョンが `v37.0.0` であることを確認

## T1: CHANGELOG.md に [v37.0.0] エントリを追加

- [x] `## [v36.9.0]` の `---` セパレータ直後に `## [v37.0.0]` エントリを挿入
- [x] 日付を `YYYY-MM-DD` 形式の実装当日の日付に変更（2026-07-09）

## T2: ★クリーンアップ — cargo clean

- [x] `cargo clean` を実行（27.9 GiB 削除）
- [x] `fav/tmp/hello.fav` が消失していないことを確認（存在・内容正常）
- [x] `cargo build` でコンパイルエラーがないことを確認（3m 05s でビルド成功）

## T3: MILESTONE.md に v37.0.0 / v36.0.0 セクション追加

- [x] `# Favnir Milestones` ヘッダの直後（`## v35.0.0` の直前）に v37.0.0 セクションを挿入
  - [x] 宣言文（`schema` / `expect` / `fav validate` / W025 / E0380〜E0384 / `fav schema diff`）を含む
  - [x] 達成コンポーネント表（v36.1〜v36.9 の 9 行）を含む
  - [x] 宣言日（2026-07-09）と宣言バージョン（v37.0.0）を含む
  - [x] セクション末尾に `---` セパレータを追加
- [x] v37.0.0 セクションの直後に v36.0.0 セクションも追加
  - [x] 宣言文（`fav deploy` / `fav ci init` / `!Effect` 廃止）を含む
  - [x] 宣言日（2026-07-08）と宣言バージョン（v36.0.0）を含む
  - [x] セクション末尾に `---` セパレータを追加
- [x] 挿入後の先頭順序が `v37.0.0` → `v36.0.0` → `v35.0.0` になっていることを確認

## T4: README.md — v36.0 / v37.0 マイルストーン宣言を追加

- [x] `**v35.0（2026-07-04）で、[Production Ready](./MILESTONE.md) マイルストーンを宣言しました。**` の直後に追加
  - [x] v36.0 Deployment Story 宣言行を追加（`"Deployment Story"` を含む）
  - [x] v37.0 Data Quality First 宣言行を追加（`"Data Quality"` を含む）
  - [x] 両方の行が追加されていることを確認（`readme_mentions_data_quality` テストで v36.0 / v37.0 両方を検証）

## T5: driver.rs — `v36900_tests::cargo_toml_version_is_36_9_0` をスタブ化

- [x] ライブアサーション → `// Stubbed: version bumped to 37.0.0` に変更

## T6: driver.rs — `v37000_tests` モジュールを新規追加

- [x] `v36900_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する（行番号: 43117）
- [x] `v36900_tests` の閉じ `}` の後に `v37000_tests` モジュールを追加
  - [x] `cargo_toml_version_is_37_0_0`（`include_str!("../Cargo.toml")`）
  - [x] `changelog_has_v37_0_0`（`include_str!("../../CHANGELOG.md")`）
  - [x] `milestone_has_data_quality_first`（`include_str!("../../MILESTONE.md")`）
  - [x] `readme_mentions_data_quality`（`include_str!("../../README.md")` で `"Data Quality First"` と `"Deployment Story"` の両方を確認）

## T7: バージョン更新（T3〜T6 すべて完了後）

- [x] `fav/Cargo.toml` バージョンを `37.0.0` に更新（T3〜T6 すべて完了・コンパイルエラー解消の後）

## T8: テスト実行

- [x] `cargo test` 全通過 — ≥ 2703 passed; 0 failed — 実測: 2703 passed
- [x] `v37000_tests` の 4 テストがすべて pass
- [x] `cargo_toml_version_is_37_0_0` が pass
- [x] `changelog_has_v37_0_0` が pass
- [x] `milestone_has_data_quality_first` が pass
- [x] `readme_mentions_data_quality` が pass

## T9: ドキュメント更新

- [x] `versions/v36-v40/v37.0.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v37.0.0（最新安定版）・v37.1.0（次バージョン）に更新
- [x] `versions/roadmap/roadmap-v36.1-v37.0.md` の v37.0.0 を完了済みにマーク（✅）
- [x] `versions/roadmap/roadmap-v36.1-v37.0.md` の v37.0.0 完了条件に「テスト数は 2703 件（4000+ は後続スプリントへ持ち越し）」を注記

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Data Quality First"` が含まれる | `milestone_has_data_quality_first` テスト ✅ |
| 2 | `README.md` に `"Data Quality"` と `"Deployment Story"` が含まれる | `readme_mentions_data_quality` テスト ✅ |
| 3 | `CHANGELOG.md` に `[v37.0.0]` が含まれる | `changelog_has_v37_0_0` テスト ✅ |
| 4 | `Cargo.toml` バージョンが `37.0.0` | `cargo_toml_version_is_37_0_0` テスト ✅ |
| 5 | `cargo clean` 実施済み | T2 実行記録 ✅（27.9 GiB 削除） |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2703） | 実測: 2703 passed, 0 failed ✅ |
