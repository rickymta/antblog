variable "region" {
  description = "Region của lab. IAM là dịch vụ toàn cục, nhưng Access Analyzer là dịch vụ theo region."
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
  default     = "platform"
}

variable "teams" {
  description = "Các team Tới Nơi; mỗi team nhận một role gắn tag Team và dùng chung policy ABAC."
  type        = set(string)
  default     = ["dispatch", "booking"]
}

variable "partner_account_id" {
  description = "Account ID 12 chữ số của đối tác đối soát (VíXanh) được phép assume role. Mặc định là account minh hoạ; hãy đổi trước khi dùng thật."
  type        = string
  default     = "123456789012"

  validation {
    condition     = can(regex("^[0-9]{12}$", var.partner_account_id))
    error_message = "partner_account_id phải gồm đúng 12 chữ số."
  }
}

variable "partner_external_id" {
  description = "External ID do hai bên thoả thuận (ngẫu nhiên, dài, không đoán được). Không commit giá trị thật."
  type        = string
  sensitive   = true

  validation {
    condition     = length(var.partner_external_id) >= 16
    error_message = "partner_external_id nên dài ít nhất 16 ký tự."
  }
}

variable "enable_unused_access_analyzer" {
  description = "Bật analyzer unused access. Loại analyzer này tính phí theo số role và user được phân tích mỗi tháng; analyzer external access không tính phí."
  type        = bool
  default     = true
}

variable "unused_access_age_days" {
  description = "Số ngày không dùng thì quyền hoặc credential bị coi là thừa."
  type        = number
  default     = 90
}
