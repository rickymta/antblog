---
slug: cassandra-partition-size-bucketing
title: "Kích thước partition và bucketing theo thời gian: một ngày hay một giờ cho chuyến Tới Nơi?"
summary: "Partition trong Cassandra không nên lớn vô hạn. Bài ước lượng số dòng và byte của một partition, chạy thí nghiệm thật 24.000 chuyến/ngày trên Cassandra 5.0.9 để so bucket ngày với bucket giờ, đọc số của nodetool cho đúng, suy ra cho TP.HCM, và ghi lại một bẫy múi giờ của cqlsh COPY FROM."
category: cassandra
level: advanced
tags: [cassandra, cs-009, cassandra-5, cql, toi-noi, partition-size, bucketing, nodetool, copy-from]
language: vi
---

Ở CS-007, màn hình điều phối dùng bảng `trips_by_city_hour` với partition key `(city_id, hour_bucket)`. Một kỹ sư hỏi lại: "Sao không bucket theo ngày cho gọn? Xem cả ngày chỉ cần đọc một partition thay vì 24." Câu hỏi đúng, và trả lời bằng cảm giác thì không đủ. TP.HCM lúc cao điểm có hàng chục chuyến mới mỗi giây; một partition theo ngày sẽ chứa bao nhiêu dòng, bao nhiêu byte, và lúc nào thì "lớn"?

Bài này ước lượng trên giấy, đo trên lab bằng 24.000 chuyến thật, rồi suy ngược cho TP.HCM bằng số tải trong hồ sơ Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">CS-009 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/cassandra-thiet-ke-theo-truy-van">CS-007 · Thiết kế bảng theo truy vấn</a> và <a href="/articles/du-an-mau-toi-noi">hồ sơ quy mô tải Tới Nơi</a>. Lab: <code>examples/toi-noi-cassandra/p01/08-partition-size.sh</code> (chạy trong container: <code>cqlsh COPY FROM</code> và <code>nodetool</code>); kết quả thật trong <code>p01/OUTPUT.txt</code>. Sau bài này, bạn ước lượng được số dòng và byte của một partition, đọc đúng <code>nodetool tablestats</code> và <code>tablehistograms</code>, chọn bucket ngày hay giờ, và biết cái giá phải trả khi bucket.</p>
</div>

## 1. Vì sao partition lớn là vấn đề

**Partition** là đơn vị phân tán của Cassandra: mọi dòng có cùng partition key nằm trên cùng một nhóm replica (CS-003). Một partition không thể chia nhỏ ra nhiều node. Khi nó phình to:

- **Tải dồn vào ít node.** Mọi lần ghi và đọc partition đó rơi vào đúng các replica của nó, dù cụm có bao nhiêu node.
- **Compaction và repair nặng hơn.** Gộp SSTable (compaction, CS-029) và đồng bộ replica (repair, CS-044) phải xử lý cả partition lớn.
- **Đọc chậm và tốn bộ nhớ.** Đọc một dải trong partition lớn phải đi qua chỉ mục của partition; đọc cả partition có thể vượt timeout.

Tài liệu mô hình hoá của Cassandra 5.0 nêu **giới hạn cứng 2 tỉ ô (cell) mỗi partition**, và nói rõ bạn sẽ gặp vấn đề hiệu năng từ lâu trước khi chạm giới hạn đó. **Ô** (*cell*) là một giá trị của một cột thường trong một dòng.

Cassandra 5.0 không có ngưỡng "partition quá lớn" mặc định nào chặn bạn. Có một **guardrail** (cơ chế cảnh báo hoặc chặn thao tác vượt ngưỡng do quản trị đặt) cho việc này; lab đọc cấu hình mặc định của 5.0.9:

```sh
docker compose exec -T node1 grep -n "partition_size\|large_partition" /etc/cassandra/cassandra.yaml
```

```text
2119:# The warning threshold replaces the deprecated config property compaction_large_partition_warning_threshold.
2121:# partition_size_warn_threshold:
2122:# partition_size_fail_threshold:
```

Đọc phần chú thích quanh hai dòng này trong file cấu hình:

