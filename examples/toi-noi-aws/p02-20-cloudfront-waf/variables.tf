variable "region" {
  description = "Region của bucket web tĩnh."
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

variable "site_bucket_name" {
  description = "Tên bucket web tĩnh. Tên bucket là duy nhất toàn cầu, hãy thêm hậu tố riêng của bạn."
  type        = string
  default     = "toinoi-lab-web-static"
}

variable "waf_log_bucket_name" {
  description = "Tên bucket nhận log WAF. Bắt buộc bắt đầu bằng aws-waf-logs-."
  type        = string
  default     = "aws-waf-logs-toinoi-lab"
}

variable "login_rate_limit" {
  description = "Số request tối đa tới /auth/login từ một IP trong cửa sổ đánh giá trước khi bị chặn (giả định minh hoạ)."
  type        = number
  default     = 100
}

variable "acm_certificate_arn" {
  description = "ARN chứng chỉ ACM ở us-east-1 cho tên miền tuỳ chỉnh. Để trống thì dùng domain mặc định *.cloudfront.net."
  type        = string
  default     = ""
}

variable "aliases" {
  description = "Tên miền tuỳ chỉnh gắn vào distribution (chỉ dùng khi có acm_certificate_arn)."
  type        = list(string)
  default     = []
}

variable "api_origin_domain" {
  description = "DNS name công khai của ALB internet-facing cho API (ví dụ alb-api.toinoi.example, có chứng chỉ khớp tên). ALB internal cần VPC origin, lab không dựng. Để trống thì distribution chỉ phục vụ web tĩnh."
  type        = string
  default     = ""
}

variable "origin_verify_value" {
  description = "Giá trị header X-Origin-Verify CloudFront gửi tới ALB. Sinh ngẫu nhiên, không commit giá trị thật."
  type        = string
  default     = ""
  sensitive   = true
}

variable "alb_security_group_id" {
  description = "Security group của ALB; đặt để chỉ cho phép lưu lượng từ CloudFront (origin-lock.tf)."
  type        = string
  default     = ""
}

variable "alb_listener_arn" {
  description = "ARN listener HTTPS của ALB, dùng cho rule kiểm header X-Origin-Verify."
  type        = string
  default     = ""
}

variable "api_target_group_arn" {
  description = "Target group của service API phía sau ALB."
  type        = string
  default     = ""
}
