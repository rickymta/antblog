---
slug: mssql-null-case-isnull-coalesce
title: "NULL, CASE, ISNULL và COALESCE trong dữ liệu chuyến Tới Nơi"
summary: "Ba bản báo cáo chất lượng tháng 10 ghi điểm sao 4, 4,22 và 3,88 từ cùng một bảng; màn hình CSKH hiện khu đón \"không r? (m?\"; ô tỉ lệ huỷ ngày bão báo lỗi 8134. Bài đi qua logic ba giá trị, CASE, ISNULL, COALESCE, NULLIF và IS DISTINCT FROM trên SQL Server 2022 để phân biệt thiếu dữ liệu, không áp dụng và số 0 thật."
category: mssql
level: beginner
tags: [ms-011, mssql, sql-server-2022, t-sql, null, case, isnull, coalesce, nullif, is-distinct-from, toi-noi]
language: vi
---

Sáng 31/10, ban điều hành Tới Nơi nhận **ba** bản báo cáo chất lượng tháng 10. Bản thứ nhất ghi điểm sao trung bình **4**, bản thứ hai **4,22**, bản thứ ba **3,88**. Cả ba chạy T-SQL trên cùng bảng `toinoi.trip_ratings`, cùng ảnh chụp dữ liệu, không câu nào báo lỗi.

Cùng buổi sáng, màn hình tra cứu của CSKH hiện cột khu đón của chuyến 100072 là `không r? (m?`, và ô "tỉ lệ huỷ theo thành phố ngày 20/10" trên bảng điều khiển báo `Divide by zero error encountered`. Người trực sửa vội bằng cách thay mẫu số rỗng bằng 0, và Đà Nẵng hiện "0 % huỷ" cho đúng ngày bão, ngày không có lấy một chuyến.

Mỗi con số sai vì một người đã ngầm quyết định một ô trống nghĩa là gì, hoặc vì một hàm của SQL Server làm việc khác điều người viết nghĩ. Bài này đi qua cách SQL Server xử lý NULL và các công cụ để nói rõ ý định: `IS NULL`, `CASE`, `ISNULL`, `COALESCE`, `NULLIF`, `IS DISTINCT FROM`.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-011 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode">MS-004 · kiểu dữ liệu cho tiền, thời gian và chuỗi tiếng Việt</a>, <a href="/articles/mssql-select-loc-sap-xep-top-offset">MS-007 · SELECT, WHERE, ORDER BY</a> (mục NULL trong WHERE). Lab: <code>examples/toi-noi-mssql/p02</code> (tệp <code>11-null-case-isnull-coalesce.sql</code>). Sau bài này, bạn phân biệt được thiếu dữ liệu, không áp dụng và giá trị 0 trong truy vấn lẫn báo cáo, và biết khi nào <code>ISNULL</code> khác <code>COALESCE</code>.</p>
</div>

## 1. Dữ liệu dùng trong bài

Phần 2 dùng lại schema của Phần 1 với bộ dữ liệu lớn hơn: **3.174 chuyến** đặt từ 01/10, ảnh chụp lúc **01:00 ngày 31/10/2026 giờ VN**, sinh tất định (chạy lại luôn ra đúng các số trong bài). Bộ dữ liệu này giống từng dòng với lab PostgreSQL Phần 2, nên bạn có thể đặt hai bài cạnh nhau. Bài này dùng thêm `trips.pickup_area_id` (khu đón, NULL khi mất GPS) và `trip_ratings` (tối đa một dòng mỗi chuyến; `stars` kiểu `tinyint` 1–5 hoặc NULL, `tip` kiểu `int` ≥ 0 đ hoặc NULL).

```bash
cd examples/toi-noi-mssql
node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/11-null-case-isnull-coalesce.sql
```

Kết quả bên dưới chép từ lần chạy trên SQL Server 2022 16.0.4325.4 (RTM-CU28), dạng `cột|cột` của sqlcmd; NULL hiện thành chữ `NULL`. Tệp lab có 42 kiểm tra, tất cả PASS.

| Cột | Ô trống nghĩa là | Loại | Số dòng |
|---|---|---|---|
| `trips.driver_id` | đang tìm tài xế | **thiếu**, rồi sẽ có | 2 |
| `trips.driver_id` | huỷ trước khi có người nhận | **không áp dụng** | 295 |
| `trips.merchant_id` | đơn đặt xe, bưu kiện | **không áp dụng** | 1.742 |
| `trips.pickup_area_id` | app tài xế mất GPS | **thiếu**, không bao giờ biết | 36 |
| `trip_ratings.stars` | khách bỏ trống phần sao | **thiếu** | 153 |
| `trip_ratings.tip` | khách bỏ qua bước tip | **thiếu** | 541 |
| `tip = 0` · `fare = 0` | chọn "không tip" · chuyến khuyến mãi | **giá trị thật** | 660 · 38 |

