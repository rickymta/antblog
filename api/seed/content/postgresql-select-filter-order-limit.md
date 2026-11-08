---
slug: postgresql-select-filter-order-limit
title: "SELECT đầu tiên: lọc, sắp xếp và giới hạn kết quả"
summary: "Hai nhân viên CSKH của Tới Nơi cùng đếm «chuyến ngày 02/11» ra 8 và 9, còn trang 2 của danh sách lặp lại chuyến ở trang 1. Học SELECT, WHERE, ORDER BY và LIMIT để mỗi câu hỏi có đúng một đáp án."
category: postgresql
level: intermediate
tags: [postgresql, pg-007, select, where, order-by, limit, timezone, toi-noi]
language: vi
---

Sáng 03/11, trưởng ca CSKH của Tới Nơi nhờ hai nhân viên đếm số chuyến khách đặt trong **ngày 02/11** để đối chiếu với khiếu nại. Một người báo 8, người kia báo 9. Cả hai đều chạy SQL trên cùng database, cùng bảng `trips`, không ai gõ sai tên cột. Cùng lúc đó, màn hình «chuyến theo loại dịch vụ» của đội vận hành, mỗi trang 3 dòng, hiện chuyến 100002 ở cả trang 1 lẫn trang 2, trong khi chuyến 100009 không xuất hiện ở trang nào.

Không có lỗi nào được báo. PostgreSQL trả đúng những gì câu lệnh yêu cầu — chỉ là câu lệnh chưa nói rõ «ngày 02/11» theo múi giờ nào, và hai chuyến cùng loại thì chuyến nào đứng trước. Bài này đi qua các mệnh đề của một câu đọc dữ liệu và chỉ ra chỗ mỗi mệnh đề có thể đúng cú pháp mà sai nghiệp vụ.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-007 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid">PG-004 · kiểu tiền, thời gian và định danh</a>, <a href="/articles/postgresql-insert-update-delete-returning">PG-006 · INSERT, UPDATE, DELETE và RETURNING</a>. Lab: <code>examples/toi-noi-postgresql/p01</code> (tệp <code>05-select.sql</code>). Sau bài này, bạn lấy được chuyến theo trạng thái, theo khoảng thời gian giờ Việt Nam và theo một thứ tự xác định.</p>
</div>

## 1. Dữ liệu dùng trong bài

Mọi ví dụ chạy trên fixture của lab: 15 chuyến cố định (ảnh chụp 08:30 ngày 03/11/2026, giờ VN) cộng chuyến **100029** do vòng đời ở PG-006 tạo — 16 chuyến. Tái lập rồi chạy các câu của bài:

```bash
cd examples/toi-noi-postgresql
node run.mjs p01/00-reset.sql p01/01-schema.sql p01/02-fixture-valid.sql \
  p01/03-fixture-invalid.sql p01/04-lifecycle.sql p01/05-select.sql
```

Kết quả bên dưới chép từ lần chạy thật trên PGlite (PostgreSQL 17.5), đã bỏ các dòng `-- [lab]` của runner. Mỗi tệp của lab đặt sẵn `SET search_path TO toinoi;` và `SET TIME ZONE 'Asia/Ho_Chi_Minh';`, nên tên bảng không cần tiền tố schema và giờ hiển thị theo `+07`. Fixture cài sẵn vài «cái bẫy»: 100003 và 100004 cùng đặt lúc 12:00 ngày 02/11; 100008 đặt 00:10 ngày 03/11 giờ VN — vẫn là ngày 02/11 theo UTC; 100013, 100014 chưa có tài xế nên `accepted_at` là NULL.

## 2. Một câu SELECT được đọc theo thứ tự nào?

**`SELECT`** là câu lệnh đọc dữ liệu. Ở dạng cơ bản, nó có năm phần, viết theo thứ tự cố định:

```sql
SELECT code, kind, city_id, fare, completed_at   -- danh sách cột muốn nhận (select list)
FROM trips                                       -- đọc từ bảng nào
WHERE status = 'completed'                       -- chỉ giữ dòng thoả điều kiện
ORDER BY code                                    -- sắp xếp kết quả
LIMIT 10;                                        -- lấy tối đa bao nhiêu dòng
```

Thứ tự **viết** khác thứ tự **ý nghĩa**. Về logic, câu lệnh được hiểu là: lấy các dòng của `FROM` → giữ dòng thoả `WHERE` → tính danh sách cột của `SELECT` (kể cả tên đặt lại bằng `AS`) → sắp theo `ORDER BY` → cắt bằng `OFFSET`/`LIMIT`. Đó là cách *đọc nghĩa*; cách thực thi thật do bộ lập kế hoạch (planner) chọn — Phần 3 sẽ xem bằng `EXPLAIN`.

Hiểu thứ tự logic giải thích ngay một lỗi người mới hay gặp: tên đặt bằng `AS` chưa tồn tại lúc `WHERE` chạy, nhưng đã có lúc `ORDER BY` chạy.

```sql
SELECT code, fare / 1000 AS nghin_dong FROM trips WHERE nghin_dong > 60 ORDER BY code;
```

```text
ERROR:  column "nghin_dong" does not exist
```

Lọc theo cột gốc thì chạy, và `ORDER BY` dùng được tên mới: `… WHERE fare > 60000 ORDER BY nghin_dong DESC, code` trả 6 dòng, đầu tiên là `100005 | 126`. (`fare` là `integer` đồng nên `fare / 1000` là phép chia nguyên — chỉ dùng để hiển thị, đừng dùng để tính tiền.)

Nên liệt kê cột thay vì `SELECT *`: không kéo cột thừa, không đổi nghĩa khi bảng thêm cột (PG-009 sẽ thêm `vehicle_plate` vào `drivers`).

## 3. WHERE: chỉ giữ những dòng cần

**`WHERE`** nhận một biểu thức điều kiện (predicate) và giữ dòng nào biểu thức đó ra `true`. Dòng ra `false` **hoặc NULL** đều bị loại.

Câu 1 của `05-select.sql` chính là câu mẫu ở mục 2, bỏ `LIMIT`:

```text
  code  |  kind  | city_id |  fare  |      completed_at
--------+--------+---------+--------+------------------------
 100000 | ride   |       1 |  45000 | 2026-11-02 08:24:10+07
 100001 | parcel |       2 |  38000 | 2026-11-02 10:02:00+07
 …
 100008 | ride   |       1 |      0 | 2026-11-03 00:30:00+07
 100029 | ride   |       2 |  54000 | 2026-11-03 09:31:00+07
(8 rows)
```

Lọc theo danh sách bằng `IN` — «chuyến đang có tài xế trên đường» là bốn trạng thái từ `accepted` tới `in_transit` của vòng đời:

```sql
SELECT code, kind, status, requested_at
FROM trips
WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
ORDER BY requested_at, code;
```

```text
  code  |  kind  |   status   |      requested_at
--------+--------+------------+------------------------
 100009 | food   | picked_up  | 2026-11-03 08:00:00+07
 100010 | ride   | in_transit | 2026-11-03 08:05:00+07
 100011 | parcel | arriving   | 2026-11-03 08:10:00+07
 100012 | ride   | accepted   | 2026-11-03 08:20:00+07
(4 rows)
```

Chuỗi phải khớp đúng giá trị lưu; gõ sai chính tả không báo lỗi, chỉ ra ít dòng hơn. CHECK `ck_trips_status` (PG-005) giữ cột trong 8 giá trị đã biết, nên danh sách `IN` luôn đối chiếu được với tài liệu nghiệp vụ.

### Hai cái bẫy của WHERE

**So sánh với NULL bằng `=`.** Theo tài liệu, `7 = NULL` ra NULL («không biết»), không phải `true`/`false`; `WHERE` loại dòng ra NULL nên câu dưới luôn trả 0 dòng:

```sql
SELECT code, status FROM trips WHERE driver_id = NULL;
```

