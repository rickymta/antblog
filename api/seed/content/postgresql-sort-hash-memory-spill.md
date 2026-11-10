---
slug: postgresql-sort-hash-memory-spill
title: "Sort, hash và truy vấn tràn ra đĩa: work_mem, file tạm và cái giá khi nhiều phiên cùng chạy"
summary: "Nhật ký database Tới Nơi mỗi đêm có dòng «temporary file … size 22134784» và có người đề xuất work_mem = 256MB cho cả server. Đọc Sort Method, Batches, Disk Usage, đếm file tạm và đo bộ nhớ nhân theo nút, phiên, worker."
category: postgresql
level: advanced
tags: [postgresql, pg-029, work-mem, sort, hash-join, temp-files, explain, toi-noi]
language: vi
---

Sáng thứ hai, người trực database Tới Nơi mở nhật ký server và thấy một dòng lặp lại sau mỗi lần chạy báo cáo đối soát cước:

```text
LOG:  temporary file: path "base/pgsql_tmp/pgsql_tmpPID.N", size 22134784
```

Câu báo cáo chỉ sắp 1 triệu chuyến theo cước, nhưng PostgreSQL đã ghi 22 MB ra một **file tạm** (temporary file — file đệm trên đĩa, tự xoá khi câu xong). Một người đề xuất: "đặt `work_mem = 256MB` cho cả server là hết tràn". Đề xuất đó đúng một nửa: câu này sẽ hết tràn. Nửa còn lại — bộ nhớ của *mọi* câu, *mọi* phiên — là nội dung bài này.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-029 · Phần 3/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-explain-ke-hoach-thuc-thi">PG-021 · Đọc EXPLAIN từ truy vấn tìm chuyến</a>. Lab: <code>examples/toi-noi-postgresql/p03</code> (tệp <code>29-sort-hash-spill.sql</code>, số đo ở <code>DO-DAC.md</code>) và thí nghiệm <code>p03/thu-nghiem/pg029-spill-bai-tap.sql</code>. Sau bài này, bạn đo được tràn đĩa của sort/hash và nêu được đánh đổi bộ nhớ khi có tải đồng thời.</p>
</div>

## 1. Bộ nhớ làm việc của một câu truy vấn

Ba nút hay gặp nhất cần chỗ để "giữ hết rồi mới trả": **Sort** (sắp), **Hash** (bảng băm của Hash Join) và **HashAggregate** (gom nhóm bằng bảng băm); tài liệu liệt kê thêm merge join, Memoize, `IN` subquery dạng băm. Mỗi nút được một hạn mức:

| Tham số | Áp cho | Mặc định PG 18 | Lab |
|---|---|---|---|
| `work_mem` | Sort và là cơ số cho nút băm | 4MB | 4MB |
| `hash_mem_multiplier` | Nhân với `work_mem` cho nút băm | 2,0 (từ PG 15; trước đó 1,0) | 2 |

Hạn mức của một nút băm = `work_mem × hash_mem_multiplier` = 8 MB trong lab. Vượt hạn mức, nút không lỗi mà **ghi phần dư ra file tạm** rồi đọc lại — đó là "tràn đĩa" (spill). Tài liệu nói rõ hạn mức này tính cho *từng thao tác*: một câu phức tạp có nhiều nút sort/hash chạy cùng lúc, và nhiều phiên làm vậy song song, nên tổng bộ nhớ "có thể gấp nhiều lần" `work_mem`.

Mọi số trong bài chạy trên **PostgreSQL 18.1** thật, dữ liệu 1.000.000 chuyến (19.441 page), 200.003 khách, song song (parallel query) và JIT tắt trong cluster lab trừ chỗ ghi khác.

## 2. Sort: ba cách sắp và dòng "Sort Method"

Câu của báo cáo, không có index trên `fare`:

```sql
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, fare FROM trips ORDER BY fare DESC, code;
```

```text
 Sort (actual rows=1000000.00 loops=1)
   Sort Key: fare DESC, code
   Sort Method: external merge  Disk: 21616kB
   Buffers: shared hit=6 read=19441, temp read=2702 written=2716
   ->  Seq Scan on trips (actual rows=1000000.00 loops=1)
```

`external merge` nghĩa là sắp từng phần vừa `work_mem`, ghi mỗi phần (run) ra file tạm, rồi trộn các phần. `temp written=2716` là số khối 8 kB đã ghi tạm. Đổi `work_mem` và thêm `LIMIT`:

| Biến thể | Sort Method | Bộ nhớ / đĩa | Khối tạm ghi |
|---|---|---|---|
| `work_mem = 64kB` (tối thiểu) | external merge | Disk 21.608 kB | 11.633 |
| `work_mem = 4MB` (mặc định) | external merge | Disk 21.616 kB | 2.716 |
| `work_mem = 256MB` | quicksort | Memory 55.827 kB | 0 |
| `LIMIT 20`, `work_mem = 64kB` | top-N heapsort | Memory 26 kB | 0 |

Ba điều đọc ra từ bảng:

- **Dung lượng file tạm gần như không đổi** giữa 64kB và 4MB; cái đổi là số lượt trộn — khối tạm ghi gấp hơn 4 lần (11.633 vs 2.716).
- **Bản trong bộ nhớ to hơn file tạm**: cùng 1 triệu cặp (code, fare), quicksort cần 55.827 kB trong khi file tạm chỉ 21.616 kB. Đừng nhìn "Disk: 21616kB" rồi đặt `work_mem = 22MB` và mong hết tràn.
- **Top-N heapsort** chỉ giữ N dòng tốt nhất trong một đống (heap) — 26 kB, không bao giờ tràn, dù vẫn phải đọc đủ 19.441 page. `LIMIT` đổi cả thuật toán, không chỉ cắt kết quả.

## 3. Thấy file tạm ở đâu

`EXPLAIN` không `ANALYZE` chỉ là kế hoạch: nó có nút `Sort` nhưng **không có** dòng `Sort Method` — tràn hay không chỉ biết khi câu chạy thật (thí nghiệm A, `pg029-spill-bai-tap.sql`; không có trong lab chính). Bốn nơi cho bằng chứng:

| Nơi | Cho biết | Lab đo được |
|---|---|---|
| `EXPLAIN (ANALYZE)` | Nút nào tràn, bao nhiêu (`Disk`, `Batches`, `temp read/written`) | mục 2, 4, 5 |
| `log_temp_files` | Mỗi file tạm một dòng nhật ký, kích thước byte, ghi khi file bị xoá | `size 22134784` |
| `pg_stat_database.temp_files/temp_bytes` | Đếm luỹ kế theo database, mọi file bất kể `log_temp_files` | +1 file, > 10 MB |
| `pg_stat_statements.temp_blks_written` | Luỹ kế theo **câu** (đã chuẩn hoá hằng số) | câu GROUP BY khách: vài nghìn khối |

Tài liệu ghi `log_temp_files = 0` ghi mọi file, giá trị dương chỉ ghi file từ cỡ đó trở lên, mặc định `-1` tắt. Trên server thật, đặt một ngưỡng (ví dụ vài chục MB) cho cả server rẻ hơn nhiều so với bật `0`.

Thí nghiệm C reset `pg_stat_statements` rồi chạy ba câu báo cáo; chỉ câu đếm theo khách có khối tạm:

```text
                                  cau                                   | calls | co_tran
------------------------------------------------------------------------+-------+---------
 SELECT count(*) FROM (SELECT customer_id, count(*) FROM trips GROUP BY |     1 | t
 SELECT code, fare FROM trips ORDER BY fare DESC, code LIMIT $1         |     1 | f
 SELECT count(*) FROM trips WHERE fare >= $1                            |     1 | f
```

`pg_stat_statements` phải nằm trong `shared_preload_libraries` (cần khởi động lại server) — cluster lab đã nạp sẵn, bài chỉ `CREATE EXTENSION`.

## 4. Hash Join: "Batches" lớn hơn 1

Doanh thu theo loại khách nối 1 triệu chuyến với bảng băm dựng từ 200.003 khách:

```sql
SELECT c.kind, count(*), sum(t.fare)
FROM trips AS t JOIN customers AS c ON c.id = t.customer_id
GROUP BY c.kind;
```

```text
   ->  Hash Join (actual rows=1000000.00 loops=1)
         Buffers: shared hit=715 read=21183, temp read=2803 written=2803
         ...
         ->  Hash (actual rows=200003.00 loops=1)
               Buckets: 131072  Batches: 2  Memory Usage: 6589kB
               Buffers: shared read=2457, temp written=549
```

Bảng băm không vừa 8 MB nên PostgreSQL chia khoá thành 2 **batch** (lô): giữ lô đầu trong bộ nhớ, ghi lô sau ra file tạm — và ghi luôn những dòng `trips` thuộc lô sau, vì phía dò cũng phải chia theo cùng khoá. Vì thế `temp written` ở Hash chỉ 549 khối mà ở Hash Join là 2.803. Tài liệu `using-explain` nhắc: khi số batch lớn hơn một sẽ có dùng đĩa, nhưng dòng `Memory Usage` không cho thấy phần đó.

