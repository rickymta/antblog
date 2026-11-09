-- P02 · 11-null-case-coalesce.sql — PG-011 `postgresql-null-case-coalesce`
-- NULL, CASE và COALESCE: phân biệt THIẾU DỮ LIỆU, KHÔNG ÁP DỤNG và GIÁ TRỊ 0.
--
-- Trong dữ liệu Tới Nơi, một ô trống có thể mang ba nghĩa khác nhau:
--   * chưa có / thiếu   — chuyến đang tìm tài xế; khách bỏ trống phần sao;
--                          app tài xế mất GPS nên không biết khu đón;
--   * không áp dụng     — đơn đặt xe thì không có quán; chuyến huỷ không được
--                          đánh giá;
--   * và số 0 THẬT      — chuyến khuyến mãi cước 0 đ; khách chọn "không tip".
-- SQL chỉ có một NULL cho hai nghĩa đầu; nghĩa nào phải suy từ các cột khác.

SET search_path TO toinoi;
SET TIME ZONE 'Asia/Ho_Chi_Minh';

-- ---------------------------------------------------------------------------
-- 1. So sánh với NULL không ra TRUE hay FALSE mà ra NULL ("không biết").
--    WHERE chỉ giữ dòng có điều kiện TRUE ⇒ `= NULL` không bao giờ khớp.
-- expect-line: NULL | NULL | t | f | t
SELECT NULL = NULL                  AS bang_nhau,
       NULL <> 1                    AS khac_1,
       NULL IS NULL                 AS la_null,
       NULL IS DISTINCT FROM NULL   AS khac_biet_null,
       1 IS DISTINCT FROM NULL      AS khac_biet_1;

-- expect-value: 0
SELECT count(*) FROM trips WHERE driver_id = NULL;

-- expect-value: 297
SELECT count(*) FROM trips WHERE driver_id IS NULL;

-- ---------------------------------------------------------------------------
-- 2. Một NULL, nhiều nghĩa: đọc driver_id cùng với status bằng CASE.
--    CASE dừng ở nhánh ĐẦU TIÊN đúng; nhánh ELSE bắt dữ liệu sai quy tắc.
-- expect-line: có tài xế                                   | 2877
-- expect-line: không áp dụng — huỷ trước khi có người nhận | 295
-- expect-line: chưa có — đang tìm tài xế                   | 2
SELECT CASE
         WHEN driver_id IS NOT NULL               THEN 'có tài xế'
         WHEN status IN ('requested', 'searching') THEN 'chưa có — đang tìm tài xế'
         WHEN status = 'cancelled'                THEN 'không áp dụng — huỷ trước khi có người nhận'
         ELSE 'SAI QUY TẮC — lẽ ra phải có tài xế'
       END AS y_nghia_cot_driver_id,
       count(*) AS so_chuyen
FROM trips
GROUP BY 1
ORDER BY 2 DESC;

-- ---------------------------------------------------------------------------
-- 3. Đánh giá có BỐN trường hợp, hai trong số đó cùng ra NULL sau LEFT JOIN:
--    "không có dòng đánh giá" và "có dòng nhưng stars NULL". Phân biệt bằng
--    cột KHÔNG BAO GIỜ NULL của bảng bên phải (khoá chính r.trip_id).
-- expect-line: có sao                              | 1738
-- expect-line: thiếu — chưa đánh giá               | 816
-- expect-line: không áp dụng — chuyến huỷ          | 461
-- expect-line: thiếu — mở màn đánh giá, bỏ trống sao | 153
SELECT CASE
         WHEN t.status = 'cancelled' THEN 'không áp dụng — chuyến huỷ'
         WHEN r.trip_id IS NULL      THEN 'thiếu — chưa đánh giá'
         WHEN r.stars IS NULL        THEN 'thiếu — mở màn đánh giá, bỏ trống sao'
         ELSE 'có sao'
       END AS tinh_trang_danh_gia,
       count(*) AS so_chuyen
FROM trips AS t
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status IN ('completed', 'cancelled')
GROUP BY 1
ORDER BY 2 DESC;

-- ---------------------------------------------------------------------------
-- 4. Hàm tổng hợp BỎ QUA NULL. avg(stars) là trung bình trên 1.738 lượt có
--    sao; COALESCE(stars, 0) biến "không biết" thành "0 sao" và kéo điểm xuống.
-- expect-line: 1891 | 1738 | 4.22 | 3.88
SELECT count(*)                            AS so_danh_gia,
       count(stars)                        AS co_sao,
       round(avg(stars), 2)                AS diem_tb_dung,
       round(avg(coalesce(stars, 0)), 2)   AS diem_tb_sai
