---
slug: mssql-window-function-window-clause
title: "Window function và mệnh đề WINDOW: luỹ kế, xếp hạng tài xế và so sánh ngày trước trên dữ liệu Tới Nơi"
summary: "Biểu đồ trung bình 7 ngày của Đà Nẵng bỏ trống đúng ngày bão, luỹ kế của khách doanh nghiệp nhảy 6 chuyến một lúc, top 3 tài xế lại có 4 người. Bài đi qua OVER, PARTITION BY, frame ROWS và RANGE (và giới hạn RANGE của SQL Server), ROW_NUMBER/RANK/DENSE_RANK/NTILE, LAG/LEAD, bẫy LAST_VALUE, IGNORE NULLS và mệnh đề WINDOW mới của SQL Server 2022."
category: mssql
level: intermediate
tags: [mssql, ms-015, sql-server-2022, t-sql, window-function, over, window-clause, ranking, lag-lead, toi-noi]
language: vi
---

Đầu tháng 11, đội vận hành Tới Nơi nhận ba câu hỏi về báo cáo tháng 10, và cả ba đều xuất phát từ câu T-SQL chạy không lỗi.

Biểu đồ "doanh thu trung bình 7 ngày" của Đà Nẵng có một lỗ hổng ở ngày 20/10, hôm có bão và không có chuyến nào; ngày 21/10 ngay sau đó lại cao bất thường. Bảng luỹ kế số chuyến của khách doanh nghiệp nhảy từ 3 lên 9 trong một dòng, ở lô 6 chuyến đặt qua API cùng giây. Còn bảng "top 3 tài xế Đà Nẵng" có 4 người, trong khi bảng "top 3" của TP. Hồ Chí Minh nhảy từ hạng 1 sang hạng 3.

Cả ba báo cáo dùng **window function** (hàm cửa sổ). Bài này giải thích hàm cửa sổ tính trên tập dòng nào, vì sao phải ghi **frame** ra thay vì dựa vào mặc định, và SQL Server 2022 khác PostgreSQL ở đâu: `RANGE` không nhận khoảng cách, không có `GROUPS`, nhưng có mệnh đề `WINDOW` và `IGNORE NULLS` mới.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-015 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mssql-group-by-rollup-grouping-sets">MS-012 · GROUP BY, HAVING, ROLLUP và GROUPING SETS</a>, <a href="/articles/mssql-subquery-exists-cte-temp-table">MS-014 · Subquery, EXISTS, CTE và bảng tạm</a>. Lab: <code>examples/toi-noi-mssql/p02</code> (tệp <code>15-window-function.sql</code>). Sau bài này, bạn viết được luỹ kế, trung bình trượt, xếp hạng và so sánh với dòng trước với frame ghi rõ, và biết khi nào kết quả phụ thuộc vào dòng hoà hoặc ngày thiếu dữ liệu.</p>
</div>

## 1. Dữ liệu dùng trong bài

Fixture Phần 2 (ảnh chụp 01:00 ngày 31/10/2026 giờ VN, 3.174 chuyến) có ba điểm cài sẵn cho bài này: **ngày 20/10 Đà Nẵng có bão, 0 chuyến**; khách doanh nghiệp đặt 18 chuyến, trong đó lô API ngày 10/10 lúc 09:00:00 có 6 chuyến cùng giây (100971–100976); và vài cặp tài xế cùng số chuyến hoàn thành.

```bash
cd examples/toi-noi-mssql
LAB_DB=ToiNoi_thu node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/15-window-function.sql
```

Kết quả chép từ lần chạy trên SQL Server 2022 16.0.4325.4 (RTM-CU28), 25 dòng `PASS`, 0 `FAIL`. Database lab tạo mới trên SQL Server 2022 nên ở **mức tương thích** (compatibility level) 160; mục 8 cho thấy vì sao con số này quan trọng.

## 2. Window function khác GROUP BY ở đâu?

`GROUP BY` gộp nhiều dòng thành một. Hàm cửa sổ thì **giữ nguyên từng dòng** và gắn thêm một giá trị tính trên một tập dòng "xung quanh" dòng đó. Tập dòng ấy mô tả bằng mệnh đề `OVER (…)` gồm ba phần:

| Phần | Ý nghĩa | Ví dụ |
|---|---|---|
| `PARTITION BY` | Chia nhóm, mỗi nhóm tính riêng | Mỗi thành phố một bảng xếp hạng |
| `ORDER BY` | Thứ tự trong nhóm | Theo ngày, theo số chuyến giảm dần |
| frame | Trong nhóm, dòng nào được đưa vào phép tính | 6 dòng trước tới dòng hiện tại |

