data "aws_caller_identity" "current" {}

locals {
  name        = "toinoi-${var.env}-driver-location"
  bucket_name = "toinoi-${var.env}-gps-history-${data.aws_caller_identity.current.account_id}-${var.region}"
  provisioned = var.stream_mode == "PROVISIONED"
}

# Kinesis Data Streams: partition key là driverId để phân đều và giữ thứ tự theo từng tài xế.
resource "aws_kinesis_stream" "location" {
  name             = local.name
  retention_period = var.retention_hours
  shard_count      = local.provisioned ? var.shard_count : null

  stream_mode_details {
    stream_mode = var.stream_mode
  }

  encryption_type = "KMS"
  kms_key_id      = "alias/aws/kinesis"

  # Số liệu theo shard tính thêm tiền CloudWatch; chỉ bật hai chỉ số cần cho hot shard.
  shard_level_metrics = ["WriteProvisionedThroughputExceeded", "IteratorAgeMilliseconds"]
}

# Consumer enhanced fan-out cho Lambda dispatch: có băng thông đọc riêng 2 MB/s mỗi shard,
# không tranh với Firehose (đọc theo kiểu dùng chung).
resource "aws_kinesis_stream_consumer" "dispatch" {
  name       = "lambda-dispatch"
  stream_arn = aws_kinesis_stream.location.arn
}

resource "aws_cloudwatch_metric_alarm" "write_throttled" {
  alarm_name          = "${local.name}-write-throttled"
  alarm_description   = "Producer bi tu choi ghi: thieu shard hoac hot shard"
  namespace           = "AWS/Kinesis"
  metric_name         = "WriteProvisionedThroughputExceeded"
  statistic           = "Sum"
  period              = 60
  evaluation_periods  = 3
  threshold           = 0
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    StreamName = aws_kinesis_stream.location.name
  }
}

resource "aws_cloudwatch_metric_alarm" "iterator_age" {
  alarm_name          = "${local.name}-iterator-age"
  alarm_description   = "Consumer cham hon 60 giay so voi producer"
  namespace           = "AWS/Kinesis"
  metric_name         = "GetRecords.IteratorAgeMilliseconds"
  statistic           = "Maximum"
  period              = 60
  evaluation_periods  = 5
  threshold           = 60000
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    StreamName = aws_kinesis_stream.location.name
  }
}

# Bucket lịch sử GPS. force_destroy chỉ để lab dọn nhanh; production không đặt.
resource "aws_s3_bucket" "history" {
  bucket        = local.bucket_name
  force_destroy = true
}

resource "aws_s3_bucket_public_access_block" "history" {
  bucket                  = aws_s3_bucket.history.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "history" {
  bucket = aws_s3_bucket.history.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# Glue Data Catalog: lược đồ để Firehose chuyển JSON sang Parquet. Cột city không có ở đây vì
# đã là phần của đường dẫn (partition), bài 38 dùng partition projection để Athena đọc.
resource "aws_glue_catalog_database" "gps" {
  name = "toinoi_${var.env}_gps"
}

resource "aws_glue_catalog_table" "driver_location" {
  name          = "driver_location"
  database_name = aws_glue_catalog_database.gps.name
  table_type    = "EXTERNAL_TABLE"

  parameters = {
    classification = "parquet"
  }

  storage_descriptor {
    location      = "s3://${aws_s3_bucket.history.bucket}/location/"
    input_format  = "org.apache.hadoop.hive.ql.io.parquet.MapredParquetInputFormat"
    output_format = "org.apache.hadoop.hive.ql.io.parquet.MapredParquetOutputFormat"

    ser_de_info {
      serialization_library = "org.apache.hadoop.hive.ql.io.parquet.serde.ParquetHiveSerDe"
    }

    columns {
      name = "driver_id"
      type = "string"
    }

    columns {
      name = "trip_id"
      type = "string"
    }

    columns {
      name = "lat"
      type = "double"
    }

    columns {
      name = "lng"
      type = "double"
    }

    columns {
      name = "speed"
      type = "double"
    }

    columns {
      name = "ts_ms"
      type = "bigint"
    }
  }
}
