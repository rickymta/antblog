-- P02 · 02-kiem-tra-fixture.sql — hình dạng bộ dữ liệu và bất biến của nó.
--
-- Chạy ngay sau 01. Nửa đầu in "hồ sơ" dữ liệu để bài viết trích số; nửa sau
-- là các kiểm tra bất biến (đếm dòng vi phạm, mong đợi 0), gồm các kiểm tra
-- nhiều bảng của p01/07-checks.sql áp lên dữ liệu lớn và quy tắc của bảng mới.
-- Truy vấn dùng GROUP BY / JOIN / APPLY / NOT EXISTS — giải thích ở MS-012…MS-014.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ===========================================================================
-- Hồ sơ dữ liệu
-- ===========================================================================

-- H1. Chuyến theo thành phố × loại dịch vụ × kết cục.
SELECT c.code AS tp, t.kind,
       COUNT(*)                                                                    AS tong,
       SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END)                     AS hoan_thanh,
       SUM(CASE WHEN t.status = 'cancelled' AND t.driver_id IS NULL THEN 1 ELSE 0 END)     AS huy_truoc_nhan,
       SUM(CASE WHEN t.status = 'cancelled' AND t.driver_id IS NOT NULL THEN 1 ELSE 0 END) AS huy_sau_nhan,
       SUM(CASE WHEN t.status NOT IN ('completed', 'cancelled') THEN 1 ELSE 0 END) AS dang_chay
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
GROUP BY c.code, t.kind
ORDER BY c.code, t.kind;

-- H2. Đủ 8 trạng thái của vòng đời.
SELECT status, COUNT(*) AS so_chuyen
FROM toinoi.trips
GROUP BY status
ORDER BY CASE status WHEN 'requested' THEN 1 WHEN 'searching' THEN 2 WHEN 'accepted' THEN 3
                     WHEN 'arriving' THEN 4 WHEN 'picked_up' THEN 5 WHEN 'in_transit' THEN 6
                     WHEN 'completed' THEN 7 ELSE 8 END;

-- H3. Giá trị NULL và giá trị 0 có chủ đích (MS-011).
SELECT SUM(CASE WHEN driver_id IS NULL THEN 1 ELSE 0 END)      AS khong_tai_xe,
       SUM(CASE WHEN merchant_id IS NULL THEN 1 ELSE 0 END)    AS khong_quan,
       SUM(CASE WHEN pickup_area_id IS NULL THEN 1 ELSE 0 END) AS mat_gps,
       SUM(CASE WHEN fare = 0 THEN 1 ELSE 0 END)               AS cuoc_0d
FROM toinoi.trips;

-- H4. Chuyến đặt từ 00:00 tới 06:59 giờ VN = ngày HÔM TRƯỚC theo UTC (MS-016).
SELECT COUNT(*) AS chuyen_lech_ngay_utc
FROM toinoi.trips
WHERE CAST(SWITCHOFFSET(requested_at, '+07:00') AS date) <> CAST(SWITCHOFFSET(requested_at, '+00:00') AS date);

-- H5. Năm chuyến cài sát biên ngày 15/10 (MS-016).
SELECT code, CAST(requested_at AS varchar(34)) AS requested_at,
       CAST(SWITCHOFFSET(requested_at, '+00:00') AS varchar(34)) AS gio_utc
FROM toinoi.trips
WHERE requested_at IN ('2026-10-15 00:00:00 +07:00', '2026-10-15 06:59:59 +07:00', '2026-10-15 07:00:00 +07:00',
                       '2026-10-15 23:59:59.500 +07:00', '2026-10-16 00:00:00 +07:00')
ORDER BY code;

-- H6. Chuyến TRÙNG giờ đặt (MS-015, MS-018).
SELECT COUNT(*) AS so_nhom, SUM(n) AS so_chuyen, MAX(n) AS nhom_lon_nhat
FROM (SELECT requested_at, COUNT(*) AS n FROM toinoi.trips GROUP BY requested_at HAVING COUNT(*) > 1) AS g;

SELECT CAST(requested_at AS varchar(34)) AS requested_at, COUNT(*) AS so_chuyen, MIN(code) AS ma_dau, MAX(code) AS ma_cuoi
FROM toinoi.trips
GROUP BY requested_at
HAVING COUNT(*) > 2
ORDER BY COUNT(*) DESC, requested_at;

-- H7. Đánh giá: có dòng / bỏ trống sao / bỏ qua tip / chọn tip 0 đ.
SELECT COUNT(*)                                       AS so_danh_gia,
       SUM(CASE WHEN stars IS NULL THEN 1 ELSE 0 END) AS bo_trong_sao,
       SUM(CASE WHEN tip IS NULL THEN 1 ELSE 0 END)   AS bo_qua_tip,
       SUM(CASE WHEN tip = 0 THEN 1 ELSE 0 END)       AS tip_0d
FROM toinoi.trip_ratings;

-- H8. Giao dịch theo loại × phương thức × trạng thái.
SELECT kind, method, status, COUNT(*) AS so_gd, SUM(amount) AS tong_tien
FROM toinoi.payments
GROUP BY kind, method, status
ORDER BY kind, method, status;

-- H9. Mỗi chuyến có bao nhiêu dòng payments (quan hệ một–nhiều, MS-013).
SELECT so_gd, COUNT(*) AS so_chuyen
FROM (SELECT t.id, COUNT(p.id) AS so_gd
      FROM toinoi.trips AS t LEFT JOIN toinoi.payments AS p ON p.trip_id = t.id
      GROUP BY t.id) AS x
GROUP BY so_gd
ORDER BY so_gd;

