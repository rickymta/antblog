variable "region" {
  description = "Region triển khai."
  type        = string
  default     = "ap-southeast-1"
}

variable "aws_profile" {
  description = "Tên profile AWS CLI (IAM Identity Center). null thì dùng chuỗi credential mặc định."
  type        = string
  default     = null
}

variable "env" {
  description = "Tên môi trường; dev, staging hoặc prod."
  type        = string
  default     = "dev"

  validation {
    condition     = contains(["dev", "staging", "prod"], var.env)
    error_message = "env phải là dev, staging hoặc prod."
  }
}

variable "owner" {
  description = "Giá trị tag Owner."
  type        = string
  default     = "platform"
}

variable "vpc_cidr" {
  description = "CIDR của VPC môi trường này."
  type        = string
  default     = "10.10.0.0/16"
}

variable "enable_nat_gateway" {
  description = "Tạo NAT Gateway cho private subnet (tính tiền theo giờ)."
  type        = bool
  default     = true
}

variable "db_multi_az" {
  description = "Bật Multi-AZ cho RDS."
  type        = bool
  default     = false
}

variable "db_deletion_protection" {
  description = "Bật deletion protection cho RDS."
  type        = bool
  default     = false
}

variable "db_skip_final_snapshot" {
  description = "Bỏ snapshot cuối khi xoá RDS. Chỉ true cho lab dev."
  type        = bool
  default     = true
}

variable "ecr_repository_prefix" {
  description = "Tiền tố repository ECR (bài 11)."
  type        = string
  default     = "toinoi"
}

variable "services" {
  description = "Các service chạy trên Fargate. Thêm một khoá vào map là thêm một service, không copy module."
  type = map(object({
    image_tag      = string
    container_port = optional(number, 8080)
    cpu            = optional(number, 256)
    memory         = optional(number, 512)
    desired_count  = optional(number, 2)
    needs_db       = optional(bool, false)
  }))
}
