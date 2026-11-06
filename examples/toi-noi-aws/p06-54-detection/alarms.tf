# ---------------------------------------------------------------------------
# Cảnh báo từ CloudTrail: root đăng nhập, đổi IAM policy, tắt VPC Flow Logs
# ---------------------------------------------------------------------------
resource "aws_sns_topic" "alerts" {
  name = "toinoi-${var.env}-security-alerts"

  tags = {
    Service = "security"
  }
}

resource "aws_sns_topic_subscription" "email" {
  count = var.alert_email == "" ? 0 : 1

  topic_arn = aws_sns_topic.alerts.arn
  protocol  = "email"
  endpoint  = var.alert_email
}

locals {
  iam_policy_change_events = [
    "DeleteGroupPolicy", "DeleteRolePolicy", "DeleteUserPolicy",
    "PutGroupPolicy", "PutRolePolicy", "PutUserPolicy",
    "CreatePolicy", "DeletePolicy", "CreatePolicyVersion", "DeletePolicyVersion",
    "AttachRolePolicy", "DetachRolePolicy", "AttachUserPolicy", "DetachUserPolicy",
    "AttachGroupPolicy", "DetachGroupPolicy",
  ]

  trail_alarms = {
    root-login = {
      description = "Có đăng nhập hoặc gọi API bằng root user"
      pattern     = "{ $.userIdentity.type = \"Root\" && $.userIdentity.invokedBy NOT EXISTS && $.eventType != \"AwsServiceEvent\" }"
    }
    iam-policy-change = {
      description = "IAM policy bị tạo, sửa, xoá hoặc gắn/gỡ"
      pattern     = "{ ${join(" || ", [for e in local.iam_policy_change_events : "($.eventName = ${e})"])} }"
    }
    flow-logs-deleted = {
      description = "VPC Flow Logs bị xoá"
      pattern     = "{ $.eventName = DeleteFlowLogs }"
    }
    trail-tampered = {
      description = "CloudTrail bị dừng, xoá hoặc sửa"
      pattern     = "{ ($.eventName = StopLogging) || ($.eventName = DeleteTrail) || ($.eventName = UpdateTrail) }"
    }
  }
}

resource "aws_cloudwatch_log_metric_filter" "trail" {
  for_each = local.trail_alarms

  name           = "toinoi-${var.env}-${each.key}"
  log_group_name = aws_cloudwatch_log_group.trail.name
  pattern        = each.value.pattern

  metric_transformation {
    name      = each.key
    namespace = "ToiNoi/Security"
    value     = "1"
  }
}

resource "aws_cloudwatch_metric_alarm" "trail" {
  for_each = local.trail_alarms

  alarm_name          = "toinoi-${var.env}-${each.key}"
  alarm_description   = each.value.description
  namespace           = "ToiNoi/Security"
  metric_name         = each.key
  statistic           = "Sum"
  period              = 300
  evaluation_periods  = 1
  threshold           = 1
  comparison_operator = "GreaterThanOrEqualToThreshold"
  treat_missing_data  = "notBreaching"
  alarm_actions       = [aws_sns_topic.alerts.arn]

  tags = {
    Service = "security"
  }

  depends_on = [aws_cloudwatch_log_metric_filter.trail]
}
