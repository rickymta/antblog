---
slug: cassandra-select-where-clustering-order
title: "SELECT theo khoá trong Cassandra: WHERE, thứ tự clustering và phân trang lịch sử chuyến"
summary: "Màn hình lịch sử chuyến của Tới Nơi đọc một partition đã sắp sẵn; truy vấn lọc theo trạng thái thì bị Cassandra từ chối. Bài này nêu quy tắc WHERE trên partition key và clustering key, ORDER BY, LIMIT, PER PARTITION LIMIT, ALLOW FILTERING, IN, token() và paging, kèm lỗi thật từ lab Cassandra 5.0.9."
category: cassandra
level: intermediate
tags: [cassandra, cs-006, cassandra-5, cql, toi-noi, select, allow-filtering, clustering-order, paging]
language: vi
---

Đội ứng dụng Tới Nơi viết màn hình "Lịch sử chuyến" và một màn hình nội bộ "Các chuyến bị huỷ hôm nay". Câu thứ nhất chạy ngay, trả về chuyến mới nhất trước mà không cần `ORDER BY`. Câu thứ hai bị Cassandra từ chối với một thông báo dài nhắc tới `ALLOW FILTERING`. Thêm hai chữ đó vào thì câu chạy được trên lab, và đó chính là cái bẫy: trên cụm thật, câu này đọc mọi partition trên mọi node.

Ở PostgreSQL, câu `SELECT` nào đúng cú pháp cũng chạy được; index chỉ quyết định nhanh hay chậm. Ở Cassandra, **khoá chính quyết định câu `SELECT` nào được phép**. Bài này đưa ra các quy tắc đó và lý do đằng sau mỗi quy tắc.

<div class="tkh-callout">
<span class="tkh-callout-title">CS-006 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/cassandra-keyspace-bang-khoa-chinh">CS-003 · Keyspace, bảng và khoá chính</a>, <a href="/articles/cassandra-kieu-du-lieu-cql">CS-004 · Kiểu dữ liệu CQL</a>. Lab: <code>examples/toi-noi-cassandra/p01/05-select.cql</code>, kết quả ở <code>p01/OUTPUT.txt</code>. Sau bài này, bạn viết được truy vấn hợp lệ trên partition key và clustering key, hiểu vì sao một truy vấn bị từ chối, biết khi nào <code>ALLOW FILTERING</code> chấp nhận được, dùng <code>PER PARTITION LIMIT</code> và hiểu paging.</p>
</div>

## 1. Bảng dùng trong bài

Các truy vấn chạy trên bảng của mô hình P01 (CS-003, CS-007):

| Bảng | Khoá chính | Truy vấn phục vụ |
|---|---|---|
| `trips_by_customer` | `((customer_id), requested_at, trip_id)`, `requested_at DESC` | Q2: lịch sử chuyến của một khách, mới nhất trước |
| `trips_by_driver_day` | `((driver_id, day), requested_at, trip_id)`, `requested_at DESC` | Q3: chuyến của tài xế trong một ngày |
| `trips_by_city_hour` | `((city_id, hour_bucket), requested_at, trip_id)`, `requested_at ASC` | Q4: chuyến mới tạo theo thành phố và giờ |
| `trip_events_by_trip` | `((trip_id), event_at, event_seq)` | Q5: dòng thời gian sự kiện của chuyến |

Fixture có 12 chuyến của 3 khách trong hai ngày 02 và 03/11/2026. Khách 1 là `0c000000-0000-4000-8000-000000000001`, tài xế 1 là `0d…01`.

## 2. Quy tắc WHERE

Một câu `SELECT` thường (không có `ALLOW FILTERING`) phải theo ba quy tắc:

1. **Partition key đủ mọi cột**, mỗi cột ràng buộc bằng `=` hoặc `IN`. Cassandra cần giá trị partition key để băm ra token, từ đó biết node nào giữ dữ liệu.
2. **Cột clustering ràng buộc theo đúng thứ tự khai báo**, không được bỏ qua cột đứng trước.
3. **Điều kiện dải** (`>`, `>=`, `<`, `<=`) chỉ được đặt trên **cột clustering cuối cùng** có ràng buộc; các cột clustering đứng trước nó phải là `=` (hoặc `IN`).

