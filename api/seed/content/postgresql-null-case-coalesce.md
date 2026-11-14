---
slug: postgresql-null-case-coalesce
title: "NULL, CASE và COALESCE trong dữ liệu chuyến"
summary: "Hai bản báo cáo chất lượng tháng 10 của Tới Nơi ghi điểm sao 4,22 và 3,88 từ cùng một bảng; bảng điều khiển ngày bão ghi Đà Nẵng «0 % huỷ» dù không có chuyến nào. Học logic ba giá trị của NULL, CASE, COALESCE và NULLIF để phân biệt dữ liệu thiếu, giá trị không áp dụng và số 0 thật."
category: postgresql
level: beginner
tags: [postgresql, pg-011, null, case, coalesce, nullif, is-distinct-from, toi-noi]
language: vi
---

Sáng 31/10, ban điều hành Tới Nơi nhận hai bản báo cáo chất lượng tháng 10: một bản ghi điểm sao trung bình **4,22**, bản kia **3,88** — cùng bảng `trip_ratings`, cùng ảnh chụp dữ liệu. Cùng lúc, ô «tỉ lệ huỷ theo thành phố ngày 20/10» trên bảng điều khiển báo `division by zero`. Người trực sửa vội bằng cách thay mẫu số rỗng bằng 0, và màn hình ghi Đà Nẵng «0 % huỷ» cho đúng ngày bão — ngày không có lấy một chuyến.

Không câu nào sai cú pháp. Mỗi người đã ngầm quyết định một ô trống nghĩa là gì. Bài này chỉ ra cách PostgreSQL xử lý NULL và bốn công cụ để nói rõ ý định: `IS NULL`, `CASE`, `COALESCE`, `NULLIF`.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-011 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid">PG-004 · kiểu dữ liệu</a> (ba nghĩa của NULL trong data dictionary), <a href="/articles/postgresql-select-filter-order-limit">PG-007 · SELECT, WHERE, ORDER BY</a>. Lab: <code>examples/toi-noi-postgresql/p02</code> (tệp <code>11-null-case-coalesce.sql</code>), thí nghiệm thêm ở <code>p02/thu-nghiem/pg011-*.sql</code>. Sau bài này, bạn phân biệt được thiếu dữ liệu, không áp dụng và giá trị 0 trong truy vấn lẫn báo cáo.</p>
</div>

## 1. Dữ liệu dùng trong bài

Phần 2 dùng lại schema của Phần 1 với bộ dữ liệu lớn hơn: **3.174 chuyến** đặt từ 01/10, ảnh chụp lúc **01:00 ngày 31/10/2026 giờ VN**, sinh tất định (chạy lại luôn ra đúng các số trong bài). Bài này dùng thêm `trips.pickup_area_id` (khu đón, NULL khi mất GPS) và `trip_ratings` (tối đa một dòng mỗi chuyến; `stars` 1–5 hoặc NULL, `tip` ≥ 0 đ hoặc NULL).

```bash
cd examples/toi-noi-postgresql
npm ci
node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/11-null-case-coalesce.sql
```

Kết quả chép từ lần chạy thật trên PGlite (PostgreSQL 17.5), bỏ dòng `-- [lab]` và dòng đếm `(n rows)`. **Ô trống trong bảng kết quả là NULL**, như psql mặc định; trong psql, `\pset null 'NULL'` cho NULL hiện thành chữ.

## 2. Một ô trống, ba nghĩa

| Cột | Ô trống nghĩa là | Loại | Số dòng |
|---|---|---|---|
| `trips.driver_id` | đang tìm tài xế | **thiếu** — rồi sẽ có | 2 |
| `trips.driver_id` | huỷ trước khi có người nhận | **không áp dụng** | 295 |
| `trips.merchant_id` | đơn đặt xe, bưu kiện | **không áp dụng** | 1.742 |
| `trips.pickup_area_id` | app mất GPS | **thiếu** — không bao giờ biết | 36 |
| `trip_ratings.stars` | khách bỏ trống phần sao | **thiếu** | 153 |
| `trip_ratings.tip` | khách bỏ qua bước tip | **thiếu** | 541 |
| `tip = 0` · `fare = 0` | chọn «không tip» · chuyến khuyến mãi | **giá trị thật** | 660 · 38 |

