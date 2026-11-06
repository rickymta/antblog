output "collection_endpoint" {
  description = "Endpoint của OpenSearch Serverless collection (rỗng nếu mode = domain)."
  value       = local.is_serverless ? aws_opensearchserverless_collection.search[0].collection_endpoint : null
}

output "domain_endpoint" {
  description = "Endpoint của domain (rỗng nếu mode = serverless)."
  value       = local.is_domain ? aws_opensearch_domain.search[0].endpoint : null
}
