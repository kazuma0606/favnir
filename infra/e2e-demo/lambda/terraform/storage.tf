# 既存の S3 バケットを参照（ECS/EKS 版と共用）
data "aws_s3_bucket" "demo" {
  bucket = var.bucket_name
}
