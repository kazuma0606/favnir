variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "ap-northeast-1"
}

variable "sap_base_url" {
  description = "SAP S/4HANA OData v4 base URL (e.g. https://my-s4hana.example.com/sap/opu/odata/sap)"
  type        = string
}

variable "sap_client" {
  description = "SAP client number"
  type        = string
  default     = "100"
}

variable "sap_auth" {
  description = "SAP authentication type"
  type        = string
  default     = "basic"
}

variable "sap_username" {
  description = "SAP username"
  type        = string
  sensitive   = true
}

variable "sap_password" {
  description = "SAP password"
  type        = string
  sensitive   = true
}
