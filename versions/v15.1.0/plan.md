# v15.1.0 Plan — CrossCloud 認証層（HMAC + Cognito + ECS）

Date: 2026-06-13

---

## Phase A — Docker イメージ

### A-1: `infra/e2e-demo/crosscloud/docker/Dockerfile`

```dockerfile
FROM debian:bookworm-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/*

# fav バイナリ（x86_64-unknown-linux-musl でクロスコンパイル済み）
COPY fav /usr/local/bin/fav
RUN chmod +x /usr/local/bin/fav

# migrate.fav パイプライン（v15.0.0 から変更なし）
COPY migrate.fav /app/migrate.fav

WORKDIR /app

# 接続情報は ECS タスク定義の secrets / environment から注入
ENV DATABASE_URL=""
ENV AZURE_CONN_STR=""
ENV AZURE_STORAGE_ACCOUNT=""
ENV AZURE_STORAGE_KEY=""
ENV AZURE_CONTAINER="proof"

CMD ["fav", "run", "--legacy", "/app/migrate.fav"]
```

### A-2: `infra/e2e-demo/crosscloud/scripts/build-and-push.sh`

```bash
#!/bin/bash
# build-and-push.sh — fav を Linux 向けにクロスコンパイルし ECR に push する
set -euo pipefail

REGION="${AWS_REGION:-ap-northeast-1}"
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
REPO="crosscloud-fav"
ECR_URL="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/${REPO}"
TAG="${IMAGE_TAG:-latest}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCKER_DIR="${SCRIPT_DIR}/../docker"
FAV_DIR="${SCRIPT_DIR}/../../../../fav"

echo "[build] Cross-compiling fav for x86_64-unknown-linux-musl..."
(cd "$FAV_DIR" && cargo build --release --target x86_64-unknown-linux-musl)

echo "[build] Copying artifacts..."
cp "${FAV_DIR}/target/x86_64-unknown-linux-musl/release/fav" "${DOCKER_DIR}/fav"
cp "${SCRIPT_DIR}/../src/migrate.fav" "${DOCKER_DIR}/migrate.fav"

echo "[build] Building Docker image..."
docker build --platform linux/amd64 -t "${REPO}:${TAG}" "${DOCKER_DIR}"

echo "[push] Logging in to ECR..."
aws ecr get-login-password --region "$REGION" \
  | docker login --username AWS --password-stdin "${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com"

docker tag "${REPO}:${TAG}" "${ECR_URL}:${TAG}"
docker push "${ECR_URL}:${TAG}"

echo "[push] Done: ${ECR_URL}:${TAG}"
```

---

## Phase B — Terraform: `terraform/aws/ecs.tf`（ECR + ECS）

