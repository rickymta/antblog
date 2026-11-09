---
slug: postgresql-aggregate-group-by-having
title: "COUNT, SUM, GROUP BY và HAVING"
summary: "Báo cáo tháng 10 của Tới Nơi ghi tỉ lệ huỷ TP. HCM «14 %», cước trung bình 68.269 đ và 1.103 khách hoạt động — cả ba đều lệch, dù câu SQL chạy không lỗi. Học hàm tổng hợp, GROUP BY, FILTER, HAVING và ROLLUP để mỗi con số đi kèm một mẫu số rõ ràng."
category: postgresql
level: beginner
tags: [postgresql, pg-012, aggregate, count, group-by, having, rollup, toi-noi]
language: vi
---

Sáng 31/10, đội Vận hành Tới Nơi chốt báo cáo tháng 10 cho buổi giao ban, và ba con số bị hỏi lại ngay. Tỉ lệ huỷ TP. HCM ghi **14 %**, bảng tính của đội tài chính ra 14,10 %. Cước trung bình TP. HCM ghi **68.269 đ**, cao hơn mọi con số đội giá từng thấy. Đội Marketing cộng «khách hoạt động» của ba dịch vụ được **1.103**, nhưng cả công ty chỉ có 569 khách đi chuyến hoàn thành trong tháng.

Cả ba đến từ những câu `GROUP BY` chạy không lỗi. Lỗi nằm ở **mẫu số**: chia số nguyên, lấy trung bình trên cả chuyến huỷ, cộng những con số không cộng được. Quy tắc xuyên suốt bài: mỗi cột của báo cáo phải nói được nó chia cho cái gì.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-012 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-select-filter-order-limit">PG-007 · SELECT, WHERE, ORDER BY</a>, <a href="/articles/postgresql-null-case-coalesce">PG-011 · NULL, CASE và COALESCE</a>. Lab: <code>examples/toi-noi-postgresql/p02</code> (tệp <code>12-aggregate-group-by.sql</code>), thí nghiệm thêm ở <code>p02/thu-nghiem/pg012-*.sql</code>. Sau bài này, bạn lập được bảng tổng hợp theo thành phố và dịch vụ, mỗi cột có mẫu số rõ ràng và tổng khớp một truy vấn độc lập.</p>
</div>

## 1. Dữ liệu và định nghĩa chỉ số

Bài dùng fixture Phần 2 như [PG-011](/articles/postgresql-null-case-coalesce): 3.174 chuyến, ảnh chụp 01:00 ngày 31/10/2026 giờ VN. Kết quả chép từ lần chạy thật trên PGlite (PostgreSQL 17.5), bỏ dòng `-- [lab]` và `(n rows)`.

```bash
cd examples/toi-noi-postgresql
node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/12-aggregate-group-by.sql
```

Trước khi viết SQL, viết **định nghĩa chỉ số** — data dictionary cho số liệu, như data dictionary cho cột ở PG-004:

| Chỉ số | Định nghĩa | Mẫu số |
|---|---|---|
| Kỳ báo cáo | chuyến **đặt** trong `[01/10 00:00, 31/10 00:00)` giờ VN | — |
| Đã kết thúc | `completed` + `cancelled` trong kỳ | — |
| Tỉ lệ huỷ | huỷ ÷ đã kết thúc | đã kết thúc |
| Doanh thu | Σ `fare` chuyến `completed` (chưa đối chiếu tiền thật — PG-020) | — |
| Cước trung bình | doanh thu ÷ chuyến hoàn thành | hoàn thành |
| Điểm sao trung bình | trung bình `stars` khác NULL | lượt có sao |

Sáu chuyến đặt sau nửa đêm 31/10 còn đang chạy nên nằm ngoài kỳ. Kỳ có **3.168 dòng**, cả 3.168 đã kết thúc (thí nghiệm 4 của `pg012-thu-nghiem.sql`) — con số mọi bảng trong bài phải cộng lại được.

## 2. Hàm tổng hợp

**Hàm tổng hợp** (aggregate function) nhận một tập dòng, trả một giá trị: `count`, `sum`, `avg`, `min`, `max`. Riêng `count` có ba dạng cho ba câu hỏi:

