#!/usr/bin/env bash
# Playbook "access key bị lộ": khoanh vùng bằng CloudTrail, rồi (tuỳ chọn) vô hiệu hoá key.
# Mặc định CHỈ ĐỌC và IN lệnh cô lập. Thêm --apply mới chạy `update-access-key --status Inactive`.
# Dùng: ./access-key-triage.sh <access-key-id> <iam-user-name> [profile] [--apply]
set -euo pipefail

key_id="${1:?Thiếu access key id (không dán key thật vào tệp hay lịch sử shell chia sẻ)}"
user="${2:?Thiếu tên IAM user sở hữu key}"
profile="${3:-toinoi-lab}"
apply="${4:-}"
export AWS_PAGER=""

echo "== 1. Lần dùng cuối của key (dịch vụ, region, thời điểm)"
aws iam get-access-key-last-used --access-key-id "$key_id" --profile "$profile" \
  --query 'AccessKeyLastUsed' --output json

echo
echo "== 2. Sự kiện CloudTrail gắn với key này (90 ngày gần nhất, region hiện tại)"
aws cloudtrail lookup-events --profile "$profile" \
  --lookup-attributes "AttributeKey=AccessKeyId,AttributeValue=$key_id" \
  --max-results 50 --output json |
  jq -r '.Events[] | .CloudTrailEvent | fromjson |
    [.eventTime, .eventName, .sourceIPAddress, .awsRegion, (.errorCode // "OK")] | @tsv'
echo "Cột: thời gian, API, IP nguồn, region, kết quả. Tìm IP lạ, region lạ và các API tạo user, role, key hoặc mở tài nguyên."

echo
cmd=(aws iam update-access-key --user-name "$user" --access-key-id "$key_id" --status Inactive --profile "$profile")
if [ "$apply" = "--apply" ]; then
  echo "== 3. Cô lập: vô hiệu hoá key"
  "${cmd[@]}"
  echo "Key đã Inactive. Giữ lại (chưa xoá) để điều tra; xoá sau khi kết thúc sự cố."
else
  echo "== 3. Cô lập (chưa chạy; thêm --apply để thực hiện):"
  printf '   '
  printf '%q ' "${cmd[@]}"
  echo
fi

cat <<'NOTE'

Bước tiếp theo theo playbook: xoay mọi secret key đó từng đọc được, tìm tài nguyên lạ do key tạo,
gỡ quyền thừa (bài 51), rà GuardDuty finding cùng khoảng thời gian, ghi bài học vào postmortem.
NOTE