FROM trip_ratings;

-- Ba điều kiện "= 5", "<> 5" và "IS NULL" mới phủ hết: dòng NULL không thuộc
-- `stars = 5` cũng không thuộc `stars <> 5`.
-- expect-line: 1891 | 896 | 842 | 153
SELECT count(*)                                  AS tong,
       count(*) FILTER (WHERE stars = 5)         AS nam_sao,
       count(*) FILTER (WHERE stars <> 5)        AS khac_nam_sao,
       count(*) FILTER (WHERE stars IS NULL)     AS khong_ro
FROM trip_ratings;

-- ---------------------------------------------------------------------------
-- 5. Tip: NULL (bỏ qua bước tip) KHÁC 0 (chủ động chọn không tip). Câu hỏi
--    nghiệp vụ quyết định mẫu số:
--      * "khách CÓ TRẢ LỜI tip trung bình bao nhiêu?"   → avg(tip)
--      * "mỗi lượt đánh giá mang về bao nhiêu tip?"      → sum(tip) / count(*)
--    COALESCE(tip, 0) chỉ đúng với câu thứ hai — và phải nói rõ giả định
--    "bỏ qua = không tip".
-- expect-line: 1891 | 1350 | 660 | 5565000 | 4122 | 2943
SELECT count(*)                          AS so_danh_gia,
       count(tip)                        AS co_tra_loi_tip,
       count(*) FILTER (WHERE tip = 0)   AS chon_0d,
       sum(tip)                          AS tong_tip,
       round(avg(tip))                   AS tb_tren_nguoi_tra_loi,
       round(avg(coalesce(tip, 0)))      AS tb_tren_moi_luot
FROM trip_ratings;

-- NULL "lây" qua phép tính: cước + tip ra NULL khi tip NULL, kể cả khi cước
-- 0 đ là một con số thật. Mỗi loại một ví dụ (chuyến có mã nhỏ nhất).
-- expect-line: 1 bỏ qua tip           | 100003 | 50000 | NULL  | NULL  | 50000
-- expect-line: 1 bỏ qua tip, cước 0 đ | 100879 |     0 | NULL  | NULL  |     0
-- expect-line: 2 tip 0 đ              | 100000 | 45000 |     0 | 45000 | 45000
-- expect-line: 2 tip 0 đ, cước 0 đ    | 100158 |     0 |     0 |     0 |     0
-- expect-line: 3 có tip               | 100004 | 59000 |  5000 | 64000 | 64000
-- expect-line: 3 có tip, cước 0 đ     | 100007 |     0 | 10000 | 10000 | 10000
SELECT DISTINCT ON (loai)
       loai, t.code, t.fare, r.tip,
       t.fare + r.tip              AS cuoc_cong_tip,
       t.fare + coalesce(r.tip, 0) AS cuoc_cong_tip_coalesce
FROM trips AS t
JOIN trip_ratings AS r ON r.trip_id = t.id
CROSS JOIN LATERAL (SELECT CASE WHEN r.tip IS NULL THEN '1 bỏ qua tip'
                                WHEN r.tip = 0     THEN '2 tip 0 đ'
                                ELSE '3 có tip' END ||
                           CASE WHEN t.fare = 0 THEN ', cước 0 đ' ELSE '' END AS loai) AS l
ORDER BY loai, t.code;

-- ---------------------------------------------------------------------------
-- 6. Cước 0 đ là giá trị THẬT (khuyến mãi), không phải thiếu dữ liệu. Muốn
--    "cước trung bình của chuyến có thu tiền" thì loại 0 bằng NULLIF(fare, 0)
--    — NULLIF(a, b) trả NULL khi a = b — và avg tự bỏ qua NULL.
-- expect-line: 2707 | 31 | 67339 | 68119
SELECT count(*)                         AS chuyen_hoan_thanh,
       count(*) FILTER (WHERE fare = 0) AS cuoc_0d,
       round(avg(fare))                 AS tb_moi_chuyen,
       round(avg(nullif(fare, 0)))      AS tb_chuyen_co_thu
FROM trips
WHERE status = 'completed';

