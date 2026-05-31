#!/bin/bash
# scripts/run-jobs.sh
# Kubernetes Job を順次実行する（infra/e2e-demo/eks/ から実行すること）
set -euo pipefail

AWS_REGION=ap-northeast-1
CLUSTER=favnir-eks-demo
AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
ECR_BASE="${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com"
ECR_RUNTIME="${ECR_BASE}/favnir-runtime"
ECR_TOOLCHAIN="${ECR_BASE}/favnir-toolchain"

echo "=== kubeconfig 更新 ==="
aws eks update-kubeconfig --name "$CLUSTER" --region "$AWS_REGION"

echo "=== Terraform output から Role ARN を取得 ==="
COMPILER_ROLE=$(terraform -chdir=terraform output -raw eks_compiler_role_arn)
EXECUTOR_ROLE=$(terraform -chdir=terraform output -raw eks_executor_role_arn)
echo "Compiler Role: $COMPILER_ROLE"
echo "Executor Role: $EXECUTOR_ROLE"

echo "=== Namespace + ServiceAccount 適用 ==="
kubectl apply -f k8s/namespace.yaml

# ServiceAccount に Role ARN を注入
sed \
  -e "s|COMPILER_ROLE_ARN|${COMPILER_ROLE}|g" \
  -e "s|EXECUTOR_ROLE_ARN|${EXECUTOR_ROLE}|g" \
  k8s/serviceaccount.yaml | kubectl apply -f -

echo "=== Compiler Job 起動 ==="
# 既存の Job を削除してから再作成
kubectl delete job favnir-compiler -n favnir-demo --ignore-not-found=true

sed "s|ECR_TOOLCHAIN_URI|${ECR_TOOLCHAIN}|g" k8s/compiler-job.yaml \
  | kubectl apply -f -

echo "Waiting for Compiler Job to complete (timeout: 5min)..."
kubectl wait \
  --for=condition=complete \
  job/favnir-compiler \
  -n favnir-demo \
  --timeout=300s

echo "Compiler Job logs:"
kubectl logs job/favnir-compiler -n favnir-demo

echo ""
echo "=== Executor Job 起動 ==="
kubectl delete job favnir-executor -n favnir-demo --ignore-not-found=true

sed "s|ECR_RUNTIME_URI|${ECR_RUNTIME}|g" k8s/executor-job.yaml \
  | kubectl apply -f -

echo "Waiting for Executor Job to complete (timeout: 10min)..."
kubectl wait \
  --for=condition=complete \
  job/favnir-executor \
  -n favnir-demo \
  --timeout=600s

echo "Executor Job logs:"
kubectl logs job/favnir-executor -n favnir-demo

echo ""
echo "Both Jobs completed. Run scripts/verify.sh to check results."
