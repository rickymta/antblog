variable "name" {
  description = "Tiền tố tên tài nguyên, ví dụ toinoi-dev."
  type        = string
}

variable "vpc_cidr" {
  description = "CIDR của VPC (nên là /16)."
  type        = string

  validation {
    condition     = can(cidrhost(var.vpc_cidr, 0))
    error_message = "vpc_cidr phải là CIDR hợp lệ."
  }
}

variable "az_count" {
  description = "Số AZ (2 hoặc 3). Mỗi AZ có một public và một private subnet."
  type        = number
  default     = 2

  validation {
    condition     = var.az_count >= 2 && var.az_count <= 3
    error_message = "az_count phải là 2 hoặc 3."
  }
}

variable "enable_nat_gateway" {
  description = "Tạo một NAT Gateway (ở AZ đầu tiên) cho private subnet. Tính tiền theo giờ; không có NAT thì task Fargate trong private subnet không kéo được image trừ khi có VPC endpoint."
  type        = bool
  default     = false
}
