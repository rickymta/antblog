---
slug: postgresql-explain-ke-hoach-thuc-thi
title: "Đọc EXPLAIN từ truy vấn tìm chuyến"
summary: "Màn «Lịch sử chuyến» của Tới Nơi chạy cùng một câu SQL cho khách lẻ và khách doanh nghiệp B1, nhưng PostgreSQL chọn hai cách đọc khác nhau. Học đọc EXPLAIN: kiểu scan, kiểu join, ước lượng và chỗ chênh lệch giữa kế hoạch và thực thi."
category: postgresql
level: beginner
tags: [postgresql, pg-021, explain, explain-analyze, query-plan, buffers, planner, toi-noi]
language: vi
---

Màn «Lịch sử chuyến» của Tới Nơi chỉ chạy một câu: 20 chuyến gần nhất của một khách. Kỹ sư A dán kế hoạch của khách doanh nghiệp B1 lên kênh chat — `Index Scan`, 10 page — và kết luận index đang chạy tốt. Kỹ sư B dán kế hoạch của một khách lẻ — `Bitmap Heap Scan` rồi `Sort`, 44 page — và kết luận index «bị bỏ qua». Cả hai đúng về những gì họ thấy, sai về kết luận. Cùng tuần đó, một bạn mới gõ `EXPLAIN ANALYZE DELETE …` trên môi trường thử «để xem kế hoạch» và mất 68 dòng dữ liệu.

Ba chuyện đó có chung một gốc: đọc `EXPLAIN` mà không biết dòng nào là **dự đoán**, dòng nào là **đo thật**, và lệnh nào **chạy thật câu lệnh**. Bài này mở đầu Phần 3 bằng đúng truy vấn tìm chuyến ấy.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-021 · Phần 3/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-thuc-hanh-sql-doi-soat">PG-020 · Thực hành truy vấn vận hành và đối soát</a>. Lab: <code>examples/toi-noi-postgresql/p03</code> (tệp <code>21-explain-tim-chuyen.sql</code>). Sau bài này, bạn nhận diện được kiểu scan, kiểu join, đọc được ước lượng so với thực tế và phân biệt kế hoạch với thực thi.</p>
</div>

## 1. Lab Phần 3: PostgreSQL 18.1 thật, 1 triệu chuyến

Phần 1–2 chạy trên PGlite (PostgreSQL 17.5) với vài chục chuyến — quá nhỏ để kế hoạch có ý nghĩa. Từ Phần 3, lab chạy trên **PostgreSQL 18.1 cài thật**: runner dựng một cluster tạm, nạp `trips` **1.000.000 chuyến** (19.441 page, 152 MB — lớn hơn `shared_buffers` 128 MB), 200.003 khách, 6.000 tài xế, 901.208 giao dịch.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p03 --only 21     # dựng nền (~1 phút) + chạy 21-explain-tim-chuyen.sql
```

Cluster lab tắt truy vấn song song và JIT để kế hoạch tái lập được. Output chép từ `p03/OUTPUT.txt` (lệnh dùng `TIMING OFF, SUMMARY OFF` vì runner che thời gian). Hai khách dùng xuyên bài: khách lẻ `0910000500` (40 chuyến) và B1 `0919990001` (51.151 chuyến), id lưu vào biến psql `:'kh_le'`, `:'kh_b1'` bằng `\gset`.

## 2. EXPLAIN: kế hoạch là một cây các nút

**Planner** (bộ lập kế hoạch) nhận câu SQL và chọn một **kế hoạch thực thi** (execution plan): đọc bảng theo cách nào, nối bảng theo thứ tự nào. `EXPLAIN` in kế hoạch đó **mà không chạy câu lệnh**:

```sql
EXPLAIN
SELECT code, requested_at, status, fare
FROM trips WHERE customer_id = :'kh_le'
ORDER BY requested_at DESC LIMIT 20;
```

```text
 Limit  (cost=48.05..48.08 rows=11 width=30)
   ->  Sort  (cost=48.05..48.08 rows=11 width=30)
         Sort Key: requested_at DESC
         ->  Bitmap Heap Scan on trips  (cost=4.51..47.86 rows=11 width=30)
               Recheck Cond: (customer_id = '3b3bf766-…'::uuid)
               ->  Bitmap Index Scan on ix_trips_customer_requested  (cost=0.00..4.51 rows=11 width=0)
                     Index Cond: (customer_id = '3b3bf766-…'::uuid)
