-- P03 · 23-btree.sql — PG-023 «B-tree index giúp tìm chuyến như thế nào?»
-- (`postgresql-btree-index-co-ban`).
--
-- Làm được sau bài: so sánh đường tìm dữ liệu khi CHƯA và ĐÃ có index (số page đọc),
-- nhìn cấu trúc cây B-tree (gốc → nhánh → lá → heap), và giải thích vì sao planner
-- vẫn chọn Seq Scan khi điều kiện chọn NHIỀU dòng.
-- Chạy trên database mới chép từ nền p03. Bài TỰ BỎ index ix_trips_customer_requested
-- để dựng lại tình huống "chưa có index" (chỉ trong database của bài).

CREATE EXTENSION pageinspect WITH SCHEMA public;
CREATE EXTENSION pgstattuple WITH SCHEMA public;
SET search_path TO toinoi, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

SELECT id AS kh FROM customers WHERE phone = '0910000500' \gset

-- ---------------------------------------------------------------------------
-- 1. CHƯA có index theo khách: tìm 40 chuyến trong 1.000.000 ⇒ Seq Scan đọc MỌI
--    page của heap (19.441 page ≈ 152 MB) và bỏ đi 999.960 dòng.
DROP INDEX ix_trips_customer_requested;

-- expect-match: Seq Scan on trips \(actual rows=40\.00 loops=1\)
-- expect-match: Rows Removed by Filter: 999960
-- expect-match: Buffers: shared (hit=\d+ )?read=\d+
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, requested_at, fare FROM trips WHERE customer_id = :'kh';

-- Tổng page đã chạm (hit + read) = số page của bảng.
-- expect-value: 19441
SELECT pg_relation_size('trips') / current_setting('block_size')::int AS so_page_heap;

-- ---------------------------------------------------------------------------
-- 2. TẠO index B-tree trên customer_id rồi chạy lại: đi cây (3 page) rồi đọc đúng
--    các page heap chứa 40 chuyến — 44 page thay vì 19.441.
CREATE INDEX ix_trips_customer ON trips (customer_id);

-- expect-match: actual rows=40\.00
-- expect-match: Bitmap Index Scan on ix_trips_customer|Index Scan using ix_trips_customer
-- expect-match: Buffers: shared (hit=\d+ )?read=4[0-9]$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, requested_at, fare FROM trips WHERE customer_id = :'kh';

-- Cái giá: một cấu trúc nữa trên đĩa (và phải cập nhật khi ghi — PG-025 đo).
-- 12,7 MB cho 1 triệu mục là nhờ GỘP khoá trùng (deduplication, PG 13+): khách đặt
-- nhiều chuyến ⇒ một khoá + danh sách ctid. Index (customer_id, requested_at DESC)
-- của nền thì 40,6 MB vì cặp khoá hầu như không trùng (02-kiem-tra-fixture.sql, H9).
-- expect-line: ix_trips_customer | 12.7
SELECT 'ix_trips_customer' AS index, round(pg_relation_size('ix_trips_customer') / 1048576.0, 1) AS mb;

-- ---------------------------------------------------------------------------
-- 3. Giải phẫu cây. Meta page (block 0) ghi page gốc và "level" của gốc
--    (0 = gốc cũng là lá). Level 2 ⇒ 3 tầng: gốc → nhánh → lá.
-- expect-line: 2
SELECT level FROM bt_metap('ix_trips_customer');

-- Thống kê cả cây (pgstattuple): số page lá/nhánh, độ đầy trung bình của page lá.
-- expect-line: 2 | 1611 | 10 | 90.3
SELECT tree_level, leaf_pages, internal_pages, avg_leaf_density FROM pgstatindex('ix_trips_customer');

-- Page gốc: loại 'r' (root), mỗi mục trỏ tới một page nhánh.
-- expect-line: r | 9
SELECT type, live_items FROM bt_page_stats('ix_trips_customer', (SELECT root FROM bt_metap('ix_trips_customer')));

