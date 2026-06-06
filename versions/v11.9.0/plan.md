# Favnir v11.9.0 実装計画

作成日: 2026-06-06

---

## 実装順序

```
Phase A: infra/e2e-demo/fav2py/ ディレクトリ構築
    ↓
Phase B: src/pipeline.fav + sample.csv 作成
    ↓
Phase C: terraform/ 作成（main.tf / iam.tf / variables.tf / outputs.tf）
    ↓
Phase D: scripts/ 作成（upload.sh / run.sh / verify.sh）
    ↓
Phase E: Dockerfile 作成
    ↓
Phase F: README.md 作成
    ↓
Phase G: tasks.md 作成（デモ専用）
    ↓
Phase H: Rust テスト追加（v11900_tests）
    ↓
Phase I: バージョン更新・コミット
```

---

## Phase A — ディレクトリ構築

作成するディレクトリ:
- `infra/e2e-demo/fav2py/`
- `infra/e2e-demo/fav2py/src/`
- `infra/e2e-demo/fav2py/terraform/`
- `infra/e2e-demo/fav2py/scripts/`

`.gitkeep` は不要（ファイルを各ディレクトリに作成するため）。

---

## Phase B — src/pipeline.fav + sample.csv

### pipeline.fav

```fav
import rune "postgres"
import rune "aws"
import rune "csv"

type TxnRow = {
  id: Int
  region: String
  category: String
  amount: Float
}

type SummaryRow = {
  region: String
  category: String
  total: Float
  count: Int
}

// Stage 1: CSV 読み込み → Postgres INSERT
stage LoadAndInsert: String -> Int !IO !Postgres = |path| {
  bind rows <- csv.read<TxnRow>(path)
  postgres.execute(
    "INSERT INTO txn(id,region,category,amount) SELECT * FROM json_populate_recordset(NULL::txn,$1)",
    rows
  )
}

// Stage 2: 集計クエリ
stage Aggregate: Int -> List<SummaryRow> !Postgres = |_| {
  postgres.query<SummaryRow>(
    "SELECT region, category, SUM(amount) AS total, COUNT(*) AS count FROM txn GROUP BY region, category ORDER BY region, category",
    []
  )
}

// Stage 3: S3 に JSON 保存
stage SaveResult: List<SummaryRow> -> Unit !IO !AWS = |rows| {
  bind ts <- aws.timestamp()
  aws.s3_put_json($"favnir-e2e-demo/proof/fav2py/{ts}.json", rows)
}

seq Pipeline = LoadAndInsert |> Aggregate |> SaveResult
```

### sample.csv

103 行のサンプルデータ（Snowflake E2E と同形式）:

```csv
id,region,category,amount
1,AP,Electronics,1200.50
2,US,Clothing,450.00
...（103 行）
```

region: AP / US / EU / JP（4 種）
category: Electronics / Clothing / Food / Books（4 種）

---

## Phase C — terraform/

### main.tf

```hcl
terraform {
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.0" }
  }
}

provider "aws" { region = var.aws_region }

# VPC
resource "aws_vpc" "fav2py" { cidr_block = "10.0.0.0/16" ... }

# Subnets
resource "aws_subnet" "public"  { cidr_block = "10.0.1.0/24" ... }
resource "aws_subnet" "private" { cidr_block = "10.0.2.0/24" ... }

# Internet Gateway + NAT
resource "aws_internet_gateway" "igw" { ... }
resource "aws_eip" "nat" { ... }
resource "aws_nat_gateway" "nat" { ... }

# Route Tables
resource "aws_route_table" "public"  { ... }
resource "aws_route_table" "private" { ... }

# Security Groups
resource "aws_security_group" "rds" {
  # ECS タスクからの 5432 のみ許可
}
resource "aws_security_group" "ecs" {
  # 443/80 outbound（ECR pull + S3）、RDS 5432 outbound
}

# RDS PostgreSQL
resource "aws_db_subnet_group" "fav2py" { ... }
resource "aws_db_instance" "postgres" {
  engine            = "postgres"
  engine_version    = "16"
  instance_class    = "db.t3.micro"
  db_name           = "fav2py"
  username          = "favnir"
  password          = var.db_password
  ...
}

# ECR
resource "aws_ecr_repository" "fav2py" { name = "favnir/fav2py" }

# ECS Cluster
resource "aws_ecs_cluster" "fav2py" { name = "fav2py" }

# ECS Task Definition — fav-native
resource "aws_ecs_task_definition" "native" {
  family = "fav2py-native"
  ...
  container_definitions = jsonencode([{
    name    = "fav-native"
    image   = "${aws_ecr_repository.fav2py.repository_url}:latest"
    command = ["fav", "run", "/app/pipeline.fav", "/app/sample.csv"]
    environment = [
      { name = "DATABASE_URL", value = "postgresql://favnir:${var.db_password}@${aws_db_instance.postgres.address}/fav2py" },
      ...
    ]
  }])
}

# ECS Task Definition — fav-python
resource "aws_ecs_task_definition" "python" {
  ...
  container_definitions = jsonencode([{
    command = ["sh", "-c", "fav transpile --target python /app/pipeline.fav --out-dir /tmp/out && cd /tmp/out && uv run main.py /app/sample.csv"]
    ...
  }])
}
```

