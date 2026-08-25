# Spec: v85.8.0 — SSM Parameter Store 設定（`infra/sap/`）

## Background

v85.1.0 で Rust 側の `fav.toml [sap]` 解析・env 注入基盤を実装した。
本バージョンでは SAP 接続情報を AWS SSM Parameter Store で安全に管理する Terraform を作成する。
`infra/snowflake/` の構造をベースに、SAP 専用の `infra/sap/` モジュールを構築する。

ロードマップ: `versions/roadmap/roadmap-v85.1-v86.0.md`（v85.8.0 セクション）

## Goals

- `infra/sap/` Terraform モジュールを作成し、SAP 接続情報を SSM Parameter Store で管理する
- Rust テスト 2 件を追加して **3,947 tests** を達成する

## Files to Create

| ファイル | 内容 |
|---|---|
| `infra/sap/ssm.tf` | SSM Parameter Store リソース定義 |
| `infra/sap/variables.tf` | 変数定義（`sap_base_url` / `sap_username` / `sap_password` / `sap_client` / `sap_auth` / `aws_region`） |
| `infra/sap/providers.tf` | AWS provider + Terraform backend 設定 |
| `infra/sap/outputs.tf` | SSM パス prefix 出力 |
| `infra/sap/README.md` | セットアップ手順 |

## `infra/sap/ssm.tf` 内容

```hcl
# ---------------------------------------------------------------------------
# SSM Parameter Store — SAP S/4HANA connection info
#
# base_url / client / auth は String 型で管理。
# username / password は SecureString（KMS 暗号化）で管理。
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

## `infra/sap/variables.tf` 内容

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

## `infra/sap/providers.tf` 内容

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

## `infra/sap/outputs.tf` 内容

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

## Success Criteria

- `cargo test` が **3,947 tests**, 0 failures
- `sap_infra_ssm_tf_exists`:
  - `Path::new("../infra/sap/ssm.tf").exists()` が `true`
- `sap_infra_readme_exists`:
  - `Path::new("../infra/sap/README.md").exists()` が `true`

## Error Codes

新規エラーコードなし。

## 注記

- `infra/sap/` は Terraform Apply を実行するまで実際のAWSリソースは作成されない（ファイル存在テストのみ）
- `username` / `password` は `SecureString` + `lifecycle { ignore_changes = [value] }` で Terraform state に機密情報を残さない
- `infra/snowflake/ssm.tf` と同じタグ規約（`Project = "favnir"` / `ManagedBy = "terraform"`）を使う
