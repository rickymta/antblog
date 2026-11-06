variable "region" {
  description = "Region của workload."
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

variable "hold_funds_function_arn" {
  description = "ARN Lambda giữ tiền (idempotent theo idempotencyKey). Lab không tạo hàm này; dùng hàm của bạn hoặc stub."
  type        = string
  default     = "arn:aws:lambda:ap-southeast-1:123456789012:function:toinoi-lab-payment-hold"
}

variable "capture_payment_function_arn" {
  description = "ARN Lambda thu tiền."
  type        = string
  default     = "arn:aws:lambda:ap-southeast-1:123456789012:function:toinoi-lab-payment-capture"
}

variable "release_funds_function_arn" {
  description = "ARN Lambda hoàn giữ tiền (bù trừ)."
  type        = string
  default     = "arn:aws:lambda:ap-southeast-1:123456789012:function:toinoi-lab-payment-release"
}

variable "reconcile_batch_function_arn" {
  description = "ARN Lambda đối soát một lô chuyến."
  type        = string
  default     = "arn:aws:lambda:ap-southeast-1:123456789012:function:toinoi-lab-reconcile-batch"
}

variable "dispatch_queue_url" {
  description = "URL SQS queue của dispatch (nhận lệnh FIND_DRIVER và RELEASE_DRIVER)."
  type        = string
  default     = "https://sqs.ap-southeast-1.amazonaws.com/123456789012/toinoi-lab-dispatch"
}

variable "dispatch_queue_arn" {
  description = "ARN của queue dispatch."
  type        = string
  default     = "arn:aws:sqs:ap-southeast-1:123456789012:toinoi-lab-dispatch"
}

variable "manual_review_queue_url" {
  description = "URL SQS queue cho các saga cần xử lý thủ công khi bù trừ thất bại."
  type        = string
  default     = "https://sqs.ap-southeast-1.amazonaws.com/123456789012/toinoi-lab-saga-manual-review"
}

variable "manual_review_queue_arn" {
  description = "ARN của queue xử lý thủ công."
  type        = string
  default     = "arn:aws:sqs:ap-southeast-1:123456789012:toinoi-lab-saga-manual-review"
}

variable "dispatch_table_name" {
  description = "Tên bảng DynamoDB của dispatch (bài 32)."
  type        = string
  default     = "toinoi-lab-dispatch"
}

variable "dispatch_table_arn" {
  description = "ARN bảng DynamoDB của dispatch."
  type        = string
  default     = "arn:aws:dynamodb:ap-southeast-1:123456789012:table/toinoi-lab-dispatch"
}

variable "event_bus_name" {
  description = "Tên custom event bus nhận sự kiện nghiệp vụ (bài 17)."
  type        = string
  default     = "toinoi-lab-trips"
}

variable "event_bus_arn" {
  description = "ARN của custom event bus."
  type        = string
  default     = "arn:aws:events:ap-southeast-1:123456789012:event-bus/toinoi-lab-trips"
}

variable "recon_bucket_name" {
  description = "Bucket chứa manifest chuyến mỗi ngày và kết quả đối soát."
  type        = string
  default     = "toinoi-lab-recon-123456789012"
}

variable "log_retention_days" {
  description = "Số ngày giữ log của state machine."
  type        = number
  default     = 30
}
