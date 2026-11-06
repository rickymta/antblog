output "vpc_id" {
  description = "ID của VPC."
  value       = aws_vpc.this.id
}

output "vpc_cidr_block" {
  description = "CIDR của VPC."
  value       = aws_vpc.this.cidr_block
}

output "public_subnet_ids" {
  description = "ID các public subnet, sắp theo tên AZ."
  value       = [for az in local.azs : aws_subnet.public[az].id]
}

output "private_subnet_ids" {
  description = "ID các private subnet, sắp theo tên AZ."
  value       = [for az in local.azs : aws_subnet.private[az].id]
}
