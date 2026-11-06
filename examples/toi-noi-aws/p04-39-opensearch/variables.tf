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
  default     = "search"
}

variable "mode" {
  description = "serverless: OpenSearch Serverless collection (mặc định). domain: managed domain trong VPC."
  type        = string
  default     = "serverless"

  validation {
    condition     = contains(["serverless", "domain"], var.mode)
    error_message = "mode chỉ nhận serverless hoặc domain."
  }
}

variable "principal_arn" {
  description = "ARN role hoặc user IAM được phép đọc ghi dữ liệu (ví dụ role SSO của bạn hoặc role service search)."
  type        = string
}

# --- Chỉ dùng khi mode = domain ---

variable "vpc_subnet_ids" {
  description = "Subnet của domain (private, một subnet mỗi AZ khi bật zone awareness). Chỉ dùng khi mode = domain."
  type        = list(string)
  default     = []
}

variable "vpc_security_group_ids" {
  description = "Security group của domain: chỉ cho cổng 443 từ nơi gọi. Chỉ dùng khi mode = domain."
  type        = list(string)
  default     = []
}

variable "domain_instance_type" {
  description = "Loại instance data node của domain lab."
  type        = string
  default     = "t3.small.search"
}

variable "domain_instance_count" {
  description = "Số data node. Lab dùng 1; production dùng 3 node trải 3 AZ kèm dedicated master."
  type        = number
  default     = 1
}

variable "domain_volume_size_gb" {
  description = "Dung lượng EBS mỗi data node (GB)."
  type        = number
  default     = 20
}
