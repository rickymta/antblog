-- P03 · 30-toi-uu-truy-van.sql — PG-030 «Thực hành: tối ưu truy vấn bằng bằng chứng»
-- (`postgresql-thuc-hanh-toi-uu-truy-van`).
--
-- Bàn giao của bài: kế hoạch TRƯỚC/SAU, fixture (nền p03), phép đo lặp (p03/DO-DAC.md,
-- PG-030), CHI PHÍ GHI của index được thêm, và GIỚI HẠN của kết luận.
--
-- Bài toán: màn CSKH "lịch sử chuyến HOÀN THÀNH của khách, mới nhất trước, kèm số tiền
-- đã thu" (20 dòng/trang). Khách cá nhân thấy nhanh; khách doanh nghiệp B1 (51.151
-- chuyến) thì chậm. Quy trình: (1) đặc tả + đo hiện trạng, (2) đọc kế hoạch tìm chỗ
-- tốn, (3) đặt giả thuyết và LOẠI các cách sai, (4) sửa, (5) kiểm KẾT QUẢ KHÔNG ĐỔI,
-- (6) đo lại, (7) đo cái giá phải trả, (8) ghi giới hạn.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';
-- Câu PREPARE được lập kế hoạch RIÊNG cho từng tham số ở 5 lần EXECUTE đầu, sau đó
-- PostgreSQL có thể chuyển sang kế hoạch CHUNG (generic, không biết tham số). Để mọi số
-- đo trong bài so được với nhau, bài ép kế hoạch riêng; mục 6b xem kế hoạch chung.
SET plan_cache_mode = force_custom_plan;

SELECT id AS kh_b1 FROM customers WHERE phone = '0919990001' \gset
SELECT id AS kh_le FROM customers WHERE phone = '0910000500' \gset

-- ---------------------------------------------------------------------------
-- 1. Câu gốc (đúng đặc tả). Thanh toán: thu thành công − hoàn thành công.
PREPARE lich_su_goc(uuid) AS
SELECT t.code, t.completed_at, t.kind, t.fare,
       coalesce(sum(p.amount) FILTER (WHERE p.kind = 'charge' AND p.status = 'succeeded'), 0)
     - coalesce(sum(p.amount) FILTER (WHERE p.kind = 'refund' AND p.status = 'succeeded'), 0) AS da_thu
FROM trips AS t
LEFT JOIN payments AS p ON p.trip_id = t.id
WHERE t.customer_id = $1 AND t.status = 'completed'
GROUP BY t.id
ORDER BY t.completed_at DESC, t.code DESC
LIMIT 20;

-- Kết quả tham chiếu cho B1 (md5 dùng để so với câu sau tối ưu).
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_goc(:'kh_b1');

-- ---------------------------------------------------------------------------
-- 2. Kế hoạch hiện trạng với B1 (cache nóng — chạy một lần trước rồi mới đo).
--    Đọc từ trong ra: lấy HẾT 51.151 chuyến của B1 (Bitmap trên index (customer_id,
--    requested_at), chạm 18.139 page heap), lọc còn 44.922 chuyến hoàn thành, dựng bảng
--    băm rồi QUÉT CẢ 901.208 giao dịch payments để join, gom nhóm, sắp top-20 theo
--    completed_at — rồi bỏ hết, chỉ giữ 20 dòng. ~29.000 page cho 20 dòng. Thủ phạm:
--    không có đường đi "theo completed_at của MỘT khách" nên phải làm cho mọi chuyến.
-- expect-match: Limit \(actual rows=20\.00 loops=1\)
-- expect-match: Sort Key: t\.completed_at DESC, t\.code DESC
-- expect-match: Bitmap Heap Scan on trips t \(actual rows=44922\.00 loops=1\)
-- expect-match: Seq Scan on payments p \(actual rows=901208\.00 loops=1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF) EXECUTE lich_su_goc(:'kh_b1');

-- Khách cá nhân (40 chuyến): kế hoạch KHÁC (ước ~11 chuyến ⇒ Nested Loop tra payments
-- từng chuyến, ~190 page) và nhanh. Đừng tối ưu dựa trên MỘT tham số: chậm chỉ xảy ra
-- với khách đặt rất nhiều. (Kế hoạch của khách này dựa trên ước lượng lấy mẫu ⇒ biến động.)
-- lab: bien-dong
-- expect-match: Limit \(actual rows=20\.00 loops=1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF) EXECUTE lich_su_goc(:'kh_le');