Ba quy tắc này bảo đảm mỗi câu đọc là **một đoạn liền nhau** trong một (hoặc vài) partition, vì trong partition các dòng đã được sắp theo cột clustering.

### Đọc một partition: thứ tự có sẵn

```sql
SELECT requested_at, service, status, fare_vnd, trip_id
FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001;
```

```text
 requested_at                    | service | status    | fare_vnd | trip_id
---------------------------------+---------+-----------+----------+--------------------------------------
 2026-11-03 17:45:00.000000+0700 |    ride | completed |    89000 | 07000000-0000-4000-8000-000000000011
 2026-11-03 11:20:00.000000+0700 |    food | completed |    52000 | 07000000-0000-4000-8000-000000000009
 2026-11-02 18:02:00.000000+0700 |    ride | completed |    91000 | 07000000-0000-4000-8000-000000000006
 2026-11-02 11:48:00.000000+0700 |    food | completed |    35000 | 07000000-0000-4000-8000-000000000003
 2026-11-02 07:42:00.000000+0700 |    ride | completed |    86000 | 07000000-0000-4000-8000-000000000001
```

Mới nhất trước, không cần `ORDER BY`: thứ tự `requested_at DESC` nằm trong cách bảng lưu dữ liệu. Thêm `LIMIT 2` là có ngay hai chuyến gần nhất, và Cassandra dừng đọc sau dòng thứ hai:

```text
 requested_at                    | service | fare_vnd
---------------------------------+---------+----------
 2026-11-03 17:45:00.000000+0700 |    ride |    89000
 2026-11-03 11:20:00.000000+0700 |    food |    52000
```

### Dải trên cột clustering

Chuyến của khách 1 trong ngày 02/11 giờ Việt Nam:

```sql
SELECT requested_at, service, fare_vnd
FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001
  AND requested_at >= '2026-11-02 00:00:00+0700'
  AND requested_at <  '2026-11-03 00:00:00+0700';
```

```text
 requested_at                    | service | fare_vnd
---------------------------------+---------+----------
 2026-11-02 18:02:00.000000+0700 |    ride |    91000
 2026-11-02 11:48:00.000000+0700 |    food |    35000
 2026-11-02 07:42:00.000000+0700 |    ride |    86000
```

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-006/02-dai-clustering.webp" alt="DBStudio chạy SELECT requested_at, service, fare_vnd FROM trips_by_customer WHERE customer_id = 0c000000-0000-4000-8000-000000000001 AND requested_at >= '2026-11-02 00:00:00+0700' AND requested_at < '2026-11-03 00:00:00+0700'. Lưới kết quả 3 dòng: 2026-11-02T11:02:00.000Z ride 91000; 2026-11-02T04:48:00.000Z food 35000; 2026-11-02T00:42:00.000Z ride 86000. Thanh dưới lưới ghi 3 dòng, 4 ms, Chỉ đọc: Thiếu cột khoá customer_id, trip_id trong SELECT" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 1 · Dải requested_at trong một partition.</b> Ba chuyến của ngày 02/11, vẫn mới nhất trước; lưới hiển thị giờ UTC (<code>11:02Z</code> là 18:02 giờ Việt Nam, xem CS-004). Thanh trạng thái ghi "Chỉ đọc: Thiếu cột khoá customer_id, trip_id trong SELECT": DBStudio chỉ cho sửa trên lưới khi kết quả có đủ partition key và clustering key để xác định từng dòng.</figcaption>
</figure>

Hai cận dải ghi rõ `+0700`; bỏ độ lệch thì chuỗi được hiểu theo múi giờ của node điều phối (CS-004).

### Các lỗi khi vi phạm quy tắc

**Thiếu một phần partition key.** `trips_by_driver_day` có partition key hai cột `(driver_id, day)`:

```sql
SELECT requested_at, status FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000001;
```

```text
PASS lỗi mong đợi (dòng 55): Cannot execute this query as it might involve data filtering and thus may have unpredictable performance. If you want to execute this query despite the performance unpredictability, use ALLOW FILTERING
```

