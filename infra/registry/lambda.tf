# ---------------------------------------------------------------------------
# Lambda function (container image)
# Apply AFTER pushing the first image to ECR:
#   terraform apply -target=aws_lambda_function.registry
#   terraform apply -target=aws_apigatewayv2_api.registry
# CI/CD will keep the image updated via aws lambda update-function-code
# ---------------------------------------------------------------------------

resource "aws_lambda_function" "registry" {
  function_name = local.name_prefix
  role          = aws_iam_role.registry_lambda.arn
  package_type  = "Image"
  image_uri     = "${aws_ecr_repository.registry.repository_url}:latest"
  timeout       = 30
  memory_size   = 512

  tags = {
    Project     = "favnir"
    Environment = var.environment
    ManagedBy   = "terraform"
  }

  lifecycle {
    ignore_changes = [image_uri]
  }
}

# ---------------------------------------------------------------------------
# API Gateway HTTP API — public HTTPS endpoint (auth handled in Favnir)
# ---------------------------------------------------------------------------

resource "aws_apigatewayv2_api" "registry" {
  name          = local.name_prefix
  protocol_type = "HTTP"

  cors_configuration {
    allow_origins = ["*"]
    allow_methods = ["GET", "POST"]
    allow_headers = ["content-type", "authorization"]
  }
}

resource "aws_apigatewayv2_integration" "registry" {
  api_id                 = aws_apigatewayv2_api.registry.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.registry.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "registry" {
  api_id    = aws_apigatewayv2_api.registry.id
  route_key = "$default"
  target    = "integrations/${aws_apigatewayv2_integration.registry.id}"
}

resource "aws_apigatewayv2_stage" "registry" {
  api_id      = aws_apigatewayv2_api.registry.id
  name        = "$default"
  auto_deploy = true
}

resource "aws_lambda_permission" "allow_apigw" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.registry.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.registry.execution_arn}/*/*"
}