```hcl
# terraform/aws/ecs.tf

# ── ECR ──────────────────────────────────────────────────────────────────────

resource "aws_ecr_repository" "fav" {
  name                 = "crosscloud-fav"
  image_tag_mutability = "MUTABLE"
  force_delete         = true
}

# ── ECS Cluster ──────────────────────────────────────────────────────────────

resource "aws_ecs_cluster" "crosscloud" {
  name = "favnir-crosscloud"
}

# ── IAM: ECS Task Execution Role（ECR pull + CloudWatch logs）────────────────

data "aws_iam_policy_document" "ecs_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "ecs_execution" {
  name               = "favnir-crosscloud-ecs-execution"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json
}

resource "aws_iam_role_policy_attachment" "ecs_execution_managed" {
  role       = aws_iam_role.ecs_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

# Secrets Manager 参照権限（ECS がシークレットを注入するため）
resource "aws_iam_role_policy" "ecs_execution_secrets" {
  name = "ecs-execution-secrets"
  role = aws_iam_role.ecs_execution.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = ["secretsmanager:GetSecretValue"]
      Resource = [
        aws_secretsmanager_secret.rds_conn.arn,
        aws_secretsmanager_secret.azure_conn.arn,
        aws_secretsmanager_secret.azure_storage.arn,
      ]
    }]
  })
}

# ── IAM: ECS Task Role（RDS アクセス・ランタイム権限）───────────────────────

resource "aws_iam_role" "ecs_task" {
  name               = "favnir-crosscloud-ecs-task"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json
}

# ── Security Group: ECS Tasks ─────────────────────────────────────────────────

resource "aws_security_group" "ecs_tasks" {
  name   = "favnir-crosscloud-ecs-tasks"
  vpc_id = data.aws_vpc.default.id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# RDS SG に ECS SG からの 5432 を追加許可
resource "aws_security_group_rule" "rds_from_ecs" {
  type                     = "ingress"
  from_port                = 5432
  to_port                  = 5432
  protocol                 = "tcp"
  security_group_id        = aws_security_group.rds.id
  source_security_group_id = aws_security_group.ecs_tasks.id
}

# ── Secrets Manager: Azure 接続情報（ECS タスクに注入）──────────────────────

resource "aws_secretsmanager_secret" "azure_conn" {
  name = "favnir/crosscloud/azure_conn_str"
}
resource "aws_secretsmanager_secret_version" "azure_conn" {
  secret_id     = aws_secretsmanager_secret.azure_conn.id
  secret_string = var.azure_conn_str
}

resource "aws_secretsmanager_secret" "azure_storage" {
  name = "favnir/crosscloud/azure_storage"
}
resource "aws_secretsmanager_secret_version" "azure_storage" {
  secret_id     = aws_secretsmanager_secret.azure_storage.id
  secret_string = jsonencode({
    account   = var.azure_storage_account
    key       = var.azure_storage_key
    container = var.azure_container
  })
}

# ── CloudWatch Log Group ──────────────────────────────────────────────────────

resource "aws_cloudwatch_log_group" "ecs_migrate" {
  name              = "/ecs/favnir-crosscloud-migrate"
  retention_in_days = 7
}

# ── ECS Task Definition ───────────────────────────────────────────────────────

resource "aws_ecs_task_definition" "migrate" {
  family                   = "favnir-crosscloud-migrate"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "256"
  memory                   = "512"
  execution_role_arn       = aws_iam_role.ecs_execution.arn
  task_role_arn            = aws_iam_role.ecs_task.arn

  container_definitions = jsonencode([{
    name  = "migrate"
    image = "${aws_ecr_repository.fav.repository_url}:${var.ecr_image_tag}"

    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.ecs_migrate.name
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "migrate"
      }
    }

    secrets = [
      { name = "DATABASE_URL",          valueFrom = aws_secretsmanager_secret.rds_conn.arn },
      { name = "AZURE_CONN_STR",        valueFrom = "${aws_secretsmanager_secret.azure_conn.arn}" },
    ]

    environment = [
      { name = "AZURE_CONTAINER", value = var.azure_container },
    ]

    # AZURE_STORAGE_ACCOUNT / AZURE_STORAGE_KEY は Lambda が overrides で渡す
  }])
}
```

---

## Phase C — Terraform: `terraform/aws/auth.tf`（Cognito + API GW + Lambda + DynamoDB）

