variable "aws_region" {
  default = "ap-northeast-1"
}

variable "aws_account" {
  description = "AWS Account ID"
  default     = "847333136058"
}

variable "bucket_name" {
  default = "favnir-e2e-demo"
}

variable "db_user" {
  default = "favnir"
}

variable "db_password" {
  description = "RDS master password"
  sensitive   = true
}
