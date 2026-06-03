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

variable "snowflake_account" {
  description = "Snowflake account identifier (e.g. xy12345.ap-northeast-1.aws)"
  type        = string
}

variable "snowflake_user" {
  description = "Snowflake user for Terraform operations"
  type        = string
}

variable "snowflake_admin_role" {
  description = "Snowflake role for Terraform operations"
  type        = string
  default     = "SYSADMIN"
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