SQL chỉ có **một** NULL cho cả hai nghĩa đầu; muốn biết là nghĩa nào thì phải suy từ chỗ khác: cột `status`, ràng buộc (CHECK `ck_trips_merchant_only_food` của [MS-005](/articles/mssql-create-table-constraints) buộc chỉ đơn đồ ăn mới có quán), hay chú thích cột. Còn số 0 là một **khẳng định**. Biến NULL thành 0 là thay "không biết" bằng một khẳng định, và phần lớn lỗi trong bài bắt đầu từ đó.

## 2. Logic ba giá trị

So sánh với NULL không ra TRUE hay FALSE mà ra **UNKNOWN**, giá trị chân trị thứ ba (logic ba giá trị). T-SQL không có kiểu boolean để in thẳng kết quả của một điều kiện, nên mục 1 của lab "nhìn" điều kiện qua `CASE`:

```sql
SELECT CASE WHEN NULL = NULL THEN 'TRUE' WHEN NOT (NULL = NULL) THEN 'FALSE' ELSE 'UNKNOWN' END AS null_bang_null,
       CASE WHEN NULL <> 1   THEN 'TRUE' WHEN NOT (NULL <> 1)   THEN 'FALSE' ELSE 'UNKNOWN' END AS null_khac_1,
       CASE WHEN NULL IS NULL THEN 'TRUE' ELSE 'FALSE' END                    AS null_is_null,
       CASE WHEN NULL IS DISTINCT FROM NULL THEN 'TRUE' ELSE 'FALSE' END      AS null_distinct_null,
       CASE WHEN 1 IS DISTINCT FROM NULL THEN 'TRUE' ELSE 'FALSE' END         AS mot_distinct_null;
```

```text
null_bang_null|null_khac_1|null_is_null|null_distinct_null|mot_distinct_null
--------------|-----------|------------|------------------|-----------------
UNKNOWN|UNKNOWN|TRUE|FALSE|TRUE
```

`NOT UNKNOWN` vẫn là UNKNOWN, nên cả hai nhánh `WHEN` đầu đều không khớp và câu rơi vào `ELSE`. Điều quan trọng nhất: **`WHERE` chỉ giữ dòng có điều kiện ra TRUE**; dòng ra UNKNOWN bị loại như dòng FALSE.

```text
bang_null|is_null
---------|-------
0|297
```

`WHERE driver_id = NULL` ra 0 dòng, không lỗi, không cảnh báo; `WHERE driver_id IS NULL` ra đúng 297 chuyến chưa có tài xế. Ngược lại, `CHECK` **cho qua** dòng ra UNKNOWN (MS-005), nên `CHECK (stars BETWEEN 1 AND 5)` chấp nhận `stars` NULL; muốn bắt buộc thì thêm `NOT NULL`.

Bạn có thể gặp mã cũ đặt `SET ANSI_NULLS OFF`: khi đó `driver_id = NULL` khớp cả 297 dòng (lab kiểm lại). Tài liệu Microsoft ghi tuỳ chọn này sẽ luôn là ON ở phiên bản tương lai, và ứng dụng đặt OFF sẽ gặp lỗi. Đừng dựa vào nó; viết `IS NULL`.

## 3. CASE: đọc nghĩa của NULL từ các cột khác

**`CASE WHEN điều_kiện THEN kết_quả … ELSE mặc_định END`** là `if/else` của T-SQL. SQL Server xét các nhánh **theo thứ tự** và dừng ở nhánh đầu tiên ra TRUE; không nhánh nào đúng thì lấy `ELSE`, không có `ELSE` thì ra NULL.

```sql
SELECT y_nghia_cot_driver_id, COUNT(*) AS so_chuyen
FROM (SELECT CASE
               WHEN driver_id IS NOT NULL                 THEN N'có tài xế'
               WHEN status IN ('requested', 'searching')  THEN N'chưa có — đang tìm tài xế'
               WHEN status = 'cancelled'                  THEN N'không áp dụng — huỷ trước khi có người nhận'
               ELSE N'SAI QUY TẮC — lẽ ra phải có tài xế'
             END AS y_nghia_cot_driver_id
      FROM toinoi.trips) AS x
GROUP BY y_nghia_cot_driver_id
ORDER BY so_chuyen DESC;
```

```text
y_nghia_cot_driver_id|so_chuyen
---------------------|---------
có tài xế|2877
không áp dụng — huỷ trước khi có người nhận|295
chưa có — đang tìm tài xế|2
```

Hai chi tiết riêng của SQL Server: chuỗi tiếng Việt viết `N'…'` (MS-004), và `GROUP BY` không nhận số thứ tự cột như `GROUP BY 1`, nên câu đặt `CASE` trong một bảng dẫn xuất (subquery ở `FROM`) rồi nhóm theo tên cột. Nhánh `ELSE` là câu kiểm tra miễn phí: hễ nhóm "SAI QUY TẮC" xuất hiện là có chuyến đã nhận mà mất tài xế. Hôm nay nhóm này vắng mặt nhờ CHECK của MS-005.

### Bốn trạng thái đánh giá sau LEFT JOIN

