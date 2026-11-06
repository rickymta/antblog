output "log_group_name" {
  description = "Log group của service booking."
  value       = aws_cloudwatch_log_group.booking.name
}

output "oncall_topic_arn" {
  description = "SNS topic mà composite alarm gọi."
  value       = aws_sns_topic.oncall.arn
}

output "composite_alarm_name" {
  description = "Tên composite alarm gọi on-call."
  value       = aws_cloudwatch_composite_alarm.booking_page.alarm_name
}

output "monthly_error_budget_minutes" {
  description = "Số phút lỗi cho phép trong 30 ngày với slo_target hiện tại."
  value       = 30 * 24 * 60 * (1 - var.slo_target)
}
