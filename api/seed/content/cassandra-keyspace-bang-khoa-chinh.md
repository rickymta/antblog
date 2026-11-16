---
slug: cassandra-keyspace-bang-khoa-chinh
title: "Keyspace, bảng và khoá chính trong Cassandra: partition key và clustering key của lịch sử chuyến"
summary: "Màn hình \"Chuyến của tôi\" cần mọi chuyến của một khách, mới nhất trước, trong một lần đọc. Bài này đọc keyspace toinoi và các bảng của lab P01 để giải thích khoá chính ((a, b), c, d): partition key quyết định dòng nằm ở partition nào (qua token Murmur3), clustering key quyết định thứ tự dòng trong partition."
category: cassandra
level: beginner
tags: [cassandra, cs-003, cassandra-5, cql, keyspace, primary-key, partition-key, clustering-key, token, toi-noi]
language: vi
---

Màn hình "Chuyến của tôi" trong ứng dụng khách Tới Nơi chỉ hỏi một câu: **các chuyến của khách này, mới nhất trước**. Với 2 triệu chuyến mỗi ngày, bảng chứa câu trả lời sẽ có hàng trăm triệu dòng sau một năm. Trong PostgreSQL, bạn tạo bảng `trips`, thêm index trên `(customer_id, requested_at DESC)` và để planner lo phần còn lại.

Cassandra không có planner chọn đường cho bạn. Nó trả lời nhanh **đúng một kiểu câu hỏi cho mỗi bảng**: kiểu câu hỏi được mã hoá sẵn trong khoá chính. Khoá chính trả lời hai câu: *dòng này nằm ở partition nào* (và vì vậy ở node nào), và *dòng này đứng ở vị trí nào trong partition*. Bài này đọc các bảng của lab P01 để bạn đọc được mọi khoá chính dạng `((a, b), c, d)`.

<div class="tkh-callout">
<span class="tkh-callout-title">CS-003 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/cassandra-la-gi-va-vai-tro-trong-toi-noi">CS-001</a>, <a href="/articles/cassandra-cai-dat-docker-dbstudio">CS-002</a>. Lab: <code>examples/toi-noi-cassandra/p01</code>, file <code>00-reset.cql</code>, <code>01-schema.cql</code>, <code>05-select.cql</code>. Sau bài này, bạn tạo được keyspace và bảng, đọc được khoá chính <code>((a, b), c, d)</code>, phân biệt partition key một cột và nhiều cột, và biết một dòng nằm ở partition nào, theo thứ tự nào.</p>
</div>

## 1. Keyspace: nơi chứa bảng và quy định số bản sao

File `00-reset.cql` tạo keyspace của lab:

```sql
CREATE KEYSPACE toinoi
  WITH replication = {'class': 'NetworkTopologyStrategy', 'dc1': 1}
  AND durable_writes = true;
```

- `replication` là tham số bắt buộc. **`NetworkTopologyStrategy`** đặt số bản sao (*replication factor*, RF) riêng cho từng datacenter: ở đây `dc1` giữ 1 bản sao của mỗi dòng, vì lab Phần 1 chỉ có một node. Phần 3 dựng cụm ba node và nâng lên `'dc1': 3` (CS-023). Còn một chiến lược cũ là `SimpleStrategy`, không phân biệt datacenter; chỉ nên dùng cho thử nghiệm.
- `durable_writes = true` (mặc định) nghĩa là mọi câu ghi vào keyspace đi qua commit log. Tắt nó là chấp nhận mất dữ liệu khi node sập.

Catalog `system_schema.keyspaces` cho thấy Cassandra lưu lại đúng như vậy, với tên lớp đầy đủ:

```text
 keyspace_name | durable_writes | replication
---------------+----------------+-------------------------------------------------------------------------------
        toinoi |           True | {'class': 'org.apache.cassandra.locator.NetworkTopologyStrategy', 'dc1': '1'}
```

Keyspace là đơn vị nhân bản: mọi bảng trong `toinoi` có cùng RF. Muốn bảng vị trí tài xế giữ 2 bản sao còn bảng lịch sử chuyến giữ 3, bạn phải đặt chúng vào hai keyspace khác nhau.

