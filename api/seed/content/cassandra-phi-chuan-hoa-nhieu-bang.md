---
slug: cassandra-phi-chuan-hoa-nhieu-bang
title: "Phi chuẩn hoá trong Cassandra: một chuyến Tới Nơi ở bốn bảng và cách giữ chúng khớp nhau"
summary: "Thiết kế theo truy vấn khiến một chuyến nằm ở bốn bảng. Bài chạy logged batch để đổi trạng thái cả bốn bản sao, đọc đúng bảo đảm và giới hạn của batchlog, tái hiện lỗi chỉ sửa một bảng làm cước lệch nhau, và đối soát bằng số dòng cùng tổng cước."
category: cassandra
level: advanced
tags: [cassandra, cs-008, cassandra-5, cql, toi-noi, denormalization, logged-batch, batchlog, doi-soat]
language: vi
---

18:58 ngày 03/11, tài xế 2 bấm "Đã giao" cho đơn đồ ăn chuyến 12. Trên PostgreSQL, việc này là một câu `UPDATE trips SET status = 'completed'`. Trên Cassandra, sau CS-007, chuyến 12 nằm ở **bốn bảng**: `trips` (Q1), `trips_by_customer` (Q2), `trips_by_driver_day` (Q3) và `trips_by_city_hour` (Q4), cộng một dòng mới trong `trip_events_by_trip` (Q5). Nếu chỉ một bảng được cập nhật, khách thấy chuyến đã giao trong lịch sử của mình còn màn hình thu nhập của tài xế vẫn ghi "đang giao".

Bài này trả lời ba câu hỏi: ghi vào nhiều bảng thế nào để không bỏ sót bảng nào, Cassandra bảo đảm được gì và *không* bảo đảm được gì, và làm sao phát hiện khi các bản sao đã lệch nhau.

<div class="tkh-callout">
<span class="tkh-callout-title">CS-008 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/cassandra-insert-update-delete-upsert">CS-005 · Mọi câu ghi là upsert</a> và <a href="/articles/cassandra-thiet-ke-theo-truy-van">CS-007 · Thiết kế bảng theo truy vấn</a>. Lab: <code>examples/toi-noi-cassandra/p01/07-phi-chuan-hoa.cql</code>; kết quả thật trong <code>p01/OUTPUT.txt</code>. Sau bài này, bạn ghi một thay đổi vào nhiều bảng bằng logged batch, nói đúng batch bảo đảm điều gì, biết vì sao phải có đủ khoá của từng bản sao, và viết được truy vấn đối soát.</p>
</div>

## 1. Một chuyến, bốn bản sao

**Phi chuẩn hoá** (*denormalization*) là cố ý giữ một dữ kiện ở nhiều chỗ để đọc nhanh hơn. Ở database quan hệ, đây là một lựa chọn tối ưu thêm; ở Cassandra, nó là hệ quả trực tiếp của nguyên tắc "một truy vấn một bảng" vì không có JOIN. Mỗi bản sao của chuyến 12 được định vị bằng một khoá khác nhau:

| Bảng | Khoá chính | Cần biết gì để chạm đúng dòng của chuyến 12 |
|---|---|---|
| `trips` | `(trip_id)` | `trip_id` |
| `trips_by_customer` | `((customer_id), requested_at, trip_id)` | `customer_id`, `requested_at`, `trip_id` |
| `trips_by_driver_day` | `((driver_id, day), requested_at, trip_id)` | `driver_id`, `day`, `requested_at`, `trip_id` |
| `trips_by_city_hour` | `((city_id, hour_bucket), requested_at, trip_id)` | `city_id`, `hour_bucket`, `requested_at`, `trip_id` |
| `trip_events_by_trip` | `((trip_id), event_at, event_seq)` | chỉ thêm dòng, không sửa |

Cột cuối là điều kiện để giữ các bản sao khớp nhau: **ứng dụng phải biết đủ khoá của mọi bản sao**. Vì vậy `trips` giữ cả `customer_id`, `driver_id`, `city_id`, `requested_at`: nó vừa trả lời Q1, vừa là chỗ tra ra khoá của ba bản sao còn lại. Đây cũng là nguồn dữ liệu gốc khi các bản sao lệch nhau (mục 6).

## 2. Đổi trạng thái bằng logged batch

