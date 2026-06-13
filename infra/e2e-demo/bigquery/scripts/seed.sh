#!/bin/bash
# seed.sh — BigQuery E2E テスト用 CSV を生成する
set -euo pipefail

OUTPUT="${SEED_CSV_PATH:-/tmp/seed.csv}"

cat > "$OUTPUT" <<'CSV'
user_id,full_name,email
1,  Alice Smith  ,alice@example.com
2,  Bob Jones  ,BOB@example.com
3,  Carol White  ,carol@example.com
CSV

echo "[seed] ${OUTPUT} を生成しました（3 件）"
