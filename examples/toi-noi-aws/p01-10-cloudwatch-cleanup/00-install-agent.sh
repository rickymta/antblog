#!/usr/bin/env bash
set -euo pipefail
# Cài CloudWatch agent lên các instance có tag Service=quote bằng SSM Run Command.
# Chỉ áp dụng cho instance đang chạy. Instance mới của ASG phải có agent ngay từ launch template/AMI (xem bài 10).
source "$(dirname "$0")/common.sh"
confirm_identity

if [[ -n "${EC2_ROLE_NAME:-}" ]]; then
  log "Gắn managed policy CloudWatchAgentServerPolicy vào role $EC2_ROLE_NAME"
  aws iam attach-role-policy --role-name "$EC2_ROLE_NAME" \
    --policy-arn arn:aws:iam::aws:policy/CloudWatchAgentServerPolicy
else
  echo "EC2_ROLE_NAME trống: bạn phải tự bảo đảm role của instance đã có quyền của CloudWatch agent."
fi

log "Lưu cấu hình agent vào SSM Parameter Store ($AGENT_PARAM)"
aws ssm put-parameter --name "$AGENT_PARAM" --type String --overwrite \
  --value "file://$HERE/cwagent-config.json" >/dev/null

log "Chạy cài đặt trên instance Service=quote"
aws ssm send-command --document-name AWS-RunShellScript \
  --targets "Key=tag:Service,Values=quote" \
  --parameters "commands=[\"dnf install -y amazon-cloudwatch-agent\",\"/opt/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl -a fetch-config -m ec2 -s -c ssm:$AGENT_PARAM\"]" \
  --comment "ToiNoi lab: cai CloudWatch agent" \
  --query 'Command.CommandId' --output text
echo "Xem kết quả: aws ssm list-command-invocations --details --query 'CommandInvocations[].[InstanceId,Status]'"
