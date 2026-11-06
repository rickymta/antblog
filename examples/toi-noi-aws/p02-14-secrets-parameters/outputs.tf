output "parameter_path" {
  description = "Đường dẫn gốc của parameter, dùng với GetParametersByPath."
  value       = local.prefix
}

output "db_secret_arn" {
  description = "ARN của secret DB; dùng làm valueFrom trong task definition (bài 12)."
  value       = aws_secretsmanager_secret.db.arn
}

output "kms_key_arn" {
  value = aws_kms_key.config.arn
}

output "reader_role_arn" {
  value = aws_iam_role.reader.arn
}

output "reader_policy_arn" {
  description = "Policy đọc cấu hình; gắn vào execution role (bài 12) nếu muốn ECS inject."
  value       = aws_iam_policy.reader.arn
}
