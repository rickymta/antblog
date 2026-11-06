terraform {
  required_version = ">= 1.10"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
  }

  # Không cấu hình backend thật trong lab. Khi dùng nhóm, xem bài 22 về remote state trên S3.
}

provider "aws" {
  region = var.region

  default_tags {
    tags = local.tags
  }
}

locals {
  tags = {
    Project = "ToiNoi"
    Env     = var.env
    Service = "quote"
    Owner   = var.owner
  }

  name_prefix = "toinoi-${var.env}"
  is_arm      = var.architecture == "arm64"
  ami_param   = "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-${var.architecture}"
  instance_type = coalesce(
    var.instance_type,
    local.is_arm ? "t4g.micro" : "t3.micro",
  )
}

# Không truyền subnet_id thì dùng subnet mặc định đầu tiên của default VPC (nhận IPv4 công khai, tính tiền theo giờ).
data "aws_vpc" "default" {
  count   = var.subnet_id == null ? 1 : 0
  default = true
}

data "aws_subnets" "default" {
  count = var.subnet_id == null ? 1 : 0

  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.default[0].id]
  }

  filter {
    name   = "default-for-az"
    values = ["true"]
  }
}

locals {
  subnet_id = var.subnet_id != null ? var.subnet_id : data.aws_subnets.default[0].ids[0]
}

data "aws_subnet" "selected" {
  id = local.subnet_id
}

# AMI Amazon Linux 2023 mới nhất, lấy từ tham số công khai của AWS thay vì dán ID cố định.
data "aws_ssm_parameter" "al2023" {
  name = local.ami_param
}

# Role cho EC2: chỉ có quyền của Session Manager. Không có access key trên máy.
data "aws_iam_policy_document" "ec2_assume" {
  statement {
    sid     = "EC2AssumeRole"
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "quote" {
  name               = "${local.name_prefix}-quote-ec2"
  assume_role_policy = data.aws_iam_policy_document.ec2_assume.json
}

resource "aws_iam_role_policy_attachment" "ssm" {
  role       = aws_iam_role.quote.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

resource "aws_iam_instance_profile" "quote" {
  name = "${local.name_prefix}-quote-ec2"
  role = aws_iam_role.quote.name
}

# Security group không có luật inbound: vào máy bằng Session Manager, không mở cổng 22.
resource "aws_security_group" "quote" {
  name        = "${local.name_prefix}-quote"
  description = "Toi Noi lab: quote, khong co inbound"
  vpc_id      = data.aws_subnet.selected.vpc_id

  tags = {
    Name = "${local.name_prefix}-quote"
  }
}

resource "aws_vpc_security_group_egress_rule" "all" {
  security_group_id = aws_security_group.quote.id
  cidr_ipv4         = "0.0.0.0/0"
  ip_protocol       = "-1"
}

resource "aws_instance" "quote" {
  ami                    = data.aws_ssm_parameter.al2023.value
  instance_type          = local.instance_type
  subnet_id              = local.subnet_id
  vpc_security_group_ids = [aws_security_group.quote.id]
  iam_instance_profile   = aws_iam_instance_profile.quote.name
  user_data              = file("${path.module}/user-data.sh")

  # user-data.sh trong thư mục này là bản sao của ../user-data.sh (phải giữ giống nhau).
  # Đổi user data sẽ thay instance, vì user data chỉ chạy một lần lúc boot.
  user_data_replace_on_change = true

  # IMDSv2 bắt buộc; hop limit 1 để container không với tới metadata của máy.
  metadata_options {
    http_endpoint               = "enabled"
    http_tokens                 = "required"
    http_put_response_hop_limit = 1
  }

  root_block_device {
    volume_type           = "gp3"
    volume_size           = 8
    encrypted             = true
    delete_on_termination = true

    tags = {
      Name = "${local.name_prefix}-quote-root"
    }
  }

  tags = {
    Name = "${local.name_prefix}-quote"
  }

  depends_on = [aws_iam_role_policy_attachment.ssm]
}
