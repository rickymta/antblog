variable "region" {
  description = "Region của cluster."
  type        = string
  default     = "ap-southeast-1"
}

variable "env" {
  description = "Môi trường: lab, dev, staging, prod."
  type        = string
  default     = "lab"
}

variable "cluster_name" {
  description = "Tên cluster EKS đã có (bài 42)."
  type        = string
  default     = "toinoi-lab-eks"
}

variable "photos_bucket" {
  description = "Bucket ảnh chứng từ giao hàng (bài 06)."
  type        = string
  default     = "toinoi-lab-delivery-photos"
}

variable "guardduty_detector_id" {
  description = "ID GuardDuty detector đã có trong tài khoản. Để trống thì không bật tính năng EKS ở đây."
  type        = string
  default     = ""
}