-- Doanh thu kỳ tháng 10 (đặt trong [01/10, 31/10) giờ VN, đã kết thúc).
SELECT COUNT(*) AS chuyen_ky_thang_10,
       SUM(CASE WHEN status = 'completed' THEN CAST(fare AS bigint) ELSE 0 END) AS doanh_thu
FROM toinoi.trips
WHERE requested_at >= '2026-10-01 00:00:00 +07:00' AND requested_at < '2026-10-31 00:00:00 +07:00';
GO

-- ===========================================================================
-- Bất biến — mỗi kiểm tra đếm dòng VI PHẠM, mong đợi 0
-- ===========================================================================
DECLARE @v int;

-- K1. Trạng thái ↔ tài xế ↔ mốc thời gian (lặp lại logic CHECK của trips).
SET @v = (SELECT COUNT(*) FROM toinoi.trips
          WHERE (status IN ('requested', 'searching') AND driver_id IS NOT NULL)
             OR (status IN ('accepted', 'arriving', 'picked_up', 'in_transit', 'completed') AND driver_id IS NULL)
             OR (status = 'completed' AND completed_at IS NULL) OR (status <> 'completed' AND completed_at IS NOT NULL)
             OR accepted_at < requested_at OR completed_at < accepted_at);
EXEC lab.expect_value N'K1 trạng thái ↔ tài xế ↔ mốc', @v, N'0';

-- K2. Tài xế / quán khác thành phố của chuyến.
SET @v = (SELECT COUNT(*) FROM toinoi.trips AS t JOIN toinoi.drivers AS d ON d.id = t.driver_id WHERE d.city_id <> t.city_id)
       + (SELECT COUNT(*) FROM toinoi.trips AS t JOIN toinoi.merchants AS m ON m.id = t.merchant_id WHERE m.city_id <> t.city_id);
EXEC lab.expect_value N'K2 tài xế/quán khác thành phố', @v, N'0';

-- K3. Một tài xế > 1 chuyến đang chạy, hoặc `busy` lệch với chuyến.
SET @v = (SELECT COUNT(*) FROM (
            SELECT d.code
            FROM toinoi.drivers AS d
            LEFT JOIN toinoi.trips AS t
              ON t.driver_id = d.id AND t.status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
            GROUP BY d.code, d.status
            HAVING COUNT(t.id) > 1
                OR (COUNT(t.id) = 1 AND d.status <> 'busy')
                OR (COUNT(t.id) = 0 AND d.status = 'busy')) AS x);
EXEC lab.expect_value N'K3 tài xế busy lệch chuyến đang chạy', @v, N'0';

-- K4. updated_at không sớm hơn mốc mới nhất của dòng. GREATEST có từ SQL
--     Server 2022 và bỏ qua NULL (như PostgreSQL).
SET @v = (SELECT COUNT(*) FROM toinoi.trips WHERE updated_at < GREATEST(requested_at, accepted_at, completed_at));
EXEC lab.expect_value N'K4 updated_at nhỏ hơn mốc khác', @v, N'0';

-- K5. Lịch sử khớp chuyến: dòng đầu đúng giờ đặt, dòng cuối = trạng thái hiện
--     tại và đúng bằng updated_at.
SET @v = (SELECT COUNT(*)
          FROM toinoi.trips AS t
          CROSS APPLY (SELECT MIN(e.occurred_at) AS dau, MAX(e.occurred_at) AS cuoi
                       FROM toinoi.trip_events AS e WHERE e.trip_id = t.id) AS m
          CROSS APPLY (SELECT TOP (1) e.status FROM toinoi.trip_events AS e
                       WHERE e.trip_id = t.id ORDER BY e.occurred_at DESC, e.id DESC) AS l
          WHERE m.dau <> t.requested_at OR m.cuoi <> t.updated_at OR l.status <> t.status);
EXEC lab.expect_value N'K5 lịch sử lệch chuyến', @v, N'0';

-- K6. Đánh giá chỉ có ở chuyến hoàn thành, sau mốc hoàn thành.
SET @v = (SELECT COUNT(*) FROM toinoi.trip_ratings AS r JOIN toinoi.trips AS t ON t.id = r.trip_id
          WHERE t.status <> 'completed' OR r.rated_at < t.completed_at);
EXEC lab.expect_value N'K6 đánh giá sai chỗ', @v, N'0';

-- K7. Chuyến đang chạy chưa có giao dịch; giao dịch không sớm hơn giờ đặt.
SET @v = (SELECT COUNT(*) FROM toinoi.payments AS p JOIN toinoi.trips AS t ON t.id = p.trip_id
          WHERE t.status NOT IN ('completed', 'cancelled') OR p.paid_at < t.requested_at);
EXEC lab.expect_value N'K7 giao dịch sai chỗ', @v, N'0';

-- K8. Khu đón phải là nút `zone` thuộc cây của đúng thành phố chuyến.
SET @v = (SELECT COUNT(*)
          FROM toinoi.trips AS t
          JOIN toinoi.areas AS z ON z.id = t.pickup_area_id
          JOIN toinoi.areas AS v ON v.id = z.parent_id
          JOIN toinoi.areas AS g ON g.id = v.parent_id
          WHERE z.level <> 'zone' OR g.city_id <> t.city_id);
EXEC lab.expect_value N'K8 khu đón sai cấp/thành phố', @v, N'0';

-- K9. Cây khu vực đúng tầng: cha của vùng là thành phố, cha của khu là vùng.
SET @v = (SELECT COUNT(*)
          FROM toinoi.areas AS a JOIN toinoi.areas AS p ON p.id = a.parent_id
          WHERE NOT ((a.level = 'district' AND p.level = 'city') OR (a.level = 'zone' AND p.level = 'district')));
EXEC lab.expect_value N'K9 cây khu vực sai tầng', @v, N'0';
GO