**Frame** (khung) chỉ có nghĩa với hàm gộp (`SUM`, `AVG`, `COUNT`…) và với `FIRST_VALUE`/`LAST_VALUE`. Hàm xếp hạng (`ROW_NUMBER`, `RANK`…) và `LAG`/`LEAD` không nhận frame.

Hàm cửa sổ được tính **sau** `WHERE`, `GROUP BY` và `HAVING`, ngay trước `ORDER BY` cuối. Vì vậy nó chỉ được đặt trong danh sách cột hoặc `ORDER BY`. Viết trong `WHERE` thì lỗi:

```text
PASS 3d hàm cửa sổ trong WHERE — lỗi 4108: Windowed functions can only appear in the SELECT or ORDER BY clauses.
```

Muốn lọc theo kết quả hàm cửa sổ ("top 3"), tính nó ở truy vấn trong (CTE, bảng dẫn xuất, bảng tạm) rồi lọc ở truy vấn ngoài, như các mục dưới.

## 3. Luỹ kế và trung bình 7 ngày trên lịch đầy đủ

Báo cáo doanh thu Đà Nẵng cần ba cột: doanh thu ngày, **luỹ kế** từ đầu tháng và **trung bình 7 ngày** (hôm nay và 6 ngày trước). Dữ liệu gốc là **thưa**: ngày không có chuyến thì không có dòng nào. Bước đầu tiên là dựng lịch đủ 30 ngày bằng `GENERATE_SERIES` (hàm mới của SQL Server 2022) rồi `LEFT JOIN`:

```sql
WITH lich AS (
    SELECT DATEADD(day, value, CAST('2026-10-01' AS date)) AS ngay
    FROM GENERATE_SERIES(0, 29)
),
theo_ngay AS (
    SELECT CAST(SWITCHOFFSET(requested_at, '+07:00') AS date) AS ngay,
           COUNT(*) AS so_chuyen, SUM(fare) AS doanh_thu
    FROM toinoi.trips
    WHERE city_id = 3 AND status = 'completed'
    GROUP BY CAST(SWITCHOFFSET(requested_at, '+07:00') AS date)
)
SELECT l.ngay,
       COALESCE(d.so_chuyen, 0) AS so_chuyen,
       COALESCE(d.doanh_thu, 0) AS doanh_thu,
       SUM(COALESCE(d.doanh_thu, 0)) OVER (ORDER BY l.ngay
                                          ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS luy_ke,
       CAST(ROUND(AVG(CAST(COALESCE(d.doanh_thu, 0) AS decimal(18, 2)))
                  OVER (ORDER BY l.ngay ROWS BETWEEN 6 PRECEDING AND CURRENT ROW), 0) AS int) AS tb7_lich_day
FROM lich AS l
LEFT JOIN theo_ngay AS d ON d.ngay = l.ngay;
```

Lab lưu kết quả cả tháng vào bảng tạm `#dn` rồi mới lọc 17–24/10 để in, vì hàm cửa sổ tính sau `WHERE` của cùng cấp truy vấn: lọc ngay trong câu trên thì ngày 17/10 sẽ không "thấy" sáu ngày trước nó. Cột `tb7_rows_thua` là cùng frame `ROWS BETWEEN 6 PRECEDING AND CURRENT ROW` nhưng tính trên dữ liệu thưa, trước khi ghép lịch:

```text
ngay|so_chuyen|doanh_thu|luy_ke|tb7_lich_day|tb7_rows_thua
----|---------|---------|------|------------|-------------
2026-10-17|13|718000|15952000|927429|927429
2026-10-18|16|945000|16897000|859857|859857
2026-10-19|11|637000|17534000|828571|828571
2026-10-20|0|0|17534000|672571|NULL
2026-10-21|14|872000|18406000|677286|797143
2026-10-22|11|785000|19191000|679000|789429
2026-10-23|13|941000|20132000|699714|813429
2026-10-24|14|787000|20919000|709571|812143
```

Đây là biểu đồ có lỗ hổng ở đầu bài:

- Ngày 20/10 trên dữ liệu thưa không có dòng, nên sau `LEFT JOIN` nó là NULL: biểu đồ bỏ trống đúng ngày đáng chú ý nhất.
- Từ 21/10, "6 dòng trước" của dữ liệu thưa trải trên **7 ngày lịch** (bỏ qua ngày bão), nên trung bình là 797.143 thay vì 677.286: ngày 0 đ không kéo trung bình xuống.

