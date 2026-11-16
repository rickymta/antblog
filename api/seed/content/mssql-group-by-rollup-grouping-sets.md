---
slug: mssql-group-by-rollup-grouping-sets
title: "Tổng hợp: GROUP BY, HAVING, ROLLUP và GROUPING SETS cho báo cáo tháng của Tới Nơi"
summary: "Báo cáo tháng 10 của Tới Nơi ghi tỉ lệ huỷ TP. HCM 14 %, cước trung bình 68.269 đ và 1.103 khách hoạt động; job dự phóng doanh thu năm dừng với lỗi 8115. Bài đi qua hàm tổng hợp, GROUP BY, HAVING, ROLLUP, CUBE, GROUPING SETS và STRING_AGG trên SQL Server 2022 để mỗi con số có mẫu số rõ ràng và dòng tổng đối chiếu được."
category: mssql
level: beginner
tags: [ms-012, mssql, sql-server-2022, t-sql, group-by, having, rollup, grouping-sets, aggregate, toi-noi]
language: vi
---

Ngày 31/10, Marketing gửi ban điều hành bảng tổng kết tháng 10 của Tới Nơi: tỉ lệ huỷ ở TP. HCM **14 %**, cước trung bình **68.269 đ**, **1.103** khách hoạt động. Cả ba con số đều lệch, dù câu T-SQL nào cũng chạy không lỗi. Cùng ngày, job "dự phóng doanh thu năm = tháng 10 × 12" của bộ phận tài chính dừng với `Arithmetic overflow error converting expression to data type int`, còn bảng tổng hợp xuất ra cho kế toán có **hai** dòng "TỔNG" với hai con số khác nhau.

Lỗi không nằm ở cú pháp mà ở **mẫu số** (chia cho cái gì), ở **kiểu dữ liệu** của kết quả tổng hợp và ở cách đọc dòng tổng. Bài này đi qua hàm tổng hợp, `GROUP BY`, `HAVING`, rồi `ROLLUP`, `CUBE`, `GROUPING SETS` để một câu lệnh trả cả chi tiết lẫn tổng phụ, và mỗi con số đi kèm mẫu số của nó.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-012 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/mssql-null-case-isnull-coalesce">MS-011 · NULL, CASE, ISNULL và COALESCE</a> (hàm tổng hợp bỏ NULL, <code>AVG</code> của số nguyên), <a href="/articles/mssql-select-loc-sap-xep-top-offset">MS-007 · SELECT, WHERE, ORDER BY</a>. Lab: <code>examples/toi-noi-mssql/p02</code> (tệp <code>12-group-by-rollup.sql</code>). Sau bài này, bạn lập được báo cáo theo thành phố và dịch vụ có dòng tổng, ghi rõ mẫu số của từng chỉ số và đối chiếu được tổng với một truy vấn độc lập.</p>
</div>

## 1. Dữ liệu và định nghĩa chỉ số

Bài dùng fixture Phần 2 như [MS-011](/articles/mssql-null-case-isnull-coalesce): 3.174 chuyến, ảnh chụp 01:00 ngày 31/10/2026 giờ VN.

```bash
cd examples/toi-noi-mssql
node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/12-group-by-rollup.sql
```

Kết quả chép từ lần chạy trên SQL Server 2022 16.0.4325.4 (RTM-CU28); tệp lab có 37 kiểm tra, tất cả PASS. Trước khi viết SQL, hãy viết **định nghĩa chỉ số**, tức một data dictionary cho số liệu:

| Chỉ số | Định nghĩa | Mẫu số |
|---|---|---|
| Kỳ báo cáo | chuyến **đặt** trong `[01/10 00:00, 31/10 00:00)` giờ VN | — |
| Đã kết thúc | `completed` + `cancelled` trong kỳ | — |
| Tỉ lệ huỷ | huỷ ÷ đã kết thúc | đã kết thúc |
| Doanh thu | Σ `fare` chuyến `completed` (chưa đối chiếu tiền thật, việc của MS-020) | — |
| Cước trung bình | doanh thu ÷ chuyến hoàn thành | hoàn thành |
| Điểm sao trung bình | trung bình `stars` khác NULL | lượt có sao |

Sáu chuyến đặt sau nửa đêm 31/10 còn đang chạy nên nằm ngoài kỳ. Kỳ có **3.168 dòng**, cả 3.168 đã kết thúc: đó là con số mà mọi bảng trong bài phải cộng lại ra.

## 2. Hàm tổng hợp và kiểu kết quả

