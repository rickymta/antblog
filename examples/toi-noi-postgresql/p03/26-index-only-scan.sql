-- P03 · 26-index-only-scan.sql — PG-026 «Index-only scan, INCLUDE và visibility map»
-- (`postgresql-index-only-scan-visibility-map`).
--
-- Làm được sau bài: quan sát `Heap Fetches` của Index Only Scan trước/sau khi dữ liệu bị
-- sửa và sau VACUUM; đọc visibility map bằng pg_visibility; dùng INCLUDE để một câu
-- truy vấn được trả lời hoàn toàn từ index (covering index) và biết cái giá của nó.
--
-- Ý chính: index KHÔNG lưu thông tin "phiên bản dòng này còn sống với ai" (MVCC nằm
-- ở heap). Index Only Scan chỉ bỏ qua heap được với những page mà visibility map đánh
-- dấu ALL-VISIBLE (mọi dòng trong page đều thấy được với mọi giao dịch). Page nào
-- vừa bị sửa thì mất dấu đó cho tới lần VACUUM kế tiếp ⇒ phải đọc heap ("Heap Fetches").

CREATE EXTENSION pg_visibility WITH SCHEMA public;
SET search_path TO toinoi, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

SELECT id AS kh_b1 FROM customers WHERE phone = '0919990001' \gset

-- ---------------------------------------------------------------------------
-- 1. Nền vừa VACUUM: MỌI page của trips đều all-visible.
-- expect-line: 19441 | 19441
SELECT all_visible, (SELECT relpages FROM pg_class WHERE oid = 'trips'::regclass) AS so_page
FROM pg_visibility_map_summary('trips');

-- 2. Câu chỉ cần cột NẰM TRONG index (customer_id, requested_at): giờ đặt các chuyến
--    tháng 10 của khách doanh nghiệp B1 ⇒ Index Only Scan, "Heap Fetches: 0".
-- expect-match: Index Only Scan using ix_trips_customer_requested on trips \(actual rows=9365\.00 loops=1\)
-- expect-match: Heap Fetches: 0$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';

-- Thêm một cột KHÔNG có trong index (fare) ⇒ phải đọc heap cho từng dòng.
-- expect-match: Index Scan using ix_trips_customer_requested on trips|Bitmap Heap Scan on trips
-- expect-no-match: Index Only Scan
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at, fare FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';

-- ---------------------------------------------------------------------------
-- 3. Sửa giá các chuyến tháng 10 của B1 (9.365 dòng). Page nào bị sửa mất dấu
--    all-visible ⇒ Index Only Scan phải ghé heap cho các dòng trên những page đó.
-- expect-rows: 9365
UPDATE trips SET fare = fare + 1000
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';

-- Số page còn all-visible giảm (page cũ bị sửa + page mới nhận phiên bản mới).
-- expect-range: 15000 19000
SELECT all_visible FROM pg_visibility_map_summary('trips');

-- Vẫn là Index Only Scan, nhưng giờ "Heap Fetches: 18536" (≈ 2 × 9.365): với mục index
-- trỏ vào page không all-visible, PostgreSQL phải đọc heap để biết phiên bản nào còn
-- sống. Index còn chứa CẢ mục trỏ phiên bản cũ (đã chết) — phải ghé heap mới loại được.
-- Số page chạm: 18.540 (trước khi sửa: 50).
-- expect-match: Index Only Scan using ix_trips_customer_requested on trips \(actual rows=9365\.00 loops=1\)
-- expect-match: Heap Fetches: 18536$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';

-- Một page cụ thể: page chứa chuyến tháng 10 đầu tiên của B1.
SELECT (ctid::text::point)[0]::int AS page_b1 FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07'
ORDER BY requested_at LIMIT 1 \gset
-- expect-line: f | f
SELECT all_visible, pd_all_visible FROM pg_visibility('trips', :page_b1);

