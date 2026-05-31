# ---- SQS ----

resource "aws_sqs_queue" "dlq" {
  name                      = "favnir-pipeline-dlq"
  message_retention_seconds = 86400
  tags = { Name = "favnir-pipeline-dlq" }
}

resource "aws_sqs_queue" "pipeline" {
  name                       = "favnir-pipeline"
  visibility_timeout_seconds = 300
  message_retention_seconds  = 3600

  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.dlq.arn
    maxReceiveCount     = 3
  })

  tags = { Name = "favnir-pipeline" }
}
