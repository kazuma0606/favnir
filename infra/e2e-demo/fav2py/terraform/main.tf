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

# ── VPC ───────────────────────────────────────────────────────────────────────

resource "aws_vpc" "fav2py" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true
  tags = { Name = "fav2py" }
}

resource "aws_subnet" "public" {
  vpc_id                  = aws_vpc.fav2py.id
  cidr_block              = "10.0.1.0/24"
  availability_zone       = "${var.aws_region}a"
  map_public_ip_on_launch = true
  tags = { Name = "fav2py-public" }
}

resource "aws_subnet" "private" {
  vpc_id            = aws_vpc.fav2py.id
  cidr_block        = "10.0.2.0/24"
  availability_zone = "${var.aws_region}a"
  tags = { Name = "fav2py-private" }
}

# RDS には 2 AZ のサブネットグループが必要
resource "aws_subnet" "private_b" {
  vpc_id            = aws_vpc.fav2py.id
  cidr_block        = "10.0.3.0/24"
  availability_zone = "${var.aws_region}c"
  tags = { Name = "fav2py-private-b" }
}

resource "aws_internet_gateway" "igw" {
  vpc_id = aws_vpc.fav2py.id
  tags   = { Name = "fav2py-igw" }
}

resource "aws_eip" "nat" {
  domain = "vpc"
}

resource "aws_nat_gateway" "nat" {
  allocation_id = aws_eip.nat.id
  subnet_id     = aws_subnet.public.id
  tags          = { Name = "fav2py-nat" }
  depends_on    = [aws_internet_gateway.igw]
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.fav2py.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.igw.id
  }
  tags = { Name = "fav2py-public-rt" }
}

resource "aws_route_table_association" "public" {
  subnet_id      = aws_subnet.public.id
  route_table_id = aws_route_table.public.id
}

resource "aws_route_table" "private" {
  vpc_id = aws_vpc.fav2py.id
  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.nat.id
  }
  tags = { Name = "fav2py-private-rt" }
}

resource "aws_route_table_association" "private" {
  subnet_id      = aws_subnet.private.id
  route_table_id = aws_route_table.private.id
}

resource "aws_route_table_association" "private_b" {
  subnet_id      = aws_subnet.private_b.id
  route_table_id = aws_route_table.private.id
}

# ── Security Groups ───────────────────────────────────────────────────────────

resource "aws_security_group" "rds" {
  name   = "fav2py-rds"
  vpc_id = aws_vpc.fav2py.id

  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.ecs.id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = { Name = "fav2py-rds" }
}

resource "aws_security_group" "ecs" {
  name   = "fav2py-ecs"
  vpc_id = aws_vpc.fav2py.id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = { Name = "fav2py-ecs" }
}

# ── RDS PostgreSQL ─────────────────────────────────────────────────────────────

resource "aws_db_subnet_group" "fav2py" {
  name       = "fav2py"
  subnet_ids = [aws_subnet.private.id, aws_subnet.private_b.id]
  tags       = { Name = "fav2py" }
}

resource "aws_db_parameter_group" "fav2py" {
  name   = "fav2py-pg16"
  family = "postgres16"

  parameter {
    name         = "rds.force_ssl"
    value        = "0"
    apply_method = "immediate"
  }

  tags = { Name = "fav2py" }
}

resource "aws_db_instance" "postgres" {
  identifier             = "fav2py"
  engine                 = "postgres"
  engine_version         = "16"
  instance_class         = "db.t3.micro"
  allocated_storage      = 20
  db_name                = "fav2py"
  username               = "favnir"
  password               = var.db_password
  db_subnet_group_name   = aws_db_subnet_group.fav2py.name
  vpc_security_group_ids = [aws_security_group.rds.id]
  parameter_group_name   = aws_db_parameter_group.fav2py.name
  skip_final_snapshot    = true
  publicly_accessible    = false
  apply_immediately      = true
  tags                   = { Name = "fav2py" }
}

# ── ECR ────────────────────────────────────────────────────────────────────────

resource "aws_ecr_repository" "fav2py" {
  name                 = "favnir/fav2py"
  image_tag_mutability = "MUTABLE"
  tags                 = { Name = "fav2py" }
}

# ── CloudWatch Logs ────────────────────────────────────────────────────────────

resource "aws_cloudwatch_log_group" "fav2py" {
  name              = "/ecs/fav2py"
  retention_in_days = 7
}

# ── ECS Cluster ────────────────────────────────────────────────────────────────

resource "aws_ecs_cluster" "fav2py" {
  name = "fav2py"
  tags = { Name = "fav2py" }
}

locals {
  db_url = "postgresql://favnir:${var.db_password}@${aws_db_instance.postgres.address}/fav2py"
  common_env = [
    { name = "DATABASE_URL", value = local.db_url },
    { name = "AWS_DEFAULT_REGION", value = var.aws_region },
    { name = "S3_BUCKET", value = var.s3_bucket },
  ]
  log_config = {
    logDriver = "awslogs"
    options = {
      awslogs-group         = aws_cloudwatch_log_group.fav2py.name
      awslogs-region        = var.aws_region
      awslogs-stream-prefix = "ecs"
    }
  }
}

# ── ECS Task Definition: fav-native ───────────────────────────────────────────

resource "aws_ecs_task_definition" "native" {
  family                   = "fav2py-native"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "512"
  memory                   = "1024"
  execution_role_arn       = aws_iam_role.ecs_execution.arn
  task_role_arn            = aws_iam_role.ecs_task.arn

  container_definitions = jsonencode([{
    name        = "fav-native"
    image       = "${aws_ecr_repository.fav2py.repository_url}:${var.fav_image_tag}"
    command     = ["fav", "run", "--legacy", "/app/pipeline.fav", "--", "/app/sample.csv"]
    environment = local.common_env
    logConfiguration = local.log_config
  }])
}

# ── ECS Task Definition: fav-python ───────────────────────────────────────────

resource "aws_ecs_task_definition" "python" {
  family                   = "fav2py-python"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "512"
  memory                   = "1024"
  execution_role_arn       = aws_iam_role.ecs_execution.arn
  task_role_arn            = aws_iam_role.ecs_task.arn

  container_definitions = jsonencode([{
    name    = "fav-python"
    image   = "${aws_ecr_repository.fav2py.repository_url}:${var.fav_image_tag}"
    command = [
      "python3", "/app/python/main.py", "/app/sample.csv"
    ]
    environment      = local.common_env
    logConfiguration = local.log_config
  }])
}
