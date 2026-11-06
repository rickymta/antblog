#!/usr/bin/env bash
# Đọc thử vài bản ghi từ đầu shard đầu tiên (TRIM_HORIZON) và giải mã nội dung.
# Chỉ để kiểm tra bằng tay: ứng dụng thật dùng KCL hoặc Lambda event source mapping.
source "$(dirname "$0")/common.sh"

shard="$(aws kinesis list-shards --stream-name "$STREAM" --query 'Shards[0].ShardId' --output text)"
iterator="$(aws kinesis get-shard-iterator --stream-name "$STREAM" --shard-id "$shard" \
  --shard-iterator-type TRIM_HORIZON --query ShardIterator --output text)"

aws kinesis get-records --shard-iterator "$iterator" --limit 5 --output json |
  jq -r '.Records[] | [.PartitionKey, (.Data | @base64d)] | @tsv'
