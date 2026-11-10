-- P03 · 01-fixture.sql — bộ dữ liệu LỚN, sinh TẤT ĐỊNH bằng SQL: 1.000.000 chuyến
-- trong 6 tháng (01/05 → 31/10/2026), đủ để index, kế hoạch thực thi và phép đo
-- có ý nghĩa. Mọi tên người, số điện thoại, quán, địa chỉ đều HƯ CẤU.
--
-- Mốc chụp dữ liệu: 2026-10-31 01:00 giờ VN (giống P02). Chuyến đặt trước
-- 31/10 00:00 đều đã kết thúc; 300 chuyến đặt 00:00–00:59 ngày 31/10 đang chạy.
--
-- Tất định: KHÔNG dùng random(). Mỗi chuyến số k lấy ba chuỗi md5('a'||k),
-- md5('b'||k), md5('c'||k); mỗi chuỗi cắt thành 4 số 32 bit độc lập ⇒ 12 số
-- "ngẫu nhiên" đều trong [0, 1) (hàm p03_u). Cùng PostgreSQL ⇒ cùng dữ liệu.
-- (P02 dùng hàm băm nhân p02_hash; ở quy mô triệu dòng, các "muối" khác nhau
-- của hàm đó tương quan với nhau — sẽ tạo tương quan giả giữa các cột, làm hỏng
-- bài thống kê PG-027 — nên P03 dùng md5.)
--
-- Phân bố CÓ CHỦ ĐÍCH (mô tả đầy đủ + số kiểm ở 02-kiem-tra-fixture.sql và p03/README.md):
--   * thành phố SG 55 % · HN 32 % · DN 13 %; dịch vụ ride 40 % · food 45 % · parcel 15 %;
--   * 3 khách DOANH NGHIỆP chiếm rất nhiều chuyến (B1 ≈ 5 % toàn bảng) — PG-027;
--   * khách cá nhân lệch kiểu "số ít đi rất nhiều": chỉ số khách = ⌊R·u²⌋;
--   * khu đón (pickup_area_id) PHỤ THUỘC HOÀN TOÀN vào thành phố — PG-027 (CREATE STATISTICS);
--   * heap chèn theo thứ tự thời gian đặt ⇒ thẳng hàng với requested_at và code — PG-022/024/028;
--   * trips.id là uuid dạng v4 "ngẫu nhiên" (md5) ⇒ không thẳng hàng với heap — PG-024.
--
-- Kỹ thuật nạp lô lớn (đáng ghi nhớ): tạm BỎ index phụ và khoá ngoại của trips/
-- payments, nạp bằng một câu INSERT … SELECT, rồi TẠO LẠI (CREATE INDEX dựng cây
-- bằng sắp xếp một lượt; ADD FOREIGN KEY kiểm cả bảng bằng một phép join) —
-- nhanh hơn nhiều so với cập nhật 5 index + 5 lần tra khoá ngoại cho từng dòng.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';
-- Chỉ trong phiên nạp: cho phép sắp xếp 1 triệu dòng trong bộ nhớ và dựng index
-- nhanh hơn. Không đổi cấu hình cluster mà các bài dùng (work_mem = 4MB).
SET work_mem = '256MB';
SET maintenance_work_mem = '512MB';

-- ---------------------------------------------------------------------------
-- p03_u(h, i): số thực đều trong [0, 1) lấy từ đoạn 32 bit thứ i (0..3) của chuỗi md5 h.
-- Hai hàm dưới là hàm SQL một biểu thức, IMMUTABLE ⇒ planner "nhúng" (inline) thẳng
-- vào truy vấn, không tốn lời gọi hàm. Điều kiện: thân hàm chỉ gọi hàm IMMUTABLE
-- và trả đúng kiểu khai báo. Bẫy đã gặp khi dựng lab: viết `nhan || n` với n là
-- bigint sẽ gọi anytextcat (STABLE) ⇒ hàm KHÔNG được nhúng ⇒ nạp 1 triệu chuyến
-- chậm hơn ~5 lần. Vì vậy ghi rõ n::text và chia cho float8.
CREATE FUNCTION p03_u(h text, i integer) RETURNS double precision
LANGUAGE sql IMMUTABLE STRICT PARALLEL SAFE
AS $$ SELECT ('x' || substr(h, i * 8 + 1, 8))::bit(32)::bigint / 4294967296.0::float8 $$;

