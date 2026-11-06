#!/usr/bin/env bash
# shellcheck disable=SC2016  # dấu backtick trong JMESPath là cố ý
set -euo pipefail
# Tạo launch template và Auto Scaling Group (min 2, desired 2, max 4) gắn vào target group.
# Mỗi instance tính tiền theo giây (tối thiểu 60 giây) cùng EBS gắn kèm. NAT Gateway ở bài 04 vẫn đang tính tiền theo giờ.
source "$(dirname "$0")/common.sh"
: "${APP_SUBNET_IDS:?}" "${INSTANCE_PROFILE_NAME:?}"
confirm_identity

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
SG_APP="$(sg_id "$SG_APP_NAME")"; TG="$(tg_arn)"
[[ -n "$SG_APP" && -n "$TG" ]] || { echo "Chạy 01-create-alb.sh trước." >&2; exit 1; }
# Instance mang sg-app của bài 04, nên cũng tới được RDS ở bài 07 (sg-db nhận 5432 từ sg-app).

AMI="${AMI_ID:-}"
if [[ -z "$AMI" ]]; then
  AMI="$(aws ssm get-parameter \
    --name /aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64 \
    --query Parameter.Value --output text)"
fi
echo "AMI: $AMI"
USER_DATA="$(base64 < "$HERE/user-data.sh" | tr -d '\n')"

log "Launch template (IMDSv2 bắt buộc)"
cat > "$TMP/lt.json" <<JSON
{
  "ImageId": "$AMI",
  "InstanceType": "${INSTANCE_TYPE:-t3.micro}",
  "IamInstanceProfile": { "Name": "$INSTANCE_PROFILE_NAME" },
  "SecurityGroupIds": ["$SG_APP"],
  "MetadataOptions": { "HttpTokens": "required", "HttpEndpoint": "enabled" },
  "UserData": "$USER_DATA",
  "TagSpecifications": [
    { "ResourceType": "instance", "Tags": [
      { "Key": "Project", "Value": "ToiNoi" }, { "Key": "Env", "Value": "lab" },
      { "Key": "Service", "Value": "quote" }, { "Key": "Owner", "Value": "platform" } ] },
    { "ResourceType": "volume", "Tags": [
      { "Key": "Project", "Value": "ToiNoi" }, { "Key": "Env", "Value": "lab" },
      { "Key": "Service", "Value": "quote" }, { "Key": "Owner", "Value": "platform" } ] }
  ]
}
JSON
aws ec2 create-launch-template --launch-template-name "$LT_NAME" \
  --version-description "v1" --launch-template-data "file://$TMP/lt.json" \
  --tag-specifications "ResourceType=launch-template,Tags=[$TAG_SPEC_JSON]" \
  --query 'LaunchTemplate.{Name:LaunchTemplateName,Version:LatestVersionNumber}'

log "Auto Scaling Group: health check theo ELB, grace period 120 giây"
SUBNETS="${APP_SUBNET_IDS// /,}"
aws autoscaling create-auto-scaling-group --auto-scaling-group-name "$ASG_NAME" \
  --launch-template "LaunchTemplateName=$LT_NAME,Version=\$Latest" \
  --min-size 2 --max-size 4 --desired-capacity 2 \
  --vpc-zone-identifier "$SUBNETS" \
  --target-group-arns "$TG" \
  --health-check-type ELB --health-check-grace-period 120 \
  --tags \
  "ResourceId=$ASG_NAME,ResourceType=auto-scaling-group,Key=Project,Value=ToiNoi,PropagateAtLaunch=true" \
  "ResourceId=$ASG_NAME,ResourceType=auto-scaling-group,Key=Env,Value=lab,PropagateAtLaunch=true" \
  "ResourceId=$ASG_NAME,ResourceType=auto-scaling-group,Key=Service,Value=quote,PropagateAtLaunch=true" \
  "ResourceId=$ASG_NAME,ResourceType=auto-scaling-group,Key=Owner,Value=platform,PropagateAtLaunch=true"

echo "Đợi các target healthy (thường 2-4 phút)..."
for _ in $(seq 1 30); do
  H="$(aws elbv2 describe-target-health --target-group-arn "$TG" \
    --query 'length(TargetHealthDescriptions[?TargetHealth.State==`healthy`])' --output text)"
  echo "healthy: $H/2"
  [[ "$H" -ge 2 ]] && break
  sleep 20
done
