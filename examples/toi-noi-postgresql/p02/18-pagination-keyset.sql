-- P02 · 18-pagination-keyset.sql — PG-018 `postgresql-pagination-keyset`
-- Phân trang lịch sử chuyến bằng keyset: xử lý KHOÁ SẮP XẾP TRÙNG và chứng
-- minh điều kiện trang kế tiếp.
--
-- Màn "Lịch sử chuyến" của khách doanh nghiệp "Cửa hàng Phụ Kiện Lab" (KHDN,
-- cb000000-…-000000000001): 18 chuyến bưu kiện, mới nhất trước, 5 chuyến một
-- trang. Hai lô đặt qua API có CÙNG giờ đặt đến từng giây:
--   10/10 09:00:00 — 6 chuyến (100971…100976) · 18/10 14:30:00 — 4 chuyến (101882…101885)
-- Cả hai lô đều nằm VẮT qua ranh giới trang.
--
-- Keyset (seek) pagination: thay vì "bỏ qua N dòng" (OFFSET), trang sau bắt
-- đầu NGAY SAU dòng cuối của trang trước, theo một khoá sắp xếp DUY NHẤT.
-- requested_at không duy nhất ⇒ khoá là (requested_at, code).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. Toàn bộ lịch sử theo thứ tự hiển thị, kèm vị trí và trang (để đối chiếu).
-- expect-line:  1 | 1 | 102897 | 2026-10-28 10:30:00+07
-- expect-line:  2 | 1 | 102611 | 2026-10-25 14:10:00+07
-- expect-line:  3 | 1 | 102149 | 2026-10-21 08:50:00+07
-- expect-line:  4 | 1 | 101885 | 2026-10-18 14:30:00+07
-- expect-line:  5 | 1 | 101884 | 2026-10-18 14:30:00+07
-- expect-line:  6 | 2 | 101883 | 2026-10-18 14:30:00+07
-- expect-line:  7 | 2 | 101882 | 2026-10-18 14:30:00+07
-- expect-line:  8 | 2 | 101555 | 2026-10-15 16:45:00+07
-- expect-line:  9 | 2 | 101224 | 2026-10-12 11:05:00+07
-- expect-line: 10 | 2 | 100976 | 2026-10-10 09:00:00+07
-- expect-line: 11 | 3 | 100975 | 2026-10-10 09:00:00+07
-- expect-line: 12 | 3 | 100974 | 2026-10-10 09:00:00+07
-- expect-line: 13 | 3 | 100973 | 2026-10-10 09:00:00+07
-- expect-line: 14 | 3 | 100972 | 2026-10-10 09:00:00+07
-- expect-line: 15 | 3 | 100971 | 2026-10-10 09:00:00+07
-- expect-line: 16 | 4 | 100669 | 2026-10-07 09:20:00+07
-- expect-line: 17 | 4 | 100500 | 2026-10-05 15:40:00+07
-- expect-line: 18 | 4 | 100130 | 2026-10-02 10:15:00+07
SELECT row_number() OVER w                AS vi_tri,
       (row_number() OVER w - 1) / 5 + 1  AS trang,
       code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
WINDOW w AS (ORDER BY requested_at DESC, code DESC)
ORDER BY requested_at DESC, code DESC;

-- ---------------------------------------------------------------------------
-- 2. Trang 1. Ứng dụng giữ lại "con trỏ" = khoá của dòng CUỐI trang:
--    (2026-10-18 14:30:00+07, 101884).
-- expect-line: 102897 | 2026-10-28 10:30:00+07
-- expect-line: 102611 | 2026-10-25 14:10:00+07
-- expect-line: 102149 | 2026-10-21 08:50:00+07
-- expect-line: 101885 | 2026-10-18 14:30:00+07
-- expect-line: 101884 | 2026-10-18 14:30:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at DESC, code DESC
LIMIT 5;

-- ---------------------------------------------------------------------------
-- 3. Trang 2 — SAI kiểu 1: con trỏ chỉ có requested_at, so sánh `<`.
--    Hai chuyến còn lại của lô 18/10 (101883, 101882) bị BỎ MẤT vĩnh viễn.
-- expect-line: 101555 | 2026-10-15 16:45:00+07
-- expect-line: 101224 | 2026-10-12 11:05:00+07
-- expect-line: 100976 | 2026-10-10 09:00:00+07
-- expect-line: 100975 | 2026-10-10 09:00:00+07
-- expect-line: 100974 | 2026-10-10 09:00:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND requested_at < '2026-10-18 14:30:00+07'
ORDER BY requested_at DESC, code DESC
LIMIT 5;

