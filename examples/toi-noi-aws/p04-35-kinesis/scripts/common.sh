#!/usr/bin/env bash
# Biến dùng chung.
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
export AWS_DEFAULT_REGION="$AWS_REGION"
export AWS_PAGER=""
STREAM="${STREAM:-toinoi-lab-driver-location}"