## 2. Bảng và khoá chính

Bảng lịch sử chuyến theo khách trong `01-schema.cql`:

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

Tên bảng theo quy ước của lab: `<thứ cần lấy>_by_<khoá tra cứu>`. Mỗi bảng phục vụ **một** truy vấn, và tên nói luôn truy vấn đó (CS-007).

Mọi bảng Cassandra **phải** có khoá chính. Khoá chính gồm hai phần:

- **Partition key**: phần trong cặp ngoặc trong cùng, `(customer_id)`. Mọi dòng có cùng giá trị partition key thuộc cùng một **partition**, nằm cùng nhau trên cùng các node.
- **Clustering key** (các *clustering column*): phần còn lại, `requested_at, trip_id`. Chúng phân biệt các dòng trong cùng partition và quyết định **thứ tự** lưu các dòng đó.

Cả khoá chính `(customer_id, requested_at, trip_id)` phải duy nhất: ghi một dòng có khoá trùng với dòng đã có thì dòng cũ bị ghi đè, không có lỗi trùng khoá (CS-005). Vì vậy `trip_id` có mặt trong khoá: hai chuyến của cùng một khách tạo cùng mili giây vẫn là hai dòng khác nhau.

### Đọc khoá chính `((a, b), c, d)`

Quy tắc đọc rất máy móc:

1. Lấy **phần tử đầu tiên** trong `PRIMARY KEY (…)`. Nếu nó là một cặp ngoặc, mọi cột trong ngoặc là partition key; nếu là một cột đơn, chỉ cột đó là partition key.
2. Mọi cột còn lại, theo đúng thứ tự viết, là clustering key.

| Khoá chính | Partition key | Clustering key |
|---|---|---|
| `PRIMARY KEY (trip_id)` | `trip_id` | không có |
| `PRIMARY KEY ((customer_id), requested_at, trip_id)` | `customer_id` | `requested_at`, `trip_id` |
| `PRIMARY KEY (customer_id, requested_at, trip_id)` | `customer_id` | `requested_at`, `trip_id` (cùng nghĩa với dòng trên) |
| `PRIMARY KEY ((driver_id, day), requested_at, trip_id)` | `driver_id` **và** `day` | `requested_at`, `trip_id` |
| `PRIMARY KEY (driver_id, day, requested_at, trip_id)` | chỉ `driver_id` | `day`, `requested_at`, `trip_id` |

Hai dòng cuối chỉ khác nhau một cặp ngoặc nhưng là hai mô hình khác hẳn: dòng thứ tư tạo **một partition cho mỗi tài xế mỗi ngày**; dòng thứ năm dồn **mọi chuyến trong đời một tài xế** vào một partition, partition đó lớn mãi không ngừng. Lab viết thừa cặp ngoặc quanh partition key một cột, như `((customer_id), …)`, để người đọc không phải nhớ quy tắc 1. DBStudio sinh DDL ở dạng rút gọn `PRIMARY KEY (customer_id, requested_at, trip_id)` (Hình 1 của [CS-001](/articles/cassandra-la-gi-va-vai-tro-trong-toi-noi)); hai cách viết cùng một nghĩa.

### Partition key nhiều cột và khoá chính nhiều phần

Tài liệu Cassandra dùng hai tên dễ lẫn:

- **Composite partition key**: partition key gồm nhiều cột, như `(driver_id, day)` của `trips_by_driver_day` hay `(city_id, hour_bucket)` của `trips_by_city_hour`. Cassandra băm **cả bộ giá trị** thành một token, nên muốn tìm partition thì phải biết **đủ mọi cột** của partition key.
- **Compound primary key**: khoá chính có thêm clustering column, như mọi bảng `trips_by_…` của lab. Bảng `trips` với `PRIMARY KEY (trip_id)` thì không phải: mỗi partition chỉ có đúng một dòng.

Hệ quả của composite partition key thấy ngay trong `05-select.cql`. Hỏi chuyến của tài xế 1 ngày 02/11 với đủ hai cột thì chạy:

```sql
SELECT requested_at, service, status, fare_vnd
FROM trips_by_driver_day
WHERE driver_id = 0d000000-0000-4000-8000-000000000001 AND day = '2026-11-02';
```

