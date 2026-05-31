output "lambda_compiler_arn" {
  value = aws_lambda_function.compiler.arn
}

output "lambda_executor_arn" {
  value = aws_lambda_function.executor.arn
}

output "sqs_queue_url" {
  value = aws_sqs_queue.pipeline.url
}

output "rds_endpoint" {
  value = aws_rds_cluster.demo.endpoint
}

output "vpc_id" {
  value = aws_vpc.main.id
}
