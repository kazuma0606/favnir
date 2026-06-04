output "warehouse_name" {
  description = "Snowflake warehouse name"
  value       = snowflake_warehouse.demo.name
}

output "table_fqn" {
  description = "Fully-qualified Snowflake table name"
  value       = "${snowflake_table.orders.database}.${snowflake_table.orders.schema}.${snowflake_table.orders.name}"
}

output "iam_role_arn" {
  description = "IAM role ARN for S3 proof upload"
  value       = aws_iam_role.snowflake_e2e.arn
}
