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
  description = "Team sở hữu tài nguyên (tag Owner)."
  type        = string
  default     = "data-platform"
}

variable "cities" {
  description = "Mã sáu thành phố (chữ thường như các bài 32, 35), dùng làm giá trị enum cho partition projection."
  type        = list(string)
  default     = ["hcm", "hn", "dn", "hp", "ct", "nt"]
}

variable "projection_start_date" {
  description = "Ngày bắt đầu của partition projection (yyyy-MM-dd)."
  type        = string
  default     = "2026-01-01"
}

variable "scan_limit_bytes" {
  description = "Trần dữ liệu quét mỗi query của workgroup (byte). Tối thiểu 10000000 (10 MB). Mặc định 10 GB."
  type        = number
  default     = 10737418240
}

variable "analyst_role_arn" {
  description = "Role của nhóm phân tích sẽ bị che cột customer_phone bằng Lake Formation. Rỗng thì không tạo cấu hình Lake Formation."
  type        = string
  default     = ""
}
