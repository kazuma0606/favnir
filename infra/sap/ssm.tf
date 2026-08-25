# ---------------------------------------------------------------------------
# SSM Parameter Store — SAP S/4HANA connection info
#
# base_url / client / auth は String 型で管理（非機密値）。
#   - Snowflake パターン（warehouse/database/schema）と同じ方針。
#   - lifecycle { ignore_changes } は付けない（Terraform が正の値を常に管理）。
# username / password は SecureString（KMS 暗号化）で管理。
#   - lifecycle { ignore_changes = [value] } を付与し、
#     Terraform state に平文を残さないようにする。
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
