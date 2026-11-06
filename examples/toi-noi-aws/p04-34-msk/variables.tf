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

variable "vpc_id" {
  description = "VPC của lab (bài 04)."
  type        = string
}

variable "private_subnet_ids" {
  description = "Private subnet cho broker, ba subnet ở ba AZ khác nhau (MSK Provisioned đặt broker theo số AZ này)."
  type        = list(string)

  validation {
    condition     = length(var.private_subnet_ids) == 3
    error_message = "Cần đúng ba subnet ở ba AZ khác nhau."
  }
}

variable "client_security_group_id" {
  description = "Security group của nơi gọi Kafka (task ECS bài 12 hoặc EC2 thử nghiệm bài 05). Chỉ nhóm này được vào cổng 9098 (IAM)."
  type        = string
}

variable "mode" {
  description = "provisioned: cluster có broker. serverless: MSK Serverless (chỉ IAM, không chọn broker)."
  type        = string
  default     = "provisioned"

  validation {
    condition     = contains(["provisioned", "serverless"], var.mode)
    error_message = "mode phải là provisioned hoặc serverless."
  }
}

variable "kafka_version" {
  description = "Phiên bản Kafka của MSK Provisioned (3.9.x là bản AWS khuyến nghị, chạy ZooKeeper; 3.9.x.kraft để dùng KRaft; 4.x chỉ KRaft). Kiểm danh sách hiện có: aws kafka list-kafka-versions."
  type        = string
  default     = "3.9.x"
}

variable "broker_instance_type" {
  description = "Loại broker. kafka.t3.small chỉ cho lab. Production thường kafka.m7g.*; broker Express có dạng express.m7g.* (không cấu hình đĩa; Express chỉ cho sửa một số thuộc tính cấu hình nên lab bỏ aws_msk_configuration)."
  type        = string
  default     = "kafka.t3.small"
}

variable "broker_volume_gb" {
  description = "Dung lượng EBS mỗi broker (GiB). Bỏ qua với broker Express."
  type        = number
  default     = 10
}

variable "kafkaconnect_version" {
  description = "Phiên bản Kafka Connect của MSK Connect. 3.7.x có từ 12/2024 và hợp với Debezium 2.x trở lên; 2.7.1 là bản cũ."
  type        = string
  default     = "3.7.x"
}

variable "create_connector" {
  description = "true để tạo MSK Connect với Debezium đọc bảng outbox. Cần plugin đã tải lên S3 và database đã chuẩn bị (xem sql/outbox.sql)."
  type        = bool
  default     = false
}

variable "plugin_bucket_arn" {
  description = "ARN bucket S3 chứa tệp ZIP plugin Debezium (và config provider). Chỉ dùng khi create_connector = true."
  type        = string
  default     = null
}

variable "plugin_file_key" {
  description = "Key của tệp ZIP plugin trong bucket."
  type        = string
  default     = "plugins/debezium-postgres-outbox.zip"
}

variable "db_host" {
  description = "Endpoint cluster endpoint (writer) của Aurora ở bài 31. Logical replication không đi qua RDS Proxy."
  type        = string
  default     = null
}

variable "db_security_group_id" {
  description = "Security group của database; lab thêm rule cho phép connector vào cổng 5432."
  type        = string
  default     = null
}
