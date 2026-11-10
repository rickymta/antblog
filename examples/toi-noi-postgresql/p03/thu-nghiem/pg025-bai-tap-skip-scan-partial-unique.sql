-- Thí nghiệm PG-025 «Index nhiều cột, partial và expression» — phép thử phụ và lời giải
-- bài tập trích trong bài (không có trong lab chính 25-index-nhieu-cot-partial-expression.sql).
-- Chạy: node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg025-bai-tap-skip-scan-partial-unique.sql

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- A. Skip scan khi cột đầu có RẤT NHIỀU giá trị. Ẩn ix_trips_city_requested (xoá trong
--    giao dịch rồi ROLLBACK): còn ix_trips_customer_requested (customer_id, requested_at DESC),
--    cột đầu ~192.000 giá trị. count(*) chỉ cần index ⇒ planner vẫn chọn Index Only Scan với
--    điều kiện ở cột SAU, nhưng thực chất đọc gần hết index (4.650 / 4.959 page) — rẻ hơn
--    19.441 page heap, đắt gấp ~90 lần index đúng thứ tự cột (50 page).
BEGIN;
DROP INDEX ix_trips_city_requested;
-- expect-match: Index Only Scan using ix_trips_customer_requested on trips \(actual rows=5896\.00
-- expect-match: Index Searches: 20$
-- expect-match: Buffers: shared (hit=\d+ )?read=4[56]\d\d$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips
WHERE requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-16 00:00+07';

-- Câu cần cột NGOÀI index (code) thì phải ghé heap từng dòng ⇒ planner bỏ index, quét tuần tự.
-- expect-match: Seq Scan on trips \(actual rows=5896\.00
-- expect-match: Buffers: shared (hit=\d+ )?read=19\d{3}$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code FROM trips
WHERE requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-16 00:00+07';
ROLLBACK;

-- expect-value: 4959
SELECT pg_relation_size('ix_trips_customer_requested') / 8192;

-- A2. IN trên cột đầu: mỗi giá trị trong danh sách là một lần dò cây (Index Searches: 2).
-- expect-match: Index Searches: 2$
-- expect-match: actual rows=2643\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips
WHERE city_id IN (2, 3) AND requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-16 00:00+07';

-- ---------------------------------------------------------------------------
-- B. Bài tập 2 — bẫy tham số với partial index, cách sửa: lặp lại NGUYÊN VĂN điều kiện
--    của index trong câu (hằng số) ⇒ kế hoạch chung (generic) cũng chứng minh được.
CREATE INDEX ix_trips_dang_chay ON trips (requested_at)
WHERE status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit');

PREPARE dang_chay_theo_trang_thai(text) AS
  SELECT code FROM trips
  WHERE status = $1
    AND status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit')
  ORDER BY requested_at LIMIT 50;
BEGIN;
SET LOCAL plan_cache_mode = force_generic_plan;
-- expect-match: Index Scan using ix_trips_dang_chay on trips
EXPLAIN (COSTS OFF) EXECUTE dang_chay_theo_trang_thai('searching');
-- expect-rows: 50
EXECUTE dang_chay_theo_trang_thai('searching');
ROLLBACK;
DEALLOCATE dang_chay_theo_trang_thai;

-- ---------------------------------------------------------------------------
-- C. Bài tập 3 — partial UNIQUE index làm luật nghiệp vụ: "một tài xế có tối đa MỘT
--    chuyến đang chở" (accepted … in_transit). Fixture có 200 chuyến ở 4 trạng thái này
--    nhưng chỉ 197 tài xế khác nhau ⇒ dữ liệu ĐÃ vi phạm luật, tạo index sẽ lỗi.
-- expect-line: 200 | 197
SELECT count(*), count(DISTINCT driver_id) FROM trips
WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit');

-- expect-error: 23505
CREATE UNIQUE INDEX uq_trips_tai_xe_dang_cho ON trips (driver_id)
WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit');

-- Tìm các tài xế đang "chở hai chuyến".
-- expect-line: {1099751,1099860} | {accepted,arriving}
-- expect-line: {1099873,1099885} | {picked_up,picked_up}
-- expect-line: {1099890,1099909} | {arriving,picked_up}
SELECT array_agg(code ORDER BY code) AS cac_chuyen, array_agg(status ORDER BY code) AS trang_thai
FROM trips WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
GROUP BY driver_id HAVING count(*) > 1
ORDER BY 1;

-- Xử lý theo quyết định nghiệp vụ (ở lab: huỷ chuyến nhận sau; chuyến huỷ được giữ tài xế).
-- expect-rows: 3
UPDATE trips SET status = 'cancelled'
WHERE code IN (1099860, 1099885, 1099909);

CREATE UNIQUE INDEX uq_trips_tai_xe_dang_cho ON trips (driver_id)
WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit');

-- Index chỉ chứa 197 mục.
-- expect-value: 16384
SELECT pg_relation_size('uq_trips_tai_xe_dang_cho');

-- Gán thêm một chuyến cho tài xế đang chở chuyến khác ⇒ vi phạm.
SELECT driver_id AS tx_ban FROM trips WHERE status = 'in_transit' ORDER BY code LIMIT 1 \gset
-- expect-error: 23505 uq_trips_tai_xe_dang_cho
UPDATE trips SET status = 'accepted', driver_id = :'tx_ban', accepted_at = TIMESTAMPTZ '2026-10-31 00:59+07'
WHERE code = (SELECT min(code) FROM trips WHERE status = 'searching');

-- Chuyến đã HOÀN THÀNH của cùng tài xế không nằm trong index ⇒ không vướng.
-- expect-value: t
SELECT count(*) > 0 FROM trips WHERE driver_id = :'tx_ban' AND status = 'completed';

-- ---------------------------------------------------------------------------
-- D. Bài tập 1 — expression index phải khớp ĐÚNG biểu thức. Index trên ngày giờ VN;
--    câu viết bằng date_trunc (cùng nghĩa với người đọc, khác biểu thức) ⇒ không dùng được.
CREATE INDEX ix_trips_ngay_vn ON trips (((requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date));
ANALYZE trips;
-- expect-no-match: ix_trips_ngay_vn
EXPLAIN (COSTS OFF)
SELECT count(*) FROM trips
WHERE date_trunc('day', requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') = TIMESTAMP '2026-10-15 00:00';
-- expect-match: ix_trips_ngay_vn
EXPLAIN (COSTS OFF)
SELECT count(*) FROM trips
WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = DATE '2026-10-15';
-- Cùng số dòng (5.896) theo cả hai cách viết.
-- expect-line: 5896 | 5896
SELECT
  (SELECT count(*) FROM trips WHERE date_trunc('day', requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') = TIMESTAMP '2026-10-15 00:00'),
  (SELECT count(*) FROM trips WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = DATE '2026-10-15');

-- ---------------------------------------------------------------------------
-- E. Partial index làm mất HOT của cột nằm trong ĐIỀU KIỆN của nó. Bảng nhỏ fillfactor 90
--    (page còn chỗ), chỉ có index (customer_id, requested_at); đảo status completed ↔
--    cancelled của 500 dòng rải đều (mỗi dòng thứ 20) ⇒ đều HOT. Thêm partial index
--    WHERE status IN (…đang chạy…) — status KHÔNG phải cột khoá, và cả giá trị cũ lẫn mới
--    đều NẰM NGOÀI điều kiện (không dòng nào vào/ra index) — rồi đảo 500 dòng khác
--    ⇒ không lần nào HOT. (Bảng sao bằng CREATE TABLE AS không mang CHECK của trips.)
CREATE TABLE hot_trang_thai WITH (fillfactor = 90) AS
  SELECT * FROM trips WHERE code BETWEEN 600000 AND 609999;
CREATE INDEX ON hot_trang_thai (customer_id, requested_at DESC);
-- expect-rows: 500
UPDATE hot_trang_thai
SET status = CASE status WHEN 'completed' THEN 'cancelled' ELSE 'completed' END
WHERE code % 20 = 0;
CREATE INDEX ix_hot_dang_chay ON hot_trang_thai (requested_at)
WHERE status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit');
-- expect-rows: 500
UPDATE hot_trang_thai
SET status = CASE status WHEN 'completed' THEN 'cancelled' ELSE 'completed' END
WHERE code % 20 = 10;
-- Partial index vẫn rỗng (0 chuyến đang chạy trong bảng này).
-- expect-value: 8192
SELECT pg_relation_size('ix_hot_dang_chay');
SELECT pg_stat_force_next_flush();
-- expect-line: 1000 | 500
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_user_tables WHERE relid = 'hot_trang_thai'::regclass;