```text
 requested_at                    | service | status    | fare_vnd
---------------------------------+---------+-----------+----------
 2026-11-02 11:48:00.000000+0700 |    food | completed |    35000
 2026-11-02 07:42:00.000000+0700 |    ride | completed |    86000
```

Bỏ `day`, chỉ giữ `driver_id`, thì Cassandra từ chối, vì thiếu một nửa partition key nghĩa là không tính được token, phải quét mọi partition:

```text
PASS lỗi mong đợi (dòng 55): Cannot execute this query as it might involve data filtering and thus may have unpredictable performance. If you want to execute this query despite the performance unpredictability, use ALLOW FILTERING
```

CS-006 nói kỹ về `ALLOW FILTERING` và vì sao gần như không bao giờ nên thêm nó.

## 3. Catalog nói gì về khoá của từng bảng

Cuối `01-schema.cql`, lab đọc lại định nghĩa từ bảng hệ thống `system_schema.columns`:

```sql
SELECT table_name, column_name, kind, position, clustering_order, type
FROM system_schema.columns
WHERE keyspace_name = 'toinoi'
  AND table_name IN ('trips', 'trips_by_customer', 'trips_by_driver_day', 'trips_by_city_hour', 'trip_events_by_trip');
```

Trích các dòng của ba bảng (đủ 40 dòng nằm trong `OUTPUT.txt`):

```text
 table_name          | column_name  | kind          | position | clustering_order | type
---------------------+--------------+---------------+----------+------------------+-----------
 trips_by_customer   |  customer_id | partition_key |        0 |             none |      uuid
 trips_by_customer   |    driver_id |       regular |       -1 |             none |      uuid
 trips_by_customer   | requested_at |    clustering |        0 |             desc | timestamp
 trips_by_customer   |      trip_id |    clustering |        1 |              asc |      uuid
 trips_by_driver_day |          day | partition_key |        1 |             none |      date
 trips_by_driver_day |    driver_id | partition_key |        0 |             none |      uuid
 trips_by_driver_day | requested_at |    clustering |        0 |             desc | timestamp
 trips_by_driver_day |      trip_id |    clustering |        1 |              asc |      uuid
 trip_events_by_trip |     event_at |    clustering |        0 |              asc | timestamp
 trip_events_by_trip |    event_seq |    clustering |        1 |              asc |       int
 trip_events_by_trip |      trip_id | partition_key |        0 |             none |      uuid
```

- Cột `kind` có ba giá trị: `partition_key`, `clustering`, `regular` (cột thường, không thuộc khoá). Cassandra còn có loại `static` (CS-015).
- `position` cho thứ tự trong từng nhóm: `driver_id` là phần 0, `day` là phần 1 của partition key. Cột thường có `position = -1`.
- `clustering_order` là `desc` cho `requested_at` của hai bảng lịch sử, `asc` cho `event_at` của dòng thời gian sự kiện.
- Catalog liệt kê cột **theo tên** (`customer_id`, `driver_id`, `requested_at`…), không theo thứ tự trong `CREATE TABLE`. Muốn biết thứ tự khoá, đọc `kind` và `position`, đừng đọc thứ tự dòng.

## 4. CLUSTERING ORDER BY: thứ tự được lưu sẵn

`WITH CLUSTERING ORDER BY (requested_at DESC, trip_id ASC)` nói rằng trong mỗi partition, các dòng được **lưu trên đĩa** theo `requested_at` giảm dần, hai dòng cùng `requested_at` thì theo `trip_id` tăng dần. Không khai báo thì mọi clustering column mặc định `ASC`, như `trip_events_by_trip`: dòng thời gian sự kiện đọc từ cũ tới mới.

Vì thứ tự đã nằm trong bảng, câu hỏi "Chuyến của tôi" chỉ cần nêu partition key, không cần `ORDER BY`:

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