**Hàm tổng hợp** (aggregate function) nhận một tập dòng, trả một giá trị: `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`. Riêng `COUNT` có ba dạng cho ba câu hỏi:

```sql
SELECT COUNT(*)                    AS so_chuyen,
       COUNT(driver_id)            AS co_tai_xe,
       COUNT(merchant_id)          AS co_quan,
       COUNT(DISTINCT driver_id)   AS so_tai_xe_khac_nhau,
       COUNT(DISTINCT customer_id) AS so_khach_khac_nhau,
       COUNT_BIG(*)                AS so_chuyen_big
FROM toinoi.trips;
```

```text
so_chuyen|co_tai_xe|co_quan|so_tai_xe_khac_nhau|so_khach_khac_nhau|so_chuyen_big
---------|---------|-------|-------------------|------------------|-------------
3174|2877|1432|52|576|3174
Warning: Null value is eliminated by an aggregate or other SET operation.
```

`COUNT(*)` đếm **dòng**; `COUNT(cột)` đếm giá trị **khác NULL** (và kéo theo dòng Warning của MS-011); `COUNT(DISTINCT cột)` đếm giá trị **khác nhau**, bỏ NULL. Lab có 54 tài xế nhưng chỉ 52 từng chạy: 8219 và 8220 chưa có chuyến nào. **`COUNT_BIG`** giống `COUNT` nhưng trả `bigint`; dùng khi bảng có thể vượt khoảng 2,1 tỉ dòng.

**Kiểu kết quả là chỗ SQL Server khác PostgreSQL nhiều nhất.** Lab đọc kiểu bằng `SQL_VARIANT_PROPERTY(…, 'BaseType')`:

```text
kieu_sum|kieu_avg|kieu_count|kieu_count_big
--------|--------|----------|--------------
int|int|int|bigint
```

`fare` là `int`, và theo tài liệu `SUM`/`AVG`, cả hai trả **`int`** cho đầu vào thuộc nhóm số nguyên. Hệ quả thứ nhất là `AVG` cắt phần lẻ (MS-011, và mục 5 dưới đây). Hệ quả thứ hai là `SUM` có thể **tràn**: `int` tối đa 2.147.483.647. Job dự phóng doanh thu năm ở đầu bài:

```text
PASS 1 SUM(fare * 12) tràn int — lỗi 8115: Arithmetic overflow error converting expression to data type int.
```

Doanh thu tháng 10 là 182.287.000 đ, nhân 12 vượt ngưỡng `int`. Không có kết quả sai, chỉ có lỗi, nhưng lỗi đến vào một ngày bất kỳ khi dữ liệu đủ lớn. Đổi kiểu **trước khi** cộng: `SUM(CAST(fare AS bigint) * 12)` ra 2.187.444.000. Với tổng tiền trên bảng sẽ còn lớn lên (cả năm, nhiều năm), viết `SUM(CAST(fare AS bigint))` ngay từ đầu.

## 3. GROUP BY: từ dòng thành nhóm

**`GROUP BY`** chia các dòng thành nhóm có cùng giá trị ở các cột liệt kê, rồi hàm tổng hợp tính trên từng nhóm. Bảng chính của tháng là thành phố × dịch vụ, mỗi cột ghi rõ mẫu số trong chú thích:

```sql
SELECT c.code                                                       AS tp,
       t.kind,
       COUNT(*)                                                     AS da_ket_thuc,
       SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END)      AS hoan_thanh,
       SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END)      AS huy,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END)
            / COUNT(*) AS decimal(4, 1))                            AS ty_le_huy_pct,   -- huỷ / đã kết thúc
       SUM(CASE WHEN t.status = 'completed' THEN t.fare END)        AS doanh_thu,
       CAST(ROUND(AVG(CASE WHEN t.status = 'completed' THEN CAST(t.fare AS decimal(12, 2)) END), 0)
            AS int)                                                 AS cuoc_tb          -- / hoàn thành
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY c.code, t.kind
ORDER BY c.code, t.kind;
```

```text
tp|kind|da_ket_thuc|hoan_thanh|huy|ty_le_huy_pct|doanh_thu|cuoc_tb
--|----|-----------|----------|---|-------------|---------|-------
DN|food|210|173|37|17.6|14902000|86139
DN|parcel|72|68|4|5.6|3380000|49706
DN|ride|177|148|29|16.4|8134000|54959
HN|food|503|413|90|17.9|35025000|84806
HN|parcel|165|145|20|12.1|7041000|48559
HN|ride|438|383|55|12.6|20925000|54634
SG|food|717|615|102|14.2|52898000|86013
SG|parcel|251|196|55|21.9|9290000|47398
SG|ride|635|566|69|10.9|30692000|54226
```

