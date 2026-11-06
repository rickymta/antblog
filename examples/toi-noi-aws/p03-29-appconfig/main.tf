terraform {
  required_version = ">= 1.10"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
    archive = {
      source  = "hashicorp/archive"
      version = "~> 2.7"
    }
  }

  # Không có backend thật trong lab này (xem bài 22 để dùng backend "s3" với use_lockfile = true).
}

provider "aws" {
  region = var.region

  default_tags {
    tags = local.tags
  }
}

data "aws_caller_identity" "current" {}

locals {
  prefix = "toinoi-${var.env}-quote"

  tags = {
    Project = "ToiNoi"
    Env     = var.env
    Service = "quote"
    Owner   = var.owner
  }
}