```hcl
# terraform/aws/auth.tf

# ── Cognito User Pool ─────────────────────────────────────────────────────────

resource "aws_cognito_user_pool" "crosscloud" {
  name = "favnir-crosscloud"

  password_policy {
    minimum_length    = 12
    require_uppercase = true
    require_lowercase = true
    require_numbers   = true
    require_symbols   = false
  }
}

resource "aws_cognito_user_pool_client" "crosscloud" {
  name         = "favnir-crosscloud-client"
  user_pool_id = aws_cognito_user_pool.crosscloud.id

  explicit_auth_flows = [
    "ALLOW_USER_PASSWORD_AUTH",
    "ALLOW_REFRESH_TOKEN_AUTH",
  ]

  # デモ用: シークレットなし（スクリプトから呼びやすくするため）
  generate_secret = false
}

# ── HMAC Secret ───────────────────────────────────────────────────────────────

resource "aws_secretsmanager_secret" "hmac_secret" {
  name = "favnir/crosscloud/hmac-secret"
}

resource "aws_secretsmanager_secret_version" "hmac_secret" {
  secret_id     = aws_secretsmanager_secret.hmac_secret.id
  secret_string = var.hmac_secret
}

# ── DynamoDB: nonce テーブル ──────────────────────────────────────────────────

resource "aws_dynamodb_table" "nonce" {
  name         = "favnir-crosscloud-nonce"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "nonce_id"

  attribute {
    name = "nonce_id"
    type = "S"
  }

  ttl {
    attribute_name = "expires_at"
    enabled        = true
  }
}

# ── Lambda: verifier ──────────────────────────────────────────────────────────

data "archive_file" "verifier" {
  type        = "zip"
  source_dir  = "${path.module}/../../lambda/verifier"
  output_path = "${path.module}/../../lambda/verifier.zip"
}

resource "aws_iam_role" "lambda_verifier" {
  name = "favnir-crosscloud-lambda-verifier"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })
}

resource "aws_iam_role_policy" "lambda_verifier_policy" {
  name = "lambda-verifier-policy"
  role = aws_iam_role.lambda_verifier.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["logs:CreateLogGroup", "logs:CreateLogStream", "logs:PutLogEvents"]
        Resource = "arn:aws:logs:*:*:*"
      },
      {
        Effect   = "Allow"
        Action   = ["secretsmanager:GetSecretValue"]
        Resource = [aws_secretsmanager_secret.hmac_secret.arn]
      },
      {
        Effect   = "Allow"
        Action   = ["dynamodb:PutItem"]
        Resource = [aws_dynamodb_table.nonce.arn]
      },
      {
        Effect   = "Allow"
        Action   = ["s3:PutObject"]
        Resource = ["${aws_s3_bucket.proof.arn}/auth-proof/*"]
      },
      {
        Effect   = "Allow"
        Action   = ["ecs:RunTask"]
        Resource = [aws_ecs_task_definition.migrate.arn]
      },
      {
        Effect   = "Allow"
        Action   = ["iam:PassRole"]
        Resource = [
          aws_iam_role.ecs_execution.arn,
          aws_iam_role.ecs_task.arn,
        ]
      }
    ]
  })
}

resource "aws_lambda_function" "verifier" {
  function_name    = "favnir-crosscloud-verifier"
  filename         = data.archive_file.verifier.output_path
  source_code_hash = data.archive_file.verifier.output_base64sha256
  role             = aws_iam_role.lambda_verifier.arn
  handler          = "handler.lambda_handler"
  runtime          = "python3.12"
  timeout          = 30

  environment {
    variables = {
      HMAC_SECRET_ARN   = aws_secretsmanager_secret.hmac_secret.arn
      NONCE_TABLE       = aws_dynamodb_table.nonce.name
      S3_PROOF_BUCKET   = aws_s3_bucket.proof.id
      ECS_CLUSTER_ARN   = aws_ecs_cluster.crosscloud.arn
      ECS_TASK_DEF_ARN  = aws_ecs_task_definition.migrate.arn
      ECS_SUBNETS       = join(",", data.aws_subnets.default.ids)
      ECS_SECURITY_GROUPS = aws_security_group.ecs_tasks.id
      AWS_ACCOUNT_REGION = var.aws_region
      AZURE_STORAGE_ACCOUNT = var.azure_storage_account
      AZURE_STORAGE_KEY     = var.azure_storage_key
    }
  }
}

# ── API Gateway HTTP API ──────────────────────────────────────────────────────

resource "aws_apigatewayv2_api" "crosscloud" {
  name          = "favnir-crosscloud"
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_authorizer" "cognito" {
  api_id           = aws_apigatewayv2_api.crosscloud.id
  authorizer_type  = "JWT"
  identity_sources = ["$request.header.Authorization"]
  name             = "cognito-authorizer"

  jwt_configuration {
    audience = [aws_cognito_user_pool_client.crosscloud.id]
    issuer   = "https://cognito-idp.${var.aws_region}.amazonaws.com/${aws_cognito_user_pool.crosscloud.id}"
  }
}

resource "aws_apigatewayv2_integration" "verifier" {
  api_id                 = aws_apigatewayv2_api.crosscloud.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.verifier.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "migrate" {
  api_id             = aws_apigatewayv2_api.crosscloud.id
  route_key          = "POST /migrate"
  authorization_type = "JWT"
  authorizer_id      = aws_apigatewayv2_authorizer.cognito.id
  target             = "integrations/${aws_apigatewayv2_integration.verifier.id}"
}

resource "aws_apigatewayv2_stage" "default" {
  api_id      = aws_apigatewayv2_api.crosscloud.id
  name        = "$default"
  auto_deploy = true
}

resource "aws_lambda_permission" "apigw" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.verifier.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.crosscloud.execution_arn}/*/*"
}
```

