# ── EC2 Instance Role ─────────────────────────────────────────────────────────

resource "aws_iam_role" "ec2_role" {
  name = "favnir-airgap-ec2-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "ec2.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

# S3: バイナリ・ソース・データ の読み取り + 出力・証跡 の書き込み

resource "aws_iam_role_policy" "s3_policy" {
  name = "favnir-airgap-s3"
  role = aws_iam_role.ec2_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = ["s3:GetObject"]
        Resource = [
          "arn:aws:s3:::${var.bucket_name}/airgap/binary/*",
          "arn:aws:s3:::${var.bucket_name}/airgap/src/*",
          "arn:aws:s3:::${var.bucket_name}/airgap/data/*",
        ]
      },
      {
        Effect = "Allow"
        Action = ["s3:PutObject"]
        Resource = [
          "arn:aws:s3:::${var.bucket_name}/airgap/output/*",
          "arn:aws:s3:::${var.bucket_name}/airgap/proof/*",
        ]
      }
    ]
  })
}

# SSM Session Manager（バスチョンレスアクセス用）

resource "aws_iam_role_policy_attachment" "ssm" {
  role       = aws_iam_role.ec2_role.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

resource "aws_iam_instance_profile" "ec2_profile" {
  name = "favnir-airgap-ec2-profile"
  role = aws_iam_role.ec2_role.name
}
