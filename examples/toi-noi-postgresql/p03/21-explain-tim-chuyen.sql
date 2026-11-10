-- P03 · 21-explain-tim-chuyen.sql — PG-021 «Đọc EXPLAIN từ truy vấn tìm chuyến»
-- (`postgresql-explain-ke-hoach-thuc-thi`).
--
-- Làm được sau bài: nhận diện kiểu scan (Seq / Index / Bitmap), kiểu join (Nested
-- Loop / Hash / Merge), đọc ước lượng `rows` so với `actual rows`, phân biệt EXPLAIN
-- (chỉ lập kế hoạch) với EXPLAIN ANALYZE (CHẠY THẬT), đọc BUFFERS.
--
-- Chạy trên một database MỚI chép từ nền p03 (1 triệu chuyến, vừa VACUUM ANALYZE,
-- shared_buffers TRỐNG đối với database này). PostgreSQL 18: EXPLAIN ANALYZE tự
-- kèm BUFFERS (bản 17 trở về trước phải ghi BUFFERS). Thời gian (actual time,
-- Planning/Execution Time) được runner thay bằng T — số đo thật nằm ở p03/DO-DAC.md.
-- Con số ước lượng (cost, rows=… trong ngoặc đầu) phụ thuộc ANALYZE lấy mẫu ⇒ có thể
-- lệch vài phần trăm giữa các lần chạy; số THỰC TẾ (actual rows, Buffers) thì không.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Hai khách dùng xuyên bài: một khách cá nhân (0910000500, 40 chuyến) và khách doanh
-- nghiệp B1 (0919990001, 51.151 chuyến). \gset lưu kết quả vào biến psql, dùng lại bằng :'ten_bien'.
SELECT id AS kh_le FROM customers WHERE phone = '0910000500' \gset
SELECT id AS kh_b1 FROM customers WHERE phone = '0919990001' \gset

-- ---------------------------------------------------------------------------
-- 1. BUFFERS trên cache LẠNH rồi NÓNG. Truy vấn "lịch sử 20 chuyến gần nhất của khách".
--    Lần 1: page chưa có trong shared_buffers ⇒ `shared read` (đọc từ hệ điều hành —
--    có thể là page cache của OS chứ chưa chắc là đĩa). Lần 2: cùng page ⇒ `shared hit`.
--    Khách này có 40 chuyến nằm trên 40 page heap khác nhau ("Heap Blocks: exact=40")
--    + 4 page index = 44 page cho 20 dòng kết quả.
--    (Kế hoạch của khách này dựa trên ƯỚC LƯỢNG ~11 chuyến — từ thống kê lấy mẫu; lần
--    ANALYZE khác có thể nghiêng sang Index Scan. Vì vậy 4 kết quả của khách này được
--    đánh dấu biến động, chỉ kiểm phần chắc chắn.)
-- lab: bien-dong
-- expect-match: Buffers: shared read=4\d$
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, requested_at, status, fare
FROM trips WHERE customer_id = :'kh_le'
ORDER BY requested_at DESC LIMIT 20;

-- lab: bien-dong
-- expect-match: Buffers: shared hit=4\d$
-- expect-no-match: read=
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, requested_at, status, fare
FROM trips WHERE customer_id = :'kh_le'
ORDER BY requested_at DESC LIMIT 20;

