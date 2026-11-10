-- P03 · 22-page-buffer.sql — PG-022 «Đơn vị đọc ghi nhỏ nhất của cơ sở dữ liệu là gì?»
-- (`page-don-vi-doc-ghi`, BÀI ĐÃ CÓ — chờ rà soát).
--
-- Mục đích: TÁI LẬP từng phép đo của bài cũ trên dữ liệu tương đương (schema
-- bai_cu: lược đồ lõi của hồ sơ + 20.000 chuyến, dựng ở p03/03-bai-cu-20k.sql),
-- để người viết đối chiếu số trong bài; rồi đo lại cùng các ý trên bảng 1 triệu
-- chuyến của P03 (toinoi.trips) để thấy cái gì đổi theo quy mô.
-- Bảng đối chiếu "bài cũ nói — lab đo" nằm ở p03/README.md (mục PG-022).
--
-- Khái niệm: page 8 KB · buffer (một page trong shared_buffers) · pageinspect ·
-- pg_buffercache · EXPLAIN BUFFERS shared hit/read · WAL và full page image.

-- Extension đặt vào schema public (CREATE EXTENSION mặc định dùng schema ĐẦU TIÊN
-- của search_path — ở đây sẽ là bai_cu, rồi mất tác dụng khi đổi search_path).
CREATE EXTENSION pageinspect WITH SCHEMA public;
CREATE EXTENSION pg_buffercache WITH SCHEMA public;
CREATE EXTENSION pg_walinspect WITH SCHEMA public;
SET search_path TO bai_cu, toinoi, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- K1 · Kích thước page: cố định lúc biên dịch.
-- expect-value: 8192
SHOW block_size;

-- K2 · Bảng và index của bai_cu.trips: số page, byte, số dòng trung bình mỗi page.
--     (Bài cũ: trips 374 page / 3.063.808 byte / 53,5 dòng mỗi page; trips_pkey 101;
--     uq_trips_code 57; ix_trips_city_requested 83.)
-- expect-line: ix_trips_city_requested | 83  | 679936  | 241.0
-- expect-line: trips                   | 374 | 3063808 | 53.5
-- expect-line: trips_pkey              | 102 | 835584  | 196.1
-- expect-line: uq_trips_code           | 57  | 466944  | 350.9
SELECT relname, relpages, pg_relation_size(oid) AS bytes,
       round((reltuples / relpages)::numeric, 1) AS rows_per_page
FROM pg_class
WHERE relnamespace = 'bai_cu'::regnamespace
  AND relname IN ('trips', 'trips_pkey', 'uq_trips_code', 'ix_trips_city_requested')
ORDER BY relname;

-- BẪY trong câu của bài cũ: lọc theo relname mà KHÔNG lọc schema. Database nào có
-- hai bảng cùng tên (ở đây toinoi.trips và bai_cu.trips) sẽ trả cả hai.
-- expect-value: 7
SELECT count(*) FROM pg_class
WHERE relname IN ('trips', 'trips_pkey', 'uq_trips_code', 'ix_trips_city_requested');

-- K3 · Mổ page 0 của heap bằng pageinspect (bài cũ: lower 240, upper 280, trống 40).
-- expect-line: 236 | 328 | 92 | 53
SELECT lower, upper, upper - lower AS free_bytes, (lower - 24) / 4 AS so_line_pointer
FROM page_header(get_raw_page('bai_cu.trips', 0));

-- Từng "ô" (line pointer) của page 0: vị trí byte (lp_off) và độ dài (lp_len) của dòng.
-- Dòng được ghi từ CUỐI page ngược lên: ô 1 nằm ở byte cao nhất.
-- expect-line: 1  | 8048 | 144
-- expect-line: 2  | 7904 | 144
-- expect-line: 3  | 7752 | 152
-- expect-line: 53 | 328  | 144
SELECT lp, lp_off, lp_len FROM heap_page_items(get_raw_page('bai_cu.trips', 0))
WHERE lp IN (1, 2, 3, 53) ORDER BY lp;

