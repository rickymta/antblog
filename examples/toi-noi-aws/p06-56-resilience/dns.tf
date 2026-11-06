# Health check Route 53 đọc trạng thái routing control của ARC.
resource "aws_route53_health_check" "primary" {
  type                = "RECOVERY_CONTROL"
  routing_control_arn = aws_route53recoverycontrolconfig_routing_control.primary.arn

  tags = {
    Name    = "toinoi-${var.env}-hc-primary"
    Service = "booking"
  }
}

resource "aws_route53_health_check" "secondary" {
  type                = "RECOVERY_CONTROL"
  routing_control_arn = aws_route53recoverycontrolconfig_routing_control.secondary.arn

  tags = {
    Name    = "toinoi-${var.env}-hc-secondary"
    Service = "booking"
  }
}

# Cặp bản ghi failover: PRIMARY ở ap-southeast-1, SECONDARY ở ap-northeast-1.
resource "aws_route53_record" "primary" {
  zone_id         = var.hosted_zone_id
  name            = var.record_name
  type            = "A"
  set_identifier  = "primary-ap-southeast-1"
  health_check_id = aws_route53_health_check.primary.id

  failover_routing_policy {
    type = "PRIMARY"
  }

  alias {
    name                   = var.primary_alb_dns_name
    zone_id                = var.primary_alb_zone_id
    evaluate_target_health = true
  }
}

resource "aws_route53_record" "secondary" {
  zone_id         = var.hosted_zone_id
  name            = var.record_name
  type            = "A"
  set_identifier  = "secondary-ap-northeast-1"
  health_check_id = aws_route53_health_check.secondary.id

  failover_routing_policy {
    type = "SECONDARY"
  }

  alias {
    name                   = var.secondary_alb_dns_name
    zone_id                = var.secondary_alb_zone_id
    evaluate_target_health = true
  }
}
