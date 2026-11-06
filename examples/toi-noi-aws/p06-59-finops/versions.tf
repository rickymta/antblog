terraform {
  required_version = ">= 1.10"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
  }

  # Không khai báo backend thật. Khi dùng thật, xem bài 22 (remote state trên S3 với use_lockfile).
}

# Budgets, Cost Explorer (Cost Anomaly Detection, Cost Categories) là dịch vụ toàn cầu,
# endpoint nằm ở us-east-1; provider dùng region này cho lab.
provider "aws" {
  region = var.region

  default_tags {
    tags = {
      Project = "ToiNoi"
      Env     = var.env
      Owner   = var.owner
    }
  }
}
