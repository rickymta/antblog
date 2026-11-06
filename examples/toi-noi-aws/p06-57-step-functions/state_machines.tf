resource "aws_cloudwatch_log_group" "saga" {
  name              = "/aws/vendedlogs/states/${local.saga_name}"
  retention_in_days = var.log_retention_days
}

resource "aws_cloudwatch_log_group" "recon" {
  name              = "/aws/vendedlogs/states/${local.recon_name}"
  retention_in_days = var.log_retention_days
}

# Standard: saga chạy lâu (chờ tài xế), cần lịch sử và redrive.
resource "aws_sfn_state_machine" "saga" {
  name     = local.saga_name
  role_arn = aws_iam_role.saga.arn
  type     = "STANDARD"
  publish  = true

  definition = templatefile("${path.module}/asl/booking-saga.asl.json", {
    hold_funds_function_arn      = var.hold_funds_function_arn
    capture_payment_function_arn = var.capture_payment_function_arn
    release_funds_function_arn   = var.release_funds_function_arn
    dispatch_queue_url           = var.dispatch_queue_url
    manual_review_queue_url      = var.manual_review_queue_url
    dispatch_table_name          = var.dispatch_table_name
    event_bus_name               = var.event_bus_name
  })

  logging_configuration {
    log_destination        = "${aws_cloudwatch_log_group.saga.arn}:*"
    include_execution_data = false # input có thông tin khách, không ghi vào log
    level                  = "ERROR"
  }

  tracing_configuration {
    enabled = true
  }
}

# Alias "live" trỏ vào phiên bản đã publish; chia tỉ lệ giữa hai phiên bản để canary.
resource "aws_sfn_alias" "saga_live" {
  name        = "live"
  description = "Phiên bản saga đang nhận chuyến thật."

  routing_configuration {
    state_machine_version_arn = aws_sfn_state_machine.saga.state_machine_version_arn
    weight                    = 100
  }
}

# Job đối soát: Standard cha, child Express do distributed map sinh ra.
resource "aws_sfn_state_machine" "recon" {
  name     = local.recon_name
  role_arn = aws_iam_role.recon.arn
  type     = "STANDARD"

  definition = templatefile("${path.module}/asl/reconcile-nightly.asl.json", {
    recon_bucket_name            = var.recon_bucket_name
    reconcile_batch_function_arn = var.reconcile_batch_function_arn
    event_bus_name               = var.event_bus_name
  })

  logging_configuration {
    log_destination        = "${aws_cloudwatch_log_group.recon.arn}:*"
    include_execution_data = false
    level                  = "ERROR"
  }

  tracing_configuration {
    enabled = true
  }
}
