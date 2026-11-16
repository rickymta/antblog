-- P02 · 13-join-apply.sql — MS-013 `mssql-join-apply-cardinality`
-- JOIN, APPLY và số dòng sau khi nối: kiểm quan hệ một–nhiều, KHÔNG nhân đôi
-- tổng tiền; CROSS/OUTER APPLY lấy "N dòng mới nhất mỗi nhóm".
--
-- Quan hệ trong dữ liệu P02 (đếm ở 02-kiem-tra-fixture.sql, H9):
--   trips 1 — 0..n payments      (485 chuyến 0 dòng, 2.552 chuyến 1 dòng, 137 chuyến 2 dòng)
--   trips 1 — 1..7 trip_events   (chuyến hoàn thành: 7 dòng lịch sử)
--   trips 1 — 0..1 trip_ratings  (khoá chính trip_id ⇒ không bao giờ quá 1)
--   trips n — 1 cities / drivers / merchants  (FK trỏ tới khoá chính)
-- Phạm vi: 2.707 chuyến hoàn thành; doanh thu đúng = 182.287.000 đ (MS-012).
--
-- Chạy sau 00 → 02. File chỉ ĐỌC dữ liệu.

SET NOCOUNT ON;
USE [$(LAB_DB)];
GO

-- ---------------------------------------------------------------------------
-- 1. Bốn chuyến hoàn thành: 100041 trả thẻ một lần; 100042 thẻ lỗi rồi thu
--    lại; 100048 trả một phần ví, phần còn lại thẻ; 100724 không có giao dịch.
SELECT t.code, t.fare, p.method, p.status, p.amount
FROM toinoi.trips AS t
JOIN toinoi.payments AS p ON p.trip_id = t.id
WHERE t.code IN (100041, 100042, 100048, 100724)
ORDER BY t.code, p.id;

DECLARE @dung int, @inner int, @left int, @n_inner int, @n_left int;
SELECT @dung = SUM(fare) FROM toinoi.trips WHERE code IN (100041, 100042, 100048, 100724);
SELECT @inner = SUM(t.fare), @n_inner = COUNT(*)
FROM toinoi.trips AS t JOIN toinoi.payments AS p ON p.trip_id = t.id
WHERE t.code IN (100041, 100042, 100048, 100724);
SELECT @left = SUM(t.fare), @n_left = COUNT(*)
FROM toinoi.trips AS t LEFT JOIN toinoi.payments AS p ON p.trip_id = t.id
WHERE t.code IN (100041, 100042, 100048, 100724);
EXEC lab.expect_value N'1 bốn chuyến: Σ fare đúng', @dung, N'404000';
EXEC lab.expect_value N'1 bốn chuyến: số dòng INNER JOIN', @n_inner, N'5';
EXEC lab.expect_value N'1 bốn chuyến: Σ fare sau INNER JOIN', @inner, N'529000';
EXEC lab.expect_value N'1 bốn chuyến: số dòng LEFT JOIN', @n_left, N'6';
EXEC lab.expect_value N'1 bốn chuyến: Σ fare sau LEFT JOIN', @left, N'605000';
GO

-- ---------------------------------------------------------------------------
-- 2. Đếm trước khi nối, và số dòng SAU khi nối. Nối một–nhiều cho ra MỖI
--    DÒNG BÊN "NHIỀU" một dòng, không phải mỗi chuyến.
SELECT (SELECT COUNT(*) FROM toinoi.trips WHERE status = 'completed')            AS chuyen_hoan_thanh,
       (SELECT COUNT(*) FROM toinoi.trips AS t JOIN toinoi.payments AS p ON p.trip_id = t.id
         WHERE t.status = 'completed')                                           AS dong_inner_join,
       (SELECT COUNT(*) FROM toinoi.trips AS t LEFT JOIN toinoi.payments AS p ON p.trip_id = t.id
         WHERE t.status = 'completed')                                           AS dong_left_join;

-- LỖI KINH ĐIỂN: cộng cước SAU khi nối với payments.
SELECT COUNT(*)                AS so_dong,
       COUNT(DISTINCT t.id)    AS so_chuyen,
       SUM(t.fare)             AS tong_cuoc_SAI,
       SUM(p.amount)           AS tong_gd_moi_trang_thai
FROM toinoi.trips AS t
JOIN toinoi.payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed';

