# v35.9.0 タスクリスト — v36.0 前調整・安定化

## ステータス: COMPLETE

> ロードマップ整合: `roadmap-v35.1-v36.0.md` の v35.9.0（「E2E 動作確認・CHANGELOG 更新・v36.0 宣言文草案」）に沿ったバージョン。
> 今回は `v35900_tests` モジュールと CHANGELOG エントリを新規作成する（sprint 事前作成なし）。

## T0: 事前確認

- [x] `cargo test` の実測通過数を確認（目安: 2646 以上）し、実測値をここに記録: ___
- [x] Cargo.toml バージョンが `35.8.0` であることを確認
- [x] `v35800_tests::cargo_toml_version_is_35_8_0` がライブアサーション（`assert!(cargo.contains("35.8.0"), ...)`）であることを確認
- [x] driver.rs に `v35900_tests` モジュールが存在しないことを確認（今回新規作成）
- [x] driver.rs の既存モジュール順が `v35600_tests` → `v35800_tests` → `v35700_tests` の非標準順であることを確認（`v35900_tests` はファイル末尾に追加）
- [x] `CHANGELOG.md` に `[v35.9.0]` エントリが存在しないことを確認（今回新規作成）
- [x] `examples/lambda-deploy/fav.toml` が存在し `target = "lambda"` を含むことを確認
- [x] `site/content/docs/deploy/lambda.mdx` が存在することを確認
- [x] `versions/roadmap/roadmap-v35.1-v36.0.md` に `"Deployment Story"` が含まれることを確認
- [x] `versions/current.md` の最新安定版が `v35.8.0`・次バージョンが `v35.9.0` であることを確認

## T1: CHANGELOG.md に [v35.9.0] エントリを追加

- [x] `## [v35.8.0]` の直前に `## [v35.9.0]` エントリを挿入

## T2: driver.rs — v35800_tests::cargo_toml_version_is_35_8_0 をスタブ化

- [x] ライブアサーション → `// stubbed: version bumped to 35.9.0` に変更

## T3: driver.rs — v35900_tests モジュールを新規追加

- [x] driver.rs ファイル末尾（`v35700_tests` モジュールの閉じ `}` = line 42309 の後）に `v35900_tests` モジュールを追加（注: `v35800_tests` が `v35700_tests` より前に定義されている非標準順序の末尾に追記）
  - [x] `cargo_toml_version_is_35_9_0`
  - [x] `changelog_has_v35_9_0`
  - [x] `lambda_deploy_example_exists`
  - [x] `deploy_docs_exists`
  - [x] `v36_deployment_story_planned`

## T4: バージョン更新（T2 完了後）

- [x] `fav/Cargo.toml` バージョンを `35.9.0` に更新

## T5: テスト実行

- [x] `cargo test` 全通過 — 2651 passed; 0 failed（2646 + 新規 5 件 = 2651 ✓）
- [x] `v35900_tests` の 5 テストが pass

## T6: ドキュメント更新

- [x] `versions/v30-v35/v35.9.0/tasks.md` を COMPLETE ステータスに更新
- [x] `versions/current.md` を v35.9.0（最新安定版）・v36.0.0（次バージョン）に更新

---

## 完了条件チェックリスト（spec.md 対応）

| # | spec.md 完了条件 | 確認方法 |
|---|---|---|
| 1 | `examples/lambda-deploy/fav.toml` が `"lambda"` を含む | `lambda_deploy_example_exists` テスト |
| 2 | `site/content/docs/deploy/lambda.mdx` が存在する | `deploy_docs_exists` テスト |
| 3 | `roadmap-v35.1-v36.0.md` に `"Deployment Story"` が含まれる | `v36_deployment_story_planned` テスト |
| 4 | `CHANGELOG.md` に `[v35.9.0]` が含まれる | `changelog_has_v35_9_0` テスト |
| 5 | `Cargo.toml` バージョンが `35.9.0` | `cargo_toml_version_is_35_9_0` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2651） | T5 実行結果 |

## spec-reviewer 対応（計画フェーズで適用済み）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| T3 の挿入位置記述が誤り（v35700_tests の後→ファイル末尾に） | [HIGH] | tasks.md・plan.md を正確化（line 42309 の後・非標準順注記追加） |
| T0 にモジュール定義逆順の確認項目が欠落 | [HIGH] | T0 に確認項目を追加 |
| `deploy_docs_exists` テストの検証文字列が spec.md に未記載 | [MED] | spec.md「既存確認のみ」表に `"lambda"` を含む旨を追記 |
| テスト数閾値 2646 が追加後の 2651 と不一致 | [MED] | spec.md・tasks.md を `≥ 2651` に修正 |
| v36.0 宣言文草案への対応が tasks に欠落 | [MED] | spec.md §「ロードマップとの整合」にスコープ外として明示 |
| T0 に `versions/current.md` 確認項目が欠落 | [LOW] | T0 に確認項目を追加 |
| spec-reviewer 対応セクションが欠落 | [LOW] | 本セクションを追加 |

## コードレビュー対応（実施後に記録）

| 指摘 | 優先度 | 対応 |
|---|---|---|
| `site/content/docs/deploy/lambda.mdx` のスタブ注記が "v35.8" と古い | [MED] | "v36.0 (Deployment Story milestone)" に更新 |
| モジュール定義順が v35800→v35700→v35900 と不一致 | [LOW] | 記録のみ（スプリント一括構造のため変更不要） |
