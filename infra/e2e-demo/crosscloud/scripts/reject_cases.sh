#!/bin/bash
# reject_cases.sh — 認証失敗ケースを検証する（PASS 条件: 401/409 が返ること）
# Usage: ./reject_cases.sh <api_endpoint> <hmac_secret> <cognito_client_id> <username> <password>
set -euo pipefail

API_ENDPOINT="${1:-}"
HMAC_SECRET="${2:-}"
CLIENT_ID="${3:-}"
USERNAME="${4:-}"
PASSWORD="${5:-}"

REGION="${AWS_REGION:-ap-northeast-1}"
PASS=0
FAIL=0

check() {
  local label="$1"
  local expected_status="$2"
  local actual_status="$3"
  if [ "$actual_status" = "$expected_status" ]; then
    echo "  [PASS] ${label} => HTTP ${actual_status}"
    PASS=$(( PASS + 1 ))
  else
    echo "  [FAIL] ${label} => expected HTTP ${expected_status}, got ${actual_status}"
    FAIL=$(( FAIL + 1 ))
  fi
}

# ── Cognito トークン取得 ───────────────────────────────────────────────
AUTH_RESULT=$(aws cognito-idp initiate-auth \
  --region "$REGION" \
  --auth-flow USER_PASSWORD_AUTH \
  --client-id "$CLIENT_ID" \
  --auth-parameters "USERNAME=${USERNAME},PASSWORD=${PASSWORD}" \
  --query 'AuthenticationResult.IdToken' \
  --output text)

METHOD="POST"
PATH_VAL="/migrate"
BODY='{"action":"migrate"}'
BODY_HASH=$(echo -n "$BODY" | sha256sum | cut -d' ' -f1)
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
NONCE=$(uuidgen | tr '[:upper:]' '[:lower:]')

good_sig() {
  local ts="$1" nonce="$2"
  local sts="${METHOD}
${PATH_VAL}
${ts}
${nonce}
${BODY_HASH}"
  echo -n "$sts" | openssl dgst -sha256 -hmac "$HMAC_SECRET" | sed 's/.*= //'
}

echo "=== ケース 1: 正しい署名 (200 期待) ==="
SIG=$(good_sig "$TIMESTAMP" "$NONCE")
STATUS=$(curl -sS -o /dev/null -w "%{http_code}" \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Authorization: Bearer ${AUTH_RESULT}" \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Nonce: ${NONCE}" \
  -H "X-Signature: ${SIG}" \
  -d "$BODY")
check "valid_request" "200" "$STATUS"

echo "=== ケース 2: 無効な署名 (401 期待) ==="
BAD_NONCE2=$(uuidgen | tr '[:upper:]' '[:lower:]')
TIMESTAMP2=$(date -u +%Y-%m-%dT%H:%M:%SZ)
STATUS=$(curl -sS -o /dev/null -w "%{http_code}" \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Authorization: Bearer ${AUTH_RESULT}" \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP2}" \
  -H "X-Nonce: ${BAD_NONCE2}" \
  -H "X-Signature: invalidsignaturexxx" \
  -d "$BODY")
check "invalid_signature" "401" "$STATUS"

echo "=== ケース 3: Nonce 再利用（リプレイ攻撃、409 期待）==="
# ケース 1 と同じ nonce を再送
STATUS=$(curl -sS -o /dev/null -w "%{http_code}" \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Authorization: Bearer ${AUTH_RESULT}" \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Nonce: ${NONCE}" \
  -H "X-Signature: ${SIG}" \
  -d "$BODY")
check "nonce_replay" "409" "$STATUS"

echo "=== ケース 4: Cognito トークンなし (401 期待) ==="
STATUS=$(curl -sS -o /dev/null -w "%{http_code}" \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Content-Type: application/json" \
  -d "$BODY")
check "no_jwt_token" "401" "$STATUS"

echo "=== ケース 5: 無効な Cognito トークン (401 期待) ==="
NONCE5=$(uuidgen | tr '[:upper:]' '[:lower:]')
TIMESTAMP5=$(date -u +%Y-%m-%dT%H:%M:%SZ)
SIG5=$(good_sig "$TIMESTAMP5" "$NONCE5")
STATUS=$(curl -sS -o /dev/null -w "%{http_code}" \
  -X POST "${API_ENDPOINT}/migrate" \
  -H "Authorization: Bearer invalid.jwt.token" \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP5}" \
  -H "X-Nonce: ${NONCE5}" \
  -H "X-Signature: ${SIG5}" \
  -d "$BODY")
check "invalid_jwt" "401" "$STATUS"

echo ""
echo "=== 結果 ==="
echo "PASS=${PASS} FAIL=${FAIL}"
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