Với `SET hash_mem_multiplier = 8` (hạn mức 32 MB):

```text
               Buckets: 262144  Batches: 1  Memory Usage: 13181kB
               Buffers: shared hit=2457
```

Một lô, 13.181 kB ≈ 2 × 6.589 kB, không còn `temp`. Hệ số chỉ áp cho nút băm: Sort trong cùng phiên vẫn dùng 4 MB. Con số "6.589 kB" ở hai lô nhỏ hơn 8 MB không có nghĩa là "suýt vừa": cả bảng cần ~13 MB.

## 5. HashAggregate tràn đĩa

Trước PG 13, planner tránh HashAggregate khi ước lượng vượt `work_mem`, còn khi đã chọn thì bảng băm cứ thế phình trong RAM — kể cả khi ước lượng sai. Release notes PG 13 ghi thay đổi: bảng băm của HashAggregate nay **tràn ra đĩa** khi vượt `work_mem × hash_mem_multiplier`. Đếm chuyến theo từng khách (191.852 nhóm):

```text
 HashAggregate (actual rows=191852.00 loops=1)
   Group Key: customer_id
   Batches: 5  Memory Usage: 8761kB  Disk Usage: 15360kB
   Buffers: shared hit=1001 read=18440, temp read=1643 written=3138
```

Chạy lại trong giao dịch có `SET LOCAL work_mem = '64MB'`: `Batches: 1  Memory Usage: 16409kB`, không còn `temp`. Để ý `Memory Usage: 8761kB` nhỉnh hơn hạn mức 8.192 kB một chút — hạn mức là ngưỡng để bắt đầu tràn, không phải bức tường chính xác từng byte.

## 6. Tràn đĩa có chậm không? Số đo, không phải niềm tin

Thời gian dưới đây lấy từ `p03/DO-DAC.md`, **không** đo lại cho bài. Máy: Intel Core i7-10700 (16 luồng), 64 GB RAM, NVMe SSD Kingston, Windows 11 Pro, PostgreSQL 18.1; cluster `shared_buffers` 128MB, song song/JIT/autovacuum tắt. Mỗi phép: 1 lần khởi động bỏ đi + 5 lần đo, lấy **trung vị**; `EXPLAIN (ANALYZE, TIMING OFF)` lấy *Execution Time* phía server; cache nóng (bảng 152 MB > `shared_buffers` nên phần lớn nằm ở page cache của hệ điều hành). Máy có tải khác (CPU toàn máy 22 % trước đo, 7 % sau); lệnh đo chạy đủ 3 lần trong ngày, xu hướng giữ nguyên, số tuyệt đối dao động khoảng ±20 %.

| Mã | Phép đo | Trung vị (ms) | Min–max |
|---|---|---:|---|
| G6-a | Sắp 1 triệu, `work_mem` 64kB (external merge) | 597 | 592–646 |
| G6-b | Sắp 1 triệu, 4MB (external merge) | **507** | 502–569 |
| G6-c | Sắp 1 triệu, 256MB (quicksort trong RAM) | 657 | 616–800 |
| G6-d | Đếm theo khách, 4MB (HashAggregate 5 batch) | 561 | 406–585 |
| G6-e | Như G6-d, 64MB (1 batch) | 378 | 369–430 |
| G6-f | Hash Join, `hash_mem_multiplier` 2 (2 batch) | 696 | 675–802 |
| G6-g | Hash Join, `hash_mem_multiplier` 8 (1 batch) | 585 | 561–626 |

Hai nút băm nhanh hơn khi vừa bộ nhớ (≈ −33 % và −16 %; khoảng min–max của G6-d rộng, min 406 ms — nhiễu). Với Sort thì ngược kỳ vọng: **quicksort 256MB chậm hơn external merge 4MB** ở cả ba lần chạy đủ của lệnh đo. Đây là *quan sát* trên một máy; lab chưa kiểm nguyên nhân, nên bài không đưa lời giải thích. Điều rút ra được là hẹp hơn: "tràn đĩa luôn chậm, tăng `work_mem` luôn nhanh" không đứng vững trên số đo này. Muốn kết luận cho máy của bạn, đo trên máy của bạn.