-- Kích thước trung bình một chuyến (bài cũ: 147,3 byte).
-- expect-value: 147.3
SELECT round(avg(pg_column_size(t.*)), 1) FROM bai_cu.trips AS t;

-- ---------------------------------------------------------------------------
-- K4 · Đọc: lấy một dòng cũng phải đọc cả page. Chạy mỗi câu HAI lần, ghi lần 2
--      (cache đã ấm ⇒ toàn bộ là shared hit). Bảng của bài cũ:
--        ctid = (100,7) chỉ lấy fare ................. 1 page
--        code = 112345 mọi cột ....................... 3 page (gốc index, lá index, heap)
--        code = 112346 chỉ fare ...................... 3 page
--        code BETWEEN 112000 AND 112052 (53 dòng) .... 4 page
--        53 chuyến rải rác, cách nhau 377 số ......... 159 page
--        count(*) .................................... 374 page
SELECT fare FROM trips WHERE ctid = '(100,7)';
-- expect-match: Tid Scan on trips
-- expect-match: Buffers: shared hit=1$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT fare FROM trips WHERE ctid = '(100,7)';

SELECT * FROM trips WHERE code = 112345;
-- expect-match: Buffers: shared hit=3$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code = 112345;

SELECT fare FROM trips WHERE code = 112346;
-- expect-match: Buffers: shared hit=3$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT fare FROM trips WHERE code = 112346;

SELECT count(*) FROM trips WHERE code BETWEEN 112000 AND 112052;
-- expect-match: actual rows=53\.00
-- expect-match: Buffers: shared hit=4$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code BETWEEN 112000 AND 112052;

-- 53 chuyến rải rác (100000, 100377, 100754, …). Viết bằng = ANY(mảng) ⇒ MỘT Index
-- Scan với 53 lần dò cây ("Index Searches: 53"). Bài cũ đo 159 page = 53 × 3.
SELECT count(*) FROM trips WHERE code = ANY (ARRAY(SELECT 100000 + 377 * g FROM generate_series(0, 52) AS g));
-- expect-match: actual rows=53\.00
-- expect-match: Index Searches: 53
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code = ANY (ARRAY(SELECT 100000 + 377 * g FROM generate_series(0, 52) AS g));

-- Các dòng đó nằm trên bao nhiêu page heap khác nhau? (53 — mỗi dòng một page)
-- expect-value: 53
SELECT count(DISTINCT (ctid::text::point)[0]) FROM trips
WHERE code = ANY (ARRAY(SELECT 100000 + 377 * g FROM generate_series(0, 52) AS g));

-- count(*) của bài cũ: 374 page = đọc hết heap. Trên bảng VỪA VACUUM (visibility map
-- đánh dấu mọi page "all-visible"), PostgreSQL chọn đếm trên index NHỎ NHẤT
-- (ix_trips_driver_id, 51 page nhờ gộp khoá trùng) bằng Index Only Scan, không chạm heap.
SELECT count(*) FROM trips;
-- expect-match: Index Only Scan using ix_trips_driver_id on trips
-- expect-match: Heap Fetches: 0
-- expect-match: Buffers: shared hit=51$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips;

-- Muốn thấy đúng phép đo của bài cũ: đếm một cột không có index ⇒ Seq Scan đọc hết 374 page.
SELECT count(fare) FROM trips;
-- expect-match: Seq Scan on trips
-- expect-match: Buffers: shared hit=374$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(fare) FROM trips;

-- ---------------------------------------------------------------------------
-- K5 · Buffer trong shared_buffers nhìn bằng pg_buffercache. Đẩy mọi page của
--      bai_cu.trips ra khỏi shared_buffers (hàm mới của PostgreSQL 18), rồi đọc lại.
-- expect-value: 16384
SELECT setting FROM pg_settings WHERE name = 'shared_buffers';

