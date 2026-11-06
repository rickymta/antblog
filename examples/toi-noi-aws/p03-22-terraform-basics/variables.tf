variable "region" {
  description = "Region triển khai. Mặc định theo quy ước của lộ trình."
  type        = string
  default     = "ap-southeast-1"
}

variable "aws_profile" {
  description = "Tên profile AWS CLI (IAM Identity Center). Để null thì dùng chuỗi credential mặc định."
  type        = string
  default     = null
}

variable "env" {
  description = "Môi trường: lab, dev, staging hoặc prod."
  type        = string
  default     = "lab"

  validation {
    condition     = contains(["lab", "dev", "staging", "prod"], var.env)
    error_message = "env phải là lab, dev, staging hoặc prod."
  }
}

variable "owner" {
  description = "Giá trị tag Owner."
  type        = string
  default     = "platform"
}

variable "vpc_cidr" {
  description = "CIDR của VPC."
  type        = string
  default     = "10.50.0.0/16"
}

variable "az_count" {
  description = "Số AZ dùng cho subnet (mỗi AZ có một public và một private subnet)."
  type        = number
  default     = 2

  validation {
    condition     = var.az_count >= 2 && var.az_count <= 3
    error_message = "az_count nên là 2 hoặc 3."
  }
}

variable "force_destroy_photos_bucket" {
  description = "true cho phép terraform destroy xoá cả bucket còn object. Chỉ dùng cho lab; production giữ false."
  type        = bool
  default     = false
}
