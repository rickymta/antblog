locals {
  prefix = "toinoi-${var.env}"
}

# Custom bus cho sự kiện nghiệp vụ của Tới Nơi, tách khỏi default bus (sự kiện của AWS).
resource "aws_cloudwatch_event_bus" "trips" {
  name = "${local.prefix}-trips"
}

# Queue đích của rule và DLQ cho các lần chuyển thất bại.
resource "aws_sqs_queue" "settlement" {
  name                      = "${local.prefix}-settlement"
  receive_wait_time_seconds = 20
  sqs_managed_sse_enabled   = true
}

resource "aws_sqs_queue" "rule_dlq" {
  name                      = "${local.prefix}-trips-rule-dlq"
  message_retention_seconds = 1209600
  sqs_managed_sse_enabled   = true
}

resource "aws_sqs_queue" "reconcile" {
  name                      = "${local.prefix}-reconcile"
  receive_wait_time_seconds = 20
  sqs_managed_sse_enabled   = true
}

# Rule 1: TripCompleted ở HCM với giá từ ngưỡng trở lên.
resource "aws_cloudwatch_event_rule" "trip_completed" {
  name           = "${local.prefix}-trip-completed-hcm"
  event_bus_name = aws_cloudwatch_event_bus.trips.name
  description    = "TripCompleted ở HCM, giá từ ngưỡng trở lên, chuyển tới queue settlement."

  event_pattern = jsonencode({
    source        = ["toinoi.booking"]
    "detail-type" = ["TripCompleted"]
    detail = {
      city    = ["HCM"]
      fareVnd = [{ numeric = [">=", var.min_fare_vnd] }]
    }
  })
}

# Rule 2: TripCancelled do tài xế huỷ (reason bắt đầu bằng "driver_").
resource "aws_cloudwatch_event_rule" "trip_cancelled" {
  name           = "${local.prefix}-trip-cancelled-by-driver"
  event_bus_name = aws_cloudwatch_event_bus.trips.name
  description    = "TripCancelled do tài xế huỷ, chuyển tới queue settlement."

  event_pattern = jsonencode({
    source        = ["toinoi.booking"]
    "detail-type" = ["TripCancelled"]
    detail = {
      reason = [{ prefix = "driver_" }]
    }
  })
}

resource "aws_cloudwatch_event_target" "completed_to_sqs" {
  rule           = aws_cloudwatch_event_rule.trip_completed.name
  event_bus_name = aws_cloudwatch_event_bus.trips.name
  target_id      = "settlement-queue"
  arn            = aws_sqs_queue.settlement.arn

  # Chỉ giữ trường consumer cần thay vì cả envelope.
  input_transformer {
    # Giữ eventId để consumer idempotent (bài 16).
    input_paths = {
      eventId = "$.detail.eventId"
      tripId  = "$.detail.tripId"
      city    = "$.detail.city"
      fare    = "$.detail.fareVnd"
    }
    input_template = "{\"eventId\":\"<eventId>\",\"tripId\":\"<tripId>\",\"city\":\"<city>\",\"fareVnd\":<fare>}"
  }

  retry_policy {
    maximum_event_age_in_seconds = 3600
    maximum_retry_attempts       = 10
  }

  dead_letter_config {
    arn = aws_sqs_queue.rule_dlq.arn
  }
}

resource "aws_cloudwatch_event_target" "cancelled_to_sqs" {
  rule           = aws_cloudwatch_event_rule.trip_cancelled.name
  event_bus_name = aws_cloudwatch_event_bus.trips.name
  target_id      = "settlement-queue"
  arn            = aws_sqs_queue.settlement.arn

  retry_policy {
    maximum_event_age_in_seconds = 3600
    maximum_retry_attempts       = 10
  }

  dead_letter_config {
    arn = aws_sqs_queue.rule_dlq.arn
  }
}

