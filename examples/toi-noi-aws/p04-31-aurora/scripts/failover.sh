#!/usr/bin/env bash
# Chủ động failover cluster và đo thời gian writer đổi chỗ.
# Dùng: ./failover.sh [target-instance-id]
# Cảnh báo: connection đang mở tới writer bị ngắt. Chạy trong tài khoản lab.
source "$(dirname "$0")/common.sh"

writer() {
  # shellcheck disable=SC2016  # dấu backtick là literal của JMESPath
  aws rds describe-db-clusters --db-cluster-identifier "$CLUSTER" \
    --query 'DBClusters[0].DBClusterMembers[?IsClusterWriter==`true`].DBInstanceIdentifier | [0]' \
    --output text
}

before="$(writer)"
echo "Writer hiện tại: $before"

start="$(now)"
if [[ $# -ge 1 ]]; then
  aws rds failover-db-cluster --db-cluster-identifier "$CLUSTER" --target-db-instance-identifier "$1" >/dev/null
else
  aws rds failover-db-cluster --db-cluster-identifier "$CLUSTER" >/dev/null
fi

while true; do
  after="$(writer)"
  if [[ -n "$after" && "$after" != "None" && "$after" != "$before" ]]; then
    break
  fi
  sleep 1
done
echo "Writer mới: $after sau $(($(now) - start)) giây (đo bằng API; DNS và connection pool của app có thể thấy lâu hơn)"
