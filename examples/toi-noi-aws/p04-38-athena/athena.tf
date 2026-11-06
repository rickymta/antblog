# Workgroup riêng cho phân tích: ép cấu hình, đặt trần dữ liệu quét mỗi query, bật metric.
resource "aws_athena_workgroup" "analytics" {
  name          = "toinoi-${var.env}-analytics"
  description   = "Phân tích chuyến Tới Nơi; trần quét dữ liệu mỗi query"
  force_destroy = true

  configuration {
    enforce_workgroup_configuration    = true
    publish_cloudwatch_metrics_enabled = true
    bytes_scanned_cutoff_per_query     = var.scan_limit_bytes

    result_configuration {
      output_location = "s3://${aws_s3_bucket.results.bucket}/athena/"

      encryption_configuration {
        encryption_option = "SSE_S3"
      }
    }
  }

  tags = {
    Service = "analytics"
  }
}

resource "aws_athena_named_query" "revenue_by_city_hour" {
  name        = "revenue-by-city-hour"
  description = "Doanh thu theo thành phố và giờ cho một ngày; luôn lọc dt để cắt partition"
  workgroup   = aws_athena_workgroup.analytics.id
  database    = aws_glue_catalog_database.analytics.name

  query = <<-SQL
    SELECT city,
           hour(requested_at) AS hour_of_day,
           count(*)           AS trips,
           sum(fare)          AS revenue
    FROM trips_curated
    WHERE dt = '2026-03-02'
      AND status = 'completed'
    GROUP BY city, hour(requested_at)
    ORDER BY city, hour_of_day
  SQL
}