SQL Server không có `FILTER (WHERE …)` như PostgreSQL, nên phép đếm có điều kiện viết `SUM(CASE WHEN … THEN 1 ELSE 0 END)`. Với tổng tiền và trung bình, `CASE` không có `ELSE` trả NULL cho dòng không thoả, và `SUM`/`AVG` bỏ NULL: `AVG(CASE WHEN status = 'completed' THEN … END)` chia cho đúng số chuyến hoàn thành. Một lần quét bảng cho ra sáu chỉ số, cùng một bộ lọc kỳ.

## 4. Thứ tự logic và lỗi 8120

Về logic, câu tổng hợp được hiểu theo thứ tự: `FROM` (và `JOIN`) → `WHERE` lọc **dòng** → `GROUP BY` gom nhóm → hàm tổng hợp tính trên nhóm → `HAVING` lọc **nhóm** → `SELECT` → `ORDER BY`. Sau `GROUP BY`, mỗi nhóm thành một dòng, nên mọi cột trong `SELECT` phải hoặc nằm trong `GROUP BY`, hoặc nằm trong hàm tổng hợp:

```text
PASS 4 cột không nhóm, không tổng hợp — lỗi 8120: Column 'toinoi.trips.kind' is invalid in the select list because it is not contained in either an aggregate function or the GROUP BY clause.
```

SQL Server áp quy tắc này **chặt hơn PostgreSQL**. Nhóm theo khoá chính `ci.id` rồi chọn `ci.code`, `ci.name` vẫn lỗi 8120, dù mỗi `id` chỉ có một tên (phụ thuộc hàm, MS-008):

```text
PASS 4 nhóm theo khoá chính vẫn lỗi — lỗi 8120: Column 'toinoi.cities.code' is invalid in the select list because it is not contained in either an aggregate function or the GROUP BY clause.
```

Cách sửa là liệt kê đủ: `GROUP BY ci.id, ci.code, ci.name`. Kết quả không đổi, vì thêm cột phụ thuộc vào khoá không tách nhóm nào ra:

```text
id|code|name|so_chuyen
--|----|----|---------
1|SG|TP. Hồ Chí Minh|1606
2|HN|Hà Nội|1108
3|DN|Đà Nẵng|460
```

Thứ tự logic cũng giải thích lỗi 207 khi `HAVING` dùng tên cột đặt ở `SELECT` (`HAVING so_chuyen > 500` báo "Invalid column name 'so_chuyen'"): `HAVING` chạy trước `SELECT`, nên phải viết lại biểu thức. `ORDER BY` chạy sau cùng nên dùng được tên đó.

## 5. Ba lỗi mẫu số, và AVG của số nguyên

Câu 3 của lab đặt các cách tính sai và đúng cạnh nhau, theo thành phố:

```text
tp|a_chia_nguyen|a_pct_nguyen|a_pct_dung|b_avg_int_moi_chuyen|b_cuoc_tb_sai|b_cuoc_tb_dung|c_pct_sai_mau
--|-------------|------------|----------|--------------------|-------------|--------------|-------------
DN|0|15|15.25|68010|68011|67907|17.99
HN|0|14|14.92|68348|68348|66940|17.53
SG|0|14|14.10|68269|68269|67451|16.41
```

- **(a) Chia nguyên.** `SUM(CASE …)` và `COUNT(*)` đều là `int`, nên `huỷ / đã_kết_thúc` là phép chia nguyên, ra 0. Nhân `100` trước thì ra 14, số "14 %" của Marketing: đúng phần nguyên, mất phần lẻ, và HN 14,92 % bị báo thành 14 %. Nhân `100.0` (hằng `decimal`) thì ra 14,10.
- **(b) Cước trung bình trên mọi chuyến.** `AVG(fare)` không lọc tính cả chuyến huỷ (khách chưa trả đồng nào): SG ra 68.269 đ thay vì **67.451 đ**. Cột `b_avg_int_moi_chuyen` thêm một lỗi riêng của SQL Server: `AVG` trên `int` **cắt** phần lẻ, Đà Nẵng 68.010,6 đ thành 68.010 thay vì làm tròn 68.011. Muốn làm tròn đúng, đổi sang `decimal` trước rồi `ROUND`.
- **(c) Sai mẫu số.** Chia số huỷ cho số **hoàn thành** thay vì số **đã kết thúc**: SG 16,41 % thay vì 14,10 %. Câu vẫn chạy, chỉ trả lời một câu hỏi khác.

