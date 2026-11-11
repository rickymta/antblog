-- P04 · 01-fixture.sql — bộ dữ liệu NHỎ, tất định (nạp < 1 giây).
--
-- P04 nói về giao dịch và tranh chấp: mọi hiện tượng xảy ra trên vài dòng, nên fixture chỉ
-- có vài chục dòng mỗi bảng, ĐẶT TÊN DỄ NHỚ để lịch chạy trong bài đọc được:
--   * chuyến đang chờ nhận   : mã 400101 … 400110 (status 'requested', version 1)
--   * khách                  : 0940000001 … 0940000020 (ví 1001 … 1020, mỗi ví 500.000 đ)
--   * tài xế                 : SG 9101–9112, HN 9201–9208, DN 9301–9304 (ví 2001 … 2024, 200.000 đ)
--   * tài xế trực khu        : mỗi khu 2 người, ví dụ khu SG-TT-01 (111) có 9101 và 9102
--   * hàng đợi việc          : việc 1 … 10 'gui_thong_bao' (việc 1 ưu tiên 1, còn lại 5)
-- Mốc chụp: 2026-10-31 01:00 +07 (như P02/P03). Không dùng random()/now() cho dữ liệu nền.
-- Mọi tên, số điện thoại đều HƯ CẤU.

SET search_path TO toinoi, public;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- expect-rows: 3
INSERT INTO cities (id, code, name) VALUES
  (1, 'SG', 'TP. Hồ Chí Minh'),
  (2, 'HN', 'Hà Nội'),
  (3, 'DN', 'Đà Nẵng');

-- Cây khu vực điều phối — y hệt P02 (hư cấu, KHÔNG phải đơn vị hành chính).
-- expect-rows: 34
INSERT INTO areas (id, parent_id, code, name, level, city_id) VALUES
  (1,   NULL, 'SG',       'TP. Hồ Chí Minh', 'city',     1),
  (2,   NULL, 'HN',       'Hà Nội',          'city',     2),
  (3,   NULL, 'DN',       'Đà Nẵng',         'city',     3),
  (11,  1,    'SG-TT',    'Trung tâm',       'district', NULL),
  (12,  1,    'SG-DO',    'Phía Đông',       'district', NULL),
  (13,  1,    'SG-TA',    'Phía Tây',        'district', NULL),
  (14,  1,    'SG-NA',    'Phía Nam',        'district', NULL),
  (21,  2,    'HN-TT',    'Trung tâm',       'district', NULL),
  (22,  2,    'HN-TA',    'Phía Tây',        'district', NULL),
  (23,  2,    'HN-BA',    'Phía Bắc',        'district', NULL),
  (24,  2,    'HN-NA',    'Phía Nam (mở 11/2026)', 'district', NULL),
  (31,  3,    'DN-TT',    'Trung tâm',       'district', NULL),
  (32,  3,    'DN-BI',    'Ven biển',        'district', NULL),
  (111, 11,   'SG-TT-01', 'Khu 01', 'zone', NULL),
  (112, 11,   'SG-TT-02', 'Khu 02', 'zone', NULL),
  (113, 11,   'SG-TT-03', 'Khu 03', 'zone', NULL),
  (121, 12,   'SG-DO-01', 'Khu 01', 'zone', NULL),
  (122, 12,   'SG-DO-02', 'Khu 02', 'zone', NULL),
  (123, 12,   'SG-DO-03', 'Khu 03', 'zone', NULL),
  (131, 13,   'SG-TA-01', 'Khu 01', 'zone', NULL),
  (132, 13,   'SG-TA-02', 'Khu 02', 'zone', NULL),
  (141, 14,   'SG-NA-01', 'Khu 01', 'zone', NULL),
  (142, 14,   'SG-NA-02', 'Khu 02', 'zone', NULL),
  (211, 21,   'HN-TT-01', 'Khu 01', 'zone', NULL),
  (212, 21,   'HN-TT-02', 'Khu 02', 'zone', NULL),
  (213, 21,   'HN-TT-03', 'Khu 03', 'zone', NULL),
  (221, 22,   'HN-TA-01', 'Khu 01', 'zone', NULL),
  (222, 22,   'HN-TA-02', 'Khu 02', 'zone', NULL),
  (231, 23,   'HN-BA-01', 'Khu 01', 'zone', NULL),
  (232, 23,   'HN-BA-02', 'Khu 02', 'zone', NULL),
  (311, 31,   'DN-TT-01', 'Khu 01', 'zone', NULL),
  (312, 31,   'DN-TT-02', 'Khu 02', 'zone', NULL),
  (321, 32,   'DN-BI-01', 'Khu 01', 'zone', NULL),
  (322, 32,   'DN-BI-02', 'Khu 02', 'zone', NULL);