Giới hạn: một máy, một loại câu, một phiên, dữ liệu tổng hợp, file tạm ~21 MB nằm gọn trong page cache. Trên server có đĩa chậm hơn hoặc file tạm hàng GB, kết quả có thể khác hẳn.

## 7. Nhân lên: nút × phiên × worker

Câu báo cáo thường không chỉ có một nút. Câu dưới có ba:

```text
 Sort
   Sort Key: (count(*)) DESC
   ->  HashAggregate
         Group Key: c.kind, t.city_id
         ->  Hash Join
               Hash Cond: (t.customer_id = c.id)
               ->  Seq Scan on trips t
               ->  Hash
                     ->  Seq Scan on customers c
```

Trần lý thuyết cho một phiên: Hash 8 MB + HashAggregate 8 MB + Sort 4 MB = 20 MB. Lab có `max_connections = 40`; nếu cả 40 phiên cùng chạy câu này, trần là **800 MB** — gấp hơn 6 lần `shared_buffers` 128MB. Đây là phép nhân minh hoạ (chỉ thị `expect-line: 40 | 4 | 2 | 800` của lab), không phải số đo; thực tế hiếm khi mọi nút cùng đầy.

**Worker song song nhân thêm một lần nữa.** Tài liệu ghi hạn mức như `work_mem` áp riêng cho từng worker. Thí nghiệm D bật `max_parallel_workers_per_gather = 2` trong phiên rồi sắp `SELECT *` (số kB đổi giữa các lần chạy):

```text
 Gather Merge (actual rows=1000000.00 loops=1)
   Workers Launched: 2
   ->  Sort (actual rows=333333.33 loops=3)
         Sort Method: external merge  Disk: 61376kB
         Worker 0:  Sort Method: external merge  Disk: 33784kB
         Worker 1:  Sort Method: external merge  Disk: 37568kB
```

Ba tiến trình (dòng `Sort Method` đầu là của tiến trình chính, hai dòng `Worker` của hai worker), mỗi tiến trình một Sort với hạn mức riêng; tổng ba file ≈ 132,7 MB, xấp xỉ một Sort đơn của cùng câu (132.568 kB). Một phiên song song có thể dùng gấp ba bộ nhớ sort của một phiên thường.

**Đo thật bằng pgbench** (`DO-DAC.md`, cùng máy): mỗi giao dịch là câu đếm chuyến theo khách trên cả bảng, 10 giây mỗi cấu hình, `work_mem` đặt cho mọi phiên.

| `work_mem` | Phiên | tps | Độ trễ TB (ms) | File tạm | Dữ liệu tạm (MB) | Đỉnh bộ nhớ riêng các backend (MB) |
|---|---:|---:|---:|---:|---:|---:|
| 4MB | 1 · 4 · 8 | 2,3 · 8,2 · 11,3 | 434 · 489 · 708 | 23 · 84 · 116 | 345 · 1.169 · 1.598 | 15 · 60 · 119 |
| 64MB | 1 · 4 · 8 | 3,0 · 9,1 · 15,0 | 335 · 441 · 534 | 0 · 0 · 0 | 0 | 22 · 88 · 172 |

Với câu này, 64MB cho tps cao hơn ~30 % ở 8 phiên và không ghi file tạm (4MB ghi ~1,6 GB trong 10 giây). Đổi lại, bộ nhớ riêng tăng tuyến tính theo số phiên, ~22 MB mỗi phiên. Bộ nhớ đo bằng *private bytes* của các backend trên Windows, lấy mẫu ~300 ms một lần — đỉnh thật có thể cao hơn. Phép đo chỉ có một loại câu và tối đa 8 phiên; nó minh hoạ phép nhân, không dự báo tải production.

## 8. Giảm tràn đúng chỗ

**Bớt dữ liệu phải sắp/băm trước khi tăng bộ nhớ.** `LIMIT` biến Sort thành top-N (mục 2). Index đúng thứ tự bỏ hẳn nút Sort — thí nghiệm E tạo `(fare DESC, code)`:

| Câu | Kế hoạch | Page chạm |
|---|---|---:|
| `code, fare … LIMIT 20` | Index Only Scan | 4 |
| `code, fare` toàn bộ | Index Only Scan, `Heap Fetches: 0` | — (không Sort, không file tạm) |
| `SELECT *` toàn bộ | Index Scan theo thứ tự fare | 820.697 (hit + read; cách chia đổi giữa các lần chạy) |

