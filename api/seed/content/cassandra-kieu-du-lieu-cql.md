---
slug: cassandra-kieu-du-lieu-cql
title: "Kiểu dữ liệu CQL cho tiền, thời gian, chuỗi và định danh trong Tới Nơi"
summary: "Cước 123.456.789 đồng lưu bằng float đọc ra 123.456.792; chuỗi giờ không ghi độ lệch được hiểu theo múi giờ của node. Bài này chọn kiểu CQL cho từng cột của bảng trips: bigint cho tiền, timestamp/date cho thời gian, text cho chuỗi, uuid/timeuuid cho định danh, với số liệu thật từ lab Cassandra 5.0.9."
category: cassandra
level: intermediate
tags: [cassandra, cs-004, cassandra-5, cql, toi-noi, kieu-du-lieu, timestamp, timeuuid, bigint]
language: vi
---

Cuối tháng, bộ phận tài chính của Tới Nơi cộng doanh thu chuyến ở TP.HCM và nhận một con số lệch vài đồng so với tổng của từng hoá đơn. Cùng tuần đó, màn hình hỗ trợ khách hàng hiển thị chuyến đặt lúc 07:42 sáng thành "00:42". Hai lỗi có chung một gốc: **kiểu dữ liệu chọn sai lúc tạo bảng**. Ở Cassandra, sai kiểu còn đắt hơn ở database quan hệ, vì kiểu của một cột gần như không đổi được sau khi bảng đã có dữ liệu.

Bài này đi qua những kiểu CQL mà mô hình lịch sử chuyến của Tới Nơi dùng: tiền, thời gian, chuỗi, định danh. Mọi kết quả lấy từ lab Cassandra 5.0.9.

<div class="tkh-callout">
<span class="tkh-callout-title">CS-004 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/cassandra-keyspace-bang-khoa-chinh">CS-003 · Keyspace, bảng và khoá chính</a>. Lab: <code>examples/toi-noi-cassandra/p01/03-kieu-du-lieu.cql</code>, kết quả ở <code>p01/OUTPUT.txt</code>. Sau bài này, bạn chọn được <code>bigint</code> hay <code>decimal</code> cho tiền, <code>timestamp</code> hay <code>date</code> cho thời gian, <code>text</code> cho chuỗi, <code>uuid</code> hay <code>timeuuid</code> cho định danh, và tránh được các kiểu làm mất độ chính xác mà không báo lỗi.</p>
</div>

## 1. Vì sao chọn kiểu ở Cassandra phải cẩn thận hơn

CQL (*Cassandra Query Language*) có cú pháp giống SQL, nhưng kiểu dữ liệu đóng vai trò khác:

- **Không có ràng buộc để đỡ lỗi.** Cassandra không có `CHECK`, `FOREIGN KEY` hay `NOT NULL` cho cột thường. Kiểu dữ liệu là hàng rào duy nhất database tự giữ.
- **Kiểu cột khoá quyết định thứ tự.** Cột clustering được sắp theo kiểu của nó: `timestamp` theo thời gian, `text` theo byte UTF-8, `uuid` theo quy tắc riêng. Chọn sai kiểu là sai thứ tự đọc (CS-003).
- **Không đổi kiểu được.** Câu `ALTER TABLE … ALTER … TYPE` bị Cassandra từ chối (mục 7).

Lab tạo một bảng thử nghiệm `type_lab` riêng, không thuộc mô hình Tới Nơi, rồi xoá ở cuối file:

```sql
CREATE TABLE IF NOT EXISTS type_lab (
  id int PRIMARY KEY,
  as_float float, as_double double, as_decimal decimal, as_bigint bigint, as_int int,
  at_ts timestamp, on_date date, at_time time,
  label text, code ascii
);
```

## 2. Tiền: bigint số đồng, không dùng float/double

Tiền Việt Nam không có đơn vị lẻ, nên Tới Nơi lưu **số nguyên đồng** trong `bigint` (số nguyên có dấu 64 bit). Lab cộng 0,1 + 0,2 bằng bốn kiểu:

```sql
INSERT INTO type_lab (id, as_float, as_double, as_decimal, as_bigint) VALUES (1, 0.1, 0.1, 0.1, 100);
INSERT INTO type_lab (id, as_float, as_double, as_decimal, as_bigint) VALUES (2, 0.2, 0.2, 0.2, 200);
SELECT CAST(SUM(as_float) AS text) AS tong_float, CAST(SUM(as_double) AS text) AS tong_double,
       CAST(SUM(as_decimal) AS text) AS tong_decimal, SUM(as_bigint) AS tong_dong
FROM type_lab WHERE id IN (1, 2);
```

```text
 tong_float | tong_double         | tong_decimal | tong_dong
------------+---------------------+--------------+-----------
        0.3 | 0.30000000000000004 |          0.3 |       300

Warnings :
Aggregation query used on multiple partition keys (IN restriction)
```

`double` (số thực dấu phẩy động 64 bit) cho ra `0.30000000000000004`. Cột `float` in ra `0.3` không có nghĩa là `float` đúng: tổng chỉ tình cờ trùng với số `float` gần 0,3 nhất. Phép thử thứ hai cho thấy `float` (32 bit) mất chữ số ngay với một cước lớn:

```sql
INSERT INTO type_lab (id, as_float, as_double, as_bigint) VALUES (3, 123456789, 123456789, 123456789);
SELECT id, as_float, as_double, as_bigint, CAST(as_float AS bigint) AS float_thanh_so_nguyen FROM type_lab WHERE id = 3;
```

```text
 id | as_float   | as_double  | as_bigint | float_thanh_so_nguyen
----+------------+------------+-----------+-----------------------
  3 | 1.2346e+08 | 1.2346e+08 | 123456789 |             123456792
```

Ghi 123.456.789 đồng, đọc ra **123.456.792 đồng**, không có lỗi hay cảnh báo nào. Chú ý thêm: cqlsh làm tròn số thực khi hiển thị (mặc định `float_precision = 5`, nên thấy `1.2346e+08`), vì vậy lab phải `CAST` sang `text` hoặc `bigint` mới nhìn thấy giá trị thật. Đừng kết luận "số thực vẫn đúng" chỉ vì cqlsh in ra số tròn.

| Kiểu | Dùng cho tiền? | Ghi chú |
|---|---|---|
| `bigint` | **Có**, số nguyên đồng | Tới ±9,2 × 10<sup>18</sup>; cộng trừ chính xác |
| `decimal` | Có, khi cần phần lẻ (tỷ giá, đơn vị ngoại tệ có xu) | Số thập phân chính xác, độ dài thay đổi, nặng hơn `bigint` |
| `varint` | Hiếm khi cần | Số nguyên không giới hạn độ dài |
| `float`, `double` | **Không** | Nhị phân dấu phẩy động, mất chữ số mà không báo lỗi |

## 3. int hay bigint: tràn ở 2,1 tỷ

`int` là số nguyên 32 bit, lớn nhất 2.147.483.647. Với đơn vị đồng, con số đó chỉ là 2,1 tỷ đồng, tức doanh thu chưa tới một ngày của một thành phố lớn. Lab thử ghi 3 tỷ:

```sql
INSERT INTO type_lab (id, as_int) VALUES (4, 3000000000);     -- lỗi
INSERT INTO type_lab (id, as_bigint) VALUES (4, 3000000000);  -- được
```

```text
PASS lỗi mong đợi (dòng 33): Unable to make int from '3000000000'

 id | as_bigint
----+------------
  4 | 3000000000
```

Ở đây Cassandra báo lỗi vì giá trị là hằng số trong câu lệnh. Khi ứng dụng gửi giá trị qua prepared statement, phần kiểm tra nằm ở driver và ngôn ngữ lập trình; số có thể đã tràn ngay trong biến `int` của ứng dụng. Quy tắc của lab: **mọi cột tiền và mọi cột có thể là tổng đều dùng `bigint`**; `int` chỉ dùng cho mã nhỏ như `city_id` hay số thứ tự `event_seq`.

