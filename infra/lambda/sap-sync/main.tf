# SAP 同期 Lambda — Lambda SnapStart 有効（v94.3.0）
# SnapStart により PublishedVersions 実行時のコールドスタートを大幅に削減する。
#
# 認証情報は SSM SecureString から実行時取得する（infra/sap/ssm.tf のパターンに準拠）。
# SAP_USER_SSM_PATH / SAP_PASS_SSM_PATH に SSM パス名のみを渡し、
# Lambda アプリケーションコードが起動時に SSM SDK で値を取得する。
resource "aws_lambda_function" "sap_sync" {
  function_name = "favnir-sap-sync"
  role          = var.lambda_role_arn
  runtime       = "java21"
  handler       = "io.favnir.sap.SapSyncHandler::handleRequest"
  filename      = "${path.module}/sap-sync.jar"

  snap_start {
    apply_on = "PublishedVersions"
  }

  environment {
    variables = {
      SAP_BASE_URL       = var.sap_base_url
      SAP_CLIENT_ID      = var.sap_client_id
      SAP_USER_SSM_PATH  = "/favnir/sap/username"
      SAP_PASS_SSM_PATH  = "/favnir/sap/password"
    }
  }

  tags = {
    Project   = "favnir"
    Module    = "sap-sync"
    ManagedBy = "terraform"
  }
}
