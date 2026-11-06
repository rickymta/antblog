output "address" {
  description = "DNS name của instance."
  value       = aws_db_instance.this.address
}

output "port" {
  description = "Cổng PostgreSQL."
  value       = aws_db_instance.this.port
}

output "db_name" {
  description = "Tên database mặc định."
  value       = aws_db_instance.this.db_name
}

output "master_user_secret_arn" {
  description = "ARN secret Secrets Manager chứa mật khẩu master do RDS quản lý."
  value       = aws_db_instance.this.master_user_secret[0].secret_arn
}

output "security_group_id" {
  description = "Security group của database."
  value       = aws_security_group.db.id
}
