-- P01 · 07-checks.sql — truy vấn kiểm tra bất biến dữ liệu (PG-010).
--
-- Mỗi truy vấn tìm các dòng VI PHẠM một quy tắc; kết quả đúng là 0 dòng
-- (`-- expect-rows: 0`). Có hai nhóm:
--   A. Quy tắc schema đã chặn (FK, CHECK) — kiểm lại vẫn có ích khi ràng buộc
--      bị thêm với NOT VALID, bị tắt khi nhập dữ liệu, hoặc để làm bằng chứng
--      bàn giao.
--   B. Quy tắc schema hiện KHÔNG chặn được bằng CHECK một dòng (cần so nhiều
--      dòng hoặc nhiều bảng) — đây là chỗ dữ liệu sai lọt vào thật.
-- Chạy được bất cứ lúc nào sau 02; trong `node run.mjs p01` nó chạy cuối cùng.
-- Các truy vấn dùng JOIN / NOT EXISTS / GROUP BY — P02 giải thích kỹ.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ===========================================================================
-- A. Quy tắc đã có ràng buộc
-- ===========================================================================

-- A1. Chuyến trỏ tới khách / tài xế / quán / thành phố không tồn tại (mồ côi).
-- expect-rows: 0
SELECT t.code, 'customer' AS thieu
FROM trips AS t
WHERE NOT EXISTS (SELECT 1 FROM customers AS c WHERE c.id = t.customer_id)
UNION ALL
SELECT t.code, 'driver'
FROM trips AS t
WHERE t.driver_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM drivers AS d WHERE d.id = t.driver_id)
UNION ALL
SELECT t.code, 'merchant'
FROM trips AS t
WHERE t.merchant_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM merchants AS m WHERE m.id = t.merchant_id)
UNION ALL
SELECT t.code, 'city'
FROM trips AS t
WHERE NOT EXISTS (SELECT 1 FROM cities AS ci WHERE ci.id = t.city_id);

-- A2. Dòng lịch sử trỏ tới chuyến không tồn tại.
-- expect-rows: 0
SELECT e.id, e.trip_id
FROM trip_events AS e
WHERE NOT EXISTS (SELECT 1 FROM trips AS t WHERE t.id = e.trip_id);

-- A3. Trạng thái và mốc thời gian lệch nhau (lặp lại logic các CHECK của trips).
-- expect-rows: 0
SELECT code, status, driver_id IS NOT NULL AS co_tai_xe, accepted_at, completed_at
FROM trips
WHERE (status IN ('requested', 'searching') AND driver_id IS NOT NULL)
   OR (status IN ('accepted', 'arriving', 'picked_up', 'in_transit', 'completed') AND driver_id IS NULL)
   OR ((driver_id IS NULL) <> (accepted_at IS NULL))
   OR ((status = 'completed') <> (completed_at IS NOT NULL))
   OR accepted_at < requested_at
   OR completed_at < accepted_at;

-- A4. Trùng khoá nghiệp vụ (UNIQUE đã chặn; truy vấn này dùng khi nhập dữ
--     liệu từ nguồn chưa có ràng buộc).
-- expect-rows: 0
SELECT phone, count(*) FROM customers GROUP BY phone HAVING count(*) > 1
UNION ALL
SELECT code::text, count(*) FROM drivers GROUP BY code HAVING count(*) > 1;

-- ===========================================================================
-- B. Quy tắc nhiều dòng / nhiều bảng — CHECK một dòng không diễn đạt được
-- ===========================================================================

-- B1. Tài xế chạy chuyến ở thành phố khác thành phố đăng ký.
-- expect-rows: 0
SELECT t.code, t.city_id AS thanh_pho_chuyen, d.code AS tai_xe, d.city_id AS thanh_pho_tai_xe
FROM trips AS t
JOIN drivers AS d ON d.id = t.driver_id
WHERE d.city_id <> t.city_id;

-- B2. Đơn đồ ăn đặt ở quán thuộc thành phố khác.
-- expect-rows: 0
SELECT t.code, t.city_id AS thanh_pho_chuyen, m.name, m.city_id AS thanh_pho_quan
FROM trips AS t
JOIN merchants AS m ON m.id = t.merchant_id
WHERE m.city_id <> t.city_id;

-- B3. Một tài xế cùng lúc có hơn một chuyến đang chạy.
-- expect-rows: 0
SELECT d.code, count(*) AS so_chuyen_dang_chay
FROM trips AS t
JOIN drivers AS d ON d.id = t.driver_id
WHERE t.status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
GROUP BY d.code
HAVING count(*) > 1;