DECLARE @so_dong int, @so_chuyen int, @sai int, @distinct int;
SELECT @so_dong = COUNT(*), @so_chuyen = COUNT(DISTINCT t.id), @sai = SUM(t.fare), @distinct = SUM(DISTINCT t.fare)
FROM toinoi.trips AS t JOIN toinoi.payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed';
EXEC lab.expect_value N'2 số dòng sau INNER JOIN', @so_dong, N'2800';
EXEC lab.expect_value N'2 số chuyến còn lại', @so_chuyen, N'2675';
EXEC lab.expect_value N'2 Σ cước SAI sau JOIN', @sai, N'191644000';
-- SUM(DISTINCT) loại trùng theo GIÁ TRỊ cước, không theo chuyến.
EXEC lab.expect_value N'2 SUM(DISTINCT t.fare)', @distinct, N'11259000';

-- Phân rã chênh lệch 191.644.000 − 182.287.000: cước của chuyến có 2 dòng bị
-- cộng thêm một lần (+), chuyến không có dòng nào rơi khỏi INNER JOIN (−).
DECLARE @them int, @mat int, @so_mat int;
SELECT @them = SUM(t.fare)
FROM toinoi.trips AS t
WHERE t.status = 'completed'
  AND (SELECT COUNT(*) FROM toinoi.payments AS p WHERE p.trip_id = t.id) = 2;
SELECT @mat = SUM(t.fare), @so_mat = COUNT(*)
FROM toinoi.trips AS t
WHERE t.status = 'completed'
  AND NOT EXISTS (SELECT 1 FROM toinoi.payments AS p WHERE p.trip_id = t.id);
EXEC lab.expect_value N'2 cước cộng thừa (chuyến 2 dòng)', @them, N'9433000';
EXEC lab.expect_value N'2 số chuyến rơi khỏi INNER JOIN', @so_mat, N'32';
EXEC lab.expect_value N'2 cước rơi khỏi INNER JOIN', @mat, N'76000';
GO

-- ---------------------------------------------------------------------------
-- 3. Tìm chỗ "nở" dòng: với mỗi chuyến, payments có tối đa mấy dòng?
SELECT so_dong_gd, COUNT(*) AS so_chuyen
FROM (SELECT p.trip_id, COUNT(*) AS so_dong_gd
      FROM toinoi.payments AS p JOIN toinoi.trips AS t ON t.id = p.trip_id
      WHERE t.status = 'completed'
      GROUP BY p.trip_id) AS x
GROUP BY so_dong_gd
ORDER BY so_dong_gd;

-- trip_ratings: khoá chính trip_id ⇒ LEFT JOIN không làm nở dòng.
DECLARE @lap int = (SELECT COUNT(*) FROM (SELECT trip_id FROM toinoi.trip_ratings GROUP BY trip_id HAVING COUNT(*) > 1) AS x),
        @sau int = (SELECT COUNT(*) FROM toinoi.trips AS t LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
                    WHERE t.status = 'completed');
EXEC lab.expect_value N'3 trip_ratings.trip_id lặp', @lap, N'0';
EXEC lab.expect_value N'3 trips LEFT JOIN trip_ratings', @sau, N'2707';

-- Số giá trị khoá nối bị lặp trên toàn bảng.
DECLARE @lap_p int = (SELECT COUNT(*) FROM (SELECT trip_id FROM toinoi.payments GROUP BY trip_id HAVING COUNT(*) > 1) AS x),
        @lap_e int = (SELECT COUNT(*) FROM (SELECT trip_id FROM toinoi.trip_events GROUP BY trip_id HAVING COUNT(*) > 1) AS x);
EXEC lab.expect_value N'3 payments.trip_id lặp', @lap_p, N'137';
EXEC lab.expect_value N'3 trip_events.trip_id lặp', @lap_e, N'3173';
GO

-- ---------------------------------------------------------------------------
-- 4. CÁCH ĐÚNG: gom phía "nhiều" về MỘT dòng mỗi chuyến TRƯỚC, rồi mới nối.
--    Thực thu = Σ charge − Σ refund + Σ adjustment (chỉ succeeded).
SELECT COUNT(*)                       AS so_dong,
       COUNT(DISTINCT t.id)           AS so_chuyen,
       SUM(t.fare)                    AS tong_cuoc,
       SUM(ISNULL(g.thuc_thu, 0))     AS tong_thuc_thu