**`LEFT JOIN`** giữ mọi chuyến bên trái, kể cả chuyến không có dòng đánh giá; khi đó các cột của `trip_ratings` là NULL ([MS-013](/articles/mssql-join-apply-cardinality) nói kỹ). Vậy `r.stars` NULL có hai nguyên nhân: không có dòng đánh giá, hoặc có dòng mà khách bỏ trống sao. Tách bằng một cột **không bao giờ NULL** khi dòng tồn tại, ở đây là khoá chính `r.trip_id` (câu 3 của lab):

```sql
CASE
  WHEN t.status = 'cancelled' THEN N'không áp dụng — chuyến huỷ'
  WHEN r.trip_id IS NULL      THEN N'thiếu — chưa đánh giá'
  WHEN r.stars IS NULL        THEN N'thiếu — mở màn đánh giá, bỏ trống sao'
  ELSE N'có sao'
END
```

```text
tinh_trang_danh_gia|so_chuyen
-------------------|---------
có sao|1738
thiếu — chưa đánh giá|816
không áp dụng — chuyến huỷ|461
thiếu — mở màn đánh giá, bỏ trống sao|153
```

Bốn nhóm cộng lại 3.168, đúng số chuyến đã kết thúc. Chỉ nhìn `r.stars IS NULL` thì 1.430 chuyến cần ba cách xử lý khác nhau bị gộp làm một.

### Hai bẫy của CASE

- **Thứ tự nhánh.** Đưa nhánh `status = 'cancelled'` lên đầu câu `driver_id` ở trên thì nhóm "huỷ trước khi có người nhận" phình từ 295 lên **461** (lab kiểm cả hai số): 166 chuyến huỷ *sau* khi đã có tài xế bị nhánh đầu lấy mất. Nhánh hẹp đặt trước, và tổng các nhóm phải khớp tổng số dòng.
- **Dạng CASE đơn giản không bắt được NULL.** `CASE stars WHEN NULL THEN 1 ELSE 0 END` so sánh `stars = NULL`, ra UNKNOWN, nên đếm được **0** lượt; `CASE WHEN stars IS NULL THEN 1 ELSE 0 END` đếm đúng **153**.

**`IIF(điều_kiện, a, b)`** là cách viết gọn của `CASE` hai nhánh, có từ SQL Server 2012. Điều kiện ra UNKNOWN rơi vào nhánh `b`, giống `ELSE`: `SUM(IIF(stars >= 4, 0, 1))` ra **456**, tức 303 lượt dưới 4 sao **cộng** 153 lượt không có sao. Nếu ý định là "lượt bị chấm thấp" thì câu đó đếm nhầm; viết điều kiện theo nhóm bạn muốn đếm (`IIF(stars < 4, 1, 0)`).

## 4. Hàm tổng hợp bỏ qua NULL, và AVG của số nguyên

`COUNT(cột)`, `SUM`, `AVG`, `MIN`, `MAX` **bỏ qua đầu vào NULL**; `COUNT(*)` đếm dòng nên không quan tâm NULL. Câu 4 của lab đặt ba bản báo cáo cạnh nhau:

```sql
SELECT COUNT(*)                                                            AS so_danh_gia,
       COUNT(stars)                                                        AS co_sao,
       AVG(stars)                                                          AS avg_int,
       CAST(AVG(CAST(stars AS decimal(9, 4))) AS decimal(4, 2))            AS diem_tb_dung,
       CAST(AVG(CAST(ISNULL(stars, 0) AS decimal(9, 4))) AS decimal(4, 2)) AS diem_tb_sai
FROM toinoi.trip_ratings;
```

```text
so_danh_gia|co_sao|avg_int|diem_tb_dung|diem_tb_sai
-----------|------|-------|------------|-----------
1891|1738|4|4.22|3.88
Warning: Null value is eliminated by an aggregate or other SET operation.
```

**Bản "4".** Tài liệu `AVG` ghi kiểu trả về theo kiểu đầu vào: nhóm số nguyên (`tinyint`, `smallint`, `int`) trả **`int`**. `stars` là `tinyint`, nên `AVG(stars)` tính 4,2186… rồi **cắt** phần lẻ thành 4 (lab kiểm `SQL_VARIANT_PROPERTY(AVG(stars), 'BaseType')` = `int`). Không có lỗi hay cảnh báo nào về chuyện cắt. Muốn số lẻ, đổi đầu vào sang `decimal` trước khi tính trung bình, rồi `CAST` kết quả về số chữ số cần hiển thị.

**Bản "3,88".** `ISNULL(stars, 0)` (mục 7) "cho khỏi NULL": biến 153 lượt "không biết" thành 0 sao, một mức điểm không tồn tại, rồi chia cho 1.891. Bản 4,22 chia cho 1.738 lượt có sao, đúng định nghĩa.

**Dòng Warning.** Khi hàm tổng hợp bỏ qua NULL và `ANSI_WARNINGS` đang ON (mặc định của sqlcmd và các driver), SQL Server gửi kèm thông điệp "Null value is eliminated by an aggregate or other SET operation". Đó chỉ là thông báo, không phải lỗi; trong DBStudio nó nằm ở tab **Thông điệp**. Thông điệp này có ích: nó nhắc rằng mẫu số của `AVG` nhỏ hơn số dòng.

