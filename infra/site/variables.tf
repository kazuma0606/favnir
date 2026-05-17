variable "aws_region" {
  description = "Primary AWS region"
  type        = string
  default     = "ap-northeast-1"
}

variable "site_bucket_name" {
  description = "S3 bucket name for the reference site static files"
  type        = string
  default     = "favnir-site"
}

variable "environment" {
  description = "Deployment environment (prod / staging)"
  type        = string
  default     = "prod"
}

variable "cloudfront_price_class" {
  description = "CloudFront price class (PriceClass_100 = US/EU only, PriceClass_All = global)"
  type        = string
  default     = "PriceClass_All"
}
