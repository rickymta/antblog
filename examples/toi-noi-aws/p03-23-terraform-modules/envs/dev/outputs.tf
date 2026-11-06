output "vpc_id" {
  description = "ID VPC của môi trường."
  value       = module.network.vpc_id
}

output "cluster_arn" {
  description = "ARN ECS cluster."
  value       = aws_ecs_cluster.this.arn
}

output "service_names" {
  description = "Tên các ECS service theo khoá trong var.services."
  value       = { for k, s in module.service : k => s.service_name }
}

output "db_secret_arn" {
  description = "ARN secret chứa mật khẩu master của RDS (không phải giá trị mật khẩu)."
  value       = module.rds.master_user_secret_arn
}
