#!/usr/bin/env bash
# Deploy the Favnir reference site to S3 + CloudFront
# Usage: ./scripts/deploy-site.sh [--dry-run]
set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
fi

# Resolve project root (script lives in scripts/)
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SITE_DIR="$ROOT/site"
INFRA_DIR="$ROOT/infra/site"

echo "==> Building site..."
cd "$SITE_DIR"
npm ci --prefer-offline
npm run build

# Read outputs from Terraform
echo "==> Reading Terraform outputs..."
cd "$INFRA_DIR"
BUCKET=$(terraform output -raw site_bucket)
DISTRIBUTION_ID=$(terraform output -raw distribution_id)

echo "    Bucket:         $BUCKET"
echo "    Distribution:   $DISTRIBUTION_ID"

if [[ "$DRY_RUN" == "true" ]]; then
  echo "==> [dry-run] Would sync site/out/ to s3://$BUCKET"
  echo "==> [dry-run] Would invalidate CloudFront $DISTRIBUTION_ID"
  exit 0
fi

echo "==> Syncing to S3..."
aws s3 sync "$SITE_DIR/out/" "s3://$BUCKET/" \
  --delete \
  --cache-control "public, max-age=31536000, immutable" \
  --exclude "*.html" \
  --exclude "*.json"

# HTML and JSON files get shorter TTL (re-check on each deploy)
aws s3 sync "$SITE_DIR/out/" "s3://$BUCKET/" \
  --delete \
  --cache-control "public, max-age=0, must-revalidate" \
  --exclude "*" \
  --include "*.html" \
  --include "*.json"

echo "==> Invalidating CloudFront cache..."
aws cloudfront create-invalidation \
  --distribution-id "$DISTRIBUTION_ID" \
  --paths "/*"

echo "==> Done."
cd "$INFRA_DIR"
echo "    Site URL: $(terraform output -raw cloudfront_domain)"
