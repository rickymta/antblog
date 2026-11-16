---
slug: mssql-join-apply-cardinality
title: "JOIN, APPLY và số dòng sau khi nối: tổng tiền Tới Nơi không bị nhân đôi"
summary: "Dashboard mới của Tới Nơi báo doanh thu tháng 10 là 191.644.000 đ trong khi sổ ghi 182.287.000 đ, chỉ vì thêm một dòng JOIN payments; màn hình \"2 chuyến gần nhất của mỗi tài xế\" làm mất hai tài xế mới. Bài đi qua INNER/LEFT JOIN, đếm dòng trước và sau khi nối, gom phía nhiều trước khi cộng tiền, NOT EXISTS và CROSS/OUTER APPLY trên SQL Server 2022."
category: mssql
level: beginner
tags: [ms-013, mssql, sql-server-2022, t-sql, join, cross-apply, outer-apply, cardinality, exists, toi-noi]
language: vi
---

Đội dữ liệu của Tới Nơi thêm một dòng `JOIN toinoi.payments` vào câu doanh thu của dashboard để hiện phương thức thanh toán bên cạnh. Sáng hôm sau, doanh thu tháng 10 trên dashboard là **191.644.000 đ**, trong khi bảng tổng hợp của [MS-012](/articles/mssql-group-by-rollup-grouping-sets) ghi **182.287.000 đ**. Không ai sửa phép cộng; chỉ có thêm một bảng.

Cùng tuần, đội vận hành Hà Nội nhận màn hình mới "2 chuyến hoàn thành gần nhất của mỗi tài xế" để gọi điện xác nhận chất lượng. Danh sách có 18 tài xế; hai tài xế vừa vào đội ngày 29/10, 8219 và 8220, chưa chạy chuyến nào và không xuất hiện ở đâu cả, nên cũng không ai gọi chào đón họ.

Cả hai lỗi cùng một gốc: không hỏi **mỗi dòng bên trái khớp bao nhiêu dòng bên phải** trước khi nối. Bài này đi qua `JOIN`, cách đếm dòng trước và sau khi nối, mẫu "gom trước rồi mới nối", anti-join bằng `NOT EXISTS`, và toán tử `APPLY` của SQL Server để lấy "N dòng mới nhất mỗi nhóm" mà không đánh rơi nhóm rỗng.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-013 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/mssql-null-case-isnull-coalesce">MS-011 · NULL, CASE, ISNULL và COALESCE</a>, <a href="/articles/mssql-group-by-rollup-grouping-sets">MS-012 · GROUP BY, ROLLUP và GROUPING SETS</a>, <a href="/articles/mssql-mo-hinh-quan-he-chuan-hoa">MS-008 · chuẩn hoá và quan hệ giữa các bảng</a>. Lab: <code>examples/toi-noi-mssql/p02</code> (tệp <code>13-join-apply.sql</code>). Sau bài này, bạn nối bảng mà không nhân đôi tổng tiền, tìm được dòng bị rơi và lấy N dòng mới nhất cho từng nhóm bằng <code>CROSS APPLY</code>/<code>OUTER APPLY</code>.</p>
</div>

## 1. Dữ liệu dùng trong bài

Fixture Phần 2 là ảnh chụp lúc **01:00 ngày 31/10/2026 giờ VN**: 3.174 chuyến, trong đó **2.707 chuyến hoàn thành** với tổng cước **182.287.000 đ** (MS-012 đã đối chiếu).

```bash
cd examples/toi-noi-mssql
node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/13-join-apply.sql
```

Kết quả chép từ lần chạy trên SQL Server 2022 16.0.4325.4 (RTM-CU28); tệp lab có 39 kiểm tra, tất cả PASS.

| Quan hệ | Mỗi chuyến có | Vì sao biết |
|---|---|---|
| `trips` → `payments` | 0, 1 hoặc 2 dòng (485 / 2.552 / 137 chuyến) | `payments.trip_id` **không** duy nhất |
| `trips` → `trip_events` | 1–7 dòng lịch sử (chuyến hoàn thành: 7) | `trip_events.trip_id` không duy nhất |
| `trips` → `trip_ratings` | 0 hoặc 1 dòng | `trip_id` là **khoá chính** của `trip_ratings` |
| `trips` → `cities`, `drivers`, `merchants` | 1 (hoặc 0 khi khoá ngoại NULL) | khoá ngoại trỏ tới khoá chính |

