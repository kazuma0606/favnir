#!/bin/bash
# build-and-push-verifier.sh — Lambda verifier コンテナイメージをビルドして ECR に push する
set -euo pipefail

REGION="${AWS_REGION:-ap-northeast-1}"
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
REPO="crosscloud-verifier"
ECR_URL="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/${REPO}"
TAG="${IMAGE_TAG:-latest}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LAMBDA_DIR="${SCRIPT_DIR}/../lambda/verifier"
FAV_DIR="${SCRIPT_DIR}/../../../../fav"

echo "[build] Cross-compiling fav for x86_64-unknown-linux-musl..."
(cd "$FAV_DIR" && cargo build --release --target x86_64-unknown-linux-musl)

echo "[build] Copying fav binary to lambda/verifier/..."
cp "${FAV_DIR}/target/x86_64-unknown-linux-musl/release/fav" "${LAMBDA_DIR}/fav"

echo "[build] Building Docker image (linux/amd64)..."
docker build --platform linux/amd64 -t "${REPO}:${TAG}" "${LAMBDA_DIR}"

echo "[push] Logging in to ECR..."
aws ecr get-login-password --region "$REGION" \
  | docker login --username AWS --password-stdin "${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com"

docker tag "${REPO}:${TAG}" "${ECR_URL}:${TAG}"
docker push "${ECR_URL}:${TAG}"

# 一時ファイルを掃除
rm -f "${LAMBDA_DIR}/fav"

echo "[push] Done: ${ECR_URL}:${TAG}"
