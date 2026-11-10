-- P03 · 25-index-nhieu-cot-partial-expression.sql — PG-025 «Index nhiều cột, partial
-- và expression» (`postgresql-index-nhieu-cot-partial-expression`).
--
-- Làm được sau bài: chọn index theo ĐIỀU KIỆN TRUY VẤN (thứ tự cột, điều kiện con,
-- biểu thức) và theo CHI PHÍ GHI (mỗi index thêm WAL và dung lượng cho mọi INSERT/UPDATE).
-- Thời gian ghi đo lặp ở p03/DO-DAC.md (PG-025); ở đây dùng số đo TẤT ĐỊNH: số bản
-- ghi WAL, số byte WAL (EXPLAIN (ANALYZE, WAL)), kích thước index.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. Index nhiều cột: THỨ TỰ cột quyết định index giúp được câu nào.
--    Nền đã có ix_trips_city_requested (city_id, requested_at).
-- 1a. Bằng ở cột đầu + khoảng ở cột sau: một lần dò cây, đọc một dải lá liền nhau.
--     (cache lạnh: 10 page — 768 mục nằm gọn trong vài page lá.)
-- expect-match: Index Searches: 1$
-- expect-match: Buffers: shared hit=3 read=7$
-- expect-match: actual rows=768\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips
WHERE city_id = 3 AND requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-16 00:00+07';

