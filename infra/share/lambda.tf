# ── IAM ───────────────────────────────────────────────────────────────────────

resource "aws_iam_role" "share_lambda" {
  name = "favnir-share-lambda-${var.environment}"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy" "share_lambda_s3" {
  name = "favnir-share-s3-${var.environment}"
  role = aws_iam_role.share_lambda.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["s3:GetObject", "s3:PutObject"]
        Resource = "${aws_s3_bucket.shares.arn}/*"
      },
      {
        Effect   = "Allow"
        Action   = ["logs:CreateLogGroup", "logs:CreateLogStream", "logs:PutLogEvents"]
        Resource = "arn:aws:logs:*:*:*"
      }
    ]
  })
}

# ── Lambda ────────────────────────────────────────────────────────────────────

resource "aws_lambda_function" "share" {
  function_name = "favnir-share-${var.environment}"
  role          = aws_iam_role.share_lambda.arn
  runtime       = "nodejs20.x"
  handler       = "share.handler"

  # handlers/ を zip したファイルを参照
  # デプロイ前に: cd infra/share/handlers && npm install && zip -r share.zip .
  filename         = "${path.module}/handlers/share.zip"
  # zip が存在しない場合は apply がエラーになる（意図的 — CI で zip 生成を必須化）
  source_code_hash = filebase64sha256("${path.module}/handlers/share.zip")

  environment {
    variables = {
      SHARE_BUCKET = aws_s3_bucket.shares.bucket
      AWS_REGION   = var.region
    }
  }

  timeout     = 10
  memory_size = 128

  tags = {
    Project     = "favnir"
    Environment = var.environment
  }
}

data "aws_caller_identity" "current" {}

# ── API Gateway ───────────────────────────────────────────────────────────────

resource "aws_apigatewayv2_api" "share" {
  name          = "favnir-share-${var.environment}"
  protocol_type = "HTTP"

  cors_configuration {
    allow_origins = var.environment == "prod" ? ["https://play.favnir.dev"] : ["*"]
    allow_methods = ["GET", "POST", "OPTIONS"]
    allow_headers = ["Content-Type"]
    max_age       = 300
  }
}

resource "aws_apigatewayv2_stage" "share" {
  api_id      = aws_apigatewayv2_api.share.id
  name        = "$default"
  auto_deploy = true
}

resource "aws_apigatewayv2_integration" "share" {
  api_id             = aws_apigatewayv2_api.share.id
  integration_type   = "AWS_PROXY"
  integration_uri    = aws_lambda_function.share.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "post_share" {
  api_id    = aws_apigatewayv2_api.share.id
  route_key = "POST /share"
  target    = "integrations/${aws_apigatewayv2_integration.share.id}"
}

resource "aws_apigatewayv2_route" "get_share" {
  api_id    = aws_apigatewayv2_api.share.id
  route_key = "GET /share/{slug}"
  target    = "integrations/${aws_apigatewayv2_integration.share.id}"
}

resource "aws_lambda_permission" "share_apigw" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.share.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.share.execution_arn}/*/*"
}