Chỉ có `driver_id` thì Cassandra không tính được token; dữ liệu của tài xế 1 nằm rải ở nhiều partition (mỗi ngày một partition) trên nhiều node. Câu đúng phải có cả `day`.

**Bỏ qua cột clustering đứng trước.**

```sql
SELECT * FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001
  AND trip_id = 07000000-0000-4000-8000-000000000001;
```

```text
PASS lỗi mong đợi (dòng 66): PRIMARY KEY column "trip_id" cannot be restricted as preceding column "requested_at" is not restricted
```

Trong partition, dòng sắp theo `requested_at` trước rồi mới tới `trip_id`; không biết `requested_at` thì các dòng có `trip_id` cần tìm không nằm liền nhau. Tra chuyến theo mã thì dùng bảng `trips` (Q1).

**Dải ở cột clustering không phải cột cuối.** Câu chỉ-đọc chạy thêm trên lab:

```sql
SELECT requested_at FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001
  AND requested_at > '2026-11-02 08:00:00+0700' AND requested_at < '2026-11-03 12:00:00+0700'
  AND trip_id > 07000000-0000-4000-8000-000000000000;
```

```text
InvalidRequest: … message="Clustering column "trip_id" cannot be restricted (preceding column "requested_at" is restricted by a non-EQ relation)"
```

**Dải trên partition key.** "Các chuyến của TP.HCM từ 11:00 đến 13:00" trên `trips_by_city_hour`:

```sql
SELECT * FROM trips_by_city_hour
WHERE city_id = 1 AND hour_bucket >= '2026-11-02 11:00:00+0700' AND hour_bucket < '2026-11-02 13:00:00+0700';
```

```text
PASS lỗi mong đợi (dòng 71): Cannot execute this query as it might involve data filtering and thus may have unpredictable performance. If you want to execute this query despite the performance unpredictability, use ALLOW FILTERING
```

Partition được rải trên ring theo **token** (giá trị băm của partition key), không theo giá trị. Giờ 11:00 và 12:00 là hai partition có token không liên quan gì nhau; "từ 11 đến 13 giờ" không phải một đoạn liền nhau ở đâu cả. Cách đúng là liệt kê từng partition bằng `IN` (mục 4).

## 3. ORDER BY chỉ đảo chiều clustering

`ORDER BY` không sắp xếp tuỳ ý. Nó chỉ chọn đọc partition theo chiều khai báo hoặc **đảo ngược toàn bộ**:

```sql
SELECT requested_at, service FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001
ORDER BY requested_at ASC, trip_id DESC;
```

```text
 requested_at                    | service
---------------------------------+---------
 2026-11-02 07:42:00.000000+0700 |    ride
 2026-11-02 11:48:00.000000+0700 |    food
 2026-11-02 18:02:00.000000+0700 |    ride
 2026-11-03 11:20:00.000000+0700 |    food
 2026-11-03 17:45:00.000000+0700 |    ride
```

Bảng khai báo `requested_at DESC, trip_id ASC`; câu trên đảo cả hai thành `ASC, DESC`. Kết hợp `requested_at ASC, trip_id ASC` (đảo một cột, giữ một cột) không hợp lệ. Sắp theo cột thường thì bị từ chối:

```sql
SELECT requested_at, service FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001
ORDER BY fare_vnd DESC;
```

```text
PASS lỗi mong đợi (dòng 31): Order by is currently only supported on the clustered columns of the PRIMARY KEY, got fare_vnd
```

"Chuyến đắt nhất của khách" cần một bảng khác có `fare_vnd` là cột clustering, hoặc ứng dụng tự sắp trên một partition nhỏ. Đọc ngược chiều khai báo chạy được nhưng chậm hơn đọc xuôi một chút; chọn `CLUSTERING ORDER BY` theo chiều mà truy vấn chính cần.

## 4. Nhiều partition: IN, DISTINCT, token() và PER PARTITION LIMIT

### IN trên partition key

Thu nhập của tài xế 1 trong hai ngày là **hai partition**:

```sql
SELECT day, requested_at, service, fare_vnd
FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000001 AND day IN ('2026-11-02', '2026-11-03');
```

