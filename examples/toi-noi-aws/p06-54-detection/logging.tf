data "aws_caller_identity" "current" {}

locals {
  account_id = data.aws_caller_identity.current.account_id
  trail_name = "toinoi-${var.env}-trail"
  trail_arn  = "arn:aws:cloudtrail:${var.region}:${local.account_id}:trail/${local.trail_name}"
}

# ---------------------------------------------------------------------------
# Bucket log: ở hệ thống thật nằm trong tài khoản log archive (bài 28)
# ---------------------------------------------------------------------------
resource "aws_s3_bucket" "logs" {
  bucket        = "toinoi-${var.env}-security-logs-${local.account_id}"
  force_destroy = true # lab: destroy được kể cả khi còn log

  tags = {
    Service = "security"
  }
}

resource "aws_s3_bucket_public_access_block" "logs" {
  bucket = aws_s3_bucket.logs.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_versioning" "logs" {
  bucket = aws_s3_bucket.logs.id

  versioning_configuration {
    status = "Enabled"
  }
}

data "aws_iam_policy_document" "logs_bucket" {
  statement {
    sid       = "CloudTrailAclCheck"
    effect    = "Allow"
    actions   = ["s3:GetBucketAcl"]
    resources = [aws_s3_bucket.logs.arn]

    principals {
      type        = "Service"
      identifiers = ["cloudtrail.amazonaws.com"]
    }

    condition {
      test     = "StringEquals"
      variable = "aws:SourceArn"
      values   = [local.trail_arn]
    }
  }

  statement {
    sid       = "CloudTrailWrite"
    effect    = "Allow"
    actions   = ["s3:PutObject"]
    resources = ["${aws_s3_bucket.logs.arn}/AWSLogs/${local.account_id}/*"]

    principals {
      type        = "Service"
      identifiers = ["cloudtrail.amazonaws.com"]
    }

    condition {
      test     = "StringEquals"
      variable = "s3:x-amz-acl"
      values   = ["bucket-owner-full-control"]
    }

    condition {
      test     = "StringEquals"
      variable = "aws:SourceArn"
      values   = [local.trail_arn]
    }
  }

  statement {
    sid       = "ConfigAclCheck"
    effect    = "Allow"
    actions   = ["s3:GetBucketAcl", "s3:ListBucket"]
    resources = [aws_s3_bucket.logs.arn]

    principals {
      type        = "Service"
      identifiers = ["config.amazonaws.com"]
    }

    condition {
      test     = "StringEquals"
      variable = "aws:SourceAccount"
      values   = [local.account_id]
    }
  }

  statement {
    sid       = "ConfigWrite"
    effect    = "Allow"
    actions   = ["s3:PutObject"]
    resources = ["${aws_s3_bucket.logs.arn}/config/AWSLogs/${local.account_id}/Config/*"]

    principals {
      type        = "Service"
      identifiers = ["config.amazonaws.com"]
    }

    condition {
      test     = "StringEquals"
      variable = "s3:x-amz-acl"
      values   = ["bucket-owner-full-control"]
    }

    condition {
      test     = "StringEquals"
      variable = "aws:SourceAccount"
      values   = [local.account_id]
    }
  }

  statement {
    sid       = "DenyPlainHttp"
    effect    = "Deny"
    actions   = ["s3:*"]
    resources = [aws_s3_bucket.logs.arn, "${aws_s3_bucket.logs.arn}/*"]

    principals {
      type        = "*"
      identifiers = ["*"]
    }

    condition {
      test     = "Bool"
      variable = "aws:SecureTransport"
      values   = ["false"]
    }
  }
}

resource "aws_s3_bucket_policy" "logs" {
  bucket = aws_s3_bucket.logs.id
  policy = data.aws_iam_policy_document.logs_bucket.json

  depends_on = [aws_s3_bucket_public_access_block.logs]
}

# ---------------------------------------------------------------------------
# CloudTrail: management event + data event S3, log file validation, đẩy sang CloudWatch Logs
# ---------------------------------------------------------------------------
resource "aws_cloudwatch_log_group" "trail" {
  name              = "/toinoi/${var.env}/cloudtrail"
  retention_in_days = 90

  tags = {
    Service = "security"
  }
}

data "aws_iam_policy_document" "trail_assume" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["cloudtrail.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "trail_to_logs" {
  statement {
    effect    = "Allow"
    actions   = ["logs:CreateLogStream", "logs:PutLogEvents"]
    resources = ["${aws_cloudwatch_log_group.trail.arn}:*"]
  }
}

resource "aws_iam_role" "trail_to_logs" {
  name               = "toinoi-${var.env}-cloudtrail-to-logs"
  assume_role_policy = data.aws_iam_policy_document.trail_assume.json

  tags = {
    Service = "security"
  }
}

resource "aws_iam_role_policy" "trail_to_logs" {
  name   = "write-trail-logs"
  role   = aws_iam_role.trail_to_logs.id
  policy = data.aws_iam_policy_document.trail_to_logs.json
}

resource "aws_cloudtrail" "main" {
  name                          = local.trail_name
  s3_bucket_name                = aws_s3_bucket.logs.id
  is_multi_region_trail         = true
  is_organization_trail         = var.is_organization_trail
  include_global_service_events = true
  enable_log_file_validation    = true
  cloud_watch_logs_group_arn    = "${aws_cloudwatch_log_group.trail.arn}:*"
  cloud_watch_logs_role_arn     = aws_iam_role.trail_to_logs.arn

  advanced_event_selector {
    name = "Management events"

    field_selector {
      field  = "eventCategory"
      equals = ["Management"]
    }
  }

  # Data event S3: bật theo bucket nhạy cảm, không bật cho mọi bucket vì tính phí theo số event.
  dynamic "advanced_event_selector" {
    for_each = length(var.data_event_bucket_arns) > 0 ? [1] : []

    content {
      name = "S3 object events on sensitive buckets"

      field_selector {
        field  = "eventCategory"
        equals = ["Data"]
      }

      field_selector {
        field  = "resources.type"
        equals = ["AWS::S3::Object"]
      }

      field_selector {
        field       = "resources.ARN"
        starts_with = [for arn in var.data_event_bucket_arns : "${arn}/"]
      }
    }
  }

  tags = {
    Service = "security"
  }

  depends_on = [aws_s3_bucket_policy.logs, aws_iam_role_policy.trail_to_logs]
}

# CloudTrail Lake (tuỳ chọn, tính phí theo dung lượng): ĐÓNG với khách mới từ 31/05/2026.
# Chỉ bật nếu tài khoản đã dùng Lake trước ngày đó; tài khoản mới truy vấn log group của trail
# bằng CloudWatch Logs Insights (queries/*.logsinsights).
resource "aws_cloudtrail_event_data_store" "lake" {
  count = var.enable_cloudtrail_lake ? 1 : 0

  name                           = "toinoi-${var.env}-events"
  multi_region_enabled           = true
  organization_enabled           = false
  retention_period               = 30
  termination_protection_enabled = false # lab: cho phép destroy

  advanced_event_selector {
    name = "Management events"

    field_selector {
      field  = "eventCategory"
      equals = ["Management"]
    }
  }

  tags = {
    Service = "security"
  }
}
