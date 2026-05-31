# S3 バケットは ECS 版（favnir-e2e-demo）を共用
# proof/eks/ プレフィックスに EKS 版証跡を保存する
# バケット自体は ECS 版 terraform で管理しているため、ここでは data source で参照のみ

data "aws_s3_bucket" "demo" {
  bucket = var.bucket_name
}