# Cho đúng hai rule này gửi vào queue đích; DLQ cũng cần policy riêng.
data "aws_iam_policy_document" "settlement_queue" {
  statement {
    sid       = "AllowTripRules"
    effect    = "Allow"
    actions   = ["sqs:SendMessage"]
    resources = [aws_sqs_queue.settlement.arn]

    principals {
      type        = "Service"
      identifiers = ["events.amazonaws.com"]
    }

    condition {
      test     = "ArnEquals"
      variable = "aws:SourceArn"
      values = [
        aws_cloudwatch_event_rule.trip_completed.arn,
        aws_cloudwatch_event_rule.trip_cancelled.arn,
      ]
    }
  }
}

data "aws_iam_policy_document" "rule_dlq" {
  statement {
    sid       = "AllowTripRulesToDlq"
    effect    = "Allow"
    actions   = ["sqs:SendMessage"]
    resources = [aws_sqs_queue.rule_dlq.arn]

    principals {
      type        = "Service"
      identifiers = ["events.amazonaws.com"]
    }

    condition {
      test     = "ArnEquals"
      variable = "aws:SourceArn"
      values = [
        aws_cloudwatch_event_rule.trip_completed.arn,
        aws_cloudwatch_event_rule.trip_cancelled.arn,
      ]
    }
  }
}

resource "aws_sqs_queue_policy" "settlement" {
  queue_url = aws_sqs_queue.settlement.id
  policy    = data.aws_iam_policy_document.settlement_queue.json
}

resource "aws_sqs_queue_policy" "rule_dlq" {
  queue_url = aws_sqs_queue.rule_dlq.id
  policy    = data.aws_iam_policy_document.rule_dlq.json
}

# Archive mọi sự kiện trên bus để replay khi cần xử lý lại.
resource "aws_cloudwatch_event_archive" "trips" {
  name             = "${local.prefix}-trips-archive"
  event_source_arn = aws_cloudwatch_event_bus.trips.arn
  retention_days   = var.archive_retention_days
  description      = "Lưu sự kiện chuyến để replay."
}

# Schema discovery: tự suy ra schema từ sự kiện đi qua bus.
resource "aws_schemas_discoverer" "trips" {
  source_arn  = aws_cloudwatch_event_bus.trips.arn
  description = "Khám phá schema sự kiện chuyến."
}

# EventBridge Scheduler: job đối soát 02:00 theo giờ Việt Nam.
resource "aws_scheduler_schedule_group" "toinoi" {
  name = local.prefix
}

data "aws_iam_policy_document" "scheduler_trust" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["scheduler.amazonaws.com"]
    }

    condition {
      test     = "StringEquals"
      variable = "aws:SourceAccount"
      values   = [data.aws_caller_identity.current.account_id]
    }
  }
}

data "aws_caller_identity" "current" {}

resource "aws_iam_role" "scheduler" {
  name               = "${local.prefix}-scheduler"
  assume_role_policy = data.aws_iam_policy_document.scheduler_trust.json
}

data "aws_iam_policy_document" "scheduler_send" {
  statement {
    effect    = "Allow"
    actions   = ["sqs:SendMessage"]
    resources = [aws_sqs_queue.reconcile.arn]
  }
}

resource "aws_iam_role_policy" "scheduler_send" {
  name   = "send-reconcile"
  role   = aws_iam_role.scheduler.id
  policy = data.aws_iam_policy_document.scheduler_send.json
}

resource "aws_scheduler_schedule" "reconcile_nightly" {
  name       = "${local.prefix}-reconcile-0200"
  group_name = aws_scheduler_schedule_group.toinoi.name

  schedule_expression          = "cron(0 2 * * ? *)"
  schedule_expression_timezone = "Asia/Ho_Chi_Minh"

  flexible_time_window {
    mode                      = "FLEXIBLE"
    maximum_window_in_minutes = 15
  }

  target {
    arn      = aws_sqs_queue.reconcile.arn
    role_arn = aws_iam_role.scheduler.arn
    input    = jsonencode({ job = "reconcile-payments", scope = "previous-day" })

    retry_policy {
      maximum_event_age_in_seconds = 3600
      maximum_retry_attempts       = 5
    }
  }
}
