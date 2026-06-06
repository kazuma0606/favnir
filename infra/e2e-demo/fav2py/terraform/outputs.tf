output "rds_endpoint" {
  description = "RDS PostgreSQL endpoint"
  value       = aws_db_instance.postgres.address
}

output "ecr_repository" {
  description = "ECR repository URI"
  value       = aws_ecr_repository.fav2py.repository_url
}

output "ecs_cluster_arn" {
  description = "ECS cluster ARN"
  value       = aws_ecs_cluster.fav2py.arn
}

output "native_task_def" {
  description = "ECS task definition ARN for fav-native"
  value       = aws_ecs_task_definition.native.arn
}

output "python_task_def" {
  description = "ECS task definition ARN for fav-python"
  value       = aws_ecs_task_definition.python.arn
}

output "private_subnet_id" {
  description = "Private subnet ID for ECS tasks"
  value       = aws_subnet.private.id
}

output "ecs_security_group_id" {
  description = "Security group ID for ECS tasks"
  value       = aws_security_group.ecs.id
}
