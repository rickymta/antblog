-- P02 · 12-aggregate-group-by.sql — PG-012 `postgresql-aggregate-group-by-having`
-- COUNT, SUM, GROUP BY và HAVING: tổng hợp theo thành phố và dịch vụ với MẪU
-- SỐ RÕ RÀNG.
--
-- Quy ước báo cáo tháng 10/2026 dùng trong file này (và các file sau):
--   * kỳ báo cáo = chuyến ĐẶT trong [2026-10-01 00:00, 2026-10-31 00:00) giờ VN
--     ⇒ 6 chuyến đang chạy (đặt sau nửa đêm 31/10) nằm ngoài kỳ;
--   * "chuyến đã kết thúc" = completed + cancelled — mẫu số của tỉ lệ huỷ;
--   * "doanh thu" = Σ cước chuyến completed (chưa trừ khuyến mãi, chưa đối
--     chiếu tiền thật — tiền thật là việc của PG-013/PG-020).

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. count(*) đếm DÒNG; count(cột) đếm giá trị KHÁC NULL; count(DISTINCT cột)
--    đếm giá trị khác nhau. Ba con số trả lời ba câu hỏi khác nhau.
-- expect-line: 3174 | 2877 | 1432 | 52 | 576
SELECT count(*)                    AS so_chuyen,
       count(driver_id)            AS co_tai_xe,
       count(merchant_id)          AS co_quan,
       count(DISTINCT driver_id)   AS so_tai_xe_khac_nhau,
       count(DISTINCT customer_id) AS so_khach_khac_nhau
FROM trips;

-- sum(integer) trả bigint (không tràn khi cộng nhiều chuyến), avg trả numeric.
-- expect-line: bigint | numeric | bigint
SELECT pg_typeof(sum(fare)) AS kieu_sum, pg_typeof(avg(fare)) AS kieu_avg, pg_typeof(count(*)) AS kieu_count
FROM trips;

-- ---------------------------------------------------------------------------
-- 2. Bảng chính của tháng: thành phố × dịch vụ. Mỗi cột ghi rõ mẫu số.
--    FILTER (WHERE …) cho phép nhiều phép đếm có điều kiện trong MỘT lần quét.
-- expect-line: DN | food   | 210 | 173 |  37 | 17.6 | 14902000 | 86139
-- expect-line: DN | parcel |  72 |  68 |   4 |  5.6 |  3380000 | 49706
-- expect-line: DN | ride   | 177 | 148 |  29 | 16.4 |  8134000 | 54959
-- expect-line: HN | food   | 503 | 413 |  90 | 17.9 | 35025000 | 84806
-- expect-line: HN | parcel | 165 | 145 |  20 | 12.1 |  7041000 | 48559
-- expect-line: HN | ride   | 438 | 383 |  55 | 12.6 | 20925000 | 54634
-- expect-line: SG | food   | 717 | 615 | 102 | 14.2 | 52898000 | 86013
-- expect-line: SG | parcel | 251 | 196 |  55 | 21.9 |  9290000 | 47398
-- expect-line: SG | ride   | 635 | 566 |  69 | 10.9 | 30692000 | 54226
SELECT c.code                                                    AS tp,
       t.kind,
       count(*)                                                  AS da_ket_thuc,
       count(*) FILTER (WHERE t.status = 'completed')            AS hoan_thanh,
       count(*) FILTER (WHERE t.status = 'cancelled')            AS huy,
       round(100.0 * count(*) FILTER (WHERE t.status = 'cancelled') / count(*), 1) AS ty_le_huy_pct,  -- huỷ / đã kết thúc
       sum(t.fare) FILTER (WHERE t.status = 'completed')         AS doanh_thu,
       round(avg(t.fare) FILTER (WHERE t.status = 'completed'))  AS cuoc_tb                            -- / hoàn thành
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY c.code, t.kind
ORDER BY c.code COLLATE "C", t.kind COLLATE "C";

