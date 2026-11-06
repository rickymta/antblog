output "lake_bucket" {
  description = "Bucket data lake."
  value       = aws_s3_bucket.lake.bucket
}

output "results_bucket" {
  description = "Bucket kết quả Athena."
  value       = aws_s3_bucket.results.bucket
}

output "database_name" {
  description = "Glue database."
  value       = aws_glue_catalog_database.analytics.name
}

output "workgroup_name" {
  description = "Athena workgroup."
  value       = aws_athena_workgroup.analytics.name
}
