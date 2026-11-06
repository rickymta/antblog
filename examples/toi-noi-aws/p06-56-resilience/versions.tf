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

# Control plane của ARC recovery-control-config chỉ có ở us-west-2.
provider "aws" {
  alias  = "arc"
  region = "us-west-2"

  default_tags {
    tags = {
      Project = "ToiNoi"
      Env     = var.env
      Owner   = var.owner
    }
  }
}
