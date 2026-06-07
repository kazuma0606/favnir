#!/usr/bin/env bash
# run.sh — terraform apply → ECS タスク x2 起動 → verify
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TF_DIR="$SCRIPT_DIR/../terraform"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_FILE="/tmp/fav2py-run-$TIMESTAMP.txt"
PASS=0
FAIL=0
BUCKET="${S3_BUCKET:-favnir-e2e-demo}"

log() { echo "[$(date +%H:%M:%S)] $1" | tee -a "$LOG_FILE"; }

log "=== Favnir fav2py E2E Demo ==="
log "Timestamp: $TIMESTAMP"
log ""

# 必須 env var 確認
for var in TF_VAR_db_password; do
  if [ -z "${!var:-}" ]; then
    log "ERROR: $var is not set"
    exit 1
  fi
done

# [1/5] terraform apply
log "[1/5] terraform apply ..."
cd "$TF_DIR"
terraform init -input=false -no-color >> "$LOG_FILE" 2>&1
if terraform apply -auto-approve -input=false -no-color >> "$LOG_FILE" 2>&1; then
  log "PASS: terraform apply"
  PASS=$((PASS + 1))
else
  log "FAIL: terraform apply"
  FAIL=$((FAIL + 1))
  log "=== RESULT: PASS=$PASS FAIL=$FAIL ==="
  exit 1
fi

CLUSTER=$(terraform output -raw ecs_cluster_arn)
NATIVE_DEF=$(terraform output -raw native_task_def)
PYTHON_DEF=$(terraform output -raw python_task_def)
SUBNET=$(terraform output -raw private_subnet_id)
SG=$(terraform output -raw ecs_security_group_id)
REGION="${AWS_DEFAULT_REGION:-ap-northeast-1}"

# 古い S3 JSON をクリア（verify.sh が今回の 2 件のみを比較するため）
log "[pre] clearing old proof/fav2py/*.json ..."
aws s3 rm "s3://$BUCKET/proof/fav2py/" --recursive --exclude "*.txt" --region "$REGION" 2>&1 | tee -a "$LOG_FILE" || true

run_ecs_task() {
  local label="$1"
  local task_def="$2"
  local overrides="${3:-}"

  log "[$label] starting ECS task ..."
  local run_args=(
    --cluster "$CLUSTER"
    --task-definition "$task_def"
    --launch-type FARGATE
    --network-configuration "awsvpcConfiguration={subnets=[$SUBNET],securityGroups=[$SG],assignPublicIp=DISABLED}"
    --region "$REGION"
    --query 'tasks[0].taskArn' --output text
  )
  if [ -n "$overrides" ]; then
    run_args+=(--overrides "$overrides")
  fi
  TASK_ARN=$(aws ecs run-task "${run_args[@]}")

  if [ -z "$TASK_ARN" ] || [ "$TASK_ARN" = "None" ]; then
    log "FAIL: [$label] could not start ECS task"
    FAIL=$((FAIL + 1))
    return
  fi

  log "[$label] task ARN: $TASK_ARN"
  log "[$label] waiting for completion ..."
  aws ecs wait tasks-stopped \
    --cluster "$CLUSTER" --tasks "$TASK_ARN" --region "$REGION"

  EXIT_CODE=$(aws ecs describe-tasks \
    --cluster "$CLUSTER" --tasks "$TASK_ARN" --region "$REGION" \
    --query 'tasks[0].containers[0].exitCode' --output text)

  if [ "$EXIT_CODE" = "0" ]; then
    log "PASS: [$label] exit 0"
    PASS=$((PASS + 1))
  else
    log "FAIL: [$label] exit $EXIT_CODE"
    # CloudWatch ログを取得して表示
    log "[$label] checking CloudWatch logs ..."
    aws logs get-log-events \
      --log-group-name /ecs/fav2py \
      --log-stream-name "ecs/fav-${label##fav-}/$( echo "$TASK_ARN" | awk -F'/' '{print $NF}')" \
      --limit 30 \
      --region "$REGION" \
      --query 'events[].message' --output text 2>/dev/null | tee -a "$LOG_FILE" || true
    FAIL=$((FAIL + 1))
  fi
}

# Build credential overrides for fav-native (Favnir VM reads AWS_ACCESS_KEY_ID
# from env; ECS task role provides credentials only via metadata endpoint which
# the VM does not support, so we inject the current session credentials here).
NATIVE_CRED_OVERRIDES=""
if [ -n "${AWS_ACCESS_KEY_ID:-}" ] && [ -n "${AWS_SECRET_ACCESS_KEY:-}" ]; then
  NATIVE_CRED_OVERRIDES=$(uv run --no-project python -c "
import json, os
env = [
  {'name': 'AWS_ACCESS_KEY_ID',     'value': os.environ['AWS_ACCESS_KEY_ID']},
  {'name': 'AWS_SECRET_ACCESS_KEY', 'value': os.environ['AWS_SECRET_ACCESS_KEY']},
  {'name': 'AWS_SESSION_TOKEN',     'value': os.environ.get('AWS_SESSION_TOKEN', '')},
  {'name': 'AWS_DEFAULT_REGION',    'value': '$REGION'},
]
print(json.dumps({'containerOverrides': [{'name': 'fav-native', 'environment': env}]}))
" 2>/dev/null)
  log "[fav-native] injecting AWS credentials from current session"
fi

# [2/5] fav-native
run_ecs_task "fav-native" "$NATIVE_DEF" "$NATIVE_CRED_OVERRIDES"

# [3/5] fav-python
run_ecs_task "fav-python" "$PYTHON_DEF"

# [4/5] verify
log "[4/5] running verify.sh ..."
if BUCKET="$BUCKET" "$SCRIPT_DIR/verify.sh" 2>&1 | tee -a "$LOG_FILE"; then
  PASS=$((PASS + 1))
else
  FAIL=$((FAIL + 1))
fi

# [5/5] upload run log
log "[5/5] uploading run log ..."
if aws s3 cp "$LOG_FILE" "s3://$BUCKET/proof/fav2py/run-$TIMESTAMP.txt" 2>&1; then
  log "PASS: run log uploaded to s3://$BUCKET/proof/fav2py/run-$TIMESTAMP.txt"
  PASS=$((PASS + 1))
else
  log "WARN: run log upload failed (non-fatal)"
fi

log ""
log "=== RESULT: PASS=$PASS FAIL=$FAIL ==="
[ "$FAIL" -eq 0 ] || exit 1
