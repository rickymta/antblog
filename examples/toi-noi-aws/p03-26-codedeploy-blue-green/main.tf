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

  # Không có backend thật trong lab này. Khi dùng cho dự án, xem bài 22 để cấu hình
  # backend "s3" với use_lockfile = true.
}

provider "aws" {
  region = var.region

  default_tags {
    tags = local.tags
  }
}

data "aws_caller_identity" "current" {}

locals {
  name = "toinoi-${var.env}-${var.service}"

  tags = {
    Project = "ToiNoi"
    Env     = var.env
    Service = var.service
    Owner   = var.owner
  }
}
