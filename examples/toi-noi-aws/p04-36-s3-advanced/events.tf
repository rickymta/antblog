# Bật gửi mọi event của bucket ảnh sang EventBridge. Từ đây lọc và định tuyến nằm ở rule, không nằm ở bucket.
resource "aws_s3_bucket_notification" "photos" {
  bucket      = aws_s3_bucket.photos.id
  eventbridge = true
}

resource "aws_sqs_queue" "proof_uploaded" {
  name                      = "toinoi-${var.env}-proof-uploaded"
  message_retention_seconds = 345600
  sqs_managed_sse_enabled   = true

  tags = {
    Service = "delivery-photo"
  }
}

resource "aws_cloudwatch_event_rule" "proof_uploaded" {
  name        = "toinoi-${var.env}-proof-uploaded"
  description = "Ảnh chứng từ mới trong prefix proof/ của bucket delivery-photos"

  event_pattern = jsonencode({
    source        = ["aws.s3"]
    "detail-type" = ["Object Created"]
    detail = {
      bucket = { name = [aws_s3_bucket.photos.bucket] }
      object = {
        key  = [{ prefix = "proof/" }]
        size = [{ numeric = [">", 0] }]
      }
    }
  })
}

resource "aws_cloudwatch_event_target" "proof_uploaded_sqs" {
  rule = aws_cloudwatch_event_rule.proof_uploaded.name
  arn  = aws_sqs_queue.proof_uploaded.arn
}

data "aws_iam_policy_document" "queue_from_eventbridge" {
  statement {
    sid       = "AllowEventBridgeRule"
    actions   = ["sqs:SendMessage"]
    resources = [aws_sqs_queue.proof_uploaded.arn]

    principals {
      type        = "Service"
      identifiers = ["events.amazonaws.com"]
    }

    condition {
      test     = "ArnEquals"
      variable = "aws:SourceArn"
      values   = [aws_cloudwatch_event_rule.proof_uploaded.arn]
    }
  }
}

resource "aws_sqs_queue_policy" "proof_uploaded" {
  queue_url = aws_sqs_queue.proof_uploaded.id
  policy    = data.aws_iam_policy_document.queue_from_eventbridge.json
}
