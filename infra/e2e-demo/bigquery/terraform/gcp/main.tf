terraform {
  required_providers {
    google = { source = "hashicorp/google", version = "~> 5.0" }
    local  = { source = "hashicorp/local",  version = "~> 2.0" }
  }
}

provider "google" {
  project = var.gcp_project_id
  region  = var.gcp_region
}

# ── BigQuery ──────────────────────────────────────────────────────────────────

resource "google_bigquery_dataset" "demo" {
  dataset_id  = "favnir_demo"
  description = "Favnir BigQuery E2E Demo (v15.2.0)"
  location    = var.gcp_region

  labels = {
    project = "favnir-bigquery-demo"
  }
}

resource "google_bigquery_table" "users" {
  dataset_id          = google_bigquery_dataset.demo.dataset_id
  table_id            = "users"
  deletion_protection = false

  schema = jsonencode([
    { name = "user_id",   type = "INT64",  mode = "REQUIRED", description = "ユーザー ID" },
    { name = "full_name", type = "STRING", mode = "NULLABLE", description = "氏名（正規化済み）" },
    { name = "email",     type = "STRING", mode = "NULLABLE", description = "メールアドレス（小文字）" },
  ])

  labels = {
    project = "favnir-bigquery-demo"
  }
}

# ── Service Account ───────────────────────────────────────────────────────────

resource "google_service_account" "favnir_demo" {
  account_id   = "favnir-bq-demo"
  display_name = "Favnir BigQuery Demo SA"
  project      = var.gcp_project_id
}

resource "google_project_iam_member" "bq_data_editor" {
  project = var.gcp_project_id
  role    = "roles/bigquery.dataEditor"
  member  = "serviceAccount:${google_service_account.favnir_demo.email}"
}

resource "google_project_iam_member" "bq_job_user" {
  project = var.gcp_project_id
  role    = "roles/bigquery.jobUser"
  member  = "serviceAccount:${google_service_account.favnir_demo.email}"
}

resource "google_service_account_key" "favnir_demo" {
  service_account_id = google_service_account.favnir_demo.name
  public_key_type    = "TYPE_X509_PEM_FILE"
}

# サービスアカウントキーをローカルファイルに書き出す
resource "local_file" "sa_key" {
  content         = base64decode(google_service_account_key.favnir_demo.private_key)
  filename        = "${path.module}/../../../../../fav/tmp/gcp-sa-key.json"
  file_permission = "0600"
}
