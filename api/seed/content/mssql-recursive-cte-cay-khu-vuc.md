---
slug: mssql-recursive-cte-cay-khu-vuc
title: "Recursive CTE với cây khu vực điều phối Tới Nơi trên SQL Server"
summary: "Báo cáo của Tới Nơi cộng dồn chuyến theo cây khu vực điều phối, và một bản cập nhật cây làm Hà Nội tụt từ 1.094 xuống 340 chuyến mà không có lỗi nào. Bài viết recursive CTE bằng T-SQL: phần neo, phần đệ quy, độ sâu và đường đi, MAXRECURSION và lỗi 530, phát hiện chu trình bằng chuỗi đường đi khi SQL Server không có mệnh đề CYCLE, và hierarchyid như một cách lưu cây khác."
category: mssql
level: advanced
tags: [mssql, ms-019, sql-server-2022, t-sql, recursive-cte, maxrecursion, hierarchyid, chu-trinh, toi-noi]
language: vi
---

Tới Nơi không điều phối theo quận huyện hành chính mà theo một **cây khu vực** riêng: thành phố chia thành vùng, vùng chia thành các khu đón khách. Bảng thưởng tài xế và báo cáo doanh thu vùng đều cần trả lời cùng một kiểu câu hỏi: "tổng của mọi thứ nằm **dưới** nút này". Cây hôm nay sâu ba tầng, nhưng bộ phận vận hành đã báo trước sẽ tách khu đông thành "tiểu khu", nên câu SQL không được giả định số tầng.

Một buổi sáng, báo cáo vùng Hà Nội chỉ còn 340 chuyến thay vì 1.094. Không truy vấn nào lỗi, không ràng buộc nào báo vi phạm: một bản cập nhật cây đã tạo ra **chu trình**. Bài này viết recursive CTE bằng T-SQL để duyệt cây có điều kiện dừng, cộng dồn số liệu theo cây có đối chiếu, dùng `MAXRECURSION` làm chốt an toàn, và phát hiện dữ liệu cây bị vòng trước khi tin vào con số.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-019 · Phần 2/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/mssql-subquery-exists-cte-temp-table">MS-014 · Subquery, EXISTS, CTE và bảng tạm</a>, <a href="/articles/mssql-join-apply-cardinality">MS-013 · JOIN, APPLY và số dòng sau khi nối</a>, <a href="/articles/mssql-group-by-rollup-grouping-sets">MS-012 · GROUP BY, HAVING, ROLLUP</a>. Lab: <code>examples/toi-noi-mssql/p02</code> (tệp <code>19-cay-khu-vuc.sql</code>, rồi <code>19-chu-trinh.sql</code>). Sau bài này, bạn duyệt được cây xuống và lên với điều kiện dừng, cộng dồn số liệu theo cây có đối chiếu, đặt <code>MAXRECURSION</code> có chủ đích và phát hiện chu trình bằng chuỗi đường đi.</p>
</div>

## 1. Cây khu vực nằm trong một bảng

Bảng `toinoi.areas` (thêm ở `p02/00-mo-rong-schema.sql`) lưu cây kiểu **danh sách kề** (adjacency list): mỗi nút một dòng, `parent_id` trỏ tới nút cha trong chính bảng đó qua khoá ngoại tự tham chiếu `fk_areas_parent`. Gốc là thành phố, `parent_id` NULL. Cột `level` nhận `city` / `district` (vùng) / `zone` (khu); `id` là số nhỏ dễ đọc (2 = HN, 21 = HN-TT, 211 = HN-TT-01). Hai CHECK đáng chú ý: `ck_areas_root_no_parent` (nút là thành phố khi và chỉ khi không có cha) và `ck_areas_not_own_parent` (`parent_id <> id`). Mục 7 cho thấy CHECK thứ hai chưa đủ.

Fixture có **34 nút**: 3 thành phố, 10 vùng, 21 khu; vùng `HN-NA` mới mở, chưa có khu. Chuyến gắn với khu đón qua `trips.pickup_area_id` (36 chuyến NULL vì mất GPS). Với cây cố định ba tầng, hai lần JOIN lên cha là đủ; nhưng số JOIN bằng số tầng, thêm "tiểu khu" là phải sửa mọi báo cáo. **Truy vấn đệ quy** đi theo `parent_id` tới khi hết đường, không cần biết trước độ sâu.