Đây là toàn bộ bí mật của bài: trước khi nối, phải biết phía bên kia có **tối đa mấy dòng** cho mỗi chuyến.

## 2. JOIN ghép dòng theo cặp

**JOIN** ghép dòng của hai bảng theo một điều kiện, thường là "khoá ngoại bằng khoá chính". Với **INNER JOIN** (viết tắt `JOIN`), mỗi dòng bên trái cho ra **một dòng cho mỗi dòng bên phải** thoả điều kiện: không có dòng nào thì dòng bên trái biến mất, có hai dòng thì dòng bên trái xuất hiện hai lần. **LEFT JOIN** (hay `LEFT OUTER JOIN`) làm như INNER JOIN rồi thêm một dòng có cột bên phải là NULL cho mỗi dòng bên trái không khớp: *ít nhất* một dòng cho mỗi dòng bên trái, nhưng vẫn có thể nhiều hơn.

Số dòng bên phải ứng với một dòng bên trái là **cardinality** của quan hệ: một–một, một–không-hoặc-một, một–nhiều, nhiều–một. Lấy bốn chuyến hoàn thành: 100041 trả thẻ một lần; 100042 bị thẻ từ chối rồi thu lại; 100048 trả một phần bằng ví, phần còn lại bằng thẻ; 100724 không có giao dịch nào.

```sql
SELECT t.code, t.fare, p.method, p.status, p.amount
FROM toinoi.trips AS t
JOIN toinoi.payments AS p ON p.trip_id = t.id
WHERE t.code IN (100041, 100042, 100048, 100724)
ORDER BY t.code, p.id;
```

```text
code|fare|method|status|amount
----|----|------|------|------
100041|127000|card|succeeded|127000
100042|82000|card|failed|82000
100042|82000|card|succeeded|82000
100048|119000|wallet|succeeded|104000
100048|119000|card|succeeded|15000
```

Bốn chuyến nở thành năm dòng, và 100724 biến mất. Đổi sang `LEFT JOIN` thì 100724 quay lại với ba cột NULL, thành 6 dòng. Cộng `t.fare`: đúng là **404.000**, sau INNER JOIN thành **529.000**, sau LEFT JOIN thành **605.000** (lab kiểm cả năm số). JOIN không "sai": nó trả đúng một dòng cho mỗi cặp khớp. Sai ở chỗ `SUM(t.fare)` giả định mỗi chuyến một dòng, trong khi sau khi nối **một dòng là một cặp (chuyến, giao dịch)**.

## 3. Đếm trước, đếm sau

Thói quen đầu tiên: đếm phía bạn quan tâm, rồi đếm lại sau khi nối.

```text
chuyen_hoan_thanh|dong_inner_join|dong_left_join
-----------------|---------------|--------------
2707|2800|2832
```

2.707 chuyến hoàn thành cho **2.800** dòng sau INNER JOIN, bằng số giao dịch của chúng chứ không phải số chuyến. LEFT JOIN cho **2.832**: thêm 32 chuyến không có giao dịch, mỗi chuyến một dòng NULL. Báo cáo "theo chuyến" mà số dòng sau khi nối khác số chuyến thì mọi `SUM` trên cột của chuyến đều đáng ngờ.

## 4. Tổng tiền bị nhân như thế nào

Câu truy vấn của dashboard, rút gọn:

```sql
SELECT COUNT(*)                AS so_dong,
       COUNT(DISTINCT t.id)    AS so_chuyen,
       SUM(t.fare)             AS tong_cuoc_SAI,
       SUM(p.amount)           AS tong_gd_moi_trang_thai
FROM toinoi.trips AS t
JOIN toinoi.payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed';
```

```text
so_dong|so_chuyen|tong_cuoc_SAI|tong_gd_moi_trang_thai
-------|---------|-------------|----------------------
2800|2675|191644000|189686000
```

Khoản chênh 9.357.000 đ gồm hai phần ngược chiều, lab tính riêng từng phần: **+9.433.000 đ** là cước của 125 chuyến có 2 dòng giao dịch, bị cộng thêm một lần; **−76.000 đ** là cước của 32 chuyến không có dòng nào nên rơi khỏi INNER JOIN (31 chuyến cước 0 đ và 100724). 182.287.000 + 9.433.000 − 76.000 = 191.644.000. Hai lỗi bù trừ một phần, nên "tổng gần đúng" không chứng minh câu lệnh đúng. Cột `tong_gd_moi_trang_thai` cũng không phải tiền đã thu: nó cộng cả `failed`, `pending` và cộng tiền hoàn như tiền thu.

