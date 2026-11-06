#!/usr/bin/env bash
# Biến dùng chung. Sửa cho khớp tài khoản lab của bạn hoặc xuất biến môi trường trước khi chạy.
set -euo pipefail

export AWS_REGION="${AWS_REGION:-ap-southeast-1}"
export AWS_DEFAULT_REGION="$AWS_REGION"
export AWS_PAGER=""
CLUSTER="${CLUSTER:-toinoi-lab-trips}"
SUBNET_GROUP="${SUBNET_GROUP:-toinoi-lab-trips}"
# Security group của Aurora (lấy từ: aws rds describe-db-clusters ... VpcSecurityGroups)
DB_SG="${DB_SG:-sg-0123456789abcdef0}"

now() { date +%s; }
