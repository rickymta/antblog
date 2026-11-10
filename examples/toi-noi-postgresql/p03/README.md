# P03 · Lưu trữ, index và kế hoạch thực thi

Lab cho mười bài PG-021 … PG-030 trong [khung chương trình](../../../docs/postgresql-master-curriculum.md). Từ phần này lab chạy trên **PostgreSQL 18 THẬT** (đã kiểm **18.1**), không còn PGlite 17.5 như P01–P02: runner [`pg/run-pg.mjs`](../README.md#p03-postgresql-18-that) tự dựng một cluster tạm, chạy SQL bằng `psql`, chấm chỉ thị kiểm, rồi dừng và xoá cluster.

Trạng thái kiểm chứng: **chạy thật** trên Windows 11 Pro, Intel Core i7-10700 (16 luồng), 64 GB RAM, NVMe SSD, Node 22.18.0, PostgreSQL 18.1 (bản cài EDB `C:\Program Files\PostgreSQL\18`). `npm run p03`: **591 bước, 0 FAIL** (4 expect-error, 21 expect-rows, 35 expect-value, 21 expect-range, 84 expect-line, 3 expect-md5, 149 expect-match/no-match), **2 phút 12 giây** lần ghi OUTPUT (nền 62 s + 10 bài 2–24 s/bài; khi máy bận tới ~3 phút). Linux/macOS: **chưa chạy thử**.

## Chạy

```sh
cd examples/toi-noi-postgresql
npm ci                     # chỉ cần cho test P01–P02 (PGlite); runner P03 không dùng package nào
npm run p03                # nền + 10 tệp bài, in như p03/OUTPUT.txt, kết thúc "KẾT QUẢ: ĐẠT"
npm run p03:output         # ghi lại OUTPUT.txt sau khi sửa SQL
npm run p03:thu-nghiem     # thí nghiệm (p03/thu-nghiem/manifest.json)
npm run p03:measure        # PHÉP ĐO lặp (~4 phút, KHÔNG thuộc npm test) → ghi p03/DO-DAC.md
npm test                   # gồm test/p03.test.mjs: SKIP phần PostgreSQL nếu máy không có PostgreSQL 18

node pg/run-pg.mjs p03 --only 27                         # một bài (nền vẫn dựng)
node pg/run-pg.mjs --nen p03/lab.json:p03 thu.sql        # tệp của bạn trên bản sao nền p03
node pg/run-pg.mjs p03 --only 21 --keep                  # nền + bài 21, rồi GIỮ cluster chạy (database nen_p03) để tự mở psql
```

Prerequisites: Node ≥ 22; PostgreSQL ≥ 18 với extension contrib chuẩn (pageinspect, pg_buffercache, pg_visibility, pgstattuple, pg_walinspect, pg_trgm, btree_gist, unaccent, pg_stat_statements); ~2 GB trống ở thư mục tạm. Runner tìm binary qua `PG_BIN` › `pg_config --bindir` › `C:\Program Files\PostgreSQL\18\bin` — chi tiết, cấu hình cluster tạm và định dạng chỉ thị: [README gốc của lab](../README.md#p03-postgresql-18-that).

Mỗi tệp bài chạy trên một **database mới chép từ nền** (`CREATE DATABASE … TEMPLATE nen_p03 STRATEGY = FILE_COPY`): bài độc lập nhau, và mỗi bài bắt đầu với shared_buffers **không chứa page nào** của database đó (page cache của hệ điều hành thì vẫn nóng). Đọc tệp bài bằng psql tự mở: dùng `--keep` rồi `psql -h localhost -p <cổng> -U postgres -d nen_p03`, `SET search_path TO toinoi;` (nhớ: database nền đã bị bài nào chạy trên bản sao thì không đổi — muốn thử thoải mái hãy `CREATE DATABASE thu TEMPLATE nen_p03 STRATEGY FILE_COPY`). Vòng sửa–chạy nhanh: `node pg/run-pg.mjs --attach <thư-mục-toinoi-pg-…> --nen p03/lab.json:p03 p03/27-statistics.sql` (dùng lại nền, vài giây).

## Nền (`lab.json` → `nen.p03`)

`p01/00-reset.sql` → `p01/01-schema.sql` → `p02/00-mo-rong-schema.sql` (chỉ LƯỢC ĐỒ P01/P02, không fixture của chúng) → bốn tệp nền của P03:

| Tệp | Nội dung |
|---|---|
| `00-mo-rong-schema.sql` | Chặn chạy thiếu nền và chặn PostgreSQL < 18; thêm `customers.kind` (personal/business), `merchants.address` + `merchants.attrs jsonb`, bảng `driver_shifts` (ca `tstzrange` — CHƯA có exclusion constraint, bài 28 thêm), bảng `driver_locations` đúng hồ sơ (PK `(driver_id, recorded_at)`, dữ liệu sinh ở bài 28); hai hàm tiện ích: **`uoc_luong(sql)`** (số dòng ước lượng / thực tế / hệ số lệch của nút gốc — chạy EXPLAIN ANALYZE, chỉ nhận SELECT) và **`ke_hoach(sql)`** (các nút của EXPLAIN COSTS OFF trên một dòng, không chạy câu) |
| `01-fixture.sql` | Sinh dữ liệu TẤT ĐỊNH bằng md5 (hàm `p03_u(md5, i)` → số đều [0,1), `p03_uuid(nhãn, n)` → uuid dạng v4) — **giữ lại** cho bài dùng; nạp lô kiểu bỏ index phụ + khoá ngoại rồi tạo lại; cuối tệp `VACUUM (ANALYZE)` (cluster tắt autovacuum) |
| `02-kiem-tra-fixture.sql` | Hồ sơ dữ liệu H1–H10 với số kiểm (bảng dưới) |
| `03-bai-cu-20k.sql` | Schema RIÊNG `bai_cu`: lược đồ lõi của hồ sơ (ràng buộc không đặt tên ⇒ `trips_pkey`) + bộ sinh 20.000 chuyến của hồ sơ, thay `random()`/`gen_random_uuid()`/`now()` bằng bản tất định — để tái lập phép đo của hai bài cũ PG-022, PG-024. 20.000 < 30.000 dòng mẫu ⇒ thống kê ANALYZE của `bai_cu.trips` TẤT ĐỊNH |

### Dữ liệu — vì sao 1 triệu chuyến

Index chỉ có ý nghĩa khi bảng lớn hơn nhiều so với một vài page và lớn hơn shared_buffers: `trips` 1.000.000 dòng = **19.441 page = 152 MB** (> 128 MB shared_buffers), tra một mã chuyến đi cây 3 tầng; vẫn nạp được trong 60–90 s (tuỳ tải máy). Database nền **513 MB**. Mốc chụp **2026-10-31 01:00 +07** (như P02).

| Bảng | Dòng | Phân bố / điểm cài sẵn |
|---|---:|---|
| `cities` | 3 | SG, HN, DN |
| `areas` | 86 | 3 thành phố → 11 vùng → 72 khu (id khu = vùng×10+k: 111…158, 211…246, 311…324) |
| `customers` | 200.003 | 200.000 cá nhân `0910000001…0910200000` (1–110.000 SG, …–175.000 HN, …–200.000 DN) + 3 doanh nghiệp: **B1** `0919990001` (bưu kiện SG), B2 `0919990002` (cơm trưa HN), B3 `0919990003` (bưu kiện DN) |
| `drivers` | 6.000 | mã 20001–23600 SG, –25400 HN, –26000 DN; 70 % xe máy |
| `merchants` | 15.000 | 9.000 SG / 4.500 HN / 1.500 DN; tên = món + chủ quán (+ hậu tố) ⇒ trùng tên kiểu chuỗi; địa chỉ đường HƯ CẤU; `attrs` tags (8 % có "chay"), giao_nhanh (~30 %), gio_mo |
| `trips` | **1.000.000** | 01/05 → 30/10/2026 (cuối tuần +15 %, tăng dần ~35 % theo thời gian, 24 trọng số giờ của P02) + **300 chuyến đang chạy** 31/10 00:00–00:59 (50 mỗi trạng thái); SG 55 / HN 32 / DN 13 %; ride 40 / food 45 / parcel 15 %; completed 880.042, cancelled 119.658; mã 100000–1099999 tăng theo giờ đặt; heap chèn theo thời gian |
| `payments` | 901.208 | 1 lần thu thành công cho chuyến hoàn thành có cước; ví/thẻ 3 % thu lỗi trước; huỷ-sau-nhận trả thẻ 50 % thu rồi hoàn |
| `driver_shifts` | 158.175 | Tháng 10, ~85 % số ngày; **3 ca chồng giờ cài sẵn** (tài xế 20007, 23605, 25401, nhập lúc 2026-10-20 10:00) |
| `driver_locations` | 0 | Bài 28 sinh 2.000.000 điểm |
| `trip_events`, `trip_ratings` | 0 | P03 không dùng |
| `bai_cu.*` | 6 / 50.000 / 5.000 / 2.000 / 20.000 | Bộ dữ liệu của hai bài cũ (20.000 chuyến 30 ngày, 6 thành phố, 20 chuyến cuối 'searching', ~6 % huỷ) |

Phân bố cố ý LỆCH (02 · H4–H5): B1 **51.151** chuyến (5,1 % bảng), B2 12.435, B3 3.124; khách cá nhân lệch kiểu "số ít đi rất nhiều" (chỉ số khách = ⌊R·u²⌋): nhiều nhất 1.533 chuyến; số chuyến mỗi khách: 0 → 8.151 khách · 1–2 → 56.886 · 3–9 → 119.797 · 10–99 → 15.053 · 100–999 → 111 · ≥ 1000 → 5. Số khách khác nhau có chuyến: **191.852**. Khu đón **phụ thuộc hoàn toàn** vào thành phố (H6: 0 dòng sai); 10.210 chuyến (≈ 1 %) mất GPS. Heap gần như thẳng hàng với thời gian: chỉ **11** chuyến tháng 10 nằm ở page cũ (dòng huỷ-trước-nhận 116 byte được FSM đặt vào chỗ trống ở cuối page cũ — H8). Kích thước (tất định, H9): trips 159.260.672 B; pk_trips 39,9 MB; uq_trips_code 22,5 MB; ix_trips_city_requested 31,6 MB; ix_trips_customer_requested 40,6 MB; **ix_trips_driver_id 7,1 MB** (gộp khoá trùng); payments 88,5 MB.

## Tệp → bài

| Tệp | Bài (slug) | Mức | Minh hoạ |
|---|---|---|---|
| `21-explain-tim-chuyen.sql` | PG-021 `postgresql-explain-ke-hoach-thuc-thi` | CB | BUFFERS lạnh/nóng; EXPLAIN trần (cost/rows/width) vs COSTS OFF; SETTINGS; Seq/Index/Bitmap; NL/Hash/Merge join (+ cấm Merge Join); self-join bị bỏ (PG 18); ước lượng vs thực tế (city × khu); EXPLAIN ANALYZE CHẠY câu ghi; BEGIN … ROLLBACK; VERBOSE |
| `22-page-buffer.sql` | PG-022 `page-don-vi-doc-ghi` (bài cũ) | CB | Tái lập MỌI phép đo của bài cũ trên `bai_cu` (+ chỉ chỗ lệch); pageinspect; pg_buffercache + `pg_buffercache_evict_relation` (PG 18); WAL + full page image bằng pg_walinspect; cùng phép đo trên bảng 1 triệu chuyến |
| `23-btree.sql` | PG-023 `postgresql-btree-index-co-ban` | CB | Chưa/có index (19.441 → 43 page); giải phẫu cây (bt_metap, pgstatindex, bt_page_items, deduplication); thang độ chọn lọc cột ngẫu nhiên vs cột thẳng hàng; vì sao vẫn Seq Scan; uuid ngẫu nhiên làm cây xốp, REINDEX |
| `24-heap-ctid-hot.sql` | PG-024 `heap-ctid-va-primary-key` (bài cũ) | TB | Tái lập trên `bai_cu`: thứ tự vật lý vs theo PK, ctid đổi sau UPDATE, ba UPDATE ①②③ (không HOT / HOT / dời page), chuỗi phiên bản trong page, fillfactor 100 vs 90, PK = B-tree UNIQUE, FK tới UNIQUE, CLUSTER một lần + khoá + không duy trì |
| `25-index-nhieu-cot-partial-expression.sql` | PG-025 `postgresql-index-nhieu-cot-partial-expression` | TB | Thứ tự cột; **skip scan PG 18**; đảo cột (10 vs 41 page); partial index (16 KB vs 6,7 MB) + bẫy tham số generic plan; expression index + lỗi IMMUTABLE `42P17`; chi phí ghi 0/5/9 index bằng `EXPLAIN (ANALYZE, WAL)`; UPDATE không HOT ghi cả 9 index |
| `26-index-only-scan.sql` | PG-026 `postgresql-index-only-scan-visibility-map` | TB | pg_visibility; Heap Fetches 0 → 18.536 → 0 (UPDATE, VACUUM); INCLUDE (covering) và cái giá: index to hơn + mất HOT; count(*) bằng Index Only Scan trên index nhỏ nhất |
| `27-statistics.sql` | PG-027 `postgresql-statistics-analyze-cardinality` | TB | pg_stats (n_distinct, MCV, histogram); ước lượng B1 / khách 76 chuyến / khách 0 chuyến; `SET STATISTICS 1000`; `CREATE STATISTICS (dependencies, ndistinct, mcv)` cho city ↔ khu; thống kê CŨ làm kế hoạch sai (Nested Loop) → ANALYZE (Hash Join) |
| `28-gin-gist-brin.sql` | PG-028 `postgresql-gin-gist-brin-lua-chon` | NC | GIN pg_trgm (tên, địa chỉ, không dấu với unaccent, similarity), giới hạn mẫu < 3 ký tự; GIN jsonb (`jsonb_ops` vs `jsonb_path_ops`); GiST + btree_gist EXCLUDE ca không chồng giờ (`23P01`, khoảng nửa mở); BRIN trên 2 triệu điểm GPS theo thứ tự vs xáo trộn |
| `29-sort-hash-spill.sql` | PG-029 `postgresql-sort-hash-memory-spill` | NC | external merge vs quicksort vs top-N; `pg_stat_database.temp_files`; `log_temp_files` (nhật ký server); Hash Join Batches + `hash_mem_multiplier`; HashAggregate tràn đĩa; nhân bộ nhớ theo nút × phiên; `SET LOCAL work_mem`, `temp_file_limit` |
| `30-toi-uu-truy-van.sql` | PG-030 `postgresql-thuc-hanh-toi-uu-truy-van` | NC | Quy trình 8 bước trên "lịch sử chuyến hoàn thành + tiền đã thu": kế hoạch trước/sau, loại 2 giả thuyết sai bằng bằng chứng, md5 kết quả không đổi, số page 29.197 → 98, chi phí ghi (WAL, mất HOT), giới hạn kết luận |
| `thu-nghiem/` | PG-024, PG-026 | — | Hai kịch bản NHIỀU PHIÊN — xem [thu-nghiem/README.md](thu-nghiem/README.md) |
| `do-dac.mjs` → `DO-DAC.md` | PG-021, 023, 025, 026, 028, 029, 030 | — | Phép đo lặp (trung vị/min/max) + pgbench, kèm cấu hình máy |

## Số liệu then chốt từng bài

Mọi số dưới đây có trong `OUTPUT.txt` (trừ cột "thời gian" — ở `DO-DAC.md`). Số đánh dấu ≈ đến từ thống kê lấy mẫu: dao động nhẹ giữa các lần chạy.

**PG-021.** Khách `0910000500` (40 chuyến, 40 page heap khác nhau): lần 1 `Buffers: shared read=44`, lần 2 `shared hit=44`. Planner ước **≈11** chuyến (thực 40, giá trị ngoài MCV) ⇒ **Bitmap Heap Scan + Sort** (kế hoạch này dựa trên ước lượng lấy mẫu — khối biến động trong OUTPUT); cùng câu với khách B1 ⇒ **Limit → Index Scan** ix_trips_customer_requested (10 page) — cùng SQL, khác tham số, khác kế hoạch. Seq Scan `fare >= 150000`: 3.541 dòng, **Rows Removed by Filter 996.459**, 19.441 page; chạy lại vẫn `read≈19.300` (vòng đệm 256 kB của Seq Scan trên bảng > 1/4 shared_buffers). Bitmap: DN 7 ngày 5.477 dòng, `Heap Blocks: exact=827`. Joins: Nested Loop (37 dòng, `loops=40` ở pk_drivers, `actual rows=0.93` — PG 18 in số dòng trung bình có 2 chữ số thập phân), Hash Join (6.000 tài xế, `Buckets: 8192 Batches: 1 Memory Usage: 373kB`, bitmap trên ix_trips_city_requested với `Index Searches: 5` nhờ skip scan), **Merge Join** tự nhiên: `SELECT … FROM trips JOIN payments … ORDER BY t.id` (Index Only Scan pk_trips + Index Scan ix_payments_trip); cấm Merge Join ⇒ Hash Join + Sort. Self-join trên khoá unique bị **bỏ hẳn** (PG 18). Ước lượng city=3 ∧ khu=311: **≈2.964 vs 22.222** (lệch ≈7,5×). EXPLAIN ANALYZE DELETE xoá thật 68/100 dòng (còn 32); UPDATE trong BEGIN … ROLLBACK giữ giá 31.000. `SETTINGS` in `jit = 'off', max_parallel_workers_per_gather = '0', search_path = 'toinoi'`. VERBOSE kèm `Query Identifier` (do nạp pg_stat_statements).

**PG-022** — đối chiếu bài cũ (bai_cu, 20.000 chuyến):

| Bài cũ nói | Lab đo | Kết luận |
|---|---|---|
| `block_size` 8.192 | 8192 | khớp |
| trips 374 page · 3.063.808 B · 53,5 dòng/page | 374 · 3.063.808 · 53,5 | **khớp tuyệt đối** |
| ix_trips_city_requested 83 page / 679.936 B / 241,0; uq_trips_code 57 / 466.944 / 350,9 | 83 / 679.936 / 241,0; 57 / 466.944 / 350,9 | khớp |
| trips_pkey 101 page / 827.392 B / 198,0 | **102** / 835.584 / 196,1 | lệch 1 page (uuid ngẫu nhiên khác ⇒ kiểu tách page khác) |
| page 0: lower 240, upper 280, trống 40 (54 dòng) | lower **236**, upper **328**, trống **92** (53 dòng) | lệch — phụ thuộc độ dài từng dòng của dữ liệu cụ thể |
| 1 chuyến ≈ 147,3 B (`avg(pg_column_size)`) | 147,3 | khớp |
| Buffers: ctid 1 · code= 3 · code= (chỉ fare) 3 · 53 dòng liền 4 · 53 rải rác 159 | 1 · 3 · 3 · 4 · **159** (`Index Searches: 53`) | khớp |
| `count(*)` đọc 374 page | **Index Only Scan trên ix_trips_driver_id, 51 page** (bảng vừa VACUUM); `count(fare)` mới ra Seq Scan 374 page | **bài cũ cần sửa câu**: count(*) không nhất thiết đọc heap |
| UPDATE 1 sau CHECKPOINT 7.160 B WAL (FPI 7.024), UPDATE 2 cùng page 112 B | 6.607 B (HOT_UPDATE 6.573 gồm FPI **6.508** + COMMIT 34), 114 B (80 + 34) | hiện tượng khớp, số lệch theo khoảng trống của page; hiệu LSN thô ≈ 6.640 / 120 (có đệm căn lề + bản ghi Standby — khối biến động) |
| UPDATE dời page + 5 index: 42.424 B | **47.691 B**, 8 bản ghi (5 Btree INSERT_LEAF + Heap **LOCK** + Heap UPDATE + COMMIT), 7 có ảnh page | hiện tượng khớp; có thêm bản ghi Heap LOCK bài cũ không nhắc |
| (khác) bài cũ ghi "PostgreSQL 18.6" | lab chạy 18.1 | người viết kiểm lại số phiên bản |
| (bẫy) câu `pg_class WHERE relname IN (…)` không lọc schema | trả 7 dòng khi có 2 schema cùng tên bảng | thêm `relnamespace` |

Trên bảng 1 triệu chuyến: 19.441 page, 51,4 dòng/page, ~149 B/chuyến (có thêm pickup_area_id, CHECK không tốn chỗ); uq_trips_code **level 2** (3 tầng, bai_cu level 1) ⇒ tra một mã = **4 page** (lạnh `hit=1 read=3`, nóng `hit=4`); 53 mã liền = 5 page; 53 mã rải rác = 212 page.

**PG-023.** Chưa index: Seq Scan 40 dòng, **19.441 page**, bỏ 999.960 dòng; có `ix_trips_customer` ⇒ **43 page** (3 index + 40 heap). ix_trips_customer = **12,7 MB**, level 2, 1.611 lá + 10 nhánh, độ đầy lá 90,3 %, gốc 9 mục; lá có mục gộp 11 ctid. Thang fare (cột không thẳng hàng): 0,35 % · 3,9 % · 11,1 % · **26,6 % vẫn Bitmap** · **55,5 % Seq Scan**; ở 11 % Bitmap đã chạm 19.403/19.441 page heap. Thang requested_at (correlation 1): **Index Scan tới 120 ngày (~72 %)**, 180 ngày mới Seq Scan; tháng 10 = 184.396 dòng qua Index Scan chạm ~4.200 page. pk_trips 4.848 lá, đầy **71,2 %** vs uq_trips_code 2.733 lá, 90,0 %; REINDEX pk_trips ⇒ 3.832 lá, 90,0 %.

**PG-024** — đối chiếu bài cũ:

| Bài cũ | Lab | Kết luận |
|---|---|---|
| `LIMIT 5` không ORDER BY: (0,1)…(0,5), mã 100000… | (0,1)…(0,5), 100000–100004 | khớp |
| `ORDER BY id`: ctid nhảy (336,33), (148,17)… | (177,53), (255,10), (180,17), (218,40), (138,41) — 5 page khác nhau | hình dạng khớp (uuid khác) |
| `EXPLAIN … code = 150000`: Index Scan uq_trips_code `cost=0.29..8.30 rows=1 width=125` | **y hệt** `cost=0.29..8.30 rows=1 width=125` | khớp |
| Bitmap city=1 ∧ 1 ngày: `cost=5.43..247.45 rows=111` | `cost=5.43..247.17 rows=111` | khớp (cost lệch ở phần lẻ) |
| correlation: requested_at 1,000 · code 1,000 · city_id 0,167 · fare 0,014 · driver_id 0,007 · id −0,005 · customer_id 0,000 | 1,000 · 1,000 · **0,167** · **0,014** · −0,001 · 0,009 · −0,007 | khớp tới 3 chữ số với city_id/fare; cột uuid ≈ 0 |
| ① nhận tài xế (373,14)→(373,34) không HOT; ② sửa giá →(373,35) HOT; ③ chuyến cũ page 186 →(373,36) không HOT | (373,32)→(373,47) không HOT; →(373,48) HOT; (186,54)→(373,49) không HOT; `n_tup_upd 3, n_tup_hot_upd 1` | hình dạng khớp (số ô khác) |
| HOT % | 33,3 % (1/3) | — |
| 5 index 3,4 MB > bảng 3,0 MB | index **3,3 MB**, bảng **2,9 MB** (2,92 MiB) | bài cũ làm tròn lên; ý vẫn đúng |
| CLUSTER khoá ACCESS EXCLUSIVE, chép bảng, chỉ một lần | `pg_locks`: AccessExclusiveLock (+ ShareLock phụ); correlation sau: id **1,000**, code/requested_at 0,009; `indisclustered = t`; 500 chuyến mới vào page 373–380 (cuối heap); correlation id tụt ≈0,95 | khớp; thí nghiệm 3 phiên `thu-nghiem/pg024-…` chứng minh SELECT cũng bị chặn sau CLUSTER đang chờ |

Thêm: chuỗi phiên bản ở page 373 (ô 32 → 47 → 48, cờ HOT_UPDATED trên ô 47); fillfactor 100 vs 90 (bảng 384 vs 448 page): sửa giá 2.000 chuyến cũ ⇒ HOT **1,1 %** vs **100 %**; PK = `btree, indisunique t, indisprimary t` vs UNIQUE `indisprimary f`; FK tới cột UNIQUE ⇒ `23503`.

**PG-025.** (city, ngày) DN 15/10 = 768 dòng: index (city_id, requested_at) **10 page lạnh** (`Index Searches: 1`); index đảo (requested_at, city_id) **41 page**. Chỉ điều kiện ngày trên (city_id, requested_at): **skip scan**, `Index Searches: 5`, 5.896 dòng. Partial "đang chạy" **16.384 B** vs index status đầy đủ **7.069.696 B**; generic plan (`force_generic_plan`) KHÔNG dùng partial index, custom plan có. Expression: `(requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = '2026-10-15'` → Seq Scan 5.896 dòng / ~19.400 page; `requested_at::date` làm index ⇒ **42P17**; có index biểu thức ⇒ Index Scan 129 page. WAL khi chép 50.000 chuyến: **0 index 50.001 bản ghi / 9,03 MB · 5 index 302.054 / 29,83 MB · 9 index 453.320 / 40,53 MB**; UPDATE 10.000 dòng có 9 index: 101.995 bản ghi, 15,9 MB WAL, **4 HOT / 10.000**.

**PG-026.** 19.441/19.441 page all-visible. B1 tháng 10 = 9.365 dòng: Index Only Scan, `Heap Fetches: 0`, 50 page; thêm `fare` ⇒ Bitmap Heap Scan 3.384 page. Sau UPDATE 9.365 dòng: all-visible còn ≈16.100, bảng 19.615 page, Index Only Scan **`Heap Fetches: 18536`**, 18.540 page; VACUUM ⇒ 19.615/19.615, `Heap Fetches: 0`, 96 page. INCLUDE (fare, status): Index Only Scan 70 page; index 58,9 MB vs 41,0 MB; bảng fillfactor 90: 500 UPDATE trước INCLUDE đều HOT, 500 sau: 0 HOT.

**PG-027.** customer_id: n_distinct ước **≈83.000** (thật **191.852**), 53 MCV (tổng tần suất ≈0,08), B1 đứng đầu MCV (≈0,051). Ước lượng: B1 ≈50.500 vs 51.151 (lệch ≈1,0); khách 76 chuyến (ngoài MCV) ≈11 (lệch ≈7); khách 0 chuyến ≈11. `SET STATISTICS 1000`: 1.000 MCV, khách 76 chuyến lệch ≈1,1. city ∧ khu: **2.964 vs 22.222** → `CREATE STATISTICS` → **≈22.300** (lệch 1,0); GROUP BY 2 cột: 219 nhóm ước (lệch 2,9) → 75 (1,0); `dependencies {"14 => 4": ≈0,99}` (attnum 14 = pickup_area_id, 4 = city_id; < 1 vì 1 % khu NULL). Histogram requested_at 101 mốc, 7 ngày lệch ≈1,0. Thống kê cũ: 30.000 chuyến ngày 01/11 nạp sau ANALYZE ⇒ ước **1** (lệch 30.000×), kế hoạch **Nested Loop** 30.000 vòng; ANALYZE ⇒ lệch ≈1,1–1,3, **Hash Join**.

**PG-028.** `show_trgm('Cơm Tấm')` 8 trigram. `'%tấm bà năm%'`: 75 quán, Seq Scan bỏ 14.925 dòng → GIN `Bitmap Index Scan`, 71 page heap; `'%gà%'` (< 3 ký tự) vẫn Seq Scan; không dấu `'%com tam ba nam%'` = 0 dòng → index `khong_dau(name)` (unaccent bọc IMMUTABLE) ⇒ 75; similarity `'com tam ba nan'` = **0,81**; địa chỉ `'%đường hoa sứ%quận 3,%'` 84 dòng (Recheck bỏ 2). Kích thước: merchants 3.375.104 B; GIN địa chỉ 1.425.408; tên 655.360; tên không dấu 630.784. jsonb: 1.204 quán "chay"; `jsonb_path_ops` phục vụ `@>` nhưng KHÔNG phục vụ `?`; `jsonb_ops` phục vụ `?` và to hơn. EXCLUDE: thêm ràng buộc ⇒ **23P01** (3 cặp cài sẵn) → xoá 3 dòng → thêm được (index GiST cùng tên); ca chồng ⇒ 23P01; ca liền sau `[kt, kt+2h)` hợp lệ; "đang trong ca lúc 15/10 09:00" = 1.713 (Index Only Scan trên index GiST). BRIN: driver_locations 2.000.000 dòng = **136.536.064 B**, B-tree recorded_at **14.655.488 B** (gộp khoá trùng), **BRIN 24.576 B**; mục BRIN theo thứ tự: dải 0 `{22:13:25 .. 22:14:40}`, dải 128 `{22:14:40 .. 22:15:55}`…; xáo trộn: mọi dải `{22:13:25 .. 01:00:00}`. 5 phút (61.000 dòng): theo thứ tự BRIN `Heap Blocks: lossy=539`, Recheck bỏ 3.640; xáo trộn ⇒ planner Seq Scan, ép BRIN ⇒ `lossy=16704` (cả bảng), Recheck bỏ **1.939.000**; B-tree: Index Only Scan `Heap Fetches: 61000` (bảng chưa VACUUM).

**PG-029.** Sắp 1 triệu (code, fare): work_mem 4MB `external merge Disk: 21616kB`; 64kB `Disk: 21608kB`; 256MB `quicksort Memory: 55827kB`; LIMIT 20 `top-N heapsort Memory: 26kB`. `log_temp_files = 0` ⇒ nhật ký `temporary file: … size 22134784`; `pg_stat_database`: +1 file, > 10 MB. Hash Join (bảng băm 200.003 khách): `Buckets: 131072 Batches: 2 Memory Usage: 6589kB`; `hash_mem_multiplier = 8` ⇒ `Batches: 1 … 13181kB`. HashAggregate 191.852 nhóm: `Batches: 5 Memory Usage: 8761kB Disk Usage: 15360kB`; work_mem 64MB ⇒ `Batches: 1 Memory Usage: 16409kB`. Trần minh hoạ 40 phiên × (8 + 8 + 4 MB) = 800 MB.

**PG-030.** Câu gốc, B1: Bitmap 51.151 → lọc 44.922 hoàn thành → **Seq Scan cả 901.208 payments** + Hash Right Join → HashAggregate → top-N; **29.197 page** cho 20 dòng. Giả thuyết "sắp theo requested_at": trang đầu trùng (0 khác) nhưng **35.026 / 44.922** chuyến đổi vị trí ⇒ loại. Sau `ix_trips_khach_hoan_thanh (customer_id, completed_at DESC, code DESC) WHERE status = 'completed'` + LATERAL: Limit → Index Scan (18 page) + 20 × ix_payments_trip (80 page) = **98 page**; md5 kết quả trước = sau = `c8046204e6fa07cc57d4196a9cf7cc0e`. Câu GỐC có index mới nhưng không viết lại: chỉ đổi bitmap sang index mới, vẫn quét cả payments (~28.700 page) — index một mình không đủ. **Kế hoạch CHUNG** (prepared statement sau 5 lần EXECUTE, ép bằng `plan_cache_mode = force_generic_plan`): câu viết lại thành Bitmap + sắp 44.922 chuyến (**≈17.956 page**), câu gốc thành Nested Loop tra payments 44.922 lần (**≈211.646 page**) — tệp bài đặt `force_custom_plan` ở đầu để số đo không trộn hai loại kế hoạch. Cái giá: index **43,7 MB**; nạp 5.000 chuyến ngay sau CHECKPOINT: **30.262 bản ghi / 48,95 MB WAL (6.824 ảnh page)** không index mới vs **38.677 / 72,78 MB (9.912)** có index; hoàn thành 200 chuyến trên bảng còn chỗ: 200/200 HOT trước, 0/200 sau khi có index.

## Kết quả đo (`DO-DAC.md`)

Đo thật trên máy ở trên bằng `npm run p03:measure` (1 khởi động + 7 lần, trung vị; câu nặng 5 lần; ~4 phút), lần ghi DO-DAC.md: 2026-11-10 14:03 UTC, tải CPU toàn máy 22 % trước / 7 % sau (máy có trình duyệt, IDE và tác tử khác chạy nền — xem min/max). Lệnh đo đã chạy đầy đủ 3 lần trong ngày (+1 lần thử một phần); xu hướng giữ nguyên, số tuyệt đối dao động ~±20 % (lần thứ hai máy bận: pgbench thấp hơn ~30 %, có cấu hình đảo thứ tự). Bảng đầy đủ, từng lần đo, cách đo, giới hạn: [`DO-DAC.md`](DO-DAC.md). Trung vị (ms, cache nóng, phía server trừ khi ghi "client"):

| Bài | Phép đo | Trung vị | Ghi chú |
|---|---|---:|---|
| PG-021/023 | Lịch sử khách cá nhân: có index · lạnh SB · KHÔNG index | 0,04 · 0,32 · **65,4** | 44 vs 19.441 page; "lạnh SB" vẫn có cache OS |
| PG-021 | 20 chuyến gần nhất B1 (Index Scan + Limit) | 0,02 | 10 page |
| PG-023 | fare 11 %: Bitmap (planner) · ép Seq | 67,1 · 87,6 | planner chọn đúng |
| PG-023 | fare 55 %: Seq (planner) · ép Bitmap | 95,5 · **75,5** | planner chọn Seq nhưng Bitmap đo NHANH hơn trên máy này (cache OS nóng; cost model giả định đọc ngẫu nhiên đắt gấp 4) — tư liệu cho "random_page_cost trên SSD" |
| PG-023 | requested_at 60 ngày (36 %): Index Scan (planner) · ép Seq | 37,4 · 97,8 | cột thẳng hàng: Index Scan nhanh gấp 2,6 lần |
| PG-025 | Chép 50.000 chuyến (client, gồm commit): 0 · 5 · 9 index | 79 · 603 · 846 | 5 index ≈ ×7,6 thời gian ghi |
| PG-026 | Index Only Scan (0 heap fetch) · thêm cột fare · IOS khi 18.536 heap fetch · sau VACUUM · INCLUDE | 0,59 · 4,46 · 2,71 · 0,64 · 0,75 | |
| PG-028 | Tên quán ILIKE: Seq · GIN trigram | 5,69 · 0,32 | |
| PG-028 | GPS 5 phút: BRIN (thứ tự) · Seq (thứ tự) · B-tree · Seq (xáo) · ép BRIN (xáo) | 5,59 · 85,6 · 3,33 · 90,1 · **141,0** | BRIN 24 KB ≈ B-tree 14 MB về tốc độ; BRIN trên dữ liệu xáo tệ hơn Seq Scan |
| PG-029 | Sắp 1 triệu: 64kB · 4MB · 256MB | 597 · **507** · 657 | quicksort trong RAM CHẬM hơn external merge (cả 3 lần đo đầy đủ) |
| PG-029 | HashAgg 191.852 nhóm: 4MB · 64MB | 561 · 378 | min 4MB 406 — nhiễu |
| PG-029 | Hash Join bảng băm 200k: hash_mem_multiplier 2 · 8 | 696 · 585 | |
| PG-030 | Câu gốc B1 · câu viết lại B1 · câu gốc + index mới (không viết lại) | **208,5** · **0,07** · 247,0 | ~3.000× nhanh hơn khi viết lại + index; index một mình không giúp |
| PG-030 | Kế hoạch CHUNG: câu viết lại · câu gốc (B1) | 0,07 · 419,6 | lần đo này kế hoạch chung của câu viết lại vẫn là Index Scan (98 page); lần đo trước là Bitmap + Sort (~40 ms, 17.956 page) — bấp bênh theo ANALYZE |
| PG-030 | Ghi 5.000 chuyến trong giao dịch: không · có index mới | 694 · 700 | khác biệt < nhiễu; bằng chứng chi phí ghi nên dùng WAL tất định (OUTPUT: 48,95 → 72,78 MB) |

pgbench (câu gom nhóm theo khách trên cả bảng, 10 s mỗi cấu hình):

| work_mem | Phiên | tps | Độ trễ TB (ms) | File tạm | Dữ liệu tạm (MB) | Đỉnh bộ nhớ riêng các backend (MB) |
|---|---:|---:|---:|---:|---:|---:|
| 4MB | 1 · 4 · 8 | 2,3 · 8,2 · 11,3 | 434 · 489 · 708 | 23 · 84 · 116 | 345 · 1.169 · 1.598 | 15 · 60 · 119 |
| 64MB | 1 · 4 · 8 | 3,0 · 9,1 · 15,0 | 335 · 441 · 534 | 0 · 0 · 0 | 0 | 22 · 88 · 172 |

Đọc: work_mem 64MB nhanh hơn ~30 % và không ghi file tạm (4MB ghi ~1,6 GB file tạm trong 10 giây ở 8 phiên), đổi lại bộ nhớ riêng tăng tuyến tính theo số phiên (~22 MB/phiên — một nút HashAggregate 16 MB + phần nền), 8 phiên ≈ 172 MB. Đó là phép nhân "work_mem × nút × phiên" đo thật.

## Không chạy được / chưa chạy

- **Linux/macOS: chưa chạy thử** (runner đã tính `LC_CTYPE=C.UTF-8` và bỏ đo bộ nhớ ngoài Windows — "chưa đo").
- **Cache lạnh thật (đọc đĩa)**: lab không xoá được page cache của hệ điều hành (cần quyền quản trị / khởi động lại máy); "lạnh" trong lab = lạnh với shared_buffers (`pg_buffercache_evict_relation`). `shared read` vì vậy thường là đọc từ RAM của OS — bài viết phải nói rõ.
- **Song song (Gather), JIT**: tắt trong cluster lab để tái lập; bài nào muốn minh hoạ thì `SET max_parallel_workers_per_gather = 2` / `SET jit = on` trong phiên (chưa có tệp nào làm).
- **autovacuum**: tắt; hành vi tự động (ngưỡng analyze/vacuum) chỉ được mô tả, không chạy thật ở P03.
- `track_io_timing` (I/O Timings), `auto_explain`: chưa dùng.
- Docker / psql "cách B" như P01–P02: không áp dụng cho P03 (dùng PostgreSQL cài sẵn).

## Dọn

Runner tự dừng và xoá cluster (`%TEMP%\toinoi-pg-*`). Sau `--keep` hoặc khi tiến trình bị giết: `pg_ctl stop -D <thư-mục>\data -m fast` rồi xoá thư mục. Kiểm: `tasklist /FI "IMAGENAME eq postgres.exe"` chỉ còn tiến trình của service (Session Name `Services`). Lab không để lại gì khác.

## Ghi chú cho người viết bài

- **PostgreSQL 18 khác bản 17 ở những chỗ hiện trong OUTPUT**: EXPLAIN ANALYZE **tự kèm BUFFERS**; `actual rows=40.00` (hai chữ số thập phân — trung bình mỗi vòng lặp, vd. `0.93`); dòng mới **`Index Searches: N`**; **skip scan** B-tree nhiều cột; **self-join elimination**; `pg_buffercache_evict_relation()`; `WAL: … buffers full=N` trong `EXPLAIN (WAL)`; initdb bật **data checksums** mặc định; `log_checkpoints` mặc định on. Đừng trích lẫn kế hoạch của bản 17.
- **Ước lượng dao động**: mọi `rows=` trong ngoặc cost, `n_distinct`, tần suất MCV đến từ mẫu 30.000 dòng NGẪU NHIÊN — số trong bài sẽ không khớp từng chữ số với máy người đọc; trích kèm chữ "khoảng" hoặc trích hệ số lệch. Số thực tế (actual rows, Buffers, kích thước, WAL bytes, md5) thì tất định.
- **"shared read" ≠ đọc đĩa** (page cache OS). **Seq Scan bảng lớn** dùng vòng đệm 256 kB ⇒ chạy lại vẫn `read` gần đủ 19.441 page, không "ấm" lên như bảng nhỏ.
- **count(\*) của bài PG-022 cũ**: sau VACUUM là Index Only Scan trên index nhỏ nhất (51 page) — muốn minh hoạ "đọc hết heap" thì dùng `count(fare)` hoặc chưa VACUUM.
- **Số trong bài cũ PG-022/PG-024 là của một bộ dữ liệu ngẫu nhiên khác** — lab tái lập HÌNH DẠNG và đa số con số (374 page, 53,5 dòng/page, 147,3 B, 1/3/3/4/159 page, cost Index Scan, correlation city_id/fare); các số phụ thuộc dữ liệu cụ thể (lower/upper page 0, byte WAL, ô ctid) thì khác — xem hai bảng đối chiếu.
- **Ngưỡng chuyển Seq Scan không phải con số cố định**: cột ngẫu nhiên đổi giữa 26,6 % và 55,5 %; cột thẳng hàng tới >72 % vẫn Index Scan. Đừng viết "trên 10–20 % thì Seq Scan".
- **Correlation của cột ít giá trị bị thổi phồng** (city_id 3 giá trị ≈0,42; status ≈0,79 dù phân bố ngẫu nhiên theo thời gian) do cách xếp mẫu có giá trị trùng.
- **VACUUM bỏ qua dọn index khi dòng chết ít** (dưới ~2 % số page, PG 14+, "index scan bypassed") ⇒ page còn ô LP_DEAD ⇒ Index Only Scan vẫn `Heap Fetches` sau VACUUM; `VACUUM (INDEX_CLEANUP ON)` mới về 0 (thí nghiệm pg026).
- **HOT**: UPDATE gán LẠI ĐÚNG giá trị cũ cho cột có index vẫn được HOT; cột chỉ nằm trong mệnh đề WHERE của partial index hoặc trong INCLUDE cũng làm mất HOT khi đổi.
- **pg_trgm và LC_CTYPE**: trên database `LC_CTYPE=C`, chữ có dấu không phải "chữ cái" với pg_trgm — bài nên nhắc người đọc kiểm `show_trgm` trên database của họ. `unaccent` chỉ STABLE: index phải qua hàm IMMUTABLE tự bọc (và REINDEX khi đổi từ điển).
- **WAL ngay sau CHECKPOINT** lớn bất thường (full page image) — các số WAL trong lab đều đo ngay sau `CHECKPOINT` cho so sánh được; giữa hai checkpoint, cùng thao tác ghi ít WAL hơn nhiều.
- **Thời gian**: OUTPUT.txt che mọi thời gian bằng `T`; chỉ trích thời gian từ `DO-DAC.md`, kèm cấu hình máy và giới hạn (một máy, cache OS nóng, dữ liệu tổng hợp).
- **Kế hoạch phụ thuộc ƯỚC LƯỢNG có thể đổi giữa các lần ANALYZE.** Kế hoạch của khách cá nhân `0910000500` (ước ~11 chuyến ⇒ Bitmap + Sort ở PG-021, Nested Loop ở PG-030) và **kế hoạch CHUNG** của prepared statement (PG-030 §6b) là chỗ đã thấy/được dự phòng là có thể lật; các kết quả đó nằm trong khối "biến động" của OUTPUT và chỉ được kiểm phần chắc chắn. Khi dựng lab đã gặp thật: cùng câu viết lại, kế hoạch chung lúc là Index Scan + Limit (98 page), lúc là Bitmap + Sort (~18.000 page). Đây là tư liệu tốt cho PG-030 (giới hạn kết luận) — nhưng đừng trích như thể luôn xảy ra.
- **Prepared statement chuyển sang kế hoạch chung sau 5 lần EXECUTE** (`plan_cache_mode = auto`): lần đo đầu tiên của G8 (trước khi ép `force_custom_plan`) đã trộn hai kế hoạch vào cùng một phép đo (trung vị 285 ms, max 743 ms). Đo câu PREPARE thì luôn ghi rõ plan_cache_mode.
- **work_mem lớn không luôn nhanh hơn**: sắp 1 triệu dòng với 256MB (quicksort trong RAM) đo được CHẬM hơn 4MB (external merge) ở cả ba lần đo đầy đủ trên máy lab (xem DO-DAC G6) — giả thuyết (CHƯA kiểm): file tạm ~21 MB nằm trong page cache OS nên rẻ, còn quicksort trên mảng 55 MB kém thân thiện cache CPU. Đừng viết "tràn đĩa luôn chậm"; nếu bài dùng, nên đo thêm ở máy khác.
- **Index một mình không sửa được câu gốc** ở PG-030: planner chỉ dùng index mới thay cho bitmap để lấy 44.922 chuyến, vẫn quét cả payments (~28.700 page) và đo được KHÔNG nhanh hơn (247 vs 208 ms). Phải viết lại (LIMIT trước, LATERAL sau).
- Bộ dữ liệu: `id` là uuid dạng v4 suy từ md5 (không phải `gen_random_uuid()`); mã khách/tài xế/quán tra theo số điện thoại/mã, đừng ghi cứng uuid ngoài các chỗ OUTPUT đã in.
