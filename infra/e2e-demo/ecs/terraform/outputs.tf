output "machine_a_public_ip" {
  description = "Machine A のパブリック IP（SSH 接続先）"
  value       = aws_instance.machine_a.public_ip
}

output "machine_b_instance_id" {
  description = "Machine B のインスタンス ID（SSM 接続用）"
  value       = aws_instance.machine_b.id
}

output "rds_endpoint" {
  description = "RDS クラスターのエンドポイント（seed-db.sh の DB_HOST に使用）"
  value       = aws_rds_cluster.demo.endpoint
}

output "s3_bucket_name" {
  description = "S3 バケット名"
  value       = aws_s3_bucket.demo.bucket
}

output "ecs_cluster_arn" {
  description = "ECS クラスター ARN"
  value       = aws_ecs_cluster.demo.arn
}

output "ecs_task_definition_arn" {
  description = "ECS Task Definition ARN（run-ecs-task.sh で使用）"
  value       = aws_ecs_task_definition.etl.arn
}

output "private_subnet_id" {
  description = "Private Subnet ID（ECS Task 起動時に使用）"
  value       = aws_subnet.private.id
}

output "ecs_security_group_id" {
  description = "ECS Security Group ID（ECS Task 起動時に使用）"
  value       = aws_security_group.ecs.id
}

output "ecr_uri" {
  description = "favnir/runtime ECR イメージ URI"
  value       = var.ecr_uri
}

output "db_secret_arn" {
  description = "Secrets Manager の DB URL シークレット ARN"
  value       = aws_secretsmanager_secret.db_url.arn
  sensitive   = true
}
