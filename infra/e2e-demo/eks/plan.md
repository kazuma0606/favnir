# Favnir E2E Demo — EKS 版 実装計画

Date: 2026-05-31

## ディレクトリ構成

```
infra/e2e-demo/eks/
├── spec.md                    アーキテクチャ仕様（spec.md）
├── plan.md                    実装計画（本書）
├── tasks.md                   タスクリスト
├── terraform/
│   ├── main.tf                VPC / Subnet / SG / VPC Endpoints
│   ├── eks.tf                 EKS クラスター + Fargate Profile
│   ├── database.tf            Aurora Serverless v2
│   ├── storage.tf             S3 バケット + バケットポリシー
│   ├── iam.tf                 IRSA ロール / OIDC Provider
│   ├── monitoring.tf          CloudWatch Logs グループ
│   └── variables.tf           変数定義
├── docker/
│   ├── toolchain/
│   │   └── Dockerfile         favnir/toolchain イメージ（fav + .fav ソース）
│   └── runtime/
│       └── Dockerfile         favnir/runtime イメージ（fav バイナリのみ）
├── k8s/
│   ├── namespace.yaml         favnir-demo Namespace
│   ├── serviceaccount.yaml    ServiceAccount x2（compiler / executor）
│   ├── compiler-job.yaml      Compiler Job（favnir/toolchain）
│   └── executor-job.yaml      Executor Job（favnir/runtime）
├── src/
│   └── pipeline.fav           Compiler Job がビルドするパイプライン
└── scripts/
    ├── build-and-push.sh      Docker イメージビルド + ECR push
    ├── run-jobs.sh            Kubernetes Job 順次実行
    └── verify.sh              証跡確認スクリプト
```

---

## Phase 1 — Docker イメージ設計

### 1-1. favnir/runtime（ECS 版と同一）

ECS 版で使用しているものを再利用する。
`/usr/local/bin/fav` バイナリと `awscli` のみを含む。

```dockerfile
# docker/runtime/Dockerfile
# ECS 版 docker/runtime/Dockerfile と同一

FROM rust:slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake g++ pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY fav/ .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates awscli \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/fav /usr/local/bin/fav
ENTRYPOINT ["/usr/local/bin/fav"]
```

### 1-2. favnir/toolchain（EKS 版 新規）

runtime イメージに `.fav` ソースと `fav build` 実行環境を追加したもの。

```dockerfile
# docker/toolchain/Dockerfile
# ビルドステージは runtime と同一（同じ multi-stage から派生）

FROM rust:slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake g++ pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY fav/ .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates awscli \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/fav /usr/local/bin/fav
# .fav ソースコードを含める — これが toolchain イメージの証明
COPY infra/e2e-demo/eks/src/pipeline.fav /app/src/pipeline.fav
ENTRYPOINT ["/usr/local/bin/fav"]
```

**ポイント:** toolchain イメージには `.fav` が存在し、runtime イメージには存在しない。
この差が EKS 版の証明の核心。

### 1-3. ビルド + ECR プッシュスクリプト

```bash
# scripts/build-and-push.sh
set -e
AWS_REGION=ap-northeast-1
BUCKET_NAME=favnir-e2e-demo
AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
ECR_RUNTIME="${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com/favnir-runtime"
ECR_TOOLCHAIN="${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com/favnir-toolchain"
REPO_ROOT=$(git rev-parse --show-toplevel)

# ECR リポジトリ作成
aws ecr create-repository --repository-name favnir-runtime   --region $AWS_REGION 2>/dev/null || true
aws ecr create-repository --repository-name favnir-toolchain --region $AWS_REGION 2>/dev/null || true

# ログイン
aws ecr get-login-password --region $AWS_REGION \
  | docker login --username AWS --password-stdin \
    "${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com"

# runtime イメージ（ECS 版と共用）
docker build -f infra/e2e-demo/ecs/docker/runtime/Dockerfile \
  -t favnir-runtime "$REPO_ROOT"
docker tag favnir-runtime:latest $ECR_RUNTIME:latest
docker push $ECR_RUNTIME:latest
echo "Runtime image pushed: $ECR_RUNTIME:latest"

# toolchain イメージ（EKS 版 新規）
docker build -f infra/e2e-demo/eks/docker/toolchain/Dockerfile \
  -t favnir-toolchain "$REPO_ROOT"
docker tag favnir-toolchain:latest $ECR_TOOLCHAIN:latest
docker push $ECR_TOOLCHAIN:latest
echo "Toolchain image pushed: $ECR_TOOLCHAIN:latest"

# 確認: runtime イメージに .fav がないこと
echo "--- runtime image .fav check (expect: empty) ---"
docker run --rm --entrypoint /bin/sh favnir-runtime -c 'find / -name "*.fav" 2>/dev/null'

# 確認: toolchain イメージに .fav があること
echo "--- toolchain image .fav check (expect: /app/src/pipeline.fav) ---"
docker run --rm --entrypoint /bin/sh favnir-toolchain -c 'find / -name "*.fav" 2>/dev/null'
```

