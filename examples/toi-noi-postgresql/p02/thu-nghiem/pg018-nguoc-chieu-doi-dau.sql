-- PG-018 · Hai cột ngược chiều viết lại thành so sánh bộ bằng cách ĐỔI DẤU cột
-- số: ORDER BY requested_at DESC, code ASC  ≡  ORDER BY requested_at DESC, -code DESC.
-- Cho cùng kết quả với dạng tách điều kiện của 18-pagination-keyset.sql mục 5.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- expect-line: 101884 | 2026-10-18 14:30:00+07
-- expect-line: 101885 | 2026-10-18 14:30:00+07
-- expect-line: 101555 | 2026-10-15 16:45:00+07
-- expect-line: 101224 | 2026-10-12 11:05:00+07
-- expect-line: 100971 | 2026-10-10 09:00:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at, -code) < ('2026-10-18 14:30:00+07', -101883)
ORDER BY requested_at DESC, -code DESC
LIMIT 5;