-- Trang 2 — SAI kiểu 2: `<=` thì HIỆN LẠI 101885, 101884 của trang 1. Tệ
-- hơn: nếu một lô cùng giây dài hơn cỡ trang, `<=` trả mãi cùng một trang.
-- expect-line: 101885 | 2026-10-18 14:30:00+07
-- expect-line: 101884 | 2026-10-18 14:30:00+07
-- expect-line: 101883 | 2026-10-18 14:30:00+07
-- expect-line: 101882 | 2026-10-18 14:30:00+07
-- expect-line: 101555 | 2026-10-15 16:45:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND requested_at <= '2026-10-18 14:30:00+07'
ORDER BY requested_at DESC, code DESC
LIMIT 5;

-- ---------------------------------------------------------------------------
-- 4. Trang 2 — ĐÚNG: so sánh BỘ GIÁ TRỊ (row comparison) theo đúng khoá sắp
--    xếp. (a, b) < (x, y) nghĩa là a < x, HOẶC a = x và b < y — đúng thứ tự từ
--    điển, cùng chiều với ORDER BY a DESC, b DESC.
-- expect-line: 101883 | 2026-10-18 14:30:00+07
-- expect-line: 101882 | 2026-10-18 14:30:00+07
-- expect-line: 101555 | 2026-10-15 16:45:00+07
-- expect-line: 101224 | 2026-10-12 11:05:00+07
-- expect-line: 100976 | 2026-10-10 09:00:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at, code) < ('2026-10-18 14:30:00+07', 101884)
ORDER BY requested_at DESC, code DESC
LIMIT 5;

-- Trang 3 từ con trỏ (2026-10-10 09:00:00+07, 100976): bắt đầu đúng ở giữa lô 10/10.
-- expect-line: 100975 | 2026-10-10 09:00:00+07
-- expect-line: 100974 | 2026-10-10 09:00:00+07
-- expect-line: 100973 | 2026-10-10 09:00:00+07
-- expect-line: 100972 | 2026-10-10 09:00:00+07
-- expect-line: 100971 | 2026-10-10 09:00:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at, code) < ('2026-10-10 09:00:00+07', 100976)
ORDER BY requested_at DESC, code DESC
LIMIT 5;

-- Dạng viết tách tương đương (dùng khi hệ khác không hỗ trợ so sánh bộ):
-- expect-value: 0
SELECT count(*) FROM (
  (SELECT code FROM trips
   WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
     AND (requested_at, code) < ('2026-10-18 14:30:00+07', 101884)
   EXCEPT
   SELECT code FROM trips
   WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
     AND (requested_at < '2026-10-18 14:30:00+07'
          OR (requested_at = '2026-10-18 14:30:00+07' AND code < 101884)))
  UNION ALL
  (SELECT code FROM trips
   WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
     AND (requested_at < '2026-10-18 14:30:00+07'
          OR (requested_at = '2026-10-18 14:30:00+07' AND code < 101884))
   EXCEPT
   SELECT code FROM trips
   WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
     AND (requested_at, code) < ('2026-10-18 14:30:00+07', 101884))
) AS lech;

-- ---------------------------------------------------------------------------
-- 5. Hai cột sắp NGƯỢC CHIỀU (giờ đặt mới trước, nhưng trong cùng giây mã
--    NHỎ trước) thì so sánh bộ không dùng được — phải viết tách điều kiện.
--    Con trỏ: dòng cuối trang 1 theo thứ tự này là (2026-10-18 14:30, 101883).
-- expect-line: 101884 | 2026-10-18 14:30:00+07
-- expect-line: 101885 | 2026-10-18 14:30:00+07
-- expect-line: 101555 | 2026-10-15 16:45:00+07
-- expect-line: 101224 | 2026-10-12 11:05:00+07
-- expect-line: 100971 | 2026-10-10 09:00:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at < '2026-10-18 14:30:00+07'
       OR (requested_at = '2026-10-18 14:30:00+07' AND code > 101883))
ORDER BY requested_at DESC, code ASC
LIMIT 5;

