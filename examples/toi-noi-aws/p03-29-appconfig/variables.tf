variable "region" {
  description = "Region của AppConfig và Lambda."
  type        = string
  default     = "ap-southeast-1"
}

variable "env" {
  description = "Môi trường: lab, dev, staging hoặc prod. Dùng làm tên environment AppConfig."
  type        = string
  default     = "lab"
}

variable "owner" {
  description = "Team sở hữu, dùng cho tag Owner."
  type        = string
  default     = "platform"
}

variable "fast_lab" {
  description = "true: triển khai bằng chiến lược AppConfig.AllAtOnce để lab chạy nhanh. false: dùng chiến lược linear 20 phút kèm bake 10 phút."
  type        = bool
  default     = true
}

variable "enable_alarm_rollback" {
  description = "Gắn CloudWatch alarm vào environment để AppConfig tự rollback khi alarm kêu trong lúc deploy hoặc bake."
  type        = bool
  default     = true
}

variable "appconfig_extension_layer_arn" {
  description = "ARN layer AWS AppConfig Agent Lambda extension theo region (lấy trong user guide của AppConfig). Để trống thì không tạo Lambda đọc cấu hình."
  type        = string
  default     = ""
}