Thêm `DISTINCT` không cứu được: `SUM(DISTINCT t.fare)` loại trùng theo **giá trị cước**, ra **11.259.000 đ**, vì hai chuyến cùng 50.000 đ chỉ được cộng một lần. Thứ bị trùng là *chuyến*, thứ bị loại lại là *số tiền*.

## 5. Câu hỏi phải đặt trước mỗi JOIN

> Với mỗi dòng bên trái, bên phải có **tối đa** mấy dòng?

**Đọc ràng buộc.** Cột nối bên phải là khoá chính hoặc có `UNIQUE` thì câu trả lời là "tối đa 1", cho cả dữ liệu tương lai. `trip_ratings` có khoá chính `trip_id`, nên `trips LEFT JOIN trip_ratings` giữ nguyên 2.707 dòng (lab kiểm). Chiều **nhiều–một** an toàn vì cùng lý do: nối chuyến hoàn thành với `cities`, `drivers`, `merchants` thì trước và sau đều 2.707 dòng.

**Đếm dữ liệu.** `payments.trip_id` không duy nhất: một chuyến *được phép* có nhiều giao dịch. Phân bố trong chuyến hoàn thành:

```text
so_dong_gd|so_chuyen
----------|---------
1|2550
2|125
```

Đây chỉ là bằng chứng *tạm thời*: hôm nay tối đa 2, ngày mai một khách có thể trả bằng ba phương thức. Mẫu kiểm tra nhanh cho mọi khoá nối:

```sql
SELECT trip_id FROM toinoi.payments GROUP BY trip_id HAVING COUNT(*) > 1;
```

Trên toàn bảng, `payments.trip_id` có 137 giá trị lặp, `trip_events.trip_id` có 3.173, `trip_ratings.trip_id` có 0. Số 0 chỉ nói về dữ liệu hôm nay; muốn chắc mãi mãi thì cần ràng buộc, như khoá chính của `trip_ratings`.

## 6. Cách đúng: gom phía "nhiều" trước, rồi mới nối

Báo cáo ở mức **chuyến** thì đưa `payments` về **một dòng mỗi chuyến** trước khi nối:

```sql
SELECT COUNT(*)                       AS so_dong,
       COUNT(DISTINCT t.id)           AS so_chuyen,
       SUM(t.fare)                    AS tong_cuoc,
       SUM(ISNULL(g.thuc_thu, 0))     AS tong_thuc_thu
FROM toinoi.trips AS t
LEFT JOIN (
    SELECT trip_id,
           SUM(CASE kind WHEN 'refund' THEN -amount ELSE amount END) AS thuc_thu
    FROM toinoi.payments
    WHERE status = 'succeeded'
    GROUP BY trip_id
) AS g ON g.trip_id = t.id
WHERE t.status = 'completed';
```

```text
so_dong|so_chuyen|tong_cuoc|tong_thuc_thu
-------|---------|---------|-------------
2707|2707|182287000|182531000
```

Bảng dẫn xuất `g` có `GROUP BY trip_id` nên mỗi chuyến xuất hiện tối đa một lần; `LEFT JOIN` giữ các chuyến không có giao dịch thành công, `ISNULL(…, 0)` coi chúng là 0 đ (MS-011: ở đây là tiền, nên 0 đúng nghĩa). Bằng chứng nằm ngay trong kết quả: `so_dong` = `so_chuyen` = 2.707 và `tong_cuoc` khớp MS-012.

Thực thu của Tới Nơi là Σ charge − Σ refund + Σ adjustment, chỉ tính `succeeded` (adjustment đã mang dấu). Con số này lớn hơn cước 244.000 đ. Đó không còn là lỗi JOIN mà là **sai lệch thật** của vài chuyến; tìm từng chuyến là việc của [MS-020](/articles/mssql-thuc-hanh-tsql-doi-soat).

## 7. APPLY: chạy truy vấn con cho từng dòng

