-- P02 · 15-window-function.sql — PG-015 `postgresql-window-function-frame`
-- Window function và cửa sổ tính toán: LUỸ KẾ, XẾP HẠNG, SO SÁNH VỚI DÒNG
-- TRƯỚC — với frame (khung) ghi rõ.
--
-- Hàm cửa sổ tính trên một tập dòng "xung quanh" dòng hiện tại mà KHÔNG gộp
-- các dòng lại như GROUP BY. Ba phần của OVER (…):
--   PARTITION BY  — chia nhóm (mỗi nhóm tính riêng)
--   ORDER BY      — thứ tự trong nhóm
--   frame         — dòng nào trong nhóm được đưa vào phép tính. Có ORDER BY mà
--                   không ghi frame thì mặc định là
--                   RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
--                   — "mọi dòng tới dòng hiện tại VÀ mọi dòng hoà với nó".

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. Doanh thu theo ngày của Đà Nẵng, quanh ngày bão 20/10 (0 chuyến).
--    Lịch đủ 30 ngày từ generate_series ⇒ ngày bão có dòng doanh thu 0.
--    (Viết DATE + số nguyên để ra kiểu date; generate_series(date, date,
--    interval) trả timestamptz — xem PG-016.)
--    Hàm cửa sổ tính trên CẢ THÁNG ở truy vấn trong; truy vấn ngoài mới lọc
--    các ngày cần in (cửa sổ được tính SAU WHERE của cùng cấp truy vấn).
-- expect-line: 2026-10-17 | 13 | 718000 | 15952000 | 927429 | 927429
-- expect-line: 2026-10-18 | 16 | 945000 | 16897000 | 859857 | 859857
-- expect-line: 2026-10-19 | 11 | 637000 | 17534000 | 828571 | 828571
-- expect-line: 2026-10-20 |  0 |      0 | 17534000 | 672571 | NULL
-- expect-line: 2026-10-21 | 14 | 872000 | 18406000 | 677286 | 797143
-- expect-line: 2026-10-22 | 11 | 785000 | 19191000 | 679000 | 789429
-- expect-line: 2026-10-23 | 13 | 941000 | 20132000 | 699714 | 813429
-- expect-line: 2026-10-24 | 14 | 787000 | 20919000 | 709571 | 812143
SELECT ngay, so_chuyen, doanh_thu, luy_ke, tb7_lich_day, tb7_rows_thua
FROM (
  SELECT l.ngay,
         coalesce(d.so_chuyen, 0)  AS so_chuyen,
         coalesce(d.doanh_thu, 0)  AS doanh_thu,
         -- luỹ kế: frame ghi rõ ROWS … (không phụ thuộc mặc định RANGE)
         sum(coalesce(d.doanh_thu, 0)) OVER (ORDER BY l.ngay
                                             ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS luy_ke,
         -- trung bình 7 NGÀY trên lịch đầy đủ: 6 dòng trước + dòng hiện tại = 7 ngày
         round(avg(coalesce(d.doanh_thu, 0)) OVER (ORDER BY l.ngay
                                                   ROWS BETWEEN 6 PRECEDING AND CURRENT ROW)) AS tb7_lich_day,
         -- SAI: cùng frame ROWS nhưng trên dữ liệu THƯA (không có dòng ngày bão)
         --      ⇒ "7 dòng" trải trên 8 ngày lịch quanh 20/10, và ngày bão không được tính.
         round(d.tb7_rows_thua)  AS tb7_rows_thua
  FROM (SELECT DATE '2026-10-01' + n AS ngay FROM generate_series(0, 29) AS n) AS l
  LEFT JOIN (
    SELECT ngay, so_chuyen, doanh_thu,
           avg(doanh_thu) OVER (ORDER BY ngay ROWS BETWEEN 6 PRECEDING AND CURRENT ROW) AS tb7_rows_thua
    FROM (SELECT (t.requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS ngay,
                 count(*) AS so_chuyen, sum(t.fare) AS doanh_thu
          FROM trips AS t
          WHERE t.city_id = 3 AND t.status = 'completed'
          GROUP BY 1) AS theo_ngay
  ) AS d ON d.ngay = l.ngay
) AS x
WHERE ngay BETWEEN DATE '2026-10-17' AND DATE '2026-10-24'
ORDER BY ngay;

-- Trên dữ liệu thưa, frame RANGE theo KHOẢNG THỜI GIAN mới đúng "7 ngày lịch":
-- RANGE BETWEEN interval '6 days' PRECEDING AND CURRENT ROW. Nó khớp tb7 trên
-- lịch đầy đủ ở mọi ngày CÓ dữ liệu — trừ ngày 20/10, vốn không có dòng nào.
-- expect-value: 0
SELECT count(*) AS so_ngay_lech
FROM (
  SELECT l.ngay,
         round(avg(coalesce(d.doanh_thu, 0)) OVER (ORDER BY l.ngay ROWS BETWEEN 6 PRECEDING AND CURRENT ROW)) AS a,
         d.doanh_thu IS NOT NULL AS co_du_lieu,
         round(d.tb7_range) AS b
  FROM (SELECT DATE '2026-10-01' + n AS ngay FROM generate_series(0, 29) AS n) AS l
  LEFT JOIN (
    SELECT ngay, doanh_thu,
           -- chia 7 vì ngày không có dòng phải tính là 0 đ
           sum(doanh_thu) OVER (ORDER BY ngay RANGE BETWEEN interval '6 days' PRECEDING AND CURRENT ROW)
             / least(7, ngay - DATE '2026-10-01' + 1)::numeric AS tb7_range
    FROM (SELECT (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date AS ngay, sum(fare) AS doanh_thu
          FROM trips WHERE city_id = 3 AND status = 'completed' GROUP BY 1) AS s
  ) AS d ON d.ngay = l.ngay
) AS so_sanh
WHERE co_du_lieu AND a <> b;

-- ---------------------------------------------------------------------------
-- 2. Frame mặc định khi THỨ TỰ CÓ HOÀ. 18 chuyến của khách doanh nghiệp, hai
--    lô đặt cùng giây. ORDER BY requested_at (không có frame) ⇒ RANGE: mọi
--    chuyến cùng giây nhận CÙNG giá trị luỹ kế (cả lô). ROWS + khoá phụ `code`
--    ⇒ tăng từng dòng, thứ tự xác định.
-- expect-md5: 7b5c0303318f869bf806e8f5d9ec50f0
SELECT code, requested_at, fare,
       count(*)  OVER (ORDER BY requested_at)                         AS dem_mac_dinh,
       sum(fare) OVER (ORDER BY requested_at)                         AS luy_ke_mac_dinh,
       count(*)  OVER (ORDER BY requested_at, code ROWS UNBOUNDED PRECEDING) AS dem_rows,
       sum(fare) OVER (ORDER BY requested_at, code ROWS UNBOUNDED PRECEDING) AS luy_ke_rows
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at, code;

-- ---------------------------------------------------------------------------
-- 3. Xếp hạng tài xế theo số chuyến hoàn thành trong từng thành phố. Có hoà:
--    row_number đánh số liên tục (hoà thì tuỳ khoá phụ), rank nhảy cóc sau
--    nhóm hoà (1,1,3), dense_rank không nhảy (1,1,2). Lọc "top 3" phải làm ở
--    truy vấn NGOÀI — hàm cửa sổ không dùng được trong WHERE.
-- expect-error: 42P20
SELECT d.code, rank() OVER (ORDER BY count(*) DESC) AS hang
FROM trips AS t JOIN drivers AS d ON d.id = t.driver_id
WHERE t.status = 'completed' AND rank() OVER (ORDER BY count(*) DESC) <= 3
GROUP BY d.code;

-- row_number cần khoá phụ (d.code) để thứ tự xác định; rank/dense_rank chỉ
-- xếp theo số chuyến nên hai tài xế cùng 68 chuyến cùng hạng 1. Lọc
-- dense_rank <= 3 có thể trả HƠN 3 người (DN: 8302 và 8310 cùng 40 chuyến).
-- expect-line: DN | 8305 | 44 | 1 | 1 | 1
-- expect-line: DN | 8304 | 41 | 2 | 2 | 2
-- expect-line: DN | 8302 | 40 | 3 | 3 | 3
-- expect-line: DN | 8310 | 40 | 4 | 3 | 3
-- expect-line: HN | 8210 | 63 | 1 | 1 | 1
-- expect-line: HN | 8216 | 60 | 2 | 2 | 2
-- expect-line: HN | 8205 | 59 | 3 | 3 | 3
-- expect-line: SG | 8101 | 68 | 1 | 1 | 1
-- expect-line: SG | 8109 | 68 | 2 | 1 | 1
-- expect-line: SG | 8120 | 66 | 3 | 3 | 2
-- expect-line: SG | 8108 | 64 | 4 | 4 | 3
-- expect-line: SG | 8124 | 64 | 5 | 4 | 3
SELECT tp, tai_xe, hoan_thanh, so_thu_tu, hang, hang_lien_tuc
FROM (
  SELECT c.code AS tp, d.code AS tai_xe, count(*) AS hoan_thanh,
         row_number() OVER (PARTITION BY c.code ORDER BY count(*) DESC, d.code) AS so_thu_tu,
         rank()       OVER w AS hang,
         dense_rank() OVER w AS hang_lien_tuc
  FROM trips AS t
  JOIN drivers AS d ON d.id = t.driver_id
  JOIN cities  AS c ON c.id = d.city_id
  WHERE t.status = 'completed'
  GROUP BY c.code, d.code
  WINDOW w AS (PARTITION BY c.code ORDER BY count(*) DESC)
) AS x
WHERE hang_lien_tuc <= 3
ORDER BY tp COLLATE "C", so_thu_tu;

-- ---------------------------------------------------------------------------
-- 4. So sánh với chuyến TRƯỚC của cùng khách: lag() lấy giá trị của dòng đứng
--    trước trong cửa sổ (dòng đầu không có ⇒ NULL). Khách 0910000016 — người
--    đi nhiều nhất — 6 chuyến đầu tháng.
-- expect-line: 100008 | 2026-10-01 06:12:00+07 | ride | NULL | NULL           | 1
-- expect-line: 100149 | 2026-10-02 12:45:00+07 | ride | ride | 1 day 06:33:00 | 2
-- expect-line: 100215 | 2026-10-03 07:04:00+07 | ride | ride | 18:19:00       | 3
-- expect-line: 100259 | 2026-10-03 13:02:00+07 | ride | ride | 05:58:00       | 4
-- expect-line: 100277 | 2026-10-03 17:01:00+07 | ride | ride | 03:59:00       | 5
-- expect-line: 100369 | 2026-10-04 12:07:00+07 | ride | ride | 19:06:00       | 6
SELECT t.code, t.requested_at, t.kind,
       lag(t.kind) OVER w                         AS dich_vu_truoc,
       t.requested_at - lag(t.requested_at) OVER w AS cach_chuyen_truoc,
       row_number() OVER w                        AS chuyen_thu
FROM trips AS t
JOIN customers AS c ON c.id = t.customer_id
WHERE c.phone = '0910000016'
WINDOW w AS (ORDER BY t.requested_at, t.code)
ORDER BY t.requested_at, t.code
LIMIT 6;

-- BẪY last_value: với frame mặc định (tới dòng hiện tại), "giá trị cuối" chính
-- là dòng hiện tại. Muốn chuyến cuối tháng phải mở frame tới cuối nhóm.
-- expect-line: 100008 | 100008 | 100008 | 103123
-- expect-line: 100149 | 100008 | 100149 | 103123
-- expect-line: 100215 | 100008 | 100215 | 103123
-- expect-line: 100259 | 100008 | 100259 | 103123
SELECT t.code,
       first_value(t.code) OVER (ORDER BY t.requested_at, t.code)        AS chuyen_dau,
       last_value(t.code)  OVER (ORDER BY t.requested_at, t.code)        AS last_value_SAI,
       last_value(t.code)  OVER (ORDER BY t.requested_at, t.code
                                 ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS chuyen_cuoi
FROM trips AS t
JOIN customers AS c ON c.id = t.customer_id
WHERE c.phone = '0910000016'
ORDER BY t.requested_at, t.code
LIMIT 4;

-- ---------------------------------------------------------------------------
-- 5. Tỉ trọng: cửa sổ RỖNG `OVER ()` = toàn bộ kết quả. Kết hợp với GROUP BY:
--    hàm cửa sổ chạy SAU khi gom nhóm, trên các dòng đã gom.
-- expect-line: SG | 92880000 | 182287000 | 50.95
-- expect-line: HN | 62991000 | 182287000 | 34.56
-- expect-line: DN | 26416000 | 182287000 | 14.49
SELECT c.code AS tp,
       sum(t.fare)                                              AS doanh_thu,
       sum(sum(t.fare)) OVER ()                                 AS tong,
       round(100.0 * sum(t.fare) / sum(sum(t.fare)) OVER (), 2) AS ty_trong_pct
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY doanh_thu DESC;
