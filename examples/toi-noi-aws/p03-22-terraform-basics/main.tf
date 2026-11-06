data "aws_caller_identity" "current" {}

data "aws_availability_zones" "available" {
  state = "available"
}

locals {
  prefix = "toinoi-${var.env}"
  azs    = slice(data.aws_availability_zones.available.names, 0, var.az_count)

  # Mỗi AZ có một khối /24 public (0..n-1) và một khối /24 private (10..10+n-1).
  public_cidrs  = { for i, az in local.azs : az => cidrsubnet(var.vpc_cidr, 8, i) }
  private_cidrs = { for i, az in local.azs : az => cidrsubnet(var.vpc_cidr, 8, i + 10) }
}

resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_support   = true
  enable_dns_hostnames = true

  tags = {
    Name = "${local.prefix}-vpc"
  }
}

resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name = "${local.prefix}-igw"
  }
}

resource "aws_subnet" "public" {
  for_each = local.public_cidrs

  vpc_id            = aws_vpc.main.id
  availability_zone = each.key
  cidr_block        = each.value

  tags = {
    Name = "${local.prefix}-public-${each.key}"
    Tier = "public"
  }
}

resource "aws_subnet" "private" {
  for_each = local.private_cidrs

  vpc_id            = aws_vpc.main.id
  availability_zone = each.key
  cidr_block        = each.value

  tags = {
    Name = "${local.prefix}-private-${each.key}"
    Tier = "private"
  }
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }

  tags = {
    Name = "${local.prefix}-public-rt"
  }
}

resource "aws_route_table_association" "public" {
  for_each = aws_subnet.public

  subnet_id      = each.value.id
  route_table_id = aws_route_table.public.id
}

# Route table private không có route ra Internet: lab này không tạo NAT Gateway (tính tiền theo giờ).
resource "aws_route_table" "private" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name = "${local.prefix}-private-rt"
  }
}

resource "aws_route_table_association" "private" {
  for_each = aws_subnet.private

  subnet_id      = each.value.id
  route_table_id = aws_route_table.private.id
}

# Bucket ảnh chứng từ giao hàng của service tracking.
resource "aws_s3_bucket" "delivery_photos" {
  bucket        = "${local.prefix}-delivery-photos-${data.aws_caller_identity.current.account_id}"
  force_destroy = var.force_destroy_photos_bucket

  tags = {
    Service = "tracking"
  }
}

resource "aws_s3_bucket_versioning" "delivery_photos" {
  bucket = aws_s3_bucket.delivery_photos.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "delivery_photos" {
  bucket = aws_s3_bucket.delivery_photos.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "delivery_photos" {
  bucket = aws_s3_bucket.delivery_photos.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}