-- ---------------------------------------------------------------------------
-- 4. VACUUM dọn phiên bản chết (cả mục index trỏ tới chúng) và đặt lại dấu all-visible.
VACUUM trips;

-- (Bảng đã lớn thêm 174 page vì phiên bản mới của các dòng bị sửa: 19.441 → 19.615.)
-- expect-line: 19615 | 19615
SELECT all_visible, pg_relation_size('trips') / 8192 AS so_page FROM pg_visibility_map_summary('trips');

-- expect-line: t | t
SELECT all_visible, pd_all_visible FROM pg_visibility('trips', :page_b1);

-- expect-match: Heap Fetches: 0$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';

-- Hệ quả vận hành: cluster lab TẮT autovacuum (để tái lập); trên server thật,
-- autovacuum làm việc của mục 4 — bảng sửa nhiều mà autovacuum chạy thưa thì
-- Index Only Scan "thoái hoá" dần thành Index Scan (P05 nói kỹ).

-- ---------------------------------------------------------------------------
-- 5. INCLUDE: thêm cột "đi kèm" vào lá index để trả lời trọn câu hỏi mà không cần heap.
--    Cột INCLUDE không dùng để tìm/sắp, chỉ để mang theo.
CREATE INDEX ix_trips_khach_bao_gom ON trips (customer_id, requested_at DESC) INCLUDE (fare, status);

-- expect-match: Index Only Scan using ix_trips_khach_bao_gom on trips
-- expect-match: Heap Fetches: 0$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at, fare, status FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';

-- Cái giá: index to hơn (mỗi mục mang thêm fare + status) và mọi UPDATE đổi fare/status
-- KHÔNG còn là HOT (cột có trong index ⇒ phải ghi index) — đúng hai cột hay sửa nhất của trips.
-- (Index cũ cũng đã lớn lên từ 40.624.128 byte vì các mục mới của lần UPDATE ở mục 3.)
-- expect-line: ix_trips_customer_requested | 41000960
-- expect-line: ix_trips_khach_bao_gom      | 58892288
SELECT relname, pg_relation_size(oid) AS bytes FROM pg_class
WHERE relname IN ('ix_trips_customer_requested', 'ix_trips_khach_bao_gom') AND relnamespace = 'toinoi'::regnamespace
ORDER BY relname;

-- Chứng minh "không còn HOT": bảng nhỏ fillfactor 90 (page CÒN chỗ), sửa giá 500 dòng
-- rải đều (mỗi dòng thứ 20) — trước và sau khi có index INCLUDE (fare).
-- (Lưu ý khi tự thử: UPDATE gán LẠI ĐÚNG giá trị cũ cho cột có index vẫn được HOT —
-- PostgreSQL so giá trị cũ/mới, không chỉ xem cột có nằm trong SET hay không.)
CREATE TABLE hot_thu WITH (fillfactor = 90) AS SELECT * FROM trips WHERE code BETWEEN 600000 AND 609999;
CREATE INDEX ON hot_thu (customer_id, requested_at DESC);
UPDATE hot_thu SET fare = fare + 1000 WHERE code % 20 = 0;
CREATE INDEX ON hot_thu (customer_id, requested_at DESC) INCLUDE (fare, status);
UPDATE hot_thu SET fare = fare + 1000 WHERE code % 20 = 10;
SELECT pg_stat_force_next_flush();
-- 1.000 UPDATE: 500 lần đầu (chưa có INCLUDE) đều HOT, 500 lần sau không lần nào.
-- expect-line: 1000 | 500
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_user_tables WHERE relid = 'hot_thu'::regclass;

-- ---------------------------------------------------------------------------
-- 6. count(*) cũng hưởng lợi: bảng vừa VACUUM ⇒ đếm bằng Index Only Scan trên index
--    NHỎ NHẤT (ix_trips_driver_id, gộp khoá trùng) thay vì đọc 19.441 page heap.
-- expect-match: Index Only Scan using ix_trips_driver_id on trips
-- expect-match: Heap Fetches: 0$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips;