-- Ba mục đầu của page lá đầu tiên: khoá đã SẮP (uuid tăng dần) + ctid trỏ vào heap.
-- Khoá có nhiều chuyến được GỘP (deduplication): một khoá + danh sách ctid (cột tids).
-- (Mục 1 của page lá không phải là lá dữ liệu: đó là "high key" — cận trên của page.)
-- expect-line: 2 | 00 00 30 bd 09 f1 4c 73 | (658,41)  | 11
-- expect-line: 3 | 00 00 9b 1f 52 28 48 75 | (1326,11) | NULL
-- expect-line: 4 | 00 00 d1 30 2b f1 43 52 | (4812,8)  | 4
SELECT itemoffset, left(data, 23) AS khoa_dau, htid, cardinality(tids) AS so_ctid_gop
FROM bt_page_items('ix_trips_customer', 1)
WHERE itemoffset BETWEEN 2 AND 4
ORDER BY itemoffset;

-- ---------------------------------------------------------------------------
-- 4. Khi nào planner VẪN chọn Seq Scan? Khi điều kiện chọn nhiều dòng. Index trên
--    fare (cước PHÂN BỐ NGẪU NHIÊN theo vị trí vật lý): dòng khớp rải khắp heap.
CREATE INDEX ix_trips_fare ON trips (fare);

-- Tỉ lệ dòng khớp: 0,35 % · 3,9 % · 11,1 % · 26,6 % · 55,5 %.
-- expect-line: 150000 | 3541   | 0.35
-- expect-line: 140000 | 39270  | 3.93
-- expect-line: 120000 | 111141 | 11.11
-- expect-line: 90000  | 266345 | 26.63
-- expect-line: 60000  | 555165 | 55.52
SELECT lo, count(*) FILTER (WHERE fare BETWEEN lo AND 150000) AS so_dong,
       round(100.0 * count(*) FILTER (WHERE fare BETWEEN lo AND 150000) / count(*), 2) AS phan_tram
FROM trips CROSS JOIN (VALUES (150000), (140000), (120000), (90000), (60000)) AS v (lo)
GROUP BY lo ORDER BY lo DESC;

-- Kế hoạch từng mức (ke_hoach() chỉ EXPLAIN, không chạy): tới 26,6 % vẫn Bitmap (gom
-- ctid, đọc mỗi page heap một lần theo thứ tự page), tới 55,5 % thì Seq Scan. Ngưỡng
-- chuyển nằm ĐÂU ĐÓ giữa hai mức — phụ thuộc cost model (random_page_cost = 4,
-- effective_cache_size = 4GB…) chứ không có con số cố định kiểu "trên 10 % là Seq Scan".
-- expect-line: 150000 | Bitmap Heap Scan on trips / Bitmap Index Scan on ix_trips_fare
-- expect-line: 140000 | Bitmap Heap Scan on trips / Bitmap Index Scan on ix_trips_fare
-- expect-line: 120000 | Bitmap Heap Scan on trips / Bitmap Index Scan on ix_trips_fare
-- expect-line: 90000  | Bitmap Heap Scan on trips / Bitmap Index Scan on ix_trips_fare
-- expect-line: 60000  | Seq Scan on trips
SELECT lo, ke_hoach(format('SELECT * FROM trips WHERE fare BETWEEN %s AND 150000', lo)) AS ke_hoach
FROM (VALUES (150000), (140000), (120000), (90000), (60000)) AS v (lo)
ORDER BY lo DESC;

-- Ở 11 % dòng, Bitmap đã phải chạm 19.403 / 19.441 page heap: cước rải ngẫu nhiên
-- nên gần như page nào cũng có vài chuyến khớp.
-- expect-match: Bitmap Heap Scan on trips \(actual rows=111141\.00 loops=1\)
-- expect-match: Heap Blocks: exact=19403
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE fare BETWEEN 120000 AND 150000;