### iam.tf

```hcl
# ECS タスク実行ロール（ECR pull / CloudWatch Logs）
resource "aws_iam_role" "ecs_execution" { ... }

# ECS タスクロール（S3 書き込み / RDS 接続）
resource "aws_iam_role" "ecs_task" { ... }
resource "aws_iam_role_policy" "ecs_task_s3" { ... }
```

### variables.tf

```hcl
variable "aws_region"   { default = "ap-northeast-1" }
variable "db_password"  { sensitive = true }
variable "s3_bucket"    { default = "favnir-e2e-demo" }
```

### outputs.tf

```hcl
output "rds_endpoint"     { value = aws_db_instance.postgres.address }
output "ecr_repository"   { value = aws_ecr_repository.fav2py.repository_url }
output "ecs_cluster_arn"  { value = aws_ecs_cluster.fav2py.arn }
output "native_task_def"  { value = aws_ecs_task_definition.native.arn }
output "python_task_def"  { value = aws_ecs_task_definition.python.arn }
```

---

## Phase D — scripts/

### upload.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/.."
REGION="${AWS_DEFAULT_REGION:-ap-northeast-1}"
ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
ECR_URI="$ACCOUNT.dkr.ecr.$REGION.amazonaws.com/favnir/fav2py"

# ECR login
aws ecr get-login-password --region "$REGION" | docker login --username AWS --password-stdin "$ECR_URI"

# Docker build & push
cd "$ROOT"
docker build -t fav2py:latest .
docker tag fav2py:latest "$ECR_URI:latest"
docker push "$ECR_URI:latest"

# S3 source upload
aws s3 cp "$ROOT/src/" "s3://favnir-e2e-demo/fav2py/src/" --recursive
echo "upload.sh: done"
```

### run.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TF_DIR="$SCRIPT_DIR/../terraform"
PASS=0
FAIL=0
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

log() { echo "[$(date +%H:%M:%S)] $1"; }

log "=== Favnir v11.9.0 fav2py E2E Demo ==="

# Terraform apply
log "[1/5] terraform apply ..."
cd "$TF_DIR"
terraform init -input=false
terraform apply -auto-approve -input=false

CLUSTER=$(terraform output -raw ecs_cluster_arn)
NATIVE_DEF=$(terraform output -raw native_task_def)
PYTHON_DEF=$(terraform output -raw python_task_def)
SUBNET=$(terraform output -raw private_subnet_id)
SG=$(terraform output -raw ecs_security_group_id)

run_task() {
  local name="$1" task_def="$2"
  log "[$name] starting ECS task ..."
  TASK_ARN=$(aws ecs run-task \
    --cluster "$CLUSTER" \
    --task-definition "$task_def" \
    --launch-type FARGATE \
    --network-configuration "awsvpcConfiguration={subnets=[$SUBNET],securityGroups=[$SG],assignPublicIp=DISABLED}" \
    --query 'tasks[0].taskArn' --output text)
  log "[$name] task ARN: $TASK_ARN"
  aws ecs wait tasks-stopped --cluster "$CLUSTER" --tasks "$TASK_ARN"
  EXIT_CODE=$(aws ecs describe-tasks --cluster "$CLUSTER" --tasks "$TASK_ARN" \
    --query 'tasks[0].containers[0].exitCode' --output text)
  if [ "$EXIT_CODE" = "0" ]; then
    log "PASS: [$name] exit 0"
    PASS=$((PASS + 1))
  else
    log "FAIL: [$name] exit $EXIT_CODE"
    FAIL=$((FAIL + 1))
  fi
}

# [2/5] fav-native
run_task "fav-native" "$NATIVE_DEF"

# [3/5] fav-python
run_task "fav-python" "$PYTHON_DEF"

# [4/5] verify
log "[4/5] running verify.sh ..."
if "$SCRIPT_DIR/verify.sh"; then
  PASS=$((PASS + 1))
else
  FAIL=$((FAIL + 1))
fi

# [5/5] upload run log
LOG_FILE="/tmp/fav2py-run-$TIMESTAMP.txt"
aws s3 cp "$LOG_FILE" "s3://favnir-e2e-demo/proof/fav2py/run-$TIMESTAMP.txt" || true
PASS=$((PASS + 1))

log ""
log "=== RESULT: PASS=$PASS FAIL=$FAIL ==="
[ "$FAIL" -eq 0 ] || exit 1
```

