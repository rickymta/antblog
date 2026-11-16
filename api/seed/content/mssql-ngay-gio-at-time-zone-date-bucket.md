---
slug: mssql-ngay-gio-at-time-zone-date-bucket
title: "Ngày giờ, AT TIME ZONE, DATETRUNC và DATE_BUCKET: bộ lọc ngày nghiệp vụ theo giờ Việt Nam và gom chuyến theo khung 15 phút"
summary: "Ba báo cáo cùng hỏi \"ngày 15/10 có bao nhiêu chuyến\" ra 105, 106 và 107; DATE_BUCKET theo ngày cắt mỗi ngày lúc 7 giờ sáng; DATEDIFF nói hai mốc cách nhau nửa giây là cùng một ngày. Bài đi qua datetimeoffset, SWITCHOFFSET và AT TIME ZONE, khoảng nửa hở với kiểm tra hai biên, bẫy datetime .997, DATETRUNC, DATE_BUCKET, DATEFIRST và giờ mùa hè trên SQL Server 2022."
category: mssql
level: intermediate
tags: [mssql, ms-016, sql-server-2022, t-sql, datetimeoffset, at-time-zone, datetrunc, date-bucket, timezone, toi-noi]
language: vi
---

Sáng 16/10, ba bộ phận của Tới Nơi cùng báo số chuyến **ngày 15/10** cho cuộc họp giao ban. Đội vận hành báo 106, đội tài chính báo 105, đội dữ liệu báo 107. Cả ba câu T-SQL đều chạy trên cùng database và không câu nào báo lỗi.

Cùng tuần, một kỹ sư dùng hàm mới `DATE_BUCKET` của SQL Server 2022 để gom chuyến theo ngày và thấy "ngày" của mình bắt đầu lúc 7 giờ sáng. Một kỹ sư khác viết cảnh báo "chuyến kéo dài sang ngày hôm sau" bằng `DATEDIFF(day, …)` và nhận về 0 cho chuyến đặt lúc 23:59:59 kết thúc lúc 00:00 sáng hôm sau.

Không ai gõ sai. Mỗi câu có một giả định ngầm về **ngày theo múi giờ nào** và **mốc cuối có được tính không**. Bài này làm rõ các giả định đó, dựng một bộ lọc ngày nghiệp vụ theo giờ Việt Nam có kiểm tra hai biên, rồi gom chuyến theo khung 15 phút bằng `DATETRUNC` và `DATE_BUCKET`.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-016 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode">MS-004 · kiểu dữ liệu cho tiền, thời gian và chuỗi tiếng Việt</a> (mục <code>datetimeoffset</code> và <code>AT TIME ZONE</code>), <a href="/articles/mssql-select-loc-sap-xep-top-offset">MS-007 · SELECT, lọc và sắp xếp</a> (khoảng nửa hở), <a href="/articles/mssql-window-function-window-clause">MS-015 · Window function và mệnh đề WINDOW</a>. Lab: <code>examples/toi-noi-mssql/p02</code> (tệp <code>16-ngay-gio-mui-gio.sql</code>). Sau bài này, bạn viết được bộ lọc "ngày D theo giờ VN" không phụ thuộc cách dữ liệu được ghi, kiểm nó bằng các chuyến sát biên, và gom chuyến theo khung thời gian mà không lệch múi giờ.</p>
</div>

## 1. Dữ liệu dùng trong bài

Fixture Phần 2 (3.174 chuyến, ảnh chụp 01:00 ngày 31/10/2026 giờ VN) ghi mọi mốc bằng `datetimeoffset(3)` với độ lệch `+07:00`. Năm chuyến được cài sát biên ngày 15/10 (giờ VN):

| Mã | Giờ đặt | Thuộc ngày 15/10? |
|---|---|---|
| 101489 | 15/10 00:00:00 | Có (mốc đầu) |
| 101502 | 15/10 06:59:59 | Có, nhưng là 14/10 theo UTC |
| 101503 | 15/10 07:00:00 | Có, đúng 00:00 UTC |
| 101594 | 15/10 23:59:59.5 | Có (nửa giây cuối ngày) |
| 101595 | 16/10 00:00:00 | **Không** (mốc cuối) |

Ngoài ra có **383 chuyến** đặt từ 00:00 tới 06:59 giờ VN: với chúng, ngày VN và ngày UTC khác nhau.

```bash
cd examples/toi-noi-mssql
LAB_DB=ToiNoi_thu node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql \
  p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/16-ngay-gio-mui-gio.sql
```

