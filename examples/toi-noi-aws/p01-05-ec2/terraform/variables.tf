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

variable "architecture" {
  description = "Kiến trúc CPU: x86_64 hoặc arm64 (Graviton). Image container phải có bản tương ứng."
  type        = string
  default     = "x86_64"

  validation {
    condition     = contains(["x86_64", "arm64"], var.architecture)
    error_message = "architecture phải là x86_64 hoặc arm64."
  }
}

variable "instance_type" {
  description = "Loại máy. Để null thì dùng t3.micro (x86_64) hoặc t4g.micro (arm64)."
  type        = string
  default     = null
}

variable "subnet_id" {
  description = "Subnet đặt instance (ví dụ subnet private-app của VPC bài 04, cần NAT hoặc VPC endpoint). Null thì dùng default VPC."
  type        = string
  default     = null
}
