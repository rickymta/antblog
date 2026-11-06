terraform {
  required_version = ">= 1.10"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
  }

  # Không có backend thật trong lab. Khi dùng thật, dùng backend S3 với use_lockfile = true (bài 22).
}

provider "aws" {
  region = var.region

  default_tags {
    tags = local.tags
  }
}
