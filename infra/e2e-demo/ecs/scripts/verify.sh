#!/bin/bash
# Phase 8-B〜E: 証跡・出力・ログの確認スクリプト
# terraform apply + ECS Task 完了後に実行する
#
# 使用方法（terraform ディレクトリから）:
#   bash ../scripts/verify.sh
set -e

AWS_REGION="${AWS_REGION:-ap-northeast-1}"
TF_DIR="$(dirname "$0")/../terraform"
BUCKET=$(terraform -chdir="$TF_DIR" output -raw s3_bucket_name)

PASS=0
FAIL=0

check() {
  local desc="$1"
  local result="$2"
  if [ "$result" = "ok" ]; then
    echo "  [PASS] $desc"
    PASS=$((PASS + 1))
  else
    echo "  [FAIL] $desc"
    FAIL=$((FAIL + 1))
  fi
}

echo "========================================"
echo " Favnir E2E Demo — 証跡確認"
echo " Bucket: $BUCKET"
echo "========================================"
echo ""

# ── Machine A 証跡 ────────────────────────────────────────────────────────────
echo "[Machine A] Favnir 処理系サーバの証跡"

PROOF_A=$(aws s3 ls "s3://$BUCKET/proof/machine-a/" 2>/dev/null | tail -1 | awk '{print $4}')
if [ -n "$PROOF_A" ]; then
  aws s3 cp "s3://$BUCKET/proof/machine-a/$PROOF_A" /tmp/proof-a.txt > /dev/null
  check "証跡ファイル存在" "ok"

  if grep -q "etl.fav" /tmp/proof-a.txt && grep -q "pipeline.fav" /tmp/proof-a.txt; then
    check ".fav ソースファイルが /app/src/ に存在する" "ok"
  else
    check ".fav ソースファイルが /app/src/ に存在する" "fail"
  fi
  echo "  --- 証跡内容 ---"
  cat /tmp/proof-a.txt | sed 's/^/    /'
else
  check "証跡ファイル存在" "fail"
  check ".fav ソースファイルが /app/src/ に存在する" "fail"
fi

echo ""

# ── Machine B 証跡 ────────────────────────────────────────────────────────────
echo "[Machine B] Rust VM サーバの証跡"

PROOF_B=$(aws s3 ls "s3://$BUCKET/proof/machine-b/" 2>/dev/null | tail -1 | awk '{print $4}')
if [ -n "$PROOF_B" ]; then
  aws s3 cp "s3://$BUCKET/proof/machine-b/$PROOF_B" /tmp/proof-b.txt > /dev/null
  check "証跡ファイル存在" "ok"

  # .fav ファイルが 0 件であることを確認
  FAV_LINES=$(grep -v "^===" /tmp/proof-b.txt | grep -v "^---" | grep -v "^$" \
    | grep -v "end of .fav" | grep "\.fav" | wc -l)
  if [ "$FAV_LINES" -eq 0 ]; then
    check ".fav ファイルが 0 件（ソースコードなし）" "ok"
  else
    check ".fav ファイルが 0 件（ソースコードなし）" "fail"
    echo "  検出された .fav ファイル:"
    grep "\.fav" /tmp/proof-b.txt | sed 's/^/    /'
  fi
  echo "  --- 証跡内容 ---"
  cat /tmp/proof-b.txt | sed 's/^/    /'
else
  check "証跡ファイル存在" "fail"
  check ".fav ファイルが 0 件（ソースコードなし）" "fail"
fi

echo ""

# ── ECS 証跡 ─────────────────────────────────────────────────────────────────
echo "[ECS] ETL コンテナの証跡"

PROOF_E=$(aws s3 ls "s3://$BUCKET/proof/ecs/" 2>/dev/null | tail -1 | awk '{print $4}')
if [ -n "$PROOF_E" ]; then
  aws s3 cp "s3://$BUCKET/proof/ecs/$PROOF_E" /tmp/proof-ecs.txt > /dev/null
  check "証跡ファイル存在" "ok"

  FAV_LINES=$(grep -v "^===" /tmp/proof-ecs.txt | grep -v "^---" | grep -v "^$" \
    | grep -v "end of .fav" | grep "\.fav" | wc -l)
  if [ "$FAV_LINES" -eq 0 ]; then
    check ".fav ファイルが 0 件（ソースコードなし）" "ok"
  else
    check ".fav ファイルが 0 件（ソースコードなし）" "fail"
    echo "  検出された .fav ファイル:"
    grep "\.fav" /tmp/proof-ecs.txt | sed 's/^/    /'
  fi
  echo "  --- 証跡内容 ---"
  cat /tmp/proof-ecs.txt | sed 's/^/    /'
else
  check "証跡ファイル存在" "fail"
  check ".fav ファイルが 0 件（ソースコードなし）" "fail"
fi

echo ""

# ── ETL 出力確認 ──────────────────────────────────────────────────────────────
echo "[ETL Output] S3 出力確認"

SUMMARY=$(aws s3 ls "s3://$BUCKET/output/" 2>/dev/null | grep "summary-" | tail -1 | awk '{print $4}')
if [ -n "$SUMMARY" ]; then
  check "サマリー JSON が S3/output/ に存在する" "ok"
  aws s3 cp "s3://$BUCKET/output/$SUMMARY" /tmp/summary.json > /dev/null
  echo "  --- ETL 出力内容 ---"
  cat /tmp/summary.json | sed 's/^/    /'
else
  check "サマリー JSON が S3/output/ に存在する" "fail"
fi

REPORT=$(aws s3 ls "s3://$BUCKET/output/" 2>/dev/null | grep "report-" | tail -1 | awk '{print $4}')
if [ -n "$REPORT" ]; then
  check "レポート JSON が S3/output/ に存在する (Machine B)" "ok"
  aws s3 cp "s3://$BUCKET/output/$REPORT" /tmp/report.json > /dev/null
  echo "  --- Machine B 出力内容 ---"
  cat /tmp/report.json | sed 's/^/    /'
else
  check "レポート JSON が S3/output/ に存在する (Machine B)" "fail"
fi

echo ""

# ── 結果サマリー ─────────────────────────────────────────────────────────────
echo "========================================"
echo " 結果: PASS=$PASS / FAIL=$FAIL"
echo "========================================"
echo ""
echo "証跡ファイル一覧:"
aws s3 ls "s3://$BUCKET/proof/" --recursive | sed 's/^/  /'

if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
