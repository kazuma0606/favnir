# Blue/Green deployment infrastructure stub
# Managed by: fav deploy --strategy blue-green
# TODO: add aws provider block before running terraform plan/apply

variable "env" {
  description = "Target environment (dev/staging/prod)"
  type        = string
  default     = "dev"
}

locals {
  blue_slot  = "${var.env}-blue"
  green_slot = "${var.env}-green"
}

output "blue_slot" {
  value = local.blue_slot
}

output "green_slot" {
  value = local.green_slot
}
