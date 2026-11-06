variable "region" {
  description = "Region của cả loạt bài."
  type        = string
  default     = "ap-southeast-1"
}

variable "env" {
  description = "Tên môi trường, dùng trong tag Env và tên tài nguyên."
  type        = string
  default     = "lab"

  validation {
    condition     = contains(["lab", "dev", "staging", "prod"], var.env)
    error_message = "env phải là lab, dev, staging hoặc prod."
  }
}

variable "owner" {
  description = "Tag Owner: nhóm chịu trách nhiệm."
  type        = string
  default     = "platform"
}

variable "vpc_cidr" {
  description = "Dải CIDR của VPC. Các CIDR subnet trong main.tf giả định 10.20.0.0/16."
  type        = string
  default     = "10.20.0.0/16"
}

variable "enable_nat_gateway" {
  description = "Tạo một NAT Gateway và Elastic IP. TÍNH TIỀN theo giờ và theo GB dữ liệu."
  type        = bool
  default     = false
}

variable "enable_data_nacl" {
  description = "Gắn NACL tuỳ chỉnh cho hai subnet private-data."
  type        = bool
  default     = false
}
