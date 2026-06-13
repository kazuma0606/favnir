output "dataset_id" {
  value       = google_bigquery_dataset.demo.dataset_id
  description = "BigQuery データセット ID"
}

output "table_id" {
  value       = google_bigquery_table.users.table_id
  description = "BigQuery テーブル ID"
}

output "sa_email" {
  value       = google_service_account.favnir_demo.email
  description = "サービスアカウントメールアドレス"
}

output "sa_key_path" {
  value       = local_file.sa_key.filename
  description = "サービスアカウントキーファイルのパス"
}
