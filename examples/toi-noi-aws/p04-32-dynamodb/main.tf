locals {
  table_name  = "toinoi-${var.env}-dispatch"
  provisioned = var.billing_mode == "PROVISIONED"
  on_demand   = var.billing_mode == "PAY_PER_REQUEST"
}

# Một bảng cho dữ liệu nóng của dispatch (single-table design):
#   DRIVER#<id> | STATE            trạng thái hiện tại của tài xế, có thuộc tính GSI khi AVAILABLE
#   DRIVER#<id> | LOC#<ISO time>   lịch sử vị trí, hết hạn sau 24 giờ (TTL)
#   TRIP#<id>   | META             bản ghi nhận chuyến: ai thắng khi nhiều tài xế cùng bấm nhận
#   IDEMP#<tok> | ACCEPT           khoá idempotency cho lệnh nhận chuyến
# Bản gốc của chuyến vẫn nằm ở bảng trips trong PostgreSQL (bài 31).
resource "aws_dynamodb_table" "dispatch" {
  name         = local.table_name
  billing_mode = var.billing_mode
  hash_key     = "pk"
  range_key    = "sk"

  read_capacity  = local.provisioned ? var.provisioned_read_units : null
  write_capacity = local.provisioned ? var.provisioned_write_units : null

  attribute {
    name = "pk"
    type = "S"
  }

  attribute {
    name = "sk"
    type = "S"
  }

  attribute {
    name = "gsi1pk"
    type = "S"
  }

  attribute {
    name = "gsi1sk"
    type = "S"
  }

  # Tài xế đang rảnh theo thành phố và ô địa lý. Chỉ giữ khoá (KEYS_ONLY): cập nhật vị trí
  # trong cùng ô không đổi khoá nên không ghi vào index. Muốn lat/lng ngay trong index thì dùng INCLUDE,
  # đổi lại mỗi lần đổi vị trí đều tốn thêm WCU cho index.
  global_secondary_index {
    name            = "gsi1"
    hash_key        = "gsi1pk"
    range_key       = "gsi1sk"
    projection_type = "KEYS_ONLY"
    read_capacity   = local.provisioned ? var.provisioned_read_units : null
    write_capacity  = local.provisioned ? var.provisioned_write_units : null

    dynamic "on_demand_throughput" {
      for_each = local.on_demand && var.max_write_request_units >= 0 ? [1] : []
      content {
        max_read_request_units  = var.max_read_request_units
        max_write_request_units = var.max_write_request_units
      }
    }
  }

  # Thuộc tính expires_at chứa epoch giây. TTL xoá nền, có thể trễ nhiều giờ tới nhiều ngày sau hạn:
  # query vẫn phải lọc expires_at.
  ttl {
    attribute_name = "expires_at"
    enabled        = true
  }

  point_in_time_recovery {
    enabled = true
  }

  stream_enabled   = true
  stream_view_type = "NEW_AND_OLD_IMAGES"

  server_side_encryption {
    enabled = true
  }

  dynamic "on_demand_throughput" {
    for_each = local.on_demand && var.max_write_request_units >= 0 ? [1] : []
    content {
      max_read_request_units  = var.max_read_request_units
      max_write_request_units = var.max_write_request_units
    }
  }

  dynamic "warm_throughput" {
    for_each = var.warm_write_units != null || var.warm_read_units != null ? [1] : []
    content {
      read_units_per_second  = var.warm_read_units
      write_units_per_second = var.warm_write_units
    }
  }

  # Lab cố ý cho xoá nhanh. Production: deletion_protection_enabled = true.
  deletion_protection_enabled = false
}

resource "aws_cloudwatch_metric_alarm" "write_throttle" {
  alarm_name          = "${local.table_name}-write-throttle"
  alarm_description   = "Co ghi bi throttle: kiem tra hot partition hoac tran on-demand"
  namespace           = "AWS/DynamoDB"
  metric_name         = "WriteThrottleEvents"
  statistic           = "Sum"
  period              = 60
  evaluation_periods  = 3
  threshold           = 0
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    TableName = aws_dynamodb_table.dispatch.name
  }
}
