variable "name" {
  description = "Định danh instance, ví dụ toinoi-dev-trips."
  type        = string
}

variable "vpc_id" {
  description = "VPC chứa database."
  type        = string
}

variable "subnet_ids" {
  description = "Private subnet cho DB subnet group (tối thiểu hai AZ)."
  type        = list(string)

  validation {
    condition     = length(var.subnet_ids) >= 2
    error_message = "Cần ít nhất hai subnet ở hai AZ khác nhau."
  }
}

variable "engine_version" {
  description = "Phiên bản PostgreSQL. Kiểm danh sách hợp lệ bằng aws rds describe-db-engine-versions --engine postgres."
  type        = string
  default     = "17"
}

variable "instance_class" {
  description = "Instance class của RDS."
  type        = string
  default     = "db.t4g.micro"
}

variable "allocated_storage" {
  description = "Dung lượng ban đầu (GiB)."
  type        = number
  default     = 20
}

variable "db_name" {
  description = "Tên database mặc định. Đổi giá trị này sau khi tạo sẽ buộc thay thế instance."
  type        = string
  default     = "toinoi"
}

variable "master_username" {
  description = "Tên master user. Mật khẩu do RDS tự sinh và lưu trong Secrets Manager. Đổi giá trị này sẽ buộc thay thế instance."
  type        = string
  default     = "toinoi_admin"
}

variable "multi_az" {
  description = "Bật Multi-AZ. Gấp đôi chi phí instance; bật cho staging và prod."
  type        = bool
  default     = false
}

variable "backup_retention_period" {
  description = "Số ngày giữ automated backup (0 là tắt backup)."
  type        = number
  default     = 7
}

variable "deletion_protection" {
  description = "Chặn xoá instance qua API. Mặc định true; lab dev có thể đặt false để dọn."
  type        = bool
  default     = true
}

variable "skip_final_snapshot" {
  description = "true thì không chụp snapshot cuối khi xoá. Production phải là false."
  type        = bool
  default     = false
}