```

Mỗi dòng có `->` là một **nút** (node). Nút thụt vào sâu hơn là nút con: nút này chạy trước và đưa dòng lên nút cha. Đọc **từ trong ra ngoài, từ dưới lên**: quét index lấy vị trí dòng → đọc bảng → sắp → cắt 20 dòng.

| Trường | Nghĩa |
|---|---|
| `cost=4.51..47.86` | Chi phí **khởi động** (trước khi trả dòng đầu tiên) .. chi phí **tổng** nếu nút chạy tới hết. Đơn vị tương đối, quy ước đọc tuần tự một page = 1 — **không phải mili giây** |
| `rows=11` | Số dòng nút **ước lượng** sẽ trả ra (không phải số dòng nút đọc) |
| `width=30` | Độ rộng trung bình một dòng đầu ra, byte |

Chi phí nút cha đã gồm chi phí nút con. Chú ý `rows=11`: khách này thật ra có **40** chuyến. Các số trong ngoặc được tính từ thống kê do `ANALYZE` lấy mẫu 30.000 dòng, nên máy bạn sẽ lệch đôi chút. Muốn so **hình dạng** kế hoạch, dùng `EXPLAIN (COSTS OFF)`: cùng cây, bỏ phần trong ngoặc.

## 3. EXPLAIN ANALYZE: chạy thật và đo

Thêm `ANALYZE`, PostgreSQL **chạy câu lệnh** rồi ghi số thật cạnh từng nút. Ở PostgreSQL 18, `EXPLAIN ANALYZE` tự kèm `BUFFERS` (bản 17 trở về trước phải ghi thêm `BUFFERS`). Lần đầu, trên database vừa chép (page của nó chưa nằm trong `shared_buffers`):

```sql
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, requested_at, status, fare
FROM trips WHERE customer_id = :'kh_le'
ORDER BY requested_at DESC LIMIT 20;
```

```text
 Limit (actual rows=20.00 loops=1)
   Buffers: shared hit=3 read=44
   ->  Sort (actual rows=20.00 loops=1)
         Sort Key: requested_at DESC
         Sort Method: quicksort  Memory: 27kB
         Buffers: shared hit=3 read=44
         ->  Bitmap Heap Scan on trips (actual rows=40.00 loops=1)
               Recheck Cond: (customer_id = '3b3bf766-…'::uuid)
               Heap Blocks: exact=40
               Buffers: shared read=44
               ->  Bitmap Index Scan on ix_trips_customer_requested (actual rows=40.00 loops=1)
                     Index Cond: (customer_id = '3b3bf766-…'::uuid)
                     Index Searches: 1
                     Buffers: shared read=4
 Planning:
   Buffers: shared hit=149 read=23
```

Chạy lại y nguyên câu đó, mọi dòng `Buffers` đổi thành `shared hit=44`, không còn `read`.

- **`actual rows=40.00 loops=1`**: số dòng thật mà nút trả ra, **trung bình mỗi lần chạy**; `loops` = số lần nút chạy. PostgreSQL 18 in hai chữ số thập phân vì đây là giá trị trung bình (mục 6 có `0.93`).
- **`Buffers: shared hit=… read=…`**: số page 8 KB mà nút (gồm cả nút con) đã chạm. `hit` = có sẵn trong `shared_buffers`; `read` = phải xin hệ điều hành — **chưa chắc là đọc đĩa**, page có thể nằm trong cache của hệ điều hành. [PG-022](/articles/page-don-vi-doc-ghi) đào sâu page và buffer.
- **`Heap Blocks: exact=40`**: 40 chuyến trên 40 page bảng khác nhau; cộng 4 page index = 44 page cho 20 dòng.
- **`Planning: Buffers`**: page catalog đọc lúc lập kế hoạch.

Thời gian lấy từ lệnh đo của lab (`p03/DO-DAC.md`: i7-10700, 64 GB RAM, NVMe SSD, Windows 11, PostgreSQL 18.1, trung vị 7 lần, máy có tải nền nên nhiễu khoảng ±20 %): **0,04 ms** khi page đã trong `shared_buffers`, **0,32 ms** khi vừa đẩy page khỏi `shared_buffers` (cache hệ điều hành vẫn nóng). Số của một máy, dữ liệu tổng hợp — so tỉ lệ, đừng so tuyệt đối.

## 4. Cùng câu SQL, khác tham số, khác kế hoạch

Câu y hệt, đổi sang khách B1:

```text
 Limit (actual rows=20.00 loops=1)
   Buffers: shared hit=1 read=9
   ->  Index Scan using ix_trips_customer_requested on trips (actual rows=20.00 loops=1)
         Index Cond: (customer_id = '239ec4d4-…'::uuid)
         Index Searches: 1
         Buffers: shared hit=1 read=9