Cũng vì NULL, ba điều kiện `stars = 5`, `stars <> 5` và `stars IS NULL` mới phủ hết bảng:

```text
tong|nam_sao|khac_nam_sao|khong_ro
----|-------|------------|--------
1891|896|842|153
```

**Tập rỗng.** Trừ `COUNT`, hàm tổng hợp trả NULL khi không có dòng nào. Đà Nẵng ngày 20/10 có `COUNT(*) = 0` và `SUM(fare)` là NULL (lab kiểm). Với **tiền**, `ISNULL(SUM(fare), 0)` đúng nghĩa: không có chuyến thì doanh thu là 0 đ. Với **trung bình** thì không: cước trung bình của 0 chuyến không phải 0 đ.

## 5. Tip: NULL không phải 0, và NULL lây qua phép tính

Câu hỏi nghiệp vụ quyết định mẫu số:

```text
so_danh_gia|co_tra_loi_tip|chon_0d|tong_tip|tb_tren_nguoi_tra_loi|tb_tren_moi_luot|tb_moi_luot_avg_int
-----------|--------------|-------|--------|---------------------|----------------|-------------------
1891|1350|660|5565000|4122|2943|2942
```

"Khách có trả lời bước tip cho trung bình bao nhiêu" là `AVG(tip)` trên 1.350 người: 4.122 đ. "Mỗi lượt đánh giá mang về bao nhiêu tip" là `AVG(ISNULL(tip, 0))` trên 1.891 lượt, với giả định ghi rõ "bỏ qua = không tip": 2.942,89 đ, làm tròn 2.943 đ. Cột cuối cho thấy `AVG` trên `int` lại **cắt** thành 2.942 đ. Lệch 1 đ trông vô hại, cho tới khi con số đó được nhân với số lượt của cả năm.

Trong biểu thức thường, NULL **lây**: chuyến 100003 cước 50.000 đ, bỏ qua tip, thì `fare + tip` là NULL. Ghép với quy tắc "`SUM` bỏ NULL", `SUM(fare + tip)` bỏ luôn **cả cước** của 541 lượt bỏ qua tip:

```sql
SELECT SUM(t.fare) + SUM(r.tip)          AS cong_hai_tong,
       SUM(t.fare + r.tip)               AS tong_cua_tong_dong,
       SUM(t.fare + ISNULL(r.tip, 0))    AS tong_dong_isnull
FROM toinoi.trips AS t
JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id;
```

```text
cong_hai_tong|tong_cua_tong_dong|tong_dong_isnull
-------------|------------------|----------------
134142000|98163000|134142000
```

Thiếu gần 36 triệu đồng, chỉ kèm dòng Warning ở mục 4. Ở đây `ISNULL(r.tip, 0)` đúng nghĩa vì câu hỏi là **tiền**: bỏ qua tip thì tài xế nhận 0 đ tip. Cùng một hàm, đúng với tiền, sai với điểm sao; điều quyết định là nghĩa của con số.

## 6. NULLIF và phép chia cho 0

**`NULLIF(a, b)`** trả NULL nếu `a = b`, ngược lại trả `a`, tức chiều ngược của `ISNULL`/`COALESCE`.

**Cước 0 đ.** 31 chuyến hoàn thành có cước 0 đ (khuyến mãi), là dữ liệu đúng. Cước trung bình trên 2.707 chuyến hoàn thành là **67.339 đ**; "cước trung bình của chuyến có thu tiền" là `AVG(CAST(NULLIF(fare, 0) AS decimal(12, 2)))`, làm tròn **68.119 đ**. Dữ liệu lưu số thật (0 đ); việc loại hay không quyết định ở câu truy vấn.

**Chống chia cho 0.** Câu tỉ lệ huỷ ngày 20/10 đi từ `cities`, `LEFT JOIN trips` để thành phố không có chuyến vẫn có dòng, rồi chia cho `COUNT(t.id)`, với Đà Nẵng là 0:

```text
PASS 10 tỉ lệ huỷ 20/10 chia cho 0 — lỗi 8134: Divide by zero error encountered.
```

Lỗi 8134 làm hỏng **cả câu**, mất luôn số của Hà Nội và TP. HCM. Sửa bằng `NULLIF(mẫu_số, 0)`, đồng thời nhân với `100.0` thay vì `100`:

```sql
SELECT c.code,
       COUNT(t.id) AS tong,
       100 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / NULLIF(COUNT(t.id), 0) AS pct_chia_nguyen,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END)
            / NULLIF(COUNT(t.id), 0) AS decimal(4, 1))                                      AS ty_le_huy_pct,
       COALESCE(CONVERT(nvarchar(10), /* cùng biểu thức ty_le_huy_pct */), N'không có chuyến') AS hien_thi
FROM toinoi.cities AS c
LEFT JOIN toinoi.trips AS t
  ON t.city_id = c.id
 AND t.requested_at >= '2026-10-20 00:00 +07:00' AND t.requested_at < '2026-10-21 00:00 +07:00'
GROUP BY c.code
ORDER BY c.code;
```

