output "lambda_arn" {
  description = "SAP sync Lambda function ARN"
  value       = aws_lambda_function.sap_sync.arn
}

output "lambda_function_name" {
  description = "SAP sync Lambda function name"
  value       = aws_lambda_function.sap_sync.function_name
}