```text
 code | status
------+--------
(0 rows)
```

Viết `WHERE driver_id IS NULL ORDER BY code` thì ra đúng 3 chuyến chưa có tài xế: 100006 (`cancelled` trước khi có người nhận), 100013 (`searching`), 100014 (`requested`).

**`AND` mạnh hơn `OR`.** Muốn «chuyến hoàn thành hoặc bị huỷ, ở TP. Hồ Chí Minh (`city_id = 1`)». Viết thiếu ngoặc:

```sql
SELECT code, status, city_id FROM trips
WHERE status = 'completed' OR status = 'cancelled' AND city_id = 1
ORDER BY code;
```

PostgreSQL hiểu thành `status = 'completed' OR (status = 'cancelled' AND city_id = 1)` và trả **9** dòng — gồm cả chuyến hoàn thành ở Hà Nội (100001, 100005, 100029) và Đà Nẵng (100004). Thêm ngoặc thì còn đúng **5** dòng ở SG: 100000, 100002, 100006, 100007, 100008. Hễ trộn `AND` với `OR` thì đặt ngoặc, kể cả khi bạn nhớ thứ tự ưu tiên.

## 4. Lọc theo khoảng thời gian giờ Việt Nam

Quay lại câu hỏi «chuyến đặt trong ngày 02/11». Với Tới Nơi, «ngày» luôn là **ngày theo giờ Việt Nam** — khách đặt lúc 00:10 sáng 03/11 không nghĩ mình đặt «ngày 02/11».

### Khoảng nửa hở `[from, to)`

Cách viết khuyến nghị là **khoảng nửa hở**: lấy mốc đầu (gồm), bỏ mốc cuối (không gồm), cả hai mốc ghi rõ múi giờ `+07`:

```sql
SELECT code, kind, status, requested_at
FROM trips
WHERE requested_at >= '2026-11-02 00:00:00+07'
  AND requested_at <  '2026-11-03 00:00:00+07'
ORDER BY requested_at, code;
```

```text
  code  |  kind  |  status   |      requested_at
--------+--------+-----------+------------------------
 100000 | ride   | completed | 2026-11-02 07:58:00+07
 …
 100003 | food   | cancelled | 2026-11-02 12:00:00+07
 100004 | food   | completed | 2026-11-02 12:00:00+07
 …
 100007 | parcel | completed | 2026-11-02 23:50:00+07
(8 rows)
```

Vì sao nửa hở tốt hơn hai cách quen tay?

| Cách viết | Vấn đề |
|---|---|
| `BETWEEN '…02 00:00' AND '…02 23:59:59'` | Bỏ sót chuyến lúc `23:59:59.5` — `timestamptz` lưu tới micro giây |
| `BETWEEN '…02 00:00' AND '…03 00:00'` | `BETWEEN` **gồm cả hai đầu**; chuyến đúng 00:00 ngày 03/11 bị đếm vào cả hai ngày |
| `>= '…02 00:00' AND < '…03 00:00'` | Các ngày nối nhau không chồng, không hở; mốc cuối của ngày này là mốc đầu của ngày sau |

Fixture không có chuyến nào đúng 00:00 ngày 03/11, nên ta kiểm cái bẫy thứ hai trực tiếp trên một giá trị:

```sql
SELECT timestamptz '2026-11-03 00:00:00+07'
         BETWEEN '2026-11-02 00:00:00+07' AND '2026-11-03 00:00:00+07' AS thuoc_ngay_02,
       timestamptz '2026-11-03 00:00:00+07'
         BETWEEN '2026-11-03 00:00:00+07' AND '2026-11-04 00:00:00+07' AS thuoc_ngay_03;
```

```text
 thuoc_ngay_02 | thuoc_ngay_03
---------------+---------------
 t             | t
(1 row)
```

Một khoảnh khắc thuộc hai ngày: báo cáo theo ngày cộng lại lớn hơn tổng thật.

