# P03 · Kết quả đo (PG-021 … PG-030)

> Tệp này do `npm run p03:measure` (`node pg/run-measure.mjs p03/do-dac.mjs`) GHI RA — đừng sửa tay; chạy lại lệnh để đo lại.
> Số dưới đây là đo THẬT trên máy ghi ở mục 1, một lần chạy lệnh đo; trên máy khác, con số tuyệt đối sẽ khác — so TỈ LỆ và XU HƯỚNG.

## 1. Máy và cấu hình

| Mục | Giá trị |
|---|---|
| Thời điểm đo | 2026-11-10 14:03 UTC |
| CPU | Intel(R) Core(TM) i7-10700 CPU @ 2.90GHz · 16 luồng logic |
| RAM | 64 GB |
| Hệ điều hành | Windows 11 Pro (win32 10.0.26300, x64) |
| Ổ đĩa chứa cluster tạm | C: KINGSTON SA2000M81000G | SSD | NVMe | 932 GB |
| PostgreSQL | 18.1 (C:\Program Files\PostgreSQL\18\bin) |
| Node.js | v22.18.0 |
| Cluster tạm | listen_addresses=localhost, max_connections=40, shared_buffers=128MB, work_mem=4MB, hash_mem_multiplier=2, maintenance_work_mem=64MB, effective_cache_size=4GB, random_page_cost=4, seq_page_cost=1, effective_io_concurrency=16, jit=off, max_parallel_workers_per_gather=0, max_parallel_maintenance_workers=0, default_statistics_target=100, autovacuum=off, track_io_timing=off, max_wal_size=4GB, checkpoint_timeout=30min, log_checkpoints=off, log_temp_files=-1, shared_preload_libraries=pg_stat_statements, timezone=Asia/Ho_Chi_Minh |
| Lặp | 1 lần khởi động (bỏ) + 7 lần đo (trừ khi phép đo ghi khác) |
| Tổng thời gian lệnh đo | 4,2 phút |
| Tải CPU toàn máy (trung bình 3 s) | trước khi đo 22 % · sau khi đo 7 % — cao nghĩa là có việc khác chạy song song, số đo nhiễu |

**Cách đo.** `explain` = `EXPLAIN (ANALYZE, TIMING OFF, BUFFERS, FORMAT JSON)`, lấy *Execution Time*
(thời gian thực thi phía server — không gồm lập kế hoạch, không gồm gửi kết quả về client; TIMING OFF để bớt
phụ phí đo từng nút). `client` = `\timing` của psql: từ lúc gửi tới lúc nhận xong, GỒM commit (fsync WAL) —
dùng cho câu ghi. "Buffers" = shared hit + read ở nút gốc. **Cache:** "nóng" = đã chạy một lần khởi động,
page nằm trong shared_buffers (hoặc, với bảng lớn hơn 1/4 shared_buffers quét tuần tự, trong page cache của
hệ điều hành); "lạnh SB" = trước MỖI lần đo đẩy page của bảng/index liên quan ra khỏi shared_buffers bằng
`pg_buffercache_evict_relation()` — page cache của hệ điều hành VẪN nóng (lab không xoá được cache OS), nên
"lạnh SB" chưa phải đọc đĩa thật. Mỗi nhóm chạy trên một database mới chép từ nền p03, một phiên, không tải nền.

## 2. Kết quả theo bài

### PG-021 / PG-023 · Lịch sử chuyến của khách — có/không index, cache nóng/lạnh

Khách cá nhân 0910000500 (40 chuyến) và khách doanh nghiệp B1 0919990001 (51.151 chuyến).

