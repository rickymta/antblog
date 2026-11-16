#!/bin/sh
# P01 · 08-partition-size (CS-009): cùng một ngày dữ liệu, ghi vào hai bảng khác cách chia partition.
#   trips_by_city_day   PRIMARY KEY ((city_id, day), …)          → 1 partition cho cả ngày
#   trips_by_city_hour  PRIMARY KEY ((city_id, hour_bucket), …)  → 24 partition, mỗi giờ một partition
# Dữ liệu: 24.000 chuyến hư cấu của "thành phố thử nghiệm" số 9, ngày 10/11/2026, dồn vào giờ cao điểm.
# Chạy trong container node1: cqlsh COPY FROM để nạp nhanh, nodetool để đọc kích thước partition.
set -eu
CQL="cqlsh 127.0.0.1 ${NATIVE_PORT:-19042}"
N=24000
WORK=/tmp/cs009
rm -rf "$WORK" && mkdir -p "$WORK"

echo "-- ## Bảng một partition mỗi ngày (chỉ dùng cho thí nghiệm)"
cat > "$WORK/setup.cql" <<CQL
CREATE TABLE IF NOT EXISTS toinoi.trips_by_city_day (
  city_id int, day date, requested_at timestamp, trip_id uuid,
  service text, status text, fare_vnd bigint,
  PRIMARY KEY ((city_id, day), requested_at, trip_id)
) WITH comment = 'Thí nghiệm CS-009: một partition cho cả ngày';
TRUNCATE toinoi.trips_by_city_day;
DELETE FROM toinoi.trips_by_city_hour WHERE city_id = 9 AND hour_bucket IN ($(i=0; s=''; while [ $i -lt 24 ]; do h=$(printf %02d $i); s="$s'2026-11-10 $h:00:00+0700',"; i=$((i+1)); done; echo "${s%,}"));
CQL
$CQL -f "$WORK/setup.cql"

echo "-- ## Sinh $N chuyến: 50% rơi vào 11:00–13:00 và 17:00–19:00"
awk -v n="$N" 'BEGIN {
  srand(20261110);
  split("ride food parcel", svc, " ");
  for (i = 1; i <= n; i++) {
    if (rand() < 0.5) { peak[1]=11; peak[2]=12; peak[3]=17; peak[4]=18; h = peak[int(rand()*4)+1] }
    else { h = int(rand()*24) }
    m = int(rand()*60); s = int(rand()*60);
    # cqlsh COPY FROM đọc sai độ lệch múi giờ trong chuỗi (đảo dấu), nên CSV ghi sẵn giờ UTC = giờ VN - 7.
    uh = h - 7; ud = "2026-11-10"; if (uh < 0) { uh += 24; ud = "2026-11-09" }
    ts = sprintf("%s %02d:%02d:%02d+0000", ud, uh, m, s);
    hb = sprintf("%s %02d:00:00+0000", ud, uh);
    id = sprintf("0e000000-0000-4000-8000-%012d", i);
    fare = 20000 + int(rand()*180) * 1000;
    sv = svc[int(rand()*3)+1];
    printf "9,2026-11-10,%s,%s,%s,completed,%d\n", ts, id, sv, fare > "/tmp/cs009/day.csv";
    printf "9,%s,%s,%s,%s,completed,%d\n", hb, ts, id, sv, fare > "/tmp/cs009/hour.csv";
    cnt[h]++;
  }
  printf "gio  so_chuyen\n";
  for (h = 0; h < 24; h++) printf "%02d   %d\n", h, cnt[h];
}'

echo "-- ## Bẫy của COPY FROM (cqlsh 5.0.9): độ lệch '+0700' trong CSV bị đảo dấu, giờ lệch 14 tiếng"
$CQL -e "CREATE TABLE IF NOT EXISTS toinoi.copy_tz_lab (id int PRIMARY KEY, at_ts timestamp);"
printf "1,2026-11-01 14:00:00+0700
2,2026-11-01 07:00:00+0000
" > "$WORK/tz.csv"
$CQL -e "COPY toinoi.copy_tz_lab (id, at_ts) FROM '$WORK/tz.csv' WITH HEADER = false;" > /dev/null 2>&1
$CQL -e "SELECT id, at_ts AS doc_ra FROM toinoi.copy_tz_lab WHERE id IN (1, 2);" | grep -vE '^$|rows'
echo "(dòng 1 ghi 14:00:00+0700, dòng 2 ghi 07:00:00+0000: cùng một thời điểm, nhưng dòng 1 lệch 14 giờ)"
$CQL -e "DROP TABLE toinoi.copy_tz_lab;"

echo "-- ## Nạp bằng COPY FROM (CSV ghi giờ UTC)"
$CQL -e "COPY toinoi.trips_by_city_day (city_id, day, requested_at, trip_id, service, status, fare_vnd) FROM '$WORK/day.csv' WITH HEADER = false;" 2>&1 | grep -E "^[0-9]+ rows imported" || true
$CQL -e "COPY toinoi.trips_by_city_hour (city_id, hour_bucket, requested_at, trip_id, service, status, fare_vnd) FROM '$WORK/hour.csv' WITH HEADER = false;" 2>&1 | grep -E "^[0-9]+ rows imported" || true

echo "-- ## Đếm dòng trong partition lớn nhất của mỗi bảng"
$CQL -e "SELECT COUNT(*) AS dong_partition_ngay FROM toinoi.trips_by_city_day WHERE city_id = 9 AND day = '2026-11-10';
SELECT COUNT(*) AS dong_partition_12h FROM toinoi.trips_by_city_hour WHERE city_id = 9 AND hour_bucket = '2026-11-10 12:00:00+0700';
SELECT COUNT(*) AS dong_partition_03h FROM toinoi.trips_by_city_hour WHERE city_id = 9 AND hour_bucket = '2026-11-10 03:00:00+0700';" | grep -vE '^$|^\(1 rows\)|Warnings|Aggregation'

echo "-- ## nodetool flush rồi đọc kích thước partition (byte, ước lượng theo bucket histogram) từ SSTable"
nodetool flush toinoi trips_by_city_day trips_by_city_hour
for t in trips_by_city_day trips_by_city_hour; do
  echo "[$t]"
  nodetool tablestats "toinoi.$t" | grep -E "Number of partitions \(estimate\)|Compacted partition (minimum|maximum|mean) bytes" | sed 's/^[[:space:]]*//'
done

echo "-- ## Phân bố kích thước partition (nodetool tablehistograms, cột Partition Size)"
for t in trips_by_city_day trips_by_city_hour; do
  echo "[$t]"
  nodetool tablehistograms toinoi "$t" | awk 'NR<=3 || /^(50%|95%|Max)/'
done
