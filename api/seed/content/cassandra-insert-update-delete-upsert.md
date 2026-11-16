---
slug: cassandra-insert-update-delete-upsert
title: "INSERT, UPDATE, DELETE trong Cassandra: mọi câu ghi chuyến xe đều là upsert"
summary: "Ứng dụng Tới Nơi gửi lại cùng một chuyến hai lần mà Cassandra không báo trùng khoá; câu huỷ chuyến đến sau lại không có tác dụng. Bài này giải thích upsert, WRITETIME của từng ô, last-write-wins với USING TIMESTAMP, tombstone khi ghi null hoặc DELETE, và IF NOT EXISTS, với kết quả thật từ lab Cassandra 5.0.9."
category: cassandra
level: intermediate
tags: [cassandra, cs-005, cassandra-5, cql, toi-noi, upsert, tombstone, last-write-wins, writetime]
language: vi
---

Lúc 07:40 sáng, mạng 4G của khách chập chờn, ứng dụng Tới Nơi gửi lệnh tạo chuyến hai lần. Với PostgreSQL, lần thứ hai gặp lỗi trùng khoá chính và ứng dụng biết ngay. Với Cassandra, cả hai lần đều "thành công", và dòng trong bảng mang giá trị của lần ghi có **timestamp lớn hơn**, không nhất thiết là lần đến sau. Một câu huỷ chuyến đến sau cùng vẫn có thể bị bỏ qua.

Hành vi đó không phải lỗi. Đó là mô hình ghi của Cassandra: mọi câu ghi chỉ thêm một phiên bản mới cho từng ô dữ liệu, và khi đọc, phiên bản mới nhất theo timestamp thắng. Hiểu điều này trước khi viết dòng code ghi đầu tiên sẽ tránh được nhiều sự cố khó điều tra.

<div class="tkh-callout">
<span class="tkh-callout-title">CS-005 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/cassandra-keyspace-bang-khoa-chinh">CS-003 · Keyspace, bảng và khoá chính</a>, <a href="/articles/cassandra-kieu-du-lieu-cql">CS-004 · Kiểu dữ liệu CQL</a>. Lab: <code>examples/toi-noi-cassandra/p01/04-upsert.cql</code>, kết quả ở <code>p01/OUTPUT.txt</code>. Sau bài này, bạn giải thích được upsert và last-write-wins, đọc được <code>WRITETIME</code> của từng ô, biết khi nào câu ghi sinh tombstone, và dùng đúng <code>IF NOT EXISTS</code>/<code>IF</code>.</p>
</div>

## 1. Dòng là tập các ô, mỗi ô có timestamp riêng

Trong Cassandra, một dòng không phải bản ghi nguyên khối. Mỗi cột không thuộc khoá chính của dòng là một **ô** (*cell*) gồm giá trị và **timestamp ghi** (*write timestamp*), tính bằng micro giây kể từ 1970. Timestamp này thường do driver hoặc node điều phối (coordinator) gán lúc ghi; câu lệnh cũng có thể tự đặt bằng `USING TIMESTAMP`.

Khi đọc, Cassandra gom mọi phiên bản của từng ô (có thể nằm ở memtable và nhiều SSTable) rồi giữ phiên bản có **timestamp lớn nhất**. Quy tắc này gọi là **last-write-wins** (LWW). Nó áp dụng cho từng ô, không phải cho cả dòng.

Lab dùng chuyến nháp `07000000-0000-4000-8000-000000000099` trong bảng `trips` và đặt `USING TIMESTAMP` cố định cho mọi câu ghi để kết quả tái lập được. `1793580000000000` micro giây là 07:40:00 sáng 02/11/2026 giờ Việt Nam; mỗi bước sau cộng thêm một giây.

## 2. Upsert: INSERT trùng khoá không lỗi

**Upsert** là ghi mà không quan tâm dòng đã tồn tại hay chưa: chưa có thì tạo, có rồi thì ghi đè. Trong Cassandra, `INSERT` và `UPDATE` đều là upsert.