---

## Phase D — Terraform: `variables.tf` / `outputs.tf` 追記

### variables.tf 追記

```hcl
variable "ecr_image_tag" {
  description = "ECR image tag for fav container"
  type        = string
  default     = "latest"
}

variable "azure_conn_str" {
  description = "Azure PostgreSQL connection string"
  type        = string
  sensitive   = true
}

variable "azure_storage_account" {
  description = "Azure Storage Account name"
  type        = string
}

variable "azure_storage_key" {
  description = "Azure Storage Account key"
  type        = string
  sensitive   = true
}

variable "azure_container" {
  description = "Azure Blob container name"
  type        = string
  default     = "proof"
}

variable "hmac_secret" {
  description = "HMAC-SHA256 shared secret (min 32 bytes)"
  type        = string
  sensitive   = true
}
```

### outputs.tf 追記

```hcl
output "api_gateway_endpoint" {
  value = aws_apigatewayv2_api.crosscloud.api_endpoint
}

output "ecr_repository_url" {
  value = aws_ecr_repository.fav.repository_url
}

output "ecs_cluster_name" {
  value = aws_ecs_cluster.crosscloud.name
}

output "cognito_user_pool_id" {
  value = aws_cognito_user_pool.crosscloud.id
}

output "cognito_client_id" {
  value = aws_cognito_user_pool_client.crosscloud.id
}
```

---

## Phase E — Lambda: `lambda/verifier/handler.py`

```python
# lambda/verifier/handler.py
import os
import json
import hmac
import hashlib
import base64
import time
import boto3
from datetime import datetime, timezone, timedelta

# キャッシュ（Lambda コンテナ再利用時に Secrets Manager 呼び出しを省略）
_hmac_secret: str | None = None

def _get_hmac_secret() -> str:
    global _hmac_secret
    if _hmac_secret is not None:
        return _hmac_secret
    sm = boto3.client("secretsmanager")
    resp = sm.get_secret_value(SecretId=os.environ["HMAC_SECRET_ARN"])
    _hmac_secret = resp["SecretString"]
    return _hmac_secret


def _put_proof(bucket: str, key: str, body: dict) -> None:
    s3 = boto3.client("s3")
    s3.put_object(Bucket=bucket, Key=key, Body=json.dumps(body), ContentType="application/json")


def _reject(reason: str, request_id: str, bucket: str) -> dict:
    _put_proof(bucket, f"auth-proof/deny/{request_id}.json", {
        "status": "deny",
        "reason": reason,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "request_id": request_id,
    })
    return {"statusCode": 401, "body": json.dumps({"error": reason})}


def lambda_handler(event: dict, context) -> dict:
    request_id = context.aws_request_id
    bucket     = os.environ["S3_PROOF_BUCKET"]
    headers    = {k.lower(): v for k, v in event.get("headers", {}).items()}
    body_str   = event.get("body", "") or ""

    # ── ① タイムスタンプ検証 ─────────────────────────────────────────────────
    ts_str = headers.get("x-timestamp", "")
    if not ts_str:
        return _reject("missing X-Timestamp", request_id, bucket)
    try:
        ts = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
        now = datetime.now(timezone.utc)
        if abs((now - ts).total_seconds()) > 300:
            return _reject("timestamp out of range", request_id, bucket)
    except ValueError:
        return _reject("invalid X-Timestamp format", request_id, bucket)

    # ── ② nonce チェック ──────────────────────────────────────────────────────
    nonce = headers.get("x-nonce", "")
    if not nonce:
        return _reject("missing X-Nonce", request_id, bucket)
    ddb = boto3.client("dynamodb")
    expires_at = int(time.time()) + 600  # TTL: 10分
    try:
        ddb.put_item(
            TableName=os.environ["NONCE_TABLE"],
            Item={
                "nonce_id":   {"S": nonce},
                "expires_at": {"N": str(expires_at)},
            },
            ConditionExpression="attribute_not_exists(nonce_id)",
        )
    except ddb.exceptions.ConditionalCheckFailedException:
        return _reject("replayed nonce", request_id, bucket)

    # ── ③ HMAC-SHA256 署名検証 ────────────────────────────────────────────────
    sig_b64 = headers.get("x-signature", "")
    if not sig_b64:
        return _reject("missing X-Signature", request_id, bucket)

    method    = event.get("requestContext", {}).get("http", {}).get("method", "POST")
    path      = event.get("requestContext", {}).get("http", {}).get("path", "/migrate")
    body_hash = hashlib.sha256(body_str.encode()).hexdigest()
    string_to_sign = f"{method}\n{path}\n{ts_str}\n{nonce}\n{body_hash}"

    secret = _get_hmac_secret().encode()
    expected_sig = base64.b64encode(
        hmac.new(secret, string_to_sign.encode(), hashlib.sha256).digest()
    ).decode()

    # 定数時間比較（タイミング攻撃対策）
    if not hmac.compare_digest(expected_sig, sig_b64):
        return _reject("invalid HMAC signature", request_id, bucket)

    # ── ④ ECS RunTask ─────────────────────────────────────────────────────────
    ecs = boto3.client("ecs")
    subnets = os.environ["ECS_SUBNETS"].split(",")
    sg      = os.environ["ECS_SECURITY_GROUPS"]

    run_resp = ecs.run_task(
        cluster      = os.environ["ECS_CLUSTER_ARN"],
        taskDefinition = os.environ["ECS_TASK_DEF_ARN"],
        launchType   = "FARGATE",
        networkConfiguration = {
            "awsvpcConfiguration": {
                "subnets":        subnets,
                "securityGroups": [sg],
                "assignPublicIp": "ENABLED",
            }
        },
        overrides = {
            "containerOverrides": [{
                "name": "migrate",
                "environment": [
                    {"name": "AZURE_STORAGE_ACCOUNT", "value": os.environ["AZURE_STORAGE_ACCOUNT"]},
                    {"name": "AZURE_STORAGE_KEY",     "value": os.environ["AZURE_STORAGE_KEY"]},
                ]
            }]
        }
    )

    task_arn = run_resp["tasks"][0]["taskArn"] if run_resp.get("tasks") else None

    # ── ⑤ allow proof → S3 ───────────────────────────────────────────────────
    _put_proof(bucket, f"auth-proof/allow/{request_id}.json", {
        "status":     "allow",
        "task_arn":   task_arn,
        "timestamp":  datetime.now(timezone.utc).isoformat(),
        "request_id": request_id,
    })

    return {
        "statusCode": 202,
        "body": json.dumps({"status": "accepted", "task_arn": task_arn}),
    }
```