Trước khi đổi, lab đọc trạng thái chuyến 12 ở cả bốn bảng; cả bốn câu đều trả `in_transit`. Sau đó một **logged batch** (lô ghi có nhật ký) gom năm câu ghi:

```sql
BEGIN BATCH
  UPDATE trips SET status = 'completed' WHERE trip_id = 07000000-0000-4000-8000-000000000012;
  UPDATE trips_by_customer SET status = 'completed'
    WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
  UPDATE trips_by_driver_day SET status = 'completed'
    WHERE driver_id = 0d000000-0000-4000-8000-000000000002 AND day = '2026-11-03' AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
  UPDATE trips_by_city_hour SET status = 'completed'
    WHERE city_id = 1 AND hour_bucket = '2026-11-03 18:00:00+0700' AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
  INSERT INTO trip_events_by_trip (trip_id, event_at, event_seq, status, actor)
    VALUES (07000000-0000-4000-8000-000000000012, '2026-11-03 18:58:41+0700', 7, 'completed', 'driver');
APPLY BATCH;
```

Sau batch, bốn câu `SELECT` trả `completed`, và dòng thời gian của chuyến có thêm sự kiện thứ 7:

```text
 event_seq | status
-----------+------------
         1 |  requested
         2 |  searching
         3 |   accepted
         4 |   arriving
         5 |  picked_up
         6 | in_transit
         7 |  completed
```

Trong CQL, `BEGIN BATCH` không có từ khoá nào khác chính là logged batch. Fixture của lab (`02-fixture.cql`) cũng ghi mỗi chuyến vào bốn bảng bằng đúng cách này.

### Logged batch bảo đảm gì

Khi nhận một logged batch chạm nhiều partition, **node điều phối** (*coordinator*, node nhận câu lệnh từ client) làm theo thứ tự:

1. Ghi toàn bộ batch vào **batchlog**, một bảng hệ thống, trên các node khác trong cùng datacenter (nếu cụm có nhiều node).
2. Gửi từng câu ghi tới replica của partition tương ứng.
3. Khi mọi câu đã được áp dụng, xoá bản ghi batchlog.

Nếu node điều phối chết giữa bước 2, node giữ batchlog sẽ phát lại batch sau một khoảng chờ. Vì vậy bảo đảm của logged batch là: **một khi batch đã vào batchlog, mọi câu bên trong rồi sẽ được áp dụng**, không có chuyện bảng này có còn bảng kia mất vĩnh viễn. Tài liệu Cassandra gọi đó là *atomic*, nhưng phải hiểu theo nghĩa "rồi sẽ áp dụng hết", không phải nghĩa của transaction trong database quan hệ.

Ngoài ra, mọi câu trong batch dùng **cùng một timestamp ghi** (do Cassandra cấp, hoặc do `USING TIMESTAMP` ở cấp batch). Điều này có một hệ quả cần nhớ: nếu trong cùng một batch bạn vừa `DELETE` vừa `INSERT` *cùng một khoá*, hai thao tác hoà timestamp và tombstone thắng (CS-005), nên dòng mới biến mất.

### Logged batch không bảo đảm gì

- **Không cô lập (isolation) giữa các partition.** Trong lúc batch đang áp dụng, một người đọc có thể thấy `trips` đã `completed` mà `trips_by_driver_day` vẫn `in_transit`. Tài liệu chỉ hứa cô lập *trong một partition*.
- **Không có rollback.** Batch không có "huỷ nửa chừng"; nó chỉ có "rồi sẽ xong hết".
- **Không có điều kiện liên bảng.** Không viết được "chỉ đổi nếu trạng thái hiện tại là `in_transit`" cho cả bốn bảng (mục 5).

### Chi phí

Batchlog là một lần ghi thêm (và một lần xoá) trên node khác, cộng với việc node điều phối phải chờ mọi partition trong batch. Logged batch vì thế **chậm hơn** gửi các câu riêng lẻ; dùng nó để có tính nguyên tử, không phải để tăng tốc. Lab kiểm cấu hình mặc định của Cassandra 5.0.9:

```sh
docker compose exec -T node1 grep -n "batch_size" /etc/cassandra/cassandra.yaml
```

```text
1840:batch_size_warn_threshold: 5KiB
1844:batch_size_fail_threshold: 50KiB
```

