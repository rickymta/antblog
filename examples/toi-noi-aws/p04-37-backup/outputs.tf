output "primary_vault_name" {
  description = "Vault chính."
  value       = aws_backup_vault.primary.name
}

output "dr_vault_arn" {
  description = "ARN vault nhận bản copy ở region thứ hai."
  value       = aws_backup_vault.dr.arn
}

output "backup_plan_id" {
  description = "ID backup plan."
  value       = aws_backup_plan.toinoi.id
}

output "backup_role_arn" {
  description = "Role AWS Backup dùng cho backup, copy và restore."
  value       = aws_iam_role.backup.arn
}

output "alert_topic_arn" {
  description = "Topic SNS nhận cảnh báo job lỗi."
  value       = aws_sns_topic.backup_events.arn
}
