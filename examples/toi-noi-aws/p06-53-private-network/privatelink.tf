# ---------------------------------------------------------------------------
# PrivateLink: xuất API đối soát cho đối tác thanh toán qua NLB, không peering, không lộ CIDR
# ---------------------------------------------------------------------------
resource "aws_lb" "reconcile" {
  count = var.enable_privatelink_service ? 1 : 0

  name               = "toinoi-${var.env}-reconcile"
  internal           = true
  load_balancer_type = "network"
  subnets            = local.prod_subnet_ids

  enable_cross_zone_load_balancing = false

  tags = {
    Service = "reconcile"
  }
}

resource "aws_lb_target_group" "reconcile" {
  count = var.enable_privatelink_service ? 1 : 0

  name        = "toinoi-${var.env}-reconcile"
  port        = 8443
  protocol    = "TCP"
  target_type = "ip"
  vpc_id      = aws_vpc.spoke["prod"].id

  tags = {
    Service = "reconcile"
  }
}

resource "aws_lb_listener" "reconcile" {
  count = var.enable_privatelink_service ? 1 : 0

  load_balancer_arn = aws_lb.reconcile[0].arn
  port              = 443
  protocol          = "TCP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.reconcile[0].arn
  }
}

resource "aws_vpc_endpoint_service" "reconcile" {
  count = var.enable_privatelink_service ? 1 : 0

  acceptance_required        = true
  network_load_balancer_arns = [aws_lb.reconcile[0].arn]
  allowed_principals         = var.partner_principal_arns

  tags = {
    Name    = "toinoi-${var.env}-reconcile-service"
    Service = "reconcile"
  }
}
