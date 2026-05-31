output "eks_cluster_name" {
  value = aws_eks_cluster.demo.name
}

output "eks_compiler_role_arn" {
  value = aws_iam_role.eks_compiler.arn
}

output "eks_executor_role_arn" {
  value = aws_iam_role.eks_executor.arn
}

output "vpc_id" {
  value = aws_vpc.main.id
}

output "private_subnet_a" {
  value = aws_subnet.private_a.id
}

output "private_subnet_b" {
  value = aws_subnet.private_b.id
}
