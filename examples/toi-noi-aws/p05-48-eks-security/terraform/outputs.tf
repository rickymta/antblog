output "service_role_arns" {
  description = "Role Pod Identity theo service."
  value       = { for k, r in aws_iam_role.service : k => r.arn }
}

output "eso_role_arn" {
  description = "Role của External Secrets Operator."
  value       = aws_iam_role.eso.arn
}
