output "arc_cluster_arn" {
  description = "ARN của ARC cluster; dùng với describe-cluster để lấy endpoint data plane."
  value       = aws_route53recoverycontrolconfig_cluster.main.arn
}

output "primary_routing_control_arn" {
  description = "ARN routing control của region chính."
  value       = aws_route53recoverycontrolconfig_routing_control.primary.arn
}

output "secondary_routing_control_arn" {
  description = "ARN routing control của region phụ."
  value       = aws_route53recoverycontrolconfig_routing_control.secondary.arn
}

output "fis_experiment_template_id" {
  description = "ID experiment template của FIS."
  value       = aws_fis_experiment_template.booking_resilience.id
}

output "stop_condition_alarm_arn" {
  description = "ARN alarm dùng làm stop condition."
  value       = aws_cloudwatch_metric_alarm.booking_5xx.arn
}