**`APPLY`** là toán tử riêng của T-SQL trong mệnh đề `FROM`: với **mỗi dòng** của bảng bên trái, SQL Server chạy biểu thức bảng bên phải, và biểu thức đó được dùng cột của dòng bên trái (điều mà một bảng dẫn xuất sau `JOIN` không làm được). **`CROSS APPLY`** chỉ giữ dòng bên trái nào cho ra ít nhất một dòng, giống INNER JOIN; **`OUTER APPLY`** giữ mọi dòng bên trái, điền NULL khi bên phải rỗng, giống LEFT JOIN.

Cách gom ở mục 6 viết lại bằng `APPLY` cho thấy một chi tiết về số dòng dễ bỏ sót:

```sql
FROM toinoi.trips AS t
CROSS APPLY (SELECT SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END) AS thuc_thu
             FROM toinoi.payments AS p
             WHERE p.trip_id = t.id AND p.status = 'succeeded') AS g      -- (1) không GROUP BY
-- (2) như (1) nhưng thêm GROUP BY p.trip_id
-- (3) như (2) nhưng OUTER APPLY
```

```text
cross_apply_khong_group_by|cross_apply_group_by|outer_apply_group_by
--------------------------|--------------------|--------------------
2707|2674|2707
```

- **(1)** Hàm tổng hợp **không có `GROUP BY`** luôn trả đúng một dòng, kể cả khi không dòng nào khớp (giá trị là NULL, như tập rỗng ở MS-011). Vì vậy `CROSS APPLY` giữ cả 2.707 chuyến, và tổng thực thu ra đúng 182.531.000 đ.
- **(2)** Thêm `GROUP BY p.trip_id`: không dòng khớp thì không có nhóm nào, tức 0 dòng, và `CROSS APPLY` bỏ chuyến đó. Còn 2.674 chuyến.
- **(3)** `OUTER APPLY` giữ lại cả 33 chuyến rơi ở (2).

Quy tắc nhớ: đọc phần bên phải của `APPLY` như một truy vấn độc lập và hỏi "với một chuyến không có giao dịch, câu này trả mấy dòng?". 0 dòng thì `CROSS APPLY` làm rơi chuyến.

## 8. Dòng bị rơi, và điều kiện ở ON hay WHERE

Lỗi đối xứng với "nở dòng" là **mất dòng**. Có khi dòng mất chính là thứ cần tìm: **anti-join** là phép lấy các dòng bên trái **không** có dòng khớp bên phải. Viết bằng `NOT EXISTS` là cách nói thẳng ý định nhất ([MS-014](/articles/mssql-subquery-exists-cte-temp-table) đi sâu vào `EXISTS`):

```sql
SELECT CASE WHEN t.fare = 0 THEN N'cước 0 đ — không cần thu' ELSE N'CÓ cước nhưng chưa thu' END AS loai,
       COUNT(*) AS so_chuyen,
       STRING_AGG(CASE WHEN t.fare > 0 THEN CONVERT(varchar(10), t.code) END, ', ')
           WITHIN GROUP (ORDER BY t.code) AS ma_chuyen
FROM toinoi.trips AS t
WHERE t.status = 'completed'
  AND NOT EXISTS (SELECT 1 FROM toinoi.payments AS p WHERE p.trip_id = t.id AND p.status = 'succeeded')
GROUP BY CASE WHEN t.fare = 0 THEN N'cước 0 đ — không cần thu' ELSE N'CÓ cước nhưng chưa thu' END
ORDER BY so_chuyen DESC;
```

```text
loai|so_chuyen|ma_chuyen
----|---------|---------
cước 0 đ — không cần thu|31|NULL
CÓ cước nhưng chưa thu|2|100724, 101084
```

31 chuyến cước 0 đ là đúng nghiệp vụ; 100724 (không có giao dịch nào) và 101084 (chỉ có lần thu thẻ `failed`) là sai lệch. Dạng cũ `LEFT JOIN payments … WHERE p.id IS NULL` cũng ra 33 (lab kiểm); `p.id` là khoá chính nên NULL ở đó chỉ có thể do LEFT JOIN sinh ra. `STRING_AGG` bỏ qua NULL, nên nhóm cước 0 đ có chuỗi NULL.

Để ý `p.status = 'succeeded'` ở dạng LEFT JOIN phải nằm trong **`ON`**:

```text
dieu_kien_o_WHERE|dieu_kien_o_ON
-----------------|--------------
2674|2707
```