`ROWS` đếm **dòng**, không đếm ngày. Muốn "7 ngày" thì phải có đủ một dòng cho mỗi ngày. Hai chi tiết T-SQL khác: `AVG` trên cột `int` trả `int` (cắt phần lẻ), nên lab ép `decimal` trước khi lấy trung bình; và luỹ kế cuối tháng 26.416.000 đ khớp doanh thu Đà Nẵng ở mục 7.

### RANGE của SQL Server không nhận khoảng cách

PostgreSQL giải bài này bằng frame `RANGE BETWEEN interval '6 days' PRECEDING AND CURRENT ROW` ngay trên dữ liệu thưa. SQL Server 2022 không có cách đó:

```text
PASS 1c RANGE 6 PRECEDING — lỗi 4194: RANGE is only supported with UNBOUNDED and CURRENT ROW window frame delimiters.
PASS 1d GROUPS — lỗi 102: Incorrect syntax near 'GROUPS'.
```

`RANGE` chỉ nhận `UNBOUNDED PRECEDING`, `CURRENT ROW` và `UNBOUNDED FOLLOWING`; kiểu frame `GROUPS` cũng chưa có. Trên SQL Server, "N ngày gần nhất" gần như luôn bắt đầu bằng một bảng lịch, từ `GENERATE_SERIES` như trên hoặc từ một bảng lịch cố định của hệ thống báo cáo.

## 4. Frame mặc định khi thứ tự có hoà

Khi có `ORDER BY` mà không ghi frame, SQL Server dùng mặc định `RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW`: mọi dòng từ đầu nhóm tới dòng hiện tại **và mọi dòng hoà với nó** trên khoá `ORDER BY`. Với khoá duy nhất, `RANGE` và `ROWS` cho cùng kết quả. Với khoá có hoà thì khác:

```sql
SELECT code, CAST(requested_at AS varchar(34)) AS requested_at, fare,
       COUNT(*)  OVER (ORDER BY requested_at)                               AS dem_mac_dinh,
       SUM(fare) OVER (ORDER BY requested_at)                               AS luy_ke_mac_dinh,
       COUNT(*)  OVER (ORDER BY requested_at, code ROWS UNBOUNDED PRECEDING) AS dem_rows,
       SUM(fare) OVER (ORDER BY requested_at, code ROWS UNBOUNDED PRECEDING) AS luy_ke_rows
FROM toinoi.trips
WHERE customer_id = 'CB000000-0000-4000-8000-000000000001';
```

```text
code|requested_at|fare|dem_mac_dinh|luy_ke_mac_dinh|dem_rows|luy_ke_rows
----|------------|----|------------|---------------|--------|-----------
100130|2026-10-02 10:15:00.000 +07:00|45000|1|45000|1|45000
100500|2026-10-05 15:40:00.000 +07:00|75000|2|120000|2|120000
100669|2026-10-07 09:20:00.000 +07:00|57000|3|177000|3|177000
100971|2026-10-10 09:00:00.000 +07:00|24000|9|462000|4|201000
100972|2026-10-10 09:00:00.000 +07:00|53000|9|462000|5|254000
100973|2026-10-10 09:00:00.000 +07:00|20000|9|462000|6|274000
100974|2026-10-10 09:00:00.000 +07:00|49000|9|462000|7|323000
100975|2026-10-10 09:00:00.000 +07:00|78000|9|462000|8|401000
100976|2026-10-10 09:00:00.000 +07:00|61000|9|462000|9|462000
101224|2026-10-12 11:05:00.000 +07:00|27000|10|489000|10|489000
```

Sáu chuyến của lô 10/10 cùng nhận luỹ kế 9 chuyến và 462.000 đ ở frame mặc định: đó là bảng "nhảy từ 3 lên 9" ở đầu bài. Không có dòng nào sai về nghĩa (đến 09:00:00 ngày 10/10, khách quả thật đã đặt 9 chuyến), nhưng nếu báo cáo cần "luỹ kế sau từng chuyến" thì phải dùng `ROWS` kèm tie-breaker `code`, để mỗi dòng có một vị trí duy nhất. Cả tháng, khách này có 18 chuyến với tổng cước 809.000 đ.

