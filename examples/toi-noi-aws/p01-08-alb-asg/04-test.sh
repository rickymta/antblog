#!/usr/bin/env bash
set -euo pipefail
# Gọi ALB vài lần để thấy request chia cho nhiều instance; xem trạng thái target.
source "$(dirname "$0")/common.sh"
TG="$(tg_arn)"
DNS="$(aws elbv2 describe-load-balancers --names "$ALB_NAME" --query 'LoadBalancers[0].DNSName' --output text)"
echo "ALB DNS: $DNS"

log "10 request /v1/quote: trường host phải xuất hiện từ hai instance"
for _ in $(seq 1 10); do curl -fsS "http://$DNS/v1/quote?distanceKm=5"; echo; done

log "/boom trả 500 và ghi ERROR vào log (dùng ở bài 10)"
curl -sS -o /dev/null -w '%{http_code}\n' "http://$DNS/boom"

log "Trạng thái target"
aws elbv2 describe-target-health --target-group-arn "$TG" \
  --query 'TargetHealthDescriptions[].[Target.Id,TargetHealth.State,TargetHealth.Reason]' --output table
