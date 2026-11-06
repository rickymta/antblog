output "karpenter_node_role_name" {
  description = "Tên role của node do Karpenter tạo; điền vào spec.role của EC2NodeClass."
  value       = module.karpenter.node_iam_role_name
}

output "karpenter_queue_name" {
  description = "Tên hàng đợi SQS nhận sự kiện gián đoạn Spot; điền vào settings.interruptionQueue khi cài Karpenter."
  value       = module.karpenter.queue_name
}

output "keda_role_arn" {
  description = "Role IAM của KEDA operator (Pod Identity)."
  value       = aws_iam_role.keda.arn
}
