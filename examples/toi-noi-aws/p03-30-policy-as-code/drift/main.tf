terraform {
  required_version = ">= 1.10"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
  }

  # Không có backend thật trong lab này.
}

provider "aws" {
  region = var.region

  default_tags {
    tags = {
      Project = "ToiNoi"
      Env     = var.env
      Service = "governance"
      Owner   = var.owner
    }
  }
}

# Điều kiện trước: AWS Config recorder đã bật trong tài khoản (Control Tower bật sẵn).
# Conformance pack gom ba luật phát hiện drift sau apply: S3 mã hoá, RDS không public, đủ tag.
resource "aws_config_conformance_pack" "toinoi_baseline" {
  name          = "toinoi-${var.env}-baseline"
  template_body = file("${path.module}/conformance-pack.yaml")
}
