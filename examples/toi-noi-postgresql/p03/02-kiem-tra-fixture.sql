-- P03 · 02-kiem-tra-fixture.sql — hồ sơ dữ liệu của nền P03: số dòng, phân bố,
-- các điểm "cài sẵn" cho từng bài, kích thước bảng/index. Mọi con số dưới đây
-- TẤT ĐỊNH (dữ liệu sinh bằng md5) — trừ chỗ đánh dấu `lab: bien-dong` (thống kê
-- do ANALYZE lấy mẫu ngẫu nhiên 30.000 dòng).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- H1 · Chuyến theo thành phố × dịch vụ.
-- expect-line: DN | food   | 58458  | 51518
-- expect-line: DN | parcel | 19664  | 17358
-- expect-line: DN | ride   | 51673  | 45569
-- expect-line: HN | food   | 144333 | 127055
-- expect-line: HN | parcel | 48321  | 42579
-- expect-line: HN | ride   | 128031 | 112631
-- expect-line: SG | food   | 247407 | 217505
-- expect-line: SG | parcel | 82357  | 72363
-- expect-line: SG | ride   | 219756 | 193464
SELECT c.code AS tp, t.kind, count(*) AS so_chuyen, count(*) FILTER (WHERE t.status = 'completed') AS hoan_thanh
FROM trips AS t JOIN cities AS c ON c.id = t.city_id
GROUP BY c.code, t.kind
ORDER BY c.code COLLATE "C", t.kind COLLATE "C";

-- H2 · Theo trạng thái: 300 chuyến đang chạy (mỗi trạng thái 50) — nền của partial index (PG-025).
-- expect-line: accepted   | 50
-- expect-line: arriving   | 50
-- expect-line: cancelled  | 119658
-- expect-line: completed  | 880042
-- expect-line: in_transit | 50
-- expect-line: picked_up  | 50
-- expect-line: requested  | 50
-- expect-line: searching  | 50
SELECT status, count(*) FROM trips GROUP BY status ORDER BY status COLLATE "C";

-- H3 · Theo tháng (giờ VN): số chuyến tăng dần — bảng chèn theo thời gian.
-- expect-line: 2026-05 | 148978
-- expect-line: 2026-06 | 151248
-- expect-line: 2026-07 | 164553
-- expect-line: 2026-08 | 174793
-- expect-line: 2026-09 | 176032
-- expect-line: 2026-10 | 184396
SELECT to_char(requested_at, 'YYYY-MM') AS thang, count(*) FROM trips GROUP BY 1 ORDER BY 1;

-- H4 · Khách đặt nhiều nhất: 3 khách doanh nghiệp dẫn đầu (B1 ≈ 5 % bảng trips),
-- sau đó là vài khách cá nhân "đi rất nhiều" (đuôi lệch do chỉ số khách = ⌊R·u²⌋).
-- Đây là nền của PG-027 (MCV) và PG-030 (lịch sử chuyến của khách đặt nhiều).
-- expect-line: 0919990001 | business | 51151
-- expect-line: 0919990002 | business | 12435
-- expect-line: 0919990003 | business | 3124
-- expect-line: 0910000001 | personal | 1533
-- expect-line: 0910110001 | personal | 1214
-- expect-line: 0910175001 | personal | 777
SELECT c.phone, c.kind, count(*) AS so_chuyen
FROM trips AS t JOIN customers AS c ON c.id = t.customer_id
GROUP BY c.phone, c.kind
ORDER BY so_chuyen DESC, c.phone
LIMIT 6;

-- H5 · Phân bố số chuyến mỗi khách: rất lệch. Nhóm "0" = khách chưa đặt chuyến nào.
-- expect-line: 0          | 8151
-- expect-line: 1-2        | 56886
-- expect-line: 3-9        | 119797
-- expect-line: 10-99      | 15053
-- expect-line: 100-999    | 111
-- expect-line: >= 1000    | 5
WITH dem AS (
  SELECT c.id, count(t.id) AS n
  FROM customers AS c LEFT JOIN trips AS t ON t.customer_id = c.id
  GROUP BY c.id
)
SELECT CASE WHEN n = 0 THEN '0' WHEN n <= 2 THEN '1-2' WHEN n <= 9 THEN '3-9' WHEN n <= 99 THEN '10-99'
            WHEN n <= 999 THEN '100-999' ELSE '>= 1000' END AS nhom,
       count(*) AS so_khach
FROM dem GROUP BY 1 ORDER BY min(n);

-- H6 · Khu đón PHỤ THUỘC thành phố: mọi chuyến có khu đều thuộc đúng thành phố
-- của khu (0 dòng sai). Planner KHÔNG biết điều này nếu không có CREATE STATISTICS (PG-027).
-- expect-value: 0
SELECT count(*)
FROM trips AS t
JOIN areas AS z ON z.id = t.pickup_area_id
JOIN areas AS v ON v.id = z.parent_id
WHERE v.parent_id <> t.city_id;

-- Khu đón NULL (mất GPS) ≈ 1 %.
-- expect-value: 10210
SELECT count(*) FROM trips WHERE pickup_area_id IS NULL;