Cassandra tìm partition của khách 1, rồi đọc tuần tự từ đầu partition: dòng đầu tiên đã là chuyến mới nhất. Thêm `LIMIT 20` cho trang đầu thì node dừng sau 20 dòng, không cần sắp xếp gì. Thứ tự clustering **không đổi được** sau khi tạo bảng; chọn sai thì phải tạo bảng mới và chép dữ liệu sang. Vì vậy hãy chọn thứ tự theo cách màn hình đọc dữ liệu nhiều nhất.

## 5. Partition rải theo token

Cassandra dùng **partitioner** để biến partition key thành **token**, một số nguyên 64 bit có dấu. Partitioner mặc định là **Murmur3Partitioner** (lab đọc được tên này trong `system.local`, xem CS-002): nó băm partition key bằng hàm Murmur3, cho token trong khoảng từ −2<sup>63</sup> tới 2<sup>63</sup>−1. Mỗi node nhận một số đoạn trên dải token đó (lab đặt `num_tokens: 16`); partition có token rơi vào đoạn của node nào thì node đó giữ nó. CS-021 đi sâu vào vòng token và vnode; ở bài này chỉ cần một kết luận: **vị trí của partition do giá trị băm quyết định, không do giá trị gốc**.

Hàm `token()` của CQL cho xem token của một partition key. Lab đọc token của ba khách:

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

Khách 3 đứng trước khách 2: khi đọc nhiều partition, Cassandra trả chúng theo thứ tự **token**, không theo thứ tự `customer_id`. Đọc cả bảng kèm token (truy vấn chỉ đọc chạy trên lab sau `npm run p01`; đây là cách đọc toàn bảng chỉ chấp nhận được với bảng 12 dòng):

```sql
SELECT token(customer_id) AS tk, customer_id, requested_at, trip_id, service, fare_vnd
FROM trips_by_customer;
```

```text
 tk                   | customer_id                          | requested_at                    | trip_id                              | service | fare_vnd
----------------------+--------------------------------------+---------------------------------+--------------------------------------+---------+----------
 -2983185843790857942 | 0c000000-0000-4000-8000-000000000001 | 2026-11-03 17:45:00.000000+0700 | 07000000-0000-4000-8000-000000000011 |    ride |    95000
 -2983185843790857942 | 0c000000-0000-4000-8000-000000000001 | 2026-11-03 11:20:00.000000+0700 | 07000000-0000-4000-8000-000000000009 |    food |    52000
 -2983185843790857942 | 0c000000-0000-4000-8000-000000000001 | 2026-11-02 18:02:00.000000+0700 | 07000000-0000-4000-8000-000000000006 |    ride |    91000
 -2983185843790857942 | 0c000000-0000-4000-8000-000000000001 | 2026-11-02 11:48:00.000000+0700 | 07000000-0000-4000-8000-000000000003 |    food |    35000
 -2983185843790857942 | 0c000000-0000-4000-8000-000000000001 | 2026-11-02 07:42:00.000000+0700 | 07000000-0000-4000-8000-000000000001 |    ride |    86000
   144159704313836052 | 0c000000-0000-4000-8000-000000000003 | 2026-11-03 11:35:00.000000+0700 | 07000000-0000-4000-8000-000000000010 |    ride |    57000
   144159704313836052 | 0c000000-0000-4000-8000-000000000003 | 2026-11-02 18:40:00.000000+0700 | 07000000-0000-4000-8000-000000000007 |  parcel |    28000
   144159704313836052 | 0c000000-0000-4000-8000-000000000003 | 2026-11-02 12:10:00.000000+0700 | 07000000-0000-4000-8000-000000000004 |    ride |    64000
  7165355148280075549 | 0c000000-0000-4000-8000-000000000002 | 2026-11-03 18:20:00.000000+0700 | 07000000-0000-4000-8000-000000000012 |    food |   118000
  7165355148280075549 | 0c000000-0000-4000-8000-000000000002 | 2026-11-03 07:55:00.000000+0700 | 07000000-0000-4000-8000-000000000008 |    ride |   145000
  7165355148280075549 | 0c000000-0000-4000-8000-000000000002 | 2026-11-02 17:31:00.000000+0700 | 07000000-0000-4000-8000-000000000005 |    ride |        0
  7165355148280075549 | 0c000000-0000-4000-8000-000000000002 | 2026-11-02 11:05:00.000000+0700 | 07000000-0000-4000-8000-000000000002 |    food |    42000

(12 rows)
```

