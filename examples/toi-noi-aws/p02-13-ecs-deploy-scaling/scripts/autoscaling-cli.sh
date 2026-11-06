#!/usr/bin/env bash
# Cấu hình Application Auto Scaling cho service ECS bằng AWS CLI v2:
# scalable target, target tracking theo request mỗi task, scheduled action theo giờ Việt Nam.
# Script chỉ tạo cấu hình scaling (không tính tiền riêng); task được thêm sẽ tính tiền Fargate.
set -euo pipefail

: "${CLUSTER:=toinoi-lab}"
: "${SERVICE:=toinoi-lab-booking-alb}"
: "${ALB_ARN_SUFFIX:?Dạng app/toinoi-lab-alb/<id>; lấy từ terraform state hoặc: aws elbv2 describe-load-balancers}"
: "${TG_ARN_SUFFIX:?Dạng targetgroup/toinoi-lab-booking/<id>}"
: "${MIN_CAPACITY:=2}"
: "${MAX_CAPACITY:=20}"
: "${PEAK_MIN:=6}"

RESOURCE_ID="service/${CLUSTER}/${SERVICE}"
HERE="$(cd "$(dirname "$0")/.." && pwd)"
POLICY="$(mktemp)"
trap 'rm -f "${POLICY}"' EXIT

aws application-autoscaling register-scalable-target \
  --service-namespace ecs \
  --scalable-dimension ecs:service:DesiredCount \
  --resource-id "${RESOURCE_ID}" \
  --min-capacity "${MIN_CAPACITY}" --max-capacity "${MAX_CAPACITY}"

export RESOURCE_LABEL="${ALB_ARN_SUFFIX}/${TG_ARN_SUFFIX}"
envsubst '${RESOURCE_LABEL}' < "${HERE}/target-tracking-policy.json" > "${POLICY}"

aws application-autoscaling put-scaling-policy \
  --service-namespace ecs \
  --scalable-dimension ecs:service:DesiredCount \
  --resource-id "${RESOURCE_ID}" \
  --policy-name toinoi-lab-requests-per-target \
  --policy-type TargetTrackingScaling \
  --target-tracking-scaling-policy-configuration "file://${POLICY}"

put_schedule() { # tên, cron, min
  aws application-autoscaling put-scheduled-action \
    --service-namespace ecs \
    --scalable-dimension ecs:service:DesiredCount \
    --resource-id "${RESOURCE_ID}" \
    --scheduled-action-name "$1" \
    --schedule "$2" \
    --timezone Asia/Ho_Chi_Minh \
    --scalable-target-action "MinCapacity=$3,MaxCapacity=${MAX_CAPACITY}"
}

put_schedule toinoi-lab-lunch-start  "cron(45 10 * * ? *)" "${PEAK_MIN}"
put_schedule toinoi-lab-lunch-end    "cron(15 13 * * ? *)" "${MIN_CAPACITY}"
put_schedule toinoi-lab-dinner-start "cron(45 16 * * ? *)" "${PEAK_MIN}"
put_schedule toinoi-lab-dinner-end   "cron(15 19 * * ? *)" "${MIN_CAPACITY}"

aws application-autoscaling describe-scaling-policies --service-namespace ecs --resource-id "${RESOURCE_ID}" \
  --query 'ScalingPolicies[].PolicyName'
aws application-autoscaling describe-scheduled-actions --service-namespace ecs --resource-id "${RESOURCE_ID}" \
  --query 'ScheduledActions[].[ScheduledActionName,Schedule,Timezone]' --output table