```text
 day        | requested_at                    | service | fare_vnd
------------+---------------------------------+---------+----------
 2026-11-02 | 2026-11-02 11:48:00.000000+0700 |    food |    35000
 2026-11-02 | 2026-11-02 07:42:00.000000+0700 |    ride |    86000
 2026-11-03 | 2026-11-03 11:20:00.000000+0700 |    food |    52000
 2026-11-03 | 2026-11-03 07:55:00.000000+0700 |    ride |   145000
```

Node điều phối phải hỏi các node giữ từng partition rồi gộp kết quả; ở cụm nhiều node, hai partition này có thể nằm ở hai node khác nhau. `IN` với vài giá trị là bình thường. `IN` với hàng trăm giá trị dồn tải lên một node điều phối và chờ partition chậm nhất; khi đó gửi nhiều truy vấn song song từ ứng dụng thường tốt hơn (CS-037).

`IN` trên partition key kết hợp `ORDER BY` bị từ chối khi paging đang bật (mặc định). Câu chỉ-đọc chạy thêm trên lab:

```text
InvalidRequest: … message="Cannot page queries with both ORDER BY and a IN restriction on the partition key; you must either remove the ORDER BY or the IN and sort client side, or disable paging for this query"
```

### Thứ tự giữa các partition là thứ tự token

```sql
SELECT DISTINCT customer_id, token(customer_id) AS token_cua_partition FROM trips_by_customer;
```

```text
 customer_id                          | token_cua_partition
--------------------------------------+----------------------
 0c000000-0000-4000-8000-000000000001 | -2983185843790857942
 0c000000-0000-4000-8000-000000000003 |   144159704313836052
 0c000000-0000-4000-8000-000000000002 |  7165355148280075549
```

Khách 3 đứng trước khách 2 vì token nhỏ hơn. `DISTINCT` chỉ nhận cột partition key (và cột static); thêm `status` vào thì bị từ chối với `SELECT DISTINCT queries must only request partition key columns and/or static columns (not status)`. Câu này không có partition key nên vẫn duyệt cả bảng, chỉ là mỗi partition trả một dòng.

`token()` cũng dùng được trong `WHERE`, cho phép quét bảng theo **dải token**. Câu chỉ-đọc `… WHERE token(customer_id) > 0 PER PARTITION LIMIT 1` trên lab trả về khách 3 và khách 2. Chia toàn bộ ring thành nhiều dải token rồi giao cho nhiều worker là cách đúng để duyệt bảng lớn, chi tiết ở CS-036.

### PER PARTITION LIMIT

"Chuyến gần nhất của mỗi khách" cho ba khách:

```sql
SELECT customer_id, requested_at, service, fare_vnd
FROM trips_by_customer
WHERE customer_id IN (0c000000-0000-4000-8000-000000000001, 0c000000-0000-4000-8000-000000000002, 0c000000-0000-4000-8000-000000000003)
PER PARTITION LIMIT 1;
```

```text
 customer_id                          | requested_at                    | service | fare_vnd
--------------------------------------+---------------------------------+---------+----------
 0c000000-0000-4000-8000-000000000001 | 2026-11-03 17:45:00.000000+0700 |    ride |    89000
 0c000000-0000-4000-8000-000000000002 | 2026-11-03 18:20:00.000000+0700 |    food |   118000
 0c000000-0000-4000-8000-000000000003 | 2026-11-03 11:35:00.000000+0700 |    ride |    57000
```

`LIMIT` giới hạn tổng số dòng; `PER PARTITION LIMIT` giới hạn số dòng **trong mỗi partition**. Vì mỗi partition đã sắp mới nhất trước, dòng đầu của mỗi partition chính là chuyến gần nhất. Hai mệnh đề dùng chung được.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-006/03-per-partition-limit.webp" alt="DBStudio chạy SELECT customer_id, requested_at, service, fare_vnd FROM trips_by_customer WHERE customer_id IN (ba khách 0c…01, 0c…02, 0c…03) PER PARTITION LIMIT 1. Lưới kết quả 3 dòng: khách 1, 2026-11-03T10:45:00.000Z, ride, 95000; khách 2, 2026-11-03T11:20:00.000Z, food, 118000; khách 3, 2026-11-03T04:35:00.000Z, ride, 57000. Thanh dưới ghi Chỉ đọc: Thiếu cột khoá trip_id trong SELECT" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 2 · Mỗi khách một chuyến gần nhất.</b> Ba dòng, mỗi partition một dòng. Ảnh chụp sau khi chạy hết lab P01, nên chuyến 11 của khách 1 mang cước 95000 đã sửa ở CS-008; OUTPUT của bước 05 (chạy trước bước sửa) ghi 89000.</figcaption>
</figure>

