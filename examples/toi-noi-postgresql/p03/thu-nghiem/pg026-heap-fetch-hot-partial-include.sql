-- Thí nghiệm PG-026 «Index-only scan, INCLUDE và visibility map» — phép thử phụ và lời giải
-- bài tập trích trong bài (không có trong lab chính 26-index-only-scan.sql).
-- Chạy: node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg026-heap-fetch-hot-partial-include.sql

CREATE EXTENSION pg_visibility WITH SCHEMA public;
CREATE EXTENSION pageinspect WITH SCHEMA public;
SET search_path TO toinoi, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

SELECT id AS kh_b1 FROM customers WHERE phone = '0919990001' \gset

-- Visibility map là một "fork" riêng của bảng: 2 bit mỗi page heap ⇒ 19.441 page heap
-- (159 MB) chỉ cần một page VM 8 kB.
-- expect-line: 159260672 | 8192
SELECT pg_relation_size('trips', 'main') AS heap_bytes, pg_relation_size('trips', 'vm') AS vm_bytes;

-- ---------------------------------------------------------------------------
-- A. Vì sao 18.536 chứ không phải 2 × 9.365 = 18.730? Lặp lại UPDATE của lab chính rồi
--    đếm HOT: phiên bản mới nằm CÙNG page với bản cũ (HOT) thì KHÔNG có mục index mới.
UPDATE trips SET fare = fare + 1000
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';
SELECT pg_stat_force_next_flush();
-- 9.365 UPDATE, 194 HOT ⇒ mục index = 9.365 (bản cũ) + 9.171 (bản mới không HOT) = 18.536.
-- expect-line: 9365 | 194
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_user_tables WHERE relid = 'trips'::regclass;

-- Số mục index ĐÃ bị đánh dấu chết (cột dead của bt_page_items) — trước lần đọc đầu: 0.
-- expect-value: 0
SELECT count(*) FILTER (WHERE dead)
FROM generate_series(1, pg_relation_size('ix_trips_customer_requested') / 8192 - 1) AS b,
     LATERAL bt_page_items('ix_trips_customer_requested', b);

-- Lần 1 sau UPDATE: ghé heap cho mọi mục index trỏ vào page không all-visible.
-- expect-match: Heap Fetches: 18536$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';

-- Lần đọc đó thấy phiên bản cũ đã chết với MỌI giao dịch ⇒ đánh dấu chết (LP_DEAD) một
-- phần mục index trỏ tới chúng; lần đọc sau bỏ qua các mục này, không ghé heap.
-- 6.197 mục bị đánh dấu ⇒ lần sau: 18.536 − 6.197 = 12.339 heap fetch.
-- expect-value: 6197
SELECT count(*) FILTER (WHERE dead)
FROM generate_series(1, pg_relation_size('ix_trips_customer_requested') / 8192 - 1) AS b,
     LATERAL bt_page_items('ix_trips_customer_requested', b);

-- Lần 2 (vẫn chưa VACUUM, visibility map chưa đổi): ít heap fetch hơn.
-- expect-match: Heap Fetches: 12339$
-- expect-match: Buffers: shared hit=6483$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';

-- Lần 3: ổn định.
-- expect-match: Heap Fetches: \d+$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';

-- expect-range: 15000 19000
SELECT all_visible FROM pg_visibility_map_summary('trips');

-- count(*) cũng phải ghé heap cho MỌI dòng trên 3.512 page mất dấu all-visible.
-- expect-match: Index Only Scan using ix_trips_driver_id on trips
-- expect-match: Heap Fetches: 180881$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips;

-- VACUUM VERBOSE: dòng chết nằm trên ~3.300 page (> 2 % số page) ⇒ KHÔNG bỏ qua dọn index.
-- lab: bien-dong
-- expect-no-match: index scan bypassed
VACUUM (VERBOSE) trips;

-- expect-match: Heap Fetches: 0$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';

-- ---------------------------------------------------------------------------
-- B. Ngược lại: sửa ÍT dòng, nằm gọn trong vài page ⇒ VACUUM thường bỏ qua dọn index.
-- expect-rows: 1000
UPDATE trips SET fare = fare + 1000 WHERE code BETWEEN 500000 AND 500999;
-- lab: bien-dong
-- expect-match: index scan bypassed
VACUUM (VERBOSE) trips;
-- expect-match: Heap Fetches: 1000$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code FROM trips WHERE code BETWEEN 500000 AND 500999;
VACUUM (INDEX_CLEANUP ON) trips;
-- expect-match: Heap Fetches: 0$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code FROM trips WHERE code BETWEEN 500000 AND 500999;

-- ---------------------------------------------------------------------------
-- C. Partial index + Index Only Scan: status không nằm trong index nhưng mọi mục của
--    index đều thoả điều kiện của index ⇒ không cần kiểm lại ⇒ Index Only Scan được.
CREATE INDEX ix_trips_dang_chay ON trips (requested_at)
WHERE status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit');
-- expect-match: Index Only Scan using ix_trips_dang_chay on trips
-- expect-match: actual rows=300\.00
-- expect-match: Heap Fetches: 0$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at FROM trips
WHERE status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit');

-- ---------------------------------------------------------------------------
-- D. Bài tập 2 — INCLUDE hay thêm cột vào KHOÁ? Cùng 4 cột, hai cách khai báo.
CREATE INDEX ix_khoa_4_cot ON trips (customer_id, requested_at DESC, fare, status);
CREATE INDEX ix_include_2_cot ON trips (customer_id, requested_at DESC) INCLUDE (fare, status);
-- expect-line: ix_include_2_cot            | 58892288
-- expect-line: ix_khoa_4_cot               | 58892288
-- expect-line: ix_trips_customer_requested | 41000960
SELECT relname, pg_relation_size(oid) AS bytes FROM pg_class
WHERE relname IN ('ix_khoa_4_cot', 'ix_include_2_cot', 'ix_trips_customer_requested')
  AND relnamespace = 'toinoi'::regnamespace
ORDER BY relname;

-- Bài tập 3 — UNIQUE kèm INCLUDE: tính duy nhất chỉ trên code, status chỉ "đi kèm".
CREATE UNIQUE INDEX uq_trips_code_kem_status ON trips (code) INCLUDE (status);
-- expect-match: Index Only Scan using uq_trips_code_kem_status on trips
-- expect-match: Heap Fetches: 0$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, status FROM trips WHERE code BETWEEN 500000 AND 500099;
