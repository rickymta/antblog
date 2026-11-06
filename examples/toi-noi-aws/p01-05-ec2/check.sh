#!/usr/bin/env bash
# Kiểm service quote trên instance toinoi-lab-quote qua Session Manager (send-command), không cần plugin.
# Chỉ gửi hai lệnh curl vào localhost:8080 của máy; không tạo tài nguyên.
# Dùng: ./check.sh [profile]
set -euo pipefail

profile="${1:-toinoi-lab}"
region="ap-southeast-1"
export AWS_PAGER=""
ro() { aws "$@" --profile "$profile" --region "$region"; }

instance="$(ro ec2 describe-instances \
  --filters Name=tag:Name,Values=toinoi-lab-quote Name=instance-state-name,Values=running \
  --query 'Reservations[0].Instances[0].InstanceId' --output text)"
if [ "$instance" = "None" ] || [ -z "$instance" ]; then
  echo "Không thấy instance toinoi-lab-quote đang chạy." >&2
  exit 1
fi
echo "Instance: $instance"

# Cho container tối đa ~2,5 phút khởi động (cloud-init còn đang cài Docker).
# shellcheck disable=SC2016  # $i và $(seq) phải được shell trên instance mở rộng, không phải ở đây
cmd='for i in $(seq 1 30); do curl -fsS http://localhost:8080/health/ready && break; sleep 5; done; curl -fsS http://localhost:8080/v1/quote'
command_id="$(ro ssm send-command --instance-ids "$instance" --document-name AWS-RunShellScript \
  --parameters "$(printf '{"commands":["%s"]}' "$cmd")" \
  --query Command.CommandId --output text)"

status="Pending"
for _ in $(seq 1 40); do
  sleep 5
  status="$(ro ssm get-command-invocation --command-id "$command_id" --instance-id "$instance" \
    --query Status --output text 2>/dev/null || echo Pending)"
  case "$status" in Success | Failed | TimedOut | Cancelled) break ;; esac
done

echo "Trạng thái lệnh: $status"
ro ssm get-command-invocation --command-id "$command_id" --instance-id "$instance" \
  --query StandardOutputContent --output text
[ "$status" = "Success" ]