Kết quả chép từ lần chạy trên SQL Server 2022 16.0.4325.4 (RTM-CU28), 24 dòng `PASS`, 0 `FAIL`. Tệp đặt `SET LANGUAGE us_english` ở đầu để kết quả không phụ thuộc ngôn ngữ của login (mục 8 giải thích vì sao).

## 2. Một khoảnh khắc, nhiều cách biểu diễn

Nhắc lại MS-004: `datetimeoffset` lưu **giờ trên đồng hồ kèm độ lệch** so với UTC, và so sánh theo **khoảnh khắc** (quy về UTC). Đổi cách biểu diễn có hai công cụ:

- `SWITCHOFFSET(x, '+00:00')` đổi sang một **độ lệch cố định**.
- `x AT TIME ZONE 'tên múi giờ'` đổi sang một **múi giờ có tên**, tự chọn độ lệch đúng cho thời điểm đó (có giờ mùa hè thì độ lệch đổi theo mùa). Tên là tên múi giờ Windows, kể cả khi SQL Server chạy trên Linux; giờ Việt Nam là `SE Asia Standard Time`. Catalog `sys.time_zone_info` có 141 tên trên bản của lab.

```sql
SELECT code,
       CAST(requested_at AS varchar(34))                                      AS nhu_da_luu,
       CAST(SWITCHOFFSET(requested_at, '+00:00') AS varchar(34))              AS switchoffset_utc,
       CAST(requested_at AT TIME ZONE 'UTC' AS varchar(34))                   AS at_time_zone_utc,
       CAST(requested_at AT TIME ZONE 'SE Asia Standard Time' AS varchar(34)) AS at_time_zone_vn,
       DATEPART(TZOFFSET, requested_at)                                       AS do_lech_phut,
       CAST(CAST(requested_at AS datetime2(3)) AS varchar(30))                AS bo_do_lech
FROM toinoi.trips
WHERE code = 101503;
```

```text
code|nhu_da_luu|switchoffset_utc|at_time_zone_utc|at_time_zone_vn|do_lech_phut|bo_do_lech
----|----------|----------------|----------------|---------------|------------|----------
101503|2026-10-15 07:00:00.000 +07:00|2026-10-15 00:00:00.000 +00:00|2026-10-15 00:00:00.000 +00:00|2026-10-15 07:00:00.000 +07:00|420|2026-10-15 07:00:00.000
```

Năm cột đầu là **cùng một khoảnh khắc**; so sánh `=` giữa chúng đều đúng. Cột cuối thì khác: ép sang `datetime2` **bỏ độ lệch** và giữ giờ trên đồng hồ như đã lưu, nên từ đó trở đi không ai biết 07:00 là giờ ở đâu.

SQL Server không có "múi giờ của phiên" như PostgreSQL. Chuỗi không ghi độ lệch, khi được đổi sang `datetimeoffset` để so sánh, được gán `+00:00` (MS-007). Đó là nguồn của một trong ba con số ở đầu bài.

## 3. Bảy cách đếm "ngày 15/10"

```sql
SELECT
  SUM(CASE WHEN requested_at >= '2026-10-15 00:00:00 +07:00'
            AND requested_at <  '2026-10-16 00:00:00 +07:00' THEN 1 ELSE 0 END)                    AS a_nua_mo,
  SUM(CASE WHEN CAST(requested_at AT TIME ZONE 'SE Asia Standard Time' AS date) = '2026-10-15'
           THEN 1 ELSE 0 END)                                                                       AS b_doi_gio_vn_roi_cat,
  SUM(CASE WHEN CAST(requested_at AS date) = '2026-10-15' THEN 1 ELSE 0 END)                        AS c_cat_theo_do_lech_luu,
  SUM(CASE WHEN CAST(SWITCHOFFSET(requested_at, '+00:00') AS date) = '2026-10-15' THEN 1 ELSE 0 END) AS d_cat_theo_utc_SAI,
  SUM(CASE WHEN requested_at BETWEEN '2026-10-15 00:00:00 +07:00'
                                 AND '2026-10-15 23:59:59 +07:00' THEN 1 ELSE 0 END)               AS e_between_235959_SAI,
  SUM(CASE WHEN requested_at BETWEEN '2026-10-15 00:00:00 +07:00'
                                 AND '2026-10-16 00:00:00 +07:00' THEN 1 ELSE 0 END)               AS f_between_ngay_sau_SAI,
  SUM(CASE WHEN requested_at >= '2026-10-15' AND requested_at < '2026-10-16' THEN 1 ELSE 0 END)     AS g_moc_khong_do_lech_SAI
FROM toinoi.trips;
```

