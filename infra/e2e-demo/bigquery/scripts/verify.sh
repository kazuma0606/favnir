#!/bin/bash
# verify.sh — BigQuery に 3 件 INSERT されているか確認
# Usage: ./verify.sh [gcp_project_id]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GCP_PROJECT_ID="${1:-${GCP_PROJECT_ID:-favnir-bigquery-demo}}"
BQ_DATASET="${BQ_DATASET:-favnir_demo}"
SA_KEY="${GOOGLE_APPLICATION_CREDENTIALS:-${SCRIPT_DIR}/../../../tmp/gcp-sa-key.json}"
FAV="${FAV_BIN:-$(which fav 2>/dev/null || echo "${SCRIPT_DIR}/../../../../fav/target/debug/fav")}"

PASS=0
FAIL=0

check() {
  local LABEL="$1"
  local EXPECTED="$2"
  local ACTUAL="$3"
  if [ "$ACTUAL" = "$EXPECTED" ]; then
    echo "[PASS] ${LABEL} (${ACTUAL})"
    PASS=$((PASS+1))
  else
    echo "[FAIL] ${LABEL} — expected ${EXPECTED}, got ${ACTUAL}"
    FAIL=$((FAIL+1))
  fi
}

echo "[verify] BigQuery テーブル件数確認..."
echo "  Project: ${GCP_PROJECT_ID}"
echo "  Dataset: ${BQ_DATASET}"
echo ""

export GOOGLE_APPLICATION_CREDENTIALS="$SA_KEY"

# Favnir 経由でクエリ
QUERY_FAV="${SCRIPT_DIR}/query_count.fav"
cat > "$QUERY_FAV" <<'FAV'
fn do_query() -> Result<String, String> !Gcp {
  BigQuery.query_raw(
    "favnir-bigquery-demo",
    "favnir_demo",
    "SELECT COUNT(*) AS cnt FROM `favnir_demo.users`",
    "[]"
  )
}
public fn main(ctx: AppCtx) -> Result<Unit, String> {
  bind result <- do_query()
  ctx.io.println(result)
}
FAV

OUTPUT=$("$FAV" run --legacy "$QUERY_FAV" 2>/dev/null || echo "error")
rm -f "$QUERY_FAV"

# JSON から "v":"N" を抽出
COUNT=$(echo "$OUTPUT" | sed 's/.*"v":"\([0-9]*\)".*/\1/' | head -1 || echo "0")

check "INSERT 3件" "3" "$COUNT"

echo ""
echo "PASS=${PASS} FAIL=${FAIL}"
[ "$FAIL" -eq 0 ] || exit 1
