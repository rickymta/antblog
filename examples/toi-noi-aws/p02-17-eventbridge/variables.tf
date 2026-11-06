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

variable "archive_retention_days" {
  description = "Số ngày giữ sự kiện trong archive để replay."
  type        = number
  default     = 7
}

variable "min_fare_vnd" {
  description = "Ngưỡng giá tối thiểu để rule TripCompleted gửi sự kiện tới queue đối soát."
  type        = number
  default     = 50000
}
