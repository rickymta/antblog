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

variable "vpc_id" {
  description = "VPC của lab p03-23, nơi gắn service network."
  type        = string
  default     = "vpc-0123456789abcdef0"
}

variable "security_group_ids" {
  description = "Security group áp lên liên kết VPC với service network (cho phép lưu lượng từ Lattice tới workload)."
  type        = list(string)
  default     = ["sg-0123456789abcdef0"]
}

variable "fraud_lambda_arn" {
  description = "ARN hàm Lambda fraud (bài 18) sẽ được publish qua Lattice."
  type        = string
  default     = "arn:aws:lambda:ap-southeast-1:123456789012:function:toinoi-lab-fraud"
}

variable "caller_role_arns" {
  description = "Role được phép gọi fraud qua Lattice; mặc định là task role ECS của pricing (kịch bản pricing chạy trên ECS ở bài 49)."
  type        = list(string)
  default     = ["arn:aws:iam::123456789012:role/toinoi-lab-pricing-task"]
}

variable "amp_workspace_arn" {
  description = "ARN workspace Managed Prometheus (bài 47) mà Argo Rollouts đọc để phân tích."
  type        = string
  default     = "arn:aws:aps:ap-southeast-1:123456789012:workspace/ws-aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
}