Ở `WHERE`, dòng NULL do LEFT JOIN sinh ra gặp `NULL = 'succeeded'`, ra UNKNOWN, nên bị loại; LEFT JOIN biến thành INNER JOIN mà không báo lỗi. Điều kiện ở `ON` quyết định dòng nào **khớp**; điều kiện ở `WHERE` lọc kết quả **sau khi** nối. Quy tắc nhớ: điều kiện chọn dòng của bảng bên phải đặt ở `ON`, của bảng bên trái đặt ở `WHERE`.

## 9. Hai quan hệ một–nhiều trong cùng một câu

Nối `trips` với cả `payments` lẫn `trip_events` thì hai phía "nhiều" **nhân** với nhau: mỗi giao dịch × mỗi dòng lịch sử.

```text
code|so_gd|so_su_kien|so_dong_sau_noi|cuoc|sum_amount_SAI
----|-----|----------|---------------|----|--------------
100042|2|7|14|82000|1148000
```

Chuyến 100042 có 2 giao dịch và 7 sự kiện nên ra 2 × 7 = **14** dòng, và `SUM(p.amount)` phình thành 82.000 × 2 × 7 = 1.148.000 đ. Trên cả tháng, tổng thu thành công 183.640.000 đ (dòng `charge`, mọi chuyến) phình thành **1.283.272.000 đ**, gần gấp 7. Con số đó vẫn vừa kiểu `int` của `SUM`; thêm vài tháng dữ liệu là câu sai này còn kèm lỗi tràn 8115 của MS-012. Cách sửa vẫn là mục 6: gom **từng** phía nhiều trong bảng dẫn xuất (hoặc `APPLY`) riêng, rồi mới nối với `trips`.

## 10. Câu hỏi "có hay không": đừng JOIN rồi đếm

"Bao nhiêu chuyến hoàn thành từng dùng ví VíXanh?" là câu hỏi về **chuyến**:

```text
join_dem_dong|join_distinct|exists_dem_chuyen
-------------|-------------|-----------------
1050|961|961
```

JOIN rồi `COUNT(*)` ra **1.050**, tức số dòng giao dịch ví, gồm cả `failed` và chuyến có hai dòng ví. `COUNT(DISTINCT t.id)` ra đúng **961** nhưng vẫn tạo 1.050 dòng rồi loại trùng. `WHERE EXISTS (SELECT 1 FROM toinoi.payments AS p WHERE p.trip_id = t.id AND p.method = 'wallet')` nói thẳng ý định: `EXISTS` chỉ hỏi "có ít nhất một dòng không", nên mỗi chuyến được tính một lần.

## 11. Top-N mỗi nhóm bằng APPLY

Màn hình "2 chuyến hoàn thành gần nhất của mỗi tài xế" là bài toán **top-N mỗi nhóm**. `JOIN` không có chỗ đặt `TOP (2)` cho từng tài xế; `APPLY` thì có, vì truy vấn bên phải chạy riêng cho từng tài xế:

```sql
SELECT d.code AS tai_xe, x.code AS chuyen, CAST(x.requested_at AS varchar(34)) AS requested_at, x.fare
FROM toinoi.drivers AS d
OUTER APPLY (SELECT TOP (2) t.code, t.requested_at, t.fare
             FROM toinoi.trips AS t
             WHERE t.driver_id = d.id AND t.status = 'completed'
             ORDER BY t.requested_at DESC, t.code DESC) AS x
WHERE d.code BETWEEN 8217 AND 8220
ORDER BY d.code, x.requested_at DESC, x.code DESC;
```

```text
tai_xe|chuyen|requested_at|fare
------|------|------------|----
8217|103139|2026-10-30 18:04:00.000 +07:00|88000
8217|103117|2026-10-30 13:12:00.000 +07:00|60000
8218|103153|2026-10-30 19:44:00.000 +07:00|100000
8218|103131|2026-10-30 16:52:00.000 +07:00|14000
8219|NULL|NULL|NULL
8220|NULL|NULL|NULL
```

Màn hình của đội vận hành dùng `CROSS APPLY`: với 20 tài xế Hà Nội, câu đó trả **36** dòng (18 người × 2), còn `OUTER APPLY` trả **38**, giữ 8219 và 8220 với cột NULL (lab kiểm cả ba số). Chọn `CROSS` hay `OUTER` là câu hỏi nghiệp vụ "tài xế chưa có chuyến có thuộc danh sách không?", không phải chuyện cú pháp. Hai chi tiết khác: `TOP` bên trong `APPLY` cần `ORDER BY` có tie-breaker (`code`), nếu không "2 chuyến gần nhất" có thể đổi giữa hai lần chạy khi hai chuyến cùng giờ đặt (MS-007); và tài xế có ít hơn 2 chuyến thì chỉ được số dòng có thật.

