#!/bin/bash
# run.sh — BigQuery E2E デモ実行
# Usage: ./run.sh [gcp_project_id]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

GCP_PROJECT_ID="${1:-${GCP_PROJECT_ID:-favnir-bigquery-demo}}"
BQ_DATASET="${BQ_DATASET:-favnir_demo}"
SA_KEY="${GOOGLE_APPLICATION_CREDENTIALS:-${SCRIPT_DIR}/../../../../fav/tmp/gcp-sa-key.json}"

if [ ! -f "$SA_KEY" ]; then
  echo "ERROR: サービスアカウントキーが見つかりません: $SA_KEY"
  echo "       terraform apply を実行してキーを生成してください"
  exit 1
fi

export GCP_PROJECT_ID
export BQ_DATASET
export GOOGLE_APPLICATION_CREDENTIALS="$SA_KEY"

echo "[run] GCP_PROJECT_ID=${GCP_PROJECT_ID}"
echo "[run] BQ_DATASET=${BQ_DATASET}"
echo "[run] GOOGLE_APPLICATION_CREDENTIALS=${GOOGLE_APPLICATION_CREDENTIALS}"
echo ""

# シードデータ生成
bash "${SCRIPT_DIR}/seed.sh"

# Favnir パイプライン実行
FAV="${FAV_BIN:-$(which fav 2>/dev/null || echo "${SCRIPT_DIR}/../../../../fav/target/debug/fav")}"
echo ""
"$FAV" run --legacy "${SCRIPT_DIR}/../src/demo.fav"
