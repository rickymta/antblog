#!/usr/bin/env bash
# Xem consumer lag của một group theo từng partition.
# Dùng: ./consumer-lag.sh [group, mặc định dispatch-consumer]
source "$(dirname "$0")/common.sh"

group="${1:-dispatch-consumer}"
"$KAFKA_BIN/kafka-consumer-groups.sh" --bootstrap-server "$BOOTSTRAP" --command-config "$CLIENT_PROPS" \
  --describe --group "$group"