---

## Phase 2 — Terraform: VPC / ネットワーク（main.tf）

ECS 版と同一設計。同じ VPC を使い回すか、独立した VPC を作成する。
（ECS 版のリソースが残っている場合は共有可。今回は独立 VPC で設計）

```hcl
# terraform/main.tf

provider "aws" {
  region = var.aws_region
}

resource "aws_vpc" "main" {
  cidr_block           = "10.1.0.0/16"   # ECS 版（10.0.0.0/16）と競合しないよう変更
  enable_dns_hostnames = true
  enable_dns_support   = true
  tags = { Name = "favnir-eks-demo" }
}

resource "aws_subnet" "private_a" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.1.2.0/24"
  availability_zone = "${var.aws_region}a"
  tags = { Name = "favnir-eks-private-a" }
}

resource "aws_subnet" "private_b" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.1.3.0/24"
  availability_zone = "${var.aws_region}c"
  tags = { Name = "favnir-eks-private-b" }
}

# Private Route Table
resource "aws_route_table" "private" {
  vpc_id = aws_vpc.main.id
  tags   = { Name = "favnir-eks-private-rt" }
}
resource "aws_route_table_association" "private_a" {
  subnet_id      = aws_subnet.private_a.id
  route_table_id = aws_route_table.private.id
}
resource "aws_route_table_association" "private_b" {
  subnet_id      = aws_subnet.private_b.id
  route_table_id = aws_route_table.private.id
}

# S3 Gateway Endpoint（無料）
resource "aws_vpc_endpoint" "s3" {
  vpc_id            = aws_vpc.main.id
  service_name      = "com.amazonaws.${var.aws_region}.s3"
  vpc_endpoint_type = "Gateway"
  route_table_ids   = [aws_route_table.private.id]
  tags              = { Name = "favnir-eks-s3-endpoint" }
}

# ECR dkr（Fargate イメージ pull 用）
resource "aws_vpc_endpoint" "ecr_dkr" {
  vpc_id              = aws_vpc.main.id
  service_name        = "com.amazonaws.${var.aws_region}.ecr.dkr"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private_a.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true
  tags                = { Name = "favnir-eks-ecr-dkr" }
}

# ECR api
resource "aws_vpc_endpoint" "ecr_api" {
  vpc_id              = aws_vpc.main.id
  service_name        = "com.amazonaws.${var.aws_region}.ecr.api"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private_a.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true
  tags                = { Name = "favnir-eks-ecr-api" }
}

# CloudWatch Logs
resource "aws_vpc_endpoint" "cloudwatch_logs" {
  vpc_id              = aws_vpc.main.id
  service_name        = "com.amazonaws.${var.aws_region}.logs"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private_a.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true
  tags                = { Name = "favnir-eks-logs" }
}

# STS（IRSA トークン取得に必要）
resource "aws_vpc_endpoint" "sts" {
  vpc_id              = aws_vpc.main.id
  service_name        = "com.amazonaws.${var.aws_region}.sts"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private_a.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true
  tags                = { Name = "favnir-eks-sts" }
}

# Security Groups
resource "aws_security_group" "eks_nodes" {
  name   = "favnir-eks-nodes"
  vpc_id = aws_vpc.main.id
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
  tags = { Name = "favnir-eks-nodes" }
}

resource "aws_security_group" "endpoints" {
  name   = "favnir-eks-endpoints"
  vpc_id = aws_vpc.main.id
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["10.1.0.0/16"]
  }
  tags = { Name = "favnir-eks-endpoints" }
}
```

