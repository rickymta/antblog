# ---------------------------------------------------------------------------
# Log có cấu trúc của service booking: retention, che PII, metric từ log
# ---------------------------------------------------------------------------
resource "aws_cloudwatch_log_group" "booking" {
  name              = "/toinoi/${var.env}/booking"
  retention_in_days = var.log_retention_days

  tags = {
    Service = "booking"
  }
}

# Data protection policy: kiểm toán và che email, số điện thoại Việt Nam trong log.
resource "aws_cloudwatch_log_data_protection_policy" "booking" {
  log_group_name = aws_cloudwatch_log_group.booking.name

  policy_document = jsonencode({
    Name        = "toinoi-booking-pii"
    Description = "Che email va so dien thoai trong log booking"
    Version     = "2021-06-01"

    Configuration = {
      CustomDataIdentifier = [
        { Name = "VnPhone", Regex = "(\\+84|\\b0)[0-9]{9}\\b" },
      ]
    }

    Statement = [
      {
        Sid            = "audit-policy"
        DataIdentifier = ["arn:aws:dataprotection::aws:data-identifier/EmailAddress", "VnPhone"]
        Operation = {
          Audit = {
            FindingsDestination = {}
          }
        }
      },
      {
        Sid            = "redact-policy"
        DataIdentifier = ["arn:aws:dataprotection::aws:data-identifier/EmailAddress", "VnPhone"]
        Operation = {
          Deidentify = {
            MaskConfig = {}
          }
        }
      },
    ]
  })
}

# Metric filter: đếm log mức ERROR và lấy duration_ms làm giá trị metric. Mẫu log ở samples/booking-log.json.
resource "aws_cloudwatch_log_metric_filter" "errors" {
  name           = "toinoi-${var.env}-booking-errors"
  log_group_name = aws_cloudwatch_log_group.booking.name
  pattern        = "{ $.level = \"ERROR\" }"

  metric_transformation {
    name      = "BookingErrorLogs"
    namespace = "ToiNoi/Booking"
    value     = "1"
  }
}

resource "aws_cloudwatch_log_metric_filter" "duration" {
  name           = "toinoi-${var.env}-booking-duration"
  log_group_name = aws_cloudwatch_log_group.booking.name
  pattern        = "{ $.duration_ms = * }"

  metric_transformation {
    name      = "BookingDurationMs"
    namespace = "ToiNoi/Booking"
    value     = "$.duration_ms"
    unit      = "Milliseconds"
  }
}

# Query đã lưu cho Logs Insights (nội dung ở queries/).
resource "aws_cloudwatch_query_definition" "slow_by_city" {
  name            = "toinoi/${var.env}/booking-slow-by-city"
  log_group_names = [aws_cloudwatch_log_group.booking.name]
  query_string    = file("${path.module}/queries/slow-by-city.logsinsights")
}

resource "aws_cloudwatch_query_definition" "errors_with_trace" {
  name            = "toinoi/${var.env}/booking-errors-with-trace"
  log_group_names = [aws_cloudwatch_log_group.booking.name]
  query_string    = file("${path.module}/queries/errors-with-trace.logsinsights")
}
