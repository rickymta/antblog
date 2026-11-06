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

variable "stream_mode" {
  description = "ON_DEMAND: AWS tự co giãn shard. PROVISIONED: bạn đặt shard_count."
  type        = string
  default     = "ON_DEMAND"

  validation {
    condition     = contains(["ON_DEMAND", "PROVISIONED"], var.stream_mode)
    error_message = "stream_mode phải là ON_DEMAND hoặc PROVISIONED."
  }
}

variable "shard_count" {
  description = "Số shard khi PROVISIONED. 6.250 bản ghi/giây cần ít nhất 7 shard (mỗi shard ghi tối đa 1.000 bản ghi/giây); xem scripts/shard-math.sh."
  type        = number
  default     = 8
}

variable "retention_hours" {
  description = "Thời gian giữ bản ghi trong stream (giờ): 24 tới 8760. Trên 24 giờ tính thêm tiền."
  type        = number
  default     = 24

  validation {
    condition     = var.retention_hours >= 24 && var.retention_hours <= 8760
    error_message = "retention_hours phải trong khoảng 24 tới 8760."
  }
}

variable "firehose_buffer_mb" {
  description = "Buffer size của Firehose (MiB). Với chuyển đổi Parquet và dynamic partitioning, tối thiểu 64."
  type        = number
  default     = 128
}

variable "firehose_buffer_seconds" {
  description = "Buffer interval của Firehose (giây, 60 tới 900). Một đối tượng S3 được ghi khi đầy size hoặc hết thời gian, tuỳ cái nào tới trước."
  type        = number
  default     = 300
}

variable "consumer_function_arn" {
  description = "ARN hoặc tên Lambda dispatch đọc stream qua enhanced fan-out và ghi DynamoDB (bài 32). null thì không tạo event source mapping."
  type        = string
  default     = null
}

variable "consumer_failure_destination_arn" {
  description = "ARN SQS hoặc SNS nhận thông tin lô bản ghi mà Lambda xử lý thất bại hết số lần thử. Nên đặt khi có consumer_function_arn."
  type        = string
  default     = null
}
