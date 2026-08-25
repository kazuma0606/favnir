terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  # Terraform の制約上 backend ブロックでは変数（var.*）を使用できないため
  # region をリテラルで指定する（infra/snowflake/providers.tf と同じ方針）。
  backend "s3" {
    bucket = "favnir-terraform-state"
    key    = "sap/terraform.tfstate"
    region = "ap-northeast-1"
  }
}

provider "aws" {
  region = var.aws_region
}
