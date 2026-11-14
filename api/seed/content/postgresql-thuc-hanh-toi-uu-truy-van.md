---
slug: postgresql-thuc-hanh-toi-uu-truy-van
title: "Thực hành: tối ưu truy vấn dựa trên bằng chứng"
summary: "Màn CSKH của Tới Nơi mở lịch sử chuyến của khách lẻ trong 0,13 ms nhưng của khách doanh nghiệp B1 mất 208 ms. Bài thực hành cuối Phần 3: đi từ giả thuyết tới kế hoạch, sửa, đối chiếu md5, đo lặp và tính cái giá phía ghi của index."
category: postgresql
level: advanced
tags: [postgresql, pg-030, thuc-hanh, explain, index, prepared-statement, wal, toi-noi]
language: vi
---

Tổng đài viên Tới Nơi mở màn "Lịch sử chuyến đã hoàn thành" cho khách doanh nghiệp B1 (`0919990001`, 51.151 chuyến) và chờ. Cùng màn đó với một khách cá nhân thì mở ngay. Phiếu gửi đội database chỉ có một dòng: "màn lịch sử chậm với B1, thêm index giúp".

"Thêm index" có thể đúng, vô ích, hoặc làm chậm việc ghi chuyến mới lúc cao điểm. Bài này trả lời bằng một **quy trình**: đo hiện trạng, đọc kế hoạch, đặt giả thuyết và loại giả thuyết sai, thực hiện thay đổi, chứng minh kết quả không đổi, đo lại nhiều lần, tính cái giá phía ghi, và viết ra giới hạn của kết luận.

Đây là bài thực hành cuối Phần 3. Bạn nhận đề, tự làm, rồi mới mở lời giải mẫu và tự chấm theo mục 4.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-030 · Phần 3/6 · Nâng cao</span>
<p>Đọc trước: toàn bộ PG-021 … PG-029 (danh sách ở mục 6), đặc biệt <a href="/articles/postgresql-explain-ke-hoach-thuc-thi">PG-021 · Đọc EXPLAIN</a>, <a href="/articles/postgresql-index-nhieu-cot-partial-expression">PG-025 · Index nhiều cột, partial và expression</a> và <a href="/articles/postgresql-sort-hash-memory-spill">PG-029 · Sort, hash và truy vấn tràn ra đĩa</a>. Lab: <code>examples/toi-noi-postgresql/p03</code> (tệp <code>30-toi-uu-truy-van.sql</code>, số đo ở <code>DO-DAC.md</code>) và thí nghiệm <code>p03/thu-nghiem/pg030-toi-uu-bai-tap.sql</code>. Sau bài này, bạn bàn giao được kế hoạch trước/sau, fixture, phép đo lặp và chi phí ghi của index.</p>
</div>

## 1. Đề bài

### Bối cảnh

Fixture Phần 3 (nền `p03`, PostgreSQL 18.1): 1.000.000 chuyến 01/05–31/10/2026, 901.208 giao dịch `payments`, 200.003 khách. B1 có 51.151 chuyến, trong đó 44.922 hoàn thành. `trips` đã có index `ix_trips_customer_requested (customer_id, requested_at)`, `payments` đã có `ix_payments_trip (trip_id)`.

Đặc tả màn hình: **20 chuyến hoàn thành mới nhất của một khách, xếp theo giờ hoàn thành giảm dần (hoà thì mã chuyến giảm dần), kèm số tiền đã thu** = Σ charge thành công − Σ refund thành công. Câu đang chạy, viết đúng đặc tả:

```sql
PREPARE lich_su_goc(uuid) AS
SELECT t.code, t.completed_at, t.kind, t.fare,
       coalesce(sum(p.amount) FILTER (WHERE p.kind = 'charge' AND p.status = 'succeeded'), 0)
     - coalesce(sum(p.amount) FILTER (WHERE p.kind = 'refund' AND p.status = 'succeeded'), 0) AS da_thu
FROM trips AS t
LEFT JOIN payments AS p ON p.trip_id = t.id
WHERE t.customer_id = $1 AND t.status = 'completed'
GROUP BY t.id
ORDER BY t.completed_at DESC, t.code DESC
LIMIT 20;
```

### Sản phẩm bàn giao

