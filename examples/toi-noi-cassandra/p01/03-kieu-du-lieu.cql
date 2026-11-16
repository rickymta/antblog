-- P01 · 03-kieu-du-lieu (CS-004): kiểu dữ liệu cho tiền, thời gian, chuỗi và định danh.
-- Bảng thử nghiệm type_lab nằm riêng, không thuộc mô hình; bị xoá ở cuối file.
USE toinoi;

CREATE TABLE IF NOT EXISTS type_lab (
  id        int PRIMARY KEY,
  as_float  float,
  as_double double,
  as_decimal decimal,
  as_bigint bigint,
  as_int    int,
  at_ts     timestamp,
  on_date   date,
  at_time   time,
  label     text,
  code      ascii
);

-- ## Tiền: float/double cộng 0,1 + 0,2 không ra 0,3; bigint (đồng) và decimal thì đúng
INSERT INTO type_lab (id, as_float, as_double, as_decimal, as_bigint) VALUES (1, 0.1, 0.1, 0.1, 100);
INSERT INTO type_lab (id, as_float, as_double, as_decimal, as_bigint) VALUES (2, 0.2, 0.2, 0.2, 200);
-- cqlsh làm tròn số thực khi hiển thị (float_precision = 5), nên ép sang text để thấy giá trị thật.
SELECT CAST(SUM(as_float) AS text) AS tong_float, CAST(SUM(as_double) AS text) AS tong_double,
       CAST(SUM(as_decimal) AS text) AS tong_decimal, SUM(as_bigint) AS tong_dong
FROM type_lab WHERE id IN (1, 2);

-- ## Tiền: giá trị float lưu được bao nhiêu chữ số? (cước 123.456.789 đồng)
INSERT INTO type_lab (id, as_float, as_double, as_bigint) VALUES (3, 123456789, 123456789, 123456789);
SELECT id, as_float, as_double, as_bigint, CAST(as_float AS bigint) AS float_thanh_so_nguyen FROM type_lab WHERE id = 3;

-- ## int chỉ tới 2.147.483.647: doanh thu tháng của một thành phố tràn int
-- EXPECT: Unable to make int from '3000000000'
INSERT INTO type_lab (id, as_int) VALUES (4, 3000000000);
INSERT INTO type_lab (id, as_bigint) VALUES (4, 3000000000);
SELECT id, as_bigint FROM type_lab WHERE id = 4;

-- ## timestamp: số mili giây từ 1970 theo UTC; độ lệch +0700 chỉ dùng lúc đọc chuỗi vào
INSERT INTO type_lab (id, at_ts) VALUES (5, '2026-11-02 07:42:00+0700');
INSERT INTO type_lab (id, at_ts) VALUES (6, '2026-11-02 00:42:00+0000');
INSERT INTO type_lab (id, at_ts) VALUES (7, '2026-11-02 07:42:00.123+0700');
SELECT id, at_ts, toUnixTimestamp(at_ts) AS mili_giay FROM type_lab WHERE id IN (5, 6, 7);

-- ## timestamp: chuỗi KHÔNG có độ lệch được hiểu theo múi giờ JVM của node điều phối (ở lab là Asia/Ho_Chi_Minh)
INSERT INTO type_lab (id, at_ts) VALUES (8, '2026-11-02 07:42:00');
SELECT id, at_ts FROM type_lab WHERE id = 8;

-- ## timestamp chỉ giữ tới mili giây: phần micro giây bị cắt bỏ, không báo lỗi
INSERT INTO type_lab (id, at_ts) VALUES (9, '2026-11-02 07:42:00.123456+0700');
SELECT id, at_ts, toUnixTimestamp(at_ts) AS mili_giay FROM type_lab WHERE id = 9;

-- ## date và time: ngày nghiệp vụ theo giờ Việt Nam là việc của ứng dụng
INSERT INTO type_lab (id, on_date, at_time) VALUES (10, '2026-11-02', '07:42:00.000');
SELECT id, on_date, at_time, toTimestamp(on_date) AS nua_dem_utc FROM type_lab WHERE id = 10;

-- ## text là UTF-8; ascii từ chối chữ có dấu
INSERT INTO type_lab (id, label) VALUES (11, 'Phở Thìn Lò Đúc');
SELECT id, label, textAsBlob(label) AS byte_utf8 FROM type_lab WHERE id = 11;
-- EXPECT: Invalid ASCII character in string literal
INSERT INTO type_lab (id, code) VALUES (12, 'Phở');

-- ## uuid và timeuuid: timeuuid mang thời điểm bên trong và sắp xếp theo thời gian
SELECT minTimeuuid('2026-11-02 07:42:00+0700') AS timeuuid_nho_nhat,
       toTimestamp(minTimeuuid('2026-11-02 07:42:00+0700')) AS thoi_diem_ben_trong
FROM system.local;

-- ## Kiểu của một cột không đổi được sang kiểu không tương thích
-- EXPECT: Altering column types is no longer supported
ALTER TABLE type_lab ALTER as_int TYPE bigint;

-- ## Dọn bảng thử nghiệm
DROP TABLE type_lab;
