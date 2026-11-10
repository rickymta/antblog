-- Thí nghiệm PG-024 «Clustered index thật và clustered index nhìn thấy trong
-- PostgreSQL» — lời giải bài tập. Không có trong lab chính (24-heap-ctid-hot.sql).
-- Chạy trên bộ dữ liệu bài cũ (schema bai_cu, 20.000 chuyến — p03/03-bai-cu-20k.sql).
-- Đầu tệp lặp lại đúng ba UPDATE ①②③ của 24-heap-ctid-hot.sql để page 373 ở cùng
-- trạng thái như trong bài. Hai phiên: A làm mọi việc; B (kết nối MỚI) làm bài tập 1 —
-- pg_stat_xact_user_tables của phiên A còn giữ số đếm chưa đẩy của giao dịch ①②③.

-- @A
CREATE EXTENSION pageinspect WITH SCHEMA public;
SET search_path TO bai_cu, toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ①②③ như lab chính.
-- expect-line: (373,32)
SELECT ctid FROM trips WHERE code = 119985;
BEGIN;
-- expect-line: (373,47)
UPDATE trips
SET driver_id = (SELECT id FROM drivers WHERE code = 8001), status = 'accepted',
    accepted_at = requested_at + interval '1 minute', updated_at = requested_at + interval '1 minute'
WHERE code = 119985
RETURNING ctid;
-- expect-line: (373,48)
UPDATE trips SET fare = fare + 5000 WHERE code = 119985 RETURNING ctid;
-- expect-line: (373,49)
UPDATE trips SET fare = fare + 1000 WHERE code = 110000 RETURNING ctid;
COMMIT;

-- ---------------------------------------------------------------------------
-- Bài tập 3 · Ứng dụng lưu ctid (373,32) của chuyến 119985 lúc đọc, rồi dùng lại.
-- Tid Scan theo địa chỉ cũ: phiên bản ở đó đã chết ⇒ 0 dòng; tìm theo khoá thì thấy.
-- expect-value: 0
SELECT count(*) FROM trips WHERE ctid = '(373,32)';
-- expect-line: 119985 | (373,48) | accepted
SELECT code, ctid, status FROM trips WHERE code = 119985;

-- ---------------------------------------------------------------------------
-- Bài tập 2 · Sau VACUUM, các ô của chuỗi phiên bản thành gì?
-- lp_flags: 0 = UNUSED (ô trống, dùng lại được), 1 = NORMAL, 2 = REDIRECT, 3 = DEAD.
-- INDEX_CLEANUP ON: buộc dọn index (dòng chết ít thì VACUUM thường có thể bỏ qua bước
-- dọn index, để lại ô DEAD — xem PG-026).
VACUUM (INDEX_CLEANUP ON) trips;

-- Ô 32 (phiên bản gốc, có mục index trỏ vào) ⇒ UNUSED sau khi index đã dọn.
-- Ô 47 (đích của mục index mới từ ①, đã HOT-update sang 48) ⇒ REDIRECT, lp_off = 48:
--   mục index vẫn trỏ (373,47), heap chuyển tiếp sang ô 48.
-- Ô 48 = phiên bản hiện hành.
-- expect-line: 373 | 32 | 0 |  0
-- expect-line: 373 | 47 | 2 | 48
-- expect-line: 373 | 48 | 1 | NULL
SELECT 373 AS page, lp, lp_flags, CASE WHEN lp_flags = 1 THEN NULL ELSE lp_off END AS lp_off
FROM public.heap_page_items(public.get_raw_page('bai_cu.trips', 373)) WHERE lp IN (32, 47, 48)
ORDER BY lp;

-- Page 186: ô 54 (bản cũ của ③) là ô CUỐI mảng line pointer ⇒ VACUUM không chỉ đánh
-- dấu trống mà cắt luôn khỏi mảng: page còn 53 ô.
-- expect-line: 53 | 0
SELECT max(lp), count(*) FILTER (WHERE lp_flags <> 1)
FROM public.heap_page_items(public.get_raw_page('bai_cu.trips', 186));

-- Đọc theo id vẫn tới đúng dòng: index → (373,47) → chuyển tiếp → (373,48).
-- expect-line: (373,48)
SELECT ctid FROM trips WHERE id = (SELECT id FROM trips WHERE code = 119985);

-- ---------------------------------------------------------------------------
-- Bài tập 1 · Dự đoán HOT. Chuyến 119990 ở page 373 (VACUUM vừa trả lại chỗ trống).
--   (a) đổi status — status không có index trong bai_cu ⇒ HOT
--   (b) gán requested_at = chính nó — cột CÓ index nhưng giá trị không đổi ⇒ vẫn HOT
--   (c) dời requested_at 1 phút — cột có index (ix_trips_city_requested,
--       ix_trips_customer_requested) đổi giá trị ⇒ KHÔNG HOT
-- @B
SET search_path TO bai_cu, toinoi;
-- expect-match: \(373,\d+\)
SELECT ctid FROM trips WHERE code = 119990;
BEGIN;
UPDATE trips SET status = 'cancelled', updated_at = updated_at + interval '1 minute' WHERE code = 119990;
-- expect-line: 1 | 1
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_xact_user_tables WHERE relid = 'bai_cu.trips'::regclass;
UPDATE trips SET requested_at = requested_at WHERE code = 119990;
-- expect-line: 2 | 2
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_xact_user_tables WHERE relid = 'bai_cu.trips'::regclass;
UPDATE trips SET requested_at = requested_at + interval '1 minute' WHERE code = 119990;
-- expect-line: 3 | 2
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_xact_user_tables WHERE relid = 'bai_cu.trips'::regclass;
-- Cả ba vẫn ở page 373: (c) cùng page mà KHÔNG HOT — "cùng page" chưa đủ.
-- expect-match: \(373,\d+\)
SELECT ctid FROM trips WHERE code = 119990;
ROLLBACK;

-- ---------------------------------------------------------------------------
-- Ví dụ trong bài: sau CLUSTER, \d in chữ CLUSTER cạnh index đã dùng — chỉ là cờ
-- pg_index.indisclustered, không có gì được duy trì (24-heap-ctid-hot.sql mục 7).
-- @A
CLUSTER trips USING trips_pkey;
-- expect-match: "trips_pkey" PRIMARY KEY, btree \(id\) CLUSTER
\d trips
