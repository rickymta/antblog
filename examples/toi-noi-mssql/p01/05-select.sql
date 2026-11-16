-- P01 · 05-select.sql — SELECT đầu tiên: lọc, sắp xếp, TOP và OFFSET/FETCH
-- (MS-007). Chạy sau 00 → 04.

SET NOCOUNT ON;
USE ToiNoi;
GO

-- S1. Chuyến đang chạy (chưa kết thúc), mới nhất trước.
SELECT code, kind, status, fare
FROM toinoi.trips
WHERE status NOT IN ('completed', 'cancelled')
ORDER BY requested_at DESC, code DESC;
EXEC lab.expect_value N'S1 số chuyến đang chạy', @@ROWCOUNT, N'6';
GO

-- S2. Ngày 02/11 THEO GIỜ VIỆT NAM, khoảng nửa mở [00:00, 00:00 hôm sau).
-- Hằng số mang độ lệch +07:00 nên so sánh đúng khoảnh khắc, bất kể server đặt
-- múi giờ nào. Chuyến 100007 (23:50) thuộc ngày 02; 100008 (00:10 ngày 03) không.
SELECT code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE requested_at >= '2026-11-02 00:00:00 +07:00'
  AND requested_at <  '2026-11-03 00:00:00 +07:00'
ORDER BY requested_at, code;
EXEC lab.expect_value N'S2 chuyến đặt ngày 02/11 giờ VN', @@ROWCOUNT, N'8';
GO

-- S3. Cắt NGÀY từ datetimeoffset: CAST(... AS date) lấy phần ngày của giá trị
-- như ĐÃ LƯU (ở đây là giờ địa phương +07:00) ⇒ 8 chuyến như S2. Đổi sang
-- +00:00 trước khi cắt thì được NGÀY UTC ⇒ 100008 (00:10 ngày 03 giờ VN =
-- 17:10 ngày 02 UTC) lọt vào ⇒ 9. Bọc hàm quanh cột còn làm index trên
-- requested_at khó dùng hơn (P03 đo) — ưu tiên khoảng nửa mở như S2.
SELECT (SELECT COUNT(*) FROM toinoi.trips
        WHERE CAST(requested_at AS date) = '2026-11-02')                          AS ngay_02_theo_gia_tri_luu,
       (SELECT COUNT(*) FROM toinoi.trips
        WHERE CAST(SWITCHOFFSET(requested_at, '+00:00') AS date) = '2026-11-02') AS ngay_02_theo_utc;
GO

-- S4. Thứ tự KHÔNG xác định: 100003 và 100004 cùng 12:00. Thiếu cột phụ trong
-- ORDER BY thì hai lần chạy có thể đổi chỗ — thêm code làm tie-breaker.
SELECT code, kind, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE requested_at >= '2026-11-02 11:00 +07:00' AND requested_at < '2026-11-02 13:00 +07:00'
ORDER BY requested_at, code;
GO

-- S5. TOP (3) chuyến cước cao nhất, rồi TOP (3) WITH TIES: lấy thêm các dòng
-- bằng giá trị cuối cùng.
SELECT TOP (3) code, fare FROM toinoi.trips ORDER BY fare DESC, code;
SELECT TOP (1) WITH TIES code, CAST(requested_at AS varchar(34)) AS requested_at
FROM toinoi.trips
WHERE requested_at >= '2026-11-02 12:00 +07:00'
ORDER BY requested_at;
EXEC lab.expect_value N'S5 TOP (1) WITH TIES lúc 12:00', @@ROWCOUNT, N'2';
GO

-- S6. Phân trang OFFSET/FETCH (cú pháp chuẩn, cần ORDER BY): trang 2, 5 dòng/trang.
SELECT code, status
FROM toinoi.trips
ORDER BY code
OFFSET 5 ROWS FETCH NEXT 5 ROWS ONLY;
GO

-- S7. NULL trong ORDER BY: SQL Server xếp NULL ĐẦU TIÊN khi tăng dần và chưa
-- có NULLS LAST (SQL Server 2022 không hỗ trợ cú pháp này). Đẩy NULL xuống
-- cuối bằng một biểu thức sắp xếp phụ.
SELECT TOP (5) code, status, CAST(accepted_at AS varchar(34)) AS accepted_at
FROM toinoi.trips
ORDER BY accepted_at, code;
SELECT TOP (5) code, status, CAST(accepted_at AS varchar(34)) AS accepted_at
FROM toinoi.trips
ORDER BY CASE WHEN accepted_at IS NULL THEN 1 ELSE 0 END, accepted_at, code;
GO

