-- P02 · 01-fixture.sql — bộ dữ liệu 30 ngày, sinh TẤT ĐỊNH bằng SQL.
--
-- Ảnh chụp lúc 2026-10-31 01:00 giờ VN:
--   * chuyến đặt từ 2026-10-01 00:00 tới 2026-10-30 23:59:59 (+07) đều đã kết
--     thúc (completed / cancelled);
--   * 6 chuyến đặt sau nửa đêm 31/10 đang chạy, mỗi chuyến một trạng thái.
-- Mọi tên, số điện thoại, quán, khu vực đều HƯ CẤU.
--
-- Tất định: KHÔNG dùng random(). Mọi lựa chọn "ngẫu nhiên" (loại dịch vụ, khách,
-- tài xế, cước, có huỷ không…) lấy từ hàm băm số học p02_hash(n, salt) — cùng
-- đầu vào luôn ra cùng kết quả, trên PGlite hay PostgreSQL server. Hàm chỉ để
-- sinh dữ liệu lab, không phải hàm băm an toàn; cuối file bị xoá.
--
-- Mã chuyến KHÔNG do IDENTITY cấp mà ghi thẳng (OVERRIDING SYSTEM VALUE), tăng
-- theo giờ đặt: như nạp lịch sử từ hệ cũ giữ nguyên mã. Sau khi nạp, đặt lại
-- sequence để chuyến tạo mới tiếp tục từ mã lớn nhất. id = 70000000-…-<mã>
-- (giống quy ước P01, chỉ để dễ đối chiếu — đừng suy mã từ id trong hệ thật).
--
-- Hình dạng dữ liệu và các điểm "cài sẵn" cho từng bài: p02/README.md.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- Hàm băm sinh dữ liệu: nhân Knuth (2654435761) trên 31 bit thấp, lấy 16 bit
-- cao của tích 32 bit ⇒ số nguyên 0..65535. Mọi phép tính nằm trong bigint
-- (tích lớn nhất ≈ 5,7·10^18 < 9,2·10^18), không tràn.
CREATE FUNCTION p02_hash(n bigint, salt integer) RETURNS integer
LANGUAGE sql IMMUTABLE STRICT
AS $$
  SELECT ((((n * 1000003 + salt * 7919) % 2147483648) * 2654435761) % 4294967296 / 65536)::integer
$$;

-- ---------------------------------------------------------------------------
-- Danh mục
-- expect-rows: 3
INSERT INTO cities (id, code, name) VALUES
  (1, 'SG', 'TP. Hồ Chí Minh'),
  (2, 'HN', 'Hà Nội'),
  (3, 'DN', 'Đà Nẵng');

-- Cây khu vực điều phối (hư cấu): 3 thành phố, 10 vùng, 21 khu.
-- Vùng HN-NA mới mở, CHƯA có khu nào (lá ở cấp vùng) — dùng ở PG-019.
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

-- 600 khách cá nhân, chia theo thành phố cư trú: 1–300 SG, 301–510 HN, 511–600 DN
-- (thành phố cư trú không lưu trong bảng, chỉ dùng để sinh chuyến cùng thành phố).
-- Số điện thoại 0910000001 … 0910000600: nhìn là biết số giả.
-- Thêm 1 khách doanh nghiệp đặt bưu kiện qua API (PG-018).
-- expect-rows: 601
INSERT INTO customers (id, phone, full_name, created_at)
SELECT ('c1000000-0000-4000-8000-' || lpad(n::text, 12, '0'))::uuid,
       '091' || lpad(n::text, 7, '0'),
       (ARRAY['Nguyễn', 'Trần', 'Lê', 'Phạm', 'Hoàng', 'Huỳnh', 'Phan', 'Vũ', 'Võ', 'Đặng', 'Bùi', 'Đỗ'])[1 + n % 12]
         || ' ' || (ARRAY['Văn', 'Thị', 'Minh', 'Thu', 'Quốc', 'Ngọc', 'Hữu', 'Thanh'])[1 + (n / 12) % 8]
         || ' ' || (ARRAY['An', 'Bình', 'Châu', 'Dũng', 'Giang', 'Hải', 'Khánh', 'Linh', 'Mai', 'Nam',
                          'Oanh', 'Phong', 'Quân', 'Sơn', 'Tâm', 'Uyên', 'Việt', 'Xuân', 'Yến', 'Hạnh'])[1 + (n / 96) % 20],
       TIMESTAMPTZ '2026-01-01 08:00:00+07' + n * interval '7 hours'