## 2. Recursive CTE trong T-SQL chạy như thế nào?

**Recursive CTE** (CTE đệ quy) là CTE tham chiếu chính nó. T-SQL viết bằng `WITH` thường, **không có** từ khoá `RECURSIVE`:

```sql
WITH dem (n) AS (
    SELECT 1                       -- neo
    UNION ALL
    SELECT n + 1 FROM dem          -- đệ quy: chỉ thấy các dòng của vòng TRƯỚC
    WHERE n < 5                    -- điều kiện dừng
)
SELECT n FROM dem;                 -- 5 dòng: 1 … 5
```

Tài liệu `WITH common_table_expression` mô tả cách chạy như một **vòng lặp**:

1. Chạy **phần neo** (anchor member) một lần, được tập T0.
2. Chạy **phần đệ quy** (recursive member) với đầu vào là tập của vòng trước Ti, được Ti+1. Tên CTE trong phần đệ quy chỉ là các dòng **mới** của vòng trước, không phải toàn bộ kết quả.
3. Lặp bước 2 cho tới khi một vòng trả **tập rỗng**.
4. Kết quả là `UNION ALL` của T0 … Tn.

Đi xuống từ `SG`: vòng 0 là `SG`, vòng 1 là 4 vùng, vòng 2 là 10 khu, vòng 3 không còn con nào nên dừng; tổng 15 dòng. Trên cây, vòng rỗng đến khi gặp lá. Điều đó chỉ đúng khi dữ liệu thật sự là cây (mục 7).

Ba lỗi hay gặp khi chép từ PostgreSQL hoặc viết vội, đều đã chạy trong lab:

```text
PASS WITH RECURSIVE (cú pháp PostgreSQL) — lỗi 102: Incorrect syntax near 'dem'.
PASS UNION thay cho UNION ALL — lỗi 252: Recursive common table expression 'dem' does not contain a top-level UNION ALL operator.
PASS kiểu cột lệch giữa neo và đệ quy — lỗi 240: Types don't match between the anchor and the recursive part in column "duong_di" of recursive query "cay".
```

Lỗi thứ ba đáng giải thích. Phần neo ép `CAST(code AS varchar(400))`, phần đệ quy viết `c.duong_di + '/' + a.code`. Phép nối varchar cộng độ dài, nên kết quả là `varchar(400 + 1 + 12)`, không còn là `varchar(400)`. SQL Server đòi kiểu của từng cột ở hai phần **trùng khít**, kể cả độ dài, nên phần đệ quy cũng phải `CAST(… AS varchar(400))`.

Phần đệ quy còn bị cấm một số thứ. Tài liệu liệt kê `DISTINCT`, `GROUP BY`, `HAVING`, hàm gộp vô hướng, `TOP`, `LEFT/RIGHT/FULL OUTER JOIN` (chỉ `INNER JOIN`), subquery và hint đặt lên tham chiếu đệ quy. Lab đã gặp `TOP` ở [MS-018](/articles/mssql-phan-trang-keyset) (lỗi 461). Hãy để truy vấn **ngoài** làm việc gộp và `LEFT JOIN`.

## 3. Đi xuống: độ sâu và đường đi

Mọi nút thuộc cây TP. Hồ Chí Minh, kèm độ sâu và đường đi. SQL Server không có mệnh đề `SEARCH DEPTH FIRST` của chuẩn SQL, nên tự dựng chuỗi đường đi và sắp theo nó:

```sql
WITH cay (id, code, name, do_sau, duong_di) AS (
    SELECT id, code, name, 0, CAST(code AS varchar(400))
    FROM toinoi.areas
    WHERE code = 'SG'
    UNION ALL
    SELECT a.id, a.code, a.name, c.do_sau + 1, CAST(c.duong_di + '/' + a.code AS varchar(400))
    FROM toinoi.areas AS a
    JOIN cay AS c ON a.parent_id = c.id
)
SELECT do_sau, code, name, duong_di
FROM cay
ORDER BY duong_di COLLATE Latin1_General_BIN2;
```

