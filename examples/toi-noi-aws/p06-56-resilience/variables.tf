variable "region" {
  description = "Region chính của workload."
  type        = string
  default     = "ap-southeast-1"
}

variable "dr_region" {
  description = "Region phụ (chỉ dùng để ghi chú tên; lab không tạo tài nguyên ở đó)."
  type        = string
  default     = "ap-northeast-1"
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

variable "hosted_zone_id" {
  description = "ID hosted zone Route 53 của tên miền (ví dụ toinoi.example). Giá trị mặc định chỉ để validate."
  type        = string
  default     = "Z0000000000000000000"
}

variable "record_name" {
  description = "Tên bản ghi failover."
  type        = string
  default     = "api.toinoi.example"
}

variable "primary_alb_dns_name" {
  description = "DNS name của ALB ở region chính."
  type        = string
  default     = "toinoi-lab-booking-alb-1.ap-southeast-1.elb.amazonaws.com"
}

variable "primary_alb_zone_id" {
  description = "Hosted zone ID của ALB ở region chính (output canonical_hosted_zone_id)."
  type        = string
  default     = "Z00000000PRIMARY"
}

variable "secondary_alb_dns_name" {
  description = "DNS name của ALB ở region phụ."
  type        = string
  default     = "toinoi-lab-booking-alb-2.ap-northeast-1.elb.amazonaws.com"
}

variable "secondary_alb_zone_id" {
  description = "Hosted zone ID của ALB ở region phụ."
  type        = string
  default     = "Z00000000SECOND"
}

variable "alb_arn_suffix" {
  description = "Phần cuối ARN của ALB chính (app/<tên>/<id>), dùng cho dimension LoadBalancer của alarm."
  type        = string
  default     = "app/toinoi-lab-booking-alb/0123456789abcdef"
}

variable "ecs_cluster_name" {
  description = "Tên ECS cluster có service booking (bài 12)."
  type        = string
  default     = "toinoi-lab"
}

variable "ecs_service_name" {
  description = "Tên ECS service booking."
  type        = string
  default     = "toinoi-lab-booking"
}

variable "ecs_cluster_arn" {
  description = "ARN của ECS cluster, để giới hạn quyền của role FIS."
  type        = string
  default     = "arn:aws:ecs:ap-southeast-1:123456789012:cluster/toinoi-lab"
}

variable "private_subnet_tag_value" {
  description = "Giá trị tag Tier của các subnet private bị làm mất kết nối khi thử AZ."
  type        = string
  default     = "private"
}

variable "target_az" {
  description = "AZ bị mô phỏng gián đoạn trong thí nghiệm mạng."
  type        = string
  default     = "ap-southeast-1c"
}

variable "stop_task_percent" {
  description = "Phần trăm task của booking bị dừng trong thí nghiệm."
  type        = number
  default     = 30
}

variable "experiment_duration" {
  description = "Thời lượng gián đoạn mạng (ISO 8601), ví dụ PT5M."
  type        = string
  default     = "PT5M"
}

variable "error_alarm_threshold" {
  description = "Số lỗi 5xx của target trong một phút làm kích hoạt stop condition."
  type        = number
  default     = 200
}
