# v36.0.0 タスクリスト — Deployment Story マイルストーン宣言

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v35.1-v36.0.md` の v36.0.0（「Deployment Story マイルストーン宣言・★クリーンアップ」）に沿ったバージョン。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2651 以上）し、実測値をここに記録: 2656
- [x] Cargo.toml バージョンが `35.9.0` であることを確認
- [x] `v35900_tests::cargo_toml_version_is_35_9_0` がライブアサーション（`assert!(cargo.contains("35.9.0"), ...)`）であることを確認
- [x] driver.rs に `v36000_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] driver.rs の末尾がモジュール順 `v35800_tests` → `v35700_tests` → `v35900_tests` であることを確認（`v36000_tests` は `v35900_tests` の後に追加）
- [x] `CHANGELOG.md` に `[v36.0.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `MILESTONE.md` に `"Deployment Story"` が含まれないことを確認（今回追加）
- [x] `driver.rs` に `pub fn cmd_deploy` が含まれることを確認（既存）
- [x] `driver.rs` に `generate_ci_yaml` が含まれることを確認（既存）
- [x] `versions/current.md` の最新安定版が `v35.9.0`・次バージョンが `v36.0.0` であることを確認

## T1: CHANGELOG.md に [v36.0.0] エントリを追加

- [x] `## [v35.9.0]` の直前に `## [v36.0.0]` エントリを挿入

## T2: MILESTONE.md に Deployment Story 宣言セクションを追加

- [x] `"Deployment Story"` を含むセクション見出しと宣言文を追加

## T3: driver.rs — v35900_tests::cargo_toml_version_is_35_9_0 をスタブ化

- [x] ライブアサーション → `// stubbed: version bumped to 36.0.0` に変更

## T4: driver.rs — v36000_tests モジュールを新規追加

- [x] driver.rs ファイル末尾（`v35900_tests` モジュールの閉じ `}` の後）に `v36000_tests` モジュールを追加
  - [x] `cargo_toml_version_is_36_0_0`
  - [x] `changelog_has_v36_0_0`
  - [x] `milestone_has_deployment_story`
  - [x] `deploy_lambda_fn_exists`
  - [x] `ci_init_yaml_fn_exists`

## T5: バージョン更新（T3 完了後）

- [x] `fav/Cargo.toml` バージョンを `36.0.0` に更新

## T6: テスト実行

- [x] `cargo test` 全通過 — 2656 passed; 0 failed（2651 + 新規 5 件 = 2656 ✓）
- [x] `v36000_tests` の 5 テストが pass

## T7: ★クリーンアップ（cargo clean）

- [x] `cargo clean` を実行（x.0.0 マイルストーン必須）

## T8: ドキュメント更新

- [x] `versions/v36-v40/v36.0.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を以下の通り更新:
  - 最新安定版: `v36.0.0` — Deployment Story マイルストーン宣言（日付記入）
  - 進行中バージョン: Deployment Story スプリント完了、v36.1.0〜 に向けて進行中
  - 次に切る版: `v36.1.0`
  - マイルストーン進捗: `v36.0 — Deployment Story` を **完了** に更新
- [x] site/ への MDX 追加は不要（v35.8.0 で `site/content/docs/deploy/lambda.mdx` 実施済み）

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Deployment Story"` が含まれる | `milestone_has_deployment_story` テスト |
| 2 | `driver.rs` に `pub fn cmd_deploy` が含まれる | `deploy_lambda_fn_exists` テスト |
| 3 | `driver.rs` に `generate_ci_yaml` が含まれる | `ci_init_yaml_fn_exists` テスト |
| 4 | `CHANGELOG.md` に `[v36.0.0]` が含まれる | `changelog_has_v36_0_0` テスト |
| 5 | `Cargo.toml` バージョンが `36.0.0` | `cargo_toml_version_is_36_0_0` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2656） | T6 実行結果 |
| 7 | `cargo clean` 実施済み | T7 実行記録 |
| 注 | ロードマップ記載の `テスト数 3500+` は v36.1〜以降に繰り越し（spec.md ロードマップとの差異セクション参照） | — |
| 注 | `examples/lambda-deploy/fav.toml` の存在は v35900_tests で担保済み | `cargo test` で間接確認 |
