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
  default     = "platform"
}

variable "alb_arn_suffix" {
  description = "Phần cuối ARN của ALB trước service booking (dạng app/<tên>/<id>), dùng làm dimension LoadBalancer. Giá trị mặc định chỉ để minh hoạ."
  type        = string
  default     = "app/toinoi-lab-alb/0123456789abcdef"
}

variable "slo_target" {
  description = "Mục tiêu availability của SLO, dạng tỉ lệ (0.999 = 99,9%)."
  type        = number
  default     = 0.999

  validation {
    condition     = var.slo_target > 0 && var.slo_target < 1
    error_message = "slo_target phải nằm trong khoảng (0, 1)."
  }
}

variable "latency_threshold_seconds" {
  description = "Ngưỡng latency p99 của API đặt chuyến, tính bằng giây (0.8 = 800 ms)."
  type        = number
  default     = 0.8
}

variable "alert_email" {
  description = "Email nhận cảnh báo; để trống thì chỉ tạo SNS topic. Email phải xác nhận subscription."
  type        = string
  default     = ""
}

variable "log_retention_days" {
  description = "Số ngày giữ log của service booking."
  type        = number
  default     = 30
}

variable "enable_canary" {
  description = "Tạo Synthetics canary gọi API báo giá mỗi 5 phút. Canary tính phí theo số lần chạy."
  type        = bool
  default     = false
}

variable "canary_runtime_version" {
  description = "Runtime của canary (Node.js + Puppeteer). Mã canary dùng namespace @aws/synthetics-* nên cần syn-nodejs-puppeteer-13.1 trở lên. Tra bản mới nhất bằng `aws synthetics describe-runtime-versions`."
  type        = string
  default     = "syn-nodejs-puppeteer-18.0"
}

variable "canary_target_host" {
  description = "Tên miền API mà canary gọi."
  type        = string
  default     = "api.toinoi.example"
}

variable "enable_rum" {
  description = "Tạo CloudWatch RUM app monitor cho web đặt chuyến. RUM tính phí theo số event."
  type        = bool
  default     = false
}
