#!/usr/bin/env bash
# Đọc kết quả quét basic của một image: số finding theo mức độ.
set -euo pipefail

REGION="${AWS_REGION:-ap-southeast-1}"
REPO="toinoi/booking"
TAG="${1:?Cách dùng: check-scan.sh <tag>}"

aws ecr describe-image-scan-findings --region "${REGION}" \
  --repository-name "${REPO}" --image-id imageTag="${TAG}" \
  --query 'imageScanFindings.findingSeverityCounts' --output json