```text
code|tong|pct_chia_nguyen|ty_le_huy_pct|hien_thi
----|----|---------------|-------------|--------
DN|0|NULL|NULL|không có chuyến
HN|37|13|13.5|13.5
SG|49|14|14.3|14.3
```

Cột `pct_chia_nguyen` cho thấy bẫy thứ hai: `int / int` là **phép chia nguyên**, 500 / 37 ra 13 chứ không phải 13,5. Hằng `100.0` có kiểu `decimal`, kéo cả phép tính sang `decimal`. Cột tỉ lệ giữ NULL để biểu đồ và cảnh báo phía sau biết đó là "không xác định"; chỉ cột `hien_thi` mới dịch NULL thành chữ cho người đọc. Cách sửa vội `ISNULL(…, 0)` biến một ngày không có dữ liệu thành khẳng định "Đà Nẵng không huỷ chuyến nào", và cảnh báo tỉ lệ huỷ bất thường không bao giờ chạy cho ngày đó.

Một biến thể nguy hiểm hơn: mã cũ đặt `SET ANSI_WARNINGS OFF` và `SET ARITHABORT OFF` thì `100 / 0` không lỗi mà ra NULL, kèm dòng "Division by zero occurred." (lab kiểm). Lỗi bị giấu, và bạn không phân biệt được NULL do chia cho 0 với NULL có chủ đích. Giữ hai tuỳ chọn ở ON (mặc định) và viết `NULLIF` ở đúng chỗ cần.

## 7. ISNULL và COALESCE không giống nhau

**`ISNULL(a, b)`** là hàm riêng của T-SQL: trả `a` nếu `a` khác NULL, ngược lại trả `b`. **`COALESCE(a, b, …)`** là hàm chuẩn SQL: trả đối số **đầu tiên khác NULL**. Với hai đối số cùng kiểu, hai hàm cho cùng kết quả, nên nhiều người dùng lẫn. Bốn khác biệt sau đều đã chạy trong mục 7 của lab.

**(a) Kiểu trả về.** Tài liệu `ISNULL` ghi kết quả có **kiểu của đối số đầu**; `COALESCE` lấy kiểu có **thứ tự ưu tiên kiểu dữ liệu** (data type precedence) cao nhất trong các đối số. Đây là nguồn gốc của chữ `không r? (m?` trên màn hình CSKH:

```sql
SELECT t.code,
       ISNULL(z.code, N'không rõ (mất GPS)')   AS khu_isnull,
       COALESCE(z.code, N'không rõ (mất GPS)') AS khu_coalesce,
       …                                       -- kiểu của hai cột, qua SQL_VARIANT_PROPERTY
FROM toinoi.trips AS t
LEFT JOIN toinoi.areas AS z ON z.id = t.pickup_area_id
WHERE t.code = (SELECT MIN(code) FROM toinoi.trips WHERE pickup_area_id IS NULL);
```

```text
code|khu_isnull|khu_coalesce|kieu_isnull|kieu_coalesce
----|----------|------------|-----------|-------------
100072|không r? (m?|không rõ (mất GPS)|varchar|nvarchar
```

`areas.code` là `varchar(12)`. `ISNULL` ép chuỗi thay thế về đúng `varchar(12)`: cắt còn 12 ký tự, và ký tự nào không có trong bảng mã của `varchar` (theo collation `Vietnamese_100_CI_AS`) bị thay bằng `?`. Không có lỗi, không có cảnh báo cắt chuỗi. `COALESCE` chọn `nvarchar` vì kiểu này đứng trên `varchar` trong thứ tự ưu tiên, nên giữ nguyên chuỗi.

**(b) Chữ thay cho số.** Ngược lại, đặt chữ vào chỗ trống của một cột số thì **cả hai** đều lỗi:

```text
PASS 7b ISNULL(d.code, chữ) — lỗi 245: Conversion failed when converting the nvarchar value 'đang tìm…' to data type int.
PASS 7b COALESCE(d.code, chữ) — lỗi 245: Conversion failed when converting the nvarchar value 'đang tìm…' to data type int.
```

`ISNULL` ép về `int` vì `drivers.code` là đối số đầu; `COALESCE` cũng chọn `int` vì `int` đứng trên `nvarchar`. Lỗi chỉ xuất hiện khi gặp dòng có `d.code` NULL, nên câu có thể chạy đúng hàng tháng rồi hỏng vào ngày có chuyến chưa có tài xế. Đổi số sang chữ trước: `COALESCE(CONVERT(nvarchar(10), d.code), N'đang tìm…')`.

**(c) Tính NULL của cột kết quả.** `SELECT … INTO` một bảng tạm rồi đọc `tempdb.sys.columns`:

```text
cot|is_nullable
---|-----------
tip_isnull|0
tip_coalesce|1
```

`ISNULL(tip, 0)` tạo cột `NOT NULL`; `COALESCE(tip, 0)` tạo cột cho phép NULL. Khác biệt này quan trọng khi biểu thức làm computed column cần đưa vào khoá chính, hoặc khi bạn dựa vào `SELECT INTO` để tạo bảng.

