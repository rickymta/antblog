output "connection_arn" {
  description = "Hoàn tất kết nối này trong console trước khi chạy pipeline (đang ở trạng thái PENDING)."
  value       = aws_codeconnections_connection.github.arn
}

output "pipeline_name" {
  value = aws_codepipeline.this.name
}

output "codedeploy_application" {
  value = aws_codedeploy_app.this.name
}

output "codedeploy_deployment_group" {
  value = aws_codedeploy_deployment_group.this.deployment_group_name
}

output "smoke_hook_function" {
  description = "Tên hàm dùng trong Hooks của appspec.yaml."
  value       = aws_lambda_function.smoke.function_name
}
