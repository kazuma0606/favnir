#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== Favnir SAP E2E Demo (All 4 Scenarios) ==="

# [1/3] モックサーバー起動
# Note: Docker 20.10+ (Compose V2) が必要。旧環境では docker-compose コマンドを使用してください。
echo "[1/3] Starting SAP mock server..."
if command -v docker &>/dev/null && [[ -f "$REPO_ROOT/infra/e2e-demo/sap-odata/docker-compose.yml" ]]; then
    docker compose -f "$REPO_ROOT/infra/e2e-demo/sap-odata/docker-compose.yml" up -d
else
    echo "  (docker-compose not available, skipping mock server)"
fi

# [2/3] パイプライン実行（Lambda 呼び出し）
echo "[2/3] Running pipeline via Lambda..."
"$REPO_ROOT/infra/e2e-demo/sap-odata/scripts/run.sh" || { echo "  WARNING: run.sh failed (Lambda may not be deployed — non-fatal)"; }

# [3/3] S3 出力確認
echo "[3/3] Checking S3 output..."
ENDPOINT_ARGS=()
if [[ -n "${AWS_ENDPOINT_URL:-}" ]]; then
    ENDPOINT_ARGS=(--endpoint-url "$AWS_ENDPOINT_URL")
fi
aws s3 ls s3://favnir-sap-demo/ "${ENDPOINT_ARGS[@]}" --recursive 2>/dev/null || true

echo "Done."