```sql
SELECT count(*)                    AS so_chuyen,
       count(driver_id)            AS co_tai_xe,
       count(merchant_id)          AS co_quan,
       count(DISTINCT driver_id)   AS so_tai_xe_khac_nhau,
       count(DISTINCT customer_id) AS so_khach_khac_nhau
FROM trips;
```

```text
 so_chuyen | co_tai_xe | co_quan | so_tai_xe_khac_nhau | so_khach_khac_nhau
-----------+-----------+---------+---------------------+--------------------
      3174 |      2877 |    1432 |                  52 |                576
```

`count(*)` đếm **dòng**; `count(cột)` đếm giá trị **khác NULL**; `count(DISTINCT cột)` đếm giá trị **khác nhau**, bỏ NULL. Lab có 54 tài xế nhưng chỉ 52 từng chạy — 8219 và 8220 chưa có chuyến nào.

Kiểu kết quả: `pg_typeof` cho `sum(fare)` là **`bigint`**, `avg(fare)` là **`numeric`**, `count(*)` là **`bigint`**. `fare` là `integer` (tối đa khoảng 2,1 tỉ), chỉ khoảng 32.000 chuyến 67.000 đ đã vượt, nên `sum` nâng kiểu để không tràn. Nhớ kiểu `bigint` của `count` — nó gây ra lỗi «14 %» ở mục 6. Hai điều từ PG-011 vẫn đúng: hàm tổng hợp bỏ NULL, và trên tập rỗng `count` ra 0 còn `sum`/`avg` ra NULL.

## 3. GROUP BY: từ dòng thành nhóm

**`GROUP BY`** chia dòng thành **nhóm** theo giá trị của một hay nhiều biểu thức, rồi tính hàm tổng hợp cho từng nhóm — một dòng kết quả mỗi nhóm.

```sql
SELECT ci.id, ci.code, ci.name, count(*) AS so_chuyen
FROM trips AS t JOIN cities AS ci ON ci.id = t.city_id
GROUP BY ci.id
ORDER BY ci.id;
```

```text
 id | code |      name       | so_chuyen
----+------+-----------------+-----------
  1 | SG   | TP. Hồ Chí Minh |      1606
  2 | HN   | Hà Nội          |      1108
  3 | DN   | Đà Nẵng         |       460
```

1.606 + 1.108 + 460 = 3.174: mỗi chuyến rơi vào đúng một nhóm. (`JOIN … ON` ghép chuyến với thành phố — PG-013.)

**Quy tắc:** mỗi cột trong `SELECT` phải có một giá trị duy nhất mỗi nhóm. Nhóm «SG» chứa ba loại dịch vụ, nên hỏi «`kind` của nhóm SG?» không có đáp án — PostgreSQL từ chối `SELECT c.code, t.kind, count(*) … GROUP BY c.code` thay vì chọn đại:

```text
ERROR:  column "t.kind" must appear in the GROUP BY clause or be used in an aggregate function
```

Mã `42803`. Sửa bằng cách thêm `t.kind` vào `GROUP BY`, hoặc bọc trong hàm tổng hợp. Câu `cities` phía trên là ngoại lệ hợp lệ: nhóm theo **khoá chính** `ci.id` thì chọn được `ci.code`, `ci.name` — mỗi `id` chỉ có một `name`, tài liệu gọi là **phụ thuộc hàm** (functional dependency). `GROUP BY` cũng nhận biểu thức hoặc số thứ tự cột như `GROUP BY 1`; nhóm theo cột của select list là phần PostgreSQL mở rộng so với SQL chuẩn.

## 4. Thứ tự logic: WHERE trước nhóm, HAVING sau nhóm

Thứ tự đọc nghĩa của PG-007 thêm hai bước: `FROM → WHERE → GROUP BY → HAVING → SELECT → ORDER BY → LIMIT`. Bảng dưới đếm số dòng từng bước cho câu «nhóm có tỉ lệ huỷ trên 15 %» ở mục 7:

| Bước | Làm việc trên | Còn lại |
|---|---|---|
| `FROM trips JOIN cities` | dòng | 3.174 dòng |
| `WHERE` đặt trong kỳ tháng 10 | từng **dòng** — chưa có nhóm | 3.168 dòng |
| `GROUP BY` thành phố, dịch vụ | dòng → nhóm | 9 nhóm |
| `HAVING` tỉ lệ huỷ > 15 % | từng **nhóm** | 4 nhóm |
| `SELECT` tính cột, đặt tên `AS` | nhóm | 4 dòng |
| `ORDER BY` (dùng được tên `AS`) | dòng kết quả | 4 dòng |

