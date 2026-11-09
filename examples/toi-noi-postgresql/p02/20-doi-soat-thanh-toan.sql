-- P02 · 20-doi-soat-thanh-toan.sql — PG-020 `postgresql-thuc-hanh-sql-doi-soat`
-- Thực hành: truy vấn vận hành và ĐỐI SOÁT THANH TOÁN — bàn giao SQL, expected
-- output và giải thích chênh lệch.
--
-- ĐẶC TẢ ĐỐI SOÁT THÁNG 10/2026
--   Phạm vi     chuyến ĐẶT trong [2026-10-01 00:00, 2026-10-31 00:00) giờ VN và đã
--               kết thúc (completed / cancelled) — 3.168 chuyến. Chuyến đang chạy
--               chưa đối soát.
--   Phải thu    completed ⇒ cước (fare); cancelled ⇒ 0 đ.
--   Thực thu    Σ charge − Σ refund + Σ adjustment, CHỈ giao dịch `succeeded`
--               (pending/failed không làm đổi tiền), KHÔNG lọc theo ngày giao
--               dịch: tiền của chuyến tính cho chuyến, dù thu sau nửa đêm.
--   Khớp        thực thu = phải thu.
--   Phân loại   theo thứ tự ưu tiên (dòng lệch nhận loại ĐẦU TIÊN đúng):
--     1 HOÀN KHÔNG CÓ THU   có refund thành công nhưng không có charge thành công
--     2 THU CHUYẾN HUỶ      chuyến huỷ, thực thu > 0
--     3 THIẾU THANH TOÁN    hoàn thành, cước > 0, không có charge thành công
--     4 THU TRÙNG           ≥ 2 charge thành công CÙNG số tiền (lặp yêu cầu)
--     5 LỆCH SỐ TIỀN        còn lại: có thu nhưng thực thu ≠ phải thu
--   Lưu ý: hai charge thành công KHÁC số tiền là trả một phần (ví + thẻ),
--   KHÔNG phải thu trùng.
--
-- Kết quả mong đợi (cài sẵn trong 01-fixture.sql): ĐÚNG 7 chuyến lệch, tổng
-- chênh +268.000 đ; và KHÔNG báo nhầm các ca khó nhưng đúng (điều chỉnh của
-- CSKH, giao dịch pending, trả một phần, thu lỗi rồi thu lại, huỷ đã hoàn).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 0. Sổ đối soát theo chuyến: một VIEW tạm (sống trong phiên) để các bước sau
--    dùng chung MỘT định nghĩa — không chép lại công thức ở từng truy vấn.
CREATE TEMP VIEW so_doi_soat AS
WITH pham_vi AS (
  SELECT t.id, t.code, t.kind, t.city_id, t.status, t.fare,
         (t.requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS ngay,
         CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END AS phai_thu
  FROM trips AS t
  WHERE t.status IN ('completed', 'cancelled')
    AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
),
gd AS (
  SELECT p.trip_id,
         count(*) FILTER (WHERE kind = 'charge' AND status = 'succeeded')                 AS so_thu_ok,
         count(*) FILTER (WHERE kind = 'charge' AND status = 'succeeded')
           - count(DISTINCT amount) FILTER (WHERE kind = 'charge' AND status = 'succeeded') AS so_thu_lap_tien,
         coalesce(sum(amount) FILTER (WHERE kind = 'charge'     AND status = 'succeeded'), 0) AS tong_thu,
         coalesce(sum(amount) FILTER (WHERE kind = 'refund'     AND status = 'succeeded'), 0) AS tong_hoan,
         coalesce(sum(amount) FILTER (WHERE kind = 'adjustment' AND status = 'succeeded'), 0) AS tong_dieu_chinh,
         count(*) FILTER (WHERE status <> 'succeeded')                                    AS so_gd_khong_thanh_cong,
         string_agg(kind || '/' || method || '/' || status || '/' || amount, ', ' ORDER BY paid_at, id) AS chi_tiet
  FROM payments AS p
  GROUP BY p.trip_id
)
SELECT pv.*,
       coalesce(g.so_thu_ok, 0)                                        AS so_thu_ok,
       coalesce(g.tong_thu, 0)                                         AS tong_thu,
       coalesce(g.tong_hoan, 0)                                        AS tong_hoan,
       coalesce(g.tong_dieu_chinh, 0)                                  AS tong_dieu_chinh,
       coalesce(g.tong_thu - g.tong_hoan + g.tong_dieu_chinh, 0)       AS thuc_thu,
       coalesce(g.tong_thu - g.tong_hoan + g.tong_dieu_chinh, 0) - pv.phai_thu AS chenh,
       coalesce(g.so_gd_khong_thanh_cong, 0)                           AS so_gd_khong_thanh_cong,
       coalesce(g.chi_tiet, '(không có giao dịch)')                    AS chi_tiet,
       CASE
         WHEN coalesce(g.tong_thu - g.tong_hoan + g.tong_dieu_chinh, 0) = pv.phai_thu THEN NULL
         WHEN coalesce(g.so_thu_ok, 0) = 0 AND g.tong_hoan > 0          THEN '1 HOÀN KHÔNG CÓ THU'
         WHEN pv.status = 'cancelled'                                   THEN '2 THU CHUYẾN HUỶ'
         WHEN pv.phai_thu > 0 AND coalesce(g.so_thu_ok, 0) = 0          THEN '3 THIẾU THANH TOÁN'
         WHEN g.so_thu_lap_tien > 0                                     THEN '4 THU TRÙNG'
         ELSE                                                                '5 LỆCH SỐ TIỀN'
       END AS loai_lech
FROM pham_vi AS pv
LEFT JOIN gd AS g ON g.trip_id = pv.id;

-- ---------------------------------------------------------------------------
-- 1. ĐỐI SOÁT TỔNG: số chuyến, tổng phải thu, tổng thực thu, chênh.
-- expect-line: 3168 | 182287000 | 182555000 | 268000
SELECT count(*)       AS so_chuyen,
       sum(phai_thu)  AS tong_phai_thu,
       sum(thuc_thu)  AS tong_thuc_thu,
       sum(chenh)     AS chenh
FROM so_doi_soat;

-- Sổ không được làm nở hay mất dòng: mỗi chuyến trong phạm vi đúng một dòng.
-- expect-line: 3168 | 3168 | 3168
SELECT (SELECT count(*) FROM so_doi_soat)                  AS dong_so,
       (SELECT count(DISTINCT id) FROM so_doi_soat)        AS chuyen_khac_nhau,
       (SELECT count(*) FROM trips
         WHERE status IN ('completed', 'cancelled')
           AND requested_at >= '2026-10-01 00:00+07' AND requested_at < '2026-10-31 00:00+07') AS dem_doc_lap;

-- Mọi giao dịch đều thuộc một chuyến TRONG phạm vi (không có tiền "trôi nổi"
-- của chuyến đang chạy hay chuyến tháng khác).
-- expect-value: 0
SELECT count(*) FROM payments AS p
WHERE NOT EXISTS (SELECT 1 FROM so_doi_soat AS s WHERE s.id = p.trip_id);

-- ---------------------------------------------------------------------------
-- 2. ĐÁP ÁN: danh sách chênh lệch (expected output bàn giao).
-- expect-line: 100724 | 2026-10-07 | HN | ride   | completed |  76000 |      0 |  -76000 | 3 THIẾU THANH TOÁN  | (không có giao dịch)
-- expect-line: 101084 | 2026-10-11 | DN | food   | completed |  98000 |      0 |  -98000 | 3 THIẾU THANH TOÁN  | charge/card/failed/98000
-- expect-line: 101310 | 2026-10-13 | SG | food   | completed |  85000 | 170000 |   85000 | 4 THU TRÙNG         | charge/wallet/succeeded/85000, charge/wallet/succeeded/85000
-- expect-line: 101987 | 2026-10-19 | SG | ride   | cancelled |      0 | -72000 |  -72000 | 1 HOÀN KHÔNG CÓ THU | refund/wallet/succeeded/72000
-- expect-line: 102443 | 2026-10-24 | HN | parcel | completed |  54000 |  45000 |   -9000 | 5 LỆCH SỐ TIỀN      | charge/cash/succeeded/45000
-- expect-line: 102642 | 2026-10-25 | SG | ride   | completed |  38000 | 380000 |  342000 | 5 LỆCH SỐ TIỀN      | charge/card/succeeded/380000
-- expect-line: 102739 | 2026-10-26 | SG | food   | cancelled |      0 |  96000 |   96000 | 2 THU CHUYẾN HUỶ    | charge/wallet/succeeded/96000
SELECT s.code, s.ngay, c.code AS tp, s.kind, s.status,
       s.phai_thu, s.thuc_thu, s.chenh, s.loai_lech, s.chi_tiet
FROM so_doi_soat AS s
JOIN cities AS c ON c.id = s.city_id
WHERE s.loai_lech IS NOT NULL
ORDER BY s.code;

-- Theo loại: số chuyến và tổng chênh; dòng cuối cộng lại đúng bằng chênh ở mục 1.
-- expect-line: 1 HOÀN KHÔNG CÓ THU | 1 |  -72000
-- expect-line: 2 THU CHUYẾN HUỶ    | 1 |   96000
-- expect-line: 3 THIẾU THANH TOÁN  | 2 | -174000
-- expect-line: 4 THU TRÙNG         | 1 |   85000
-- expect-line: 5 LỆCH SỐ TIỀN      | 2 |  333000
-- expect-line: TỔNG                | 7 |  268000
SELECT coalesce(loai_lech, 'TỔNG') AS loai, count(*) AS so_chuyen, sum(chenh) AS tong_chenh
FROM so_doi_soat
WHERE loai_lech IS NOT NULL
GROUP BY ROLLUP (loai_lech)
ORDER BY grouping(loai_lech), loai_lech;

-- ---------------------------------------------------------------------------
-- 3. ĐỐI SOÁT SỐ DÒNG: mọi chuyến trong phạm vi rơi vào ĐÚNG MỘT nhóm, các
--    nhóm cộng lại = 3.168.
-- expect-line: KHỚP — hoàn thành, có thu                 | 2671 | 181941000
-- expect-line: KHỚP — chuyến huỷ, không giao dịch        |  447 |         0
-- expect-line: KHỚP — hoàn thành cước 0 đ, không cần thu |   31 |         0
-- expect-line: KHỚP — chuyến huỷ, đã hoàn đủ             |   12 |   1008000
-- expect-line: LỆCH                                      |    7 |    691000
-- expect-line: TỔNG                                      | 3168 | 183640000
SELECT coalesce(nhom, 'TỔNG') AS nhom, count(*) AS so_chuyen, sum(tong_thu) AS tong_tien_charge
FROM (
  SELECT tong_thu,
         CASE
           WHEN loai_lech IS NOT NULL                       THEN 'LỆCH'
           WHEN status = 'completed' AND phai_thu = 0       THEN 'KHỚP — hoàn thành cước 0 đ, không cần thu'
           WHEN status = 'completed'                        THEN 'KHỚP — hoàn thành, có thu'
           WHEN tong_hoan > 0                               THEN 'KHỚP — chuyến huỷ, đã hoàn đủ'
           ELSE                                                  'KHỚP — chuyến huỷ, không giao dịch'
         END AS nhom
  FROM so_doi_soat
) AS x
GROUP BY ROLLUP (nhom)
ORDER BY grouping(nhom), count(*) DESC;

-- ---------------------------------------------------------------------------
-- 4. Các ca KHÓ NHƯNG ĐÚNG — lý do quy tắc phải tính đủ refund, adjustment và
--    chỉ lấy `succeeded`. Không ca nào được báo lệch.
-- expect-line: 100042 | completed |  82000 |  82000 | NULL | charge/card/failed/82000, charge/card/succeeded/82000
-- expect-line: 100048 | completed | 119000 | 119000 | NULL | charge/wallet/succeeded/104000, charge/card/succeeded/15000
-- expect-line: 102235 | completed |  72000 |  72000 | NULL | charge/wallet/succeeded/82000, adjustment/wallet/succeeded/-10000
-- expect-line: 103162 | completed |  57000 |  57000 | NULL | charge/card/succeeded/57000, charge/card/pending/57000
-- expect-line: 103165 | completed |  28000 |  28000 | NULL | charge/cash/succeeded/23000, adjustment/wallet/succeeded/5000
SELECT code, status, phai_thu, thuc_thu, loai_lech, chi_tiet
FROM so_doi_soat
WHERE code IN (100042, 100048, 102235, 103162, 103165)
ORDER BY code;

-- Một chuyến huỷ đã trừ thẻ rồi hoàn đủ (charge + refund, thực thu 0).
-- expect-line: 100366 | cancelled | 0 | 0 | NULL | charge/card/succeeded/103000, refund/card/succeeded/103000
SELECT code, status, phai_thu, thuc_thu, loai_lech, chi_tiet
FROM so_doi_soat
WHERE status = 'cancelled' AND tong_hoan > 0 AND loai_lech IS NULL
ORDER BY code
LIMIT 1;

-- Quy tắc NGÂY THƠ (so tổng MỌI charge, bất kể trạng thái, bỏ qua refund và
-- adjustment) báo nhầm hàng trăm chuyến — đối soát sai còn tệ hơn không đối soát.
-- expect-line: 115 | 7
SELECT count(*) FILTER (WHERE ngay_tho <> phai_thu) AS quy_tac_ngay_tho,
       count(*) FILTER (WHERE loai_lech IS NOT NULL) AS quy_tac_dung
FROM (
  SELECT s.phai_thu, s.loai_lech,
         coalesce((SELECT sum(p.amount) FROM payments AS p
                   WHERE p.trip_id = s.id AND p.kind = 'charge'), 0) AS ngay_tho
  FROM so_doi_soat AS s
) AS x;

-- ---------------------------------------------------------------------------
-- 5. CHÊNH LỆCH THỜI ĐIỂM — không phải lỗi: gom tiền theo NGÀY GIAO DỊCH thì
--    tiền của chuyến đặt cuối ngày rơi sang ngày hôm sau. Theo ngày đặt chuyến
--    và theo ngày giao dịch (paid_at, giờ VN), hai ngày quanh 15/10:
-- expect-line: 2026-10-15 | 5862000 | 5823000
-- expect-line: 2026-10-16 | 5244000 | 5228000
SELECT ngay,
       (SELECT sum(s.thuc_thu) FROM so_doi_soat AS s WHERE s.ngay = d.ngay) AS theo_ngay_dat_chuyen,
       (SELECT sum(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END)
          FROM payments AS p
         WHERE p.status = 'succeeded'
           AND (p.paid_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = d.ngay)   AS theo_ngay_giao_dich
FROM (VALUES (DATE '2026-10-15'), (DATE '2026-10-16')) AS d (ngay)
ORDER BY ngay;

-- Cả tháng: điều chỉnh +5.000 đ của chuyến 103165 (đặt 30/10) được CSKH ghi
-- lúc 00:05 ngày 31/10 ⇒ nằm ngoài "tháng 10 theo ngày giao dịch".
-- expect-line: 182555000 | 182550000 | 5000
SELECT (SELECT sum(thuc_thu) FROM so_doi_soat) AS theo_ngay_dat_chuyen,
       (SELECT sum(CASE kind WHEN 'refund' THEN -amount ELSE amount END) FROM payments
         WHERE status = 'succeeded'
           AND paid_at >= '2026-10-01 00:00+07' AND paid_at < '2026-10-31 00:00+07') AS theo_ngay_giao_dich_thang_10,
       (SELECT sum(CASE kind WHEN 'refund' THEN -amount ELSE amount END) FROM payments
         WHERE status = 'succeeded' AND paid_at >= '2026-10-31 00:00+07')            AS ghi_nhan_sau_31_10;

-- ---------------------------------------------------------------------------
-- 6. VIỆC CẦN THEO DÕI (không đổi tiền, nhưng phải có người xử lý): giao dịch
--    còn `pending` sau khi chuyến đã hoàn thành hơn 1 giờ (tính tới lúc chụp
--    dữ liệu 2026-10-31 01:00).
-- expect-line: 103162 | TH-103162-P | 57000 | 2026-10-30 22:17:37+07
SELECT t.code, p.provider_ref, p.amount, p.paid_at
FROM payments AS p
JOIN trips AS t ON t.id = p.trip_id
WHERE p.status = 'pending'
  AND t.completed_at < TIMESTAMPTZ '2026-10-31 01:00+07' - interval '1 hour'
ORDER BY t.code;

DROP VIEW so_doi_soat;
