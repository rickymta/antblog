env      = "dev"
owner    = "platform"
vpc_cidr = "10.10.0.0/16"

# NAT Gateway tính tiền theo giờ; tắt đi thì task Fargate trong private subnet không kéo được image từ ECR.
enable_nat_gateway = true

db_multi_az            = false
db_deletion_protection = false
db_skip_final_snapshot = true

# Mỗi dòng là một service. Thêm tracking chỉ cần thêm một khối.
services = {
  quote = {
    image_tag     = "1.0.0"
    desired_count = 2
  }
  booking = {
    image_tag = "1.0.0"
    needs_db  = true
  }
  dispatch = {
    image_tag = "1.0.0"
    cpu       = 512
    memory    = 1024
    needs_db  = true
  }
}
