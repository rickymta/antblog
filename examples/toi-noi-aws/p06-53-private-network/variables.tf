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
  default     = "platform"
}

variable "spoke_cidrs" {
  description = "CIDR của hai VPC spoke. Không được chồng lấn nhau và nên nằm trong 10.0.0.0/8 để route tổng hợp ở firewall hoạt động."
  type        = map(string)
  default = {
    dev  = "10.10.0.0/16"
    prod = "10.20.0.0/16"
  }
}

variable "inspection_cidr" {
  description = "CIDR của VPC inspection (chứa Network Firewall)."
  type        = string
  default     = "10.99.0.0/16"
}

variable "enable_network_firewall" {
  description = "Tạo Network Firewall trong VPC inspection. Mỗi endpoint firewall tính phí theo giờ và theo GB xử lý."
  type        = bool
  default     = true
}

variable "enable_egress_nat" {
  description = "Tạo Internet Gateway, subnet public và NAT Gateway ở VPC inspection để egress ra Internet qua firewall. Mặc định tắt vì NAT tính phí theo giờ và theo GB."
  type        = bool
  default     = false
}

variable "enable_privatelink_service" {
  description = "Tạo NLB nội bộ và VPC endpoint service để xuất API đối soát cho đối tác. NLB tính phí theo giờ."
  type        = bool
  default     = true
}

variable "partner_principal_arns" {
  description = "ARN principal của đối tác được phép tạo endpoint tới service đối soát, ví dụ arn:aws:iam::<account-đối-tác>:root. Để trống thì chưa ai được phép."
  type        = list(string)
  default     = []
}

variable "allowed_egress_domains" {
  description = "Tên miền được phép đi ra Internet qua firewall (HTTP host hoặc TLS SNI). Dấu chấm đầu nghĩa là cả tên miền con."
  type        = list(string)
  default     = [".amazonaws.com", "api.vixanh.example"]
}