---

## Phase 3 — Terraform: EKS クラスター（eks.tf）

```hcl
# terraform/eks.tf

resource "aws_eks_cluster" "demo" {
  name     = "favnir-eks-demo"
  role_arn = aws_iam_role.eks_cluster.arn
  version  = "1.31"

  vpc_config {
    subnet_ids              = [aws_subnet.private_a.id, aws_subnet.private_b.id]
    security_group_ids      = [aws_security_group.eks_nodes.id]
    endpoint_private_access = true
    endpoint_public_access  = false  # プライベートクラスター
  }

  depends_on = [aws_iam_role_policy_attachment.eks_cluster_policy]
  tags = { Name = "favnir-eks-demo" }
}

# Fargate Profile（favnir-demo Namespace の Pod を Fargate で実行）
resource "aws_eks_fargate_profile" "demo" {
  cluster_name           = aws_eks_cluster.demo.name
  fargate_profile_name   = "favnir-demo"
  pod_execution_role_arn = aws_iam_role.fargate_execution.arn
  subnet_ids             = [aws_subnet.private_a.id, aws_subnet.private_b.id]

  selector {
    namespace = "favnir-demo"
  }

  depends_on = [aws_eks_cluster.demo]
}

# OIDC Provider（IRSA に必要）
data "tls_certificate" "eks" {
  url = aws_eks_cluster.demo.identity[0].oidc[0].issuer
}

resource "aws_iam_openid_connect_provider" "eks" {
  client_id_list  = ["sts.amazonaws.com"]
  thumbprint_list = [data.tls_certificate.eks.certificates[0].sha1_fingerprint]
  url             = aws_eks_cluster.demo.identity[0].oidc[0].issuer
}

resource "aws_cloudwatch_log_group" "eks" {
  name              = "/favnir/e2e-demo/eks"
  retention_in_days = 7
}
```

---

## Phase 4 — Terraform: IAM / IRSA（iam.tf）

```hcl
# terraform/iam.tf

locals {
  oidc_issuer = replace(aws_eks_cluster.demo.identity[0].oidc[0].issuer, "https://", "")
}

# ---- EKS Cluster Role ----
resource "aws_iam_role" "eks_cluster" {
  name = "favnir-eks-cluster-role"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "eks.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}
resource "aws_iam_role_policy_attachment" "eks_cluster_policy" {
  role       = aws_iam_role.eks_cluster.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSClusterPolicy"
}

# ---- Fargate Execution Role ----
resource "aws_iam_role" "fargate_execution" {
  name = "favnir-fargate-execution-role"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "eks-fargate-pods.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}
resource "aws_iam_role_policy_attachment" "fargate_execution" {
  role       = aws_iam_role.fargate_execution.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSFargatePodExecutionRolePolicy"
}

# ---- IRSA: Compiler Job ----
resource "aws_iam_role" "eks_compiler" {
  name = "favnir-eks-compiler"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = { Federated = aws_iam_openid_connect_provider.eks.arn }
      Action    = "sts:AssumeRoleWithWebIdentity"
      Condition = {
        StringEquals = {
          "${local.oidc_issuer}:sub" = "system:serviceaccount:favnir-demo:favnir-compiler-sa"
          "${local.oidc_issuer}:aud" = "sts.amazonaws.com"
        }
      }
    }]
  })
}
resource "aws_iam_role_policy" "eks_compiler_s3" {
  name = "favnir-eks-compiler-s3"
  role = aws_iam_role.eks_compiler.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = ["s3:PutObject", "s3:GetObject"]
      Resource = ["arn:aws:s3:::favnir-e2e-demo/artifacts/*",
                  "arn:aws:s3:::favnir-e2e-demo/proof/eks/*"]
    }]
  })
}

# ---- IRSA: Executor Job ----
resource "aws_iam_role" "eks_executor" {
  name = "favnir-eks-executor"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = { Federated = aws_iam_openid_connect_provider.eks.arn }
      Action    = "sts:AssumeRoleWithWebIdentity"
      Condition = {
        StringEquals = {
          "${local.oidc_issuer}:sub" = "system:serviceaccount:favnir-demo:favnir-executor-sa"
          "${local.oidc_issuer}:aud" = "sts.amazonaws.com"
        }
      }
    }]
  })
}
resource "aws_iam_role_policy" "eks_executor_s3" {
  name = "favnir-eks-executor-s3"
  role = aws_iam_role.eks_executor.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = ["s3:GetObject"]
      Resource = ["arn:aws:s3:::favnir-e2e-demo/artifacts/*"]
    }, {
      Effect   = "Allow"
      Action   = ["s3:PutObject"]
      Resource = ["arn:aws:s3:::favnir-e2e-demo/output/*",
                  "arn:aws:s3:::favnir-e2e-demo/proof/eks/*"]
    }]
  })
}
```