Frame mặc định còn có cái giá về thực thi. Chạy luỹ kế cước trên cả bảng `trips` với `SET STATISTICS IO ON` (đã chạy, một lần đo trên lab), bản mặc định có một bảng `Worktable` với hàng nghìn lần đọc trang, bản `ROWS UNBOUNDED PRECEDING` thì `Worktable` bằng 0. Phần 3 đọc chuyện này trong execution plan; quy tắc ở đây đơn giản: **có `ORDER BY` trong `OVER` thì ghi frame ra**.

## 5. Xếp hạng tài xế và mệnh đề WINDOW

Bảng "top 3 tài xế mỗi thành phố" theo số chuyến hoàn thành. Ba hàm xếp hạng xử lý dòng hoà khác nhau:

- `ROW_NUMBER()`: đánh số 1, 2, 3… liên tục; hai dòng hoà được thứ tự theo tie-breaker.
- `RANK()`: dòng hoà cùng hạng, hạng sau **nhảy cóc** (1, 1, 3).
- `DENSE_RANK()`: dòng hoà cùng hạng, hạng sau **không nhảy** (1, 1, 2).
- `NTILE(n)`: chia nhóm thành `n` phần gần bằng nhau theo thứ tự.

Bốn hàm dùng chung một cách chia nhóm. **Mệnh đề `WINDOW`** (mới trong SQL Server 2022) cho đặt tên một định nghĩa cửa sổ, viết sau `HAVING` và trước `ORDER BY`. Một cửa sổ có tên còn làm nền cho cửa sổ khác, và `OVER (ten ORDER BY …)` thêm phần còn thiếu vào cửa sổ đó:

```sql
SELECT c.code AS tp, d.code AS tai_xe, COUNT(*) AS hoan_thanh,
       ROW_NUMBER() OVER (theo_tp ORDER BY COUNT(*) DESC, d.code) AS so_thu_tu,
       RANK()       OVER xep_hang AS hang,
       DENSE_RANK() OVER xep_hang AS hang_lien_tuc,
       NTILE(4)     OVER (theo_tp ORDER BY COUNT(*) DESC, d.code) AS nhom_4
INTO #hang
FROM toinoi.trips AS t
JOIN toinoi.drivers AS d ON d.id = t.driver_id
JOIN toinoi.cities  AS c ON c.id = d.city_id
WHERE t.status = 'completed'
GROUP BY c.code, d.code
WINDOW theo_tp  AS (PARTITION BY c.code),
       xep_hang AS (theo_tp ORDER BY COUNT(*) DESC);

SELECT tp, tai_xe, hoan_thanh, so_thu_tu, hang, hang_lien_tuc
FROM #hang
WHERE hang_lien_tuc <= 3
ORDER BY tp, so_thu_tu;
```

```text
tp|tai_xe|hoan_thanh|so_thu_tu|hang|hang_lien_tuc
--|------|----------|---------|----|-------------
DN|8305|44|1|1|1
DN|8304|41|2|2|2
DN|8302|40|3|3|3
DN|8310|40|4|3|3
HN|8210|63|1|1|1
HN|8216|60|2|2|2
HN|8205|59|3|3|3
SG|8101|68|1|1|1
SG|8109|68|2|1|1
SG|8120|66|3|3|2
SG|8108|64|4|4|3
SG|8124|64|5|4|3
```

Hàm cửa sổ ở đây chạy trên **các dòng đã gom** (mỗi tài xế một dòng), nên `ORDER BY COUNT(*)` hợp lệ trong `OVER`. Đọc kết quả để trả lời câu hỏi đầu bài:

- Lọc `hang_lien_tuc <= 3` trả 4 người ở Đà Nẵng (8302 và 8310 cùng 40 chuyến) và 5 người ở TP. Hồ Chí Minh: `DENSE_RANK` đếm **mức** số chuyến, không đếm người.
- Lọc `hang <= 3` ở TP. Hồ Chí Minh ra 3 người (8101, 8109, 8120) nhưng không ai hạng 2, vì hai người cùng hạng 1. Đây là bảng "nhảy từ hạng 1 sang hạng 3" mà đội vận hành thấy.
- Lọc `so_thu_tu <= 3` luôn đúng 3 người, nhưng ai được vào khi hoà là do tie-breaker `d.code` quyết, một quy tắc cần ghi vào định nghĩa báo cáo.

Không hàm nào "đúng" hơn; câu hỏi nghiệp vụ quyết định. "Thưởng cho 3 người" thì cần `ROW_NUMBER` cộng một quy tắc phân xử rõ ràng; "vinh danh mọi người thuộc 3 mức cao nhất" thì `DENSE_RANK`.

