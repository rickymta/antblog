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
  description = "VPC chứa task (từ bài 04)."
  type        = string
}

variable "private_subnet_ids" {
  description = "Các private subnet cho task, nên trải trên ít nhất hai AZ."
  type        = list(string)
}

variable "image" {
  description = "Tham chiếu image theo digest, dạng <registry>/toinoi/booking@sha256:... (từ bài 11)."
  type        = string
}

variable "db_secret_arn" {
  description = "ARN đầy đủ của secret chứa nguyên connection string Npgsql (Secrets Manager). Không phải secret JSON của lab p02-14."
  type        = string
}

variable "kms_key_arn" {
  description = "ARN KMS key mã hoá secret nếu dùng customer managed key; null nếu dùng khoá mặc định."
  type        = string
  default     = null
}

variable "photos_bucket" {
  description = "Bucket ảnh chứng từ giao hàng (từ bài 06)."
  type        = string
  default     = "toinoi-lab-delivery-photos"
}

variable "auth_authority" {
  description = "Issuer của identity provider (giá trị minh hoạ)."
  type        = string
  default     = "https://id.toinoi.example"
}

variable "auth_audience" {
  description = "Audience của JWT (giá trị minh hoạ)."
  type        = string
  default     = "toinoi-api"
}

variable "desired_count" {
  description = "Số task mong muốn."
  type        = number
  default     = 1
}

variable "manage_workload" {
  description = "true: Terraform tạo cả task definition và service. false: chỉ tạo nền (cluster, log group, role, security group) để bạn tự chạy lệnh AWS CLI trong scripts/."
  type        = bool
  default     = true
}