-- ---------------------------------------------------------------------------
-- 3. Giả thuyết SAI #1: "sắp theo requested_at là đủ, đã có sẵn index". Sai đặc tả: thứ
--    tự HOÀN THÀNH khác thứ tự ĐẶT (chuyến đặt trước có thể xong sau). Ở trang đầu của B1
--    tình cờ trùng (0 chuyến khác), nhưng xét cả lịch sử thì hàng chục nghìn chuyến đổi
--    vị trí — trang 2, 3… sẽ sai. Bằng chứng phải xét đủ, đừng chỉ nhìn 20 dòng đầu.
-- expect-value: 0
SELECT count(*) FROM (
  (SELECT code FROM trips WHERE customer_id = :'kh_b1' AND status = 'completed' ORDER BY completed_at DESC, code DESC LIMIT 20)
  EXCEPT
  (SELECT code FROM trips WHERE customer_id = :'kh_b1' AND status = 'completed' ORDER BY requested_at DESC, code DESC LIMIT 20)
) AS lech_trang_dau;

-- expect-range: 30000 40000
SELECT count(*) FILTER (WHERE theo_hoan_thanh <> theo_dat) AS so_chuyen_doi_vi_tri
FROM (SELECT row_number() OVER (ORDER BY completed_at DESC, code DESC) AS theo_hoan_thanh,
             row_number() OVER (ORDER BY requested_at DESC, code DESC) AS theo_dat
      FROM trips WHERE customer_id = :'kh_b1' AND status = 'completed') AS x;

-- Giả thuyết SAI #2: "thêm index trên payments(trip_id, status)". payments ĐÃ có
-- ix_payments_trip; planner quét cả payments chỉ vì phải join cho 44.922 chuyến một lúc.
-- Gốc vấn đề là xử lý 44.922 chuyến để lấy 20 — index trên payments không đổi điều đó.

-- ---------------------------------------------------------------------------
-- 4. Sửa: (a) index THEO ĐÚNG đường truy cập — khách + completed_at giảm dần, CHỈ cho
--    chuyến hoàn thành (partial: bỏ 12 % chuyến huỷ/đang chạy khỏi index); (b) viết lại
--    để lấy 20 chuyến TRƯỚC rồi mới tính tiền cho đúng 20 chuyến đó (LATERAL).
CREATE INDEX ix_trips_khach_hoan_thanh ON trips (customer_id, completed_at DESC, code DESC)
WHERE status = 'completed';

PREPARE lich_su_moi(uuid) AS
SELECT t.code, t.completed_at, t.kind, t.fare, tien.da_thu
FROM (
  SELECT id, code, completed_at, kind, fare
  FROM trips
  WHERE customer_id = $1 AND status = 'completed'
  ORDER BY completed_at DESC, code DESC
  LIMIT 20
) AS t
CROSS JOIN LATERAL (
  SELECT coalesce(sum(p.amount) FILTER (WHERE p.kind = 'charge' AND p.status = 'succeeded'), 0)
       - coalesce(sum(p.amount) FILTER (WHERE p.kind = 'refund' AND p.status = 'succeeded'), 0) AS da_thu
  FROM payments AS p WHERE p.trip_id = t.id
) AS tien
ORDER BY t.completed_at DESC, t.code DESC;

-- ---------------------------------------------------------------------------
-- 5. KẾT QUẢ KHÔNG ĐỔI: cùng md5 với câu gốc (bước 1), cho cả B1 và khách cá nhân.
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_moi(:'kh_b1');

-- Câu gốc (KHÔNG viết lại) khi đã có index mới — vẫn đúng kết quả…
-- expect-md5: c8046204e6fa07cc57d4196a9cf7cc0e
EXECUTE lich_su_goc(:'kh_b1');

-- … nhưng gần như KHÔNG nhanh hơn: planner dùng index mới chỉ để thay bitmap lấy 44.922
-- chuyến, rồi vẫn quét cả payments, gom nhóm, sắp. Index một mình không đủ — phải viết
-- lại để "lấy 20 trước, tính tiền sau" (lich_su_moi).
-- expect-match: Bitmap Index Scan on ix_trips_khach_hoan_thanh
-- expect-match: Seq Scan on payments p \(actual rows=901208\.00 loops=1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF) EXECUTE lich_su_goc(:'kh_b1');

-- expect-value: 0
SELECT count(*) FROM (
  (SELECT * FROM (SELECT t.code, t.completed_at, t.fare FROM trips AS t WHERE t.customer_id = :'kh_le' AND t.status = 'completed'
                  ORDER BY t.completed_at DESC, t.code DESC LIMIT 20) AS a)
  EXCEPT
  (SELECT code, completed_at, fare FROM (SELECT * FROM trips WHERE customer_id = :'kh_le' AND status = 'completed') AS b
   ORDER BY completed_at DESC, code DESC LIMIT 20)
) AS khac;

