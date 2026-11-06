# Streams kích hoạt Lambda cập nhật thống kê khi một chuyến chuyển sang ACCEPTED.
# Chỉ tạo khi bạn truyền stats_function_arn (Lambda viết ở bài 18). Role của Lambda cần quyền đọc stream
# (dynamodb:DescribeStream, GetRecords, GetShardIterator, ListStreams) và quyền ghi vào đích lỗi.
resource "aws_lambda_event_source_mapping" "stats" {
  count = var.stats_function_arn == null ? 0 : 1

  event_source_arn  = aws_dynamodb_table.dispatch.stream_arn
  function_name     = var.stats_function_arn
  starting_position = "LATEST"

  batch_size                         = 100
  maximum_batching_window_in_seconds = 5
  parallelization_factor             = 1
  bisect_batch_on_function_error     = true
  maximum_retry_attempts             = 3
  maximum_record_age_in_seconds      = 3600
  function_response_types            = ["ReportBatchItemFailures"]

  # Chỉ gọi Lambda cho bản ghi META của chuyến vừa sang ACCEPTED; bỏ qua toạ độ và TTL.
  filter_criteria {
    filter {
      pattern = jsonencode({
        eventName = ["MODIFY"]
        dynamodb = {
          NewImage = {
            sk     = { S = ["META"] }
            status = { S = ["ACCEPTED"] }
          }
        }
      })
    }
  }

  dynamic "destination_config" {
    for_each = var.stats_failure_destination_arn == null ? [] : [1]
    content {
      on_failure {
        destination_arn = var.stats_failure_destination_arn
      }
    }
  }
}