```

Thống kê cho planner biết B1 có rất nhiều chuyến (B1 nằm trong danh sách giá trị phổ biến, khoảng 51 nghìn chuyến). Đọc hết rồi sắp thì quá đắt; đi index `(customer_id, requested_at DESC)` theo đúng thứ tự cần và **dừng sau 20 dòng** thì rẻ — 10 page. Với khách lẻ, planner ước khoảng 11 chuyến: gom vị trí bằng bitmap rồi sắp 11 dòng rẻ hơn. Cả hai kế hoạch đều dùng `ix_trips_customer_requested`; kỹ sư B kết luận «index bị bỏ qua» chỉ vì không nhận ra `Bitmap Index Scan` cũng là dùng index.

Bài học cho cuộc tranh luận ở đầu bài: **một kế hoạch chỉ đúng cho tham số đã dùng** — dán kế hoạch thì ghi kèm tham số. Kế hoạch của khách lẻ dựa trên ước lượng lấy mẫu, một lần `ANALYZE` khác có thể nghiêng sang `Index Scan`. [PG-027](/articles/postgresql-statistics-analyze-cardinality) giải thích ước lượng 11 từ đâu ra.

## 5. Ba kiểu đọc bảng

| Nút | Cách đọc | Hợp khi |
|---|---|---|
| `Seq Scan` | Đọc **mọi page** của bảng, lọc từng dòng | Lấy phần lớn bảng, hoặc không có index phù hợp |
| `Index Scan` | Đi cây index, mỗi mục khớp thì đọc page bảng chứa dòng đó | Ít dòng, hoặc cần đúng thứ tự của index (như B1 + `LIMIT`) |
| `Bitmap Index Scan` + `Bitmap Heap Scan` | Gom **mọi** vị trí khớp thành bitmap theo page, rồi đọc mỗi page bảng **một lần** theo thứ tự page | Vài nghìn dòng rải trên nhiều page |

Còn `Index Only Scan` — trả lời chỉ từ index, không ghé bảng — gặp ở mục 6 và được mổ xẻ ở [PG-026](/articles/postgresql-index-only-scan-visibility-map).

**Seq Scan.** Không có index trên `fare`:

```sql
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips WHERE fare >= 150000;
```

```text
 Aggregate (actual rows=1.00 loops=1)
   Buffers: shared hit=47 read=19394
   ->  Seq Scan on trips (actual rows=3541.00 loops=1)
         Filter: (fare >= 150000)
         Rows Removed by Filter: 996459
         Buffers: shared hit=47 read=19394
```

`Filter` là điều kiện áp **sau khi** đọc dòng lên; `Rows Removed by Filter: 996459` = số dòng đọc rồi bỏ. 19.441 page cho 3.541 dòng. Chạy lần hai vẫn `hit=190 read=19251`: với bảng lớn hơn một phần tư `shared_buffers`, Seq Scan chỉ dùng một vòng đệm nhỏ (256 kB) để khỏi đẩy dữ liệu khác ra khỏi cache — nên bảng này không «ấm lên» như bảng nhỏ.

**Index Scan.** Dải mã chuyến nhỏ: `WHERE code BETWEEN 600000 AND 600050` ra `Index Scan using uq_trips_code on trips` với `Index Cond: ((code >= 600000) AND (code <= 600050))`. **`Index Cond`** là điều kiện dùng để **đi cây index** — khác hẳn `Filter`.

**Bitmap.** Truy vấn các chuyến ở Đà Nẵng trong 7 ngày:

```sql
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*), sum(fare) FROM trips
WHERE city_id = 3 AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07'
                  AND requested_at <  TIMESTAMPTZ '2026-10-08 00:00+07';