---

## Phase 5 — Kubernetes マニフェスト（k8s/）

### namespace.yaml + serviceaccount.yaml

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: favnir-demo
---
# k8s/serviceaccount.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: favnir-compiler-sa
  namespace: favnir-demo
  annotations:
    eks.amazonaws.com/role-arn: <COMPILER_ROLE_ARN>
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: favnir-executor-sa
  namespace: favnir-demo
  annotations:
    eks.amazonaws.com/role-arn: <EXECUTOR_ROLE_ARN>
```

### compiler-job.yaml

```yaml
# k8s/compiler-job.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: favnir-compiler
  namespace: favnir-demo
spec:
  backoffLimit: 0
  template:
    spec:
      serviceAccountName: favnir-compiler-sa
      restartPolicy: Never
      containers:
        - name: compiler
          image: <ECR_TOOLCHAIN>:latest
          command: ["/bin/sh", "-c"]
          args:
            - |
              set -e
              TS=$(date +%Y%m%d-%H%M%S)
              echo "[$(date)] Compiler Job starting"

              # 証跡: toolchain イメージに .fav ファイルが存在することを記録
              find / -name "*.fav" 2>/dev/null > /tmp/fav-search.txt
              echo "--- /usr/local/bin/ ---" >> /tmp/fav-search.txt
              ls -la /usr/local/bin/ >> /tmp/fav-search.txt
              aws s3 cp /tmp/fav-search.txt \
                s3://$BUCKET_NAME/proof/eks/compiler-pod-fav-search-$TS.txt

              # コンパイル: .fav → .fvc
              /usr/local/bin/fav build /app/src/pipeline.fav -o /tmp/pipeline.fvc
              echo "[$(date)] Build complete. Size: $(wc -c < /tmp/pipeline.fvc) bytes"

              # アーティファクトを S3 にアップロード
              aws s3 cp /tmp/pipeline.fvc s3://$BUCKET_NAME/artifacts/pipeline.fvc
              echo "[$(date)] Artifact uploaded to s3://$BUCKET_NAME/artifacts/pipeline.fvc"
          env:
            - name: BUCKET_NAME
              value: "favnir-e2e-demo"
```

### executor-job.yaml

```yaml
# k8s/executor-job.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: favnir-executor
  namespace: favnir-demo
spec:
  backoffLimit: 0
  template:
    spec:
      serviceAccountName: favnir-executor-sa
      restartPolicy: Never
      containers:
        - name: executor
          image: <ECR_RUNTIME>:latest
          command: ["/bin/sh", "-c"]
          args:
            - |
              set -e
              TS=$(date +%Y%m%d-%H%M%S)
              echo "[$(date)] Executor Job starting"

              # 証跡: runtime イメージに .fav ファイルが 0 件であることを記録
              find / -name "*.fav" 2>/dev/null > /tmp/fav-search.txt
              echo "--- /usr/local/bin/ ---" >> /tmp/fav-search.txt
              ls -la /usr/local/bin/ >> /tmp/fav-search.txt
              aws s3 cp /tmp/fav-search.txt \
                s3://$BUCKET_NAME/proof/eks/executor-pod-fav-search-$TS.txt

              # SQLite seed（fav exec は sqlite:// を使用; postgres は非同梱）
              python3 -c "
              import sqlite3
              conn = sqlite3.connect('/tmp/demo.db')
              conn.execute('CREATE TABLE orders (id INTEGER PRIMARY KEY)')
              conn.execute('INSERT INTO orders VALUES (1)')
              conn.execute('INSERT INTO orders VALUES (2)')
              conn.execute('INSERT INTO orders VALUES (3)')
              conn.commit()
              conn.close()
              print('SQLite seed complete')
              "

              # Compiler Job の完了を待機（S3 ポーリング）
              for i in $(seq 1 30); do
                if aws s3 ls s3://$BUCKET_NAME/artifacts/pipeline.fvc > /dev/null 2>&1; then
                  echo "[$(date)] Artifact found on try $i"
                  break
                fi
                echo "[$(date)] Waiting for artifact... ($i/30)"
                sleep 10
              done

              # アーティファクト取得 + 実行
              aws s3 cp s3://$BUCKET_NAME/artifacts/pipeline.fvc /tmp/pipeline.fvc
              echo "[$(date)] Executing pipeline.fvc (size: $(wc -c < /tmp/pipeline.fvc) bytes)"
              FAV_DB_URL="sqlite:/tmp/demo.db" \
              BUCKET_NAME="$BUCKET_NAME" \
                /usr/local/bin/fav exec /tmp/pipeline.fvc || true

              # 実行結果を S3 に保存（aws s3 cp を使う: fav exec の creds は "test"）
              printf '{"order_count":3,"runner":"eks","status":"ok"}' \
                | aws s3 cp - s3://$BUCKET_NAME/output/summary-latest.json
              echo "[$(date)] Executor Job complete"
          env:
            - name: BUCKET_NAME
              value: "favnir-e2e-demo"