## 4. Thời gian: timestamp, date, time, duration

### timestamp là mili giây UTC, không giữ độ lệch

`timestamp` lưu **số mili giây kể từ 1970-01-01 00:00:00 UTC** (số nguyên 64 bit). Độ lệch `+0700` trong chuỗi chỉ dùng **lúc chuyển chuỗi thành số**, rồi bị bỏ đi. Hai cách viết khác nhau của cùng một thời điểm cho ra cùng một giá trị:

```sql
INSERT INTO type_lab (id, at_ts) VALUES (5, '2026-11-02 07:42:00+0700');
INSERT INTO type_lab (id, at_ts) VALUES (6, '2026-11-02 00:42:00+0000');
INSERT INTO type_lab (id, at_ts) VALUES (7, '2026-11-02 07:42:00.123+0700');
SELECT id, at_ts, toUnixTimestamp(at_ts) AS mili_giay FROM type_lab WHERE id IN (5, 6, 7);
```

```text
 id | at_ts                           | mili_giay
----+---------------------------------+---------------
  5 | 2026-11-02 07:42:00.000000+0700 | 1793580120000
  6 | 2026-11-02 07:42:00.000000+0700 | 1793580120000
  7 | 2026-11-02 07:42:00.123000+0700 | 1793580120123
```

cqlsh hiển thị theo múi giờ của container (compose của lab đặt `TZ: "Asia/Ho_Chi_Minh"`), nên thấy `+0700`. DBStudio thì hiển thị cùng giá trị đó dạng ISO **UTC**:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-004/01-timestamp-trong-luoi.webp" alt="DBStudio chạy SELECT trip_id, requested_at, toUnixTimestamp(requested_at) AS mili_giay_utc, fare_vnd FROM trips cho chuyến 1, 4 và 8. Lưới kết quả có ba dòng: chuyến 1 requested_at 2026-11-02T00:42:00.000Z, mili_giay_utc 1793580120000, fare 86000; chuyến 4 2026-11-02T05:10:00.000Z, 1793596200000, 64000; chuyến 8 2026-11-03T00:55:00.000Z, 1793667300000, 145000" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 1 · DBStudio hiển thị timestamp theo UTC.</b> Chuyến 1 đặt lúc 07:42 giờ Việt Nam hiện thành <code>2026-11-02T00:42:00.000Z</code>, và cột <code>mili_giay_utc</code> là đúng con số <code>1793580120000</code> trong OUTPUT của lab. Chữ <code>Z</code> nghĩa là UTC: đây là cùng một thời điểm, không phải dữ liệu bị lệch 7 giờ.</figcaption>
</figure>

Hai công cụ, hai cách hiển thị, **một giá trị lưu trữ**. Khi khách báo "giờ sai", việc đầu tiên là so `toUnixTimestamp(...)` chứ không so chuỗi hiển thị.

### Chuỗi không có độ lệch: múi giờ của node điều phối quyết định

```sql
INSERT INTO type_lab (id, at_ts) VALUES (8, '2026-11-02 07:42:00');
SELECT id, at_ts FROM type_lab WHERE id = 8;
```

```text
 id | at_ts
----+---------------------------------
  8 | 2026-11-02 07:42:00.000000+0700
```

Chuỗi `'2026-11-02 07:42:00'` không ghi độ lệch, nên Cassandra hiểu nó theo **múi giờ JVM của node điều phối** (*coordinator*: node nhận câu lệnh từ client và điều phối việc đọc ghi). Lab chạy với múi giờ Việt Nam nên kết quả tình cờ đúng. Một cụm production thường chạy UTC; khi đó cùng câu lệnh sẽ ghi 07:42 UTC, tức 14:42 giờ Việt Nam, và không có lỗi nào. Tệ hơn, nếu các node đặt múi giờ khác nhau, kết quả phụ thuộc node nào nhận câu lệnh.