```text
a_nua_mo|b_doi_gio_vn_roi_cat|c_cat_theo_do_lech_luu|d_cat_theo_utc_SAI|e_between_235959_SAI|f_between_ngay_sau_SAI|g_moc_khong_do_lech_SAI
--------|--------------------|----------------------|------------------|--------------------|----------------------|-----------------------
106|106|106|105|105|107|105
```

Ba con số của cuộc họp giao ban là ba nhóm cách viết:

- **106 (đúng):** (a) khoảng **nửa hở** `[00:00 ngày 15, 00:00 ngày 16)` với hai mốc ghi `+07:00`; (b) đổi sang giờ VN bằng `AT TIME ZONE` rồi cắt lấy ngày; (c) cắt thẳng `CAST(… AS date)`.
- **105:** (d) cắt ngày sau khi đổi sang UTC: thực chất đếm khung 07:00 ngày 15 tới 07:00 ngày 16 giờ VN; (e) `BETWEEN … 23:59:59` bỏ sót chuyến lúc 23:59:59.5; (g) mốc không ghi độ lệch được hiểu là `+00:00`, nên cũng là khung 07:00–07:00 như (d).
- **107:** (f) `BETWEEN` **gồm cả hai đầu**, nên chuyến đúng 00:00 ngày 16 bị đếm vào cả ngày 15 lẫn ngày 16.

### Cách (c) chỉ đúng nhờ cách ghi dữ liệu

`CAST(datetimeoffset AS date)` lấy ngày của giờ trên đồng hồ **như đã lưu**. Fixture ghi mọi mốc bằng `+07:00` nên (c) ra 106. Lab chép cùng các khoảnh khắc sang bảng tạm nhưng ghi bằng `+00:00` (như một job nhập dữ liệu từ hệ thống ghi giờ UTC) rồi đếm lại:

```text
PASS 2b dữ liệu ghi +00:00: nửa mở | CAST date | AT TIME ZONE rồi cắt = 106|105|106
```

Khoảnh khắc không đổi, nên khoảng nửa hở vẫn 106 và cách (b) vẫn 106; cách (c) tụt xuống 105. Trên cùng một bảng, các dòng ghi bằng độ lệch khác nhau sẽ được (c) xếp vào ngày khác nhau, mà không có lỗi nào.

## 4. Kiểm tra hai biên

Một bộ lọc ngày chỉ đáng tin khi nó **giữ** chuyến đúng 00:00:00 ngày 15 và chuyến 23:59:59.5, **loại** chuyến đúng 00:00:00 ngày 16. In từng chuyến biên với kết quả của bốn bộ lọc (1 = được tính):

```text
code|requested_at|a_nua_mo|e_between_235959|f_between_ngay_sau|d_ngay_utc
----|------------|--------|----------------|------------------|----------
101489|2026-10-15 00:00:00.000 +07:00|1|1|1|0
101502|2026-10-15 06:59:59.000 +07:00|1|1|1|0
101503|2026-10-15 07:00:00.000 +07:00|1|1|1|1
101594|2026-10-15 23:59:59.500 +07:00|1|0|1|1
101595|2026-10-16 00:00:00.000 +07:00|0|0|1|1
```

Chỉ cột `a_nua_mo` có đúng mẫu 1-1-1-1-0. Bảng này nên đi kèm mọi báo cáo theo ngày như một test: thêm chuyến biên vào fixture, chạy bộ lọc, so từng dòng.

### Dựng biên từ tham số ngày

Ứng dụng thường truyền vào một **ngày** (`date`), không phải hai mốc. Dựng mốc đầu bằng cách gắn múi giờ VN vào nửa đêm của ngày đó, mốc cuối là mốc đầu cộng một ngày:

```sql
DECLARE @ngay date = '2026-10-15';
DECLARE @tu  datetimeoffset(3) = CAST(@ngay AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time';
DECLARE @den datetimeoffset(3) = DATEADD(day, 1, @tu);

SELECT COUNT(*) FROM toinoi.trips WHERE requested_at >= @tu AND requested_at < @den;
```

```text
tu|tu_cach_2|den_truoc|so_chuyen
--|---------|---------|---------
2026-10-15 00:00:00.000 +07:00|2026-10-15 00:00:00.000 +07:00|2026-10-16 00:00:00.000 +07:00|106
```

