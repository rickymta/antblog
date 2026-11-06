data "aws_availability_zones" "available" {
  state = "available"
}

locals {
  azs    = slice(data.aws_availability_zones.available.names, 0, 2)
  az_idx = { for i, az in local.azs : tostring(i) => az }

  # Mỗi (spoke, AZ) một subnet private.
  spoke_subnets = {
    for pair in setproduct(keys(var.spoke_cidrs), keys(local.az_idx)) :
    "${pair[0]}-${pair[1]}" => {
      spoke = pair[0]
      idx   = tonumber(pair[1])
      az    = local.az_idx[pair[1]]
    }
  }
}

# ---------------------------------------------------------------------------
# VPC spoke: dev và prod, chỉ có subnet private
# ---------------------------------------------------------------------------
resource "aws_vpc" "spoke" {
  for_each = var.spoke_cidrs

  cidr_block           = each.value
  enable_dns_support   = true
  enable_dns_hostnames = true

  tags = {
    Name    = "toinoi-${each.key}-vpc"
    Service = "network"
  }
}

resource "aws_subnet" "spoke_private" {
  for_each = local.spoke_subnets

  vpc_id            = aws_vpc.spoke[each.value.spoke].id
  cidr_block        = cidrsubnet(var.spoke_cidrs[each.value.spoke], 8, each.value.idx)
  availability_zone = each.value.az

  tags = {
    Name    = "toinoi-${each.value.spoke}-private-${each.value.idx}"
    Service = "network"
  }
}

resource "aws_route_table" "spoke_private" {
  for_each = var.spoke_cidrs

  vpc_id = aws_vpc.spoke[each.key].id

  tags = {
    Name    = "toinoi-${each.key}-private"
    Service = "network"
  }
}

resource "aws_route_table_association" "spoke_private" {
  for_each = local.spoke_subnets

  subnet_id      = aws_subnet.spoke_private[each.key].id
  route_table_id = aws_route_table.spoke_private[each.value.spoke].id
}

# Mọi thứ không phải của VPC đi qua Transit Gateway (tới firewall ở VPC inspection).
resource "aws_route" "spoke_default_to_tgw" {
  for_each = var.spoke_cidrs

  route_table_id         = aws_route_table.spoke_private[each.key].id
  destination_cidr_block = "0.0.0.0/0"
  transit_gateway_id     = aws_ec2_transit_gateway.main.id

  depends_on = [aws_ec2_transit_gateway_vpc_attachment.spoke]
}

# ---------------------------------------------------------------------------
# VPC inspection: subnet cho TGW attachment, subnet cho firewall endpoint, (tuỳ chọn) public + NAT
# ---------------------------------------------------------------------------
resource "aws_vpc" "inspection" {
  cidr_block           = var.inspection_cidr
  enable_dns_support   = true
  enable_dns_hostnames = true

  tags = {
    Name    = "toinoi-inspection-vpc"
    Service = "network"
  }
}

resource "aws_subnet" "inspection_tgw" {
  for_each = local.az_idx

  vpc_id            = aws_vpc.inspection.id
  cidr_block        = cidrsubnet(var.inspection_cidr, 8, tonumber(each.key))
  availability_zone = each.value

  tags = {
    Name    = "toinoi-inspection-tgw-${each.key}"
    Service = "network"
  }
}

resource "aws_subnet" "inspection_firewall" {
  for_each = local.az_idx

  vpc_id            = aws_vpc.inspection.id
  cidr_block        = cidrsubnet(var.inspection_cidr, 8, 10 + tonumber(each.key))
  availability_zone = each.value

  tags = {
    Name    = "toinoi-inspection-firewall-${each.key}"
    Service = "network"
  }
}

resource "aws_subnet" "inspection_public" {
  for_each = var.enable_egress_nat ? local.az_idx : {}

  vpc_id            = aws_vpc.inspection.id
  cidr_block        = cidrsubnet(var.inspection_cidr, 8, 20 + tonumber(each.key))
  availability_zone = each.value

  tags = {
    Name    = "toinoi-inspection-public-${each.key}"
    Service = "network"
  }
}

resource "aws_internet_gateway" "inspection" {
  count = var.enable_egress_nat ? 1 : 0

  vpc_id = aws_vpc.inspection.id

  tags = {
    Name    = "toinoi-inspection-igw"
    Service = "network"
  }
}

