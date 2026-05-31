#!/bin/bash
# scripts/build-and-push.sh
# favnir/runtime + favnir/toolchain を ECR にビルド・プッシュする
# リポジトリルート（favnir/）から実行すること
set -euo pipefail

AWS_REGION=ap-northeast-1
BUCKET_NAME=favnir-e2e-demo
AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
ECR_RUNTIME="${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com/favnir-runtime"
ECR_TOOLCHAIN="${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com/favnir-toolchain"

echo "=== ECR login ==="
aws ecr get-login-password --region "$AWS_REGION" \
  | docker login --username AWS --password-stdin \
    "${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com"

# ECR リポジトリ作成（既存なら無視）
aws ecr create-repository --repository-name favnir-runtime   --region "$AWS_REGION" 2>/dev/null || true
aws ecr create-repository --repository-name favnir-toolchain --region "$AWS_REGION" 2>/dev/null || true

echo ""
echo "=== Build favnir/runtime (ECS Dockerfile を流用) ==="
docker build \
  -f infra/e2e-demo/ecs/docker/runtime/Dockerfile \
  -t favnir-runtime \
  .
docker tag favnir-runtime:latest "$ECR_RUNTIME:latest"
docker push "$ECR_RUNTIME:latest"
echo "Runtime image: $ECR_RUNTIME:latest"

echo ""
echo "=== Build favnir/toolchain ==="
docker build \
  -f infra/e2e-demo/eks/docker/toolchain/Dockerfile \
  -t favnir-toolchain \
  .
docker tag favnir-toolchain:latest "$ECR_TOOLCHAIN:latest"
docker push "$ECR_TOOLCHAIN:latest"
echo "Toolchain image: $ECR_TOOLCHAIN:latest"

echo ""
echo "=== Image verification ==="
echo "--- runtime: .fav check (expect: empty) ---"
docker run --rm --entrypoint /bin/sh favnir-runtime \
  -c 'find / -name "*.fav" 2>/dev/null; echo "(done)"'

echo "--- toolchain: .fav check (expect: /app/src/pipeline.fav) ---"
docker run --rm --entrypoint /bin/sh favnir-toolchain \
  -c 'find / -name "*.fav" 2>/dev/null; echo "(done)"'

echo ""
echo "ECR_RUNTIME:   $ECR_RUNTIME:latest"
echo "ECR_TOOLCHAIN: $ECR_TOOLCHAIN:latest"