Với `datetime2`, `AT TIME ZONE` **gắn** độ lệch của múi giờ vào giờ trên đồng hồ (MS-004), đúng điều cần ở đây. Cột `tu_cach_2` là `TODATETIMEOFFSET(CAST(@ngay AS datetime2(3)), '+07:00')`, cho cùng khoảnh khắc; dùng được cho Việt Nam vì độ lệch cố định quanh năm, còn với múi giờ có giờ mùa hè thì phải dùng `AT TIME ZONE` (mục 9). Bộ lọc giữ nguyên cột `requested_at`, không bọc hàm, nên vẫn SARGable khi Phần 3 thêm index.

## 5. Bẫy datetime: 23:59:59.999 thành 00:00 hôm sau

Hệ thống cũ hay lưu giờ bằng kiểu `datetime`, chính xác tới khoảng 1/300 giây: phần mili giây chỉ có đuôi .000, .003 hoặc .007. Mốc `'…23:59:59.999'` vì thế bị **làm tròn** sang ngày hôm sau. Lab chép giờ đặt (giờ VN) sang một bảng tạm kiểu `datetime` rồi lọc:

```text
moc_999|moc_997|between_999|between_997|nua_mo
-------|-------|-----------|-----------|------
2026-10-16 00:00:00.000|2026-10-15 23:59:59.997|107|106|106
```

`BETWEEN '2026-10-15' AND '2026-10-15 23:59:59.999'` thực chất là `… AND '2026-10-16 00:00:00'` và đếm thừa chuyến 101595. Mốc `.997` cho đúng 106, nhưng chỉ đúng với `datetime`: đổi cột sang `datetime2` hay `datetimeoffset` thì nó lại bỏ sót khoảng `.997–.999`. Khoảng nửa hở `>= '2026-10-15' AND < '2026-10-16'` đúng với mọi kiểu và mọi độ chính xác, nên không cần nhớ con số `.997` nào.

## 6. DATEDIFF đếm ranh giới theo UTC, DATEPART đọc giờ như đã lưu

Hai hàm quen thuộc đọc `datetimeoffset` theo hai cách khác nhau:

```text
cung_ngay_vn_cach_2_gio|khac_ngay_vn_cach_nua_giay|bo_do_lech_roi_tinh|gio_theo_gia_tri_luu
-----------------------|--------------------------|-------------------|--------------------
1|0|1|6
```

- `DATEDIFF(day, '15/10 06:00 +07:00', '15/10 08:00 +07:00')` ra **1**: `DATEDIFF` đếm số **ranh giới** (nửa đêm) vượt qua, và với `datetimeoffset` nó tính trên giá trị UTC; 23:00 ngày 14 tới 01:00 ngày 15 theo UTC vượt một nửa đêm.
- `DATEDIFF(day, '15/10 23:59:59.5 +07:00', '16/10 00:00 +07:00')` ra **0**: theo UTC, hai mốc cùng nằm trong ngày 15/10. Đây là cảnh báo "kéo sang hôm sau" bị im ở đầu bài.
- Ép cả hai về `datetime2` (giờ VN trên đồng hồ) trước khi tính thì ra 1, đúng nghĩa "khác ngày theo giờ VN".
- `DATEPART(hour, '15/10 06:00 +07:00')` ra **6**: `DATEPART` đọc giờ trên đồng hồ như đã lưu, không quy về UTC.

Muốn đếm "khác ngày theo giờ VN", đổi về giờ VN rồi bỏ độ lệch (`CAST(SWITCHOFFSET(x, '+07:00') AS datetime2(3))`) trước khi gọi `DATEDIFF`. Muốn đo **khoảng thời gian** (phút, giây) thì `DATEDIFF` trên `datetimeoffset` là đúng, vì quy về UTC không làm sai hiệu số.

## 7. Gom theo ngày VN, theo giờ, và DATETRUNC, DATE_BUCKET

Gom theo ngày VN và ngày UTC cho ba ngày giữa tháng:

```text
ngay|theo_ngay_vn|theo_ngay_utc
----|------------|-------------
2026-10-14|100|102
2026-10-15|106|105
2026-10-16|103|104
```

Biên tháng cũng lệch: kỳ 01–30/10 giờ VN có 3.168 chuyến, nhưng gom theo ngày UTC tương ứng (30/09–30/10) ra 3.174, vì 6 chuyến đặt 00:30–00:58 ngày 31/10 giờ VN vẫn là 30/10 theo UTC. Cả tháng có 383 chuyến mà ngày VN khác ngày UTC, đúng bằng tổng số chuyến đặt 0 giờ tới 6 giờ:

```text
gio_vn|so_chuyen
------|---------
0|68
1|28
2|39
3|39
4|25
5|61
6|123
7|193
```

