#!/usr/bin/env bash
# shellcheck disable=SC2016  # dấu backtick trong JMESPath là cố ý
set -euo pipefail
# Dừng service trên MỘT instance (qua SSM) rồi xem ALB và ASG phản ứng khác nhau thế nào.
source "$(dirname "$0")/common.sh"
confirm_identity
TG="$(tg_arn)"

IID="$(aws autoscaling describe-auto-scaling-groups --auto-scaling-group-names "$ASG_NAME" \
  --query 'AutoScalingGroups[0].Instances[?LifecycleState==`InService`]|[0].InstanceId' --output text)"
echo "Instance bị dừng service: $IID"
aws ssm send-command --instance-ids "$IID" --document-name AWS-RunShellScript \
  --parameters 'commands=["systemctl stop toinoi-quote"]' --comment "ToiNoi lab: gia lap loi" \
  --query 'Command.CommandId' --output text

for i in $(seq 1 24); do
  log "Lần kiểm $i"
  echo "ALB (target health):"
  aws elbv2 describe-target-health --target-group-arn "$TG" \
    --query 'TargetHealthDescriptions[].[Target.Id,TargetHealth.State]' --output text
  echo "ASG (health + lifecycle):"
  aws autoscaling describe-auto-scaling-groups --auto-scaling-group-names "$ASG_NAME" \
    --query 'AutoScalingGroups[0].Instances[].[InstanceId,HealthStatus,LifecycleState]' --output text
  sleep 30
done
echo "Hoạt động gần đây của ASG:"
aws autoscaling describe-scaling-activities --auto-scaling-group-name "$ASG_NAME" --max-items 5 \
  --query 'Activities[].[StartTime,StatusCode,Description]' --output text
