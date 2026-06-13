# v15.1.0 Tasks — CrossCloud 認証層（HMAC + Cognito + ECS Fargate）

Date: 2026-06-13
Branch: master

---

## Phase A — Docker イメージ

- [x] A-1: `infra/e2e-demo/crosscloud/docker/Dockerfile` を作成
  - `debian:bookworm-slim` ベース
  - `fav` バイナリ（x86_64-unknown-linux-musl）を `/usr/local/bin/fav` に配置
  - `migrate.fav` を `/app/migrate.fav` に配置
  - 環境変数デフォルト（`DATABASE_URL` / `AZURE_CONN_STR` 等）を宣言

- [x] A-2: `infra/e2e-demo/crosscloud/scripts/build-and-push.sh` を作成
  - `cargo build --release --target x86_64-unknown-linux-musl`
  - `docker build --platform linux/amd64`
  - ECR へ push（`aws ecr get-login-password | docker login`）

---

## Phase B — Terraform: ECS / ECR（`terraform/aws/ecs.tf`）

- [x] B-1: `infra/e2e-demo/crosscloud/terraform/aws/ecs.tf` を作成
  - `aws_ecr_repository` crosscloud-fav
  - `aws_ecs_cluster` favnir-crosscloud
  - `aws_iam_role` ecs-execution（ECR pull + CloudWatch logs + Secrets Manager 参照）
  - `aws_iam_role` ecs-task（RDS + Secrets Manager アクセス）
  - `aws_security_group` ecs-tasks + RDS SG への ingress ルール追加
  - `aws_secretsmanager_secret` azure_conn_str / azure_storage（ECS 注入用）
  - `aws_cloudwatch_log_group` /ecs/favnir-crosscloud-migrate
  - `aws_ecs_task_definition` migrate（Fargate, 256CPU/512MB, secrets 参照）

---

## Phase C — Terraform: 認証層（`terraform/aws/auth.tf`）

- [x] C-1: `infra/e2e-demo/crosscloud/terraform/aws/auth.tf` を作成
  - `aws_cognito_user_pool` favnir-crosscloud
  - `aws_cognito_user_pool_client`（`USER_PASSWORD_AUTH` 有効、シークレットなし）
  - `aws_secretsmanager_secret` hmac-secret（`favnir/crosscloud/hmac-secret`）
  - `aws_dynamodb_table` nonce（TTL 有効、PAY_PER_REQUEST）
  - `aws_iam_role` lambda-verifier（Secrets Manager + DynamoDB + S3 + ECS RunTask + IAM PassRole）
  - `aws_lambda_function` verifier（`package_type = "Image"` に変更、Favnir コンテナ）
  - `aws_apigatewayv2_api` crosscloud（HTTP API）
  - `aws_apigatewayv2_authorizer` cognito（JWT Authorizer）
  - `aws_apigatewayv2_integration` verifier（AWS_PROXY）
  - `aws_apigatewayv2_route` POST /migrate（JWT 認可）
  - `aws_apigatewayv2_stage` $default（auto_deploy）
  - `aws_lambda_permission` apigw → Lambda

- [x] C-2: `terraform/aws/variables.tf` に追記
  - `ecr_image_tag` / `azure_conn_str` / `azure_storage_account` / `azure_storage_key` / `azure_container` / `hmac_secret`

- [x] C-3: `terraform/aws/outputs.tf` に追記
  - `api_gateway_endpoint` / `ecr_repository_url` / `ecs_cluster_name` / `cognito_user_pool_id` / `cognito_client_id`

---

## Phase D — Lambda: `lambda/verifier/` (Option A: Favnir コンテナ)

> **変更**: Python handler.py → Favnir コンテナ（verifier.fav + bootstrap + Dockerfile）

- [x] D-1: `infra/e2e-demo/crosscloud/lambda/verifier/verifier.fav` を作成
  - `Crypto.hmac_sha256_raw` / `Crypto.sha256_raw` で HMAC 検証
  - `AWS.dynamo_put_item_cond_raw` でノンスチェック（リプレイ防止）
  - `AWS.ecs_run_task_raw` で ECS 移行タスク起動
  - `AWS.s3_put_object_raw` で証跡保存