Lab tính giờ bằng `DATEPART(hour, SWITCHOFFSET(requested_at, '+07:00'))`: đổi về `+07:00` trước, để kết quả không phụ thuộc dữ liệu được ghi bằng độ lệch nào.

### DATETRUNC và DATE_BUCKET (SQL Server 2022)

`DATETRUNC(phan, x)` cắt bỏ mọi phần nhỏ hơn `phan` (ngày, tháng, giờ…). `DATE_BUCKET(phan, n, x [, goc])` chia trục thời gian thành các khung dài `n` đơn vị tính từ một mốc gốc (origin), và trả mốc đầu của khung chứa `x`. Hai hàm trả cùng kiểu với đầu vào, nhưng đọc `datetimeoffset` khác nhau. Thử trên một mốc sát biên, `06:59:59 +07:00` ngày 15/10:

```text
datetrunc_day|date_bucket_day|date_bucket_day_goc_vn|date_bucket_15_phut|datetrunc_month|date_bucket_month
-------------|---------------|----------------------|-------------------|---------------|-----------------
2026-10-15 00:00:00.000 +07:00|2026-10-14 07:00:00.000 +07:00|2026-10-15 00:00:00.000 +07:00|2026-10-15 06:45:00.000 +07:00|2026-10-01 00:00:00.000 +07:00|2026-10-01 07:00:00.000 +07:00
```

- `DATETRUNC(day, …)` cắt theo giờ trên đồng hồ **như đã lưu** (`+07:00`): ra 00:00 ngày 15, như `CAST AS date` ở cách (c), kể cả điểm yếu của nó.
- `DATE_BUCKET(day, 1, …)` không ghi origin thì tính khung từ `1900-01-01 00:00` **UTC**: "ngày" của nó chạy từ 07:00 sáng tới 07:00 sáng hôm sau giờ VN, nên 06:59:59 ngày 15 thuộc khung bắt đầu 07:00 ngày 14. `DATE_BUCKET(month, …)` lệch y như vậy.
- Truyền origin là một nửa đêm giờ VN (`'2026-01-01 00:00:00 +07:00'`) thì khung ngày khớp ngày VN.
- Khung **15 phút** không bị ảnh hưởng, vì độ lệch +7 giờ là bội của 15 phút.

Đếm "ngày 15/10" bằng ba cách: `DATE_BUCKET(day)` không origin ra **105** (khung 07:00–07:00), có origin VN ra **106**, `DATETRUNC(day)` ra **106** (assertion `8b`). Quy tắc: dùng `DATE_BUCKET` cho khung theo ngày, tuần, tháng thì luôn truyền origin có `+07:00`.

### Khung 15 phút cho giờ cao điểm

Đội điều phối cần số chuyến mỗi 15 phút trong giờ cao điểm sáng 15/10. Lịch khung dựng bằng `GENERATE_SERIES` (như lịch ngày ở MS-015) để khung không có chuyến vẫn hiện:

```sql
DECLARE @tu datetimeoffset(3) = '2026-10-15 07:00:00 +07:00';
WITH khung AS (
    SELECT DATEADD(minute, 15 * value, @tu) AS bat_dau FROM GENERATE_SERIES(0, 7)
),
chuyen AS (
    SELECT DATE_BUCKET(minute, 15, requested_at) AS bat_dau, COUNT(*) AS so_chuyen
    FROM toinoi.trips
    WHERE requested_at >= @tu AND requested_at < DATEADD(hour, 2, @tu)
    GROUP BY DATE_BUCKET(minute, 15, requested_at)
)
SELECT FORMAT(k.bat_dau, 'HH:mm') AS khung_15_phut, COALESCE(c.so_chuyen, 0) AS so_chuyen
FROM khung AS k
LEFT JOIN chuyen AS c ON c.bat_dau = k.bat_dau
ORDER BY k.bat_dau;
```

```text
khung_15_phut|so_chuyen
-------------|---------
07:00|3
07:15|1
07:30|1
07:45|3
08:00|1
08:15|1
08:30|1
08:45|3
```

Khung 07:00 gồm chuyến 101503 đặt đúng 07:00:00: `DATE_BUCKET` gán mốc đầu cho khung bắt đầu từ nó, cùng quy ước nửa hở `[07:00, 07:15)`. Cả tháng, khung đông nhất là **09:00 ngày 10/10 với 7 chuyến**, trong đó 6 chuyến là lô API của khách doanh nghiệp: con số đỉnh của một khung ngắn dễ bị một lô đặt hàng loạt chi phối.

## 8. Thứ trong tuần phụ thuộc DATEFIRST