Ba lỗi có chung một cách phòng: viết định nghĩa chỉ số ở mục 1 trước, rồi đặt tử số và mẫu số thành **cột riêng** trong kết quả (như `da_ket_thuc`, `huy` ở mục 3) để người đọc tự chia lại được.

## 6. WHERE lọc dòng, HAVING lọc nhóm

`WHERE` chạy trước khi gom, nên không chứa hàm tổng hợp; **`HAVING`** chạy sau khi gom, nên chứa được. Nhóm thành phố × dịch vụ có tỉ lệ huỷ trên 15 %:

```sql
SELECT c.code AS tp, t.kind, COUNT(*) AS da_ket_thuc,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(4, 1)) AS ty_le_huy_pct
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY c.code, t.kind
HAVING 100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) > 15
ORDER BY ty_le_huy_pct DESC, c.code, t.kind;
```

```text
tp|kind|da_ket_thuc|ty_le_huy_pct
--|----|-----------|-------------
SG|parcel|251|21.9
HN|food|503|17.9
DN|food|210|17.6
DN|ride|177|16.4
```

Câu thứ hai tìm tài xế hoàn thành từ 63 chuyến trở lên. Điều kiện trên **dòng** (`status = 'completed'`, kỳ báo cáo) ở `WHERE`; điều kiện trên **nhóm** (`COUNT(*) >= 63`) ở `HAVING`:

```text
tai_xe|tp|hoan_thanh|doanh_thu
------|--|----------|---------
8101|SG|68|4736000
8109|SG|68|4744000
8120|SG|66|4158000
8108|SG|64|4085000
8124|SG|64|4346000
8107|SG|63|4539000
8210|HN|63|4160000
```

Đội vận hành chỉ cần một dòng mỗi thành phố để dán vào tin nhắn. **`STRING_AGG(biểu_thức, dấu_phân_cách)`** (có từ SQL Server 2017) ghép các giá trị của nhóm thành một chuỗi, và `WITHIN GROUP (ORDER BY …)` cố định thứ tự ghép:

```sql
SELECT tp, COUNT(*) AS so_tai_xe,
       STRING_AGG(CONVERT(varchar(10), tai_xe), ', ') WITHIN GROUP (ORDER BY tai_xe) AS danh_sach
FROM (/* câu tài xế >= 63 chuyến ở trên */) AS x
GROUP BY tp
ORDER BY tp;
```

```text
tp|so_tai_xe|danh_sach
--|---------|---------
HN|1|8210
SG|6|8101, 8107, 8108, 8109, 8120, 8124
```

Thiếu `WITHIN GROUP`, thứ tự trong chuỗi không được bảo đảm, và hai lần chạy có thể cho hai chuỗi khác nhau, giống `ORDER BY` thiếu tie-breaker ở MS-007.

## 7. Tổng phụ và tổng cộng với ROLLUP

**`GROUP BY ROLLUP (c.code, t.kind)`** sinh ba cấp nhóm trong một câu: `(tp, kind)`, `(tp)` và `()` (tổng tất cả). Ở dòng tổng phụ, cột bị "cuộn" hiện NULL; **`GROUPING(cột)`** trả 1 khi cột đó đang bị cuộn:

```sql
SELECT CASE WHEN GROUPING(c.code) = 1 THEN N'TỔNG' ELSE c.code END              AS tp,
       CASE WHEN GROUPING(t.kind) = 1 THEN N'(mọi dịch vụ)' ELSE t.kind END     AS kind,
       SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END)                  AS hoan_thanh,
       SUM(CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END)             AS doanh_thu
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY ROLLUP (c.code, t.kind)
ORDER BY GROUPING(c.code), c.code, GROUPING(t.kind), t.kind;
```

```text
tp|kind|hoan_thanh|doanh_thu
--|----|----------|---------
DN|food|173|14902000
DN|parcel|68|3380000
DN|ride|148|8134000
DN|(mọi dịch vụ)|389|26416000
HN|food|413|35025000
HN|parcel|145|7041000
HN|ride|383|20925000
HN|(mọi dịch vụ)|941|62991000
SG|food|615|52898000
SG|parcel|196|9290000
SG|ride|566|30692000
SG|(mọi dịch vụ)|1377|92880000
TỔNG|(mọi dịch vụ)|2707|182287000
```