FROM generate_series(1, 600) AS n
UNION ALL
SELECT 'cb000000-0000-4000-8000-000000000001', '0919990001', 'Cửa hàng Phụ Kiện Lab (KHDN)',
       TIMESTAMPTZ '2026-03-15 09:00:00+07';

-- Tài xế: SG 8101–8124, HN 8201–8220, DN 8301–8310. 8219 và 8220 vừa đăng ký
-- 29/10, CHƯA chạy chuyến nào (PG-014: bẫy NOT IN). Bốn tài xế đang có chuyến
-- dở lúc chụp (8101, 8102, 8201, 8301) là `busy`; còn lại online/offline.
-- expect-rows: 54
INSERT INTO drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
SELECT ('d1000000-0000-4000-8000-' || lpad(code::text, 12, '0'))::uuid,
       code,
       (ARRAY['Bùi', 'Hồ', 'Ngô', 'Đỗ', 'Lý', 'Trịnh', 'Mai', 'Tạ', 'Lương', 'Đinh'])[1 + code % 10]
         || ' ' || (ARRAY['Văn', 'Thị', 'Quang', 'Thanh', 'Đức', 'Kim'])[1 + (code / 10) % 6]
         || ' ' || (ARRAY['Khoa', 'Lan', 'Minh', 'Nam', 'Oanh', 'Phúc', 'Quý', 'Sang', 'Tú', 'Vinh', 'Hưng', 'Lộc'])[1 + (code / 7) % 12],
       city_id,
       CASE WHEN code % 4 = 0 THEN 'car' ELSE 'bike' END,
       CASE WHEN code IN (8101, 8102, 8201, 8301) THEN 'busy'
            WHEN code IN (8219, 8220) THEN 'offline'
            WHEN p02_hash(code, 30) % 3 = 0 THEN 'online'
            ELSE 'offline' END,
       CASE WHEN code IN (8219, 8220) THEN TIMESTAMPTZ '2026-10-29 09:00:00+07'
            ELSE TIMESTAMPTZ '2026-06-01 09:00:00+07' + (code % 100) * interval '1 day' END
FROM (SELECT 8100 + g AS code, 1::smallint AS city_id FROM generate_series(1, 24) AS g
      UNION ALL
      SELECT 8200 + g, 2::smallint FROM generate_series(1, 20) AS g
      UNION ALL
      SELECT 8300 + g, 3::smallint FROM generate_series(1, 10) AS g) AS d
ORDER BY code;

-- Quán: SG 101–112, HN 201–208, DN 301–305.
-- expect-rows: 25
INSERT INTO merchants (id, city_id, name, created_at)
SELECT ('e1000000-0000-4000-8000-' || lpad(n::text, 12, '0'))::uuid,
       (n / 100)::smallint,
       (ARRAY['Cơm Tấm', 'Bún Bò', 'Phở', 'Bánh Mì', 'Trà Sữa', 'Gà Rán', 'Bún Chả', 'Mì Quảng', 'Bánh Xèo', 'Cháo'])[1 + n % 10]
         || ' Lab ' || n,
       TIMESTAMPTZ '2026-05-01 08:00:00+07'
FROM (SELECT 100 + g AS n FROM generate_series(1, 12) AS g
      UNION ALL SELECT 200 + g FROM generate_series(1, 8) AS g
      UNION ALL SELECT 300 + g FROM generate_series(1, 5) AS g) AS m
ORDER BY n;