```sql
INSERT INTO trips (trip_id, service, city_id, status, fare_vnd)
VALUES (07000000-0000-4000-8000-000000000099, 'ride', 1, 'requested', 50000)
USING TIMESTAMP 1793580000000000;
INSERT INTO trips (trip_id, service, city_id, status, fare_vnd)
VALUES (07000000-0000-4000-8000-000000000099, 'ride', 1, 'searching', 52000)
USING TIMESTAMP 1793580001000000;
SELECT trip_id, status, fare_vnd, WRITETIME(status) AS wt_status FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;
```

```text
 trip_id                              | status    | fare_vnd | wt_status
--------------------------------------+-----------+----------+------------------
 07000000-0000-4000-8000-000000000099 | searching |    52000 | 1793580001000000
```

Không có lỗi trùng khoá chính. Cassandra không đọc trước khi ghi nên không biết dòng đã có; nhờ vậy câu ghi rất rẻ, nhưng database cũng không chặn hộ bạn việc ghi trùng.

### INSERT chỉ ghi các cột được nêu

```sql
INSERT INTO trips (trip_id, status)
VALUES (07000000-0000-4000-8000-000000000099, 'accepted')
USING TIMESTAMP 1793580002000000;
SELECT trip_id, service, status, fare_vnd FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;
```

```text
 trip_id                              | service | status   | fare_vnd
--------------------------------------+---------+----------+----------
 07000000-0000-4000-8000-000000000099 |    ride | accepted |    52000
```

`INSERT` thứ ba chỉ nêu `status`, nên `service` và `fare_vnd` giữ nguyên. Đây không phải "thay cả dòng" như nhiều người đoán. Đọc `WRITETIME` của từng ô sẽ thấy rõ:

```sql
SELECT WRITETIME(service) AS wt_service, WRITETIME(fare_vnd) AS wt_fare, WRITETIME(status) AS wt_status
FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;
```

```text
 wt_service       | wt_fare          | wt_status
------------------+------------------+------------------
 1793580001000000 | 1793580001000000 | 1793580002000000
```

Ba ô của cùng một dòng mang ba thời điểm ghi khác nhau. `WRITETIME()` chỉ dùng được với cột thường; gọi trên cột khoá chính bị từ chối (`Cannot use selection function writetime on PRIMARY KEY part trip_id`, câu chỉ-đọc chạy thêm trên lab).

Trên dữ liệu fixture, DBStudio cho thấy cùng hiện tượng ở hai chuyến bị sửa ở bài CS-008:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-005/01-writetime-tung-o.webp" alt="DBStudio chạy SELECT trip_id, status, fare_vnd, WRITETIME(status) AS wt_status, WRITETIME(fare_vnd) AS wt_fare FROM trips WHERE trip_id IN (chuyến 11, chuyến 12). Lưới kết quả: chuyến 11 status completed, fare_vnd 95000, wt_status bắt đầu bằng 179484687997, wt_fare bắt đầu bằng 1794846888; chuyến 12 status completed, fare_vnd 118000, wt_status bắt đầu bằng 179484688844, wt_fare bắt đầu bằng 1794846879. Các số WRITETIME bị cắt bớt vì cột hẹp" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 1 · WRITETIME của từng ô.</b> Chuyến 11: cước được sửa thành 95000 ở CS-008 nên <code>wt_fare</code> lớn hơn <code>wt_status</code>. Chuyến 12: trạng thái được chuyển sang <code>completed</code> ở CS-008 nên ngược lại, <code>wt_status</code> lớn hơn. Cùng một dòng, mỗi ô có lịch sử ghi riêng.</figcaption>
</figure>

Lưới DBStudio cắt bớt số dài. Giá trị đầy đủ, đọc bằng cqlsh trên lab sau `npm run p01` (con số phụ thuộc thời điểm bạn chạy lab):

```text
 trip_id                              | status    | fare_vnd | wt_status        | wt_fare
--------------------------------------+-----------+----------+------------------+------------------
 07000000-0000-4000-8000-000000000011 | completed |    95000 | 1794846879978053 | 1794846888468145
 07000000-0000-4000-8000-000000000012 | completed |   118000 | 1794846888449323 | 1794846879988185
```

## 3. UPDATE cũng tạo dòng

```sql
UPDATE trips USING TIMESTAMP 1793580003000000
SET status = 'requested', fare_vnd = 40000
WHERE trip_id = 07000000-0000-4000-8000-000000000098;
SELECT trip_id, status, fare_vnd FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000098;
```