15/10/2026 là thứ Năm. `DATEPART(weekday, …)` đánh số theo `@@DATEFIRST` của **phiên**, và giá trị này đi theo ngôn ngữ của login (`us_english`: 7, tuần bắt đầu Chủ nhật) hoặc `SET DATEFIRST`:

```text
datefirst|weekday|ten_thu|datetrunc_week|datetrunc_iso_week|date_bucket_week
---------|-------|-------|--------------|------------------|----------------
7|5|Thursday|2026-10-11|2026-10-12|2026-10-12
```

Sau `SET DATEFIRST 1`, cùng ngày đó `DATEPART(weekday)` ra 4 và `DATETRUNC(week)` ra 12/10 (assertion `9b`). `DATETRUNC(iso_week)` và `DATE_BUCKET(week)` không đổi: tuần ISO luôn bắt đầu thứ Hai, còn origin mặc định 01/01/1900 của `DATE_BUCKET` là một ngày thứ Hai. `DATENAME(weekday)` còn đổi theo ngôn ngữ của phiên.

Báo cáo "chuyến theo thứ" không nên phụ thuộc thiết lập của người chạy. Cách lab dùng: số ngày kể từ 01/01/1900 chia lấy dư cho 7, ra 0 = thứ Hai … 6 = Chủ nhật bất kể `DATEFIRST`:

```text
thu_0_la_thu_hai|so_chuyen
----------------|---------
0|403
1|385
2|402
3|505
4|502
5|490
6|487
```

Đọc kết quả cẩn thận: từ 01/10 tới 30/10 có 5 ngày thứ Năm (3) và 5 ngày thứ Sáu (4), các thứ khác chỉ 4 ngày; thứ Bảy (5) còn được cộng 6 chuyến đầu ngày 31/10. So tổng theo thứ là so các mẫu số khác nhau: chia cho số ngày trước khi so.

## 9. Giờ mùa hè: vì sao ghi múi giờ bằng tên

Việt Nam không có giờ mùa hè, nên `+07:00` và `SE Asia Standard Time` cho cùng kết quả quanh năm. Đối tác ở New York thì khác: ngày 01/11/2026 đồng hồ lùi từ `-04:00` về `-05:00`, ngày đó dài 25 giờ.

```text
bat_dau|dateadd_giu_do_lech|doi_ve_gio_dia_phuong|cong_tren_dong_ho|so_gio_cua_ngay
-------|-------------------|---------------------|-----------------|---------------
2026-11-01 00:00:00 -04:00|2026-11-02 00:00:00 -04:00|2026-11-01 23:00:00 -05:00|2026-11-02 00:00:00 -05:00|25
```

`DATEADD(day, 1, …)` trên `datetimeoffset` **giữ nguyên độ lệch** `-04:00`, nên kết quả là 23:00 tối 01/11 theo giờ địa phương, không phải nửa đêm. Cách đúng là cộng trên giờ đồng hồ (`datetime2`) rồi mới `AT TIME ZONE 'Eastern Standard Time'`, như cách dựng biên ở mục 4. Mốc 01:30 sáng 01/11 xảy ra **hai lần** ở New York; `AT TIME ZONE` với `datetime2` chọn lần thứ nhất (`-04:00`), còn đi từ UTC (05:30Z và 06:30Z) thì phân biệt được hai lần (assertion `10b`). Lưu khoảnh khắc bằng `datetimeoffset`, và chỉ chuyển sang giờ địa phương bằng tên múi giờ ở bước hiển thị hoặc dựng biên.

## 10. Làm trong DBStudio

- Chạy cả tệp bằng `Ctrl+Shift+Enter`; mỗi bảng đếm là một result set riêng, dòng `PASS` ở tab **Thông điệp**.
- `SET DATEFIRST` và `SET LANGUAGE` là thiết lập **của phiên**, mà mỗi tab query của DBStudio là một phiên riêng: chạy `SET DATEFIRST 1` ở một tab không đổi kết quả của tab khác. Muốn kết quả lặp lại được, đặt chúng ở đầu script như lab.
- Bảng khung 15 phút export được ra CSV hoặc XLSX cho đội điều phối.

