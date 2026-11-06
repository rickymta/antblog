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
  description = "Team sở hữu tài nguyên (tag Owner)."
  type        = string
  default     = "security"
}

variable "alert_email" {
  description = "Email nhận cảnh báo; để trống thì chỉ tạo SNS topic, không có subscription. Email phải xác nhận subscription."
  type        = string
  default     = ""
}

variable "data_event_bucket_arns" {
  description = "ARN các bucket cần ghi data event S3 (ai đọc, ghi object). Bucket chưa tồn tại vẫn hợp lệ cho trail."
  type        = list(string)
  default     = ["arn:aws:s3:::toinoi-lab-delivery-photos"]
}

variable "is_organization_trail" {
  description = "Tạo trail cho cả tổ chức. Chỉ đặt true khi chạy trong management account hoặc delegated admin; lab một tài khoản để false."
  type        = bool
  default     = false
}

variable "enable_cloudtrail_lake" {
  description = "Tạo event data store CloudTrail Lake để truy vấn SQL. Lake đóng với khách mới từ 31/05/2026: chỉ đặt true nếu tài khoản đã dùng Lake trước đó. Tính phí theo dung lượng nạp và lưu."
  type        = bool
  default     = false
}

variable "enable_config" {
  description = "Bật AWS Config recorder, delivery channel và rule managed. Tính phí theo configuration item và lần đánh giá rule."
  type        = bool
  default     = true
}

variable "enable_guardduty" {
  description = "Bật GuardDuty. Tính phí theo lượng log phân tích; có thời gian dùng thử miễn phí với tài khoản mới."
  type        = bool
  default     = true
}

variable "guardduty_features" {
  description = "Các protection plan bật thêm ngoài nguồn dữ liệu nền (CloudTrail, VPC Flow Logs, DNS). Runtime Monitoring và EBS malware scan tính phí riêng nên không bật mặc định."
  type        = set(string)
  default     = ["S3_DATA_EVENTS", "RDS_LOGIN_EVENTS", "EKS_AUDIT_LOGS", "LAMBDA_NETWORK_LOGS"]
}

variable "enable_security_hub_v2" {
  description = "Bật Security Hub phiên bản mới (tổng hợp rủi ro, exposure finding, định dạng OCSF)."
  type        = bool
  default     = true
}

variable "enable_security_hub_cspm" {
  description = "Bật Security Hub CSPM với chuẩn AWS Foundational Security Best Practices. CSPM cần Config bật để đánh giá control."
  type        = bool
  default     = true
}

variable "enable_inspector" {
  description = "Bật Inspector quét EC2, ECR và Lambda. Tính phí theo số tài nguyên được quét."
  type        = bool
  default     = true
}

variable "enable_macie" {
  description = "Bật Macie và tạo custom data identifier (CCCD, số điện thoại). Job phân loại tính phí theo GB quét nên chỉ chạy khi macie_bucket_names không rỗng."
  type        = bool
  default     = false
}

variable "macie_bucket_names" {
  description = "Tên bucket (đã tồn tại) cho job Macie chạy một lần. Để trống thì không tạo job."
  type        = list(string)
  default     = []
}
