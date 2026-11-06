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

variable "vpc_id" {
  description = "ID VPC của cluster (cùng giá trị đã dùng ở lab p05-42)."
  type        = string
}

variable "private_subnet_ids" {
  description = "ID các private subnet; EFS đặt một mount target ở mỗi subnet, mỗi AZ một subnet."
  type        = list(string)

  validation {
    condition     = length(var.private_subnet_ids) >= 2
    error_message = "Cần ít nhất hai private subnet ở hai AZ khác nhau."
  }
}

variable "node_security_group_id" {
  description = "Security group của node (output node_security_group_id của lab p05-42); chỉ security group này được vào cổng NFS 2049."
  type        = string
}
