#!/bin/bash
# scripts/trigger.sh
# S3 に pipeline.fav を投入してデモを開始する
# infra/e2e-demo/lambda/ から実行すること
set -euo pipefail

BUCKET=favnir-e2e-demo
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== Favnir Lambda E2E Demo — トリガー ==="
echo ""

# 既存の出力・証跡をクリア（前回実行の残骸を除去）
echo "前回の証跡をクリア..."
aws s3 rm "s3://${BUCKET}/proof/lambda/" --recursive 2>/dev/null || true
aws s3 rm "s3://${BUCKET}/artifacts/pipeline.fvc" 2>/dev/null || true
aws s3 rm "s3://${BUCKET}/output/summary-latest.json" 2>/dev/null || true
aws s3 rm "s3://${BUCKET}/output/report-latest.json" 2>/dev/null || true

echo ""
echo "source/pipeline.fav を S3 に投入..."
aws s3 cp "${SCRIPT_DIR}/../src/pipeline.fav" "s3://${BUCKET}/source/pipeline.fav"
echo "Uploaded: s3://${BUCKET}/source/pipeline.fav"

echo ""
echo "Lambda が自動起動します。ログを確認するには:"
echo "  aws logs tail /aws/lambda/favnir-compiler --follow --region ap-northeast-1"
echo "  aws logs tail /aws/lambda/favnir-executor --follow --region ap-northeast-1"
echo ""
echo "完了後に verify.sh で証跡を確認:"
echo "  bash scripts/verify.sh"
