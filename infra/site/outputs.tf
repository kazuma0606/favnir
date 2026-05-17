output "cloudfront_domain" {
  description = "CloudFront distribution domain name (access the site via HTTPS)"
  value       = "https://${aws_cloudfront_distribution.site.domain_name}"
}

output "site_bucket" {
  description = "S3 bucket name for the reference site"
  value       = aws_s3_bucket.site.id
}

output "distribution_id" {
  description = "CloudFront distribution ID (used for cache invalidation)"
  value       = aws_cloudfront_distribution.site.id
}

output "site_bucket_arn" {
  description = "S3 bucket ARN"
  value       = aws_s3_bucket.site.arn
}
