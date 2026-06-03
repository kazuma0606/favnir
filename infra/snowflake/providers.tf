terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    snowflake = {
      source  = "Snowflake-Labs/snowflake"
      version = "~> 0.87"
    }
  }
  backend "s3" {
    bucket = "favnir-terraform-state"
    key    = "snowflake/terraform.tfstate"
    region = "ap-northeast-1"
  }
}

provider "aws" {
  region = var.aws_region
}

provider "snowflake" {
  account = var.snowflake_account
  user    = var.snowflake_user
  role    = var.snowflake_admin_role
}
