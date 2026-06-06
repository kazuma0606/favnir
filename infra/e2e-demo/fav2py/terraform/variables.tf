variable "aws_region" {
  description = "AWS region"
  default     = "ap-northeast-1"
}

variable "db_password" {
  description = "RDS master password"
  sensitive   = true
}

variable "s3_bucket" {
  description = "S3 bucket for proof artifacts"
  default     = "favnir-e2e-demo"
}

variable "fav_image_tag" {
  description = "Docker image tag for fav2py ECR image"
  default     = "latest"
}
