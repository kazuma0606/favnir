variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-1"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}

variable "snowflake_organization" {
  description = "Snowflake organization name (from app.snowflake.com/<org>/...)"
  type        = string
}

variable "snowflake_account_name" {
  description = "Snowflake account name (from app.snowflake.com/.../<account>)"
  type        = string
}

variable "snowflake_user" {
  description = "Snowflake user for Terraform operations"
  type        = string
}

variable "snowflake_admin_role" {
  description = "Snowflake role for Terraform operations"
  type        = string
  default     = "ACCOUNTADMIN"
}

variable "snowflake_private_key_path" {
  description = "Path to RSA private key PEM file for Snowflake JWT auth"
  type        = string
  default     = "./snowflake_rsa_key.p8"
}

variable "snowflake_warehouse_size" {
  description = "Snowflake warehouse size"
  type        = string
  default     = "X-SMALL"
}

variable "snowflake_database" {
  description = "Snowflake database name"
  type        = string
  default     = "FAVNIR"
}

variable "snowflake_schema" {
  description = "Snowflake schema name"
  type        = string
  default     = "PUBLIC"
}
