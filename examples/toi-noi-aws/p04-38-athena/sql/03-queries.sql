-- Các truy vấn mẫu trên trips_curated (bảng Parquet đã partition theo dt, city).

-- 1) Doanh thu theo thành phố và giờ trong một ngày. Có lọc dt nên chỉ quét 6 partition của ngày đó.
SELECT city,
       hour(requested_at) AS hour_of_day,
       count(*)           AS trips,
       sum(fare)          AS revenue
FROM trips_curated
WHERE dt = '2026-03-02'
  AND status = 'completed'
GROUP BY city, hour(requested_at)
ORDER BY city, hour_of_day;

-- 2) Giờ cao điểm 11:00-13:00 và 17:00-19:00 của Hà Nội trong một tuần.
SELECT dt, hour(requested_at) AS hour_of_day, count(*) AS trips
FROM trips_curated
WHERE dt BETWEEN '2026-03-02' AND '2026-03-08'
  AND city = 'hn'
  AND hour(requested_at) IN (11, 12, 17, 18)
GROUP BY dt, hour(requested_at)
ORDER BY dt, hour_of_day;

-- 3) KHÔNG làm thế này: không lọc dt nên quét toàn bộ partition và mọi cột được chọn.
-- Workgroup sẽ huỷ câu lệnh nếu vượt bytes_scanned_cutoff_per_query.
SELECT *
FROM trips_curated
WHERE status = 'completed';

-- 4) Cùng câu hỏi nhưng chọn ít cột: Parquet là cột nên chỉ đọc các cột được nhắc tới.
SELECT city, sum(fare) AS revenue
FROM trips_curated
WHERE dt >= '2026-03-01' AND dt < '2026-04-01'
GROUP BY city;
