variable "region" {
  description = "Region của lab."
  type        = string
  default     = "ap-southeast-1"
}

variable "env" {
  description = "Môi trường: lab, dev, staging hoặc prod."
  type        = string
  default     = "lab"
}

variable "owner" {
  description = "Team sở hữu, dùng cho tag Owner."
  type        = string
  default     = "platform"
}

variable "max_receive_count" {
  description = "Số lần một message được nhận trước khi chuyển sang DLQ."
  type        = number
  default     = 5
}

variable "visibility_timeout_seconds" {
  description = "Visibility timeout của hai queue chính. Phải lớn hơn thời gian xử lý lâu nhất của consumer."
  type        = number
  default     = 60
}

variable "alert_email" {
  description = "Email nhận thông báo từ topic. Để trống thì không tạo subscription email."
  type        = string
  default     = ""
}
