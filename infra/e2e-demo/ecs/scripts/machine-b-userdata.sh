#!/bin/bash
# Machine B (Private EC2) — Rust VM サーバ
# 役割: .fvc アーティファクトのみで fav exec を実行する（.fav ソース一切なし）
set -e
exec > >(tee -a /var/log/favnir-machine-b.log) 2>&1

BUCKET="${bucket_name}"
echo "[$(date)] Machine B starting"

# ── fav バイナリのインストール（ソースコードは一切インストールしない）─────────
aws s3 cp "s3://$BUCKET/bootstrap/fav" /usr/local/bin/fav
chmod +x /usr/local/bin/fav
echo "[$(date)] fav installed: $(fav --version)"

# ── 証跡: .fav ファイルが一切存在しないことを S3 に記録 ──────────────────────
# 期待値: .fav 検索結果が 0 件
TS=$(date +%Y%m%d-%H%M%S)
{
  echo "=== Machine B: Rust VM only (no .fav source files) ==="
  echo "=== Date: $(date) ==="
  echo ""
  echo "--- .fav file search (expect: 0 results) ---"
  find / -name "*.fav" 2>/dev/null
  echo "(end of .fav search)"
  echo ""
  echo "--- /usr/local/bin/ ---"
  ls -la /usr/local/bin/
} > /tmp/machine-b-proof.txt

aws s3 cp /tmp/machine-b-proof.txt \
  "s3://$BUCKET/proof/machine-b/fav-search-$TS.txt"
echo "[$(date)] Proof uploaded: s3://$BUCKET/proof/machine-b/fav-search-$TS.txt"

# ── pipeline.fvc が S3 に存在するまで待機（Machine A の完了を待つ）─────────────
echo "[$(date)] Waiting for pipeline.fvc..."
for i in $(seq 1 30); do
  if aws s3 ls "s3://$BUCKET/artifacts/pipeline.fvc" > /dev/null 2>&1; then
    echo "[$(date)] Artifact found"
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "[$(date)] ERROR: pipeline.fvc not found after 5 minutes"
    exit 1
  fi
  echo "[$(date)] Waiting... ($i/30)"
  sleep 10
done

# ── アーティファクトを取得して実行 ────────────────────────────────────────────
aws s3 cp "s3://$BUCKET/artifacts/pipeline.fvc" /tmp/pipeline.fvc
echo "[$(date)] Executing pipeline.fvc (no .fav source, artifact only)..."
BUCKET_NAME="$BUCKET" fav exec /tmp/pipeline.fvc
echo "[$(date)] Pipeline complete"

# ── 実行ログを S3 に保存 ──────────────────────────────────────────────────────
aws s3 cp /var/log/favnir-machine-b.log \
  "s3://$BUCKET/logs/machine-b-$(date +%Y%m%d-%H%M%S).log"

# ── 自己 stop ─────────────────────────────────────────────────────────────────
INSTANCE_ID=$(curl -s http://169.254.169.254/latest/meta-data/instance-id)
echo "[$(date)] Stopping instance: $INSTANCE_ID"
aws ec2 stop-instances --instance-ids "$INSTANCE_ID"
