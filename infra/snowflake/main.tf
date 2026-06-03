locals {
  tags = {
    Project     = "favnir"
    Environment = var.environment
    ManagedBy   = "terraform"
  }
  snowflake_account = "${var.snowflake_organization}-${var.snowflake_account_name}"
}

# ---------------------------------------------------------------------------
# Warehouse
# ---------------------------------------------------------------------------

resource "snowflake_warehouse" "favnir" {
  name                = "FAVNIR_WH"
  warehouse_size      = var.snowflake_warehouse_size
  auto_suspend        = 60
  auto_resume         = true
  initially_suspended = true

  comment = "Favnir application warehouse — managed by Terraform"
}

# ---------------------------------------------------------------------------
# Database
# ---------------------------------------------------------------------------

resource "snowflake_database" "favnir" {
  name    = var.snowflake_database
  comment = "Favnir application database — managed by Terraform"
}

# ---------------------------------------------------------------------------
# Schema
# ---------------------------------------------------------------------------

resource "snowflake_schema" "public" {
  database = snowflake_database.favnir.name
  name     = var.snowflake_schema
  comment  = "Favnir default schema — managed by Terraform"
}

# ---------------------------------------------------------------------------
# Application role
# ---------------------------------------------------------------------------

resource "snowflake_account_role" "favnir_app" {
  name    = "FAVNIR_APP"
  comment = "Favnir application role — managed by Terraform"
}

resource "snowflake_grant_privileges_to_account_role" "warehouse_usage" {
  account_role_name = snowflake_account_role.favnir_app.name
  privileges        = ["USAGE"]

  on_account_object {
    object_type = "WAREHOUSE"
    object_name = snowflake_warehouse.favnir.name
  }
}

resource "snowflake_grant_privileges_to_account_role" "database_usage" {
  account_role_name = snowflake_account_role.favnir_app.name
  privileges        = ["USAGE"]

  on_account_object {
    object_type = "DATABASE"
    object_name = snowflake_database.favnir.name
  }
}

resource "snowflake_grant_privileges_to_account_role" "schema_privileges" {
  account_role_name = snowflake_account_role.favnir_app.name
  privileges        = ["USAGE", "CREATE TABLE", "CREATE VIEW"]

  on_schema {
    schema_name = "\"${snowflake_database.favnir.name}\".\"${snowflake_schema.public.name}\""
  }
}