Đây là thứ tự logic để hiểu câu lệnh; PostgreSQL có thể thực thi khác miễn ra cùng kết quả (Phần 3 xem bằng `EXPLAIN`).

Thứ tự đó giải thích ba lỗi hay gặp — cả ba chạy thật ở thí nghiệm 1–3 của `pg012-thu-nghiem.sql`:

| Câu viết sai | Lỗi | Vì sao |
|---|---|---|
| `WHERE count(*) > 1000` | `42803 aggregate functions are not allowed in WHERE` | `WHERE` chạy trên dòng, chưa có nhóm |
| `count(*) AS so_chuyen … HAVING so_chuyen > 1000` | `42703 column "so_chuyen" does not exist` | tên `AS` sinh ở bước `SELECT`, sau `HAVING` |
| `GROUP BY c.code HAVING t.status = 'completed'` | `42803 column "t.status" must appear in the GROUP BY clause…` | điều kiện trên dòng thuộc về `WHERE` |

## 5. FILTER: nhiều phép đếm có điều kiện trong một lần quét

Bảng chính của báo cáo cần nhiều phép đếm trên các tập dòng khác nhau của **cùng một nhóm**. **`FILTER (WHERE …)`** gắn sau hàm tổng hợp để hàm chỉ nhận dòng thoả điều kiện:

```sql
SELECT c.code AS tp, t.kind,
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
```

```text
 tp |  kind  | da_ket_thuc | hoan_thanh | huy | ty_le_huy_pct | doanh_thu | cuoc_tb
----+--------+-------------+------------+-----+---------------+-----------+---------
 DN | food   |         210 |        173 |  37 |          17.6 |  14902000 |   86139
 DN | parcel |          72 |         68 |   4 |           5.6 |   3380000 |   49706
 DN | ride   |         177 |        148 |  29 |          16.4 |   8134000 |   54959
 HN | food   |         503 |        413 |  90 |          17.9 |  35025000 |   84806
 HN | parcel |         165 |        145 |  20 |          12.1 |   7041000 |   48559
 HN | ride   |         438 |        383 |  55 |          12.6 |  20925000 |   54634
 SG | food   |         717 |        615 | 102 |          14.2 |  52898000 |   86013
 SG | parcel |         251 |        196 |  55 |          21.9 |   9290000 |   47398
 SG | ride   |         635 |        566 |  69 |          10.9 |  30692000 |   54226
```

Cột `da_ket_thuc` cộng lại đúng 3.168. Chú thích `-- huỷ / đã kết thúc` cạnh biểu thức cho người đọc thấy mẫu số mà không phải đoán.

Trong mã cũ bạn sẽ gặp dạng `CASE` (PG-011): `count(CASE WHEN status = 'cancelled' THEN 1 END)` và `sum(CASE WHEN … THEN 1 ELSE 0 END)` đều ra 461 chuyến huỷ, `sum(CASE WHEN status = 'completed' THEN fare END)` ra 182.287.000 đ — trùng `FILTER` (thí nghiệm 5). `FILTER` nói thẳng ý định; bài dùng nó từ đây.

## 6. Ba lỗi mẫu số trên cùng một dữ liệu

Câu 3 của lab tính lại các chỉ số theo thành phố, mỗi chỉ số một cách sai và một cách đúng — chính là ba con số bị hỏi lại:

```sql
SELECT c.code AS tp,
       count(*) FILTER (WHERE t.status = 'cancelled') / count(*)                   AS a_chia_nguyen,
       100 * count(*) FILTER (WHERE t.status = 'cancelled') / count(*)             AS a_pct_nguyen,
       round(100.0 * count(*) FILTER (WHERE t.status = 'cancelled') / count(*), 2) AS a_pct_dung,
       round(avg(t.fare))                                                         AS b_cuoc_tb_sai,
       round(avg(t.fare) FILTER (WHERE t.status = 'completed'))                   AS b_cuoc_tb_dung,
       round(100.0 * count(*) FILTER (WHERE t.status = 'cancelled')
             / count(*) FILTER (WHERE t.status = 'completed'), 2)                 AS c_pct_sai_mau
FROM trips AS t JOIN cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY c.code
ORDER BY c.code COLLATE "C";
```

