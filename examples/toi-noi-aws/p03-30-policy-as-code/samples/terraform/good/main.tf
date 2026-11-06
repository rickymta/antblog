# BẢN ĐÚNG: cùng ba tài nguyên nhưng qua các rule của Tới Nơi.
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
}

locals {
  tags = {
    Project = "ToiNoi"
    Env     = "lab"
    Service = "tracking"
    Owner   = "platform"
  }
}

data "aws_caller_identity" "current" {}

resource "aws_kms_key" "data" {
  description         = "Khoa du lieu cua lab policy as code"
  enable_key_rotation = true
  tags                = local.tags

  # Key policy tường minh: tài khoản quản lý key qua IAM, không để mặc định ngầm.
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid       = "AccountAdministersKeyViaIam"
      Effect    = "Allow"
      Principal = { AWS = "arn:aws:iam::${data.aws_caller_identity.current.account_id}:root" }
      Action    = "kms:*"
      Resource  = "*"
    }]
  })
}

resource "aws_s3_bucket" "photos" {
  #checkov:skip=CKV_AWS_18:Access log ghi ở bucket log tập trung (bài 54), hạn 2027-06-30
  #checkov:skip=CKV_AWS_144:Replication liên region làm ở bài 36-37 khi có yêu cầu DR, hạn 2027-06-30
  #checkov:skip=CKV2_AWS_62:Ảnh chứng từ chưa cần event notification; bài 36 bổ sung, hạn 2027-06-30
  bucket = "toinoi-lab-delivery-photos-good"
  tags   = local.tags
}

resource "aws_s3_bucket_lifecycle_configuration" "photos" {
  bucket = aws_s3_bucket.photos.id

  rule {
    id     = "cleanup"
    status = "Enabled"

    filter {}

    abort_incomplete_multipart_upload {
      days_after_initiation = 7
    }

    noncurrent_version_expiration {
      noncurrent_days = 90
    }
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "photos" {
  bucket = aws_s3_bucket.photos.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm     = "aws:kms"
      kms_master_key_id = aws_kms_key.data.arn
    }
  }
}

resource "aws_s3_bucket_public_access_block" "photos" {
  bucket                  = aws_s3_bucket.photos.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_versioning" "photos" {
  bucket = aws_s3_bucket.photos.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_security_group" "db" {
  name        = "toinoi-lab-db-good"
  description = "PostgreSQL chi cho subnet ung dung"
  vpc_id      = var.vpc_id
  tags        = local.tags

  ingress {
    description = "PostgreSQL tu subnet ung dung"
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = [var.db_subnet_cidr]
  }
}

resource "aws_db_instance" "trips" {
  #checkov:skip=CKV_AWS_118:Enhanced monitoring cần IAM role riêng; lab dùng metric CloudWatch mặc định (bài 55), hạn 2027-06-30
  #checkov:skip=CKV2_AWS_30:Không log mọi câu lệnh ở 560 UPDATE/giây; dùng log_min_duration_statement trong parameter group (bài 31), hạn 2027-06-30
  identifier                          = "toinoi-lab-trips-good"
  engine                              = "postgres"
  instance_class                      = "db.t4g.micro"
  allocated_storage                   = 20
  username                            = "toinoi"
  manage_master_user_password         = true
  db_subnet_group_name                = var.db_subnet_group_name
  vpc_security_group_ids              = [aws_security_group.db.id]
  publicly_accessible                 = false
  storage_encrypted                   = true
  kms_key_id                          = aws_kms_key.data.arn
  backup_retention_period             = 7
  deletion_protection                 = true
  multi_az                            = true
  auto_minor_version_upgrade          = true
  copy_tags_to_snapshot               = true
  iam_database_authentication_enabled = true
  enabled_cloudwatch_logs_exports     = ["postgresql", "upgrade"]
  performance_insights_enabled        = true
  performance_insights_kms_key_id     = aws_kms_key.data.arn
  skip_final_snapshot                 = true
  tags                                = local.tags
}
