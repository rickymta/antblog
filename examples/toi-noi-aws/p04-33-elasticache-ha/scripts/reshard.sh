#!/usr/bin/env bash
# Resharding online: đổi số shard của replication group.
# Dùng: ./reshard.sh <số node group mới>
# Với Terraform, đổi num_node_groups cũng thực hiện việc này. Slot được chia lại giữa các shard,
# client cluster-aware tự cập nhật bản đồ slot nhờ MOVED/ASK.
source "$(dirname "$0")/common.sh"

count="${1:?số node group mới}"
aws elasticache modify-replication-group-shard-configuration \
  --replication-group-id "$RG" \
  --node-group-count "$count" \
  --apply-immediately >/dev/null
echo "Đã gửi yêu cầu resharding tới $count shard. Theo dõi: aws elasticache describe-replication-groups --replication-group-id $RG --query 'ReplicationGroups[0].Status'"
