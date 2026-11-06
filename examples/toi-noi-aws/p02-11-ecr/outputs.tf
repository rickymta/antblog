output "repository_url" {
  description = "URL repository, dùng làm tiền tố khi docker tag/push."
  value       = aws_ecr_repository.service.repository_url
}

output "repository_arn" {
  description = "ARN repository, dùng trong IAM policy."
  value       = aws_ecr_repository.service.arn
}