`NTILE(4)` trên 24 tài xế TP. Hồ Chí Minh cho 4 nhóm 6 người (nhóm 1: 63–68 chuyến, nhóm 4: 42–52). Khi số dòng không chia hết, các nhóm đầu nhận thêm một dòng; hoà ở ranh giới nhóm cũng do tie-breaker quyết.

## 6. LAG, LEAD và giá trị đầu, cuối

`LAG(cot)` lấy giá trị của dòng **đứng trước** trong cửa sổ, `LEAD(cot)` lấy dòng **đứng sau**; dòng không có hàng xóm thì nhận NULL (hoặc giá trị mặc định ở tham số thứ ba). Sáu chuyến đầu tháng của khách 0910000016, người đi nhiều nhất, với một cửa sổ có tên dùng bốn lần:

```sql
SELECT TOP (6) t.code, CAST(t.requested_at AS varchar(34)) AS requested_at, t.kind,
       LAG(t.kind) OVER w                                           AS dich_vu_truoc,
       DATEDIFF(minute, LAG(t.requested_at) OVER w, t.requested_at) AS phut_tu_chuyen_truoc,
       LEAD(t.code) OVER w                                          AS chuyen_sau,
       ROW_NUMBER() OVER w                                          AS chuyen_thu
FROM toinoi.trips AS t
JOIN toinoi.customers AS c ON c.id = t.customer_id
WHERE c.phone = '0910000016'
WINDOW w AS (ORDER BY t.requested_at, t.code)
ORDER BY t.requested_at, t.code;
```

```text
code|requested_at|kind|dich_vu_truoc|phut_tu_chuyen_truoc|chuyen_sau|chuyen_thu
----|------------|----|-------------|--------------------|----------|----------
100008|2026-10-01 06:12:00.000 +07:00|ride|NULL|NULL|100149|1
100149|2026-10-02 12:45:00.000 +07:00|ride|ride|1833|100215|2
100215|2026-10-03 07:04:00.000 +07:00|ride|ride|1099|100259|3
100259|2026-10-03 13:02:00.000 +07:00|ride|ride|358|100277|4
100277|2026-10-03 17:01:00.000 +07:00|ride|ride|239|100369|5
100369|2026-10-04 12:07:00.000 +07:00|ride|ride|1146|100429|6
```

T-SQL không có kiểu `interval`: khoảng cách giữa hai thời điểm là `DATEDIFF` với đơn vị chọn trước (ở đây là phút). `TOP (6)` được áp sau khi hàm cửa sổ tính xong, nên `LEAD` của dòng thứ 6 vẫn thấy chuyến 100429 nằm ngoài kết quả.

### Bẫy LAST_VALUE

`FIRST_VALUE` và `LAST_VALUE` lấy giá trị ở dòng đầu và dòng cuối **của frame**, không phải của cả nhóm. Với frame mặc định (tới dòng hiện tại), "dòng cuối" chính là dòng hiện tại:

```text
code|chuyen_dau|last_value_sai|chuyen_cuoi
----|----------|--------------|-----------
100008|100008|100008|103123
100149|100008|100149|103123
100215|100008|100215|103123
100259|100008|100259|103123
```

Cột `last_value_sai` chỉ lặp lại mã của chính dòng đó. Cột `chuyen_cuoi` ghi frame `ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING` nên mới ra chuyến cuối tháng 103123.

### IGNORE NULLS (SQL Server 2022)

Câu "doanh thu hôm nay so với **ngày có dữ liệu gần nhất**" trên lịch Đà Nẵng, nếu để ngày bão là NULL thay vì 0:

```sql
SELECT ngay, doanh_thu,
       LAG(doanh_thu) OVER (ORDER BY ngay) AS lag_thuong,
       LAST_VALUE(doanh_thu) IGNORE NULLS
           OVER (ORDER BY ngay ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING) AS ngay_co_du_lieu_truoc
FROM (SELECT ngay, NULLIF(doanh_thu, 0) AS doanh_thu FROM #dn) AS s;
```

```text
ngay|doanh_thu|lag_thuong|ngay_co_du_lieu_truoc
----|---------|----------|---------------------
2026-10-19|637000|945000|945000
2026-10-20|NULL|637000|637000
2026-10-21|872000|NULL|637000
2026-10-22|785000|872000|872000
```