Cước chuyến 11 là 95000 chứ không phải 89000 như ở mục 4, vì `07-phi-chuan-hoa.cql` (CS-008) đã sửa cước chuyến này sau đó. Ba điều đọc được từ kết quả:

1. Các dòng **cùng token đứng liền nhau**: đó là một partition.
2. Các partition xếp theo token tăng dần.
3. Trong mỗi partition, dòng xếp theo `requested_at` giảm dần, đúng `CLUSTERING ORDER BY`.

<figure class="tkh-fig"><div class="tkh-fig-box"><svg viewBox="0 0 900 360" role="img" aria-label="Dải token Murmur3 từ âm 2 mũ 63 tới 2 mũ 63 trừ 1. Ba partition của bảng trips_by_customer đặt theo token: khách 1 token âm 2983185843790857942 có 5 chuyến, khách 3 token 144159704313836052 có 3 chuyến, khách 2 token 7165355148280075549 có 4 chuyến. Trong mỗi partition, các dòng xếp theo requested_at giảm dần."><defs><marker id="cs3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0L10 5L0 10z" class="f-muted"/></marker></defs><text x="40" y="22" class="t-b">Dải token (Murmur3Partitioner)</text><line x1="40" y1="50" x2="860" y2="50" class="s-muted" stroke-width="2" marker-end="url(#cs3-a)"/><text x="40" y="72" class="t-small">−2^63</text><text x="800" y="72" class="t-small">2^63 − 1</text><g class="f-phys s-phys"><circle cx="317" cy="50" r="6"/><circle cx="456" cy="50" r="6"/><circle cx="768" cy="50" r="6"/></g><g class="f-none s-phys dash"><path d="M317 56V100H155V110"/><path d="M456 56V110"/><path d="M768 56V100H745V110"/></g><g class="f-phys-soft s-phys"><rect x="20" y="110" width="270" height="200" rx="8"/><rect x="315" y="110" width="270" height="200" rx="8"/><rect x="610" y="110" width="270" height="200" rx="8"/></g><g class="t-b"><text x="34" y="132">Partition: khách 1</text><text x="329" y="132">Partition: khách 3</text><text x="624" y="132">Partition: khách 2</text></g><g class="t-mono t-phys"><text x="34" y="150">token −2983185843790857942</text><text x="329" y="150">token 144159704313836052</text><text x="624" y="150">token 7165355148280075549</text></g><g class="f-surface s-line"><rect x="30" y="160" width="250" height="24" rx="4"/><rect x="30" y="188" width="250" height="24" rx="4"/><rect x="30" y="216" width="250" height="24" rx="4"/><rect x="30" y="244" width="250" height="24" rx="4"/><rect x="30" y="272" width="250" height="24" rx="4"/><rect x="325" y="160" width="250" height="24" rx="4"/><rect x="325" y="188" width="250" height="24" rx="4"/><rect x="325" y="216" width="250" height="24" rx="4"/><rect x="620" y="160" width="250" height="24" rx="4"/><rect x="620" y="188" width="250" height="24" rx="4"/><rect x="620" y="216" width="250" height="24" rx="4"/><rect x="620" y="244" width="250" height="24" rx="4"/></g><g class="t-mono"><text x="40" y="177">03/11 17:45 · chuyến 11</text><text x="40" y="205">03/11 11:20 · chuyến 09</text><text x="40" y="233">02/11 18:02 · chuyến 06</text><text x="40" y="261">02/11 11:48 · chuyến 03</text><text x="40" y="289">02/11 07:42 · chuyến 01</text><text x="335" y="177">03/11 11:35 · chuyến 10</text><text x="335" y="205">02/11 18:40 · chuyến 07</text><text x="335" y="233">02/11 12:10 · chuyến 04</text><text x="630" y="177">03/11 18:20 · chuyến 12</text><text x="630" y="205">03/11 07:55 · chuyến 08</text><text x="630" y="233">02/11 17:31 · chuyến 05</text><text x="630" y="261">02/11 11:05 · chuyến 02</text></g><text x="20" y="335" class="t-small">Partition key customer_id → token → vị trí trên dải token (và node giữ đoạn đó). Trong partition: requested_at DESC, trip_id ASC phá hoà.</text></svg></div><figcaption><b>Hình 1 · Partition và clustering của trips_by_customer.</b> Partition key quyết định "ở đâu": mỗi khách thành một partition, đặt trên dải token theo giá trị băm của <code>customer_id</code>, nên khách 3 đứng giữa khách 1 và khách 2. Clustering key quyết định "theo thứ tự nào": trong từng ô, chuyến mới nhất nằm trên cùng.</figcaption></figure>

