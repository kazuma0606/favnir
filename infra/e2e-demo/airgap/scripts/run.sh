#!/usr/bin/env bash
# scripts/run.sh
# EC2 を起動し、user_data の完了を待つ
# infra/e2e-demo/airgap/ から実行すること
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TF_DIR="$SCRIPT_DIR/../terraform"
BUCKET="favnir-e2e-demo"

echo "=== Favnir Airgap E2E Demo — Run ==="
echo ""

# terraform apply
echo "[1/3] terraform apply..."
cd "$TF_DIR"
terraform init -upgrade -no-color > /dev/null
terraform apply -auto-approve -no-color
echo ""

# EC2 instance ID 取得
INSTANCE_ID=$(terraform output -raw instance_id)
echo "[2/3] EC2 instance launched: $INSTANCE_ID"
echo "      (Private IP: $(terraform output -raw instance_private_ip))"
echo "      Waiting for user_data to complete..."

# user_data 完了を proof ファイルの S3 出現で検知（最大 10 分）
MAX_WAIT=600
ELAPSED=0
INTERVAL=15

while [ $ELAPSED -lt $MAX_WAIT ]; do
  PROOF_COUNT=$(aws s3 ls "s3://$BUCKET/airgap/proof/" 2>/dev/null | grep "proof-" | wc -l)
  if [ "$PROOF_COUNT" -gt 0 ]; then
    echo ""
    echo "[3/3] Proof file detected in S3. Pipeline completed."
    echo ""
    echo "Proof files:"
    aws s3 ls "s3://$BUCKET/airgap/proof/" | awk '{print "  " $4}'
    echo ""
    echo "Run next: bash scripts/verify.sh"
    exit 0
  fi
  echo "  ... waiting (${ELAPSED}s elapsed)"
  sleep $INTERVAL
  ELAPSED=$((ELAPSED + INTERVAL))
done

echo "ERROR: Timed out waiting for proof file. Check EC2 logs via SSM:"
echo "  aws ssm start-session --target $INSTANCE_ID"
exit 1
