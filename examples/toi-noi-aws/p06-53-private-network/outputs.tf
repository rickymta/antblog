output "transit_gateway_id" {
  description = "ID của Transit Gateway."
  value       = aws_ec2_transit_gateway.main.id
}

output "tgw_route_table_ids" {
  description = "Route table TGW theo segment, dùng với search-transit-gateway-routes."
  value = merge(
    { for k, rt in aws_ec2_transit_gateway_route_table.spoke : k => rt.id },
    { inspection = aws_ec2_transit_gateway_route_table.inspection.id },
  )
}

output "firewall_name" {
  description = "Tên Network Firewall (null nếu tắt)."
  value       = one(aws_networkfirewall_firewall.main[*].name)
}

output "prod_vpc_endpoint_ids" {
  description = "ID các VPC endpoint của VPC prod."
  value = merge(
    { s3 = aws_vpc_endpoint.s3.id, dynamodb = aws_vpc_endpoint.dynamodb.id },
    { for k, e in aws_vpc_endpoint.interface : k => e.id },
  )
}

output "reconcile_endpoint_service_name" {
  description = "Tên service để đối tác tạo interface endpoint (null nếu tắt)."
  value       = one(aws_vpc_endpoint_service.reconcile[*].service_name)
}
