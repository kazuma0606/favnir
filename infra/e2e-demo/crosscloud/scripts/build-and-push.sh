#!/bin/bash
# build-and-push.sh — fav を Linux 向けにクロスコンパイルして ECR に push する
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

echo "[build] Copying artifacts to docker/..."
cp "${FAV_DIR}/target/x86_64-unknown-linux-musl/release/fav" "${DOCKER_DIR}/fav"
cp "${SCRIPT_DIR}/../src/migrate.fav" "${DOCKER_DIR}/migrate.fav"

echo "[build] Building Docker image (linux/amd64)..."
docker build --platform linux/amd64 -t "${REPO}:${TAG}" "${DOCKER_DIR}"

echo "[push] Logging in to ECR..."
aws ecr get-login-password --region "$REGION" \
  | docker login --username AWS --password-stdin "${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com"

docker tag "${REPO}:${TAG}" "${ECR_URL}:${TAG}"
docker push "${ECR_URL}:${TAG}"

# 一時ファイルを掃除
rm -f "${DOCKER_DIR}/fav" "${DOCKER_DIR}/migrate.fav"

echo "[push] Done: ${ECR_URL}:${TAG}"
