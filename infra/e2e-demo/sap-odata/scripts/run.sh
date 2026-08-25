#!/usr/bin/env bash
set -euo pipefail

FUNCTION_NAME="favnir-sap-e2e-demo"
REGION="${AWS_DEFAULT_REGION:-ap-northeast-1}"
OUTPUT_FILE="/tmp/sap-demo-output.json"

# LocalStack / 本番切り替えは AWS_ENDPOINT_URL 環境変数で行う
# 例: export AWS_ENDPOINT_URL=http://localhost:4566
ENDPOINT_ARGS=()
if [[ -n "${AWS_ENDPOINT_URL:-}" ]]; then
  ENDPOINT_ARGS=(--endpoint-url "$AWS_ENDPOINT_URL")
fi

echo "=== Favnir SAP E2E Demo ==="
echo "Invoking Lambda: $FUNCTION_NAME (region: $REGION)"

aws lambda invoke \
  --function-name "$FUNCTION_NAME" \
  --region "$REGION" \
  --cli-binary-format raw-in-base64-out \
  --payload '{}' \
  "${ENDPOINT_ARGS[@]}" \
  "$OUTPUT_FILE"

echo "=== Result ==="
cat "$OUTPUT_FILE"
echo ""
echo "Done."
