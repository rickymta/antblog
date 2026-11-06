#!/usr/bin/env bash
# shellcheck disable=SC2016  # dấu backtick trong JMESPath là cố ý
set -euo pipefail
# Mở cổng 80 trên sg-alb của bài 04, tạo ALB, target group và listener HTTP :80.
# ALB tính tiền theo giờ tồn tại và theo LCU (kết nối mới, kết nối đang hoạt động, byte, số rule).
source "$(dirname "$0")/common.sh"
: "${VPC_ID:?Không thấy VPC toinoi-lab-vpc: chạy lab bài 04 hoặc điền VPC_ID trong env.sh}" "${PUBLIC_SUBNET_IDS:?}"
confirm_identity

CIDR="${ALB_INGRESS_CIDR:-}"
if [[ -z "$CIDR" ]]; then CIDR="$(curl -fsS https://checkip.amazonaws.com | tr -d '[:space:]')/32"; fi
echo "ALB chỉ nhận cổng 80 từ: $CIDR"

log "Security group của bài 04: sg-alb (thêm cổng 80 từ IP của bạn) và sg-app (đã nhận 8080 từ sg-alb)"
SG_ALB="$(sg_id "$SG_ALB_NAME")"; SG_APP="$(sg_id "$SG_APP_NAME")"
[[ -n "$SG_ALB" && -n "$SG_APP" ]] || { echo "Không thấy $SG_ALB_NAME / $SG_APP_NAME: chạy lab bài 04 trước." >&2; exit 1; }
# Bài 04 chỉ mở 443; bài này chưa có chứng chỉ (bài 09) nên thử bằng HTTP 80, và chỉ cho IP của bạn.
aws ec2 authorize-security-group-ingress --group-id "$SG_ALB" \
  --ip-permissions "IpProtocol=tcp,FromPort=80,ToPort=80,IpRanges=[{CidrIp=$CIDR,Description=toinoi-lab-08-http}]" \
  >/dev/null 2>&1 || echo "(luật cổng 80 cho $CIDR đã có)"
echo "SG ALB: $SG_ALB, SG app: $SG_APP"

log "Target group với health check /health/ready"
TG="$(tg_arn)"
if [[ -z "$TG" ]]; then
  TG="$(aws elbv2 create-target-group --name "$TG_NAME" \
    --protocol HTTP --port 8080 --vpc-id "$VPC_ID" --target-type instance \
    --health-check-protocol HTTP --health-check-path /health/ready \
    --health-check-interval-seconds 15 --health-check-timeout-seconds 5 \
    --healthy-threshold-count 2 --unhealthy-threshold-count 3 \
    --matcher HttpCode=200 --tags "${TAG_SHORT[@]}" \
    --query 'TargetGroups[0].TargetGroupArn' --output text)"
fi
aws elbv2 modify-target-group-attributes --target-group-arn "$TG" --attributes \
  Key=deregistration_delay.timeout_seconds,Value=30 \
  Key=stickiness.enabled,Value=false >/dev/null
echo "TG: $TG"

log "Application Load Balancer"
ALB="$(alb_arn)"
if [[ -z "$ALB" ]]; then
  # shellcheck disable=SC2086
  ALB="$(aws elbv2 create-load-balancer --name "$ALB_NAME" --type application \
    --scheme internet-facing --ip-address-type ipv4 \
    --subnets $PUBLIC_SUBNET_IDS --security-groups "$SG_ALB" --tags "${TAG_SHORT[@]}" \
    --query 'LoadBalancers[0].LoadBalancerArn' --output text)"
fi
aws elbv2 wait load-balancer-available --load-balancer-arns "$ALB"
aws elbv2 modify-load-balancer-attributes --load-balancer-arn "$ALB" --attributes \
  Key=routing.http.drop_invalid_header_fields.enabled,Value=true >/dev/null

if [[ -z "$(aws elbv2 describe-listeners --load-balancer-arn "$ALB" --query 'Listeners[?Port==`80`].ListenerArn' --output text)" ]]; then
  aws elbv2 create-listener --load-balancer-arn "$ALB" --protocol HTTP --port 80 \
    --default-actions "Type=forward,TargetGroupArn=$TG" >/dev/null
fi
aws elbv2 describe-load-balancers --names "$ALB_NAME" \
  --query 'LoadBalancers[0].{DNS:DNSName,State:State.Code,AZs:AvailabilityZones[].ZoneName}'
