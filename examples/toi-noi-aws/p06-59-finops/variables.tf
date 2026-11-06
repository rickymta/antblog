variable "region" {
  description = "Region của provider. Các API chi phí là toàn cầu, endpoint ở us-east-1."
  type        = string
  default     = "us-east-1"
}

variable "env" {
  description = "Môi trường: lab, dev, staging hoặc prod."
  type        = string
  default     = "lab"
}

variable "owner" {
  description = "Team sở hữu tài nguyên (tag Owner)."
  type        = string
  default     = "finops"
}

variable "alert_emails" {
  description = "Email nhận cảnh báo budget và anomaly. Đổi sang hộp thư của bạn trước khi apply."
  type        = list(string)
  default     = ["finops@toinoi.example"]
}

variable "monthly_limit" {
  description = "Hạn mức budget tháng (USD) cho tài nguyên gắn Project=ToiNoi. Con số do bạn chọn, không phải khuyến nghị."
  type        = string
  default     = "100"
}

variable "anomaly_threshold" {
  description = "Ngưỡng tác động tuyệt đối (USD) để gửi cảnh báo anomaly. Do bạn chọn."
  type        = number
  default     = 20
}

variable "enable_commitment_budgets" {
  description = "Tạo budget coverage và utilization của Savings Plans. Chỉ bật sau khi đã mua Savings Plans."
  type        = bool
  default     = false
}

variable "commitment_utilization_floor" {
  description = "Cảnh báo khi utilization của Savings Plans thấp hơn mức này (phần trăm)."
  type        = number
  default     = 90
}

variable "commitment_coverage_floor" {
  description = "Cảnh báo khi coverage của Savings Plans thấp hơn mức này (phần trăm)."
  type        = number
  default     = 70
}
