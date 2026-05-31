#!/bin/bash
# Favnir E2E Demo — Docker イメージビルド + ECR プッシュ + EC2 用バイナリ配置
# リポジトリルートから実行すること:
#   bash infra/e2e-demo/ecs/scripts/build-and-push.sh
set -e

AWS_REGION=${AWS_REGION:-ap-northeast-1}
BUCKET_NAME=${BUCKET_NAME:-favnir-e2e-demo}
AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
ECR_URI="${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com/favnir-runtime"

echo "[1/4] Building Docker image (multi-stage)..."
docker build \
  -f infra/e2e-demo/ecs/docker/runtime/Dockerfile \
  -t favnir-runtime \
  .

echo "[2/4] Verifying: no .fav files in image..."
FAV_FILES=$(docker run --rm favnir-runtime find / -name "*.fav" 2>/dev/null || true)
if [ -n "$FAV_FILES" ]; then
  echo "ERROR: .fav files found in image:"
  echo "$FAV_FILES"
  exit 1
fi
echo "  OK: 0 .fav files found"

echo "[3/4] Extracting binary -> uploading to S3 (for EC2 bootstrap)..."
docker create --name tmp-fav favnir-runtime
docker cp tmp-fav:/usr/local/bin/fav /tmp/fav-linux
docker rm tmp-fav
aws s3 cp /tmp/fav-linux "s3://${BUCKET_NAME}/bootstrap/fav"
echo "  Uploaded: s3://${BUCKET_NAME}/bootstrap/fav"

echo "[4/4] Pushing image to ECR..."
aws ecr create-repository \
  --repository-name favnir-runtime \
  --region "$AWS_REGION" 2>/dev/null || true

aws ecr get-login-password --region "$AWS_REGION" \
  | docker login --username AWS --password-stdin \
    "${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com"

docker tag favnir-runtime:latest "$ECR_URI:latest"
docker push "$ECR_URI:latest"
echo "  Pushed: $ECR_URI:latest"

echo ""
echo "Done."
echo "  ECR image : $ECR_URI:latest"
echo "  EC2 binary: s3://${BUCKET_NAME}/bootstrap/fav"
