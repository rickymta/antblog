output "configuration_endpoint" {
  description = "Endpoint cấu hình của cluster mode: client cluster-aware dùng địa chỉ này để khám phá shard."
  value       = aws_elasticache_replication_group.main.configuration_endpoint_address
}

output "replication_group_id" {
  value = aws_elasticache_replication_group.main.replication_group_id
}

output "memorydb_cluster_endpoint" {
  description = "Endpoint của cluster MemoryDB (nếu create_memorydb = true)."
  value       = var.create_memorydb ? aws_memorydb_cluster.dispatch[0].cluster_endpoint[0].address : null
}