```text
 tp | a_chia_nguyen | a_pct_nguyen | a_pct_dung | b_cuoc_tb_sai | b_cuoc_tb_dung | c_pct_sai_mau
----+---------------+--------------+------------+---------------+----------------+---------------
 DN |             0 |           15 |      15.25 |         68011 |          67907 |         17.99
 HN |             0 |           14 |      14.92 |         68348 |          66940 |         17.53
 SG |             0 |           14 |      14.10 |         68269 |          67451 |         16.41
```

- **a) Chia số nguyên.** `bigint / bigint` bỏ phần thập phân: huỷ ÷ tổng < 1 nên ra 0; nhân 100 trước thì ra 14 — «14 %» của báo cáo. Viết `100.0 * …`: hằng `100.0` kiểu `numeric` kéo cả phép tính sang số thập phân, `round(…, 2)` làm tròn khi hiển thị.
- **b) Trung bình trên sai tập dòng.** `avg(t.fare)` tính cả chuyến huỷ, vốn mang giá báo lúc đặt nhưng không ai trả: 68.269 đ thay vì 67.451 đ.
- **c) Đúng phép chia, sai mẫu số.** Huỷ ÷ hoàn thành = 16,41 % thay vì 14,10 %. Công thức không vô lý, nhưng không phải «tỉ lệ huỷ» theo định nghĩa ở mục 1 — hai đội dùng hai mẫu số thì không bao giờ khớp số.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Quy tắc mẫu số của Tới Nơi</span>
<p>Mỗi tỉ lệ và trung bình có mẫu số ghi trong định nghĩa chỉ số và trong chú thích SQL. Tỉ lệ dùng <code>100.0 *</code> trước phép chia; mẫu số có thể bằng 0 thì bọc <code>NULLIF(…, 0)</code> (PG-011). Trung bình luôn hỏi «trên những dòng nào?» — câu trả lời nằm trong <code>FILTER</code>.</p>
</div>

## 7. WHERE lọc dòng, HAVING lọc nhóm

**`HAVING`** là điều kiện trên **nhóm**, chạy sau `GROUP BY` nên dùng được hàm tổng hợp. Nhóm thành phố × dịch vụ có tỉ lệ huỷ trên 15 %:

```sql
SELECT c.code AS tp, t.kind, count(*) AS da_ket_thuc,
       round(100.0 * count(*) FILTER (WHERE t.status = 'cancelled') / count(*), 1) AS ty_le_huy_pct
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY c.code, t.kind
HAVING count(*) FILTER (WHERE t.status = 'cancelled') * 100.0 / count(*) > 15
ORDER BY ty_le_huy_pct DESC, c.code COLLATE "C", t.kind COLLATE "C";
```

```text
 tp |  kind  | da_ket_thuc | ty_le_huy_pct
----+--------+-------------+---------------
 SG | parcel |         251 |          21.9
 HN | food   |         503 |          17.9
 DN | food   |         210 |          17.6
 DN | ride   |         177 |          16.4
```

`HAVING` phải viết lại biểu thức vì tên `ty_le_huy_pct` chưa có ở bước đó (mục 4); `ORDER BY` thì dùng được tên. Để ý `HAVING` so trên tỉ lệ **chưa làm tròn**: so trên số đã `round` thì nhóm 15,04 % thành 15,0 và trượt điều kiện.

Câu kế tiếp trong tệp lab dùng cả hai mệnh đề — tài xế hoàn thành từ 63 chuyến: `WHERE t.status = 'completed' AND <kỳ> … GROUP BY d.code, c.code HAVING count(*) >= 63` ra 7 tài xế, đầu danh sách là 8101 và 8109 cùng 68 chuyến (`ORDER BY hoan_thanh DESC, tai_xe` — mã tài xế làm tie-breaker như PG-007). `status = 'completed'` là điều kiện trên **dòng** nên ở `WHERE`, để `count(*)` chỉ đếm chuyến hoàn thành; `count(*) >= 63` trên **nhóm** nên ở `HAVING`.

## 8. Tổng phụ và tổng cộng với ROLLUP