| Mã | Phép đo | Cách đo · cache | Trung vị (ms) | Min | Max | N | Buffers (trung vị) | Kế hoạch / ghi chú |
|---|---|---|---:|---:|---:|---:|---:|---|
| G1-a | Khách cá nhân, có index (customer_id, requested_at) | explain · nóng | 0,04 | 0,04 | 0,07 | 7 | 44 | Bitmap Heap Scan (trips) → Bitmap Index Scan (ix_trips_customer_requested) |
| G1-b | Như G1-a, lạnh SB (đẩy trips + index ra khỏi shared_buffers trước mỗi lần) | explain · lạnh SB | 0,32 | 0,30 | 0,32 | 7 | 44 | Bitmap Heap Scan (trips) → Bitmap Index Scan (ix_trips_customer_requested) |
| G1-c | Khách cá nhân, KHÔNG index (Seq Scan 19.441 page) | explain · nóng | 65,41 | 65,23 | 66,51 | 5 | 19.441 | Seq Scan (trips) |
| G1-d | 20 chuyến gần nhất của B1 (Index Scan + Limit) | explain · nóng | 0,02 | 0,02 | 0,03 | 7 | 10 | Limit → Index Scan (ix_trips_customer_requested) |
| G1-e | 20 chuyến gần nhất khách cá nhân (Bitmap + Sort) | explain · nóng | 0,06 | 0,04 | 0,08 | 7 | 44 | Limit → Sort → Bitmap Heap Scan (trips) → Bitmap Index Scan (ix_trips_customer_requested); quicksort 27kB Memory |

<details><summary>Từng lần đo (ms)</summary>

- G1-a: 0,06 · 0,07 · 0,04 · 0,04 · 0,04 · 0,04 · 0,04
- G1-b: 0,32 · 0,31 · 0,30 · 0,32 · 0,32 · 0,32 · 0,30
- G1-c: 66,51 · 65,23 · 65,41 · 65,76 · 65,23
- G1-d: 0,03 · 0,02 · 0,02 · 0,02 · 0,02 · 0,02 · 0,03
- G1-e: 0,06 · 0,08 · 0,04 · 0,05 · 0,07 · 0,07 · 0,06

</details>

### PG-023 · Độ chọn lọc: index hay Seq Scan (cột fare ngẫu nhiên, cột requested_at thẳng hàng)

Cùng câu SELECT * với hai mức dòng khớp; so kế hoạch planner chọn với kế hoạch bị ép (enable_* = off).

| Mã | Phép đo | Cách đo · cache | Trung vị (ms) | Min | Max | N | Buffers (trung vị) | Kế hoạch / ghi chú |
|---|---|---|---:|---:|---:|---:|---:|---|
| G2-a | fare 120k–150k (11 %) — planner chọn Bitmap | explain · nóng | 67,10 | 65,48 | 68,04 | 5 | 19.501 | Bitmap Heap Scan (trips) → Bitmap Index Scan (ix_trips_fare) |
| G2-b | fare 120k–150k (11 %) — ép Seq Scan | explain · nóng | 87,58 | 82,48 | 93,90 | 5 | 19.441 | Seq Scan (trips) |
| G2-c | fare 60k–150k (55 %) — planner chọn Seq Scan | explain · nóng | 95,52 | 93,21 | 103,68 | 5 | 19.441 | Seq Scan (trips) |
| G2-d | fare 60k–150k (55 %) — ép Bitmap | explain · nóng | 75,46 | 74,03 | 91,73 | 5 | 19.916 | Bitmap Heap Scan (trips) → Bitmap Index Scan (ix_trips_fare) |
| G2-e | requested_at 60 ngày (~36 %) — planner chọn Index Scan | explain · nóng | 37,37 | 36,09 | 38,96 | 5 | 8.200 | Index Scan (ix_trips_requested) |
| G2-f | requested_at 60 ngày (~36 %) — ép Seq Scan | explain · nóng | 97,80 | 96,71 | 99,98 | 5 | 19.441 | Seq Scan (trips) |

<details><summary>Từng lần đo (ms)</summary>

- G2-a: 67,30 · 67,10 · 65,48 · 66,14 · 68,04
- G2-b: 91,42 · 93,90 · 87,52 · 87,58 · 82,48
- G2-c: 95,52 · 93,21 · 94,08 · 95,64 · 103,68
- G2-d: 75,47 · 74,03 · 75,19 · 75,46 · 91,73
- G2-e: 37,37 · 38,96 · 36,09 · 37,62 · 37,37
- G2-f: 98,07 · 99,98 · 97,72 · 97,80 · 96,71

</details>

### PG-025 · Chi phí ghi: chép 50.000 chuyến vào bảng có 0 / 5 / 9 index

