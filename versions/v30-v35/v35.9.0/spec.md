# v35.9.0 spec — v36.0 前調整・安定化

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v35.9.0 |
| テーマ | v35.1〜v35.8 の機能統合確認・v36.0 Deployment Story 宣言前の安定化 |
| 前提 | v35.8.0（v35.0C）COMPLETE — `!Effect` 廃止を全レイヤーで完結 |
| 完了条件 | `v35900_tests` 全テスト pass・`cargo test` 0 failures |

## 背景と目的

v35.1〜v35.8 のスプリントで以下を達成した：

- **Deployment Story 機能**（v35.1〜v35.3）: `fav deploy --target lambda/docker`・`fav ci init`
- **`!Effect` 廃止完結**（v35.4〜v35.8）: AST/parser/checker/LSP/error_catalog/MCP/docs_server すべてクリーン
- **ドキュメント統一**（v35.6）: サイト MDX 128 件・ctx-syntax-guide・README/MILESTONE 更新

本バージョンは v36.0 宣言前の安定化スプリントとして、これらの達成状況を横断的に確認する。

## 実装スコープ

### 新規実装（本セッションで実施）

| ファイル | 変更内容 |
|---|---|
| `CHANGELOG.md` | `## [v35.9.0]` エントリ追加 |
| `fav/src/driver.rs` | `v35900_tests` モジュール（5 件）追加 |
| `fav/src/driver.rs` | `v35800_tests::cargo_toml_version_is_35_8_0` をスタブ化 |
| `fav/Cargo.toml` | バージョン `35.8.0` → `35.9.0` |

### 既存確認のみ（追加実装なし）

| 確認内容 | 状態 |
|---|---|
| `examples/lambda-deploy/` — Lambda デプロイデモ | 存在（`fav.toml` に `target = "lambda"` あり） |
| `site/content/docs/deploy/lambda.mdx` — Lambda デプロイドキュメント | 存在（v35.1.0 で作成済み）・ファイル内容に `"lambda"` を含む（`deploy_docs_exists` テストで検証） |
| `!Effect` 廃止完結 — lsp/completion.rs・docs_server.rs・mcp/mod.rs | 確認済み（v35.7〜v35.8 テスト通過） |
| `versions/roadmap/roadmap-v35.1-v36.0.md` — v36.0 Deployment Story 計画 | 存在 |

## v35900_tests の設計（新規作成）

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_35_9_0` | Cargo.toml に `"35.9.0"` が含まれる |
| `changelog_has_v35_9_0` | `CHANGELOG.md` に `[v35.9.0]` が含まれる |
| `lambda_deploy_example_exists` | `examples/lambda-deploy/fav.toml` が存在し `"lambda"` を含む |
| `deploy_docs_exists` | `site/content/docs/deploy/lambda.mdx` が存在し `"lambda"` を含む |
| `v36_deployment_story_planned` | `roadmap-v35.1-v36.0.md` に `"Deployment Story"` が含まれる |

## v35800_tests スタブ化

`v35800_tests::cargo_toml_version_is_35_8_0` は現在ライブアサーション（`assert!(cargo.contains("35.8.0"), ...)`）。Cargo.toml を 35.9.0 にバンプする前にスタブ化が必要。

## ロードマップとの整合

`roadmap-v35.1-v36.0.md` の v35.9.0 計画（「E2E 動作確認・CHANGELOG 更新・v36.0 宣言文草案」）と本バージョンは整合している。

「v36.0 宣言文草案」については、`roadmap-v35.1-v36.0.md` の v36.0.0 セクション（177〜188 行）にすでに宣言文が記述されているため、本バージョンでの追加タスクは不要。`v36_deployment_story_planned` テストがロードマップの宣言文存在を間接確認する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `examples/lambda-deploy/fav.toml` が `"lambda"` を含む | `lambda_deploy_example_exists` テスト |
| 2 | `site/content/docs/deploy/lambda.mdx` が存在する | `deploy_docs_exists` テスト |
| 3 | `roadmap-v35.1-v36.0.md` に `"Deployment Story"` が含まれる | `v36_deployment_story_planned` テスト |
| 4 | `CHANGELOG.md` に `[v35.9.0]` が含まれる | `changelog_has_v35_9_0` テスト |
| 5 | `Cargo.toml` バージョンが `35.9.0` | `cargo_toml_version_is_35_9_0` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2651 = 前バージョン 2646 + 新規 5 件） | `cargo test` 実行結果 |
