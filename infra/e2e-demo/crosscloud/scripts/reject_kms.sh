#!/bin/bash
# reject_kms.sh — KMS ECDSA 署名の拒否ケース検証（v15.1.5）
# Usage: ./reject_kms.sh <api_endpoint> <kms_key_id> <cognito_client_id> <username> <password>
# 期待結果: PASS=2 FAIL=0
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
PASS=0
FAIL=0

check() {
  local LABEL="$1"
  local EXPECTED="$2"
  local ACTUAL="$3"
  if [ "$ACTUAL" = "$EXPECTED" ]; then
    echo "[PASS] ${LABEL} (${ACTUAL})"
    PASS=$((PASS+1))
  else
    echo "[FAIL] ${LABEL} — expected ${EXPECTED}, got ${ACTUAL}"
    FAIL=$((FAIL+1))
  fi
}

# ── Cognito 認証トークン取得 ───────────────────────────────────────────
AUTH_TOKEN=$(aws cognito-idp initiate-auth \
  --region "$REGION" \
  --auth-flow USER_PASSWORD_AUTH \
  --client-id "$CLIENT_ID" \
  --auth-parameters "USERNAME=${USERNAME},PASSWORD=${PASSWORD}" \
  --query 'AuthenticationResult.IdToken' \
  --output text)

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
BODY='{"action":"migrate"}'
BODY_HASH=$(echo -n "$BODY" | sha256sum | cut -d' ' -f1)

# ── 正当な署名を取得しておく ─────────────────────────────────────────
NONCE_VALID=$(uuidgen 2>/dev/null || uv run python -c "import uuid; print(uuid.uuid4())")
STRING_TO_SIGN="POST
/migrate-kms
${TIMESTAMP}
${NONCE_VALID}
${BODY_HASH}"

VALID_SIG=$(MSYS_NO_PATHCONV=1 aws kms sign \
  --region "$REGION" \
  --key-id "$KMS_KEY_ID" \
  --signing-algorithm ECDSA_SHA_256 \
  --message-type RAW \
  --message "$(printf '%s' "$STRING_TO_SIGN" | base64 | tr -d '\n')" \
  --query "Signature" \
  --output text)

# ── ケース 1: 改ざんボディ（StringToSign 不一致 → ECDSA 検証失敗 → 401）──
echo ""
echo "[REJECT 1] 改ざんボディ（ECDSA 検証失敗）→ 401 期待"
TAMPERED_BODY='{"action":"drop_all_tables"}'
STATUS=$(curl -sS -o /dev/null -w "%{http_code}" \
  -X POST "${API_ENDPOINT}/migrate-kms" \
  -H "Authorization: Bearer ${AUTH_TOKEN}" \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Nonce: ${NONCE_VALID}" \
  -H "X-Signature: ${VALID_SIG}" \
  -H "X-KMS-Key-Id: ${KMS_KEY_ID}" \
  -d "$TAMPERED_BODY")
check "改ざんボディ → 401" "401" "$STATUS"

# ── ケース 2: ランダム（不正）署名（DER 検証失敗 → 401）────────────────
echo ""
echo "[REJECT 2] ランダム署名（不正 DER バイト列）→ 401 期待"
NONCE_RAND=$(uuidgen 2>/dev/null || uv run python -c "import uuid; print(uuid.uuid4())")
RANDOM_SIG=$(dd if=/dev/urandom bs=64 count=1 2>/dev/null | base64 | tr -d '\n')
STATUS=$(curl -sS -o /dev/null -w "%{http_code}" \
  -X POST "${API_ENDPOINT}/migrate-kms" \
  -H "Authorization: Bearer ${AUTH_TOKEN}" \
  -H "Content-Type: application/json" \
  -H "X-Timestamp: ${TIMESTAMP}" \
  -H "X-Nonce: ${NONCE_RAND}" \
  -H "X-Signature: ${RANDOM_SIG}" \
  -H "X-KMS-Key-Id: ${KMS_KEY_ID}" \
  -d "$BODY")
check "ランダム署名 → 401" "401" "$STATUS"

# ── 結果 ─────────────────────────────────────────────────────────────
echo ""
echo "REJECT PASS=${PASS} FAIL=${FAIL}"
if [ "$FAIL" -ne 0 ]; then
  exit 1
fi
