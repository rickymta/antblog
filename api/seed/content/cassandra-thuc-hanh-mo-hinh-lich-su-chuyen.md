---
slug: cassandra-thuc-hanh-mo-hinh-lich-su-chuyen
title: "Thực hành Cassandra: mô hình lịch sử chuyến và dòng thời gian sự kiện của Tới Nơi"
summary: "Bài thực hành tổng kết Phần 1: bàn giao keyspace, năm bảng, fixture 12 chuyến, truy vấn Q1–Q5 và bộ kiểm tra K1–K10 chạy thật trên Cassandra 5.0.9 (20 PASS, 0 FAIL), kèm tiêu chí chấm, hai bài mở rộng và những gì Phần 1 chưa làm."
category: cassandra
level: advanced
tags: [cassandra, cs-010, cassandra-5, cql, toi-noi, thuc-hanh, data-modeling, fixture, kiem-tra]
language: vi
---

Đội backend Tới Nơi chuẩn bị chuyển lịch sử chuyến và dòng thời gian sự kiện ra khỏi PostgreSQL, vì hai bảng này chỉ thêm dòng, đọc theo khoá và tăng ~16 triệu dòng mỗi ngày. Trước khi viết một dòng code ứng dụng, trưởng nhóm yêu cầu một bản bàn giao mô hình dữ liệu: schema chạy được, dữ liệu mẫu, các truy vấn mà app sẽ gọi, và **bằng chứng** rằng mỗi truy vấn đọc đúng một partition, đúng thứ tự, còn truy vấn sai thì bị chặn.

Đây là bài thực hành cuối Phần 1. Mọi thứ cần bàn giao đã nằm trong lab `p01`; bài gom chúng lại thành một gói có tiêu chí chấm rõ ràng, để bạn tự làm lại từ đầu hoặc dùng làm mẫu cho mô hình của mình.

<div class="tkh-callout">
<span class="tkh-callout-title">CS-010 · Phần 1/6 · Nâng cao · Thực hành</span>
<p>Đọc trước: CS-001 đến CS-009, đặc biệt <a href="/articles/cassandra-thiet-ke-theo-truy-van">CS-007</a>, <a href="/articles/cassandra-phi-chuan-hoa-nhieu-bang">CS-008</a> và <a href="/articles/cassandra-partition-size-bucketing">CS-009</a>. Lab: toàn bộ <code>examples/toi-noi-cassandra/p01</code> (file <code>00</code> → <code>09</code>), kết quả thật trong <code>p01/OUTPUT.txt</code>. Sau bài này, bạn bàn giao được một mô hình Cassandra gồm CQL, fixture, truy vấn kiểm tra và bằng chứng chạy, và tự chấm nó theo tiêu chí.</p>
</div>

## 1. Đề bài

Thiết kế và bàn giao phần dữ liệu **lịch sử chuyến** và **dòng thời gian sự kiện** của Tới Nơi trên Apache Cassandra 5.0, phục vụ năm access pattern:

| Mã | Màn hình | Câu hỏi | Thứ tự |
|---|---|---|---|
| Q1 | Chi tiết chuyến | Một chuyến theo `trip_id` | một dòng |
| Q2 | App khách: "Chuyến của tôi" | Chuyến của một khách | mới nhất trước |
| Q3 | App tài xế: "Thu nhập hôm nay" | Chuyến của một tài xế trong một ngày (giờ Việt Nam), cùng số chuyến và tổng cước | mới nhất trước |
| Q4 | Điều phối | Chuyến mới tạo trong một thành phố, theo giờ | cũ nhất trước |
| Q5 | Chi tiết chuyến | Dòng thời gian sự kiện của một chuyến | cũ nhất trước |

Ràng buộc:

- Mỗi truy vấn của ứng dụng đọc **đúng một partition**, không `ALLOW FILTERING`.
- Tiền là số nguyên đồng (`bigint`), thời điểm là `timestamp` (CS-004).
- Partition có giới hạn kích thước ở tải của hồ sơ Tới Nơi (CS-009).
- Các bản sao của một chuyến phải ghi cùng nhau và đối soát được (CS-008).
- Dữ liệu mẫu tái lập được: UUID cố định, địa điểm hư cấu hoặc địa danh công cộng.

