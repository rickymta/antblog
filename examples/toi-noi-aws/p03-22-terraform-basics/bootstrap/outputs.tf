output "state_bucket" {
  description = "Tên bucket để điền vào backend S3."
  value       = aws_s3_bucket.state.bucket
}
