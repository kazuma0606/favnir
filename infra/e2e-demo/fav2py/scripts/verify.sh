#!/usr/bin/env bash
# verify.sh — S3 出力を比較（Fav ネイティブ版 vs Python トランスパイル版）
set -euo pipefail

BUCKET="${S3_BUCKET:-favnir-e2e-demo}"
PREFIX="proof/fav2py"
PASS=0
FAIL=0

log() { echo "[verify] $1"; }

log "=== fav2py verify ==="

# [1] S3 に 2 件以上の結果ファイルが存在するか確認
log "[1/4] checking S3 proof files ..."
LATEST=$(aws s3 ls "s3://$BUCKET/$PREFIX/" --recursive 2>/dev/null \
  | grep "\.json$" | sort | tail -2 | awk '{print $4}')
COUNT=$(echo "$LATEST" | grep -c "\.json" || true)

if [ "$COUNT" -lt 2 ]; then
  log "FAIL: expected 2 result files in s3://$BUCKET/$PREFIX/, got $COUNT"
  FAIL=$((FAIL + 1))
else
  log "PASS: 2 result files found"
  PASS=$((PASS + 1))
fi

if [ "$FAIL" -gt 0 ]; then
  log ""
  log "=== RESULT: PASS=$PASS FAIL=$FAIL ==="
  exit 1
fi

# [2] ファイルをダウンロード
FILE1=$(echo "$LATEST" | head -1)
FILE2=$(echo "$LATEST" | tail -1)

log "[2/4] downloading results ..."
aws s3 cp "s3://$BUCKET/$FILE1" /tmp/fav2py_result1.json
aws s3 cp "s3://$BUCKET/$FILE2" /tmp/fav2py_result2.json
log "PASS: downloaded $FILE1 and $FILE2"
PASS=$((PASS + 1))

# [3] region/category/total で比較（count は実行順依存のため除外）
log "[3/4] comparing outputs ..."
DIGEST1=$(jq -r '.[] | "\(.region):\(.category):\(.total)"' \
  /tmp/fav2py_result1.json | sort | sha256sum | awk '{print $1}')
DIGEST2=$(jq -r '.[] | "\(.region):\(.category):\(.total)"' \
  /tmp/fav2py_result2.json | sort | sha256sum | awk '{print $1}')

if [ "$DIGEST1" = "$DIGEST2" ]; then
  log "PASS: native output == python output (digest: ${DIGEST1:0:16}...)"
  PASS=$((PASS + 1))
else
  log "FAIL: outputs differ"
  log "  file1 (${FILE1##*/}): $(jq -c '.[0:2]' /tmp/fav2py_result1.json)"
  log "  file2 (${FILE2##*/}): $(jq -c '.[0:2]' /tmp/fav2py_result2.json)"
  FAIL=$((FAIL + 1))
fi

# [4] レコード件数確認（region × category = 最大 16 件）
log "[4/4] validating record count ..."
COUNT1=$(jq 'length' /tmp/fav2py_result1.json)
COUNT2=$(jq 'length' /tmp/fav2py_result2.json)
if [ "$COUNT1" -eq "$COUNT2" ] && [ "$COUNT1" -gt 0 ]; then
  log "PASS: both results have $COUNT1 records"
  PASS=$((PASS + 1))
else
  log "FAIL: record count mismatch (file1=$COUNT1, file2=$COUNT2)"
  FAIL=$((FAIL + 1))
fi

log ""
log "=== RESULT: PASS=$PASS FAIL=$FAIL ==="
[ "$FAIL" -eq 0 ] || exit 1