```text
do_sau|code|name|duong_di
------|----|----|--------
0|SG|TP. Hồ Chí Minh|SG
1|SG-DO|Phía Đông|SG/SG-DO
2|SG-DO-01|Khu 01|SG/SG-DO/SG-DO-01
2|SG-DO-02|Khu 02|SG/SG-DO/SG-DO-02
2|SG-DO-03|Khu 03|SG/SG-DO/SG-DO-03
1|SG-NA|Phía Nam|SG/SG-NA
2|SG-NA-01|Khu 01|SG/SG-NA/SG-NA-01
2|SG-NA-02|Khu 02|SG/SG-NA/SG-NA-02
1|SG-TA|Phía Tây|SG/SG-TA
2|SG-TA-01|Khu 01|SG/SG-TA/SG-TA-01
2|SG-TA-02|Khu 02|SG/SG-TA/SG-TA-02
1|SG-TT|Trung tâm|SG/SG-TT
2|SG-TT-01|Khu 01|SG/SG-TT/SG-TT-01
2|SG-TT-02|Khu 02|SG/SG-TT/SG-TT-02
2|SG-TT-03|Khu 03|SG/SG-TT/SG-TT-03
```

Cha đứng ngay trên các con, đúng thứ tự duyệt theo chiều sâu. Hai quyết định đằng sau:

- **`varchar(400)`** đủ cho đường đi khoảng 30 tầng với mã dài nhất 12 ký tự. Cây sâu hơn thì phép `CAST` cắt chuỗi một cách im lặng; chọn độ dài theo độ sâu tối đa mà nghiệp vụ cam kết, và kiểm độ sâu (mục 6).
- **`COLLATE Latin1_General_BIN2`** so từng byte, ký tự `/` (0x2F) đứng trước chữ số và chữ cái, nên `SG/SG-DO` luôn đứng trước `SG/SG-DO/…` và trước `SG/SG-NA`. Database lab dùng `Vietnamese_100_CI_AS` cho chữ tiếng Việt ([MS-004](/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode)); một khoá sắp xếp máy dựng ra như chuỗi đường đi không cần quy tắc ngôn ngữ, cần quy tắc cố định.

## 4. Đi lên: tổ tiên của một khu

Đảo chiều JOIN: dòng mới là **cha** của dòng vòng trước.

```sql
WITH to_tien (id, parent_id, code, level, bac) AS (
    SELECT id, parent_id, code, level, 0 FROM toinoi.areas WHERE code = 'HN-TA-02'
    UNION ALL
    SELECT a.id, a.parent_id, a.code, a.level, t.bac + 1
    FROM toinoi.areas AS a
    JOIN to_tien AS t ON a.id = t.parent_id
)
SELECT bac, code, level FROM to_tien ORDER BY bac;
```

```text
PASS tổ tiên của HN-TA-02 = HN-TA-02 > HN-TA > HN
```

Đi lên dừng khi gặp gốc (`parent_id` NULL thì JOIN không khớp dòng nào). Trên dữ liệu có vòng, đi lên **không bao giờ** gặp gốc. Đó là việc của chốt an toàn ở mục sau.

## 5. MAXRECURSION: chốt an toàn có sẵn

SQL Server giới hạn số vòng đệ quy của mỗi câu lệnh, mặc định **100**. Vòng 101 làm câu lệnh dừng với lỗi 530. Đổi giới hạn bằng query hint `OPTION (MAXRECURSION n)` ở cuối câu lệnh ngoài cùng, `n` từ 0 tới 32767; `0` nghĩa là không giới hạn.

```text
PASS đếm tới 200 với MAXRECURSION mặc định — lỗi 530: The statement terminated. The maximum recursion 100 has been exhausted before statement completion.
PASS đếm tới 200 với MAXRECURSION 200 = 200
PASS MAXRECURSION vượt 32767 — lỗi 310: The value 32768 specified for the MAXRECURSION option exceeds the allowed maximum of 32767.
```

Một chi tiết dễ bỏ sót: lỗi 530 **không thu hồi các dòng đã gửi**. Đếm tới 10 với `OPTION (MAXRECURSION 3)`:

```text
n
-
1
2
3
4
Msg 530: The statement terminated. The maximum recursion 3 has been exhausted before statement completion.
```

Client nhận 4 dòng của các vòng 0 … 3 rồi mới nhận lỗi. Ứng dụng đọc kết quả theo luồng có thể đã xử lý một phần trước khi thấy lỗi.