13 dòng = 9 dòng chi tiết + 3 tổng phụ + 1 tổng cộng. `ORDER BY GROUPING(…)` đặt dòng tổng sau các dòng chi tiết của nó. Một con số tổng chỉ đáng tin khi một cách tính **độc lập** cho cùng kết quả: lab chạy thêm `COUNT(*), SUM(fare)` thẳng trên `trips`, không `JOIN`, không `ROLLUP`, và nhận đúng `2707 | 182287000`. Con số **182.287.000 đ** sẽ theo bạn tới [MS-013](/articles/mssql-join-apply-cardinality), khi `JOIN` với `payments` làm nó phình lên.

**NULL của ROLLUP và NULL của dữ liệu.** Ở bảng trên `c.code`, `t.kind` đều NOT NULL nên an toàn. Thử với khu đón của chuyến Đà Nẵng, nơi 4 chuyến mất GPS, và gắn nhãn bằng `ISNULL(z.code, 'TONG')` thay vì `GROUPING()`:

```text
nhan_isnull|nhan_grouping|so_chuyen
-----------|-------------|---------
TONG|không rõ (mất GPS)|4
DN-BI-01|DN-BI-01|112
DN-BI-02|DN-BI-02|119
DN-TT-01|DN-TT-01|113
DN-TT-02|DN-TT-02|112
TONG|TỔNG|460
```

Cột `nhan_isnull` có **hai** dòng `TONG`: dòng 4 là nhóm mất GPS, dòng 460 mới là tổng. Đây là bảng có hai dòng "TỔNG" của kế toán ở đầu bài. Cột `nhan_grouping` dùng `CASE WHEN GROUPING(z.code) = 1 THEN N'TỔNG' ELSE COALESCE(z.code, N'không rõ (mất GPS)') END` nên ra đúng. Vẫn là bài học MS-011: một ô NULL, hai nghĩa, phân biệt bằng một nguồn khác.

## 8. GROUPING SETS, CUBE và GROUPING_ID

`ROLLUP (a, b)` chỉ cuộn từ phải sang trái: có tổng theo thành phố nhưng không có tổng theo dịch vụ. **`GROUPING SETS`** cho bạn liệt kê đúng các cấp cần, mỗi cấp là một danh sách cột trong ngoặc; `()` là tổng cộng. **`CUBE (a, b)`** sinh mọi tổ hợp, ở đây bằng `GROUPING SETS ((a, b), (a), (b), ())`. Lab kiểm cả hai cùng ra 16 dòng (9 + 3 + 3 + 1).

**`GROUPING_ID(a, b)`** ghép các bit `GROUPING` thành một số nguyên: bit của `a` có trọng số 2, của `b` có trọng số 1. Với hai cột: 0 = chi tiết, 1 = cuộn `kind` (tổng theo thành phố), 2 = cuộn `tp` (tổng theo dịch vụ), 3 = tổng cộng. Câu dưới chỉ giữ các dòng tổng bằng `HAVING GROUPING_ID(…) > 0`:

```sql
SELECT GROUPING_ID(c.code, t.kind)                                      AS cap,
       CASE WHEN GROUPING(c.code) = 1 THEN N'*' ELSE c.code END         AS tp,
       CASE WHEN GROUPING(t.kind) = 1 THEN N'*' ELSE t.kind END         AS kind,
       COUNT(*)                                                         AS da_ket_thuc,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(4, 1)) AS ty_le_huy_pct
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at >= '2026-10-01 00:00 +07:00' AND t.requested_at < '2026-10-31 00:00 +07:00'
GROUP BY GROUPING SETS ((c.code, t.kind), (c.code), (t.kind), ())
HAVING GROUPING_ID(c.code, t.kind) > 0
ORDER BY cap, tp, kind;
```

```text
cap|tp|kind|da_ket_thuc|ty_le_huy_pct
---|--|----|-----------|-------------
1|DN|*|459|15.3
1|HN|*|1106|14.9
1|SG|*|1603|14.1
2|*|food|1430|16.0
2|*|parcel|488|16.2
2|*|ride|1250|12.2
3|*|*|3168|14.6
```

Mỗi dòng tổng **tính lại** tỉ lệ từ các dòng gốc của nó, không lấy trung bình các tỉ lệ con. Tỉ lệ huỷ cả tháng là 461 ÷ 3.168 = 14,55 % (dòng `cap = 3`, làm tròn 14,6), còn trung bình cộng 9 tỉ lệ ở mục 3 là **14,35 %** (lab kiểm): nhóm 72 chuyến nặng ngang nhóm 717 chuyến. Cột `cap` cũng tiện cho tầng hiển thị: ứng dụng tô đậm dòng `cap = 3`, thụt lề dòng `cap = 1`, không cần đoán từ NULL.