- `partition_size_warn_threshold` / `partition_size_fail_threshold` **mặc định tắt** (null). Đặt dạng `100MiB`, `1GiB`.
- Guardrail chỉ được kiểm **khi ghi SSTable** (flush và compaction), không phải lúc client ghi; vượt ngưỡng fail cũng chỉ ghi lỗi vào log, không dừng thao tác.
- Kiểm **theo từng SSTable**, nên một partition lớn trải trên nhiều SSTable có thể không bị phát hiện.
- Ngưỡng cảnh báo này thay cho tham số cũ `compaction_large_partition_warning_threshold`, nay đã deprecated (các bản 4.x đặt mặc định 100 MiB cho tham số cũ).

Nghĩa là trên 5.0 mặc định, **không có gì cảnh báo bạn** về partition lớn: phải ước lượng khi thiết kế và đo bằng `nodetool` khi vận hành. Ngưỡng cụ thể là quyết định của đội; kinh nghiệm phổ biến trong cộng đồng là giữ partition ở mức vài MB tới vài chục MB và coi trên 100 MB là đáng báo động. Đó là quy ước thực hành, không phải giới hạn trong tài liệu chính thức.

## 2. Ước lượng trên giấy: số ô và số byte

Tài liệu Cassandra đưa công thức số giá trị (ô) trong một partition:

```text
Nv = Nr × (Nc − Npk − Ns) + Ns
```

với `Nr` số dòng, `Nc` số cột, `Npk` số cột khoá chính, `Ns` số cột static. Áp cho bảng thí nghiệm một partition mỗi ngày:

```sql
CREATE TABLE toinoi.trips_by_city_day (
  city_id int, day date, requested_at timestamp, trip_id uuid,
  service text, status text, fare_vnd bigint,
  PRIMARY KEY ((city_id, day), requested_at, trip_id)
) WITH comment = 'Thí nghiệm CS-009: một partition cho cả ngày';
```

`Nc = 7`, `Npk = 4`, `Ns = 0`, nên mỗi dòng có 3 ô (`service`, `status`, `fare_vnd`). Với 24.000 chuyến trong một ngày: `Nv = 24.000 × 3 = 72.000` ô. Số byte thì phụ thuộc kiểu dữ liệu, độ dài chuỗi và metadata của từng ô; cách thực tế nhất là đo một mẫu rồi nhân lên, như mục sau.

Bảng `trips_by_city_hour` có cùng các cột, chỉ khác partition key `(city_id, hour_bucket)`. Cùng dữ liệu đó chia thành 24 partition, mỗi partition khoảng 1/24 nếu tải đều; nhưng tải của Tới Nơi **không đều**.

## 3. Thí nghiệm: 24.000 chuyến vào hai bảng

Script `08-partition-size.sh` sinh 24.000 chuyến hư cấu của "thành phố thử nghiệm" số 9 trong ngày 10/11/2026, theo đúng hình dạng tải trong hồ sơ Tới Nơi: **50% chuyến rơi vào bốn giờ cao điểm** 11:00–13:00 và 17:00–19:00. Phân bố theo giờ (giờ Việt Nam) mà script in ra:

```text
gio  so_chuyen
00   466
03   492
07   490
10   541
11   3424
12   3547
13   461
17   3520
18   3507
23   498
```

(Trích; đủ 24 giờ trong `OUTPUT.txt`, các giờ thấp điểm đều trong khoảng 450–545 chuyến.) Cùng 24.000 chuyến được nạp vào cả hai bảng bằng `cqlsh COPY FROM`:

```text
24000 rows imported from 1 files in 0.997 seconds (0 skipped).
24000 rows imported from 1 files in 0.949 seconds (0 skipped).
```

Đếm số dòng trong partition lớn nhất của mỗi bảng (đếm trong một partition nên hợp lệ):

```text
 dong_partition_ngay
---------------------
               24000
 dong_partition_12h
--------------------
               3547
 dong_partition_03h
--------------------
                492
```