FROM toinoi.trips AS t
LEFT JOIN (
    SELECT trip_id,
           SUM(CASE kind WHEN 'refund' THEN -amount ELSE amount END) AS thuc_thu
    FROM toinoi.payments
    WHERE status = 'succeeded'
    GROUP BY trip_id
) AS g ON g.trip_id = t.id
WHERE t.status = 'completed';

DECLARE @c int, @tt int;
SELECT @c = SUM(t.fare), @tt = SUM(ISNULL(g.thuc_thu, 0))
FROM toinoi.trips AS t
LEFT JOIN (SELECT trip_id, SUM(CASE kind WHEN 'refund' THEN -amount ELSE amount END) AS thuc_thu
           FROM toinoi.payments WHERE status = 'succeeded' GROUP BY trip_id) AS g ON g.trip_id = t.id
WHERE t.status = 'completed';
EXEC lab.expect_value N'4 gom trước: Σ cước', @c, N'182287000';
EXEC lab.expect_value N'4 gom trước: Σ thực thu', @tt, N'182531000';
GO

-- ---------------------------------------------------------------------------
-- 5. APPLY: chạy một truy vấn con CHO TỪNG DÒNG bên trái (được tham chiếu
--    cột của dòng đó). Số dòng ra phụ thuộc vào truy vấn con:
--    * hàm tổng hợp KHÔNG có GROUP BY luôn trả đúng 1 dòng (kể cả khi 0 dòng
--      khớp — giá trị NULL) ⇒ CROSS APPLY giữ mọi chuyến;
--    * có GROUP BY thì 0 dòng khớp ⇒ 0 nhóm ⇒ CROSS APPLY bỏ chuyến đó;
--    * OUTER APPLY giữ chuyến bên trái như LEFT JOIN.
SELECT (SELECT COUNT(*) FROM toinoi.trips AS t
        CROSS APPLY (SELECT SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
                     FROM toinoi.payments AS p
                     WHERE p.trip_id = t.id AND p.status = 'succeeded') AS g
        WHERE t.status = 'completed')                                         AS cross_apply_khong_group_by,
       (SELECT COUNT(*) FROM toinoi.trips AS t
        CROSS APPLY (SELECT SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
                     FROM toinoi.payments AS p
                     WHERE p.trip_id = t.id AND p.status = 'succeeded'
                     GROUP BY p.trip_id) AS g
        WHERE t.status = 'completed')                                         AS cross_apply_group_by,
       (SELECT COUNT(*) FROM toinoi.trips AS t
        OUTER APPLY (SELECT SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
                     FROM toinoi.payments AS p
                     WHERE p.trip_id = t.id AND p.status = 'succeeded'
                     GROUP BY p.trip_id) AS g
        WHERE t.status = 'completed')                                         AS outer_apply_group_by;

DECLARE @ap int = (SELECT SUM(ISNULL(g.thuc_thu, 0)) FROM toinoi.trips AS t
                   CROSS APPLY (SELECT SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
                                FROM toinoi.payments AS p
                                WHERE p.trip_id = t.id AND p.status = 'succeeded') AS g
                   WHERE t.status = 'completed'),
        @ap_gb int = (SELECT COUNT(*) FROM toinoi.trips AS t
                      CROSS APPLY (SELECT 1 AS x FROM toinoi.payments AS p
                                   WHERE p.trip_id = t.id AND p.status = 'succeeded'
                                   GROUP BY p.trip_id) AS g
                      WHERE t.status = 'completed');
EXEC lab.expect_value N'5 CROSS APPLY tổng hợp: Σ thực thu', @ap, N'182531000';
EXEC lab.expect_value N'5 CROSS APPLY có GROUP BY: số chuyến', @ap_gb, N'2674';
GO

-- ---------------------------------------------------------------------------
-- 6. Dòng bị RƠI: chuyến hoàn thành KHÔNG có giao dịch thành công nào.
--    Anti-join bằng NOT EXISTS (hoặc LEFT JOIN … WHERE p.id IS NULL).
SELECT CASE WHEN t.fare = 0 THEN N'cước 0 đ — không cần thu' ELSE N'CÓ cước nhưng chưa thu' END AS loai,
       COUNT(*) AS so_chuyen,
       STRING_AGG(CASE WHEN t.fare > 0 THEN CONVERT(varchar(10), t.code) END, ', ')
           WITHIN GROUP (ORDER BY t.code) AS ma_chuyen
