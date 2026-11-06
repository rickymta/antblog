data "aws_caller_identity" "current" {}

# Đổi ARN phiên assume-role (arn:aws:sts::...:assumed-role/...) thành ARN role có path,
# đúng dạng mà aws:PrincipalArn trong bucket policy so khớp.
data "aws_iam_session_context" "current" {
  arn = data.aws_caller_identity.current.arn
}

locals {
  bucket_name = "toinoi-lab-tfstate-${data.aws_caller_identity.current.account_id}-${var.region}"

  # Chuyển mẫu wildcard của IAM (* và ?) thành regex để kiểm role đang chạy Terraform có nằm trong danh sách không.
  caller_role_arn = data.aws_iam_session_context.current.issuer_arn
  caller_allowed = length(var.state_access_principal_arns) == 0 || anytrue([
    for p in var.state_access_principal_arns :
    can(regex("^${replace(replace(replace(replace(p, ".", "\\."), "+", "\\+"), "*", ".*"), "?", ".")}$", local.caller_role_arn))
  ])
}

resource "aws_s3_bucket" "state" {
  bucket = local.bucket_name

  # State chứa tài nguyên thật và có thể chứa secret: không cho destroy nhầm.
  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_s3_bucket_versioning" "state" {
  bucket = aws_s3_bucket.state.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "state" {
  bucket = aws_s3_bucket.state.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "state" {
  bucket = aws_s3_bucket.state.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

data "aws_iam_policy_document" "state" {
  statement {
    sid       = "DenyInsecureTransport"
    effect    = "Deny"
    actions   = ["s3:*"]
    resources = [aws_s3_bucket.state.arn, "${aws_s3_bucket.state.arn}/*"]

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

  # Chỉ tạo statement này khi đã khai báo danh sách role được phép.
  dynamic "statement" {
    for_each = length(var.state_access_principal_arns) > 0 ? [1] : []

    content {
      sid       = "DenyAllExceptStateRoles"
      effect    = "Deny"
      actions   = ["s3:*"]
      resources = [aws_s3_bucket.state.arn, "${aws_s3_bucket.state.arn}/*"]

      principals {
        type        = "*"
        identifiers = ["*"]
      }

      condition {
        test     = "ArnNotLike"
        variable = "aws:PrincipalArn"
        values   = var.state_access_principal_arns
      }
    }
  }
}

resource "aws_s3_bucket_policy" "state" {
  bucket = aws_s3_bucket.state.id
  policy = data.aws_iam_policy_document.state.json

  depends_on = [aws_s3_bucket_public_access_block.state]

  # Chặn tự khoá: không áp policy "Deny mọi principal khác" nếu chính role đang apply không có trong danh sách.
  lifecycle {
    precondition {
      condition     = local.caller_allowed
      error_message = "Role đang chạy Terraform (${local.caller_role_arn}) không khớp state_access_principal_arns; áp policy này sẽ khoá chính bạn khỏi bucket state."
    }
  }
}