## 5. ALLOW FILTERING: khi nào chấp nhận được

Lọc theo cột thường bị từ chối, kể cả khi đã có partition key:

```sql
SELECT requested_at, status FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000001 AND status = 'completed';
```

```text
PASS lỗi mong đợi (dòng 36): Cannot execute this query as it might involve data filtering and thus may have unpredictable performance. If you want to execute this query despite the performance unpredictability, use ALLOW FILTERING
```

**ALLOW FILTERING** cho phép Cassandra đọc các dòng ứng viên rồi bỏ những dòng không khớp. Chi phí là số dòng **phải đọc**, không phải số dòng trả về. Vì vậy có hai trường hợp rất khác nhau:

| Trường hợp | Ví dụ | Đọc bao nhiêu | Đánh giá |
|---|---|---|---|
| Có partition key, partition nhỏ và có giới hạn | Chuyến `completed` của khách 1 | Một partition (vài trăm dòng) | Chấp nhận được |
| Không có partition key | Mọi chuyến `cancelled` | **Mọi partition trên mọi node** | Không dùng ở đường chạy của ứng dụng |

Trường hợp thứ nhất trên lab trả 5 dòng (cả 5 chuyến của khách 1 đều `completed`). Trường hợp thứ hai cũng chạy được trên lab:

```sql
SELECT customer_id, requested_at, status FROM trips_by_customer
WHERE status = 'cancelled'
ALLOW FILTERING;
```

```text
 customer_id                          | requested_at                    | status
--------------------------------------+---------------------------------+-----------
 0c000000-0000-4000-8000-000000000002 | 2026-11-02 17:31:00.000000+0700 | cancelled
```

Một dòng kết quả, nhưng để có nó Cassandra đã đọc cả 12 dòng của bảng. Với khoảng 2 triệu chuyến mỗi ngày của Tới Nơi, câu này đọc toàn bộ lịch sử chuyến để trả về vài nghìn chuyến bị huỷ, và rất dễ timeout trước khi xong. Câu cùng loại trong DBStudio khi chưa thêm `ALLOW FILTERING`:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-006/01-loi-allow-filtering.webp" alt="DBStudio chạy SELECT requested_at, status FROM trips_by_customer WHERE status = 'cancelled'. Thanh lỗi màu đỏ phía trên và tab Thông điệp hiển thị: Cannot execute this query as it might involve data filtering and thus may have unpredictable performance. If you want to execute this query despite the performance unpredictability, use ALLOW FILTERING. Góc phải ghi Lỗi, 131 ms" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 3 · Cassandra từ chối truy vấn lọc.</b> Câu không có partition key và lọc theo cột thường <code>status</code>; Cassandra trả lỗi ngay, trước khi đọc dữ liệu. Thông báo gợi ý thêm <code>ALLOW FILTERING</code>, nhưng cách sửa đúng ở đây là một bảng mới có <code>status</code> trong partition key (CS-007), hoặc SAI index (CS-018).</figcaption>
</figure>

Quy tắc thực dụng: chỉ dùng `ALLOW FILTERING` khi câu đã có **đủ partition key** và bạn biết partition đó nhỏ. Còn lại, coi thông báo lỗi là tín hiệu thiết kế bảng chưa phục vụ truy vấn này.

## 6. COUNT(*) không có partition key là quét toàn bảng

Fixture đếm số dòng mỗi bảng sau khi nạp, và cqlsh in kèm cảnh báo:

```text
 bang  | so_dong
-------+---------
 trips |      12

Warnings :
Aggregation query used without partition key
```