-- ---------------------------------------------------------------------------
-- 6. CHỨNG MINH điều kiện trang kế tiếp: đi hết mọi trang bằng recursive CTE
--    (PG-019 giải thích cú pháp) — mỗi bước lấy 5 dòng sau con trỏ của bước
--    trước. Đúng khi: hợp các trang = toàn bộ lịch sử, KHÔNG dòng nào lặp,
--    không dòng nào thiếu.
-- expect-line: 1 | {102897,102611,102149,101885,101884}
-- expect-line: 2 | {101883,101882,101555,101224,100976}
-- expect-line: 3 | {100975,100974,100973,100972,100971}
-- expect-line: 4 | {100669,100500,100130}
WITH RECURSIVE trang (so_trang, ma, con_tro_at, con_tro_code) AS (
  SELECT 1, p.ma, p.ats[cardinality(p.ats)], p.ma[cardinality(p.ma)]
  FROM (SELECT array_agg(code ORDER BY requested_at DESC, code DESC) AS ma,
               array_agg(requested_at ORDER BY requested_at DESC, code DESC) AS ats
        FROM (SELECT code, requested_at FROM trips
              WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
              ORDER BY requested_at DESC, code DESC LIMIT 5) AS dau) AS p
  UNION ALL
  SELECT tr.so_trang + 1, nx.ma, nx.ats[cardinality(nx.ats)], nx.ma[cardinality(nx.ma)]
  FROM trang AS tr
  CROSS JOIN LATERAL (
    SELECT array_agg(code ORDER BY requested_at DESC, code DESC) AS ma,
           array_agg(requested_at ORDER BY requested_at DESC, code DESC) AS ats
    FROM (SELECT code, requested_at FROM trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at, code) < (tr.con_tro_at, tr.con_tro_code)
          ORDER BY requested_at DESC, code DESC LIMIT 5) AS sau
  ) AS nx
  WHERE nx.ma IS NOT NULL          -- hết dòng ⇒ array_agg trả NULL ⇒ dừng
)
SELECT so_trang, ma FROM trang ORDER BY so_trang;

-- Bằng chứng: 18 mã qua các trang, 18 mã khác nhau, đúng bằng số chuyến.
-- expect-line: 4 | 18 | 18 | 18
WITH RECURSIVE trang (so_trang, ma, con_tro_at, con_tro_code) AS (
  SELECT 1, p.ma, p.ats[cardinality(p.ats)], p.ma[cardinality(p.ma)]
  FROM (SELECT array_agg(code ORDER BY requested_at DESC, code DESC) AS ma,
               array_agg(requested_at ORDER BY requested_at DESC, code DESC) AS ats
        FROM (SELECT code, requested_at FROM trips
              WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
              ORDER BY requested_at DESC, code DESC LIMIT 5) AS dau) AS p
  UNION ALL
  SELECT tr.so_trang + 1, nx.ma, nx.ats[cardinality(nx.ats)], nx.ma[cardinality(nx.ma)]
  FROM trang AS tr
  CROSS JOIN LATERAL (
    SELECT array_agg(code ORDER BY requested_at DESC, code DESC) AS ma,
           array_agg(requested_at ORDER BY requested_at DESC, code DESC) AS ats
    FROM (SELECT code, requested_at FROM trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (requested_at, code) < (tr.con_tro_at, tr.con_tro_code)
          ORDER BY requested_at DESC, code DESC LIMIT 5) AS sau
  ) AS nx
  WHERE nx.ma IS NOT NULL
)
SELECT (SELECT count(*) FROM trang)                                   AS so_trang,
       (SELECT count(*) FROM trang, unnest(ma) AS m)                  AS tong_ma_qua_cac_trang,
       (SELECT count(DISTINCT m) FROM trang, unnest(ma) AS m)         AS ma_khac_nhau,
       (SELECT count(*) FROM trips
        WHERE customer_id = 'cb000000-0000-4000-8000-000000000001')   AS so_chuyen;

-- ---------------------------------------------------------------------------
-- 7. OFFSET trôi khi dữ liệu thay đổi giữa hai lần tải trang. Khách xem trang
--    1, trong lúc đó đặt thêm một chuyến; trang 2 bằng OFFSET 5 lặp lại
--    101884 (vừa thấy ở trang 1). Keyset không bị ảnh hưởng. (Giao dịch được
--    ROLLBACK; sequence mã chuyến vẫn tiêu mất một số — PG-006.)
BEGIN;

-- expect-line: 103174
INSERT INTO trips (kind, city_id, customer_id, status, fare, requested_at)
VALUES ('parcel', 1, 'cb000000-0000-4000-8000-000000000001', 'requested', 30000, '2026-10-31 00:59:00+07')
RETURNING code;

-- expect-line: 101884 | 2026-10-18 14:30:00+07
-- expect-line: 101883 | 2026-10-18 14:30:00+07
-- expect-line: 101882 | 2026-10-18 14:30:00+07
-- expect-line: 101555 | 2026-10-15 16:45:00+07
-- expect-line: 101224 | 2026-10-12 11:05:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY requested_at DESC, code DESC
LIMIT 5 OFFSET 5;

-- expect-line: 101883 | 2026-10-18 14:30:00+07
-- expect-line: 101882 | 2026-10-18 14:30:00+07
-- expect-line: 101555 | 2026-10-15 16:45:00+07
-- expect-line: 101224 | 2026-10-12 11:05:00+07
-- expect-line: 100976 | 2026-10-10 09:00:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at, code) < ('2026-10-18 14:30:00+07', 101884)
ORDER BY requested_at DESC, code DESC
LIMIT 5;

ROLLBACK;

-- expect-value: 18
SELECT count(*) FROM trips WHERE customer_id = 'cb000000-0000-4000-8000-000000000001';
