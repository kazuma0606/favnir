#!/bin/bash
# scripts/build-and-push.sh
# favnir-lambda-compiler + favnir-lambda-executor を ECR にビルド・プッシュする
# リポジトリルート（favnir/）から実行すること
set -euo pipefail

AWS_REGION=ap-northeast-1
AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
ECR_BASE="${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com"
ECR_COMPILER="${ECR_BASE}/favnir-lambda-compiler"
ECR_EXECUTOR="${ECR_BASE}/favnir-lambda-executor"

echo "=== ECR login ==="
aws ecr get-login-password --region "$AWS_REGION" \
  | docker login --username AWS --password-stdin "$ECR_BASE"

# ECR リポジトリ作成（既存なら無視）
aws ecr create-repository --repository-name favnir-lambda-compiler --region "$AWS_REGION" 2>/dev/null || true
aws ecr create-repository --repository-name favnir-lambda-executor  --region "$AWS_REGION" 2>/dev/null || true

echo ""
echo "=== Build favnir-lambda-compiler ==="
docker build \
  -f infra/e2e-demo/lambda/docker/compiler/Dockerfile \
  -t favnir-lambda-compiler \
  .
docker tag favnir-lambda-compiler:latest "$ECR_COMPILER:latest"
docker push "$ECR_COMPILER:latest"
echo "Compiler image: $ECR_COMPILER:latest"

echo ""
echo "=== Build favnir-lambda-executor ==="
docker build \
  -f infra/e2e-demo/lambda/docker/executor/Dockerfile \
  -t favnir-lambda-executor \
  .
docker tag favnir-lambda-executor:latest "$ECR_EXECUTOR:latest"
docker push "$ECR_EXECUTOR:latest"
echo "Executor image: $ECR_EXECUTOR:latest"

echo ""
echo "=== Image verification ==="
echo "--- compiler: .fav check (expect: /app/src/pipeline.fav) ---"
docker run --rm --entrypoint /bin/sh favnir-lambda-compiler \
  -c 'find / -name "*.fav" 2>/dev/null; echo "(done)"'

echo "--- executor: .fav check (expect: empty) ---"
docker run --rm --entrypoint /bin/sh favnir-lambda-executor \
  -c 'find / -name "*.fav" 2>/dev/null; echo "(done)"'

echo ""
echo "ECR_COMPILER: $ECR_COMPILER:latest"
echo "ECR_EXECUTOR: $ECR_EXECUTOR:latest"