---

## Phase F — スクリプト

### `scripts/run_with_auth.sh`

```bash
#!/bin/bash
# run_with_auth.sh — Cognito JWT + HMAC 署名で API Gateway に POST し ECS タスクを起動する
set -euo pipefail

REGION="${AWS_REGION:-ap-northeast-1}"
API_ENDPOINT="${API_ENDPOINT:?set API_ENDPOINT to the API Gateway URL}"
COGNITO_CLIENT_ID="${COGNITO_CLIENT_ID:?set COGNITO_CLIENT_ID}"
COGNITO_USERNAME="${COGNITO_USERNAME:?set COGNITO_USERNAME}"
COGNITO_PASSWORD="${COGNITO_PASSWORD:?set COGNITO_PASSWORD}"
ECS_CLUSTER="${ECS_CLUSTER:-favnir-crosscloud}"

# 1. Cognito JWT 取得
echo "[auth] Getting Cognito JWT..."
AUTH_RESULT=$(aws cognito-idp initiate-auth \
  --auth-flow USER_PASSWORD_AUTH \
  --client-id "$COGNITO_CLIENT_ID" \
  --auth-parameters USERNAME="$COGNITO_USERNAME",PASSWORD="$COGNITO_PASSWORD" \
  --region "$REGION" \
  --query 'AuthenticationResult.IdToken' \
  --output text)

# 2. HMAC_SECRET を Secrets Manager から取得
echo "[auth] Fetching HMAC_SECRET..."
HMAC_SECRET=$(aws secretsmanager get-secret-value \
  --secret-id favnir/crosscloud/hmac-secret \
  --region "$REGION" \
  --query SecretString --output text)

# 3. StringToSign 構築
METHOD="POST"
PATH_="/migrate"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
NONCE=$(python3 -c "import uuid; print(str(uuid.uuid4()))")
BODY='{"job_type":"crosscloud-migration"}'
BODY_HASH=$(echo -n "$BODY" | openssl dgst -sha256 | awk '{print $2}')

STRING_TO_SIGN="${METHOD}
${PATH_}
${TIMESTAMP}
${NONCE}
${BODY_HASH}"

# 4. HMAC-SHA256 署名
SIGNATURE=$(printf '%s' "$STRING_TO_SIGN" | openssl dgst -sha256 -hmac "$HMAC_SECRET" -binary | base64)

# 5. API Gateway に POST
echo "[auth] Sending signed request to ${API_ENDPOINT}${PATH_}..."
RESPONSE=$(curl -s -w "\n%{http_code}" \
  -X POST "${API_ENDPOINT}${PATH_}" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${AUTH_RESULT}" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Nonce: ${NONCE}" \
  -H "X-Signature: ${SIGNATURE}" \
  -d "$BODY")

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
RESP_BODY=$(echo "$RESPONSE" | head -1)

echo "[auth] Response: HTTP ${HTTP_CODE} — ${RESP_BODY}"
[ "$HTTP_CODE" != "202" ] && { echo "[auth] REJECTED"; exit 1; }

# 6. ECS タスク完了待機
TASK_ARN=$(echo "$RESP_BODY" | python3 -c "import sys, json; print(json.load(sys.stdin).get('task_arn',''))" 2>/dev/null || echo "")
if [ -n "$TASK_ARN" ] && [ "$TASK_ARN" != "None" ]; then
  echo "[ecs] Waiting for task to complete: $TASK_ARN"
  aws ecs wait tasks-stopped --cluster "$ECS_CLUSTER" --tasks "$TASK_ARN" --region "$REGION"

  EXIT_CODE=$(aws ecs describe-tasks \
    --cluster "$ECS_CLUSTER" \
    --tasks "$TASK_ARN" \
    --region "$REGION" \
    --query 'tasks[0].containers[0].exitCode' \
    --output text)

  echo "[ecs] Task exit code: $EXIT_CODE"
  [ "$EXIT_CODE" = "0" ] && echo "[run] SUCCESS" || { echo "[run] FAILED (exit ${EXIT_CODE})"; exit 1; }
fi
```

