# Favnir E2E Demo — Lambda 版

S3 イベント駆動で Favnir パイプラインを Lambda コンテナ + SQS + Aurora PostgreSQL で実行する E2E デモ。

## 実行結果

```
=== Favnir Lambda E2E Demo — 証跡確認 ===

[PASS] Compiler Lambda: 証跡ファイルが S3 に存在する
[PASS] Compiler Lambda: pipeline.fav が存在する（compiler イメージ）
[PASS] S3: artifacts/pipeline.fvc が存在する
[PASS] Executor Lambda: 証跡ファイルが S3 に存在する
[PASS] Executor Lambda: .fav ファイルが 0 件（runtime イメージ）
[PASS] サマリー JSON が S3/output/ に存在する

結果: PASS=6 / FAIL=0
```

実行日: 2026-05-31

## アーキテクチャ

```
S3 source/pipeline.fav
        │
        │ ObjectCreated イベント
        ▼
┌─────────────────────────────┐
│  Lambda A: favnir-compiler  │  (コンテナ, 512MB, timeout=120s)
│  ① 証跡収集 → S3            │
│  ② fav build pipeline.fav  │
│  ③ pipeline.fvc → S3       │
│  ④ SQS send-message        │
└─────────────────────────────┘
        │
        │ SQS EventSourceMapping (batch_size=1)
        ▼
┌─────────────────────────────┐
│  Lambda B: favnir-executor  │  (コンテナ, 512MB, timeout=300s)
│  ① 証跡収集 → S3            │
│  ② pipeline.fvc ← S3       │
│  ③ fav exec → RDS Aurora   │
│  ④ summary-latest.json → S3│
└─────────────────────────────┘
        │
        ▼
Aurora Serverless v2 (PostgreSQL 16.6)
S3 output/summary-latest.json
```

### 証跡の確認内容

| チェック | 確認事項 |
|---|---|
| Compiler 証跡ファイル存在 | `proof/lambda/compiler-pod-fav-search-*.txt` |
| Compiler イメージに `.fav` あり | toolchain イメージに `pipeline.fav` が含まれる |
| `artifacts/pipeline.fvc` 存在 | コンパイル成功の証跡 |
| Executor 証跡ファイル存在 | `proof/lambda/executor-pod-fav-search-*.txt` |
| Executor イメージに `.fav` 0 件 | runtime イメージはソース非同梱 |
| `output/summary-latest.json` 存在 | パイプライン完走の証跡 |

## ECS / EKS との比較

| 項目 | ECS (Fargate) | EKS (Fargate) | Lambda (Container) |
|---|---|---|---|
| 起動形態 | コンテナタスク | Pod (Job) | イベント駆動関数 |
| トリガー | S3 → SQS → ECS Task | kubectl apply (手動) | S3 イベント通知 |
| スケール | タスク定義 | Job replica | 自動 (EventSourceMapping) |
| Compiler 実行時間 | ~15s | ~20s | ~9s |
| Executor 実行時間 | ~30s | ~35s | ~43s |
| RDS | Aurora Serverless v2 | Aurora Serverless v2 | Aurora Serverless v2 |
| VPC Endpoint | 5種 | 5種 | 6種 (SQS追加) |
| イメージ種別 | Linux/amd64 | Linux/amd64 | Linux/amd64 (single-platform) |

## 実行手順

### 前提条件

- AWS CLI 設定済み（ap-northeast-1）
- Terraform >= 1.0
- Docker（buildx 有効）
- S3 バケット `favnir-e2e-demo` が存在すること

### 1. Docker イメージをビルドして ECR にプッシュ

```bash
# ECR ログイン
aws ecr get-login-password --region ap-northeast-1 \
  | docker login --username AWS --password-stdin \
    847333136058.dkr.ecr.ap-northeast-1.amazonaws.com

# Compiler イメージ（Favnir repo ルートから実行）
docker build --platform linux/amd64 --provenance=false \
  -f infra/e2e-demo/lambda/docker/compiler/Dockerfile \
  -t 847333136058.dkr.ecr.ap-northeast-1.amazonaws.com/favnir-lambda-compiler:latest .
docker push 847333136058.dkr.ecr.ap-northeast-1.amazonaws.com/favnir-lambda-compiler:latest

# Executor イメージ
docker build --platform linux/amd64 --provenance=false \
  -f infra/e2e-demo/lambda/docker/executor/Dockerfile \
  -t 847333136058.dkr.ecr.ap-northeast-1.amazonaws.com/favnir-lambda-executor:latest .
docker push 847333136058.dkr.ecr.ap-northeast-1.amazonaws.com/favnir-lambda-executor:latest
```

### 2. Terraform でインフラを構築

```bash
cd infra/e2e-demo/lambda/terraform
terraform init
terraform apply -var="db_password=<パスワード>"
```

### 3. RDS データベースをシード

```bash
CLUSTER_ARN=$(aws rds describe-db-clusters \
  --db-cluster-identifier favnir-lambda-demo \
  --query 'DBClusters[0].DBClusterArn' --output text)

# Secrets Manager にシークレットを作成（Data API 用）
# ...

# orders テーブル作成
aws rds-data execute-statement \
  --resource-arn "$CLUSTER_ARN" --secret-arn "$SECRET_ARN" \
  --database favnirdb \
  --sql "CREATE TABLE IF NOT EXISTS orders (id SERIAL PRIMARY KEY, item TEXT NOT NULL, amount INT NOT NULL, created_at TIMESTAMP DEFAULT NOW())"

# シードデータ挿入
aws rds-data execute-statement \
  --resource-arn "$CLUSTER_ARN" --secret-arn "$SECRET_ARN" \
  --database favnirdb \
  --sql "INSERT INTO orders (item, amount) VALUES ('widget-a', 100), ('widget-b', 200), ('widget-c', 150)"
```

### 4. デモを実行

```bash
cd infra/e2e-demo/lambda
bash scripts/trigger.sh
# → source/pipeline.fav を S3 に投入し、Lambda が自動起動
```

### 5. 証跡を確認

```bash
# Lambda ログ確認（約 60〜90 秒後）
MSYS_NO_PATHCONV=1 aws logs tail '/aws/lambda/favnir-compiler' --since 5m --region ap-northeast-1
MSYS_NO_PATHCONV=1 aws logs tail '/aws/lambda/favnir-executor' --since 5m --region ap-northeast-1

# 証跡確認スクリプト
bash scripts/verify.sh
# → PASS=6 / FAIL=0
```

### 6. クリーンアップ

```bash
cd infra/e2e-demo/lambda/terraform
terraform destroy -var="db_password=<パスワード>"
# S3 バケットは削除されない（証跡保存）
```

## 既知の制約

| 制約 | 内容 |
|---|---|
| Lambda コンテナ形式 | OCI manifest list 非対応。`--platform linux/amd64 --provenance=false` でビルド必須 |
| Lambda Runtime | AWS base image 非使用のため Custom Runtime bootstrap が必要 |
| `AWS_DEFAULT_REGION` | Lambda 予約済み環境変数。Terraform で設定不可（Lambda が自動設定） |
| `find /` の exit code | `set -e` 環境では `|| true` が必要（`/proc` アクセス失敗で非 0 終了） |
| Aurora Data API | `enable_http_endpoint = true` + Secrets Manager シークレット必須 |
| VPC + Lambda | NAT Gateway なし。SQS Interface endpoint が必要（S3 は Gateway で無料） |
