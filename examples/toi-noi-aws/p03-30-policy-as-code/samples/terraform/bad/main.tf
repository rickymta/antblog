# BẢN SAI CÓ CHỦ ĐÍCH: mỗi khối bên dưới vi phạm ít nhất một rule của pipeline.
# Dùng để thấy công cụ bắt lỗi gì; không bao giờ apply.
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

# Thiếu mã hoá, thiếu chặn public, thiếu tag Project và Env.
resource "aws_s3_bucket" "photos" {
  bucket = "toinoi-lab-delivery-photos-bad"
}

# Mở cổng PostgreSQL cho cả Internet.
resource "aws_security_group" "db" {
  name        = "toinoi-lab-db-bad"
  description = "Database cho phep moi noi"
  vpc_id      = var.vpc_id

  ingress {
    description = "PostgreSQL tu moi dia chi"
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# publicly_accessible = true và storage_encrypted = false.
resource "aws_db_instance" "trips" {
  identifier                  = "toinoi-lab-trips-bad"
  engine                      = "postgres"
  instance_class              = "db.t4g.micro"
  allocated_storage           = 20
  username                    = "toinoi"
  manage_master_user_password = true
  db_subnet_group_name        = var.db_subnet_group_name
  vpc_security_group_ids      = [aws_security_group.db.id]
  publicly_accessible         = true
  storage_encrypted           = false
  skip_final_snapshot         = true
}
