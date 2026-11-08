-- P01 · 05-select.sql — SELECT đầu tiên: lọc, sắp xếp, giới hạn (PG-007).
--
-- Chạy sau 02 và 04. Dữ liệu: 15 chuyến của fixture + chuyến 100029 ở 04.
-- File này chỉ ĐỌC (trừ các lệnh SET của phiên).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- 1. Lọc theo một trạng thái. Chỉ lấy cột cần dùng thay vì SELECT *.
-- expect-rows: 8
SELECT code, kind, city_id, fare, completed_at
FROM trips
WHERE status = 'completed'
ORDER BY code;

-- 2. Lọc theo danh sách trạng thái: chuyến đang có tài xế trên đường.
-- expect-rows: 4
SELECT code, kind, status, requested_at
FROM trips
WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
ORDER BY requested_at, code;

-- 3. Chuyến đặt trong NGÀY 02/11 theo giờ Việt Nam: khoảng nửa mở
--    [00:00 ngày 02/11, 00:00 ngày 03/11) với mốc ghi rõ +07.
--    Không dùng BETWEEN … '23:59:59' (bỏ sót 23:59:59.5) và không dùng hàm trên
--    cột (khó dùng index — P03).
-- expect-rows: 8
SELECT code, kind, status, requested_at
FROM trips
WHERE requested_at >= '2026-11-02 00:00:00+07'
  AND requested_at <  '2026-11-03 00:00:00+07'
ORDER BY requested_at, code;

-- 4. timestamptz lưu một KHOẢNH KHẮC; TimeZone của phiên chỉ đổi cách HIỂN THỊ
--    và cách hiểu chuỗi không ghi múi giờ. Đổi phiên sang UTC để thấy rõ.
SET TIME ZONE 'UTC';

-- Cùng hai chuyến, giờ hiển thị đổi sang +00. Chuyến 100008 (00:10 ngày 03/11
-- giờ VN) giờ hiện là 17:10 ngày 02/11.
-- expect-rows: 2
SELECT code, requested_at
FROM trips
WHERE code IN (100007, 100008)
ORDER BY code;

-- Cái bẫy: ép kiểu `::date` dùng TimeZone của PHIÊN. Ở phiên UTC, "ngày 02/11"
-- gom thêm chuyến 100008 ⇒ 9 thay vì 8. Câu thứ hai nói rõ múi giờ nên luôn
-- đúng bất kể phiên đang đặt gì.
SELECT count(*) FILTER (WHERE requested_at::date = DATE '2026-11-02') AS theo_mui_gio_phien,
       count(*) FILTER (WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = DATE '2026-11-02')
         AS theo_gio_viet_nam
FROM trips;

SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- 5. LIMIT: ba chuyến đặt gần nhất. Thứ tự mới → cũ, hoà giờ thì mã lớn trước —
--    `code` là cột duy nhất nên thứ tự luôn xác định.
-- expect-rows: 3
SELECT code, kind, status, requested_at
FROM trips
ORDER BY requested_at DESC, code DESC
LIMIT 3;

-- 6. Thứ tự KHÔNG xác định. 100003 và 100004 cùng đặt lúc 12:00. Chỉ
--    ORDER BY requested_at thì PostgreSQL được trả hai dòng hoà theo thứ tự
--    bất kỳ; thứ tự đang thấy dưới đây là ngẫu nhiên may mắn, không phải cam kết.
-- expect-rows: 2
SELECT code, city_id, requested_at
FROM trips
WHERE requested_at = '2026-11-02 12:00:00+07'
ORDER BY requested_at;

--    Hậu quả cụ thể: phân trang. Màn hình "chuyến theo loại dịch vụ", mỗi
--    trang 3 dòng, sắp theo `kind` — cột có rất nhiều giá trị trùng (5 chuyến
--    food). Với LIMIT nhỏ, PostgreSQL dùng thuật toán "top-N" chỉ giữ N dòng
--    tốt nhất, nên các dòng hoà ra theo thứ tự khác với khi sắp toàn bộ.
-- Trang 1:
-- expect-rows: 3
SELECT code, kind, status
FROM trips
ORDER BY kind
LIMIT 3;

-- Trang 2: 100002 và 100004 xuất hiện LẦN NỮA, trong khi 100009 và 100013
-- (cũng là food) không nằm ở trang nào trong hai trang đầu.
-- expect-rows: 3
SELECT code, kind, status
FROM trips
ORDER BY kind
LIMIT 3 OFFSET 3;

-- Cách sửa: thêm cột duy nhất làm tie-breaker ở cuối ORDER BY ⇒ mọi dòng có
-- một vị trí xác định, hai trang không chồng lên nhau. (Phân trang lịch sử
-- dài bằng keyset thay cho OFFSET: PG-018.)
-- expect-rows: 3
SELECT code, kind, status
FROM trips
ORDER BY kind, code
LIMIT 3;

-- expect-rows: 3
SELECT code, kind, status
FROM trips
ORDER BY kind, code
LIMIT 3 OFFSET 3;

-- 7. NULL khi sắp xếp: mặc định NULL được coi là LỚN NHẤT — đứng CUỐI khi ASC,
--    đứng ĐẦU khi DESC. Viết rõ NULLS FIRST / NULLS LAST khi điều đó quan trọng.
-- expect-rows: 7
SELECT code, status, accepted_at
FROM trips
WHERE requested_at >= '2026-11-03 08:00:00+07'
ORDER BY accepted_at DESC NULLS LAST, code;

-- 8. Sắp xếp chữ tiếng Việt phụ thuộc COLLATION của database. PGlite dùng "C"
--    (so theo mã byte UTF-8 ⇒ 'Đ' xếp sau 'V'); ảnh postgres:17.5 trong
--    docker-compose.yml khởi tạo với en_US.utf8 nên thứ tự có thể khác. Ghi rõ
--    COLLATE "C" để kết quả lab giống nhau trên mọi môi trường; muốn xếp theo
--    thói quen người Việt cần collation phù hợp (ICU) — ngoài phạm vi P01.
SELECT full_name
FROM customers
ORDER BY full_name COLLATE "C";
