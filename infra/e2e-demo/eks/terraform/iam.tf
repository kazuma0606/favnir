locals {
  oidc_issuer = replace(aws_eks_cluster.demo.identity[0].oidc[0].issuer, "https://", "")
}

# ---- EKS Cluster Role ----

resource "aws_iam_role" "eks_cluster" {
  name = "favnir-eks-cluster-role"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "eks.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy_attachment" "eks_cluster_policy" {
  role       = aws_iam_role.eks_cluster.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSClusterPolicy"
}

# ---- Fargate Execution Role ----

resource "aws_iam_role" "fargate_execution" {
  name = "favnir-fargate-execution-role"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "eks-fargate-pods.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy_attachment" "fargate_execution" {
  role       = aws_iam_role.fargate_execution.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSFargatePodExecutionRolePolicy"
}

# CloudWatch Logs 書き込み（Fargate Pod ログ用）
resource "aws_iam_role_policy" "fargate_execution_logs" {
  name = "fargate-execution-logs"
  role = aws_iam_role.fargate_execution.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = [
        "logs:CreateLogStream",
        "logs:CreateLogGroup",
        "logs:PutLogEvents",
        "logs:DescribeLogStreams",
      ]
      Resource = "arn:aws:logs:*:*:*"
    }]
  })
}

# ---- IRSA: Compiler Job ----

resource "aws_iam_role" "eks_compiler" {
  name = "favnir-eks-compiler"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Federated = aws_iam_openid_connect_provider.eks.arn }
      Action    = "sts:AssumeRoleWithWebIdentity"
      Condition = {
        StringEquals = {
          "${local.oidc_issuer}:sub" = "system:serviceaccount:favnir-demo:favnir-compiler-sa"
          "${local.oidc_issuer}:aud" = "sts.amazonaws.com"
        }
      }
    }]
  })
}

resource "aws_iam_role_policy" "eks_compiler_s3" {
  name = "favnir-eks-compiler-s3"
  role = aws_iam_role.eks_compiler.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = ["s3:PutObject", "s3:GetObject", "s3:ListBucket"]
      Resource = [
        "arn:aws:s3:::${var.bucket_name}",
        "arn:aws:s3:::${var.bucket_name}/artifacts/*",
        "arn:aws:s3:::${var.bucket_name}/proof/eks/*",
      ]
    }]
  })
}

# ---- IRSA: Executor Job ----

resource "aws_iam_role" "eks_executor" {
  name = "favnir-eks-executor"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Federated = aws_iam_openid_connect_provider.eks.arn }
      Action    = "sts:AssumeRoleWithWebIdentity"
      Condition = {
        StringEquals = {
          "${local.oidc_issuer}:sub" = "system:serviceaccount:favnir-demo:favnir-executor-sa"
          "${local.oidc_issuer}:aud" = "sts.amazonaws.com"
        }
      }
    }]
  })
}

resource "aws_iam_role_policy" "eks_executor_s3" {
  name = "favnir-eks-executor-s3"
  role = aws_iam_role.eks_executor.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = ["s3:GetObject", "s3:ListBucket"]
        Resource = [
          "arn:aws:s3:::${var.bucket_name}",
          "arn:aws:s3:::${var.bucket_name}/artifacts/*",
        ]
      },
      {
        Effect = "Allow"
        Action = ["s3:PutObject"]
        Resource = [
          "arn:aws:s3:::${var.bucket_name}/output/*",
          "arn:aws:s3:::${var.bucket_name}/proof/eks/*",
        ]
      }
    ]
  })
}
