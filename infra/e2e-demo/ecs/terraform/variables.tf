variable "aws_region" {
  description = "AWS リージョン"
  type        = string
  default     = "ap-northeast-1"
}

variable "bucket_name" {
  description = "S3 バケット名（グローバルで一意である必要あり）"
  type        = string
  default     = "favnir-e2e-demo"
}

variable "my_ip_cidr" {
  description = "Machine A への SSH を許可する開発者 IP（例: 203.0.113.1/32）"
  type        = string
}

variable "key_pair_name" {
  description = "Machine A に使用する EC2 Key Pair 名"
  type        = string
}

variable "db_user" {
  description = "RDS 管理者ユーザー名"
  type        = string
  default     = "favnir"
}

variable "db_password" {
  description = "RDS 管理者パスワード"
  type        = string
  sensitive   = true
}

variable "ecr_uri" {
  description = "favnir/runtime ECR イメージ URI（アカウント番号込み、タグなし）"
  type        = string
  # 例: 123456789012.dkr.ecr.ap-northeast-1.amazonaws.com/favnir-runtime
}