### `scripts/reject_cases.sh`

```bash
#!/bin/bash
# reject_cases.sh — 4 ケースの拒否テスト（全て 401 が返ることを確認）
set -euo pipefail

REGION="${AWS_REGION:-ap-northeast-1}"
API_ENDPOINT="${API_ENDPOINT:?set API_ENDPOINT}"
COGNITO_CLIENT_ID="${COGNITO_CLIENT_ID:?set COGNITO_CLIENT_ID}"
COGNITO_USERNAME="${COGNITO_USERNAME:?set COGNITO_USERNAME}"
COGNITO_PASSWORD="${COGNITO_PASSWORD:?set COGNITO_PASSWORD}"

PASS=0; FAIL=0

expect_reject() {
  local label="$1"; shift
  local code
  code=$(curl -s -o /dev/null -w "%{http_code}" "$@")
  if [ "$code" = "401" ] || [ "$code" = "403" ]; then
    echo "[REJECT PASS] $label — HTTP ${code}"
    PASS=$((PASS+1))
  else
    echo "[REJECT FAIL] $label — expected 401/403, got ${code}"
    FAIL=$((FAIL+1))
  fi
}

# JWT 取得（正常ケース用）
JWT=$(aws cognito-idp initiate-auth \
  --auth-flow USER_PASSWORD_AUTH \
  --client-id "$COGNITO_CLIENT_ID" \
  --auth-parameters USERNAME="$COGNITO_USERNAME",PASSWORD="$COGNITO_PASSWORD" \
  --region "$REGION" \
  --query 'AuthenticationResult.IdToken' \
  --output text)

HMAC_SECRET=$(aws secretsmanager get-secret-value \
  --secret-id favnir/crosscloud/hmac-secret \
  --region "$REGION" \
  --query SecretString --output text)

BODY='{"job_type":"crosscloud-migration"}'

# ─── REJECT 1: X-Signature なし ─────────────────────────────────────────────
TS=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
NONCE=$(python3 -c "import uuid; print(str(uuid.uuid4()))")
expect_reject "[REJECT 1] No HMAC signature" \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${JWT}" \
  -H "X-Timestamp: ${TS}" \
  -H "X-Nonce: ${NONCE}" \
  -d "$BODY"

# ─── REJECT 2: 期限切れタイムスタンプ ───────────────────────────────────────
OLD_TS="2020-01-01T00:00:00Z"
NONCE=$(python3 -c "import uuid; print(str(uuid.uuid4()))")
BODY_HASH=$(echo -n "$BODY" | openssl dgst -sha256 | awk '{print $2}')
STS="POST\n/migrate\n${OLD_TS}\n${NONCE}\n${BODY_HASH}"
SIG=$(printf "$STS" | openssl dgst -sha256 -hmac "$HMAC_SECRET" -binary | base64)
expect_reject "[REJECT 2] Expired timestamp" \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${JWT}" \
  -H "X-Timestamp: ${OLD_TS}" \
  -H "X-Nonce: ${NONCE}" \
  -H "X-Signature: ${SIG}" \
  -d "$BODY"

# ─── REJECT 3: nonce リプレイ ─────────────────────────────────────────────────
TS=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
FIXED_NONCE="replay-nonce-00000000-0000-0000-0000-000000000001"
BODY_HASH=$(echo -n "$BODY" | openssl dgst -sha256 | awk '{print $2}')
STS="POST\n/migrate\n${TS}\n${FIXED_NONCE}\n${BODY_HASH}"
SIG=$(printf "$STS" | openssl dgst -sha256 -hmac "$HMAC_SECRET" -binary | base64)
# 1回目（nonce 登録）
curl -s -o /dev/null \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${JWT}" \
  -H "X-Timestamp: ${TS}" \
  -H "X-Nonce: ${FIXED_NONCE}" \
  -H "X-Signature: ${SIG}" \
  -d "$BODY"
# 2回目（リプレイ）
expect_reject "[REJECT 3] Nonce replay" \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${JWT}" \
  -H "X-Timestamp: ${TS}" \
  -H "X-Nonce: ${FIXED_NONCE}" \
  -H "X-Signature: ${SIG}" \
  -d "$BODY"

# ─── REJECT 4: JWT なし（API GW が弾く）────────────────────────────────────────
TS=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
NONCE=$(python3 -c "import uuid; print(str(uuid.uuid4()))")
expect_reject "[REJECT 4] No JWT token" \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TS}" \
  -H "X-Nonce: ${NONCE}" \
  -d "$BODY"

echo ""
echo "Reject tests: PASS=${PASS} FAIL=${FAIL}"
[ "$FAIL" -eq 0 ] && echo "ALL REJECT CASES PASS" || { echo "SOME REJECT CASES FAILED"; exit 1; }
```