Cùng truy vấn trong DBStudio, đếm sáu partition giờ bằng `IN` và `GROUP BY hour_bucket`:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-009/01-dong-moi-partition-gio.webp" alt="Query editor DBStudio chạy SELECT hour_bucket, COUNT(*) AS so_dong FROM trips_by_city_hour WHERE city_id = 9 AND hour_bucket IN sáu giờ 03, 07, 11, 12, 17, 18 ngày 2026-11-10 +0700 GROUP BY hour_bucket; lưới kết quả 6 dòng, thấy năm dòng đầu: 2026-11-09T20:00:00.000Z 492, 2026-11-10T00:00:00.000Z 490, 2026-11-10T04:00:00.000Z 3424, 2026-11-10T05:00:00.000Z 3547, 2026-11-10T10:00:00.000Z 3520" width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 1 · Số dòng của từng partition giờ.</b> Giờ cao điểm (11h, 12h, 17h, 18h) có khoảng 3.500 dòng, giờ thấp điểm khoảng 490, chênh nhau hơn 7 lần. Lưới DBStudio hiện <code>hour_bucket</code> theo UTC: dòng đầu <code>2026-11-09T20:00Z</code> chính là 03:00 ngày 10/11 giờ Việt Nam. Dòng thứ sáu (18h, 3.507 dòng) nằm dưới, phải cuộn lưới mới thấy.</figcaption>
</figure>

### Đọc kích thước bằng nodetool

Sau `nodetool flush` (đẩy memtable xuống SSTable), lab đọc `nodetool tablestats`:

```text
[trips_by_city_day]
Number of partitions (estimate): 1
Compacted partition minimum bytes: 1358103
Compacted partition maximum bytes: 1629722
Compacted partition mean bytes: 1629722
[trips_by_city_hour]
Number of partitions (estimate): 35
Compacted partition minimum bytes: 30
Compacted partition maximum bytes: 219342
Compacted partition mean bytes: 24775
```

và cột `Partition Size`, `Cell Count` của `nodetool tablehistograms`:

```text
toinoi/trips_by_city_day histograms
Percentile      ...    Partition Size        Cell Count
Max             ...           1629722             73457
toinoi/trips_by_city_hour histograms
Percentile      ...    Partition Size        Cell Count
50%             ...               103                 3
95%             ...            219342             11864
Max             ...            219342             11864
```

(Đã lược các cột độ trễ; bản đầy đủ trong `OUTPUT.txt`.) Những con số này **là ước lượng**, và phải đọc đúng cách:

- Cassandra không ghi lại kích thước chính xác của từng partition; nó đếm vào một **histogram** có các bucket tăng dần theo cấp số nhân (mỗi bucket lớn hơn bucket trước khoảng 1,2 lần). `nodetool` in ra **cận trên của bucket** chứa giá trị.
- Bảng ngày chỉ có một partition, vậy mà minimum là 1.358.103 và maximum là 1.629.722: đó chính là hai biên của bucket chứa partition đó (1.629.722 ÷ 1.358.103 ≈ 1,2). Kích thước thật nằm trong khoảng này, **khoảng 1,4–1,6 MB**.
- `Cell Count` 73.457 cũng là cận trên bucket; số ô thật theo công thức mục 2 là 72.000.
- Với bảng giờ, partition lớn nhất (12h, 3.547 dòng) rơi vào bucket có cận trên 219.342 byte, tức **dưới khoảng 219 KB**; số ô thật 3.547 × 3 = 10.641, cận trên bucket là 11.864.
- `Number of partitions (estimate): 35` gồm 24 partition giờ của thành phố 9 và 11 partition giờ của TP.HCM, Hà Nội từ fixture; trung vị 103 byte và 3 ô là các partition một chuyến của fixture.

Chia ra theo dòng: 1.358.103–1.629.722 byte cho 24.000 dòng là **khoảng 57–68 byte mỗi dòng** (gồm cả clustering key và metadata). Đây là kết quả đo trên một bảng hẹp, chỉ có ba cột thường ngắn; bảng có `pickup`, `dropoff` (vài chục ký tự mỗi cột) sẽ rộng hơn nhiều.

## 4. Suy ra cho TP.HCM thật

Hồ sơ Tới Nơi cho **2 triệu chuyến mỗi ngày** trên 6 thành phố, **50% rơi vào 4 giờ cao điểm**, cao điểm **≈ 70 chuyến/giây** toàn hệ thống. Hồ sơ không tách theo thành phố, nên bài **giả định TP.HCM chiếm 40%** (thành phố lớn nhất); bạn thay tỉ lệ khác thì nhân lại theo cùng cách.