**(d) Số đối số và số lần tính.** `ISNULL` nhận đúng hai đối số (ba đối số là lỗi 174 "The isnull function requires 2 argument(s)."); `COALESCE` nhận nhiều. Tài liệu `COALESCE` còn ghi hàm này được viết lại thành một biểu thức `CASE`, nên một đối số là subquery có thể bị **tính hai lần**, và dưới tải đồng thời, hai lần tính có thể cho hai kết quả khác nhau; `ISNULL` chỉ tính một lần.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Quy tắc chọn của Tới Nơi</span>
<p>Mặc định dùng <code>COALESCE</code> (chuẩn SQL, kiểu kết quả theo thứ tự ưu tiên, nhiều đối số). Dùng <code>ISNULL</code> khi cần cột kết quả <code>NOT NULL</code> hoặc khi đối số là subquery tốn kém, và khi đó kiểm kiểu của đối số đầu đủ rộng cho giá trị thay thế. Hễ giá trị thay thế khác kiểu cột, đổi kiểu tường minh bằng <code>CONVERT</code>.</p>
</div>

## 8. COALESCE để hiển thị, và IS DISTINCT FROM

Chỗ dùng hợp lý nhất của `COALESCE` là tầng hiển thị. Câu 8 của lab gắn nghĩa cho từng ô trống:

```sql
SELECT t.code, t.kind, t.status,
       COALESCE(m.name, CASE WHEN t.kind = 'food' THEN N'LỖI: đơn đồ ăn thiếu quán'
                             ELSE N'— (không áp dụng)' END)              AS quan,
       COALESCE(z.code, N'không rõ (mất GPS)')                           AS khu_don,
       COALESCE(CONVERT(nvarchar(10), d.code),
                CASE WHEN t.status IN ('requested', 'searching') THEN N'đang tìm…'
                     ELSE N'— (huỷ trước khi nhận)' END)                 AS tai_xe
FROM toinoi.trips AS t
LEFT JOIN toinoi.merchants AS m ON m.id = t.merchant_id
LEFT JOIN toinoi.areas     AS z ON z.id = t.pickup_area_id
LEFT JOIN toinoi.drivers   AS d ON d.id = t.driver_id
WHERE …   -- năm chuyến mẫu, xem tệp lab
ORDER BY t.code;
```

```text
code|kind|status|quan|khu_don|tai_xe
----|----|------|----|-------|------
100001|food|cancelled|Mì Quảng Lab 207|HN-TA-01|— (huỷ trước khi nhận)
100002|food|completed|Bún Chả Lab 106|SG-DO-03|8117
100007|ride|completed|— (không áp dụng)|HN-BA-02|8217
100072|parcel|completed|— (không áp dụng)|không rõ (mất GPS)|8110
103172|food|searching|Gà Rán Lab 205|HN-TT-03|đang tìm…
```

Đối số thứ hai có thể là cả một `CASE`, nên cùng một `driver_id` NULL hiện thành "đang tìm…" hay "huỷ trước khi nhận" tuỳ trạng thái. Quy tắc của bài: cột để **tính tiếp** giữ NULL, chỉ cột để **đọc** mới lấp chữ.

### `<>` cũng bỏ rơi NULL

"Bao nhiêu chuyến **không** đón ở khu SG-TT-01 (id 111)?"

```text
tong|o_111|khac_111|khac_biet_111
----|-----|--------|-------------
3174|162|2976|3012
```

162 + 2.976 thiếu đúng 36 chuyến mất GPS. **`IS DISTINCT FROM`**, mới có ở SQL Server 2022, so sánh như `<>` nhưng coi NULL là một giá trị; dạng phủ định `IS NOT DISTINCT FROM` so sánh như `=` mà coi hai NULL là bằng nhau. "Chắc chắn đón ngoài SG-TT-01" là 2.976, "không chắc đón ở SG-TT-01" là 3.012. Chọn câu nào tuỳ nghiệp vụ, miễn là chọn có chủ đích. Trước 2022, người ta viết `ISNULL(pickup_area_id, -1) <> 111`, cũng ra 3.012 nhưng phụ thuộc vào việc `-1` không bao giờ là một id thật, và bọc cột trong hàm làm điều kiện mất tính SARGable (MS-007). Bẫy cùng loại là `NOT IN (danh sách có NULL)` trả 0 dòng, đã gặp ở MS-007 và sẽ quay lại ở [MS-014](/articles/mssql-subquery-exists-cte-temp-table).

## 9. Làm trong DBStudio

- `Ctrl+Enter` chạy batch đang chứa con trỏ, `Ctrl+Shift+Enter` chạy cả tệp; các dòng `PASS …` và cảnh báo "Null value is eliminated…" nằm ở tab **Thông điệp**, không nằm trong lưới kết quả. Đọc tab này mỗi khi một con số trung bình trông lạ.
- Khi xuất kết quả ra CSV hay XLSX để gửi kế toán, xuất cột đã dịch nghĩa (như `hien_thi` ở mục 6) bên cạnh cột số giữ NULL: người nhận đọc được lý do, còn bảng tính vẫn cộng đúng cột số.

## 10. So với PostgreSQL

