-- PG-018 · Trang TRƯỚC và cờ "còn trang sau" với keyset. Thứ tự hiển thị
-- (requested_at DESC, code DESC), 5 chuyến/trang.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Đang ở trang 3; dòng ĐẦU trang 3 là (2026-10-10 09:00, 100975). Trang trước
-- = 5 dòng đứng NGAY TRƯỚC nó: đảo dấu so sánh (>) và đảo chiều sắp (ASC) để
-- lấy 5 dòng gần nhất, rồi sắp lại theo thứ tự hiển thị ở truy vấn ngoài.
-- expect-line: 101883 | 2026-10-18 14:30:00+07
-- expect-line: 101882 | 2026-10-18 14:30:00+07
-- expect-line: 101555 | 2026-10-15 16:45:00+07
-- expect-line: 101224 | 2026-10-12 11:05:00+07
-- expect-line: 100976 | 2026-10-10 09:00:00+07
SELECT code, requested_at
FROM (SELECT code, requested_at
      FROM trips
      WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
        AND (requested_at, code) > ('2026-10-10 09:00:00+07', 100975)
      ORDER BY requested_at ASC, code ASC
      LIMIT 5) AS truoc
ORDER BY requested_at DESC, code DESC;

-- Quên đảo chiều sắp (vẫn DESC): lấy 5 dòng MỚI NHẤT trong mọi dòng phía
-- trước — tức trang 1 — chứ không phải trang 2.
-- expect-line: 102897 | 2026-10-28 10:30:00+07
-- expect-line: 102611 | 2026-10-25 14:10:00+07
-- expect-line: 102149 | 2026-10-21 08:50:00+07
-- expect-line: 101885 | 2026-10-18 14:30:00+07
-- expect-line: 101884 | 2026-10-18 14:30:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at, code) > ('2026-10-10 09:00:00+07', 100975)
ORDER BY requested_at DESC, code DESC
LIMIT 5;

-- "Còn trang sau không?": xin n + 1 = 6 dòng, hiển thị 5, dòng thứ 6 chỉ để
-- biết còn. Sau con trỏ của trang 2 (100976) còn ≥ 6 dòng ⇒ có trang 3.
-- expect-value: 6
SELECT count(*) FROM (
  SELECT 1 FROM trips
  WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
    AND (requested_at, code) < ('2026-10-10 09:00:00+07', 100976)
  ORDER BY requested_at DESC, code DESC
  LIMIT 6) AS x;

-- Sau con trỏ của trang 3 (100971) chỉ còn 3 dòng ⇒ trang 4 là trang cuối.
-- expect-value: 3
SELECT count(*) FROM (
  SELECT 1 FROM trips
  WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
    AND (requested_at, code) < ('2026-10-10 09:00:00+07', 100971)
  ORDER BY requested_at DESC, code DESC
  LIMIT 6) AS x;

-- Bài tập 2: trang trước của trang 2 (dòng đầu trang 2 = (18/10 14:30, 101883))
-- là trang 1 — chính hai chuyến cùng giây 101885, 101884 phải nằm ở đó.
-- expect-line: 102897 | 2026-10-28 10:30:00+07
-- expect-line: 102611 | 2026-10-25 14:10:00+07
-- expect-line: 102149 | 2026-10-21 08:50:00+07
-- expect-line: 101885 | 2026-10-18 14:30:00+07
-- expect-line: 101884 | 2026-10-18 14:30:00+07
SELECT code, requested_at
FROM (SELECT code, requested_at
      FROM trips
      WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
        AND (requested_at, code) > ('2026-10-18 14:30:00+07', 101883)
      ORDER BY requested_at ASC, code ASC
      LIMIT 5) AS truoc
ORDER BY requested_at DESC, code DESC;

-- Trước trang 1 không còn gì: câu "trang trước" trả 0 dòng ⇒ ẩn nút.
-- expect-rows: 0
SELECT code
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at, code) > ('2026-10-28 10:30:00+07', 102897)
ORDER BY requested_at ASC, code ASC
LIMIT 5;
