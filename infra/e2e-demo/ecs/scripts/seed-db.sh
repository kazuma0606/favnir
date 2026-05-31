#!/bin/bash
# データベースにサンプルデータを投入するスクリプト
# Machine A から実行する（同一 VPC 内から RDS に接続）
#
# 使用方法:
#   DB_HOST=<rds-endpoint> DB_PASS=<password> bash seed-db.sh
#
# terraform output rds_endpoint で RDS エンドポイントを取得できる
set -e

DB_HOST="${DB_HOST:?DB_HOST is required}"
DB_USER="${DB_USER:-favnir}"
DB_PASS="${DB_PASS:?DB_PASS is required}"
DB_NAME="${DB_NAME:-demo}"

echo "[$(date)] Installing psql..."
apt-get update -qq && apt-get install -y -qq postgresql-client

echo "[$(date)] Seeding database: $DB_HOST/$DB_NAME"

PGPASSWORD="$DB_PASS" psql \
  -h "$DB_HOST" \
  -U "$DB_USER" \
  -d "$DB_NAME" << 'SQL'

-- orders テーブルの作成
CREATE TABLE IF NOT EXISTS orders (
  id          SERIAL PRIMARY KEY,
  customer    VARCHAR(100) NOT NULL,
  amount      NUMERIC(10,2) NOT NULL,
  created_at  TIMESTAMP DEFAULT NOW()
);

-- 既存データをクリア（冪等性のため）
TRUNCATE TABLE orders RESTART IDENTITY;

-- サンプルデータ投入
INSERT INTO orders (customer, amount) VALUES
  ('Alice', 1200.00),
  ('Bob',    800.50),
  ('Alice',  350.00),
  ('Carol', 2100.00),
  ('Bob',    450.75),
  ('Carol',  600.00),
  ('Alice', 3200.00),
  ('Dave',   980.00),
  ('Carol',  125.50),
  ('Dave',  1500.00);

-- 投入確認
SELECT customer, COUNT(*) AS orders, SUM(amount) AS total
FROM orders
GROUP BY customer
ORDER BY customer;

SQL

echo "[$(date)] Seed complete"
