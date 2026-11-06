#!/usr/bin/env bash
# Ép một shard failover bằng API của ElastiCache và theo dõi tới khi replication group available trở lại.
# Dùng: ./test-failover.sh [node-group-id, mặc định 0001]
# API giới hạn số lần gọi mỗi 24 giờ cho mỗi node group (xem User Guide). Chạy trong tài khoản lab.
source "$(dirname "$0")/common.sh"

shard="${1:-0001}"
start="$(date +%s)"

aws elasticache test-failover --replication-group-id "$RG" --node-group-id "$shard" >/dev/null

while true; do
  status="$(aws elasticache describe-replication-groups --replication-group-id "$RG" \
    --query 'ReplicationGroups[0].Status' --output text)"
  [[ "$status" == "available" ]] && break
  sleep 2
done
echo "Replication group $RG available sau $(($(date +%s) - start)) giây (đây là trạng thái API, không phải độ trễ client thấy)."
# shellcheck disable=SC2016  # dấu backtick là literal của JMESPath
aws elasticache describe-replication-groups --replication-group-id "$RG" \
  --query 'ReplicationGroups[0].NodeGroups[].[NodeGroupId,NodeGroupMembers[?CurrentRole==`primary`].CacheClusterId|[0]]' \
  --output text