**`GROUP BY ROLLUP (c.code, t.kind)`** sinh ba cấp nhóm trong một câu: `(tp, kind)`, `(tp)` và `()` — tổng tất cả. Ở dòng tổng phụ, cột bị «cuộn» hiện NULL; **`grouping(cột)`** trả 1 khi cột đó đang bị cuộn:

```sql
SELECT CASE WHEN grouping(c.code) = 1 THEN 'TỔNG' ELSE c.code END          AS tp,
       CASE WHEN grouping(t.kind) = 1 THEN '(mọi dịch vụ)' ELSE t.kind END AS kind,
       count(*) FILTER (WHERE t.status = 'completed')                      AS hoan_thanh,
       sum(t.fare) FILTER (WHERE t.status = 'completed')                   AS doanh_thu
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00+07' AND t.requested_at < '2026-10-31 00:00+07'
GROUP BY ROLLUP (c.code, t.kind)
ORDER BY grouping(c.code), c.code COLLATE "C", grouping(t.kind), t.kind COLLATE "C";
```

```text
  tp  |     kind      | hoan_thanh | doanh_thu
------+---------------+------------+-----------
 DN   | food          |        173 |  14902000
 …
 DN   | (mọi dịch vụ) |        389 |  26416000
 …
 SG   | (mọi dịch vụ) |       1377 |  92880000
 TỔNG | (mọi dịch vụ) |       2707 | 182287000
```

Một con số tổng chỉ đáng tin khi một đường tính **độc lập** ra cùng kết quả. Lab đối chiếu bằng câu tổng thẳng trên `trips` — không `JOIN`, không `ROLLUP`: `count(*), sum(fare) … WHERE status = 'completed'` trong kỳ ra đúng `2707 | 182287000`. Con số **182.287.000 đ** sẽ theo bạn tới PG-013 (khi `JOIN` với `payments` làm nó phình) và PG-020 (khi đối chiếu tiền thật).

**NULL của ROLLUP và NULL của dữ liệu.** Tài liệu cảnh báo: cột được nhóm mà có NULL thật thì NULL đó không phân biệt được với NULL của dòng tổng phụ. Ở bảng trên `c.code`, `t.kind` đều NOT NULL nên an toàn. Thử với khu đón của chuyến Đà Nẵng — 4 chuyến mất GPS — và gắn nhãn bằng `coalesce(z.code, 'TỔNG')` thay vì `grouping()` (thí nghiệm 6): sau bốn dòng khu `DN-BI-01` … `DN-TT-02` là **hai** dòng `TỔNG | 4` và `TỔNG | 460` — dòng 4 là nhóm mất GPS, dòng 460 mới là tổng. Viết `CASE WHEN grouping(z.code) = 1 THEN 'TỔNG' ELSE coalesce(z.code, 'không rõ (mất GPS)') END` thì ra đúng. Vẫn là bài học PG-011: một ô NULL, hai nghĩa, phân biệt bằng một nguồn khác.

## 9. Hai mẫu số trong cùng một dòng

Câu 7 của lab nối chuyến hoàn thành trong kỳ với `trip_ratings` bằng `LEFT JOIN`, nhóm theo thành phố, và đặt cạnh nhau `count(r.trip_id)`, `100.0 * count(r.trip_id) / count(*)`, `count(r.stars)`, `avg(r.stars)`:

```text
 tp | hoan_thanh | co_danh_gia | do_phu_pct | co_sao | diem_tb
----+------------+-------------+------------+--------+---------
 DN |        389 |         266 |       68.4 |    251 |    4.30
 HN |        941 |         670 |       71.2 |    619 |    4.17
 SG |       1377 |         955 |       69.4 |    868 |    4.23
```

**Độ phủ** chia cho chuyến hoàn thành; `r.trip_id` chỉ NULL ở chuyến không có dòng đánh giá (PG-011), nên `count(r.trip_id)` đếm chuyến *có* đánh giá. **Điểm trung bình** chia cho lượt có sao — `avg` tự bỏ NULL. Đặt `co_danh_gia` và `co_sao` ngay trong bảng là cách rẻ nhất để người đọc kiểm mẫu số: 4,30 của Đà Nẵng dựa trên 251 lượt, ít hơn hẳn 868 lượt của TP. HCM.

