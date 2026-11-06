#!/usr/bin/env bash
# Kiểm ASL bằng chính API của Step Functions (cần AWS CLI v2 và credential, không tạo tài nguyên).
# Placeholder ${...} được thay bằng giá trị giả để đúng định dạng ARN và URL.
set -euo pipefail

cd "$(dirname "$0")/.."
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

for f in asl/*.asl.json; do
  out="$TMP/$(basename "$f")"
  sed -E \
    -e 's#\$\{[a-z_]*function_arn\}#arn:aws:lambda:ap-southeast-1:123456789012:function:stub#g' \
    -e 's#\$\{[a-z_]*queue_url\}#https://sqs.ap-southeast-1.amazonaws.com/123456789012/stub#g' \
    -e 's#\$\{dispatch_table_name\}#toinoi-lab-dispatch#g' \
    -e 's#\$\{event_bus_name\}#toinoi-lab-trips#g' \
    -e 's#\$\{recon_bucket_name\}#toinoi-lab-recon-123456789012#g' \
    "$f" >"$out"
  echo "== $f"
  aws stepfunctions validate-state-machine-definition \
    --type STANDARD --definition "file://$out" \
    --query '{result: result, diagnostics: diagnostics}' --output json
done