-- ---------------------------------------------------------------------------
-- 2. EXPLAIN trần: chỉ LẬP KẾ HOẠCH, không chạy. Mỗi nút có (cost=khởi_động..tổng
--    rows=ước_lượng width=byte_mỗi_dòng). cost là đơn vị tương đối (1 = đọc tuần tự
--    một page), không phải mili giây. Planner ƯỚC ~11 chuyến cho khách này (thực tế
--    40 — giá trị không nằm trong danh sách phổ biến nên lấy trung bình, xem PG-027)
--    ⇒ chọn gom ctid bằng Bitmap rồi SẮP 11 dòng, rẻ hơn đi index theo thứ tự.
-- lab: bien-dong
-- expect-match: ^ Limit  \(cost=
-- expect-match: ->  (Bitmap Heap Scan on trips|Index Scan using ix_trips_customer_requested on trips)  \(cost=
EXPLAIN
SELECT code, requested_at, status, fare
FROM trips WHERE customer_id = :'kh_le'
ORDER BY requested_at DESC LIMIT 20;

-- COSTS OFF bỏ phần số để so sánh HÌNH DẠNG kế hoạch (cây đọc từ trong ra ngoài,
-- từ dưới lên: nút con thụt vào sau "->", chạy trước và đưa dòng lên nút cha).
-- lab: bien-dong
-- expect-match: ^ Limit$
-- expect-match: Index Cond: \(customer_id = '3b3bf766-29e0-4cca-8a69-7b8beba432de'::uuid\)
EXPLAIN (COSTS OFF)
SELECT code, requested_at, status, fare
FROM trips WHERE customer_id = :'kh_le'
ORDER BY requested_at DESC LIMIT 20;

-- CÙNG câu lệnh, khách B1 (51.151 chuyến): planner biết B1 rất đông (B1 nằm trong
-- danh sách giá trị phổ biến) ⇒ đi index (customer_id, requested_at DESC) theo đúng
-- thứ tự cần, dừng sau 20 dòng — không cần sắp. Cùng SQL, khác tham số, khác kế hoạch.
-- expect-line: Limit
-- expect-line: ->  Index Scan using ix_trips_customer_requested on trips
-- expect-line: Index Cond: (customer_id = '239ec4d4-ca1d-4ae9-8459-a2f5227a39c1'::uuid)
EXPLAIN (COSTS OFF)
SELECT code, requested_at, status, fare
FROM trips WHERE customer_id = :'kh_b1'
ORDER BY requested_at DESC LIMIT 20;

-- Đọc 10 page cho 20 dòng (chuyến gần nhất của B1 nằm sát nhau ở cuối heap).
-- expect-match: Buffers: shared (hit=\d+ )?read=\d+
-- expect-match: ->  Index Scan using ix_trips_customer_requested on trips \(actual rows=20\.00 loops=1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, requested_at, status, fare
FROM trips WHERE customer_id = :'kh_b1'
ORDER BY requested_at DESC LIMIT 20;

-- SETTINGS: in các tham số planner KHÁC mặc định — ghi kèm khi chia sẻ một kế hoạch.
-- expect-match: Settings: .*jit = 'off'.*max_parallel_workers_per_gather = '0'
EXPLAIN (COSTS OFF, SETTINGS)
SELECT count(*) FROM trips WHERE fare >= 150000;

-- ---------------------------------------------------------------------------
-- 3. Ba kiểu đọc bảng.
-- 3a. Seq Scan: không có index trên fare ⇒ đọc MỌI page, lọc từng dòng.
--     "Rows Removed by Filter" = số dòng đọc lên rồi bỏ đi.
--     Bảng lớn hơn 1/4 shared_buffers ⇒ Seq Scan dùng một VÒNG ĐỆM nhỏ (256 kB) để
--     không đẩy hết dữ liệu khác ra khỏi cache — nên chạy lại vẫn thấy `read`.
-- expect-match: ->  Seq Scan on trips \(actual rows=3541\.00 loops=1\)
-- expect-match: Filter: \(fare >= 150000\)
-- expect-match: Rows Removed by Filter: 996459
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips WHERE fare >= 150000;

-- expect-match: Buffers: shared (hit=\d+ )?read=\d+
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips WHERE fare >= 150000;

-- 3b. Index Scan: dải mã chuyến nhỏ — đi cây uq_trips_code rồi đọc heap theo từng ctid.
-- expect-line: Index Scan using uq_trips_code on trips
-- expect-line: Index Cond: ((code >= 600000) AND (code <= 600050))
EXPLAIN (COSTS OFF)
SELECT code, status, fare FROM trips WHERE code BETWEEN 600000 AND 600050;

-- 3c. Bitmap Heap Scan: chuyến Đà Nẵng trong 7 ngày (~ vài nghìn dòng rải trên hàng
--     nghìn page). Bước 1 gom MỌI ctid từ index thành bitmap theo page; bước 2 đọc mỗi
--     page heap MỘT lần theo thứ tự page. "Heap Blocks: exact=N" = số page heap đã đọc.
-- expect-match: Bitmap Heap Scan on trips \(actual rows=\d+\.00 loops=1\)
-- expect-match: ->  Bitmap Index Scan on ix_trips_city_requested
-- expect-match: Heap Blocks: exact=\d+
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*), sum(fare) FROM trips
WHERE city_id = 3 AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07' AND requested_at < TIMESTAMPTZ '2026-10-08 00:00+07';

-- ---------------------------------------------------------------------------
-- 4. Ba kiểu join.
-- 4a. Nested Loop: ít dòng bên ngoài (chuyến của MỘT khách), mỗi dòng tra index
--     khoá chính của drivers. "loops=N" ở nút bên trong = số lần tra.
-- lab: bien-dong
-- expect-match: Nested Loop
-- expect-match: ->  Index Scan using pk_drivers on drivers d
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT t.code, t.requested_at, d.code AS tai_xe
FROM trips AS t JOIN drivers AS d ON d.id = t.driver_id
WHERE t.customer_id = :'kh_le';

-- 4b. Hash Join: nhiều dòng hai phía — dựng bảng băm từ phía NHỎ (6.000 tài xế),
--     quét phía lớn và dò bảng băm. "Buckets/Batches/Memory Usage" ở nút Hash.
-- expect-match: Hash Join
-- expect-match: Hash Cond: \(t\.driver_id = d\.id\)
-- expect-match: Batches: 1  Memory Usage: \d+kB
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT d.vehicle_type, count(*), sum(t.fare)
FROM trips AS t JOIN drivers AS d ON d.id = t.driver_id
WHERE t.requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07' AND t.requested_at < TIMESTAMPTZ '2026-10-16 00:00+07'
GROUP BY d.vehicle_type;

-- 4c. Merge Join: hai đầu vào ĐÃ SẮP theo khoá join — trộn như kéo khoá kéo. Câu
--     dưới cần MỌI cặp chuyến–thanh toán theo thứ tự trips.id: pk_trips (Index Only
--     Scan) và ix_payments_trip đều trả dòng theo đúng thứ tự đó ⇒ không phải sắp,
--     không phải dựng bảng băm. Chỉ EXPLAIN (câu trả ~900.000 dòng).
-- expect-line: Merge Join
-- expect-line: Merge Cond: (t.id = p.trip_id)
-- expect-line: ->  Index Only Scan using pk_trips on trips t
-- expect-line: ->  Index Scan using ix_payments_trip on payments p
EXPLAIN (COSTS OFF)
SELECT t.id, p.amount
FROM trips AS t JOIN payments AS p ON p.trip_id = t.id
ORDER BY t.id;

-- Cùng câu, cấm Merge Join trong một giao dịch (SET LOCAL hết hiệu lực khi giao dịch
-- kết thúc) ⇒ planner phải dựng bảng băm rồi SẮP lại 900.000 dòng. Chỉ để HỌC —
-- đừng tắt planner trên production.
BEGIN;
SET LOCAL enable_mergejoin = off;
-- expect-match: Hash Join
-- expect-match: Sort Key: t\.id
EXPLAIN (COSTS OFF)
SELECT t.id, p.amount
FROM trips AS t JOIN payments AS p ON p.trip_id = t.id
ORDER BY t.id;
ROLLBACK;

-- Bên lề (PostgreSQL 18): tự join một bảng với chính nó theo khoá UNIQUE là thừa —
-- planner 18 BỎ HẲN phép join (self-join elimination), chỉ còn một lần quét.
-- expect-line: Aggregate
-- expect-line: ->  Seq Scan on trips b
EXPLAIN (COSTS OFF)
SELECT count(*) FROM trips AS a JOIN trips AS b ON b.id = a.id;

-- ---------------------------------------------------------------------------
-- 5. Ước lượng (rows=… trong ngoặc đầu) so với thực tế (actual rows=…).
--    Chuyến Đà Nẵng (city_id = 3) đón ở khu 311 (khu 311 thuộc Đà Nẵng). Planner coi
--    hai điều kiện ĐỘC LẬP ⇒ nhân hai xác suất ⇒ ước lượng THẤP hơn thực tế ~7 lần
--    (1/13 % chuyến ở Đà Nẵng). PG-027 sửa bằng CREATE STATISTICS.
-- expect-match: Bitmap Heap Scan on trips|Seq Scan on trips
EXPLAIN (ANALYZE, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips WHERE city_id = 3 AND pickup_area_id = 311;

-- Kiểm có chặn khoảng (không phụ thuộc số ước lượng chính xác): lệch từ 5 tới 10 lần.
-- uoc_luong() là hàm tiện ích của nền P03 (00-mo-rong-schema.sql).
-- expect-range: 5 10
SELECT lech FROM uoc_luong($$SELECT * FROM trips WHERE city_id = 3 AND pickup_area_id = 311$$);

-- Số thực tế (tất định).
-- expect-value: 22222
SELECT thuc_te FROM uoc_luong($$SELECT * FROM trips WHERE city_id = 3 AND pickup_area_id = 311$$);

-- ---------------------------------------------------------------------------
-- 6. EXPLAIN ANALYZE CHẠY THẬT câu lệnh. Với câu GHI, kết quả ghi là thật.
CREATE TEMP TABLE thu_explain AS SELECT code, fare FROM trips WHERE code < 100100;

-- EXPLAIN không ANALYZE: chỉ in kế hoạch, KHÔNG xoá dòng nào.
EXPLAIN (COSTS OFF) DELETE FROM thu_explain WHERE fare > 50000;

-- expect-value: 100
SELECT count(*) FROM thu_explain;

-- EXPLAIN ANALYZE: câu DELETE đã chạy thật ("actual rows" là số dòng đã xoá).
-- expect-match: Delete on thu_explain \(actual rows=0\.00 loops=1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF) DELETE FROM thu_explain WHERE fare > 50000;

-- Số dòng đã giảm: phân tích một câu ghi = THỰC HIỆN câu ghi đó.
-- expect-value: 32
SELECT count(*) FROM thu_explain;

-- Cách an toàn: bọc trong giao dịch rồi ROLLBACK.
-- expect-value: 31000
SELECT fare FROM trips WHERE code = 600000;

BEGIN;
-- expect-match: Update on trips
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
UPDATE trips SET fare = fare + 1000 WHERE code = 600000;
ROLLBACK;

-- Giá vẫn như cũ sau ROLLBACK.
-- expect-value: 31000
SELECT fare FROM trips WHERE code = 600000;

-- ---------------------------------------------------------------------------
-- 7. Đọc nhanh các dòng đáng chú ý của một kế hoạch đầy đủ (VERBOSE thêm tên cột
--    đầu ra và schema). "Planning:" có Buffers riêng — page catalog đọc lúc lập kế hoạch.
-- expect-match: Output: code, requested_at
-- expect-match: Query Identifier: Q
EXPLAIN (ANALYZE, VERBOSE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT t.code, t.requested_at
FROM trips AS t
WHERE t.customer_id = :'kh_b1' AND t.requested_at >= TIMESTAMPTZ '2026-10-30 00:00+07'
ORDER BY t.requested_at DESC;
