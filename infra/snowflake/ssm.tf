# ---------------------------------------------------------------------------
# SSM Parameter Store — Snowflake connection info
#
# account / warehouse / database / schema are managed by Terraform.
# private_key / user / role are stored manually via `aws ssm put-parameter`
# to keep secrets out of Terraform state.
# ---------------------------------------------------------------------------

resource "aws_ssm_parameter" "snowflake_account" {
  name        = "/favnir/snowflake/account"
  description = "Snowflake account identifier"
  type        = "SecureString"
  value       = local.snowflake_account

  tags = {
    Project   = "favnir"
    ManagedBy = "terraform"
  }

  lifecycle {
    ignore_changes = [value]
  }
}

resource "aws_ssm_parameter" "snowflake_warehouse" {
  name        = "/favnir/snowflake/warehouse"
  description = "Snowflake warehouse name"
  type        = "String"
  value       = snowflake_warehouse.favnir.name

  tags = {
    Project   = "favnir"
    ManagedBy = "terraform"
  }
}

resource "aws_ssm_parameter" "snowflake_database" {
  name        = "/favnir/snowflake/database"
  description = "Snowflake database name"
  type        = "String"
  value       = snowflake_database.favnir.name

  tags = {
    Project   = "favnir"
    ManagedBy = "terraform"
  }
}

resource "aws_ssm_parameter" "snowflake_schema" {
  name        = "/favnir/snowflake/schema"
  description = "Snowflake schema name"
  type        = "String"
  value       = snowflake_schema.public.name

  tags = {
    Project   = "favnir"
    ManagedBy = "terraform"
  }
}

# ---------------------------------------------------------------------------
# The following parameters are stored manually (not managed by Terraform):
#
#   /favnir/snowflake/private_key  (SecureString)  RSA private key PEM
#   /favnir/snowflake/user         (SecureString)  Snowflake app username
#   /favnir/snowflake/role         (String)        Snowflake app role (FAVNIR_APP)
#
# To store them:
#   aws ssm put-parameter \
#     --name "/favnir/snowflake/private_key" \
#     --type "SecureString" \
#     --value "$(cat snowflake_rsa_key.p8)" \
#     --overwrite
# ---------------------------------------------------------------------------
