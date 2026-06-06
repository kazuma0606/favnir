variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-1"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "production"
}

variable "lambda_image_uri" {
  description = "ECR image URI for the Lambda function (set after first push)"
  type        = string
  default     = ""
}

variable "fav_client_token" {
  description = "Static token embedded in fav binary for registry access"
  type        = string
  sensitive   = true
}

variable "registry_admin_token" {
  description = "Admin token for rune publish operations"
  type        = string
  sensitive   = true
}