`MAXRECURSION` đặt có chủ đích theo nghiệp vụ, không phải để "cho chạy được": cây khu vực cam kết tối đa vài tầng, nên giới hạn 100 mặc định đã rộng rãi. Gặp 530 trên cây này nghĩa là **dữ liệu sai**, không phải giới hạn quá chặt. Đặt `MAXRECURSION 0` để "sửa" lỗi 530 trên dữ liệu có vòng là biến một lỗi rõ ràng thành một câu chạy tới khi hết thời gian chờ.

## 6. Cộng dồn theo cây và đối chiếu

Câu hỏi của báo cáo: số chuyến và doanh thu (chuyến hoàn thành) của **mọi khu nằm dưới** mỗi nút. Bước 1 dựng bảng **tổ tiên – hậu duệ**: mỗi cặp (tổ tiên, hậu duệ) một dòng, kể cả cặp (nút, chính nó). Bước 2 nối với chuyến.

```sql
WITH to_tien_hau_due (to_tien_id, hau_due_id) AS (
    SELECT id, id FROM toinoi.areas              -- mỗi nút là hậu duệ của chính nó
    UNION ALL
    SELECT c.to_tien_id, a.id
    FROM to_tien_hau_due AS c
    JOIN toinoi.areas AS a ON a.parent_id = c.hau_due_id
)
SELECT n.code, n.level,
       COUNT(t.id)                                                   AS so_chuyen,
       SUM(CASE WHEN t.status = 'completed' THEN t.fare ELSE 0 END)  AS doanh_thu
FROM toinoi.areas AS n
JOIN to_tien_hau_due AS c ON c.to_tien_id = n.id
LEFT JOIN toinoi.trips AS t ON t.pickup_area_id = c.hau_due_id
WHERE n.level <> 'zone'
GROUP BY n.id, n.code, n.level
ORDER BY n.code;
```

```text
code|level|so_chuyen|doanh_thu
----|-----|---------|---------
DN|city|456|26124000
DN-BI|district|231|13618000
DN-TT|district|225|12506000
HN|city|1094|62246000
HN-BA|district|340|19456000
HN-NA|district|0|0
HN-TA|district|295|16496000
HN-TT|district|459|26294000
SG|city|1588|92143000
SG-DO|district|481|28484000
SG-NA|district|315|17353000
SG-TA|district|315|18523000
SG-TT|district|477|27783000
Warning: Null value is eliminated by an aggregate or other SET operation.
```

Ba điểm:

- **`LEFT JOIN` và `COUNT(t.id)`** giữ `HN-NA` với 0 chuyến. `COUNT(*)` sẽ đếm dòng NULL của `LEFT JOIN` thành 1. Dòng `Warning` là thông điệp của SQL Server khi `COUNT(t.id)` bỏ qua NULL (khi `ANSI_WARNINGS` bật, như phiên sqlcmd của lab): đúng ý đồ ở đây, không phải lỗi.
- **Việc gộp nằm ở truy vấn ngoài.** Phần đệ quy chỉ dựng cặp tổ tiên – hậu duệ, vì nó không được có `GROUP BY` hay `LEFT JOIN`.
- **Mọi con số theo cây cần một câu đối chiếu.** Tổng của các thành phố qua cây phải bằng đếm thẳng chuyến có khu đón:

```text
qua_cay|truc_tiep|mat_gps
-------|---------|-------
3138|3138|36
```

3.138 + 36 chuyến mất GPS = 3.174, đúng tổng số chuyến của fixture. Trước khi tin các con số trên, lab còn kiểm cây **lành**: đi xuống từ mọi gốc (`parent_id IS NULL`) với chốt `WHERE c.do_sau < 10` trong phần đệ quy, rồi so số nút đi tới được với tổng số nút:

```text
tong_nut|di_toi_tu_goc|sau_nhat|so_la
--------|-------------|--------|-----
34|34|2|22
```

Chốt `do_sau < 10` thuộc về chính truy vấn: trên dữ liệu có vòng, nó dừng sau 10 vòng thay vì chạy tới lỗi 530.

## 7. Khi dữ liệu không còn là cây

### 7.1 Một bản cập nhật qua mọi ràng buộc

`19-chu-trinh.sql` chạy toàn bộ trong `BEGIN TRANSACTION … ROLLBACK`. Bản cập nhật **đảo cha** hai vùng Hà Nội: HN-TT (21) nhận cha là khu HN-TA-01 (221), HN-TA (22) nhận cha là khu HN-TT-01 (211).

