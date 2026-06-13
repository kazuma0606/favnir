variable "gcp_project_id" {
  type        = string
  description = "GCP プロジェクト ID"
  default     = "favnir-bigquery-demo"
}

variable "gcp_region" {
  type        = string
  description = "BigQuery ロケーション / GCP リージョン"
  default     = "asia-northeast1"
}
