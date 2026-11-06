#!/usr/bin/env bash
# Minh hoạ envelope encryption bằng AWS CLI và openssl:
#   1. GenerateDataKey: KMS trả data key (plaintext) và bản mã hoá của nó.
#   2. Mã hoá hoá đơn cục bộ bằng data key, rồi bỏ plaintext key.
#   3. Lưu data key đã mã hoá cạnh dữ liệu; giải mã lại bằng kms:Decrypt với đúng encryption context.
#   4. Giải mã với encryption context sai phải THẤT BẠI.
# CHỈ để học cơ chế. Production dùng AWS Encryption SDK (AES-GCM, có xác thực), không tự ghép openssl enc.
# Cần quyền kms:GenerateDataKey và kms:Decrypt trên key. Mỗi lần chạy gọi KMS 3 lần (tính tiền theo request).
# Dùng: ./envelope-demo.sh [alias] [profile]
set -euo pipefail

alias_name="${1:-alias/toinoi-lab-trips}"
profile="${2:-toinoi-lab}"
export AWS_PAGER=""
context="Service=booking,Purpose=receipt"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

b64_to_hex() { openssl base64 -d -A | od -An -tx1 | tr -d ' \n'; }

echo "== 1. Xin data key từ $alias_name"
resp="$(aws kms generate-data-key --key-id "$alias_name" --key-spec AES_256 \
  --encryption-context "$context" --output json --profile "$profile")"
plain_b64="$(printf '%s' "$resp" | jq -r '.Plaintext')"
printf '%s' "$resp" | jq -r '.CiphertextBlob' | openssl base64 -d -A >"$work/receipt.key.enc"
key_hex="$(printf '%s' "$plain_b64" | b64_to_hex)"
unset resp plain_b64

echo "== 2. Mã hoá hoá đơn cục bộ bằng data key (AES-256-CBC, chỉ minh hoạ)"
iv_hex="$(openssl rand -hex 16)"
printf 'Hoa don chuyen #1001: 84.000 VND, tai xe T-204\n' >"$work/receipt.txt"
openssl enc -aes-256-cbc -K "$key_hex" -iv "$iv_hex" -in "$work/receipt.txt" -out "$work/receipt.enc"
printf '%s' "$iv_hex" >"$work/receipt.iv"
unset key_hex
rm -f "$work/receipt.txt"
echo "   Tệp lưu cạnh nhau: receipt.enc (dữ liệu), receipt.key.enc (data key đã mã hoá), receipt.iv"

echo "== 3. Giải mã: nhờ KMS mở data key, rồi giải mã cục bộ"
key_hex="$(aws kms decrypt --ciphertext-blob "fileb://$work/receipt.key.enc" \
  --encryption-context "$context" --query Plaintext --output text --profile "$profile" | b64_to_hex)"
openssl enc -d -aes-256-cbc -K "$key_hex" -iv "$(cat "$work/receipt.iv")" -in "$work/receipt.enc"
unset key_hex

echo "== 4. Giải mã với encryption context sai (phải bị từ chối)"
if aws kms decrypt --ciphertext-blob "fileb://$work/receipt.key.enc" \
  --encryption-context "Service=tracking,Purpose=receipt" \
  --query Plaintext --output text --profile "$profile" >/dev/null 2>&1; then
  echo "SAI: KMS giải mã được với context sai." >&2
  exit 1
fi
echo "   Đúng như mong đợi: KMS từ chối vì encryption context không khớp."