## 9. Hai mẫu số trong cùng một dòng, và những số không cộng được

Câu 8 của lab nối chuyến hoàn thành trong kỳ với `trip_ratings` bằng `LEFT JOIN`, nhóm theo thành phố:

```text
tp|hoan_thanh|co_danh_gia|do_phu_pct|co_sao|diem_tb
--|----------|-----------|----------|------|-------
DN|389|266|68.4|251|4.30
HN|941|670|71.2|619|4.17
SG|1377|955|69.4|868|4.23
```

**Độ phủ** chia cho chuyến hoàn thành: `COUNT(r.trip_id)` đếm chuyến *có* dòng đánh giá vì `r.trip_id` chỉ NULL khi không có dòng. **Điểm trung bình** chia cho lượt có sao, và tính trên `CAST(r.stars AS decimal(9, 4))` để không bị cắt thành 4. Đặt `co_danh_gia` và `co_sao` ngay trong bảng là cách rẻ nhất để người đọc kiểm mẫu số: 4,30 của Đà Nẵng dựa trên 251 lượt, ít hơn hẳn 868 lượt của TP. HCM.

Con số 1.103 khách của Marketing đến từ việc cộng `COUNT(DISTINCT customer_id)` của từng dịch vụ. Câu 9 của lab tính lại bằng `ROLLUP`:

```text
kind|so_khach|hoan_thanh|chuyen_moi_khach
----|--------|----------|----------------
food|441|1201|2.72
parcel|241|409|1.70
ride|421|1097|2.61
TỔNG|569|2707|4.76
```

`hoan_thanh` cộng được: 1.201 + 409 + 1.097 = 2.707. `so_khach` thì không: 441 + 241 + 421 = 1.103, trong khi dòng TỔNG (ROLLUP tính lại `COUNT(DISTINCT)` từ dòng gốc) chỉ có **569**. Khách vừa đặt xe vừa gọi đồ ăn được đếm ở cả hai dòng. Số "khác nhau" và tỉ lệ của cấp trên phải **tính lại**, không cộng từ cấp dưới.

## 10. Làm trong DBStudio

- `Ctrl+Enter` chạy batch đang chứa con trỏ: tệp lab chia mỗi mục thành một batch `GO`, nên bạn chạy riêng được câu `ROLLUP` hay `GROUPING SETS` để xem kết quả.
- Các lỗi mong đợi (8115, 8120, 207) chạy qua `lab.expect_error`, nên chỉ thấy dòng `PASS …` ở tab **Thông điệp**. Muốn thấy lỗi thật, chép câu SQL trong chuỗi ra một tab mới và chạy.
- Xuất bảng ROLLUP ra XLSX cho kế toán thì giữ cột `cap` (hoặc hai cột `GROUPING`) trong file: người nhận lọc được dòng chi tiết mà không phải đoán dòng nào là tổng.

## 11. So với PostgreSQL

Cùng câu hỏi, cùng dữ liệu với [PG-012](/articles/postgresql-aggregate-group-by-having); những chỗ khác nhau thật sự:

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| `SUM` của `integer` | trả `bigint`, không tràn ở quy mô lab | trả `int`, tràn ở 2.147.483.647 (lỗi 8115) |
| `AVG` của `integer` | trả `numeric` | trả `int`, cắt phần lẻ |
| `COUNT(*)` | `bigint` | `int`; có `COUNT_BIG` trả `bigint` |
| Đếm có điều kiện | `count(*) FILTER (WHERE …)` | `SUM(CASE WHEN … THEN 1 ELSE 0 END)` |
| Nhóm theo khoá chính rồi chọn cột khác của bảng đó | được (phụ thuộc hàm) | lỗi 8120; liệt kê đủ cột |
| Ghép chuỗi trong nhóm | `string_agg(x, ', ' ORDER BY x)` | `STRING_AGG(x, ', ') WITHIN GROUP (ORDER BY x)` |
| Mã nhóm dạng số | `GROUPING(a, b)` nhận nhiều cột, trả bitmask | `GROUPING_ID(a, b)`; `GROUPING` chỉ nhận một cột |
| `ROLLUP`, `CUBE`, `GROUPING SETS` | có | có, cùng ý nghĩa |

