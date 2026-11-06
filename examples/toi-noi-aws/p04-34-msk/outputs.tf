output "cluster_arn" {
  description = "ARN cluster MSK; dùng cho lệnh aws kafka get-bootstrap-brokers và IAM policy."
  value       = local.cluster_arn
}

output "msk_security_group_id" {
  value = aws_security_group.msk.id
}

output "dispatch_consumer_policy_arn" {
  description = "Policy đọc booking.events.v1 cho consumer của dispatch."
  value       = aws_iam_policy.dispatch_consumer.arn
}

output "dispatch_producer_policy_arn" {
  description = "Policy ghi dispatch.events.v1 cho service dispatch."
  value       = aws_iam_policy.dispatch_producer.arn
}

output "bootstrap_brokers_sasl_iam" {
  description = "Bootstrap brokers (IAM, cổng 9098) của MSK Provisioned; với Serverless dùng aws kafka get-bootstrap-brokers."
  value       = one(aws_msk_cluster.main[*].bootstrap_brokers_sasl_iam)
}