- [x] D-2: `infra/e2e-demo/crosscloud/lambda/verifier/bootstrap` を作成
  - Lambda Runtime API ループ（`/invocation/next` → `fav run --legacy verifier.fav` → `/response`）
  - API GW HTTP v2 イベント解析（`jq`）、env var 設定
  - exit code で HTTP 200/401/409/500 を返却

- [x] D-3: `infra/e2e-demo/crosscloud/lambda/verifier/Dockerfile` を作成
  - `public.ecr.aws/lambda/provided:al2023` ベース
  - `jq` インストール、`fav` バイナリ + `verifier.fav` + `bootstrap` 配置

- [x] D-4: `fav/src/backend/vm.rs` に新 VM primitive 追加
  - `AWS.dynamo_put_item_cond_raw` — DynamoDB PutItem + ConditionExpression
  - `AWS.ecs_run_task_raw` — ECS Fargate RunTask（SigV4）

- [x] D-5: `terraform/aws/auth.tf` を更新
  - `aws_ecr_repository "verifier"` 追加
  - `aws_lambda_function "verifier"` を `package_type = "Image"` に変更
  - `data archive_file` を削除

- [x] D-6: `terraform/aws/outputs.tf` に `verifier_ecr_url` 追加

- [x] D-7: `scripts/build-and-push-verifier.sh` を作成

---

## Phase E — スクリプト

- [x] E-1: `scripts/run_with_auth.sh` を作成
  - Cognito `initiate-auth`（`USER_PASSWORD_AUTH`）で JWT 取得
  - Secrets Manager から `HMAC_SECRET` 取得
  - `StringToSign` 構築（Method / Path / Timestamp / Nonce / SHA256(Body)）
  - `openssl dgst -sha256 -hmac` で署名生成
  - API Gateway に署名付き POST
  - `202` 確認 → `aws ecs wait tasks-stopped` でポーリング
  - ECS タスクの exit code 確認（0 = SUCCESS）

- [x] E-2: `scripts/reject_cases.sh` を作成
  - `[REJECT 1]` X-Signature なし → 401
  - `[REJECT 2]` 期限切れタイムスタンプ → 401
  - `[REJECT 3]` nonce リプレイ（同一 nonce 2回目）→ 401
  - `[REJECT 4]` JWT なし（API GW が弾く）→ 401
  - 全ケース: PASS/FAIL カウント + 終了コード

---

## Phase F — Rust テスト + バージョンバンプ

- [x] F-1: `v151000_tests` モジュールを `fav/src/driver.rs` に追加（6 テスト）
  - `version_is_15_1_0`, `verifier_fav_parses`, `verifier_fav_has_hmac_and_nonce`
  - `verifier_fav_has_aws_effects`, `crosscloud_auth_infra_structure`, `new_vm_primitives_are_referenced`

- [x] F-2: `v150000_tests` の `version_is_15_0_0` を `>=` 比較に修正

- [x] F-3: `fav/Cargo.toml` バージョンを `"15.1.0"` にバンプ

- [x] F-4: `cargo test v151000` で 6 件全パス確認

---

## Phase G — 全テスト（ユニット）

- [x] G-1: `cargo test v151000` 全 6 件パス
- [x] G-2: `cargo test` 全件パス（1549/1549 — 既存 flaky test 除く）

---

## Phase H — インフラ構築 + E2E 実行（要 AWS/Azure 環境）

- [x] H-1: `terraform/aws` に v15.1.0 リソース追加
  - `terraform init && terraform apply`（新変数: `azure_conn_str` / `hmac_secret` 等）
  - 出力: `api_gateway_endpoint`, `ecr_repository_url`, `cognito_user_pool_id`, `cognito_client_id`
  - ※ `fav/Dockerfile.builder` を使い Linux x86_64 バイナリをビルドして Lambda コンテナに梱包