## 10. Những con số không cộng được

Con số 1.103 của Marketing đến từ câu 8 của lab — khách khác nhau theo dịch vụ, có `ROLLUP`:

```text
  kind  | so_khach | hoan_thanh | chuyen_moi_khach
--------+----------+------------+------------------
 food   |      441 |       1201 |             2.72
 parcel |      241 |        409 |             1.70
 ride   |      421 |       1097 |             2.61
 TỔNG   |      569 |       2707 |             4.76
```

`hoan_thanh` cộng được: 1.201 + 409 + 1.097 = 2.707. `so_khach` thì không: 441 + 241 + 421 = 1.103, trong khi dòng TỔNG — `ROLLUP` tính lại `count(DISTINCT customer_id)` từ dòng gốc — chỉ có **569**. Khách vừa đặt xe vừa gọi đồ ăn được đếm ở cả hai dòng. Số «khác nhau» của cấp trên phải **tính lại**, không cộng từ cấp dưới.

Tỉ lệ cũng không lấy trung bình cộng được: tỉ lệ huỷ cả tháng là 461 ÷ 3.168 = **14,55 %**, còn trung bình cộng 9 tỉ lệ ở bảng mục 5 là **14,35 %** (thí nghiệm 7) — nhóm 72 chuyến có trọng số bằng nhóm 717 chuyến. Muốn tỉ lệ cấp trên, cộng tử số và mẫu số rồi mới chia.

## 11. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Tỉ lệ ra 0 hoặc số tròn | `bigint / bigint` chia nguyên | `100.0 * …` trước khi chia |
| Cước trung bình cao bất thường | `avg` trên cả chuyến huỷ | `avg(fare) FILTER (WHERE status = 'completed')` |
| `must appear in the GROUP BY clause` | cột không nhóm, không tổng hợp | thêm vào `GROUP BY` hoặc nhóm theo khoá chính |
| `aggregate functions are not allowed in WHERE` | điều kiện nhóm ở `WHERE` | chuyển sang `HAVING` |
| Hai dòng «TỔNG» sau `ROLLUP` | `COALESCE` gắn nhãn cả NULL dữ liệu | `grouping(cột)` |
| Tổng khách các dòng > tổng thật | cộng `count(DISTINCT)` | tính lại ở cấp trên |
| Tỉ lệ cấp trên lệch | trung bình các tỉ lệ con | Σ tử ÷ Σ mẫu |

## 12. Đánh đổi

- **`FILTER` hay `CASE`?** `FILTER` ngắn, rõ ý; `sum(CASE …)` chạy ở nhiều database hơn.
- **Một câu nhiều cột hay nhiều câu?** Bảng mục 5 tính sáu chỉ số trong một lần quét; tách sáu câu thì mỗi câu tự lọc kỳ — sáu chỗ có thể lệch nhau.
- **`ROLLUP` hay cộng ở bảng tính?** `ROLLUP` tính lại từ dòng gốc nên đúng cả với `count(DISTINCT)` và tỉ lệ; cộng ở Excel chỉ đúng với số đếm và tổng tiền. Đổi lại phải đọc NULL bằng `grouping()`.

## 13. Bài tập

Lời giải đã chạy thật trong `p02/thu-nghiem/pg012-bai-tap.sql` (nền như mục 1).

**Bài 1.** Lập bảng tháng 10 theo **dịch vụ** cho cả nước: đã kết thúc, hoàn thành, huỷ, tỉ lệ huỷ, doanh thu, cước trung bình. Kiểm hai tổng trước khi nộp.

<details><summary><b>Lời giải</b></summary>
<p>Câu ở mục 5, bỏ <code>c.code</code> khỏi <code>SELECT</code>/<code>GROUP BY</code> và bỏ <code>JOIN cities</code>. Kết quả: food <code>1430 | 1201 | 229 | 16.0 | 102825000 | 85616</code>, parcel <code>488 | 409 | 79 | 16.2 | 19711000 | 48193</code>, ride <code>1250 | 1097 | 153 | 12.2 | 59751000 | 54468</code>. Hai tổng kiểm: đã kết thúc cộng lại 3.168, doanh thu cộng lại 182.287.000 đ. Tỉ lệ huỷ và cước trung bình thì không cộng; tỉ lệ cả nước tính lại là 14,55 %.</p>
</details>

