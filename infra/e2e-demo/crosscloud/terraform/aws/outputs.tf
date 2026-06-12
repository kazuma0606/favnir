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
