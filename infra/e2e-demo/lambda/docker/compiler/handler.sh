#!/bin/bash
# handler.sh — favnir-lambda-compiler
# Lambda コンテナのエントリポイント（シェルスクリプトとして直接実行）
set -euo pipefail

TS=$(date +%Y%m%d-%H%M%S)
echo "[${TS}] Compiler Lambda starting"

# 証跡: toolchain イメージに pipeline.fav が存在することを記録
echo "=== find / -name '*.fav' ===" > /tmp/fav-search.txt
find / -name "*.fav" 2>/dev/null >> /tmp/fav-search.txt || true
echo "=== /usr/local/bin/ ===" >> /tmp/fav-search.txt
ls -la /usr/local/bin/ >> /tmp/fav-search.txt
aws s3 cp /tmp/fav-search.txt \
  "s3://${BUCKET_NAME}/proof/lambda/compiler-pod-fav-search-${TS}.txt"
echo "[${TS}] Proof uploaded"

# コンパイル: /app/src/pipeline.fav → /tmp/pipeline.fvc
/usr/local/bin/fav build /app/src/pipeline.fav -o /tmp/pipeline.fvc
echo "[${TS}] Build complete. Size: $(wc -c < /tmp/pipeline.fvc) bytes"

# アーティファクトを S3 にアップロード
aws s3 cp /tmp/pipeline.fvc "s3://${BUCKET_NAME}/artifacts/pipeline.fvc"
echo "[${TS}] Artifact uploaded to s3://${BUCKET_NAME}/artifacts/pipeline.fvc"

# SQS にメッセージ送信（executor へ通知）
aws sqs send-message \
  --region "${AWS_REGION}" \
  --queue-url "${SQS_QUEUE_URL}" \
  --message-body "{\"artifact_key\":\"artifacts/pipeline.fvc\",\"timestamp\":\"${TS}\"}"
echo "[${TS}] SQS message sent"

echo "[${TS}] Compiler Lambda done"