| Đại lượng | Cách tính | Kết quả |
|---|---|---|
| Chuyến/ngày ở TP.HCM | 2.000.000 × 40% | 800.000 |
| Chuyến trong một giờ cao điểm | 800.000 × 50% ÷ 4 | 100.000 |
| Kiểm chéo bằng tốc độ cao điểm | 70 chuyến/giây × 3.600 × 40% | 100.800 |
| Chuyến trong một giờ thấp điểm | 800.000 × 50% ÷ 20 | 20.000 |
| Byte mỗi dòng (cận trên đo ở lab) | 1.629.722 ÷ 24.000 | ≈ 68 B |
| **Partition ngày** | 800.000 × 68 B; 800.000 × 3 ô | **≈ 54 MB; 2,4 triệu ô** |
| **Partition giờ cao điểm** | 100.000 × 68 B; 100.000 × 3 ô | **≈ 6,8 MB; 300.000 ô** |

Với bucket ngày, mỗi partition của TP.HCM vào khoảng 54 MB *chỉ với ba cột hẹp*; thêm điểm đón, điểm trả là vượt 100 MB. Mọi chuyến mới của thành phố trong cả ngày đều ghi vào cùng một partition, tức cùng một nhóm replica: một **partition nóng** (*hot partition*). Bucket giờ cắt partition lớn nhất xuống khoảng 6,8 MB, và tải ghi chuyển sang partition mới mỗi giờ.

Bucket giờ vẫn chưa hoàn hảo: trong giờ cao điểm, khoảng 28 chuyến/giây (100.000 ÷ 3.600) vẫn dồn vào một partition. Với Tới Nơi con số này chấp nhận được. Nếu tải cao hơn nhiều, bước tiếp theo là thêm một cột **shard** vào partition key, ví dụ `(city_id, hour_bucket, shard)` với `shard` là băm của `trip_id` chia lấy dư cho N, đổi lại phải đọc N partition cho mỗi giờ. Partition nóng và cách phát hiện bằng `toppartitions` là chủ đề của CS-033.

Hai bảng còn lại của P01 dùng cùng phép tính:

- `trips_by_driver_day`: một tài xế chạy vài chục chuyến mỗi ngày, partition chỉ vài KB. Bucket ngày ở đây không phải vì kích thước, mà vì màn hình "Thu nhập hôm nay" hỏi đúng một ngày.
- `trips_by_customer`: **không có bucket**, partition lớn dần theo đời khách hàng. Một khách rất tích cực đi 5 chuyến/ngày tạo khoảng 1.800 dòng/năm; với dòng rộng khoảng 150 B (có điểm đón, điểm trả) là khoảng 270 KB/năm. Chấp nhận được trong nhiều năm; nếu cần, giới hạn bằng chính sách lưu giữ hoặc bucket theo năm.

## 5. Cái giá của bucketing

Bucket không miễn phí. Ba chi phí phải tính trước:

**Đọc nhiều partition.** Màn hình điều phối xem hai giờ liền nhau là hai partition. CS-006 đã chạy cách đọc này bằng `IN` trên partition key:

```text
 hour_bucket                     | requested_at                    | service
---------------------------------+---------------------------------+---------
 2026-11-02 17:00:00.000000+0700 | 2026-11-02 17:31:00.000000+0700 |    ride
 2026-11-02 18:00:00.000000+0700 | 2026-11-02 18:02:00.000000+0700 |    ride
```

Với vài bucket, `IN` tiện. Với báo cáo cả ngày (24 bucket) hay cả tuần (168 bucket), `IN` dài bắt node điều phối gom kết quả từ nhiều partition trên nhiều node rồi giữ trong bộ nhớ; cách tốt hơn là driver gửi **nhiều câu riêng chạy song song**, mỗi câu một partition (CS-037).