```

```text
 Aggregate (actual rows=1.00 loops=1)
   ->  Bitmap Heap Scan on trips (actual rows=5477.00 loops=1)
         Recheck Cond: ((city_id = 3) AND (requested_at >= …) AND (requested_at < …))
         Heap Blocks: exact=827
         Buffers: shared hit=4 read=850
         ->  Bitmap Index Scan on ix_trips_city_requested (actual rows=5477.00 loops=1)
               Index Cond: ((city_id = 3) AND (requested_at >= …) AND (requested_at < …))
               Buffers: shared hit=3 read=24
```

5.477 dòng trên 827 page bảng (`Heap Blocks: exact=827`), 27 page index. `Recheck Cond` là điều kiện được kiểm tra lại trên dòng đã đọc khi cần (bitmap quá lớn thì phải chuyển sang chế độ chỉ nhớ page, «lossy»); ở đây mọi page đều `exact`.

## 6. Ba kiểu join

**Nested Loop** — với mỗi dòng phía ngoài, tra phía trong một lần. Chuyến của khách lẻ kèm mã tài xế:

```text
 Nested Loop (actual rows=37.00 loops=1)
   ->  Bitmap Heap Scan on trips t (actual rows=40.00 loops=1)
         …
   ->  Index Scan using pk_drivers on drivers d (actual rows=0.93 loops=40)
         Index Cond: (id = t.driver_id)
         Index Searches: 37
         Buffers: shared hit=64 read=47
```

`loops=40`: nút bên trong chạy 40 lần, mỗi lần trả trung bình 0,93 dòng — 37 chuyến có tài xế, 3 chuyến không (`driver_id` NULL không khớp `=`, nên chỉ 37 lần dò thật: `Index Searches: 37`). Nhân `actual rows` với `loops` mới ra tổng.

**Hash Join** — dựng bảng băm từ phía nhỏ, quét phía lớn rồi dò. Doanh thu ngày 15/10 theo loại xe:

```text
 HashAggregate (actual rows=2.00 loops=1)
   ->  Hash Join (actual rows=5484.00 loops=1)
         Hash Cond: (t.driver_id = d.id)
         ->  Bitmap Heap Scan on trips t (actual rows=5896.00 loops=1)
               …
               ->  Bitmap Index Scan on ix_trips_city_requested (actual rows=5896.00 loops=1)
                     Index Searches: 5
         ->  Hash (actual rows=6000.00 loops=1)
               Buckets: 8192  Batches: 1  Memory Usage: 373kB
               ->  Seq Scan on drivers d (actual rows=6000.00 loops=1)
```

`Batches: 1` nghĩa là bảng băm 6.000 tài xế vừa bộ nhớ ([PG-029](/articles/postgresql-sort-hash-memory-spill) xem trường hợp bảng không vừa). `Index Searches: 5`: câu chỉ lọc ngày nhưng index bắt đầu bằng `city_id`; PostgreSQL 18 có **skip scan** — nhảy qua từng giá trị cột đầu — nên vẫn dùng được index này ([PG-025](/articles/postgresql-index-nhieu-cot-partial-expression)).

**Merge Join** — hai đầu vào đã sắp theo khoá join, trộn như kéo khoá kéo:

```text
 Merge Join
   Merge Cond: (t.id = p.trip_id)
   ->  Index Only Scan using pk_trips on trips t
   ->  Index Scan using ix_payments_trip on payments p