## 11. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Ba người đếm một ngày ra ba số | `BETWEEN`, mốc không ghi độ lệch, cắt ngày theo UTC | Nửa hở `>= @tu AND < @den`, mốc `+07:00` |
| Số theo ngày đổi khi có job nhập dữ liệu mới | `CAST(… AS date)` / `DATETRUNC(day)` theo độ lệch đã lưu | Đổi sang giờ VN trước (`AT TIME ZONE`), hoặc nửa hở |
| Ngày 15 đếm thừa chuyến 00:00 ngày 16 | `datetime` làm tròn `.999` lên ngày sau | Nửa hở, không dùng `.997`/`.999` |
| `DATE_BUCKET(day)` cắt ngày lúc 07:00 | Origin mặc định theo UTC | Truyền origin `'… 00:00:00 +07:00'` |
| `DATEDIFF(day)` ra 0 cho hai ngày VN khác nhau | Tính trên UTC | Đổi về giờ VN, bỏ độ lệch rồi mới `DATEDIFF` |
| Thứ trong tuần lệch 1 giữa hai máy | `DATEFIRST` theo ngôn ngữ login | `SET DATEFIRST` tường minh hoặc công thức `% 7` |
| Lỗi 9820 với `'Asia/Ho_Chi_Minh'` | SQL Server dùng tên múi giờ Windows | `'SE Asia Standard Time'` |
| "+1 ngày" ra 23:00 ở nơi có giờ mùa hè | `DATEADD` giữ độ lệch cũ | Cộng trên `datetime2` rồi `AT TIME ZONE` |

## 12. Đánh đổi

- **Nửa hở hay hàm trên cột:** `CAST(x AT TIME ZONE … AS date) = @ngay` dễ đọc và đúng, nhưng tính lại ở mỗi dòng và làm mất khả năng dùng index; nửa hở để nguyên cột.
- **`DATETRUNC` hay `DATE_BUCKET`:** `DATETRUNC` gọn cho ngày/tháng/năm và theo giờ đã lưu; `DATE_BUCKET` linh hoạt (15 phút, 2 giờ, 7 ngày) nhưng phải chọn origin, và quên origin là lệch múi giờ.
- **Lưu `+07:00` hay UTC:** lưu `+07:00` dễ đọc khi xem dữ liệu thô và làm `CAST AS date` ra đúng ngày VN; lưu UTC thống nhất với hệ thống khác. Câu lệnh viết đúng (nửa hở, `AT TIME ZONE`) thì cho cùng kết quả với cả hai cách lưu; chỉ câu lệnh cẩu thả mới phụ thuộc vào cách ghi dữ liệu.
- **Bảng lịch cố định hay `GENERATE_SERIES`:** `GENERATE_SERIES` đủ cho báo cáo ngắn; bảng lịch có cột thứ, tuần, ngày lễ thì đáng giá khi nhiều báo cáo cần cùng định nghĩa.

## 13. So với PostgreSQL

Cùng dữ liệu với [PG-016](/articles/postgresql-date-time-timezone): đúng 106 chuyến ngày 15/10, 100/106/103 theo ngày VN và 102/105/104 theo ngày UTC, 3.168 so với 3.174 ở biên tháng. Khác biệt thật:

| Chủ đề | PostgreSQL 17 | SQL Server 2022 |
|---|---|---|
| Kiểu thời điểm | `timestamptz` lưu UTC, không giữ độ lệch gốc | `datetimeoffset` lưu kèm độ lệch |
| Múi giờ của phiên | `TimeZone` quyết định hiển thị, `::date`, chuỗi không ghi múi giờ | Không có; chuỗi không ghi độ lệch = `+00:00` |
| Tên múi giờ | IANA (`Asia/Ho_Chi_Minh`) | Windows (`SE Asia Standard Time`) |
| Cắt theo ngày | `date_trunc('day', x, 'Asia/Ho_Chi_Minh')` | `DATETRUNC(day, x)` theo độ lệch đã lưu |
| Khung tuỳ ý | `date_bin(interval, x, origin)` | `DATE_BUCKET(part, n, x, origin)`, origin mặc định UTC |
| Hiệu hai thời điểm | `interval` | `DATEDIFF(đơn vị, a, b)`, đếm ranh giới theo UTC |
| Thứ trong tuần | `extract(isodow …)` cố định | `DATEPART(weekday)` theo `DATEFIRST` của phiên |

## 14. Bài tập