```text
 trip_id                              | status    | fare_vnd
--------------------------------------+-----------+----------
 07000000-0000-4000-8000-000000000098 | requested |    40000
```

Chuyến `…098` chưa từng tồn tại, vậy mà `UPDATE` tạo ra nó. Nếu ứng dụng muốn "chỉ sửa khi chuyến đã có", phải dùng `IF EXISTS` (mục 6) hoặc tự đảm bảo bằng thiết kế. Một lỗi thường gặp là `UPDATE` với mã chuyến gõ sai: database không báo gì, chỉ sinh thêm một dòng rác.

## 4. Last-write-wins và USING TIMESTAMP

Câu sau đến **sau** về mặt thời gian thực, nhưng mang timestamp **cũ hơn** ô `status` hiện có (`…9999000000` nhỏ hơn `…0002000000`):

```sql
UPDATE trips USING TIMESTAMP 1793579999000000
SET status = 'cancelled'
WHERE trip_id = 07000000-0000-4000-8000-000000000099;
SELECT trip_id, status, WRITETIME(status) AS wt_status FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;
```

```text
 trip_id                              | status   | wt_status
--------------------------------------+----------+------------------
 07000000-0000-4000-8000-000000000099 | accepted | 1793580002000000
```

Câu huỷ chuyến "thành công" nhưng không có tác dụng: `status` vẫn là `accepted`. Trong thực tế bạn hiếm khi tự viết `USING TIMESTAMP`; tình huống tương tự xảy ra khi **đồng hồ** của hai máy ứng dụng (hoặc hai node điều phối) lệch nhau, và máy chạy chậm giờ gửi câu ghi mới nhất. Khi hai lần ghi có timestamp bằng nhau, Cassandra vẫn chọn một bên theo quy tắc cố định (xoá thắng ghi; giữa hai giá trị, giá trị lớn hơn theo byte thắng), không phải theo thứ tự đến.

Hệ quả cho Tới Nơi: chuyển trạng thái chuyến (`requested` → `accepted` → `cancelled`) **không** nên dựa vào "ai ghi sau". Bài CS-032 tái hiện lỗi lệch đồng hồ; chuyển trạng thái có điều kiện dùng lightweight transaction (mục 6, CS-034).

## 5. Tombstone: ghi null và DELETE

**Tombstone** là một bản ghi đánh dấu "giá trị này đã bị xoá", mang timestamp như mọi lần ghi khác. Cassandra không xoá dữ liệu tại chỗ (SSTable là file bất biến); nó ghi tombstone, và khi đọc, tombstone có timestamp lớn hơn sẽ che các phiên bản cũ. Tombstone chỉ thật sự biến mất sau `gc_grace_seconds` (mặc định 864000 giây, tức 10 ngày) và một lần compaction.

### Ghi null là xoá ô

```sql
INSERT INTO trips (trip_id, pickup, dropoff)
VALUES (07000000-0000-4000-8000-000000000099, 'Chợ Bến Thành, Q1', null)
USING TIMESTAMP 1793580004000000;
SELECT trip_id, pickup, dropoff, WRITETIME(dropoff) AS wt_dropoff FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000099;
```

```text
 trip_id                              | pickup            | dropoff | wt_dropoff
--------------------------------------+-------------------+---------+------------
 07000000-0000-4000-8000-000000000099 | Chợ Bến Thành, Q1 |    null |       null
```

`dropoff = null` không phải "để trống": Cassandra ghi một tombstone cho ô đó, và `WRITETIME(dropoff)` trả `null` vì ô không còn giá trị sống. Một bảng mà ứng dụng luôn gửi đủ mọi cột, kể cả cột `null`, sẽ sinh tombstone ở mỗi lần ghi. Với prepared statement, hãy **không gán** (*unset*) tham số thay vì gán `null` khi không muốn đổi cột đó (CS-037).

### DELETE cột, DELETE dòng