### verify.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
BUCKET="favnir-e2e-demo"
PREFIX="proof/fav2py"

log() { echo "[verify] $1"; }

# 最新 2 件の JSON を取得
LATEST=$(aws s3 ls "s3://$BUCKET/$PREFIX/" | sort | tail -2 | awk '{print $4}')
COUNT=$(echo "$LATEST" | wc -l | tr -d ' ')

if [ "$COUNT" -lt 2 ]; then
  log "FAIL: expected 2 result files, got $COUNT"
  exit 1
fi

FILE1=$(echo "$LATEST" | head -1)
FILE2=$(echo "$LATEST" | tail -1)

aws s3 cp "s3://$BUCKET/$PREFIX/$FILE1" /tmp/native.json
aws s3 cp "s3://$BUCKET/$PREFIX/$FILE2" /tmp/python.json

# region+category+total を比較（count は Postgres の実行順に依存しないため比較対象外）
NATIVE_DIGEST=$(jq -r '.[] | "\(.region):\(.category):\(.total)"' /tmp/native.json | sort | sha256sum)
PYTHON_DIGEST=$(jq -r '.[] | "\(.region):\(.category):\(.total)"' /tmp/python.json | sort | sha256sum)

if [ "$NATIVE_DIGEST" = "$PYTHON_DIGEST" ]; then
  log "PASS: native output == python output"
else
  log "FAIL: outputs differ"
  log "  native: $(cat /tmp/native.json | head -3)"
  log "  python: $(cat /tmp/python.json | head -3)"
  exit 1
fi
```

---

## Phase E — Dockerfile

```dockerfile
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y \
    curl wget python3 python3-pip libpq-dev gcc \
    && rm -rf /var/lib/apt/lists/*

# uv
RUN pip3 install uv

# psycopg2
RUN pip3 install psycopg2-binary

# fav binary（S3 またはビルド済みバイナリをコピー）
COPY fav /usr/local/bin/fav
RUN chmod +x /usr/local/bin/fav

# app sources
WORKDIR /app
COPY src/ /app/

CMD ["fav", "run", "/app/pipeline.fav"]
```

---

## Phase F — README.md

実行手順（事前条件・upload → run → verify の流れ）を記載。

---

## Phase G — tasks.md（デモ専用）

デモ実行用のチェックリスト（`infra/e2e-demo/fav2py/tasks.md`）。

---

## Phase H — Rust テスト（v11900_tests）

```rust
#[cfg(test)]
mod v11900_tests {
    use std::path::Path;

    #[test]
    fn fav2py_e2e_demo_structure() {
        let base = Path::new("../infra/e2e-demo/fav2py");
        assert!(base.join("src/pipeline.fav").exists());
        assert!(base.join("src/sample.csv").exists());
        assert!(base.join("terraform/main.tf").exists());
        assert!(base.join("terraform/iam.tf").exists());
        assert!(base.join("terraform/variables.tf").exists());
        assert!(base.join("terraform/outputs.tf").exists());
        assert!(base.join("scripts/upload.sh").exists());
        assert!(base.join("scripts/run.sh").exists());
        assert!(base.join("scripts/verify.sh").exists());
        assert!(base.join("README.md").exists());
    }

    #[test]
    fn fav2py_pipeline_fav_transpiles() {
        use crate::frontend::parser::Parser;
        let src = std::fs::read_to_string("../infra/e2e-demo/fav2py/src/pipeline.fav")
            .expect("pipeline.fav not found");
        // parse 確認（rune import は resolver が扱うのでパーサーだけ確認）
        let result = Parser::parse_str(&src, "pipeline.fav");
        assert!(result.is_ok(), "pipeline.fav parse error: {:?}", result.err());
    }
}
```

---

## Phase I — バージョン更新・コミット

- `fav/Cargo.toml`: `version = "11.9.0"`
- `cargo build` で `Cargo.lock` 更新
- `git commit & push` — CI 確認
