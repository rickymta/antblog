-- PG-020 · Bài tập mở rộng (không có trong lab). Dựng lại VIEW tạm so_doi_soat
-- NGUYÊN VĂN như p02/20-doi-soat-thanh-toan.sql mục 0, rồi làm ba bài tập.
-- Nền: p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql
SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

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
-- Bài 1. Đối soát theo thành phố: ba dòng + tổng, tổng phải khớp mục 1 của lab.
-- expect-line: DN   |  459 |  26416000 |  26318000 |  -98000 | 1
-- expect-line: HN   | 1106 |  62991000 |  62906000 |  -85000 | 2
-- expect-line: SG   | 1603 |  92880000 |  93331000 |  451000 | 4
-- expect-line: TỔNG | 3168 | 182287000 | 182555000 |  268000 | 7
SELECT coalesce(c.code, 'TỔNG') AS tp, count(*) AS so_chuyen,
       sum(s.phai_thu) AS phai_thu, sum(s.thuc_thu) AS thuc_thu, sum(s.chenh) AS chenh,
       count(*) FILTER (WHERE s.loai_lech IS NOT NULL) AS so_lech
FROM so_doi_soat AS s
JOIN cities AS c ON c.id = s.city_id
GROUP BY ROLLUP (c.code)
ORDER BY grouping(c.code), c.code COLLATE "C";

-- ---------------------------------------------------------------------------
-- Bài 2. CSKH giảm 10.000 đ cho chuyến 100048 (cước 119.000) — ba cách ghi.
-- (a) chỉ ghi một refund 10.000 vào ví
BEGIN;
-- expect-rows: 1
INSERT INTO payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
SELECT id, 'refund', 'wallet', 'succeeded', 10000, 'VX-100048-R', '2026-10-01 15:00+07' FROM trips WHERE code = 100048;
-- expect-line: 100048 | 119000 | 109000 | -10000 | 5 LỆCH SỐ TIỀN
SELECT code, phai_thu, thuc_thu, chenh, loai_lech FROM so_doi_soat WHERE code = 100048;
-- expect-line: 8 | 258000
SELECT count(*) FILTER (WHERE loai_lech IS NOT NULL), sum(chenh) FROM so_doi_soat;
ROLLBACK;

-- (b) chỉ ghi adjustment −10.000 (đúng loại giao dịch, nhưng cước vẫn 119.000)
BEGIN;
-- expect-rows: 1
INSERT INTO payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
SELECT id, 'adjustment', 'wallet', 'succeeded', -10000, 'VX-100048-A', '2026-10-01 15:00+07' FROM trips WHERE code = 100048;
-- expect-line: 100048 | 119000 | 109000 | -10000 | 5 LỆCH SỐ TIỀN
SELECT code, phai_thu, thuc_thu, chenh, loai_lech FROM so_doi_soat WHERE code = 100048;
ROLLBACK;

-- (c) sửa cước còn 109.000 VÀ ghi adjustment −10.000 (cách của ca 102235)
BEGIN;
-- expect-rows: 1
UPDATE trips SET fare = 109000 WHERE code = 100048;
-- expect-rows: 1
INSERT INTO payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
SELECT id, 'adjustment', 'wallet', 'succeeded', -10000, 'VX-100048-A', '2026-10-01 15:00+07' FROM trips WHERE code = 100048;
-- expect-line: 100048 | 109000 | 109000 | 0 | NULL
SELECT code, phai_thu, thuc_thu, chenh, loai_lech FROM so_doi_soat WHERE code = 100048;
-- expect-line: 7 | 268000
SELECT count(*) FILTER (WHERE loai_lech IS NOT NULL), sum(chenh) FROM so_doi_soat;
ROLLBACK;

-- ---------------------------------------------------------------------------
-- Bài 3. Siết tín hiệu THU TRÙNG: hai charge thành công cùng số tiền, CÙNG phương
-- thức, cách nhau không quá 60 giây. So với tín hiệu của lab trên MỌI chuyến trong
-- phạm vi (kể cả chuyến khớp).
-- expect-line: 101310 | wallet | 85000 | 00:00:03 | t | t
WITH thu_ok AS (
  SELECT p.trip_id, p.id, p.method, p.amount, p.paid_at
  FROM payments AS p
  WHERE p.kind = 'charge' AND p.status = 'succeeded'
)
SELECT s.code, b.method, b.amount, b.paid_at - a.paid_at AS cach_nhau,
       true AS tin_hieu_lab,
       (b.method = a.method AND b.paid_at - a.paid_at <= interval '60 seconds') AS tin_hieu_chat
FROM thu_ok AS a
JOIN thu_ok AS b ON b.trip_id = a.trip_id AND b.amount = a.amount AND (b.paid_at, b.id) > (a.paid_at, a.id)
JOIN so_doi_soat AS s ON s.id = a.trip_id
ORDER BY s.code;

-- Đáp án 7 dòng không đổi khi thay tín hiệu (chỉ 101310 có cặp charge trùng tiền).
-- expect-value: 7
SELECT count(*) FROM so_doi_soat WHERE loai_lech IS NOT NULL;

-- Ca phân biệt được hai tín hiệu: thẻ của 100048 bị trừ thêm 15.000 HAI NGÀY sau.
-- Tín hiệu lab gọi là THU TRÙNG; tín hiệu chặt thì không (rơi về LỆCH SỐ TIỀN).
BEGIN;
-- expect-rows: 1
INSERT INTO payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
SELECT id, 'charge', 'card', 'succeeded', 15000, 'TH-100048-3', '2026-10-03 09:00+07' FROM trips WHERE code = 100048;
-- expect-line: 100048 | 15000 | 4 THU TRÙNG | 1 day 19:30:02 | f
WITH thu_ok AS (
  SELECT p.trip_id, p.id, p.method, p.amount, p.paid_at
  FROM payments AS p
  WHERE p.kind = 'charge' AND p.status = 'succeeded'
)
SELECT s.code, s.chenh, s.loai_lech, b.paid_at - a.paid_at AS cach_nhau,
       (b.method = a.method AND b.paid_at - a.paid_at <= interval '60 seconds') AS tin_hieu_chat
FROM thu_ok AS a
JOIN thu_ok AS b ON b.trip_id = a.trip_id AND b.amount = a.amount AND (b.paid_at, b.id) > (a.paid_at, a.id)
JOIN so_doi_soat AS s ON s.id = a.trip_id
WHERE s.code = 100048;
ROLLBACK;
