variable "region" {
  description = "Region chính, nơi có tài nguyên cần backup."
  type        = string
  default     = "ap-southeast-1"
}

variable "dr_region" {
  description = "Region đặt vault nhận bản copy. Phải khác region chính."
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

variable "backup_tag_key" {
  description = "Tag key để chọn tài nguyên cần backup."
  type        = string
  default     = "Backup"
}

variable "backup_tag_value" {
  description = "Giá trị tag để vào kế hoạch backup hằng ngày."
  type        = string
  default     = "daily"
}

variable "daily_retention_days" {
  description = "Số ngày giữ backup hằng ngày ở vault chính."
  type        = number
  default     = 35
}

variable "dr_retention_days" {
  description = "Số ngày giữ bản copy ở vault region thứ hai."
  type        = number
  default     = 90
}

variable "enable_vault_lock" {
  description = "Bật Vault Lock cho vault chính. Mặc định tắt: khi hết thời gian cooling-off, khoá trở thành bất biến và không gỡ được."
  type        = bool
  default     = false
}

variable "vault_lock_changeable_for_days" {
  description = "Số ngày cooling-off trước khi Vault Lock thành compliance mode (tối thiểu 3). Trong thời gian này vẫn xoá được khoá."
  type        = number
  default     = 3
}

variable "alert_email" {
  description = "Email nhận cảnh báo job backup lỗi (rỗng nghĩa là không đăng ký email)."
  type        = string
  default     = ""
}

variable "restore_test_resource_arns" {
  description = "ARN các instance RDS dùng trong restore testing (rỗng thì chọn theo tag)."
  type        = list(string)
  default     = []
}
