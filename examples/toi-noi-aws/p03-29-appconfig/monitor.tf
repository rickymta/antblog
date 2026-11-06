# Alarm trên metric tuỳ chỉnh do service báo giá phát ra (ví dụ tỉ lệ 5xx tính theo phút).
# Khi alarm ở trạng thái ALARM trong lúc deploy hoặc bake, AppConfig rollback cấu hình.
resource "aws_cloudwatch_metric_alarm" "quote_5xx" {
  alarm_name          = "${local.prefix}-http5xx"
  alarm_description   = "Tỉ lệ 5xx của quote vượt ngưỡng: dừng và rollback deployment AppConfig"
  namespace           = "Toinoi/Quote"
  metric_name         = "Http5xxRate"
  statistic           = "Average"
  period              = 60
  evaluation_periods  = 3
  threshold           = 1
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"
}

data "aws_iam_policy_document" "assume_appconfig" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["appconfig.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "appconfig_monitor" {
  statement {
    actions   = ["cloudwatch:DescribeAlarms"]
    resources = ["*"]
  }
}

resource "aws_iam_role" "appconfig_monitor" {
  name               = "${local.prefix}-appconfig-monitor"
  assume_role_policy = data.aws_iam_policy_document.assume_appconfig.json
}

resource "aws_iam_role_policy" "appconfig_monitor" {
  name   = "read-alarm"
  role   = aws_iam_role.appconfig_monitor.id
  policy = data.aws_iam_policy_document.appconfig_monitor.json
}
