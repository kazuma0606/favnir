# v36.0.0 spec — Deployment Story マイルストーン宣言

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v36.0.0 |
| テーマ | Deployment Story マイルストーン宣言・★クリーンアップ |
| 前提 | v35.9.0 COMPLETE — v36.0 前調整・安定化完了 |
| 完了条件 | `v36000_tests` 全テスト pass・`cargo test` 0 failures・`MILESTONE.md` 更新 |

## 背景と目的

v35.1〜v35.9 のスプリントで以下を達成した。本バージョンはこれらを統合して Deployment Story マイルストーンを正式宣言し、v36 世代に移行する。

### 達成内容

| バージョン | 内容 |
|---|---|
| v35.1.0 | `fav deploy --target lambda` — Lambda 自動デプロイ・bootstrap.zip パッケージング |
| v35.2.0 | `fav deploy --target docker` — Dockerfile 自動生成・`docker build` 実行 |
| v35.3.0 | `fav ci init` — GitHub Actions CI ワークフロー自動生成 |
| v35.4.0 | `!Effect` E0374 ハードエラー化 |
| v35.5.0 | Effect enum・effects フィールド・parse_effects_acc 完全削除 |
| v35.6.0 | ctx 構文統一（MDX 128 件）+ Production Ready 宣言 |
| v35.7.0 | `docs_server.rs !Effect` 完全除去 |
| v35.8.0 | LSP / error_catalog / MCP / help !Effect 廃止完結 |
| v35.9.0 | v36.0 前調整・安定化（E2E 確認・lambda-deploy デモ確認） |

## ロードマップとの差異

`roadmap-v35.1-v36.0.md` の v36.0 宣言文には `deploy.fav`・`fav rollback`・`fav deploy status` が含まれていたが、v35.4〜v35.8 が `!Effect` 廃止完結スプリントに切り替わったため未実装。

これらは後続スプリント（v36.1〜）に繰り越す。本バージョンの宣言文は「実際に実装された機能」に限定する。

また、ロードマップの完了条件「テスト数 3500+」は現時点で 2651 件であり未達。後続スプリントで増加させる。

ロードマップ記載の「GitHub Issues P1/P2 ラベル付きオープンバグ 0 件」条件は Favnir が OSS 公開前のため GitHub Issues が存在しない。本バージョンでは対象外とする。

## 実装スコープ

| ファイル | 変更内容 |
|---|---|
| `MILESTONE.md` | v36.0 Deployment Story 宣言セクション追加 |
| `CHANGELOG.md` | `## [v36.0.0]` エントリ追加 |
| `fav/src/driver.rs` | `v35900_tests::cargo_toml_version_is_35_9_0` スタブ化 |
| `fav/src/driver.rs` | `v36000_tests` モジュール（5 件）追加 |
| `fav/Cargo.toml` | バージョン `35.9.0` → `36.0.0` |
| ビルドキャッシュ | `cargo clean`（★クリーンアップ） |

## v36000_tests の設計

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_36_0_0` | Cargo.toml に `"36.0.0"` が含まれる |
| `changelog_has_v36_0_0` | `CHANGELOG.md` に `[v36.0.0]` が含まれる |
| `milestone_has_deployment_story` | `MILESTONE.md` に `"Deployment Story"` が含まれる |
| `deploy_lambda_fn_exists` | `driver.rs` に `pub fn cmd_deploy` が含まれる |
| `ci_init_yaml_fn_exists` | `driver.rs` に `generate_ci_yaml` が含まれる |

テスト 4・5 は `include_str!("driver.rs")` で driver.rs 自身を参照する（他のテストで実績済みのパターン）。

## 宣言文

```
fav deploy --target lambda で Lambda に自動デプロイし、
fav deploy --target docker で Docker イメージを生成し、
fav ci init で GitHub Actions CI を自動設定できる。
!Effect 廃止（v35.4〜v35.8）により、すべての API が ctx: AppCtx ベースに統一された。

これが Favnir v36.0 — Deployment Story の姿である。
```

## ★クリーンアップ

v36.0.0 は x.0.0 マイルストーンのため `cargo clean` が必須（v31〜v35 の x.0.0 と同規約）。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Deployment Story"` が含まれる | `milestone_has_deployment_story` テスト |
| 2 | `driver.rs` に `pub fn cmd_deploy` が含まれる | `deploy_lambda_fn_exists` テスト |
| 3 | `driver.rs` に `generate_ci_yaml` が含まれる | `ci_init_yaml_fn_exists` テスト |
| 4 | `CHANGELOG.md` に `[v36.0.0]` が含まれる | `changelog_has_v36_0_0` テスト |
| 5 | `Cargo.toml` バージョンが `36.0.0` | `cargo_toml_version_is_36_0_0` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2656） | `cargo test` 実行結果 |
| 7 | `cargo clean` 実施済み | T2 実行記録 |
| 8 | `examples/lambda-deploy/fav.toml` が存在し `lambda` を含む（v35900_tests で検証済み） | `cargo test` 実行で間接確認 |