```sql
DELETE fare_vnd FROM trips USING TIMESTAMP 1793580005000000 WHERE trip_id = 07000000-0000-4000-8000-000000000098;
SELECT trip_id, status, fare_vnd FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000098;
DELETE FROM trips USING TIMESTAMP 1793580006000000 WHERE trip_id = 07000000-0000-4000-8000-000000000098;
SELECT COUNT(*) AS con_lai FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000098;
```

```text
 trip_id                              | status    | fare_vnd
--------------------------------------+-----------+----------
 07000000-0000-4000-8000-000000000098 | requested |     null

 con_lai
---------
       0
```

`DELETE fare_vnd` sinh tombstone cho một ô; `DELETE FROM … WHERE trip_id = …` sinh tombstone cho cả partition (ở bảng `trips`, mỗi partition là một chuyến).

### Tombstone thắng câu ghi cũ hơn

```sql
INSERT INTO trips (trip_id, status) VALUES (07000000-0000-4000-8000-000000000098, 'requested') USING TIMESTAMP 1793580005500000;
SELECT COUNT(*) AS con_lai FROM trips WHERE trip_id = 07000000-0000-4000-8000-000000000098;
```

```text
 con_lai
---------
       0
```

Câu `INSERT` đến sau lần xoá, nhưng timestamp của nó (`…5500000`) nhỏ hơn timestamp của tombstone (`…6000000`), nên dữ liệu không "sống lại". Chiều ngược lại mới nguy hiểm: nếu tombstone đã bị dọn (sau `gc_grace_seconds`) trong khi một bản sao cũ chưa nhận được nó, dữ liệu đã xoá có thể xuất hiện lại. Đó là chủ đề của CS-031 và CS-044 (repair).

### DELETE theo dải clustering

Với bảng có cột clustering, có thể xoá một dải dòng trong partition. Lab ghi ba sự kiện cho chuyến nháp rồi xoá các sự kiện sau 09:00:30:

```sql
DELETE FROM trip_events_by_trip
WHERE trip_id = 07000000-0000-4000-8000-000000000099 AND event_at > '2026-11-02 09:00:30+0700';
SELECT event_at, event_seq, status FROM trip_events_by_trip WHERE trip_id = 07000000-0000-4000-8000-000000000099;
```

```text
 event_at                        | event_seq | status
---------------------------------+-----------+-----------
 2026-11-02 09:00:00.000000+0700 |         1 | requested
```

Hai sự kiện `searching` và `accepted` bị che bởi một **range tombstone** duy nhất: một dấu xoá cho cả dải, không phải một tombstone cho mỗi dòng.

### DELETE phải chỉ rõ partition

```sql
DELETE FROM trips WHERE status = 'cancelled';
```

```text
PASS lỗi mong đợi (dòng 75): Some partition key parts are missing: trip_id
```

Không có "xoá mọi chuyến bị huỷ". `DELETE` (và `UPDATE`) cần đủ partition key, vì Cassandra phải biết ghi tombstone vào partition nào mà không cần đọc trước. Muốn dọn theo điều kiện, ứng dụng phải đọc ra danh sách khoá rồi xoá từng partition.

## 6. Không đổi được khoá chính, và ghi có điều kiện

### Không SET cột khoá chính

```sql
UPDATE trips SET trip_id = 07000000-0000-4000-8000-000000000097 WHERE trip_id = 07000000-0000-4000-8000-000000000099;
```

```text
PASS lỗi mong đợi (dòng 45): PRIMARY KEY part trip_id found in SET part
```

Khoá chính quyết định dòng nằm ở partition nào, node nào (CS-003). "Đổi khoá" thực chất là ghi một dòng mới ở chỗ khác và xoá dòng cũ. Điều này quan trọng với các bảng như `trips_by_customer`, nơi `requested_at` là cột clustering: sửa giờ đặt của một chuyến nghĩa là xoá rồi ghi lại (CS-008).

### IF NOT EXISTS và IF điều kiện

Khi thật sự cần "chỉ ghi nếu …", CQL có câu ghi có điều kiện, gọi là **lightweight transaction** (LWT). Cassandra chạy một vòng thoả thuận Paxos giữa các bản sao để đọc giá trị hiện tại và chỉ ghi khi điều kiện đúng. Kết quả trả về cột `[applied]`:

```sql
INSERT INTO trips (trip_id, status) VALUES (07000000-0000-4000-8000-000000000099, 'requested') IF NOT EXISTS;
UPDATE trips SET status = 'arriving' WHERE trip_id = 07000000-0000-4000-8000-000000000099 IF status = 'accepted';
UPDATE trips SET status = 'arriving' WHERE trip_id = 07000000-0000-4000-8000-000000000099 IF status = 'searching';
```

```text
 [applied] | trip_id                              | city_id | customer_id | driver_id | dropoff | fare_vnd | pickup            | requested_at | service | status
-----------+--------------------------------------+---------+-------------+-----------+---------+----------+-------------------+--------------+---------+----------
     False | 07000000-0000-4000-8000-000000000099 |       1 |        null |      null |    null |    52000 | Chợ Bến Thành, Q1 |         null |    ride | accepted

 [applied]
-----------
      True

 [applied] | status
-----------+----------
     False | arriving
```

- `IF NOT EXISTS` thất bại vì chuyến đã có, và Cassandra **trả kèm dòng hiện tại** để ứng dụng biết vì sao.
- `IF status = 'accepted'` đúng, nên trạng thái chuyển sang `arriving`.
- `IF status = 'searching'` sai; kết quả kèm giá trị hiện tại `arriving`.

DBStudio hiển thị cùng dạng kết quả khi chạy `IF NOT EXISTS` với chuyến 1 của fixture:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-005/02-if-not-exists.webp" alt="DBStudio chạy INSERT INTO trips (trip_id, status) VALUES (chuyến 07000000-0000-4000-8000-000000000001, 'requested') IF NOT EXISTS. Tab kết quả #1 INSERT có một dòng: cột [applied] = false, tiếp theo là trip_id 07000000-0000-4000-8000-000000000001, city_id 1, customer_id 0c000000-0000-4000-8000-000000000001 và driver_id của dòng đang có" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 2 · IF NOT EXISTS không áp dụng.</b> Câu <code>INSERT</code> có điều kiện trả về một dòng kết quả: <code>[applied] = false</code> cùng các cột của chuyến 1 đang có trong bảng. Dữ liệu không bị ghi đè, và ứng dụng đọc được ngay dòng đang có.</figcaption>
</figure>

LWT đắt hơn câu ghi thường nhiều lần (nhiều vòng trao đổi giữa các bản sao) và chỉ bảo đảm trong **một partition**. Dùng nó cho chỗ thật sự cần, như hai tài xế cùng bấm nhận một chuyến; chi tiết ở CS-034 và bài thực hành CS-040.

### Không trộn USING TIMESTAMP với LWT

```sql
UPDATE trips USING TIMESTAMP 1793580009000000 SET status = 'x'
WHERE trip_id = 07000000-0000-4000-8000-000000000099 IF EXISTS;
```

```text
PASS lỗi mong đợi (dòng 54): Cannot provide custom timestamp for conditional updates
```

LWT tự gán timestamp theo vòng Paxos để các câu có điều kiện được sắp đúng thứ tự; cho phép tự đặt timestamp sẽ phá bảo đảm đó. Một lưu ý đi kèm: trộn ghi LWT và ghi thường trên **cùng một ô** làm mất bảo đảm của LWT, vì câu ghi thường vẫn theo last-write-wins.

## 7. Lỗi thường gặp

- **Dựa vào lỗi trùng khoá để chống tạo chuyến hai lần.** Cassandra không có lỗi đó. Sinh `trip_id` ở client một lần và gửi lại cùng mã khi thử lại: ghi trùng chỉ là ghi đè cùng giá trị (CS-039 nói về idempotency).
- **`UPDATE` với khoá sai** tạo dòng mới thay vì báo "không tìm thấy".
- **Gửi `null` cho mọi cột không đổi.** Mỗi `null` là một tombstone; đọc partition nhiều tombstone sẽ chậm và có thể bị huỷ (CS-031).
- **Đổi trạng thái theo thứ tự đến.** Câu ghi có timestamp nhỏ hơn bị bỏ qua mà không báo lỗi; dùng LWT hoặc thiết kế ghi thêm sự kiện thay vì ghi đè.
- **Xoá theo điều kiện cột thường.** `DELETE` cần đủ partition key.

