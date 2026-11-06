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
  description = "Private subnet đặt cache, nên trải trên ít nhất hai AZ."
  type        = list(string)
}

variable "client_security_group_id" {
  description = "Security group của nơi gọi cache (task ECS hoặc EC2 thử nghiệm). Chỉ nhóm này được vào cổng 6379."
  type        = string
}

variable "mode" {
  description = "node: replication group theo node (có chọn node type). serverless: ElastiCache Serverless."
  type        = string
  default     = "node"

  validation {
    condition     = contains(["node", "serverless"], var.mode)
    error_message = "mode phải là node hoặc serverless."
  }
}

variable "engine" {
  description = "Engine: valkey hoặc redis."
  type        = string
  default     = "valkey"
}

variable "engine_version" {
  description = "Phiên bản engine cho mode node."
  type        = string
  default     = "8.0"
}

variable "major_engine_version" {
  description = "Phiên bản chính cho mode serverless."
  type        = string
  default     = "8"
}

variable "parameter_group_name" {
  description = "Parameter group mặc định khớp engine_version (ví dụ default.valkey8, default.redis7)."
  type        = string
  default     = "default.valkey8"
}

variable "node_type" {
  description = "Node type cho mode node."
  type        = string
  default     = "cache.t4g.micro"
}

variable "num_cache_clusters" {
  description = "Số node (1 primary + replica). 2 trở lên mới bật được failover tự động."
  type        = number
  default     = 2
}

variable "auth_token" {
  description = "AUTH token (mode node, cần TLS). null: không đặt AUTH, chỉ dựa vào security group và TLS. Đừng commit giá trị thật."
  type        = string
  default     = null
  sensitive   = true
}
