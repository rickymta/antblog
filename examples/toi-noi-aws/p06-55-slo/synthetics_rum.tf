# ---------------------------------------------------------------------------
# Synthetics canary (đặt chuyến thử mỗi 5 phút) và RUM cho web: đều tuỳ chọn
# ---------------------------------------------------------------------------
data "archive_file" "canary" {
  type        = "zip"
  source_dir  = "${path.module}/canary"
  output_path = "${path.module}/.build/canary.zip"
}

resource "aws_s3_bucket" "canary" {
  count = var.enable_canary ? 1 : 0

  bucket_prefix = "toinoi-${var.env}-canary-"
  force_destroy = true

  tags = {
    Service = "booking"
  }
}

resource "aws_s3_bucket_public_access_block" "canary" {
  count = var.enable_canary ? 1 : 0

  bucket = aws_s3_bucket.canary[0].id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

data "aws_iam_policy_document" "canary_assume" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "canary" {
  count = var.enable_canary ? 1 : 0

  statement {
    effect    = "Allow"
    actions   = ["s3:PutObject", "s3:GetObject"]
    resources = ["${aws_s3_bucket.canary[0].arn}/*"]
  }

  statement {
    effect    = "Allow"
    actions   = ["s3:GetBucketLocation"]
    resources = [aws_s3_bucket.canary[0].arn]
  }

  statement {
    effect    = "Allow"
    actions   = ["s3:ListAllMyBuckets"]
    resources = ["*"]
  }

  statement {
    effect    = "Allow"
    actions   = ["logs:CreateLogGroup", "logs:CreateLogStream", "logs:PutLogEvents"]
    resources = ["arn:aws:logs:${var.region}:*:log-group:/aws/lambda/cwsyn-*"]
  }

  statement {
    effect    = "Allow"
    actions   = ["cloudwatch:PutMetricData"]
    resources = ["*"]

    condition {
      test     = "StringEquals"
      variable = "cloudwatch:namespace"
      values   = ["CloudWatchSynthetics"]
    }
  }
}

resource "aws_iam_role" "canary" {
  count = var.enable_canary ? 1 : 0

  name               = "toinoi-${var.env}-booking-canary"
  assume_role_policy = data.aws_iam_policy_document.canary_assume.json

  tags = {
    Service = "booking"
  }
}

resource "aws_iam_role_policy" "canary" {
  count = var.enable_canary ? 1 : 0

  name   = "canary-runtime"
  role   = aws_iam_role.canary[0].id
  policy = data.aws_iam_policy_document.canary[0].json
}

resource "aws_synthetics_canary" "booking_quote" {
  count = var.enable_canary ? 1 : 0

  name                 = "toinoi-${var.env}-quote"
  artifact_s3_location = "s3://${aws_s3_bucket.canary[0].id}/canary/"
  execution_role_arn   = aws_iam_role.canary[0].arn
  handler              = "booking-canary.handler"
  zip_file             = data.archive_file.canary.output_path
  runtime_version      = var.canary_runtime_version
  start_canary         = true

  success_retention_period = 2
  failure_retention_period = 14

  schedule {
    expression = "rate(5 minutes)"
  }

  run_config {
    timeout_in_seconds = 60

    environment_variables = {
      TARGET_HOST = var.canary_target_host
    }
  }

  tags = {
    Service = "booking"
  }

  depends_on = [aws_iam_role_policy.canary]
}

resource "aws_cloudwatch_metric_alarm" "canary_failed" {
  count = var.enable_canary ? 1 : 0

  alarm_name          = "toinoi-${var.env}-canary-failed"
  alarm_description   = "Canary báo giá thất bại 2 lần liên tiếp."
  namespace           = "CloudWatchSynthetics"
  metric_name         = "SuccessPercent"
  statistic           = "Average"
  period              = 300
  evaluation_periods  = 2
  datapoints_to_alarm = 2
  threshold           = 100
  comparison_operator = "LessThanThreshold"
  treat_missing_data  = "breaching"
  alarm_actions       = [aws_sns_topic.oncall.arn]

  dimensions = {
    CanaryName = aws_synthetics_canary.booking_quote[0].name
  }

  tags = {
    Service = "booking"
  }
}

resource "aws_rum_app_monitor" "web" {
  count = var.enable_rum ? 1 : 0

  name           = "toinoi-${var.env}-web"
  domain         = "toinoi.example"
  cw_log_enabled = false

  app_monitor_configuration {
    allow_cookies       = true
    enable_xray         = true
    session_sample_rate = 0.1
    telemetries         = ["errors", "performance", "http"]
  }

  tags = {
    Service = "web"
  }
}
