variable "snowflake_account" {
  description = "Snowflake account identifier (e.g. xy12345.ap-northeast-1.aws)"
  type        = string
}

variable "snowflake_user" {
  description = "Snowflake username"
  type        = string
}

variable "snowflake_role" {
  description = "Snowflake role to use"
  type        = string
  default     = "SYSADMIN"
}

variable "snowflake_warehouse" {
  description = "Snowflake warehouse name"
  type        = string
  default     = "DEMO_WH"
}

variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-1"
}

variable "s3_bucket" {
  description = "S3 bucket for proof artifacts"
  type        = string
  default     = "favnir-e2e-demo"
}