`LAG` ngày 21/10 trỏ vào ngày bão nên ra NULL. `LAST_VALUE … IGNORE NULLS` bỏ qua dòng NULL trong frame (mọi ngày **trước** hôm nay) và lấy 637.000 của 19/10. `IGNORE NULLS` là cú pháp mới của SQL Server 2022 cho `FIRST_VALUE`/`LAST_VALUE`; trên bản CU28 của lab, `LAG(doanh_thu) IGNORE NULLS` cũng chạy và ngày 21/10 cũng ra 637.000 (assertion `5c`). Chọn NULL hay 0 cho ngày bão là quyết định nghiệp vụ: 0 cho trung bình trượt ở mục 3, NULL cho câu "so với ngày có hoạt động gần nhất".

## 7. Tỉ trọng: OVER () sau GROUP BY

Cửa sổ rỗng `OVER ()` là toàn bộ kết quả. Kết hợp với `GROUP BY`, hàm cửa sổ chạy trên các dòng đã gom, nên `SUM(SUM(t.fare)) OVER ()` là tổng của các tổng:

```sql
SELECT c.code AS tp,
       SUM(t.fare)                                                                     AS doanh_thu,
       SUM(SUM(t.fare)) OVER ()                                                        AS tong,
       CAST(ROUND(100.0 * SUM(t.fare) / SUM(SUM(t.fare)) OVER (), 2) AS decimal(5, 2)) AS ty_trong_pct
FROM toinoi.trips AS t
JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.status = 'completed'
GROUP BY c.code
ORDER BY doanh_thu DESC, tp;
```

```text
tp|doanh_thu|tong|ty_trong_pct
--|---------|----|------------
SG|92880000|182287000|50.95
HN|62991000|182287000|34.56
DN|26416000|182287000|14.49
```

`100.0 *` đặt ở đầu để phép chia là phép chia số thập phân; viết `100 * SUM(…) / SUM(SUM(…)) OVER ()` với toàn số nguyên thì SQL Server chia nguyên. Thêm `PARTITION BY c.code` vào `OVER` là có tỉ trọng **trong** mỗi thành phố (bài tập 3).

## 8. Mức tương thích: WINDOW và GENERATE_SERIES cần 160

Mệnh đề `WINDOW` và `GENERATE_SERIES` chỉ có khi database ở **mức tương thích** 160 (SQL Server 2022). Mức tương thích là thiết lập của từng database, quyết định một số hành vi của T-SQL và bộ tối ưu. Database nâng cấp từ bản cũ (restore, attach) thường giữ mức cũ cho tới khi có người đổi. Lab đổi database sang 150 để thử rồi đặt lại 160:

```text
PASS 7a WINDOW ở mức 150 — lỗi 102: Incorrect syntax near 'w'.
PASS 7b GENERATE_SERIES ở mức 150 — lỗi 208: Invalid object name 'GENERATE_SERIES'.
PASS 7c IGNORE NULLS + DATE_BUCKET ở mức 150 = 10/07:30
PASS 7d mức tương thích sau khi đặt lại = 160
```

`IGNORE NULLS` và `DATE_BUCKET` (MS-016) vẫn chạy ở mức 150; `WINDOW` và `GENERATE_SERIES` thì không. Một chi tiết khi thử: đổi mức tương thích bằng `ALTER DATABASE CURRENT SET COMPATIBILITY_LEVEL = 150` rồi chạy câu thử **trong cùng batch** thì câu đó vẫn chạy theo mức cũ, nên lab tách mỗi bước thành một batch `GO` riêng. Nâng mức tương thích trên database thật là một thay đổi cần kế hoạch và kiểm thử, MS-059 (Phần 6) bàn riêng.

## 9. Làm trong DBStudio

