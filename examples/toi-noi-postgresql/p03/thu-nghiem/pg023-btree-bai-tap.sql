-- Thí nghiệm PG-023 «B-tree index giúp tìm chuyến như thế nào?» — ví dụ phụ, lỗi
-- thường gặp và lời giải bài tập. Không có trong lab chính (23-btree.sql).
-- Như 23-btree.sql: bỏ ix_trips_customer_requested của nền, thay bằng index một cột
-- ix_trips_customer (chỉ trong database của tệp này).

CREATE EXTENSION pageinspect WITH SCHEMA public;
SET search_path TO toinoi, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

SELECT id AS kh FROM customers WHERE phone = '0910000500' \gset
SELECT id AS b1 FROM customers WHERE phone = '0919990001' \gset

DROP INDEX ix_trips_customer_requested;
CREATE INDEX ix_trips_customer ON trips (customer_id);

-- ---------------------------------------------------------------------------
-- 1. Page gốc của ix_trips_customer: mỗi mục là (khoá phân cách, page con). Mục đầu
--    không có khoá (âm vô cùng); ctid ở page nhánh là "số page con", không phải heap.
-- expect-rows: 3
SELECT itemoffset, ctid, left(data, 23) AS khoa
FROM bt_page_items('ix_trips_customer', (SELECT root FROM bt_metap('ix_trips_customer')))
WHERE itemoffset <= 3 ORDER BY itemoffset;

-- Ba tầng: gốc (1 page) → nhánh (9 page) → lá. Tầng 0 là lá.
-- expect-rows: 3
SELECT btpo_level AS tang, type, count(*) AS so_page
FROM bt_multi_page_stats('ix_trips_customer', 1, (pg_relation_size('ix_trips_customer') / 8192 - 1)::int)
GROUP BY btpo_level, type ORDER BY btpo_level DESC;

-- ---------------------------------------------------------------------------
-- 2. Lỗi thường gặp: bọc cột bằng ép kiểu ⇒ index trên customer_id không dùng được.
-- expect-match: Seq Scan on trips
-- expect-match: Rows Removed by Filter: 999960
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code FROM trips WHERE customer_id::text = :'kh';

-- Viết điều kiện trên CỘT trần, ép kiểu phía hằng số ⇒ index dùng được.
-- expect-match: ix_trips_customer
-- expect-match: actual rows=40\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code FROM trips WHERE customer_id = :'kh'::uuid;

-- ---------------------------------------------------------------------------
-- 3. LIKE có tiền tố cố định dùng được B-tree (database lab có collation C);
--    tiền tố là '%' thì không.
-- expect-line: C
SELECT datcollate FROM pg_database WHERE datname = current_database();
-- expect-match: Index (Only )?Scan using uq_customers_phone
EXPLAIN (COSTS OFF) SELECT phone FROM customers WHERE phone LIKE '091000050%';
-- expect-match: Seq Scan on customers
EXPLAIN (COSTS OFF) SELECT phone FROM customers WHERE phone LIKE '%0500';

-- ---------------------------------------------------------------------------
-- 4. Bài tập 1: index trên cột ít giá trị (status). Giá trị hiếm ⇒ index; giá trị
--    chiếm 88 % bảng ⇒ Seq Scan.
CREATE INDEX ix_trips_status ON trips (status);
-- expect-line: cancelled | 119658
-- expect-line: completed | 880042
SELECT status, count(*) FROM trips WHERE status IN ('completed', 'cancelled') GROUP BY status ORDER BY status;
-- expect-match: Seq Scan on trips
-- expect-no-match: ix_trips_status
SELECT ke_hoach($$SELECT * FROM trips WHERE status = 'completed'$$);
-- expect-match: ix_trips_status
SELECT ke_hoach($$SELECT * FROM trips WHERE status = 'searching'$$);
-- expect-match: actual rows=50\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE status = 'searching';

-- ---------------------------------------------------------------------------
-- 5. Bài tập 2: khách doanh nghiệp B1 (51.151 chuyến, 5,1 % bảng) qua ix_trips_customer.
-- expect-match: actual rows=51151\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, fare FROM trips WHERE customer_id = :'b1';

-- ---------------------------------------------------------------------------
-- 6. Bài tập 3: tra một mã chuyến qua uq_trips_code (level 2 ⇒ 3 tầng) = 3 page index
--    + 1 page heap.
-- expect-line: 2
SELECT level FROM bt_metap('uq_trips_code');
-- Lần chạy đầu trong phiên chạm thêm vài page (không lặp lại ở lần sau) ⇒ không so.
-- lab: bien-dong
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code = 600000;
-- Chạy lại: đúng 4 page — gốc, nhánh, lá của uq_trips_code + 1 page heap.
-- expect-match: Buffers: shared hit=4$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code = 600000;
