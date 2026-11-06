output "lb_controller_role_arn" {
  description = "Role IAM mà AWS Load Balancer Controller nhận qua Pod Identity."
  value       = module.lb_controller_pod_identity.iam_role_arn
}

output "external_dns_role_arn" {
  description = "Role IAM mà ExternalDNS nhận qua Pod Identity."
  value       = module.external_dns_pod_identity.iam_role_arn
}
