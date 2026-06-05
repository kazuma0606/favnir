output "instance_id" {
  description = "EC2 instance ID"
  value       = aws_instance.favnir_ec2.id
}

output "instance_private_ip" {
  description = "EC2 private IP (no public IP — airgap)"
  value       = aws_instance.favnir_ec2.private_ip
}

output "vpc_id" {
  description = "VPC ID"
  value       = aws_vpc.airgap.id
}

output "s3_proof_prefix" {
  description = "S3 path for proof files"
  value       = "s3://${var.bucket_name}/airgap/proof/"
}

output "s3_output_key" {
  description = "S3 path for ETL output"
  value       = "s3://${var.bucket_name}/airgap/output/summary.json"
}
