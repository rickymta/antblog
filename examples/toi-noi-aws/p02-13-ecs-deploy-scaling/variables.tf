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

variable "service" {
  description = "Tên service."
  type        = string
  default     = "booking"
}

variable "vpc_id" {
  description = "VPC của lab (bài 04)."
  type        = string
}

variable "public_subnet_ids" {
  description = "Public subnet cho ALB, ít nhất hai AZ."
  type        = list(string)
}

variable "private_subnet_ids" {
  description = "Private subnet cho task Fargate."
  type        = list(string)
}

variable "cluster_name" {
  description = "Cluster ECS đã tạo ở lab p02-12."
  type        = string
  default     = "toinoi-lab"
}

variable "task_definition_arn" {
  description = "ARN task definition (kèm revision) từ lab p02-12. Container phải tên booking, cổng 8080."
  type        = string
}

variable "desired_count" {
  description = "Số task ban đầu. Sau đó Application Auto Scaling điều chỉnh."
  type        = number
  default     = 4
}

variable "min_percent" {
  description = "minimumHealthyPercent của rolling update."
  type        = number
  default     = 100
}

variable "max_percent" {
  description = "maximumPercent của rolling update."
  type        = number
  default     = 200
}

variable "capacity_min" {
  description = "Số task tối thiểu ngoài giờ cao điểm."
  type        = number
  default     = 2
}

variable "capacity_max" {
  description = "Số task tối đa cho Application Auto Scaling."
  type        = number
  default     = 20
}

variable "peak_min" {
  description = "Số task tối thiểu trong khung cao điểm."
  type        = number
  default     = 6
}

variable "requests_per_target" {
  description = "Ngưỡng ALBRequestCountPerTarget (request mỗi task mỗi phút). Giả định, phải đo bằng load test."
  type        = number
  default     = 12000
}

variable "alarm_5xx_threshold" {
  description = "Số lỗi 5xx từ target trong một phút để alarm và kích hoạt rollback deploy."
  type        = number
  default     = 50
}
