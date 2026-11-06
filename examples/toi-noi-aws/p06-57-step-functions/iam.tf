data "aws_caller_identity" "current" {}

locals {
  saga_name  = "toinoi-${var.env}-booking-saga"
  recon_name = "toinoi-${var.env}-reconcile-nightly"

  recon_state_machine_arn = "arn:aws:states:${var.region}:${data.aws_caller_identity.current.account_id}:stateMachine:${local.recon_name}"
  recon_execution_arn     = "arn:aws:states:${var.region}:${data.aws_caller_identity.current.account_id}:execution:${local.recon_name}/*"
}

data "aws_iam_policy_document" "sfn_assume" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["states.amazonaws.com"]
    }

    # Chỉ Step Functions của tài khoản này được assume role.
    condition {
      test     = "StringEquals"
      variable = "aws:SourceAccount"
      values   = [data.aws_caller_identity.current.account_id]
    }
  }
}

# Cặp quyền ghi log và trace mà mọi state machine cần.
data "aws_iam_policy_document" "observability" {
  statement {
    sid = "LogDelivery"
    actions = [
      "logs:CreateLogDelivery",
      "logs:GetLogDelivery",
      "logs:UpdateLogDelivery",
      "logs:DeleteLogDelivery",
      "logs:ListLogDeliveries",
      "logs:PutResourcePolicy",
      "logs:DescribeResourcePolicies",
      "logs:DescribeLogGroups",
    ]
    resources = ["*"]
  }

  statement {
    sid       = "XRay"
    actions   = ["xray:PutTraceSegments", "xray:PutTelemetryRecords", "xray:GetSamplingRules", "xray:GetSamplingTargets"]
    resources = ["*"]
  }
}

# Role của saga: chỉ đúng những gì file ASL gọi.
data "aws_iam_policy_document" "saga" {
  source_policy_documents = [data.aws_iam_policy_document.observability.json]

  statement {
    sid     = "InvokePaymentFunctions"
    actions = ["lambda:InvokeFunction"]
    resources = [
      var.hold_funds_function_arn,
      "${var.hold_funds_function_arn}:*",
      var.capture_payment_function_arn,
      "${var.capture_payment_function_arn}:*",
      var.release_funds_function_arn,
      "${var.release_funds_function_arn}:*",
    ]
  }

  statement {
    sid       = "SendToQueues"
    actions   = ["sqs:SendMessage"]
    resources = [var.dispatch_queue_arn, var.manual_review_queue_arn]
  }

  statement {
    sid       = "UpdateTripState"
    actions   = ["dynamodb:UpdateItem"]
    resources = [var.dispatch_table_arn]
  }

  statement {
    sid       = "PublishEvents"
    actions   = ["events:PutEvents"]
    resources = [var.event_bus_arn]
  }
}

# Role của job đối soát: đọc manifest, ghi kết quả, chạy Lambda, tự chạy child execution.
data "aws_iam_policy_document" "recon" {
  source_policy_documents = [data.aws_iam_policy_document.observability.json]

  statement {
    sid       = "ReadManifests"
    actions   = ["s3:GetObject"]
    resources = ["arn:aws:s3:::${var.recon_bucket_name}/manifests/*"]
  }

  statement {
    sid       = "ListBucket"
    actions   = ["s3:ListBucket"]
    resources = ["arn:aws:s3:::${var.recon_bucket_name}"]
  }

  statement {
    sid       = "WriteResults"
    actions   = ["s3:PutObject"]
    resources = ["arn:aws:s3:::${var.recon_bucket_name}/results/*"]
  }

  statement {
    sid       = "InvokeReconcile"
    actions   = ["lambda:InvokeFunction"]
    resources = [var.reconcile_batch_function_arn, "${var.reconcile_batch_function_arn}:*"]
  }

  statement {
    sid       = "PublishEvents"
    actions   = ["events:PutEvents"]
    resources = [var.event_bus_arn]
  }

  statement {
    sid       = "StartChildExecutions"
    actions   = ["states:StartExecution"]
    resources = [local.recon_state_machine_arn]
  }

  statement {
    sid       = "ManageChildExecutions"
    actions   = ["states:DescribeExecution", "states:StopExecution"]
    resources = [local.recon_execution_arn]
  }
}

resource "aws_iam_role" "saga" {
  name               = "${local.saga_name}-role"
  assume_role_policy = data.aws_iam_policy_document.sfn_assume.json
}

resource "aws_iam_role_policy" "saga" {
  name   = "${local.saga_name}-policy"
  role   = aws_iam_role.saga.id
  policy = data.aws_iam_policy_document.saga.json
}

resource "aws_iam_role" "recon" {
  name               = "${local.recon_name}-role"
  assume_role_policy = data.aws_iam_policy_document.sfn_assume.json
}

resource "aws_iam_role_policy" "recon" {
  name   = "${local.recon_name}-policy"
  role   = aws_iam_role.recon.id
  policy = data.aws_iam_policy_document.recon.json
}
