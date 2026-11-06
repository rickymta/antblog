data "aws_caller_identity" "current" {}

locals {
  account_id = data.aws_caller_identity.current.account_id
  bucket     = "toinoi-${var.env}-kms-demo-${local.account_id}"
}

# ---------------------------------------------------------------------------
# 1. Key policy: gốc của quyền trên key
# ---------------------------------------------------------------------------
# Một bản policy cho mỗi region: primary dùng var.region, replica dùng var.dr_region,
# vì kms:ViaService và principal của CloudWatch Logs ghi tên region.
data "aws_iam_policy_document" "key" {
  for_each = toset([var.region, var.dr_region])

  # Ủy quyền cho IAM: không có statement này thì IAM policy vô hiệu với key.
  statement {
    sid       = "EnableIamPolicies"
    effect    = "Allow"
    actions   = ["kms:*"]
    resources = ["*"]

    principals {
      type        = "AWS"
      identifiers = ["arn:aws:iam::${local.account_id}:root"]
    }
  }

  # Key admin: quản trị vòng đời key, KHÔNG có Encrypt/Decrypt/GenerateDataKey.
  dynamic "statement" {
    for_each = length(var.key_admin_arns) > 0 ? [1] : []

    content {
      sid    = "KeyAdministrators"
      effect = "Allow"
      actions = [
        "kms:Create*",
        "kms:Describe*",
        "kms:Enable*",
        "kms:List*",
        "kms:Put*",
        "kms:Update*",
        "kms:Revoke*",
        "kms:Disable*",
        "kms:Get*",
        "kms:Delete*",
        "kms:TagResource",
        "kms:UntagResource",
        "kms:ScheduleKeyDeletion",
        "kms:CancelKeyDeletion",
        "kms:RotateKeyOnDemand",
      ]
      resources = ["*"]

      principals {
        type        = "AWS"
        identifiers = var.key_admin_arns
      }
    }
  }

  # Key user của service booking: dùng key trực tiếp, nhưng chỉ khi encryption context khớp.
  dynamic "statement" {
    for_each = length(var.key_user_arns) > 0 ? [1] : []

    content {
      sid    = "BookingUsesKeyWithContext"
      effect = "Allow"
      actions = [
        "kms:Encrypt",
        "kms:Decrypt",
        "kms:GenerateDataKey",
        "kms:DescribeKey",
      ]
      resources = ["*"]

      principals {
        type        = "AWS"
        identifiers = var.key_user_arns
      }

      condition {
        test     = "StringEquals"
        variable = "kms:EncryptionContext:Service"
        values   = ["booking"]
      }
    }
  }

  # Mọi principal trong tài khoản dùng key, nhưng chỉ thông qua S3 hoặc Secrets Manager.
  statement {
    sid    = "UseKeyOnlyViaS3AndSecrets"
    effect = "Allow"
    actions = [
      "kms:Encrypt",
      "kms:Decrypt",
      "kms:ReEncrypt*",
      "kms:GenerateDataKey*",
      "kms:DescribeKey",
    ]
    resources = ["*"]

    principals {
      type        = "AWS"
      identifiers = ["*"]
    }

    condition {
      test     = "StringEquals"
      variable = "kms:CallerAccount"
      values   = [local.account_id]
    }

    condition {
      test     = "StringEquals"
      variable = "kms:ViaService"
      values = [
        "s3.${each.key}.amazonaws.com",
        "secretsmanager.${each.key}.amazonaws.com",
      ]
    }
  }

  # Grant tạm thời cho tài nguyên AWS (EBS volume, RDS instance) trong tài khoản này.
  statement {
    sid       = "GrantsForEbsAndRds"
    effect    = "Allow"
    actions   = ["kms:CreateGrant", "kms:ListGrants", "kms:RevokeGrant"]
    resources = ["*"]

    principals {
      type        = "AWS"
      identifiers = ["*"]
    }

    condition {
      test     = "Bool"
      variable = "kms:GrantIsForAWSResource"
      values   = ["true"]
    }

    condition {
      test     = "StringEquals"
      variable = "kms:CallerAccount"
      values   = [local.account_id]
    }

    condition {
      test     = "StringEquals"
      variable = "kms:ViaService"
      values = [
        "ec2.${each.key}.amazonaws.com",
        "rds.${each.key}.amazonaws.com",
      ]
    }
  }

  # CloudWatch Logs mã hoá log group bằng key này, chỉ cho log group của Tới Nơi.
  statement {
    sid    = "CloudWatchLogsUseKey"
    effect = "Allow"
    actions = [
      "kms:Encrypt*",
      "kms:Decrypt*",
      "kms:ReEncrypt*",
      "kms:GenerateDataKey*",
      "kms:Describe*",
    ]
    resources = ["*"]

    principals {
      type        = "Service"
      identifiers = ["logs.${each.key}.amazonaws.com"]
    }

    condition {
      test     = "ArnLike"
      variable = "kms:EncryptionContext:aws:logs:arn"
      values   = ["arn:aws:logs:${each.key}:${local.account_id}:log-group:/toinoi/${var.env}/*"]
    }
  }
}

