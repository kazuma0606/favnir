#!/bin/bash
# run_with_auth.sh — Cognito 認証 + HMAC 署名付きで /migrate を呼び出す
# Usage: ./run_with_auth.sh <api_endpoint> <hmac_secret> <cognito_user_pool_id> <cognito_client_id> <username> <password>
set -euo pipefail

API_ENDPOINT="${1:-}"
HMAC_SECRET="${2:-}"
USER_POOL_ID="${3:-}"
CLIENT_ID="${4:-}"
USERNAME="${5:-}"
PASSWORD="${6:-}"

if [ -z "$API_ENDPOINT" ] || [ -z "$HMAC_SECRET" ] || [ -z "$USER_POOL_ID" ] || [ -z "$CLIENT_ID" ]; then
  echo "Usage: $0 <api_endpoint> <hmac_secret> <cognito_user_pool_id> <cognito_client_id> <username> <password>"
  exit 1
fi

REGION="${AWS_REGION:-ap-northeast-1}"

# ── Cognito 認証トークン取得 ───────────────────────────────────────────
echo "[1] Cognito 認証中..."
AUTH_RESULT=$(aws cognito-idp initiate-auth \
  --region "$REGION" \
  --auth-flow USER_PASSWORD_AUTH \
  --client-id "$CLIENT_ID" \
  --auth-parameters "USERNAME=${USERNAME},PASSWORD=${PASSWORD}" \
  --query 'AuthenticationResult.IdToken' \
  --output text)
echo "[1] OK — IdToken 取得完了"

# ── HMAC-SHA256 署名を計算 ─────────────────────────────────────────────
METHOD="POST"
PATH="/migrate"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
NONCE=$(uuidgen | tr '[:upper:]' '[:lower:]')
BODY='{"action":"migrate"}'

BODY_HASH=$(echo -n "$BODY" | sha256sum | cut -d' ' -f1)
STRING_TO_SIGN="${METHOD}
${PATH}
${TIMESTAMP}
${NONCE}
${BODY_HASH}"

SIGNATURE=$(echo -n "$STRING_TO_SIGN" | openssl dgst -sha256 -hmac "$HMAC_SECRET" | sed 's/.*= //')

echo "[2] 署名計算完了"
echo "    Timestamp : $TIMESTAMP"
echo "    Nonce     : $NONCE"
echo "    Signature : $SIGNATURE"

# ── API Gateway に POST ────────────────────────────────────────────────
echo "[3] POST ${API_ENDPOINT}/migrate ..."
HTTP_STATUS=$(curl -sS -o /tmp/response.json -w "%{http_code}" \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Authorization: Bearer ${AUTH_RESULT}" \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Nonce: ${NONCE}" \
  -H "X-Signature: ${SIGNATURE}" \
  -d "$BODY")

echo "[3] HTTP Status: ${HTTP_STATUS}"
echo "[3] Response:"
cat /tmp/response.json
echo

if [ "$HTTP_STATUS" = "200" ]; then
  echo "[OK] 移行リクエスト受理 — ECS タスクが起動されました"
else
  echo "[FAIL] 予期しないステータス: ${HTTP_STATUS}"
  exit 1
fi
