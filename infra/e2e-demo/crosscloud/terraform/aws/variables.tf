variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-1"
}

variable "rds_password" {
  description = "RDS master password"
  type        = string
  sensitive   = true
}

variable "env_suffix" {
  description = "Suffix appended to resource names (e.g. dev, prod)"
  type        = string
  default     = "dev"
}