## 2. Gói bàn giao

| Thành phần | File | Nội dung |
|---|---|---|
| Keyspace | `00-reset.cql` | `toinoi`, NetworkTopologyStrategy `{'dc1': 1}`; in phiên bản cluster |
| Schema | `01-schema.cql` | Năm bảng Q1–Q5, mỗi bảng có `comment` ghi mã truy vấn |
| Fixture | `02-fixture.cql` | 12 chuyến của 3 khách, 3 tài xế, 2 thành phố (TP.HCM, Hà Nội), ngày 02–03/11/2026; mỗi chuyến ghi vào 4 bảng trong một logged batch; 79 sự kiện |
| Truy vấn nghiệp vụ | `06-truy-van-nghiep-vu.cql` | Mỗi màn hình một truy vấn, cộng hai truy vấn "không được phép" |
| Đường ghi | `07-phi-chuan-hoa.cql` | Đổi trạng thái bằng logged batch, sửa lệch, đối soát |
| Kích thước | `08-partition-size.sh` | Bucket ngày và bucket giờ, kết quả đo bằng `nodetool` |
| Bộ kiểm tra | `09-kiem-tra.cql` | K1–K10 |
| Bằng chứng | `OUTPUT.txt` | Kết quả thật trên Cassandra 5.0.9 |

### Schema

| Bảng | Khoá chính | Thứ tự clustering | Phục vụ |
|---|---|---|---|
| `trips` | `(trip_id)` | — | Q1, nguồn gốc khoá cho các bản sao |
| `trips_by_customer` | `((customer_id), requested_at, trip_id)` | `requested_at DESC, trip_id ASC` | Q2 |
| `trips_by_driver_day` | `((driver_id, day), requested_at, trip_id)` | `requested_at DESC, trip_id ASC` | Q3 |
| `trips_by_city_hour` | `((city_id, hour_bucket), requested_at, trip_id)` | `requested_at ASC, trip_id ASC` | Q4 |
| `trip_events_by_trip` | `((trip_id), event_at, event_seq)` | `event_at ASC, event_seq ASC` | Q5 |

Lý do chọn từng khoá nằm ở CS-007; lý do bucket giờ cho Q4 nằm ở CS-009 (partition ngày của TP.HCM ước lượng khoảng 54 MB với cột hẹp, partition giờ cao điểm khoảng 6,8 MB).

### Fixture

Fixture đếm lại số dòng mỗi bảng ngay sau khi nạp:

```text
 bang                | so_dong
---------------------+---------
 trips               |      12
 trips_by_customer   |      12
 trips_by_driver_day |      12
 trips_by_city_hour  |      12
 trip_events_by_trip |      79
```

(Kết quả thật in thành năm khối, mỗi khối kèm cảnh báo `Aggregation query used without partition key`; gộp lại ở đây.) Bốn bảng chuyến cùng 12 dòng là dấu hiệu đầu tiên rằng logged batch của fixture đã ghi đủ bản sao. Fixture cố ý có một chuyến bị huỷ (chuyến 5, cước 0) và một chuyến đang giao (chuyến 12, `in_transit`) để CS-008 có trạng thái để đổi.

Duyệt bảng sự kiện bằng "Mở dữ liệu" trong DBStudio:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-010/01-du-lieu-dong-thoi-gian.webp" alt="Tab trip_events_by_trip của DBStudio, mục Dữ liệu: cây bên trái mở keyspace toinoi, nhóm Bảng có sáu bảng; lưới dữ liệu có các cột trip_id, event_at, event_seq, actor, note; bảy dòng đầu của chuyến 07000000-…-000000000001 với event_seq 1 đến 7, tiếp theo là chuyến …010 rồi chuyến …002; note đều NULL; thanh dưới ghi Đã tải 80 dòng" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 1 · Duyệt bảng sự kiện.</b> Trong mỗi chuyến, sự kiện xếp theo <code>event_at</code> rồi <code>event_seq</code>; còn giữa các chuyến, thứ tự là thứ tự token của <code>trip_id</code> (chuyến 1, rồi 10, rồi 2), không theo mã chuyến hay thời gian. "Đã tải 80 dòng" là 79 sự kiện của fixture cộng sự kiện <code>completed</code> của chuyến 12 thêm ở CS-008. Duyệt toàn bảng như thế này chỉ dùng để xem dữ liệu lab, không phải truy vấn của ứng dụng.</figcaption>
</figure>

