# Favnir E2E Demo — ECS 版 実装計画

Date: 2026-05-30

## Terraform ディレクトリ構成

```
infra/e2e-demo/ecs/
├── spec.md                    アーキテクチャ仕様（本書と対）
├── plan.md                    実装計画（本書）
├── tasks.md                   タスクリスト
├── terraform/
│   ├── main.tf                VPC / Subnet / SG / VPC Endpoints
│   ├── compute.tf             Machine A (Public EC2) / Machine B (Private EC2)
│   ├── ecs.tf                 ECS Cluster / Task Definition / IAM
│   ├── database.tf            Aurora Serverless v2
│   ├── storage.tf             S3 バケット + バケットポリシー
│   ├── iam.tf                 EC2 Instance Profile / ECS Task Role
│   ├── monitoring.tf          CloudWatch Logs グループ
│   └── variables.tf           変数定義
├── docker/
│   └── runtime/
│       └── Dockerfile         favnir/runtime イメージ（fav バイナリのみ）
├── src/
│   ├── pipeline.fav           Machine B 実行用パイプライン（デモ用）
│   └── etl.fav                ECS ETL パイプライン（メイン）
└── scripts/
    ├── machine-a-userdata.sh  Machine A 起動スクリプト
    ├── machine-b-userdata.sh  Machine B 起動スクリプト
    └── build-and-push.sh      Docker イメージビルド + ECR push
```

---

## Phase 1 — 事前準備

### 1-1. favnir/runtime Docker イメージ（multi-stage build）

Linux バイナリのビルドは Docker multi-stage で完結させる。
WSL や EC2 での事前ビルドは不要。

```dockerfile
# docker/runtime/Dockerfile

# ---- Stage 1: ビルダー ----
# rust:slim-bookworm は debian-based (glibc) のため
# duckdb (bundled) / wasmtime (cranelift) の C++ native 依存と互換性あり
# Alpine (musl) は glibc 依存ライブラリとの相性問題があるため使用しない
FROM rust:slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake g++ pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY fav/ .
RUN cargo build --release

# ---- Stage 2: ランタイム ----
# .fav ファイルは一切含まない — これが証明の核心
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates awscli \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/fav /usr/local/bin/fav
ENTRYPOINT ["/usr/local/bin/fav"]
```

ビルドコンテキストはリポジトリルートから実行する（`fav/` ディレクトリを COPY するため）：

```bash
# リポジトリルートから実行
docker build -f infra/e2e-demo/ecs/docker/runtime/Dockerfile -t favnir-runtime .
```

### 1-2. ECR リポジトリ作成 + プッシュ + EC2 用バイナリを S3 に配置

Docker multi-stage でビルドしたバイナリをイメージから抽出し、
Machine A / Machine B が起動時に S3 から取得できるよう配置する。

```bash
# scripts/build-and-push.sh
set -e
AWS_REGION=ap-northeast-1
BUCKET_NAME=favnir-e2e-demo
AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
ECR_URI="${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com/favnir-runtime"

# ECR リポジトリ作成（既存の場合は無視）
aws ecr create-repository --repository-name favnir-runtime --region $AWS_REGION 2>/dev/null || true

# Docker イメージをビルド（multi-stage: Rust ビルド → debian-slim ランタイム）
docker build -f infra/e2e-demo/ecs/docker/runtime/Dockerfile -t favnir-runtime .

# イメージから fav バイナリを抽出 → S3 に配置（EC2 インスタンスが起動時に取得）
docker create --name tmp-fav favnir-runtime
docker cp tmp-fav:/usr/local/bin/fav /tmp/fav-linux
docker rm tmp-fav
aws s3 cp /tmp/fav-linux s3://$BUCKET_NAME/bootstrap/fav
echo "Binary uploaded: s3://$BUCKET_NAME/bootstrap/fav"

# ECR にプッシュ
aws ecr get-login-password --region $AWS_REGION \
  | docker login --username AWS --password-stdin "${AWS_ACCOUNT}.dkr.ecr.${AWS_REGION}.amazonaws.com"
docker tag favnir-runtime:latest $ECR_URI:latest
docker push $ECR_URI:latest
echo "Image pushed: $ECR_URI:latest"
```

**ポイント:** バイナリの単一ソース — Docker multi-stage の成果物を ECS イメージと EC2 の両方で共用。
別途 Linux ビルド環境（WSL 等）は不要。

### 1-3. イメージ確認（.fav ファイルが存在しないことの事前確認）

```bash
# .fav ファイルが存在しないことをローカルで確認
docker run --rm favnir-runtime find / -name "*.fav" 2>/dev/null
# → 出力なし（0件）であること

# fav バイナリが正常に動くことを確認
docker run --rm favnir-runtime fav --version
```

---