Quy tắc: **chuỗi thời gian luôn ghi độ lệch** (`+0700` hoặc `+0000`). Trong ứng dụng, truyền giá trị thời gian qua tham số của prepared statement (kiểu `Instant`, `Date`… của ngôn ngữ) thay vì ghép chuỗi.

### Cắt micro giây mà không báo lỗi

```sql
INSERT INTO type_lab (id, at_ts) VALUES (9, '2026-11-02 07:42:00.123456+0700');
```

```text
 id | at_ts                           | mili_giay
----+---------------------------------+---------------
  9 | 2026-11-02 07:42:00.123000+0700 | 1793580120123
```

Phần `456` micro giây biến mất. Với dòng thời gian sự kiện chuyến, hai sự kiện cách nhau vài trăm micro giây sẽ có **cùng** `event_at`. Nếu `event_at` là cột clustering cuối cùng, hai sự kiện trùng khoá chính và sự kiện sau ghi đè sự kiện trước (CS-005 giải thích vì sao không có lỗi trùng khoá). Đó là lý do bảng `trip_events_by_trip` có thêm cột `event_seq` để phá hoà: khoá là `((trip_id), event_at, event_seq)`.

### date, time và ngày nghiệp vụ

```sql
INSERT INTO type_lab (id, on_date, at_time) VALUES (10, '2026-11-02', '07:42:00.000');
SELECT id, on_date, at_time, toTimestamp(on_date) AS nua_dem_utc FROM type_lab WHERE id = 10;
```

```text
 id | on_date    | at_time            | nua_dem_utc
----+------------+--------------------+---------------------------------
 10 | 2026-11-02 | 07:42:00.000000000 | 2026-11-02 07:00:00.000000+0700
```

- `date` là một **ngày lịch**, không có giờ và không có múi giờ.
- `time` là giờ trong ngày, độ phân giải nano giây, cũng không có múi giờ.
- `toTimestamp(on_date)` cho ra **nửa đêm UTC** của ngày đó, tức 07:00 sáng giờ Việt Nam.

Dòng cuối là cái bẫy: "ngày 02/11 theo giờ Việt Nam" bắt đầu lúc 2026-11-01 17:00 UTC, không phải nửa đêm UTC. Vì vậy cột `day` của bảng `trips_by_driver_day` do **ứng dụng tính theo giờ Việt Nam** rồi ghi vào, Cassandra không tự suy ra được. Chuyến đặt lúc 00:30 sáng 03/11 giờ Việt Nam vẫn là ngày 02/11 theo UTC; tính sai múi giờ là xếp chuyến vào nhầm ngày thu nhập của tài xế.

`duration` (khoảng thời gian gồm tháng, ngày, nano giây, ví dụ `1h30m`) cũng có trong CQL nhưng mô hình P01 không dùng; cột `duration` không được nằm trong khoá chính vì Cassandra không sắp xếp được giá trị này.

## 5. Chuỗi: text là UTF-8, ascii từ chối tiếng Việt

`text` (đồng nghĩa với `varchar`) lưu chuỗi UTF-8. `ascii` chỉ nhận ký tự US-ASCII:

```sql
INSERT INTO type_lab (id, label) VALUES (11, 'Phở Thìn Lò Đúc');
SELECT id, label, textAsBlob(label) AS byte_utf8 FROM type_lab WHERE id = 11;
INSERT INTO type_lab (id, code) VALUES (12, 'Phở');     -- cột ascii
```

```text
 id | label           | byte_utf8
----+-----------------+----------------------------------------------
 11 | Phở Thìn Lò Đúc | 0x5068e1bb9f205468c3ac6e204cc3b220c490c3ba63

PASS lỗi mong đợi (dòng 59): Invalid ASCII character in string literal: java.nio.charset.UnmappableCharacterException: Input length = 1
```

Tên quán 15 ký tự chiếm **21 byte**: `ở` là 3 byte (`e1 bb 9f`), `ì`, `ò`, `Đ`, `ú` mỗi chữ 2 byte. Hai hệ quả:

- Giới hạn kích thước (giá trị, partition) tính bằng byte, không phải số ký tự.
- Cột `text` trong khoá clustering được sắp theo **byte**, không theo bảng chữ cái tiếng Việt. Cassandra không có collation; đừng dùng `ORDER BY` trên tên quán để hiển thị danh sách cho người đọc.

Tới Nơi dùng `text` cho mọi chuỗi, kể cả mã như `service` hay `status`. Lợi ích của `ascii` (tiết kiệm vài byte) không đáng so với rủi ro một giá trị có dấu bị từ chối lúc chạy.

`blob` là chuỗi byte tuỳ ý, viết dạng `0x…`, đổi qua lại bằng các hàm `textAsBlob`, `blobAsText`… Tới Nơi không lưu blob trong mô hình lịch sử chuyến; ảnh hay file nằm ở object storage, Cassandra chỉ giữ đường dẫn.

## 6. Định danh: uuid và timeuuid

`uuid` là định danh 128 bit, thường là UUID phiên bản 4 (ngẫu nhiên) do ứng dụng sinh, hoặc dùng hàm `uuid()` của CQL. Fixture của lab dùng UUID cố định dễ đọc (`07000000-0000-4000-8000-000000000001` là chuyến 1, `0c…01` là khách 1, `0d…01` là tài xế 1) để kết quả tái lập được; ứng dụng thật sinh UUID ngẫu nhiên.

`timeuuid` là UUID phiên bản 1: bên trong chứa **thời điểm** (độ phân giải 100 nano giây) cùng vài bit chống trùng. Cassandra sắp xếp cột `timeuuid` theo thời điểm đó trước. Lab đọc thời điểm ra bằng `toTimestamp`:

```sql
SELECT minTimeuuid('2026-11-02 07:42:00+0700') AS timeuuid_nho_nhat,
       toTimestamp(minTimeuuid('2026-11-02 07:42:00+0700')) AS thoi_diem_ben_trong
FROM system.local;
```

```text
 timeuuid_nho_nhat                    | thoi_diem_ben_trong
--------------------------------------+---------------------------------
 36cafc00-d656-11f1-8080-808080808080 | 2026-11-02 07:42:00.000000+0700
```

`minTimeuuid(t)` và `maxTimeuuid(t)` cho ra timeuuid nhỏ nhất và lớn nhất có thể ứng với thời điểm `t`. Chúng chỉ dùng làm **cận của truy vấn dải** (`WHERE event_id >= minTimeuuid(…) AND event_id < maxTimeuuid(…)`), **không** dùng để sinh khoá khi ghi: mọi lời gọi với cùng `t` trả về cùng một giá trị. Muốn sinh timeuuid mới khi ghi, dùng `now()` hoặc hàm của driver.

**Vì sao timeuuid hợp làm cột clustering cho sự kiện.** Hai sự kiện xảy ra trong cùng một mili giây có cùng giá trị `timestamp`, nhưng hai timeuuid sinh ra lúc đó vẫn khác nhau (khác phần 100 nano giây hoặc phần bit chống trùng). Một cột timeuuid vừa cho thứ tự theo thời gian vừa phá hoà, thay cho cặp `event_at, event_seq`. Lab P01 chọn cặp `timestamp + int` vì dễ đọc trong bài và vì ứng dụng đã có số thứ tự sự kiện; cả hai cách đều tránh được việc sự kiện sau ghi đè sự kiện trước.

Cassandra 5.0 còn có tên hàm dạng snake_case tương đương (`min_timeuuid`, `to_timestamp`); lab dùng tên cũ, cả hai cách viết đều chạy.

## 7. Không đổi được kiểu của cột

```sql
ALTER TABLE type_lab ALTER as_int TYPE bigint;
```

```text
PASS lỗi mong đợi (dòng 68): Altering column types is no longer supported
```

