# Favnir E2E Demo — Lambda 版 実装計画

Date: 2026-05-31

## ディレクトリ構成

```
infra/e2e-demo/lambda/
├── spec.md                         本仕様書
├── plan.md                         本計画書
├── tasks.md                        タスクチェックリスト
├── docker/
│   ├── compiler/
│   │   └── Dockerfile              favnir-toolchain ベース（Lambda CMD 追加）
│   │   └── handler.sh              Lambda ハンドラースクリプト
│   └── executor/
│       └── Dockerfile              favnir-runtime ベース（Lambda CMD 追加）
│       └── handler.sh              Lambda ハンドラースクリプト
├── src/
│   └── pipeline.fav                ECS 版と同じ（RDS PostgreSQL 対応）
├── terraform/
│   ├── main.tf                     VPC Endpoint SQS 追加（ECS 版 VPC 流用）
│   ├── lambda.tf                   Lambda A/B + EventSourceMapping
│   ├── sqs.tf                      SQS キュー + DLQ
│   ├── iam.tf                      Lambda 実行ロール x2
│   ├── s3_trigger.tf               S3 イベント通知
│   ├── storage.tf                  既存 S3 バケット（data source）
│   ├── database.tf                 既存 RDS（data source）
│   ├── variables.tf
│   └── outputs.tf
└── scripts/
    ├── build-and-push.sh           Docker ビルド + ECR プッシュ
    ├── trigger.sh                  S3 に pipeline.fav を投入してデモ起動
    └── verify.sh                   証跡確認（PASS=6/FAIL=0）
```

---

## Phase 1 — Docker イメージ

### compiler イメージ

EKS 版 `favnir-toolchain` をベースに Lambda 用 CMD を追加。

```dockerfile
# docker/compiler/Dockerfile
FROM 847333136058.dkr.ecr.ap-northeast-1.amazonaws.com/favnir-toolchain:latest

COPY docker/compiler/handler.sh /handler.sh
RUN chmod +x /handler.sh

CMD ["/handler.sh"]
```

```bash
# docker/compiler/handler.sh
#!/bin/sh
set -euo pipefail
TS=$(date +%Y%m%d-%H%M%S)

echo "[${TS}] Compiler Lambda starting"

# 証跡: .fav ファイルの存在確認
echo "=== find / -name '*.fav' ===" > /tmp/fav-search.txt
find / -name "*.fav" 2>/dev/null >> /tmp/fav-search.txt
aws s3 cp /tmp/fav-search.txt \
  "s3://${BUCKET_NAME}/proof/lambda/compiler-pod-fav-search-${TS}.txt"

# S3 からソースを取得（イベントで渡された SOURCE_KEY）
aws s3 cp "s3://${BUCKET_NAME}/${SOURCE_KEY}" /tmp/pipeline.fav

# コンパイル
/usr/local/bin/fav build /tmp/pipeline.fav -o /tmp/pipeline.fvc

# アーティファクトを S3 にアップロード
aws s3 cp /tmp/pipeline.fvc "s3://${BUCKET_NAME}/artifacts/pipeline.fvc"

# SQS にメッセージ送信
aws sqs send-message \
  --queue-url "${SQS_QUEUE_URL}" \
  --message-body "{\"artifact_key\":\"artifacts/pipeline.fvc\",\"timestamp\":\"${TS}\"}"

echo "[${TS}] Compiler Lambda done"
```

### executor イメージ

```dockerfile
# docker/executor/Dockerfile
FROM 847333136058.dkr.ecr.ap-northeast-1.amazonaws.com/favnir-runtime:latest

COPY docker/executor/handler.sh /handler.sh
RUN chmod +x /handler.sh

CMD ["/handler.sh"]
```

```bash
# docker/executor/handler.sh
#!/bin/sh
set -euo pipefail
TS=$(date +%Y%m%d-%H%M%S)

echo "[${TS}] Executor Lambda starting"

# 証跡: .fav ファイルが 0 件であることを確認
echo "=== find / -name '*.fav' ===" > /tmp/fav-search.txt
find / -name "*.fav" 2>/dev/null >> /tmp/fav-search.txt
aws s3 cp /tmp/fav-search.txt \
  "s3://${BUCKET_NAME}/proof/lambda/executor-pod-fav-search-${TS}.txt"

# S3 からアーティファクトを取得
aws s3 cp "s3://${BUCKET_NAME}/artifacts/pipeline.fvc" /tmp/pipeline.fvc

# 実行（RDS → S3）
FAV_DB_URL="${DB_URL}" /usr/local/bin/fav exec /tmp/pipeline.fvc

echo "[${TS}] Executor Lambda done"
```

---

## Phase 2 — Terraform: SQS

