#!/bin/bash
# Machine A (Public EC2) — Favnir 処理系サーバ
# 役割: .fav ソースをコンパイルして .fvc アーティファクトを S3 にアップロードする
set -e
exec > >(tee -a /var/log/favnir-machine-a.log) 2>&1

BUCKET="${bucket_name}"
echo "[$(date)] Machine A starting"

# ── fav バイナリのインストール ─────────────────────────────────────────────────
aws s3 cp "s3://$BUCKET/bootstrap/fav" /usr/local/bin/fav
chmod +x /usr/local/bin/fav
echo "[$(date)] fav installed: $(fav --version)"

# ── Favnir ソースを配置 ────────────────────────────────────────────────────────
mkdir -p /app/src
cat > /app/src/etl.fav << 'FAVSRC'
${etl_src}
FAVSRC

cat > /app/src/pipeline.fav << 'FAVSRC'
${pipeline_src}
FAVSRC

echo "[$(date)] Source files written"

# ── 証跡: Machine A のファイル一覧を S3 に保存 ───────────────────────────────
# 期待値: /app/src/*.fav が存在し、/usr/local/bin/fav が存在する
TS=$(date +%Y%m%d-%H%M%S)
{
  echo "=== Machine A: Favnir toolchain server ==="
  echo "=== Date: $(date) ==="
  echo ""
  echo "--- /app/src/ (Favnir source files) ---"
  find /app/src -type f | sort
  echo ""
  echo "--- /usr/local/bin/ ---"
  ls -la /usr/local/bin/
} > /tmp/machine-a-filelist.txt

aws s3 cp /tmp/machine-a-filelist.txt \
  "s3://$BUCKET/proof/machine-a/filelist-$TS.txt"
echo "[$(date)] Proof uploaded: s3://$BUCKET/proof/machine-a/filelist-$TS.txt"

# ── ビルド: .fav → .fvc ───────────────────────────────────────────────────────
echo "[$(date)] Building etl.fav..."
fav build /app/src/etl.fav -o /tmp/etl.fvc
echo "[$(date)] Building pipeline.fav..."
fav build /app/src/pipeline.fav -o /tmp/pipeline.fvc

# ── アーティファクトを S3 にアップロード ──────────────────────────────────────
aws s3 cp /tmp/etl.fvc      "s3://$BUCKET/artifacts/etl.fvc"
aws s3 cp /tmp/pipeline.fvc "s3://$BUCKET/artifacts/pipeline.fvc"
echo "[$(date)] Artifacts uploaded"
echo "[$(date)] Machine A done"