-- 1b. CHỈ có điều kiện ở cột SAU (mọi thành phố trong một ngày). PostgreSQL 18 có
--     "skip scan": dò cây một lần cho TỪNG giá trị của cột đầu (city_id có 3 giá trị)
--     — thấy ở "Index Searches" > 1. Bản 17 trở về trước không làm được như vậy.
-- expect-match: Index Cond: \(\(requested_at >= .* AND \(requested_at < .*\)\)
-- expect-match: Index Searches: [3-9]$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips
WHERE requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-16 00:00+07';

-- 1c. Đảo thứ tự cột: (requested_at, city_id). Câu 1a giờ phải đi qua MỌI mục của
--     ngày đó (cả 3 thành phố, 5.896 mục) rồi lọc city_id ngay trong index ⇒ 41 page
--     thay vì 10 cho cùng 768 dòng kết quả.
CREATE INDEX ix_trips_requested_city ON trips (requested_at, city_id);

BEGIN;
-- Tạm ẩn index (city_id, requested_at) bằng cách xoá trong giao dịch rồi ROLLBACK.
DROP INDEX ix_trips_city_requested;
-- expect-match: Index Only Scan using ix_trips_requested_city on trips
-- expect-match: actual rows=768\.00
-- expect-match: Buffers: shared hit=16 read=25$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips
WHERE city_id = 3 AND requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-16 00:00+07';
ROLLBACK;

-- So số page đọc của hai index cho cùng câu 1a (Index Only Scan, cache nóng).
-- expect-match: Index Only Scan using ix_trips_city_requested on trips
-- expect-match: Buffers: shared hit=\d+$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips
WHERE city_id = 3 AND requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-16 00:00+07';

-- ---------------------------------------------------------------------------
-- 2. Partial index: chỉ đánh index phần dữ liệu hay hỏi. Màn điều phối hỏi liên tục
--    "chuyến đang chạy" — 300 trong 1.000.000 chuyến.
CREATE INDEX ix_trips_dang_chay ON trips (requested_at)
WHERE status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit');
CREATE INDEX ix_trips_status ON trips (status);

-- Partial index chỉ chứa 300 mục ⇒ 2 page; index đầy đủ trên status: 1 triệu mục.
-- expect-line: ix_trips_dang_chay | 16384
-- expect-line: ix_trips_status    | 7069696
SELECT relname, pg_relation_size(oid) AS bytes FROM pg_class
WHERE relname IN ('ix_trips_dang_chay', 'ix_trips_status') AND relnamespace = 'toinoi'::regnamespace
ORDER BY relname;

-- Câu khớp ĐÚNG điều kiện của index ⇒ dùng partial index.
-- expect-line: Index Scan using ix_trips_dang_chay on trips
EXPLAIN (COSTS OFF)
SELECT code, status FROM trips
WHERE status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit')
ORDER BY requested_at;

-- Điều kiện HẸP hơn mà planner chứng minh được là KÉO THEO điều kiện của index
-- (status = 'searching' ⇒ status IN (…)) ⇒ vẫn dùng được.
-- expect-match: ix_trips_dang_chay
EXPLAIN (COSTS OFF)
SELECT code FROM trips WHERE status = 'searching' ORDER BY requested_at;

-- Điều kiện không kéo theo ⇒ không dùng được partial index.
-- expect-no-match: ix_trips_dang_chay
EXPLAIN (COSTS OFF)
SELECT count(*) FROM trips WHERE status = 'cancelled' AND requested_at >= TIMESTAMPTZ '2026-10-30 00:00+07';

-- BẪY: câu có THAM SỐ ($1). Với kế hoạch chung (generic plan) planner không biết $1
-- lúc lập kế hoạch ⇒ không chứng minh được $1 nằm trong danh sách ⇒ bỏ partial index.
PREPARE chuyen_theo_trang_thai(text) AS
  SELECT code FROM trips WHERE status = $1 ORDER BY requested_at LIMIT 50;
BEGIN;
SET LOCAL plan_cache_mode = force_generic_plan;
-- expect-no-match: ix_trips_dang_chay
EXPLAIN (COSTS OFF) EXECUTE chuyen_theo_trang_thai('searching');
SET LOCAL plan_cache_mode = force_custom_plan;
-- expect-match: ix_trips_dang_chay
EXPLAIN (COSTS OFF) EXECUTE chuyen_theo_trang_thai('searching');
ROLLBACK;
DEALLOCATE chuyen_theo_trang_thai;

-- ---------------------------------------------------------------------------
-- 3. Expression index: "chuyến của NGÀY 15/10 theo giờ Việt Nam" viết bằng biểu thức
--    trên cột ⇒ index thường trên requested_at KHÔNG dùng được cho biểu thức đó.
-- expect-match: Seq Scan on trips
-- expect-match: actual rows=5896\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = DATE '2026-10-15';

-- Biểu thức phụ thuộc thiết lập phiên (requested_at::date đổi theo TimeZone của phiên)
-- KHÔNG được phép làm index: hàm/phép ép kiểu phải IMMUTABLE.
-- expect-error: 42P17
CREATE INDEX ix_trips_ngay_sai ON trips ((requested_at::date));

-- AT TIME ZONE với tên múi giờ CỐ ĐỊNH là IMMUTABLE ⇒ index được.
CREATE INDEX ix_trips_ngay_vn ON trips (((requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date));
-- Thống kê cho BIỂU THỨC chỉ có sau ANALYZE (ước lượng dòng chính xác hơn).
ANALYZE trips;

-- Câu viết ĐÚNG biểu thức của index ⇒ dùng index.
-- expect-match: ix_trips_ngay_vn
-- expect-match: actual rows=5896\.00
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = DATE '2026-10-15';

-- Cách viết khác mà KẾT QUẢ như nhau (khoảng nửa mở trên cột gốc) dùng index thường
-- trên cột — thường là lựa chọn tốt hơn: không cần thêm index nào (PG-016).
-- expect-match: ix_trips_city_requested|ix_trips_requested_city
EXPLAIN (COSTS OFF)
SELECT * FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-16 00:00+07';

-- ---------------------------------------------------------------------------
-- 4. CHI PHÍ GHI của index. Chép 50.000 chuyến (mã 600000–649999) vào một bảng trống
--    với số index khác nhau; đo WAL (số bản ghi, số byte) của đúng câu INSERT bằng
--    EXPLAIN (ANALYZE, WAL). CHECKPOINT trước mỗi lần để số "ảnh page" (fpi) so được.
CREATE TABLE ghi_thu (LIKE trips INCLUDING DEFAULTS);
ALTER TABLE ghi_thu ALTER COLUMN code DROP IDENTITY IF EXISTS;

-- Kết quả (bản ghi WAL · byte WAL): 0 index 50.001 · ~9,0 MB; 5 index 302.054 · ~29,8 MB;
-- 9 index 453.320 · ~40,5 MB. "buffers full" (mới ở PG 18) = số lần phải đẩy WAL ra
-- đĩa vì bộ đệm WAL đầy — phụ thuộc thời điểm, đừng so giữa các lần chạy.
-- 4a. Không index nào.
CHECKPOINT;
-- expect-match: WAL: records=50001 fpi=\d+ bytes=\d+
EXPLAIN (ANALYZE, WAL, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
INSERT INTO ghi_thu SELECT * FROM trips WHERE code BETWEEN 600000 AND 649999;

-- 4b. Như trips của nền: khoá chính (uuid ngẫu nhiên) + mã + 3 index phụ = 5 index.
TRUNCATE ghi_thu;
ALTER TABLE ghi_thu ADD CONSTRAINT pk_ghi_thu PRIMARY KEY (id);
ALTER TABLE ghi_thu ADD CONSTRAINT uq_ghi_thu_code UNIQUE (code);
CREATE INDEX ON ghi_thu (city_id, requested_at);
CREATE INDEX ON ghi_thu (customer_id, requested_at DESC);
CREATE INDEX ON ghi_thu (driver_id);
CHECKPOINT;
-- expect-match: WAL: records=302054 fpi=\d+ bytes=\d+
EXPLAIN (ANALYZE, WAL, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
INSERT INTO ghi_thu SELECT * FROM trips WHERE code BETWEEN 600000 AND 649999;

-- 4c. Thêm 4 index của bài này (đảo cột, partial, status, expression) = 9 index.
TRUNCATE ghi_thu;
CREATE INDEX ON ghi_thu (requested_at, city_id);
CREATE INDEX ON ghi_thu (requested_at)
  WHERE status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit');
CREATE INDEX ON ghi_thu (status);
CREATE INDEX ON ghi_thu (((requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date));
CHECKPOINT;
-- expect-match: WAL: records=453320 fpi=\d+ bytes=\d+
EXPLAIN (ANALYZE, WAL, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
INSERT INTO ghi_thu SELECT * FROM trips WHERE code BETWEEN 600000 AND 649999;

-- Dung lượng sau 50.000 dòng: bảng và từng index (partial index gần như không tốn gì
-- vì 50.000 chuyến cũ đều đã kết thúc).
SELECT c.relname, pg_relation_size(c.oid) AS bytes
FROM pg_class AS c
WHERE c.oid = 'ghi_thu'::regclass
   OR c.oid IN (SELECT indexrelid FROM pg_index WHERE indrelid = 'ghi_thu'::regclass)
ORDER BY c.relkind DESC, bytes DESC, c.relname;

-- 4d. UPDATE cũng trả giá cho index: sửa giá 10.000 chuyến. fare KHÔNG có index,
--     nhưng page đầy (fillfactor 100) ⇒ phiên bản mới dời page ⇒ ghi vào CẢ 9 index
--     (chỉ 4/10.000 là HOT). ~102.000 bản ghi WAL, ~15,9 MB cho 10.000 dòng.
CHECKPOINT;
-- expect-match: WAL: records=\d+ fpi=\d+ bytes=\d+
EXPLAIN (ANALYZE, WAL, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
UPDATE ghi_thu SET fare = fare + 1000 WHERE code BETWEEN 600000 AND 609999;

SELECT pg_stat_force_next_flush();
-- expect-line: 10000 | 4
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_user_tables WHERE relid = 'ghi_thu'::regclass;
