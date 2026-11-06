# ---------------------------------------------------------------------------
# Transit Gateway: hub nối dev, prod và inspection. Tắt association và propagation mặc định
# để chính bạn quyết định VPC nào thấy VPC nào bằng route table riêng.
# ---------------------------------------------------------------------------
resource "aws_ec2_transit_gateway" "main" {
  description                     = "Hub mang Toi Noi: dev, prod va inspection"
  default_route_table_association = "disable"
  default_route_table_propagation = "disable"
  dns_support                     = "enable"

  tags = {
    Name    = "toinoi-${var.env}-tgw"
    Service = "network"
  }
}

resource "aws_ec2_transit_gateway_vpc_attachment" "spoke" {
  for_each = var.spoke_cidrs

  transit_gateway_id = aws_ec2_transit_gateway.main.id
  vpc_id             = aws_vpc.spoke[each.key].id
  subnet_ids         = [for k, s in local.spoke_subnets : aws_subnet.spoke_private[k].id if s.spoke == each.key]

  # Nếu để mặc định, TGW tự gắn attachment vào route table mặc định và ta mất kiểm soát segment.
  transit_gateway_default_route_table_association = false
  transit_gateway_default_route_table_propagation = false

  tags = {
    Name    = "toinoi-${each.key}-attachment"
    Service = "network"
  }
}

resource "aws_ec2_transit_gateway_vpc_attachment" "inspection" {
  transit_gateway_id = aws_ec2_transit_gateway.main.id
  vpc_id             = aws_vpc.inspection.id
  subnet_ids         = [for s in aws_subnet.inspection_tgw : s.id]

  # Appliance mode giữ hai chiều của một flow đi qua cùng một firewall endpoint (cùng AZ).
  appliance_mode_support = "enable"

  transit_gateway_default_route_table_association = false
  transit_gateway_default_route_table_propagation = false

  tags = {
    Name    = "toinoi-inspection-attachment"
    Service = "network"
  }
}

# Hai route table cho hai segment spoke, và một route table cho inspection.
resource "aws_ec2_transit_gateway_route_table" "spoke" {
  for_each = var.spoke_cidrs

  transit_gateway_id = aws_ec2_transit_gateway.main.id

  tags = {
    Name    = "toinoi-tgw-rt-${each.key}"
    Service = "network"
  }
}

resource "aws_ec2_transit_gateway_route_table" "inspection" {
  transit_gateway_id = aws_ec2_transit_gateway.main.id

  tags = {
    Name    = "toinoi-tgw-rt-inspection"
    Service = "network"
  }
}

resource "aws_ec2_transit_gateway_route_table_association" "spoke" {
  for_each = var.spoke_cidrs

  transit_gateway_attachment_id  = aws_ec2_transit_gateway_vpc_attachment.spoke[each.key].id
  transit_gateway_route_table_id = aws_ec2_transit_gateway_route_table.spoke[each.key].id
}

resource "aws_ec2_transit_gateway_route_table_association" "inspection" {
  transit_gateway_attachment_id  = aws_ec2_transit_gateway_vpc_attachment.inspection.id
  transit_gateway_route_table_id = aws_ec2_transit_gateway_route_table.inspection.id
}

# Route table của spoke KHÔNG có route tới spoke khác: chỉ có mặc định 0.0.0.0/0 sang inspection.
resource "aws_ec2_transit_gateway_route" "spoke_default_to_inspection" {
  for_each = var.spoke_cidrs

  destination_cidr_block         = "0.0.0.0/0"
  transit_gateway_attachment_id  = aws_ec2_transit_gateway_vpc_attachment.inspection.id
  transit_gateway_route_table_id = aws_ec2_transit_gateway_route_table.spoke[each.key].id
}

# Route table inspection học CIDR của hai spoke để đường về hoạt động.
resource "aws_ec2_transit_gateway_route_table_propagation" "spoke_to_inspection" {
  for_each = var.spoke_cidrs

  transit_gateway_attachment_id  = aws_ec2_transit_gateway_vpc_attachment.spoke[each.key].id
  transit_gateway_route_table_id = aws_ec2_transit_gateway_route_table.inspection.id
}
