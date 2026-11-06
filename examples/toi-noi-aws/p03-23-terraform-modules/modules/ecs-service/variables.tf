variable "name" {
  description = "Tiền tố môi trường, ví dụ toinoi-dev. Tên service được ghép thành <name>-<service_name>."
  type        = string
}

variable "env" {
  description = "Tên môi trường (dev, staging, prod); dùng cho log group /toinoi/<env>/<service>."
  type        = string
}

variable "service_name" {
  description = "Tên service của Tới Nơi: quote, booking, dispatch, tracking."
  type        = string

  validation {
    condition     = can(regex("^[a-z][a-z0-9-]{1,30}$", var.service_name))
    error_message = "service_name chỉ gồm chữ thường, số và dấu gạch ngang."
  }
}

variable "cluster_arn" {
  description = "ARN của ECS cluster dùng chung."
  type        = string
}

variable "vpc_id" {
  description = "VPC của service."
  type        = string
}

variable "subnet_ids" {
  description = "Private subnet cho task Fargate."
  type        = list(string)
}

variable "image" {
  description = "Image đầy đủ trong ECR, nên ghim digest (repo@sha256:...) cho môi trường thật."
  type        = string
}

variable "container_port" {
  description = "Cổng container lắng nghe."
  type        = number
  default     = 8080
}

variable "cpu" {
  description = "CPU units của task Fargate (256, 512, 1024...). Phải đi cặp hợp lệ với memory."
  type        = number
  default     = 256
}

variable "memory" {
  description = "Bộ nhớ MiB của task Fargate."
  type        = number
  default     = 512
}

variable "desired_count" {
  description = "Số task mong muốn."
  type        = number
  default     = 2

  validation {
    condition     = var.desired_count >= 1
    error_message = "desired_count phải >= 1."
  }
}

variable "environment" {
  description = "Biến môi trường (không phải secret) truyền vào container."
  type        = map(string)
  default     = {}
}

variable "db_secret_arn" {
  description = "ARN secret Secrets Manager do RDS quản lý. null nếu service không dùng database."
  type        = string
  default     = null
}

variable "target_group_arn" {
  description = "ARN target group của ALB (bài 13). null nếu service chỉ là worker nội bộ."
  type        = string
  default     = null
}

variable "ingress_cidr_blocks" {
  description = "CIDR được phép gọi vào cổng container. Rỗng nghĩa là không mở ingress."
  type        = list(string)
  default     = []
}

variable "log_retention_days" {
  description = "Số ngày giữ log trong CloudWatch Logs."
  type        = number
  default     = 14
}