SQL chỉ có **một** NULL cho cả hai nghĩa đầu; muốn biết là nghĩa nào thì phải suy từ chỗ khác — cột `status`, ràng buộc (CHECK `ck_trips_merchant_only_food` của PG-005 buộc chỉ đơn đồ ăn mới có quán), hay chú thích cột (lab ghi `COMMENT ON COLUMN trip_ratings.tip` là «NULL = bỏ qua bước tip, 0 = chọn không tip»). Còn số 0 là một **khẳng định**. Biến NULL thành 0 là thay «không biết» bằng một khẳng định — phần lớn lỗi trong bài bắt đầu từ đó.

## 3. Logic ba giá trị

So sánh với NULL không ra `true`/`false` mà ra NULL — giá trị chân trị thứ ba, đọc là «không biết»:

```sql
SELECT NULL = NULL AS bang_nhau, NULL <> 1 AS khac_1, NULL IS NULL AS la_null,
       NULL IS DISTINCT FROM NULL AS khac_biet_null, 1 IS DISTINCT FROM NULL AS khac_biet_1;
```

```text
 bang_nhau | khac_1 | la_null | khac_biet_null | khac_biet_1
-----------+--------+---------+----------------+-------------
           |        | t       | f              | t
```

Khi ghép điều kiện, kết quả chắc chắn khi vế đã biết đã đủ để quyết định: `false AND NULL` là `false`, `true OR NULL` là `true`; còn `true AND NULL`, `false OR NULL`, `NOT NULL` vẫn là NULL (bảng chân trị ở thí nghiệm 1 của `pg011-logic-ba-gia-tri.sql`).

Điều quan trọng nhất: **`WHERE` chỉ giữ dòng có điều kiện ra `true`**; dòng ra NULL bị loại như dòng `false`.

```sql
SELECT count(*) FROM trips WHERE driver_id = NULL;   -- 0
SELECT count(*) FROM trips WHERE driver_id IS NULL;  -- 297
```

Câu đầu không lỗi, không cảnh báo — chỉ ra 0. Ngược lại, `CHECK` **cho qua** dòng ra NULL (PG-005), nên `CHECK (stars BETWEEN 1 AND 5)` chấp nhận `stars` NULL; muốn bắt buộc thì thêm NOT NULL.

## 4. CASE: đọc nghĩa của NULL từ các cột khác

**`CASE WHEN điều_kiện THEN kết_quả … ELSE mặc_định END`** là `if/else` của SQL. PostgreSQL xét các nhánh **theo thứ tự**, dừng ở nhánh đầu tiên ra `true`; không nhánh nào đúng thì lấy `ELSE`, không có `ELSE` thì ra NULL.

```sql
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
```

```text
            y_nghia_cot_driver_id            | so_chuyen
---------------------------------------------+-----------
 có tài xế                                   |      2877
 không áp dụng — huỷ trước khi có người nhận |       295
 chưa có — đang tìm tài xế                   |         2
```

(`GROUP BY 1` gom dòng cùng giá trị cột thứ nhất để `count(*)` đếm từng nhóm — PG-012.) Nhánh `ELSE` là câu kiểm tra miễn phí: hễ nhóm này xuất hiện là có chuyến đã nhận mà mất tài xế. Hôm nay nhóm này vắng mặt nhờ ràng buộc `ck_trips_assigned_has_driver`.

### Bốn trạng thái đánh giá sau LEFT JOIN

**`LEFT JOIN`** giữ mọi chuyến bên trái, kể cả chuyến không có dòng đánh giá; khi đó các cột của `trip_ratings` là NULL (PG-013 nói kỹ). Vậy `r.stars` NULL có hai nguyên nhân: không có dòng đánh giá, hoặc có dòng mà khách bỏ trống sao. Tách bằng một cột **không bao giờ NULL** khi dòng tồn tại — khoá chính `r.trip_id`:

```sql
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
```

```text
          tinh_trang_danh_gia          | so_chuyen
---------------------------------------+-----------
 có sao                                |      1738
 thiếu — chưa đánh giá                 |       816
 không áp dụng — chuyến huỷ            |       461
 thiếu — mở màn đánh giá, bỏ trống sao |       153
```

Bốn nhóm cộng lại 3.168 — đúng số chuyến đã kết thúc. Chỉ nhìn `r.stars IS NULL` thì 1.430 chuyến cần ba cách xử lý khác nhau bị gộp làm một.

### Hai bẫy của CASE