| Mã | Sản phẩm | Yêu cầu tối thiểu |
|---|---|---|
| D1 | Hiện trạng | Kết quả tham chiếu có md5; thời gian đo lặp cho khách lớn và khách nhỏ |
| D2 | Chẩn đoán | Kế hoạch có `ANALYZE, BUFFERS`; chỉ ra nút tốn kém và vì sao |
| D3 | Giả thuyết | Ít nhất hai giả thuyết, mỗi giả thuyết có bằng chứng để giữ hoặc loại |
| D4 | Thay đổi | DDL/SQL mới; md5 kết quả trước = sau |
| D5 | Đo lại | Kế hoạch sau, số page, thời gian lặp; cả kế hoạch riêng lẫn kế hoạch chung của prepared statement |
| D6 | Cái giá | Dung lượng, WAL khi ghi, ảnh hưởng HOT |
| D7 | Giới hạn | Máy, dữ liệu, cache, điều chưa kiểm |

Không đổi đặc tả màn hình. Thay đổi dữ liệu chỉ trong giao dịch có `ROLLBACK` hoặc trên bảng thử.

## 2. Gợi ý trước khi làm

- **Đo trước khi sửa, đo nhiều lần**: trung vị + min–max, như `DO-DAC.md`.
- **Đọc kế hoạch từ trong ra** ([PG-021](/articles/postgresql-explain-ke-hoach-thuc-thi)): nút nào xử lý nhiều dòng hơn số dòng cuối cùng cần?
- **Thử với hai tham số**: khách 40 chuyến và khách 51.151 chuyến có thể có kế hoạch khác nhau ([PG-027](/articles/postgresql-statistics-analyze-cardinality)).
- **"Kết quả không đổi" do máy kiểm tra**: md5 trước và sau.
- **Index có giá** ([PG-025](/articles/postgresql-index-nhieu-cot-partial-expression)): WAL mỗi lần chèn, và cột trong index làm UPDATE mất HOT ([PG-024](/articles/heap-ctid-va-primary-key)).
- **Prepared statement có thể đổi kế hoạch sau vài lần chạy** — ghi rõ `plan_cache_mode` mỗi khi đo.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p03 --only 30                  # nền ~1–1,5 phút + bài 30, kết thúc "KẾT QUẢ: ĐẠT"
node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg030-toi-uu-bai-tap.sql
```

## 3. Lời giải mẫu

Lời giải là `30-toi-uu-truy-van.sql`. Đầu tệp đặt `SET plan_cache_mode = force_custom_plan` để mọi số đo so sánh được với nhau (lý do ở 3.6). Mở từng phần **sau khi** đã có bản của bạn.

### 3.1 D1 — Hiện trạng

<details><summary><b>Lời giải</b></summary>
<p>Kết quả tham chiếu: <code>EXECUTE lich_su_goc(:'kh_b1')</code> trả 20 dòng, từ <code>1099655 | 2026-10-31 00:20:08+07 | parcel | 73000 | 73000</code> tới <code>1099214 | 2026-10-30 21:33:18+07 | parcel | 65000 | 65000</code>. md5 của toàn bộ kết quả (ô nối <code>" | "</code>, dòng nối bằng xuống dòng) là <code>c8046204e6fa07cc57d4196a9cf7cc0e</code> — chỉ thị <code>expect-md5</code> của runner giữ con số này cho mọi bước sau.</p>
<p>Thời gian (<code>DO-DAC.md</code>, nhóm G8): Intel Core i7-10700 16 luồng, 64 GB RAM, NVMe SSD, Windows 11 Pro, PostgreSQL 18.1, <code>shared_buffers</code> 128MB, tắt song song, JIT và autovacuum; <code>EXPLAIN (ANALYZE, TIMING OFF)</code> lấy <i>Execution Time</i> phía server, cache nóng, 1 lần khởi động + 7 lần đo, trung vị.</p>
<table>
<thead><tr><th>Mã</th><th>Câu</th><th>Trung vị (ms)</th><th>Min–max</th><th>Page chạm</th></tr></thead>
<tbody>
<tr><td>G8-a</td><td>Câu gốc, B1</td><td><b>208,53</b></td><td>194,65–246,70</td><td>29.197</td></tr>
<tr><td>G8-b</td><td>Câu gốc, khách cá nhân 40 chuyến</td><td>0,13</td><td>0,13–0,18</td><td>187</td></tr>
</tbody>
</table>
<p>Chênh lệch hơn nghìn lần. Tải của máy đo khác nhau giữa các lần (CPU 22 % trước, 7 % sau): số tuyệt đối dao động ~±20 % giữa ba lần chạy lệnh đo, tỉ lệ giữ nguyên.</p>
</details>

### 3.2 D2 — Chẩn đoán: 29.197 page cho 20 dòng

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-text"> Limit (actual rows=20.00 loops=1)
   Buffers: shared hit=13904 read=15293
   -&gt;  Sort (actual rows=20.00 loops=1)
         Sort Key: t.completed_at DESC, t.code DESC
         Sort Method: top-N heapsort  Memory: 29kB
         -&gt;  HashAggregate (actual rows=44922.00 loops=1)
               Group Key: t.id
               -&gt;  Hash Right Join (actual rows=45722.00 loops=1)
                     Hash Cond: (p.trip_id = t.id)
                     -&gt;  Seq Scan on payments p (actual rows=901208.00 loops=1)
                           Buffers: shared hit=143 read=10660
                     -&gt;  Hash (actual rows=44922.00 loops=1)
                           -&gt;  Bitmap Heap Scan on trips t (actual rows=44922.00 loops=1)
                                 Filter: (status = 'completed'::text)
                                 Rows Removed by Filter: 6229
                                 Heap Blocks: exact=18139
                                 -&gt;  Bitmap Index Scan on ix_trips_customer_requested (actual rows=51151.00 loops=1)</code></pre>
<p>Đọc từ trong ra: lấy <b>hết</b> 51.151 chuyến của B1 (18.139 page heap — chuyến của B1 rải khắp bảng), lọc còn 44.922; dựng bảng băm; <b>quét cả 901.208 giao dịch</b> (10.803 page) để nối; gom nhóm 44.922 chuyến; sắp xếp, lấy 20 rồi bỏ 44.902 dòng. Thủ phạm là <b>thứ tự công việc</b>: không có đường truy cập "chuyến hoàn thành của một khách theo <code>completed_at</code>", nên planner phải làm mọi việc cho mọi chuyến trước khi biết cần 20 chuyến nào.</p>
<p>Khách cá nhân: planner ước khoảng một chục chuyến, chọn Nested Loop (36 chuyến, tra <code>ix_payments_trip</code> 36 lần, ~190 page; kế hoạch biến động theo ANALYZE). Thử với tham số nhỏ sẽ không thấy vấn đề.</p>
</details>

### 3.3 D3 — Giả thuyết: giữ một, loại hai

<details><summary><b>Lời giải</b></summary>
<p><b>Giả thuyết 1 — "sắp theo <code>requested_at</code>, đã có index sẵn".</b> Nhanh, nhưng có đúng đặc tả không? So trang đầu của hai thứ tự: <code>EXCEPT</code> ra <b>0</b> chuyến khác — trông như đúng. Xét cả lịch sử của B1 bằng hai <code>row_number()</code>: <b>35.026 / 44.922</b> chuyến đổi vị trí. Chuyến đặt trước có thể xong sau; trang 2, 3… sẽ sai. <b>Loại</b> — và nhớ rằng bằng chứng trên 20 dòng đầu chưa đủ.</p>
<p><b>Giả thuyết 2 — "thêm index <code>payments(trip_id, status)</code>".</b> <code>payments</code> đã có <code>ix_payments_trip</code>; planner quét cả bảng chỉ vì phải nối cho 44.922 chuyến một lúc, khi đó băm cả bảng rẻ hơn tra 44.922 lần. Index mới trên <code>payments</code> không đổi được số chuyến phải xử lý. <b>Loại.</b></p>
<p><b>Giả thuyết 3 — "index đúng đường truy cập là đủ".</b> Tạo index partial (mục 3.4) nhưng <i>không</i> viết lại câu. Kế hoạch:</p>
<pre><code class="language-text">                     -&gt;  Seq Scan on payments p (actual rows=901208.00 loops=1)
                     ...
                                 -&gt;  Bitmap Index Scan on ix_trips_khach_hoan_thanh (actual rows=44922.00 loops=1)</code></pre>
<p>Planner chỉ dùng index mới thay cho bitmap cũ để lấy 44.922 chuyến, rồi vẫn quét payments: 28.679 page. Đo (G8-f): trung vị <b>247,02 ms</b> (227,87–310,48), không nhanh hơn câu gốc. Index <b>cần</b> nhưng <b>chưa đủ</b>: câu phải được viết để "lấy 20 trước, tính tiền sau".</p>
</details>

### 3.4 D4 — Thay đổi

<details><summary><b>Lời giải</b></summary>
<p>(a) Index theo đúng đường truy cập: khách, rồi <code>completed_at</code> giảm dần, hoà thì <code>code</code> giảm dần — đúng thứ tự <code>ORDER BY</code>; partial <code>WHERE status = 'completed'</code> bỏ 12 % chuyến huỷ/đang chạy khỏi index. (b) Viết lại: lấy 20 chuyến trong subquery có <code>LIMIT</code>, rồi tính tiền cho đúng 20 chuyến đó bằng <code>LATERAL</code>.</p>
<pre><code class="language-sql">CREATE INDEX ix_trips_khach_hoan_thanh ON trips (customer_id, completed_at DESC, code DESC)
WHERE status = 'completed';
PREPARE lich_su_moi(uuid) AS
SELECT t.code, t.completed_at, t.kind, t.fare, tien.da_thu
FROM (
  SELECT id, code, completed_at, kind, fare
  FROM trips
  WHERE customer_id = $1 AND status = 'completed'
  ORDER BY completed_at DESC, code DESC
  LIMIT 20
) AS t
CROSS JOIN LATERAL (
  SELECT coalesce(sum(p.amount) FILTER (WHERE p.kind = 'charge' AND p.status = 'succeeded'), 0)
       - coalesce(sum(p.amount) FILTER (WHERE p.kind = 'refund' AND p.status = 'succeeded'), 0) AS da_thu
  FROM payments AS p WHERE p.trip_id = t.id
) AS tien
ORDER BY t.completed_at DESC, t.code DESC;</code></pre>
<p><b>Kết quả không đổi:</b> <code>EXECUTE lich_su_moi(:'kh_b1')</code> có md5 <code>c8046204e6fa07cc57d4196a9cf7cc0e</code> — trùng D1; câu gốc chạy lại khi đã có index cũng cùng md5. Với khách cá nhân, so 20 dòng của hai cách bằng <code>EXCEPT</code>: 0 dòng khác. LATERAL giữ đúng tiền vì aggregate không <code>GROUP BY</code> luôn trả một dòng, kể cả chuyến không có giao dịch, và <code>coalesce</code> đổi NULL thành 0 — tương đương <code>LEFT JOIN … GROUP BY t.id</code>.</p>
</details>

### 3.5 D5 — Đo lại: 29.197 → 98 page

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-text"> Nested Loop (actual rows=20.00 loops=1)
   Buffers: shared hit=71 read=27
   -&gt;  Limit (actual rows=20.00 loops=1)
         Buffers: shared hit=18
         -&gt;  Index Scan using ix_trips_khach_hoan_thanh on trips (actual rows=20.00 loops=1)
               Index Cond: (customer_id = '239ec4d4-ca1d-4ae9-8459-a2f5227a39c1'::uuid)
   -&gt;  Aggregate (actual rows=1.00 loops=20)
         -&gt;  Index Scan using ix_payments_trip on payments p (actual rows=1.00 loops=20)
               Index Searches: 20
               Buffers: shared hit=53 read=27</code></pre>
<p>Đi index theo đúng thứ tự, dừng sau 20 mục (18 page), tra payments 20 lần (80 page): <b>98 page</b>, không còn nút Sort. Thời gian (G8, cùng điều kiện D1):</p>
<table>
<thead><tr><th>Mã</th><th>Câu</th><th>Trung vị (ms)</th><th>Min–max</th></tr></thead>
<tbody>
<tr><td>G8-a</td><td>Trước — câu gốc, B1</td><td>208,53</td><td>194,65–246,70</td></tr>
<tr><td>G8-f</td><td>Câu gốc + index mới, B1</td><td>247,02</td><td>227,87–310,48</td></tr>
<tr><td>G8-d</td><td>Sau — câu viết lại + index, B1</td><td><b>0,07</b></td><td>0,07–0,10</td></tr>
<tr><td>G8-e</td><td>Sau — khách cá nhân</td><td>0,11</td><td>0,07–0,27</td></tr>
</tbody>
</table>
<p>Với B1, nhanh khoảng 3.000 lần; với khách cá nhân, xấp xỉ như cũ (0,13 → 0,11 ms, trong ngưỡng nhiễu). Số page tất định hơn thời gian: dùng 29.197 → 98 làm bằng chứng chính, thời gian làm bằng chứng phụ.</p>
</details>

### 3.6 D5 (tiếp) — Kế hoạch chung của prepared statement

<details><summary><b>Lời giải</b></summary>
<p>Ứng dụng thường dùng prepared statement. Với <code>plan_cache_mode = auto</code> (mặc định), tài liệu <code>PREPARE</code> mô tả luật hiện hành: 5 lần thực thi đầu dùng <b>kế hoạch riêng</b> (custom — lập lại với giá trị tham số cụ thể), sau đó lập <b>kế hoạch chung</b> (generic — không biết tham số) và dùng nó nếu chi phí ước lượng không cao hơn trung bình kế hoạch riêng quá nhiều. Kế hoạch chung hiện <code>$1</code> trong <code>EXPLAIN</code>.</p>
<p>Lab ép <code>force_generic_plan</code> để xem kế hoạch chung với B1:</p>
<table>
<thead><tr><th>Câu</th><th>Kế hoạch chung (lần chạy ghi OUTPUT)</th><th>Page</th></tr></thead>
<tbody>
<tr><td>Câu viết lại</td><td>Bitmap Heap Scan 44.922 chuyến → Sort top-N → 20 lần tra payments</td><td>17.956</td></tr>
<tr><td>Câu gốc</td><td>Nested Loop: Index Scan 44.922 chuyến, tra <code>ix_payments_trip</code> 44.922 lần, Sort <code>external merge Disk: 3360kB</code></td><td>211.646</td></tr>
</tbody>
</table>
<p>Kế hoạch chung lập cho khách "trung bình" (ước khoảng 5–12 chuyến, từ <code>n_distinct</code> lấy mẫu) nên sai với B1, và <b>bấp bênh</b>: trong lần chạy lệnh đo, kế hoạch chung của câu viết lại hoá ra là Index Scan + Limit, 98 page, 0,07 ms (G8-h); khi dựng lab đã gặp cả hai dạng. Câu gốc ở kế hoạch chung đo được 419,62 ms (G8-i, 398,31–463,71, 5 lần). Lần đo đầu tiên của lab, khi chưa ép <code>force_custom_plan</code>, đã trộn hai loại kế hoạch vào cùng một phép đo (trung vị 285 ms, max 743 ms) — lý do tệp bài ép kế hoạch riêng ngay dòng đầu.</p>
<p>Kết luận "98 page" vì vậy chỉ chắc chắn với kế hoạch riêng. Cách xử lý tuỳ ứng dụng: <code>force_custom_plan</code> cho câu/phiên này (tốn thời gian lập kế hoạch mỗi lần), tách một đường xử lý riêng cho khách doanh nghiệp, hoặc không dùng prepared statement có tên.</p>
</details>

### 3.7 D6 — Cái giá phía ghi

<details><summary><b>Lời giải</b></summary>
<p><b>Dung lượng.</b> <code>ix_trips_khach_hoan_thanh</code> = 43.712.512 B (43,7 MB), lớn hơn <code>ix_trips_customer_requested</code> có sẵn (40.624.128 B) dù bỏ 12 % dòng: khoá rộng hơn (thêm <code>code</code>). Partial index không đồng nghĩa với nhỏ hơn.</p>
<p><b>WAL khi nạp chuyến mới.</b> Chép 5.000 chuyến hoàn thành sang ngày 02/11 trong giao dịch rồi <code>ROLLBACK</code>, đo bằng <code>EXPLAIN (ANALYZE, WAL)</code> ngay sau <code>CHECKPOINT</code>:</p>
<pre><code class="language-text">-- không có index mới
 Insert on trips (actual rows=0.00 loops=1)
   WAL: records=30262 fpi=6824 bytes=48951170 buffers full=5124
-- có index mới
 Insert on trips (actual rows=0.00 loops=1)
   WAL: records=38677 fpi=9912 bytes=72780993 buffers full=6233</code></pre>
<p>+8.415 bản ghi, +3.088 ảnh page, 48,95 → 72,78 MB (+49 %). Ngay sau <code>CHECKPOINT</code> mỗi page bị chạm lần đầu được ghi nguyên ảnh vào WAL (full page image), nên con số lớn hơn lúc ghi ổn định giữa hai checkpoint; dùng nó để <b>so</b> hai phương án, đừng nhân lên thành dung lượng WAL mỗi ngày. Thời gian ghi 5.000 chuyến (G8-c/G8-g, đo trong giao dịch rồi ROLLBACK, không gồm commit): 693,62 vs 699,59 ms — chênh nhỏ hơn nhiễu, nên bằng chứng chi phí ghi là WAL (tất định), không phải đồng hồ.</p>
<p><b>Mất HOT.</b> Index mới chứa <code>completed_at</code> và có <code>status</code> trong mệnh đề <code>WHERE</code> — hai cột đổi đúng lúc chuyến hoàn thành. Lab chính thử trên bảng 10.000 chuyến fillfactor 90: 200 lần "đánh dấu hoàn thành" trước khi có index mới và 200 lần sau, tổng <code>n_tup_upd 400, n_tup_hot_upd 200</code>. Thí nghiệm tách từng đợt (fillfactor 50 để page chắc còn chỗ): trước index mới <code>200 | 200</code>, sau index mới <code>400 | 200</code> — đợt sau không lần nào HOT. Hồ sơ Tới Nơi có khoảng 70 chuyến mới/giây và 560 UPDATE/giây lúc cao điểm vào bảng này: mỗi UPDATE mất HOT phải ghi thêm mục vào <i>mọi</i> index của bảng (PG-025). Đó là chi phí phải đo trên tải ghi trước khi bật, không chỉ trên màn CSKH.</p>
</details>

### 3.8 D7 — Giới hạn và bản bàn giao

<details><summary><b>Lời giải</b></summary>
<ul>
<li>Một máy, một phiên, cache nóng, dữ liệu tổng hợp 1 triệu chuyến; production có tải đồng thời, cache khác, phân bố khách khác.</li>
<li>Cải thiện lớn chỉ với khách đặt rất nhiều; khách thường vốn đã nhanh.</li>
<li>Kết luận thời gian đúng cho kế hoạch riêng; kế hoạch chung có thể khác giữa các lần ANALYZE.</li>
<li>Chi phí ghi đo bằng WAL ngay sau CHECKPOINT và HOT trên bảng thử; chưa đo throughput ghi ở 70 chèn + 560 sửa mỗi giây.</li>
<li>Nếu nghiệp vụ chấp nhận "mới nhất theo giờ đặt", dùng index có sẵn rẻ hơn thêm index — nhưng đó là đổi đặc tả, quyết định của nghiệp vụ.</li>
</ul>
<p>Bàn giao: <code>30-toi-uu-truy-van.sql</code> (chỉ thị <code>expect-md5</code>, <code>expect-match</code>, <code>expect-range</code> ở mọi bước), phần PG-030 của <code>OUTPUT.txt</code>, nhóm G8 của <code>DO-DAC.md</code> và danh sách trên. Người nhận chạy lại bằng một lệnh và thấy cùng md5.</p>
</details>

## 4. Tiêu chí tự chấm

Mỗi tiêu chí 10 điểm; từ 80 điểm là đạt để sang Phần 4; tiêu chí 2, 4 và 7 bắt buộc.

| # | Tiêu chí | Đạt khi |
|---|---|---|
| 1 | Hiện trạng có số | Đo lặp, trung vị + min–max, ghi máy, cache |
| 2 | Kết quả tham chiếu | md5 của câu gốc trước khi sửa bất cứ gì |
| 3 | Chẩn đoán đúng nút | Chỉ ra quét cả payments và xử lý 44.922 chuyến để lấy 20 |
| 4 | Loại giả thuyết dựa trên bằng chứng | Có ít nhất một giả thuyết bị loại bằng số (35.026 chuyến đổi vị trí, hoặc 247 vs 208 ms) |
| 5 | Thay đổi tối thiểu | Một index + viết lại một câu; không đổi đặc tả |
| 6 | Đo lại cùng điều kiện | Cùng máy, cùng cache, cùng `plan_cache_mode`; page trước/sau |
| 7 | md5 không đổi | Trước = sau, cho ít nhất khách lớn và một khách nhỏ |
| 8 | Kế hoạch chung | Có xem kế hoạch generic và ghi rõ kế hoạch này bấp bênh |
| 9 | Chi phí ghi | WAL + HOT, có nói giới hạn của số WAL sau CHECKPOINT |
| 10 | Giới hạn | Liệt kê điều chưa kiểm, không suy rộng sang production |

## 5. Bẫy hay gặp khi tự làm

- **Đo một lần, trên khách nhỏ.** Khách 40 chuyến không bao giờ chậm; vấn đề nằm ở đuôi phân bố.
- **So 20 dòng đầu rồi kết luận "kết quả như nhau".** Giả thuyết 1 khớp trang đầu nhưng sai 35.026 vị trí.
- **Tạo index rồi dừng.** Câu gốc + index mới vẫn quét cả payments (G8-f).
- **Đo prepared statement mà không ghi `plan_cache_mode`.** Sau 5 lần thực thi, kế hoạch có thể đổi giữa chừng phép đo.
- **Nghĩ partial index luôn nhỏ, luôn rẻ.** 43,7 MB và mất HOT ở đúng UPDATE quan trọng nhất của bảng.

## 6. Tổng kết Phần 3

| Bài | Bạn đã có gì cho sản phẩm cuối phần |
|---|---|
| [PG-021 · Đọc EXPLAIN](/articles/postgresql-explain-ke-hoach-thuc-thi) | Đọc kế hoạch từ trong ra, `ANALYZE, BUFFERS`, ước lượng vs thực tế |
| [PG-022 · Page — đơn vị đọc ghi](/articles/page-don-vi-doc-ghi) | Đếm page thay vì đếm mili giây |
| [PG-023 · B-tree index](/articles/postgresql-btree-index-co-ban) | Đường tìm dữ liệu có/không index, độ chọn lọc |
| [PG-024 · Heap, ctid và primary key](/articles/heap-ctid-va-primary-key) | HOT, fillfactor — nền cho chi phí ghi ở 3.7 |
| [PG-025 · Index nhiều cột, partial, expression](/articles/postgresql-index-nhieu-cot-partial-expression) | Thứ tự cột theo truy vấn; WAL của index khi ghi |
| [PG-026 · Index-only scan và visibility map](/articles/postgresql-index-only-scan-visibility-map) | Khi nào index đủ cột để không ghé heap |
| [PG-027 · Statistics và cardinality](/articles/postgresql-statistics-analyze-cardinality) | Vì sao khách nhỏ và B1 có kế hoạch khác, kế hoạch chung sai với B1 |
| [PG-028 · GIN, GiST và BRIN](/articles/postgresql-gin-gist-brin-lua-chon) | Không phải truy vấn nào cũng cần B-tree |
| [PG-029 · Sort, hash và tràn đĩa](/articles/postgresql-sort-hash-memory-spill) | Sort top-N, Hash Join, bộ nhớ nhân lên theo phiên |

Sản phẩm của phần đúng như khung: báo cáo trước/sau index, dữ liệu đo và giới hạn kết luận. **Chưa** có: hai giao dịch cùng sửa một chuyến, row lock, isolation level (Phần 4); autovacuum, quyền truy cập (Phần 5).

## 7. Bài tập mở rộng

Lời giải chạy được ở `p03/thu-nghiem/pg030-toi-uu-bai-tap.sql` (không có trong lab chính).

**Bài 1.** Màn hình có nút "Trang sau". Viết trang 2 bằng keyset trên index mới, chứng minh trùng kết quả `OFFSET 20 LIMIT 20`, và so số page ở trang 200.

<details><summary><b>Lời giải</b></summary>
<p>Mốc là dòng cuối trang 1 (<code>2026-10-30 21:33:18+07</code>, <code>1099214</code>). Hai cột cùng <code>DESC</code> nên so sánh theo bộ giá trị (row comparison) đúng chiều (<a href="/articles/postgresql-pagination-keyset">PG-018</a>):</p>
<pre><code class="language-sql">SELECT code, completed_at FROM trips
WHERE customer_id = :'kh_b1' AND status = 'completed'
  AND (completed_at, code) &lt; (:'moc_ht'::timestamptz, :'moc_ma'::int)
ORDER BY completed_at DESC, code DESC LIMIT 20;</code></pre>
<p>md5 của 20 dòng bằng md5 của bản <code>OFFSET 20</code> (<code>t</code>). Kế hoạch: <code>Index Only Scan using ix_trips_khach_hoan_thanh</code> với <code>Index Cond: … ROW(completed_at, code) &lt; ROW(…)</code>, không Sort, <code>Heap Fetches: 0</code>. Ở trang 200 (<code>OFFSET 3980</code>): OFFSET chạm <b>29</b> page, keyset <b>5</b> — OFFSET phải đi qua 4.000 mục rồi bỏ 3.980. Chênh lệch tăng theo số trang.</p>
</details>

**Bài 2.** Ở 3.7, UPDATE đổi cả `status` (trong mệnh đề `WHERE` của index) lẫn `completed_at` (cột khoá). Nếu chỉ đổi `status`, UPDATE có còn HOT không?

<details><summary><b>Lời giải</b></summary>
<p>Bảng thử 10.000 chuyến (<code>CREATE TABLE … AS</code> không chép CHECK), fillfactor 50 để page chắc còn chỗ, bốn đợt 200 UPDATE trên các chuyến huỷ khác nhau; cột cuối là <code>n_tup_upd | n_tup_hot_upd</code> luỹ kế:</p><table><thead><tr><th>Đợt</th><th>Index</th><th>Đổi cột</th><th>Luỹ kế</th></tr></thead><tbody><tr><td>1</td><td>chỉ <code>(customer_id, requested_at DESC)</code></td><td><code>status</code> + <code>completed_at</code></td><td>200 | 200</td></tr><tr><td>2</td><td>+ index partial mới</td><td><code>status</code> + <code>completed_at</code></td><td>400 | 200</td></tr><tr><td>3</td><td>như 2</td><td>chỉ <code>status</code></td><td>600 | 200</td></tr><tr><td>4</td><td>như 2</td><td>chỉ <code>fare</code></td><td>800 | 400</td></tr></tbody></table><p>Đợt 3: không lần nào HOT dù <code>completed_at</code> giữ nguyên. Đợt 4 là đối chứng: cột ngoài mọi index vẫn HOT khi đã có index mới. Cột chỉ xuất hiện trong mệnh đề <code>WHERE</code> của partial index cũng được tính là "cột có index" khi xét HOT — tài liệu chỉ nói "các cột được index tham chiếu", thí nghiệm cho thấy điều kiện partial thuộc nhóm đó.</p>
</details>

**Bài 3.** Không ép gì (`plan_cache_mode = auto`), chạy `EXECUTE lich_su_moi` với B1 bảy lần. Đếm kế hoạch riêng/chung và chứng minh kết quả không đổi.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT name, custom_plans, generic_plans FROM pg_prepared_statements;</code></pre>
<p>Lần chạy thí nghiệm: <code>lich_su_moi | 5 | 2</code> — 5 lần riêng như luật, 2 lần sau chuyển sang chung. Cách chia sau lần thứ 5 phụ thuộc chi phí ước lượng của kế hoạch chung (tức là ANALYZE lấy mẫu), nên thí nghiệm chỉ kiểm tổng = 7 và số kế hoạch riêng trong [5, 7]. Cả bảy lần, và thêm một lần ép <code>force_generic_plan</code>, đều có md5 <code>c8046204e6fa07cc57d4196a9cf7cc0e</code>: kế hoạch đổi, kết quả không đổi — điều planner luôn phải giữ, và là lý do md5 là bằng chứng "đúng" còn số page/thời gian là bằng chứng "nhanh".</p>
</details>

## 8. Đọc tiếp

- [PostgreSQL 18 · PREPARE](https://www.postgresql.org/docs/18/sql-prepare.html): luật chọn kế hoạch riêng/chung.
- [PostgreSQL 18 · Query Planning (`plan_cache_mode`)](https://www.postgresql.org/docs/18/runtime-config-query.html).
- [PostgreSQL 18 · pg_prepared_statements](https://www.postgresql.org/docs/18/view-pg-prepared-statements.html): `custom_plans`, `generic_plans`.
- [PostgreSQL 18 · Heap-Only Tuples (HOT)](https://www.postgresql.org/docs/18/storage-hot.html).
- Lab: `p03/30-toi-uu-truy-van.sql`, `p03/OUTPUT.txt`, `p03/DO-DAC.md` (G8), `p03/README.md` (ghi chú về kế hoạch biến động), `p03/thu-nghiem/pg030-toi-uu-bai-tap.sql`.

Bài tiếp theo — mở đầu Phần 4 · Giao dịch, MVCC và đồng thời: [PG-031: Transaction, COMMIT và ROLLBACK](/articles/postgresql-transaction-commit-rollback)