Cùng mẫu đó trả lời "giao dịch mới nhất của mỗi chuyến", một dòng mỗi chuyến kể cả chuyến không có giao dịch:

```sql
SELECT t.code, t.fare, lp.method, lp.status, lp.amount
FROM toinoi.trips AS t
OUTER APPLY (SELECT TOP (1) p.method, p.status, p.amount
             FROM toinoi.payments AS p
             WHERE p.trip_id = t.id
             ORDER BY p.paid_at DESC, p.id DESC) AS lp
WHERE t.code IN (100041, 100042, 100048, 100724)
ORDER BY t.code;
```

```text
code|fare|method|status|amount
----|----|------|------|------
100041|127000|card|succeeded|127000
100042|82000|card|succeeded|82000
100048|119000|card|succeeded|15000
100724|76000|NULL|NULL|NULL
```

Đây là chỗ `APPLY` hơn `JOIN` rõ nhất: kết quả có đúng một dòng mỗi chuyến **theo cấu trúc câu lệnh**, không cần gom hay loại trùng sau đó. Lưu ý 100048 chỉ hiện giao dịch thẻ 15.000 đ: "mới nhất" không phải "tổng", nên đừng cộng cột `amount` này thành doanh thu. Window function với `ROW_NUMBER()` là cách thứ hai cho top-N mỗi nhóm, sẽ gặp ở [MS-015](/articles/mssql-window-function-window-clause); hiệu năng của hai cách (index phù hợp cho `APPLY` + `TOP`) để dành cho Phần 3.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn bước trước khi tin một tổng sau JOIN</span>
<p>(1) Viết ra <b>grain</b> của câu trả lời, tức mỗi dòng đại diện cho cái gì: chuyến, giao dịch hay khách. (2) Với mỗi bảng được nối, trả lời "tối đa mấy dòng mỗi dòng bên trái" bằng ràng buộc (khoá chính, <code>UNIQUE</code>); không có ràng buộc thì gom về một dòng trước, hoặc dùng <code>APPLY</code> với hàm tổng hợp hay <code>TOP (1)</code>. (3) Đếm số dòng trước và sau khi nối; chênh lệch phải giải thích được bằng con số. (4) Đối chiếu tổng với một truy vấn độc lập không JOIN, ở đây là 182.287.000 đ cước tháng 10.</p>
</div>

## 12. Làm trong DBStudio

- Mỗi mục của `13-join-apply.sql` là một batch `GO`; bôi đen riêng câu "LỖI KINH ĐIỂN" ở mục 2 rồi `Ctrl+Enter` để thấy con số 191.644.000 đ, sau đó chạy câu gom trước ở mục 4 để so.
- Khi thử một câu JOIN mới trên dữ liệu thật, chạy trước một câu `COUNT(*)` cùng `FROM … JOIN … WHERE`; nếu con số lớn hơn số dòng bên trái, dừng lại trước khi viết `SUM`. `Ctrl+.` huỷ câu đang chạy nếu bạn lỡ nối hai bảng lớn mà quên điều kiện.

## 13. So với PostgreSQL

Cùng câu hỏi, cùng dữ liệu với [PG-013](/articles/postgresql-join-quan-he-cardinality): mọi con số của mục 2–10 giống hệt, vì cardinality là tính chất của dữ liệu chứ không của engine. Khác biệt nằm ở cú pháp:

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| Truy vấn con chạy theo từng dòng bên trái | `CROSS JOIN LATERAL (…)`, `LEFT JOIN LATERAL (…) ON true` | `CROSS APPLY (…)`, `OUTER APPLY (…)` |
| N dòng đầu trong truy vấn con | `ORDER BY … LIMIT n` | `TOP (n) … ORDER BY …` |
| Một dòng mỗi nhóm theo thứ tự | `DISTINCT ON (khoá) … ORDER BY` | `OUTER APPLY (SELECT TOP (1) …)` hoặc `ROW_NUMBER()` |
| Đếm có điều kiện | `count(*) FILTER (WHERE …)` | `SUM(CASE WHEN … THEN 1 ELSE 0 END)` |
| Kiểu của `SUM` trên số nguyên | `bigint` | `int`: tổng phình sau JOIN có thể thành lỗi 8115 |