-- 20 khách SG. id = c4000000-…-<số thứ tự> để dễ đối chiếu (đừng suy id từ số trong hệ thật).
-- expect-rows: 20
INSERT INTO customers (id, phone, full_name, created_at)
SELECT ('c4000000-0000-4000-8000-' || lpad(n::text, 12, '0'))::uuid,
       '094' || lpad(n::text, 7, '0'),
       (ARRAY['Nguyễn', 'Trần', 'Lê', 'Phạm', 'Hoàng', 'Võ', 'Đặng', 'Bùi'])[1 + n % 8]
         || ' ' || (ARRAY['Văn', 'Thị', 'Minh', 'Thu', 'Quốc'])[1 + n % 5]
         || ' ' || (ARRAY['An', 'Bình', 'Châu', 'Dũng', 'Giang', 'Hải', 'Khánh', 'Linh', 'Mai', 'Nam', 'Oanh'])[1 + n % 11],
       TIMESTAMPTZ '2026-03-01 08:00:00+07' + n * interval '1 day'
FROM generate_series(1, 20) AS n;

-- 24 tài xế: SG 9101–9112, HN 9201–9208, DN 9301–9304; tất cả đang online.
-- expect-rows: 24
INSERT INTO drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
SELECT ('d4000000-0000-4000-8000-' || lpad(code::text, 12, '0'))::uuid,
       code,
       (ARRAY['Bùi', 'Hồ', 'Ngô', 'Đỗ', 'Lý', 'Trịnh', 'Mai', 'Tạ'])[1 + code % 8]
         || ' ' || (ARRAY['Văn', 'Thị', 'Quang', 'Đức'])[1 + code % 4]
         || ' ' || (ARRAY['Khoa', 'Lan', 'Minh', 'Nam', 'Phúc', 'Quý', 'Sang', 'Tú', 'Vinh'])[1 + code % 9],
       (code / 100 - 90)::smallint,
       CASE WHEN code % 4 = 0 THEN 'car' ELSE 'bike' END,
       'online',
       TIMESTAMPTZ '2026-06-01 09:00:00+07'
FROM (SELECT 9100 + g AS code FROM generate_series(1, 12) AS g
      UNION ALL SELECT 9200 + g FROM generate_series(1, 8) AS g
      UNION ALL SELECT 9300 + g FROM generate_series(1, 4) AS g) AS d
ORDER BY code;

-- expect-rows: 2
INSERT INTO merchants (id, city_id, name, created_at) VALUES
  ('e4000000-0000-4000-8000-000000000001', 1, 'Cơm Tấm Lab 1', TIMESTAMPTZ '2026-05-01 08:00:00+07'),
  ('e4000000-0000-4000-8000-000000000002', 1, 'Bún Bò Lab 2',  TIMESTAMPTZ '2026-05-01 08:00:00+07');

-- 30 chuyến đã xong ngày 30/10 (mã 400001–400030): 27 hoàn thành, 3 huỷ sau khi nhận (mã 400008, 400016, 400024).
-- Mã ghi thẳng (OVERRIDING SYSTEM VALUE) như nạp lịch sử, id = a4000000-…-<mã>.
-- expect-rows: 30
INSERT INTO trips (id, code, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at, pickup_area_id)
OVERRIDING SYSTEM VALUE
SELECT ('a4000000-0000-4000-8000-' || lpad((400000 + n)::text, 12, '0'))::uuid,
       400000 + n,
       k.kind,
       1,
       ('c4000000-0000-4000-8000-' || lpad((1 + (n - 1) % 20)::text, 12, '0'))::uuid,
       ('d4000000-0000-4000-8000-' || lpad((9101 + (n - 1) % 12)::text, 12, '0'))::uuid,
       CASE WHEN k.kind = 'food' THEN 'e4000000-0000-4000-8000-00000000000' || (1 + n % 2) END::uuid,
       CASE WHEN n % 8 = 0 THEN 'cancelled' ELSE 'completed' END,
       20000 + (n * 3700) % 60000 / 1000 * 1000,
       r.t,
       r.t + interval '2 minutes',
       CASE WHEN n % 8 = 0 THEN NULL ELSE r.t + interval '25 minutes' END,
       r.t + interval '25 minutes',
       (ARRAY[111, 112, 113, 121, 122, 123])[1 + n % 6]
FROM generate_series(1, 30) AS n
CROSS JOIN LATERAL (SELECT (ARRAY['ride', 'food', 'parcel'])[1 + n % 3] AS kind) AS k
CROSS JOIN LATERAL (SELECT TIMESTAMPTZ '2026-10-30 07:00:00+07' + n * interval '23 minutes' AS t) AS r;

