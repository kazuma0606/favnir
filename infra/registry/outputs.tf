output "ecr_repository_url" {
  description = "ECR repository URL"
  value       = aws_ecr_repository.registry.repository_url
}

output "dynamodb_table_name" {
  description = "DynamoDB table name"
  value       = aws_dynamodb_table.registry.name
}

output "packages_bucket" {
  description = "S3 bucket for rune packages"
  value       = aws_s3_bucket.packages.bucket
}

output "api_url" {
  description = "API Gateway HTTP API URL (available after lambda.tf apply)"
  value       = try(aws_apigatewayv2_stage.registry.invoke_url, "not yet created")
}
