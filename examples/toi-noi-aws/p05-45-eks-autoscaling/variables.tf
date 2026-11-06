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

variable "private_subnet_ids" {
  description = "ID các private subnet mà Karpenter được đặt node vào (cùng giá trị lab p05-42)."
  type        = list(string)

  validation {
    condition     = length(var.private_subnet_ids) >= 2
    error_message = "Cần ít nhất hai private subnet ở hai AZ khác nhau."
  }
}

variable "node_security_group_id" {
  description = "Security group của node (output node_security_group_id của lab p05-42); node do Karpenter tạo dùng lại nó."
  type        = string
}

variable "dispatch_queue_name" {
  description = "Tên hàng đợi SQS mà dispatch-worker đọc (bài 16); KEDA được cấp quyền đọc thuộc tính của hàng đợi này."
  type        = string
  default     = "toinoi-lab-dispatch-jobs"
}