**Ứng dụng phải tính bucket.** `hour_bucket` không có trong nghiệp vụ. Mỗi lần ghi và đọc, ứng dụng phải làm tròn `requested_at` xuống đầu giờ **theo giờ Việt Nam**, giống hệt nhau ở mọi service. Hai service làm tròn khác nhau (một theo UTC, một theo giờ Việt Nam) sẽ ghi vào bucket này và đọc ở bucket khác. Lab ghi `hour_bucket` dạng `'2026-11-03 18:00:00+0700'`; đó là cùng thời điểm với `2026-11-03T11:00:00Z` mà DBStudio hiển thị.

**Đổi kích thước bucket là migration.** Bucket là một phần của khoá chính; chuyển từ giờ sang 15 phút nghĩa là bảng mới, ghi kép và nạp lại dữ liệu cũ (CS-048). Hãy ước lượng như mục 4 trước khi chọn.

Thêm một chi tiết nhỏ: bucket rỗng (giờ không có chuyến) vẫn là một câu đọc, trả về không dòng. Với màn hình điều phối điều này vô hại; với job quét hàng nghìn bucket thì là chi phí thật.

## 6. Bẫy của cqlsh COPY FROM: độ lệch múi giờ bị đảo dấu

Khi dựng thí nghiệm này, lab gặp một lỗi đáng ghi lại. CSV ban đầu ghi thời điểm kèm độ lệch giờ Việt Nam, như cách lab vẫn viết trong CQL (`'2026-11-02 18:20:00+0700'`). Dữ liệu nạp vào lệch 14 giờ. Script tái hiện bằng một bảng nhỏ với hai dòng CSV cùng một thời điểm, viết theo hai cách:

```text
1,2026-11-01 14:00:00+0700
2,2026-11-01 07:00:00+0000
```

Đọc lại sau `COPY toinoi.copy_tz_lab (id, at_ts) FROM …`:

```text
 id | doc_ra
----+---------------------------------
  1 | 2026-11-02 04:00:00.000000+0700
  2 | 2026-11-01 14:00:00.000000+0700
```

Dòng 2 (ghi bằng UTC) đúng: 14:00 giờ Việt Nam. Dòng 1 thành 04:00 *ngày hôm sau*: cqlsh đã hiểu `+0700` như `-0700` (14:00 tại UTC−7 là 21:00 UTC, tức 04:00 ngày 02/11 giờ Việt Nam), lệch 14 giờ. Không có lỗi hay cảnh báo nào. Câu `INSERT` với cùng chuỗi `+0700` thì đúng (mọi fixture của lab dùng cách này); lỗi nằm ở bộ đọc CSV của cqlsh.

Đây là phát hiện trên **cqlsh đi kèm Cassandra 5.0.9**; bài chưa kiểm các bản khác. Cách tránh mà script `08-partition-size.sh` áp dụng:

- **Ghi CSV bằng UTC** (`+0000`); script sinh dữ liệu trừ 7 giờ trước khi ghi.
- **Kiểm sau khi nạp**: đọc lại vài dòng có thời điểm đã biết, so với nguồn trước khi tin dữ liệu.
- Với dữ liệu lớn, dùng công cụ nạp khác hoặc driver ứng dụng (prepared statement với giá trị timestamp, không qua chuỗi).

## 7. Lỗi thường gặp

- **Chọn bucket theo cảm giác.** Tính số dòng × byte của partition lớn nhất (giờ cao điểm, thành phố lớn nhất), không phải partition trung bình.
- **Tin số của nodetool như số chính xác.** `Compacted partition maximum bytes` và `Cell Count` là cận trên của bucket histogram; dùng chúng để biết bậc độ lớn.
- **Tưởng Cassandra sẽ cảnh báo.** Guardrail kích thước partition của 5.0 tắt mặc định và chỉ kiểm khi flush/compaction.
- **Tính bucket theo UTC ở một service, giờ Việt Nam ở service khác.** Dữ liệu "biến mất" vì ghi và đọc ở hai partition khác nhau.
- **Nạp CSV có `+0700` bằng cqlsh COPY FROM 5.0.9.** Thời điểm lệch 14 giờ mà không có lỗi; ghi CSV bằng UTC.

## 8. So với PostgreSQL