Batch chạm nhiều partition lớn hơn 5 KiB sinh cảnh báo trong log, lớn hơn 50 KiB bị từ chối. Một batch cho một chuyến (bốn dòng hẹp và một sự kiện) nằm xa ngưỡng; một batch gom hàng trăm chuyến "cho nhanh" thì không. Logged và unlogged batch, cùng các ngưỡng này, là chủ đề của CS-016.

## 3. Thiếu khoá thì không cập nhật được bản sao

Giả sử service chỉ nhận `trip_id` và `customer_id` từ app rồi cập nhật bảng Q2:

```sql
UPDATE trips_by_customer SET status = 'completed'
WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND trip_id = 07000000-0000-4000-8000-000000000012;
```

```text
Some clustering keys are missing: requested_at
```

`UPDATE` trong Cassandra phải chỉ đúng một dòng bằng **toàn bộ khoá chính**, và không có index nào để tìm `requested_at` từ `trip_id` trong bảng này. Đường ghi đúng là: đọc `trips` theo `trip_id` (Q1) để lấy `customer_id`, `driver_id`, `city_id`, `requested_at`, tính `day` và `hour_bucket` theo giờ Việt Nam, rồi mới dựng batch. Đây là một lần **đọc trước khi ghi** (*read-before-write*) mà thiết kế phi chuẩn hoá buộc phải có; nên làm một lần trong một hàm dùng chung, đừng để mỗi nơi trong code tự tính khoá.

### Đổi một cột thuộc khoá: xoá rồi ghi mới

`requested_at` nằm trong khoá của ba bảng. Nếu bộ phận vận hành cần sửa giờ đặt của một chuyến (ví dụ đồng hồ điện thoại khách lệch), `UPDATE` không làm được. Bộ kiểm tra K10 ở CS-010 xác nhận lỗi:

```text
PRIMARY KEY part requested_at found in SET part
```

Cách duy nhất là xoá dòng ở khoá cũ và ghi dòng ở khoá mới, cho cả ba bảng, trong một logged batch. Khối dưới đây là **minh hoạ, không có trong lab** (giờ mới 18:21 vẫn cùng ngày và cùng giờ, nên `day` và `hour_bucket` không đổi):

```sql
BEGIN BATCH
  UPDATE trips SET requested_at = '2026-11-03 18:21:00+0700' WHERE trip_id = 07000000-0000-4000-8000-000000000012;
  DELETE FROM trips_by_customer
    WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
  INSERT INTO trips_by_customer (customer_id, requested_at, trip_id, service, status, pickup, dropoff, fare_vnd, driver_id)
    VALUES (0c000000-0000-4000-8000-000000000002, '2026-11-03 18:21:00+0700', 07000000-0000-4000-8000-000000000012, 'food', 'completed', 'Lẩu Dê 404, Q3', 'Chung cư Sunrise City, Q7', 118000, 0d000000-0000-4000-8000-000000000002);
  -- … tương tự DELETE + INSERT cho trips_by_driver_day và trips_by_city_hour
APPLY BATCH;
```

Ba điểm cần ý: câu `INSERT` phải mang **đủ mọi cột** (đọc từ `trips` hoặc từ dòng cũ), vì dòng mới không thừa hưởng gì từ dòng cũ; nếu giờ mới rơi sang giờ khác hay ngày khác thì `hour_bucket` và `day` của dòng mới cũng đổi; và `DELETE` sinh tombstone (CS-005), nên việc "đổi khoá" thường xuyên là dấu hiệu khoá chọn chưa đúng.

## 4. Lỗi thường gặp nhất: chỉ sửa một bảng

Bộ phận chăm sóc khách hàng điều chỉnh cước chuyến 11 lên 95.000 đ, và công cụ nội bộ chỉ sửa bảng `trips`:

```sql
UPDATE trips SET fare_vnd = 95000 WHERE trip_id = 07000000-0000-4000-8000-000000000011;
```

Đọc lại cước ở hai bảng:

```text
 fare_o_trips
--------------
        95000

 fare_o_trips_by_customer
--------------------------
                    89000
```

Không có lỗi nào; Cassandra không biết hai bảng "là cùng một chuyến". Khách mở "Chuyến của tôi" thấy 89.000 đ, còn tổng đài tra theo mã chuyến thấy 95.000 đ.

