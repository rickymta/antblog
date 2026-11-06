#!/usr/bin/env bash
set -euo pipefail
# Dọn toàn bộ tài nguyên Phần 01 do các lab 06, 07, 08, 10 tạo, theo thứ tự ngược với lúc dựng.
# KHÔNG hoàn tác được. Lab 02-05 (CLI, IAM, VPC, EC2) dọn bằng README của chúng, script này chỉ nhắc.
source "$(dirname "$0")/common.sh"
confirm_identity

log "1) Lab 10: alarm, dashboard, log group, SNS, budget (chung tên với bài 02), cấu hình agent"
aws cloudwatch delete-alarms --alarm-names toinoi-lab-quote-target-5xx toinoi-lab-quote-unhealthy-hosts toinoi-lab-quote-log-errors 2>/dev/null || true
aws cloudwatch delete-dashboards --dashboard-names "$DASHBOARD_NAME" 2>/dev/null || true
aws logs delete-metric-filter --log-group-name "$LOG_GROUP" --filter-name quote-error-count 2>/dev/null || true
aws logs delete-log-group --log-group-name "$LOG_GROUP" 2>/dev/null || true
aws sns delete-topic --topic-arn "$TOPIC_ARN" 2>/dev/null || true
aws budgets delete-budget --region us-east-1 --account-id "$ACCOUNT_ID" --budget-name "$BUDGET_NAME" 2>/dev/null || true
aws ssm delete-parameter --name "$AGENT_PARAM" 2>/dev/null || true
if [[ -n "${EC2_ROLE_NAME:-}" ]]; then
  aws iam detach-role-policy --role-name "$EC2_ROLE_NAME" \
    --policy-arn arn:aws:iam::aws:policy/CloudWatchAgentServerPolicy 2>/dev/null || true
fi

run_lab_cleanup() { # thư mục lab, tên hiển thị
  local script="$HERE/../$1/99-cleanup.sh"
  if [[ -f "$script" && -f "$HERE/../$1/env.sh" ]]; then
    log "$2"
    ASSUME_YES=1 bash "$script"
  else
    log "$2: bỏ qua (thiếu $1/env.sh hoặc script). Dọn tay theo README của lab đó."
  fi
}
run_lab_cleanup p01-08-alb-asg "2) Lab 08: ALB, ASG, launch template, security group"
run_lab_cleanup p01-07-rds     "3) Lab 07: RDS instance, snapshot, subnet group, parameter group"
run_lab_cleanup p01-06-s3      "4) Lab 06: bucket S3 (kể cả mọi version)"

cat <<'TXT'

5) Còn lại phải dọn bằng script của lab tương ứng, theo thứ tự (mỗi script mặc định chỉ in lệnh):
   - Lab 05: ../p01-05-ec2/cleanup.sh toinoi-lab --apply   (instance, snapshot, role và instance profile)
             hoặc terraform destroy trong p01-05-ec2/terraform nếu đã dựng bằng Terraform.
   - Lab 04: ../p01-04-vpc/cleanup.sh toinoi-lab --apply   (NAT Gateway trước, rồi Elastic IP, subnet,
             route table, IGW, security group, VPC) hoặc terraform destroy trong p01-04-vpc/terraform.
   - Lab 03: ../p01-03-iam-policies/cleanup.sh toinoi-lab --apply
   - Lab 02: budget đã xoá ở bước 1; user, group, permission set của IAM Identity Center giữ lại nếu còn học tiếp.
   - Bài 09: bản ghi Route 53 và hosted zone nếu bạn đã tạo; chứng chỉ ACM không dùng nữa.
TXT

log "6) Kiểm lại phần còn sót"
ASSUME_YES=1 bash "$HERE/90-audit-leftovers.sh"
