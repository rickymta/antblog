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
  description = "Private subnet đặt cluster và proxy, ít nhất hai AZ (Aurora dùng cả 3 AZ cho lưu trữ, instance đặt theo subnet group)."
  type        = list(string)

  validation {
    condition     = length(var.private_subnet_ids) >= 2
    error_message = "Cần ít nhất hai subnet ở hai AZ khác nhau."
  }
}

variable "client_security_group_id" {
  description = "Security group của nơi gọi database (task ECS bài 12 hoặc EC2 thử nghiệm bài 05). Chỉ nhóm này được vào RDS Proxy."
  type        = string
}

variable "engine_version" {
  description = "Phiên bản Aurora PostgreSQL. Kiểm danh sách hiện có bằng: aws rds describe-db-engine-versions --engine aurora-postgresql. Scale to zero cần bản đủ mới (xem User Guide của Aurora Serverless v2)."
  type        = string
  default     = "17.11"
}

variable "parameter_group_family" {
  description = "Family của cluster parameter group, phải khớp bản chính của engine_version (ví dụ aurora-postgresql17)."
  type        = string
  default     = "aurora-postgresql17"
}

variable "serverless_min_acu" {
  description = "ACU tối thiểu của Serverless v2. 0 bật auto-pause, nhưng trong lab này writer và reader tier 0–1 vẫn không dừng vì có RDS Proxy và rds.logical_replication = 1; từ 0,5 trở lên thì luôn có capacity."
  type        = number
  default     = 0.5
}

variable "serverless_max_acu" {
  description = "ACU tối đa. Cũng ảnh hưởng tới max_connections mặc định của instance."
  type        = number
  default     = 4
}

variable "seconds_until_auto_pause" {
  description = "Chỉ dùng khi serverless_min_acu = 0: số giây không có kết nối trước khi instance tạm dừng (300–86400)."
  type        = number
  default     = 900
}

variable "storage_mode" {
  description = "aurora (Standard, tính tiền theo I/O) hoặc aurora-iopt1 (I/O-Optimized, không tính tiền I/O)."
  type        = string
  default     = "aurora"

  validation {
    condition     = contains(["aurora", "aurora-iopt1"], var.storage_mode)
    error_message = "storage_mode phải là aurora hoặc aurora-iopt1."
  }
}

variable "proxy_secret_arn" {
  description = "ARN secret chứa username và password của user ứng dụng (toinoi_app) để RDS Proxy dùng. Để null thì lab dùng secret master do RDS quản lý (chỉ tiện cho lab, production dùng user riêng)."
  type        = string
  default     = null
}

variable "iam_auth_for_proxy" {
  description = "REQUIRED: client phải dùng token IAM để vào proxy. DISABLED: client dùng mật khẩu."
  type        = string
  default     = "REQUIRED"

  validation {
    condition     = contains(["REQUIRED", "DISABLED"], var.iam_auth_for_proxy)
    error_message = "iam_auth_for_proxy phải là REQUIRED hoặc DISABLED."
  }
}
