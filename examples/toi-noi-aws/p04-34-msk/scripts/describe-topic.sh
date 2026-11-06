#!/usr/bin/env bash
# Xem leader, replica và ISR của từng partition: ISR ít hơn replica nghĩa là đang thiếu đồng bộ.
# Dùng: ./describe-topic.sh [topic, mặc định booking.events.v1]
source "$(dirname "$0")/common.sh"

topic="${1:-booking.events.v1}"
"$KAFKA_BIN/kafka-topics.sh" --bootstrap-server "$BOOTSTRAP" --command-config "$CLIENT_PROPS" \
  --describe --topic "$topic"
