# Favnir E2E Demo — Lambda 版 タスクリスト

Date: 2026-05-31

## Phase 1 — Docker イメージ

### 1-A: favnir-lambda-compiler
- [x] `docker/compiler/handler.sh` を作成
  - 証跡収集（find *.fav → S3）
  - S3 からソース取得 (`source/pipeline.fav`)
  - `fav build` → `/tmp/pipeline.fvc`
  - `aws s3 cp` でアーティファクトを S3 に
  - `aws sqs send-message` で SQS に通知
- [x] `docker/compiler/Dockerfile` を作成
  - `FROM debian:bookworm-slim`（Rust builder stage + runtime stage）
  - `COPY handler.sh` + Lambda Custom Runtime `bootstrap`
- [x] ECR に `favnir-lambda-compiler` リポジトリを作成

### 1-B: favnir-lambda-executor
- [x] `docker/executor/handler.sh` を作成
  - 証跡収集（find *.fav → S3, 0 件確認）
  - S3 から `pipeline.fvc` をダウンロード
  - `FAV_DB_URL=... fav exec pipeline.fvc`
  - report-latest.json → summary-latest.json へコピー
- [x] `docker/executor/Dockerfile` を作成
  - `.fav` ファイルはコピーしない（runtime イメージ）
- [x] ECR に `favnir-lambda-executor` リポジトリを作成

### 1-C: Lambda Custom Runtime bootstrap
- [x] `docker/compiler/bootstrap` を作成（Lambda Runtime API ループ）
- [x] `docker/executor/bootstrap` を作成（Lambda Runtime API ループ）

---

## Phase 2 — src/pipeline.fav

### 2-A: pipeline.fav の確認・流用
- [x] ECS 版 `src/pipeline.fav` をコピー
  - RDS PostgreSQL 対応（`DB.connect` / `DB.query_raw`）
  - `AWS.s3_put_object_raw` で S3 書き込み
  - `runner: "lambda"` として記録

---

## Phase 3 — Terraform: SQS

### 3-A: SQS キュー
- [x] `terraform/sqs.tf` を作成
  - `favnir-pipeline` Standard Queue
  - `favnir-pipeline-dlq` Dead Letter Queue

---

## Phase 4 — Terraform: VPC / ネットワーク

### 4-A: 新規 VPC（10.2.0.0/16）
- [x] `terraform/main.tf` を作成
  - private_a (ap-northeast-1a) / private_b (ap-northeast-1c)
  - Lambda / RDS / Endpoints Security Groups
  - VPC Endpoints: S3 Gateway, SQS/ECR dkr/ECR api/CW Logs/STS Interface（全て両 AZ）

---

## Phase 5 — Terraform: IAM

### 5-A: Lambda Compiler 実行ロール
- [x] `terraform/iam.tf` に `aws_iam_role.lambda_compiler` を作成

### 5-B: Lambda Executor 実行ロール
- [x] `aws_iam_role.lambda_executor` を作成

---

## Phase 6 — Terraform: Lambda 関数

### 6-A: Lambda A（compiler）
- [x] `terraform/lambda.tf` に `aws_lambda_function.compiler` を作成
  - `package_type = "Image"`, `timeout = 120`, `memory_size = 512`
  - 環境変数: `BUCKET_NAME`, `SQS_QUEUE_URL`

### 6-B: Lambda B（executor）
- [x] `aws_lambda_function.executor` を作成
  - `timeout = 300`, `memory_size = 512`
  - 環境変数: `BUCKET_NAME`, `DB_URL`

### 6-C: SQS → Lambda B EventSourceMapping
- [x] `aws_lambda_event_source_mapping.sqs_to_executor` を作成（`batch_size = 1`）

---

## Phase 7 — Terraform: S3 イベント通知

### 7-A: S3 → Lambda A トリガー
- [x] `terraform/s3_trigger.tf` を作成
  - `aws_s3_bucket_notification` — `source/*.fav` ObjectCreated
  - `aws_lambda_permission` — S3 から Lambda を invoke する権限

---

## Phase 8 — Terraform: ストレージ + DB

### 8-A: リソース作成
- [x] `terraform/database.tf` — Aurora Serverless v2 (PostgreSQL 16.6, 0.5-1.0 ACU)
- [x] `terraform/variables.tf` — `db_password` (sensitive), `aws_region`, `aws_account`
- [x] `terraform/outputs.tf` — Lambda ARN, SQS URL, RDS endpoint

---

## Phase 9 — スクリプト

### 9-A: trigger.sh
- [x] `scripts/trigger.sh` を作成

### 9-B: verify.sh
- [x] `scripts/verify.sh` を作成

---

## Phase 10 — デプロイと検証

### 10-A: terraform apply
- [x] `terraform init` + `terraform apply`
  - 合計 35 リソース作成

### 10-B: デモ実行
- [x] RDS `orders` テーブル作成（Data API 経由）
- [x] 3 行シード挿入
- [x] `bash scripts/trigger.sh` で S3 に pipeline.fav を投入
- [x] Compiler Lambda: 9s で完了
- [x] SQS → Executor Lambda: 43s で完了

### 10-C: 証跡確認
- [x] `bash scripts/verify.sh` → **PASS=6 / FAIL=0**

---

## Phase 11 — README + クリーンアップ

### 11-A: README.md の作成
- [x] 実行結果サマリー（PASS=6/FAIL=0）
- [x] アーキテクチャ図
- [x] ECS/EKS との比較表
- [x] 実行手順

### 11-B: クリーンアップ
- [x] `terraform destroy`（S3 は残す）

---

## 完了条件サマリー

| 確認項目 | 担当 | 状態 |
|---|---|---|
| compiler Lambda に `.fav` が存在 | docker/compiler | ✓ PASS |
| executor Lambda に `.fav` が 0 件 | docker/executor | ✓ PASS |
| S3 投入 → compiler Lambda 自動起動 | s3_trigger.tf | ✓ PASS |
| compiler → SQS → executor 連携 | sqs.tf + lambda.tf | ✓ PASS |
| executor が RDS → S3 パイプラインを完走 | executor handler.sh | ✓ PASS |
| `bash scripts/verify.sh` → PASS=6/FAIL=0 | verify.sh | ✓ PASS |
| `terraform destroy` 後コストゼロ | — | ✓ 実施済み |
