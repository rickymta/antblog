#!/usr/bin/env bash
# Tải bản trích OSM Việt Nam từ Geofabrik (khoảng 330 MB). Dữ liệu © OpenStreetMap contributors, giấy phép ODbL.
# Sau đó chạy lab LOC-06 để cắt bbox nội thành TP.HCM ra data/hcm.pbf.
set -euo pipefail
dir="$(cd "$(dirname "$0")" && pwd)"
out="$dir/vietnam-latest.osm.pbf"
if [[ -s "$out" ]]; then
  echo "đã có $out"
  exit 0
fi
curl -fL --retry 3 -o "$out" https://download.geofabrik.de/asia/vietnam-latest.osm.pbf
ls -la "$out"