`COUNT(*)`, `SUM`, `MAX`… không có partition key phải đọc mọi partition, giống `ALLOW FILTERING` không có partition key. Bảng 12 dòng thì không sao; bảng hàng tỷ dòng thì timeout. Đếm trong **một partition** là truy vấn hợp lệ và rẻ. Câu chỉ-đọc trên lab đếm chuyến của khách 1:

```sql
SELECT COUNT(*), SUM(fare_vnd) FROM trips_by_customer WHERE customer_id = 0c000000-0000-4000-8000-000000000001;
```

```text
 count | system.sum(fare_vnd)
-------+----------------------
     5 |               359000
```

Tổng `359000` đã gồm cước 95000 của chuyến 11 sau khi sửa ở CS-008. Số đếm cho cả hệ thống (tổng chuyến trong ngày) thì giữ bằng bảng tổng hợp riêng hoặc counter (CS-014), hoặc tính ở hệ phân tích.

## 7. Phân trang (paging)

Cassandra không trả một kết quả lớn trong một lần. Driver gửi truy vấn với **kích thước trang** (*page size*, ví dụ 5000 dòng ở Java driver 4.x); node trả một trang cùng một **paging state**, một chuỗi byte đánh dấu vị trí đã đọc tới. Muốn trang tiếp, driver gửi lại cùng câu truy vấn kèm paging state đó. Phần lớn driver làm việc này tự động khi bạn duyệt hết kết quả.

Hai điều cần nhớ khi làm màn hình "xem thêm":

- **CQL không có `OFFSET`.** Câu chỉ-đọc `… LIMIT 2 OFFSET 2` trên lab báo lỗi cú pháp (`mismatched input 'OFFSET' expecting EOF`). Để làm "trang 2", ứng dụng giữ paging state của trang trước, hoặc dùng giá trị clustering cuối cùng đã thấy làm cận: `WHERE customer_id = … AND requested_at < <giờ của dòng cuối trang trước>`. Cách thứ hai bền hơn vì không phụ thuộc chuỗi byte của driver.
- **Paging state chỉ dùng cho đúng câu truy vấn đã sinh ra nó**, trong thời gian ngắn; đừng lưu lâu dài hay cho người dùng sửa.

DBStudio 0.1.7 dùng cơ chế này: theo mã nguồn của driver Cassandra trong DBStudio, câu `SELECT` được đọc từng trang bằng paging state, khi chạm giới hạn số dòng hiển thị thì giữ lại paging state để lấy tiếp; lưới "Mở dữ liệu" lưu paging state theo vị trí, vì CQL không có `OFFSET` để nhảy trang. cqlsh có lệnh `PAGING` để bật/tắt và đặt kích thước trang khi dùng tương tác.

Paging không làm truy vấn rẻ hơn: duyệt hết một câu `ALLOW FILTERING` không có partition key vẫn quét cả bảng, chỉ là chia thành nhiều lần gọi.

## 8. Lỗi thường gặp

- **Thêm `ALLOW FILTERING` cho hết lỗi.** Chạy nhanh trên lab 12 dòng, timeout trên production. Đọc lại câu: đã có đủ partition key chưa?
- **Dùng `ORDER BY` để sắp theo cột thường.** Chỉ đảo được chiều clustering; muốn thứ tự khác thì cần bảng khác.
- **Lọc dải trên cột bucket** (`hour_bucket >= …`). Partition key chỉ nhận `=` hoặc `IN`; liệt kê từng bucket.
- **`COUNT(*)` toàn bảng để hiển thị số liệu.** Cảnh báo `Aggregation query used without partition key` là dấu hiệu quét toàn bảng.
- **Bỏ độ lệch múi giờ trong cận dải `requested_at`.** Kết quả phụ thuộc múi giờ của node điều phối.

## 9. So với PostgreSQL

| Tình huống | PostgreSQL ([PG-007](/articles/postgresql-select-filter-order-limit)) | Cassandra 5.0 |
|---|---|---|
| `WHERE` trên cột bất kỳ | Chạy được; index quyết định tốc độ | Bị từ chối nếu không theo khoá chính (trừ `ALLOW FILTERING` hoặc SAI index) |
| `ORDER BY` | Cột bất kỳ | Chỉ chiều clustering, trong partition |
| Phân trang | `LIMIT … OFFSET` hoặc keyset | Paging state hoặc keyset theo cột clustering; không có `OFFSET` |
| `COUNT(*)` toàn bảng | Chậm khi bảng lớn nhưng vẫn là một truy vấn bình thường | Quét mọi node, có cảnh báo, dễ timeout |

