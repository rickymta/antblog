output "key_arn" {
  description = "ARN của key trips ở region chính."
  value       = aws_kms_key.trips.arn
}

output "key_alias" {
  description = "Alias để ứng dụng và script tham chiếu (không dùng key ID trong code)."
  value       = aws_kms_alias.trips.name
}

output "replica_key_arn" {
  description = "ARN replica ở region DR (null nếu tắt)."
  value       = one(aws_kms_replica_key.dr[*].arn)
}

output "bucket_name" {
  description = "Bucket demo SSE-KMS."
  value       = aws_s3_bucket.demo.bucket
}
