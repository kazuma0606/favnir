#!/usr/bin/env bash
# seed.sh — AWS RDS PostgreSQL に customers テーブルを作成して 1000 行 INSERT
# Usage: bash scripts/seed.sh "$RDS_CONN_STR"
set -euo pipefail

RDS_CONN="${1:-${RDS_CONN_STR:-}}"
if [ -z "$RDS_CONN" ]; then
  echo "ERROR: RDS_CONN_STR が未設定です。" >&2
  echo "Usage: bash scripts/seed.sh \"host=... user=... password=... dbname=...\"" >&2
  exit 1
fi

echo "[seed] customers テーブル作成..."
psql "$RDS_CONN" <<'SQL'
CREATE TABLE IF NOT EXISTS customers (
  customer_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email       TEXT NOT NULL,
  full_name   TEXT NOT NULL,
  status      TEXT NOT NULL DEFAULT 'active',
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
TRUNCATE customers;
SQL

echo "[seed] 1000 行 INSERT (full_name に先頭/末尾スペースを含める)..."
psql "$RDS_CONN" <<'SQL'
INSERT INTO customers (email, full_name, status)
SELECT
  'user' || n || '@example.com',
  '  Test User ' || n || '  ',
  CASE WHEN n % 3 = 0 THEN 'inactive' ELSE 'active' END
FROM generate_series(1, 1000) AS n;
SQL

echo "[seed] 確認..."
psql "$RDS_CONN" -c "SELECT COUNT(*) AS total_rows FROM customers;"
echo "[seed] 完了"