-- ---------------------------------------------------------------------------
-- 7. COALESCE để HIỂN THỊ: lấy giá trị đầu tiên khác NULL. Mỗi chỗ trống được
--    gắn đúng nghĩa của nó thay vì để ô rỗng.
-- expect-line: 100001 | food   | cancelled | Mì Quảng Lab 207  | HN-TA-01           | — (huỷ trước khi nhận)
-- expect-line: 100002 | food   | completed | Bún Chả Lab 106   | SG-DO-03           | 8117
-- expect-line: 100007 | ride   | completed | — (không áp dụng) | HN-BA-02           | 8217
-- expect-line: 100072 | parcel | completed | — (không áp dụng) | không rõ (mất GPS) | 8110
-- expect-line: 103172 | food   | searching | Gà Rán Lab 205    | HN-TT-03           | đang tìm…
SELECT t.code, t.kind, t.status,
       coalesce(m.name,
                CASE WHEN t.kind = 'food' THEN 'LỖI: đơn đồ ăn thiếu quán'
                     ELSE '— (không áp dụng)' END)                  AS quan,
       coalesce(z.code, 'không rõ (mất GPS)')                      AS khu_don,
       coalesce(d.code::text,
                CASE WHEN t.status IN ('requested', 'searching') THEN 'đang tìm…'
                     ELSE '— (huỷ trước khi nhận)' END)            AS tai_xe
FROM trips AS t
LEFT JOIN merchants AS m ON m.id = t.merchant_id
LEFT JOIN areas     AS z ON z.id = t.pickup_area_id
LEFT JOIN drivers   AS d ON d.id = t.driver_id
WHERE t.code IN (SELECT min(code) FROM trips WHERE pickup_area_id IS NULL
                 UNION ALL SELECT min(code) FROM trips WHERE kind = 'food' AND status = 'completed'
                 UNION ALL SELECT min(code) FROM trips WHERE status = 'cancelled' AND driver_id IS NULL
                 UNION ALL SELECT max(code) FROM trips WHERE status = 'searching'
                 UNION ALL SELECT min(code) FROM trips WHERE fare = 0)
ORDER BY t.code;

-- ---------------------------------------------------------------------------
-- 8. `<>` cũng bỏ rơi NULL. "Chuyến KHÔNG đón ở khu SG-TT-01 (id 111)":
--    `<> 111` không đếm 36 chuyến mất GPS; `IS DISTINCT FROM 111` đếm cả chúng.
--    Câu nào đúng tuỳ nghiệp vụ — nhưng phải là lựa chọn có chủ đích.
-- expect-line: 3174 | 162 | 2976 | 3012
SELECT count(*)                                              AS tong,
       count(*) FILTER (WHERE pickup_area_id = 111)          AS o_111,
       count(*) FILTER (WHERE pickup_area_id <> 111)         AS khac_111,
       count(*) FILTER (WHERE pickup_area_id IS DISTINCT FROM 111) AS khac_biet_111
FROM trips;

-- ---------------------------------------------------------------------------
-- 9. NULLIF chống chia cho 0. Ngày 20/10 Đà Nẵng có bão, 0 chuyến: tỉ lệ huỷ
--    = huỷ / tổng chia cho 0 ⇒ LỖI cả câu, mất luôn kết quả của SG và HN.
-- expect-error: 22012
SELECT c.code,
       count(t.id) AS tong,
       100 * count(t.id) FILTER (WHERE t.status = 'cancelled') / count(t.id) AS ty_le_huy_pct
FROM cities AS c
LEFT JOIN trips AS t
  ON t.city_id = c.id
 AND t.requested_at >= '2026-10-20 00:00+07' AND t.requested_at < '2026-10-21 00:00+07'
GROUP BY c.code
ORDER BY c.code COLLATE "C";

-- NULLIF(mẫu, 0) ⇒ tỉ lệ của DN là NULL = "không xác định". KHÔNG COALESCE
-- thành 0 %: "0 % huỷ" là một khẳng định sai về một ngày không có chuyến nào.
-- expect-line: DN |  0 | NULL | không có chuyến
-- expect-line: HN | 37 | 13.5 | 13.5
-- expect-line: SG | 49 | 14.3 | 14.3
SELECT c.code,
       count(t.id) AS tong,
       round(100.0 * count(t.id) FILTER (WHERE t.status = 'cancelled') / nullif(count(t.id), 0), 1) AS ty_le_huy_pct,
       coalesce(round(100.0 * count(t.id) FILTER (WHERE t.status = 'cancelled') / nullif(count(t.id), 0), 1)::text,
                'không có chuyến') AS hien_thi
FROM cities AS c
LEFT JOIN trips AS t
  ON t.city_id = c.id
 AND t.requested_at >= '2026-10-20 00:00+07' AND t.requested_at < '2026-10-21 00:00+07'
GROUP BY c.code
ORDER BY c.code COLLATE "C";