Thêm một lý do sẽ rõ ở Phần 3: so sánh trực tiếp trên cột để nguyên giá trị cột, nên index trên `requested_at` dùng được; bọc cột trong hàm hay phép ép kiểu thường làm mất khả năng đó.

### Vì sao người thứ hai đếm ra 9?

Nhân viên thứ hai mở công cụ SQL có phiên đặt múi giờ UTC (mặc định hay gặp ở server và container). Theo PG-004, `timestamptz` lưu một **khoảnh khắc**; `TimeZone` của phiên chỉ quyết định cách hiển thị và cách hiểu chuỗi không ghi múi giờ. Đổi phiên sang UTC để thấy:

```sql
SET TIME ZONE 'UTC';

SELECT code, requested_at
FROM trips
WHERE code IN (100007, 100008)
ORDER BY code;
```

```text
  code  |      requested_at
--------+------------------------
 100007 | 2026-11-02 16:50:00+00
 100008 | 2026-11-02 17:10:00+00
(2 rows)
```

Theo UTC, chuyến 100008 rơi vào ngày 02/11. Mọi cách viết dựa vào múi giờ phiên đều đếm nhầm nó:

```sql
SELECT count(*) FILTER (WHERE requested_at::date = DATE '2026-11-02') AS theo_mui_gio_phien,
       count(*) FILTER (WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = DATE '2026-11-02')
         AS theo_gio_viet_nam
FROM trips;
```

```text
 theo_mui_gio_phien | theo_gio_viet_nam
--------------------+-------------------
                  9 |                 8
(1 row)
```

(`count(*) FILTER (…)` đếm có điều kiện — Phần 2 sẽ giải thích; ở đây chỉ cần đọc hai con số.) Ép `::date` dùng múi giờ **phiên**, nên kết quả phụ thuộc người chạy. Mốc **không** ghi múi giờ cũng vậy: vẫn ở phiên UTC, `WHERE requested_at >= '2026-11-02 00:00:00' AND requested_at < '2026-11-03 00:00:00'` đếm ra **9** (đã chạy thật); cùng câu với mốc ghi `+07` trả **8** ở bất kỳ phiên nào. Tài liệu PostgreSQL mô tả đúng hành vi này: chuỗi nhập cho `timestamptz` không nêu múi giờ thì được coi là ở múi giờ của tham số `TimeZone`.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Quy tắc lọc ngày của Tới Nơi</span>
<p>Mốc lọc luôn ghi rõ <code>+07</code> (hoặc ứng dụng truyền tham số <code>timestamptz</code> đã có múi giờ), luôn dạng nửa hở <code>&gt;= from AND &lt; to</code>. Câu nào đổi kết quả sau <code>SET TIME ZONE 'UTC'</code> là câu đang dựa vào phiên. PG-016 ở Phần 2 sẽ quay lại với tuần, tháng và kiểm hai biên.</p>
</div>

## 5. ORDER BY: sắp xếp và tie-breaker

Không có `ORDER BY`, PostgreSQL trả dòng theo thứ tự **không xác định**. Tài liệu nói rõ thứ tự khi đó phụ thuộc kiểu quét, kiểu nối và thứ tự trên đĩa, và «must not be relied on» — không được dựa vào, dù bảng nhỏ trong lab có thể trả đúng thứ tự chèn nhiều lần liền.

`ORDER BY` nhận một hay nhiều khoá sắp xếp, mỗi khoá có chiều riêng: `ASC` (tăng, mặc định) hoặc `DESC` (giảm). `ORDER BY requested_at DESC, code` nghĩa là giờ đặt giảm dần, **code tăng dần** — chiều không lan sang khoá sau.

### Khoá hoà nhau thì sao?

Khi nhiều dòng bằng nhau ở mọi khoá sắp xếp, PostgreSQL được phép trả chúng theo thứ tự bất kỳ. Câu 6 của lab lấy hai chuyến cùng đặt lúc 12:00 với `ORDER BY requested_at` và nhận 100003 rồi 100004 — thứ tự may mắn, không phải cam kết. Cách sửa là thêm **tie-breaker** — một khoá sắp xếp cuối cùng có giá trị duy nhất, thường là khoá chính hoặc cột UNIQUE. Ở Tới Nơi đó là `code` (`uq_trips_code`). Câu 5 của lab, ba chuyến đặt gần nhất:

```sql
SELECT code, kind, status, requested_at
FROM trips
ORDER BY requested_at DESC, code DESC
LIMIT 3;
```

```text
  code  | kind |  status   |      requested_at
--------+------+-----------+------------------------
 100029 | ride | completed | 2026-11-03 09:00:00+07
 100014 | ride | requested | 2026-11-03 08:29:30+07
 100013 | food | searching | 2026-11-03 08:25:00+07
(3 rows)
```

Một tập khoá sắp xếp gọi là **xác định** (deterministic) khi không hai dòng nào bằng nhau trên toàn bộ tập khoá. `(requested_at, code)` xác định vì riêng `code` đã duy nhất. `(kind, status)` thì không.

### NULL đứng ở đâu?

Mặc định, NULL được coi là **lớn hơn** mọi giá trị: đứng cuối khi `ASC`, đứng **đầu** khi `DESC`. Sắp «chuyến sáng 03/11, ai được nhận muộn nhất lên đầu»:

```sql
SELECT code, status, accepted_at
FROM trips
WHERE requested_at >= '2026-11-03 08:00:00+07'
ORDER BY accepted_at DESC, code;
```

```text
  code  |   status   |      accepted_at
--------+------------+------------------------
 100013 | searching  |
 100014 | requested  |
 100029 | completed  | 2026-11-03 09:01:00+07
 100012 | accepted   | 2026-11-03 08:21:00+07
 100011 | arriving   | 2026-11-03 08:12:00+07
 100010 | in_transit | 2026-11-03 08:06:00+07
 100009 | picked_up  | 2026-11-03 08:02:00+07
(7 rows)
```

Hai chuyến chưa ai nhận chiếm đầu danh sách — có thể đúng ý (cần chú ý chuyến chưa có tài xế), có thể không. Câu 7 của lab nói rõ ý định bằng `ORDER BY accepted_at DESC NULLS LAST, code`: năm dòng có giờ nhận giữ nguyên thứ tự, 100013 rồi 100014 xuống cuối. Hai dòng NULL hoà nhau trên `accepted_at`; `code` phân định chúng.

### Sắp chữ tiếng Việt

Thứ tự chuỗi do **collation** (quy tắc so sánh chuỗi) quyết định. PGlite dùng collation `C`, so theo byte UTF-8: `Đ` có mã lớn hơn mọi chữ Latin không dấu, nên «Đặng Ngọc Hà» đứng sau «Võ Minh Đức»:

```sql
SELECT full_name
FROM customers
ORDER BY full_name COLLATE "C";
```

Kết quả: Lê Hoàng Cường, Nguyễn Văn An, Phạm Thu Dung, Trần Thị Bình, Võ Minh Đức, Đặng Ngọc Hà. Lab ghi `COLLATE "C"` để kết quả giống nhau giữa PGlite và server Docker (locale khác — chưa kiểm trên máy viết bài). Sắp theo thói quen người Việt cần collation phù hợp, ngoài phạm vi Phần 1.

## 6. LIMIT, OFFSET và phân trang

**`LIMIT n`** lấy tối đa `n` dòng đầu của kết quả đã sắp; **`OFFSET k`** bỏ qua `k` dòng đầu. Kết hợp lại thành phân trang: trang `p` (đánh từ 1), mỗi trang `n` dòng là `LIMIT n OFFSET (p - 1) * n`.

Tài liệu PostgreSQL cảnh báo hai điều: dùng `LIMIT` mà không có `ORDER BY` ràng thứ tự duy nhất thì nhận «một tập con không đoán trước được»; và planner tính cả `LIMIT`/`OFFSET` khi chọn kế hoạch, nên các trang khác nhau có thể dùng kế hoạch khác nhau và cho kết quả không nhất quán. Màn hình «chuyến theo loại dịch vụ» của đội vận hành dính đúng lỗi này:

```sql
SELECT code, kind, status FROM trips ORDER BY kind LIMIT 3;            -- trang 1
SELECT code, kind, status FROM trips ORDER BY kind LIMIT 3 OFFSET 3;   -- trang 2
```

```text
  code  | kind |  status
--------+------+-----------
 100003 | food | cancelled
 100002 | food | completed
 100004 | food | completed
(3 rows)

  code  |  kind  |  status
--------+--------+-----------
 100004 | food   | completed
 100002 | food   | completed
 100007 | parcel | completed
(3 rows)
```

Có 5 chuyến `food` nhưng hai trang đầu chỉ cho thấy 3, trong đó 100002 và 100004 lặp lại; 100009 và 100013 biến mất. Mỗi câu vẫn sắp đúng theo `kind`, nhưng hai câu không cam kết cùng thứ tự cho dòng hoà. Thứ tự trên là quan sát trên PGlite 17.5, lặp lại được trong lab; server hay dữ liệu khác có thể ra thứ tự khác.

Thêm `code` làm tie-breaker:

```sql
SELECT code, kind, status FROM trips ORDER BY kind, code LIMIT 3;
SELECT code, kind, status FROM trips ORDER BY kind, code LIMIT 3 OFFSET 3;
```

```text
  code  | kind |  status
--------+------+-----------
 100002 | food | completed
 100003 | food | cancelled
 100004 | food | completed
(3 rows)

  code  |  kind  |  status
--------+--------+-----------
 100009 | food   | picked_up
 100013 | food   | searching
 100001 | parcel | completed
(3 rows)
```

Mỗi dòng có một vị trí duy nhất, các trang ghép lại vừa khít.

`OFFSET` vẫn còn hai giới hạn mà tie-breaker không chữa được:

- **Chi phí:** dòng bị `OFFSET` bỏ qua vẫn phải được tính trong server; trang thứ 1.000 của lịch sử chuyến trên bảng 730 triệu dòng/năm là cái giá thật.
- **Dữ liệu chạy:** giữa hai lần bấm «trang sau», một chuyến mới chen vào đầu danh sách thì mọi dòng dịch xuống một bậc — người dùng thấy lại một dòng của trang trước.

Cả hai được giải bằng **phân trang keyset** («lấy các dòng đứng sau dòng cuối tôi đã thấy») ở PG-018, Phần 2. Ở Phần 1, `LIMIT`/`OFFSET` có tie-breaker là đủ cho danh sách ngắn.

## 7. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Hai người đếm cùng một ngày ra số khác nhau | `::date` hoặc mốc không ghi múi giờ ⇒ phụ thuộc phiên | Mốc ghi `+07`, nửa hở |
| Báo cáo theo ngày cộng lại lớn hơn tổng | `BETWEEN` gồm mốc cuối | `>= from AND < to` |
| `WHERE x = NULL` luôn rỗng | So sánh với NULL ra NULL | `IS NULL` |
| Lọc «SG» mà ra chuyến Hà Nội | Trộn `AND`/`OR` không ngoặc | Đặt ngoặc |
| Trang 2 lặp dòng trang 1 | `ORDER BY` không xác định | Thêm tie-breaker duy nhất |
| NULL chiếm đầu danh sách «mới nhất» | `DESC` mặc định `NULLS FIRST` | Ghi rõ `NULLS LAST` |
| Tên vừa đặt bằng `AS` «does not exist» | `WHERE` chạy trước select list | Lọc theo cột gốc |

## 8. Đánh đổi