## Phase 2 — Terraform: VPC / ネットワーク（main.tf）

```hcl
# terraform/main.tf

provider "aws" {
  region = var.aws_region
}

resource "aws_vpc" "main" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true
  tags = { Name = "favnir-e2e-demo" }
}

# Public Subnet (Machine A)
resource "aws_subnet" "public" {
  vpc_id                  = aws_vpc.main.id
  cidr_block              = "10.0.1.0/24"
  map_public_ip_on_launch = true
  availability_zone       = "${var.aws_region}a"
  tags = { Name = "favnir-public" }
}

# Private Subnet (Machine B + ECS + RDS)
resource "aws_subnet" "private" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.0.2.0/24"
  availability_zone = "${var.aws_region}a"
  tags = { Name = "favnir-private" }
}

# Internet Gateway (Machine A 用)
resource "aws_internet_gateway" "igw" {
  vpc_id = aws_vpc.main.id
  tags   = { Name = "favnir-igw" }
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.igw.id
  }
}
resource "aws_route_table_association" "public" {
  subnet_id      = aws_subnet.public.id
  route_table_id = aws_route_table.public.id
}

# --- VPC Endpoints (NAT Gateway 不使用) ---

# S3 Gateway Endpoint (無料)
resource "aws_vpc_endpoint" "s3" {
  vpc_id          = aws_vpc.main.id
  service_name    = "com.amazonaws.${var.aws_region}.s3"
  vpc_endpoint_type = "Gateway"
  route_table_ids = [aws_route_table.private.id]
}

# CloudWatch Logs Interface Endpoint
resource "aws_vpc_endpoint" "cloudwatch_logs" {
  vpc_id              = aws_vpc.main.id
  service_name        = "com.amazonaws.${var.aws_region}.logs"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true
}

# SSM Interface Endpoint
resource "aws_vpc_endpoint" "ssm" {
  vpc_id              = aws_vpc.main.id
  service_name        = "com.amazonaws.${var.aws_region}.ssm"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true
}

# ECR dkr Endpoint (ECS イメージ pull 用)
resource "aws_vpc_endpoint" "ecr_dkr" {
  vpc_id              = aws_vpc.main.id
  service_name        = "com.amazonaws.${var.aws_region}.ecr.dkr"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true
}

# ECR api Endpoint
resource "aws_vpc_endpoint" "ecr_api" {
  vpc_id              = aws_vpc.main.id
  service_name        = "com.amazonaws.${var.aws_region}.ecr.api"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true
}

# Private Route Table (VPC Endpoint 経由)
resource "aws_route_table" "private" {
  vpc_id = aws_vpc.main.id
}
resource "aws_route_table_association" "private" {
  subnet_id      = aws_subnet.private.id
  route_table_id = aws_route_table.private.id
}

# Security Groups
resource "aws_security_group" "machine_a" {
  name   = "favnir-machine-a"
  vpc_id = aws_vpc.main.id
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = [var.my_ip_cidr]   # 開発者 IP のみ SSH 許可
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "machine_b" {
  name   = "favnir-machine-b"
  vpc_id = aws_vpc.main.id
  # Inbound: SSM のみ（SSH 不要）
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "ecs" {
  name   = "favnir-ecs"
  vpc_id = aws_vpc.main.id
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "rds" {
  name   = "favnir-rds"
  vpc_id = aws_vpc.main.id
  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.machine_b.id, aws_security_group.ecs.id]
  }
}

resource "aws_security_group" "endpoints" {
  name   = "favnir-endpoints"
  vpc_id = aws_vpc.main.id
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/16"]
  }
}
```

---

## Phase 3 — Terraform: EC2（compute.tf）

```hcl
# terraform/compute.tf

# ---- Machine A: Favnir 処理系（Public EC2）----

data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"]  # Canonical
  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-noble-24.04-amd64-server-*"]
  }
}

resource "aws_instance" "machine_a" {
  ami                         = data.aws_ami.ubuntu.id
  instance_type               = "t3.micro"
  subnet_id                   = aws_subnet.public.id
  vpc_security_group_ids      = [aws_security_group.machine_a.id]
  iam_instance_profile        = aws_iam_instance_profile.ec2.name
  associate_public_ip_address = true

  user_data = templatefile("${path.module}/../scripts/machine-a-userdata.sh", {
    bucket_name = aws_s3_bucket.demo.bucket
    etl_src     = file("${path.module}/../src/etl.fav")
    pipeline_src = file("${path.module}/../src/pipeline.fav")
  })

  tags = { Name = "favnir-machine-a" }
}

# ---- Machine B: Rust VM のみ（Private EC2）----

resource "aws_instance" "machine_b" {
  ami                    = data.aws_ami.ubuntu.id
  instance_type          = "t3.micro"
  subnet_id              = aws_subnet.private.id
  vpc_security_group_ids = [aws_security_group.machine_b.id]
  iam_instance_profile   = aws_iam_instance_profile.ec2.name

  user_data = templatefile("${path.module}/../scripts/machine-b-userdata.sh", {
    bucket_name = aws_s3_bucket.demo.bucket
    db_url      = "postgres://${var.db_user}:${var.db_password}@${aws_rds_cluster.demo.endpoint}/demo"
  })

  tags = { Name = "favnir-machine-b" }
}
```