-- p03_uuid(nhan, n): uuid dạng v4 (bit phiên bản = 4, biến thể = 8) suy từ md5 — trông ngẫu nhiên
-- nhưng tất định; cùng (nhãn, số) luôn ra cùng uuid nên các bảng tự khớp khoá ngoại.
CREATE FUNCTION p03_uuid(nhan text, n bigint) RETURNS uuid
LANGUAGE sql IMMUTABLE STRICT PARALLEL SAFE
AS $$ SELECT overlay(overlay(md5(nhan || n::text) placing '4' from 13) placing '8' from 17)::uuid $$;

-- ---------------------------------------------------------------------------
-- Danh mục
-- expect-rows: 3
INSERT INTO cities (id, code, name) VALUES
  (1, 'SG', 'TP. Hồ Chí Minh'),
  (2, 'HN', 'Hà Nội'),
  (3, 'DN', 'Đà Nẵng');

-- Cây khu vực (hư cấu): SG 5 vùng × 8 khu, HN 4 vùng × 6 khu, DN 2 vùng × 4 khu
-- ⇒ 3 thành phố + 11 vùng + 72 khu. id khu = id vùng × 10 + số khu (111…158, 211…246, 311…324).
-- expect-rows: 86
INSERT INTO areas (id, parent_id, code, name, level, city_id)
SELECT id, NULL, code, name, 'city', id FROM (VALUES (1, 'SG', 'TP. Hồ Chí Minh'), (2, 'HN', 'Hà Nội'), (3, 'DN', 'Đà Nẵng')) AS c (id, code, name)
UNION ALL
SELECT c.id * 10 + v, c.id, c.code || '-V' || v, 'Vùng ' || v, 'district', NULL
FROM (VALUES (1, 'SG', 5), (2, 'HN', 4), (3, 'DN', 2)) AS c (id, code, so_vung)
CROSS JOIN LATERAL generate_series(1, c.so_vung) AS v
UNION ALL
SELECT (c.id * 10 + v) * 10 + k, c.id * 10 + v, c.code || '-V' || v || '-' || lpad(k::text, 2, '0'), 'Khu ' || lpad(k::text, 2, '0'), 'zone', NULL
FROM (VALUES (1, 'SG', 5, 8), (2, 'HN', 4, 6), (3, 'DN', 2, 4)) AS c (id, code, so_vung, so_khu)
CROSS JOIN LATERAL generate_series(1, c.so_vung) AS v
CROSS JOIN LATERAL generate_series(1, c.so_khu) AS k;

-- 200.000 khách cá nhân: 1–110.000 cư trú SG, 110.001–175.000 HN, 175.001–200.000 DN
-- (thành phố cư trú không lưu, chỉ dùng để sinh chuyến). Số điện thoại 0910000001…
-- expect-rows: 200000
INSERT INTO customers (id, phone, full_name, created_at, kind)
SELECT p03_uuid('kh', n),
       '091' || lpad(n::text, 7, '0'),
       (ARRAY['Nguyễn', 'Trần', 'Lê', 'Phạm', 'Hoàng', 'Huỳnh', 'Phan', 'Vũ', 'Võ', 'Đặng', 'Bùi', 'Đỗ', 'Hồ', 'Ngô', 'Dương', 'Lý'])[1 + n % 16]
         || ' ' || (ARRAY['Văn', 'Thị', 'Minh', 'Thu', 'Quốc', 'Ngọc', 'Hữu', 'Thanh', 'Gia', 'Bảo'])[1 + (n / 16) % 10]
         || ' ' || (ARRAY['An', 'Bình', 'Châu', 'Dũng', 'Giang', 'Hải', 'Khánh', 'Linh', 'Mai', 'Nam', 'Oanh', 'Phong',
                          'Quân', 'Sơn', 'Tâm', 'Uyên', 'Việt', 'Xuân', 'Yến', 'Hạnh', 'Lộc', 'Trinh', 'Tùng', 'Vy'])[1 + (n / 160) % 24],
       TIMESTAMPTZ '2024-01-01 08:00:00+07' + n * interval '5 minutes',
       'personal'
FROM generate_series(1, 200000) AS n;

