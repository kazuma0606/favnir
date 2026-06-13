#!/bin/bash
# build-and-push-verifier-v2.sh — Lambda verifier_v2 (KMS ECDSA 版) コンテナをビルドして ECR に push する
set -euo pipefail

REGION="${AWS_REGION:-ap-northeast-1}"
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
REPO="crosscloud-verifier-v2"
ECR_URL="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/${REPO}"
TAG="${IMAGE_TAG:-latest}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LAMBDA_DIR="${SCRIPT_DIR}/../lambda/verifier_v2"
FAV_DIR="${SCRIPT_DIR}/../../../../fav"

# ── fav Linux バイナリをビルド ───────────────────────────────────────
echo "[build] Building fav Linux binary via Dockerfile.builder..."
docker build --no-cache \
  -f "${FAV_DIR}/Dockerfile.builder" \
  --tag fav-builder:latest \
  "${FAV_DIR}"

echo "[build] Extracting fav binary from builder image..."
docker create --name fav-builder-tmp fav-builder:latest
docker cp fav-builder-tmp:/usr/local/bin/fav "${LAMBDA_DIR}/fav"
docker rm fav-builder-tmp
chmod +x "${LAMBDA_DIR}/fav"

# ── ECR ログイン ─────────────────────────────────────────────────────
echo "[push] Logging in to ECR..."
aws ecr get-login-password --region "$REGION" \
  | docker login --username AWS --password-stdin "${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com"

# ── verifier_v2 イメージをビルド・push ──────────────────────────────
echo "[build] Building verifier_v2 image (linux/amd64)..."
docker buildx build \
  --platform linux/amd64 \
  --provenance=false \
  -t "${ECR_URL}:${TAG}" \
  --push \
  "${LAMBDA_DIR}"

# 一時ファイルを掃除
rm -f "${LAMBDA_DIR}/fav"

echo "[push] Done: ${ECR_URL}:${TAG}"