---

## Phase 4 — Terraform: ECS（ecs.tf）

```hcl
# terraform/ecs.tf

resource "aws_ecs_cluster" "demo" {
  name = "favnir-e2e-demo"
}

resource "aws_ecs_task_definition" "etl" {
  family                   = "favnir-etl"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "256"
  memory                   = "512"
  execution_role_arn       = aws_iam_role.ecs_execution.arn
  task_role_arn            = aws_iam_role.ecs_task.arn

  container_definitions = jsonencode([
    # Init: 証跡収集（.fav ファイルが0件であることを記録）
    {
      name      = "proof-collector"
      image     = "${var.ecr_uri}:latest"
      essential = false
      command   = [
        "/bin/sh", "-c",
        join("\n", [
          "TS=$(date +%Y%m%d-%H%M%S)",
          "find / -name '*.fav' 2>/dev/null > /tmp/fav-search.txt",
          "echo '--- /usr/local/bin/ ---' >> /tmp/fav-search.txt",
          "ls -la /usr/local/bin/ >> /tmp/fav-search.txt",
          "aws s3 cp /tmp/fav-search.txt s3://${var.bucket_name}/proof/ecs/fav-search-$TS.txt",
        ])
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = "/favnir/e2e-demo/ecs"
          "awslogs-region"        = var.aws_region
          "awslogs-stream-prefix" = "proof"
        }
      }
    },
    # Main: ETL 実行
    {
      name      = "etl-runner"
      image     = "${var.ecr_uri}:latest"
      essential = true
      dependsOn = [{ containerName = "proof-collector", condition = "COMPLETE" }]
      command   = [
        "/bin/sh", "-c",
        join("\n", [
          "aws s3 cp s3://${var.bucket_name}/artifacts/etl.fvc /tmp/etl.fvc",
          "FAV_DB_URL=$DB_URL fav exec /tmp/etl.fvc",
        ])
      ]
      environment = [
        { name = "BUCKET_NAME", value = var.bucket_name }
      ]
      secrets = [
        { name = "DB_URL", valueFrom = aws_secretsmanager_secret.db_url.arn }
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = "/favnir/e2e-demo/ecs"
          "awslogs-region"        = var.aws_region
          "awslogs-stream-prefix" = "etl"
        }
      }
    }
  ])
}

resource "aws_cloudwatch_log_group" "ecs" {
  name              = "/favnir/e2e-demo/ecs"
  retention_in_days = 7
}
```

---

## Phase 5 — Machine A ユーザーデータスクリプト

```bash
#!/bin/bash
# scripts/machine-a-userdata.sh
set -e
exec > >(tee -a /var/log/favnir-machine-a.log) 2>&1

BUCKET="${bucket_name}"
echo "[$(date)] Machine A starting"

# fav バイナリをインストール
aws s3 cp s3://$BUCKET/bootstrap/fav /usr/local/bin/fav
chmod +x /usr/local/bin/fav

# Favnir ソースを配置
mkdir -p /app/src /app/self
cat > /app/src/etl.fav << 'FAVSRC'
${etl_src}
FAVSRC

cat > /app/src/pipeline.fav << 'FAVSRC'
${pipeline_src}
FAVSRC

# 証跡: Machine A のファイル一覧を S3 に保存
TS=$(date +%Y%m%d-%H%M%S)
find /app -type f | sort > /tmp/machine-a-filelist.txt
echo "--- /usr/local/bin/ ---" >> /tmp/machine-a-filelist.txt
ls -la /usr/local/bin/ >> /tmp/machine-a-filelist.txt
aws s3 cp /tmp/machine-a-filelist.txt s3://$BUCKET/proof/machine-a/filelist-$TS.txt
echo "[$(date)] Proof uploaded: s3://$BUCKET/proof/machine-a/filelist-$TS.txt"

# ビルド: .fav → .fvc
fav build /app/src/etl.fav      -o /tmp/etl.fvc
fav build /app/src/pipeline.fav -o /tmp/pipeline.fvc

# アーティファクトを S3 にアップロード
aws s3 cp /tmp/etl.fvc      s3://$BUCKET/artifacts/etl.fvc
aws s3 cp /tmp/pipeline.fvc s3://$BUCKET/artifacts/pipeline.fvc

echo "[$(date)] Machine A done — artifacts uploaded"
```