-- 3 khách doanh nghiệp (đặt qua API): B1 bưu kiện SG, B2 cơm trưa văn phòng HN, B3 bưu kiện DN.
-- expect-rows: 3
INSERT INTO customers (id, phone, full_name, created_at, kind)
VALUES (p03_uuid('kh-dn', 1), '0919990001', 'Cửa hàng Phụ Kiện Lab (KHDN)', TIMESTAMPTZ '2025-03-15 09:00:00+07', 'business'),
       (p03_uuid('kh-dn', 2), '0919990002', 'Bếp Trưa Văn Phòng Lab (KHDN)', TIMESTAMPTZ '2025-06-01 09:00:00+07', 'business'),
       (p03_uuid('kh-dn', 3), '0919990003', 'Shop Hoa Tươi Lab (KHDN)', TIMESTAMPTZ '2025-09-10 09:00:00+07', 'business');

-- 6.000 tài xế: mã 20001–23600 SG, 23601–25400 HN, 25401–26000 DN.
-- expect-rows: 6000
INSERT INTO drivers (id, code, full_name, city_id, vehicle_type, status, created_at)
SELECT p03_uuid('tx', n), 20000 + n,
       (ARRAY['Nguyễn', 'Trần', 'Lê', 'Phạm', 'Hoàng', 'Phan', 'Vũ', 'Võ', 'Đặng', 'Bùi', 'Đỗ', 'Ngô'])[1 + n % 12]
         || ' ' || (ARRAY['Văn', 'Minh', 'Quốc', 'Hữu', 'Thanh', 'Đức', 'Công', 'Thị'])[1 + (n / 12) % 8]
         || ' ' || (ARRAY['Tài', 'Lực', 'Phúc', 'Thắng', 'Hiếu', 'Kiên', 'Trung', 'Nghĩa', 'Đạt', 'Hùng', 'Lan', 'Hoa'])[1 + (n / 96) % 12],
       CASE WHEN n <= 3600 THEN 1 WHEN n <= 5400 THEN 2 ELSE 3 END,
       CASE WHEN p03_u(md5('tx' || n), 0) < 0.7 THEN 'bike' ELSE 'car' END,
       CASE WHEN p03_u(md5('tx' || n), 1) < 0.3 THEN 'online' ELSE 'offline' END,
       TIMESTAMPTZ '2025-01-01 08:00:00+07' + n * interval '1 hour'
FROM generate_series(1, 6000) AS n;

-- 15.000 quán: 1–9.000 SG, 9.001–13.500 HN, 13.501–15.000 DN. Tên ghép từ món +
-- chủ quán (+ hậu tố), nên có nhiều quán trùng tên kiểu chuỗi cửa hàng; địa chỉ
-- dùng tên đường HƯ CẤU. attrs: tags món, giao nhanh (~30 %), giờ mở cửa.
-- expect-rows: 15000
INSERT INTO merchants (id, city_id, name, created_at, address, attrs)
SELECT p03_uuid('q', n), city_id, ten, TIMESTAMPTZ '2024-06-01 08:00:00+07' + n * interval '30 minutes', dia_chi, attrs
FROM (
  SELECT n, h,
         CASE WHEN n <= 9000 THEN 1 WHEN n <= 13500 THEN 2 ELSE 3 END AS city_id,
         m.mon || ' ' || (ARRAY['Cô Ba', 'Bà Năm', 'Chú Tư', 'Anh Hai', 'Chị Út', 'O Hà', 'Bác Sáu', 'Dì Bảy',
                                'Ông Tám', 'Cậu Mười', 'Mợ Chín', 'Bé Na'])[1 + floor(p03_u(h, 1) * 12)::int]
           || (ARRAY['', '', '', ' Gia Truyền', ' 24h', ' - Chi Nhánh 2', ' - Chi Nhánh 3', ' Sạch'])[1 + floor(p03_u(h, 2) * 8)::int] AS ten,
         (1 + floor(p03_u(h, 3) * 480)::int) || ' Đường '
           || (ARRAY['Hoa Sứ', 'Cây Gòn', 'Bến Đá', 'Sông Mây', 'Ngọc Lan', 'Đồng Xanh', 'Phú Mỹ Hạ', 'Bạch Mai Tây',
                     'Lò Gốm Cũ', 'Kênh Đào', 'Cầu Tre', 'Vườn Chuối', 'Hồ Sen', 'Ga Cũ', 'Chợ Nổi', 'Mây Trắng',
                     'Hải Đăng', 'Rạch Cát', 'Bàu Sen', 'Gò Dầu Hạ'])[1 + (n * 7) % 20]
           || ', Phường ' || (1 + n % 15) || ', Quận ' || (1 + (n / 15) % 12)
           || ', ' || (ARRAY['TP. Hồ Chí Minh', 'Hà Nội', 'Đà Nẵng'])[CASE WHEN n <= 9000 THEN 1 WHEN n <= 13500 THEN 2 ELSE 3 END] AS dia_chi,
         jsonb_build_object(
           'tags', CASE WHEN p03_u(h, 2) < 0.08 THEN jsonb_build_array(m.tag, 'chay') ELSE jsonb_build_array(m.tag) END,
           'giao_nhanh', p03_u(h, 3) < 0.3,
           'gio_mo', lpad((5 + floor(p03_u(h, 1) * 6)::int)::text, 2, '0') || ':00') AS attrs
  FROM generate_series(1, 15000) AS n
  CROSS JOIN LATERAL (SELECT md5('q' || n) AS h) AS x
  CROSS JOIN LATERAL (
    SELECT (ARRAY['Cơm Tấm', 'Bún Bò Huế', 'Phở Bò', 'Bánh Mì', 'Bún Chả', 'Hủ Tiếu Nam Vang', 'Trà Sữa', 'Cà Phê',
                  'Bánh Xèo', 'Gà Rán', 'Lẩu Thái', 'Cháo Lòng', 'Bún Riêu', 'Mì Quảng', 'Cơm Gà', 'Bánh Cuốn',
                  'Xôi Gà', 'Chè', 'Bò Kho', 'Cơm Văn Phòng'])[1 + floor(p03_u(x.h, 0) * 20)::int] AS mon,
           (ARRAY['com', 'bun', 'pho', 'banh_mi', 'bun', 'hu_tieu', 'tra_sua', 'ca_phe', 'banh_xeo', 'ga_ran',
                  'lau', 'chao', 'bun', 'mi', 'com', 'banh_cuon', 'xoi', 'che', 'bo_kho', 'com'])[1 + floor(p03_u(x.h, 0) * 20)::int] AS tag
  ) AS m
) AS q
ORDER BY n;

