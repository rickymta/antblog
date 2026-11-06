terraform {
  required_version = ">= 1.10"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
  }
}

provider "aws" {
  region  = var.region
  profile = var.aws_profile

  # Mọi resource nhận bộ tag chuẩn của Tới Nơi mà không phải lặp lại từng chỗ.
  default_tags {
    tags = {
      Project = "ToiNoi"
      Env     = var.env
      Service = "platform"
      Owner   = var.owner
    }
  }
}
