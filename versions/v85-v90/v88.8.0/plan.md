# Plan: v88.8.0 — E2E デモ Lambda 基盤

## 実装ステップ

### Step 1: Terraform ファイル作成

`infra/e2e-demo/sap-odata/terraform/` ディレクトリに以下を作成する。

#### `variables.tf`

```hcl
variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-1"
}

variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "dev"
}
```

#### `ssm.tf`

```hcl
data "aws_ssm_parameter" "sap_base_url" {
  name = "/favnir/sap/base_url"
}

data "aws_ssm_parameter" "sap_username" {
  name = "/favnir/sap/username"
}

data "aws_ssm_parameter" "sap_password" {
  name            = "/favnir/sap/password"
  with_decryption = true
}

data "aws_ssm_parameter" "sap_client" {
  name = "/favnir/sap/client"
}
```

#### `main.tf`

```hcl
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# S3 出力バケット
resource "aws_s3_bucket" "demo_output" {
  bucket = "favnir-sap-e2e-demo-output-${var.environment}"
}

# IAM ロール
resource "aws_iam_role" "demo_role" {
  name = "favnir-sap-e2e-demo-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "basic_exec" {
  role       = aws_iam_role.demo_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "ssm_s3_policy" {
  name = "favnir-sap-e2e-demo-ssm-s3"
  role = aws_iam_role.demo_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["ssm:GetParameter", "ssm:GetParameters"]
        Resource = "arn:aws:ssm:${var.aws_region}:*:parameter/favnir/sap/*"
      },
      {
        Effect   = "Allow"
        Action   = ["s3:PutObject", "s3:GetObject"]
        Resource = "${aws_s3_bucket.demo_output.arn}/*"
      }
    ]
  })
}

# Lambda 関数
resource "aws_lambda_function" "favnir_sap_e2e_demo" {
  function_name = "favnir-sap-e2e-demo"
  role          = aws_iam_role.demo_role.arn
  runtime       = "provided.al2"
  handler       = "bootstrap"
  filename      = "${path.module}/../lambda/bootstrap.zip"

  environment {
    variables = {
      SAP_BASE_URL = data.aws_ssm_parameter.sap_base_url.value
      SAP_USERNAME = data.aws_ssm_parameter.sap_username.value
      SAP_PASSWORD = data.aws_ssm_parameter.sap_password.value
      SAP_CLIENT   = data.aws_ssm_parameter.sap_client.value
      OUTPUT_BUCKET = aws_s3_bucket.demo_output.bucket
    }
  }
}
```

### Step 2: `scripts/run.sh` 作成

```bash
#!/usr/bin/env bash
set -euo pipefail

FUNCTION_NAME="favnir-sap-e2e-demo"
REGION="${AWS_DEFAULT_REGION:-ap-northeast-1}"
OUTPUT_FILE="/tmp/sap-demo-output.json"

echo "=== Favnir SAP E2E Demo ==="
echo "Invoking Lambda: $FUNCTION_NAME"

aws lambda invoke \
  --function-name "$FUNCTION_NAME" \
  --region "$REGION" \
  --payload '{}' \
  "$OUTPUT_FILE"

echo "=== Result ==="
cat "$OUTPUT_FILE"
echo ""
echo "Done."
```

### Step 3: `fav/src/driver.rs` に `mod v88800_tests` を追加

`mod v88700_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v88800_tests {
    #[test]
    fn sap_e2e_demo_terraform_exists() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/terraform/main.tf",
        )
        .expect("infra/e2e-demo/sap-odata/terraform/main.tf should exist");
        assert!(
            content.contains("favnir-sap-e2e-demo"),
            "main.tf should reference favnir-sap-e2e-demo"
        );
    }

    #[test]
    fn sap_e2e_demo_run_script_exists() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/scripts/run.sh",
        )
        .expect("infra/e2e-demo/sap-odata/scripts/run.sh should exist");
        let lower = content.to_lowercase();
        assert!(
            lower.contains("lambda"),
            "run.sh should reference lambda"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

4,011 + 2 = 4,013 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