- **Thứ tự nhánh.** Đưa nhánh `status = 'cancelled'` lên đầu câu `driver_id` ở trên: nhóm «huỷ trước khi có người nhận» phình từ 295 lên **461**, vì 166 chuyến huỷ *sau* khi đã có tài xế bị nhánh đầu lấy mất (thí nghiệm 3). Nhánh hẹp đặt trước, và tổng các nhóm phải khớp tổng số dòng.
- **Dạng CASE đơn giản không bắt được NULL.** `CASE stars WHEN NULL THEN 'không rõ' ELSE 'có sao' END` so sánh bằng `=`, nên cả 1.891 lượt — kể cả 153 lượt bỏ trống — thành «có sao» (thí nghiệm 2). Viết `CASE WHEN stars IS NULL THEN …`.

## 5. Hàm tổng hợp bỏ qua NULL

`count(cột)`, `sum`, `avg`, `min`, `max` **bỏ qua đầu vào NULL**; `count(*)` đếm dòng nên không quan tâm NULL. Đó là nguồn gốc hai con số của ban điều hành:

```sql
SELECT count(*) AS so_danh_gia, count(stars) AS co_sao,
       round(avg(stars), 2)              AS diem_tb_dung,
       round(avg(coalesce(stars, 0)), 2) AS diem_tb_sai
FROM trip_ratings;
```

```text
 so_danh_gia | co_sao | diem_tb_dung | diem_tb_sai
-------------+--------+--------------+-------------
        1891 |   1738 |         4.22 |        3.88
```

`avg(stars)` chia cho 1.738 lượt có sao. Bản 3,88 dùng `coalesce(stars, 0)` (mục 8) «cho khỏi NULL» — biến 153 lượt «không biết» thành 0 sao, một mức điểm không tồn tại, rồi chia cho 1.891.

Cũng vì NULL, `stars = 5` (896) và `stars <> 5` (842) chỉ phủ 1.738 dòng; 153 dòng còn lại không thuộc nhóm nào. Chia khách thành hai nhóm bằng `=`/`<>` thì phải luôn tự hỏi: nhóm NULL nằm ở đâu?

**Tập rỗng.** Tài liệu ghi rõ: trừ `count`, hàm tổng hợp trả NULL khi không có dòng nào — `sum` của 0 dòng là NULL, không phải 0. Đà Nẵng ngày 20/10 cho `count(*) = 0`, `sum(fare)` và `avg(fare)` đều NULL (thí nghiệm 5). Với **tiền**, `coalesce(sum(fare), 0)` đúng nghĩa: không có chuyến thì doanh thu là 0 đ. Với **trung bình** thì không: cước trung bình của 0 chuyến không phải 0 đ.

## 6. Tip: NULL không phải 0, và NULL lây qua phép tính

Câu hỏi nghiệp vụ quyết định mẫu số:

```sql
SELECT count(*) AS so_danh_gia, count(tip) AS co_tra_loi_tip,
       count(*) FILTER (WHERE tip = 0) AS chon_0d, sum(tip) AS tong_tip,
       round(avg(tip))              AS tb_tren_nguoi_tra_loi,
       round(avg(coalesce(tip, 0))) AS tb_tren_moi_luot
FROM trip_ratings;
```

```text
 so_danh_gia | co_tra_loi_tip | chon_0d | tong_tip | tb_tren_nguoi_tra_loi | tb_tren_moi_luot
-------------+----------------+---------+----------+-----------------------+------------------
        1891 |           1350 |     660 |  5565000 |                  4122 |             2943
```

(`count(*) FILTER (WHERE …)` là đếm có điều kiện — PG-012.) «Khách có trả lời tip trung bình bao nhiêu» là 4.122 đ; «mỗi lượt đánh giá mang về bao nhiêu tip» là 2.943 đ, với giả định ghi rõ «bỏ qua = không tip». Cả hai đúng, cho hai câu hỏi khác nhau.

Trong biểu thức thường, NULL **lây**: chuyến 100003 cước 50.000 đ, bỏ qua tip, thì `fare + tip` là NULL. Ghép với quy tắc «`sum` bỏ NULL», `sum(fare + tip)` bỏ luôn **cả cước** của 541 lượt bỏ qua tip (thí nghiệm 6):

```sql
SELECT sum(t.fare) + sum(r.tip)         AS cong_hai_tong,
       sum(t.fare + r.tip)              AS tong_cua_tong_dong,
       sum(t.fare + coalesce(r.tip, 0)) AS tong_dong_coalesce
FROM trips AS t JOIN trip_ratings AS r ON r.trip_id = t.id;
```

