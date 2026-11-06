output "vpc_id" {
  description = "ID của VPC."
  value       = aws_vpc.main.id
}

output "subnet_ids" {
  description = "ID subnet theo tên (public-a, app-a, data-a, ...)."
  value       = { for k, s in aws_subnet.this : k => s.id }
}

output "security_group_ids" {
  description = "ID các security group: alb, app, db."
  value = {
    alb = aws_security_group.alb.id
    app = aws_security_group.app.id
    db  = aws_security_group.db.id
  }
}