Hệ quả thực tế của việc rải theo token: **dải giá trị trên partition key không có nghĩa**. `city_id = 1 AND hour_bucket >= 11:00 AND hour_bucket < 13:00` trên `trips_by_city_hour` bị từ chối (`05-select.cql`, mục "Dải trên partition key"), vì giờ 11 và giờ 12 là hai partition có token cách nhau tuỳ ý, có thể nằm trên hai node khác nhau. Muốn đọc nhiều partition thì liệt kê đủ từng giá trị bằng `IN` (CS-006). Ngược lại, dải trên **clustering column** trong một partition (chuyến của khách 1 trong ngày 02/11) là phép đọc rẻ nhất có thể, vì các dòng đã nằm liền nhau và đã sắp xếp.

### Bẫy khi đặt tên cột token

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-003/01-loi-token-tu-khoa.webp" alt="DBStudio, tab Truy vấn 1 trên kết nối toinoi-lab, keyspace toinoi. Câu SELECT token(customer_id) AS token, customer_id, requested_at, service, fare_vnd FROM trips_by_customer; bị báo lỗi đỏ: line 1:29 no viable alternative at input 'token' (SELECT token(customer_id) AS [token]...), chữ token sau AS bị gạch chân đỏ, tab Thông điệp ghi Dòng 1, cột 30, nút Đến vị trí lỗi." width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 2 · Không dùng được <code>token</code> làm bí danh cột.</b> Nhìn chữ <code>token</code> thứ hai, sau <code>AS</code>: DBStudio gạch chân đỏ và trình phân tích CQL báo lỗi cú pháp ở cột 30. <code>token</code> là từ khoá của CQL; đặt bí danh khác (<code>tk</code>, <code>token_cua_partition</code>) hoặc đặt trong ngoặc kép (<code>AS "token"</code>).</figcaption>
</figure>

cqlsh báo đúng lỗi đó (`SyntaxException: line 1:29 no viable alternative at input 'token'`), còn `AS "token"` thì chạy được. Đây cũng là lý do lab đặt bí danh `token_cua_partition`.

## 6. Lỗi thường gặp

- **Quên cặp ngoặc của partition key nhiều cột.** `PRIMARY KEY (driver_id, day, …)` biến `day` thành clustering column, mọi chuyến của tài xế dồn vào một partition ngày càng lớn (CS-009 đo kích thước partition).
- **Bỏ cột phá hoà khỏi khoá.** `PRIMARY KEY ((customer_id), requested_at)` thì hai chuyến cùng mili giây của một khách ghi đè lẫn nhau mà không báo lỗi.
- **Mong `ORDER BY` theo cột bất kỳ.** `ORDER BY` chỉ đảo được thứ tự clustering đã khai báo; `ORDER BY fare_vnd` bị từ chối (CS-006).
- **Mong sửa được khoá.** Không đổi được giá trị cột thuộc khoá chính bằng `UPDATE`, cũng không đổi được `CLUSTERING ORDER BY` bằng `ALTER TABLE`. Khoá sai thì tạo bảng mới.
- **Đọc thứ tự token là thứ tự dữ liệu.** Kết quả đọc nhiều partition trông có thứ tự "lạ" vì đó là thứ tự token; đừng dựa vào nó trong ứng dụng.

## 7. So với PostgreSQL