-- 10 chuyến ĐANG CHỜ tài xế nhận (mã 400101–400110), đặt lúc 00:50–00:59 ngày 31/10.
-- Đây là các chuyến các bài tranh nhau nhận / sửa / huỷ.
-- expect-rows: 10
INSERT INTO trips (id, code, kind, city_id, customer_id, status, fare, requested_at, updated_at, pickup_area_id)
OVERRIDING SYSTEM VALUE
SELECT ('a4000000-0000-4000-8000-' || lpad((400100 + n)::text, 12, '0'))::uuid,
       400100 + n,
       CASE WHEN n % 2 = 0 THEN 'parcel' ELSE 'ride' END,
       1,
       ('c4000000-0000-4000-8000-' || lpad(n::text, 12, '0'))::uuid,
       'requested',
       30000 + n * 1000,
       TIMESTAMPTZ '2026-10-31 00:49:00+07' + n * interval '1 minute',
       TIMESTAMPTZ '2026-10-31 00:49:00+07' + n * interval '1 minute',
       (ARRAY[111, 112, 113])[1 + n % 3]
FROM generate_series(1, 10) AS n;

-- Chuyến tạo mới trong bài tiếp tục từ mã 400200.
SELECT setval(pg_get_serial_sequence('toinoi.trips', 'code'), 400200, false) AS ma_chuyen_ke_tiep;

-- Lịch sử trạng thái: requested → accepted → completed/cancelled; chuyến đang chờ chỉ có requested.
-- expect-rows: 100
INSERT INTO trip_events (trip_id, status, occurred_at)
SELECT t.id, e.status, e.at
FROM trips AS t
CROSS JOIN LATERAL (VALUES
  ('requested', t.requested_at, true),
  ('accepted', t.accepted_at, t.accepted_at IS NOT NULL),
  (t.status, t.updated_at, t.status IN ('completed', 'cancelled'))
) AS e(status, at, co)
WHERE e.co
ORDER BY t.code, e.at;

-- Ví: khách 1001–1020 (500.000 đ), tài xế 2001–2024 (200.000 đ).
-- expect-rows: 44
INSERT INTO wallets (id, owner_kind, customer_id, driver_id, balance, updated_at)
SELECT 1000 + n, 'customer', ('c4000000-0000-4000-8000-' || lpad(n::text, 12, '0'))::uuid, NULL, 500000,
       TIMESTAMPTZ '2026-10-31 00:00:00+07'
FROM generate_series(1, 20) AS n
UNION ALL
SELECT 2000 + row_number() OVER (ORDER BY code), 'driver', NULL, id, 200000, TIMESTAMPTZ '2026-10-31 00:00:00+07'
FROM drivers;

-- Tài xế trực khu: mỗi khu 2 người, tất cả đang trực.
--   SG: 111 (9101, 9102), 112 (9103, 9104), 113 (9105, 9106), 121, 122, 123 …
--   HN: 211 (9201, 9202), 212, 213, 221 · DN: 311 (9301, 9302), 312 (9303, 9304)
-- expect-rows: 24
INSERT INTO driver_duty (driver_id, area_id, on_duty, changed_at)
SELECT d.id,
       CASE d.city_id
         WHEN 1 THEN (ARRAY[111, 112, 113, 121, 122, 123])[1 + (d.code - 9101) / 2]
         WHEN 2 THEN (ARRAY[211, 212, 213, 221])[1 + (d.code - 9201) / 2]
         ELSE (ARRAY[311, 312])[1 + (d.code - 9301) / 2]
       END,
       true,
       TIMESTAMPTZ '2026-10-31 00:00:00+07'
FROM drivers AS d;

-- Hàng đợi: 10 việc gửi thông báo cho 10 chuyến đang chờ. Việc 1 ưu tiên 1 (gấp), còn lại 5.
-- expect-rows: 10
INSERT INTO jobs (kind, trip_id, priority, run_after, created_at)
SELECT 'gui_thong_bao', t.id, CASE WHEN t.code = 400101 THEN 1 ELSE 5 END,
       TIMESTAMPTZ '2026-10-31 01:00:00+07', TIMESTAMPTZ '2026-10-31 01:00:00+07'
FROM trips AS t
WHERE t.status = 'requested'
ORDER BY t.code;

-- Bảng nhỏ ⇒ ANALYZE đọc HẾT (không lấy mẫu) ⇒ thống kê tất định. Cluster tắt autovacuum.
VACUUM (ANALYZE);