PostgreSQL cũng có "partition" ([PG-051](/articles/postgresql-partition-co-ban)), nhưng đó là chia một bảng lớn thành nhiều bảng con trên **cùng một máy** để dọn dữ liệu cũ và lọc nhanh hơn; một partition PostgreSQL chứa hàng trăm triệu dòng là bình thường. Partition của Cassandra là đơn vị phân tán và đơn vị đọc, nên phải nhỏ hơn nhiều bậc. Bucket theo thời gian của Cassandra gần với partition theo ngày của PostgreSQL về hình thức, nhưng mục tiêu là giới hạn kích thước từng partition và rải tải, chứ không phải xoá dữ liệu cũ nhanh.

## 9. Bài tập

**Bài 1.** Giả sử TP.HCM chỉ chiếm 25% số chuyến. Tính lại số dòng và byte của partition ngày và partition giờ cao điểm. Kết luận về bucket có đổi không?

<details><summary><b>Lời giải</b></summary><p>Chuyến/ngày: 2.000.000 × 25% = 500.000; giờ cao điểm: 500.000 × 50% ÷ 4 = 62.500. Partition ngày ≈ 500.000 × 68 B ≈ 34 MB (1,5 triệu ô); partition giờ ≈ 62.500 × 68 B ≈ 4,25 MB. Kết luận không đổi: partition ngày vẫn là một partition nóng nhận mọi lần ghi của thành phố cả ngày, và sẽ vượt 100 MB khi thêm cột hoặc khi tải tăng; bucket giờ vẫn hợp.</p></details>

**Bài 2.** Bảng sự kiện `trip_events_by_trip` có partition key `trip_id`. Hồ sơ Tới Nơi có ~16 triệu sự kiện/ngày cho 2 triệu chuyến. Partition có cần bucket không?

<details><summary><b>Lời giải</b></summary><p>Trung bình 8 sự kiện mỗi chuyến, và một chuyến kết thúc trong vài chục phút: partition có giới hạn tự nhiên, vài trăm byte tới vài KB. Không cần bucket. Số partition rất lớn (2 triệu mỗi ngày) là điều tốt: tải rải đều trên ring.</p></details>

**Bài 3.** Một báo cáo cần tổng cước theo giờ của TP.HCM trong 7 ngày. Viết kế hoạch đọc trên `trips_by_city_hour` và nêu vì sao không dùng một câu `IN` với 168 giá trị.

<details><summary><b>Lời giải</b></summary><p>Ứng dụng sinh 168 giá trị <code>hour_bucket</code> theo giờ Việt Nam, gửi 168 câu <code>SELECT SUM(fare_vnd) … WHERE city_id = 1 AND hour_bucket = ?</code> bằng prepared statement, chạy song song có giới hạn (ví dụ 16 câu cùng lúc), rồi cộng ở ứng dụng. Một câu <code>IN</code> 168 giá trị bắt một node điều phối gom kết quả từ 168 partition trên nhiều node, dễ timeout, và lỗi một partition làm hỏng cả câu. Báo cáo lớn hơn nữa nên chạy trên hệ phân tích (CS-055).</p></details>

## 10. Đọc tiếp

- [Evaluating and refining data models](https://cassandra.apache.org/doc/5.0/cassandra/developing/data-modeling/data-modeling_refining.html) — công thức số ô, kích thước partition, bucketing.
- [nodetool tablestats](https://cassandra.apache.org/doc/5.0/cassandra/managing/tools/nodetool/tablestats.html) và [nodetool tablehistograms](https://cassandra.apache.org/doc/5.0/cassandra/managing/tools/nodetool/tablehistograms.html).
- [cassandra.yaml](https://cassandra.apache.org/doc/5.0/cassandra/managing/configuration/cass_yaml_file.html) — guardrail `partition_size_warn_threshold`.
- [cqlsh: COPY FROM](https://cassandra.apache.org/doc/5.0/cassandra/managing/tools/cqlsh.html) — tuỳ chọn nạp CSV.
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi): số tải dùng trong mục 4.

Bài tiếp theo: [CS-010 · Thực hành: mô hình lịch sử chuyến và dòng thời gian sự kiện](/articles/cassandra-thuc-hanh-mo-hinh-lich-su-chuyen).