-- ---------------------------------------------------------------------------
-- Nạp chuyến. Bỏ tạm 3 index phụ + 5 khoá ngoại của trips (tạo lại bên dưới,
-- đúng định nghĩa của p01/01-schema.sql và p02/00-mo-rong-schema.sql).
DROP INDEX ix_trips_city_requested;
DROP INDEX ix_trips_customer_requested;
DROP INDEX ix_trips_driver_id;
ALTER TABLE trips DROP CONSTRAINT fk_trips_city, DROP CONSTRAINT fk_trips_customer,
  DROP CONSTRAINT fk_trips_driver, DROP CONSTRAINT fk_trips_merchant, DROP CONSTRAINT fk_trips_pickup_area;

-- Chuyến k = 1 … 999.700 rải trên 183 ngày (01/05 → 30/10): ngày cuối tuần đông
-- hơn 15 %, số chuyến tăng dần ~35 % từ đầu tới cuối kỳ; trong ngày theo 24 trọng
-- số giờ của P02 (cao điểm trưa và chiều tối). k = 999.701 … 1.000.000 là 300
-- chuyến đang chạy, đặt cách nhau 12 giây từ 31/10 00:00. Mã chuyến = 99.999 + k
-- tăng đúng theo giờ đặt; INSERT theo thứ tự mã ⇒ heap thẳng hàng với thời gian.
-- expect-rows: 1000000
INSERT INTO trips (id, code, kind, city_id, customer_id, driver_id, merchant_id, status, fare,
                   requested_at, accepted_at, completed_at, updated_at, pickup_area_id)