Mỗi lần đo: TRUNCATE + CHECKPOINT rồi `INSERT … SELECT` 50.000 dòng (đo phía client, gồm commit). Cùng định nghĩa index như 25-index-nhieu-cot-partial-expression.sql.

| Mã | Phép đo | Cách đo · cache | Trung vị (ms) | Min | Max | N | Buffers (trung vị) | Kế hoạch / ghi chú |
|---|---|---|---:|---:|---:|---:|---:|---|
| G3-a | 0 index | client · nóng | 79,46 | 60,33 | 91,46 | 5 | — | — |
| G3-b | 5 index (như trips của nền) | client · nóng | 602,60 | 568,30 | 620,84 | 5 | — | — |
| G3-c | 9 index (+ đảo cột, partial, status, expression) | client · nóng | 846,08 | 833,01 | 864,62 | 5 | — | — |

<details><summary>Từng lần đo (ms)</summary>

- G3-a: 82,28 · 79,46 · 60,33 · 78,95 · 91,46
- G3-b: 620,84 · 583,03 · 568,30 · 616,49 · 602,60
- G3-c: 853,39 · 846,08 · 833,01 · 839,88 · 864,62

</details>

### PG-026 · Index Only Scan, heap fetch và VACUUM

Giờ đặt các chuyến tháng 10 của B1 (9.365 dòng).

| Mã | Phép đo | Cách đo · cache | Trung vị (ms) | Min | Max | N | Buffers (trung vị) | Kế hoạch / ghi chú |
|---|---|---|---:|---:|---:|---:|---:|---|
| G4-a | Index Only Scan, bảng vừa VACUUM (Heap Fetches 0) | explain · nóng | 0,59 | 0,57 | 0,62 | 7 | 50 | Index Only Scan (ix_trips_customer_requested) |
| G4-b | Thêm cột fare (phải đọc heap) | explain · nóng | 4,46 | 3,90 | 5,55 | 7 | 3.384 | Bitmap Heap Scan (trips) → Bitmap Index Scan (ix_trips_customer_requested) |
| G4-c | Index Only Scan sau khi sửa 9.365 dòng, CHƯA VACUUM (Heap Fetches 18.536) | explain · nóng | 2,71 | 2,63 | 3,00 | 7 | 6.483 | Index Only Scan (ix_trips_customer_requested) |
| G4-d | Như G4-c, sau VACUUM | explain · nóng | 0,64 | 0,62 | 0,78 | 7 | 96 | Index Only Scan (ix_trips_customer_requested) |
| G4-e | Covering index INCLUDE (fare, status) | explain · nóng | 0,75 | 0,74 | 0,78 | 7 | 70 | Index Only Scan (ix_trips_khach_bao_gom) |

<details><summary>Từng lần đo (ms)</summary>

- G4-a: 0,60 · 0,59 · 0,58 · 0,57 · 0,57 · 0,62 · 0,60
- G4-b: 3,90 · 4,32 · 4,46 · 4,65 · 5,55 · 4,12 · 4,53
- G4-c: 3,00 · 2,71 · 2,66 · 2,63 · 2,93 · 2,89 · 2,64
- G4-d: 0,78 · 0,65 · 0,64 · 0,62 · 0,63 · 0,64 · 0,78
- G4-e: 0,78 · 0,76 · 0,74 · 0,74 · 0,74 · 0,75 · 0,75

</details>

### PG-028 · GIN trigram và BRIN (dữ liệu theo thứ tự vs xáo trộn)

Dữ liệu GPS sinh như 28-gin-gist-brin.sql (2.000.000 điểm). Câu BRIN: "5 phút gần nhất" (61.000 điểm). Các phép đo BRIN tắt Index Scan để loại khoá chính (driver_id, recorded_at).

