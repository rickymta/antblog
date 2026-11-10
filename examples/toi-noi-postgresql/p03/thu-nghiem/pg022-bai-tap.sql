-- Thí nghiệm PG-022 «Đơn vị đọc ghi nhỏ nhất của cơ sở dữ liệu là gì?» — kiểm thêm
-- vài số trong bài và lời giải ba bài tập. Không có trong lab chính
-- (p03/22-page-buffer.sql). Chạy trên bản sao nền p03 (schema bai_cu = 20.000 chuyến):
--   node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg022-bai-tap.sql
CREATE EXTENSION pg_buffercache WITH SCHEMA public;
SET search_path TO bai_cu, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- Kiểm câu trong bài: 53 chuyến liền mã 112000–112052 nằm trên đúng 2 page heap,
-- còn index uq_trips_code của bai_cu có 2 tầng (level 1: gốc + lá) ⇒ 4 page.
-- expect-value: 2
SELECT count(DISTINCT (ctid::text::point)[0]) FROM trips WHERE code BETWEEN 112000 AND 112052;

-- Kiểm số liệu Hình 1 của bài: 53 dòng của page 0 dài 136–152 byte, cộng lại đúng
-- 7.864 byte = 8.192 − upper (328) ⇒ 24 header + 212 line pointer + 92 trống + 7.864.
CREATE EXTENSION pageinspect WITH SCHEMA public;
-- expect-line: 136 | 152 | 7864 | 53
SELECT min(lp_len), max(lp_len), sum(lp_len), count(*) FROM heap_page_items(get_raw_page('bai_cu.trips', 0));

-- Page 373 (page cuối, nơi bài sửa giá hai chuyến 119998, 119999): còn 1.800 byte trống
-- giữa lower và upper ⇒ ảnh page trong WAL nhỏ hơn 8.192 byte.
-- expect-line: 208 | 2008 | 1800
SELECT lower, upper, upper - lower AS trong FROM page_header(get_raw_page('bai_cu.trips', 373));

-- ---------------------------------------------------------------------------
-- Bài 1. 1.000 chuyến liền mã 112000–112999: dự đoán số page rồi đo.
-- 1.000 ÷ 53,5 ≈ 19 page heap (+1 nếu dải không bắt đầu ở đầu page) + vài page index.
-- Đo: 19 page heap; Index Scan chạm 24 page = 19 heap + 5 index (gốc + 4 lá).
-- expect-value: 19
SELECT count(DISTINCT (ctid::text::point)[0]) FROM trips WHERE code BETWEEN 112000 AND 112999;

-- expect-match: Index Scan using uq_trips_code on trips \(actual rows=1000\.00 loops=1\)
-- expect-match: ^   Buffers: shared hit=24$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code BETWEEN 112000 AND 112999;

-- ---------------------------------------------------------------------------
-- Bài 2. Đếm số buffer của bai_cu.trips trong shared_buffers sau khi đẩy hết ra rồi
-- đọc MỘT chuyến theo mã: chỉ đúng 1 page heap được nạp (2 page còn lại thuộc index).
SELECT buffers_evicted >= 0 AS ok FROM pg_buffercache_evict_relation('bai_cu.trips');
SELECT buffers_evicted >= 0 AS ok FROM pg_buffercache_evict_relation('bai_cu.uq_trips_code');

-- expect-match: Buffers: shared read=3$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code = 112345;

-- expect-line: trips         | 1
-- expect-line: uq_trips_code | 2
SELECT c.relname, count(*) AS so_buffer
FROM pg_buffercache AS b
JOIN pg_class AS c ON c.relfilenode = b.relfilenode AND c.relnamespace = 'bai_cu'::regnamespace
WHERE b.reldatabase = (SELECT oid FROM pg_database WHERE datname = current_database())
  AND c.relname IN ('trips', 'uq_trips_code')
GROUP BY c.relname ORDER BY c.relname;

-- ---------------------------------------------------------------------------
-- Bài 3. count(*) khi bảng CHƯA được VACUUM sau một đợt sửa: visibility map mất dấu
-- all-visible ở các page bị sửa ⇒ Index Only Scan không còn "miễn phí".
-- expect-rows: 4000
UPDATE trips SET fare = fare + 1000 WHERE code % 5 = 0;

-- 4.000 phiên bản mới không vừa page cũ (fillfactor 100) ⇒ bảng 374 → 449 page; planner
-- chọn Seq Scan đọc cả 449 page.
-- expect-match: ->  Seq Scan on trips \(actual rows=20000\.00 loops=1\)
-- expect-match: ^   Buffers: shared hit=449$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips;

VACUUM trips;

-- Sau VACUUM: lại Index Only Scan trên ix_trips_driver_id, 61 page (index cũng to ra vì
-- mỗi UPDATE không HOT thêm một mục vào mọi index).
-- expect-match: Index Only Scan using ix_trips_driver_id on trips
-- expect-match: Heap Fetches: 0
-- expect-match: ^   Buffers: shared hit=61$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips;