**Sửa lệch** bằng cách ghi giá trị đúng (lấy từ nguồn gốc `trips`) vào mọi bản sao trong một batch:

```sql
BEGIN BATCH
  UPDATE trips_by_customer SET fare_vnd = 95000
    WHERE customer_id = 0c000000-0000-4000-8000-000000000001 AND requested_at = '2026-11-03 17:45:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000011;
  UPDATE trips_by_driver_day SET fare_vnd = 95000
    WHERE driver_id = 0d000000-0000-4000-8000-000000000002 AND day = '2026-11-03' AND requested_at = '2026-11-03 17:45:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000011;
  UPDATE trips_by_city_hour SET fare_vnd = 95000
    WHERE city_id = 1 AND hour_bucket = '2026-11-03 17:00:00+0700' AND requested_at = '2026-11-03 17:45:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000011;
APPLY BATCH;
```

Sau đó `trips_by_customer` trả `95000`. Vì mọi câu ghi là upsert, chạy lại batch sửa lệch nhiều lần cũng cho cùng kết quả: thao tác này **idempotent**, nên job sửa lệch có thể thử lại an toàn.

Bài học không phải "nhớ sửa đủ bốn bảng", mà là **mọi đường ghi phải đi qua một hàm duy nhất** biết danh sách bản sao. Công cụ nội bộ, script vận hành, job nhập dữ liệu đều gọi hàm đó.

## 5. Batch có điều kiện chỉ trong một partition của một bảng

Ý tưởng tự nhiên: "chỉ chuyển sang `completed` nếu đang `in_transit`, và làm cho cả bốn bảng". Ghép điều kiện `IF` (lightweight transaction, P04) vào batch nhiều bảng:

```sql
BEGIN BATCH
  UPDATE trips SET status = 'completed' WHERE trip_id = 07000000-0000-4000-8000-000000000012 IF status = 'in_transit';
  UPDATE trips_by_customer SET status = 'completed'
    WHERE customer_id = 0c000000-0000-4000-8000-000000000002 AND requested_at = '2026-11-03 18:20:00+0700' AND trip_id = 07000000-0000-4000-8000-000000000012;
APPLY BATCH;
```

```text
Batch with conditions cannot span multiple tables
```

Batch có điều kiện chỉ được phép trong **một partition của một bảng**. Mẫu thường dùng là hai bước: ghi có điều kiện vào bảng gốc `trips` (một partition, `IF status = 'in_transit'`); nếu `[applied] = true` thì mới gửi logged batch cập nhật các bản sao. Giữa hai bước có một khe hở: service có thể chết sau bước một. Khe hở đó là lý do cần đối soát định kỳ ở mục sau. Tranh chấp trạng thái chuyến bằng LWT là chủ đề của CS-034 và CS-040.

## 6. Đối soát: số dòng và tổng cước phải bằng nhau

**Đối soát** là so các bản sao với nhau (hoặc với nguồn gốc) để phát hiện lệch. Cách rẻ nhất là so hai con số tổng hợp: số chuyến và tổng cước của mỗi bảng.

```sql
SELECT (text)'trips' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips;
SELECT (text)'trips_by_customer' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_customer;
SELECT (text)'trips_by_driver_day' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_driver_day;
SELECT (text)'trips_by_city_hour' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc
FROM trips_by_city_hour WHERE city_id IN (1, 2) ALLOW FILTERING;
```

```text
 bang                | so_chuyen | tong_cuoc
---------------------+-----------+-----------
 trips               |        12 |    813000
 trips_by_customer   |        12 |    813000
 trips_by_driver_day |        12 |    813000
 trips_by_city_hour  |        12 |    813000
```

