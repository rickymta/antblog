#!/usr/bin/env bash
set -euo pipefail
# Xoá ASG (kèm scheduled action và policy), launch template, ALB (kèm listener), target group,
# và gỡ luật cổng 80 đã thêm vào sg-alb. Hai security group thuộc bài 04 nên để cleanup.sh của bài 04 xoá.
# KHÔNG hoàn tác được. Chỉ chạy trong tài khoản lab.
source "$(dirname "$0")/common.sh"
confirm_identity

log "Xoá Auto Scaling Group (--force-delete kết thúc cả instance)"
if [[ "$(aws autoscaling describe-auto-scaling-groups --auto-scaling-group-names "$ASG_NAME" \
  --query 'length(AutoScalingGroups)' --output text)" != "0" ]]; then
  aws autoscaling delete-auto-scaling-group --auto-scaling-group-name "$ASG_NAME" --force-delete
  for _ in $(seq 1 40); do
    [[ "$(aws autoscaling describe-auto-scaling-groups --auto-scaling-group-names "$ASG_NAME" \
      --query 'length(AutoScalingGroups)' --output text)" == "0" ]] && break
    echo "đợi ASG và instance biến mất..."; sleep 15
  done
fi

log "Xoá launch template"
aws ec2 delete-launch-template --launch-template-name "$LT_NAME" >/dev/null 2>&1 || echo "(không còn)"

log "Xoá ALB và target group"
ALB="$(alb_arn)"
if [[ -n "$ALB" ]]; then
  aws elbv2 delete-load-balancer --load-balancer-arn "$ALB"
  aws elbv2 wait load-balancers-deleted --load-balancer-arns "$ALB"
fi
TG="$(tg_arn)"
if [[ -n "$TG" ]]; then
  for _ in $(seq 1 10); do
    aws elbv2 delete-target-group --target-group-arn "$TG" 2>/dev/null && break
    echo "đợi target group hết bị tham chiếu..."; sleep 10
  done
fi

log "Gỡ luật cổng 80 mà 01-create-alb.sh đã thêm vào sg-alb (giữ nguyên security group của bài 04)"
SG_ALB="$(sg_id "$SG_ALB_NAME")"
if [[ -n "$SG_ALB" ]]; then
  # shellcheck disable=SC2016  # backtick trong JMESPath là cố ý
  for rule in $(aws ec2 describe-security-group-rules --filters "Name=group-id,Values=$SG_ALB" \
    --query 'SecurityGroupRules[?!IsEgress && FromPort==`80` && Description==`toinoi-lab-08-http`].SecurityGroupRuleId' --output text); do
    aws ec2 revoke-security-group-ingress --group-id "$SG_ALB" --security-group-rule-ids "$rule" >/dev/null
    echo "đã gỡ $rule"
  done
fi

log "Kiểm lại"
aws elbv2 describe-load-balancers --query "LoadBalancers[?starts_with(LoadBalancerName, 'toinoi-lab-')].LoadBalancerName" 2>/dev/null || true
aws autoscaling describe-auto-scaling-groups --query "AutoScalingGroups[?starts_with(AutoScalingGroupName, 'toinoi-lab-')].AutoScalingGroupName"
