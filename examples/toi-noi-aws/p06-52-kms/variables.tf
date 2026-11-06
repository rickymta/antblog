variable "region" {
  description = "Region chính."
  type        = string
  default     = "ap-southeast-1"
}

variable "dr_region" {
  description = "Region đặt replica của multi-region key. Phải khác region chính."
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

variable "key_admin_arns" {
  description = "ARN các role quản trị key (tạo, tắt, xoá, sửa policy), nhưng không dùng key để mã hoá hay giải mã. Role phải tồn tại sẵn. Để trống thì chỉ IAM của tài khoản quản trị key."
  type        = list(string)
  default     = []
}

variable "key_user_arns" {
  description = "ARN các role của service booking được dùng key trực tiếp, chỉ khi encryption context có Service=booking. Role phải tồn tại sẵn."
  type        = list(string)
  default     = []
}

variable "enable_multi_region_replica" {
  description = "Tạo replica ở dr_region. Mỗi key (primary và replica) tính phí hằng tháng riêng."
  type        = bool
  default     = true
}

variable "deletion_window_days" {
  description = "Số ngày chờ trước khi key bị xoá thật sau khi lên lịch xoá."
  type        = number
  default     = 7

  validation {
    condition     = var.deletion_window_days >= 7 && var.deletion_window_days <= 30
    error_message = "deletion_window_days phải từ 7 đến 30."
  }
}
