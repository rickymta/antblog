-- P01 · 00-reset.sql — dựng lại schema lab `toinoi` từ đầu.
--
-- Mọi bảng của lab nằm trong schema riêng `toinoi`, không đụng vào `public`
-- hay dữ liệu khác trong database. Chạy lại file này bất cứ lúc nào để về
-- trạng thái trống; sau đó chạy tiếp 01 → 07.
--
-- CẢNH BÁO: DROP SCHEMA ... CASCADE xoá MỌI đối tượng trong schema `toinoi`.
-- Chỉ chạy trên database lab.

DROP SCHEMA IF EXISTS toinoi CASCADE;

CREATE SCHEMA toinoi;

COMMENT ON SCHEMA toinoi IS 'Lab PostgreSQL qua dự án Tới Nơi — dữ liệu hư cấu, có thể xoá và dựng lại';

-- search_path quyết định tên bảng không ghi schema (vd. `trips`) được tìm ở đâu.
-- Thiết lập này chỉ sống trong phiên hiện tại, nên mỗi file sau đều đặt lại.
SET search_path TO toinoi;

-- Múi giờ phiên: mọi timestamptz được HIỂN THỊ theo giờ Việt Nam (+07).
-- Giá trị lưu trong bảng không phụ thuộc thiết lập này (xem 05-select.sql).
SET TIME ZONE 'Asia/Ho_Chi_Minh';

SELECT current_schema() AS schema_hien_tai,
       current_setting('TimeZone') AS mui_gio_phien,
       current_setting('server_version') AS phien_ban;
