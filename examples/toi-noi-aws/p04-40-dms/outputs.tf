output "source_endpoint_arn" {
  description = "ARN endpoint nguồn."
  value       = aws_dms_endpoint.source.endpoint_arn
}

output "target_endpoint_arn" {
  description = "ARN endpoint đích."
  value       = aws_dms_endpoint.target.endpoint_arn
}

output "replication_task_arn" {
  description = "ARN replication task (rỗng nếu dùng Serverless)."
  value       = var.use_serverless ? null : aws_dms_replication_task.trips[0].replication_task_arn
}

output "replication_config_arn" {
  description = "ARN replication config của DMS Serverless (rỗng nếu dùng instance)."
  value       = var.use_serverless ? aws_dms_replication_config.trips[0].arn : null
}
