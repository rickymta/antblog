#!/usr/bin/env bash
# Hàm dùng chung. File này được `source`, không chạy trực tiếp.
# shellcheck disable=SC2034,SC2054
set -euo pipefail
export AWS_PAGER=""

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ ! -f "$HERE/env.sh" ]]; then
  echo "Thiếu env.sh. Chạy: cp env.sh.example env.sh rồi sửa giá trị." >&2
  exit 1
fi
# shellcheck source=/dev/null
source "$HERE/env.sh"

export AWS_DEFAULT_REGION="${AWS_REGION:?AWS_REGION chưa đặt}"
if [[ -z "${AWS_PROFILE:-}" ]]; then unset AWS_PROFILE; fi

# Cùng tên với lab bài 07 và 08.
ALB_NAME="toinoi-lab-quote-alb"
TG_NAME="toinoi-lab-quote-tg"
ASG_NAME="toinoi-lab-quote-asg"
DB_ID="${DB_ID:-toinoi-lab-trips}"

TOPIC_NAME="toinoi-lab-alerts"
LOG_GROUP="/toinoi/lab/quote"
METRIC_NS="ToiNoi/Quote"
DASHBOARD_NAME="toinoi-lab-quote"
BUDGET_NAME="toinoi-lab-monthly"   # cùng tên với budget của bài 02: đã có thì dùng lại
# Tiền tố AmazonCloudWatch- khớp quyền ssm:GetParameter có sẵn trong CloudWatchAgentServerPolicy.
AGENT_PARAM="AmazonCloudWatch-toinoi-lab-quote"
TAGS_KV=(Key=Project,Value=ToiNoi Key=Env,Value=lab Key=Service,Value=quote Key=Owner,Value=platform)

log() { printf '\n== %s\n' "$*"; }

confirm_identity() {
  log "Danh tính đang dùng"
  aws sts get-caller-identity --output table
  if [[ "${ASSUME_YES:-0}" != "1" ]]; then
    read -r -p "Đây có phải tài khoản LAB của bạn không? Gõ 'yes' để tiếp tục: " ans
    [[ "$ans" == "yes" ]] || { echo "Dừng." >&2; exit 1; }
  fi
}

ACCOUNT_ID="$(aws sts get-caller-identity --query Account --output text)"
TOPIC_ARN="arn:aws:sns:${AWS_REGION}:${ACCOUNT_ID}:${TOPIC_NAME}"

# Các dimension của ALB/target group lấy từ ARN (rỗng nếu chưa có tài nguyên).
alb_dim() {
  local arn
  arn="$(aws elbv2 describe-load-balancers --names "$ALB_NAME" --query 'LoadBalancers[0].LoadBalancerArn' --output text 2>/dev/null || true)"
  [[ -n "$arn" && "$arn" != "None" ]] && printf '%s' "${arn#*:loadbalancer/}" || true
}
tg_dim() {
  local arn
  arn="$(aws elbv2 describe-target-groups --names "$TG_NAME" --query 'TargetGroups[0].TargetGroupArn' --output text 2>/dev/null || true)"
  [[ -n "$arn" && "$arn" != "None" ]] && printf '%s' "${arn##*:}" || true
}
