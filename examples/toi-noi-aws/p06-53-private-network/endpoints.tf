# ---------------------------------------------------------------------------
# VPC endpoint cho VPC prod: ECS task kéo image và đọc secret mà không cần NAT
# ---------------------------------------------------------------------------
locals {
  prod_subnet_ids = [for k, s in local.spoke_subnets : aws_subnet.spoke_private[k].id if s.spoke == "prod"]

  interface_endpoints = toset(["ecr.api", "ecr.dkr", "secretsmanager", "sts", "logs"])
}

# Gateway endpoint: miễn phí, chỉ thêm route vào route table, có endpoint policy.
resource "aws_vpc_endpoint" "s3" {
  vpc_id            = aws_vpc.spoke["prod"].id
  service_name      = "com.amazonaws.${var.region}.s3"
  vpc_endpoint_type = "Gateway"
  route_table_ids   = [aws_route_table.spoke_private["prod"].id]
  policy            = file("${path.module}/policies/s3-endpoint-policy.json")

  tags = {
    Name    = "toinoi-prod-s3"
    Service = "network"
  }
}

resource "aws_vpc_endpoint" "dynamodb" {
  vpc_id            = aws_vpc.spoke["prod"].id
  service_name      = "com.amazonaws.${var.region}.dynamodb"
  vpc_endpoint_type = "Gateway"
  route_table_ids   = [aws_route_table.spoke_private["prod"].id]

  tags = {
    Name    = "toinoi-prod-dynamodb"
    Service = "network"
  }
}

resource "aws_security_group" "endpoints" {
  name        = "toinoi-${var.env}-prod-endpoints"
  description = "Cho HTTPS tu VPC prod toi interface endpoint"
  vpc_id      = aws_vpc.spoke["prod"].id

  tags = {
    Name    = "toinoi-prod-endpoints"
    Service = "network"
  }
}

resource "aws_vpc_security_group_ingress_rule" "endpoints_https" {
  security_group_id = aws_security_group.endpoints.id
  description       = "HTTPS tu VPC prod"
  cidr_ipv4         = var.spoke_cidrs["prod"]
  from_port         = 443
  to_port           = 443
  ip_protocol       = "tcp"
}

# Interface endpoint: ENI trong subnet, DNS private đè tên công khai, tính phí theo giờ và GB.
resource "aws_vpc_endpoint" "interface" {
  for_each = local.interface_endpoints

  vpc_id              = aws_vpc.spoke["prod"].id
  service_name        = "com.amazonaws.${var.region}.${each.key}"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = local.prod_subnet_ids
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true

  # Chỉ Secrets Manager có endpoint policy riêng trong lab; các endpoint khác dùng policy mặc định.
  policy = each.key == "secretsmanager" ? file("${path.module}/policies/secretsmanager-endpoint-policy.json") : null

  tags = {
    Name    = "toinoi-prod-${replace(each.key, ".", "-")}"
    Service = "network"
  }
}