## 12. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Tỉ lệ ra 0 hoặc số tròn | `int / int` chia nguyên | `100.0 * …` trước khi chia |
| Cước trung bình lệch 1 đ | `AVG` trên `int` cắt phần lẻ | `ROUND(AVG(CAST(x AS decimal(…))), 0)` |
| Cước trung bình cao bất thường | `AVG` trên cả chuyến huỷ | `AVG(CASE WHEN status = 'completed' THEN … END)` |
| Lỗi 8115 khi cộng tiền | `SUM(int)` trả `int` | `SUM(CAST(x AS bigint))` |
| Lỗi 8120 | cột không nhóm, không tổng hợp | thêm vào `GROUP BY` (kể cả cột phụ thuộc khoá chính) |
| Lỗi 207 ở `HAVING` | dùng tên cột đặt ở `SELECT` | viết lại biểu thức |
| Hai dòng "TỔNG" sau `ROLLUP` | `ISNULL`/`COALESCE` gắn nhãn cả NULL dữ liệu | `GROUPING(cột)` |
| Tổng khách các dòng > tổng thật | cộng `COUNT(DISTINCT)` | tính lại ở cấp trên (`ROLLUP`, `GROUPING SETS`) |
| Tỉ lệ cấp trên lệch | trung bình các tỉ lệ con | Σ tử ÷ Σ mẫu |
| Chuỗi `STRING_AGG` đổi thứ tự | thiếu `WITHIN GROUP (ORDER BY …)` | thêm, kèm tie-breaker |

## 13. Đánh đổi

- **Một câu nhiều cột hay nhiều câu?** Bảng ở mục 3 tính sáu chỉ số trong một lần quét; tách sáu câu thì mỗi câu tự lọc kỳ, sáu chỗ có thể lệch nhau.
- **`ROLLUP`/`GROUPING SETS` hay cộng ở bảng tính?** Tổng trong SQL tính lại từ dòng gốc nên đúng cả với `COUNT(DISTINCT)` và tỉ lệ; cộng ở Excel chỉ đúng với số đếm và tổng tiền. Đổi lại phải đọc NULL bằng `GROUPING()`.
- **`ROLLUP` hay `GROUPING SETS`?** `ROLLUP` ngắn khi cấp tổng đi theo thứ bậc (thành phố → dịch vụ); `GROUPING SETS` rõ hơn khi cần các cấp không lồng nhau, và không sinh cấp thừa như `CUBE` với nhiều cột (`CUBE` của 4 cột là 16 cấp).
- **`CAST` sang `decimal` ở đâu?** Đổi kiểu từng dòng trước khi tổng hợp (`AVG(CAST(x AS decimal(…)))`) giữ đủ chữ số lẻ; đổi sau (`CAST(AVG(x) AS decimal(…))`) thì phần lẻ đã mất.

## 14. Bài tập

Lời giải đã chạy trong phần "Bài tập" ở cuối `12-group-by-rollup.sql` (nền như mục 1).

**Bài 1.** Lập bảng tháng 10 theo **dịch vụ** cho cả nước: đã kết thúc, hoàn thành, huỷ, tỉ lệ huỷ, doanh thu, cước trung bình, kèm một dòng tổng. Dòng tổng nào cộng được, dòng nào phải tính lại?

<details><summary><b>Lời giải</b></summary>
<p>Câu ở mục 3, bỏ <code>c.code</code> và <code>JOIN cities</code>, đổi thành <code>GROUP BY ROLLUP (kind)</code> và nhãn <code>CASE WHEN GROUPING(kind) = 1 THEN N'TỔNG' ELSE kind END</code>. Kết quả đã chạy:</p>
<pre><code class="language-text">kind|da_ket_thuc|hoan_thanh|huy|ty_le_huy_pct|doanh_thu|cuoc_tb
food|1430|1201|229|16.0|102825000|85616
parcel|488|409|79|16.2|19711000|48193
ride|1250|1097|153|12.2|59751000|54468
TỔNG|3168|2707|461|14.6|182287000|67339</code></pre>
<p>Số đếm và doanh thu của dòng TỔNG bằng tổng các dòng trên (3.168, 182.287.000 đ). Tỉ lệ huỷ 14,6 % và cước trung bình 67.339 đ thì <code>ROLLUP</code> tính lại từ dòng gốc; trung bình cộng ba tỉ lệ (14,8 %) hay ba cước trung bình đều sai.</p>
</details>

