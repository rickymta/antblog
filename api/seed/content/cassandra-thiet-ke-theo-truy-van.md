---
slug: cassandra-thiet-ke-theo-truy-van
title: "Thiết kế bảng theo truy vấn trong Cassandra: từ màn hình Tới Nơi tới khoá chính"
summary: "Ở Cassandra, bảng được thiết kế từ câu hỏi của ứng dụng chứ không từ sơ đồ thực thể. Bài đi từ màn hình app Tới Nơi tới năm access pattern Q1–Q5, suy ra partition key, clustering key và tên bảng, rồi chạy thử từng truy vấn trên lab Cassandra 5.0."
category: cassandra
level: intermediate
tags: [cassandra, cs-007, cassandra-5, cql, toi-noi, data-modeling, query-first, partition-key, clustering-key]
language: vi
---

Tối 03/11, đội backend Tới Nơi họp để thêm màn hình "Thu nhập hôm nay" cho app tài xế. Một kỹ sư quen PostgreSQL mở bảng `trips` và đề xuất: "Lọc theo `driver_id` và ngày là xong." Trên PostgreSQL thì đúng là xong: thêm một index, planner tự lo. Trên Cassandra, câu đó bị từ chối ngay, vì `driver_id` không phải partition key của `trips`, và Cassandra không có planner nào đi tìm hộ dòng nằm rải trên mọi node.

Bài này đổi thứ tự suy nghĩ: **bắt đầu từ câu hỏi của ứng dụng, rồi mới vẽ bảng**. Đây là cách cả năm bảng của lab P01 được tạo ra, và cũng là kỹ năng quan trọng nhất của người thiết kế dữ liệu trên Cassandra.

<div class="tkh-callout">
<span class="tkh-callout-title">CS-007 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/cassandra-keyspace-bang-khoa-chinh">CS-003 · Keyspace, bảng và khoá chính</a> và <a href="/articles/cassandra-select-where-clustering-order">CS-006 · SELECT theo khoá</a>. Lab: <code>examples/toi-noi-cassandra/p01</code>, file <code>01-schema.cql</code> và <code>06-truy-van-nghiep-vu.cql</code>; kết quả thật trong <code>p01/OUTPUT.txt</code>. Sau bài này, bạn liệt kê được access pattern của một tính năng, vẽ sơ đồ truy vấn kiểu Chebotko, suy ra partition key và clustering key cho từng bảng, và nhận ra truy vấn nào cần bảng mới.</p>
</div>

## 1. Vì sao không bắt đầu từ sơ đồ thực thể?

Với database quan hệ, quy trình quen thuộc là: vẽ thực thể và quan hệ, chuẩn hoá để mỗi dữ kiện nằm ở một chỗ ([PG-008](/articles/postgresql-mo-hinh-quan-he-chuan-hoa)), rồi viết truy vấn bằng JOIN và index. Cách làm này dựa trên ba thứ mà Cassandra không có:

| Database quan hệ dựa vào | Cassandra 5.0 |
|---|---|
| JOIN giữa các bảng | Không có JOIN |
| Planner tự chọn index cho câu `WHERE` bất kỳ | Câu `SELECT` phải nêu đủ partition key; lọc cột thường bị từ chối nếu không có `ALLOW FILTERING` (CS-006) |
| Khoá ngoại giữ quan hệ giữa các bảng | Không có khoá ngoại; mỗi bảng đứng một mình |

Mở sơ đồ keyspace `toinoi` trong DBStudio (chuột phải vào keyspace → "Xem sơ đồ") sẽ thấy điều này rõ nhất:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-007/01-so-do-keyspace.webp" alt="Tab Sơ đồ của DBStudio cho keyspace toinoi: sáu hộp bảng trip_events_by_trip, trips_by_city_day, trips_by_city_hour, trips_by_driver_day, trips_by_customer, trips; các cột khoá có biểu tượng chìa khoá màu vàng; giữa các hộp không có đường nối nào; góc phải ghi 6/6 bảng" width="2800" height="1720" loading="lazy"></div>
<figcaption><b>Hình 1 · Sơ đồ keyspace không có đường quan hệ.</b> Sáu bảng (gồm <code>trips_by_city_day</code> của thí nghiệm CS-009) đứng rời nhau: Cassandra không có khoá ngoại để vẽ đường nối. Cột có biểu tượng chìa là cột thuộc khoá chính. Cùng các cột <code>trip_id</code>, <code>requested_at</code>, <code>fare_vnd</code> xuất hiện ở nhiều bảng: đó là phi chuẩn hoá có chủ đích, chủ đề của CS-008.</figcaption>
</figure>

