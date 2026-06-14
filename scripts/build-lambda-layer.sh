#!/usr/bin/env bash
# scripts/build-lambda-layer.sh
# fav バイナリを AWS Lambda 用に cross-compile して zip にパッケージングする (v15.5.0)
#
# 前提:
#   - cross crate がインストール済み: cargo install cross
#   - Docker が起動中であること（cross は Docker を使用）
#
# 使い方:
#   ./scripts/build-lambda-layer.sh [output.zip]
#
# 出力:
#   function.zip（または指定パス）— Lambda に直接アップロード可能

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FAV_DIR="$(cd "$SCRIPT_DIR/../fav" && pwd)"
OUTPUT_ZIP="${1:-function.zip}"

echo "=== Favnir Lambda Layer Build ==="
echo "Source: $FAV_DIR"
echo "Output: $OUTPUT_ZIP"

# cross-compile（Linux x86_64 musl）
echo ""
echo "Step 1: cross-compiling fav binary..."
cd "$FAV_DIR"
cross build --release --target x86_64-unknown-linux-musl --bin fav

BINARY="$FAV_DIR/target/x86_64-unknown-linux-musl/release/fav"
if [ ! -f "$BINARY" ]; then
    echo "Error: binary not found at $BINARY"
    exit 1
fi

# 一時ディレクトリにパッケージング
TMP_DIR="$(mktemp -d)"
trap "rm -rf $TMP_DIR" EXIT

cp "$BINARY" "$TMP_DIR/fav"

# bootstrap スクリプト（Lambda custom runtime エントリポイント）
cat > "$TMP_DIR/bootstrap" << 'EOF'
#!/bin/sh
# Lambda custom runtime bootstrap for Favnir
set -e

FAV_FILE="${FAV_FILE:-/var/task/main.fav}"

while true; do
  # Lambda Runtime API から次のリクエストを取得
  REQUEST=$(curl -sS "http://${AWS_LAMBDA_RUNTIME_API}/2018-06-01/runtime/invocation/next")
  REQUEST_ID=$(curl -sS -D - "http://${AWS_LAMBDA_RUNTIME_API}/2018-06-01/runtime/invocation/next" -o /dev/null | grep Lambda-Runtime-Aws-Request-Id | tr -d '\r' | awk '{print $2}')

  # fav パイプラインを実行
  if RESPONSE=$(/var/task/fav run --legacy "$FAV_FILE" 2>&1); then
    STATUS="success"
  else
    STATUS="error"
  fi

  # レスポンスを Lambda Runtime API に送信
  curl -sS -X POST \
    "http://${AWS_LAMBDA_RUNTIME_API}/2018-06-01/runtime/invocation/${REQUEST_ID}/response" \
    -H "Content-Type: application/json" \
    -d "{\"status\": \"${STATUS}\", \"output\": $(echo "$RESPONSE" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')}"
done
EOF
chmod +x "$TMP_DIR/bootstrap"

# zip 生成
echo ""
echo "Step 2: packaging into $OUTPUT_ZIP..."
(cd "$TMP_DIR" && zip -j "$OLDPWD/$OUTPUT_ZIP" fav bootstrap)

echo ""
echo "=== Build complete ==="
echo "Output: $OUTPUT_ZIP"
echo "Contents:"
unzip -l "$OUTPUT_ZIP"
