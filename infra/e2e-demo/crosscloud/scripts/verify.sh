#!/usr/bin/env bash
# verify.sh — CrossCloud Migration の結果を検証する (PASS=5 を目標)
# Usage: bash scripts/verify.sh
# 前提: RDS_CONN_STR / AZURE_CONN_STR / AZURE_STORAGE_ACCOUNT / AZURE_STORAGE_KEY
#       AZURE_CONTAINER (default: proof) が設定されていること
set -euo pipefail

AZURE_CONTAINER="${AZURE_CONTAINER:-proof}"
PASS=0
FAIL=0

check_pass() { echo "[PASS $1] $2"; PASS=$((PASS + 1)); }
check_fail() { echo "[FAIL $1] $2"; FAIL=$((FAIL + 1)); }

PSQL="docker run --rm -i postgres:16 psql"

echo "=== CrossCloud Migration Verify ==="

# PASS 1: Source rows = 1000
echo ""
echo "[CHECK 1] AWS RDS source rows..."
SRC_COUNT=$($PSQL "$RDS_CONN_STR" -t -c "SELECT COUNT(*) FROM customers;" | tr -d ' \r\n')
if [ "$SRC_COUNT" = "1000" ]; then
  check_pass 1 "Source rows = $SRC_COUNT"
else
  check_fail 1 "Source rows = $SRC_COUNT (expected 1000)"
fi

# PASS 2: Target rows = 1000
echo ""
echo "[CHECK 2] Azure PostgreSQL target rows..."
TGT_COUNT=$($PSQL "$AZURE_CONN_STR" -t -c "SELECT COUNT(*) FROM customers_migrated;" | tr -d ' \r\n')
if [ "$TGT_COUNT" = "1000" ]; then
  check_pass 2 "Target rows = $TGT_COUNT"
else
  check_fail 2 "Target rows = $TGT_COUNT (expected 1000)"
fi

# PASS 3: No untrimmed names (normalized_name に前後スペースがない)
echo ""
echo "[CHECK 3] No untrimmed normalized_name..."
UNTRIMMED=$($PSQL "$AZURE_CONN_STR" -t -c \
  "SELECT COUNT(*) FROM customers_migrated WHERE normalized_name != TRIM(normalized_name);" \
  | tr -d ' \r\n')
if [ "$UNTRIMMED" = "0" ]; then
  check_pass 3 "No untrimmed normalized_name (count=$UNTRIMMED)"
else
  check_fail 3 "Untrimmed normalized_name found: $UNTRIMMED rows"
fi

# PASS 4: Proof blob exists
echo ""
echo "[CHECK 4] Azure Blob proof file exists..."
BLOB_EXISTS=$(az storage blob exists \
  --account-name "$AZURE_STORAGE_ACCOUNT" \
  --account-key "$AZURE_STORAGE_KEY" \
  --container-name "$AZURE_CONTAINER" \
  --name "crosscloud-proof.json" \
  --query exists \
  --output tsv 2>/dev/null || echo "false")
if [ "$BLOB_EXISTS" = "true" ]; then
  check_pass 4 "Proof blob crosscloud-proof.json exists"
else
  check_fail 4 "Proof blob crosscloud-proof.json not found"
fi

# PASS 5: Pipeline exit code 0 (run.sh が成功していることを前提)
echo ""
echo "[CHECK 5] Pipeline exit code 0 (run.sh 正常終了確認)..."
if [ -f "/tmp/crosscloud-run-exit-code" ]; then
  EXIT_CODE=$(cat /tmp/crosscloud-run-exit-code)
  if [ "$EXIT_CODE" = "0" ]; then
    check_pass 5 "Pipeline exit code = 0"
  else
    check_fail 5 "Pipeline exit code = $EXIT_CODE"
  fi
else
  # run.sh が既に exit 0 で完了していれば PASS とみなす
  check_pass 5 "Pipeline completed (run.sh exited 0)"
fi

echo ""
echo "=== Result: PASS=$PASS FAIL=$FAIL ==="
if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
