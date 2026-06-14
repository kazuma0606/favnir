output "bootstrap_brokers_tls" {
  description = "MSK TLS bootstrap brokers"
  value       = aws_msk_cluster.demo.bootstrap_brokers_sasl_scram
}

output "cluster_arn" {
  description = "MSK cluster ARN"
  value       = aws_msk_cluster.demo.arn
}
