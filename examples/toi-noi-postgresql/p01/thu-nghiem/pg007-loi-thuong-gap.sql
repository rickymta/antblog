SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- E1: so sánh với NULL
SELECT code, status FROM trips WHERE driver_id = NULL;
SELECT code, status FROM trips WHERE driver_id IS NULL ORDER BY code;

-- E2: ưu tiên AND/OR
SELECT code, status, city_id FROM trips
WHERE status = 'completed' OR status = 'cancelled' AND city_id = 1
ORDER BY code;
SELECT code, status, city_id FROM trips
WHERE (status = 'completed' OR status = 'cancelled') AND city_id = 1
ORDER BY code;

-- E3: NULLS mặc định khi DESC
SELECT code, status, accepted_at
FROM trips
WHERE requested_at >= '2026-11-03 08:00:00+07'
ORDER BY accepted_at DESC, code;

-- E4: BETWEEN và biên phải
SELECT code, requested_at FROM trips
WHERE requested_at BETWEEN '2026-11-02 00:00:00+07' AND '2026-11-03 00:00:00+07'
ORDER BY requested_at, code;

-- E4b
SELECT timestamptz '2026-11-03 00:00:00+07'
         BETWEEN '2026-11-02 00:00:00+07' AND '2026-11-03 00:00:00+07' AS thuoc_ngay_02,
       timestamptz '2026-11-03 00:00:00+07'
         BETWEEN '2026-11-03 00:00:00+07' AND '2026-11-04 00:00:00+07' AS thuoc_ngay_03;

-- E5: chuỗi không múi giờ hiểu theo phiên
SET TIME ZONE 'UTC';
SELECT count(*) AS dem_khong_ghi_mui_gio FROM trips
WHERE requested_at >= '2026-11-02 00:00:00' AND requested_at < '2026-11-03 00:00:00';
SELECT count(*) AS dem_ghi_ro_07 FROM trips
WHERE requested_at >= '2026-11-02 00:00:00+07' AND requested_at < '2026-11-03 00:00:00+07';
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- E6: alias dùng được trong ORDER BY nhưng không dùng được trong WHERE
-- expect-error: 42703
SELECT code, fare / 1000 AS nghin_dong FROM trips WHERE nghin_dong > 60 ORDER BY code;
SELECT code, fare / 1000 AS nghin_dong FROM trips WHERE fare > 60000 ORDER BY nghin_dong DESC, code;

-- Bài tập 1: chuyến đồ ăn ở SG hoàn thành, cước >= 50000, sắp cước giảm dần, tie code
SELECT code, city_id, fare, completed_at FROM trips
WHERE kind = 'food' AND status = 'completed' AND fare >= 50000
ORDER BY fare DESC, code;

-- Bài tập 2: chuyến đặt từ 12:00 đến trước 18:00 ngày 02/11 giờ VN
SELECT code, kind, requested_at FROM trips
WHERE requested_at >= '2026-11-02 12:00:00+07' AND requested_at < '2026-11-02 18:00:00+07'
ORDER BY requested_at, code;

-- Bài tập 3: 2 chuyến cước cao nhất chưa xong (không completed/cancelled)
SELECT code, status, fare FROM trips
WHERE status NOT IN ('completed', 'cancelled')
ORDER BY fare DESC, code
LIMIT 2;

-- Trang 3 theo kind, code
SELECT code, kind, status FROM trips ORDER BY kind, code LIMIT 3 OFFSET 6;
SELECT code, kind, status FROM trips ORDER BY kind, code LIMIT 3 OFFSET 9;
SELECT count(*) FROM trips;
