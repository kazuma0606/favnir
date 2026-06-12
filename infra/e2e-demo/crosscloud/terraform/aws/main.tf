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

# ── Network / Security ─────────────────────────────────────────────────────────

data "aws_vpc" "default" {
  default = true
}

data "aws_subnets" "default" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.default.id]
  }
}

resource "aws_security_group" "rds" {
  name        = "favnir-crosscloud-rds-${var.env_suffix}"
  description = "Allow PostgreSQL access for crosscloud demo"
  vpc_id      = data.aws_vpc.default.id

  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "PostgreSQL demo only"
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name    = "favnir-crosscloud-rds-${var.env_suffix}"
    Project = "favnir-crosscloud"
  }
}

# ── RDS ────────────────────────────────────────────────────────────────────────

resource "aws_db_subnet_group" "main" {
  name       = "favnir-crosscloud-${var.env_suffix}"
  subnet_ids = data.aws_subnets.default.ids

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_db_parameter_group" "pg16_no_ssl" {
  name   = "favnir-crosscloud-pg16-${var.env_suffix}"
  family = "postgres16"

  parameter {
    name  = "rds.force_ssl"
    value = "0"
  }

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_db_instance" "source" {
  identifier             = "favnir-crosscloud-src-${var.env_suffix}"
  engine                 = "postgres"
  engine_version         = "16"
  instance_class         = "db.t3.micro"
  allocated_storage      = 20
  db_name                = "appdb"
  username               = "favnir"
  password               = var.rds_password
  parameter_group_name   = aws_db_parameter_group.pg16_no_ssl.name
  db_subnet_group_name   = aws_db_subnet_group.main.name
  vpc_security_group_ids = [aws_security_group.rds.id]
  publicly_accessible    = true
  skip_final_snapshot    = true
  apply_immediately      = true

  tags = {
    Name    = "favnir-crosscloud-source"
    Project = "favnir-crosscloud"
  }
}

# ── S3 証跡バケット ─────────────────────────────────────────────────────────────

resource "aws_s3_bucket" "proof" {
  bucket        = "favnir-crosscloud-proof-${var.env_suffix}"
  force_destroy = true

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_s3_bucket_ownership_controls" "proof" {
  bucket = aws_s3_bucket.proof.id

  rule {
    object_ownership = "BucketOwnerPreferred"
  }
}

resource "aws_s3_bucket_public_access_block" "proof" {
  bucket = aws_s3_bucket.proof.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# ── Secrets Manager (RDS 接続文字列) ───────────────────────────────────────────

resource "aws_secretsmanager_secret" "rds_conn" {
  name                    = "favnir/crosscloud/rds-conn-${var.env_suffix}"
  recovery_window_in_days = 0

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "aws_secretsmanager_secret_version" "rds_conn" {
  secret_id = aws_secretsmanager_secret.rds_conn.id
  secret_string = "host=${aws_db_instance.source.address} port=5432 user=favnir password=${var.rds_password} dbname=appdb sslmode=disable"
}