```sql
UPDATE toinoi.areas
SET parent_id = CASE id WHEN 21 THEN 221 WHEN 22 THEN 211 END
OUTPUT inserted.id, inserted.code, deleted.parent_id, inserted.parent_id INTO @doi
WHERE id IN (21, 22);
```

```text
id|code|cha_cu|cha_moi
--|----|------|-------
21|HN-TT|2|221
22|HN-TA|2|211
PASS đảo cha hai vùng HN (mọi ràng buộc vẫn qua) = 21:2→221, 22:2→211
PASS CHECK của areas bị tắt hoặc không tin cậy = 0
```

Không lỗi. Khoá ngoại thấy cha tồn tại; `ck_areas_root_no_parent` thấy vùng có cha; `ck_areas_not_own_parent` thấy 21 ≠ 221. Mọi CHECK vẫn bật và tin cậy. Nhưng CHECK chỉ nhìn **một dòng**, còn vòng `21 → 221 → 22 → 211 → 21` chỉ hiện ra khi đi qua bốn dòng.

### 7.2 Hệ quả không báo lỗi

```text
số nút dưới HN | số chuyến HN: 5 | 340
```

Đi xuống từ HN chỉ còn HN, HN-BA, HN-NA và hai khu của HN-BA: báo cáo tụt từ 1.094 xuống **340** chuyến. Đi xuống vẫn **dừng**, vì các nút trên vòng không còn đi tới được từ gốc nào. Chu trình nằm ngoài tầm với nên không làm truy vấn chạy mãi, chỉ làm con số sai.

Đi lên từ một khu trên vòng thì khác. Chốt theo độ sâu (`bac < 5`) cho thấy vòng lặp lại:

```text
PASS đi lên từ HN-TT-01 với chốt độ sâu = HN-TT-01 > HN-TT > HN-TA-01 > HN-TA > HN-TT-01 > HN-TT
```

Chốt độ sâu chỉ để an toàn; nó không nói được kết quả có chứa vòng hay không.

### 7.3 Phát hiện chu trình bằng chuỗi đường đi

PostgreSQL có mệnh đề `CYCLE` tự đánh dấu dòng quay lại. SQL Server **không có**. Cách tự viết: mang theo chuỗi `/id/id/…/` các nút đã đi qua; dòng nào có id đã nằm trong chuỗi thì đánh dấu và **không đệ quy tiếp**.

```sql
WITH to_tien (id, parent_id, code, bac, da_qua, la_chu_trinh) AS (
    SELECT id, parent_id, code, 0,
           CAST('/' + CAST(id AS varchar(10)) + '/' AS varchar(400)), 0
    FROM toinoi.areas WHERE code = 'HN-TT-01'
    UNION ALL
    SELECT a.id, a.parent_id, a.code, t.bac + 1,
           CAST(t.da_qua + CAST(a.id AS varchar(10)) + '/' AS varchar(400)),
           CASE WHEN CHARINDEX('/' + CAST(a.id AS varchar(10)) + '/', t.da_qua) > 0 THEN 1 ELSE 0 END
    FROM toinoi.areas AS a JOIN to_tien AS t ON a.id = t.parent_id
    WHERE t.la_chu_trinh = 0                       -- dòng đã quay vòng thì dừng
)
SELECT bac, code, da_qua, la_chu_trinh FROM to_tien ORDER BY bac;
```

```text
bac|code|da_qua|la_chu_trinh
---|----|------|------------
0|HN-TT-01|/211/|0
1|HN-TT|/211/21/|0
2|HN-TA-01|/211/21/221/|0
3|HN-TA|/211/21/221/22/|0
4|HN-TT-01|/211/21/221/22/211/|1
```

Dấu `/` ở **hai đầu** mỗi id là bắt buộc: tìm `'1'` trong `'/211/'` sẽ khớp nhầm, tìm `'/1/'` thì không. Truy vấn tự dừng ở dòng đánh dấu, không cần `MAXRECURSION`.

### 7.4 Truy vấn kiểm tra cho cả bảng

Chạy định kỳ, hoặc trước khi nhận một bản cập nhật cây:

a) **Nút không đi tới được từ gốc nào**: nút nằm trên vòng và mọi hậu duệ của chúng. Đi xuống từ các gốc rồi `NOT EXISTS`. Ở PostgreSQL có thể dùng `UNION` để truy vấn tự dừng trên vòng; SQL Server bắt buộc `UNION ALL`, nhưng đi xuống từ gốc vốn không chạm tới vòng nên vẫn dừng.

```text
id|code
--|----
21|HN-TT
22|HN-TA
211|HN-TT-01
212|HN-TT-02
213|HN-TT-03
221|HN-TA-01
222|HN-TA-02
```

b) **Nút nằm trên vòng**: đi lên từ chính nó, dừng trước khi đi lại một nút đã qua (`CHARINDEX(…) = 0` trong `WHERE` của phần đệ quy), rồi giữ dòng mà bước kế tiếp sẽ quay về điểm xuất phát (`l.parent_id = l.xuat_phat`):

```text
id|code|vong
--|----|----
21|HN-TT|/21/221/22/211/
22|HN-TA|/22/211/21/221/
211|HN-TT-01|/211/21/221/22/
221|HN-TA-01|/221/22/211/21/
```

7 nút không tới được gốc, 4 nút nằm trên vòng. Hai truy vấn này nên là một kiểm tra bất biến trong quy trình nhận dữ liệu cây, cùng loại với K1–K9 của `02-kiem-tra-fixture.sql`.

### 7.5 Truy vấn "trần" trên vòng: lỗi 530 và giao dịch hỏng

Đi lên từ HN-TT-01 **không có** chốt nào. Trên PostgreSQL, câu này chạy tới khi hết bộ nhớ hoặc hết thời gian chờ. Trên SQL Server, `MAXRECURSION` mặc định chặn nó ở vòng 101:

```text
Msg 530: The statement terminated. The maximum recursion 100 has been exhausted before statement completion.
PASS truy vấn trần trên vòng (lỗi | XACT_STATE) = 530 | -1
```

Chú ý cột thứ hai. Lab bắt lỗi bằng `TRY…CATCH` bên trong `BEGIN TRANSACTION`, và sau lỗi 530 `XACT_STATE()` trả **-1**: giao dịch đã hỏng, chỉ còn `ROLLBACK` được, không commit được và không chạy tiếp câu ghi nào. Vì vậy lab đặt thí nghiệm này cuối cùng, ngay trước `ROLLBACK`. Một job nhận dữ liệu cây chạy truy vấn đệ quy trong cùng giao dịch với các câu ghi sẽ mất cả giao dịch khi gặp dữ liệu có vòng. Kiểm vòng bằng truy vấn có chốt **trước** khi mở giao dịch ghi.

Sau `ROLLBACK`:

```text
PASS sau ROLLBACK (nút không tới gốc | cha của 21, 22) = 0 | 21:2,22:2
```

## 8. hierarchyid: một cách lưu cây khác

SQL Server có kiểu dữ liệu hệ thống **`hierarchyid`**: mỗi nút lưu vị trí của mình trong cây dưới dạng đường đi nén, kèm các phương thức như `GetLevel()`, `GetAncestor(n)`, `IsDescendantOf(h)` và `ToString()`. Bài tập 3 dựng giá trị `hierarchyid` từ đường đi bằng id (`/2/21/211/`) rồi đếm chuyến của cây HN bằng `IsDescendantOf`, không cần bảng tổ tiên – hậu duệ:

```text
PASS Bài 3: hierarchyid của 211 | cấp | số chuyến HN = /2/21/211/ | 3 | 1094
```

`GetLevel()` trả 3 vì gốc `/` là cấp 0, thành phố là cấp 1. Cùng 1.094 chuyến như mục 6. Đổi lại, khi lưu `hierarchyid` thành cột, chuyển một nhánh sang cha khác là phải tính lại giá trị cho cả nhánh, và chính kiểu dữ liệu không ngăn hai nút trỏ sai cha theo nghiệp vụ. Lab chỉ dựng `hierarchyid` tạm trong truy vấn; cây của Tới Nơi vẫn lưu bằng `parent_id`.

## 9. Làm trong DBStudio

