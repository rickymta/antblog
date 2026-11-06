# Amazon Application Recovery Controller (ARC, trước đây là Route 53 ARC): cluster, control panel, routing control.
# Cluster tính tiền theo giờ; dọn bằng terraform destroy khi xong lab.

resource "aws_route53recoverycontrolconfig_cluster" "main" {
  provider = aws.arc
  name     = "toinoi-${var.env}-arc"
}

resource "aws_route53recoverycontrolconfig_control_panel" "main" {
  provider    = aws.arc
  name        = "toinoi-${var.env}-booking"
  cluster_arn = aws_route53recoverycontrolconfig_cluster.main.arn
}

resource "aws_route53recoverycontrolconfig_routing_control" "primary" {
  provider          = aws.arc
  name              = "toinoi-${var.env}-primary-ap-southeast-1"
  cluster_arn       = aws_route53recoverycontrolconfig_cluster.main.arn
  control_panel_arn = aws_route53recoverycontrolconfig_control_panel.main.arn
}

resource "aws_route53recoverycontrolconfig_routing_control" "secondary" {
  provider          = aws.arc
  name              = "toinoi-${var.env}-secondary-ap-northeast-1"
  cluster_arn       = aws_route53recoverycontrolconfig_cluster.main.arn
  control_panel_arn = aws_route53recoverycontrolconfig_control_panel.main.arn
}

# Safety rule: luôn có ít nhất một routing control ở trạng thái On,
# để không vô tình tắt cả hai region.
resource "aws_route53recoverycontrolconfig_safety_rule" "at_least_one_on" {
  provider          = aws.arc
  name              = "toinoi-${var.env}-at-least-one-region-on"
  control_panel_arn = aws_route53recoverycontrolconfig_control_panel.main.arn
  wait_period_ms    = 5000

  asserted_controls = [
    aws_route53recoverycontrolconfig_routing_control.primary.arn,
    aws_route53recoverycontrolconfig_routing_control.secondary.arn,
  ]

  rule_config {
    inverted  = false
    threshold = 1
    type      = "ATLEAST"
  }
}
