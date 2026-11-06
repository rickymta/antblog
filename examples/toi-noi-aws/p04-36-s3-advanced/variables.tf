variable "region" {
  description = "Region chính của lab."
  type        = string
  default     = "ap-southeast-1"
}

variable "replica_region" {
  description = "Region đặt bản sao hoá đơn (CRR). Phải khác region chính."
  type        = string
  default     = "ap-northeast-1"
}

variable "env" {
  description = "Môi trường: lab, dev, staging hoặc prod."
  type        = string
  default     = "lab"
}

variable "owner" {
  description = "Team sở hữu tài nguyên (tag Owner)."
  type        = string
  default     = "data-platform"
}

variable "object_lock_mode" {
  description = "Chế độ Object Lock mặc định cho bucket hoá đơn. Lab dùng GOVERNANCE để dọn được; production mới cân nhắc COMPLIANCE."
  type        = string
  default     = "GOVERNANCE"

  validation {
    condition     = contains(["GOVERNANCE", "COMPLIANCE"], var.object_lock_mode)
    error_message = "object_lock_mode chỉ nhận GOVERNANCE hoặc COMPLIANCE."
  }
}

variable "object_lock_days" {
  description = "Số ngày giữ mặc định. Lab đặt 1 ngày; hoá đơn thật phải giữ 10 năm (đổi days thành years = 10 trong invoices.tf)."
  type        = number
  default     = 1
}

variable "enable_rtc" {
  description = "Bật Replication Time Control (có phí riêng, mục tiêu 15 phút) cho rule replication hoá đơn."
  type        = bool
  default     = false
}

variable "create_table_bucket" {
  description = "Tạo S3 table bucket và namespace cho Iceberg (bài 38 sẽ query bằng Athena)."
  type        = bool
  default     = true
}