- Mỗi câu `SELECT` trong `15-window-function.sql` là một result set riêng; dòng `PASS` nằm ở tab **Thông điệp**. Chạy cả tệp bằng `Ctrl+Shift+Enter`.
- Bảng tạm `#dn` (lịch Đà Nẵng) tạo ở batch đầu và được mục 6 dùng lại: trong cùng một tab, bạn chạy từng batch bằng `Ctrl+Enter` theo thứ tự mà không mất nó. Mở tab mới là phiên mới, phải chạy lại batch tạo `#dn`.
- Bảng xếp hạng tài xế export được ra CSV hoặc XLSX để gửi đội vận hành.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Trung bình 7 ngày trống hoặc cao bất thường quanh ngày không có dữ liệu | `ROWS` trên dữ liệu thưa đếm dòng, không đếm ngày | Lịch đủ ngày (`GENERATE_SERIES` + `LEFT JOIN`) |
| Lỗi 4194 với `RANGE … 6 PRECEDING` | SQL Server chỉ cho `RANGE` với `UNBOUNDED`/`CURRENT ROW` | Lịch đủ ngày + `ROWS` |
| Luỹ kế nhảy nhiều dòng một lúc | Frame mặc định `RANGE` gộp dòng hoà | `ROWS UNBOUNDED PRECEDING` + tie-breaker |
| "Top 3" ra 4 hoặc 5 người | `DENSE_RANK`/`RANK` có hoà | Chọn hàm theo câu hỏi; `ROW_NUMBER` + quy tắc phân xử |
| Lỗi 4108 | Hàm cửa sổ trong `WHERE`/`HAVING` | Tính ở truy vấn trong, lọc ở ngoài |
| `LAST_VALUE` trả chính dòng hiện tại | Frame mặc định dừng ở dòng hiện tại | `ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING` |
| Trung bình mất phần lẻ, tỉ trọng ra 0 | Phép tính trên `int` | `CAST(… AS decimal)`, `100.0 *` |
| `Incorrect syntax near 'w'` dù đúng cú pháp | Database ở mức tương thích dưới 160 | Kiểm `sys.databases.compatibility_level` |

## 11. Đánh đổi

- **Lịch đủ ngày hay dữ liệu thưa:** lịch tốn một bước JOIN nhưng làm mọi phép "N ngày" đúng nghĩa và cho ngày trống hiện trên biểu đồ; dữ liệu thưa chỉ ổn khi chắc chắn ngày nào cũng có dòng.
- **NULL hay 0 cho ngày trống:** 0 đúng cho tổng và trung bình theo ngày lịch; NULL giữ được thông tin "không có hoạt động" cho `IGNORE NULLS` và cho người đọc.
- **`ROWS` hay `RANGE`:** `ROWS` cộng tie-breaker cho kết quả xác định và rẻ hơn khi thực thi; `RANGE` chỉ nên dùng khi muốn mọi dòng hoà nhận chung một giá trị, và khi đó ghi nó ra thay vì dựa vào mặc định.
- **Mệnh đề `WINDOW` hay viết lặp `OVER (…)`:** `WINDOW` ngắn và tránh định nghĩa lệch nhau giữa các cột, nhưng bắt buộc mức tương thích 160; mã chạy trên database chưa nâng mức thì viết lặp.

## 12. So với PostgreSQL

Cùng dữ liệu với [PG-015](/articles/postgresql-window-function-frame); các con số trùng khớp (672.571 / NULL / 797.143, lô 3 → 9, 8101 và 8109 cùng hạng 1, tỉ trọng 50,95 / 34,56 / 14,49 %). Khác biệt thật:

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| `RANGE` theo khoảng cách | `RANGE BETWEEN interval '6 days' PRECEDING …` | Không có (lỗi 4194); dùng lịch đủ ngày + `ROWS` |
| Frame `GROUPS`, `EXCLUDE` | Có | Không có |
| Mệnh đề `WINDOW` | Có từ lâu | Mới ở 2022, cần mức tương thích 160 |
| `IGNORE NULLS` | Không có | `FIRST_VALUE`/`LAST_VALUE` (và `LAG`/`LEAD` trên bản lab) |
| Sinh dãy ngày | `generate_series(date, date, interval)` trả `timestamptz` | `GENERATE_SERIES(start, stop)` trả số; cộng bằng `DATEADD` |
| Hiệu hai thời điểm | `interval` | `DATEDIFF(đơn vị, a, b)` trả số nguyên |
| Hàm cửa sổ trong `WHERE` | Lỗi `42P20` | Lỗi 4108 |

## 13. Bài tập

**Bài 1.** Lấy **một** chuyến hoàn thành có cước cao nhất ở mỗi thành phố; hoà thì lấy mã nhỏ hơn. Sau đó đổi sang `RANK()` và giải thích vì sao số dòng tăng.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT tp, code
FROM (SELECT c.code AS tp, t.code,
             ROW_NUMBER() OVER (PARTITION BY t.city_id ORDER BY t.fare DESC, t.code) AS n
      FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
      WHERE t.status = 'completed') AS x
WHERE n = 1
ORDER BY tp;</code></pre>
<p>Kết quả đã chạy: DN 101355, HN 101699, SG 101601. Đổi sang <code>RANK() OVER (PARTITION BY t.city_id ORDER BY t.fare DESC)</code> và lọc hạng 1 thì ra <b>11</b> chuyến: 2 ở DN, 4 ở HN, 5 ở SG, vì cước tối đa (150.000 đ ở DN và SG, 149.000 đ ở HN) xuất hiện nhiều lần. Cả hai có assertion <code>BT1</code> trong lab.</p>
</details>

