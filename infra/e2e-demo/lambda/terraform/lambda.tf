# ---- Lambda Functions ----

locals {
  ecr_compiler = "${var.aws_account}.dkr.ecr.${var.aws_region}.amazonaws.com/favnir-lambda-compiler"
  ecr_executor = "${var.aws_account}.dkr.ecr.${var.aws_region}.amazonaws.com/favnir-lambda-executor"
}

resource "aws_cloudwatch_log_group" "compiler" {
  name              = "/aws/lambda/favnir-compiler"
  retention_in_days = 7
}

resource "aws_cloudwatch_log_group" "executor" {
  name              = "/aws/lambda/favnir-executor"
  retention_in_days = 7
}

resource "aws_lambda_function" "compiler" {
  function_name = "favnir-compiler"
  role          = aws_iam_role.lambda_compiler.arn
  package_type  = "Image"
  image_uri     = "${local.ecr_compiler}:latest"
  timeout       = 120
  memory_size   = 512

  vpc_config {
    subnet_ids         = [aws_subnet.private_a.id, aws_subnet.private_b.id]
    security_group_ids = [aws_security_group.lambda.id]
  }

  environment {
    variables = {
      BUCKET_NAME   = var.bucket_name
      SQS_QUEUE_URL = aws_sqs_queue.pipeline.url
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.compiler,
    aws_iam_role_policy.lambda_compiler,
  ]

  tags = { Name = "favnir-compiler" }
}

resource "aws_lambda_function" "executor" {
  function_name = "favnir-executor"
  role          = aws_iam_role.lambda_executor.arn
  package_type  = "Image"
  image_uri     = "${local.ecr_executor}:latest"
  timeout       = 300
  memory_size   = 512

  vpc_config {
    subnet_ids         = [aws_subnet.private_a.id, aws_subnet.private_b.id]
    security_group_ids = [aws_security_group.lambda.id]
  }

  environment {
    variables = {
      BUCKET_NAME = var.bucket_name
      DB_URL      = "postgres://${var.db_user}:${var.db_password}@${aws_rds_cluster.demo.endpoint}:5432/${aws_rds_cluster.demo.database_name}"
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.executor,
    aws_iam_role_policy.lambda_executor,
    aws_rds_cluster_instance.demo,
  ]

  tags = { Name = "favnir-executor" }
}

# SQS → Lambda executor のトリガー
resource "aws_lambda_event_source_mapping" "sqs_to_executor" {
  event_source_arn = aws_sqs_queue.pipeline.arn
  function_name    = aws_lambda_function.executor.arn
  batch_size       = 1
  enabled          = true
}
