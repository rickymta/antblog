output "stream_name" {
  description = "Tên Kinesis stream."
  value       = aws_kinesis_stream.location.name
}

output "stream_arn" {
  value = aws_kinesis_stream.location.arn
}

output "consumer_arn" {
  description = "ARN consumer enhanced fan-out cho Lambda dispatch."
  value       = aws_kinesis_stream_consumer.dispatch.arn
}

output "firehose_name" {
  value = aws_kinesis_firehose_delivery_stream.gps.name
}

output "bucket_name" {
  description = "Bucket chứa Parquet lịch sử GPS."
  value       = aws_s3_bucket.history.bucket
}
