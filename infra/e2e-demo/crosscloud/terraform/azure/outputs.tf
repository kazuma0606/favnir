output "postgresql_fqdn" {
  description = "Azure PostgreSQL Flexible Server FQDN"
  value       = azurerm_postgresql_flexible_server.target.fqdn
}

output "storage_account_name" {
  description = "Azure Storage Account name"
  value       = azurerm_storage_account.proof.name
}

output "storage_account_key" {
  description = "Azure Storage Account primary access key"
  value       = azurerm_storage_account.proof.primary_access_key
  sensitive   = true
}

output "azure_conn_str" {
  description = "Azure PostgreSQL connection string (libpq format)"
  value       = "host=${azurerm_postgresql_flexible_server.target.fqdn} port=5432 user=favnir password=${var.azure_pg_password} dbname=appdb sslmode=require"
  sensitive   = true
}
