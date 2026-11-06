output "cluster_name" {
  description = "Tên cluster EKS."
  value       = module.eks.cluster_name
}

output "cluster_endpoint" {
  description = "Endpoint API server."
  value       = module.eks.cluster_endpoint
}

output "update_kubeconfig_command" {
  description = "Lệnh ghi context kubectl cho cluster này."
  value       = "aws eks update-kubeconfig --name ${module.eks.cluster_name} --region ${var.region} --alias ${module.eks.cluster_name}"
}

output "cluster_security_group_id" {
  description = "Cluster security group do EKS tạo (gắn vào ENI của control plane)."
  value       = module.eks.cluster_security_group_id
}

output "node_security_group_id" {
  description = "Security group của node do module tạo."
  value       = module.eks.node_security_group_id
}

output "booking_role_arn" {
  description = "Role IAM mà pod booking nhận qua Pod Identity."
  value       = aws_iam_role.booking.arn
}
