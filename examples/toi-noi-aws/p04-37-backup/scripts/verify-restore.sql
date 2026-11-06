-- Kiểm toàn vẹn sau restore: chạy trên DB đã restore, rồi so kết quả với DB nguồn (hoặc với số liệu đã ghi lại lúc backup).
-- Dùng trong bước diễn tập của bài 37; thay khoảng thời gian cho phù hợp với recovery point.

-- 1) Số dòng và mốc thời gian mới nhất: cho biết dữ liệu tới thời điểm nào (đầu vào cho RPO thực tế).
SELECT count(*) AS trips_rows,
       max(updated_at) AS latest_update
FROM trips;

-- 2) Đối soát theo ngày và thành phố cho 7 ngày gần nhất.
SELECT date_trunc('day', requested_at) AS dt,
       city_id,
       count(*) AS trips,
       sum(fare) AS fare_sum
FROM trips
WHERE requested_at >= now() - interval '7 days'
GROUP BY 1, 2
ORDER BY 1, 2;

-- 3) Checksum theo ngày: hash các dòng đã sắp thứ tự, so với nguồn.
SELECT date_trunc('day', requested_at) AS dt,
       md5(string_agg(id::text || ':' || status || ':' || fare::text, ',' ORDER BY id)) AS checksum
FROM trips
WHERE requested_at >= now() - interval '2 days'
GROUP BY 1
ORDER BY 1;

-- 4) Ràng buộc còn nguyên: mọi chuyến completed phải có completed_at.
SELECT count(*) AS bad_completed
FROM trips
WHERE status = 'completed' AND completed_at IS NULL;