(Kết quả thật in thành bốn khối riêng, gộp lại ở đây cho dễ so.) Mỗi câu kèm cảnh báo `Aggregation query used without partition key`, riêng câu cuối là `Aggregation query used on multiple partition keys (IN restriction)`: Cassandra nhắc rằng đây là quét toàn bảng. Điều kiện `city_id IN (1, 2)` loại dữ liệu thử nghiệm của thành phố 9 mà CS-009 ghi vào cùng bảng.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-008/01-doi-soat-bon-bang.webp" alt="Query editor DBStudio với bốn câu SELECT đối soát trên trips, trips_by_customer, trips_by_driver_day, trips_by_city_hour; dưới có bốn tab #1 SELECT đến #4 SELECT và tab Thông điệp; tab #1 đang mở với một dòng: bang trips, so_chuyen 12, tong_cuoc 813000; góc phải ghi 4 câu lệnh 178 ms" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 1 · Đối soát bốn bảng trong DBStudio.</b> Chạy cả script (<code>Ctrl+Shift+Enter</code>) cho bốn tab kết quả, mỗi tab một bảng; tab #1 là <code>trips</code> với 12 chuyến, 813.000 đ. Bấm lần lượt #2–#4 phải thấy cùng hai con số. Tab "Thông điệp" chứa các cảnh báo aggregation không có partition key.</figcaption>
</figure>

Phép so này bắt được dòng thiếu (một bảng chỉ có 11 chuyến) và cước lệch (như chuyến 11 ở mục 4, trước khi sửa: tổng của `trips` hơn tổng của các bản sao đúng 6.000 đ). Nó **không** bắt được trạng thái lệch, vì không có phép cộng nào trên cột `status`; và `GROUP BY` của Cassandra chỉ nhóm được theo cột khoá chính, nên không đếm theo `status` được.

Hai lưu ý khi đưa đối soát lên dữ liệu thật:

- `COUNT(*)` không có partition key quét cả bảng, có thể timeout với hàng trăm triệu dòng. Job đối soát thật chia bảng theo **token range** và quét từng đoạn (CS-036), hoặc đối soát theo từng partition nghiệp vụ (ví dụ tổng cước của một tài xế trong một ngày, so với tổng tính từ `trips`).
- So từng dòng mới bắt được trạng thái lệch: duyệt `trips` theo token range, với mỗi chuyến dựng khoá của ba bản sao, đọc chúng và so `status`, `fare_vnd`. Dòng lệch được sửa bằng batch như mục 4, lấy `trips` làm nguồn gốc.

## 7. Khi nào cân nhắc materialized view hoặc SAI

Cassandra có hai cơ chế để bớt tự tay giữ bản sao:

- **Materialized view** (MV): Cassandra tự duy trì một bảng sắp theo khoá khác từ bảng gốc. Trong 5.0, MV vẫn là tính năng **thử nghiệm** và tắt mặc định (`materialized_views_enabled: false` trong `cassandra.yaml` của lab); bản sao trong view có thể lệch so với bảng gốc. CS-019 tái hiện và giải thích vì sao production thường tự quản bảng tra cứu như bài này.
- **Storage-Attached Index** (SAI, mới ở 5.0): index trên cột không thuộc khoá, ví dụ `status`, giúp lọc trong một partition hay một tập partition mà không cần bảng mới. SAI hợp với truy vấn phụ, tần suất thấp; nó không thay được bảng có thứ tự sẵn cho màn hình chính. CS-018 đo giới hạn của SAI.

Cả hai thuộc Phần 2. Ở P01, bốn bảng tự quản cùng logged batch và đối soát là cách làm rõ ràng nhất.

## 8. Lỗi thường gặp

- **Mỗi nơi trong code tự ghi một bảng.** Công cụ nội bộ sửa `trips`, service sửa ba bảng kia: lệch là chuyện sớm muộn. Gom về một hàm ghi.
- **Nghĩ logged batch là transaction.** Không có isolation, không rollback, không điều kiện liên bảng. Người đọc có thể thấy trạng thái nửa chừng trong vài mili giây.
- **Dùng batch để tăng tốc nạp dữ liệu.** Batch nhiều partition chậm hơn và vượt `batch_size_fail_threshold` rất nhanh.
- **Thiếu khoá của bản sao.** Lỗi `Some clustering keys are missing` nghĩa là service không có đủ thông tin; đọc `trips` trước.
- **DELETE và INSERT cùng khoá trong một batch.** Cùng timestamp, tombstone thắng, dòng mới biến mất.
- **Không có job đối soát.** Batch bảo đảm "rồi sẽ áp dụng hết" cho những gì đã vào batch; nó không cứu được bug ghi thiếu bảng hay service chết giữa bước LWT và batch.

## 9. So với PostgreSQL

