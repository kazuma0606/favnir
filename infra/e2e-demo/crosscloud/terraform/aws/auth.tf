# ── Cognito User Pool ─────────────────────────────────────────────────────────

resource "aws_cognito_user_pool" "crosscloud" {
  name = "favnir-crosscloud-${var.env_suffix}"

  password_policy {
    minimum_length    = 12
    require_uppercase = true
    require_lowercase = true
    require_numbers   = true
    require_symbols   = false
  }

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_cognito_user_pool_client" "crosscloud" {
  name         = "favnir-crosscloud-client"
  user_pool_id = aws_cognito_user_pool.crosscloud.id

  explicit_auth_flows = [
    "ALLOW_USER_PASSWORD_AUTH",
    "ALLOW_REFRESH_TOKEN_AUTH",
  ]

  # デモ用: クライアントシークレットなし（スクリプトから直接呼びやすくするため）
  generate_secret = false
}

# ── HMAC Secret ───────────────────────────────────────────────────────────────

resource "aws_secretsmanager_secret" "hmac_secret" {
  name                    = "favnir/crosscloud/hmac-secret-${var.env_suffix}"
  recovery_window_in_days = 0

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_secretsmanager_secret_version" "hmac_secret" {
  secret_id     = aws_secretsmanager_secret.hmac_secret.id
  secret_string = var.hmac_secret
}

# ── DynamoDB: nonce テーブル（リプレイ防止）──────────────────────────────────

resource "aws_dynamodb_table" "nonce" {
  name         = "favnir-crosscloud-nonce-${var.env_suffix}"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "nonce_id"

  attribute {
    name = "nonce_id"
    type = "S"
  }

  ttl {
    attribute_name = "expires_at"
    enabled        = true
  }

  tags = {
    Project = "favnir-crosscloud"
  }
}

# ── Lambda: verifier ─────────────────────────────────────────────────────────

resource "aws_ecr_repository" "verifier" {
  name                 = "crosscloud-verifier"
  image_tag_mutability = "MUTABLE"
  force_delete         = true

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_iam_role" "lambda_verifier" {
  name = "favnir-crosscloud-lambda-verifier-${var.env_suffix}"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_iam_role_policy" "lambda_verifier_policy" {
  name = "lambda-verifier-policy"
  role = aws_iam_role.lambda_verifier.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["logs:CreateLogGroup", "logs:CreateLogStream", "logs:PutLogEvents"]
        Resource = "arn:aws:logs:*:*:*"
      },
      {
        Effect   = "Allow"
        Action   = ["secretsmanager:GetSecretValue"]
        Resource = [aws_secretsmanager_secret.hmac_secret.arn]
      },
      {
        Effect   = "Allow"
        Action   = ["dynamodb:PutItem"]
        Resource = [aws_dynamodb_table.nonce.arn]
      },
      {
        Effect   = "Allow"
        Action   = ["s3:PutObject"]
        Resource = ["${aws_s3_bucket.proof.arn}/auth-proof/*"]
      },
      {
        Effect   = "Allow"
        Action   = ["ecs:RunTask"]
        Resource = ["*"]
      },
      {
        Effect   = "Allow"
        Action   = ["iam:PassRole"]
        Resource = [
          aws_iam_role.ecs_execution.arn,
          aws_iam_role.ecs_task.arn,
        ]
      }
    ]
  })
}

resource "aws_lambda_function" "verifier" {
  function_name = "favnir-crosscloud-verifier-${var.env_suffix}"
  package_type  = "Image"
  image_uri     = "${aws_ecr_repository.verifier.repository_url}:${var.ecr_image_tag}"
  role          = aws_iam_role.lambda_verifier.arn
  timeout       = 30

  environment {
    variables = {
      HMAC_SECRET_ARN       = aws_secretsmanager_secret.hmac_secret.arn
      NONCE_TABLE           = aws_dynamodb_table.nonce.name
      S3_PROOF_BUCKET       = aws_s3_bucket.proof.id
      ECS_CLUSTER_ARN       = aws_ecs_cluster.crosscloud.arn
      ECS_TASK_DEF_ARN      = aws_ecs_task_definition.migrate.arn
      ECS_SUBNETS           = join(",", tolist(data.aws_subnets.default.ids))
      ECS_SECURITY_GROUP    = aws_security_group.ecs_tasks.id
      AZURE_STORAGE_ACCOUNT = var.azure_storage_account
      AZURE_STORAGE_KEY     = var.azure_storage_key
    }
  }

  tags = {
    Project = "favnir-crosscloud"
  }
}

# ── API Gateway HTTP API ──────────────────────────────────────────────────────

resource "aws_apigatewayv2_api" "crosscloud" {
  name          = "favnir-crosscloud-${var.env_suffix}"
  protocol_type = "HTTP"

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_apigatewayv2_authorizer" "cognito" {
  api_id           = aws_apigatewayv2_api.crosscloud.id
  authorizer_type  = "JWT"
  identity_sources = ["$request.header.Authorization"]
  name             = "cognito-authorizer"

  jwt_configuration {
    audience = [aws_cognito_user_pool_client.crosscloud.id]
    issuer   = "https://cognito-idp.${var.aws_region}.amazonaws.com/${aws_cognito_user_pool.crosscloud.id}"
  }
}

resource "aws_apigatewayv2_integration" "verifier" {
  api_id                 = aws_apigatewayv2_api.crosscloud.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.verifier.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "migrate" {
  api_id             = aws_apigatewayv2_api.crosscloud.id
  route_key          = "POST /migrate"
  authorization_type = "JWT"
  authorizer_id      = aws_apigatewayv2_authorizer.cognito.id
  target             = "integrations/${aws_apigatewayv2_integration.verifier.id}"
}

resource "aws_apigatewayv2_stage" "default" {
  api_id      = aws_apigatewayv2_api.crosscloud.id
  name        = "$default"
  auto_deploy = true

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_lambda_permission" "apigw" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.verifier.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.crosscloud.execution_arn}/*/*"
}