### Chạy lại từ đầu

```sh
cd examples/toi-noi-cassandra
npm run db:up      # chờ node1 healthy
npm run p01        # 00 → 09 trên keyspace toinoi mới
npm test           # chạy lại p01 và so từng dòng với p01/OUTPUT.txt
```

`npm test` bỏ qua những gì phụ thuộc máy và thời điểm chạy: thời gian của `COPY FROM`, độ trễ và ước lượng kích thước của `nodetool`.

## 3. Bộ kiểm tra K1–K10 và kết quả thật

Bộ kiểm tra có hai loại. Loại **dương** chạy truy vấn của ứng dụng và so kết quả. Loại **âm** chạy truy vấn mà mô hình *phải từ chối*; dòng chú thích `-- EXPECT: <chuỗi>` ngay trước câu lệnh khai báo thông điệp lỗi mong đợi, và runner chấm `PASS` khi câu lệnh lỗi đúng thông điệp đó.

| Mã | Kiểm điều gì | Kết quả thật |
|---|---|---|
| K1 | Q1 trả đúng một chuyến theo mã (chuyến 5) | 1 dòng: `cancelled`, cước `0` |
| K2 | Q2 của khách 3, mới nhất trước | 3 dòng: 03/11 11:35 ride 57.000 → 02/11 18:40 parcel 28.000 → 02/11 12:10 ride 64.000 |
| K3 | Q3 của tài xế 1 ngày 03/11, số chuyến và tổng cước | `2` chuyến, `197000` |
| K4 | Q4 của Hà Nội, giờ 11:00 ngày 03/11 | 1 dòng: 11:35 ride 57.000 |
| K5 | Q5 của chuyến huỷ: đúng ba sự kiện, kết thúc bằng `cancelled` | `requested` (customer) → `searching` (system) → `cancelled` (customer) |
| K6 | Chuyến 12 đã `completed` ở cả bốn bảng sau CS-008 | 4 lần `completed` |
| K7 | Đối soát: số chuyến và tổng cước bốn bảng bằng nhau | 4 lần `12 | 813000` |
| K8 | Truy vấn không có partition key bị từ chối (3 câu) | 3 PASS: `Cannot execute this query as it might involve data filtering…` |
| K9 | Thứ tự chỉ đảo được theo clustering đã khai báo | PASS: `Order by is currently only supported on the clustered columns of the PRIMARY KEY, got fare_vnd` |
| K10 | Khoá chính không đổi được bằng UPDATE | PASS: `PRIMARY KEY part requested_at found in SET part` |

Ba câu của K8 đáng đọc kỹ, vì mỗi câu tương ứng một sai lầm thiết kế hay gặp:

```sql
SELECT * FROM trips_by_customer WHERE status = 'completed';                         -- lọc cột thường
SELECT * FROM trips_by_driver_day WHERE driver_id = 0d000000-0000-4000-8000-000000000001; -- thiếu một phần partition key (day)
SELECT * FROM trips WHERE customer_id = 0c000000-0000-4000-8000-000000000001;       -- hỏi bảng sai (anti-pattern CS-007)
```

K5 trong DBStudio:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-010/02-dong-thoi-gian-chuyen-huy.webp" alt="Query editor DBStudio chạy SELECT event_at, event_seq, status, actor FROM trip_events_by_trip WHERE trip_id = 07000000-0000-4000-8000-000000000005; lưới kết quả 3 dòng: 2026-11-02T10:31:00.000Z 1 requested customer, 2026-11-02T10:31:17.000Z 2 searching system, 2026-11-02T10:32:51.000Z 3 cancelled customer; thanh dưới ghi 3 dòng và Chỉ đọc: Thiếu cột khoá trip_id trong SELECT" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 2 · Dòng thời gian của chuyến bị huỷ (K5).</b> Ba sự kiện trong một partition, cũ nhất trước: khách đặt lúc 17:31:00 giờ Việt Nam (<code>10:31Z</code> trên lưới UTC), hệ thống tìm tài xế, khách huỷ sau 1 phút 51 giây. Không có <code>accepted</code> nên chuyến chưa có tài xế nhận, khớp với cước 0 ở K1.</figcaption>
</figure>