## 8. So với PostgreSQL

| Hành vi | PostgreSQL ([PG-006](/articles/postgresql-insert-update-delete-returning)) | Cassandra 5.0 |
|---|---|---|
| `INSERT` trùng khoá chính | Lỗi `duplicate key`, trừ khi viết `ON CONFLICT` | Không lỗi, ghi đè (upsert) |
| `UPDATE` dòng chưa có | 0 dòng bị ảnh hưởng | Tạo dòng mới |
| Biết dòng nào bị ảnh hưởng | `RETURNING`, số dòng | Không có `RETURNING`; câu ghi thường không trả gì. LWT trả `[applied]` và giá trị hiện tại |
| Thứ tự hai câu ghi | Khoá dòng, transaction quyết định | Timestamp lớn hơn thắng, từng ô |
| `DELETE` | Đánh dấu dòng chết, VACUUM dọn | Ghi tombstone, compaction dọn sau `gc_grace_seconds` |
| Xoá theo điều kiện bất kỳ | `DELETE … WHERE status = …` | Phải có partition key |

## 9. Bài tập

**Bài 1.** Một ô `status` có ba phiên bản: `accepted` (timestamp 100), `cancelled` (timestamp 90, đến sau cùng), tombstone (timestamp 95). Đọc ra giá trị gì?

<details><summary><b>Lời giải</b></summary><p><code>accepted</code>. Phiên bản có timestamp lớn nhất là 100. Tombstone 95 chỉ che được những phiên bản cũ hơn nó (ở đây là <code>cancelled</code> 90); thứ tự đến không quan trọng.</p></details>

**Bài 2.** Ứng dụng tạo chuyến bằng `INSERT INTO trips (trip_id, …, driver_id, pickup, dropoff, …) VALUES (…, null, …, null, …)` cho mọi cột chưa có giá trị. Có vấn đề gì? Sửa thế nào?

<details><summary><b>Lời giải</b></summary><p>Mỗi cột ghi <code>null</code> sinh một tombstone, nên mỗi chuyến mới đã mang sẵn vài tombstone. Sửa: chỉ liệt kê các cột có giá trị trong <code>INSERT</code>, hoặc để tham số ở trạng thái unset trong prepared statement.</p></details>

**Bài 3.** Khách đổi giờ đặt của một chuyến hẹn trước từ 07:42 sang 08:00. Bảng `trips_by_customer` có khoá `((customer_id), requested_at, trip_id)`. Viết các bước cập nhật.

<details><summary><b>Lời giải</b></summary><p><code>requested_at</code> là cột clustering nên không <code>SET</code> được. Phải <code>DELETE</code> dòng có <code>requested_at = 07:42</code> và <code>INSERT</code> dòng mới với <code>requested_at = 08:00</code> (cùng các cột khác), tốt nhất trong cùng một logged batch với các bảng còn lại (CS-008). Bảng <code>trips</code> có khoá <code>trip_id</code> thì chỉ cần <code>UPDATE … SET requested_at</code>.</p></details>

**Bài 4.** Vì sao `UPDATE … USING TIMESTAMP … IF EXISTS` bị từ chối?

<details><summary><b>Lời giải</b></summary><p>Câu có điều kiện là lightweight transaction; Cassandra tự gán timestamp theo vòng Paxos để các LWT trên cùng partition được sắp đúng thứ tự. Timestamp do người dùng đặt có thể phá thứ tự đó, nên Cassandra báo <code>Cannot provide custom timestamp for conditional updates</code>.</p></details>

## 10. Đọc tiếp

- [Data manipulation (INSERT, UPDATE, DELETE, BATCH)](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/dml.html): cú pháp, `USING TIMESTAMP`, câu ghi có điều kiện.
- [Storage engine](https://cassandra.apache.org/doc/5.0/cassandra/architecture/storage-engine.html): memtable, SSTable và vì sao xoá là ghi thêm.
- [PG-006](/articles/postgresql-insert-update-delete-returning): INSERT, UPDATE, DELETE và `RETURNING` ở PostgreSQL.

Bài tiếp theo: [CS-006 · SELECT theo khoá: WHERE, thứ tự clustering và phân trang](/articles/cassandra-select-where-clustering-order).
