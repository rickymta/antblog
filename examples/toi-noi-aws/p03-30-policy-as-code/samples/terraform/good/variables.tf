variable "region" {
  description = "Region của mẫu. Chỉ dùng để chạy công cụ kiểm tĩnh, không tạo tài nguyên."
  type        = string
  default     = "ap-southeast-1"
}

variable "vpc_id" {
  description = "VPC giả định chứa database."
  type        = string
  default     = "vpc-0123456789abcdef0"
}

variable "db_subnet_group_name" {
  description = "DB subnet group private giả định."
  type        = string
  default     = "toinoi-lab-private"
}

variable "db_subnet_cidr" {
  description = "CIDR của subnet ứng dụng được phép vào cổng 5432 (bản good)."
  type        = string
  default     = "10.20.0.0/20"
}
