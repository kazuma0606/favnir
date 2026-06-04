# Favnir v10.9.0 実装計画

Date: 2026-06-04
Theme: E2E テスト（実 Snowflake インスタンス）

---

## Phase A: infra/e2e-demo/snowflake/ ディレクトリ構築

`infra/e2e-demo/ecs/` の構造を参考に Snowflake E2E デモディレクトリを作成する。

```
infra/e2e-demo/snowflake/
├── README.md
├── src/demo.fav
├── terraform/
│   ├── main.tf
│   ├── variables.tf
│   ├── outputs.tf
│   ├── warehouse.tf
│   ├── table.tf
│   └── iam.tf
└── scripts/run.sh
```

---

## Phase B: src/demo.fav

LoadCsv |> TransformRows |> SnowflakeInsert |> QuerySummary の 4 ステージパイプライン。

```favnir
import rune "snowflake"
import rune "csv"
import rune "aws"

type OrderRow   = { order_id: Int  customer: String  amount: Float  region: String }
type SummaryRow = { region: String  total: Float  count: Int }

stage LoadCsv: String -> List<OrderRow> !IO = |path| {
  csv.read<OrderRow>(path)
}

stage TransformRows: List<OrderRow> -> List<OrderRow> = |rows| {
  List.filter(rows, |r| r.amount > 0.0)
}

stage SnowflakeInsert: List<OrderRow> -> Int !Snowflake = |rows| {
  bind sql <- Json.encode_raw(rows)
  snowflake.execute($"INSERT INTO DEMO_DB.PUBLIC.ORDERS SELECT $1:order_id::INT, $1:customer::STRING, $1:amount::FLOAT, $1:region::STRING FROM (SELECT PARSE_JSON('{sql}') v)")
}

stage QuerySummary: Int -> Unit !Snowflake !AWS = |_| {
  bind rows <- snowflake.query<SummaryRow>(
    "SELECT region, SUM(amount) AS total, COUNT(*) AS count FROM DEMO_DB.PUBLIC.ORDERS GROUP BY region ORDER BY region"
  )
  bind ts  <- aws.timestamp()
  aws.s3_put_json($"favnir-e2e-demo/proof/snowflake/summary-{ts}.json", rows)
}

seq DemoPipeline = LoadCsv |> TransformRows |> SnowflakeInsert |> QuerySummary
```

---

## Phase C: terraform/

### main.tf

```hcl
terraform {
  required_providers {
    snowflake = {
      source  = "Snowflake-Labs/snowflake"
      version = "~> 0.87"
    }
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "snowflake" {
  account   = var.snowflake_account
  username  = var.snowflake_user
  role      = var.snowflake_role
}

provider "aws" {
  region = var.aws_region
}
```

### variables.tf

```hcl
variable "snowflake_account"   { type = string }
variable "snowflake_user"      { type = string }
variable "snowflake_role"      { type = string  default = "SYSADMIN" }
variable "aws_region"          { type = string  default = "ap-northeast-1" }
variable "s3_bucket"           { type = string  default = "favnir-e2e-demo" }
```

### warehouse.tf

```hcl
resource "snowflake_warehouse" "demo" {
  name           = "DEMO_WH"
  warehouse_size = "X-SMALL"
  auto_suspend   = 60
  auto_resume    = true
}
```

### table.tf

```hcl
resource "snowflake_database" "demo" {
  name = "DEMO_DB"
}

resource "snowflake_schema" "public" {
  database = snowflake_database.demo.name
  name     = "PUBLIC"
}

resource "snowflake_table" "orders" {
  database = snowflake_database.demo.name
  schema   = snowflake_schema.public.name
  name     = "ORDERS"

  column { name = "order_id"  type = "NUMBER(38,0)" nullable = false }
  column { name = "customer"  type = "VARCHAR(256)"  nullable = false }
  column { name = "amount"    type = "FLOAT"         nullable = false }
  column { name = "region"    type = "VARCHAR(64)"   nullable = false }
}
```

### iam.tf

```hcl
resource "aws_iam_role" "snowflake_e2e" {
  name               = "favnir-snowflake-e2e"
  assume_role_policy = data.aws_iam_policy_document.assume.json
}

data "aws_iam_policy_document" "assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

resource "aws_iam_role_policy" "s3_proof" {
  name = "s3-proof-write"
  role = aws_iam_role.snowflake_e2e.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = ["s3:PutObject", "s3:GetObject"]
      Resource = "arn:aws:s3:::${var.s3_bucket}/proof/snowflake/*"
    }]
  })
}
```

### outputs.tf

```hcl
output "warehouse_name" { value = snowflake_warehouse.demo.name }
output "table_fqn"      { value = "${snowflake_table.orders.database}.${snowflake_table.orders.schema}.${snowflake_table.orders.name}" }
```

---

## Phase D: scripts/run.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_FAV="$SCRIPT_DIR/../src/demo.fav"
CSV_PATH="$SCRIPT_DIR/../src/sample.csv"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_FILE="/tmp/snowflake-e2e-$TIMESTAMP.txt"

echo "=== Favnir v10.9.0 Snowflake E2E Demo ===" | tee "$LOG_FILE"
echo "Timestamp: $TIMESTAMP" | tee -a "$LOG_FILE"

# 型チェック
echo "[1/4] fav check..." | tee -a "$LOG_FILE"
fav check "$DEMO_FAV" && echo "PASS: type check" | tee -a "$LOG_FILE"

# 実行（LoadCsv -> TransformRows -> SnowflakeInsert -> QuerySummary）
echo "[2/4] fav run..." | tee -a "$LOG_FILE"
fav run "$DEMO_FAV" "$CSV_PATH" && echo "PASS: pipeline run" | tee -a "$LOG_FILE"

# 証跡 S3 アップロード
echo "[3/4] uploading run log..." | tee -a "$LOG_FILE"
aws s3 cp "$LOG_FILE" "s3://favnir-e2e-demo/proof/snowflake/run-$TIMESTAMP.txt"
echo "PASS: proof uploaded" | tee -a "$LOG_FILE"

echo "=== PASS=4 FAIL=0 ===" | tee -a "$LOG_FILE"
```

---

## Phase E: Rust テスト（+1）

`driver.rs` の `v10900_tests` モジュールに 1 件追加:

```rust
#[cfg(test)]
mod v10900_tests {
    #[test]
    fn snowflake_e2e_demo_structure() {
        let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let base = std::path::Path::new(&root)
            .parent().unwrap()
            .join("infra/e2e-demo/snowflake");
        assert!(base.exists(),           "infra/e2e-demo/snowflake/ must exist");
        assert!(base.join("src/demo.fav").exists(), "src/demo.fav must exist");
        assert!(base.join("README.md").exists(),    "README.md must exist");
    }
}
```

---

## Phase F: README.md

`infra/e2e-demo/snowflake/README.md` に実行手順を記載:
1. 前提条件（Snowflake アカウント・AWS 認証情報・fav バイナリ）
2. `terraform init && terraform apply`
3. 環境変数の設定（`SNOWFLAKE_ACCOUNT` / `SNOWFLAKE_PRIVATE_KEY` 等）
4. `./scripts/run.sh` 実行
5. S3 証跡の確認

---

## Phase G: バージョン更新

- `fav/Cargo.toml` version → `"10.9.0"`
- `fav/self/cli.fav` の `run_version` → `"10.9.0"`

---

## Phase H: self-check + cargo test

1. `fav check --legacy-check self/compiler.fav`
2. `cargo test bootstrap`
3. `cargo test` 全件（1283 件）
