variable "region" {
  description = "Region triển khai."
  type        = string
  default     = "ap-southeast-1"
}

variable "env" {
  description = "Môi trường: lab, dev, staging hoặc prod."
  type        = string
  default     = "lab"
}

variable "owner" {
  description = "Team sở hữu tài nguyên (tag Owner)."
  type        = string
  default     = "platform"
}

variable "vpc_id" {
  description = "VPC của lab (bài 04)."
  type        = string
}

variable "private_subnet_ids" {
  description = "Private subnet đặt cache, nên trải trên ít nhất hai AZ (tốt nhất ba)."
  type        = list(string)

  validation {
    condition     = length(var.private_subnet_ids) >= 2
    error_message = "Cần ít nhất hai subnet ở hai AZ khác nhau."
  }
}

variable "client_security_group_id" {
  description = "Security group của nơi gọi cache. Chỉ nhóm này được vào cổng 6379 (ElastiCache) và 6379 (MemoryDB)."
  type        = string
}

variable "engine" {
  description = "Engine của ElastiCache: valkey hoặc redis."
  type        = string
  default     = "valkey"
}

variable "engine_version" {
  description = "Phiên bản engine ElastiCache. Kiểm danh sách hiện có: aws elasticache describe-cache-engine-versions --engine valkey."
  type        = string
  default     = "8.0"
}

variable "parameter_group_family" {
  description = "Family của parameter group, phải khớp engine và phiên bản (ví dụ valkey8, redis7)."
  type        = string
  default     = "valkey8"
}

variable "node_type" {
  description = "Loại node ElastiCache."
  type        = string
  default     = "cache.t4g.micro"
}

variable "num_node_groups" {
  description = "Số shard (node group) khi bật cluster mode. Đổi giá trị này là resharding online."
  type        = number
  default     = 2
}

variable "replicas_per_node_group" {
  description = "Số replica mỗi shard. Từ 1 trở lên mới có failover tự động."
  type        = number
  default     = 1

  validation {
    condition     = var.replicas_per_node_group >= 1
    error_message = "Cần ít nhất 1 replica mỗi shard để có automatic failover."
  }
}

variable "maxmemory_policy" {
  description = "Chính sách loại khoá khi hết bộ nhớ: allkeys-lru, volatile-lru, volatile-ttl, noeviction..."
  type        = string
  default     = "allkeys-lru"
}

variable "reserved_memory_percent" {
  description = "Phần trăm bộ nhớ node để dành cho overhead, fork khi snapshot và bộ đệm; không dành cho dữ liệu."
  type        = number
  default     = 25
}

variable "create_memorydb" {
  description = "true để tạo thêm cluster MemoryDB cho trạng thái dispatch cần bền vững."
  type        = bool
  default     = false
}

variable "memorydb_node_type" {
  description = "Loại node MemoryDB."
  type        = string
  default     = "db.t4g.small"
}

variable "memorydb_engine_version" {
  description = "Phiên bản engine MemoryDB (Valkey). Kiểm tra bản được hỗ trợ trong User Guide của MemoryDB."
  type        = string
  default     = "7.2"
}

variable "memorydb_num_shards" {
  description = "Số shard của MemoryDB."
  type        = number
  default     = 1
}

variable "memorydb_replicas_per_shard" {
  description = "Số replica mỗi shard của MemoryDB. Dữ liệu bền nhờ transaction log đa AZ kể cả khi bằng 0, nhưng replica quyết định tốc độ failover và khả năng đọc."
  type        = number
  default     = 1
}
