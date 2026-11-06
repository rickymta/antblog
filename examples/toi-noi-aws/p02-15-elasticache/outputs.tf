output "endpoint" {
  description = "Địa chỉ kết nối (cổng 6379, bắt buộc TLS)."
  value = try(
    aws_elasticache_replication_group.node[0].primary_endpoint_address,
    aws_elasticache_serverless_cache.serverless[0].endpoint[0].address,
  )
}

output "cache_security_group_id" {
  value = aws_security_group.cache.id
}