```hcl
# terraform/sqs.tf

resource "aws_sqs_queue" "dlq" {
  name                      = "favnir-pipeline-dlq"
  message_retention_seconds = 86400  # 1 day
  tags = { Name = "favnir-pipeline-dlq" }
}

resource "aws_sqs_queue" "pipeline" {
  name                       = "favnir-pipeline"
  visibility_timeout_seconds = 300
  message_retention_seconds  = 3600
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.dlq.arn
    maxReceiveCount     = 3
  })
  tags = { Name = "favnir-pipeline" }
}
```

---

## Phase 3 — Terraform: VPC Endpoint (SQS)

ECS 版 VPC に SQS Interface Endpoint を追加（data source で既存 VPC を参照）。

```hcl
# terraform/main.tf

# 既存 VPC/Subnet を data source で参照
data "aws_vpc" "demo" {
  filter {
    name   = "tag:Name"
    values = ["favnir-ecs-demo"]
  }
}

data "aws_subnet" "private" {
  filter {
    name   = "tag:Name"
    values = ["favnir-ecs-private"]
  }
}

data "aws_security_group" "endpoints" {
  filter {
    name   = "tag:Name"
    values = ["favnir-ecs-endpoints"]
  }
}

# SQS VPC Endpoint（Lambda in VPC から SQS に接続するために必要）
resource "aws_vpc_endpoint" "sqs" {
  vpc_id              = data.aws_vpc.demo.id
  service_name        = "com.amazonaws.${var.aws_region}.sqs"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [data.aws_subnet.private.id]
  security_group_ids  = [data.aws_security_group.endpoints.id]
  private_dns_enabled = true
  tags                = { Name = "favnir-lambda-sqs" }
}
```

---

## Phase 4 — Terraform: IAM

```hcl
# terraform/iam.tf

# Compiler Lambda 実行ロール
resource "aws_iam_role" "lambda_compiler" {
  name = "favnir-lambda-compiler"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy" "lambda_compiler" {
  name = "favnir-lambda-compiler-policy"
  role = aws_iam_role.lambda_compiler.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["s3:GetObject"]
        Resource = "arn:aws:s3:::favnir-e2e-demo/source/*"
      },
      {
        Effect   = "Allow"
        Action   = ["s3:PutObject"]
        Resource = [
          "arn:aws:s3:::favnir-e2e-demo/artifacts/*",
          "arn:aws:s3:::favnir-e2e-demo/proof/lambda/*"
        ]
      },
      {
        Effect   = "Allow"
        Action   = ["sqs:SendMessage"]
        Resource = aws_sqs_queue.pipeline.arn
      },
      {
        Effect   = "Allow"
        Action   = ["logs:CreateLogGroup", "logs:CreateLogStream", "logs:PutLogEvents"]
        Resource = "arn:aws:logs:*:*:*"
      },
      {
        Effect   = "Allow"
        Action   = ["ec2:CreateNetworkInterface", "ec2:DescribeNetworkInterfaces", "ec2:DeleteNetworkInterface"]
        Resource = "*"
      }
    ]
  })
}

# Executor Lambda 実行ロール（同パターン、S3 GetObject + SQS 受信権限）
resource "aws_iam_role" "lambda_executor" { ... }
resource "aws_iam_role_policy" "lambda_executor" { ... }
```

---

## Phase 5 — Terraform: Lambda 関数

```hcl
# terraform/lambda.tf

locals {
  ecr_compiler = "${var.aws_account}.dkr.ecr.${var.aws_region}.amazonaws.com/favnir-lambda-compiler"
  ecr_executor = "${var.aws_account}.dkr.ecr.${var.aws_region}.amazonaws.com/favnir-lambda-executor"
}

resource "aws_lambda_function" "compiler" {
  function_name = "favnir-compiler"
  role          = aws_iam_role.lambda_compiler.arn
  package_type  = "Image"
  image_uri     = "${local.ecr_compiler}:latest"
  timeout       = 120
  memory_size   = 512

  vpc_config {
    subnet_ids         = [data.aws_subnet.private.id]
    security_group_ids = [aws_security_group.lambda.id]
  }

  environment {
    variables = {
      BUCKET_NAME   = "favnir-e2e-demo"
      SQS_QUEUE_URL = aws_sqs_queue.pipeline.url
    }
  }
}

resource "aws_lambda_function" "executor" {
  function_name = "favnir-executor"
  role          = aws_iam_role.lambda_executor.arn
  package_type  = "Image"
  image_uri     = "${local.ecr_executor}:latest"
  timeout       = 300
  memory_size   = 512

  vpc_config {
    subnet_ids         = [data.aws_subnet.private.id]
    security_group_ids = [aws_security_group.lambda.id]
  }

  environment {
    variables = {
      BUCKET_NAME = "favnir-e2e-demo"
      DB_URL      = var.db_url  # Secrets Manager or SSM から取得
    }
  }
}

# SQS → Lambda B のトリガー
resource "aws_lambda_event_source_mapping" "sqs_to_executor" {
  event_source_arn = aws_sqs_queue.pipeline.arn
  function_name    = aws_lambda_function.executor.arn
  batch_size       = 1
}
```