## 14. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Tổng tăng sau khi "chỉ thêm một JOIN" | Cột tiền của phía "một" lặp theo phía "nhiều" | Gom phía nhiều về một dòng mỗi khoá rồi nối |
| Tổng giảm, vài chuyến biến mất | INNER JOIN bỏ chuyến không có dòng bên phải | `LEFT JOIN` và xử lý NULL có chủ đích |
| `LEFT JOIN` ra cùng số dòng với `JOIN` | Điều kiện của bảng phải nằm ở `WHERE` | Chuyển điều kiện đó vào `ON` |
| `SUM(DISTINCT …)` nhỏ bất thường | Loại trùng theo giá trị, không theo dòng | Loại trùng theo khoá (gom trước) |
| Tổng phình gấp nhiều lần | Hai quan hệ một–nhiều nối cùng lúc | Gom từng phía trong truy vấn con riêng |
| Danh sách top-N thiếu nhóm rỗng | `CROSS APPLY` bỏ dòng bên trái khi bên phải rỗng | `OUTER APPLY` |
| `CROSS APPLY` làm rơi chuyến không có giao dịch | Truy vấn bên phải có `GROUP BY`, trả 0 dòng | Bỏ `GROUP BY` (tổng hợp vô hướng) hoặc `OUTER APPLY` |
| "2 dòng mới nhất" đổi giữa hai lần chạy | `TOP` trong `APPLY` thiếu tie-breaker | `ORDER BY thời_điểm DESC, code DESC` |

## 15. Đánh đổi

- **Gom trước hay `COUNT(DISTINCT …)` sau:** `COUNT(DISTINCT khoá)` đủ để đếm nhưng không cứu được `SUM` của phía "một"; gom trước dài hơn vài dòng và đúng cho mọi phép tổng hợp.
- **Bảng dẫn xuất + `JOIN` hay `APPLY`:** bảng dẫn xuất gom một lần cho cả bảng, dễ đọc khi cần nhiều cột tổng; `APPLY` viết được `TOP (n)`, "dòng mới nhất" và điều kiện phụ thuộc dòng bên trái. Cả hai cho cùng kết quả ở mục 6–7; chênh lệch hiệu năng phụ thuộc index và kế hoạch thực thi, đo ở Phần 3.
- **INNER hay LEFT, CROSS hay OUTER:** bản "INNER/CROSS" ngắn và đúng khi phía phải chắc chắn có dòng; bản "LEFT/OUTER" an toàn hơn nhưng phải xử lý NULL ở mọi cột bên phải.
- **Một câu lớn hay nhiều bước:** một câu nhiều JOIN tiện cho dashboard nhưng khó kiểm tra; MS-014 tách nó thành các bước CTE và bảng tạm có số dòng để đối chiếu.

## 16. Bài tập

Lời giải đã chạy trong phần "Bài tập" ở cuối `13-join-apply.sql` (nền như mục 1).

**Bài 1.** Kế toán muốn biết **tiền đã thu** qua từng phương thức (chuyến hoàn thành, giao dịch thành công). Đồng nghiệp viết `SUM(t.fare) … GROUP BY p.method` sau khi nối `trips` với `payments`. Chỉ ra chỗ sai và viết lại.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT p.method,
       COUNT(*)                                                         AS so_gd,
       SUM(t.fare)                                                      AS sai_sum_fare,
       SUM(CASE p.kind WHEN 'refund' THEN -p.amount ELSE p.amount END)  AS da_thu
FROM toinoi.trips AS t
JOIN toinoi.payments AS p ON p.trip_id = t.id
WHERE t.status = 'completed' AND p.status = 'succeeded'
GROUP BY p.method
ORDER BY p.method;</code></pre>
<p>Kết quả đã chạy: card <code>686 | 48371000 | 46932000</code>, cash <code>1054 | 69538000 | 69524000</code>, wallet <code>963 | 66580000 | 66075000</code>. Cột sai gán <b>cả cước chuyến</b> cho mỗi dòng giao dịch: 100048 được tính 119.000 đ ở ví và 119.000 đ ở thẻ; tổng cột sai là 184.489.000 đ, lớn hơn cả cước lẫn thực thu. Grain ở đây là <b>giao dịch</b>, nên cộng <code>p.amount</code> (có dấu theo loại); tổng cột đúng là 182.531.000 đ, khớp thực thu ở mục 6. Chiều nối giao dịch → chuyến là nhiều–một nên không nở dòng.</p>
</details>

