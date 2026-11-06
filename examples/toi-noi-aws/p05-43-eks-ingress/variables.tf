variable "region" {
  description = "Region triển khai."
  type        = string
  default     = "ap-southeast-1"
}

variable "aws_profile" {
  description = "Tên profile AWS CLI (IAM Identity Center). null thì dùng chuỗi credential mặc định."
  type        = string
  default     = null
}

variable "env" {
  description = "Tên môi trường; cluster là toinoi-<env>-eks (đã dựng ở lab p05-42)."
  type        = string
  default     = "lab"
}

variable "owner" {
  description = "Giá trị tag Owner."
  type        = string
  default     = "platform"
}

variable "hosted_zone_arn" {
  description = "ARN hosted zone Route 53 của toinoi.example mà ExternalDNS được phép ghi (bài 09)."
  type        = string
  default     = "arn:aws:route53:::hostedzone/Z0123456789EXAMPLE"
}
