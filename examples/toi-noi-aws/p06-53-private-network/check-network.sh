#!/usr/bin/env bash
# Kiểm (chỉ ĐỌC) mạng riêng sau khi `terraform apply`: endpoint, route của TGW và trạng thái firewall.
# Cần: terraform đã apply trong thư mục này, jq, AWS CLI v2.
# Dùng: ./check-network.sh [profile]
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
profile="${1:-toinoi-lab}"
export AWS_PAGER=""
out="$(terraform -chdir="$here" output -json)"

echo "== VPC endpoint của VPC prod (mong đợi: gateway s3, dynamodb; interface ecr.api, ecr.dkr, secretsmanager, sts, logs)"
aws ec2 describe-vpc-endpoints --profile "$profile" \
  --filters "Name=tag:Project,Values=ToiNoi" \
  --query 'VpcEndpoints[].[ServiceName,VpcEndpointType,State]' --output table

echo
echo "== Route active trong từng route table của TGW"
for name in $(printf '%s' "$out" | jq -r '.tgw_route_table_ids.value | keys[]'); do
  rt="$(printf '%s' "$out" | jq -r --arg n "$name" '.tgw_route_table_ids.value[$n]')"
  echo "-- $name ($rt)"
  aws ec2 search-transit-gateway-routes --profile "$profile" \
    --transit-gateway-route-table-id "$rt" --filters "Name=state,Values=active" \
    --query 'Routes[].[DestinationCidrBlock,Type,State]' --output table
done
echo "Mong đợi: dev và prod chỉ có 0.0.0.0/0 sang inspection (không có CIDR của nhau); inspection có CIDR của dev và prod (propagated)."

fw="$(printf '%s' "$out" | jq -r '.firewall_name.value // empty')"
if [ -n "$fw" ]; then
  echo
  echo "== Trạng thái Network Firewall $fw (mong đợi READY)"
  aws network-firewall describe-firewall --profile "$profile" --firewall-name "$fw" \
    --query 'FirewallStatus.Status' --output text
fi

svc="$(printf '%s' "$out" | jq -r '.reconcile_endpoint_service_name.value // empty')"
if [ -n "$svc" ]; then
  echo
  echo "== Endpoint service để gửi cho đối tác: $svc"
fi
