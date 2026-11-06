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

data "aws_availability_zones" "available" {
  state = "available"

  filter {
    name   = "zone-type"
    values = ["availability-zone"]
  }
}

locals {
  tags = {
    Project = "ToiNoi"
    Env     = var.env
    Service = "network"
    Owner   = var.owner
  }

  name_prefix = "toinoi-${var.env}"
  azs         = slice(data.aws_availability_zones.available.names, 0, 2)

  # Sáu subnet: public cho ALB và NAT, private-app cho service, private-data cho RDS.
  subnets = {
    public-a = { cidr = "10.20.0.0/24", az = local.azs[0], tier = "public" }
    public-b = { cidr = "10.20.1.0/24", az = local.azs[1], tier = "public" }
    app-a    = { cidr = "10.20.10.0/24", az = local.azs[0], tier = "app" }
    app-b    = { cidr = "10.20.11.0/24", az = local.azs[1], tier = "app" }
    data-a   = { cidr = "10.20.20.0/24", az = local.azs[0], tier = "data" }
    data-b   = { cidr = "10.20.21.0/24", az = local.azs[1], tier = "data" }
  }
}

resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_support   = true
  enable_dns_hostnames = true

  tags = {
    Name = "${local.name_prefix}-vpc"
  }
}

resource "aws_subnet" "this" {
  for_each = local.subnets

  vpc_id            = aws_vpc.main.id
  cidr_block        = each.value.cidr
  availability_zone = each.value.az

  tags = {
    Name = "${local.name_prefix}-${each.key}"
    Tier = each.value.tier
  }
}

resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name = "${local.name_prefix}-igw"
  }
}

# NAT Gateway chỉ tạo khi enable_nat_gateway = true: tính tiền theo giờ và theo GB.
resource "aws_eip" "nat" {
  count  = var.enable_nat_gateway ? 1 : 0
  domain = "vpc"

  tags = {
    Name = "${local.name_prefix}-nat-eip"
  }
}

resource "aws_nat_gateway" "main" {
  count         = var.enable_nat_gateway ? 1 : 0
  allocation_id = aws_eip.nat[0].id
  subnet_id     = aws_subnet.this["public-a"].id

  tags = {
    Name = "${local.name_prefix}-nat-a"
  }

  depends_on = [aws_internet_gateway.main]
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }

  tags = {
    Name = "${local.name_prefix}-public"
  }
}

resource "aws_route_table_association" "public" {
  for_each = { for k, v in local.subnets : k => v if v.tier == "public" }

  subnet_id      = aws_subnet.this[each.key].id
  route_table_id = aws_route_table.public.id
}

# Mỗi AZ một route table private-app; có luật ra NAT khi bật NAT Gateway.
resource "aws_route_table" "app" {
  for_each = { for k, v in local.subnets : k => v if v.tier == "app" }

  vpc_id = aws_vpc.main.id

  tags = {
    Name = "${local.name_prefix}-${each.key}"
  }
}

resource "aws_route" "app_to_nat" {
  for_each = var.enable_nat_gateway ? aws_route_table.app : {}

  route_table_id         = each.value.id
  destination_cidr_block = "0.0.0.0/0"
  nat_gateway_id         = aws_nat_gateway.main[0].id
}

resource "aws_route_table_association" "app" {
  for_each = aws_route_table.app

  subnet_id      = aws_subnet.this[each.key].id
  route_table_id = each.value.id
}

# Route table private-data: chỉ có luật local, không có đường ra Internet.
resource "aws_route_table" "data" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name = "${local.name_prefix}-data"
  }
}

resource "aws_route_table_association" "data" {
  for_each = { for k, v in local.subnets : k => v if v.tier == "data" }

  subnet_id      = aws_subnet.this[each.key].id
  route_table_id = aws_route_table.data.id
}

# Security group: sg-alb -> sg-app:8080 -> sg-db:5432.
resource "aws_security_group" "alb" {
  name        = "${local.name_prefix}-alb"
  description = "Toi Noi lab: ALB nhan HTTPS tu Internet"
  vpc_id      = aws_vpc.main.id

  tags = {
    Name = "${local.name_prefix}-alb"
  }
}

resource "aws_security_group" "app" {
  name        = "${local.name_prefix}-app"
  description = "Toi Noi lab: service app, chi nhan tu ALB"
  vpc_id      = aws_vpc.main.id

  tags = {
    Name = "${local.name_prefix}-app"
  }
}

resource "aws_security_group" "db" {
  name        = "${local.name_prefix}-db"
  description = "Toi Noi lab: PostgreSQL, chi nhan tu app"
  vpc_id      = aws_vpc.main.id

  tags = {
    Name = "${local.name_prefix}-db"
  }
}

resource "aws_vpc_security_group_ingress_rule" "alb_https" {
  security_group_id = aws_security_group.alb.id
  cidr_ipv4         = "0.0.0.0/0"
  ip_protocol       = "tcp"
  from_port         = 443
  to_port           = 443
}

resource "aws_vpc_security_group_ingress_rule" "app_from_alb" {
  security_group_id            = aws_security_group.app.id
  referenced_security_group_id = aws_security_group.alb.id
  ip_protocol                  = "tcp"
  from_port                    = 8080
  to_port                      = 8080
}

resource "aws_vpc_security_group_ingress_rule" "db_from_app" {
  security_group_id            = aws_security_group.db.id
  referenced_security_group_id = aws_security_group.app.id
  ip_protocol                  = "tcp"
  from_port                    = 5432
  to_port                      = 5432
}

# Terraform xoá luật egress mặc định của security group; khai báo lại những chiều cần thiết.
resource "aws_vpc_security_group_egress_rule" "alb_to_app" {
  security_group_id            = aws_security_group.alb.id
  referenced_security_group_id = aws_security_group.app.id
  ip_protocol                  = "tcp"
  from_port                    = 8080
  to_port                      = 8080
}

# sg-db cố ý không có luật egress: security group stateful nên gói trả lời cho kết nối vào vẫn đi được.
# RDS chỉ cần egress khi database chủ động gọi ra ngoài (ví dụ extension aws_s3, aws_lambda).
resource "aws_vpc_security_group_egress_rule" "app_all" {
  security_group_id = aws_security_group.app.id
  cidr_ipv4         = "0.0.0.0/0"
  ip_protocol       = "-1"
}

# NACL tuỳ chọn cho subnet private-data (chưa kiểm chứng với RDS ở bài 07).
resource "aws_network_acl" "data" {
  count = var.enable_data_nacl ? 1 : 0

  vpc_id     = aws_vpc.main.id
  subnet_ids = [for k, v in local.subnets : aws_subnet.this[k].id if v.tier == "data"]

  ingress {
    rule_no    = 100
    protocol   = "tcp"
    action     = "allow"
    cidr_block = "10.20.10.0/23"
    from_port  = 5432
    to_port    = 5432
  }

  egress {
    rule_no    = 100
    protocol   = "tcp"
    action     = "allow"
    cidr_block = "10.20.10.0/23"
    from_port  = 1024
    to_port    = 65535
  }

  tags = {
    Name = "${local.name_prefix}-data"
  }
}
