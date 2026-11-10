-- P03 · 24-heap-ctid-hot.sql — PG-024 «Clustered index thật và clustered index nhìn
-- thấy trong PostgreSQL» (`heap-ctid-va-primary-key`, BÀI ĐÃ CÓ — chờ rà soát).
--
-- Mục đích: TÁI LẬP các khẳng định và kết quả của bài cũ trên schema bai_cu (lược đồ
-- lõi của hồ sơ + 20.000 chuyến, p03/03-bai-cu-20k.sql — bảng nhỏ hơn 30.000 dòng nên
-- thống kê ANALYZE tất định), để người viết đối chiếu:
--   heap + ctid · ctid đổi sau UPDATE · HOT update (điều kiện: không đổi cột có index
--   VÀ page còn chỗ) · fillfactor · primary key chỉ là B-tree + UNIQUE + NOT NULL ·
--   CLUSTER chỉ sắp MỘT lần, không duy trì.
-- Bảng "bài cũ nói — lab đo" ở p03/README.md (mục PG-024).

SET search_path TO bai_cu, toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. Thứ tự VẬT LÝ (không ORDER BY — đọc heap từ page 0): ctid (0,1), (0,2)… và mã
--    chuyến tăng dần — vì chuyến được chèn theo thời gian.
-- expect-line: (0,1) | 100000 | completed
-- expect-line: (0,2) | 100001 | completed
-- expect-line: (0,3) | 100002 | completed
-- expect-line: (0,4) | 100003 | completed
-- expect-line: (0,5) | 100004 | cancelled
SELECT ctid, code, status FROM trips LIMIT 5;

-- Thứ tự PRIMARY KEY (ORDER BY id): ctid nhảy khắp tệp heap.
SELECT ctid, left(id::text, 8) AS id, code FROM trips ORDER BY id LIMIT 5;

-- Bao nhiêu page heap khác nhau chứa 5 dòng đầu theo id? (rải rác)
-- expect-value: 5
SELECT count(DISTINCT (ctid::text::point)[0]) FROM (SELECT ctid FROM trips ORDER BY id LIMIT 5) AS x;

-- ---------------------------------------------------------------------------
-- 2. Tìm theo primary key hay theo index thường: cùng một đường — đi cây B-tree lấy
--    ctid rồi đọc page heap. Bài cũ: Index Scan using uq_trips_code … rows=1 width=125.
-- expect-match: Index Scan using uq_trips_code on trips  \(cost=[0-9.]+\.\.[0-9.]+ rows=1 width=125\)
EXPLAIN SELECT * FROM trips WHERE code = 150000;

-- expect-line: Index Scan using trips_pkey on trips
-- expect-line: Index Cond: (id = 'ab31fe65-42a9-425f-8abd-48b358eb8098'::uuid)
EXPLAIN (COSTS OFF) SELECT * FROM trips WHERE id = 'ab31fe65-42a9-425f-8abd-48b358eb8098';

-- Nhiều dòng khớp ⇒ Bitmap Heap Scan (bài cũ dùng now() - 1 ngày; lab cố định mốc chụp).
-- Bài cũ ước 111 dòng; dữ liệu 20.000 chuyến/30 ngày/6 thành phố ⇒ ~111 chuyến/ngày/TP.
-- expect-match: Bitmap Heap Scan on trips  \(cost=[0-9.]+\.\.[0-9.]+ rows=11[0-9] width=125\)
-- expect-match: ->  Bitmap Index Scan on ix_trips_city_requested
EXPLAIN SELECT * FROM trips
WHERE city_id = 1 AND requested_at >= TIMESTAMPTZ '2026-10-31 01:00+07' - interval '1 day';

-- Tra thẳng theo địa chỉ vật lý ⇒ Tid Scan (chỉ để chẩn đoán).
-- expect-line: Tid Scan on trips
-- expect-line: TID Cond: (ctid = '(0,1)'::tid)
EXPLAIN (COSTS OFF) SELECT * FROM trips WHERE ctid = '(0,1)';

