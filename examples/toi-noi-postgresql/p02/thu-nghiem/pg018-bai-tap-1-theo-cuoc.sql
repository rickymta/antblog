-- PG-018 · Bài tập 1: lịch sử chuyến KHDN sắp theo CƯỚC giảm dần (nhiều chuyến
-- trùng cước), 5 chuyến/trang. Khoá sắp duy nhất: (fare, code).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Trang 1. Con trỏ = (57000, 101555).
-- expect-line: 100975 | 78000
-- expect-line: 100500 | 75000
-- expect-line: 102611 | 61000
-- expect-line: 100976 | 61000
-- expect-line: 101555 | 57000
SELECT code, fare
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
ORDER BY fare DESC, code DESC
LIMIT 5;

-- Trang 2 đúng.
-- expect-line: 100669 | 57000
-- expect-line: 101883 | 53000
-- expect-line: 100972 | 53000
-- expect-line: 101885 | 49000
-- expect-line: 100974 | 49000
SELECT code, fare
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (fare, code) < (57000, 101555)
ORDER BY fare DESC, code DESC
LIMIT 5;

-- Con trỏ chỉ có cước, so `<`: mất 100669 (cùng 57.000 đ với dòng cuối trang 1).
-- expect-line: 101883 | 53000
-- expect-line: 100972 | 53000
-- expect-line: 101885 | 49000
-- expect-line: 100974 | 49000
-- expect-line: 100130 | 45000
SELECT code, fare
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND fare < 57000
ORDER BY fare DESC, code DESC
LIMIT 5;

-- Chứng minh: đi hết các trang — 4 trang, 18 mã, 18 mã khác nhau.
-- expect-line: 4 | 18 | 18
WITH RECURSIVE trang (so_trang, ma, con_tro_fare, con_tro_code) AS (
  SELECT 1, p.ma, p.fares[cardinality(p.fares)], p.ma[cardinality(p.ma)]
  FROM (SELECT array_agg(code ORDER BY fare DESC, code DESC) AS ma,
               array_agg(fare ORDER BY fare DESC, code DESC) AS fares
        FROM (SELECT code, fare FROM trips
              WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
              ORDER BY fare DESC, code DESC LIMIT 5) AS dau) AS p
  UNION ALL
  SELECT tr.so_trang + 1, nx.ma, nx.fares[cardinality(nx.fares)], nx.ma[cardinality(nx.ma)]
  FROM trang AS tr
  CROSS JOIN LATERAL (
    SELECT array_agg(code ORDER BY fare DESC, code DESC) AS ma,
           array_agg(fare ORDER BY fare DESC, code DESC) AS fares
    FROM (SELECT code, fare FROM trips
          WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
            AND (fare, code) < (tr.con_tro_fare, tr.con_tro_code)
          ORDER BY fare DESC, code DESC LIMIT 5) AS sau
  ) AS nx
  WHERE nx.ma IS NOT NULL
)
SELECT (SELECT count(*) FROM trang)                           AS so_trang,
       (SELECT count(*) FROM trang, unnest(ma) AS m)          AS tong_ma,
       (SELECT count(DISTINCT m) FROM trang, unnest(ma) AS m) AS ma_khac_nhau;
