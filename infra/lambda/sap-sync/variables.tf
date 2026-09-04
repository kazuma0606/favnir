variable "sap_base_url" {
  description = "SAP S/4HANA OData base URL (e.g. https://my-sap.example.com/sap/opu/odata/sap)"
  type        = string
}

# infra/sap/ssm.tf の sap_client と変数名を統一する（既存パターン準拠）
variable "sap_client_id" {
  description = "SAP client ID (corresponds to /favnir/sap/client in SSM)"
  type        = string
  default     = "100"
}

variable "lambda_role_arn" {
  description = "IAM role ARN for Lambda execution (must have CloudWatch Logs and SSM GetParameter permission)"
  type        = string
}
