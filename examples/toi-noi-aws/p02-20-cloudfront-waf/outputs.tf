output "distribution_id" {
  value = aws_cloudfront_distribution.main.id
}

output "distribution_domain_name" {
  value = aws_cloudfront_distribution.main.domain_name
}

output "web_acl_arn" {
  value = aws_wafv2_web_acl.edge.arn
}

output "site_bucket" {
  value = aws_s3_bucket.site.id
}
