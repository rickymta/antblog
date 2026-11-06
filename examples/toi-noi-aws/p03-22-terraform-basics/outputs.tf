output "vpc_id" {
  description = "ID của VPC."
  value       = aws_vpc.main.id
}

output "public_subnet_ids" {
  description = "ID các public subnet theo AZ."
  value       = { for az, s in aws_subnet.public : az => s.id }
}

output "private_subnet_ids" {
  description = "ID các private subnet theo AZ."
  value       = { for az, s in aws_subnet.private : az => s.id }
}

output "delivery_photos_bucket" {
  description = "Tên bucket ảnh chứng từ giao hàng."
  value       = aws_s3_bucket.delivery_photos.bucket
}