-- B4. Trạng thái tài xế lệch với chuyến: có chuyến đang chạy mà không `busy`,
--     hoặc `busy` mà không có chuyến nào đang chạy.
-- expect-rows: 0
SELECT d.code, d.status, a.so_chuyen_dang_chay
FROM drivers AS d
LEFT JOIN (
  SELECT driver_id, count(*) AS so_chuyen_dang_chay
  FROM trips
  WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
  GROUP BY driver_id
) AS a ON a.driver_id = d.id
WHERE (a.driver_id IS NOT NULL AND d.status <> 'busy')
   OR (a.driver_id IS NULL AND d.status = 'busy');

-- B5. updated_at phải không sớm hơn mốc mới nhất của chính dòng đó.
--     (greatest() bỏ qua NULL.)
-- expect-rows: 0
SELECT code, status, updated_at, greatest(requested_at, accepted_at, completed_at) AS moc_moi_nhat
FROM trips
WHERE updated_at < greatest(requested_at, accepted_at, completed_at);

-- B6. Nghi trùng chuyến: cùng khách, cùng loại, đặt cách nhau dưới 1 phút và cả
--     hai còn đang chạy — dấu hiệu khách bấm "Đặt" hai lần (P04 · idempotency).
-- expect-rows: 0
SELECT a.code AS chuyen_1, b.code AS chuyen_2, a.customer_id
FROM trips AS a
JOIN trips AS b
  ON  b.customer_id = a.customer_id
  AND b.kind = a.kind
  AND b.code > a.code
  AND b.requested_at - a.requested_at < interval '1 minute'
  AND a.requested_at - b.requested_at < interval '1 minute'
WHERE a.status NOT IN ('completed', 'cancelled')
  AND b.status NOT IN ('completed', 'cancelled');

-- B7. Lịch sử không khớp chuyến (chỉ xét chuyến CÓ lịch sử; 15 chuyến fixture
--     cố ý không nạp lịch sử): dòng lịch sử sớm hơn giờ đặt, dòng đầu không
--     phải `requested`, hoặc dòng mới nhất khác trạng thái hiện tại.
-- expect-rows: 0
SELECT t.code, t.status, f.status AS su_kien_dau, l.status AS su_kien_cuoi
FROM trips AS t
JOIN LATERAL (
  SELECT e.status, e.occurred_at FROM trip_events AS e
  WHERE e.trip_id = t.id ORDER BY e.occurred_at, e.id LIMIT 1
) AS f ON true
JOIN LATERAL (
  SELECT e.status FROM trip_events AS e
  WHERE e.trip_id = t.id ORDER BY e.occurred_at DESC, e.id DESC LIMIT 1
) AS l ON true
WHERE f.occurred_at < t.requested_at
   OR f.status <> 'requested'
   OR l.status <> t.status;

-- ===========================================================================
-- C. Bằng chứng về chính schema
-- ===========================================================================

-- C1. Ràng buộc chưa VALIDATE (thêm bằng NOT VALID rồi bỏ quên).
-- expect-rows: 0
SELECT conrelid::regclass AS bang, conname
FROM pg_constraint
WHERE connamespace = 'toinoi'::regnamespace AND NOT convalidated;

-- C2. Ràng buộc đặt tên sai quy ước pk_/fk_/uq_/ck_.
--     Lưu ý: PG 17 không ghi NOT NULL vào pg_constraint (PG 18 mới ghi), nên
--     NOT NULL không xuất hiện ở đây.
-- expect-rows: 0
SELECT conrelid::regclass AS bang, conname, contype
FROM pg_constraint
WHERE connamespace = 'toinoi'::regnamespace
  AND conname !~ '^(pk|fk|uq|ck)_';

-- C3. Kiểm kê ràng buộc theo bảng và loại (p = primary key, f = foreign key,
--     u = unique, c = check).
SELECT conrelid::regclass AS bang,
       count(*) FILTER (WHERE contype = 'p') AS p,
       count(*) FILTER (WHERE contype = 'f') AS f,
       count(*) FILTER (WHERE contype = 'u') AS u,
       count(*) FILTER (WHERE contype = 'c') AS c
FROM pg_constraint
WHERE connamespace = 'toinoi'::regnamespace
GROUP BY conrelid
ORDER BY conrelid::regclass::text COLLATE "C";

-- C4. Số dòng cuối cùng của từng bảng.
-- expect-rows: 1
SELECT (SELECT count(*) FROM cities)      AS cities,
       (SELECT count(*) FROM customers)   AS customers,
       (SELECT count(*) FROM drivers)     AS drivers,
       (SELECT count(*) FROM merchants)   AS merchants,
       (SELECT count(*) FROM trips)       AS trips,
       (SELECT count(*) FROM trip_events) AS trip_events;
