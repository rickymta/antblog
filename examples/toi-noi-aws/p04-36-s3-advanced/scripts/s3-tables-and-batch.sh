#!/usr/bin/env bash
# Các lệnh tham khảo cho S3 Tables, Access Point và Storage Lens. Đọc từng lệnh rồi tự chạy: script KHÔNG chạy gì khi bạn chỉ mở nó.
# Dùng `bash scripts/s3-tables-and-batch.sh run-tables` để tạo bảng Iceberg mẫu trong table bucket đã dựng bằng Terraform.
set -euo pipefail

REGION="${AWS_REGION:-ap-southeast-1}"

usage() {
  cat <<USAGE
Cách dùng: $0 <lệnh>
  run-tables      Tạo bảng Iceberg trips.trip_events trong table bucket toinoi-lab-trips-tables
  access-point    In lệnh tạo access point VPC-only cho bucket ảnh (chỉ in, không chạy)
USAGE
}

case "${1:-}" in
  run-tables)
    : "${AWS_PROFILE:?Đặt AWS_PROFILE trỏ vào tài khoản lab}"
    ACCOUNT_ID="$(aws sts get-caller-identity --query Account --output text)"
    ARN="arn:aws:s3tables:${REGION}:${ACCOUNT_ID}:bucket/toinoi-lab-trips-tables"
    echo "Tạo bảng trips.trip_events trong ${ARN}"
    read -r -p "Tiếp tục? [y/N] " ans
    [ "$ans" = "y" ] || exit 0
    aws s3tables create-table --region "$REGION" --table-bucket-arn "$ARN" \
      --namespace trips --name trip_events --format ICEBERG
    aws s3tables list-tables --region "$REGION" --table-bucket-arn "$ARN" --namespace trips
    ;;
  access-point)
    cat <<'CMD'
aws s3control create-access-point \
  --account-id 123456789012 \
  --name toinoi-lab-support-ap \
  --bucket toinoi-lab-delivery-photos-123456789012 \
  --vpc-configuration VpcId=vpc-0abc1234
CMD
    ;;
  *)
    usage
    ;;
esac
