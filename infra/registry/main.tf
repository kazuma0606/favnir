locals {
  name_prefix = "favnir-registry"
}

# ---------------------------------------------------------------------------
# ECR — container image repository
# ---------------------------------------------------------------------------

resource "aws_ecr_repository" "registry" {
  name                 = local.name_prefix
  image_tag_mutability = "MUTABLE"

  image_scanning_configuration {
    scan_on_push = true
  }

  tags = {
    Project     = "favnir"
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}

resource "aws_ecr_lifecycle_policy" "registry" {
  repository = aws_ecr_repository.registry.name

  policy = jsonencode({
    rules = [{
      rulePriority = 1
      description  = "Keep last 5 images"
      selection = {
        tagStatus   = "any"
        countType   = "imageCountMoreThan"
        countNumber = 5
      }
      action = { type = "expire" }
    }]
  })
}

# ---------------------------------------------------------------------------
# DynamoDB — rune metadata table
# ---------------------------------------------------------------------------

resource "aws_dynamodb_table" "registry" {
  name         = "favnir-rune-registry"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "name"

  attribute {
    name = "name"
    type = "S"
  }

  tags = {
    Project     = "favnir"
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}

# ---------------------------------------------------------------------------
# S3 — rune package storage
# ---------------------------------------------------------------------------

resource "aws_s3_bucket" "packages" {
  bucket = "favnir-rune-packages"

  tags = {
    Project     = "favnir"
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}

resource "aws_s3_bucket_public_access_block" "packages" {
  bucket                  = aws_s3_bucket.packages.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "packages" {
  bucket = aws_s3_bucket.packages.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# ---------------------------------------------------------------------------
# IAM — Lambda execution role
# ---------------------------------------------------------------------------

data "aws_iam_policy_document" "lambda_assume_role" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "registry_lambda" {
  name               = "${local.name_prefix}-lambda"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume_role.json

  tags = {
    Project   = "favnir"
    ManagedBy = "terraform"
  }
}

resource "aws_iam_role_policy_attachment" "basic_execution" {
  role       = aws_iam_role.registry_lambda.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

data "aws_iam_policy_document" "registry_access" {
  statement {
    sid    = "DynamoDBAccess"
    effect = "Allow"
    actions = [
      "dynamodb:GetItem",
      "dynamodb:PutItem",
      "dynamodb:DeleteItem",
      "dynamodb:Scan",
      "dynamodb:Query",
    ]
    resources = [aws_dynamodb_table.registry.arn]
  }

  statement {
    sid    = "S3Access"
    effect = "Allow"
    actions = [
      "s3:GetObject",
      "s3:PutObject",
      "s3:DeleteObject",
      "s3:ListBucket",
    ]
    resources = [
      aws_s3_bucket.packages.arn,
      "${aws_s3_bucket.packages.arn}/*",
    ]
  }
}

resource "aws_iam_role_policy" "registry_access" {
  name   = "registry-access"
  role   = aws_iam_role.registry_lambda.id
  policy = data.aws_iam_policy_document.registry_access.json
}

# Lambda function and Function URL are defined in lambda.tf
# Apply lambda.tf after pushing the container image to ECR
