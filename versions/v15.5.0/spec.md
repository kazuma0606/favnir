# v15.5.0 Spec — `fav deploy`（AWS Lambda デプロイ CLI）

Date: 2026-06-14
Branch: master

---

## テーマ

`.fav` パイプラインを AWS Lambda として直接デプロイできるようにする。
「書いたパイプラインをそのままクラウドで動かせる」体験を実現する。

`fav deploy` コマンドのコア実装（`cmd_deploy`・`[deploy]` toml セクション・`--dry-run`）は
v4.11.0 時点で実装済み。v15.5.0 では以下を補完して完成宣言する:

- `DeployConfig` に `target` / `function_name` フィールドを追加（ロードマップ仕様準拠）
- `scripts/build-lambda-layer.sh`（クロスコンパイル + zip パッケージング）
- `site/content/docs/deploy.mdx`（ユーザーガイド）
- `v155000_tests`（3 件）

---

## スコープ

### A: `DeployConfig` 拡張（toml.rs）

ロードマップで定義されている `target` / `function_name` フィールドを追加:

```toml
[deploy]
target        = "aws-lambda"          # 対象プラットフォーム（"aws-lambda" | "azure-function"）
function_name = "my-pipeline"         # Lambda 関数名（省略時はプロジェクト名）
role_arn      = "arn:aws:iam::..."    # Lambda 実行ロール ARN
runtime       = "provided.al2023"     # Lambda ランタイム
region        = "ap-northeast-1"      # AWS リージョン
memory_mb     = 512                   # メモリ割り当て（MB）
timeout_sec   = 300                   # タイムアウト（秒）
s3_bucket     = "my-deploy-bucket"    # デプロイ zip アップロード先 S3 バケット
```

`memory_mb` / `timeout_sec` は既存 `memory` / `timeout` のエイリアスとして許容。

### B: `fav deploy` コマンド（既実装）

v4.11.0 実装済みの 3 ステップフロー:

```
Step 1: Package .fav files → /tmp/<project>-<timestamp>.zip
Step 2: Upload to s3://<s3_bucket>/deploys/<project>/<timestamp>.zip
Step 3: Update Lambda function '<function_name>'
```

`--dry-run` フラグ: 実際のデプロイを行わずステップを表示するのみ。

### C: `scripts/build-lambda-layer.sh`

`fav` バイナリを Lambda 用に cross-compile して zip にパッケージングするスクリプト:

1. `cross build --release --target x86_64-unknown-linux-musl --bin fav`
2. `bootstrap` シェルスクリプト（`fav run --legacy $FAV_FILE`）を同梱
3. `function.zip` として出力

**前提**: `cross` crate + Docker が必要（CI/Linux 環境推奨）。

### D: `site/content/docs/deploy.mdx`

`fav deploy` ユーザーガイド:

- `fav.toml [deploy]` 設定リファレンス
- `fav deploy --dry-run` の出力例
- `scripts/build-lambda-layer.sh` の使い方
- 必要 IAM 権限リスト

### E: テスト（v155000_tests — 3 件）

1. `version_is_15_5_0`: Cargo.toml version == "15.5.0"
2. `deploy_toml_schema_parses`: `[deploy]` セクション（target / function_name / memory_mb / timeout_sec）が正しく解析される
3. `deploy_cmd_exists`: `driver.rs` に `fn cmd_deploy` が存在する

---

## 完了条件

1. `cargo test v155000` → 3/3 パス
2. `cargo test` → リグレッションなし
3. `Cargo.toml version == "15.5.0"`
4. `fav deploy --dry-run` が正常動作
5. `scripts/build-lambda-layer.sh` が存在する
6. `site/content/docs/deploy.mdx` が存在する

---

## 新規 Cargo 依存

なし。zip 生成は既存 `zip 0.6` を流用（`package_project_zip` は v4.11.0 実装済み）。

---

## 既知の制約・スコープ外

- Azure Function デプロイは v16.x 以降
- `fav build --target wasm32` からの Lambda へのデプロイは対象外
- ECR（コンテナイメージ）デプロイは対象外（zip デプロイのみ）
- バッチデプロイ（複数関数同時更新）は対象外
- Lambda レイヤーとしてのデプロイは対象外

---

## 参照

- `versions/roadmap-v15.1-v16.0.md` — v15.5.0 セクション
- `fav/src/driver.rs` — `cmd_deploy` / `package_project_zip` / `deploy_upload_to_s3` / `deploy_update_lambda`
- `fav/src/toml.rs` — `DeployConfig` 構造体
- `fav/src/main.rs` — `deploy` コマンドのルーティング
- `infra/e2e-demo/crosscloud/` — Lambda デプロイパターンの参考