Vì không có JOIN, mỗi câu hỏi của ứng dụng phải được trả lời bằng **một bảng đã sắp sẵn dữ liệu đúng cho câu hỏi đó**. Muốn biết cần những bảng nào, phải biết trước ứng dụng sẽ hỏi gì. Vì vậy cách làm này thường được gọi là thiết kế **query-first** (truy vấn trước): tài liệu Cassandra bắt đầu mô hình hoá từ việc liệt kê truy vấn của ứng dụng.

## 2. Quy trình query-first trong năm bước

| Bước | Việc | Sản phẩm |
|---|---|---|
| 1 | Liệt kê màn hình và luồng nghiệp vụ (workflow) | Danh sách màn hình, nút bấm, job |
| 2 | Viết mỗi lần đọc dữ liệu thành một **access pattern** | Q1, Q2… với tham số đầu vào và thứ tự kết quả |
| 3 | Mỗi access pattern một bảng (hoặc dùng chung nếu khớp hẳn) | Danh sách bảng |
| 4 | Suy ra khoá chính: partition key từ tham số, clustering key từ thứ tự và tính duy nhất | `PRIMARY KEY (...)` + `CLUSTERING ORDER BY` |
| 5 | Kiểm kích thước partition và cách giữ các bản sao khớp nhau | Ước lượng (CS-009), đường ghi + đối soát (CS-008) |

**Access pattern** (mẫu truy cập) là mô tả một lần đọc: *biết trước những gì* (tham số), *cần lấy gì*, *theo thứ tự nào*, *bao nhiêu dòng*. Nó cụ thể hơn "cần xem lịch sử chuyến": phải nói được "biết `customer_id`, lấy các chuyến mới nhất trước, mỗi trang 20 chuyến".

### Bước 1–2: từ màn hình tới Q1–Q5

Với phần lịch sử chuyến của Tới Nơi, đội sản phẩm có bốn màn hình. Mỗi màn hình sinh ra một hoặc hai access pattern:

| Màn hình / workflow | Access pattern | Biết trước | Thứ tự kết quả |
|---|---|---|---|
| App khách: "Chuyến của tôi" | **Q2** · lịch sử chuyến của một khách | `customer_id` | mới nhất trước |
| Bấm vào một chuyến | **Q1** · chi tiết một chuyến | `trip_id` | một dòng |
| Bấm vào một chuyến | **Q5** · dòng thời gian sự kiện của chuyến | `trip_id` | cũ nhất trước |
| App tài xế: "Thu nhập hôm nay" | **Q3** · chuyến của một tài xế trong một ngày | `driver_id`, ngày (giờ Việt Nam) | mới nhất trước |
| Màn hình điều phối | **Q4** · chuyến mới tạo trong một thành phố, theo giờ | `city_id`, giờ | cũ nhất trước |

Hai điểm cần để ý. Một màn hình có thể cần hai truy vấn (bấm vào chuyến cần cả Q1 và Q5). Và cột "biết trước" là thứ quyết định partition key ở bước 4: ứng dụng chỉ đọc được một partition khi nó **biết đủ giá trị của partition key tại thời điểm hỏi**.

### Bước 3: sơ đồ truy vấn kiểu Chebotko

