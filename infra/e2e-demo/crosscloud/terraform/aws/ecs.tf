# ── ECR ──────────────────────────────────────────────────────────────────────

resource "aws_ecr_repository" "fav" {
  name                 = "crosscloud-fav"
  image_tag_mutability = "MUTABLE"
  force_delete         = true

  tags = {
    Project = "favnir-crosscloud"
  }
}

# ── ECS Cluster ───────────────────────────────────────────────────────────────

resource "aws_ecs_cluster" "crosscloud" {
  name = "favnir-crosscloud"

  tags = {
    Project = "favnir-crosscloud"
  }
}

# ── IAM: ECS Task Execution Role（ECR pull + CloudWatch + Secrets Manager）────

data "aws_iam_policy_document" "ecs_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "ecs_execution" {
  name               = "favnir-crosscloud-ecs-execution"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_iam_role_policy_attachment" "ecs_execution_managed" {
  role       = aws_iam_role.ecs_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_iam_role_policy" "ecs_execution_secrets" {
  name = "ecs-execution-secrets"
  role = aws_iam_role.ecs_execution.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = ["secretsmanager:GetSecretValue"]
      Resource = [
        aws_secretsmanager_secret.rds_conn.arn,
        aws_secretsmanager_secret.azure_conn.arn,
      ]
    }]
  })
}

# ── IAM: ECS Task Role（ランタイム権限）────────────────────────────────────────

resource "aws_iam_role" "ecs_task" {
  name               = "favnir-crosscloud-ecs-task"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json

  tags = {
    Project = "favnir-crosscloud"
  }
}

# ── Security Group: ECS Tasks ─────────────────────────────────────────────────

resource "aws_security_group" "ecs_tasks" {
  name        = "favnir-crosscloud-ecs-tasks-${var.env_suffix}"
  description = "ECS tasks outbound access"
  vpc_id      = data.aws_vpc.default.id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Project = "favnir-crosscloud"
  }
}

# RDS SG に ECS SG からの 5432 を追加許可
resource "aws_security_group_rule" "rds_from_ecs" {
  type                     = "ingress"
  from_port                = 5432
  to_port                  = 5432
  protocol                 = "tcp"
  security_group_id        = aws_security_group.rds.id
  source_security_group_id = aws_security_group.ecs_tasks.id
  description              = "Allow ECS tasks to reach RDS"
}

# ── Secrets Manager: Azure 接続情報（ECS タスクに注入）──────────────────────

resource "aws_secretsmanager_secret" "azure_conn" {
  name                    = "favnir/crosscloud/azure-conn-${var.env_suffix}"
  recovery_window_in_days = 0

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_secretsmanager_secret_version" "azure_conn" {
  secret_id     = aws_secretsmanager_secret.azure_conn.id
  secret_string = var.azure_conn_str
}

# ── CloudWatch Log Group ──────────────────────────────────────────────────────

resource "aws_cloudwatch_log_group" "ecs_migrate" {
  name              = "/ecs/favnir-crosscloud-migrate"
  retention_in_days = 7

  tags = {
    Project = "favnir-crosscloud"
  }
}

# ── ECS Task Definition ───────────────────────────────────────────────────────

resource "aws_ecs_task_definition" "migrate" {
  family                   = "favnir-crosscloud-migrate"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "256"
  memory                   = "512"
  execution_role_arn       = aws_iam_role.ecs_execution.arn
  task_role_arn            = aws_iam_role.ecs_task.arn

  container_definitions = jsonencode([{
    name  = "migrate"
    image = "${aws_ecr_repository.fav.repository_url}:${var.ecr_image_tag}"

    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.ecs_migrate.name
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "migrate"
      }
    }

    secrets = [
      { name = "DATABASE_URL",   valueFrom = aws_secretsmanager_secret.rds_conn.arn },
      { name = "AZURE_CONN_STR", valueFrom = aws_secretsmanager_secret.azure_conn.arn },
    ]

    environment = [
      { name = "AZURE_CONTAINER", value = var.azure_container },
    ]

    # AZURE_STORAGE_ACCOUNT / AZURE_STORAGE_KEY は Lambda verifier が
    # ECS RunTask の containerOverrides で動的に渡す
  }])

  tags = {
    Project = "favnir-crosscloud"
  }
}
