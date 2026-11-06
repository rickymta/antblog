# Che cột PII bằng Lake Formation. Chỉ tạo khi cung cấp analyst_role_arn.
# Cảnh báo: bật Lake Formation enforcement cho bảng làm thay đổi cách cấp quyền; thử ở tài khoản lab.
resource "aws_lakeformation_resource" "lake" {
  count = var.analyst_role_arn == "" ? 0 : 1

  arn                     = aws_s3_bucket.lake.arn
  use_service_linked_role = true
}

resource "aws_lakeformation_permissions" "analyst_select_without_pii" {
  count = var.analyst_role_arn == "" ? 0 : 1

  principal   = var.analyst_role_arn
  permissions = ["SELECT"]

  table_with_columns {
    database_name         = aws_glue_catalog_database.analytics.name
    name                  = aws_glue_catalog_table.trips_curated.name
    wildcard              = true
    excluded_column_names = ["customer_phone"]
  }
}