```text
 cong_hai_tong | tong_cua_tong_dong | tong_dong_coalesce
---------------+--------------------+--------------------
     134142000 |           98163000 |          134142000
```

Thiếu gần 36 triệu đồng mà không có lỗi hay cảnh báo nào. Ở đây `coalesce(r.tip, 0)` đúng nghĩa vì câu hỏi là **tiền**: bỏ qua tip thì tài xế nhận 0 đ tip. Cùng một hàm, đúng với tiền, sai với điểm sao — điều quyết định là nghĩa của con số.

## 7. NULLIF: biến một giá trị thành NULL có chủ đích

**`NULLIF(a, b)`** trả NULL nếu `a = b`, ngược lại trả `a` — chiều ngược của `COALESCE`.

**Cước 0 đ.** 31 chuyến hoàn thành có cước 0 đ (khuyến mãi) — dữ liệu đúng. `avg(fare)` trên 2.707 chuyến hoàn thành ra **67.339 đ**; «cước trung bình của chuyến có thu tiền» là `avg(nullif(fare, 0))` = **68.119 đ**. Lưu dữ liệu thật (0 đ), còn việc loại hay không thì quyết định ở câu truy vấn.

**Chống chia cho 0.** Câu tỉ lệ huỷ ngày 20/10 đi từ `cities`, `LEFT JOIN trips` để thành phố không có chuyến vẫn có dòng, rồi chia cho `count(t.id)` — với Đà Nẵng là 0:

```text
ERROR:  division by zero
```

Lỗi `22012` làm hỏng **cả câu**, mất luôn số của Hà Nội và TP. HCM. Sửa bằng `NULLIF(mẫu_số, 0)`:

```sql
SELECT c.code, count(t.id) AS tong,
       round(100.0 * count(t.id) FILTER (WHERE t.status = 'cancelled')
             / nullif(count(t.id), 0), 1) AS ty_le_huy_pct
FROM cities AS c
LEFT JOIN trips AS t
  ON t.city_id = c.id
 AND t.requested_at >= '2026-10-20 00:00+07' AND t.requested_at < '2026-10-21 00:00+07'
GROUP BY c.code
ORDER BY c.code COLLATE "C";
```

```text
 code | tong | ty_le_huy_pct
------+------+---------------
 DN   |    0 |
 HN   |   37 |          13.5
 SG   |   49 |          14.3
```

Cột tỉ lệ giữ NULL để biểu đồ và cảnh báo phía sau biết đó là «không xác định»; câu lab có thêm cột `hien_thi` = `coalesce(tỉ_lệ::text, 'không có chuyến')` cho người đọc. Cách sửa vội `coalesce(…, 0)` biến một ngày không có dữ liệu thành khẳng định «Đà Nẵng không huỷ chuyến nào» — và cảnh báo tỉ lệ huỷ bất thường không bao giờ được kích hoạt cho ngày đó.

## 8. COALESCE: lấp chỗ trống khi hiển thị

**`COALESCE(a, b, …)`** trả đối số **đầu tiên khác NULL**. Theo tài liệu, hàm này chỉ tính các đối số cần thiết (giống `CASE`), và mọi đối số phải chuyển được về **một kiểu chung**. Chỗ dùng hợp lý nhất là ở tầng hiển thị — câu 7 của lab gắn nghĩa cho từng ô trống (trích hai cột):

```sql
SELECT t.code, t.status,
       coalesce(d.code::text,
                CASE WHEN t.status IN ('requested', 'searching') THEN 'đang tìm…'
                     ELSE '— (huỷ trước khi nhận)' END) AS tai_xe,
       coalesce(z.code, 'không rõ (mất GPS)')           AS khu_don
FROM trips AS t
LEFT JOIN drivers AS d ON d.id = t.driver_id
LEFT JOIN areas   AS z ON z.id = t.pickup_area_id
WHERE …   -- vài chuyến mẫu, xem tệp lab
```

```text
  code  |  status   |         tai_xe         |      khu_don
--------+-----------+------------------------+--------------------
 100001 | cancelled | — (huỷ trước khi nhận) | HN-TA-01
 100072 | completed | 8110                   | không rõ (mất GPS)
 103172 | searching | đang tìm…              | HN-TT-03
 …
```

Đối số thứ hai có thể là cả một `CASE`, nên cùng một `driver_id` NULL hiện thành «đang tìm…» hay «huỷ trước khi nhận» tuỳ trạng thái. Để ý `d.code::text`: `drivers.code` là `integer`, bỏ phép ép kiểu thì PostgreSQL cố đọc chữ thành số và báo `invalid input syntax for type integer: "đang tìm…"` (thí nghiệm 7). Quy tắc của bài: cột để **tính tiếp** giữ NULL, chỉ cột để **đọc** mới lấp chữ.