| Mã | Phép đo | Cách đo · cache | Trung vị (ms) | Min | Max | N | Buffers (trung vị) | Kế hoạch / ghi chú |
|---|---|---|---:|---:|---:|---:|---:|---|
| G5-a | Tên quán ILIKE '%tấm bà năm%' — không index (Seq Scan 15.000 quán) | explain · nóng | 5,69 | 5,50 | 6,59 | 7 | 412 | Seq Scan (merchants) |
| G5-b | Như G5-a — GIN gin_trgm_ops | explain · nóng | 0,32 | 0,31 | 0,35 | 7 | 93 | Bitmap Heap Scan (merchants) → Bitmap Index Scan (ix_merchants_name_trgm) |
| G5-c | GPS 5 phút — BRIN, bảng THEO THỨ TỰ | explain · nóng | 5,59 | 5,51 | 6,56 | 7 | 541 | Aggregate → Bitmap Heap Scan (driver_locations) → Bitmap Index Scan (ix_dl_brin) |
| G5-d | GPS 5 phút — Seq Scan, bảng theo thứ tự | explain · nóng | 85,59 | 84,68 | 86,76 | 5 | 16.667 | Aggregate → Seq Scan (driver_locations) |
| G5-e | GPS 5 phút — B-tree trên recorded_at (Index Only Scan) | explain · nóng | 3,33 | 3,31 | 3,63 | 7 | 58 | Aggregate → Index Only Scan (ix_dl_btree) |
| G5-f | GPS 5 phút — bảng XÁO TRỘN, planner chọn (Seq Scan) | explain · nóng | 90,11 | 87,79 | 92,22 | 5 | 16.704 | Aggregate → Seq Scan (driver_locations_xao) |
| G5-g | GPS 5 phút — bảng XÁO TRỘN, ép BRIN | explain · nóng | 140,96 | 139,14 | 150,77 | 5 | 16.706 | Aggregate → Bitmap Heap Scan (driver_locations_xao) → Bitmap Index Scan (ix_dl_xao_brin) |

<details><summary>Từng lần đo (ms)</summary>

- G5-a: 6,59 · 5,69 · 6,22 · 5,58 · 5,63 · 5,73 · 5,50
- G5-b: 0,33 · 0,32 · 0,31 · 0,32 · 0,35 · 0,31 · 0,34
- G5-c: 5,59 · 5,54 · 5,51 · 6,56 · 5,57 · 5,71 · 5,59
- G5-d: 86,76 · 85,26 · 84,68 · 85,77 · 85,59
- G5-e: 3,36 · 3,63 · 3,33 · 3,33 · 3,32 · 3,47 · 3,31
- G5-f: 90,11 · 92,21 · 88,00 · 87,79 · 92,22
- G5-g: 150,77 · 140,18 · 140,96 · 139,14 · 141,31

</details>

### PG-029 · Sort và hash theo work_mem (một phiên)

| Mã | Phép đo | Cách đo · cache | Trung vị (ms) | Min | Max | N | Buffers (trung vị) | Kế hoạch / ghi chú |
|---|---|---|---:|---:|---:|---:|---:|---|
| G6-a | Sắp 1 triệu chuyến, work_mem 64kB (external merge) | explain · nóng | 597,49 | 591,51 | 646,17 | 5 | 19.441 | Sort → Seq Scan (trips); external merge 21608kB Disk |
| G6-b | Sắp 1 triệu chuyến, work_mem 4MB (external merge) | explain · nóng | 507,43 | 501,54 | 568,79 | 5 | 19.441 | Sort → Seq Scan (trips); external merge 21616kB Disk |
| G6-c | Sắp 1 triệu chuyến, work_mem 256MB (quicksort) | explain · nóng | 656,76 | 616,43 | 800,46 | 5 | 19.441 | Sort → Seq Scan (trips); quicksort 55827kB Memory |
| G6-d | Đếm chuyến theo khách (191.852 nhóm), work_mem 4MB | explain · nóng | 561,38 | 406,40 | 585,45 | 5 | 19.441 | Aggregate → Seq Scan (trips); HashAgg Batches 5 |
| G6-e | Như G6-d, work_mem 64MB | explain · nóng | 377,59 | 368,76 | 429,92 | 5 | 19.441 | Aggregate → Seq Scan (trips); HashAgg Batches 1 |
| G6-f | Doanh thu theo loại khách (Hash Join, bảng băm 200.003 khách), hash_mem_multiplier 2 | explain · nóng | 696,48 | 675,23 | 801,56 | 5 | 21.898 | Aggregate → Hash Join → Seq Scan (trips) → Hash → Seq Scan (customers); HashAgg Batches 1; Hash Batches 2 |
| G6-g | Như G6-f, hash_mem_multiplier 8 (một batch) | explain · nóng | 585,18 | 561,35 | 625,83 | 5 | 21.898 | Aggregate → Hash Join → Seq Scan (trips) → Hash → Seq Scan (customers); HashAgg Batches 1; Hash Batches 1 |

