# S3 Tables: table bucket lưu bảng Iceberg do S3 tự compaction. Chưa có bảng nào được tạo ở đây;
# bảng tạo bằng Athena/Spark hoặc `aws s3tables create-table` (xem README).
resource "aws_s3tables_table_bucket" "trips" {
  count = var.create_table_bucket ? 1 : 0

  name = "toinoi-${var.env}-trips-tables"
}

resource "aws_s3tables_namespace" "trips" {
  count = var.create_table_bucket ? 1 : 0

  namespace        = "trips"
  table_bucket_arn = aws_s3tables_table_bucket.trips[0].arn
}
