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

variable "ecr_image_tag" {
  description = "ECR image tag for fav container"
  type        = string
  default     = "latest"
}

variable "azure_conn_str" {
  description = "Azure PostgreSQL connection string (injected into ECS task)"
  type        = string
  sensitive   = true
}

variable "azure_storage_account" {
  description = "Azure Storage Account name"
  type        = string
}

variable "azure_storage_key" {
  description = "Azure Storage Account key"
  type        = string
  sensitive   = true
}

variable "azure_container" {
  description = "Azure Blob container name"
  type        = string
  default     = "proof"
}

variable "hmac_secret" {
  description = "HMAC-SHA256 shared secret for request signing (min 32 bytes)"
  type        = string
  sensitive   = true
}
