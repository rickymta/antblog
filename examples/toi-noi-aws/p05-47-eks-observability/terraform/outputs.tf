output "amp_workspace_id" {
  description = "ID workspace Prometheus."
  value       = aws_prometheus_workspace.this.id
}

output "amp_remote_write_url" {
  description = "URL remote write để dán vào cấu hình ADOT collector."
  value       = "${aws_prometheus_workspace.this.prometheus_endpoint}api/v1/remote_write"
}

output "alerts_topic_arn" {
  description = "Topic SNS nhận cảnh báo."
  value       = aws_sns_topic.alerts.arn
}

output "log_groups" {
  description = "Log group theo namespace."
  value       = [for g in aws_cloudwatch_log_group.app : g.name]
}
