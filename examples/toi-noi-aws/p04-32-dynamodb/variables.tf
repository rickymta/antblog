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

variable "billing_mode" {
  description = "PAY_PER_REQUEST (on-demand) hoặc PROVISIONED."
  type        = string
  default     = "PAY_PER_REQUEST"

  validation {
    condition     = contains(["PAY_PER_REQUEST", "PROVISIONED"], var.billing_mode)
    error_message = "billing_mode phải là PAY_PER_REQUEST hoặc PROVISIONED."
  }
}

variable "max_write_request_units" {
  description = "Chỉ với on-demand: trần số write request units mỗi giây (chặn hoá đơn phình khi có lỗi). -1 là không đặt trần."
  type        = number
  default     = 2000
}

variable "max_read_request_units" {
  description = "Chỉ với on-demand: trần số read request units mỗi giây. -1 là không đặt trần."
  type        = number
  default     = 2000
}

variable "provisioned_write_units" {
  description = "Chỉ với PROVISIONED: WCU của bảng và của GSI."
  type        = number
  default     = 100
}

variable "provisioned_read_units" {
  description = "Chỉ với PROVISIONED: RCU của bảng và của GSI."
  type        = number
  default     = 100
}

variable "warm_write_units" {
  description = "Warm throughput ghi mong muốn (đơn vị/giây) để làm nóng bảng trước giờ cao điểm. null là để mặc định."
  type        = number
  default     = null
}

variable "warm_read_units" {
  description = "Warm throughput đọc mong muốn (đơn vị/giây). null là để mặc định."
  type        = number
  default     = null
}

variable "stats_function_arn" {
  description = "ARN hoặc tên Lambda nhận DynamoDB Streams để cập nhật thống kê (viết ở bài 18). null thì không tạo event source mapping."
  type        = string
  default     = null
}

variable "stats_failure_destination_arn" {
  description = "ARN SQS hoặc SNS nhận bản ghi Streams mà Lambda xử lý thất bại hết số lần thử. Bắt buộc khi đặt stats_function_arn."
  type        = string
  default     = null
}
