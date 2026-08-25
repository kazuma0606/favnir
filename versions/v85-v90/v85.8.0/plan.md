# Plan: v85.8.0 — SSM Parameter Store 設定（`infra/sap/`）

## Step 1: 前提確認

- `cargo test` を実行し、3,945 tests, 0 failures を確認する
- `fav/src/driver.rs` に `mod v85700_tests` が存在することを確認する（v85.7.0 完了済みの証拠）
- `infra/snowflake/ssm.tf` が存在することを確認する（パターン参照用）

## Step 2: `infra/sap/` ディレクトリとファイルを作成

以下の順序で作成する。

### 2-1: `infra/sap/providers.tf`

```hcl
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  backend "s3" {
    bucket = "favnir-terraform-state"
    key    = "sap/terraform.tfstate"
    region = "ap-northeast-1"
  }
}

provider "aws" {
  region = var.aws_region
}
```

### 2-2: `infra/sap/variables.tf`

```hcl
variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-1"
}

variable "sap_base_url" {
  description = "SAP S/4HANA OData v4 base URL (e.g. https://my-s4hana.example.com/sap/opu/odata/sap)"
  type        = string
}

variable "sap_client" {
  description = "SAP client number"
  type        = string
  default     = "100"
}

variable "sap_auth" {
  description = "SAP authentication type"
  type        = string
  default     = "basic"
}

variable "sap_username" {
  description = "SAP username"
  type        = string
  sensitive   = true
}

variable "sap_password" {
  description = "SAP password"
  type        = string
  sensitive   = true
}
```

### 2-3: `infra/sap/ssm.tf`

```hcl
# ---------------------------------------------------------------------------
# SSM Parameter Store — SAP S/4HANA connection info
# ---------------------------------------------------------------------------

resource "aws_ssm_parameter" "sap_base_url" {
  name        = "/favnir/sap/base_url"
  description = "SAP S/4HANA OData v4 base URL"
  type        = "String"
  value       = var.sap_base_url

  tags = {
    Project   = "favnir"
    ManagedBy = "terraform"
  }
}

resource "aws_ssm_parameter" "sap_client" {
  name        = "/favnir/sap/client"
  description = "SAP client number"
  type        = "String"
  value       = var.sap_client

  tags = {
    Project   = "favnir"
    ManagedBy = "terraform"
  }
}

resource "aws_ssm_parameter" "sap_auth" {
  name        = "/favnir/sap/auth"
  description = "SAP authentication type (basic or oauth2)"
  type        = "String"
  value       = var.sap_auth

  tags = {
    Project   = "favnir"
    ManagedBy = "terraform"
  }
}

resource "aws_ssm_parameter" "sap_username" {
  name        = "/favnir/sap/username"
  description = "SAP username (SecureString)"
  type        = "SecureString"
  value       = var.sap_username

  tags = {
    Project   = "favnir"
    ManagedBy = "terraform"
  }

  lifecycle {
    ignore_changes = [value]
  }
}

resource "aws_ssm_parameter" "sap_password" {
  name        = "/favnir/sap/password"
  description = "SAP password (SecureString)"
  type        = "SecureString"
  value       = var.sap_password

  tags = {
    Project   = "favnir"
    ManagedBy = "terraform"
  }

  lifecycle {
    ignore_changes = [value]
  }
}
```

### 2-4: `infra/sap/outputs.tf`

```hcl
output "ssm_prefix" {
  description = "SSM Parameter Store path prefix for SAP connection info"
  value       = "/favnir/sap/"
}

output "sap_base_url_ssm_name" {
  description = "SSM parameter name for SAP base URL"
  value       = aws_ssm_parameter.sap_base_url.name
}
```

### 2-5: `infra/sap/README.md`

セットアップ手順（前提条件 / 変数設定 / terraform init・plan・apply コマンド）を記述する。

## Step 3: `fav/src/driver.rs` に `mod v85800_tests` を追加

`mod v85700_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v85800_tests {
    use std::path::Path;

    #[test]
    fn sap_infra_ssm_tf_exists() {
        assert!(
            Path::new("../infra/sap/ssm.tf").exists(),
            "infra/sap/ssm.tf should exist"
        );
    }

    #[test]
    fn sap_infra_readme_exists() {
        assert!(
            Path::new("../infra/sap/README.md").exists(),
            "infra/sap/README.md should exist"
        );
    }
}
```

## Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3947 tests, 0 failures
```

## Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v85.8.0 エントリを追加する。

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