Trên PostgreSQL, chuyến nằm ở một bảng; các "bản sao" là index do database tự cập nhật trong cùng transaction, nên không thể lệch. Khi PostgreSQL phi chuẩn hoá (bảng tổng hợp, cột suy ra), việc ghi vẫn nằm trong một transaction có isolation. Cassandra chuyển trách nhiệm đó sang ứng dụng: bản sao là bảng thật, nguyên tử theo nghĩa batchlog, và đối soát là một phần bắt buộc của thiết kế chứ không phải việc làm thêm.

## 10. Bài tập

**Bài 1.** Chuyến 7 (parcel, khách 3, tài xế 3, 02/11 18:40, Hà Nội) bị huỷ sau khi hoàn thành do sự cố. Viết logged batch đổi trạng thái thành `cancelled` ở bốn bảng và ghi sự kiện. Bạn cần đọc những gì trước?

<details><summary><b>Lời giải</b></summary><p>Đọc <code>trips</code> theo <code>trip_id</code> để lấy <code>customer_id</code>, <code>driver_id</code>, <code>city_id = 2</code>, <code>requested_at = '2026-11-02 18:40:00+0700'</code>; tính <code>day = '2026-11-02'</code> và <code>hour_bucket = '2026-11-02 18:00:00+0700'</code>; đọc Q5 để biết <code>event_seq</code> lớn nhất. Batch gồm bốn câu <code>UPDATE … SET status = 'cancelled'</code> với đủ khoá của từng bảng, và một <code>INSERT INTO trip_events_by_trip</code> với <code>event_seq</code> kế tiếp. Nếu Tới Nơi có bảng chuyến huỷ (bài tập CS-007), thêm một <code>INSERT</code> vào bảng đó trong cùng batch.</p></details>

**Bài 2.** Đối soát ban đêm báo `trips_by_driver_day` có 11 chuyến, ba bảng kia 12 chuyến, tổng cước ít hơn 52.000 đ. Nêu cách tìm chuyến thiếu và cách sửa.

<details><summary><b>Lời giải</b></summary><p>Duyệt <code>trips</code> (theo token range khi dữ liệu lớn), với mỗi chuyến dựng khoá <code>(driver_id, day, requested_at, trip_id)</code> và đọc <code>trips_by_driver_day</code>; dòng không trả về là chuyến thiếu (ứng viên đầu tiên: chuyến có cước 52.000 đ). Sửa bằng <code>INSERT</code> đầy đủ cột lấy từ <code>trips</code>; upsert nên chạy lại an toàn. Sau đó tìm đường ghi nào đã bỏ qua bảng này.</p></details>

**Bài 3.** Vì sao không thể dùng một batch có `IF status = 'in_transit'` cho cả bốn bảng? Đề xuất trình tự an toàn và nói rõ khe hở còn lại.

<details><summary><b>Lời giải</b></summary><p>Cassandra từ chối với <code>Batch with conditions cannot span multiple tables</code>: điều kiện chạy bằng Paxos trên một partition. Trình tự: (1) <code>UPDATE trips … IF status = 'in_transit'</code>; (2) nếu <code>[applied]</code> là true, gửi logged batch cho ba bản sao và sự kiện. Khe hở: service chết sau (1) thì bản sao còn trạng thái cũ cho tới khi job đối soát so <code>status</code> từng dòng với <code>trips</code> và sửa.</p></details>

## 11. Đọc tiếp

- [CQL: Data manipulation, mục BATCH](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/dml.html) — logged/unlogged, timestamp chung, isolation trong một partition.
- [Materialized views](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/mvs.html) và [SAI overview](https://cassandra.apache.org/doc/5.0/cassandra/developing/cql/indexing/sai/sai-overview.html).
- [cassandra.yaml](https://cassandra.apache.org/doc/5.0/cassandra/managing/configuration/cass_yaml_file.html) — `batch_size_warn_threshold`, `batch_size_fail_threshold`.
- [PG-008 · Mô hình quan hệ và chuẩn hoá](/articles/postgresql-mo-hinh-quan-he-chuan-hoa): phi chuẩn hoá khi có transaction.

Bài tiếp theo: [CS-009 · Kích thước partition và bucketing theo thời gian](/articles/cassandra-partition-size-bucketing).