Cùng câu hỏi, cùng dữ liệu với [PG-011](/articles/postgresql-null-case-coalesce); những chỗ khác nhau thật sự:

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| `AVG` của cột số nguyên | trả `numeric`, giữ phần lẻ (4,22) | trả `int`, **cắt** phần lẻ (4) |
| Hàm thay NULL | `coalesce` | `COALESCE` và `ISNULL` (kiểu theo đối số đầu, cột kết quả NOT NULL) |
| So sánh coi NULL là giá trị | `IS [NOT] DISTINCT FROM` từ lâu | `IS [NOT] DISTINCT FROM` từ 2022 |
| Đếm có điều kiện | `count(*) FILTER (WHERE …)` | `SUM(CASE WHEN … THEN 1 ELSE 0 END)` |
| Chia cho 0 | lỗi `22012` | lỗi 8134 (hoặc NULL nếu tắt `ANSI_WARNINGS` và `ARITHABORT`) |
| Hàm tổng hợp bỏ NULL | không báo gì | thông điệp "Null value is eliminated…" khi `ANSI_WARNINGS` ON |
| Chữ vào cột số | `coalesce(d.code, 'đang tìm…')` lỗi kiểu | lỗi 245, với cả `ISNULL` lẫn `COALESCE` |

## 11. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| `WHERE x = NULL` luôn 0 dòng | so sánh với NULL ra UNKNOWN | `IS NULL` |
| Điểm trung bình là số tròn (4) | `AVG` trên `tinyint`/`int` trả `int` | `AVG(CAST(x AS decimal(…)))` |
| Điểm trung bình tụt | `AVG(ISNULL(stars, 0))` | `AVG` trên cột gốc kèm `COUNT(stars)` |
| Nhóm "bằng" + "khác" < tổng | `=`, `<>` loại NULL | thêm nhóm `IS NULL` hoặc `IS DISTINCT FROM` |
| `CASE x WHEN NULL` không khớp | CASE đơn giản dùng `=` | `CASE WHEN x IS NULL` |
| `SUM(a + b)` thiếu tiền | NULL lây qua `+` | `SUM(a) + ISNULL(SUM(b), 0)` |
| Lỗi 8134 mất cả bảng | mẫu số 0 ở một nhóm | `NULLIF(mẫu_số, 0)` |
| Tỉ lệ ra 13 thay vì 13,5 | `int / int` chia nguyên | nhân `100.0` trước khi chia |
| Chữ hiển thị bị cắt, có dấu `?` | `ISNULL` lấy kiểu `varchar(n)` của đối số đầu | `COALESCE`, hoặc `CONVERT` đối số đầu sang `nvarchar` đủ dài |
| Lỗi 245 khi gặp dòng NULL | lấp chữ vào cột số | `CONVERT(nvarchar(…), cột)` trước |

## 12. Đánh đổi

- **NULL hay giá trị giả (`-1`, `''`)?** Giá trị giả giúp `=` "chạy được" nhưng bị `AVG`, `SUM`, `MIN` tính vào, và CHECK phải nới. NULL buộc xử lý rõ ràng; giá trị giả cho phép quên.
- **Một NULL hay thêm cột lý do?** Khi "thiếu" và "không áp dụng" chung một cột mà không suy được từ cột khác, một cột lý do (ví dụ `tip_status`) đáng giá hơn quy ước ngầm. Ở fixture này, `status` và CHECK đủ để `CASE` suy ra.
- **`ISNULL` hay `COALESCE`?** `ISNULL` ngắn, tính một lần, cho cột NOT NULL, nhưng kiểu kết quả theo đối số đầu là cái bẫy với chuỗi. `COALESCE` chuẩn và an toàn về kiểu hơn, nhưng có thể tính subquery hai lần.
- **`tip NOT NULL DEFAULT 0`?** Dễ viết báo cáo, nhưng mất vĩnh viễn khác biệt giữa 660 khách chọn "không tip" và 541 khách bỏ qua, đúng thứ đội sản phẩm muốn đo.

## 13. Bài tập

Lời giải đã chạy trong phần "Bài tập" ở cuối `11-null-case-isnull-coalesce.sql` (nền như mục 1).

**Bài 1.** Viết `CASE` dịch `trips.merchant_id` thành "có quán", "không áp dụng — không phải đơn đồ ăn", "LỖI — đơn đồ ăn thiếu quán" rồi đếm. Dự đoán số dòng nhóm LỖI.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT y_nghia_merchant_id, COUNT(*) AS so_chuyen
FROM (SELECT CASE
               WHEN merchant_id IS NOT NULL THEN N'có quán'
               WHEN kind &lt;&gt; 'food'          THEN N'không áp dụng — không phải đơn đồ ăn'
               ELSE N'LỖI — đơn đồ ăn thiếu quán'
             END AS y_nghia_merchant_id
      FROM toinoi.trips) AS x
GROUP BY y_nghia_merchant_id
ORDER BY so_chuyen DESC;</code></pre>
<p>Kết quả đã chạy: "không áp dụng" 1.742, "có quán" 1.432; nhóm LỖI không xuất hiện vì <code>ck_trips_merchant_only_food</code> chặn từ lúc ghi. Nhánh <code>ELSE</code> vẫn có ích khi chạy câu này trên dữ liệu chép từ hệ thống cũ chưa có ràng buộc.</p>
</details>

