# Lambda dispatch đọc stream qua enhanced fan-out và cập nhật vị trí mới nhất vào DynamoDB (bài 32).
# Chỉ tạo khi bạn truyền consumer_function_arn. Role của Lambda cần quyền
# kinesis:DescribeStreamSummary, ListShards, SubscribeToShard, DescribeStreamConsumer, và quyền ghi vào đích lỗi.
resource "aws_lambda_event_source_mapping" "dispatch" {
  count = var.consumer_function_arn == null ? 0 : 1

  event_source_arn  = aws_kinesis_stream_consumer.dispatch.arn
  function_name     = var.consumer_function_arn
  starting_position = "LATEST"

  batch_size                         = 500
  maximum_batching_window_in_seconds = 2
  parallelization_factor             = 2
  bisect_batch_on_function_error     = true
  maximum_retry_attempts             = 3
  maximum_record_age_in_seconds      = 300
  function_response_types            = ["ReportBatchItemFailures"]

  dynamic "destination_config" {
    for_each = var.consumer_failure_destination_arn == null ? [] : [1]
    content {
      on_failure {
        destination_arn = var.consumer_failure_destination_arn
      }
    }
  }
}