<details><summary>Từng lần đo (ms)</summary>

- G6-a: 597,49 · 591,51 · 612,02 · 646,17 · 595,22
- G6-b: 568,79 · 501,54 · 508,90 · 503,73 · 507,43
- G6-c: 616,43 · 717,09 · 800,46 · 656,76 · 628,27
- G6-d: 406,40 · 421,78 · 561,38 · 585,45 · 565,15
- G6-e: 429,92 · 368,76 · 377,59 · 381,43 · 376,09
- G6-f: 696,48 · 689,24 · 675,23 · 801,56 · 707,42
- G6-g: 561,35 · 579,65 · 585,18 · 590,53 · 625,83

</details>

### PG-030 · Tối ưu lịch sử chuyến hoàn thành của khách: trước / sau + chi phí ghi

Câu gốc và câu viết lại như 30-toi-uu-truy-van.sql. Mặc định ép kế hoạch RIÊNG cho từng tham số (plan_cache_mode = force_custom_plan) — để auto thì sau 5 lần EXECUTE PostgreSQL có thể đổi sang kế hoạch CHUNG giữa chừng và trộn hai kế hoạch vào cùng một phép đo; G8-h/G8-i đo riêng kế hoạch chung. "Ghi 5.000 chuyến" đo trong giao dịch rồi ROLLBACK + VACUUM sau mỗi lần (không gồm commit; VACUUM dọn dòng/mục index chết của lần trước để các lần đo như nhau).

| Mã | Phép đo | Cách đo · cache | Trung vị (ms) | Min | Max | N | Buffers (trung vị) | Kế hoạch / ghi chú |
|---|---|---|---:|---:|---:|---:|---:|---|
| G8-a | TRƯỚC — câu gốc, khách B1 | explain · nóng | 208,53 | 194,65 | 246,70 | 7 | 29.197 | Limit → Sort → Aggregate → Hash Join → Seq Scan (payments) → Hash → Bitmap Heap Scan (trips) → Bitmap Index Scan (ix_trips_customer_requested); top-N heapsort 29kB Memory; HashAgg Batches 1; Hash Batches 1 |
| G8-b | TRƯỚC — câu gốc, khách cá nhân | explain · nóng | 0,13 | 0,13 | 0,18 | 7 | 187 | Limit → Sort → Aggregate → Sort → Nested Loop → Index Scan (ix_trips_customer_requested) → Index Scan (ix_payments_trip); quicksort 29kB Memory; quicksort 28kB Memory |
| G8-c | Ghi 5.000 chuyến — KHÔNG có index mới | client · nóng | 693,62 | 678,12 | 723,05 | 5 | — | — |
| G8-d | SAU — câu viết lại + index partial mới, khách B1 | explain · nóng | 0,07 | 0,07 | 0,10 | 7 | 98 | Nested Loop → Limit → Index Scan (ix_trips_khach_hoan_thanh) → Aggregate → Index Scan (ix_payments_trip) |
| G8-e | SAU — câu viết lại, khách cá nhân | explain · nóng | 0,11 | 0,07 | 0,27 | 7 | 103 | Nested Loop → Limit → Index Scan (ix_trips_khach_hoan_thanh) → Aggregate → Index Scan (ix_payments_trip) |
| G8-f | Câu GỐC (không viết lại) khi đã có index mới, khách B1 | explain · nóng | 247,02 | 227,87 | 310,48 | 7 | 28.679 | Limit → Sort → Aggregate → Hash Join → Seq Scan (payments) → Hash → Bitmap Heap Scan (trips) → Bitmap Index Scan (ix_trips_khach_hoan_thanh); top-N heapsort 29kB Memory; HashAgg Batches 1; Hash Batches 1 |
| G8-h | Kế hoạch CHUNG (generic) của câu viết lại, khách B1 | explain · nóng | 0,07 | 0,07 | 0,08 | 5 | 98 | Nested Loop → Limit → Index Scan (ix_trips_khach_hoan_thanh) → Aggregate → Index Scan (ix_payments_trip) |
| G8-i | Kế hoạch CHUNG của câu gốc, khách B1 | explain · nóng | 419,62 | 398,31 | 463,71 | 5 | 211.646 | Limit → Sort → Aggregate → Sort → Nested Loop → Index Scan (ix_trips_khach_hoan_thanh) → Index Scan (ix_payments_trip); top-N heapsort 29kB Memory; external merge 3360kB Disk |
| G8-g | Ghi 5.000 chuyến — CÓ index mới | client · nóng | 699,59 | 694,91 | 829,95 | 5 | — | — |

