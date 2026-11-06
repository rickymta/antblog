data "aws_caller_identity" "current" {}

locals {
  account_id     = data.aws_caller_identity.current.account_id
  photos_bucket  = "toinoi-${var.env}-delivery-photos-${local.account_id}"
  invoice_bucket = "toinoi-${var.env}-invoices-${local.account_id}"
  replica_bucket = "toinoi-${var.env}-invoices-replica-${local.account_id}"
}

# Bucket ảnh chứng từ đã tạo ở bài 06. Ở đây khai báo lại để gắn lifecycle,
# Intelligent-Tiering và event notification; khi dùng thật hãy import bucket cũ.
resource "aws_s3_bucket" "photos" {
  bucket = local.photos_bucket

  tags = {
    Service = "delivery-photo"
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

# Hai rule tách theo prefix: proof/ (ảnh chứng từ) đi Standard -> IA -> Glacier Instant Retrieval,
# logs/ đi thẳng vào Intelligent-Tiering vì mẫu truy cập khó đoán.
resource "aws_s3_bucket_lifecycle_configuration" "photos" {
  bucket = aws_s3_bucket.photos.id

  rule {
    id     = "proof-photos-tiering"
    status = "Enabled"

    # Chỉ áp cho object >= 128 KB: object nhỏ hơn tính như 128 KB ở lớp IA nên chuyển lớp không có lợi.
    filter {
      and {
        prefix                   = "proof/"
        object_size_greater_than = 131072
      }
    }

    transition {
      days          = 30
      storage_class = "STANDARD_IA"
    }

    transition {
      days          = 90
      storage_class = "GLACIER_IR"
    }

    noncurrent_version_expiration {
      noncurrent_days           = 30
      newer_noncurrent_versions = 3
    }

    abort_incomplete_multipart_upload {
      days_after_initiation = 7
    }
  }

  rule {
    id     = "logs-to-intelligent-tiering"
    status = "Enabled"

    filter {
      prefix = "logs/"
    }

    transition {
      days          = 0
      storage_class = "INTELLIGENT_TIERING"
    }

    expiration {
      days = 400
    }
  }

  depends_on = [aws_s3_bucket_versioning.photos]
}

# Cấu hình Intelligent-Tiering với hai tầng archive tuỳ chọn (cần chấp nhận truy xuất chậm).
resource "aws_s3_bucket_intelligent_tiering_configuration" "logs" {
  bucket = aws_s3_bucket.photos.id
  name   = "logs-archive"

  filter {
    prefix = "logs/"
  }

  tiering {
    access_tier = "ARCHIVE_ACCESS"
    days        = 90
  }

  tiering {
    access_tier = "DEEP_ARCHIVE_ACCESS"
    days        = 180
  }
}