-- H7 · Ca làm việc chồng giờ cài sẵn (PG-028): đúng 3 cặp.
-- expect-line: 20007 | 2
-- expect-line: 23605 | 2
-- expect-line: 25401 | 2
SELECT d.code, count(*) AS so_ca_dinh_nhau
FROM driver_shifts AS a
JOIN driver_shifts AS b ON b.driver_id = a.driver_id AND b.id <> a.id AND b.shift && a.shift
JOIN drivers AS d ON d.id = a.driver_id
GROUP BY d.code ORDER BY d.code;

-- H8 · Heap gần như thẳng hàng với thời gian đặt. Lấy page của chuyến tháng 10 có
-- mã nhỏ nhất làm mốc: gần như mọi chuyến tháng 10 nằm từ page đó trở đi — TRỪ một
-- ít chuyến huỷ-trước-khi-nhận (dòng nhỏ, 116 byte) được đặt vào chỗ trống còn sót
-- ở cuối các page cũ: khi page hiện tại đầy, INSERT hỏi bản đồ chỗ trống (FSM) và
-- dùng lại chỗ đó. "Dòng nằm theo chỗ trống lúc ghi", không theo khoá nào (PG-024).
-- expect-line: 15856 | 11 | 0
WITH moc AS (
  SELECT (ctid::text::point)[0]::int AS p
  FROM trips WHERE code = (SELECT min(code) FROM trips WHERE requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07')
)
SELECT moc.p AS page_dau_thang_10,
       count(*) FILTER (WHERE t.requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07' AND (t.ctid::text::point)[0] < moc.p) AS chuyen_t10_nam_page_cu,
       count(*) FILTER (WHERE t.requested_at <  TIMESTAMPTZ '2026-10-01 00:00+07' AND (t.ctid::text::point)[0] > moc.p) AS chuyen_cu_nam_sau_moc
FROM trips AS t CROSS JOIN moc
GROUP BY moc.p;

-- H9 · Kích thước (byte, tất định với cùng bản PostgreSQL vì nạp cùng thứ tự).
-- Đáng chú ý: ix_trips_driver_id chỉ 7 MB dù có ~880.000 mục — B-tree GỘP các mục
-- trùng khoá (deduplication, từ PG 13): 6.000 tài xế ⇒ mỗi khoá một danh sách ctid.
-- expect-line: trips                       | table | 159260672
-- expect-line: pk_trips                    | index |  39919616
-- expect-line: uq_trips_code               | index |  22487040
-- expect-line: ix_trips_city_requested     | index |  31563776
-- expect-line: ix_trips_customer_requested | index |  40624128
-- expect-line: ix_trips_driver_id          | index |   7077888
-- expect-line: payments                    | table |  88498176
-- expect-line: pk_payments                 | index |  20267008
-- expect-line: uq_payments_provider_ref    | index |  33464320
-- expect-line: ix_payments_trip            | index |  28131328
SELECT c.relname, CASE c.relkind WHEN 'r' THEN 'table' ELSE 'index' END AS loai, pg_relation_size(c.oid) AS bytes
FROM pg_class AS c
WHERE c.relnamespace = 'toinoi'::regnamespace
  AND c.relname IN ('trips', 'pk_trips', 'uq_trips_code', 'ix_trips_city_requested', 'ix_trips_customer_requested',
                    'ix_trips_driver_id', 'payments', 'pk_payments', 'uq_payments_provider_ref', 'ix_payments_trip')
ORDER BY CASE WHEN c.relname LIKE '%payments%' THEN 1 ELSE 0 END, c.relkind DESC, c.oid;

-- Tổng database (bảng + index + TOAST) — để biết vì sao mỗi bài chép database mất vài giây.
-- lab: bien-dong
SELECT pg_size_pretty(pg_database_size(current_database())) AS kich_thuoc_database;

-- H10 · Thống kê sau ANALYZE (lấy MẪU 30.000 dòng ⇒ số lẻ dao động giữa các lần chạy).
-- Heap thẳng hàng tuyệt đối với code và requested_at (correlation = 1), không
-- liên quan tới id (uuid ngẫu nhiên, ≈ 0). BẪY khi đọc: cột ít giá trị (city_id 3
-- giá trị, status 8) vẫn có correlation dương đáng kể dù phân bố ngẫu nhiên theo
-- thời gian — khi sắp mẫu theo giá trị, các dòng trùng giá trị được xếp theo vị trí
-- vật lý, nên correlation của cột ít giá trị bị "thổi phồng". Ước lượng n_distinct
-- của customer_id cũng THẤP hơn thực tế nhiều (≈ 8x.xxx so với ~192.000) — PG-027.
-- lab: bien-dong
SELECT attname, n_distinct, round(correlation::numeric, 3) AS correlation
FROM pg_stats
WHERE schemaname = 'toinoi' AND tablename = 'trips'
  AND attname IN ('id', 'code', 'requested_at', 'city_id', 'customer_id', 'driver_id', 'pickup_area_id', 'status')
ORDER BY abs(correlation) DESC, attname;

-- Kiểm có chặn khoảng thay cho số chính xác: correlation(code) = 1, |correlation(id)| < 0,05.
-- expect-line: t | t
SELECT max(correlation) FILTER (WHERE attname = 'code') > 0.999,
       abs(max(correlation) FILTER (WHERE attname = 'id')) < 0.05
FROM pg_stats WHERE schemaname = 'toinoi' AND tablename = 'trips';