-- ---------------------------------------------------------------------------
-- 3. Heap đang thẳng hàng với cột nào? (bảng 20.000 dòng ⇒ ANALYZE đọc mọi dòng ⇒
--    số dưới TẤT ĐỊNH.) Bài cũ: requested_at 1,000 · code 1,000 · city_id 0,167 ·
--    fare 0,014 · driver_id 0,007 · id −0,005 · customer_id 0,000.
-- expect-line: code         | 1.000
-- expect-line: requested_at | 1.000
-- expect-line: city_id      | 0.167
-- expect-line: fare         | 0.014
-- expect-line: id           | 0.009
-- expect-line: customer_id  | -0.007
-- expect-line: driver_id    | -0.001
SELECT attname, round(correlation::numeric, 3) AS correlation
FROM pg_stats
WHERE schemaname = 'bai_cu' AND tablename = 'trips'
  AND attname IN ('requested_at', 'code', 'city_id', 'fare', 'driver_id', 'id', 'customer_id')
ORDER BY abs(correlation) DESC, attname;

-- ---------------------------------------------------------------------------
-- 4. id không đổi, ctid thì đổi. Ba UPDATE trong MỘT giao dịch, đếm bằng
--    pg_stat_xact_user_tables (thống kê của riêng giao dịch đang chạy).
--    Chuyến 119985 đang 'searching' (nằm ở page cuối 373, còn chỗ trống).
--    (Bài cũ: (373,14) → (373,34) → (373,35); chuyến cũ dời sang (373,36). Số ô khác
--    vì dữ liệu khác, HÌNH DẠNG giống: ① ② cùng page, ③ dời từ page 186 sang 373.)
-- expect-line: 119985 | (373,32) | searching
SELECT code, ctid, status FROM trips WHERE code = 119985;

BEGIN;

-- ① Nhận tài xế: đổi driver_id — cột CÓ index (ix_trips_driver_id) ⇒ KHÔNG thể HOT.
-- expect-line: 119985 | (373,47)
UPDATE trips
SET driver_id = (SELECT id FROM drivers WHERE code = 8001), status = 'accepted',
    accepted_at = requested_at + interval '1 minute', updated_at = requested_at + interval '1 minute'
WHERE code = 119985
RETURNING code, ctid;

-- expect-line: 1 | 0
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_xact_user_tables WHERE relid = 'bai_cu.trips'::regclass;

-- ② Sửa giá ngay sau đó: fare KHÔNG có index, page 373 còn chỗ ⇒ HOT: phiên bản mới
--    cùng page, không index nào phải ghi thêm.
-- expect-line: 119985 | (373,48)
UPDATE trips SET fare = fare + 5000 WHERE code = 119985 RETURNING code, ctid;

-- expect-line: 2 | 1
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_xact_user_tables WHERE relid = 'bai_cu.trips'::regclass;

-- ③ Sửa giá một chuyến CŨ trên page 186 đã đầy (fillfactor mặc định 100): không đổi
--    cột có index nhưng page hết chỗ ⇒ phiên bản mới DỜI sang page khác ⇒ KHÔNG HOT.
-- expect-line: 110000 | (186,54)
SELECT code, ctid FROM trips WHERE code = 110000;

-- expect-line: 110000 | (373,49)
UPDATE trips SET fare = fare + 1000 WHERE code = 110000 RETURNING code, ctid;

-- expect-line: 3 | 1
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_xact_user_tables WHERE relid = 'bai_cu.trips'::regclass;

COMMIT;

