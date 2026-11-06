data "aws_iam_policy_document" "firehose_assume" {
  statement {
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["firehose.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "firehose" {
  statement {
    sid       = "ReadStream"
    actions   = ["kinesis:DescribeStream", "kinesis:GetShardIterator", "kinesis:GetRecords", "kinesis:ListShards"]
    resources = [aws_kinesis_stream.location.arn]
  }

  statement {
    sid = "WriteBucket"
    actions = [
      "s3:AbortMultipartUpload",
      "s3:GetBucketLocation",
      "s3:GetObject",
      "s3:ListBucket",
      "s3:ListBucketMultipartUploads",
      "s3:PutObject",
    ]
    resources = [aws_s3_bucket.history.arn, "${aws_s3_bucket.history.arn}/*"]
  }

  statement {
    sid     = "ReadSchema"
    actions = ["glue:GetTable", "glue:GetTableVersion", "glue:GetTableVersions"]
    resources = [
      "arn:aws:glue:${var.region}:${data.aws_caller_identity.current.account_id}:catalog",
      aws_glue_catalog_database.gps.arn,
      aws_glue_catalog_table.driver_location.arn,
    ]
  }

  statement {
    sid       = "Logs"
    actions   = ["logs:PutLogEvents"]
    resources = ["${aws_cloudwatch_log_group.firehose.arn}:*"]
  }
}

resource "aws_iam_role" "firehose" {
  name               = "${local.name}-firehose"
  assume_role_policy = data.aws_iam_policy_document.firehose_assume.json
}

resource "aws_iam_role_policy" "firehose" {
  name   = "stream-to-s3"
  role   = aws_iam_role.firehose.id
  policy = data.aws_iam_policy_document.firehose.json
}

resource "aws_cloudwatch_log_group" "firehose" {
  name              = "/toinoi/${var.env}/firehose/${local.name}"
  retention_in_days = 14
}

resource "aws_cloudwatch_log_stream" "firehose" {
  name           = "S3Delivery"
  log_group_name = aws_cloudwatch_log_group.firehose.name
}

# Amazon Data Firehose (tên cũ Kinesis Data Firehose): đọc stream, trích city từ JSON,
# chuyển sang Parquet theo lược đồ Glue và ghi S3 theo city, ngày, giờ.
# Dynamic partitioning phải bật ngay khi tạo; không bật thêm được sau.
resource "aws_kinesis_firehose_delivery_stream" "gps" {
  name        = "${local.name}-to-s3"
  destination = "extended_s3"

  kinesis_source_configuration {
    kinesis_stream_arn = aws_kinesis_stream.location.arn
    role_arn           = aws_iam_role.firehose.arn
  }

  extended_s3_configuration {
    role_arn   = aws_iam_role.firehose.arn
    bucket_arn = aws_s3_bucket.history.arn

    # Ngày và giờ trong đường dẫn là thời điểm Firehose nhận bản ghi, không phải thời điểm tài xế gửi.
    prefix              = "location/city=!{partitionKeyFromQuery:city}/dt=!{timestamp:yyyy-MM-dd}/hour=!{timestamp:HH}/"
    error_output_prefix = "errors/!{firehose:error-output-type}/dt=!{timestamp:yyyy-MM-dd}/"

    buffering_size     = var.firehose_buffer_mb
    buffering_interval = var.firehose_buffer_seconds
    compression_format = "UNCOMPRESSED"

    dynamic_partitioning_configuration {
      enabled = true
    }

    processing_configuration {
      enabled = true

      processors {
        type = "MetadataExtraction"

        parameters {
          parameter_name  = "MetadataExtractionQuery"
          parameter_value = "{city: .city}"
        }

        parameters {
          parameter_name  = "JsonParsingEngine"
          parameter_value = "JQ-1.6"
        }
      }
    }

    data_format_conversion_configuration {
      input_format_configuration {
        deserializer {
          open_x_json_ser_de {}
        }
      }

      output_format_configuration {
        serializer {
          parquet_ser_de {
            compression = "SNAPPY"
          }
        }
      }

      schema_configuration {
        database_name = aws_glue_catalog_database.gps.name
        table_name    = aws_glue_catalog_table.driver_location.name
        role_arn      = aws_iam_role.firehose.arn
        region        = var.region
      }
    }

    cloudwatch_logging_options {
      enabled         = true
      log_group_name  = aws_cloudwatch_log_group.firehose.name
      log_stream_name = aws_cloudwatch_log_stream.firehose.name
    }
  }

  depends_on = [aws_iam_role_policy.firehose]
}

resource "aws_cloudwatch_metric_alarm" "firehose_freshness" {
  alarm_name          = "${local.name}-firehose-freshness"
  alarm_description   = "Du lieu cu nhat chua toi S3 qua 20 phut"
  namespace           = "AWS/Firehose"
  metric_name         = "DeliveryToS3.DataFreshness"
  statistic           = "Maximum"
  period              = 300
  evaluation_periods  = 2
  threshold           = 1200
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    DeliveryStreamName = aws_kinesis_firehose_delivery_stream.gps.name
  }
}
