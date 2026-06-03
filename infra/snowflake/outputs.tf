output "snowflake_warehouse_name" {
  description = "Snowflake warehouse name"
  value       = snowflake_warehouse.favnir.name
}

output "snowflake_database_name" {
  description = "Snowflake database name"
  value       = snowflake_database.favnir.name
}

output "snowflake_schema_name" {
  description = "Snowflake schema name"
  value       = snowflake_schema.public.name
}

output "snowflake_app_role" {
  description = "Snowflake application role name"
  value       = snowflake_account_role.favnir_app.name
}

output "ssm_prefix" {
  description = "SSM Parameter Store path prefix for Snowflake connection info"
  value       = "/favnir/snowflake/"
}
