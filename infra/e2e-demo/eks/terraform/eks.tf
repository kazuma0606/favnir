# ---- EKS Cluster ----

resource "aws_eks_cluster" "demo" {
  name     = "favnir-eks-demo"
  role_arn = aws_iam_role.eks_cluster.arn
  version  = "1.31"

  vpc_config {
    subnet_ids              = [aws_subnet.private_a.id, aws_subnet.private_b.id]
    security_group_ids      = [aws_security_group.eks_nodes.id]
    endpoint_private_access = true
    endpoint_public_access  = true
  }

  enabled_cluster_log_types = ["api", "audit"]

  depends_on = [
    aws_iam_role_policy_attachment.eks_cluster_policy,
    aws_cloudwatch_log_group.eks_cluster,
  ]

  tags = { Name = "favnir-eks-demo" }
}

# ---- Fargate Profiles ----

resource "aws_eks_fargate_profile" "demo" {
  cluster_name           = aws_eks_cluster.demo.name
  fargate_profile_name   = "favnir-demo"
  pod_execution_role_arn = aws_iam_role.fargate_execution.arn
  subnet_ids             = [aws_subnet.private_a.id, aws_subnet.private_b.id]

  selector {
    namespace = "favnir-demo"
  }

  depends_on = [
    aws_iam_role_policy_attachment.fargate_execution,
    aws_eks_cluster.demo,
  ]

  tags = { Name = "favnir-eks-fargate" }
}

# CoreDNS needs a Fargate profile to schedule in Fargate-only clusters
resource "aws_eks_fargate_profile" "kube_system" {
  cluster_name           = aws_eks_cluster.demo.name
  fargate_profile_name   = "kube-system"
  pod_execution_role_arn = aws_iam_role.fargate_execution.arn
  subnet_ids             = [aws_subnet.private_a.id, aws_subnet.private_b.id]

  selector {
    namespace = "kube-system"
  }

  depends_on = [
    aws_iam_role_policy_attachment.fargate_execution,
    aws_eks_cluster.demo,
  ]

  tags = { Name = "favnir-eks-kube-system" }
}

# ---- OIDC Provider（IRSA に必要）----

data "tls_certificate" "eks" {
  url = aws_eks_cluster.demo.identity[0].oidc[0].issuer
}

resource "aws_iam_openid_connect_provider" "eks" {
  client_id_list  = ["sts.amazonaws.com"]
  thumbprint_list = [data.tls_certificate.eks.certificates[0].sha1_fingerprint]
  url             = aws_eks_cluster.demo.identity[0].oidc[0].issuer
}

# ---- CloudWatch Logs ----

resource "aws_cloudwatch_log_group" "eks_cluster" {
  name              = "/aws/eks/favnir-eks-demo/cluster"
  retention_in_days = 7
}

resource "aws_cloudwatch_log_group" "eks_jobs" {
  name              = "/favnir/e2e-demo/eks"
  retention_in_days = 7
}