---

## Phase 6 — Machine B ユーザーデータスクリプト

```bash
#!/bin/bash
# scripts/machine-b-userdata.sh
set -e
exec > >(tee -a /var/log/favnir-machine-b.log) 2>&1

BUCKET="${bucket_name}"
DB_URL="${db_url}"
echo "[$(date)] Machine B starting"

# fav バイナリをインストール（ソースコードなし）
aws s3 cp s3://$BUCKET/bootstrap/fav /usr/local/bin/fav
chmod +x /usr/local/bin/fav

# 証跡: .fav ファイルが一切存在しないことを記録
TS=$(date +%Y%m%d-%H%M%S)
echo "=== .fav file search (expect: 0 results) ===" > /tmp/machine-b-proof.txt
find / -name "*.fav" 2>/dev/null >> /tmp/machine-b-proof.txt
echo "=== /usr/local/bin/ ===" >> /tmp/machine-b-proof.txt
ls -la /usr/local/bin/ >> /tmp/machine-b-proof.txt
aws s3 cp /tmp/machine-b-proof.txt s3://$BUCKET/proof/machine-b/fav-search-$TS.txt
echo "[$(date)] Proof uploaded: s3://$BUCKET/proof/machine-b/fav-search-$TS.txt"

# Machine A のビルド完了を待機（pipeline.fvc が S3 に存在するまでポーリング）
for i in $(seq 1 30); do
  if aws s3 ls s3://$BUCKET/artifacts/pipeline.fvc > /dev/null 2>&1; then
    echo "[$(date)] Artifact found"
    break
  fi
  echo "[$(date)] Waiting for artifact... ($i/30)"
  sleep 10
done

# アーティファクトを取得して実行
aws s3 cp s3://$BUCKET/artifacts/pipeline.fvc /tmp/pipeline.fvc
FAV_DB_URL="$DB_URL" fav exec /tmp/pipeline.fvc

echo "[$(date)] Machine B pipeline complete"

# ログを S3 に保存
aws s3 cp /var/log/favnir-machine-b.log \
  s3://$BUCKET/logs/machine-b-$(date +%Y%m%d-%H%M%S).log

# 自己 stop
INSTANCE_ID=$(curl -s http://169.254.169.254/latest/meta-data/instance-id)
aws ec2 stop-instances --instance-ids $INSTANCE_ID
```

---

## Phase 7 — ETL Favnir ソース（src/etl.fav）

```favnir
import rune "postgres"
import rune "aws"

type Order   = { id: Int  customer: String  amount: Float  created_at: String }
type Summary = { customer: String  total: Float  count: Int }

stage ExtractOrders: String -> List<Order> !Db = |conn_str| {
  bind conn <- postgres.connect(conn_str)
  postgres.query<Order>(conn, "SELECT id, customer, amount, created_at FROM orders")
}

stage Summarize: List<Order> -> List<Summary> = |orders| {
  List.map(
    List.group_by(orders, |o| o.customer),
    |group| Summary {
      customer: group.key
      total:    List.sum(List.map(group.items, |o| o.amount))
      count:    List.length(group.items)
    }
  )
}

stage SaveSummary: List<Summary> -> Unit !AWS = |summaries| {
  bind ts <- aws.timestamp()
  aws.s3_put_json($"output/summary-{ts}.json", summaries)
}

seq EtlPipeline = ExtractOrders |> Summarize |> SaveSummary
```

---

## 実行手順（まとめ）

```bash
# 1. Linux バイナリをビルド（WSL）
cd /mnt/c/Users/yoshi/favnir/fav && cargo build --release
cp target/release/fav ../infra/e2e-demo/ecs/docker/runtime/fav-linux-x86_64

# 2. Docker イメージをビルド・ECR にプッシュ
cd infra/e2e-demo/ecs && bash scripts/build-and-push.sh

# 3. Terraform で全リソースを構築
cd infra/e2e-demo/ecs/terraform
terraform init
terraform apply -var="my_ip_cidr=$(curl -s ifconfig.me)/32"

# 4. 完了確認
aws s3 ls s3://favnir-e2e-demo/proof/ --recursive   # 証跡ファイル
aws s3 ls s3://favnir-e2e-demo/output/ --recursive  # ETL 出力

# 5. デモ後にクリーンアップ
terraform destroy
```

---

## 注意事項

- `fav exec` は `FAV_DB_URL` 環境変数で DB 接続情報を受け取る
- ECS Secrets Manager 統合により DB パスワードはコンテナ定義に平文で書かない
- Machine B・ECS は Private Subnet で Internet アクセスなし（VPC Endpoint 経由のみ）
- Aurora Serverless v2 の `min_capacity = 0.5` 設定でデモ後のコストを最小化
- デモ後は必ず `terraform destroy` でクラスターを削除