- [x] H-2: `scripts/build-and-push-verifier.sh` 実行
  - `docker build --no-cache -f fav/Dockerfile.builder` でクロスコンパイル
  - `docker buildx build --platform linux/amd64 --provenance=false` で Lambda イメージをビルド
  - ECR crosscloud-verifier に push 成功

- [x] H-3: Cognito テストユーザー作成
  - `aws cognito-idp admin-create-user` + `admin-set-user-password --permanent`

- [x] H-4: `bash scripts/reject_cases.sh`
  - **PASS=5 FAIL=0** を確認（2026-06-13 実証）
  - ケース 5（JWT なし / API GW 弾き）も追加して 5 ケース

- [x] H-5: `bash scripts/run_with_auth.sh`
  - HTTP 200 + ECS タスク起動確認（2026-06-13 実証）
  - task ARN: `arn:aws:ecs:ap-northeast-1:847333136058:task/favnir-crosscloud/b4f67567488a4cbb8b120d3abac4582f`

- [x] H-6: S3 proof ファイル確認
  - `s3://favnir-crosscloud-proof-dev/auth-proof/3e8142f8-...json`（176 bytes）
  - `s3://favnir-crosscloud-proof-dev/auth-proof/790cddfb-...json`（176 bytes）
  - 証跡 JSON: `{"status":"ok","request_id":"<uuid>","task_arn":"<ecs-task-arn>"}`

- [x] H-7: `terraform destroy`（課金リソース後片付け）— 2026-06-13 完了（36 resources destroyed）

---

## Phase I — コミット

- [x] I-1: コミット完了
  - `feat: v15.1.0 — CrossCloud E2E Phase H 完了（PASS=5 実証）`（db996de）
  - `docs: v15.1.0 アーキテクチャ・デバッグ記録 + v15.1.5 計画書 + インフラファイル整備`（0e35654）

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `lambda/verifier/verifier.fav` が存在する | [x] |
| `lambda/verifier/Dockerfile` / `bootstrap` が存在する | [x] |
| `docker/Dockerfile` が存在する | [x] |
| `terraform/aws/ecs.tf` に `aws_ecs_cluster` が含まれる | [x] |
| `terraform/aws/auth.tf` に `aws_cognito_user_pool` / `aws_apigatewayv2_api` が含まれる | [x] |
| `terraform/aws/auth.tf` に `aws_dynamodb_table` が含まれる | [x] |
| `vm.rs` に `AWS.dynamo_put_item_cond_raw` / `AWS.ecs_run_task_raw` が含まれる | [x] |
| `cargo test v151000` 全 6 件パス | [x] |
| `cargo test` 全件パス（リグレッションなし、1550 pass） | [x] |
| `CARGO_PKG_VERSION == "15.1.0"` | [x] |
| `scripts/reject_cases.sh` が PASS=5 FAIL=0 を出力する（要 AWS 環境） | [x] 2026-06-13 実証 |
| ECS Fargate タスクが起動する（要 AWS 環境） | [x] 2026-06-13 実証 |
| auth-proof が S3 に保存される（要 AWS 環境） | [x] 2026-06-13 実証 |
| `terraform destroy` 完了（36 resources） | [x] 2026-06-13 実証 |
| bootstrap デバッグログ削除（production 化） | [x] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v15.1.0/spec.md` | 仕様・スコープ |
| `versions/v15.1.0/plan.md` | 各フェーズの具体的な変更内容 |
| `versions/roadmap-v15.1-v16.0.md` | v15.1.0〜v16.0.0 ロードマップ |
| `infra/e2e-demo/crosscloud/plan.md` | CrossCloud フル版設計（Phase 1 仕様） |
| `infra/e2e-demo/fav2py/terraform/main.tf` | ECS / ECR Terraform パターン参考 |
| `infra/e2e-demo/crosscloud/src/migrate.fav` | v15.0.0 fav パイプライン（変更なし） |
