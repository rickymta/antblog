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

# Region chính: nơi có bucket nguồn.
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

# Region thứ hai: nơi đặt bản sao hoá đơn (Cross-Region Replication).
provider "aws" {
  alias  = "replica"
  region = var.replica_region

  default_tags {
    tags = {
      Project = "ToiNoi"
      Env     = var.env
      Owner   = var.owner
    }
  }
}