---

## Phase 6 — Terraform: S3 イベント通知

```hcl
# terraform/s3_trigger.tf

resource "aws_s3_bucket_notification" "compiler_trigger" {
  bucket = "favnir-e2e-demo"

  lambda_function {
    lambda_function_arn = aws_lambda_function.compiler.arn
    events              = ["s3:ObjectCreated:*"]
    filter_prefix       = "source/"
    filter_suffix       = ".fav"
  }

  depends_on = [aws_lambda_permission.s3_invoke_compiler]
}

resource "aws_lambda_permission" "s3_invoke_compiler" {
  statement_id  = "AllowS3Invoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.compiler.function_name
  principal     = "s3.amazonaws.com"
  source_arn    = "arn:aws:s3:::favnir-e2e-demo"
}
```

---

## Phase 7 — スクリプト

### scripts/build-and-push.sh

```bash
#!/bin/bash
# ECS 版の favnir-runtime + EKS 版の favnir-toolchain を Lambda 用にラップして ECR プッシュ
# リポジトリルート（favnir/）から実行すること

AWS_REGION=ap-northeast-1
AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
ECR_BASE="${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com"

aws ecr get-login-password --region $AWS_REGION | \
  docker login --username AWS --password-stdin $ECR_BASE

aws ecr create-repository --repository-name favnir-lambda-compiler --region $AWS_REGION 2>/dev/null || true
aws ecr create-repository --repository-name favnir-lambda-executor  --region $AWS_REGION 2>/dev/null || true

# Compiler イメージ（toolchain ベース + handler.sh）
docker build -f infra/e2e-demo/lambda/docker/compiler/Dockerfile \
  -t favnir-lambda-compiler .
docker tag favnir-lambda-compiler:latest \
  "$ECR_BASE/favnir-lambda-compiler:latest"
docker push "$ECR_BASE/favnir-lambda-compiler:latest"

# Executor イメージ（runtime ベース + handler.sh）
docker build -f infra/e2e-demo/lambda/docker/executor/Dockerfile \
  -t favnir-lambda-executor .
docker tag favnir-lambda-executor:latest \
  "$ECR_BASE/favnir-lambda-executor:latest"
docker push "$ECR_BASE/favnir-lambda-executor:latest"
```

### scripts/trigger.sh

```bash
#!/bin/bash
# S3 に pipeline.fav を投入してデモを開始する
BUCKET=favnir-e2e-demo
aws s3 cp src/pipeline.fav "s3://${BUCKET}/source/pipeline.fav"
echo "Uploaded source/pipeline.fav. Lambda will trigger automatically."
echo "Monitor: aws logs tail /aws/lambda/favnir-compiler --follow"
```

### scripts/verify.sh

EKS 版と同パターン（proof/lambda/ プレフィックス対応）で 6 チェック。

---

## Phase 8 — デプロイと検証

```bash
# 1. Docker ビルド + ECR プッシュ（リポジトリルートから）
bash infra/e2e-demo/lambda/scripts/build-and-push.sh

# 2. Terraform apply（RDS 接続情報が必要）
cd infra/e2e-demo/lambda/terraform
terraform init
terraform apply -var="db_url=postgres://favnir:<pass>@<rds-endpoint>:5432/favnirdb"

# 3. デモ起動（S3 投入）
cd ..
bash scripts/trigger.sh

# 4. ログ確認
aws logs tail /aws/lambda/favnir-compiler --follow
aws logs tail /aws/lambda/favnir-executor --follow

# 5. 証跡確認
bash scripts/verify.sh
# 期待: PASS=6 / FAIL=0
```

---

## 既知の制約・注意点

| 制約 | 詳細 | 対策 |
|---|---|---|
| Lambda コンテナと RIE | Lambda コンテナは通常 Lambda Runtime Interface を実装する必要があるが、シェルスクリプトを直接実行する場合は `AWS_LAMBDA_RUNTIME_API` が存在しない | `handler.sh` を直接 CMD として実行（バッチ的に動かす） |
| Lambda コールドスタート | コンテナイメージは初回起動が遅い（~10 秒） | timeout を余裕を持って設定 |
| Lambda in VPC の ENI | VPC 内 Lambda は ENI 作成に時間がかかる | 初回デプロイ後は Warmed Up 状態が維持される |
| SQS → Lambda のバッチ処理 | batch_size=1 にして 1 メッセージずつ処理 | デモ用途なので問題なし |
| `fav exec` と DB 接続 | `FAV_DB_URL` 環境変数で RDS に直接接続 | RDS SG に Lambda SG からの 5432 許可が必要 |
| ECS 版 RDS 流用 | Lambda のセキュリティグループを ECS 版 RDS の inbound に追加する必要がある | Terraform で `aws_security_group_rule` を追加 |
