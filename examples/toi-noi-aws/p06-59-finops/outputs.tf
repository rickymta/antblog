output "budget_name" {
  description = "Tên budget tháng theo tag Project."
  value       = aws_budgets_budget.project_monthly.name
}

output "anomaly_monitor_arns" {
  description = "ARN hai anomaly monitor."
  value = [
    aws_ce_anomaly_monitor.by_service.arn,
    aws_ce_anomaly_monitor.by_tag.arn,
  ]
}

output "cost_category_arn" {
  description = "ARN Cost Category theo lớp."
  value       = aws_ce_cost_category.layer.arn
}