-- Phiên bản cũ vẫn nằm trong page (đã "chết", chờ VACUUM): soi page 373 bằng
-- pageinspect — t_ctid của phiên bản cũ trỏ tới phiên bản mới (chuỗi phiên bản).
CREATE EXTENSION pageinspect WITH SCHEMA public;
-- Ô 32 = phiên bản gốc (trỏ sang 47, không HOT vì ① đổi driver_id); ô 47 = sau ①
-- (trỏ sang 48, ĐÃ cập nhật kiểu HOT ở ②); ô 48 = phiên bản hiện hành (trỏ chính nó).
-- expect-line: 32 | (373,47) | f
-- expect-line: 47 | (373,48) | t
-- expect-line: 48 | (373,48) | f
SELECT lp, t_ctid, (t_infomask2 & 16384) <> 0 AS hot_updated
FROM public.heap_page_items(public.get_raw_page('bai_cu.trips', 373))
WHERE lp IN (32, 47, 48) ORDER BY lp;

-- Tỉ lệ HOT trên bảng (câu của bài cũ). Thống kê tích luỹ được đẩy định kỳ ⇒ ép đẩy trước.
SELECT pg_stat_force_next_flush();
-- expect-line: 3 | 1 | 33.3
SELECT n_tup_upd, n_tup_hot_upd,
       round(100.0 * n_tup_hot_upd / nullif(n_tup_upd, 0), 1) AS hot_pct
FROM pg_stat_user_tables
WHERE relid = 'bai_cu.trips'::regclass;

-- ---------------------------------------------------------------------------
-- 5. fillfactor: chừa chỗ trong page cho HOT. Hai bản sao 20.000 chuyến, fillfactor 100
--    và 90; sửa giá 2.000 chuyến cũ rải đều (mỗi chuyến thứ 10) trên cả hai.
CREATE TABLE ff100 WITH (fillfactor = 100) AS SELECT * FROM trips ORDER BY code;
CREATE TABLE ff90  WITH (fillfactor = 90)  AS SELECT * FROM trips ORDER BY code;
CREATE INDEX ON ff100 (driver_id);
CREATE INDEX ON ff90 (driver_id);

-- (CREATE TABLE AS ghi bằng đường chèn hàng loạt nên ra 384 page chứ không đúng 374.)
-- expect-line: ff100 | 384
-- expect-line: ff90  | 448
SELECT relname, pg_relation_size(oid) / 8192 AS so_page FROM pg_class
WHERE oid IN ('bai_cu.ff100'::regclass, 'bai_cu.ff90'::regclass) ORDER BY relname;

UPDATE ff100 SET fare = fare + 1000 WHERE code % 10 = 0;
UPDATE ff90  SET fare = fare + 1000 WHERE code % 10 = 0;
SELECT pg_stat_force_next_flush();

-- MỌI UPDATE trên ff90 là HOT; trên ff100 chỉ 21/2.000 (page đầy, phải dời page).
-- expect-line: ff100 | 2000 | 21   | 1.1
-- expect-line: ff90  | 2000 | 2000 | 100.0
SELECT relname, n_tup_upd, n_tup_hot_upd, round(100.0 * n_tup_hot_upd / n_tup_upd, 1) AS hot_pct
FROM pg_stat_user_tables WHERE relid IN ('bai_cu.ff100'::regclass, 'bai_cu.ff90'::regclass) ORDER BY relname;

-- ---------------------------------------------------------------------------
-- 6. Primary key = một index B-tree UNIQUE + ràng buộc NOT NULL, đánh dấu "primary".
--    Về cấu trúc, trips_pkey ngang hàng uq_trips_code.
-- expect-line: trips_pkey    | btree | t | t | p
-- expect-line: uq_trips_code | btree | t | f | u
SELECT i.indexrelid::regclass AS index, am.amname, i.indisunique, i.indisprimary, c.contype
FROM pg_index AS i
JOIN pg_class AS ic ON ic.oid = i.indexrelid
JOIN pg_am AS am ON am.oid = ic.relam
JOIN pg_constraint AS c ON c.conindid = i.indexrelid
WHERE i.indrelid = 'bai_cu.trips'::regclass AND c.contype IN ('p', 'u')
ORDER BY 1::text;

