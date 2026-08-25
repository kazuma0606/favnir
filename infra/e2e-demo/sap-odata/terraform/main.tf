terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

data "aws_caller_identity" "current" {}

# S3 出力バケット
resource "aws_s3_bucket" "demo_output" {
  bucket = "favnir-sap-e2e-demo-output-${var.environment}"
}

# IAM ロール
resource "aws_iam_role" "demo_role" {
  name = "favnir-sap-e2e-demo-role-${var.environment}"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "basic_exec" {
  role       = aws_iam_role.demo_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy" "ssm_s3_policy" {
  name = "favnir-sap-e2e-demo-ssm-s3-${var.environment}"
  role = aws_iam_role.demo_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["ssm:GetParameter", "ssm:GetParameters"]
        Resource = "arn:aws:ssm:${var.aws_region}:${data.aws_caller_identity.current.account_id}:parameter/favnir/sap/*"
      },
      {
        Effect   = "Allow"
        Action   = ["s3:PutObject", "s3:GetObject"]
        Resource = "${aws_s3_bucket.demo_output.arn}/*"
      }
    ]
  })
}

# Lambda 関数（favnir-sap-e2e-demo）
# Note: count = 0 — bootstrap.zip は v88.9.0 で整備する。v88.9.0 で count = 1 に変更すること。
# セキュリティ: SAP 認証情報は SSM パラメータパスのみを渡し、Lambda が実行時に SSM から取得する。
resource "aws_lambda_function" "favnir_sap_e2e_demo" {
  count         = 0
  function_name = "favnir-sap-e2e-demo-${var.environment}"
  role          = aws_iam_role.demo_role.arn
  runtime       = "provided.al2"
  handler       = "bootstrap"
  filename      = "${path.module}/../lambda/bootstrap.zip"

  environment {
    variables = {
      SAP_BASE_URL_SSM_PATH  = data.aws_ssm_parameter.sap_base_url.name
      SAP_USERNAME_SSM_PATH  = data.aws_ssm_parameter.sap_username.name
      SAP_PASSWORD_SSM_PATH  = data.aws_ssm_parameter.sap_password.name
      SAP_CLIENT_SSM_PATH    = data.aws_ssm_parameter.sap_client.name
      OUTPUT_BUCKET          = aws_s3_bucket.demo_output.bucket
    }
  }
}