SELECT buffers_evicted > 0 AS da_day_ra FROM pg_buffercache_evict_relation('bai_cu.trips');

-- expect-value: 0
SELECT count(*) FROM pg_buffercache
WHERE relfilenode = pg_relation_filenode('bai_cu.trips') AND reldatabase = (SELECT oid FROM pg_database WHERE datname = current_database());

-- Lần đọc này phải lấy 374 page từ hệ điều hành (read) — có thể từ page cache của
-- OS chứ không chắc từ đĩa: PostgreSQL không phân biệt được.
-- expect-match: Buffers: shared read=374$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(fare) FROM trips;

-- Giờ cả 374 page nằm trong shared_buffers, mỗi page đúng một buffer.
-- expect-line: 374 | 1
SELECT count(*) AS so_buffer, min(usagecount) AS usagecount
FROM pg_buffercache
WHERE relfilenode = pg_relation_filenode('bai_cu.trips') AND reldatabase = (SELECT oid FROM pg_database WHERE datname = current_database())
  AND relforknumber = 0;

-- ---------------------------------------------------------------------------
-- K6 · Ghi cũng theo page, và LẦN ĐẦU sau checkpoint đắt hơn hẳn (full page image).
--      Hai UPDATE liên tiếp sửa giá hai chuyến CÙNG NẰM trên page cuối của heap.
--      Bài cũ: update_1 = 7.160 byte WAL (chứa ảnh page 7.024 byte), update_2 = 112 byte.
-- expect-line: 119998 | 373
-- expect-line: 119999 | 373
SELECT code, (ctid::text::point)[0]::int AS page FROM trips WHERE code IN (119998, 119999) ORDER BY code;

CHECKPOINT;
SELECT pg_current_wal_insert_lsn() AS lsn_truoc \gset
UPDATE trips SET fare = fare + 1000 WHERE code = 119998;
SELECT pg_current_wal_insert_lsn() AS lsn_giua \gset
UPDATE trips SET fare = fare + 1000 WHERE code = 119999;
SELECT pg_current_wal_insert_lsn() AS lsn_sau \gset

-- Từng bản ghi WAL của hai câu (bỏ bản ghi "Standby" do tiến trình nền tự ghi định
-- kỳ — không thuộc câu UPDATE). fpi_length = độ dài ảnh page (full page image).
-- expect-line: 1 | Heap        | HOT_UPDATE | 6573 | 6508
-- expect-line: 1 | Transaction | COMMIT     | 34   | 0
-- expect-line: 2 | Heap        | HOT_UPDATE | 80   | 0
-- expect-line: 2 | Transaction | COMMIT     | 34   | 0
SELECT CASE WHEN start_lsn < :'lsn_giua' THEN 1 ELSE 2 END AS cau, resource_manager, record_type,
       record_length, fpi_length
FROM pg_get_wal_records_info(:'lsn_truoc', :'lsn_sau')
WHERE resource_manager <> 'Standby'
ORDER BY start_lsn;

-- Tổng theo câu (khớp cách bài cũ trừ hai LSN, trừ bản ghi Standby nếu có).
-- expect-line: 1 | 6607
-- expect-line: 2 | 114
SELECT CASE WHEN start_lsn < :'lsn_giua' THEN 1 ELSE 2 END AS cau, sum(record_length) AS wal_bytes
FROM pg_get_wal_records_info(:'lsn_truoc', :'lsn_sau')
WHERE resource_manager <> 'Standby'
GROUP BY 1 ORDER BY 1;

-- Hiệu LSN thô như bài cũ (gồm cả phần đệm căn lề 8 byte giữa các bản ghi và bản
-- ghi Standby nếu tiến trình nền chen vào) — nên có thể lớn hơn tổng trên vài byte.
-- lab: bien-dong
SELECT pg_wal_lsn_diff(:'lsn_giua', :'lsn_truoc') AS update_1_bytes,
       pg_wal_lsn_diff(:'lsn_sau', :'lsn_giua') AS update_2_bytes;