**Sơ đồ Chebotko** (đặt theo Artem Chebotko, người đề xuất cùng các cộng sự) là ký pháp mô hình hoá dùng cho Cassandra, có hai phần. *Sơ đồ luồng ứng dụng* vẽ các màn hình nối với nhau bằng mũi tên, mỗi mũi tên ghi truy vấn mà bước chuyển đó cần. *Sơ đồ logic* vẽ mỗi bảng thành một hộp, mỗi cột đánh dấu vai trò: **K** là cột partition key, **C↑**/**C↓** là cột clustering tăng/giảm dần, **S** là cột static (CS-015). Viết bằng chữ, sơ đồ của P01 như sau:

```text
Luồng ứng dụng
  [Chuyến của tôi] --Q2--> [Chi tiết chuyến] --Q1, Q5
  [Thu nhập hôm nay] --Q3
  [Điều phối thành phố] --Q4

Sơ đồ logic
  trips (Q1)              trips_by_customer (Q2)    trips_by_driver_day (Q3)
    trip_id       K         customer_id   K           driver_id     K
    customer_id             requested_at  C↓          day           K
    driver_id               trip_id       C↑          requested_at  C↓
    city_id                 service, status,          trip_id       C↑
    requested_at            pickup, dropoff,          customer_id, service,
    status, fare_vnd...     fare_vnd, driver_id       status, fare_vnd

  trips_by_city_hour (Q4)   trip_events_by_trip (Q5)
    city_id       K           trip_id    K
    hour_bucket   K           event_at   C↑
    requested_at  C↑          event_seq  C↑
    trip_id       C↑          status, actor, note
    service, status, fare_vnd
```

Sơ đồ logic được chuyển gần như một-một thành `p01/01-schema.cql`. Ví dụ bảng Q2:

```sql
CREATE TABLE trips_by_customer (
  customer_id  uuid,
  requested_at timestamp,
  trip_id      uuid,
  service      text,
  status       text,
  pickup       text,
  dropoff      text,
  fare_vnd     bigint,
  driver_id    uuid,
  PRIMARY KEY ((customer_id), requested_at, trip_id)
) WITH CLUSTERING ORDER BY (requested_at DESC, trip_id ASC)
  AND comment = 'Q2: lịch sử chuyến theo khách, mới nhất trước';
```

Lab đọc lại khoá của các bảng từ `system_schema.columns`; trích phần của Q2 và Q4 (đầy đủ trong `OUTPUT.txt`):

```text
 table_name          | column_name  | kind          | position | clustering_order | type
  trips_by_city_hour |      city_id | partition_key |        0 |             none |       int
  trips_by_city_hour |  hour_bucket | partition_key |        1 |             none | timestamp
  trips_by_city_hour | requested_at |    clustering |        0 |              asc | timestamp
  trips_by_city_hour |      trip_id |    clustering |        1 |              asc |      uuid
   trips_by_customer |  customer_id | partition_key |        0 |             none |      uuid
   trips_by_customer | requested_at |    clustering |        0 |             desc | timestamp
   trips_by_customer |      trip_id |    clustering |        1 |              asc |      uuid
```

## 3. Chọn partition key: đọc một partition, phân bố đều

**Partition key** quyết định dòng nằm ở partition nào và vì thế ở node nào (CS-003). Có hai quy tắc, và một partition key tốt phải thoả cả hai:

1. **Mỗi truy vấn đọc đúng một partition.** Partition key phải gồm đúng những gì ứng dụng biết lúc hỏi. Q3 biết `driver_id` và ngày, nên partition key là `(driver_id, day)`.
2. **Dữ liệu phân bố đều và mỗi partition có giới hạn.** Có nhiều giá trị khác nhau (để rải đều trên các node) và mỗi partition không phình mãi theo thời gian.

Áp hai quy tắc cho các lựa chọn từng được bàn ở Tới Nơi:

| Partition key đề xuất | Quy tắc 1 | Quy tắc 2 | Kết luận |
|---|---|---|---|
| `customer_id` cho Q2 | Đạt: app khách luôn biết mình là ai | 4 triệu khách hoạt động, mỗi khách vài chuyến mỗi tuần | Dùng |
| `driver_id` cho Q3 | Đạt | Một tài xế chạy nhiều năm, partition lớn dần không giới hạn; màn hình chỉ cần một ngày | Thêm `day` |
| `(driver_id, day)` cho Q3 | Đạt: app tính ngày theo giờ Việt Nam | Mỗi partition là một ngày làm việc, vài chục chuyến | Dùng |
| `city_id` cho Q4 | Đạt | Chỉ 6 thành phố ⇒ 6 partition cho toàn bộ dữ liệu, mỗi partition nhận mọi chuyến của thành phố mãi mãi | Không dùng |
| `(city_id, hour_bucket)` cho Q4 | Đạt: màn hình điều phối xem theo giờ | Mỗi giờ một partition mới | Dùng; kích thước kiểm ở CS-009 |

`hour_bucket` là ví dụ đầu tiên của **bucketing**: thêm vào partition key một cột thời gian đã làm tròn (đầu giờ, đầu ngày) để cắt dữ liệu vô hạn thành các partition có giới hạn. Cột này không có sẵn trong nghiệp vụ; **ứng dụng phải tự tính** khi ghi và khi đọc. CS-009 đo xem bucket theo ngày hay theo giờ hợp với Tới Nơi.

## 4. Chọn clustering key: thứ tự và tính duy nhất

**Clustering key** là các cột còn lại của khoá chính; chúng sắp xếp các dòng *bên trong* một partition. Clustering key làm hai việc cùng lúc.

**Một: quyết định thứ tự đọc ra.** Q2 cần mới nhất trước, nên `requested_at DESC`. Q4 và Q5 cần theo thời gian xuôi, nên để mặc định `ASC`. Thứ tự này ghi sẵn trên đĩa, nên `LIMIT 3` lấy ngay ba chuyến gần nhất mà không phải sắp xếp gì (CS-006). Muốn đọc theo thứ tự khác cột clustering, ví dụ theo cước, thì không làm được bằng `ORDER BY`: lab kiểm lỗi này ở K9 (`Order by is currently only supported on the clustered columns of the PRIMARY KEY`).

**Hai: làm cho khoá chính duy nhất.** Mọi câu ghi trong Cassandra là upsert (CS-005): ghi trùng khoá chính thì dòng sau đè dòng trước, không có lỗi trùng khoá nào báo cho bạn. Nếu khoá của Q2 chỉ là `((customer_id), requested_at)`, một khách đặt xe và gọi đồ ăn trong cùng một mili giây sẽ mất một chuyến mà không ai biết. Thêm `trip_id` vào cuối clustering key để **phá hoà** (tie-breaker): hai chuyến cùng thời điểm vẫn là hai dòng, và `trip_id` không làm đổi thứ tự theo thời gian vì nó đứng sau `requested_at`.

Bảng sự kiện dùng cùng ý tưởng nhưng với `event_seq` (số thứ tự sự kiện trong chuyến) thay vì một UUID. Hai sự kiện của một chuyến có thể mang cùng `event_at` (ví dụ hệ thống ghi `searching` ngay khi khách bấm đặt); `event_seq` vừa giữ cả hai dòng, vừa cho thứ tự đúng nghiệp vụ khi thời điểm bằng nhau.

| Bảng | Clustering key | Thứ tự phục vụ | Cột phá hoà |
|---|---|---|---|
| `trips_by_customer` | `requested_at DESC, trip_id ASC` | Q2 mới nhất trước | `trip_id` |
| `trips_by_driver_day` | `requested_at DESC, trip_id ASC` | Q3 mới nhất trước | `trip_id` |
| `trips_by_city_hour` | `requested_at ASC, trip_id ASC` | Q4 theo thời gian xuôi | `trip_id` |
| `trip_events_by_trip` | `event_at ASC, event_seq ASC` | Q5 cũ nhất trước | `event_seq` |
| `trips` | (không có) | Q1 đúng một dòng | không cần |

## 5. Đặt tên bảng: `<thứ cần lấy>_by_<khoá tra cứu>`

Lab đặt tên theo mẫu `<thứ cần lấy>_by_<khoá tra cứu>`: `trips_by_customer`, `trips_by_driver_day`, `trips_by_city_hour`, `trip_events_by_trip`. Tên nói luôn truy vấn mà bảng phục vụ, nên người đọc code biết ngay `trips_by_customer` không trả lời được câu hỏi theo tài xế. Bảng tra theo khoá tự nhiên của thực thể (`trips` theo `trip_id`) giữ tên ngắn. Mỗi bảng còn ghi mã truy vấn trong `comment` (`'Q2: lịch sử chuyến theo khách, mới nhất trước'`), để DDL tự giải thích khi xem trong DBStudio.

## 6. Chạy thử: mỗi màn hình đọc đúng một partition

File `06-truy-van-nghiep-vu.cql` chạy lại từng màn hình. "Chuyến của tôi" của khách 2, trang đầu ba chuyến:

```sql
SELECT requested_at, service, status, pickup, dropoff, fare_vnd
FROM trips_by_customer
WHERE customer_id = 0c000000-0000-4000-8000-000000000002
LIMIT 3;
```

```text
 requested_at                    | service | status     | pickup                    | dropoff                   | fare_vnd
---------------------------------+---------+------------+---------------------------+---------------------------+----------
 2026-11-03 18:20:00.000000+0700 |    food | in_transit |            Lẩu Dê 404, Q3 | Chung cư Sunrise City, Q7 |   118000
 2026-11-03 07:55:00.000000+0700 |    ride |  completed | Chung cư Sunrise City, Q7 |      Sân bay Tân Sơn Nhất |   145000
 2026-11-02 17:31:00.000000+0700 |    ride |  cancelled | Văn phòng Etown, Tân Bình |         Chợ Bến Thành, Q1 |        0
```

Bấm vào chuyến 8 cần hai truy vấn, mỗi truy vấn một partition: Q1 trả một dòng chi tiết từ `trips`, Q5 trả bảy sự kiện từ `requested` (07:55:00) tới `completed` (08:24:57). Màn hình "Thu nhập hôm nay" của tài xế 2 ngày 03/11 đọc danh sách rồi cộng tiền **trong cùng partition**:

```sql
SELECT COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc_dong
FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000002 AND day = '2026-11-03';
```

```text
 so_chuyen | tong_cuoc_dong
-----------+----------------
         2 |         207000
```

`COUNT` và `SUM` ở đây an toàn vì chúng chỉ quét một partition nhỏ. Cùng câu `COUNT(*)` mà không có partition key sẽ phải quét cả bảng (CS-006).

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-007/02-thu-nhap-tai-xe.webp" alt="Query editor DBStudio chạy script hai câu SELECT trên trips_by_driver_day cho tài xế 0d…02 ngày 2026-11-03; hai tab kết quả #1 SELECT và #2 SELECT, đang mở tab #1 với hai dòng: 2026-11-03T11:20:00.000Z food completed 118000 và 2026-11-03T10:45:00.000Z ride completed 95000; thanh dưới ghi 2 dòng và Chỉ đọc: Thiếu cột khoá driver_id, day, trip_id trong SELECT; thanh trạng thái ghi Tự động commit" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 2 · Q3 trong DBStudio: chạy cả script, mỗi câu một tab kết quả.</b> <code>Ctrl+Shift+Enter</code> chạy hai câu, kết quả nằm ở tab #1 và #2. Lưới hiển thị giờ UTC (<code>11:20Z</code> là 18:20 giờ Việt Nam). Ảnh chụp sau khi chạy hết P01, nên chuyến 12 đã <code>completed</code> và chuyến 11 có cước 95.000 đ do các bước ở CS-008; lúc chạy file 06 thì vẫn là <code>in_transit</code>, 89.000 đ và tổng 207.000 đ như khối kết quả ở trên. Dòng "Chỉ đọc" nhắc rằng muốn sửa trên lưới thì SELECT phải có đủ cột khoá.</figcaption>
</figure>

Sau toàn bộ P01, chạy lại câu tổng trên lab (chỉ đọc) cho `2 | 213000`, khớp với 118.000 + 95.000 trong ảnh.

## 7. Anti-pattern: hỏi bảng `trips` theo khách

Câu hỏi "lịch sử chuyến của khách 2" đã có bảng Q2. Nhưng thói quen SQL sẽ kéo bạn về bảng `trips`, vì bảng đó cũng có cột `customer_id`:

```sql
SELECT trip_id, requested_at, status FROM trips
WHERE customer_id = 0c000000-0000-4000-8000-000000000002;
```

```text
Cannot execute this query as it might involve data filtering and thus may have unpredictable performance. If you want to execute this query despite the performance unpredictability, use ALLOW FILTERING
```

Thêm `ALLOW FILTERING` thì câu chạy được:

```text
 trip_id                              | requested_at                    | status
--------------------------------------+---------------------------------+------------
 07000000-0000-4000-8000-000000000002 | 2026-11-02 11:05:00.000000+0700 |  completed
 07000000-0000-4000-8000-000000000005 | 2026-11-02 17:31:00.000000+0700 |  cancelled
 07000000-0000-4000-8000-000000000012 | 2026-11-03 18:20:00.000000+0700 | in_transit
 07000000-0000-4000-8000-000000000008 | 2026-11-03 07:55:00.000000+0700 |  completed
```

Đọc kỹ kết quả: chuyến ngày 03/11 lúc 18:20 đứng *trước* chuyến 03/11 lúc 07:55. Các dòng ra theo **thứ tự token của `trip_id`** (mỗi chuyến là một partition riêng), không theo thời gian; và `trips` không có cột clustering nên `ORDER BY` cũng không cứu được. Tệ hơn, để tìm bốn dòng này Cassandra đọc **mọi partition của bảng trên mọi node**. Trên lab 12 chuyến thì không sao; với 730 triệu chuyến mỗi năm, đây là một lần quét toàn cụm cho mỗi lần khách mở app.

## 8. Câu hỏi mới thì cần bảng mới

Bộ phận vận hành hỏi thêm: "các chuyến bị huỷ trong một giờ ở TP.HCM". Bảng Q4 có `city_id`, `hour_bucket` và cột `status`, nên thử lọc:

```sql
SELECT trip_id FROM trips_by_city_hour
WHERE city_id = 1 AND hour_bucket = '2026-11-02 17:00:00+0700' AND status = 'cancelled';
```

Câu này bị từ chối với cùng thông điệp `Cannot execute this query as it might involve data filtering…`. Câu này đọc trong một partition, nên `ALLOW FILTERING` ở đây không làm quét toàn cụm; nhưng nó vẫn đọc mọi chuyến của giờ đó (hàng chục nghìn ở giờ cao điểm thật, xem CS-009) để giữ lại vài chuyến huỷ. Quy trình query-first trả lời: đây là access pattern mới, hãy quay lại bước 2 và thiết kế bảng cho nó. Bạn làm việc này ở Bài 1 bên dưới.

## 9. Lỗi thường gặp

- **Vẽ ERD trước rồi "dịch" sang CQL.** Kết quả là bảng `trips` đầy đủ cột nhưng không trả lời được truy vấn nào ngoài Q1. Hãy bắt đầu từ danh sách màn hình.
- **Partition key thiếu thứ ứng dụng không biết lúc đọc.** Ví dụ đưa `requested_at` chính xác vào partition key của Q2: khách không thể biết trước giờ đặt của mọi chuyến để hỏi. Partition key chỉ gồm tham số có sẵn ở bước 2.
- **Quên cột phá hoà.** Khoá `((driver_id, day), requested_at)` trông hợp lý, cho tới khi hai chuyến cùng mili giây ghi đè nhau mà không có lỗi nào.
- **Đặt partition key có quá ít giá trị.** `city_id` một mình, hay `service` (chỉ 3 giá trị), dồn dữ liệu vào vài partition khổng lồ và vài node nóng.
- **Dùng `ALLOW FILTERING` để vá một truy vấn thiếu bảng.** Chấp nhận được khi chỉ lọc trong một partition nhỏ; ngoài trường hợp đó, đó là dấu hiệu cần bảng mới.

## 10. So với PostgreSQL

Ở [PG-010](/articles/postgresql-thuc-hanh-schema-chuyen-xe), một bảng `trips` chuẩn hoá cộng vài index trả lời cả Q1–Q4; thêm câu hỏi mới thường chỉ cần thêm index, và planner chọn đường đọc. Ở Cassandra, *người thiết kế* làm việc của planner từ trước: mỗi truy vấn có bảng riêng, thứ tự nằm sẵn trên đĩa, và thêm câu hỏi mới nghĩa là thêm bảng (cùng việc nạp lại dữ liệu cũ vào bảng đó). Đổi lại, mỗi lần đọc chỉ chạm một partition trên vài node, nên độ trễ đọc giữ ổn định khi cụm lớn dần.

## 11. Bài tập

**Bài 1.** Thiết kế bảng cho access pattern "các chuyến bị huỷ trong một giờ ở TP.HCM, theo thời điểm huỷ, cũ nhất trước". Viết `CREATE TABLE`, giải thích partition key, clustering key, và khi nào ứng dụng ghi vào bảng này.

<details><summary><b>Lời giải</b></summary><p>Câu hỏi là "huỷ trong một giờ", nên bucket theo <b>thời điểm huỷ</b>, không theo giờ đặt:</p>
<pre><code>CREATE TABLE cancelled_trips_by_city_hour (
  city_id      int,
  hour_bucket  timestamp,   -- cancelled_at làm tròn xuống đầu giờ (giờ Việt Nam)
  cancelled_at timestamp,
  trip_id      uuid,
  requested_at timestamp,
  service      text,
  cancelled_by text,        -- 'customer' | 'driver' | 'system'
  PRIMARY KEY ((city_id, hour_bucket), cancelled_at, trip_id)
) WITH comment = 'Chuyến bị huỷ theo thành phố và giờ huỷ';</code></pre>
<p>Partition key <code>(city_id, hour_bucket)</code> vì màn hình biết thành phố và giờ; bucket giờ giữ partition có giới hạn. Clustering <code>cancelled_at ASC</code> cho thứ tự cũ nhất trước, <code>trip_id</code> phá hoà. Ứng dụng ghi một dòng vào bảng này đúng lúc chuyển chuyến sang <code>cancelled</code>, trong cùng logged batch cập nhật trạng thái ở các bảng khác (CS-008). Bảng chỉ chứa chuyến huỷ nên partition nhỏ hơn nhiều so với <code>trips_by_city_hour</code>.</p></details>

**Bài 2.** Màn hình mới của app tài xế: "10 chuyến gần nhất của tôi, không quan tâm ngày". Bảng `trips_by_driver_day` có trả lời được không? Nếu không, đề xuất cách làm.

<details><summary><b>Lời giải</b></summary><p>Không trực tiếp: partition key cần cả <code>day</code>, nên phải hỏi lần lượt hôm nay, hôm qua… cho tới khi đủ 10 chuyến (tài xế nghỉ vài ngày thì số lần hỏi tăng). Hai cách: (a) chấp nhận đọc ngược theo ngày, giới hạn số ngày tối đa; (b) thêm bảng <code>trips_by_driver</code> với khoá <code>((driver_id), requested_at, trip_id)</code> DESC, kèm TTL hoặc bucket theo tháng để partition không phình mãi. Chọn theo tần suất của màn hình và kích thước ước lượng (CS-009).</p></details>

**Bài 3.** Một đồng nghiệp đề xuất khoá `PRIMARY KEY ((trip_id), event_seq)` cho bảng sự kiện, bỏ `event_at` khỏi khoá. Nêu một lợi ích và một rủi ro.

<details><summary><b>Lời giải</b></summary><p>Lợi ích: khoá ngắn hơn, <code>event_seq</code> đã đủ phân biệt và đủ thứ tự nếu luôn tăng. Rủi ro: <code>event_seq</code> phải do một bên duy nhất cấp, tăng đúng; nếu hai dịch vụ cùng ghi sự kiện với cùng số thứ tự thì dòng sau đè dòng trước (upsert). Giữ <code>event_at</code> đứng trước cho thứ tự theo thời gian thật và giảm khả năng đè nhau; <code>event_seq</code> chỉ còn vai trò phá hoà.</p></details>

## 12. Đọc tiếp

- [Data modeling: Introduction](https://cassandra.apache.org/doc/5.0/cassandra/developing/data-modeling/intro.html) và [Defining application queries](https://cassandra.apache.org/doc/5.0/cassandra/developing/data-modeling/data-modeling_queries.html) — quy trình query-first trong tài liệu Cassandra 5.0.
- [Logical data modeling](https://cassandra.apache.org/doc/5.0/cassandra/developing/data-modeling/data-modeling_logical.html) — sơ đồ Chebotko và cách đánh dấu K, C↑, C↓, S.
- [CS-006 · SELECT theo khoá](/articles/cassandra-select-where-clustering-order): vì sao truy vấn thiếu partition key bị từ chối.
- [PG-008 · Mô hình quan hệ và chuẩn hoá](/articles/postgresql-mo-hinh-quan-he-chuan-hoa): cách nghĩ ngược lại ở database quan hệ.

Bài tiếp theo: [CS-008 · Phi chuẩn hoá: một truy vấn một bảng và giữ các bản sao khớp nhau](/articles/cassandra-phi-chuan-hoa-nhieu-bang).
