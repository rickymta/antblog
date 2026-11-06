variable "region" {
  description = "Region triển khai."
  type        = string
  default     = "ap-southeast-1"
}

variable "aws_profile" {
  description = "Tên profile AWS CLI (IAM Identity Center). null thì dùng chuỗi credential mặc định."
  type        = string
  default     = null
}

variable "env" {
  description = "Tên môi trường; tên cluster là toinoi-<env>-eks."
  type        = string
  default     = "lab"
}

variable "owner" {
  description = "Giá trị tag Owner."
  type        = string
  default     = "platform"
}

variable "kubernetes_version" {
  description = "Phiên bản Kubernetes của cluster, dạng 1.NN. Ví dụ minh hoạ: hãy xem trang Kubernetes versions của EKS để chọn bản còn standard support."
  type        = string
  default     = "1.36"

  validation {
    condition     = can(regex("^1\\.[0-9]{2}$", var.kubernetes_version))
    error_message = "kubernetes_version phải có dạng 1.NN, ví dụ 1.36."
  }
}

variable "vpc_id" {
  description = "ID VPC dựng ở lab p03-23 (output vpc_id của envs/dev)."
  type        = string
}

variable "private_subnet_ids" {
  description = "ID các private subnet (cho node và ENI của control plane), tối thiểu hai AZ."
  type        = list(string)

  validation {
    condition     = length(var.private_subnet_ids) >= 2
    error_message = "Cần ít nhất hai private subnet ở hai AZ khác nhau."
  }
}

variable "public_subnet_ids" {
  description = "ID các public subnet, chỉ dùng để gắn tag cho load balancer ở bài 43."
  type        = list(string)
}

variable "api_allowed_cidrs" {
  description = "CIDR được gọi public endpoint của API server. Thay bằng dải IP văn phòng hoặc VPN của bạn; giá trị mặc định là dải ví dụ theo RFC 5737."
  type        = list(string)
  default     = ["203.0.113.0/24"]
}

variable "ci_role_arn" {
  description = "ARN role IAM của pipeline CI (bài 25) được deploy vào namespace booking và dispatch."
  type        = string
  default     = "arn:aws:iam::123456789012:role/toinoi-lab-ci-deploy"
}

variable "dev_role_arn" {
  description = "ARN role IAM của lập trình viên, chỉ được xem toàn cluster."
  type        = string
  default     = "arn:aws:iam::123456789012:role/toinoi-lab-dev"
}

variable "proof_bucket_name" {
  description = "Tên bucket ảnh chứng từ giao hàng (bài 06) mà pod booking được ghi vào proof/*."
  type        = string
  default     = "toinoi-lab-delivery-photos"
}

variable "base_node_desired_size" {
  description = "Số node on-demand của node group base (chạy add-on và service cần ổn định)."
  type        = number
  default     = 2
}

variable "spot_node_desired_size" {
  description = "Số node Spot ban đầu của node group spot."
  type        = number
  default     = 1
}