FROM toinoi.trips AS t
WHERE t.status = 'completed'
  AND NOT EXISTS (SELECT 1 FROM toinoi.payments AS p WHERE p.trip_id = t.id AND p.status = 'succeeded')
GROUP BY CASE WHEN t.fare = 0 THEN N'cước 0 đ — không cần thu' ELSE N'CÓ cước nhưng chưa thu' END
ORDER BY so_chuyen DESC;

DECLARE @ne int = (SELECT COUNT(*) FROM toinoi.trips AS t WHERE t.status = 'completed'
                   AND NOT EXISTS (SELECT 1 FROM toinoi.payments AS p WHERE p.trip_id = t.id AND p.status = 'succeeded')),
        @lj int = (SELECT COUNT(*) FROM toinoi.trips AS t
                   LEFT JOIN toinoi.payments AS p ON p.trip_id = t.id AND p.status = 'succeeded'
                   WHERE t.status = 'completed' AND p.id IS NULL);
EXEC lab.expect_value N'6 NOT EXISTS', @ne, N'33';
EXEC lab.expect_value N'6 LEFT JOIN … IS NULL', @lj, N'33';

-- Điều kiện của bảng bên phải: ở ON hay ở WHERE cho kết quả khác nhau.
SELECT (SELECT COUNT(DISTINCT t.id) FROM toinoi.trips AS t
          LEFT JOIN toinoi.payments AS p ON p.trip_id = t.id
         WHERE t.status = 'completed' AND p.status = 'succeeded')      AS dieu_kien_o_WHERE,
       (SELECT COUNT(DISTINCT t.id) FROM toinoi.trips AS t
          LEFT JOIN toinoi.payments AS p ON p.trip_id = t.id AND p.status = 'succeeded'
         WHERE t.status = 'completed')                                 AS dieu_kien_o_ON;

DECLARE @w int = (SELECT COUNT(DISTINCT t.id) FROM toinoi.trips AS t
                  LEFT JOIN toinoi.payments AS p ON p.trip_id = t.id
                  WHERE t.status = 'completed' AND p.status = 'succeeded');
EXEC lab.expect_value N'6 điều kiện ở WHERE', @w, N'2674';
GO

-- ---------------------------------------------------------------------------
-- 7. Hai quan hệ một–nhiều trong CÙNG một câu nhân với nhau.
SELECT t.code,
       COUNT(DISTINCT p.id)   AS so_gd,
       COUNT(DISTINCT e.id)   AS so_su_kien,
       COUNT(*)               AS so_dong_sau_noi,
       MAX(t.fare)            AS cuoc,
       SUM(p.amount)          AS sum_amount_SAI
FROM toinoi.trips AS t
JOIN toinoi.payments    AS p ON p.trip_id = t.id
JOIN toinoi.trip_events AS e ON e.trip_id = t.id
WHERE t.code = 100042
GROUP BY t.code;

DECLARE @thu int, @thu_sai int;
SELECT @thu = SUM(amount) FROM toinoi.payments WHERE status = 'succeeded' AND kind = 'charge';
SELECT @thu_sai = SUM(p.amount)
FROM toinoi.payments AS p JOIN toinoi.trip_events AS e ON e.trip_id = p.trip_id
WHERE p.status = 'succeeded' AND p.kind = 'charge';
EXEC lab.expect_value N'7 Σ thu thành công', @thu, N'183640000';
EXEC lab.expect_value N'7 Σ thu sau JOIN trip_events', @thu_sai, N'1283272000';
GO

-- ---------------------------------------------------------------------------
-- 8. Nối NHIỀU–MỘT (chuyến → thành phố, tài xế, quán) không làm nở dòng.
DECLARE @truoc int = (SELECT COUNT(*) FROM toinoi.trips WHERE status = 'completed'),
        @sau int = (SELECT COUNT(*) FROM toinoi.trips AS t
                    JOIN toinoi.cities         AS c ON c.id = t.city_id
                    JOIN toinoi.drivers        AS d ON d.id = t.driver_id
                    LEFT JOIN toinoi.merchants AS m ON m.id = t.merchant_id
                    WHERE t.status = 'completed');
