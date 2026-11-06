variable "region" {
  description = "Region chứa bucket state."
  type        = string
  default     = "ap-southeast-1"
}

variable "aws_profile" {
  description = "Tên profile AWS CLI (IAM Identity Center)."
  type        = string
  default     = null
}

variable "owner" {
  description = "Giá trị tag Owner."
  type        = string
  default     = "platform"
}

variable "state_access_principal_arns" {
  description = <<-EOT
    Danh sách ARN role (cho phép wildcard) được đọc/ghi state. Khi không rỗng, mọi principal khác bị từ chối.
    Phải gồm cả role bạn đang dùng. Role Identity Center có path, ví dụ
    arn:aws:iam::123456789012:role/aws-reserved/sso.amazonaws.com/*/AWSReservedSSO_AdministratorAccess_*
    Precondition trong main.tf từ chối apply nếu role hiện tại không khớp danh sách.
  EOT
  type        = list(string)
  default     = []
}