---

## Phase G — Rust テスト: `v151000_tests` + バージョンバンプ

### G-1: `v151000_tests` モジュールを追加（`v150000_tests` の直前）

```rust
// ── v151000_tests (v15.1.0) — CrossCloud 認証層 ────────────────────────────
#[cfg(test)]
mod v151000_tests {
    fn crosscloud_base() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .join("infra/e2e-demo/crosscloud")
    }

    fn read_tf(path: &str) -> String {
        let full = crosscloud_base().join(path);
        std::fs::read_to_string(&full)
            .unwrap_or_else(|_| panic!("{} should exist", path))
    }

    #[test]
    fn version_is_15_1_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "15.1.0");
    }

    #[test]
    fn crosscloud_auth_structure() {
        let base = crosscloud_base();
        let required = [
            "lambda/verifier/handler.py",
            "docker/Dockerfile",
            "scripts/run_with_auth.sh",
            "scripts/reject_cases.sh",
            "scripts/build-and-push.sh",
        ];
        for path in &required {
            assert!(base.join(path).exists(),
                "infra/e2e-demo/crosscloud/{} should exist", path);
        }
    }

    #[test]
    fn crosscloud_terraform_has_ecs_and_ecr() {
        let tf = read_tf("terraform/aws/ecs.tf");
        assert!(tf.contains("aws_ecs_cluster"),
            "ecs.tf should contain aws_ecs_cluster");
        assert!(tf.contains("aws_ecr_repository"),
            "ecs.tf should contain aws_ecr_repository");
        assert!(tf.contains("aws_ecs_task_definition"),
            "ecs.tf should contain aws_ecs_task_definition");
    }

    #[test]
    fn crosscloud_terraform_has_cognito_and_apigw() {
        let tf = read_tf("terraform/aws/auth.tf");
        assert!(tf.contains("aws_cognito_user_pool"),
            "auth.tf should contain aws_cognito_user_pool");
        assert!(tf.contains("aws_apigatewayv2_api"),
            "auth.tf should contain aws_apigatewayv2_api");
    }

    #[test]
    fn crosscloud_terraform_has_dynamodb_nonce() {
        let tf = read_tf("terraform/aws/auth.tf");
        assert!(tf.contains("aws_dynamodb_table"),
            "auth.tf should contain aws_dynamodb_table");
        assert!(tf.contains("nonce"),
            "auth.tf dynamodb table should be for nonce");
    }
}
```

