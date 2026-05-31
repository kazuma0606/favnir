resource "aws_ecs_cluster" "demo" {
  name = "favnir-e2e-demo"
  tags = { Name = "favnir-e2e-demo" }
}

resource "aws_cloudwatch_log_group" "ecs" {
  name              = "/favnir/e2e-demo/ecs"
  retention_in_days = 7
}

resource "aws_ecs_task_definition" "etl" {
  family                   = "favnir-etl"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "256"
  memory                   = "512"
  execution_role_arn       = aws_iam_role.ecs_execution.arn
  task_role_arn            = aws_iam_role.ecs_task.arn

  container_definitions = jsonencode([
    # ── Container 1: 証跡収集 ──────────────────────────────────────────────
    # etl-runner より先に完了させる（dependsOn: COMPLETE）
    # .fav ファイルが0件であることを S3 に記録する
    {
      name      = "proof-collector"
      image     = "${var.ecr_uri}:latest"
      essential = false
      command   = [
        "/bin/sh", "-c",
        join(" && ", [
          "TS=$(date +%Y%m%d-%H%M%S)",
          "echo '=== .fav file search (expect: 0 results) ===' > /tmp/proof.txt",
          "find / -name '*.fav' 2>/dev/null >> /tmp/proof.txt",
          "echo '=== /usr/local/bin/ ===' >> /tmp/proof.txt",
          "ls -la /usr/local/bin/ >> /tmp/proof.txt",
          "aws s3 cp /tmp/proof.txt s3://${var.bucket_name}/proof/ecs/fav-search-$TS.txt",
          "echo 'Proof uploaded'"
        ])
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = "/favnir/e2e-demo/ecs"
          "awslogs-region"        = var.aws_region
          "awslogs-stream-prefix" = "proof"
        }
      }
    },
    # ── Container 2: ETL 実行 ──────────────────────────────────────────────
    # proof-collector の完了後に起動
    # S3 から etl.fvc を取得し fav exec で実行
    {
      name      = "etl-runner"
      image     = "${var.ecr_uri}:latest"
      essential = true
      dependsOn = [{
        containerName = "proof-collector"
        condition     = "COMPLETE"
      }]
      command = [
        "/bin/sh", "-c",
        join(" && ", [
          "aws s3 cp s3://${var.bucket_name}/artifacts/etl.fvc /tmp/etl.fvc",
          "echo 'Artifact fetched: etl.fvc'",
          "FAV_DB_URL=$DB_URL BUCKET_NAME=${var.bucket_name} fav exec /tmp/etl.fvc",
          "echo 'ETL complete'"
        ])
      ]
      secrets = [{
        name      = "DB_URL"
        valueFrom = aws_secretsmanager_secret.db_url.arn
      }]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = "/favnir/e2e-demo/ecs"
          "awslogs-region"        = var.aws_region
          "awslogs-stream-prefix" = "etl"
        }
      }
    }
  ])
}
