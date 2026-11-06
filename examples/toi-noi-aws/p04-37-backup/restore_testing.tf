# Restore testing: mỗi tuần AWS Backup tự restore thử một recovery point mới nhất,
# đo thời gian và (tuỳ chọn) chạy bước kiểm của bạn, rồi xoá tài nguyên restore.
resource "aws_backup_restore_testing_plan" "weekly" {
  name                         = "toinoi_${var.env}_weekly_restore"
  schedule_expression          = "cron(0 20 ? * SUN *)"
  schedule_expression_timezone = "Asia/Ho_Chi_Minh"
  start_window_hours           = 6

  recovery_point_selection {
    algorithm             = "LATEST_WITHIN_WINDOW"
    include_vaults        = [aws_backup_vault.primary.arn]
    recovery_point_types  = ["SNAPSHOT"]
    selection_window_days = 7
  }
}

resource "aws_backup_restore_testing_selection" "rds" {
  name                      = "toinoi_${var.env}_rds_trips"
  restore_testing_plan_name = aws_backup_restore_testing_plan.weekly.name
  protected_resource_type   = "RDS"
  iam_role_arn              = aws_iam_role.backup.arn
  validation_window_hours   = 1

  protected_resource_arns = length(var.restore_test_resource_arns) > 0 ? var.restore_test_resource_arns : null

  dynamic "protected_resource_conditions" {
    for_each = length(var.restore_test_resource_arns) == 0 ? [1] : []
    content {
      string_equals {
        key   = "aws:ResourceTag/${var.backup_tag_key}"
        value = var.backup_tag_value
      }
    }
  }
}
