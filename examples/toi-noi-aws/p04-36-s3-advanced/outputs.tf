output "photos_bucket" {
  description = "Bucket ảnh chứng từ."
  value       = aws_s3_bucket.photos.bucket
}

output "invoices_bucket" {
  description = "Bucket hoá đơn có Object Lock."
  value       = aws_s3_bucket.invoices.bucket
}

output "replica_bucket" {
  description = "Bucket bản sao ở region thứ hai."
  value       = aws_s3_bucket.replica.bucket
}

output "proof_queue_url" {
  description = "Hàng đợi nhận event ảnh chứng từ mới qua EventBridge."
  value       = aws_sqs_queue.proof_uploaded.url
}

output "table_bucket_arn" {
  description = "ARN của S3 table bucket (rỗng nếu create_table_bucket = false)."
  value       = var.create_table_bucket ? aws_s3tables_table_bucket.trips[0].arn : null
}
