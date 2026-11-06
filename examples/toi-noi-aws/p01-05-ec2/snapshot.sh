#!/usr/bin/env bash
# Chụp snapshot ổ root của instance toinoi-lab-quote (minh hoạ EBS snapshot). Snapshot TÍNH TIỀN theo GB lưu.
# MẶC ĐỊNH CHỈ IN LỆNH (dry-run). Xoá lại bằng ./cleanup.sh. Dùng: ./snapshot.sh [profile] [--apply]
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

volume="$(aws ec2 describe-instances --profile "$profile" --region "$region" \
  --filters Name=tag:Name,Values=toinoi-lab-quote Name=instance-state-name,Values=running \
  --query 'Reservations[0].Instances[0].BlockDeviceMappings[0].Ebs.VolumeId' --output text)"
if [ "$volume" = "None" ] || [ -z "$volume" ]; then
  echo "Không thấy instance toinoi-lab-quote đang chạy." >&2
  exit 1
fi

tag_spec='ResourceType=snapshot,Tags=[{Key=Name,Value=toinoi-lab-quote-root-snap},{Key=Project,Value=ToiNoi},{Key=Env,Value=lab},{Key=Service,Value=quote},{Key=Owner,Value=platform}]'
if [ "$apply" -eq 1 ]; then
  aws ec2 create-snapshot --volume-id "$volume" --description "Toi Noi lab: quote root volume" \
    --tag-specifications "$tag_spec" --query SnapshotId --output text --profile "$profile" --region "$region"
else
  echo "Dry-run: sẽ chạy"
  echo "  aws ec2 create-snapshot --volume-id $volume --description 'Toi Noi lab: quote root volume' --tag-specifications '$tag_spec'"
fi
