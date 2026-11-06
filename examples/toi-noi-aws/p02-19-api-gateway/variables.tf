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

variable "jwt_issuer" {
  description = "Issuer của identity provider (URL có /.well-known/openid-configuration). Giá trị mặc định chỉ để minh hoạ; request thật sẽ bị 401 cho tới khi bạn trỏ tới một issuer hợp lệ, ví dụ user pool Cognito."
  type        = string
  default     = "https://id.toinoi.example/"
}

variable "jwt_audience" {
  description = "Danh sách audience chấp nhận (claim aud hoặc client_id trong token)."
  type        = list(string)
  default     = ["toinoi-app"]
}

variable "cors_allow_origins" {
  description = "Origin được phép gọi API từ trình duyệt."
  type        = list(string)
  default     = ["https://app.toinoi.example"]
}

variable "route_rate_limit" {
  description = "Giới hạn tốc độ (request/giây) cho route GET /v1/quotes."
  type        = number
  default     = 50
}

variable "route_burst_limit" {
  description = "Giới hạn burst cho route GET /v1/quotes."
  type        = number
  default     = 100
}

variable "domain_name" {
  description = "Tên miền tuỳ chỉnh cho HTTP API."
  type        = string
  default     = "api.toinoi.example"
}

variable "certificate_arn" {
  description = "ARN chứng chỉ ACM (cùng region với API, vì đây là regional endpoint). Để trống thì không tạo custom domain."
  type        = string
  default     = ""
}