## 9. `<>` cũng bỏ rơi NULL: IS DISTINCT FROM

«Bao nhiêu chuyến **không** đón ở khu SG-TT-01 (id 111)?»

```sql
SELECT count(*) AS tong,
       count(*) FILTER (WHERE pickup_area_id = 111)                AS o_111,
       count(*) FILTER (WHERE pickup_area_id <> 111)               AS khac_111,
       count(*) FILTER (WHERE pickup_area_id IS DISTINCT FROM 111) AS khac_biet_111
FROM trips;
```

```text
 tong | o_111 | khac_111 | khac_biet_111
------+-------+----------+---------------
 3174 |   162 |     2976 |          3012
```

162 + 2.976 thiếu đúng 36 chuyến mất GPS. **`IS DISTINCT FROM`** so sánh như `<>` nhưng coi NULL là một giá trị (dạng phủ định: `IS NOT DISTINCT FROM`). «Chắc chắn đón ngoài SG-TT-01» là 2.976, «không chắc đón ở SG-TT-01» là 3.012 — chọn câu nào tuỳ nghiệp vụ, miễn là chọn có chủ đích. Bẫy cùng loại là `NOT IN (danh sách có NULL)` trả 0 dòng — PG-014.

Một điểm ngược với `=`: `DISTINCT` và `GROUP BY` gom mọi NULL vào **một** nhóm, còn `count(DISTINCT cột)` bỏ NULL — `SELECT DISTINCT stars` ra 6 dòng, `count(DISTINCT stars)` ra 5 (thí nghiệm 8).

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| `WHERE x = NULL` luôn 0 dòng | so sánh với NULL ra NULL | `IS NULL` |
| Điểm trung bình tụt | `avg(coalesce(stars, 0))` | `avg(stars)` kèm `count(stars)` |
| Nhóm «bằng» + «khác» < tổng | `=`, `<>` loại NULL | thêm nhóm `IS NULL` / `IS DISTINCT FROM` |
| `CASE x WHEN NULL` không khớp | CASE đơn giản dùng `=` | `CASE WHEN x IS NULL` |
| Một nhóm CASE phình to | nhánh rộng đứng trước | nhánh hẹp trước; đối chiếu tổng |
| `sum(a + b)` thiếu tiền | NULL lây qua `+` | `sum(a) + coalesce(sum(b), 0)` |
| `division by zero` mất cả bảng | mẫu số 0 ở một nhóm | `NULLIF(mẫu_số, 0)` |
| «0 %» cho ngày không có dữ liệu | `coalesce(tỉ_lệ, 0)` | giữ NULL, dịch chữ ở cột hiển thị |

## 11. Đánh đổi

- **NULL hay giá trị giả (`-1`, `''`)?** Giá trị giả giúp `=` «chạy được» nhưng bị `avg`, `sum`, `min` tính vào, và CHECK phải nới. NULL buộc xử lý rõ ràng; giá trị giả cho phép quên.
- **Một NULL hay thêm cột lý do?** Khi «thiếu» và «không áp dụng» chung một cột mà không suy được từ cột khác, một cột lý do (vd. `tip_status`) đáng giá hơn quy ước ngầm. Ở fixture này, `status` và CHECK đủ để `CASE` suy ra.
- **`tip NOT NULL DEFAULT 0`?** Dễ viết báo cáo, nhưng mất vĩnh viễn khác biệt giữa 660 khách chọn «không tip» và 541 khách bỏ qua — đúng thứ đội sản phẩm muốn đo.

## 12. Bài tập

Lời giải đã chạy thật trong `p02/thu-nghiem/pg011-bai-tap.sql` (nền như mục 1).

**Bài 1.** Viết `CASE` dịch `trips.merchant_id` thành «có quán», «không áp dụng — không phải đơn đồ ăn», «LỖI — đơn đồ ăn thiếu quán» rồi đếm. Dự đoán số dòng nhóm LỖI.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT CASE
         WHEN merchant_id IS NOT NULL THEN 'có quán'
         WHEN kind &lt;&gt; 'food'          THEN 'không áp dụng — không phải đơn đồ ăn'
         ELSE 'LỖI — đơn đồ ăn thiếu quán'
       END AS y_nghia_merchant_id,
       count(*) AS so_chuyen
