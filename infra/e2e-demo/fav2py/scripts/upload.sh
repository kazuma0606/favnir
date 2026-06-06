#!/usr/bin/env bash
# upload.sh — Docker ビルド + ECR push + S3 ソースアップロード
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/.."
REGION="${AWS_DEFAULT_REGION:-ap-northeast-1}"
ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
ECR_URI="$ACCOUNT.dkr.ecr.$REGION.amazonaws.com/favnir/fav2py"

log() { echo "[upload] $1"; }

log "=== fav2py upload ==="
log "ECR: $ECR_URI"

# ECR login
log "logging in to ECR ..."
aws ecr get-login-password --region "$REGION" \
  | docker login --username AWS --password-stdin "$ECR_URI"

# Docker build
log "building Docker image ..."
cd "$ROOT"
docker build -t fav2py:latest .

# Tag & push
log "pushing to ECR ..."
docker tag fav2py:latest "$ECR_URI:latest"
docker push "$ECR_URI:latest"

# S3 source upload
log "uploading sources to S3 ..."
aws s3 cp "$ROOT/src/" "s3://favnir-e2e-demo/fav2py/src/" --recursive

log "upload.sh: done"
