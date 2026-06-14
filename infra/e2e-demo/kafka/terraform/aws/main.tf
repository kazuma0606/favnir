terraform {
  required_version = ">= 1.5"
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

# ── VPC ──────────────────────────────────────────────────────────────────────
resource "aws_vpc" "main" {
  cidr_block = "10.0.0.0/16"
  tags = { Name = "${var.prefix}-vpc" }
}

resource "aws_subnet" "private_a" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.0.1.0/24"
  availability_zone = "${var.aws_region}a"
  tags = { Name = "${var.prefix}-subnet-a" }
}

resource "aws_subnet" "private_b" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.0.2.0/24"
  availability_zone = "${var.aws_region}b"
  tags = { Name = "${var.prefix}-subnet-b" }
}

# ── Security Group ────────────────────────────────────────────────────────────
resource "aws_security_group" "msk" {
  name   = "${var.prefix}-msk-sg"
  vpc_id = aws_vpc.main.id

  ingress {
    from_port   = 9092
    to_port     = 9096
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/8"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# ── MSK Cluster ──────────────────────────────────────────────────────────────
resource "aws_msk_cluster" "demo" {
  cluster_name           = "${var.prefix}-cluster"
  kafka_version          = "3.6.0"
  number_of_broker_nodes = 2

  broker_node_group_info {
    instance_type   = "kafka.t3.small"
    client_subnets  = [aws_subnet.private_a.id, aws_subnet.private_b.id]
    security_groups = [aws_security_group.msk.id]

    storage_info {
      ebs_storage_info { volume_size = 10 }
    }
  }

  client_authentication {
    sasl {
      scram = true
    }
  }

  encryption_info {
    encryption_in_transit {
      client_broker = "TLS"
      in_cluster    = true
    }
  }

  tags = { Project = var.prefix }
}

# ── Secrets Manager (SASL credentials) ───────────────────────────────────────
resource "aws_secretsmanager_secret" "msk_creds" {
  name = "${var.prefix}/msk/credentials"
}

resource "aws_secretsmanager_secret_version" "msk_creds" {
  secret_id     = aws_secretsmanager_secret.msk_creds.id
  secret_string = jsonencode({
    username = var.msk_username
    password = var.msk_password
  })
}
