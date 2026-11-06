variable "region" {
  description = "Region của cluster và các dịch vụ quan sát."
  type        = string
  default     = "ap-southeast-1"
}

variable "env" {
  description = "Môi trường: lab, dev, staging, prod."
  type        = string
  default     = "lab"
}

variable "cluster_name" {
  description = "Tên cluster EKS đã có (bài 42)."
  type        = string
  default     = "toinoi-lab-eks"
}

variable "namespaces" {
  description = "Namespace theo bounded context; mỗi namespace có một log group riêng."
  type        = list(string)
  default     = ["booking", "dispatch", "tracking"]
}

variable "log_retention_days" {
  description = "Số ngày giữ log ứng dụng trong CloudWatch Logs."
  type        = number
  default     = 14
}

variable "alert_email" {
  description = "Email nhận cảnh báo từ topic SNS (cần xác nhận subscription). Để trống nếu không muốn tạo subscription."
  type        = string
  default     = ""
}

variable "create_grafana" {
  description = "Tạo Amazon Managed Grafana workspace (cần bật IAM Identity Center trước). Mặc định không tạo."
  type        = bool
  default     = false
}

variable "enable_container_insights" {
  description = "Cài add-on amazon-cloudwatch-observability (Container Insights enhanced). Mặc định không bật vì trùng chức năng log với Fluent Bit."
  type        = bool
  default     = false
}
