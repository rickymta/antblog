locals {
  account_id = data.aws_caller_identity.current.account_id
  prefix     = "/toinoi/${var.env}/booking"
}

# KMS key riêng cho cấu hình và secret của service booking.
# Policy mặc định ủy quyền cho IAM của tài khoản; quyền cụ thể cấp qua IAM policy.
resource "aws_kms_key" "config" {
  description             = "toinoi-${var.env}-booking: SecureString va secret"
  enable_key_rotation     = true
  deletion_window_in_days = 7

  tags = {
    Service = "booking"
  }
}

resource "aws_kms_alias" "config" {
  name          = "alias/toinoi-${var.env}-config"
  target_key_id = aws_kms_key.config.key_id
}

# Parameter Store: cấu hình thường (String), phân cấp theo /toinoi/<env>/<service>/...
resource "aws_ssm_parameter" "quote_cache_ttl" {
  name  = "${local.prefix}/quote-cache-ttl-seconds"
  type  = "String"
  value = "60"

  tags = {
    Service = "booking"
  }
}

resource "aws_ssm_parameter" "max_active_trips" {
  name  = "${local.prefix}/max-active-trips-per-driver"
  type  = "String"
  value = "3"

  tags = {
    Service = "booking"
  }
}

resource "aws_ssm_parameter" "payment_gateway_url" {
  name  = "${local.prefix}/payment-gateway-url"
  type  = "String"
  value = "https://sandbox.vixanh.example"

  tags = {
    Service = "booking"
  }
}

# SecureString: giá trị thật đặt bằng AWS CLI (scripts/seed-values.sh) để không nằm trong state.
resource "aws_ssm_parameter" "vixanh_api_key" {
  name   = "${local.prefix}/vixanh-api-key"
  type   = "SecureString"
  key_id = aws_kms_key.config.arn
  value  = "CHANGE_ME_AFTER_APPLY"

  lifecycle {
    ignore_changes = [value]
  }

  tags = {
    Service = "booking"
  }
}

# Secrets Manager: chỉ tạo "vỏ" secret. Giá trị đặt bằng CLI để không lưu trong state.
# recovery_window_in_days = 0 chỉ dành cho lab: xoá ngay, không giữ 7-30 ngày.
resource "aws_secretsmanager_secret" "db" {
  name                    = "toinoi/${var.env}/booking/db"
  description             = "Thong tin dang nhap PostgreSQL cua service booking (JSON: username, password, host, port, dbname)"
  kms_key_id              = aws_kms_key.config.arn
  recovery_window_in_days = 0

  tags = {
    Service = "booking"
  }
}

# Role chỉ đọc cấu hình: quyền theo ARN, theo tag của secret và theo dịch vụ dùng KMS.
data "aws_iam_policy_document" "reader_trust" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "AWS"
      identifiers = [var.reader_principal_arn]
    }
  }
}

resource "aws_iam_role" "reader" {
  name               = "toinoi-${var.env}-booking-config-reader"
  assume_role_policy = data.aws_iam_policy_document.reader_trust.json
}

resource "aws_iam_policy" "reader" {
  name = "toinoi-${var.env}-booking-config-read"
  policy = templatefile("${path.module}/reader-policy.json", {
    REGION      = var.region
    ACCOUNT_ID  = local.account_id
    ENV         = var.env
    KMS_KEY_ARN = aws_kms_key.config.arn
  })
}

resource "aws_iam_role_policy_attachment" "reader" {
  role       = aws_iam_role.reader.name
  policy_arn = aws_iam_policy.reader.arn
}
