-- P02 · 02-kiem-tra-fixture.sql — hình dạng bộ dữ liệu và bất biến của nó.
--
-- Chạy ngay sau 01. Nửa đầu in "hồ sơ" dữ liệu để bài viết trích số; nửa sau
-- là các kiểm tra bất biến (đúng = 0 dòng), gồm các kiểm tra nhiều bảng của
-- p01/07-checks.sql áp lên dữ liệu lớn, cộng quy tắc của các bảng mới.
-- Truy vấn dùng GROUP BY / JOIN / NOT EXISTS — giải thích ở PG-012…PG-014.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ===========================================================================
-- Hồ sơ dữ liệu
-- ===========================================================================

-- H1. Chuyến theo thành phố × loại dịch vụ × kết cục.
-- expect-md5: bf39c7882537b2a1571f12783e4242c3
SELECT c.code AS tp, t.kind,
       count(*)                                                         AS tong,
       count(*) FILTER (WHERE t.status = 'completed')                   AS hoan_thanh,
       count(*) FILTER (WHERE t.status = 'cancelled' AND t.driver_id IS NULL)     AS huy_truoc_nhan,
       count(*) FILTER (WHERE t.status = 'cancelled' AND t.driver_id IS NOT NULL) AS huy_sau_nhan,
       count(*) FILTER (WHERE t.status NOT IN ('completed', 'cancelled'))         AS dang_chay
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
GROUP BY c.code, t.kind
ORDER BY c.code COLLATE "C", t.kind COLLATE "C";

-- H2. Đủ 8 trạng thái của vòng đời.
-- expect-line: requested  | 1
-- expect-line: searching  | 1
-- expect-line: accepted   | 1
-- expect-line: arriving   | 1
-- expect-line: picked_up  | 1
-- expect-line: in_transit | 1
-- expect-line: completed  | 2707
-- expect-line: cancelled  | 461
SELECT status, count(*) AS so_chuyen
FROM trips
GROUP BY status
ORDER BY array_position(
  ARRAY['requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled'],
  status);

-- H3. Giá trị NULL và giá trị 0 có chủ đích (PG-011).
-- expect-line: 297 | 1742 | 36 | 38
SELECT count(*) FILTER (WHERE driver_id IS NULL)      AS khong_tai_xe,
       count(*) FILTER (WHERE merchant_id IS NULL)    AS khong_quan,
       count(*) FILTER (WHERE pickup_area_id IS NULL) AS mat_gps,
       count(*) FILTER (WHERE fare = 0)               AS cuoc_0d
FROM trips;

-- H4. Chuyến đặt từ 00:00 tới 06:59 giờ VN = ngày HÔM TRƯỚC theo UTC (PG-016).
-- expect-value: 383
SELECT count(*) AS chuyen_lech_ngay_utc
FROM trips
WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date
   <> (requested_at AT TIME ZONE 'UTC')::date;

-- H5. Năm chuyến cài sát biên ngày 15/10 (PG-016) — mã nằm giữa các chuyến
--     sinh tự động vì mã tăng theo giờ đặt.
-- expect-line: 101489 | 2026-10-15 00:00:00+07   | 2026-10-14 17:00:00
-- expect-line: 101502 | 2026-10-15 06:59:59+07   | 2026-10-14 23:59:59
-- expect-line: 101503 | 2026-10-15 07:00:00+07   | 2026-10-15 00:00:00
-- expect-line: 101594 | 2026-10-15 23:59:59.5+07 | 2026-10-15 16:59:59.5
-- expect-line: 101595 | 2026-10-16 00:00:00+07   | 2026-10-15 17:00:00
SELECT code, requested_at, requested_at AT TIME ZONE 'UTC' AS gio_utc
FROM trips
WHERE requested_at IN ('2026-10-15 00:00:00+07', '2026-10-15 06:59:59+07', '2026-10-15 07:00:00+07',
                       '2026-10-15 23:59:59.5+07', '2026-10-16 00:00:00+07')
ORDER BY code;

-- H6. Chuyến TRÙNG giờ đặt (PG-015, PG-018): số nhóm, số chuyến trong nhóm,
--     nhóm lớn nhất. Hai nhóm lớn nhất là hai lô API của khách doanh nghiệp.
-- expect-line: 148 | 305 | 6
SELECT count(*) AS so_nhom, sum(n) AS so_chuyen, max(n) AS nhom_lon_nhat
FROM (SELECT requested_at, count(*) AS n FROM trips GROUP BY requested_at HAVING count(*) > 1) AS g;

-- expect-line: 2026-10-10 09:00:00+07 | 6 | 100971 | 100976
-- expect-line: 2026-10-18 14:30:00+07 | 4 | 101882 | 101885
-- expect-line: 2026-10-05 13:12:00+07 | 3 | 100489 | 100491
-- expect-line: 2026-10-15 13:12:00+07 | 3 | 101541 | 101543
-- expect-line: 2026-10-30 13:12:00+07 | 3 | 103116 | 103118
SELECT requested_at, count(*) AS so_chuyen, min(code) AS ma_dau, max(code) AS ma_cuoi
FROM trips
GROUP BY requested_at
HAVING count(*) > 2
ORDER BY count(*) DESC, requested_at;

