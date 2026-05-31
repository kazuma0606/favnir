#!/bin/bash
# scripts/verify.sh
# EKS E2E デモの証跡を確認する（infra/e2e-demo/eks/ から実行すること）
set -euo pipefail

BUCKET=favnir-e2e-demo
PASS=0
FAIL=0

check() {
  local label="$1"
  local result="$2"
  if [ "$result" = "PASS" ]; then
    echo "[PASS] $label"
    PASS=$((PASS + 1))
  else
    echo "[FAIL] $label"
    FAIL=$((FAIL + 1))
  fi
}

echo "=== Favnir EKS E2E Demo — 証跡確認 ==="
echo ""

# 1. Compiler Pod 証跡ファイルの存在
COMPILER_FILE=$(aws s3 ls "s3://${BUCKET}/proof/eks/" \
  | grep "compiler-pod-fav-search" | sort | tail -1 | awk '{print $4}')
check "Compiler Pod: 証跡ファイルが S3 に存在する" \
  "$([ -n "$COMPILER_FILE" ] && echo PASS || echo FAIL)"

# 2. Compiler Pod 証跡に pipeline.fav が存在する（toolchain イメージ確認）
if [ -n "$COMPILER_FILE" ]; then
  aws s3 cp "s3://${BUCKET}/proof/eks/${COMPILER_FILE}" /tmp/eks-compiler-proof.txt \
    > /dev/null 2>&1
  check "Compiler Pod: pipeline.fav が存在する（toolchain イメージ）" \
    "$(grep -q "pipeline.fav" /tmp/eks-compiler-proof.txt && echo PASS || echo FAIL)"
else
  check "Compiler Pod: pipeline.fav が存在する（toolchain イメージ）" "FAIL"
fi

# 3. artifacts/pipeline.fvc が S3 に存在する
check "S3: artifacts/pipeline.fvc が存在する" \
  "$(aws s3 ls "s3://${BUCKET}/artifacts/pipeline.fvc" > /dev/null 2>&1 && echo PASS || echo FAIL)"

# 4. Executor Pod 証跡ファイルの存在
EXECUTOR_FILE=$(aws s3 ls "s3://${BUCKET}/proof/eks/" \
  | grep "executor-pod-fav-search" | sort | tail -1 | awk '{print $4}')
check "Executor Pod: 証跡ファイルが S3 に存在する" \
  "$([ -n "$EXECUTOR_FILE" ] && echo PASS || echo FAIL)"

# 5. Executor Pod 証跡に .fav ファイルが 0 件（runtime イメージ確認）
if [ -n "$EXECUTOR_FILE" ]; then
  aws s3 cp "s3://${BUCKET}/proof/eks/${EXECUTOR_FILE}" /tmp/eks-executor-proof.txt \
    > /dev/null 2>&1
  # .fav で終わる行のみカウント（=== ヘッダー行の誤検知を除外）
  # grep -c exits 1 on zero matches; use || to assign 0 safely
  FAV_COUNT=$(grep -c "\.fav$" /tmp/eks-executor-proof.txt 2>/dev/null) || FAV_COUNT=0
  check "Executor Pod: .fav ファイルが 0 件（runtime イメージ）" \
    "$([ "$FAV_COUNT" -eq 0 ] && echo PASS || echo FAIL)"
else
  check "Executor Pod: .fav ファイルが 0 件（runtime イメージ）" "FAIL"
fi

# 6. output/summary-latest.json が S3 に存在する
check "サマリー JSON が S3/output/ に存在する" \
  "$(aws s3 ls "s3://${BUCKET}/output/summary-latest.json" > /dev/null 2>&1 && echo PASS || echo FAIL)"

echo ""
echo "結果: PASS=${PASS} / FAIL=${FAIL}"

if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