**Bài 2.** Liệt kê quán có **ít nhất 50 đơn** trong tháng **và** tỉ lệ huỷ từ 20 % trở lên. Điều kiện nào ở `WHERE`, điều kiện nào ở `HAVING`? Chọn được `m.name` khi chỉ nhóm theo `m.id` không?

<details><summary><b>Lời giải</b></summary>
<p>Từ <code>trips JOIN merchants JOIN cities</code>, lọc kỳ ở <code>WHERE</code>, <code>GROUP BY m.id, c.code</code>, rồi <code>HAVING count(*) &gt;= 50 AND count(*) FILTER (WHERE t.status = 'cancelled') * 100.0 / count(*) &gt;= 20</code> (câu đầy đủ trong tệp lời giải).</p>
<p>Bốn quán: Bún Bò Lab 201 (HN, 63 đơn, 27,0 %), Bún Bò Lab 101 (SG, 70, 24,3 %), Bánh Mì Lab 203 (HN, 58, 22,4 %), Gà Rán Lab 105 (SG, 54, 20,4 %). Kỳ báo cáo là điều kiện trên dòng (<code>WHERE</code>), hai ngưỡng trên nhóm (<code>HAVING</code>). Nhóm theo khoá chính <code>m.id</code> nên chọn được <code>m.name</code>; <code>c.code</code> thuộc bảng khác nên phải nằm trong <code>GROUP BY</code>. Ngưỡng 50 đơn loại Bún Bò Lab 301 (46 đơn, 26,1 %): tỉ lệ cao nhưng mẫu nhỏ.</p>
</details>

**Bài 3.** Tìm tài xế có điểm sao trung bình **dưới 4,0**, chỉ xét người có **ít nhất 35 lượt có sao** trong tháng. Một bạn đặt ngưỡng là `count(*) >= 35`. Kết quả khác gì?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT d.code AS tai_xe, count(*) AS hoan_thanh, count(r.stars) AS co_sao,
       round(avg(r.stars), 2) AS diem_tb
FROM trips AS t
JOIN drivers AS d ON d.id = t.driver_id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at &gt;= '2026-10-01 00:00+07' AND t.requested_at &lt; '2026-10-31 00:00+07'
GROUP BY d.code
HAVING count(r.stars) &gt;= 35 AND avg(r.stars) &lt; 4
ORDER BY diem_tb, tai_xe;</code></pre>
<p>Bốn người: 8202 (36 lượt có sao, 3,94), 8211 (40, 3,95), 8215 (39, 3,95), 8209 (38, 3,97). Ngưỡng phải đặt trên <b>mẫu số của <code>avg</code></b>, tức <code>count(r.stars)</code>. Với <code>count(*) &gt;= 35</code> (đếm chuyến hoàn thành) lọt thêm 8105 (51 chuyến, chỉ 31 lượt có sao, 3,74) và 8110 (57 chuyến, 33 lượt, 3,91) — điểm của họ dựa trên ít lượt hơn mức đội chất lượng đặt ra.</p>
</details>

## 14. Đọc tiếp

- [PostgreSQL 17 · Aggregate Functions](https://www.postgresql.org/docs/17/functions-aggregate.html): kiểu trả về, hành vi trên tập rỗng.
- [PostgreSQL 17 · Table Expressions](https://www.postgresql.org/docs/17/queries-table-expressions.html): `GROUP BY`, `HAVING`, `GROUPING SETS`, `ROLLUP`.
- [PostgreSQL 17 · Aggregate Expressions](https://www.postgresql.org/docs/17/sql-expressions.html#SYNTAX-AGGREGATES): cú pháp `FILTER`, `DISTINCT`.
- [PostgreSQL 17 · Grouping Operations](https://www.postgresql.org/docs/17/functions-grouping.html): hàm `GROUPING`.
- Lab: `p02/12-aggregate-group-by.sql`, `p02/OUTPUT.txt`; thí nghiệm `p02/thu-nghiem/pg012-thu-nghiem.sql`, `p02/thu-nghiem/pg012-bai-tap.sql`.

Bài tiếp theo: [PG-013 · JOIN và số dòng sau khi nối bảng](/articles/postgresql-join-quan-he-cardinality).
