#!/usr/bin/env bash
# Tạo các topic của Tới Nơi: partition theo tripId, RF=3, min.insync.replicas=2, retention 7 ngày.
# Chạy trên máy trong VPC có quyền kafka-cluster:CreateTopic (không cấp quyền này cho service thường).
source "$(dirname "$0")/common.sh"

create() { # tên partitions retention_ms
  "$KAFKA_BIN/kafka-topics.sh" --bootstrap-server "$BOOTSTRAP" --command-config "$CLIENT_PROPS" \
    --create --if-not-exists --topic "$1" --partitions "$2" --replication-factor 3 \
    --config min.insync.replicas=2 --config "retention.ms=$3"
}

create booking.events.v1 12 604800000     # key = tripId; 7 ngày
create dispatch.events.v1 12 604800000    # key = tripId
# Vị trí GPS đi qua Kinesis Data Streams ở bài 35, không tạo topic ở đây.
"$KAFKA_BIN/kafka-topics.sh" --bootstrap-server "$BOOTSTRAP" --command-config "$CLIENT_PROPS" --list