- Mỗi truy vấn đệ quy của `19-cay-khu-vuc.sql` nằm trong một batch `GO`; `Ctrl+Enter` chạy batch đang đứng, `Ctrl+Shift+Enter` chạy cả tệp. Dòng `Warning: Null value is eliminated …` và các dòng `PASS …` nằm ở tab **Thông điệp**, tách khỏi lưới kết quả.
- `19-chu-trinh.sql` tự mở và đóng giao dịch. Nếu muốn thử bản cập nhật "đảo cha" bằng tay, bấm "Bắt đầu" trên thanh công cụ transaction, chạy câu `UPDATE` và các truy vấn kiểm tra, rồi bấm "Rollback". Sau lỗi 530 ở mục 7.5, "Rollback" là nút duy nhất còn ý nghĩa.

## 10. So với PostgreSQL

Cùng cây, cùng chu trình với [PG-019](/articles/postgresql-recursive-cte-cay-khu-vuc):

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| Từ khoá | `WITH RECURSIVE` | `WITH` (viết `RECURSIVE` là lỗi 102) |
| Nối neo với đệ quy | `UNION ALL` hoặc `UNION` (khử trùng lặp, tự dừng trên vòng) | Chỉ `UNION ALL` (lỗi 252) |
| Kiểu cột | Phần neo quyết định | Hai phần phải trùng khít, kể cả độ dài (lỗi 240) |
| Thứ tự theo chiều sâu | `SEARCH DEPTH FIRST BY` | Tự dựng chuỗi đường đi, sắp với collation nhị phân |
| Phát hiện chu trình | `CYCLE id SET … USING …` | Tự mang chuỗi `/id/…/`, kiểm bằng `CHARINDEX` |
| Truy vấn trần trên vòng | Chạy tới khi hết tài nguyên | Dừng ở vòng 101, lỗi 530 (`MAXRECURSION`) |
| Kiểu dữ liệu cây | Extension `ltree` | `hierarchyid` có sẵn |

## 11. Lỗi thường gặp và đánh đổi

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Lỗi 240 | Cột chuỗi ở phần đệ quy dài hơn phần neo | `CAST` ở **cả hai** phần về cùng kiểu |
| Lỗi 530 trên cây "vài tầng" | Dữ liệu có vòng | Chạy truy vấn kiểm tra mục 7.4; đừng tăng `MAXRECURSION` |
| Báo cáo tụt số mà không lỗi | Một nhánh bị tách khỏi gốc | Câu đối chiếu "qua cây = đếm thẳng" chạy cùng báo cáo |
| Nút không có chuyến biến mất | `JOIN` thay vì `LEFT JOIN`, hoặc `COUNT(*)` | `LEFT JOIN` + `COUNT(t.id)` ở truy vấn ngoài |
| Thứ tự cây lộn xộn | Không có `ORDER BY`, hoặc sắp chuỗi đường đi theo collation ngôn ngữ | `ORDER BY duong_di COLLATE Latin1_General_BIN2` |
| Phát hiện vòng khớp nhầm | Tìm id không có dấu phân cách hai đầu | Tìm `'/' + id + '/'` |

Đánh đổi:

| Cách lưu và truy vấn cây | Ưu | Nhược |
|---|---|---|
| Danh sách kề + recursive CTE (bài này) | Sửa một nút = sửa một dòng; không giả định số tầng | Mỗi lần đọc phải duyệt; chu trình phải canh bằng truy vấn |
| Danh sách kề + JOIN theo số tầng | Đơn giản khi cây cố định | Thêm tầng là sửa mọi truy vấn |
| Bảng tổ tiên – hậu duệ lưu sẵn | Đọc "mọi hậu duệ" chỉ là một JOIN | Đổi cây phải cập nhật nhiều dòng và giữ đồng bộ |
| Cột `hierarchyid` | Hỏi tổ tiên/hậu duệ bằng phương thức có sẵn | Chuyển nhánh là tính lại cả nhánh |

Hiệu năng (index `ix_areas_parent` cho JOIN đi xuống) để Phần 3 đo. Hai người sửa hai nút **cùng lúc**, mỗi thay đổi riêng không tạo vòng nhưng ghép lại thì có, là chuyện của Phần 4.

## 12. Bài tập

**Bài 1.** Liệt kê các nút lá của cây Hà Nội kèm đường đi đầy đủ từ gốc. Lá nào không phải khu?

