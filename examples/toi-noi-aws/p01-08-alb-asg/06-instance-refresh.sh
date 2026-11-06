#!/usr/bin/env bash
set -euo pipefail
# Tạo launch template version mới (đổi instance type, hoặc đặt AMI_ID mới) và để ASG thay dần từng instance.
source "$(dirname "$0")/common.sh"
confirm_identity

NEW_TYPE="${NEW_INSTANCE_TYPE:-t3.small}"
DATA="{\"InstanceType\":\"$NEW_TYPE\"}"
if [[ -n "${AMI_ID:-}" ]]; then DATA="{\"InstanceType\":\"$NEW_TYPE\",\"ImageId\":\"$AMI_ID\"}"; fi

log "Launch template version mới"
aws ec2 create-launch-template-version --launch-template-name "$LT_NAME" \
  --source-version 1 --version-description "doi instance type" \
  --launch-template-data "$DATA" --query 'LaunchTemplateVersion.VersionNumber' --output text

log "Instance refresh: giữ tối thiểu 50% công suất khoẻ trong lúc thay"
aws autoscaling start-instance-refresh --auto-scaling-group-name "$ASG_NAME" \
  --desired-configuration "LaunchTemplate={LaunchTemplateName=$LT_NAME,Version=\$Latest}" \
  --preferences "MinHealthyPercentage=50,InstanceWarmup=120"
echo "Theo dõi: aws autoscaling describe-instance-refreshes --auto-scaling-group-name $ASG_NAME"