### Bảng tổng của cả lab

Runner đếm `PASS`/`FAIL` của mọi câu có `-- EXPECT` trong cả mười file, và mọi lỗi *không* khai báo đều tính là `FAIL`. Cuối `OUTPUT.txt`:

```text
-- [lab] File đã chạy: 10
-- [lab] PASS: 20
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT
```

20 PASS phân theo file: `03-kieu-du-lieu` 3, `04-upsert` 3, `05-select` 5, `06-truy-van-nghiep-vu` 2, `07-phi-chuan-hoa` 2, `09-kiem-tra` 5. Tất cả là lỗi mong đợi: mô hình không chỉ trả lời đúng câu hỏi được thiết kế, mà còn từ chối đúng những câu hỏi nó không được thiết kế để trả lời.

## 4. Tiêu chí chấm

Dùng bảng này để tự chấm bản bàn giao của bạn (hoặc của đồng nghiệp). Đạt cần mọi dòng "bắt buộc".

| Tiêu chí | Cách kiểm | Mức |
|---|---|---|
| Mỗi access pattern có đúng một bảng, tên `<thứ>_by_<khoá>`, `comment` ghi mã truy vấn | Đọc `01-schema.cql` | Bắt buộc |
| Mỗi truy vấn ứng dụng nêu đủ partition key, không `ALLOW FILTERING` | Đọc `06`, `09`; K1–K5 chạy không cảnh báo | Bắt buộc |
| Thứ tự đúng nằm trong `CLUSTERING ORDER BY`, không cần `ORDER BY` ở truy vấn | K2, K5 | Bắt buộc |
| Khoá chính duy nhất: có cột phá hoà (`trip_id`, `event_seq`) | Đọc schema; giải thích được vì sao | Bắt buộc |
| Tiền `bigint`, thời điểm `timestamp`, ngày nghiệp vụ tính theo giờ Việt Nam | Đọc schema, fixture | Bắt buộc |
| Bản sao ghi cùng nhau bằng logged batch; có truy vấn đối soát | `02`, `07`; K6, K7 | Bắt buộc |
| Truy vấn sai bị từ chối đúng thông điệp | K8–K10 | Bắt buộc |
| Ước lượng kích thước partition lớn nhất có phép tính minh bạch | Bảng tính kiểu CS-009 | Bắt buộc |
| `npm test` xanh, `OUTPUT.txt` cập nhật | Chạy lại | Bắt buộc |
| Có bảng cho access pattern mở rộng (mục 5) | Thêm file, thêm kiểm tra | Điểm cộng |

## 5. Mở rộng

**Mở rộng 1 · Chuyến bị huỷ trong một giờ ở một thành phố.** Đây là bài tập của CS-007: bảng `cancelled_trips_by_city_hour` với khoá `((city_id, hour_bucket), cancelled_at, trip_id)`, bucket theo **giờ huỷ**. Phần còn lại của việc bàn giao: thêm dòng vào bảng này trong cùng logged batch khi chuyến chuyển sang `cancelled` (sửa batch của chuyến 5 trong fixture), thêm một kiểm tra dương ("giờ 17:00 ngày 02/11 ở TP.HCM có đúng một chuyến huỷ") và mở rộng K7 thành đối soát năm bảng.

<details><summary><b>Gợi ý kiểm tra</b></summary><p>Chuyến 5 huỷ lúc 17:32:51 (sự kiện thứ 3), nên <code>hour_bucket = '2026-11-02 17:00:00+0700'</code>. Kiểm tra dương: <code>SELECT trip_id, cancelled_at FROM cancelled_trips_by_city_hour WHERE city_id = 1 AND hour_bucket = '2026-11-02 17:00:00+0700';</code> phải trả đúng một dòng chuyến 5. Đối soát: số dòng của bảng mới phải bằng số chuyến <code>cancelled</code> trong <code>trips</code>, tức 1 với fixture.</p></details>

