#!/usr/bin/env bash
# run.sh — terraform apply → ECS タスク x2 起動 → verify
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TF_DIR="$SCRIPT_DIR/../terraform"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_FILE="/tmp/fav2py-run-$TIMESTAMP.txt"
PASS=0
FAIL=0

log() { echo "[$(date +%H:%M:%S)] $1" | tee -a "$LOG_FILE"; }

log "=== Favnir v11.9.0 fav2py E2E Demo ==="
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

run_ecs_task() {
  local label="$1"
  local task_def="$2"

  log "[$label] starting ECS task ..."
  TASK_ARN=$(aws ecs run-task \
    --cluster "$CLUSTER" \
    --task-definition "$task_def" \
    --launch-type FARGATE \
    --network-configuration \
      "awsvpcConfiguration={subnets=[$SUBNET],securityGroups=[$SG],assignPublicIp=DISABLED}" \
    --region "$REGION" \
    --query 'tasks[0].taskArn' --output text)

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
    FAIL=$((FAIL + 1))
  fi
}

# [2/5] fav-native
run_ecs_task "fav-native" "$NATIVE_DEF"

# [3/5] fav-python
run_ecs_task "fav-python" "$PYTHON_DEF"

# [4/5] verify
log "[4/5] running verify.sh ..."
if S3_BUCKET="${S3_BUCKET:-favnir-e2e-demo}" "$SCRIPT_DIR/verify.sh" 2>&1 | tee -a "$LOG_FILE"; then
  PASS=$((PASS + 1))
else
  FAIL=$((FAIL + 1))
fi

# [5/5] upload run log
log "[5/5] uploading run log ..."
BUCKET="${S3_BUCKET:-favnir-e2e-demo}"
if aws s3 cp "$LOG_FILE" "s3://$BUCKET/proof/fav2py/run-$TIMESTAMP.txt" >> "$LOG_FILE" 2>&1; then
  log "PASS: run log uploaded"
  PASS=$((PASS + 1))
else
  log "WARN: run log upload failed (non-fatal)"
fi

log ""
log "=== RESULT: PASS=$PASS FAIL=$FAIL ==="
[ "$FAIL" -eq 0 ] || exit 1
