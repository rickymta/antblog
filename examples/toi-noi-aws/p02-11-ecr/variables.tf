variable "region" {
  description = "Region của registry ECR."
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

variable "service" {
  description = "Tên service, dùng làm tên repository."
  type        = string
  default     = "booking"
}

variable "org_id" {
  description = "ID AWS Organization (dạng o-xxxxxxxxxx) được phép pull image từ tài khoản khác. Giá trị mẫu là giả."
  type        = string
  default     = "o-exampleorgid"
}

variable "enable_pull_through_cache" {
  description = "Tạo pull-through cache rule cho ECR Public (public.ecr.aws)."
  type        = bool
  default     = true
}
