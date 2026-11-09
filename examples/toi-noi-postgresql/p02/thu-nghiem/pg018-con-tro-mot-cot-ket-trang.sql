-- PG-018 · Con trỏ một cột với `<=` KẸT trang khi một lô cùng giây dài hơn cỡ
-- trang; với `<` thì mất cả phần còn lại của lô. Cỡ trang 3, lô 10/10 09:00 có
-- 6 chuyến (100971…100976).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- Trang bắt đầu ở lô 10/10 (con trỏ một cột: requested_at <= 10/10 09:00).
-- expect-line: 100976 | 2026-10-10 09:00:00+07
-- expect-line: 100975 | 2026-10-10 09:00:00+07
-- expect-line: 100974 | 2026-10-10 09:00:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND requested_at <= '2026-10-10 09:00:00+07'
ORDER BY requested_at DESC, code DESC
LIMIT 3;

-- Dòng cuối có requested_at = 10/10 09:00 ⇒ con trỏ KHÔNG đổi ⇒ trang "sau"
-- y hệt trang vừa rồi. Bấm "Xem thêm" mãi vẫn ra 3 chuyến này.
-- expect-line: 100976 | 2026-10-10 09:00:00+07
-- expect-line: 100975 | 2026-10-10 09:00:00+07
-- expect-line: 100974 | 2026-10-10 09:00:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND requested_at <= '2026-10-10 09:00:00+07'
ORDER BY requested_at DESC, code DESC
LIMIT 3;

-- Đổi sang `<`: nhảy qua cả lô, MẤT 100973, 100972, 100971.
-- expect-line: 100669 | 2026-10-07 09:20:00+07
-- expect-line: 100500 | 2026-10-05 15:40:00+07
-- expect-line: 100130 | 2026-10-02 10:15:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND requested_at < '2026-10-10 09:00:00+07'
ORDER BY requested_at DESC, code DESC
LIMIT 3;

-- Con trỏ hai cột (requested_at, code) từ dòng cuối 100974: đi tiếp đúng.
-- expect-line: 100973 | 2026-10-10 09:00:00+07
-- expect-line: 100972 | 2026-10-10 09:00:00+07
-- expect-line: 100971 | 2026-10-10 09:00:00+07
SELECT code, requested_at
FROM trips
WHERE customer_id = 'cb000000-0000-4000-8000-000000000001'
  AND (requested_at, code) < ('2026-10-10 09:00:00+07', 100974)
ORDER BY requested_at DESC, code DESC
LIMIT 3;