<details><summary><b>Lời giải</b></summary>
<p>Đi xuống từ <code>HN</code> với chuỗi đường đi như mục 3; ở truy vấn ngoài giữ nút không có con (<code>NOT EXISTS (SELECT 1 FROM toinoi.areas AS con WHERE con.parent_id = c.id)</code>), sắp theo đường đi với collation nhị phân. Lab: <code>PASS Bài 1: lá của HN = HN/HN-BA/HN-BA-01, HN/HN-BA/HN-BA-02, HN/HN-NA, HN/HN-TA/HN-TA-01, HN/HN-TA/HN-TA-02, HN/HN-TT/HN-TT-01, HN/HN-TT/HN-TT-02, HN/HN-TT/HN-TT-03</code>. Lá duy nhất không phải khu là <code>HN-NA</code>: vùng mới mở chưa chia khu thì chuyến ở đó chưa có khu đón hợp lệ, phải chia khu trước khi nhận chuyến.</p>
</details>

**Bài 2.** Mỗi thành phố, vùng nào có doanh thu cao nhất? Thứ tự phải xác định khi hai vùng hoà.

<details><summary><b>Lời giải</b></summary>
<p>Dùng bảng tổ tiên – hậu duệ của mục 6, gộp doanh thu cho các nút <code>district</code>, rồi <code>ROW_NUMBER() OVER (PARTITION BY parent_id ORDER BY doanh_thu DESC, code)</code> và giữ hạng 1. <code>code</code> là tie-breaker. Lab: <code>PASS Bài 2: vùng doanh thu cao nhất mỗi thành phố = DN-BI 13618000, HN-TT 26294000, SG-DO 28484000</code>.</p>
</details>

**Bài 3.** Dựng giá trị `hierarchyid` cho mọi nút từ đường đi bằng id, rồi đếm chuyến của cây Hà Nội bằng `IsDescendantOf` thay cho bảng tổ tiên – hậu duệ.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">WITH cay (id, duong_di) AS (
    SELECT id, CAST('/' + CAST(id AS varchar(10)) + '/' AS varchar(400))
    FROM toinoi.areas WHERE parent_id IS NULL
    UNION ALL
    SELECT a.id, CAST(c.duong_di + CAST(a.id AS varchar(10)) + '/' AS varchar(400))
    FROM toinoi.areas AS a JOIN cay AS c ON a.parent_id = c.id
),
nut AS (SELECT id, CAST(duong_di AS hierarchyid) AS h FROM cay)
SELECT COUNT(*)
FROM toinoi.trips AS t
JOIN nut AS k  ON k.id = t.pickup_area_id
JOIN nut AS hn ON hn.id = 2
WHERE k.h.IsDescendantOf(hn.h) = 1;</code></pre>
<p>Lab: <code>PASS Bài 3: hierarchyid của 211 | cấp | số chuyến HN = /2/21/211/ | 3 | 1094</code>. <code>IsDescendantOf</code> coi một nút là hậu duệ của chính nó, giống cặp (nút, chính nó) trong bảng tổ tiên – hậu duệ.</p>
</details>

## 13. Đọc tiếp

- Trước đó: [MS-018 · Phân trang lịch sử chuyến: OFFSET/FETCH và keyset](/articles/mssql-phan-trang-keyset).
- CTE không đệ quy và bảng tạm: [MS-014](/articles/mssql-subquery-exists-cte-temp-table). `LEFT JOIN` và số dòng sau khi nối: [MS-013](/articles/mssql-join-apply-cardinality). Collation: [MS-004](/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode).
- Giao dịch, `XACT_STATE()` và `TRY…CATCH`: MS-031 (Phần 4).
- Cùng cây trên PostgreSQL: [PG-019 · Recursive CTE với cây khu vực](/articles/postgresql-recursive-cte-cay-khu-vuc).
- Microsoft Learn: [WITH common_table_expression](https://learn.microsoft.com/sql/t-sql/queries/with-common-table-expression-transact-sql), [Query hints (MAXRECURSION)](https://learn.microsoft.com/sql/t-sql/queries/hints-transact-sql-query), [hierarchyid data type method reference](https://learn.microsoft.com/sql/t-sql/data-types/hierarchyid-data-type-method-reference), [XACT_STATE](https://learn.microsoft.com/sql/t-sql/functions/xact-state-transact-sql).

Bài tiếp theo: [MS-020 · Thực hành: truy vấn vận hành và đối soát thanh toán](/articles/mssql-thuc-hanh-tsql-doi-soat)
