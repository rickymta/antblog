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
  region = var.region

  default_tags {
    tags = {
      Project = "ToiNoi"
      Env     = var.env
      Service = "edge"
      Owner   = var.owner
    }
  }
}

# WAF scope CLOUDFRONT và ACM cho CloudFront chỉ tạo được ở us-east-1.
provider "aws" {
  alias  = "use1"
  region = "us-east-1"

  default_tags {
    tags = {
      Project = "ToiNoi"
      Env     = var.env
      Service = "edge"
      Owner   = var.owner
    }
  }
}
