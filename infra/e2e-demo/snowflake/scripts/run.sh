#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_FAV="$SCRIPT_DIR/../src/demo.fav"
CSV_PATH="$SCRIPT_DIR/../src/sample.csv"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_FILE="/tmp/snowflake-e2e-$TIMESTAMP.txt"
PASS=0
FAIL=0

log() { echo "$1" | tee -a "$LOG_FILE"; }

log "=== Favnir v10.9.0 Snowflake E2E Demo ==="
log "Timestamp: $TIMESTAMP"
log ""

# Check required env vars
for var in SNOWFLAKE_ACCOUNT SNOWFLAKE_USER SNOWFLAKE_PRIVATE_KEY; do
  if [ -z "${!var:-}" ]; then
    log "ERROR: $var is not set"
    exit 1
  fi
done

# Stage 1: Type check
log "[1/4] fav check $DEMO_FAV ..."
if fav check "$DEMO_FAV" >> "$LOG_FILE" 2>&1; then
  log "PASS: type check"
  PASS=$((PASS + 1))
else
  log "FAIL: type check"
  FAIL=$((FAIL + 1))
fi

# Stage 2: Run pipeline (LoadCsv -> TransformRows -> SnowflakeInsert -> QuerySummary)
log "[2/4] fav run $DEMO_FAV $CSV_PATH ..."
if fav run "$DEMO_FAV" "$CSV_PATH" >> "$LOG_FILE" 2>&1; then
  log "PASS: pipeline run"
  PASS=$((PASS + 1))
else
  log "FAIL: pipeline run"
  FAIL=$((FAIL + 1))
fi

# Stage 3: Verify summary exists in S3
log "[3/4] verifying S3 proof ..."
if aws s3 ls "s3://favnir-e2e-demo/proof/snowflake/" | grep -q "summary-"; then
  log "PASS: S3 summary proof exists"
  PASS=$((PASS + 1))
else
  log "FAIL: S3 summary proof not found"
  FAIL=$((FAIL + 1))
fi

# Stage 4: Upload run log
log "[4/4] uploading run log ..."
if aws s3 cp "$LOG_FILE" "s3://favnir-e2e-demo/proof/snowflake/run-$TIMESTAMP.txt"; then
  log "PASS: run log uploaded"
  PASS=$((PASS + 1))
else
  log "FAIL: run log upload failed"
  FAIL=$((FAIL + 1))
fi

log ""
log "=== RESULT: PASS=$PASS FAIL=$FAIL ==="

if [ "$FAIL" -ne 0 ]; then
  exit 1
fi
