variable "aws_region" {
  description = "AWS region"
  default     = "ap-northeast-1"
}

variable "prefix" {
  description = "Resource name prefix"
  default     = "favnir-kafka-demo"
}

variable "msk_username" {
  description = "MSK SASL username"
  default     = "favnir"
}

variable "msk_password" {
  description = "MSK SASL password"
  sensitive   = true
}