EXEC lab.expect_value N'8 nhiều–một: trước', @truoc, N'2707';
EXEC lab.expect_value N'8 nhiều–một: sau', @sau, N'2707';

-- Câu hỏi "có hay không": JOIN đếm DÒNG giao dịch; EXISTS đếm CHUYẾN.
SELECT (SELECT COUNT(*) FROM toinoi.trips AS t JOIN toinoi.payments AS p ON p.trip_id = t.id
         WHERE t.status = 'completed' AND p.method = 'wallet')                 AS join_dem_dong,
       (SELECT COUNT(DISTINCT t.id) FROM toinoi.trips AS t JOIN toinoi.payments AS p ON p.trip_id = t.id
         WHERE t.status = 'completed' AND p.method = 'wallet')                 AS join_distinct,
       (SELECT COUNT(*) FROM toinoi.trips AS t
         WHERE t.status = 'completed'
           AND EXISTS (SELECT 1 FROM toinoi.payments AS p WHERE p.trip_id = t.id AND p.method = 'wallet')) AS exists_dem_chuyen;

DECLARE @ex int = (SELECT COUNT(*) FROM toinoi.trips AS t WHERE t.status = 'completed'
                   AND EXISTS (SELECT 1 FROM toinoi.payments AS p WHERE p.trip_id = t.id AND p.method = 'wallet'));
EXEC lab.expect_value N'8 EXISTS: chuyến từng dùng ví', @ex, N'961';
GO

-- ---------------------------------------------------------------------------
-- 9. Top-N mỗi nhóm bằng APPLY: 2 chuyến hoàn thành GẦN NHẤT của mỗi tài xế
--    Hà Nội. TOP trong APPLY cần ORDER BY có tie-breaker (code).
--    CROSS APPLY bỏ tài xế không có chuyến (8219, 8220); OUTER APPLY giữ họ.
SELECT d.code AS tai_xe, x.code AS chuyen, CAST(x.requested_at AS varchar(34)) AS requested_at, x.fare
FROM toinoi.drivers AS d
OUTER APPLY (SELECT TOP (2) t.code, t.requested_at, t.fare
             FROM toinoi.trips AS t
             WHERE t.driver_id = d.id AND t.status = 'completed'
             ORDER BY t.requested_at DESC, t.code DESC) AS x
WHERE d.code BETWEEN 8217 AND 8220
ORDER BY d.code, x.requested_at DESC, x.code DESC;

DECLARE @cross int = (SELECT COUNT(*) FROM toinoi.drivers AS d
                      CROSS APPLY (SELECT TOP (2) t.code FROM toinoi.trips AS t
                                   WHERE t.driver_id = d.id AND t.status = 'completed'
                                   ORDER BY t.requested_at DESC, t.code DESC) AS x
                      WHERE d.city_id = 2),
        @outer int = (SELECT COUNT(*) FROM toinoi.drivers AS d
                      OUTER APPLY (SELECT TOP (2) t.code FROM toinoi.trips AS t
                                   WHERE t.driver_id = d.id AND t.status = 'completed'
                                   ORDER BY t.requested_at DESC, t.code DESC) AS x
                      WHERE d.city_id = 2),
        @tx_hn int = (SELECT COUNT(*) FROM toinoi.drivers WHERE city_id = 2);
EXEC lab.expect_value N'9 tài xế Hà Nội', @tx_hn, N'20';
EXEC lab.expect_value N'9 CROSS APPLY TOP (2): số dòng', @cross, N'36';
EXEC lab.expect_value N'9 OUTER APPLY TOP (2): số dòng', @outer, N'38';

-- Giao dịch MỚI NHẤT của mỗi chuyến (1 dòng mỗi chuyến, kể cả chuyến không
-- có giao dịch) — thay cho JOIN rồi lọc.
SELECT t.code, t.fare, lp.method, lp.status, lp.amount
FROM toinoi.trips AS t
OUTER APPLY (SELECT TOP (1) p.method, p.status, p.amount
             FROM toinoi.payments AS p
             WHERE p.trip_id = t.id
             ORDER BY p.paid_at DESC, p.id DESC) AS lp