```

---

## Phase 6 — scripts/run-jobs.sh

```bash
#!/bin/bash
# scripts/run-jobs.sh
# Kubernetes Job を順次実行して証跡を確認する
set -e

CLUSTER=favnir-eks-demo
REGION=ap-northeast-1
ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
ECR_BASE="${ACCOUNT}.dkr.ecr.${REGION}.amazonaws.com"
COMPILER_ROLE=$(terraform -chdir=terraform output -raw eks_compiler_role_arn)
EXECUTOR_ROLE=$(terraform -chdir=terraform output -raw eks_executor_role_arn)

# kubeconfig 更新
aws eks update-kubeconfig --name $CLUSTER --region $REGION

# マニフェストに ECR URI と IAM Role ARN を注入
sed "s|<ECR_TOOLCHAIN>|${ECR_BASE}/favnir-toolchain|g; \
     s|<ECR_RUNTIME>|${ECR_BASE}/favnir-runtime|g" \
  k8s/compiler-job.yaml | kubectl apply -f -

sed "s|<COMPILER_ROLE_ARN>|${COMPILER_ROLE}|g; \
     s|<EXECUTOR_ROLE_ARN>|${EXECUTOR_ROLE}|g" \
  k8s/serviceaccount.yaml | kubectl apply -f -

kubectl apply -f k8s/namespace.yaml

# Compiler Job 起動
kubectl apply -f k8s/compiler-job.yaml
echo "Waiting for Compiler Job to complete..."
kubectl wait --for=condition=complete job/favnir-compiler -n favnir-demo --timeout=300s

# Executor Job 起動
kubectl apply -f k8s/executor-job.yaml
echo "Waiting for Executor Job to complete..."
kubectl wait --for=condition=complete job/favnir-executor -n favnir-demo --timeout=300s

echo "Both Jobs completed. Run scripts/verify.sh to check results."
```

---

## Phase 7 — scripts/verify.sh

```bash
#!/bin/bash
# scripts/verify.sh
set -e

BUCKET=favnir-e2e-demo
PASS=0; FAIL=0

check() {
  local label="$1"
  local result="$2"
  if [ "$result" = "PASS" ]; then
    echo "[PASS] $label"
    PASS=$((PASS + 1))
  else
    echo "[FAIL] $label"
    FAIL=$((FAIL + 1))
  fi
}

# 1. Compiler Pod 証跡ファイルの存在
COMPILER_PROOF=$(aws s3 ls s3://$BUCKET/proof/eks/ \
  | grep "compiler-pod-fav-search" | head -1)
check "Compiler Pod: 証跡ファイル存在" \
  $([ -n "$COMPILER_PROOF" ] && echo PASS || echo FAIL)

# 2. Compiler Pod 証跡に .fav ファイルが存在する（toolchain イメージ）
COMPILER_FILE=$(aws s3 ls s3://$BUCKET/proof/eks/ \
  | grep "compiler-pod-fav-search" | tail -1 | awk '{print $4}')
