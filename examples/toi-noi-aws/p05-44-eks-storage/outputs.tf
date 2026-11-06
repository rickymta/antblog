output "efs_file_system_id" {
  description = "ID EFS; điền vào fileSystemId của StorageClass toinoi-efs."
  value       = aws_efs_file_system.photo_cache.id
}

output "ebs_csi_role_arn" {
  description = "Role IAM của EBS CSI driver (Pod Identity)."
  value       = module.ebs_csi_pod_identity.iam_role_arn
}

output "efs_csi_role_arn" {
  description = "Role IAM của EFS CSI driver (Pod Identity)."
  value       = module.efs_csi_pod_identity.iam_role_arn
}
