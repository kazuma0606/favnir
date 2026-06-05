variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-1"
}

variable "aws_account" {
  description = "AWS account ID"
  type        = string
  sensitive   = true
}

variable "bucket_name" {
  description = "S3 bucket for demo artifacts"
  type        = string
  default     = "favnir-e2e-demo"
}

variable "favnir_binary_key" {
  description = "S3 key of the Favnir binary"
  type        = string
  default     = "airgap/binary/fav"
}
