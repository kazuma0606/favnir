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

# ── VPC ──────────────────────────────────────────────────────────────────────

resource "aws_vpc" "airgap" {
  cidr_block           = "10.3.0.0/16"
  enable_dns_support   = true
  enable_dns_hostnames = true

  tags = { Name = "favnir-airgap-vpc" }
}

# ── Private Subnet ────────────────────────────────────────────────────────────

resource "aws_subnet" "private_a" {
  vpc_id            = aws_vpc.airgap.id
  cidr_block        = "10.3.1.0/24"
  availability_zone = "${var.aws_region}a"

  tags = { Name = "favnir-airgap-private-a" }
}

# ── Route Table（IGW なし・S3 Gateway のみ）──────────────────────────────────

resource "aws_route_table" "private" {
  vpc_id = aws_vpc.airgap.id
  tags   = { Name = "favnir-airgap-rt-private" }
}

resource "aws_route_table_association" "private_a" {
  subnet_id      = aws_subnet.private_a.id
  route_table_id = aws_route_table.private.id
}

# ── Security Group: EC2 ───────────────────────────────────────────────────────
# アウトバウンドは VPC 内 443 のみ（Endpoints 経由）

resource "aws_security_group" "ec2" {
  name        = "favnir-airgap-ec2"
  description = "Airgap EC2 - VPC endpoint access only"
  vpc_id      = aws_vpc.airgap.id

  egress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = [aws_vpc.airgap.cidr_block]
    description = "HTTPS to VPC interface endpoints (SSM)"
  }

  egress {
    from_port       = 443
    to_port         = 443
    protocol        = "tcp"
    prefix_list_ids = [aws_vpc_endpoint.s3.prefix_list_id]
    description     = "HTTPS to S3 via gateway endpoint"
  }

  tags = { Name = "favnir-airgap-ec2-sg" }
}

# ── Security Group: VPC Endpoints ────────────────────────────────────────────

resource "aws_security_group" "endpoints" {
  name        = "favnir-airgap-endpoints"
  description = "Allow HTTPS from EC2 to VPC endpoints (airgap)"
  vpc_id      = aws_vpc.airgap.id

  ingress {
    from_port       = 443
    to_port         = 443
    protocol        = "tcp"
    security_groups = [aws_security_group.ec2.id]
    description     = "HTTPS from EC2"
  }

  tags = { Name = "favnir-airgap-endpoints-sg" }
}

# ── VPC Endpoint: S3 Gateway（無料）──────────────────────────────────────────

resource "aws_vpc_endpoint" "s3" {
  vpc_id            = aws_vpc.airgap.id
  service_name      = "com.amazonaws.${var.aws_region}.s3"
  vpc_endpoint_type = "Gateway"
  route_table_ids   = [aws_route_table.private.id]

  tags = { Name = "favnir-airgap-s3-endpoint" }
}

# ── VPC Endpoint: SSM Interface（バスチョンレスアクセス用）─────────────────────

resource "aws_vpc_endpoint" "ssm" {
  vpc_id              = aws_vpc.airgap.id
  service_name        = "com.amazonaws.${var.aws_region}.ssm"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private_a.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true

  tags = { Name = "favnir-airgap-ssm-endpoint" }
}

resource "aws_vpc_endpoint" "ssmmessages" {
  vpc_id              = aws_vpc.airgap.id
  service_name        = "com.amazonaws.${var.aws_region}.ssmmessages"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private_a.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true

  tags = { Name = "favnir-airgap-ssmmessages-endpoint" }
}

resource "aws_vpc_endpoint" "ec2messages" {
  vpc_id              = aws_vpc.airgap.id
  service_name        = "com.amazonaws.${var.aws_region}.ec2messages"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private_a.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true

  tags = { Name = "favnir-airgap-ec2messages-endpoint" }
}
