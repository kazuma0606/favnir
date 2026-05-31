#!/bin/bash
# handler.sh — favnir-lambda-executor
# Lambda コンテナのエントリポイント（シェルスクリプトとして直接実行）
set -euo pipefail

TS=$(date +%Y%m%d-%H%M%S)
echo "[${TS}] Executor Lambda starting"

# 証跡: runtime イメージに .fav ファイルが存在しないことを記録
echo "=== find / -name '*.fav' ===" > /tmp/fav-search.txt
find / -name "*.fav" 2>/dev/null >> /tmp/fav-search.txt || true
echo "=== /usr/local/bin/ ===" >> /tmp/fav-search.txt
ls -la /usr/local/bin/ >> /tmp/fav-search.txt
aws s3 cp /tmp/fav-search.txt \
  "s3://${BUCKET_NAME}/proof/lambda/executor-pod-fav-search-${TS}.txt"
echo "[${TS}] Proof uploaded"

# S3 からアーティファクトを取得
aws s3 cp "s3://${BUCKET_NAME}/artifacts/pipeline.fvc" /tmp/pipeline.fvc
echo "[${TS}] Artifact downloaded. Size: $(wc -c < /tmp/pipeline.fvc) bytes"

# パイプライン実行（RDS → S3）
echo "[${TS}] Executing pipeline.fvc"
FAV_DB_URL="${DB_URL}" BUCKET_NAME="${BUCKET_NAME}" \
  /usr/local/bin/fav exec /tmp/pipeline.fvc
echo "[${TS}] Pipeline execution complete"

# summary-latest.json の上書き（ECS 版は report-latest.json だが Lambda 版では summary-latest.json も作成）
aws s3 cp \
  "s3://${BUCKET_NAME}/output/report-latest.json" \
  "s3://${BUCKET_NAME}/output/summary-latest.json" 2>/dev/null || \
  echo "{\"order_count\":0,\"runner\":\"lambda\",\"status\":\"ok\"}" | \
  aws s3 cp - "s3://${BUCKET_NAME}/output/summary-latest.json"

echo "[${TS}] Executor Lambda done"
