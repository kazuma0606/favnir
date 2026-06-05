#!/usr/bin/env bash
# scripts/verify.sh
# 証跡を確認し、PASS=5/FAIL=0 を確認後に EC2 を terraform destroy する
# infra/e2e-demo/airgap/ から実行すること
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TF_DIR="$SCRIPT_DIR/../terraform"
BUCKET="favnir-e2e-demo"
PASS=0
FAIL=0

check() {
  local label="$1"
  local result="$2"
  if [ "$result" = "PASS" ]; then
    echo "[PASS] $label"
    PASS=$((PASS + 1))
  else
    echo "[FAIL] $label"
    FAIL=$((FAIL + 1))
  fi
}

echo "=== Favnir Airgap E2E Demo — 証跡確認 ==="
echo ""

# 1. proof ファイルが S3 に存在する
PROOF_FILE=$(aws s3 ls "s3://${BUCKET}/airgap/proof/" \
  | grep "proof-" | sort | tail -1 | awk '{print $4}')
check "proof ファイルが S3 に存在する" \
  "$([ -n "$PROOF_FILE" ] && echo PASS || echo FAIL)"

# 2. proof に "which fav: not found" が含まれる（システム未汚染）
if [ -n "$PROOF_FILE" ]; then
  aws s3 cp "s3://${BUCKET}/airgap/proof/${PROOF_FILE}" /tmp/airgap-proof.txt \
    > /dev/null 2>&1
  check "which fav → not found（システム PATH 未汚染）" \
    "$(grep -q "not found" /tmp/airgap-proof.txt && echo PASS || echo FAIL)"
else
  check "which fav → not found（システム PATH 未汚染）" "FAIL"
fi

# 3. proof に "Dropped:" 行が含まれる（品質チェックログの存在）
if [ -f /tmp/airgap-proof.txt ]; then
  check "ドロップ率ログが proof に存在する（Dropped: 行）" \
    "$(grep -q "Dropped:" /tmp/airgap-proof.txt && echo PASS || echo FAIL)"

  echo ""
  echo "--- ETL ログ抜粋 ---"
  grep -E "^\[INFO\]|\[WARN\]" /tmp/airgap-proof.txt | head -20
  echo "---"
  echo ""
else
  check "ドロップ率ログが proof に存在する（Dropped: 行）" "FAIL"
fi

# 4. airgap/output/summary.json が S3 に存在する
check "airgap/output/summary.json が S3 に存在する" \
  "$(aws s3 ls "s3://${BUCKET}/airgap/output/summary.json" > /dev/null 2>&1 && echo PASS || echo FAIL)"

echo ""
echo "結果: PASS=${PASS} / FAIL=${FAIL}"
echo ""

if [ "$FAIL" -gt 0 ]; then
  echo "FAIL があります。EC2 ログを確認してください:"
  cd "$TF_DIR"
  INSTANCE_ID=$(terraform output -raw instance_id 2>/dev/null || echo "unknown")
  echo "  aws ssm start-session --target $INSTANCE_ID"
  exit 1
fi

# 5. PASS 確認後 EC2 を terraform destroy（後片付け）
echo "[PASS] 全チェック完了。EC2 を削除します..."
cd "$TF_DIR"
terraform destroy -auto-approve -no-color
echo ""
echo "[Done] EC2 terminated. S3 の証跡・出力は保持されています。"
echo "  Proof : s3://${BUCKET}/airgap/proof/"
echo "  Output: s3://${BUCKET}/airgap/output/summary.json"
PASS=$((PASS + 1))
echo ""
echo "最終結果: PASS=${PASS} / FAIL=${FAIL}"
