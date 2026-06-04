data "aws_iam_policy_document" "assume_ec2" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "snowflake_e2e" {
  name               = "favnir-snowflake-e2e"
  assume_role_policy = data.aws_iam_policy_document.assume_ec2.json
  description        = "IAM role for Favnir Snowflake e2e demo"
}

resource "aws_iam_role_policy" "s3_proof" {
  name = "s3-proof-write"
  role = aws_iam_role.snowflake_e2e.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = ["s3:PutObject", "s3:GetObject"]
        Resource = "arn:aws:s3:::${var.s3_bucket}/proof/snowflake/*"
      }
    ]
  })
}

resource "aws_iam_instance_profile" "snowflake_e2e" {
  name = "favnir-snowflake-e2e"
  role = aws_iam_role.snowflake_e2e.name
}