-- ---------------------------------------------------------------------------
-- 3. Ba lỗi mẫu số hay gặp, cùng một dữ liệu:
--    a) chia số nguyên: count/count là phép chia NGUYÊN ⇒ 0 hoặc làm tròn xuống;
--    b) cước trung bình tính trên MỌI chuyến (gồm chuyến huỷ — chưa ai trả tiền);
--    c) tỉ lệ huỷ chia cho số chuyến HOÀN THÀNH thay vì số chuyến đã kết thúc.
-- expect-line: DN | 0 | 15 | 15.25 | 68011 | 67907 | 17.99
-- expect-line: HN | 0 | 14 | 14.92 | 68348 | 66940 | 17.53
-- expect-line: SG | 0 | 14 | 14.10 | 68269 | 67451 | 16.41
SELECT c.code                                                                   AS tp,
       count(*) FILTER (WHERE t.status = 'cancelled') / count(*)                 AS a_chia_nguyen,
       100 * count(*) FILTER (WHERE t.status = 'cancelled') / count(*)           AS a_pct_nguyen,
       round(100.0 * count(*) FILTER (WHERE t.status = 'cancelled') / count(*), 2) AS a_pct_dung,
       round(avg(t.fare))                                                       AS b_cuoc_tb_sai,
       round(avg(t.fare) FILTER (WHERE t.status = 'completed'))                 AS b_cuoc_tb_dung,
       round(100.0 * count(*) FILTER (WHERE t.status = 'cancelled')
             / count(*) FILTER (WHERE t.status = 'completed'), 2)               AS c_pct_sai_mau
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY c.code
ORDER BY c.code COLLATE "C";

-- ---------------------------------------------------------------------------
-- 4. Mọi cột trong SELECT phải hoặc nằm trong GROUP BY, hoặc nằm trong hàm
--    tổng hợp. PostgreSQL từ chối thay vì "chọn đại một giá trị".
-- expect-error: 42803
SELECT c.code, t.kind, count(*)
FROM trips AS t JOIN cities AS c ON c.id = t.city_id
GROUP BY c.code;

-- Ngoại lệ hợp lệ: nhóm theo KHOÁ CHÍNH thì được chọn cột khác của chính bảng
-- đó (phụ thuộc hàm) — cities.name đi theo cities.id.
-- expect-line: 1 | SG | TP. Hồ Chí Minh | 1606
-- expect-line: 2 | HN | Hà Nội          | 1108
-- expect-line: 3 | DN | Đà Nẵng         |  460
SELECT ci.id, ci.code, ci.name, count(*) AS so_chuyen
FROM trips AS t JOIN cities AS ci ON ci.id = t.city_id
GROUP BY ci.id
ORDER BY ci.id;

-- ---------------------------------------------------------------------------
-- 5. WHERE lọc DÒNG trước khi gom nhóm; HAVING lọc NHÓM sau khi gom.
--    Nhóm thành phố × dịch vụ có tỉ lệ huỷ trên 15 %:
-- expect-line: SG | parcel | 251 | 21.9
-- expect-line: HN | food   | 503 | 17.9
-- expect-line: DN | food   | 210 | 17.6
-- expect-line: DN | ride   | 177 | 16.4
SELECT c.code AS tp, t.kind, count(*) AS da_ket_thuc,
       round(100.0 * count(*) FILTER (WHERE t.status = 'cancelled') / count(*), 1) AS ty_le_huy_pct
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY c.code, t.kind
HAVING count(*) FILTER (WHERE t.status = 'cancelled') * 100.0 / count(*) > 15
ORDER BY ty_le_huy_pct DESC, c.code COLLATE "C", t.kind COLLATE "C";

-- Tài xế hoàn thành từ 63 chuyến trở lên trong tháng. Điều kiện trên DÒNG
-- (status = completed) ở WHERE; điều kiện trên NHÓM (count >= 63) ở HAVING.
-- expect-line: 8101 | SG | 68 | 4736000
-- expect-line: 8109 | SG | 68 | 4744000
-- expect-line: 8120 | SG | 66 | 4158000
-- expect-line: 8108 | SG | 64 | 4085000
-- expect-line: 8124 | SG | 64 | 4346000
-- expect-line: 8107 | SG | 63 | 4539000
-- expect-line: 8210 | HN | 63 | 4160000
SELECT d.code AS tai_xe, c.code AS tp, count(*) AS hoan_thanh, sum(t.fare) AS doanh_thu
FROM trips AS t
JOIN drivers AS d ON d.id = t.driver_id
JOIN cities AS c ON c.id = d.city_id
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY d.code, c.code
HAVING count(*) >= 63
ORDER BY hoan_thanh DESC, tai_xe;