**Bài 1.** Đội tài chính cần số chuyến hoàn thành và doanh thu (tổng cước) của ngày 15/10 giờ VN, viết sao cho ứng dụng chỉ truyền vào một tham số `date`.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">DECLARE @ngay date = '2026-10-15';
DECLARE @tu datetimeoffset(3) = CAST(@ngay AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time';
SELECT COUNT(*) AS so_chuyen, SUM(fare) AS doanh_thu
FROM toinoi.trips
WHERE status = 'completed' AND requested_at &gt;= @tu AND requested_at &lt; DATEADD(day, 1, @tu);</code></pre>
<p>Kết quả đã chạy: <b>89 chuyến, 5.862.000 đ</b> (assertion <code>BT1</code>). Biên dựng từ tham số nên không phụ thuộc cách viết chuỗi ở phía ứng dụng; cột <code>requested_at</code> để nguyên, không bọc hàm.</p>
</details>

**Bài 2.** Đếm chuyến Đà Nẵng từng ngày 19–21/10 theo giờ VN, sao cho ngày bão 20/10 vẫn có dòng với số 0.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT l.ngay, x.n AS chuyen_dn
FROM (SELECT DATEADD(day, value, CAST('2026-10-19' AS date)) AS ngay FROM GENERATE_SERIES(0, 2)) AS l
CROSS APPLY (SELECT COUNT(*) AS n FROM toinoi.trips AS t
             WHERE t.city_id = 3
               AND t.requested_at &gt;= CAST(l.ngay AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time'
               AND t.requested_at &lt;  CAST(DATEADD(day, 1, l.ngay) AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time') AS x
ORDER BY l.ngay;</code></pre>
<p>Kết quả đã chạy: 19/10 <b>13</b>, 20/10 <b>0</b>, 21/10 <b>17</b> (assertion <code>BT2</code>). <code>COUNT(*)</code> trong <code>CROSS APPLY</code> luôn trả một dòng, kể cả khi không có chuyến nào, nên ngày bão không biến mất; mỗi ngày có biên nửa hở riêng dựng từ lịch.</p>
</details>

**Bài 3.** Tính thời gian trung bình (phút, một chữ số lẻ) từ lúc đặt tới lúc hoàn thành theo loại dịch vụ. `DATEDIFF` trên `datetimeoffset` tính theo UTC: điều đó có làm sai kết quả không?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT kind,
       CAST(ROUND(AVG(CAST(DATEDIFF(second, requested_at, completed_at) AS decimal(12, 2))) / 60, 1) AS decimal(6, 1)) AS phut
FROM toinoi.trips
WHERE status = 'completed'
GROUP BY kind
ORDER BY kind;</code></pre>
<p>Kết quả đã chạy: food <b>29,0</b>, parcel <b>28,0</b>, ride <b>28,4</b> phút (assertion <code>BT3</code>, khớp PG-016). Không sai: đo khoảng thời gian thì quy về UTC là đúng, vì hiệu hai khoảnh khắc không phụ thuộc múi giờ. <code>DATEDIFF</code> chỉ gây bất ngờ khi đếm <em>ranh giới</em> (ngày, tuần) mà người đọc hiểu theo giờ VN. Lấy hiệu theo giây rồi chia 60 để không mất phần lẻ, và ép <code>decimal</code> trước <code>AVG</code>.</p>
</details>

## 15. Đọc tiếp

- Microsoft Learn: [datetimeoffset](https://learn.microsoft.com/sql/t-sql/data-types/datetimeoffset-transact-sql), [datetime](https://learn.microsoft.com/sql/t-sql/data-types/datetime-transact-sql), [AT TIME ZONE](https://learn.microsoft.com/sql/t-sql/queries/at-time-zone-transact-sql), [SWITCHOFFSET](https://learn.microsoft.com/sql/t-sql/functions/switchoffset-transact-sql), [TODATETIMEOFFSET](https://learn.microsoft.com/sql/t-sql/functions/todatetimeoffset-transact-sql), [sys.time_zone_info](https://learn.microsoft.com/sql/relational-databases/system-catalog-views/sys-time-zone-info-transact-sql), [DATETRUNC](https://learn.microsoft.com/sql/t-sql/functions/datetrunc-transact-sql), [DATE_BUCKET](https://learn.microsoft.com/sql/t-sql/functions/date-bucket-transact-sql), [DATEDIFF](https://learn.microsoft.com/sql/t-sql/functions/datediff-transact-sql), [DATEPART](https://learn.microsoft.com/sql/t-sql/functions/datepart-transact-sql), [SET DATEFIRST](https://learn.microsoft.com/sql/t-sql/statements/set-datefirst-transact-sql).
- Cùng chủ đề trên PostgreSQL: [PG-016 · Ngày giờ và múi giờ](/articles/postgresql-date-time-timezone).
- Lab: `examples/toi-noi-mssql/p02/16-ngay-gio-mui-gio.sql`. Index cho cột thời gian và bộ lọc SARGable: MS-024 (Phần 3).

Bài tiếp theo: [MS-017 · Upsert: MERGE và các mẫu an toàn hơn](/articles/mssql-upsert-merge)
