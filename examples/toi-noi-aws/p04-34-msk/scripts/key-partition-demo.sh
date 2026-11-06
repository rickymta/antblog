#!/usr/bin/env bash
# Minh hoạ vì sao khoá partition quyết định thứ tự và độ cân bằng: đếm số partition thực sự được dùng
# khi khoá là tripId (nhiều giá trị) so với cityId (6 giá trị). Chạy cục bộ, không cần Kafka.
# Phép băm ở đây là cksum, chỉ mang tính minh hoạ; Kafka dùng murmur2.
set -euo pipefail

partitions=12

used() { # đọc các khoá từ stdin, in số partition khác nhau
  local k h
  while read -r k; do
    h="$(printf '%s' "$k" | cksum | cut -d' ' -f1)"
    echo $((h % partitions))
  done | sort -un | wc -l | tr -d ' '
}

echo "Khoá = tripId (1000 chuyến): $(for i in $(seq 1 1000); do echo "t$i"; done | used) / $partitions partition được dùng"
echo "Khoá = cityId (6 thành phố): $(printf '%s\n' hcm hn dn ct hp nt | used) / $partitions partition được dùng"
