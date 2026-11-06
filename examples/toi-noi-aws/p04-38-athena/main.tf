data "aws_caller_identity" "current" {}

locals {
  account_id     = data.aws_caller_identity.current.account_id
  lake_bucket    = "toinoi-${var.env}-lake-${local.account_id}"
  results_bucket = "toinoi-${var.env}-athena-results-${local.account_id}"
  database_name  = "toinoi_${var.env}_analytics"
  curated_path   = "s3://${local.lake_bucket}/curated/trips/"
}

# Bucket data lake: raw/ (JSON từ Firehose), curated/ (Parquet theo dt, city), serving/ (bảng tổng hợp).
resource "aws_s3_bucket" "lake" {
  bucket = local.lake_bucket

  tags = {
    Service = "analytics"
  }
}

resource "aws_s3_bucket_public_access_block" "lake" {
  bucket                  = aws_s3_bucket.lake.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# Bucket kết quả Athena: dọn tự động sau 7 ngày để không phình.
resource "aws_s3_bucket" "results" {
  bucket = local.results_bucket

  tags = {
    Service = "analytics"
  }
}

resource "aws_s3_bucket_public_access_block" "results" {
  bucket                  = aws_s3_bucket.results.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_lifecycle_configuration" "results" {
  bucket = aws_s3_bucket.results.id

  rule {
    id     = "expire-athena-results"
    status = "Enabled"

    filter {
      prefix = "athena/"
    }

    expiration {
      days = 7
    }
  }
}

resource "aws_glue_catalog_database" "analytics" {
  name        = local.database_name
  description = "Lakehouse phân tích chuyến của Tới Nơi (lab bài 38)"
}

# Bảng Parquet đã partition theo dt và city. Dùng partition projection nên không cần crawler
# hay MSCK REPAIR TABLE: Athena tự suy ra đường dẫn partition từ template.
resource "aws_glue_catalog_table" "trips_curated" {
  name          = "trips_curated"
  database_name = aws_glue_catalog_database.analytics.name
  table_type    = "EXTERNAL_TABLE"

  parameters = {
    "classification"              = "parquet"
    "parquet.compression"         = "ZSTD"
    "projection.enabled"          = "true"
    "projection.dt.type"          = "date"
    "projection.dt.format"        = "yyyy-MM-dd"
    "projection.dt.range"         = "${var.projection_start_date},NOW"
    "projection.dt.interval"      = "1"
    "projection.dt.interval.unit" = "DAYS"
    "projection.city.type"        = "enum"
    "projection.city.values"      = join(",", var.cities)
    "storage.location.template"   = "${local.curated_path}dt=$${dt}/city=$${city}/"
    "has_encrypted_data"          = "false"
  }

  partition_keys {
    name = "dt"
    type = "string"
  }

  partition_keys {
    name = "city"
    type = "string"
  }

  storage_descriptor {
    location      = local.curated_path
    input_format  = "org.apache.hadoop.hive.ql.io.parquet.MapredParquetInputFormat"
    output_format = "org.apache.hadoop.hive.ql.io.parquet.MapredParquetOutputFormat"

    ser_de_info {
      name                  = "parquet"
      serialization_library = "org.apache.hadoop.hive.ql.io.parquet.serde.ParquetHiveSerDe"
      parameters = {
        "serialization.format" = "1"
      }
    }

    columns {
      name = "trip_id"
      type = "string"
    }
    columns {
      name = "code"
      type = "bigint"
    }
    columns {
      name = "kind"
      type = "string"
    }
    columns {
      name = "customer_id"
      type = "string"
    }
    columns {
      name    = "customer_phone"
      type    = "string"
      comment = "PII: che bằng Lake Formation cho nhóm phân tích"
    }
    columns {
      name = "driver_id"
      type = "string"
    }
    columns {
      name = "merchant_id"
      type = "string"
    }
    columns {
      name = "status"
      type = "string"
    }
    columns {
      name    = "fare"
      type    = "int"
      comment = "đồng"
    }
    columns {
      name = "requested_at"
      type = "timestamp"
    }
    columns {
      name = "accepted_at"
      type = "timestamp"
    }
    columns {
      name = "completed_at"
      type = "timestamp"
    }
  }
}