-- ---------------------------------------------------------------------------
-- Kế hoạch chuyến (bảng TẠM, chỉ sống trong phiên): mỗi dòng là một chuyến với
-- đủ thuộc tính và các mốc thời gian, kể cả mốc không lưu trong trips (đón
-- khách) — để sinh trips, trip_events, trip_ratings, payments nhất quán.
--
-- Số chuyến mỗi (ngày, thành phố) = cơ sở (SG 50 · HN 35 · DN 15) × 1,2 nếu là
-- thứ Bảy/Chủ nhật, cộng dao động −2…+2. Ngày 20/10 Đà Nẵng có bão: 0 chuyến
-- (một ngày TRỐNG cho PG-015/PG-016). Giờ đặt rải theo hồ sơ giờ trong ngày
-- (cao điểm 11–13 h và 17–19 h; 12 % chuyến rơi vào 00:00–06:59 giờ VN = ngày
-- hôm trước theo UTC, cho PG-016). Giờ đặt của chuyến sinh tự động làm tròn
-- tới PHÚT (như nguồn lịch sử chỉ lưu tới phút) ⇒ nhiều chuyến trùng giờ đặt
-- (PG-015, PG-018); các chuyến đặc biệt bên dưới ghi giờ chính xác.
-- expect-rows: 3174
CREATE TEMP TABLE p02_plan AS
WITH gio (h, w) AS (
  VALUES (0, 2), (1, 1), (2, 1), (3, 1), (4, 1), (5, 2), (6, 4), (7, 6), (8, 6), (9, 4),
         (10, 4), (11, 8), (12, 9), (13, 5), (14, 3), (15, 3), (16, 4), (17, 7), (18, 9),
         (19, 7), (20, 5), (21, 4), (22, 2), (23, 2)            -- tổng trọng số = 100
),
gio_luy_ke AS (
  SELECT h, w, (sum(w) OVER (ORDER BY h) - w)::numeric AS truoc FROM gio
),
tp (city_id, co_so) AS (VALUES (1, 50), (2, 35), (3, 15)),
o AS (                                   -- số chuyến của từng (ngày, thành phố)
  SELECT g.d, DATE '2026-10-01' + g.d AS ngay, tp.city_id,
         CASE WHEN tp.city_id = 3 AND DATE '2026-10-01' + g.d = DATE '2026-10-20' THEN 0
              ELSE tp.co_so * (CASE WHEN extract(isodow FROM DATE '2026-10-01' + g.d) >= 6 THEN 12 ELSE 10 END) / 10
                   + (g.d * 7 + tp.city_id * 3) % 5 - 2
         END AS so_chuyen
  FROM generate_series(0, 29) AS g (d) CROSS JOIN tp
),
sinh AS (                                -- chuyến thường: vị trí i trong ngày → giờ đặt
  SELECT o.d * 1000 + o.city_id * 300 + i AS k, o.city_id, o.ngay,
         (i + 0.5) * 100.0 / o.so_chuyen AS p
  FROM o CROSS JOIN LATERAL generate_series(0, o.so_chuyen - 1) AS i
),
thuong AS (
  SELECT s.k, s.city_id::smallint AS city_id,
         ((s.ngay + make_interval(hours => gl.h,
                                  mins  => floor((s.p - gl.truoc) / gl.w * 60)::integer,
                                  secs  => 0))::timestamp
            AT TIME ZONE 'Asia/Ho_Chi_Minh') AS requested_at,
         NULL::text AS kind_dat, NULL::text AS ket_cuc_dat, NULL::uuid AS customer_dat,
         NULL::integer AS driver_code_dat, 'thuong'::text AS nguon
  FROM sinh AS s
  JOIN gio_luy_ke AS gl ON s.p >= gl.truoc AND s.p < gl.truoc + gl.w
),
dac_biet (k, city_id, requested_at, kind_dat, ket_cuc_dat, customer_dat, driver_code_dat, nguon) AS (
  VALUES
  -- PG-016: chuyến ở sát biên ngày 15/10 (giờ VN).
  (900001, 1, TIMESTAMPTZ '2026-10-15 00:00:00+07',   'ride', 'completed', NULL::uuid, NULL::integer, 'bien'),
  (900002, 1, TIMESTAMPTZ '2026-10-15 06:59:59+07',   'ride', 'completed', NULL, NULL, 'bien'),
  (900003, 1, TIMESTAMPTZ '2026-10-15 07:00:00+07',   'ride', 'completed', NULL, NULL, 'bien'),
  (900004, 1, TIMESTAMPTZ '2026-10-15 23:59:59.5+07', 'ride', 'completed', NULL, NULL, 'bien'),
  (900005, 1, TIMESTAMPTZ '2026-10-16 00:00:00+07',   'ride', 'completed', NULL, NULL, 'bien'),
  -- PG-018: 18 chuyến bưu kiện của khách doanh nghiệp; hai lô đặt qua API cùng
  -- một thời điểm (6 chuyến lúc 10/10 09:00, 4 chuyến lúc 18/10 14:30).
  (900201, 1, TIMESTAMPTZ '2026-10-02 10:15:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900202, 1, TIMESTAMPTZ '2026-10-05 15:40:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900203, 1, TIMESTAMPTZ '2026-10-07 09:20:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900204, 1, TIMESTAMPTZ '2026-10-10 09:00:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900205, 1, TIMESTAMPTZ '2026-10-10 09:00:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900206, 1, TIMESTAMPTZ '2026-10-10 09:00:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900207, 1, TIMESTAMPTZ '2026-10-10 09:00:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900208, 1, TIMESTAMPTZ '2026-10-10 09:00:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900209, 1, TIMESTAMPTZ '2026-10-10 09:00:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900210, 1, TIMESTAMPTZ '2026-10-12 11:05:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900211, 1, TIMESTAMPTZ '2026-10-15 16:45:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900212, 1, TIMESTAMPTZ '2026-10-18 14:30:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900213, 1, TIMESTAMPTZ '2026-10-18 14:30:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900214, 1, TIMESTAMPTZ '2026-10-18 14:30:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900215, 1, TIMESTAMPTZ '2026-10-18 14:30:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900216, 1, TIMESTAMPTZ '2026-10-21 08:50:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900217, 1, TIMESTAMPTZ '2026-10-25 14:10:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  (900218, 1, TIMESTAMPTZ '2026-10-28 10:30:00+07', 'parcel', 'completed', 'cb000000-0000-4000-8000-000000000001', NULL, 'khdn'),
  -- Sáu chuyến đang chạy lúc chụp 2026-10-31 01:00, mỗi chuyến một trạng thái.
  (900101, 1, TIMESTAMPTZ '2026-10-31 00:30:00+07', 'ride',   'in_transit', NULL, 8101, 'dang_chay'),
  (900102, 2, TIMESTAMPTZ '2026-10-31 00:38:00+07', 'food',   'picked_up',  NULL, 8201, 'dang_chay'),
  (900103, 1, TIMESTAMPTZ '2026-10-31 00:46:00+07', 'parcel', 'arriving',   NULL, 8102, 'dang_chay'),
  (900104, 3, TIMESTAMPTZ '2026-10-31 00:50:00+07', 'ride',   'accepted',   NULL, 8301, 'dang_chay'),
  (900105, 2, TIMESTAMPTZ '2026-10-31 00:55:00+07', 'food',   'searching',  NULL, NULL, 'dang_chay'),
  (900106, 1, TIMESTAMPTZ '2026-10-31 00:58:00+07', 'ride',   'requested',  NULL, NULL, 'dang_chay')
),
tat_ca AS (
  SELECT * FROM thuong
  UNION ALL
  SELECT k, city_id::smallint, requested_at, kind_dat, ket_cuc_dat, customer_dat, driver_code_dat, nguon FROM dac_biet
),
-- Thông số theo thành phố: số tài xế đang hoạt động, số quán, các khu, dải khách.
thong_so (city_id, so_tai_xe, so_quan, khu, kh_dau, kh_so) AS (
  VALUES (1::smallint, 24, 12, ARRAY[111, 112, 113, 121, 122, 123, 131, 132, 141, 142], 1, 300),
         (2::smallint, 18,  8, ARRAY[211, 212, 213, 221, 222, 231, 232], 301, 210),
         (3::smallint, 10,  5, ARRAY[311, 312, 321, 322], 511, 90)
),
b1 AS (                                  -- loại dịch vụ và kết cục
  SELECT t.*, ts.so_tai_xe, ts.so_quan, ts.khu, ts.kh_dau, ts.kh_so,
         coalesce(t.kind_dat, CASE WHEN p02_hash(t.k, 1) % 100 < 40 THEN 'ride'
                                   WHEN p02_hash(t.k, 1) % 100 < 85 THEN 'food'
                                   ELSE 'parcel' END) AS kind,
         coalesce(t.ket_cuc_dat, CASE WHEN p02_hash(t.k, 2) % 1000 < 90  THEN 'huy_truoc_nhan'
                                      WHEN p02_hash(t.k, 2) % 1000 < 150 THEN 'huy_sau_nhan'
                                      ELSE 'completed' END) AS ket_cuc
  FROM tat_ca AS t JOIN thong_so AS ts USING (city_id)
),
b2 AS (                                  -- ai, ở đâu, bao nhiêu tiền
  SELECT b1.*,
         coalesce(customer_dat,
                  ('c1000000-0000-4000-8000-' || lpad((kh_dau +
                     CASE WHEN p02_hash(k, 5) % 5 = 0 THEN p02_hash(k, 17) % (kh_so / 10)   -- 20 %: nhóm khách đi nhiều
                          ELSE p02_hash(k, 17) % kh_so END)::text, 12, '0'))::uuid) AS customer_id,
         CASE WHEN ket_cuc IN ('huy_truoc_nhan', 'requested', 'searching') THEN NULL
              ELSE ('d1000000-0000-4000-8000-' || lpad(coalesce(driver_code_dat,
                      city_id * 100 + 8000 + 1 + p02_hash(k, 6) % so_tai_xe)::text, 12, '0'))::uuid
         END AS driver_id,
         CASE WHEN kind = 'food'
              THEN ('e1000000-0000-4000-8000-' || lpad((city_id * 100 + 1 + p02_hash(k, 7) % so_quan)::text, 12, '0'))::uuid
         END AS merchant_id,
         CASE WHEN p02_hash(k, 11) % 100 = 0 THEN NULL                       -- 1 %: mất GPS
              ELSE khu[1 + p02_hash(k, 10) % cardinality(khu)] END AS pickup_area_id,
         CASE kind
           WHEN 'ride'   THEN CASE WHEN p02_hash(k, 4) % 100 < 2 THEN 0   -- khuyến mãi 0 đ
                                   ELSE 12000 + p02_hash(k, 3) % 89 * 1000 END
           WHEN 'food'   THEN 25000 + p02_hash(k, 3) % 126 * 1000
           ELSE               18000 + p02_hash(k, 3) % 63 * 1000
         END AS fare,
         -- Các khoảng thời gian (giây/phút) của vòng đời.
         20 + p02_hash(k, 8) % 280  AS giay_cho_nhan,      -- đặt → tài xế nhận
         3 + p02_hash(k, 9) % 8     AS phut_toi_don,       -- nhận → đón/lấy hàng
         5 + p02_hash(k, 15) % 30   AS phut_di,            -- đón → hoàn thành
         p02_hash(k, 16)            AS h_huy
  FROM b1
),
b3 AS (                                  -- các mốc thời gian
  SELECT b2.*,
         CASE WHEN driver_id IS NOT NULL THEN requested_at + giay_cho_nhan * interval '1 second' END AS accepted_at,
         CASE WHEN ket_cuc IN ('completed', 'picked_up', 'in_transit')
              THEN requested_at + giay_cho_nhan * interval '1 second' + phut_toi_don * interval '1 minute' END AS picked_up_at
  FROM b2
)
SELECT row_number() OVER (ORDER BY requested_at, k) + 99999 AS code,
       k, nguon, kind, city_id, customer_id, driver_id, merchant_id, pickup_area_id, fare, ket_cuc,
       CASE WHEN ket_cuc IN ('huy_truoc_nhan', 'huy_sau_nhan') THEN 'cancelled' ELSE ket_cuc END AS status,
       requested_at, accepted_at, picked_up_at,
       CASE WHEN ket_cuc = 'completed' THEN picked_up_at + phut_di * interval '1 minute' END AS completed_at,
       CASE ket_cuc
         WHEN 'huy_truoc_nhan' THEN requested_at + (60 + h_huy % 300) * interval '1 second'
         WHEN 'huy_sau_nhan'   THEN accepted_at + (30 + h_huy % 450) * interval '1 second'
       END AS cancelled_at,
       -- Bước cuối của vòng đời (1 requested … 7 completed); chuyến huỷ thêm bước 8.
       CASE ket_cuc WHEN 'requested' THEN 1 WHEN 'searching' THEN 2 WHEN 'accepted' THEN 3
                    WHEN 'arriving' THEN 4 WHEN 'picked_up' THEN 5 WHEN 'in_transit' THEN 6
                    WHEN 'completed' THEN 7 WHEN 'huy_truoc_nhan' THEN 2 WHEN 'huy_sau_nhan' THEN 4
       END AS buoc_cuoi
FROM b3;

-- Kế hoạch phải có mã duy nhất và đúng số chuyến đặc biệt.
-- expect-value: 0
SELECT count(*) - count(DISTINCT code) FROM p02_plan;

-- ---------------------------------------------------------------------------
-- Chuyến. updated_at = mốc cuối cùng của vòng đời (thoả kiểm tra B5 của P01).
-- expect-rows: 3174
INSERT INTO trips (id, code, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at, pickup_area_id)
OVERRIDING SYSTEM VALUE
SELECT ('70000000-0000-4000-8000-' || lpad(code::text, 12, '0'))::uuid,
       code, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
       requested_at, accepted_at, completed_at,
       CASE ket_cuc
         WHEN 'requested'  THEN requested_at
         WHEN 'searching'  THEN requested_at + interval '2 seconds'
         WHEN 'accepted'   THEN accepted_at
         WHEN 'arriving'   THEN accepted_at + interval '10 seconds'
         WHEN 'picked_up'  THEN picked_up_at
         WHEN 'in_transit' THEN picked_up_at + interval '30 seconds'
         WHEN 'completed'  THEN completed_at
         ELSE cancelled_at
       END,
       pickup_area_id
FROM p02_plan
ORDER BY code;

-- Mã chuyến tạo mới tiếp tục sau mã lớn nhất đã nạp.
-- expect-value: 103173
SELECT setval(pg_get_serial_sequence('trips', 'code'), (SELECT max(code) FROM trips)) AS ma_cuoi;

-- ---------------------------------------------------------------------------
-- Lịch sử trạng thái: mỗi chuyến đi qua các bước 1..buoc_cuoi; chuyến huỷ thêm
-- dòng `cancelled`. Thêm theo (mã chuyến, thứ tự bước) ⇒ id tăng tất định.
-- expect-rows: 20685
INSERT INTO trip_events (trip_id, status, occurred_at)
SELECT ('70000000-0000-4000-8000-' || lpad(p.code::text, 12, '0'))::uuid, e.status, e.occurred_at
FROM p02_plan AS p
CROSS JOIN LATERAL (VALUES
  (1, 'requested',  p.requested_at),
  (2, 'searching',  p.requested_at + interval '2 seconds'),
  (3, 'accepted',   p.accepted_at),
  (4, 'arriving',   p.accepted_at + interval '10 seconds'),
  (5, 'picked_up',  p.picked_up_at),
  (6, 'in_transit', p.picked_up_at + interval '30 seconds'),
  (7, 'completed',  p.completed_at),
  (8, 'cancelled',  p.cancelled_at)
) AS e (buoc, status, occurred_at)
WHERE e.buoc <= p.buoc_cuoi OR (e.buoc = 8 AND p.status = 'cancelled')
ORDER BY p.code, e.buoc;

-- ---------------------------------------------------------------------------
-- Đánh giá: ~70 % chuyến hoàn thành. Trong đó ~6 % bỏ trống sao (stars NULL);
-- tip: ~30 % bỏ qua (NULL), ~45 % chọn 0 đ, còn lại 5.000 / 10.000 / 20.000 đ.
-- expect-rows: 1891
INSERT INTO trip_ratings (trip_id, stars, tip, rated_at)
SELECT t.id,
       CASE WHEN v < 6 THEN NULL WHEN v < 62 THEN 5 WHEN v < 86 THEN 4
            WHEN v < 93 THEN 3 WHEN v < 97 THEN 2 ELSE 1 END,
       CASE WHEN tp < 30 THEN NULL WHEN tp < 75 THEN 0 WHEN tp < 90 THEN 5000
            WHEN tp < 97 THEN 10000 ELSE 20000 END,
       t.completed_at + (1 + p02_hash(t.code, 18) % 10) * interval '1 minute'
FROM trips AS t
CROSS JOIN LATERAL (SELECT p02_hash(t.code, 13) % 100 AS v, p02_hash(t.code, 14) % 100 AS tp) AS x
WHERE t.status = 'completed' AND p02_hash(t.code, 12) % 100 < 70
ORDER BY t.code;

-- ---------------------------------------------------------------------------
-- Thanh toán bình thường (trước khi cài sai lệch):
--   * chuyến hoàn thành, cước > 0: tiền mặt ~40 %, ví ~35 %, thẻ ~25 %.
--     - ví/thẻ ~4 %: lần đầu `failed`, 1 phút sau thu lại thành công;
--     - ví ~5 % (cước > 40.000): trả một phần bằng ví, phần còn lại bằng thẻ;
--   * chuyến cước 0 đ: không có giao dịch nào (không có gì để thu);
--   * chuyến huỷ SAU khi nhận, trả thẻ, ~50 %: đã trừ tiền lúc nhận rồi hoàn đủ
--     lúc huỷ (charge + refund, thực thu 0);
--   * chuyến đang chạy: chưa có giao dịch.
-- expect-rows: 3168
CREATE TEMP TABLE p02_pay_plan AS
SELECT t.code, t.id AS trip_id, t.status, t.fare, t.accepted_at, t.completed_at, t.updated_at,
       CASE WHEN p02_hash(t.code, 20) % 100 < 40 THEN 'cash'
            WHEN p02_hash(t.code, 20) % 100 < 75 THEN 'wallet'
            ELSE 'card' END AS method,
       p02_hash(t.code, 21) % 100 < 4 AS lan_dau_loi,
       p02_hash(t.code, 22) % 100 < 5 AS tra_mot_phan,
       10000 + p02_hash(t.code, 24) % 3 * 5000 AS phan_the,
       p02_hash(t.code, 23) % 100 < 50 AS tru_truoc
FROM trips AS t
WHERE t.status IN ('completed', 'cancelled');

-- expect-rows: 2821
INSERT INTO payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
SELECT trip_id, kind, method, status, amount,
       CASE method WHEN 'wallet' THEN 'VX-' WHEN 'card' THEN 'TH-' END || code || '-' || hau_to,
       paid_at
FROM (
  -- (a) lần thu lỗi trước đó
  SELECT code, 1 AS thu_tu, trip_id, 'charge' AS kind, method, 'failed' AS status, fare AS amount,
         '1' AS hau_to, completed_at AS paid_at
  FROM p02_pay_plan
  WHERE status = 'completed' AND fare > 0 AND method <> 'cash' AND lan_dau_loi
  UNION ALL
  -- (b) lần thu thành công (trả một phần: phần ví)
  SELECT code, 2, trip_id, 'charge',
         CASE WHEN tra_mot_phan_hop_le THEN 'wallet' ELSE method END, 'succeeded',
         CASE WHEN tra_mot_phan_hop_le THEN fare - phan_the ELSE fare END,
         CASE WHEN lan_dau_loi AND method <> 'cash' THEN '2' ELSE '1' END,
         completed_at + CASE WHEN lan_dau_loi AND method <> 'cash' THEN interval '1 minute' ELSE interval '0' END
  FROM (SELECT *, method = 'wallet' AND tra_mot_phan AND NOT lan_dau_loi AND fare > 40000 AS tra_mot_phan_hop_le
        FROM p02_pay_plan) AS x
  WHERE status = 'completed' AND fare > 0
  UNION ALL
  -- (c) trả một phần: phần thẻ
  SELECT code, 3, trip_id, 'charge', 'card', 'succeeded', phan_the, '2', completed_at + interval '20 seconds'
  FROM p02_pay_plan
  WHERE status = 'completed' AND fare > 40000 AND method = 'wallet' AND tra_mot_phan AND NOT lan_dau_loi
  UNION ALL
  -- (d) chuyến huỷ sau khi nhận: trừ tiền thẻ lúc nhận…
  SELECT code, 4, trip_id, 'charge', 'card', 'succeeded', fare, '1', accepted_at
  FROM p02_pay_plan
  WHERE status = 'cancelled' AND accepted_at IS NOT NULL AND method = 'card' AND tru_truoc AND fare > 0
  UNION ALL
  -- (e) … rồi hoàn đủ lúc huỷ
  SELECT code, 5, trip_id, 'refund', 'card', 'succeeded', fare, 'R', updated_at
  FROM p02_pay_plan
  WHERE status = 'cancelled' AND accepted_at IS NOT NULL AND method = 'card' AND tru_truoc AND fare > 0
) AS g
ORDER BY code, thu_tu;

-- ---------------------------------------------------------------------------
-- Các ca "khó nhưng ĐÚNG" — đối soát không được báo nhầm (PG-020):
--   A1 103165: tài xế thu thiếu 5.000 đ tiền mặt, CSKH trực đêm trừ ví khách
--      5.000 đ một giờ sau — đã sang ngày 31/10 (adjustment +5.000) ⇒ thực thu
--      vẫn = cước, nhưng tiền nằm ở hai ngày khác nhau (PG-020 mục 5).
--   A2 102235: khách khiếu nại, cước sửa từ 82.000 xuống 72.000; ví đã trừ
--      82.000 nên CSKH ghi adjustment −10.000 ⇒ thực thu = cước mới.
--   P1 103162: app thẻ treo, để lại một giao dịch `pending` thừa bên cạnh lần
--      thu thành công ⇒ không đổi tiền, chỉ cần theo dõi.
-- expect-rows: 1
UPDATE payments SET amount = 23000
WHERE trip_id = '70000000-0000-4000-8000-000000103165' AND kind = 'charge' AND method = 'cash' AND amount = 28000;

-- expect-rows: 1
UPDATE payments SET amount = 82000
WHERE trip_id = '70000000-0000-4000-8000-000000102235' AND kind = 'charge' AND method = 'wallet' AND amount = 72000;

-- expect-rows: 3
INSERT INTO payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
SELECT t.id, v.kind, v.method, v.status, v.amount, v.provider_ref, t.completed_at + v.tre
FROM (VALUES
  (102235::bigint, 'adjustment', 'wallet', 'succeeded', -10000, 'VX-102235-A', interval '2 hours'),
  (103165,         'adjustment', 'wallet', 'succeeded',   5000, 'VX-103165-A', interval '1 hour'),
  (103162,         'charge',     'card',   'pending',    57000, 'TH-103162-P', interval '2 minutes')
) AS v (code, kind, method, status, amount, provider_ref, tre)
JOIN trips AS t ON t.code = v.code
ORDER BY v.code;

-- ---------------------------------------------------------------------------
-- SAI LỆCH CÀI SẴN cho bài thực hành đối soát (PG-020) — đúng 7 chuyến:
--   D1 100724 THIẾU THANH TOÁN  tài xế không ghi nhận tiền mặt (xoá giao dịch)
--   D2 101084 THIẾU THANH TOÁN  thẻ bị từ chối, không ai thu lại
--   D3 101310 THU TRÙNG         ví trừ hai lần (app gửi lại yêu cầu), 3 giây sau
--   D4 101987 HOÀN KHÔNG CÓ THU chuyến huỷ trước khi nhận nhưng ví lại hoàn 72.000
--   D5 102443 LỆCH SỐ TIỀN      tài xế nhập 45.000 thay vì 54.000 (đảo chữ số)
--   D6 102642 LỆCH SỐ TIỀN      thẻ trừ 380.000 thay vì 38.000 (thừa một số 0)
--   D7 102739 THU CHUYẾN HUỶ    chuyến huỷ sau khi nhận nhưng ví vẫn bị trừ
-- expect-rows: 1
DELETE FROM payments WHERE trip_id = '70000000-0000-4000-8000-000000100724';

-- expect-rows: 1
UPDATE payments SET status = 'failed'
WHERE trip_id = '70000000-0000-4000-8000-000000101084' AND status = 'succeeded';

-- expect-rows: 2
UPDATE payments SET amount = CASE trip_id
                               WHEN '70000000-0000-4000-8000-000000102443' THEN 45000
                               WHEN '70000000-0000-4000-8000-000000102642' THEN 380000
                             END
WHERE trip_id IN ('70000000-0000-4000-8000-000000102443', '70000000-0000-4000-8000-000000102642')
  AND kind = 'charge' AND status = 'succeeded';

-- Thời điểm: D3 3 giây sau lần trừ đầu; D4 1 giờ sau khi huỷ; D7 lúc tài xế nhận.
-- expect-rows: 3
INSERT INTO payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
SELECT t.id, v.kind, 'wallet', 'succeeded', v.amount, v.provider_ref,
       CASE v.code
         WHEN 101310 THEN (SELECT p.paid_at FROM payments AS p
                           WHERE p.trip_id = t.id AND p.status = 'succeeded') + interval '3 seconds'
         WHEN 101987 THEN t.updated_at + interval '1 hour'
         WHEN 102739 THEN t.accepted_at
       END
FROM (VALUES
  (101310::bigint, 'charge', 85000, 'VX-101310-2'),
  (101987,         'refund', 72000, 'VX-101987-R'),
  (102739,         'charge', 96000, 'VX-102739-1')
) AS v (code, kind, amount, provider_ref)
JOIN trips AS t ON t.code = v.code
ORDER BY v.code;

-- ---------------------------------------------------------------------------
-- Dọn công cụ sinh dữ liệu: dữ liệu đã nằm trong bảng, không cần hàm/bảng tạm.
DROP TABLE p02_pay_plan;
DROP TABLE p02_plan;
DROP FUNCTION p02_hash(bigint, integer);

-- Cập nhật thống kê cho bộ lập kế hoạch sau khi nạp hàng chục nghìn dòng.
-- Không đổi KẾT QUẢ truy vấn nào, chỉ đổi cách chạy. Đo một lần trên máy viết
-- bài (PGlite 17.5): thiếu dòng này, cả bộ P02 mất ~30 giây vì vài truy vấn
-- đối soát ở 14 và 20 chọn kế hoạch tồi (mỗi câu 4–8 giây); có nó, ~3 giây.
-- Server thật có autovacuum tự ANALYZE định kỳ; P03 giải thích thống kê và EXPLAIN.
ANALYZE;

-- Số dòng sau khi nạp (02-kiem-tra-fixture.sql mô tả chi tiết hình dạng).
-- expect-line: 3 | 34 | 601 | 54 | 25 | 3174 | 20685 | 1891 | 2826
SELECT (SELECT count(*) FROM cities)       AS cities,
       (SELECT count(*) FROM areas)        AS areas,
       (SELECT count(*) FROM customers)    AS customers,
       (SELECT count(*) FROM drivers)      AS drivers,
       (SELECT count(*) FROM merchants)    AS merchants,
       (SELECT count(*) FROM trips)        AS trips,
       (SELECT count(*) FROM trip_events)  AS trip_events,
       (SELECT count(*) FROM trip_ratings) AS trip_ratings,
       (SELECT count(*) FROM payments)     AS payments;
