-- Đối soát nguồn và đích: chạy CÙNG câu lệnh trên cả hai database rồi so kết quả.
-- Nên chạy trên một cửa sổ dữ liệu đã ổn định (ví dụ ngày hôm qua), không chạy trên partition đang bị ghi.

-- 1) Số dòng toàn bảng (nhanh với bảng nhỏ; với trips 730 triệu dòng hãy đối soát theo ngày thay vì toàn bảng).
SELECT 'cities' AS tbl, count(*) FROM cities
UNION ALL SELECT 'customers', count(*) FROM customers
UNION ALL SELECT 'drivers', count(*) FROM drivers
UNION ALL SELECT 'merchants', count(*) FROM merchants;

-- 2) Số dòng và tổng giá theo ngày và thành phố cho một khoảng ngày.
SELECT date_trunc('day', requested_at)::date AS dt,
       city_id,
       count(*)  AS trips,
       sum(fare) AS fare_sum
FROM trips
WHERE requested_at >= date '2026-03-01' AND requested_at < date '2026-03-08'
GROUP BY 1, 2
ORDER BY 1, 2;

-- 3) Checksum theo ngày: hash các dòng đã sắp thứ tự. Khác nhau nghĩa là có dòng lệch.
SELECT date_trunc('day', requested_at)::date AS dt,
       md5(string_agg(id::text || ':' || status || ':' || fare::text || ':' || coalesce(driver_id::text, ''),
                      ',' ORDER BY id)) AS checksum
FROM trips
WHERE requested_at >= date '2026-03-01' AND requested_at < date '2026-03-03'
GROUP BY 1
ORDER BY 1;

-- 4) Mốc mới nhất của dữ liệu đang được nhân bản (so nguồn với đích để thấy độ trễ ứng dụng).
SELECT max(updated_at) AS latest_trip_update FROM trips;
