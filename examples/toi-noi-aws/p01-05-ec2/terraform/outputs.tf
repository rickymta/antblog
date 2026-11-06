output "instance_id" {
  description = "ID của instance quote."
  value       = aws_instance.quote.id
}

output "session_command" {
  description = "Lệnh mở phiên Session Manager (cần session-manager-plugin trên máy bạn)."
  value       = "aws ssm start-session --target ${aws_instance.quote.id}"
}