-- ---------------------------------------------------------------------------
-- 6. Kế hoạch SAU: đi index partial theo đúng thứ tự, dừng sau 20 dòng, tra payments
--    20 lần. Số page chạm giảm từ hàng chục nghìn xuống vài chục.
-- expect-match: Index Scan using ix_trips_khach_hoan_thanh on trips
-- expect-match: Limit \(actual rows=20\.00 loops=1\)
-- expect-no-match: Sort Key
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF) EXECUTE lich_su_moi(:'kh_b1');

-- So sánh số page chạm (Buffers ở nút gốc) trước/sau, cache nóng, đo bằng hàm nhỏ.
CREATE FUNCTION so_page_cham(sql text) RETURNS bigint LANGUAGE plpgsql AS $$
DECLARE j jsonb;
BEGIN
  EXECUTE 'EXPLAIN (ANALYZE, BUFFERS, TIMING OFF, FORMAT JSON) ' || sql INTO j;
  RETURN (j -> 0 -> 'Plan' ->> 'Shared Hit Blocks')::bigint + (j -> 0 -> 'Plan' ->> 'Shared Read Blocks')::bigint;
END $$;

BEGIN;
SET LOCAL enable_indexscan = on;
-- Trước: ép không dùng index mới (xoá trong giao dịch rồi ROLLBACK) để đo lại hiện trạng.
DROP INDEX ix_trips_khach_hoan_thanh;
-- expect-range: 20000 60000
SELECT so_page_cham(format('EXECUTE lich_su_goc(%L)', :'kh_b1')) AS page_truoc;
ROLLBACK;

-- (Sau: 98 page = 18 page index/heap cho 20 chuyến + 80 page cho 20 lần tra payments.)
-- expect-range: 20 200
SELECT so_page_cham(format('EXECUTE lich_su_moi(%L)', :'kh_b1')) AS page_sau;

-- ---------------------------------------------------------------------------
-- 6b. GIỚI HẠN quan trọng: kế hoạch CHUNG. Ứng dụng dùng prepared statement (nhiều driver
--     làm vậy mặc định) ⇒ sau 5 lần EXECUTE, PostgreSQL có thể dùng một kế hoạch cho MỌI
--     khách, lập theo khách "trung bình" (ước ~5–12 chuyến — suy từ n_distinct do ANALYZE
--     LẤY MẪU). Kế hoạch chung nào được chọn vì thế đổi giữa các lần ANALYZE (cả hai trường
--     hợp đã gặp khi dựng lab): câu viết lại có khi vẫn đi index lấy 20 dòng (~98 page), có
--     khi thành Bitmap + SẮP cả 44.922 chuyến của B1 (~18.000 page); câu gốc thành Nested
--     Loop tra payments 44.922 lần (~212.000 page). Chính sự bấp bênh này là bằng chứng:
--     kết luận "98 page" chỉ chắc chắn với kế hoạch RIÊNG.
BEGIN;
SET LOCAL plan_cache_mode = force_generic_plan;
-- Kế hoạch chung mang tham số $1 (không biết khách nào lúc lập kế hoạch).
-- lab: bien-dong
-- expect-match: Index Cond: \(customer_id = \$1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF) EXECUTE lich_su_moi(:'kh_b1');

-- expect-range: 20 40000
SELECT so_page_cham(format('EXECUTE lich_su_moi(%L)', :'kh_b1')) AS page_ke_hoach_chung_moi;

-- lab: bien-dong
-- expect-match: Index Cond: \(customer_id = \$1\)
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF) EXECUTE lich_su_goc(:'kh_b1');

-- expect-range: 20000 300000
SELECT so_page_cham(format('EXECUTE lich_su_goc(%L)', :'kh_b1')) AS page_ke_hoach_chung_goc;
ROLLBACK;
-- Cách xử lý (tuỳ ứng dụng): plan_cache_mode = force_custom_plan cho câu/phiên này, tách
-- đường riêng cho khách doanh nghiệp, hoặc không dùng prepared statement có tên. Thời gian
-- của từng kế hoạch: DO-DAC.md (G8). Kết luận "98 page" chỉ đúng với kế hoạch RIÊNG.

-- ---------------------------------------------------------------------------
-- 7. CÁI GIÁ của index mới.
-- 7a. Dung lượng (partial bỏ dòng ngoài điều kiện nhưng khoá rộng hơn ⇒ có thể LỚN hơn index đầy đủ hẹp; đo, đừng đoán).
-- expect-line: ix_trips_customer_requested | 40624128
-- expect-line: ix_trips_khach_hoan_thanh   | 43712512
SELECT relname, pg_relation_size(oid) AS bytes FROM pg_class
WHERE relname IN ('ix_trips_customer_requested', 'ix_trips_khach_hoan_thanh') AND relnamespace = 'toinoi'::regnamespace
ORDER BY relname;