### G-2: `v150000_tests` の `version_is_15_0_0` を `>=` 比較に修正

```rust
assert!(env!("CARGO_PKG_VERSION") >= "15.0.0",
    "expected >= 15.0.0, got {}", env!("CARGO_PKG_VERSION"));
```

### G-3: `fav/Cargo.toml` バージョンを `"15.1.0"` にバンプ

---

## Phase H — `cargo test v151000` + 全件テスト

```bash
cargo test v151000  # 5 件全パス
cargo test          # 全件パス（リグレッションなし）
```

---

## Phase I — インフラ構築 + E2E 実行

```bash
cd infra/e2e-demo/crosscloud

# 1. AWS terraform（既存 + 認証層を追加）
cd terraform/aws
terraform init
terraform apply -auto-approve \
  -var="rds_password=<RDS_PW>" \
  -var="env_suffix=<SUFFIX>" \
  -var="azure_conn_str=<AZURE_CONN>" \
  -var="azure_storage_account=<AZ_ACCT>" \
  -var="azure_storage_key=<AZ_KEY>" \
  -var="hmac_secret=<HMAC_32BYTES>"

# 2. ECR に fav イメージを push
cd ../..
bash scripts/build-and-push.sh

# 3. Cognito テストユーザー作成
POOL_ID=$(terraform -chdir=terraform/aws output -raw cognito_user_pool_id)
aws cognito-idp admin-create-user \
  --user-pool-id "$POOL_ID" \
  --username testuser \
  --temporary-password "TmpPass1!" \
  --region ap-northeast-1
aws cognito-idp admin-set-user-password \
  --user-pool-id "$POOL_ID" \
  --username testuser \
  --password "StrongPass1!" \
  --permanent \
  --region ap-northeast-1

# 4. 拒否テスト（4ケース）
export API_ENDPOINT=$(terraform -chdir=terraform/aws output -raw api_gateway_endpoint)
export COGNITO_CLIENT_ID=$(terraform -chdir=terraform/aws output -raw cognito_client_id)
export COGNITO_USERNAME=testuser
export COGNITO_PASSWORD="StrongPass1!"
bash scripts/reject_cases.sh
# → REJECT PASS=4 FAIL=0

# 5. 正常ケース実行（migrate.fav が ECS で動く）
bash scripts/run_with_auth.sh
# → [run] SUCCESS

# 6. proof ファイル確認
BUCKET=$(terraform -chdir=terraform/aws output -raw s3_proof_bucket)
aws s3 ls "s3://${BUCKET}/auth-proof/"
# → allow/ と deny/ の両方に JSON が存在
```

---

## Phase J — コミット

```bash
git add infra/e2e-demo/crosscloud/docker/ \
        infra/e2e-demo/crosscloud/lambda/ \
        infra/e2e-demo/crosscloud/terraform/aws/ecs.tf \
        infra/e2e-demo/crosscloud/terraform/aws/auth.tf \
        infra/e2e-demo/crosscloud/terraform/aws/variables.tf \
        infra/e2e-demo/crosscloud/terraform/aws/outputs.tf \
        infra/e2e-demo/crosscloud/scripts/ \
        fav/src/driver.rs \
        fav/Cargo.toml \
        fav/Cargo.lock \
        versions/v15.1.0/

git commit -m "feat: v15.1.0 — CrossCloud 認証層（HMAC + Cognito + ECS Fargate）"
```
