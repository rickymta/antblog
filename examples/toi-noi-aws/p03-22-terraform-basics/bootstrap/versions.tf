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

  default_tags {
    tags = {
      Project = "ToiNoi"
      Env     = "lab"
      Service = "terraform-state"
      Owner   = var.owner
    }
  }
}
