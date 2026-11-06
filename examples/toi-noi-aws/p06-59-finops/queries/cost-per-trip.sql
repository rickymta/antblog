-- Athena trên CUR 2.0 (Data Exports) và bảng chuyến của lakehouse (bài 38).
-- Tên database/bảng/cột dưới đây là GIẢ ĐỊNH; kiểm schema thật bằng DESCRIBE trước khi chạy.
-- Dùng element_at(map, key) thay cho map[key]: trong Athena (Trino), map[key] báo lỗi khi khoá vắng,
-- còn element_at trả NULL, nên dòng thiếu tag không làm hỏng cả truy vấn.

-- 1. Khoá tag nào đang có trong map resource_tags? (tên khoá phụ thuộc cách export đặt)
SELECT DISTINCT k AS tag_key
FROM cur_db.cur2
CROSS JOIN UNNEST(map_keys(resource_tags)) AS t(k)
WHERE line_item_usage_start_date >= date_add('day', -7, current_date)
ORDER BY 1;

-- 2. Chi phí theo service và theo tháng cho tài nguyên Project=ToiNoi.
SELECT date_trunc('month', line_item_usage_start_date) AS month,
       element_at(resource_tags, 'user_service')                   AS service,
       sum(line_item_unblended_cost)                   AS cost
FROM cur_db.cur2
WHERE element_at(resource_tags, 'user_project') = 'ToiNoi'
  AND line_item_usage_start_date >= date_add('month', -3, date_trunc('month', current_date))
GROUP BY 1, 2
ORDER BY 1, 3 DESC;

-- 3. Chi phí mỗi chuyến theo tháng.
WITH cost AS (
  SELECT date_trunc('month', line_item_usage_start_date) AS month,
         sum(line_item_unblended_cost)                   AS cost
  FROM cur_db.cur2
  WHERE element_at(resource_tags, 'user_project') = 'ToiNoi'
  GROUP BY 1
), trips AS (
  SELECT date_trunc('month', trip_date) AS month,
         sum(trip_count)                AS trips
  FROM analytics.trips_daily
  GROUP BY 1
)
SELECT c.month, c.cost, t.trips, c.cost / t.trips AS cost_per_trip
FROM cost c
JOIN trips t ON c.month = t.month
ORDER BY c.month;

-- 4. Những dòng chi phí KHÔNG có tag Service: nơi báo cáo bị mù.
SELECT line_item_product_code, sum(line_item_unblended_cost) AS cost
FROM cur_db.cur2
WHERE element_at(resource_tags, 'user_project') = 'ToiNoi'
  AND element_at(resource_tags, 'user_service') IS NULL
  AND line_item_usage_start_date >= date_add('month', -1, date_trunc('month', current_date))
GROUP BY 1
ORDER BY 2 DESC;