<details><summary>Từng lần đo (ms)</summary>

- G8-a: 208,53 · 197,65 · 194,65 · 231,99 · 217,64 · 246,70 · 195,95
- G8-b: 0,18 · 0,15 · 0,13 · 0,13 · 0,13 · 0,13 · 0,13
- G8-c: 693,62 · 718,69 · 685,78 · 723,05 · 678,12
- G8-d: 0,10 · 0,07 · 0,10 · 0,07 · 0,07 · 0,07 · 0,07
- G8-e: 0,09 · 0,07 · 0,16 · 0,24 · 0,27 · 0,11 · 0,07
- G8-f: 310,48 · 266,55 · 295,28 · 247,02 · 235,93 · 232,63 · 227,87
- G8-h: 0,08 · 0,07 · 0,07 · 0,07 · 0,07
- G8-i: 429,85 · 413,21 · 419,62 · 463,71 · 398,31
- G8-g: 829,95 · 699,59 · 698,46 · 703,29 · 694,91

</details>

### PG-029 · Nhiều phiên đồng thời × work_mem (pgbench)

Mỗi giao dịch pgbench chạy một câu gom nhóm theo khách trên cả bảng (HashAggregate 191.852 nhóm — tràn đĩa ở work_mem 4MB, vừa bộ nhớ ở 64MB). work_mem đặt qua PGOPTIONS cho mọi phiên pgbench. Mỗi cấu hình 10 giây.

```sql
SELECT customer_id, count(*) AS n FROM toinoi.trips GROUP BY customer_id ORDER BY n DESC LIMIT 10;
```

| Cấu hình | Phiên đồng thời | tps | Độ trễ TB (ms) | Giao dịch | File tạm | Dữ liệu tạm (MB) | Đỉnh bộ nhớ riêng các backend (MB) |
|---|---:|---:|---:|---:|---:|---:|---:|
| work_mem 4MB | 1 | 2,3 | 433,5 | 23 | 23 | 345 | 15 |
| work_mem 4MB | 4 | 8,2 | 488,6 | 84 | 84 | 1.169 | 60 |
| work_mem 4MB | 8 | 11,3 | 707,8 | 116 | 116 | 1.598 | 119 |
| work_mem 64MB | 1 | 3,0 | 335,2 | 30 | 0 | 0 | 22 |
| work_mem 64MB | 4 | 9,1 | 440,9 | 92 | 0 | 0 | 88 |
| work_mem 64MB | 8 | 15,0 | 534,2 | 152 | 0 | 0 | 172 |

## 3. Giới hạn của các số đo

- Một máy, một lần chạy lệnh đo, dữ liệu tổng hợp (1 triệu chuyến) — số tuyệt đối KHÔNG mang sang máy khác
  hay production được; dùng để so sánh TƯƠNG ĐỐI giữa các phương án trên cùng máy, cùng lúc.
- Ổ đĩa và page cache của hệ điều hành ảnh hưởng mạnh tới các phép đo "lạnh"/Seq Scan; lab không xoá được cache OS.
- `explain` không tính thời gian lập kế hoạch và truyền kết quả; câu trả nhiều dòng sẽ chậm hơn thế khi chạy thật.
- Máy đo có thể có tiến trình khác chạy nền (trình duyệt, IDE, service PostgreSQL khác) — xem độ chênh min/max.
- pgbench: tải tổng hợp một loại câu; bộ nhớ đo bằng *private bytes* của các backend pgbench (Windows,
  lấy mẫu ~300 ms một lần) — đỉnh thật có thể cao hơn giữa hai lần lấy mẫu.
