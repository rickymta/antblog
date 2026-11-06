#!/usr/bin/env bash
set -euo pipefail
# Hai target tracking policy (request/target và CPU) + bốn scheduled action cho hai khung cao điểm.
# Giờ Việt Nam = UTC+7. Cron của scheduled action tính theo UTC (mặc định), nên 11:00 VN = 04:00 UTC.
source "$(dirname "$0")/common.sh"
confirm_identity

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
ALB="$(alb_arn)"; TG="$(tg_arn)"
[[ -n "$ALB" && -n "$TG" ]] || { echo "Chạy 01 và 02 trước." >&2; exit 1; }

# Resource label = <phần sau "loadbalancer/" của ARN ALB>/<phần "targetgroup/..." của ARN TG>
LABEL="${ALB#*:loadbalancer/}/${TG##*:}"
echo "ResourceLabel: $LABEL"

log "Target tracking 1: ALBRequestCountPerTarget (tính theo PHÚT, mỗi target)"
cat > "$TMP/tt-req.json" <<JSON
{
  "PredefinedMetricSpecification": {
    "PredefinedMetricType": "ALBRequestCountPerTarget",
    "ResourceLabel": "$LABEL"
  },
  "TargetValue": 600.0
}
JSON
aws autoscaling put-scaling-policy --auto-scaling-group-name "$ASG_NAME" \
  --policy-name toinoi-lab-quote-req-per-target --policy-type TargetTrackingScaling \
  --target-tracking-configuration "file://$TMP/tt-req.json" --query PolicyARN --output text

log "Target tracking 2: CPU trung bình 50%"
cat > "$TMP/tt-cpu.json" <<JSON
{
  "PredefinedMetricSpecification": { "PredefinedMetricType": "ASGAverageCPUUtilization" },
  "TargetValue": 50.0
}
JSON
aws autoscaling put-scaling-policy --auto-scaling-group-name "$ASG_NAME" \
  --policy-name toinoi-lab-quote-cpu --policy-type TargetTrackingScaling \
  --target-tracking-configuration "file://$TMP/tt-cpu.json" --query PolicyARN --output text

# Làm nóng trước 30 phút so với 11:00 và 17:00 (giờ VN). Hành động hạ chỉ nâng/hạ min-size:
# số instance thực tế do target tracking tự co lại, không bị cắt đột ngột đúng 13:00 / 19:00.
log "Scheduled actions (cron UTC)"
put_sched() { # tên, cron, min, max
  aws autoscaling put-scheduled-update-group-action --auto-scaling-group-name "$ASG_NAME" \
    --scheduled-action-name "$1" --recurrence "$2" --min-size "$3" --max-size "$4"
  echo "$1: $2 min=$3 max=$4"
}
put_sched toinoi-lab-lunch-warm   "30 3 * * *"  3 6   # 10:30 VN
put_sched toinoi-lab-lunch-end    "0 6 * * *"   2 6   # 13:00 VN
put_sched toinoi-lab-dinner-warm  "30 9 * * *"  3 6   # 16:30 VN
put_sched toinoi-lab-dinner-end   "0 12 * * *"  2 6   # 19:00 VN

aws autoscaling describe-scheduled-actions --auto-scaling-group-name "$ASG_NAME" \
  --query 'ScheduledUpdateGroupActions[].[ScheduledActionName,Recurrence,MinSize,MaxSize]' --output table
