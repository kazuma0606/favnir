output "api_url" {
  description = "Share API Gateway URL（NEXT_PUBLIC_SHARE_API_URL に設定する）"
  value       = aws_apigatewayv2_stage.share.invoke_url
}

output "s3_bucket" {
  description = "共有コードを保存する S3 バケット名"
  value       = aws_s3_bucket.shares.bucket
}
