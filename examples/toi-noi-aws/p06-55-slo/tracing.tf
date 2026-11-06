# ---------------------------------------------------------------------------
# X-Ray: luật lấy mẫu và group lọc trace chậm theo thành phố
# ---------------------------------------------------------------------------

# Thanh toán: lấy mẫu 100%, đây là luồng không được mất dấu.
resource "aws_xray_sampling_rule" "payments" {
  rule_name      = "toinoi-${var.env}-payments"
  priority       = 100
  version        = 1
  reservoir_size = 1
  fixed_rate     = 1.0
  url_path       = "/v1/payments/*"
  host           = "*"
  http_method    = "*"
  service_name   = "toinoi-*"
  service_type   = "*"
  resource_arn   = "*"

  tags = {
    Service = "payment"
  }
}

# Đặt chuyến: mỗi giây lấy chắc 1 trace, phần còn lại 5%, để tiết kiệm ở 70 chuyến/giây.
resource "aws_xray_sampling_rule" "booking" {
  rule_name      = "toinoi-${var.env}-booking"
  priority       = 200
  version        = 1
  reservoir_size = 1
  fixed_rate     = 0.05
  url_path       = "/v1/trips*"
  host           = "*"
  http_method    = "POST"
  service_name   = "toinoi-*"
  service_type   = "*"
  resource_arn   = "*"

  tags = {
    Service = "booking"
  }
}

# Group: trace chậm hơn ngưỡng ở thành phố Hồ Chí Minh. `city` là annotation (được đánh chỉ mục).
resource "aws_xray_group" "slow_hcm" {
  group_name        = "toinoi-${var.env}-slow-hcm"
  filter_expression = "annotation.city = \"HCM\" AND responsetime > ${var.latency_threshold_seconds}"

  insights_configuration {
    insights_enabled      = true
    notifications_enabled = false
  }

  tags = {
    Service = "booking"
  }
}
