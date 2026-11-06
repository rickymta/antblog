# Bucket hoá đơn: versioning bắt buộc, Object Lock, nhân bản sang region thứ hai.
resource "aws_s3_bucket" "invoices" {
  bucket              = local.invoice_bucket
  object_lock_enabled = true

  tags = {
    Service = "billing"
  }
}

resource "aws_s3_bucket_public_access_block" "invoices" {
  bucket                  = aws_s3_bucket.invoices.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_versioning" "invoices" {
  bucket = aws_s3_bucket.invoices.id

  versioning_configuration {
    status = "Enabled"
  }
}

# Retention mặc định áp cho mọi object mới không tự khai retention.
# Lab: GOVERNANCE 1 ngày. Hoá đơn thật: đổi sang years = 10 (và COMPLIANCE nếu pháp chế yêu cầu WORM thật sự).
resource "aws_s3_bucket_object_lock_configuration" "invoices" {
  bucket = aws_s3_bucket.invoices.id

  rule {
    default_retention {
      mode = var.object_lock_mode
      days = var.object_lock_days
    }
  }

  depends_on = [aws_s3_bucket_versioning.invoices]
}

resource "aws_s3_bucket" "replica" {
  provider            = aws.replica
  bucket              = local.replica_bucket
  object_lock_enabled = true

  tags = {
    Service = "billing"
    Role    = "replica"
  }
}

resource "aws_s3_bucket_public_access_block" "replica" {
  provider                = aws.replica
  bucket                  = aws_s3_bucket.replica.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_versioning" "replica" {
  provider = aws.replica
  bucket   = aws_s3_bucket.replica.id

  versioning_configuration {
    status = "Enabled"
  }
}

# Bản sao ở region thứ hai chuyển thẳng xuống lớp lưu lâu dài: không ai đọc trừ khi mất region chính.
resource "aws_s3_bucket_lifecycle_configuration" "replica" {
  provider = aws.replica
  bucket   = aws_s3_bucket.replica.id

  rule {
    id     = "replica-to-glacier-ir"
    status = "Enabled"

    filter {}

    transition {
      days          = 30
      storage_class = "GLACIER_IR"
    }
  }

  depends_on = [aws_s3_bucket_versioning.replica]
}

data "aws_iam_policy_document" "replication_trust" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["s3.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "replication" {
  name               = "toinoi-${var.env}-s3-replication"
  assume_role_policy = data.aws_iam_policy_document.replication_trust.json
}

data "aws_iam_policy_document" "replication" {
  statement {
    sid       = "ReadSourceConfig"
    actions   = ["s3:GetReplicationConfiguration", "s3:ListBucket"]
    resources = [aws_s3_bucket.invoices.arn]
  }

  statement {
    sid = "ReadSourceObjects"
    actions = [
      "s3:GetObjectVersionForReplication",
      "s3:GetObjectVersionAcl",
      "s3:GetObjectVersionTagging",
      "s3:GetObjectRetention",
      "s3:GetObjectLegalHold",
    ]
    resources = ["${aws_s3_bucket.invoices.arn}/*"]
  }

  statement {
    sid = "WriteReplica"
    actions = [
      "s3:ReplicateObject",
      "s3:ReplicateDelete",
      "s3:ReplicateTags",
    ]
    resources = ["${aws_s3_bucket.replica.arn}/*"]
  }
}

resource "aws_iam_role_policy" "replication" {
  name   = "replicate-invoices"
  role   = aws_iam_role.replication.id
  policy = data.aws_iam_policy_document.replication.json
}

resource "aws_s3_bucket_replication_configuration" "invoices" {
  bucket = aws_s3_bucket.invoices.id
  role   = aws_iam_role.replication.arn

  rule {
    id       = "invoices-to-ap-northeast-1"
    status   = "Enabled"
    priority = 1

    filter {
      prefix = "invoices/"
    }

    delete_marker_replication {
      status = "Disabled"
    }

    source_selection_criteria {
      replica_modifications {
        status = "Enabled"
      }
    }

    destination {
      bucket        = aws_s3_bucket.replica.arn
      storage_class = "STANDARD"

      dynamic "replication_time" {
        for_each = var.enable_rtc ? [1] : []
        content {
          status = "Enabled"
          time {
            minutes = 15
          }
        }
      }

      dynamic "metrics" {
        for_each = var.enable_rtc ? [1] : []
        content {
          status = "Enabled"
          event_threshold {
            minutes = 15
          }
        }
      }
    }
  }

  depends_on = [
    aws_s3_bucket_versioning.invoices,
    aws_s3_bucket_versioning.replica,
  ]
}
