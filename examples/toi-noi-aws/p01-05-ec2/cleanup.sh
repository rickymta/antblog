#!/usr/bin/env bash
# Dọn lab bài 05: terminate instance, xoá instance profile, role, security group và snapshot của lab.
# MẶC ĐỊNH CHỈ IN LỆNH XOÁ (dry-run); lệnh chỉ-đọc vẫn gọi thật. Thêm --apply để xoá.
# Dùng: ./cleanup.sh [profile] [--apply]
set -euo pipefail

profile="toinoi-lab"
region="ap-southeast-1"
apply=0
for arg in "$@"; do
  case "$arg" in
    --apply) apply=1 ;;
    -*) echo "Tuỳ chọn không hợp lệ: $arg" >&2; exit 2 ;;
    *) profile="$arg" ;;
  esac
done
export AWS_PAGER=""

show() {
  printf '  aws' >&2
  local a quote="'\\''"
  for a in "$@"; do
    if [[ "$a" =~ ^[A-Za-z0-9_./:=@%+,-]+$ ]]; then
      printf ' %s' "$a" >&2
    else
      printf " '%s'" "${a//\'/$quote}" >&2
    fi
  done
  printf '\n' >&2
}
ro() { aws "$@" --profile "$profile" --region "$region"; }
rw() {
  if [ "$apply" -eq 1 ]; then
    aws "$@" --profile "$profile" --region "$region"
  else
    show "$@"
  fi
}
role="toinoi-lab-quote-ec2"
echo "Dọn lab bài 05 ($([ "$apply" -eq 1 ] && echo THẬT || echo dry-run))"

# 1. Instance (chỉ các máy có tag Name=toinoi-lab-quote chưa terminate).
instances="$(ro ec2 describe-instances \
  --filters Name=tag:Name,Values=toinoi-lab-quote Name=tag:Project,Values=ToiNoi \
  Name=instance-state-name,Values=pending,running,stopping,stopped \
  --query 'Reservations[].Instances[].InstanceId' --output text)"
if [ -n "$instances" ]; then
  # shellcheck disable=SC2086  # nhiều ID tách bởi khoảng trắng là cố ý
  rw ec2 terminate-instances --instance-ids $instances >/dev/null
  # shellcheck disable=SC2086
  [ "$apply" -eq 1 ] && aws ec2 wait instance-terminated --instance-ids $instances --profile "$profile" --region "$region"
fi

# 2. Snapshot do lab tạo (tag Name=toinoi-lab-quote-root-snap, chủ sở hữu là chính bạn).
for snap in $(ro ec2 describe-snapshots --owner-ids self \
  --filters Name=tag:Name,Values=toinoi-lab-quote-root-snap Name=tag:Project,Values=ToiNoi \
  --query 'Snapshots[].SnapshotId' --output text); do
  rw ec2 delete-snapshot --snapshot-id "$snap"
done

# 3. Security group riêng của lab (chỉ có ở chế độ default VPC).
sg="$(ro ec2 describe-security-groups --filters Name=group-name,Values=toinoi-lab-quote Name=tag:Project,Values=ToiNoi \
  --query 'SecurityGroups[0].GroupId' --output text)"
if [ "$sg" != "None" ] && [ -n "$sg" ]; then
  rw ec2 delete-security-group --group-id "$sg"
fi

# 4. Instance profile, role.
if ro iam get-role --role-name "$role" >/dev/null 2>&1; then
  rw iam remove-role-from-instance-profile --instance-profile-name "$role" --role-name "$role"
  rw iam delete-instance-profile --instance-profile-name "$role"
  rw iam detach-role-policy --role-name "$role" --policy-arn arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore
  rw iam delete-role --role-name "$role"
fi
[ "$apply" -eq 1 ] && echo "Đã dọn xong." || echo "Dry-run xong; thêm --apply để xoá thật."