-- ---------------------------------------------------------------------------
-- 6. Tổng phụ và tổng cộng trong MỘT câu: ROLLUP (thành phố, dịch vụ) sinh
--    các nhóm (tp, kind), (tp) và (). Cột bị "cuộn" hiện NULL — dùng
--    GROUPING() để phân biệt NULL tổng phụ với NULL dữ liệu.
-- expect-md5: f37ee173d11468c274a79466abe1c1e7
SELECT CASE WHEN grouping(c.code) = 1 THEN 'TỔNG' ELSE c.code END              AS tp,
       CASE WHEN grouping(t.kind) = 1 THEN '(mọi dịch vụ)' ELSE t.kind END     AS kind,
       count(*) FILTER (WHERE t.status = 'completed')                          AS hoan_thanh,
       sum(t.fare) FILTER (WHERE t.status = 'completed')                       AS doanh_thu
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY ROLLUP (c.code, t.kind)
ORDER BY grouping(c.code), c.code COLLATE "C", grouping(t.kind), t.kind COLLATE "C";

-- Tổng cộng của ROLLUP phải bằng một truy vấn tổng độc lập (đối chiếu chéo).
-- expect-line: 2707 | 182287000
SELECT count(*) AS hoan_thanh, sum(fare) AS doanh_thu
FROM trips
WHERE status = 'completed'
  AND requested_at >= '2026-10-01 00:00+07' AND requested_at < '2026-10-31 00:00+07';

-- ---------------------------------------------------------------------------
-- 7. Đánh giá theo thành phố: HAI mẫu số khác nhau trong cùng một dòng.
--    * độ phủ  = số chuyến có dòng đánh giá / số chuyến hoàn thành
--    * điểm TB = trung bình trên các lượt CÓ sao (avg bỏ qua NULL)
-- expect-line: DN |  389 | 266 | 68.4 | 251 | 4.30
-- expect-line: HN |  941 | 670 | 71.2 | 619 | 4.17
-- expect-line: SG | 1377 | 955 | 69.4 | 868 | 4.23
SELECT c.code                                          AS tp,
       count(*)                                        AS hoan_thanh,
       count(r.trip_id)                                AS co_danh_gia,
       round(100.0 * count(r.trip_id) / count(*), 1)   AS do_phu_pct,
       count(r.stars)                                  AS co_sao,
       round(avg(r.stars), 2)                          AS diem_tb
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY c.code
ORDER BY c.code COLLATE "C";

-- ---------------------------------------------------------------------------
-- 8. Khách hoạt động: đếm khách KHÁC NHAU, rồi số chuyến trung bình mỗi khách.
--    count(DISTINCT …) không cộng được qua các nhóm: khách đi cả ride lẫn food
--    bị đếm ở cả hai dòng — tổng các dòng LỚN HƠN số khách thật (dòng cuối).
-- expect-line: food   | 441 | 1201 | 2.72
-- expect-line: parcel | 241 |  409 | 1.70
-- expect-line: ride   | 421 | 1097 | 2.61
-- expect-line: TỔNG   | 569 | 2707 | 4.76
SELECT coalesce(t.kind, 'TỔNG') AS kind,
       count(DISTINCT t.customer_id)                            AS so_khach,
       count(*)                                                 AS hoan_thanh,
       round(count(*)::numeric / count(DISTINCT t.customer_id), 2) AS chuyen_moi_khach
FROM trips AS t
WHERE t.status = 'completed'
  AND t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY ROLLUP (t.kind)
ORDER BY grouping(t.kind), t.kind COLLATE "C";