resource "aws_eip" "nat" {
  for_each = var.enable_egress_nat ? local.az_idx : {}

  domain = "vpc"

  tags = {
    Name    = "toinoi-inspection-nat-${each.key}"
    Service = "network"
  }
}

resource "aws_nat_gateway" "inspection" {
  for_each = var.enable_egress_nat ? local.az_idx : {}

  allocation_id = aws_eip.nat[each.key].id
  subnet_id     = aws_subnet.inspection_public[each.key].id

  tags = {
    Name    = "toinoi-inspection-nat-${each.key}"
    Service = "network"
  }

  depends_on = [aws_internet_gateway.inspection]
}

# Firewall endpoint theo AZ: tra từ trạng thái đồng bộ của firewall.
locals {
  firewall_endpoints = var.enable_network_firewall ? {
    for s in aws_networkfirewall_firewall.main[0].firewall_status[0].sync_states :
    s.availability_zone => s.attachment[0].endpoint_id
  } : {}
}

# Route table của subnet TGW attachment: mọi lưu lượng vào VPC inspection phải qua firewall.
resource "aws_route_table" "inspection_tgw" {
  for_each = local.az_idx

  vpc_id = aws_vpc.inspection.id

  tags = {
    Name    = "toinoi-inspection-tgw-${each.key}"
    Service = "network"
  }
}

resource "aws_route_table_association" "inspection_tgw" {
  for_each = local.az_idx

  subnet_id      = aws_subnet.inspection_tgw[each.key].id
  route_table_id = aws_route_table.inspection_tgw[each.key].id
}

resource "aws_route" "inspection_tgw_to_firewall" {
  for_each = var.enable_network_firewall ? local.az_idx : {}

  route_table_id         = aws_route_table.inspection_tgw[each.key].id
  destination_cidr_block = "0.0.0.0/0"
  vpc_endpoint_id        = local.firewall_endpoints[each.value]
}

# Route table của subnet firewall: về spoke qua TGW; ra Internet qua NAT (nếu bật).
resource "aws_route_table" "inspection_firewall" {
  for_each = local.az_idx

  vpc_id = aws_vpc.inspection.id

  tags = {
    Name    = "toinoi-inspection-firewall-${each.key}"
    Service = "network"
  }
}

resource "aws_route_table_association" "inspection_firewall" {
  for_each = local.az_idx

  subnet_id      = aws_subnet.inspection_firewall[each.key].id
  route_table_id = aws_route_table.inspection_firewall[each.key].id
}

resource "aws_route" "inspection_firewall_to_spokes" {
  for_each = local.az_idx

  route_table_id         = aws_route_table.inspection_firewall[each.key].id
  destination_cidr_block = "10.0.0.0/8"
  transit_gateway_id     = aws_ec2_transit_gateway.main.id

  depends_on = [aws_ec2_transit_gateway_vpc_attachment.inspection]
}

resource "aws_route" "inspection_firewall_to_nat" {
  for_each = var.enable_egress_nat ? local.az_idx : {}

  route_table_id         = aws_route_table.inspection_firewall[each.key].id
  destination_cidr_block = "0.0.0.0/0"
  nat_gateway_id         = aws_nat_gateway.inspection[each.key].id
}

# Subnet public (nếu bật): ra Internet qua IGW, về spoke qua firewall.
resource "aws_route_table" "inspection_public" {
  for_each = var.enable_egress_nat ? local.az_idx : {}

  vpc_id = aws_vpc.inspection.id

  tags = {
    Name    = "toinoi-inspection-public-${each.key}"
    Service = "network"
  }
}

resource "aws_route_table_association" "inspection_public" {
  for_each = var.enable_egress_nat ? local.az_idx : {}

  subnet_id      = aws_subnet.inspection_public[each.key].id
  route_table_id = aws_route_table.inspection_public[each.key].id
}

resource "aws_route" "inspection_public_to_igw" {
  for_each = var.enable_egress_nat ? local.az_idx : {}

  route_table_id         = aws_route_table.inspection_public[each.key].id
  destination_cidr_block = "0.0.0.0/0"
  gateway_id             = aws_internet_gateway.inspection[0].id
}

resource "aws_route" "inspection_public_to_spokes" {
  for_each = var.enable_egress_nat && var.enable_network_firewall ? local.az_idx : {}

  route_table_id         = aws_route_table.inspection_public[each.key].id
  destination_cidr_block = "10.0.0.0/8"
  vpc_endpoint_id        = local.firewall_endpoints[each.value]
}