-- 7b. WAL khi nạp một ngày chuyến mới (5.000 chuyến hoàn thành chép từ 24/10 sang 02/11),
--     KHÔNG có và CÓ index mới — đo trong giao dịch rồi ROLLBACK để dữ liệu không đổi.
--     Ngay sau CHECKPOINT nên phần lớn WAL là ẢNH PAGE (fpi) của các page index bị chạm
--     lần đầu — với khoá ngẫu nhiên (uuid, khách) gần như mỗi dòng chạm một page khác.
--     Dòng "Trigger for constraint …: calls=5000" = 5 lần tra khoá ngoại cho mỗi dòng.
BEGIN;
DROP INDEX ix_trips_khach_hoan_thanh;
CHECKPOINT;
-- expect-match: WAL: records=\d+ fpi=\d+ bytes=\d+
EXPLAIN (ANALYZE, WAL, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at, pickup_area_id)
SELECT p03_uuid('ngay-moi', code), kind, city_id, customer_id, driver_id, merchant_id, status, fare,
       requested_at + interval '9 days', accepted_at + interval '9 days', completed_at + interval '9 days',
       updated_at + interval '9 days', pickup_area_id
FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-10-24 00:00+07' AND status = 'completed'
ORDER BY code LIMIT 5000;
ROLLBACK;

CHECKPOINT;
BEGIN;
-- expect-match: WAL: records=\d+ fpi=\d+ bytes=\d+
EXPLAIN (ANALYZE, WAL, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
INSERT INTO trips (id, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at, pickup_area_id)
SELECT p03_uuid('ngay-moi', code), kind, city_id, customer_id, driver_id, merchant_id, status, fare,
       requested_at + interval '9 days', accepted_at + interval '9 days', completed_at + interval '9 days',
       updated_at + interval '9 days', pickup_area_id
FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-10-24 00:00+07' AND status = 'completed'
ORDER BY code LIMIT 5000;
ROLLBACK;

-- 7c. Index mới đụng tới cột status (điều kiện partial) và completed_at — hai cột đổi
--     đúng lúc chuyến HOÀN THÀNH. Cột nằm trong định nghĩa index (kể cả chỉ trong mệnh đề
--     WHERE của partial index) mà đổi giá trị ⇒ UPDATE đó KHÔNG thể là HOT. Thử trên bản
--     sao 10.000 chuyến có fillfactor 90 (page CÒN chỗ, để chỉ còn điều kiện "cột có index"):
--     đánh dấu 2 × 200 chuyến huỷ thành hoàn thành, trước và sau khi tạo index kiểu mới.
CREATE TABLE hoan_thanh_thu WITH (fillfactor = 90) AS
SELECT * FROM trips WHERE code BETWEEN 600000 AND 609999;
CREATE INDEX ON hoan_thanh_thu (customer_id, requested_at DESC);
UPDATE hoan_thanh_thu SET status = 'completed', completed_at = updated_at
WHERE code IN (SELECT code FROM hoan_thanh_thu WHERE status = 'cancelled' AND driver_id IS NOT NULL ORDER BY code LIMIT 200);
CREATE INDEX ON hoan_thanh_thu (customer_id, completed_at DESC, code DESC) WHERE status = 'completed';
UPDATE hoan_thanh_thu SET status = 'completed', completed_at = updated_at
WHERE code IN (SELECT code FROM hoan_thanh_thu WHERE status = 'cancelled' AND driver_id IS NOT NULL ORDER BY code LIMIT 200);
SELECT pg_stat_force_next_flush();
-- 200 lần đầu (chưa có index mới) đều HOT; 200 lần sau không lần nào.
-- expect-line: 400 | 200
SELECT n_tup_upd, n_tup_hot_upd FROM pg_stat_user_tables WHERE relid = 'hoan_thanh_thu'::regclass;

-- ---------------------------------------------------------------------------
-- 8. GIỚI HẠN của kết luận (ghi kèm bàn giao):
--   * Đo trên một máy (cấu hình ở DO-DAC.md), dữ liệu tổng hợp 1 triệu chuyến, cache nóng,
--     một phiên; production có tải đồng thời, cache khác, dữ liệu lệch khác.
--   * Cải thiện lớn CHỈ với khách đặt rất nhiều (B1); khách thường vốn đã nhanh.
--   * Index mới làm MỌI lần chèn/hoàn thành chuyến tốn thêm WAL và I/O (7b, 7c) — với
--     70 chuyến mới/giây và 560 UPDATE/giây lúc cao điểm, cần đo cả phía ghi trước khi bật.
--   * Nếu màn CSKH chỉ cần "mới nhất theo giờ đặt", đổi ĐẶC TẢ (dùng index sẵn có) rẻ hơn
--     thêm index — nhưng đó là quyết định nghiệp vụ, không phải kỹ thuật.
DEALLOCATE ALL;
