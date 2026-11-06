# Khoá ALB để chỉ nhận lưu lượng đi qua CloudFront. Hai lớp:
# 1) security group chỉ cho prefix list origin-facing của CloudFront vào cổng 443;
# 2) listener rule chỉ chuyển tiếp khi có header bí mật X-Origin-Verify đúng giá trị.
# Chỉ tạo khi bạn truyền ALB (alb_security_group_id, alb_listener_arn, api_target_group_arn).
# Quản lý rule mặc định của listener (trả 403 khi thiếu header) ở nơi bạn dựng ALB.

locals {
  lock_alb = var.alb_security_group_id != "" && var.alb_listener_arn != "" && var.api_target_group_arn != ""
}

data "aws_ec2_managed_prefix_list" "cloudfront" {
  count = local.lock_alb ? 1 : 0
  name  = "com.amazonaws.global.cloudfront.origin-facing"
}

resource "aws_vpc_security_group_ingress_rule" "alb_from_cloudfront" {
  count             = local.lock_alb ? 1 : 0
  security_group_id = var.alb_security_group_id
  description       = "HTTPS tu CloudFront"
  prefix_list_id    = data.aws_ec2_managed_prefix_list.cloudfront[0].id
  ip_protocol       = "tcp"
  from_port         = 443
  to_port           = 443
}

resource "aws_lb_listener_rule" "api_with_origin_header" {
  count        = local.lock_alb ? 1 : 0
  listener_arn = var.alb_listener_arn
  priority     = 10

  action {
    type             = "forward"
    target_group_arn = var.api_target_group_arn
  }

  condition {
    http_header {
      http_header_name = "X-Origin-Verify"
      values           = [var.origin_verify_value]
    }
  }
}
