output "ssm_prefix" {
  description = "SSM Parameter Store path prefix for SAP connection info"
  value       = "/favnir/sap/"
}

output "sap_base_url_ssm_name" {
  description = "SSM parameter name for SAP base URL"
  value       = aws_ssm_parameter.sap_base_url.name
}

output "sap_client_ssm_name" {
  description = "SSM parameter name for SAP client number"
  value       = aws_ssm_parameter.sap_client.name
}

output "sap_auth_ssm_name" {
  description = "SSM parameter name for SAP auth type"
  value       = aws_ssm_parameter.sap_auth.name
}

output "sap_username_ssm_name" {
  description = "SSM parameter name for SAP username"
  value       = aws_ssm_parameter.sap_username.name
}

output "sap_password_ssm_name" {
  description = "SSM parameter name for SAP password"
  value       = aws_ssm_parameter.sap_password.name
}
