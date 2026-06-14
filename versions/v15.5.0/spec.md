# v15.5.0 Spec — `fav deploy`（AWS Lambda デプロイ CLI）

Date: 2026-06-14

## 概要

`.fav` パイプラインを AWS Lambda として直接デプロイできるようにする。
`fav deploy` コマンドで zip 生成 → S3 アップロード → Lambda 更新 の 3 ステップを自動化する。

コア機能（`cmd_deploy`・`[deploy]` toml セクション・`--dry-run`）は v4.11.0 時点で実装済み。
v15.5.0 では以下の不足分を補完して完成宣言する。

## 実装内容

### 1. `DeployConfig` 拡張（toml.rs）

ロードマップ spec で定義されている `target` / `function_name` フィールドを追加:

```toml
[deploy]
target        = "aws-lambda"        # 対象プラットフォーム
function_name = "my-pipeline"       # Lambda 関数名
role_arn      = "arn:aws:iam::..."
runtime       = "provided.al2023"
region        = "ap-northeast-1"
memory_mb     = 512
timeout_sec   = 300
s3_bucket     = "my-deploy-bucket"
```

### 2. `scripts/build-lambda-layer.sh`

`fav` バイナリを Lambda 用に cross-compile して zip にパッケージングするスクリプト:

- `cargo build --release --target x86_64-unknown-linux-musl`
- `bootstrap` シェルスクリプト（`fav run --legacy $FAV_FILE`）を同梱
- `function.zip` として出力

### 3. `site/content/docs/deploy.mdx`

`fav deploy` の使い方ドキュメント:

- `fav.toml [deploy]` 設定リファレンス
- `fav deploy --dry-run` の出力例
- AWS Lambda デプロイ手順ガイド

### 4. v155000_tests（3件）

| テスト名 | 内容 |
|---|---|
| `version_is_15_5_0` | Cargo.toml のバージョンが 15.5.0 |
| `deploy_toml_schema_parses` | `[deploy]` セクションの解析が正しく動作する |
| `deploy_cmd_exists` | driver.rs に `cmd_deploy` 関数が存在する |

## 完了条件

- `cargo test v155000` → 3/3 PASS
- `cargo test` → 既存テストのリグレッションなし
- `fav deploy --dry-run` が実行可能で正常出力