**Mở rộng 2 · Thu nhập của tài xế theo tuần.** App tài xế thêm màn hình "Tuần này". Có hai lựa chọn: đọc 7 partition của `trips_by_driver_day` (7 câu song song, cộng ở ứng dụng), hoặc thêm bảng `trips_by_driver_week` với khoá `((driver_id, week_start), requested_at, trip_id)`.

<details><summary><b>Gợi ý chọn</b></summary><p>Một tài xế chạy vài chục chuyến mỗi ngày, nên partition tuần vẫn nhỏ (vài trăm dòng). Bảng tuần cho màn hình một lần đọc, đổi lại thêm một bản sao phải ghi trong batch và đối soát. Đọc 7 partition ngày không thêm bản sao nào, chỉ thêm 6 câu đọc nhỏ. Nếu màn hình tuần mở ít, chọn cách đọc 7 ngày; nếu nó là màn hình chính, chọn bảng tuần. Dù chọn cách nào, <code>week_start</code> phải tính theo giờ Việt Nam và thống nhất ngày bắt đầu tuần (thứ Hai).</p></details>

## 6. Những gì Phần 1 chưa làm

Bản bàn giao này đủ để chốt **mô hình dữ liệu**, chưa đủ để chạy production. Các giới hạn cần ghi rõ trong bản bàn giao:

- **Một node, RF = 1.** Keyspace `toinoi` dùng `{'dc1': 1}`: mỗi dòng chỉ có một bản, node chết là mất. Cụm 3 node, replication factor và consistency level là Phần 3.
- **Consistency level mặc định của cqlsh** (`ONE`). Với một node, mọi mức đều như nhau; câu hỏi "đọc có thấy bản ghi vừa ghi không" chỉ có nghĩa khi có nhiều replica (CS-024).
- **Chưa đo hiệu năng.** Số duy nhất của P01 là ước lượng kích thước partition của `nodetool`, không phải độ trễ hay thông lượng. Đo tải có điều kiện ghi rõ là CS-038.
- **Đối soát bằng quét toàn bảng.** `COUNT(*)` không có partition key chỉ chấp nhận được với 12 chuyến; dữ liệu thật cần quét theo token range (CS-036).
- **Chưa có xác thực, TLS, backup.** Lab chạy không mật khẩu (Phần 5).
- **Tranh chấp trạng thái chưa xử lý.** Hai tài xế cùng nhận một chuyến cần lightweight transaction (Phần 4); P01 chỉ đổi trạng thái tuần tự.

## 7. Dẫn sang Phần 2

Phần 1 chỉ dùng các kiểu cột đơn giản và câu ghi cơ bản. Phần 2 đưa thêm các công cụ CQL cho nghiệp vụ: collection (nhãn, ảnh của chuyến), user-defined type cho địa chỉ đón/trả, TTL cho dữ liệu tạm, counter, static column, batch logged/unlogged chi tiết hơn, SAI và materialized view. Sản phẩm cuối Phần 2 là chat tài xế–khách và hộp thư thông báo, xây trên cùng quy trình query-first của bài này.

## 8. Đọc tiếp

- [Data modeling](https://cassandra.apache.org/doc/5.0/cassandra/developing/data-modeling/intro.html) — toàn bộ quy trình mô hình hoá trong tài liệu Cassandra 5.0.
- [CQL: Data definition](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/ddl.html) — `CREATE TABLE`, `PRIMARY KEY`, `CLUSTERING ORDER BY`.
- [PG-010 · Thực hành schema chuyến xe](/articles/postgresql-thuc-hanh-schema-chuyen-xe): cùng bài toán bàn giao, giải bằng PostgreSQL.
- `examples/toi-noi-cassandra/README.md` trong repo: cách chạy lab, cổng, kết nối DBStudio.

Bài tiếp theo: CS-011 · Collection: set, list, map — Phần 2 (đang viết).
