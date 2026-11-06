output "logs_bucket" {
  description = "Bucket nhận log CloudTrail và Config."
  value       = aws_s3_bucket.logs.bucket
}

output "trail_name" {
  description = "Tên trail, dùng với `aws cloudtrail get-trail-status`."
  value       = aws_cloudtrail.main.name
}

output "alert_topic_arn" {
  description = "SNS topic nhận cảnh báo bảo mật."
  value       = aws_sns_topic.alerts.arn
}

output "event_data_store_arn" {
  description = "ARN event data store CloudTrail Lake (null nếu tắt)."
  value       = one(aws_cloudtrail_event_data_store.lake[*].arn)
}
