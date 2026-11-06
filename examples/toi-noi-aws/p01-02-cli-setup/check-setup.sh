#!/usr/bin/env bash
# Kiểm thiết lập AWS CLI v2 và profile SSO. Chỉ ĐỌC, không tạo tài nguyên nào.
# Dùng: ./check-setup.sh [profile]      (mặc định: $AWS_PROFILE hoặc toinoi-lab)
set -euo pipefail

profile="${1:-${AWS_PROFILE:-toinoi-lab}}"
expected_region="ap-southeast-1"
problems=0
export AWS_PAGER=""

ok() { printf 'OK   %s\n' "$*"; }
bad() { printf 'LỖI  %s\n' "$*" >&2; problems=$((problems + 1)); }

if ! command -v aws >/dev/null 2>&1; then
  bad "không tìm thấy lệnh aws; cài AWS CLI v2 trước (xem bài 02)"
  exit 1
fi

version="$(aws --version 2>&1)"
case "$version" in
  aws-cli/2.*) ok "$version" ;;
  *) bad "cần AWS CLI v2, đang có: $version" ;;
esac

# Biến môi trường chứa key dài hạn sẽ ghi đè profile SSO.
for name in AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_SESSION_TOKEN; do
  if [ -n "${!name:-}" ]; then
    bad "biến $name đang được đặt và sẽ ghi đè profile; chạy: unset $name"
  fi
done

if aws configure list-profiles 2>/dev/null | grep -Fxq "$profile"; then
  ok "profile '$profile' có trong cấu hình"
else
  bad "không có profile '$profile'; chạy 'aws configure sso' hoặc xem config.example"
  exit 1
fi

region="$(aws configure get region --profile "$profile" 2>/dev/null || true)"
if [ "$region" = "$expected_region" ]; then
  ok "region mặc định của profile là $region"
else
  bad "region mặc định là '${region:-chưa đặt}', cả loạt dùng $expected_region"
fi

if identity="$(aws sts get-caller-identity --profile "$profile" \
  --query '[Account,Arn]' --output text 2>&1)"; then
  account="${identity%%$'\t'*}"
  arn="${identity#*$'\t'}"
  ok "đang đăng nhập: $arn (account $account)"
  if [ "$account" = "123456789012" ]; then
    printf 'LƯU Ý 123456789012 là account minh hoạ của bài viết; của bạn sẽ khác.\n'
  fi
else
  bad "không lấy được danh tính; thử: aws sso login --profile $profile"
  printf '%s\n' "$identity" >&2
  exit 1
fi

if regions="$(aws ec2 describe-regions --profile "$profile" \
  --query 'Regions[].RegionName' --output text 2>&1)"; then
  case " ${regions//$'\t'/ } " in
    *" $expected_region "*) ok "region $expected_region đang được bật" ;;
    *) bad "region $expected_region không nằm trong danh sách region đang bật" ;;
  esac
else
  bad "không liệt kê được region: $regions"
fi

if [ "$problems" -eq 0 ]; then
  echo "Thiết lập ổn."
else
  echo "Có $problems vấn đề cần xử lý." >&2
  exit 1
fi