WHERE t.code IN (100041, 100042, 100048, 100724)
ORDER BY t.code;
GO

-- ===========================================================================
-- Bài tập (lời giải trong bài viết)
-- ===========================================================================

-- BT1. Tiền đã thu theo phương thức (chuyến hoàn thành, giao dịch thành công).
--      Câu sai gán CẢ CƯỚC chuyến cho mỗi dòng giao dịch.
SELECT p.method,
       COUNT(*)                                                         AS so_gd,
       SUM(t.fare)                                                      AS sai_sum_fare,
       SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END)  AS da_thu
FROM toinoi.trips AS t
JOIN toinoi.payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed' AND p.status = 'succeeded'
GROUP BY p.method
ORDER BY p.method;

DECLARE @sai_tong int, @dung_tong int;
SELECT @sai_tong = SUM(t.fare), @dung_tong = SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END)
FROM toinoi.trips AS t JOIN toinoi.payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed' AND p.status = 'succeeded';
EXEC lab.expect_value N'BT1 Σ fare theo dòng giao dịch (sai)', @sai_tong, N'184489000';
EXEC lab.expect_value N'BT1 Σ đã thu', @dung_tong, N'182531000';
GO

-- BT2. trips LEFT JOIN payments LEFT JOIN trip_ratings: số dòng và điểm TB.
DECLARE @dong int, @tb decimal(4, 2), @dong2 int, @tb2 decimal(4, 2), @gd int;
SELECT @dong = COUNT(*), @tb = CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2))
FROM toinoi.trips AS t
LEFT JOIN toinoi.payments AS p ON p.trip_id = t.id
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed';
SELECT @dong2 = COUNT(*), @tb2 = CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2)), @gd = SUM(g.so_gd)
FROM toinoi.trips AS t
OUTER APPLY (SELECT COUNT(*) AS so_gd FROM toinoi.payments AS p WHERE p.trip_id = t.id) AS g
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed';
SELECT @dong AS so_dong_sai, @tb AS diem_tb_sai, @dong2 AS so_dong, @tb2 AS diem_tb, @gd AS so_gd;
EXEC lab.expect_value N'BT2 số dòng sau hai LEFT JOIN', @dong, N'2832';
EXEC lab.expect_value N'BT2 điểm TB bị lệch', @tb, N'4.24';
EXEC lab.expect_value N'BT2 sau khi gom: số dòng', @dong2, N'2707';
EXEC lab.expect_value N'BT2 sau khi gom: điểm TB', @tb2, N'4.22';
GO

-- BT3. 3 chuyến hoàn thành cước cao nhất mỗi thành phố ngày 20/10 (giờ VN).
SELECT c.code AS tp, x.code, x.fare
FROM toinoi.cities AS c
OUTER APPLY (SELECT TOP (3) t.code, t.fare
             FROM toinoi.trips AS t
             WHERE t.city_id = c.id AND t.status = 'completed'
               AND t.requested_at >= '2026-10-20 00:00 +07:00' AND t.requested_at < '2026-10-21 00:00 +07:00'
             ORDER BY t.fare DESC, t.code) AS x
ORDER BY c.code, x.fare DESC, x.code;

DECLARE @bt3_cross int = (SELECT COUNT(*) FROM toinoi.cities AS c
                          CROSS APPLY (SELECT TOP (3) t.code FROM toinoi.trips AS t
                                       WHERE t.city_id = c.id AND t.status = 'completed'
                                         AND t.requested_at >= '2026-10-20 00:00 +07:00' AND t.requested_at < '2026-10-21 00:00 +07:00'
                                       ORDER BY t.fare DESC, t.code) AS x),
        @bt3_outer int = (SELECT COUNT(*) FROM toinoi.cities AS c
                          OUTER APPLY (SELECT TOP (3) t.code FROM toinoi.trips AS t
                                       WHERE t.city_id = c.id AND t.status = 'completed'
                                         AND t.requested_at >= '2026-10-20 00:00 +07:00' AND t.requested_at < '2026-10-21 00:00 +07:00'
                                       ORDER BY t.fare DESC, t.code) AS x);
EXEC lab.expect_value N'BT3 CROSS APPLY số dòng', @bt3_cross, N'6';
EXEC lab.expect_value N'BT3 OUTER APPLY số dòng', @bt3_outer, N'7';
GO
