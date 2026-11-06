output "cluster_name" {
  value = aws_ecs_cluster.main.name
}

output "execution_role_arn" {
  value = aws_iam_role.execution.arn
}

output "task_role_arn" {
  value = aws_iam_role.task.arn
}

output "log_group_name" {
  value = aws_cloudwatch_log_group.service.name
}

output "task_security_group_id" {
  value = aws_security_group.task.id
}

output "task_definition_arn" {
  value = one(aws_ecs_task_definition.booking[*].arn)
}
