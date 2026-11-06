#!/usr/bin/env bash
# Tải bản trích OSM Việt Nam từ Geofabrik (khoảng 330 MB). Dữ liệu © OpenStreetMap contributors, giấy phép ODbL.
# Sau đó chạy lab LOC-06 (loc06_osm_do_thi): lọc bbox nội thành TP.HCM và biên dịch ra data/hcm-graph.bin.
# (osmpbf chỉ đọc PBF, không ghi được, nên không có file hcm.pbf.)
set -euo pipefail
dir="$(cd "$(dirname "$0")" && pwd)"
out="$dir/vietnam-latest.osm.pbf"
if [[ -s "$out" ]]; then
  echo "đã có $out"
  exit 0
fi
curl -fL --retry 3 -o "$out" https://download.geofabrik.de/asia/vietnam-latest.osm.pbf
ls -la "$out"
