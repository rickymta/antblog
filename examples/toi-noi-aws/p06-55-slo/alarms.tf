# ---------------------------------------------------------------------------
# Alarm theo triệu chứng: burn rate, latency p99, anomaly detection, composite
# ---------------------------------------------------------------------------
resource "aws_sns_topic" "oncall" {
  name = "toinoi-${var.env}-oncall"

  tags = {
    Service = "booking"
  }
}

resource "aws_sns_topic_subscription" "email" {
  count = var.alert_email == "" ? 0 : 1

  topic_arn = aws_sns_topic.oncall.arn
  protocol  = "email"
  endpoint  = var.alert_email
}

locals {
  error_budget = 1 - var.slo_target
}

# Burn rate nhanh: tốc độ đốt error budget >= 14,4 lần mức cho phép, liên tục 1 giờ (12 chu kỳ 5 phút).
# Xấp xỉ cho lab: tỉ lệ lỗi theo từng chu kỳ 5 phút, không phải cửa sổ trượt 1 giờ chuẩn.
resource "aws_cloudwatch_metric_alarm" "burn_fast" {
  alarm_name          = "toinoi-${var.env}-booking-burn-fast"
  alarm_description   = "Availability API booking đang đốt error budget nhanh (burn rate >= 14,4)."
  comparison_operator = "GreaterThanOrEqualToThreshold"
  threshold           = 14.4
  evaluation_periods  = 12
  datapoints_to_alarm = 12
  treat_missing_data  = "notBreaching"

  metric_query {
    id          = "errors"
    return_data = false

    metric {
      namespace   = "AWS/ApplicationELB"
      metric_name = "HTTPCode_Target_5XX_Count"
      period      = 300
      stat        = "Sum"

      dimensions = {
        LoadBalancer = var.alb_arn_suffix
      }
    }
  }

  metric_query {
    id          = "requests"
    return_data = false

    metric {
      namespace   = "AWS/ApplicationELB"
      metric_name = "RequestCount"
      period      = 300
      stat        = "Sum"

      dimensions = {
        LoadBalancer = var.alb_arn_suffix
      }
    }
  }

  metric_query {
    id          = "burn"
    expression  = "IF(requests > 0, (errors / requests) / ${local.error_budget}, 0)"
    label       = "Burn rate"
    return_data = true
  }

  tags = {
    Service = "booking"
  }
}

# Latency p99 của API booking (percentile, không dùng trung bình).
resource "aws_cloudwatch_metric_alarm" "latency_p99" {
  alarm_name          = "toinoi-${var.env}-booking-latency-p99"
  alarm_description   = "p99 TargetResponseTime vượt ngưỡng SLO."
  namespace           = "AWS/ApplicationELB"
  metric_name         = "TargetResponseTime"
  extended_statistic  = "p99"
  period              = 300
  evaluation_periods  = 3
  datapoints_to_alarm = 3
  threshold           = var.latency_threshold_seconds
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    LoadBalancer = var.alb_arn_suffix
  }

  tags = {
    Service = "booking"
  }
}

# Anomaly detection: dải bình thường của lưu lượng học từ lịch sử; báo khi vọt ra ngoài dải trên.
resource "aws_cloudwatch_metric_alarm" "traffic_anomaly" {
  alarm_name          = "toinoi-${var.env}-booking-traffic-anomaly"
  alarm_description   = "Lưu lượng đặt chuyến vượt dải bình thường (anomaly detection band)."
  comparison_operator = "GreaterThanUpperThreshold"
  evaluation_periods  = 3
  datapoints_to_alarm = 3
  threshold_metric_id = "band"
  treat_missing_data  = "notBreaching"

  metric_query {
    id          = "requests"
    return_data = true

    metric {
      namespace   = "AWS/ApplicationELB"
      metric_name = "RequestCount"
      period      = 300
      stat        = "Sum"

      dimensions = {
        LoadBalancer = var.alb_arn_suffix
      }
    }
  }

  metric_query {
    id          = "band"
    expression  = "ANOMALY_DETECTION_BAND(requests, 2)"
    label       = "Dải bình thường"
    return_data = true
  }

  tags = {
    Service = "booking"
  }
}

# Alarm lỗi từ log (metric filter) để có thêm một tín hiệu độc lập với ALB.
resource "aws_cloudwatch_metric_alarm" "error_logs" {
  alarm_name          = "toinoi-${var.env}-booking-error-logs"
  alarm_description   = "Service booking ghi nhiều log mức ERROR."
  namespace           = "ToiNoi/Booking"
  metric_name         = "BookingErrorLogs"
  statistic           = "Sum"
  period              = 300
  evaluation_periods  = 2
  datapoints_to_alarm = 2
  threshold           = 20
  comparison_operator = "GreaterThanOrEqualToThreshold"
  treat_missing_data  = "notBreaching"

  tags = {
    Service = "booking"
  }

  depends_on = [aws_cloudwatch_log_metric_filter.errors]
}

# Composite: chỉ gọi người khi cả lỗi lẫn latency cùng xấu, hoặc burn rate nhanh. Alarm con không gửi SNS riêng.
resource "aws_cloudwatch_composite_alarm" "booking_page" {
  alarm_name        = "toinoi-${var.env}-booking-page"
  alarm_description = "Gọi on-call: burn rate nhanh, hoặc lỗi log và latency p99 cùng vượt ngưỡng."
  alarm_rule        = "ALARM(\"${aws_cloudwatch_metric_alarm.burn_fast.alarm_name}\") OR (ALARM(\"${aws_cloudwatch_metric_alarm.error_logs.alarm_name}\") AND ALARM(\"${aws_cloudwatch_metric_alarm.latency_p99.alarm_name}\"))"
  alarm_actions     = [aws_sns_topic.oncall.arn]
  ok_actions        = [aws_sns_topic.oncall.arn]

  tags = {
    Service = "booking"
  }
}