Dòng cuối là cái bẫy: index không chứa đủ cột nên mỗi dòng phải ghé heap theo thứ tự ngẫu nhiên — ~820.000 lượt chạm page thay vì 19.441. Planner tính ra kế hoạch này rẻ hơn Seq Scan + Sort; lab **không đo thời gian** của nó nên bài không nói nó nhanh hay chậm. Hết tràn đĩa không đồng nghĩa với rẻ hơn.

**Tăng bộ nhớ cho riêng câu cần.** `SET LOCAL work_mem = '128MB'` trong giao dịch của báo cáo; ra khỏi giao dịch, `SHOW work_mem` lại là `4MB`. Giữ mặc định server nhỏ cho hàng nghìn câu ngắn của ứng dụng.

**Đặt trần đĩa.** `temp_file_limit` (mặc định `-1`, không giới hạn) giới hạn tổng file tạm *một tiến trình* dùng cùng lúc; vượt thì giao dịch bị huỷ. Thí nghiệm B:

```text
SET LOCAL temp_file_limit = '10MB';
SELECT count(*) FROM (SELECT code, fare FROM trips ORDER BY fare DESC, code OFFSET 0) AS x;
ERROR:  temporary file size exceeds "temp_file_limit" (10240kB)
-- [lab] SQLSTATE 53400
```

Cùng trần đó, câu `LIMIT 20` chạy bình thường; và `CREATE TEMP TABLE … AS` 1 triệu dòng với trần `1MB` vẫn chạy — tài liệu ghi bảng tạm tường minh không tính vào giới hạn này.

## 9. Lỗi thường gặp

- **Đọc `EXPLAIN` thiếu `ANALYZE` rồi kết luận "không tràn".** Kế hoạch không chứa `Sort Method` hay `Batches` thật.
- **Coi `work_mem` là bộ nhớ của cả câu hay cả server.** Nó là hạn mức mỗi nút, mỗi tiến trình; nhân với số nút, số phiên, số worker.
- **Suy `work_mem` cần đặt từ con số `Disk:`.** Bản trong bộ nhớ to hơn file tạm (55.827 kB vs 21.616 kB).
- **Quên `hash_mem_multiplier`.** Nút băm được gấp đôi `work_mem` từ PG 15; tính trần mà bỏ hệ số là thiếu một nửa.
- **Đặt `work_mem` lớn trong `postgresql.conf` vì một báo cáo.** Dùng `SET LOCAL` cho báo cáo đó.
- **Tin "tràn đĩa luôn chậm".** Mục 6 có phản ví dụ trên một máy; đo trước khi chỉnh.

## 10. Đánh đổi

| Lựa chọn | Được | Mất |
|---|---|---|
| Tăng `work_mem` toàn server | Ít file tạm, nút băm nhanh hơn trong phép đo | Bộ nhớ nhân theo nút × phiên × worker; rủi ro hết RAM |
| `SET LOCAL` cho câu báo cáo | Chỉ câu cần mới tốn | Phải sửa ở ứng dụng/job; dễ quên khi thêm câu mới |
| Tăng `hash_mem_multiplier` | Chỉ nút băm được thêm bộ nhớ | Vẫn nhân theo phiên; Sort không đổi |
| Index đúng thứ tự | Bỏ Sort, top-N đọc vài page | Chi phí ghi mỗi lần chèn/sửa (PG-025); có thể thành đọc heap ngẫu nhiên |
| `temp_file_limit` | Một câu tồi không ghi đầy đĩa | Câu hợp lệ nhưng lớn bị huỷ (`53400`) — phải chọn trần theo báo cáo lớn nhất |

## 11. Bài tập

**Bài 1.** Ba câu báo cáo chạy hằng đêm. Không đọc nhật ký server, chỉ ra câu nào tràn đĩa và tràn bao nhiêu khối.

<details><summary><b>Lời giải</b></summary>
<p>Dùng <code>pg_stat_statements</code> (cần <code>shared_preload_libraries</code>): reset, chạy các câu, rồi xếp theo <code>temp_blks_written</code>. Lời giải chạy được ở mục C của <code>pg029-spill-bai-tap.sql</code>:</p>
<pre><code class="language-sql">SELECT pg_stat_statements_reset();
-- … chạy ba câu …
SELECT left(regexp_replace(query, '\s+', ' ', 'g'), 70) AS cau, calls,
       temp_blks_written &gt; 0 AS co_tran
FROM pg_stat_statements
WHERE dbid = (SELECT oid FROM pg_database WHERE datname = current_database())
  AND query ~ 'FROM trips' AND query !~ 'pg_stat_statements'
