output "writer_endpoint" {
  description = "Cluster endpoint (luôn trỏ tới writer hiện tại). Dùng cho migration và công cụ quản trị, không dùng cho app khi đã có proxy."
  value       = aws_rds_cluster.main.endpoint
}

output "reader_endpoint" {
  description = "Reader endpoint của cluster (DNS luân phiên giữa các reader)."
  value       = aws_rds_cluster.main.reader_endpoint
}

output "reporting_endpoint" {
  description = "Custom endpoint cho báo cáo (chỉ instance reporting)."
  value       = aws_rds_cluster_endpoint.reporting.endpoint
}

output "proxy_endpoint" {
  description = "Endpoint đọc/ghi của RDS Proxy: command của service booking nối vào đây."
  value       = aws_db_proxy.main.endpoint
}

output "proxy_read_only_endpoint" {
  description = "Endpoint chỉ đọc của RDS Proxy: query của service booking nối vào đây."
  value       = aws_db_proxy_endpoint.read_only.endpoint
}

output "master_secret_arn" {
  description = "Secret master do RDS quản lý."
  value       = aws_rds_cluster.main.master_user_secret[0].secret_arn
}

output "app_connect_policy_arn" {
  description = "Policy rds-db:connect gắn vào role của service."
  value       = aws_iam_policy.app_connect.arn
}

output "cluster_identifier" {
  value = aws_rds_cluster.main.cluster_identifier
}
