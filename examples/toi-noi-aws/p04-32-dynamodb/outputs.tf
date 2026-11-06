output "table_name" {
  description = "Tên bảng dispatch."
  value       = aws_dynamodb_table.dispatch.name
}

output "table_arn" {
  description = "ARN bảng; dùng trong IAM policy của service dispatch và tracking."
  value       = aws_dynamodb_table.dispatch.arn
}

output "stream_arn" {
  description = "ARN của DynamoDB Stream."
  value       = aws_dynamodb_table.dispatch.stream_arn
}
