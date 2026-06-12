variable "azure_location" {
  description = "Azure region"
  type        = string
  default     = "japaneast"
}

variable "azure_pg_password" {
  description = "Azure PostgreSQL administrator password"
  type        = string
  sensitive   = true
}

variable "env_suffix" {
  description = "Suffix appended to resource names (e.g. dev, prod)"
  type        = string
  default     = "dev"
}