-- H7. Đánh giá: có dòng / bỏ trống sao / bỏ qua tip / chọn tip 0 đ.
-- expect-line: 1891 | 153 | 541 | 660
SELECT count(*)                                 AS so_danh_gia,
       count(*) FILTER (WHERE stars IS NULL)    AS bo_trong_sao,
       count(*) FILTER (WHERE tip IS NULL)      AS bo_qua_tip,
       count(*) FILTER (WHERE tip = 0)          AS tip_0d
FROM trip_ratings;

-- H8. Giao dịch theo loại × phương thức × trạng thái.
-- expect-rows: 9
SELECT kind, method, status, count(*) AS so_gd, sum(amount) AS tong_tien
FROM payments
GROUP BY kind, method, status
ORDER BY kind COLLATE "C", method COLLATE "C", status COLLATE "C";

-- H9. Mỗi chuyến có bao nhiêu dòng payments (quan hệ một–nhiều, PG-013).
-- expect-line: 0 | 485
-- expect-line: 1 | 2552
-- expect-line: 2 | 137
SELECT so_gd, count(*) AS so_chuyen
FROM (SELECT t.id, count(p.id) AS so_gd
      FROM trips AS t LEFT JOIN payments AS p ON p.trip_id = t.id
      GROUP BY t.id) AS x
GROUP BY so_gd
ORDER BY so_gd;

-- ===========================================================================
-- Bất biến — mỗi truy vấn tìm dòng VI PHẠM, đúng là 0 dòng
-- ===========================================================================

-- K1. Trạng thái ↔ tài xế ↔ mốc thời gian (lặp lại logic CHECK của trips).
-- expect-rows: 0
SELECT code FROM trips
WHERE (status IN ('requested', 'searching') AND driver_id IS NOT NULL)
   OR (status IN ('accepted', 'arriving', 'picked_up', 'in_transit', 'completed') AND driver_id IS NULL)
   OR ((status = 'completed') <> (completed_at IS NOT NULL))
   OR accepted_at < requested_at OR completed_at < accepted_at;

-- K2. Tài xế / quán khác thành phố của chuyến (p01/07 B1, B2).
-- expect-rows: 0
SELECT t.code FROM trips AS t JOIN drivers AS d ON d.id = t.driver_id WHERE d.city_id <> t.city_id
UNION ALL
SELECT t.code FROM trips AS t JOIN merchants AS m ON m.id = t.merchant_id WHERE m.city_id <> t.city_id;

-- K3. Một tài xế > 1 chuyến đang chạy, hoặc `busy` lệch với chuyến (p01/07 B3, B4).
-- expect-rows: 0
SELECT d.code, d.status, count(t.id) AS dang_chay
FROM drivers AS d
LEFT JOIN trips AS t
  ON t.driver_id = d.id AND t.status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
GROUP BY d.code, d.status
HAVING count(t.id) > 1
    OR (count(t.id) = 1) <> (d.status = 'busy');

-- K4. updated_at không sớm hơn mốc mới nhất của dòng (p01/07 B5).
-- expect-rows: 0
SELECT code FROM trips WHERE updated_at < greatest(requested_at, accepted_at, completed_at);

-- K5. Lịch sử khớp chuyến: dòng đầu `requested` đúng giờ đặt, dòng cuối = trạng
--     thái hiện tại, dòng cuối đúng bằng updated_at (p01/07 B7, chặt hơn).
-- expect-rows: 0
SELECT t.code
FROM trips AS t
JOIN LATERAL (SELECT min(e.occurred_at) AS dau, max(e.occurred_at) AS cuoi FROM trip_events AS e WHERE e.trip_id = t.id) AS m ON true
JOIN LATERAL (SELECT e.status FROM trip_events AS e WHERE e.trip_id = t.id ORDER BY e.occurred_at DESC, e.id DESC LIMIT 1) AS l ON true
WHERE m.dau IS DISTINCT FROM t.requested_at
   OR m.cuoi IS DISTINCT FROM t.updated_at
   OR l.status <> t.status;

-- K6. Đánh giá chỉ có ở chuyến hoàn thành, sau mốc hoàn thành.
-- expect-rows: 0
SELECT t.code FROM trip_ratings AS r JOIN trips AS t ON t.id = r.trip_id
WHERE t.status <> 'completed' OR r.rated_at < t.completed_at;

-- K7. Chuyến đang chạy chưa có giao dịch; mọi giao dịch không sớm hơn giờ đặt.
-- expect-rows: 0
SELECT p.id FROM payments AS p JOIN trips AS t ON t.id = p.trip_id
WHERE t.status NOT IN ('completed', 'cancelled') OR p.paid_at < t.requested_at;

-- K8. Khu đón phải là nút cấp `zone` và thuộc cây của đúng thành phố chuyến
--     (cây 3 tầng: khu → vùng → thành phố).
-- expect-rows: 0
SELECT t.code
FROM trips AS t
JOIN areas AS z ON z.id = t.pickup_area_id
JOIN areas AS v ON v.id = z.parent_id
JOIN areas AS g ON g.id = v.parent_id
WHERE z.level <> 'zone' OR g.city_id <> t.city_id;

-- K9. Cây khu vực đúng tầng: cha của vùng là thành phố, cha của khu là vùng.
-- expect-rows: 0
SELECT a.code, a.level, p.level AS cap_cha
FROM areas AS a JOIN areas AS p ON p.id = a.parent_id
WHERE (a.level, p.level) NOT IN (('district', 'city'), ('zone', 'district'));