**Bài 2.** Liệt kê quán có **ít nhất 50 đơn** trong tháng **và** tỉ lệ huỷ từ 20 % trở lên. Điều kiện nào ở `WHERE`, điều kiện nào ở `HAVING`? Nhóm theo `m.id` có đủ để chọn `m.name` không?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT m.name AS quan, c.code AS tp, COUNT(*) AS so_don,
       CAST(100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) AS decimal(4, 1)) AS ty_le_huy_pct
FROM toinoi.trips AS t
JOIN toinoi.merchants AS m ON m.id = t.merchant_id
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.requested_at &gt;= '2026-10-01 00:00 +07:00' AND t.requested_at &lt; '2026-10-31 00:00 +07:00'
GROUP BY m.id, m.name, c.code
HAVING COUNT(*) &gt;= 50
   AND 100.0 * SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) / COUNT(*) &gt;= 20
ORDER BY ty_le_huy_pct DESC, quan;</code></pre>
<p>Kết quả đã chạy, bốn quán: Bún Bò Lab 201 (HN, 63 đơn, 27,0 %), Bún Bò Lab 101 (SG, 70, 24,3 %), Bánh Mì Lab 203 (HN, 58, 22,4 %), Gà Rán Lab 105 (SG, 54, 20,4 %). Kỳ báo cáo là điều kiện trên dòng (<code>WHERE</code>), hai ngưỡng trên nhóm (<code>HAVING</code>). Khác PostgreSQL, nhóm riêng theo <code>m.id</code> rồi chọn <code>m.name</code> là lỗi 8120, nên <code>m.name</code> phải có trong <code>GROUP BY</code>. Giữ <code>m.id</code> trong nhóm để hai quán trùng tên không bị gộp.</p>
</details>

**Bài 3.** Tìm tài xế có điểm sao trung bình **dưới 4,0**, chỉ xét người có **ít nhất 35 lượt có sao** trong tháng. Một bạn đặt ngưỡng là `COUNT(*) >= 35`. Kết quả khác gì?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT d.code AS tai_xe, COUNT(*) AS hoan_thanh, COUNT(r.stars) AS co_sao,
       CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2)) AS diem_tb
FROM toinoi.trips AS t
JOIN toinoi.drivers AS d ON d.id = t.driver_id
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed'
  AND t.requested_at &gt;= '2026-10-01 00:00 +07:00' AND t.requested_at &lt; '2026-10-31 00:00 +07:00'
GROUP BY d.code
HAVING COUNT(r.stars) &gt;= 35 AND AVG(CAST(r.stars AS decimal(9, 4))) &lt; 4
ORDER BY diem_tb, tai_xe;</code></pre>
<p>Kết quả đã chạy, bốn người: 8202 (36 lượt có sao, 3,94), 8211 (40, 3,95), 8215 (39, 3,95), 8209 (38, 3,97). Ngưỡng phải đặt trên <b>mẫu số của <code>AVG</code></b>, tức <code>COUNT(r.stars)</code>. Với <code>COUNT(*) &gt;= 35</code> (đếm chuyến hoàn thành) danh sách thành 6 người: lọt thêm hai tài xế có nhiều chuyến nhưng chưa đủ 35 lượt chấm sao, nên điểm của họ dựa trên ít lượt hơn mức đội chất lượng đặt ra.</p>
</details>

## 15. Đọc tiếp

- Microsoft Learn: [SELECT - GROUP BY](https://learn.microsoft.com/sql/t-sql/queries/select-group-by-transact-sql) (gồm `ROLLUP`, `CUBE`, `GROUPING SETS`), [HAVING](https://learn.microsoft.com/sql/t-sql/queries/select-having-transact-sql), [GROUPING](https://learn.microsoft.com/sql/t-sql/functions/grouping-transact-sql), [GROUPING_ID](https://learn.microsoft.com/sql/t-sql/functions/grouping-id-transact-sql), [SUM](https://learn.microsoft.com/sql/t-sql/functions/sum-transact-sql), [AVG](https://learn.microsoft.com/sql/t-sql/functions/avg-transact-sql), [COUNT_BIG](https://learn.microsoft.com/sql/t-sql/functions/count-big-transact-sql), [STRING_AGG](https://learn.microsoft.com/sql/t-sql/functions/string-agg-transact-sql).
- Cùng chủ đề trên PostgreSQL: [PG-012 · COUNT, SUM, GROUP BY và HAVING](/articles/postgresql-aggregate-group-by-having).
- Lab: `examples/toi-noi-mssql/p02/12-group-by-rollup.sql`.

Bài tiếp theo: [MS-013 · JOIN, APPLY và số dòng sau khi nối](/articles/mssql-join-apply-cardinality)