**Bài 2.** Dự đoán số dòng của chuyến hoàn thành `LEFT JOIN payments LEFT JOIN trip_ratings`, rồi tính điểm sao trung bình trên kết quả đó. Vì sao lệch so với 4,22 của MS-011? Sửa để mỗi chuyến đúng một dòng.

<details><summary><b>Lời giải</b></summary>
<p>2.832 dòng (<code>payments</code> làm nở, <code>trip_ratings</code> thì không); điểm trung bình <b>4,24</b> vì chuyến có hai giao dịch được tính sao hai lần. Thay <code>payments</code> bằng một <code>OUTER APPLY</code> trả đúng một dòng mỗi chuyến:</p>
<pre><code class="language-sql">SELECT COUNT(*) AS so_dong,
       CAST(AVG(CAST(r.stars AS decimal(9, 4))) AS decimal(4, 2)) AS diem_tb,
       SUM(g.so_gd) AS so_gd
FROM toinoi.trips AS t
OUTER APPLY (SELECT COUNT(*) AS so_gd FROM toinoi.payments AS p WHERE p.trip_id = t.id) AS g
LEFT JOIN toinoi.trip_ratings AS r ON r.trip_id = t.id
WHERE t.status = 'completed';</code></pre>
<p>Kết quả đã chạy: <code>2707 | 4.22 | 2800</code>. <code>COUNT(*)</code> không có <code>GROUP BY</code> luôn trả một dòng (0 khi không có giao dịch), nên ở đây <code>CROSS APPLY</code> cũng cho cùng kết quả.</p>
</details>

**Bài 3.** Lập danh sách **3 chuyến hoàn thành có cước cao nhất của mỗi thành phố trong ngày 20/10** (giờ VN). Đà Nẵng hôm đó có bão, không có chuyến nào; báo cáo vẫn phải có dòng Đà Nẵng. Hai chuyến cùng cước thì chuyến mã nhỏ hơn đứng trước.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT c.code AS tp, x.code, x.fare
FROM toinoi.cities AS c
OUTER APPLY (SELECT TOP (3) t.code, t.fare
             FROM toinoi.trips AS t
             WHERE t.city_id = c.id AND t.status = 'completed'
               AND t.requested_at &gt;= '2026-10-20 00:00 +07:00' AND t.requested_at &lt; '2026-10-21 00:00 +07:00'
             ORDER BY t.fare DESC, t.code) AS x
ORDER BY c.code, x.fare DESC, x.code;</code></pre>
<p>Kết quả đã chạy, 7 dòng: <code>DN|NULL|NULL</code>; HN 102088 và 102107 (cùng 142.000 đ), 102098 (134.000 đ); SG 102045 (148.000 đ), 102099 (142.000 đ), 102092 (134.000 đ). Với <code>CROSS APPLY</code> chỉ còn 6 dòng và Đà Nẵng biến mất. Hai chuyến HN cùng 142.000 đ cho thấy vì sao cần tie-breaker <code>t.code</code>; nếu nghiệp vụ muốn lấy cả các chuyến hoà ở vị trí thứ 3, dùng <code>TOP (3) WITH TIES</code> (MS-007).</p>
</details>

## 17. Đọc tiếp

- Microsoft Learn: [FROM clause plus JOIN, APPLY, PIVOT](https://learn.microsoft.com/sql/t-sql/queries/from-transact-sql) (cú pháp `CROSS APPLY`/`OUTER APPLY`), [Joins](https://learn.microsoft.com/sql/relational-databases/performance/joins), [EXISTS](https://learn.microsoft.com/sql/t-sql/language-elements/exists-transact-sql), [TOP](https://learn.microsoft.com/sql/t-sql/queries/top-transact-sql), [STRING_AGG](https://learn.microsoft.com/sql/t-sql/functions/string-agg-transact-sql).
- Cùng chủ đề trên PostgreSQL: [PG-013 · JOIN và số dòng sau khi nối bảng](/articles/postgresql-join-quan-he-cardinality).
- Lab: `examples/toi-noi-mssql/p02/13-join-apply.sql`.

Bài tiếp theo: [MS-014 · Subquery, EXISTS, CTE và bảng tạm](/articles/mssql-subquery-exists-cte-temp-table)