-- Trường hợp thứ ba của bài cũ: sửa giá một chuyến CŨ trên page đã đầy, ngay sau
-- checkpoint ⇒ phiên bản mới không vừa page cũ ⇒ dời sang page khác (KHÔNG phải HOT)
-- ⇒ phải thêm mục vào CẢ 5 index, mỗi page index bị sửa lần đầu lại kèm một ảnh page.
-- (Bài cũ: 42.424 byte.) Có thêm bản ghi Heap LOCK: khi phiên bản mới phải sang page
-- khác, PostgreSQL tạm khoá dòng cũ trong lúc nhả khoá page để tìm page mới.
CHECKPOINT;
SELECT pg_current_wal_insert_lsn() AS lsn_a \gset
UPDATE trips SET fare = fare + 1000 WHERE code = 110000;
SELECT pg_current_wal_insert_lsn() AS lsn_b \gset

-- expect-line: Btree       | INSERT_LEAF | 5
-- expect-line: Heap        | LOCK        | 1
-- expect-line: Heap        | UPDATE      | 1
-- expect-line: Transaction | COMMIT      | 1
SELECT resource_manager, record_type, count(*) AS so_ban_ghi
FROM pg_get_wal_records_info(:'lsn_a', :'lsn_b')
WHERE resource_manager <> 'Standby'
GROUP BY 1, 2 ORDER BY 1, 2;

-- expect-line: 8 | 7 | 47691
SELECT count(*) AS so_ban_ghi, count(*) FILTER (WHERE fpi_length > 0) AS co_anh_page, sum(record_length) AS wal_bytes
FROM pg_get_wal_records_info(:'lsn_a', :'lsn_b')
WHERE resource_manager <> 'Standby';

-- ---------------------------------------------------------------------------
-- K7 · Cùng các ý trên bảng 1 triệu chuyến (toinoi.trips — lược đồ P01/P02, có thêm
--      CHECK, pickup_area_id). Cây index sâu hơn ⇒ tra một mã chuyến tốn 4 page (gốc,
--      nhánh, lá, heap) thay vì 3.
SET search_path TO toinoi, public;

-- expect-line: trips | 19441 | 51.4
SELECT relname, relpages, round((reltuples / relpages)::numeric, 1) AS rows_per_page
FROM pg_class WHERE oid = 'toinoi.trips'::regclass;

-- expect-value: 149.2
SELECT round(avg(pg_column_size(t.*)), 1) FROM (SELECT * FROM trips WHERE code BETWEEN 600000 AND 619999) AS t;

-- Độ sâu cây B-tree: level của gốc (0 = gốc cũng là lá). 1 triệu mã ⇒ level 2 = 3 tầng.
-- expect-line: uq_trips_code      | 2
-- expect-line: bai_cu.uq_trips_code | 1
SELECT 'uq_trips_code' AS index, level FROM bt_metap('toinoi.uq_trips_code')
UNION ALL
SELECT 'bai_cu.uq_trips_code', level FROM bt_metap('bai_cu.uq_trips_code');

-- Lạnh rồi nóng trên database vừa chép (shared_buffers chưa có page nào của trips).
-- (1 page — gốc của uq_trips_code — đã nằm trong shared_buffers từ câu bt_metap ở trên.)
-- expect-match: Buffers: shared hit=1 read=3$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code = 777777;

-- expect-match: Buffers: shared hit=4$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code = 777777;

-- Cùng 53 dòng: liền nhau theo mã (5 page = 3 index + 2 heap) và rải rác (212 page = 53 × 4).
-- expect-match: Buffers: shared hit=1 read=4$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code BETWEEN 700000 AND 700052;

-- expect-match: Index Searches: 53
-- expect-match: Buffers: shared hit=102 read=110$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code = ANY (ARRAY(SELECT 100000 + 18000 * g FROM generate_series(0, 52) AS g));