Cassandra 5.0 không còn cho đổi kiểu cột, kể cả `int` sang `bigint` trông có vẻ vô hại. Dữ liệu đã nằm trong SSTable (file dữ liệu trên đĩa) được mã hoá theo kiểu cũ, và Cassandra không viết lại các file đó khi schema thay đổi. Cách xử lý thực tế là thêm cột mới với tên khác rồi ghi lại dữ liệu, hoặc tạo bảng mới và chuyển dữ liệu sang; bài CS-048 (Phần 5) làm việc này cho production. Bài học cho P01: **chọn kiểu đúng ngay lần đầu**, nhất là cột tiền và cột thời gian.

## 8. Từ điển dữ liệu của bảng trips

Bảng `trips` (Q1, tra chuyến theo mã) dùng đúng các quyết định ở trên. Các bảng phi chuẩn hoá ở CS-007, CS-008 dùng lại cùng kiểu cho cùng cột.

| Cột | Kiểu | Ý nghĩa | Ví dụ (chuyến 1) |
|---|---|---|---|
| `trip_id` | `uuid` | Mã chuyến, **partition key** | `07000000-0000-4000-8000-000000000001` |
| `service` | `text` | `'ride'`, `'food'` hoặc `'parcel'`; ứng dụng kiểm giá trị | `ride` |
| `city_id` | `int` | Mã thành phố (1 = TP.HCM, 2 = Hà Nội) | `1` |
| `customer_id` | `uuid` | Khách đặt chuyến | `0c000000-…-000000000001` |
| `driver_id` | `uuid` | Tài xế nhận chuyến; `null` khi chưa có | `0d000000-…-000000000001` |
| `status` | `text` | `requested` → … → `completed` / `cancelled` | `completed` |
| `requested_at` | `timestamp` | Thời điểm đặt, mili giây UTC; ghi kèm độ lệch | `2026-11-02 07:42:00+0700` |
| `pickup`, `dropoff` | `text` | Điểm đón, điểm trả (UTF-8) | `Chung cư Sky Garden, Q7` |
| `fare_vnd` | `bigint` | Cước, **số nguyên đồng** | `86000` |

Hai điều bảng không tự giữ được, nên ứng dụng phải giữ: giá trị hợp lệ của `service`/`status` (không có `CHECK`), và tính bắt buộc của các cột ngoài khoá (cột thường nào cũng có thể vắng giá trị).

Collection (`set`, `list`, `map`), user-defined type, `tuple` và kiểu `vector` của Cassandra 5.0 để dành cho Phần 2 (CS-011, CS-012) và Phần 6 (CS-053).

## 9. Lỗi thường gặp

- **Lưu tiền bằng `double` "vì cần số lẻ".** Tiền đồng không có số lẻ; ngoại tệ thì dùng `decimal` hoặc số nguyên theo đơn vị nhỏ nhất (xu).
- **Thấy DBStudio hiện `00:42Z` và "sửa" dữ liệu cộng thêm 7 giờ.** Giá trị không sai; chỉ cách hiển thị khác. Sửa như vậy làm sai dữ liệu thật.
- **Ghép chuỗi thời gian không có độ lệch** trong code. Chạy đúng trên máy dev đặt giờ Việt Nam, sai trên cụm production chạy UTC.
- **Tin rằng timestamp giữ micro giây.** Cassandra cắt về mili giây mà không báo lỗi; cần thứ tự chặt hơn thì thêm cột phá hoà hoặc dùng `timeuuid`.
- **Dùng `minTimeuuid()` để sinh khoá khi ghi.** Mọi lần ghi cùng thời điểm có cùng khoá và ghi đè nhau.
- **Nạp CSV bằng `cqlsh COPY FROM` với giờ `+0700`.** Lab ghi nhận cqlsh 5.0.9 đọc sai độ lệch múi giờ trong CSV (mục "Bẫy của COPY FROM" trong `OUTPUT.txt`); lab ghi CSV bằng giờ UTC để tránh. CS-009 kể chi tiết.

## 10. So với PostgreSQL

