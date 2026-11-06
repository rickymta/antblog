# ---------------------------------------------------------------------------
# GuardDuty: phát hiện bất thường, kể cả Extended Threat Detection (attack sequence)
# ---------------------------------------------------------------------------
resource "aws_guardduty_detector" "main" {
  count = var.enable_guardduty ? 1 : 0

  enable                       = true
  finding_publishing_frequency = "FIFTEEN_MINUTES"

  tags = {
    Service = "security"
  }
}

resource "aws_guardduty_detector_feature" "plan" {
  for_each = var.enable_guardduty ? var.guardduty_features : toset([])

  detector_id = aws_guardduty_detector.main[0].id
  name        = each.value
  status      = "ENABLED"
}

# Finding mức HIGH trở lên (severity >= 7) báo về SNS.
resource "aws_cloudwatch_event_rule" "guardduty_high" {
  count = var.enable_guardduty ? 1 : 0

  name        = "toinoi-${var.env}-guardduty-high"
  description = "GuardDuty finding co severity tu 7 tro len"

  event_pattern = jsonencode({
    source        = ["aws.guardduty"]
    "detail-type" = ["GuardDuty Finding"]
    detail = {
      severity = [{ numeric = [">=", 7] }]
    }
  })

  tags = {
    Service = "security"
  }
}

resource "aws_cloudwatch_event_target" "guardduty_high" {
  count = var.enable_guardduty ? 1 : 0

  rule = aws_cloudwatch_event_rule.guardduty_high[0].name
  arn  = aws_sns_topic.alerts.arn
}

data "aws_iam_policy_document" "alerts_topic" {
  statement {
    sid       = "AllowEventBridgePublish"
    effect    = "Allow"
    actions   = ["sns:Publish"]
    resources = [aws_sns_topic.alerts.arn]

    principals {
      type        = "Service"
      identifiers = ["events.amazonaws.com"]
    }
  }
}

resource "aws_sns_topic_policy" "alerts" {
  arn    = aws_sns_topic.alerts.arn
  policy = data.aws_iam_policy_document.alerts_topic.json
}

# ---------------------------------------------------------------------------
# Security Hub: phiên bản mới (tổng hợp rủi ro) và CSPM (chuẩn và control)
# ---------------------------------------------------------------------------
resource "aws_securityhub_account_v2" "main" {
  count = var.enable_security_hub_v2 ? 1 : 0
}

resource "aws_securityhub_account" "cspm" {
  count = var.enable_security_hub_cspm ? 1 : 0

  enable_default_standards  = false
  auto_enable_controls      = true
  control_finding_generator = "SECURITY_CONTROL"
}

resource "aws_securityhub_standards_subscription" "fsbp" {
  count = var.enable_security_hub_cspm ? 1 : 0

  standards_arn = "arn:aws:securityhub:${var.region}::standards/aws-foundational-security-best-practices/v/1.0.0"

  depends_on = [aws_securityhub_account.cspm]
}

# Automation rule (CSPM): finding HIGH của tài nguyên gắn Env=prod được nâng lên CRITICAL.
resource "aws_securityhub_automation_rule" "prod_high_to_critical" {
  count = var.enable_security_hub_cspm ? 1 : 0

  rule_name   = "toinoi-prod-high-to-critical"
  description = "Finding HIGH tren tai nguyen Env=prod duoc nang len CRITICAL de on-call xu ly truoc"
  rule_order  = 1

  criteria {
    severity_label {
      comparison = "EQUALS"
      value      = "HIGH"
    }

    resource_tags {
      comparison = "EQUALS"
      key        = "Env"
      value      = "prod"
    }
  }

  actions {
    type = "FINDING_FIELDS_UPDATE"

    finding_fields_update {
      severity {
        label = "CRITICAL"
      }

      note {
        text       = "Nang muc do vi tai nguyen thuoc moi truong prod."
        updated_by = "toinoi-automation"
      }
    }
  }

  depends_on = [aws_securityhub_account.cspm]
}

# ---------------------------------------------------------------------------
# Inspector (quét lỗ hổng EC2, ECR, Lambda) và Macie (PII trong S3)
# ---------------------------------------------------------------------------
resource "aws_inspector2_enabler" "main" {
  count = var.enable_inspector ? 1 : 0

  account_ids    = [local.account_id]
  resource_types = ["EC2", "ECR", "LAMBDA"]
}

resource "aws_macie2_account" "main" {
  count = var.enable_macie ? 1 : 0

  finding_publishing_frequency = "FIFTEEN_MINUTES"
  status                       = "ENABLED"
}

resource "aws_macie2_custom_data_identifier" "cccd" {
  count = var.enable_macie ? 1 : 0

  name                   = "toinoi-cccd-vn"
  description            = "Số căn cước công dân Việt Nam: 12 chữ số bắt đầu bằng 0"
  regex                  = "\\b0[0-9]{11}\\b"
  keywords               = ["cccd", "can cuoc", "căn cước"]
  maximum_match_distance = 50

  depends_on = [aws_macie2_account.main]
}

resource "aws_macie2_custom_data_identifier" "phone" {
  count = var.enable_macie ? 1 : 0

  name                   = "toinoi-phone-vn"
  description            = "Số điện thoại di động Việt Nam: 0 hoặc +84 rồi 9 chữ số"
  regex                  = "(\\+84|\\b0)[0-9]{9}\\b"
  keywords               = ["sdt", "dien thoai", "điện thoại", "phone"]
  maximum_match_distance = 50

  depends_on = [aws_macie2_account.main]
}

resource "aws_macie2_classification_job" "pii_once" {
  count = var.enable_macie && length(var.macie_bucket_names) > 0 ? 1 : 0

  name                       = "toinoi-${var.env}-pii-once"
  job_type                   = "ONE_TIME"
  custom_data_identifier_ids = [aws_macie2_custom_data_identifier.cccd[0].id, aws_macie2_custom_data_identifier.phone[0].id]

  s3_job_definition {
    bucket_definitions {
      account_id = local.account_id
      buckets    = var.macie_bucket_names
    }
  }

  depends_on = [aws_macie2_account.main]
}