# ---------------------------------------------------------------------------
# 2. Key đối xứng multi-region, rotation tự động, alias và replica
# ---------------------------------------------------------------------------
resource "aws_kms_key" "trips" {
  description             = "Khoá mã hoá dữ liệu khách của Tới Nơi (ảnh chứng từ, log, secret)."
  key_usage               = "ENCRYPT_DECRYPT"
  enable_key_rotation     = true
  rotation_period_in_days = 365
  deletion_window_in_days = var.deletion_window_days
  multi_region            = var.enable_multi_region_replica
  policy                  = data.aws_iam_policy_document.key[var.region].json

  tags = {
    Service = "platform"
  }
}

resource "aws_kms_alias" "trips" {
  name          = "alias/toinoi-${var.env}-trips"
  target_key_id = aws_kms_key.trips.key_id
}

resource "aws_kms_replica_key" "dr" {
  count    = var.enable_multi_region_replica ? 1 : 0
  provider = aws.dr

  description             = "Replica ở ${var.dr_region} của key trips (multi-region key)."
  primary_key_arn         = aws_kms_key.trips.arn
  deletion_window_in_days = var.deletion_window_days

  # Replica có key policy riêng (không đồng bộ từ primary): dùng bản policy của region DR.
  policy = data.aws_iam_policy_document.key[var.dr_region].json

  tags = {
    Service = "platform"
  }
}

resource "aws_kms_alias" "dr" {
  count    = var.enable_multi_region_replica ? 1 : 0
  provider = aws.dr

  name          = "alias/toinoi-${var.env}-trips"
  target_key_id = aws_kms_replica_key.dr[0].key_id
}

# ---------------------------------------------------------------------------
# 3. S3 mã hoá bằng SSE-KMS với bucket key, và bucket policy chặn sai khoá
# ---------------------------------------------------------------------------
resource "aws_s3_bucket" "demo" {
  bucket        = local.bucket
  force_destroy = true # lab: cho phép destroy kể cả khi còn object

  tags = {
    Service = "delivery-photo"
  }
}

resource "aws_s3_bucket_public_access_block" "demo" {
  bucket = aws_s3_bucket.demo.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "demo" {
  bucket = aws_s3_bucket.demo.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm     = "aws:kms"
      kms_master_key_id = aws_kms_key.trips.arn
    }

    # Bucket key: S3 dùng một key trung gian cho cả bucket, giảm mạnh số lần gọi KMS.
    bucket_key_enabled = true
  }
}

data "aws_iam_policy_document" "bucket" {
  # Deny với điều kiện phủ định (kể cả ...IfExists) vẫn khớp khi header vắng mặt, nên mọi
  # statement dưới đây kèm điều kiện Null = false: chỉ chặn khi client CÓ gửi header.
  # Upload không gửi header nào thì rơi vào mã hoá mặc định của bucket (SSE-KMS ở trên).
  statement {
    sid       = "DenyUploadsAskingForAnotherAlgorithm"
    effect    = "Deny"
    actions   = ["s3:PutObject"]
    resources = ["${aws_s3_bucket.demo.arn}/*"]

    principals {
      type        = "*"
      identifiers = ["*"]
    }

    condition {
      test     = "Null"
      variable = "s3:x-amz-server-side-encryption"
      values   = ["false"]
    }

    condition {
      test     = "StringNotEquals"
      variable = "s3:x-amz-server-side-encryption"
      values   = ["aws:kms"]
    }
  }

  statement {
    sid       = "DenyUploadsWithAnotherKey"
    effect    = "Deny"
    actions   = ["s3:PutObject"]
    resources = ["${aws_s3_bucket.demo.arn}/*"]

    principals {
      type        = "*"
      identifiers = ["*"]
    }

    condition {
      test     = "Null"
      variable = "s3:x-amz-server-side-encryption-aws-kms-key-id"
      values   = ["false"]
    }

    condition {
      test     = "StringNotEquals"
      variable = "s3:x-amz-server-side-encryption-aws-kms-key-id"
      values   = [aws_kms_key.trips.arn]
    }
  }

  # Gửi "aws:kms" mà không kèm key id thì S3 dùng key AWS managed aws/s3: chặn luôn.
  statement {
    sid       = "DenyKmsHeaderWithoutKeyId"
    effect    = "Deny"
    actions   = ["s3:PutObject"]
    resources = ["${aws_s3_bucket.demo.arn}/*"]

    principals {
      type        = "*"
      identifiers = ["*"]
    }

    condition {
      test     = "Null"
      variable = "s3:x-amz-server-side-encryption"
      values   = ["false"]
    }

    condition {
      test     = "Null"
      variable = "s3:x-amz-server-side-encryption-aws-kms-key-id"
      values   = ["true"]
    }
  }

  statement {
    sid       = "DenyPlainHttp"
    effect    = "Deny"
    actions   = ["s3:*"]
    resources = [aws_s3_bucket.demo.arn, "${aws_s3_bucket.demo.arn}/*"]

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

resource "aws_s3_bucket_policy" "demo" {
  bucket = aws_s3_bucket.demo.id
  policy = data.aws_iam_policy_document.bucket.json

  depends_on = [aws_s3_bucket_public_access_block.demo]
}

# ---------------------------------------------------------------------------
# 4. CloudWatch Logs mã hoá bằng cùng key
# ---------------------------------------------------------------------------
resource "aws_cloudwatch_log_group" "booking" {
  name              = "/toinoi/${var.env}/booking"
  retention_in_days = 14
  kms_key_id        = aws_kms_key.trips.arn

  tags = {
    Service = "booking"
  }

  depends_on = [aws_kms_key.trips]
}