## 10. Bài tập

**Bài 1.** Câu nào hợp lệ trên `trips_by_driver_day` (khoá `((driver_id, day), requested_at, trip_id)`)? (a) `WHERE driver_id = X AND day = D AND requested_at > T`; (b) `WHERE driver_id = X AND day = D AND trip_id = Y`; (c) `WHERE driver_id = X AND day IN (D1, D2)`; (d) `WHERE driver_id = X AND day > D`.

<details><summary><b>Lời giải</b></summary><p>(a) hợp lệ: đủ partition key, dải trên cột clustering đầu. (b) không: bỏ qua <code>requested_at</code> đứng trước <code>trip_id</code>. (c) hợp lệ: <code>IN</code> trên partition key, hai partition. (d) không: dải trên một cột của partition key.</p></details>

**Bài 2.** Màn hình "3 chuyến gần nhất của mỗi khách trong danh sách 10 khách VIP". Viết câu CQL trên `trips_by_customer`.

<details><summary><b>Lời giải</b></summary><p><code>SELECT customer_id, requested_at, service, fare_vnd FROM trips_by_customer WHERE customer_id IN (…10 mã…) PER PARTITION LIMIT 3;</code> Mỗi partition đã sắp mới nhất trước nên ba dòng đầu là ba chuyến gần nhất. Với danh sách dài hơn nhiều, gửi các truy vấn một khách song song từ ứng dụng.</p></details>

**Bài 3.** Bộ phận vận hành muốn "mọi chuyến bị huỷ hôm nay ở TP.HCM". Vì sao `SELECT … FROM trips_by_customer WHERE status = 'cancelled' ALLOW FILTERING` không chấp nhận được, và bạn đề xuất gì với các bảng đang có?

<details><summary><b>Lời giải</b></summary><p>Câu không có partition key nên đọc mọi partition trên mọi node (toàn bộ lịch sử chuyến) để lọc ra một phần nhỏ. Với bảng đang có: đọc <code>trips_by_city_hour</code> theo từng giờ của ngày (<code>city_id = 1 AND hour_bucket IN (…24 giờ…)</code>, hoặc 24 truy vấn song song) rồi lọc <code>status</code> ở ứng dụng hay bằng <code>ALLOW FILTERING</code> trong các partition đã chỉ rõ. Nếu đây là truy vấn thường xuyên, thiết kế một bảng riêng phục vụ nó (CS-007).</p></details>

**Bài 4.** Làm nút "Xem thêm" cho lịch sử chuyến, mỗi lần 20 chuyến, không dùng paging state. Viết câu cho trang thứ hai.

<details><summary><b>Lời giải</b></summary><p>Giữ <code>requested_at</code> (và <code>trip_id</code>) của dòng cuối trang trước, rồi: <code>SELECT … FROM trips_by_customer WHERE customer_id = X AND requested_at &lt; T_cuoi LIMIT 20;</code> Nếu hai chuyến của cùng một khách có thể trùng <code>requested_at</code> và rơi vào ranh giới trang, cận <code>&lt;</code> sẽ bỏ sót chuyến còn lại; khi đó đọc với <code>requested_at &lt;= T_cuoi</code> và bỏ qua các <code>trip_id</code> đã hiển thị, hoặc dùng paging state.</p></details>

## 11. Đọc tiếp

- [Data manipulation: SELECT](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/dml.html): quy tắc `WHERE`, `ORDER BY`, `PER PARTITION LIMIT`, `ALLOW FILTERING`.
- [CQL functions](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/functions.html): `token()`, các hàm aggregate.
- [PG-007](/articles/postgresql-select-filter-order-limit): lọc, sắp xếp và giới hạn ở PostgreSQL.
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi): số liệu tải dùng để đánh giá chi phí quét.

Bài tiếp theo: [CS-007 · Thiết kế bảng theo truy vấn](/articles/cassandra-thiet-ke-theo-truy-van).