-- 55,5 %: Seq Scan đọc đúng 19.441 page, tuần tự. Ép dùng index (SET LOCAL trong giao
-- dịch) ⇒ vẫn chạm cả 19.441 page heap CỘNG thêm page index — nhiều page hơn, lại
-- qua hai cấu trúc. Thời gian thật của hai cách: p03/DO-DAC.md (PG-023).
-- expect-match: Seq Scan on trips \(actual rows=555165\.00 loops=1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE fare BETWEEN 60000 AND 150000;

BEGIN;
SET LOCAL enable_seqscan = off;
-- expect-match: Bitmap Heap Scan on trips
-- expect-match: Heap Blocks: exact=19441
-- expect-match: ->  Bitmap Index Scan on ix_trips_fare \(actual rows=555165\.00 loops=1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE fare BETWEEN 60000 AND 150000;
ROLLBACK;

-- ---------------------------------------------------------------------------
-- 5. Cột THẲNG HÀNG với heap (requested_at, correlation = 1) cho kết quả khác hẳn:
--    các dòng khớp nằm liền một dải page ⇒ Index Scan rẻ cả khi lấy rất nhiều dòng.
CREATE INDEX ix_trips_requested ON trips (requested_at);

-- 1 ngày (0,6 %) · 7 ngày (4 %) · 30 ngày (~18 %) · 60 ngày (~36 %) · 120 ngày (~72 %)
-- · 180 ngày (~98 %): Index Scan tới tận 72 % — so với 26,6 % → Bitmap, 55,5 % → Seq
-- Scan ở cột fare không thẳng hàng. Correlation quyết định giá của việc "đi index".
-- expect-line: 1   | Index Scan using ix_trips_requested on trips
-- expect-line: 7   | Index Scan using ix_trips_requested on trips
-- expect-line: 30  | Index Scan using ix_trips_requested on trips
-- expect-line: 60  | Index Scan using ix_trips_requested on trips
-- expect-line: 120 | Index Scan using ix_trips_requested on trips
-- expect-line: 180 | Seq Scan on trips
SELECT ngay, ke_hoach(format($$SELECT * FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-10-31 00:00+07' - interval '%s days' AND requested_at < TIMESTAMPTZ '2026-10-31 00:00+07'$$, ngay)) AS ke_hoach
FROM (VALUES (1), (7), (30), (60), (120), (180)) AS v (ngay)
ORDER BY ngay;

-- Tháng 10: 184 nghìn dòng qua Index Scan, chỉ chạm ~4.200 page (heap liền một dải + lá index).
-- expect-match: Index Scan using ix_trips_requested on trips \(actual rows=184396\.00 loops=1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-31 01:00+07';

-- Hệ số correlation mà planner dùng (pg_stats, từ ANALYZE lấy mẫu).
-- lab: bien-dong
SELECT attname, round(correlation::numeric, 3) AS correlation
FROM pg_stats WHERE schemaname = 'toinoi' AND tablename = 'trips' AND attname IN ('fare', 'requested_at')
ORDER BY attname;

-- ---------------------------------------------------------------------------
-- 6. Thứ tự chèn ảnh hưởng tới chính cây. pk_trips (uuid NGẪU NHIÊN, chèn vào giữa
--    cây ⇒ page lá bị tách đôi liên tục) xốp hơn uq_trips_code (mã TĂNG DẦN, luôn chèn
--    ở mép phải). Cả hai đều 1.000.000 mục.
-- expect-line: pk_trips      | 4848 | 71.19
-- expect-line: uq_trips_code | 2733 | 90.01
SELECT 'pk_trips' AS index, leaf_pages, avg_leaf_density FROM pgstatindex('pk_trips')
UNION ALL
SELECT 'uq_trips_code', leaf_pages, avg_leaf_density FROM pgstatindex('uq_trips_code');

-- Dựng lại cây từ dữ liệu đã sắp (REINDEX) ⇒ page lá đầy ~90 % ⇒ nhỏ đi.
REINDEX INDEX pk_trips;
-- expect-line: pk_trips | 3832 | 89.98
SELECT 'pk_trips' AS index, leaf_pages, avg_leaf_density FROM pgstatindex('pk_trips');