-- S8. NULL trong WHERE: "= NULL" luôn UNKNOWN ⇒ 0 dòng; phải dùng IS NULL.
SELECT (SELECT COUNT(*) FROM toinoi.trips WHERE driver_id = NULL)  AS bang_null,
       (SELECT COUNT(*) FROM toinoi.trips WHERE driver_id IS NULL) AS is_null;
-- Tài xế chưa từng có chuyến: NOT IN gặp NULL trong danh sách ⇒ 0 dòng (sai);
-- NOT EXISTS ⇒ đúng tài xế 8006.
SELECT COUNT(*) AS not_in FROM toinoi.drivers
WHERE id NOT IN (SELECT driver_id FROM toinoi.trips);
SELECT d.code AS not_exists FROM toinoi.drivers AS d
WHERE NOT EXISTS (SELECT 1 FROM toinoi.trips AS t WHERE t.driver_id = d.id);
GO

-- S9. Collation tiếng Việt. Trong bảng chữ cái tiếng Việt, ă, â, đ, ê, ô, ơ, ư
-- là CHỮ CÁI RIÊNG, còn huyền/sắc/hỏi/ngã/nặng là DẤU THANH.
--   * Vietnamese_100_CI_AS: A < Ă < Â … D < Đ < E; không phân biệt hoa thường.
--   * Latin1_General: coi Đ, Ă, Â chỉ là biến thể của D, A ⇒ thứ tự khác.
--   * _AI (accent-insensitive) bỏ qua DẤU THANH; với collation tiếng Việt nó
--     KHÔNG gộp ê với e, nên 'Nguyễn' = 'Nguyên' nhưng ≠ 'Nguyen'.
-- (Hai STRING_AGG có thứ tự khác nhau không được nằm cùng một SELECT — lỗi
-- 8711 — nên mỗi thứ tự một truy vấn con.)
SELECT (SELECT STRING_AGG(n, N', ') WITHIN GROUP (ORDER BY n COLLATE Vietnamese_100_CI_AS)
        FROM (VALUES (N'Dung'), (N'Đức'), (N'Duy'), (N'An'), (N'Ân'), (N'Ăn'), (N'Bình'), (N'Em')) AS v(n)) AS thu_tu_vietnamese,
       (SELECT STRING_AGG(n, N', ') WITHIN GROUP (ORDER BY n COLLATE Latin1_General_100_CI_AS)
        FROM (VALUES (N'Dung'), (N'Đức'), (N'Duy'), (N'An'), (N'Ân'), (N'Ăn'), (N'Bình'), (N'Em')) AS v(n)) AS thu_tu_latin1;

SELECT (SELECT COUNT(*) FROM toinoi.customers WHERE full_name = N'nguyễn văn an')  AS khong_phan_biet_hoa,
       (SELECT COUNT(*) FROM toinoi.customers WHERE full_name = N'Nguyen Van An')  AS bo_dau_CI_AS,
       (SELECT COUNT(*) FROM toinoi.customers
        WHERE full_name = N'Nguyên Văn An' COLLATE Vietnamese_100_CI_AI)           AS bo_thanh_vn_AI,
       (SELECT COUNT(*) FROM toinoi.customers
        WHERE full_name = N'Nguyen Van An' COLLATE Vietnamese_100_CI_AI)           AS bo_het_dau_vn_AI,
       (SELECT COUNT(*) FROM toinoi.customers
        WHERE full_name = N'Nguyen Van An' COLLATE Latin1_General_100_CI_AI)       AS bo_het_dau_latin_AI;
GO

-- S10. LIKE và ký tự đại diện: tìm khách họ "Nguyễn" / tên chứa "Hà".
SELECT phone, full_name FROM toinoi.customers WHERE full_name LIKE N'Nguyễn %';
SELECT phone, full_name FROM toinoi.customers WHERE full_name LIKE N'%Hà';
GO