OVERRIDING SYSTEM VALUE
-- Trọng số 24 giờ (tổng 100) và mốc luỹ kế đầu mỗi giờ: vị trí p ∈ [0, 100) của
-- chuyến trong ngày rơi vào giờ width_bucket(p, moc) — không cần phép join.
WITH hang (w, moc) AS (
  SELECT ARRAY[2, 1, 1, 1, 1, 2, 4, 6, 6, 4, 4, 8, 9, 5, 3, 3, 4, 7, 9, 7, 5, 4, 2, 2]::float8[],
         ARRAY[0, 2, 3, 4, 5, 6, 8, 12, 18, 24, 28, 32, 40, 49, 54, 57, 60, 64, 71, 80, 87, 92, 96, 98]::float8[]
),
ngay AS (
  SELECT d, (CASE WHEN extract(isodow FROM DATE '2026-05-01' + d) >= 6 THEN 1.15 ELSE 1.0 END) * (0.85 + 0.30 * d / 182.0) AS w
  FROM generate_series(0, 182) AS d
),
-- MATERIALIZED: số chuyến từng ngày tính một lần (183 dòng), không bị chép vào
-- từng dòng của 1 triệu chuyến.
dem AS MATERIALIZED (
  SELECT d, floor(999700 * c0 / tong)::int AS k0,
         floor(999700 * (c0 + w) / tong)::int - floor(999700 * c0 / tong)::int AS so
  FROM (SELECT d, w, sum(w) OVER (ORDER BY d) - w AS c0, sum(w) OVER () AS tong FROM ngay) AS x
),
sinh AS (
  SELECT dem.k0 + i + 1 AS k, dem.d, (i + 0.5) * 100.0::float8 / dem.so AS p
  FROM dem CROSS JOIN LATERAL generate_series(0, dem.so - 1) AS i
),
thoi_diem AS (
  SELECT s.k,
         ((DATE '2026-05-01' + s.d)::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh')
           + make_interval(hours => g.gio - 1,
                           secs => floor((s.p - hang.moc[g.gio]) / hang.w[g.gio] * 3600)) AS requested_at,
         NULL::text AS trang_thai_dat
  FROM sinh AS s CROSS JOIN hang CROSS JOIN LATERAL (SELECT width_bucket(s.p, hang.moc) AS gio) AS g
  UNION ALL
  SELECT k, TIMESTAMPTZ '2026-10-31 00:00:00+07' + (k - 999701) * interval '12 seconds',
         (ARRAY['requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit'])[1 + k % 6]
  FROM generate_series(999701, 1000000) AS k
),
-- MATERIALIZED + OFFSET 0: tính 3 chuỗi md5 và 12 số u MỘT lần mỗi chuyến rồi lưu
-- lại. Thiếu hai chốt này, planner "kéo phẳng" các CTE phía sau và tính lại md5/p03_u
-- ở MỌI chỗ tham chiếu (city_id dùng ở hơn chục chỗ) — nạp chậm hơn vài lần.
ngau_nhien AS MATERIALIZED (
  SELECT t.k, t.requested_at, t.trang_thai_dat,
         p03_u(h.ha, 0) AS ua0, p03_u(h.ha, 1) AS ua1, p03_u(h.ha, 2) AS ua2, p03_u(h.ha, 3) AS ua3,
         p03_u(h.hb, 0) AS ub0, p03_u(h.hb, 1) AS ub1, p03_u(h.hb, 2) AS ub2, p03_u(h.hb, 3) AS ub3,
         p03_u(h.hc, 0) AS uc0, p03_u(h.hc, 1) AS uc1, p03_u(h.hc, 2) AS uc2, p03_u(h.hc, 3) AS uc3
  FROM thoi_diem AS t
  CROSS JOIN LATERAL (SELECT md5('a' || t.k::text) AS ha, md5('b' || t.k::text) AS hb, md5('c' || t.k::text) AS hc OFFSET 0) AS h
),
b1 AS (                                      -- thành phố, dịch vụ
  SELECT n.*,
         CASE WHEN ua0 < 0.55 THEN 1 WHEN ua0 < 0.87 THEN 2 ELSE 3 END::smallint AS city_id,
         CASE WHEN ua1 < 0.40 THEN 'ride' WHEN ua1 < 0.85 THEN 'food' ELSE 'parcel' END AS kind
  FROM ngau_nhien AS n
),
b2 AS (                                      -- khách doanh nghiệp? kết cục? dải id theo thành phố
  SELECT b1.*,
         CASE WHEN city_id = 1 AND kind = 'parcel' AND ua2 < 0.62 THEN 1
              WHEN city_id = 2 AND kind = 'food'   AND ua2 < 0.085 THEN 2
              WHEN city_id = 3 AND kind = 'parcel' AND ua2 < 0.16 THEN 3 END AS kh_dn,
         CASE WHEN trang_thai_dat IS NOT NULL THEN trang_thai_dat
              WHEN ub0 < 0.07 THEN 'huy_truoc_nhan'
              WHEN ub0 < 0.12 THEN 'huy_sau_nhan'
              ELSE 'completed' END AS ket_cuc,
         (ARRAY[1, 110001, 175001])[city_id] AS kh_dau, (ARRAY[110000, 65000, 25000])[city_id] AS kh_so,
         (ARRAY[1, 3601, 5401])[city_id] AS tx_dau, (ARRAY[3600, 1800, 600])[city_id] AS tx_so,
         (ARRAY[1, 9001, 13501])[city_id] AS q_dau, (ARRAY[9000, 4500, 1500])[city_id] AS q_so
  FROM b1
),
b3 AS (
  SELECT b2.*,
         ket_cuc NOT IN ('huy_truoc_nhan', 'requested', 'searching') AS co_tai_xe,
         CASE WHEN kh_dn IS NOT NULL THEN p03_uuid('kh-dn', kh_dn)
              ELSE p03_uuid('kh', kh_dau + floor(kh_so * ua3 ^ 2)::int) END AS customer_id,
         CASE WHEN ket_cuc NOT IN ('huy_truoc_nhan', 'requested', 'searching')
              THEN p03_uuid('tx', tx_dau + floor(tx_so * ub1)::int) END AS driver_id,
         CASE WHEN kind = 'food' THEN p03_uuid('q', q_dau + floor(q_so * ub2 ^ 1.5)::int) END AS merchant_id,
         CASE WHEN ub3 < 0.01 THEN NULL                         -- 1 %: mất GPS
              WHEN city_id = 1 THEN (11 + floor(5 * uc0)::int) * 10 + 1 + floor(8 * uc1 ^ 1.3)::int
              WHEN city_id = 2 THEN (21 + floor(4 * uc0)::int) * 10 + 1 + floor(6 * uc1 ^ 1.3)::int
              ELSE                  (31 + floor(2 * uc0)::int) * 10 + 1 + floor(4 * uc1 ^ 1.3)::int
         END AS pickup_area_id,
         CASE kind
           WHEN 'ride' THEN CASE WHEN uc2 < 0.02 THEN 0 ELSE 12000 + floor(uc3 * 89)::int * 1000 END
           WHEN 'food' THEN 25000 + floor(uc3 * 126)::int * 1000
           ELSE             18000 + floor(uc3 * 63)::int * 1000
         END AS fare,
         20 + floor(ua2 * 280)::int AS giay_cho_nhan,
         3 + floor(ub2 * 8)::int AS phut_toi_don,
         5 + floor(uc2 * 30)::int AS phut_di
  FROM b2
),
b4 AS (
  SELECT b3.*,
         CASE WHEN co_tai_xe THEN requested_at + giay_cho_nhan * interval '1 second' END AS accepted_at
  FROM b3
),
b5 AS (
  SELECT b4.*,
         CASE WHEN ket_cuc IN ('completed', 'picked_up', 'in_transit')
              THEN accepted_at + phut_toi_don * interval '1 minute' END AS picked_up_at
  FROM b4
)
SELECT p03_uuid('cx', k), 99999 + k, kind, city_id, customer_id, driver_id, merchant_id,
       CASE WHEN ket_cuc IN ('huy_truoc_nhan', 'huy_sau_nhan') THEN 'cancelled' ELSE ket_cuc END,
       fare, requested_at, accepted_at,
       CASE WHEN ket_cuc = 'completed' THEN picked_up_at + phut_di * interval '1 minute' END,
       CASE ket_cuc
         WHEN 'completed'      THEN picked_up_at + phut_di * interval '1 minute'
         WHEN 'huy_truoc_nhan' THEN requested_at + (60 + giay_cho_nhan) * interval '1 second'
         WHEN 'huy_sau_nhan'   THEN accepted_at + (30 + phut_di * 10) * interval '1 second'
         WHEN 'requested'      THEN requested_at
         WHEN 'searching'      THEN requested_at + interval '2 seconds'
         WHEN 'accepted'       THEN accepted_at
         WHEN 'arriving'       THEN accepted_at + interval '10 seconds'
         WHEN 'picked_up'      THEN picked_up_at
         ELSE                       picked_up_at + interval '30 seconds'
       END,
       pickup_area_id
FROM b5
ORDER BY k;

-- Mã chuyến tạo mới tiếp tục sau mã lớn nhất đã nạp.
-- expect-value: 1099999
SELECT setval(pg_get_serial_sequence('trips', 'code'), (SELECT max(code) FROM trips)) AS ma_cuoi;

-- Tạo lại khoá ngoại (kiểm cả bảng một lượt) và 3 index phụ — đúng định nghĩa gốc.
ALTER TABLE trips ADD CONSTRAINT fk_trips_city     FOREIGN KEY (city_id)     REFERENCES cities (id);
ALTER TABLE trips ADD CONSTRAINT fk_trips_customer FOREIGN KEY (customer_id) REFERENCES customers (id);
ALTER TABLE trips ADD CONSTRAINT fk_trips_driver   FOREIGN KEY (driver_id)   REFERENCES drivers (id);
ALTER TABLE trips ADD CONSTRAINT fk_trips_merchant FOREIGN KEY (merchant_id) REFERENCES merchants (id);
ALTER TABLE trips ADD CONSTRAINT fk_trips_pickup_area FOREIGN KEY (pickup_area_id) REFERENCES areas (id);
CREATE INDEX ix_trips_city_requested     ON trips (city_id, requested_at);
CREATE INDEX ix_trips_customer_requested ON trips (customer_id, requested_at DESC);
CREATE INDEX ix_trips_driver_id          ON trips (driver_id);

-- ---------------------------------------------------------------------------
-- Thanh toán (đơn giản hơn P02 — P03 không đối soát, chỉ cần khối lượng thật):
--   * chuyến hoàn thành, cước > 0: một lần thu thành công; tiền mặt ~40 %, ví ~35 %, thẻ ~25 %;
--     ví/thẻ ~3 %: lần đầu `failed`, 1 phút sau thu lại thành công;
--   * chuyến huỷ SAU khi nhận, trả thẻ, ~50 %: đã trừ lúc nhận, hoàn đủ lúc huỷ;
--   * chuyến đang chạy, cước 0 đ, huỷ trước nhận: không có giao dịch.
-- Thứ tự chèn theo (mã chuyến, lượt) ⇒ payments cũng thẳng hàng với thời gian.
DROP INDEX ix_payments_trip;
ALTER TABLE payments DROP CONSTRAINT fk_payments_trip;

-- expect-rows: 901208
INSERT INTO payments (trip_id, kind, method, status, amount, provider_ref, paid_at)
WITH c AS MATERIALIZED (
  SELECT t.id AS trip_id, t.code, t.status, t.fare, t.accepted_at, t.completed_at, t.updated_at,
         CASE WHEN p03_u(x.h, 0) < 0.40 THEN 'cash' WHEN p03_u(x.h, 0) < 0.75 THEN 'wallet' ELSE 'card' END AS pt,
         p03_u(x.h, 1) < 0.03 AS loi_lan_dau,
         p03_u(x.h, 2) < 0.50 AS tru_truoc
  FROM trips AS t CROSS JOIN LATERAL (SELECT md5('d' || t.code) AS h OFFSET 0) AS x
  WHERE t.status IN ('completed', 'cancelled') AND t.fare > 0
)
SELECT c.trip_id, v.kind, v.method, v.status, v.amount,
       CASE v.method WHEN 'wallet' THEN 'VX-' WHEN 'card' THEN 'TH-' END || c.code || '-' || v.luot, v.paid_at
FROM c
CROSS JOIN LATERAL (VALUES
  -- (luot, kind, method, status, amount, paid_at, có dòng này không)
  (1, 'charge', c.pt, 'failed',    c.fare, c.completed_at,                       c.status = 'completed' AND c.pt <> 'cash' AND c.loi_lan_dau),
  (2, 'charge', c.pt, 'succeeded', c.fare, c.completed_at + CASE WHEN c.pt <> 'cash' AND c.loi_lan_dau THEN interval '1 minute' ELSE interval '0' END,
                                                                                  c.status = 'completed'),
  (3, 'charge', 'card', 'succeeded', c.fare, c.accepted_at,                       c.status = 'cancelled' AND c.accepted_at IS NOT NULL AND c.pt = 'card' AND c.tru_truoc),
  (4, 'refund', 'card', 'succeeded', c.fare, c.updated_at,                        c.status = 'cancelled' AND c.accepted_at IS NOT NULL AND c.pt = 'card' AND c.tru_truoc)
) AS v (luot, kind, method, status, amount, paid_at, co)
WHERE v.co
ORDER BY c.code, v.luot;

ALTER TABLE payments ADD CONSTRAINT fk_payments_trip FOREIGN KEY (trip_id) REFERENCES trips (id);
CREATE INDEX ix_payments_trip ON payments (trip_id);

-- ---------------------------------------------------------------------------
-- Ca làm việc tháng 10 của 6.000 tài xế: ~85 % số ngày có ca; mỗi tài xế một
-- khung quen (sáng 05:xx, chiều 13:xx, đêm 20:xx), dài 8–10 giờ ⇒ tự nhiên không
-- chồng nhau (ca đêm kết thúc trước 07:00 hôm sau).
-- expect-rows: 158172
INSERT INTO driver_shifts (driver_id, shift, created_at)
SELECT p03_uuid('tx', n),
       tstzrange(bd, bd + (8 + floor(p03_u(h, 1) * 3)::int) * interval '1 hour', '[)'),
       TIMESTAMPTZ '2026-09-25 08:00:00+07'
FROM generate_series(1, 6000) AS n
CROSS JOIN generate_series(0, 30) AS d
CROSS JOIN LATERAL (SELECT md5('ca' || n || '-' || d) AS h) AS x
CROSS JOIN LATERAL (
  SELECT ((DATE '2026-10-01' + d)::timestamp AT TIME ZONE 'Asia/Ho_Chi_Minh')
         + (ARRAY[5, 13, 20])[1 + n % 3] * interval '1 hour'
         + floor(p03_u(h, 2) * 60)::int * interval '1 minute' AS bd
) AS b
WHERE p03_u(h, 0) < 0.85
ORDER BY d, n;

-- CÀI SẴN 3 ca nhập trùng/chồng giờ (lỗi nhập liệu) cho PG-028: tài xế 20007 có
-- ca 15/10 chồng một phần, 23605 có ca nhập hai lần y hệt, 25401 có ca bao trùm.
-- expect-rows: 3
INSERT INTO driver_shifts (driver_id, shift, created_at)
SELECT p03_uuid('tx', v.n), v.ca, TIMESTAMPTZ '2026-10-20 10:00:00+07'
FROM (
  SELECT 7 AS n, (SELECT tstzrange(lower(shift) + interval '2 hours', upper(shift) + interval '2 hours')
                  FROM driver_shifts WHERE driver_id = p03_uuid('tx', 7) AND lower(shift) >= TIMESTAMPTZ '2026-10-15 00:00+07' ORDER BY lower(shift) LIMIT 1) AS ca
  UNION ALL
  SELECT 3605, (SELECT shift FROM driver_shifts WHERE driver_id = p03_uuid('tx', 3605) AND lower(shift) >= TIMESTAMPTZ '2026-10-15 00:00+07' ORDER BY lower(shift) LIMIT 1)
  UNION ALL
  SELECT 5401, (SELECT tstzrange(lower(shift) - interval '1 hour', upper(shift) + interval '1 hour')
                FROM driver_shifts WHERE driver_id = p03_uuid('tx', 5401) AND lower(shift) >= TIMESTAMPTZ '2026-10-15 00:00+07' ORDER BY lower(shift) LIMIT 1)
) AS v;

-- ---------------------------------------------------------------------------
-- Dọn công cụ sinh dữ liệu? KHÔNG: p03_u / p03_uuid được GIỮ LẠI vì các bài
-- (25, 28, 29, 30) dùng để sinh thêm dữ liệu tất định ngay trong bài.

-- VACUUM: đặt hint bit, dựng visibility map (mọi page "all-visible") và cập nhật
-- thống kê (ANALYZE). Nền của mọi bài bắt đầu từ trạng thái "vừa VACUUM xong".
-- Cluster lab tắt autovacuum (tái lập) ⇒ phải chạy tay ở đây.
VACUUM (ANALYZE) cities, areas, customers, drivers, merchants, trips, payments, driver_shifts;

-- Số dòng sau khi nạp.
-- expect-line: 3 | 86 | 200003 | 6000 | 15000 | 1000000 | 901208 | 158175 | 0 | 0 | 0
SELECT (SELECT count(*) FROM cities)           AS cities,
       (SELECT count(*) FROM areas)            AS areas,
       (SELECT count(*) FROM customers)        AS customers,
       (SELECT count(*) FROM drivers)          AS drivers,
       (SELECT count(*) FROM merchants)        AS merchants,
       (SELECT count(*) FROM trips)            AS trips,
       (SELECT count(*) FROM payments)         AS payments,
       (SELECT count(*) FROM driver_shifts)    AS driver_shifts,
       (SELECT count(*) FROM driver_locations) AS driver_locations,
       (SELECT count(*) FROM trip_events)      AS trip_events,
       (SELECT count(*) FROM trip_ratings)     AS trip_ratings;
