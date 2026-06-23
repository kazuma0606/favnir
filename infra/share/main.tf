# ── S3 バケット ────────────────────────────────────────────────────────────────

resource "aws_s3_bucket" "shares" {
  bucket = "favnir-playground-shares-${var.environment}"

  tags = {
    Project     = "favnir"
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}

# Public Access Block（必須 — コードを公開しない）
resource "aws_s3_bucket_public_access_block" "shares" {
  bucket = aws_s3_bucket.shares.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# サーバーサイド暗号化（AES256）
resource "aws_s3_bucket_server_side_encryption_configuration" "shares" {
  bucket = aws_s3_bucket.shares.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# Lifecycle: 90日 TTL
resource "aws_s3_bucket_lifecycle_configuration" "shares_ttl" {
  bucket = aws_s3_bucket.shares.id

  rule {
    id     = "expire-shares"
    status = "Enabled"

    filter {
      prefix = "shares/"
    }

    expiration {
      days = 90
    }
  }
}
