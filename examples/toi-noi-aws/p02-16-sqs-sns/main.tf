locals {
  prefix = "toinoi-${var.env}"

  # Hai consumer của sự kiện BookingCreated.
  # dispatch-push: chỉ nhận chuyến ride và food ở HCM, HN để gửi push cho tài xế.
  # stats: nhận mọi chuyến để cập nhật thống kê.
  consumers = {
    dispatch-push = {
      filter = {
        city    = ["HCM", "HN"]
        service = ["ride", "food"]
      }
    }
    stats = {
      filter = null
    }
  }
}

resource "aws_sns_topic" "booking_events" {
  name = "${local.prefix}-booking-events"
}

# DLQ giữ message lâu hơn queue chính: thời gian giữ tính từ lúc message vào queue gốc.
resource "aws_sqs_queue" "dlq" {
  for_each = local.consumers

  name                      = "${local.prefix}-${each.key}-dlq"
  message_retention_seconds = 1209600
  sqs_managed_sse_enabled   = true
}

resource "aws_sqs_queue" "main" {
  for_each = local.consumers

  name                       = "${local.prefix}-${each.key}"
  visibility_timeout_seconds = var.visibility_timeout_seconds
  receive_wait_time_seconds  = 20
  message_retention_seconds  = 345600
  sqs_managed_sse_enabled    = true

  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.dlq[each.key].arn
    maxReceiveCount     = var.max_receive_count
  })
}

# Chỉ queue chính này được phép đẩy message vào DLQ tương ứng.
resource "aws_sqs_queue_redrive_allow_policy" "dlq" {
  for_each  = local.consumers
  queue_url = aws_sqs_queue.dlq[each.key].id

  redrive_allow_policy = jsonencode({
    redrivePermission = "byQueue"
    sourceQueueArns   = [aws_sqs_queue.main[each.key].arn]
  })
}

# Cho đúng topic này (và không topic nào khác) gửi vào queue.
data "aws_iam_policy_document" "sns_to_sqs" {
  for_each = local.consumers

  statement {
    sid       = "AllowBookingTopicSend"
    effect    = "Allow"
    actions   = ["sqs:SendMessage"]
    resources = [aws_sqs_queue.main[each.key].arn]

    principals {
      type        = "Service"
      identifiers = ["sns.amazonaws.com"]
    }

    condition {
      test     = "ArnEquals"
      variable = "aws:SourceArn"
      values   = [aws_sns_topic.booking_events.arn]
    }
  }
}

resource "aws_sqs_queue_policy" "main" {
  for_each  = local.consumers
  queue_url = aws_sqs_queue.main[each.key].id
  policy    = data.aws_iam_policy_document.sns_to_sqs[each.key].json
}

resource "aws_sns_topic_subscription" "queue" {
  for_each = local.consumers

  topic_arn            = aws_sns_topic.booking_events.arn
  protocol             = "sqs"
  endpoint             = aws_sqs_queue.main[each.key].arn
  raw_message_delivery = true

  filter_policy_scope = each.value.filter == null ? null : "MessageAttributes"
  filter_policy       = each.value.filter == null ? null : jsonencode(each.value.filter)

  depends_on = [aws_sqs_queue_policy.main]
}

resource "aws_sns_topic_subscription" "email" {
  count = var.alert_email == "" ? 0 : 1

  topic_arn = aws_sns_topic.booking_events.arn
  protocol  = "email"
  endpoint  = var.alert_email
}
