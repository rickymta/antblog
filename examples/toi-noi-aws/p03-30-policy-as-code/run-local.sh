#!/usr/bin/env bash
# Chạy cổng kiểm IaC trên máy, chỉ bằng Docker (không cần cài Terraform hay Checkov).
# Không tạo tài nguyên cloud. Dùng: ./run-local.sh [fmt|validate|checkov|tflint|cfn-guard|all]
set -euo pipefail
cd "$(dirname "$0")"

TF_IMAGE="hashicorp/terraform:1.10"
CHECKOV_IMAGE="bridgecrew/checkov:3.4.3" # ghim phiên bản để kết quả lặp lại được; nâng trong PR riêng
TFLINT_IMAGE="ghcr.io/terraform-linters/tflint:v0.65.0"

checkov() {
  docker run --rm -v "$PWD:/w" -w /w "$CHECKOV_IMAGE" "$@"
}

step_fmt() {
  echo "== terraform fmt -check"
  docker run --rm -v "$PWD:/w" -w /w "$TF_IMAGE" fmt -check -recursive
}

step_validate() {
  for dir in samples/terraform/bad samples/terraform/good drift; do
    echo "== terraform validate $dir"
    docker run --rm -v "$PWD/$dir:/w" -w /w "$TF_IMAGE" init -backend=false -input=false > /dev/null
    docker run --rm -v "$PWD/$dir:/w" -w /w "$TF_IMAGE" validate
    rm -rf "$dir/.terraform" "$dir/.terraform.lock.hcl"
  done
}

step_checkov() {
  # Bản SAI phải bị bắt (exit code khác 0), bản ĐÚNG phải qua (exit code 0).
  echo "== checkov: bản sai, mong đợi có lỗi"
  if checkov -d samples/terraform/bad --external-checks-dir policies/checkov --compact --quiet; then
    echo "LỖI: bản sai không bị Checkov bắt" >&2
    return 1
  fi
  echo "== checkov: bản đúng, mong đợi sạch (skip có lý do nằm cạnh tài nguyên)"
  checkov --config-file .checkov.yaml -d samples/terraform/good
  echo "== checkov: template CloudFormation sai, mong đợi có lỗi"
  if checkov -f samples/cloudformation/bad.yaml --framework cloudformation --compact --quiet; then
    echo "LỖI: template sai không bị Checkov bắt" >&2
    return 1
  fi
}

step_tflint() {
  # --init tải plugin AWS từ GitHub nên cần mạng; có thể bị giới hạn tốc độ nếu không có token.
  echo "== tflint"
  docker run --rm -v "$PWD:/data" -w /data --entrypoint /bin/sh "$TFLINT_IMAGE" \
    -c "tflint --init && tflint --recursive"
}

step_cfn_guard() {
  local guard="${CFN_GUARD:-cfn-guard}"
  if ! command -v "$guard" > /dev/null; then
    echo "== cfn-guard chưa cài trên máy, bỏ qua (xem README để cài)"
    return 0
  fi
  echo "== cfn-guard test"
  "$guard" test --rules-file policies/cfn-guard/toinoi.guard --test-data policies/cfn-guard/toinoi.tests.yaml
  echo "== cfn-guard validate: bản sai, mong đợi FAIL"
  if "$guard" validate --data samples/cloudformation/bad.yaml --rules policies/cfn-guard/toinoi.guard --show-summary fail; then
    echo "LỖI: bad.yaml không bị cfn-guard bắt" >&2
    return 1
  fi
  echo "== cfn-guard validate: bản đúng, mong đợi PASS"
  "$guard" validate --data samples/cloudformation/good.yaml --rules policies/cfn-guard/toinoi.guard --show-summary all
}

case "${1:-all}" in
  fmt) step_fmt ;;
  validate) step_validate ;;
  checkov) step_checkov ;;
  tflint) step_tflint ;;
  cfn-guard) step_cfn_guard ;;
  all) step_fmt; step_validate; step_checkov; step_tflint; step_cfn_guard ;;
  *) echo "Dùng: $0 [fmt|validate|checkov|tflint|cfn-guard|all]" >&2; exit 2 ;;
esac
