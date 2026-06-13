#!/bin/bash
# run_with_kms.sh — Cognito 認証 + KMS ECDSA 署名付きで /migrate を呼び出す（v15.1.5）
# Usage: ./run_with_kms.sh <api_endpoint> <kms_key_id> <cognito_client_id> <username> <password>
set -euo pipefail

API_ENDPOINT="${1:-}"
KMS_KEY_ID="${2:-}"
CLIENT_ID="${3:-}"
USERNAME="${4:-}"
PASSWORD="${5:-}"

if [ -z "$API_ENDPOINT" ] || [ -z "$KMS_KEY_ID" ] || [ -z "$CLIENT_ID" ]; then
  echo "Usage: $0 <api_endpoint> <kms_key_id> <cognito_client_id> <username> <password>"
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

# ── StringToSign 構築 ──────────────────────────────────────────────────
METHOD="POST"
PATH_="/migrate-kms"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
NONCE=$(uuidgen 2>/dev/null || python3 -c "import uuid; print(uuid.uuid4())" | tr '[:upper:]' '[:lower:]')
BODY='{"action":"migrate"}'

BODY_HASH=$(echo -n "$BODY" | sha256sum | cut -d' ' -f1)
STRING_TO_SIGN="${METHOD}
${PATH_}
${TIMESTAMP}
${NONCE}
${BODY_HASH}"

echo "[2] StringToSign 構築完了"

# ── KMS ECDSA_SHA_256 で署名 ───────────────────────────────────────────
echo "[2] KMS 署名中..."
SIGNATURE=$(MSYS_NO_PATHCONV=1 aws kms sign \
  --region "$REGION" \
  --key-id "$KMS_KEY_ID" \
  --signing-algorithm ECDSA_SHA_256 \
  --message-type RAW \
  --message "$(printf '%s' "$STRING_TO_SIGN" | base64 | tr -d '\n')" \
  --query "Signature" \
  --output text)

echo "[2] 署名完了"
echo "    Timestamp : $TIMESTAMP"
echo "    Nonce     : $NONCE"
echo "    Signature : ${SIGNATURE:0:40}..."

# ── API Gateway に POST ────────────────────────────────────────────────
echo "[3] POST ${API_ENDPOINT}/migrate ..."
HTTP_STATUS=$(curl -sS -o /tmp/kms_response.json -w "%{http_code}" \
  -X POST "${API_ENDPOINT}/migrate-kms" \
  -H "Authorization: Bearer ${AUTH_RESULT}" \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Nonce: ${NONCE}" \
  -H "X-Signature: ${SIGNATURE}" \
  -H "X-KMS-Key-Id: ${KMS_KEY_ID}" \
  -d "$BODY")

echo "[3] HTTP Status: ${HTTP_STATUS}"
echo "[3] Response:"
cat /tmp/kms_response.json
echo

if [ "$HTTP_STATUS" = "200" ]; then
  echo "[OK] KMS 署名検証通過 — ECS タスクが起動されました"
else
  echo "[FAIL] 予期しないステータス: ${HTTP_STATUS}"
  exit 1
fi