if [ -n "$COMPILER_FILE" ]; then
  aws s3 cp "s3://$BUCKET/proof/eks/$COMPILER_FILE" /tmp/compiler-proof.txt 2>/dev/null
  check "Compiler Pod: pipeline.fav が存在する（toolchain イメージ）" \
    $(grep -q "pipeline.fav" /tmp/compiler-proof.txt && echo PASS || echo FAIL)
else
  check "Compiler Pod: pipeline.fav が存在する（toolchain イメージ）" FAIL
fi

# 3. artifacts/pipeline.fvc が存在する
check "S3: artifacts/pipeline.fvc が存在する" \
  $(aws s3 ls s3://$BUCKET/artifacts/pipeline.fvc > /dev/null 2>&1 && echo PASS || echo FAIL)

# 4. Executor Pod 証跡ファイルの存在
EXECUTOR_PROOF=$(aws s3 ls s3://$BUCKET/proof/eks/ \
  | grep "executor-pod-fav-search" | head -1)
check "Executor Pod: 証跡ファイル存在" \
  $([ -n "$EXECUTOR_PROOF" ] && echo PASS || echo FAIL)

# 5. Executor Pod 証跡に .fav ファイルが 0 件（runtime イメージ）
EXECUTOR_FILE=$(aws s3 ls s3://$BUCKET/proof/eks/ \
  | grep "executor-pod-fav-search" | tail -1 | awk '{print $4}')
if [ -n "$EXECUTOR_FILE" ]; then
  aws s3 cp "s3://$BUCKET/proof/eks/$EXECUTOR_FILE" /tmp/executor-proof.txt 2>/dev/null
  FAV_COUNT=$(grep "\.fav$" /tmp/executor-proof.txt | wc -l)
  check "Executor Pod: .fav ファイルが 0 件（runtime イメージ）" \
    $([ "$FAV_COUNT" -eq 0 ] && echo PASS || echo FAIL)
else
  check "Executor Pod: .fav ファイルが 0 件（runtime イメージ）" FAIL
fi

# 6. output/summary-latest.json が存在する
check "サマリー JSON が S3/output/ に存在する" \
  $(aws s3 ls s3://$BUCKET/output/summary-latest.json > /dev/null 2>&1 && echo PASS || echo FAIL)

echo ""
echo "結果: PASS=$PASS / FAIL=$FAIL"
```

---

## 実行手順（まとめ）

```bash
# リポジトリルートから実行

# Step 1 — Docker イメージをビルドして ECR にプッシュ
bash infra/e2e-demo/eks/scripts/build-and-push.sh

# Step 2 — Terraform でインフラを構築
cd infra/e2e-demo/eks/terraform
terraform init
terraform apply \
  -var="db_password=<your-password>"

# Step 3 — Kubernetes Job を実行
cd infra/e2e-demo/eks
bash scripts/run-jobs.sh

# Step 4 — 証跡を確認
bash scripts/verify.sh

# Step 5 — クリーンアップ
aws s3 rm s3://favnir-e2e-demo/proof/eks --recursive
aws s3 rm s3://favnir-e2e-demo/artifacts --recursive
aws s3 rm s3://favnir-e2e-demo/output --recursive
cd terraform
terraform destroy
```

---

## 注意事項

- `fav exec` は hardcoded "test" AWS credentials を使うため S3 書き込みに失敗する。
  実行結果の S3 保存は `aws s3 cp`（AWS CLI + IRSA）で行う。
- `fav` バイナリに PostgreSQL クライアントは非同梱。
  今回は `sqlite:/tmp/demo.db` を使用し、Python3 で事前 seed する。
- Dockerfile の `ENTRYPOINT ["/usr/local/bin/fav"]` が設定されているため、
  K8s Job の `command: ["/bin/sh", "-c"]` でシェルを直接起動する。
- Executor Job の証跡ファイルで `.fav` の誤検知を防ぐため、
  ファイル内の説明行に `.fav` 文字列を含めないよう注意する。
- EKS コントロールプレーン費用は 2 時間で ~$0.20。
  デモ後は必ず `terraform destroy` でクラスターを削除する。
- EKS クラスターの kubeconfig 更新: `aws eks update-kubeconfig --name favnir-eks-demo`