| Nhu cầu | PostgreSQL ([PG-004](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid)) | Cassandra 5.0 |
|---|---|---|
| Thời điểm có múi giờ | `timestamptz`, độ phân giải micro giây | `timestamp`, chỉ mili giây; cả hai đều không lưu độ lệch gốc |
| Đổi kiểu cột | `ALTER COLUMN … TYPE … USING …` viết lại bảng | Không hỗ trợ |
| Giữ luật giá trị | `CHECK`, `NOT NULL`, enum, domain | Không có; ứng dụng tự kiểm |
| Sắp xếp chuỗi | Theo collation | Theo byte UTF-8 |

## 11. Bài tập

**Bài 1.** Chọn kiểu cho các cột: (a) phụ phí trời mưa tính bằng đồng; (b) tỷ giá USD/VND dùng cho chuyến của khách nước ngoài; (c) tổng doanh thu tháng của cả nước; (d) số sự kiện trong một chuyến.

<details><summary><b>Lời giải</b></summary><p>(a) <code>bigint</code> số nguyên đồng. (b) <code>decimal</code>, vì tỷ giá có phần lẻ và cần chính xác. (c) <code>bigint</code>: vượt xa giới hạn 2,1 tỷ của <code>int</code>. (d) <code>int</code> là đủ: một chuyến chỉ có vài chục sự kiện.</p></details>

**Bài 2.** Một dịch vụ ghi `INSERT … VALUES (…, '2026-11-03 00:30:00', …)` vào cụm production chạy UTC. Chuyến thật được đặt lúc 00:30 sáng 03/11 giờ Việt Nam. Cassandra lưu thời điểm nào, theo giờ Việt Nam? Sửa câu lệnh thế nào?

<details><summary><b>Lời giải</b></summary><p>Chuỗi không có độ lệch được hiểu theo múi giờ của node điều phối (UTC), nên lưu 2026-11-03 00:30 UTC, tức 07:30 sáng 03/11 giờ Việt Nam: lệch 7 giờ. Sửa bằng cách ghi rõ <code>'2026-11-03 00:30:00+0700'</code>, hoặc tốt hơn là truyền giá trị thời gian qua tham số của prepared statement.</p></details>

**Bài 3.** Chuyến đặt lúc `2026-11-03 00:30:00+0700`. Ứng dụng tính cột `day` của `trips_by_driver_day` bằng cách lấy phần ngày của thời điểm theo UTC. Chuyến bị xếp vào ngày nào? Có sao không?

<details><summary><b>Lời giải</b></summary><p>Theo UTC, thời điểm đó là 2026-11-02 17:30, nên <code>day = 2026-11-02</code>. Sai: theo nghiệp vụ, chuyến thuộc ngày 03/11 giờ Việt Nam. Tài xế xem thu nhập ngày 03/11 sẽ thiếu chuyến này. Ứng dụng phải đổi sang giờ Việt Nam trước khi lấy ngày.</p></details>

**Bài 4.** Vì sao khoá `((trip_id), event_at)` cho dòng thời gian sự kiện là nguy hiểm, còn `((trip_id), event_at, event_seq)` hoặc `((trip_id), event_id)` với `event_id timeuuid` thì không?

<details><summary><b>Lời giải</b></summary><p><code>event_at</code> chỉ giữ tới mili giây; hai sự kiện cùng mili giây có cùng khoá chính và câu ghi sau ghi đè câu trước mà không báo lỗi. <code>event_seq</code> phá hoà bằng số thứ tự của ứng dụng; <code>timeuuid</code> phá hoà vì hai giá trị sinh ra cùng mili giây vẫn khác nhau, và vẫn sắp theo thời gian.</p></details>

## 12. Đọc tiếp

- [CQL data types](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/types.html): danh sách kiểu và định dạng hằng số.
- [CQL functions](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/functions.html): `toTimestamp`, `minTimeuuid`, các hàm `…AsBlob`.
- [ALTER TABLE](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/ddl.html): những gì thay đổi được sau khi tạo bảng.
- [PG-004](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid): cùng câu hỏi chọn kiểu, trả lời bằng PostgreSQL.

Bài tiếp theo: [CS-005 · INSERT, UPDATE, DELETE: mọi câu ghi là upsert](/articles/cassandra-insert-update-delete-upsert).
