#!/usr/bin/env bash
# scripts/upload.sh
# Favnir バイナリ・パイプライン・CSV を S3 にアップロードする
# infra/e2e-demo/airgap/ から実行すること
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUCKET="favnir-e2e-demo"

# Favnir バイナリのパスを確認
FAV_BIN=$(which fav 2>/dev/null || echo "")
if [ -z "$FAV_BIN" ]; then
  echo "ERROR: fav binary not found in PATH. Build it first: cd fav && cargo build --release"
  exit 1
fi

echo "=== Favnir Airgap E2E Demo — Upload ==="
echo "Binary: $FAV_BIN ($(fav --version 2>&1 | head -1))"
echo "Bucket: s3://$BUCKET/airgap/"
echo ""

# 1. Favnir バイナリ
echo "[1/4] Uploading Favnir binary..."
aws s3 cp "$FAV_BIN" "s3://$BUCKET/airgap/binary/fav"
echo "  → s3://$BUCKET/airgap/binary/fav"

# 2. analyze.fav
echo "[2/4] Uploading analyze.fav..."
aws s3 cp "$SCRIPT_DIR/../src/analyze.fav" "s3://$BUCKET/airgap/src/analyze.fav"
echo "  → s3://$BUCKET/airgap/src/analyze.fav"

# 3. CSV データ（3ファイル）
echo "[3/4] Uploading CSV data..."
aws s3 cp "$SCRIPT_DIR/../src/txn_jan.csv" "s3://$BUCKET/airgap/data/txn_jan.csv"
aws s3 cp "$SCRIPT_DIR/../src/txn_feb.csv" "s3://$BUCKET/airgap/data/txn_feb.csv"
aws s3 cp "$SCRIPT_DIR/../src/txn_mar.csv" "s3://$BUCKET/airgap/data/txn_mar.csv"
echo "  → s3://$BUCKET/airgap/data/ (3 files)"

# 4. 確認
echo "[4/4] Verifying uploads..."
aws s3 ls "s3://$BUCKET/airgap/" --recursive | awk '{print "  " $4}'

echo ""
echo "Upload complete. Run: bash scripts/run.sh"