- **Liệt kê cột hay `SELECT *`:** liệt kê dài hơn nhưng ổn định khi schema đổi. `*` để khám phá, liệt kê trong mã ứng dụng.
- **Tie-breaker `code` hay `id`:** cả hai duy nhất; `code` tăng theo giờ tạo nên thứ tự có nghĩa với người vận hành, `id` là uuid ngẫu nhiên.
- **Nửa hở hay hàm trên cột:** `(requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = '2026-11-02'` cũng ra đúng 8 và dễ đọc, nhưng biến đổi cột ở mỗi dòng; nửa hở để nguyên cột — có lợi khi có index (Phần 3).
- **`OFFSET` hay keyset:** `OFFSET` đơn giản, nhảy trang bất kỳ; keyset rẻ và ổn định hơn nhưng chỉ đi «trang sau/trước».

## 9. Bài tập

**Bài 1.** Đội quán ăn muốn xem các đơn đồ ăn đã hoàn thành có cước từ 50.000 đ trở lên, cước cao nhất lên đầu, thứ tự phải xác định. Viết câu lệnh và dự đoán số dòng trước khi chạy.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT code, city_id, fare, completed_at FROM trips
WHERE kind = 'food' AND status = 'completed' AND fare &gt;= 50000
ORDER BY fare DESC, code;</code></pre>
<p>Kết quả chạy thật: 2 dòng — 100002 (62.000 đ, SG) rồi 100004 (55.000 đ, Đà Nẵng). Đơn 100003 (71.000 đ) bị loại vì đã huỷ; <code>code</code> giữ thứ tự cố định khi hai đơn cùng cước.</p>
</details>

**Bài 2.** Lấy các chuyến đặt trong khung **12:00 đến trước 18:00 ngày 02/11 giờ VN**. Chuyến 100006 đặt đúng 18:00 có thuộc khung không? Câu lệnh phải cho cùng kết quả dù phiên đặt UTC.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT code, kind, requested_at FROM trips
WHERE requested_at &gt;= '2026-11-02 12:00:00+07'
  AND requested_at &lt;  '2026-11-02 18:00:00+07'
ORDER BY requested_at, code;</code></pre>
<p>Kết quả: 100003, 100004 (cùng 12:00 — mốc đầu được gồm) và 100005 (17:30). Chuyến 100006 lúc 18:00 thuộc khung kế tiếp vì mốc cuối không gồm. Hai mốc đều ghi <code>+07</code> nên kết quả không phụ thuộc <code>TimeZone</code> của phiên.</p>
</details>

**Bài 3.** Điều phối viên cần **2 chuyến chưa kết thúc có cước cao nhất** (chưa `completed`, chưa `cancelled`) để ưu tiên theo dõi. Viết câu lệnh, rồi giải thích vì sao `ORDER BY fare DESC LIMIT 2` không có tie-breaker là rủi ro dù hôm nay ra đúng.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT code, status, fare FROM trips
WHERE status NOT IN ('completed', 'cancelled')
ORDER BY fare DESC, code
LIMIT 2;</code></pre>
<p>Kết quả: 100010 (<code>in_transit</code>, 98.000 đ) và 100013 (<code>searching</code>, 85.000 đ). Hôm nay không có hai chuyến cùng cước ở ranh giới nên bỏ <code>code</code> vẫn ra như vậy; nhưng nếu chuyến thứ hai và thứ ba cùng 85.000 đ, <code>LIMIT 2</code> chọn tuỳ ý và có thể đổi giữa các lần chạy.</p>
</details>

## 10. Đọc tiếp

- [PostgreSQL 17 · Sorting Rows (ORDER BY)](https://www.postgresql.org/docs/17/queries-order.html).
- [PostgreSQL 17 · LIMIT and OFFSET](https://www.postgresql.org/docs/17/queries-limit.html).
- [PostgreSQL 17 · Comparison Functions and Operators](https://www.postgresql.org/docs/17/functions-comparison.html).
- [PostgreSQL 17 · Date/Time Types](https://www.postgresql.org/docs/17/datatype-datetime.html).
- Lab: `examples/toi-noi-postgresql/p01/05-select.sql` và kết quả đầy đủ ở `p01/OUTPUT.txt`.

Bài tiếp theo: [PG-008 · Chuẩn hoá và ranh giới mô hình dữ liệu](/articles/postgresql-mo-hinh-quan-he-chuan-hoa).
