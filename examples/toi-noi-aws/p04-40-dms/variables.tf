variable "region" {
  description = "Region của lab."
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

variable "subnet_ids" {
  description = "Private subnet (ít nhất hai AZ) đặt replication instance hoặc cấu hình DMS Serverless."
  type        = list(string)
}

variable "security_group_ids" {
  description = "Security group của DMS: cần ra được cổng 5432 tới nguồn và đích."
  type        = list(string)
}

variable "source_server_name" {
  description = "Địa chỉ PostgreSQL nguồn (tự vận hành trên EC2), chỉ là hostname hoặc IP riêng, không phải secret."
  type        = string
}

variable "source_database_name" {
  description = "Tên database nguồn."
  type        = string
  default     = "toinoi"
}

variable "source_secret_arn" {
  description = "ARN secret Secrets Manager chứa username và password của user DMS trên nguồn."
  type        = string
}

variable "target_server_name" {
  description = "Writer endpoint của cluster Aurora PostgreSQL đích."
  type        = string
}

variable "target_database_name" {
  description = "Tên database đích."
  type        = string
  default     = "toinoi"
}

variable "target_secret_arn" {
  description = "ARN secret Secrets Manager chứa username và password của user DMS trên đích."
  type        = string
}

variable "use_serverless" {
  description = "true: dùng DMS Serverless (aws_dms_replication_config). false: replication instance."
  type        = bool
  default     = false
}

variable "instance_class" {
  description = "Lớp replication instance (chỉ dùng khi use_serverless = false). Chọn theo kết quả đo, không theo cảm tính."
  type        = string
  default     = "dms.t3.medium"
}

variable "allocated_storage_gb" {
  description = "Dung lượng đĩa replication instance (GB) cho log và thay đổi được cache."
  type        = number
  default     = 100
}

variable "multi_az" {
  description = "Replication instance Multi-AZ (đắt gấp đôi, chỉ bật khi cần chịu lỗi trong lúc CDC dài)."
  type        = bool
  default     = false
}

variable "min_capacity_units" {
  description = "DMS Serverless: số DMS Capacity Unit tối thiểu."
  type        = number
  default     = 2
}

variable "max_capacity_units" {
  description = "DMS Serverless: số DMS Capacity Unit tối đa (trần chi phí)."
  type        = number
  default     = 16
}

variable "start_task" {
  description = "Tự khởi động task khi apply. Mặc định false: bạn khởi động thủ công sau khi kiểm nguồn và đích."
  type        = bool
  default     = false
}

variable "create_service_roles" {
  description = "Tạo hai role mức tài khoản dms-vpc-role và dms-cloudwatch-logs-role. Chỉ bật nếu tài khoản chưa có (tạo trùng sẽ lỗi)."
  type        = bool
  default     = false
}
