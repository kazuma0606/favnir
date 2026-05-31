resource "aws_s3_bucket" "demo" {
  bucket        = var.bucket_name
  force_destroy = true
  tags          = { Name = "favnir-e2e-demo" }
}

resource "aws_s3_bucket_public_access_block" "demo" {
  bucket                  = aws_s3_bucket.demo.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_versioning" "demo" {
  bucket = aws_s3_bucket.demo.id
  versioning_configuration {
    status = "Enabled"
  }
}

# バケットポリシー: EC2 Instance Profile と ECS Task Role のみアクセス許可
resource "aws_s3_bucket_policy" "demo" {
  bucket = aws_s3_bucket.demo.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid       = "AllowEC2Role"
        Effect    = "Allow"
        Principal = { AWS = aws_iam_role.ec2.arn }
        Action    = ["s3:GetObject", "s3:PutObject", "s3:ListBucket"]
        Resource  = [aws_s3_bucket.demo.arn, "${aws_s3_bucket.demo.arn}/*"]
      },
      {
        Sid       = "AllowECSTaskRole"
        Effect    = "Allow"
        Principal = { AWS = aws_iam_role.ecs_task.arn }
        Action    = ["s3:GetObject", "s3:PutObject", "s3:ListBucket"]
        Resource  = [aws_s3_bucket.demo.arn, "${aws_s3_bucket.demo.arn}/*"]
      }
    ]
  })
}

# ── Secrets Manager: RDS 接続情報 ──────────────────────────────────────────────

resource "aws_secretsmanager_secret" "db_url" {
  name                    = "favnir-demo-db-url"
  recovery_window_in_days = 0  # デモ用: 即削除可能
}

resource "aws_secretsmanager_secret_version" "db_url" {
  secret_id     = aws_secretsmanager_secret.db_url.id
  secret_string = "postgres://${var.db_user}:${var.db_password}@${aws_rds_cluster.demo.endpoint}/demo"
}