ORDER BY temp_blks_written DESC, query;</code></pre>
<p>Chỉ câu GROUP BY khách có <code>co_tran = t</code>; một lần chạy thí nghiệm ghi 2.153 khối (8 kB mỗi khối, ≈ 17 MB; thí nghiệm chỉ kiểm khoảng 1.000–10.000 vì số này đổi giữa các lần chạy). Muốn biết nút nào tràn thì mới cần <code>EXPLAIN (ANALYZE)</code> cho riêng câu đó.</p>
</details>

**Bài 2.** Đội vận hành muốn chặn mọi câu ghi quá 10 MB file tạm. Dự đoán câu sắp 1 triệu dòng (không LIMIT) và câu top-20 sẽ ra sao, và một job `CREATE TEMP TABLE … AS` 1 triệu dòng có bị chặn không.

<details><summary><b>Lời giải</b></summary>
<p>Câu sắp đầy đủ cần một file tạm ~21 MB ⇒ bị huỷ với <code>ERROR: temporary file size exceeds "temp_file_limit" (10240kB)</code>, SQLSTATE <code>53400</code>. Câu top-20 dùng top-N heapsort 26 kB ⇒ chạy bình thường. Bảng tạm tường minh không tính vào <code>temp_file_limit</code> ⇒ job chạy được kể cả với trần 1MB. Cả ba đã chạy ở mục B của thí nghiệm. Hệ quả vận hành: trần phải lớn hơn báo cáo hợp lệ lớn nhất, và nó không thay được giới hạn dung lượng cho bảng tạm.</p>
</details>

**Bài 3.** Server production giả định: <code>max_connections = 200</code>, <code>work_mem = 16MB</code>, <code>hash_mem_multiplier = 2</code>, bật <code>max_parallel_workers_per_gather = 2</code>. Tính trần bộ nhớ sort/hash nếu 50 phiên cùng chạy câu ba nút ở mục 7, mỗi nút đều chạy song song với 2 worker. Vì sao con số này không phải "RAM cần mua"?

<details><summary><b>Lời giải</b></summary>
<p>Một tiến trình: 32 + 32 + 16 = 80 MB. Mỗi phiên song song có 3 tiến trình (chính + 2 worker) ⇒ 240 MB. 50 phiên ⇒ 12.000 MB ≈ 11,7 GB — chỉ riêng sort/hash, chưa kể <code>shared_buffers</code> và bộ nhớ nền mỗi backend. Thí nghiệm D xác nhận phần "mỗi worker một Sort, mỗi Sort một hạn mức" (Worker 0, Worker 1 và tiến trình chính đều có dòng <code>Sort Method</code> riêng). Đây là <b>trần</b>: không phải nút nào cũng đầy, không phải câu nào cũng được chạy song song (thí nghiệm D: câu chỉ lấy <code>code, fare</code> thì planner không chọn song song). Nhưng nó là con số phải so với RAM trước khi đổi cấu hình; đo thực tế bằng tải giống production như cách pgbench ở mục 7.</p>
</details>

## 12. Đọc tiếp

- [PostgreSQL 18 · Resource Consumption](https://www.postgresql.org/docs/18/runtime-config-resource.html): `work_mem`, `hash_mem_multiplier`, `temp_file_limit`.
- [PostgreSQL 18 · What to Log](https://www.postgresql.org/docs/18/runtime-config-logging.html): `log_temp_files`.
- [PostgreSQL 18 · Cumulative Statistics System](https://www.postgresql.org/docs/18/monitoring-stats.html): `pg_stat_database.temp_files`, `temp_bytes`.
- [PostgreSQL 18 · pg_stat_statements](https://www.postgresql.org/docs/18/pgstatstatements.html): `temp_blks_written`.
- [PostgreSQL 18 · Using EXPLAIN](https://www.postgresql.org/docs/18/using-explain.html); release notes [PG 13](https://www.postgresql.org/docs/13/release-13.html) (HashAggregate tràn đĩa) và [PG 15](https://www.postgresql.org/docs/15/release-15.html) (`hash_mem_multiplier` mặc định 2,0).
- Lab: `p03/29-sort-hash-spill.sql`, `p03/OUTPUT.txt`, `p03/DO-DAC.md` (G6 và pgbench), `p03/thu-nghiem/pg029-spill-bai-tap.sql`.

Bài tiếp theo: [PG-030 · Thực hành: tối ưu truy vấn bằng bằng chứng](/articles/postgresql-thuc-hanh-toi-uu-truy-van).
