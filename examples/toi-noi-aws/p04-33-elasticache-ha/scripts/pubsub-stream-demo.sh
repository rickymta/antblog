#!/usr/bin/env bash
# So pub/sub (không bền, người nhận offline là mất) với Streams (bền, có consumer group).
# Dùng: ENDPOINT=<endpoint> ./pubsub-stream-demo.sh   (cluster mode: đặt REDIS_CLUSTER=1; không cluster: REDIS_CLUSTER=0)
source "$(dirname "$0")/common.sh"

channel='tracking:{trip:42}'
stream='dispatch:{trip:42}:events'

echo "== Pub/sub: PUBLISH khi chưa có ai SUBSCRIBE thì số người nhận là 0 và thông điệp biến mất"
cli PUBLISH "$channel" '{"lat":10.7769,"lng":106.7009}'

echo "== Streams: XADD giữ thông điệp lại, giới hạn độ dài xấp xỉ bằng MAXLEN ~"
cli XADD "$stream" MAXLEN '~' 10000 '*' event DriverAssigned driver d001 >/dev/null
cli XADD "$stream" MAXLEN '~' 10000 '*' event DriverArrived driver d001 >/dev/null

# Tạo consumer group; lỗi BUSYGROUP khi chạy lại là bình thường.
cli XGROUP CREATE "$stream" notification '0' MKSTREAM 2>/dev/null || true

echo "== Consumer 'worker-1' đọc từ group 'notification'"
cli XREADGROUP GROUP notification worker-1 COUNT 10 STREAMS "$stream" '>'

echo "== Số thông điệp đang chờ xác nhận (XACK) của group"
cli XPENDING "$stream" notification
