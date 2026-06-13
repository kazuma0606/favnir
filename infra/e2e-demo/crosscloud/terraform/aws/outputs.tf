output "rds_endpoint" {
  description = "RDS PostgreSQL endpoint (host:port)"
  value       = "${aws_db_instance.source.address}:${aws_db_instance.source.port}"
}

output "s3_proof_bucket" {
  description = "S3 bucket name for proof artifacts"
  value       = aws_s3_bucket.proof.bucket
}

output "rds_conn_secret_arn" {
  description = "Secrets Manager ARN storing the RDS connection string"
  value       = aws_secretsmanager_secret.rds_conn.arn
}

output "api_gateway_endpoint" {
  description = "API Gateway HTTP API endpoint URL"
  value       = aws_apigatewayv2_api.crosscloud.api_endpoint
}

output "ecr_repository_url" {
  description = "ECR repository URL for the fav container image"
  value       = aws_ecr_repository.fav.repository_url
}

output "ecs_cluster_name" {
  description = "ECS cluster name"
  value       = aws_ecs_cluster.crosscloud.name
}

output "cognito_user_pool_id" {
  description = "Cognito User Pool ID"
  value       = aws_cognito_user_pool.crosscloud.id
}

output "cognito_client_id" {
  description = "Cognito User Pool Client ID"
  value       = aws_cognito_user_pool_client.crosscloud.id
}

output "verifier_ecr_url" {
  description = "ECR repository URL for the Lambda verifier container image"
  value       = aws_ecr_repository.verifier.repository_url
}
