#!/bin/bash
# ECS Task を手動起動するスクリプト（Phase 8-D）
# terraform apply 後に terraform output で値を取得してから実行する
#
# 使用方法（terraform ディレクトリから）:
#   bash ../scripts/run-ecs-task.sh
set -e

AWS_REGION="${AWS_REGION:-ap-northeast-1}"
TF_DIR="$(dirname "$0")/../terraform"

echo "[1/3] Fetching Terraform outputs..."
CLUSTER_ARN=$(terraform -chdir="$TF_DIR" output -raw ecs_cluster_arn)
TASK_DEF_ARN=$(terraform -chdir="$TF_DIR" output -raw ecs_task_definition_arn)
SUBNET_ID=$(terraform -chdir="$TF_DIR" output -raw private_subnet_id)
SG_ID=$(terraform -chdir="$TF_DIR" output -raw ecs_security_group_id)
BUCKET=$(terraform -chdir="$TF_DIR" output -raw s3_bucket_name)

echo "  Cluster  : $CLUSTER_ARN"
echo "  Task Def : $TASK_DEF_ARN"
echo "  Subnet   : $SUBNET_ID"
echo "  SG       : $SG_ID"

echo "[2/3] Checking artifacts exist in S3..."
aws s3 ls "s3://$BUCKET/artifacts/etl.fvc" \
  || { echo "ERROR: etl.fvc not found in S3. Run Machine A first."; exit 1; }
echo "  OK: etl.fvc found"

echo "[3/3] Starting ECS Task..."
TASK_ARN=$(aws ecs run-task \
  --region "$AWS_REGION" \
  --cluster "$CLUSTER_ARN" \
  --task-definition "$TASK_DEF_ARN" \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={
    subnets=[$SUBNET_ID],
    securityGroups=[$SG_ID],
    assignPublicIp=DISABLED
  }" \
  --query "tasks[0].taskArn" \
  --output text)

echo "  Task ARN: $TASK_ARN"
echo ""
echo "Watching task status (Ctrl+C to stop watching, task continues running)..."

# タスクが STOPPED になるまでポーリング
for i in $(seq 1 60); do
  STATUS=$(aws ecs describe-tasks \
    --region "$AWS_REGION" \
    --cluster "$CLUSTER_ARN" \
    --tasks "$TASK_ARN" \
    --query "tasks[0].lastStatus" \
    --output text)
  echo "  [$i/60] Status: $STATUS"

  if [ "$STATUS" = "STOPPED" ]; then
    EXIT_CODE=$(aws ecs describe-tasks \
      --region "$AWS_REGION" \
      --cluster "$CLUSTER_ARN" \
      --tasks "$TASK_ARN" \
      --query "tasks[0].containers[?name=='etl-runner'].exitCode" \
      --output text)
    echo ""
    if [ "$EXIT_CODE" = "0" ]; then
      echo "ECS Task completed successfully (exit code: 0)"
    else
      echo "ECS Task failed (exit code: $EXIT_CODE)"
      exit 1
    fi
    break
  fi
  sleep 10
done