-- Khoá ngoại tham chiếu được cột UNIQUE như tham chiếu khoá chính.
CREATE TABLE ghi_chu_chuyen (trip_code bigint NOT NULL REFERENCES trips (code), noi_dung text NOT NULL);
-- expect-error: 23503 ghi_chu_chuyen_trip_code_fkey
INSERT INTO ghi_chu_chuyen VALUES (999999, 'mã không tồn tại');

-- Năm index "đè" lên heap: tổng kích thước index so với bảng (bài cũ: 3,4 MB vs 3,0 MB;
-- 374 page × 8.192 byte = 2,92 MiB nên "3,0 MB" của bài cũ là làm tròn lên).
-- expect-line: 2.9 | 3.3
SELECT round(pg_relation_size('bai_cu.trips') / 1048576.0, 1) AS bang_mb,
       round(pg_indexes_size('bai_cu.trips') / 1048576.0, 1) AS index_mb;

-- ---------------------------------------------------------------------------
-- 7. CLUSTER: chép lại heap theo thứ tự một index — MỘT lần. Giữ khoá ACCESS
--    EXCLUSIVE suốt lúc chạy (không ai đọc được bảng, kể cả SELECT).
BEGIN;
CLUSTER trips USING trips_pkey;
-- (ShareLock đi kèm là khoá phụ trong lúc dựng lại index; khoá quyết định là ACCESS EXCLUSIVE.)
-- expect-line: AccessExclusiveLock
-- expect-line: ShareLock
SELECT mode FROM pg_locks WHERE relation = 'bai_cu.trips'::regclass AND pid = pg_backend_pid() ORDER BY mode;
COMMIT;
ANALYZE trips;

-- Sau CLUSTER: heap thẳng hàng với id, MẤT thẳng hàng với thời gian đặt. Mọi ctid đã đổi.
-- expect-line: id           | 1.000
-- expect-line: code         | 0.009
-- expect-line: requested_at | 0.009
SELECT attname, round(correlation::numeric, 3) AS correlation
FROM pg_stats WHERE schemaname = 'bai_cu' AND tablename = 'trips' AND attname IN ('id', 'code', 'requested_at')
ORDER BY abs(correlation) DESC, attname;

-- PostgreSQL chỉ GHI NHỚ index đã dùng (để lần sau gõ `CLUSTER trips` không cần USING).
-- expect-line: trips_pkey | t
SELECT indexrelid::regclass, indisclustered FROM pg_index WHERE indrelid = 'bai_cu.trips'::regclass AND indisclustered;

-- … nhưng KHÔNG duy trì thứ tự: 500 chuyến mới (id ngẫu nhiên) đều nằm ở CUỐI heap
-- (page 373 còn chỗ rồi tới các page mới 374–380), không chen vào đúng chỗ theo id.
-- expect-rows: 500
INSERT INTO trips (id, kind, city_id, customer_id, status, fare, requested_at, updated_at)
SELECT p03_uuid('bc-moi', g), 'ride', 1, (SELECT id FROM customers ORDER BY phone LIMIT 1), 'searching', 30000,
       TIMESTAMPTZ '2026-10-31 01:00+07' + g * interval '1 second', TIMESTAMPTZ '2026-10-31 01:00+07' + g * interval '1 second'
FROM generate_series(1, 500) AS g;

-- expect-line: 373 | 380
SELECT min((ctid::text::point)[0])::int AS page_dau, max((ctid::text::point)[0])::int AS page_cuoi
FROM trips WHERE requested_at > TIMESTAMPTZ '2026-10-31 01:00+07';

-- Correlation của id bắt đầu tụt (1,000 → ~0,95) — càng ghi nhiều càng tụt.
ANALYZE trips;
-- expect-range: 0.9 0.99
SELECT round(correlation::numeric, 3) FROM pg_stats WHERE schemaname = 'bai_cu' AND tablename = 'trips' AND attname = 'id';
