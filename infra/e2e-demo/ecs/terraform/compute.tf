data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"]  # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-amd64-server-*"]
  }
  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

# ── Machine A: Favnir 処理系（Public EC2）────────────────────────────────────

resource "aws_instance" "machine_a" {
  ami                         = data.aws_ami.ubuntu.id
  instance_type               = "t3.micro"
  subnet_id                   = aws_subnet.public.id
  vpc_security_group_ids      = [aws_security_group.machine_a.id]
  iam_instance_profile        = aws_iam_instance_profile.ec2.name
  associate_public_ip_address = true
  key_name                    = var.key_pair_name

  user_data = base64encode(templatefile(
    "${path.module}/../scripts/machine-a-userdata.sh",
    {
      bucket_name  = var.bucket_name
      etl_src      = file("${path.module}/../src/etl.fav")
      pipeline_src = file("${path.module}/../src/pipeline.fav")
    }
  ))

  tags = { Name = "favnir-machine-a" }
}

# ── Machine B: Rust VM のみ（Private EC2）────────────────────────────────────

resource "aws_instance" "machine_b" {
  ami                    = data.aws_ami.ubuntu.id
  instance_type          = "t3.micro"
  subnet_id              = aws_subnet.private.id
  vpc_security_group_ids = [aws_security_group.machine_b.id]
  iam_instance_profile   = aws_iam_instance_profile.ec2.name

  user_data = base64encode(templatefile(
    "${path.module}/../scripts/machine-b-userdata.sh",
    {
      bucket_name = var.bucket_name
    }
  ))

  tags = { Name = "favnir-machine-b" }
}

# ── CloudWatch Logs グループ (EC2 用) ────────────────────────────────────────

resource "aws_cloudwatch_log_group" "machine_a" {
  name              = "/favnir/e2e-demo/machine-a"
  retention_in_days = 7
}

resource "aws_cloudwatch_log_group" "machine_b" {
  name              = "/favnir/e2e-demo/machine-b"
  retention_in_days = 7
}
