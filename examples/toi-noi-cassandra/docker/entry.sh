#!/bin/sh
# Bọc entrypoint của ảnh cassandra: đặt cổng CQL (native_transport_port) theo NATIVE_PORT để
# cổng trong container trùng cổng mở ra máy host. Kết hợp CASSANDRA_BROADCAST_RPC_ADDRESS=127.0.0.1,
# driver chạy trên máy host (DBStudio, ứng dụng) nhận địa chỉ node là 127.0.0.1:<cổng> và kết nối được,
# thay vì IP nội bộ của mạng Docker.
set -eu
yaml=/etc/cassandra/cassandra.yaml
sed -i "s/^native_transport_port:.*/native_transport_port: ${NATIVE_PORT:-9042}/" "$yaml"
exec docker-entrypoint.sh cassandra -f