Trong [PG-003](/articles/postgresql-bang-dong-cot-primary-foreign-key), khoá chính là ràng buộc định danh dòng, đi kèm một B-tree index; thứ tự lưu dòng và cách truy vấn được chọn độc lập, bạn thêm index cho mỗi kiểu câu hỏi mới. Trong Cassandra, khoá chính **chính là** cách lưu: partition key chọn node và partition, clustering key chọn thứ tự trên đĩa. Một câu hỏi mới với khoá tra cứu khác thường nghĩa là **một bảng mới** chứa cùng dữ liệu, sắp theo cách khác (CS-007, CS-008). Cassandra cũng không có khoá ngoại: `customer_id` trong `trips_by_customer` không trỏ tới bảng khách nào.

## 8. Bài tập

**Bài 1.** Với `PRIMARY KEY ((city_id, hour_bucket), requested_at, trip_id)` của `trips_by_city_hour`, nêu partition key, clustering key, và số partition mà một thành phố sinh ra trong một ngày.

<details><summary><b>Lời giải</b></summary><p>Partition key là <code>(city_id, hour_bucket)</code>; clustering key là <code>requested_at</code>, <code>trip_id</code>. Mỗi thành phố sinh 24 partition mỗi ngày, mỗi giờ một partition. Đọc chuyến trong một giờ cần biết cả <code>city_id</code> và <code>hour_bucket</code>.</p></details>

**Bài 2.** Viết `CREATE TABLE` cho hộp thư thông báo: đọc theo người nhận, mới nhất trước; mỗi thông báo có mã `notification_id uuid`, thời điểm `sent_at timestamp`, tiêu đề và nội dung.

<details><summary><b>Lời giải</b></summary><p><code>CREATE TABLE notifications_by_user (user_id uuid, sent_at timestamp, notification_id uuid, title text, body text, PRIMARY KEY ((user_id), sent_at, notification_id)) WITH CLUSTERING ORDER BY (sent_at DESC, notification_id ASC);</code> — <code>notification_id</code> phá hoà khi hai thông báo cùng mili giây. Với người dùng lâu năm, partition sẽ lớn dần; CS-009 và CS-020 bàn cách chia theo tháng.</p></details>

**Bài 3.** Đọc `system_schema.columns` của `trips_by_city_hour` trong `OUTPUT.txt`. Cột nào có `kind = partition_key` và `position = 1`? `clustering_order` của `requested_at` là gì, và vì sao khác với `trips_by_customer`?

<details><summary><b>Lời giải</b></summary><p><code>hour_bucket</code> là phần thứ hai (position 1) của partition key. <code>requested_at</code> có <code>clustering_order = asc</code>, vì màn hình điều phối đọc chuyến trong giờ theo thứ tự xảy ra; còn "Chuyến của tôi" cần mới nhất trước nên dùng <code>desc</code>.</p></details>

**Bài 4.** Đồng nghiệp chạy `SELECT customer_id FROM trips_by_customer PER PARTITION LIMIT 1;` và nói "Cassandra sắp khách theo thứ tự lạ, chắc lỗi". Giải thích.

<details><summary><b>Lời giải</b></summary><p>Kết quả ra theo thứ tự token: khách 1 (−2983185843790857942), khách 3 (144159704313836052), khách 2 (7165355148280075549). Partition rải theo giá trị băm Murmur3 của <code>customer_id</code>, không theo giá trị gốc. Đó là thiết kế, không phải lỗi.</p></details>

## 9. Đọc tiếp

- [Data Definition (DDL)](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/ddl.html) — `CREATE KEYSPACE`, `CREATE TABLE`, khoá chính, `CLUSTERING ORDER BY`.
- [Data modeling: introduction](https://cassandra.apache.org/doc/5.0/cassandra/developing/data-modeling/intro.html) — partition và mô hình hoá theo truy vấn.
- [Dynamo: consistent hashing using a token ring](https://cassandra.apache.org/doc/5.0/cassandra/architecture/dynamo.html) — partitioner và token.
- [PG-003](/articles/postgresql-bang-dong-cot-primary-foreign-key): khoá chính và khoá ngoại trong PostgreSQL.

Bài tiếp theo: [CS-004 · Kiểu dữ liệu CQL cho tiền, thời gian, chuỗi và định danh](/articles/cassandra-kieu-du-lieu-cql).
