output "service_name" {
  description = "Tên ECS service."
  value       = aws_ecs_service.this.name
}

output "security_group_id" {
  description = "Security group của task (để database cho phép kết nối)."
  value       = aws_security_group.service.id
}

output "execution_role_arn" {
  description = "ARN execution role."
  value       = aws_iam_role.execution.arn
}

output "task_role_arn" {
  description = "ARN task role."
  value       = aws_iam_role.task.arn
}

output "task_definition_arn" {
  description = "ARN task definition hiện tại."
  value       = aws_ecs_task_definition.this.arn
}
