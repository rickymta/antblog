output "boundary_policy_arn" {
  description = "ARN của permission boundary; dev phải gắn đúng ARN này khi tạo role."
  value       = aws_iam_policy.lambda_dev_boundary.arn
}

output "team_role_arns" {
  description = "Role của từng team, đã gắn tag Team."
  value       = { for k, r in aws_iam_role.team : k => r.arn }
}

output "partner_role_arn" {
  description = "Role đối tác đối soát assume (kèm external ID)."
  value       = aws_iam_role.partner.arn
}

output "external_analyzer_arn" {
  description = "ARN analyzer external access, dùng cho list-findings-v2."
  value       = aws_accessanalyzer_analyzer.external.arn
}

output "unused_analyzer_arn" {
  description = "ARN analyzer unused access (null nếu tắt)."
  value       = one(aws_accessanalyzer_analyzer.unused[*].arn)
}
