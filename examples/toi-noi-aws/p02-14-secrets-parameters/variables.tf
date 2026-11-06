variable "region" {
  description = "Region triển khai."
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

variable "reader_principal_arn" {
  description = "ARN của principal được assume role đọc cấu hình trong lab (ví dụ role SSO của bạn). Giá trị mẫu: arn:aws:iam::123456789012:role/aws-reserved/sso.amazonaws.com/ap-southeast-1/AWSReservedSSO_Lab_0123456789abcdef"
  type        = string
}
