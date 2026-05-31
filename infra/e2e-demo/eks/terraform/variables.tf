variable "aws_region" {
  default = "ap-northeast-1"
}

variable "db_password" {
  description = "Aurora Serverless DB password"
  type        = string
  sensitive   = true
}

variable "db_user" {
  default = "favnir"
}

variable "bucket_name" {
  default = "favnir-e2e-demo"
}
