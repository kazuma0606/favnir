#!/usr/bin/env bash
# run.sh — CrossCloud Migration パイプラインを実行する
# Usage: bash scripts/run.sh
# 前提:
#   - RDS_CONN_SECRET_ARN: Secrets Manager ARN (terraform output rds_conn_secret_arn)
#   - AZURE_CONN_STR: Azure PostgreSQL 接続文字列
#   - AZURE_STORAGE_ACCOUNT: Azure Storage Account 名
#   - AZURE_STORAGE_KEY: Azure Storage Account キー
#   - AZURE_CONTAINER: Blob コンテナ名 (default: proof)
#   - FAV_BIN: fav バイナリパス (default: fav)
set -euo pipefail

FAV_BIN="${FAV_BIN:-fav}"
AZURE_CONTAINER="${AZURE_CONTAINER:-proof}"

# 環境変数チェック
for var in RDS_CONN_SECRET_ARN AZURE_CONN_STR AZURE_STORAGE_ACCOUNT AZURE_STORAGE_KEY; do
  if [ -z "${!var:-}" ]; then
    echo "ERROR: 環境変数 $var が未設定です。" >&2
    exit 1
  fi
done

# Secrets Manager から RDS 接続文字列を取得
echo "[run] Secrets Manager から DATABASE_URL を取得..."
DATABASE_URL=$(aws secretsmanager get-secret-value \
  --secret-id "$RDS_CONN_SECRET_ARN" \
  --query SecretString \
  --output text)
echo "[run] DATABASE_URL 取得完了"

# パイプライン実行
# Postgres.query_raw は DATABASE_URL を自動参照する
# AzurePostgres.execute_raw は AZURE_CONN_STR を第1引数として受け取る
echo "[run] fav run --legacy src/migrate.fav ..."
export DATABASE_URL
export AZURE_CONN_STR
export AZURE_STORAGE_ACCOUNT
export AZURE_STORAGE_KEY
export AZURE_CONTAINER

"$FAV_BIN" run --legacy src/migrate.fav

echo "[run] パイプライン完了 (exit 0)"