**Bài 2.** Kế toán cần "cước + tip" của chuyến hoàn thành tháng 10 (đặt trong `[01/10, 31/10)` giờ VN) theo thành phố. Một bạn viết `SUM(t.fare + r.tip)` với `LEFT JOIN trip_ratings`. Sai ở đâu?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT c.code AS tp,
       SUM(t.fare + r.tip)                  AS sai_cong_trong_sum,
       SUM(t.fare) + ISNULL(SUM(r.tip), 0)  AS cuoc_cong_tip
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at &gt;= '2026-10-01 00:00 +07:00' AND t.requested_at &lt; '2026-10-31 00:00 +07:00'
GROUP BY c.code
ORDER BY c.code;</code></pre>
<p>Kết quả đã chạy (sai / đúng): DN 13.866.000 / <b>27.221.000</b>, HN 34.957.000 / <b>64.981.000</b>, SG 49.340.000 / <b>95.650.000</b>. Cột sai mất gần một nửa: <code>r.tip</code> NULL ở cả chuyến không có dòng đánh giá (LEFT JOIN) lẫn chuyến bỏ qua tip, và <code>fare + NULL</code> kéo cả cước ra khỏi <code>SUM</code>. <code>ISNULL</code> bọc ngoài <code>SUM(r.tip)</code> lo trường hợp cả nhóm không có tip nào.</p>
</details>

**Bài 3.** Lập bảng ngày 20/10: mỗi thành phố một dòng (kể cả Đà Nẵng), gồm số chuyến hoàn thành, số lượt có sao, điểm trung bình hai chữ số lẻ và một cột hiển thị không bao giờ ghi số 0 giả. Câu có bị lỗi 8134 không?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT c.code AS tp, COUNT(t.id) AS hoan_thanh, COUNT(r.stars) AS co_sao,
       CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2)) AS diem_tb,
       COALESCE(CONVERT(nvarchar(10), CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2))),
                CASE WHEN COUNT(t.id) = 0 THEN N'không có chuyến'
                     ELSE N'chưa có lượt chấm sao' END) AS hien_thi
FROM toinoi.cities AS c
LEFT JOIN toinoi.trips AS t
  ON t.city_id = c.id AND t.status = 'completed'
 AND t.requested_at &gt;= '2026-10-20 00:00 +07:00' AND t.requested_at &lt; '2026-10-21 00:00 +07:00'
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
GROUP BY c.code
ORDER BY c.code;</code></pre>
<p>Kết quả đã chạy: <code>DN|0|0|NULL|không có chuyến</code>, <code>HN|32|22|4.23|4.23</code>, <code>SG|42|28|4.39|4.39</code>. Không lỗi: <code>AVG</code> trên tập rỗng trả NULL; chỉ phép chia tự viết mới cần <code>NULLIF</code>. Hai chỗ dễ sai: quên <code>CAST</code> sang <code>decimal</code> thì HN và SG cùng ra 4; đưa điều kiện của <code>trips</code> xuống <code>WHERE</code> thì dòng Đà Nẵng bị loại (MS-013).</p>
</details>

## 14. Đọc tiếp

- Microsoft Learn: [NULL and UNKNOWN](https://learn.microsoft.com/sql/t-sql/language-elements/null-and-unknown-transact-sql), [CASE](https://learn.microsoft.com/sql/t-sql/language-elements/case-transact-sql), [IIF](https://learn.microsoft.com/sql/t-sql/functions/logical-functions-iif-transact-sql), [ISNULL](https://learn.microsoft.com/sql/t-sql/functions/isnull-transact-sql), [COALESCE](https://learn.microsoft.com/sql/t-sql/language-elements/coalesce-transact-sql) (mục so sánh với ISNULL), [NULLIF](https://learn.microsoft.com/sql/t-sql/language-elements/nullif-transact-sql), [IS [NOT] DISTINCT FROM](https://learn.microsoft.com/sql/t-sql/queries/is-distinct-from-transact-sql), [AVG](https://learn.microsoft.com/sql/t-sql/functions/avg-transact-sql) (bảng kiểu trả về), [Data type precedence](https://learn.microsoft.com/sql/t-sql/data-types/data-type-precedence-transact-sql), [SET ANSI_NULLS](https://learn.microsoft.com/sql/t-sql/statements/set-ansi-nulls-transact-sql), [SET ANSI_WARNINGS](https://learn.microsoft.com/sql/t-sql/statements/set-ansi-warnings-transact-sql).
- Cùng chủ đề trên PostgreSQL: [PG-011 · NULL, CASE và COALESCE trong dữ liệu chuyến](/articles/postgresql-null-case-coalesce).
- Lab: `examples/toi-noi-mssql/p02/11-null-case-isnull-coalesce.sql`.

Bài tiếp theo: [MS-012 · Tổng hợp: GROUP BY, HAVING, ROLLUP và GROUPING SETS](/articles/mssql-group-by-rollup-grouping-sets)