FROM trips GROUP BY 1 ORDER BY 2 DESC;</code></pre>
<p>«Không áp dụng» 1.742, «có quán» 1.432; nhóm LỖI không xuất hiện vì <code>ck_trips_merchant_only_food</code> chặn từ lúc ghi. Nhánh <code>ELSE</code> vẫn có ích khi chạy câu này trên dữ liệu chép từ hệ thống cũ chưa có ràng buộc.</p>
</details>

**Bài 2.** Kế toán cần «cước + tip» của chuyến hoàn thành tháng 10 (đặt trong `[01/10, 31/10)` giờ VN) theo thành phố. Một bạn viết `sum(t.fare + r.tip)` với `LEFT JOIN trip_ratings`. Sai ở đâu?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT c.code AS tp,
       sum(t.fare + r.tip)                   AS sai_cong_trong_sum,
       sum(t.fare) + coalesce(sum(r.tip), 0) AS cuoc_cong_tip
FROM trips AS t
JOIN cities AS c ON c.id = t.city_id
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at &gt;= '2026-10-01 00:00+07' AND t.requested_at &lt; '2026-10-31 00:00+07'
GROUP BY c.code ORDER BY c.code COLLATE "C";</code></pre>
<p>Kết quả (sai / đúng): DN 13.866.000 / <b>27.221.000</b>, HN 34.957.000 / <b>64.981.000</b>, SG 49.340.000 / <b>95.650.000</b>. Cột sai mất gần một nửa: <code>r.tip</code> NULL ở cả chuyến không có dòng đánh giá (LEFT JOIN) lẫn chuyến bỏ qua tip, và <code>fare + NULL</code> kéo cả cước ra khỏi <code>sum</code>.</p>
</details>

**Bài 3.** Lập bảng ngày 20/10: mỗi thành phố một dòng (kể cả Đà Nẵng), gồm số chuyến hoàn thành, số lượt có sao, điểm trung bình và một cột hiển thị không bao giờ ghi số 0 giả. Câu có bị chia cho 0 như mục 7 không?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT c.code AS tp, count(t.id) AS hoan_thanh, count(r.stars) AS co_sao,
       round(avg(r.stars), 2) AS diem_tb,
       coalesce(round(avg(r.stars), 2)::text,
                CASE WHEN count(t.id) = 0 THEN 'không có chuyến'
                     ELSE 'chưa có lượt chấm sao' END) AS hien_thi
FROM cities AS c
LEFT JOIN trips AS t
  ON t.city_id = c.id AND t.status = 'completed'
 AND t.requested_at &gt;= '2026-10-20 00:00+07' AND t.requested_at &lt; '2026-10-21 00:00+07'
LEFT JOIN trip_ratings AS r ON r.trip_id = t.id
GROUP BY c.code ORDER BY c.code COLLATE "C";</code></pre>
<p>DN <code>0 | 0 | (NULL) | không có chuyến</code>, HN <code>32 | 22 | 4.23</code>, SG <code>42 | 28 | 4.39</code>. Không lỗi: <code>avg</code> trên tập rỗng trả NULL; chỉ phép chia tự viết mới cần <code>NULLIF</code>. Điều kiện của <code>trips</code> nằm trong <code>ON</code>: đưa xuống <code>WHERE</code> thì dòng Đà Nẵng (có <code>t.status</code> NULL) bị loại — PG-013.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 17 · Conditional Expressions](https://www.postgresql.org/docs/17/functions-conditional.html): `CASE`, `COALESCE`, `NULLIF`.
- [PostgreSQL 17 · Comparison Functions and Operators](https://www.postgresql.org/docs/17/functions-comparison.html): `IS NULL`, `IS DISTINCT FROM`.
- [PostgreSQL 17 · Logical Operators](https://www.postgresql.org/docs/17/functions-logical.html): bảng chân trị với NULL.
- [PostgreSQL 17 · Aggregate Functions](https://www.postgresql.org/docs/17/functions-aggregate.html): NULL và tập rỗng.
- Lab: `p02/11-null-case-coalesce.sql`, `p02/OUTPUT.txt`; thí nghiệm `p02/thu-nghiem/pg011-logic-ba-gia-tri.sql`, `p02/thu-nghiem/pg011-bai-tap.sql`.

Bài tiếp theo: [PG-012 · COUNT, SUM, GROUP BY và HAVING](/articles/postgresql-aggregate-group-by-having).