```

Câu `SELECT t.id, p.amount FROM trips t JOIN payments p ON p.trip_id = t.id ORDER BY t.id` cần kết quả theo `t.id`; cả hai index đã trả dòng đúng thứ tự đó nên không phải sắp. Cấm Merge Join trong một giao dịch (`SET LOCAL enable_mergejoin = off`) thì planner chuyển sang `Hash Join` hai `Seq Scan` rồi `Sort` 900 nghìn dòng. Các biến `enable_*` chỉ để học và chẩn đoán — đừng tắt trên production.

Bên lề: `SELECT count(*) FROM trips a JOIN trips b ON b.id = a.id` trên PostgreSQL 18 ra chỉ `Aggregate → Seq Scan on trips b` — planner 18 tự bỏ phép tự nối thừa theo khoá unique (tắt được bằng `enable_self_join_elimination`). Đừng ngạc nhiên khi bảng bạn viết trong SQL không có mặt trong kế hoạch.

## 7. Ước lượng so với thực tế

Chỗ đáng đọc nhất của `EXPLAIN ANALYZE` là đặt `rows=` (ước lượng) cạnh `actual rows=` (thực tế) ở **từng nút** để so sánh:

```sql
EXPLAIN (ANALYZE, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips WHERE city_id = 3 AND pickup_area_id = 311;
```

```text
 Aggregate  (cost=24363.61..24363.62 rows=1 width=8) (actual rows=1.00 loops=1)
   ->  Bitmap Heap Scan on trips  (cost=2972.62..24356.62 rows=2798 width=0) (actual rows=22222.00 loops=1)
         Recheck Cond: (city_id = 3)
         Filter: (pickup_area_id = 311)
         Rows Removed by Filter: 107573
         Heap Blocks: exact=19424
         ->  Bitmap Index Scan on ix_trips_city_requested  (cost=0.00..2971.92 rows=129533 width=0) (actual rows=129795.00 loops=1)
```

Ước khoảng 2.800, thật 22.222 — lệch khoảng 8 lần (hàm `uoc_luong()` của lab ra hệ số 7,9 ở lần chạy này). Khu 311 thuộc hẳn Đà Nẵng, nhưng planner coi hai điều kiện **độc lập** và nhân hai xác suất: tỉ lệ chuyến ở khu 311 × tỉ lệ chuyến ở Đà Nẵng (~13 %). Nút `Bitmap Index Scan` thì ước sát (129.533 vs 129.795) — lỗi nằm ở chỗ kết hợp điều kiện. Lệch ở nút dưới sẽ lan lên: nút cha dựa trên con số sai để chọn kiểu join và kiểu sắp. Để ý thêm `Heap Blocks: exact=19424` — Đà Nẵng rải khắp bảng nên bitmap chạm gần hết 19.441 page. [PG-027](/articles/postgresql-statistics-analyze-cardinality) sửa ước lượng này bằng `CREATE STATISTICS`.

## 8. EXPLAIN ANALYZE chạy thật câu lệnh ghi

Lab chép 100 chuyến vào bảng tạm `thu_explain`, rồi:

| Bước | Kết quả |
|---|---|
| `EXPLAIN (COSTS OFF) DELETE FROM thu_explain WHERE fare > 50000;` | In `Delete on thu_explain` → `Seq Scan`; bảng vẫn **100** dòng |
| `EXPLAIN (ANALYZE, …) DELETE FROM thu_explain WHERE fare > 50000;` | `Seq Scan … (actual rows=68.00 …)`, `Rows Removed by Filter: 32`; bảng còn **32** dòng |

Đó chính là 68 dòng mà bạn mới ở đầu bài đã xoá. Nút gốc `Delete on …` in `actual rows=0.00` vì nút này không trả dòng lên trên; số dòng bị xoá nằm ở nút con. Cách an toàn mà tài liệu PostgreSQL khuyên: bọc trong giao dịch rồi huỷ.

```sql
BEGIN;
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
UPDATE trips SET fare = fare + 1000 WHERE code = 600000;
ROLLBACK;
```

```text
 Update on trips (actual rows=0.00 loops=1)
   Buffers: shared hit=29 read=24 dirtied=8
   ->  Index Scan using uq_trips_code on trips (actual rows=1.00 loops=1)
         Index Cond: (code = 600000)
```

`dirtied=8`: câu UPDATE đã làm «bẩn» thật 8 page trong bộ nhớ. Sau `ROLLBACK`, chuyến 600000 vẫn 31.000 đ. `ROLLBACK` không thu hồi số đã lấy từ sequence và không huỷ được việc ngoài database (trigger gửi tin, gọi dịch vụ khác) — câu có tác dụng phụ như vậy thì đừng `ANALYZE` trên môi trường thật.

## 9. Ba tuỳ chọn nên dùng khi chia sẻ kế hoạch

```sql
EXPLAIN (COSTS OFF, SETTINGS) SELECT count(*) FROM trips WHERE fare >= 150000;
```

```text
 Aggregate
   ->  Seq Scan on trips
         Filter: (fare >= 150000)
 Settings: jit = 'off', max_parallel_workers_per_gather = '0', search_path = 'toinoi'
```

- **`SETTINGS`** in các tham số ảnh hưởng tới planner mà đang khác giá trị mặc định — người đọc kế hoạch của bạn biết ngay vì sao không có nút song song.
- **`VERBOSE`** thêm dòng `Output:` (các cột mà mỗi nút trả ra), tên bảng kèm schema (`toinoi.trips t`) và `Query Identifier` khi `compute_query_id` bật (lab nạp `pg_stat_statements` nên có).
- **`FORMAT JSON`** cho chương trình đọc; hàm `uoc_luong()` của lab dùng nó để lấy `Plan Rows` và `Actual Rows`.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách làm đúng |
|---|---|---|
| «Index bị bỏ qua» khi thấy `Bitmap Heap Scan` | Không nhận ra Bitmap Index Scan cũng là dùng index | Đọc tên index trong `Bitmap Index Scan on …` |
| Coi `cost` là mili giây | `cost` là đơn vị tương đối của planner | Thời gian chỉ có ở `EXPLAIN ANALYZE` (bỏ `TIMING OFF`) |
| Kết luận từ kế hoạch của một tham số | Kế hoạch phụ thuộc ước lượng của đúng giá trị đó | Thử giá trị phổ biến và hiếm; ghi tham số khi chia sẻ |
| Chỉ nhìn `actual rows` ở nút gốc | Lệch ước lượng nằm ở nút con | So `rows=` với `actual rows × loops` từng nút |
| Mất dữ liệu sau `EXPLAIN ANALYZE` | `ANALYZE` chạy thật câu ghi | `BEGIN … ROLLBACK`, hoặc chỉ `EXPLAIN` |
| `read` lớn ⇒ «đĩa chậm» | `read` có thể đến từ cache hệ điều hành | Đo với `track_io_timing` hoặc công cụ hệ điều hành trước khi đổ cho đĩa |

## 11. Đánh đổi và giới hạn kết luận

- **`EXPLAIN` trần** an toàn, nhanh, nhưng chỉ là dự đoán. **`EXPLAIN ANALYZE`** cho số đo thật, đổi lại tốn đúng thời gian của câu lệnh, chạy thật mọi tác dụng phụ, và việc đo thời gian từng nút cũng tốn thêm (`TIMING OFF` giảm phần này).
- Trên dữ liệu lab, `Buffers` và `actual rows` tất định; `cost`, `rows=` dao động theo mỗi lần `ANALYZE`; thời gian phụ thuộc máy và cache. Mang **cách đọc** sang hệ thống khác, đừng mang con số.
- Lab tắt song song và JIT; server mặc định có thể thêm nút `Gather` và dòng JIT — cây đọc theo cùng nguyên tắc.

## 12. Bài tập

Lời giải đầy đủ chạy được: `p03/thu-nghiem/pg021-bai-tap.sql` (không có trong lab chính; chạy `node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg021-bai-tap.sql`).

**Bài 1.** Viết câu «các chuyến tài xế mã 20007 nhận trong ngày 15/10 (giờ VN)» rồi `EXPLAIN ANALYZE`. Bảng chỉ có index `(driver_id)` và `(city_id, requested_at)`. Kế hoạch dùng index nào, điều kiện ngày nằm ở `Index Cond` hay `Filter`, đọc bao nhiêu page bảng?

<details><summary><b>Lời giải</b></summary>
<p>Tài xế có 139 chuyến, 2 chuyến trong ngày. Lần chạy khi viết bài cho ra <code>BitmapAnd</code> của hai <code>Bitmap Index Scan</code>: trên <code>ix_trips_driver_id</code> (139 dòng) và trên <code>ix_trips_city_requested</code> (5.896 chuyến trong ngày, <code>Index Searches: 5</code> nhờ skip scan) — mỗi điều kiện là <code>Index Cond</code> của một index, giao hai bitmap rồi mới đọc bảng: <code>Heap Blocks: exact=2</code>. Kế hoạch dựa trên ước lượng nên máy bạn có thể ra cây khác; số thật 2 và 139 thì không đổi.</p>
</details>

**Bài 2.** Bạn cần xem kế hoạch của câu «cộng 1.000 đ cho 300 chuyến đang chạy» (`requested_at >= '2026-10-31 00:00+07'`) kèm số đo thật mà không để lại thay đổi. Viết các lệnh và chỉ ra dòng nào cho biết số chuyến bị sửa.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">BEGIN;
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
UPDATE trips SET fare = fare + 1000
WHERE requested_at &gt;= TIMESTAMPTZ '2026-10-31 00:00+07';
ROLLBACK;</code></pre>
<p>Kết quả: <code>Update on trips (actual rows=0.00 loops=1)</code> → <code>Index Scan using ix_trips_city_requested on trips (actual rows=300.00 loops=1)</code> (<code>Index Searches: 4</code>, skip scan). Số chuyến bị sửa là 300 ở nút con. Thí nghiệm kiểm tra thêm: trong giao dịch tổng cước đã tăng đúng 300.000 đ; sau <code>ROLLBACK</code> tổng về như cũ. <code>EXPLAIN</code> không <code>ANALYZE</code> của một câu <code>INSERT</code> thì không thêm dòng nào.</p>
</details>

**Bài 3.** Dự đoán ước lượng của `city_id = 1 AND pickup_area_id = 111` (khu 111 thuộc Sài Gòn, ~55 % chuyến) lệch khoảng bao nhiêu lần, so với ví dụ Đà Nẵng ở mục 7. Còn `city_id = 3 AND kind = 'parcel'`?

<details><summary><b>Lời giải</b></summary>
<p>Planner nhân thêm tỉ lệ của thành phố, nên ước thấp khoảng 1/0,55 ≈ 1,8 lần với Sài Gòn và 1/0,13 ≈ 7,7 lần với Đà Nẵng. Đo bằng <code>uoc_luong()</code>: SG khu 111 ước khoảng 12.800, thật 22.053 (lệch 1,7); DN khu 311 ước khoảng 2.900, thật 22.222 (7,7); Đà Nẵng × bưu kiện ước khoảng 19.300, thật 19.664 (1,0) — hai điều kiện gần như độc lập thì phép nhân đúng. Thành phố càng nhỏ, lỗi giả định độc lập càng lớn.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 18 · Using EXPLAIN](https://www.postgresql.org/docs/18/using-explain.html): cost, `loops`, `Rows Removed by Filter`, `Index Searches`, cảnh báo với câu ghi.
- [PostgreSQL 18 · EXPLAIN](https://www.postgresql.org/docs/18/sql-explain.html): đủ các tuỳ chọn `ANALYZE`, `BUFFERS`, `SETTINGS`, `VERBOSE`, `FORMAT`, `WAL`; nghĩa của hit/dirtied/written.
- [PostgreSQL 18 · Release Notes](https://www.postgresql.org/docs/18/release-18.html): BUFFERS tự bật, số dòng thập phân, đếm lượt dò index, skip scan, bỏ tự nối thừa.
- Lab: `examples/toi-noi-postgresql/p03/21-explain-tim-chuyen.sql`, kết quả đầy đủ ở `p03/OUTPUT.txt`, thời gian ở `p03/DO-DAC.md`.

Bài tiếp theo: [PG-022 · Đơn vị đọc ghi nhỏ nhất của cơ sở dữ liệu là gì?](/articles/page-don-vi-doc-ghi)