**Bài 2.** Với khách 0910000016, tìm khoảng nghỉ dài nhất giữa hai chuyến liên tiếp trong tháng: in mã chuyến trước, mã chuyến sau và số phút.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT TOP (1) code, chuyen_sau, phut
FROM (SELECT t.code,
             LEAD(t.code) OVER w AS chuyen_sau,
             DATEDIFF(minute, t.requested_at, LEAD(t.requested_at) OVER w) AS phut
      FROM toinoi.trips AS t JOIN toinoi.customers AS c ON c.id = t.customer_id
      WHERE c.phone = '0910000016'
      WINDOW w AS (ORDER BY t.requested_at, t.code)) AS x
WHERE phut IS NOT NULL
ORDER BY phut DESC, code;</code></pre>
<p>Kết quả đã chạy: 102372 → 102664, <b>3.477 phút</b> (từ 12:46 ngày 23/10 tới 22:43 ngày 25/10). Chuyến cuối tháng có <code>LEAD</code> NULL nên bị loại bằng <code>phut IS NOT NULL</code>; <code>code</code> trong <code>ORDER BY</code> phân xử nếu hai khoảng nghỉ bằng nhau (assertion <code>BT2</code>).</p>
</details>

**Bài 3.** Tính tỉ trọng doanh thu của từng loại dịch vụ **trong** Đà Nẵng (tổng ba dịch vụ của Đà Nẵng là 100 %).

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT c.code AS tp, t.kind,
       CAST(ROUND(100.0 * SUM(t.fare) / SUM(SUM(t.fare)) OVER (PARTITION BY c.code), 2) AS decimal(5, 2)) AS pct
FROM toinoi.trips AS t JOIN toinoi.cities AS c ON c.id = t.city_id
WHERE t.status = 'completed' AND c.code = 'DN'
GROUP BY c.code, t.kind
ORDER BY t.kind;</code></pre>
<p>Kết quả đã chạy: food 56,41 %, parcel 12,80 %, ride 30,79 % (assertion <code>BT3</code>; lab tính cho cả ba thành phố rồi lọc DN, kết quả như nhau vì cửa sổ đã chia theo thành phố). <code>PARTITION BY c.code</code> làm mẫu số là tổng của thành phố thay vì tổng cả hệ thống.</p>
</details>

## 14. Đọc tiếp

- Microsoft Learn: [OVER clause](https://learn.microsoft.com/sql/t-sql/queries/select-over-clause-transact-sql) (gồm ROWS/RANGE và frame mặc định), [WINDOW clause](https://learn.microsoft.com/sql/t-sql/queries/select-window-transact-sql), [ROW_NUMBER](https://learn.microsoft.com/sql/t-sql/functions/row-number-transact-sql), [RANK](https://learn.microsoft.com/sql/t-sql/functions/rank-transact-sql), [DENSE_RANK](https://learn.microsoft.com/sql/t-sql/functions/dense-rank-transact-sql), [NTILE](https://learn.microsoft.com/sql/t-sql/functions/ntile-transact-sql), [LAG](https://learn.microsoft.com/sql/t-sql/functions/lag-transact-sql), [LEAD](https://learn.microsoft.com/sql/t-sql/functions/lead-transact-sql), [FIRST_VALUE](https://learn.microsoft.com/sql/t-sql/functions/first-value-transact-sql), [LAST_VALUE](https://learn.microsoft.com/sql/t-sql/functions/last-value-transact-sql), [GENERATE_SERIES](https://learn.microsoft.com/sql/t-sql/functions/generate-series-transact-sql), [ALTER DATABASE compatibility level](https://learn.microsoft.com/sql/t-sql/statements/alter-database-transact-sql-compatibility-level).
- Cùng chủ đề trên PostgreSQL: [PG-015 · Window function và frame](/articles/postgresql-window-function-frame).
- Lab: `examples/toi-noi-mssql/p02/15-window-function.sql`. Chi phí của frame và `Worktable` trong execution plan: MS-021 (Phần 3).

Bài tiếp theo: [MS-016 · Ngày giờ, AT TIME ZONE, DATETRUNC và DATE_BUCKET](/articles/mssql-ngay-gio-at-time-zone-date-bucket)
