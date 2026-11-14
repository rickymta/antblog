---
slug: postgresql-index-nhieu-cot-partial-expression
title: "Index nhiều cột, partial index và expression index"
summary: "Ba đội của Tới Nơi cùng xin thêm index cho bảng trips 1 triệu chuyến đang nhận 560 UPDATE/giây. Bài này đọc thứ tự cột, skip scan của PostgreSQL 18, partial và expression index bằng kế hoạch thật, rồi tính giá ghi bằng WAL."
category: postgresql
level: intermediate
tags: [postgresql, pg-025, index, btree, partial-index, expression-index, skip-scan, toi-noi]
language: vi
---

Thứ Hai, ba yêu cầu cùng tới bàn DBA của Tới Nơi. Đội điều phối muốn màn «chuyến đang chạy» mở tức thì — 300 chuyến lẫn trong 1 triệu. Đội vận hành Đà Nẵng muốn báo cáo «chuyến DN ngày 15/10» nhanh hơn. CSKH than câu «chuyến ngày 15/10 theo giờ Việt Nam» chạy hàng chục mili giây và quét cả bảng. Mỗi đội đề xuất một index, ai cũng có lý.

Bảng `trips` đã có 5 index và, theo hồ sơ dự án, nhận khoảng 560 UPDATE/giây lúc cao điểm. Mỗi index mới giúp một kiểu câu đọc, nhưng mọi câu ghi đều trả giá cho nó. Bài này đọc ba loại index — nhiều cột, partial, expression — bằng kế hoạch thật, rồi đo cái giá đó bằng WAL.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-025 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/heap-ctid-va-primary-key">PG-024 · Heap, ctid và primary key</a> (HOT update), <a href="/articles/postgresql-btree-index-co-ban">PG-023 · B-tree index cơ bản</a>. Lab: <code>examples/toi-noi-postgresql/p03</code> (tệp <code>25-index-nhieu-cot-partial-expression.sql</code>, thí nghiệm <code>thu-nghiem/pg025-bai-tap-skip-scan-partial-unique.sql</code>). Sau bài này, bạn chọn được index theo điều kiện truy vấn và theo chi phí ghi.</p>
</div>

## 1. Dữ liệu và cách đọc số trong bài

Lab Phần 3 chạy trên **PostgreSQL 18.1 thật** (Phần 1–2 dùng PGlite 17.5), bảng `trips` 1.000.000 chuyến 01/05–30/10/2026 + 300 chuyến đang chạy, **19.441 page** heap. Index có sẵn: `pk_trips (id)`, `uq_trips_code (code)`, `ix_trips_city_requested (city_id, requested_at)`, `ix_trips_customer_requested (customer_id, requested_at DESC)`, `ix_trips_driver_id`.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p03 --only 25
node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg025-bai-tap-skip-scan-partial-unique.sql
```

Mỗi tệp chạy trên bản sao mới của database nền, shared_buffers rỗng (cache hệ điều hành vẫn nóng — `read` không có nghĩa là đọc đĩa). PostgreSQL 18 tự in `Buffers` khi `EXPLAIN ANALYZE` và thêm dòng mới **`Index Searches`**: số lần node đó dò cây từ gốc xuống. Số page, số dòng, byte WAL trong bài là tất định; thời gian lấy từ `p03/DO-DAC.md` (một máy i7-10700, NVMe, trung vị 5 lần — chỉ để so tương đối).

## 2. Index nhiều cột: thứ tự cột là tất cả

Index nhiều cột (multicolumn index) B-tree sắp mục theo cột thứ nhất, trong cùng giá trị cột thứ nhất mới sắp theo cột thứ hai — như danh bạ sắp theo họ rồi tới tên. Với `(city_id, requested_at)`, mọi chuyến Đà Nẵng (`city_id = 3`) nằm liền nhau và đã theo thứ tự thời gian. Câu của đội vận hành:

```sql
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT count(*) FROM trips
WHERE city_id = 3 AND requested_at >= TIMESTAMPTZ '2026-10-15 00:00+07'
                  AND requested_at <  TIMESTAMPTZ '2026-10-16 00:00+07';
```

```text
 Aggregate (actual rows=1.00 loops=1)
   Buffers: shared hit=3 read=7
   ->  Index Only Scan using ix_trips_city_requested on trips (actual rows=768.00 loops=1)
         Index Cond: ((city_id = 3) AND (requested_at >= '2026-10-15 00:00:00+07'::timestamp with time zone) AND …)
         Heap Fetches: 0
         Index Searches: 1
         Buffers: shared hit=3 read=7
```

Một lần dò cây tới mục đầu tiên `(3, 15/10 00:00)`, rồi đọc tiếp các lá liền nhau tới khi gặp `(3, 16/10)`: **10 page** cho 768 chuyến. (`Index Only Scan` và `Heap Fetches` là chủ đề của [PG-026](/articles/postgresql-index-only-scan-visibility-map); ở đây chỉ cần biết câu không phải đọc heap.)

Giờ tạo index đảo cột `(requested_at, city_id)` và tạm xoá index cũ trong giao dịch (`BEGIN; DROP INDEX …; EXPLAIN …; ROLLBACK;`):

```text
   ->  Index Only Scan using ix_trips_requested_city on trips (actual rows=768.00 loops=1)
         Index Cond: ((requested_at >= '2026-10-15 00:00:00+07'::timestamp with time zone) AND (requested_at < …) AND (city_id = 3))
         Index Searches: 1
         Buffers: shared hit=16 read=25
```

Cùng 768 dòng, nay **41 page**. Index đảo xếp theo thời gian trước, nên mục của ba thành phố trong ngày 15/10 đan xen nhau; muốn lấy 768 chuyến DN phải đi qua cả 5.896 mục của ngày đó và loại bớt ngay trong index. Tài liệu PostgreSQL nói gọn quy tắc: điều kiện **bằng** ở các cột đầu, cộng điều kiện **khoảng** ở cột đầu tiên không có điều kiện bằng, là phần giới hạn vùng index phải quét; điều kiện ở cột phía sau chỉ được kiểm trong index — đỡ ghé bảng nhưng không đỡ đọc index.

| Câu hỏi | `(city_id, requested_at)` | `(requested_at, city_id)` |
|---|---|---|
| `city_id = 3` ∧ một ngày | 10 page, vùng quét vừa khít | 41 page, quét cả ngày rồi lọc |
| Chỉ một ngày, mọi thành phố | dùng được nhờ skip scan (mục 3) | vùng quét vừa khít |
| `city_id IN (2, 3)` ∧ một ngày | `Index Searches: 2`, 2.643 dòng | quét cả ngày rồi lọc |

Quy tắc thực dụng: cột so **bằng** đặt trước, cột so **khoảng** hoặc dùng để sắp xếp đặt sau. Tài liệu còn nhắc index nhiều cột nên dùng dè dặt, và trên ba cột hiếm khi có ích trừ khi cách dùng bảng rất cố định.

## 3. Skip scan của PostgreSQL 18

Câu «mọi chuyến ngày 15/10», không nói thành phố, chạy trên chính `(city_id, requested_at)`:

```text
   ->  Index Only Scan using ix_trips_city_requested on trips (actual rows=5896.00 loops=1)
         Index Cond: ((requested_at >= '2026-10-15 00:00:00+07'::timestamp with time zone) AND …)
         Index Searches: 5
         Buffers: shared hit=20 read=30
```

Đây là **skip scan** — tính năng mới của bản 18 (release notes: «Allow skip scans of btree indexes»). Theo tài liệu, B-tree tự sinh một điều kiện bằng «ảo» cho cột đầu, lần lượt khớp từng giá trị có trong index, rồi áp điều kiện của cột sau cho mỗi giá trị — như chạy `city_id = 1 AND …`, `city_id = 2 AND …`, `city_id = 3 AND …`. `city_id` chỉ có 3 giá trị nên chỉ mất 5 lần dò (mỗi thành phố một lần, cộng vài lần dò để tìm giá trị kế tiếp — con số chính xác là chi tiết cài đặt), 50 page. Trước bản 18 không có cơ chế này: điều kiện chỉ ở cột sau không thu hẹp được phần index phải đọc.

Skip scan đáng giá khi cột đầu **ít** giá trị. Thí nghiệm tạm xoá `ix_trips_city_requested`, chỉ để lại `(customer_id, requested_at DESC)` — cột đầu khoảng 192.000 giá trị:

```text
   ->  Index Only Scan using ix_trips_customer_requested on trips (actual rows=5896.00 loops=1)
         Index Cond: ((requested_at >= '2026-10-15 00:00:00+07'::timestamp with time zone) AND …)
         Index Searches: 20
         Buffers: shared hit=34 read=4616
```

Planner vẫn chọn index, nhưng đọc **khoảng 4.650 page** — gần hết 4.959 page của nó. Chỉ nhờ `count(*)` không cần đọc heap nên cách này vẫn rẻ hơn quét 19.441 page bảng. Đổi sang `SELECT code` (cột ngoài index) thì planner bỏ index, `Seq Scan … Buffers: shared read=19441`. Skip scan là lưới an toàn khi thứ tự cột chưa khớp câu hỏi, không thay được thứ tự cột đúng: 50 page so với 4.650 page.

## 4. Partial index: chỉ đánh index phần hay được truy vấn

**Partial index** (index một phần) chỉ chứa mục cho các dòng thoả một điều kiện — gọi là **predicate** của index. Màn điều phối chỉ hỏi 6 trạng thái chưa kết thúc:

```sql
CREATE INDEX ix_trips_dang_chay ON trips (requested_at)
WHERE status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit');
CREATE INDEX ix_trips_status ON trips (status);   -- để so kích thước
```

```text
      relname       |  bytes
--------------------+---------
 ix_trips_dang_chay |   16384
 ix_trips_status    | 7069696
```

300 mục — 2 page (16 KB, gồm metapage) — so với 6,7 MB cho index đầy đủ. Nó cũng không lớn dần theo 730 triệu chuyến/năm: chuyến kết thúc thì rời index.

Planner chỉ dùng partial index khi **chứng minh được** điều kiện của câu kéo theo predicate, và việc chứng minh diễn ra lúc lập kế hoạch:

| Câu | Kế hoạch (lab) | Vì sao |
|---|---|---|
| `WHERE status IN (…6 trạng thái…) ORDER BY requested_at` | `Index Scan using ix_trips_dang_chay` | Khớp đúng predicate |
| `WHERE status = 'searching' ORDER BY requested_at` | `Index Scan using ix_trips_dang_chay`, `Filter: (status = 'searching')` | `'searching'` ∈ danh sách ⇒ kéo theo predicate |
| `WHERE status = 'cancelled' AND requested_at >= '30/10'` | `BitmapAnd` của `ix_trips_requested_city` và `ix_trips_status` | Không kéo theo ⇒ không dùng được |

Tài liệu nói rõ hệ thống chỉ nhận ra những suy luận đơn giản (kiểu `x < 1` kéo theo `x < 2`); ngoài ra điều kiện phải khớp nguyên văn một phần `WHERE` của câu.

### Bẫy: câu có tham số

Ứng dụng thường gửi `WHERE status = $1`. Với **generic plan** (kế hoạch chung, lập một lần cho mọi giá trị tham số), planner không biết `$1` là gì nên không chứng minh được — lab ép hai chế độ bằng `plan_cache_mode`:

```text
-- SET LOCAL plan_cache_mode = force_generic_plan;
 Limit
   ->  Index Scan using ix_trips_requested_city on trips
         Filter: (status = $1)
-- SET LOCAL plan_cache_mode = force_custom_plan;
 Limit
   ->  Index Scan using ix_trips_dang_chay on trips
         Filter: (status = 'searching'::text)
```

Ở chế độ mặc định `auto`, prepared statement được lập kế hoạch riêng cho 5 lần thực thi đầu rồi mới cân nhắc chuyển sang kế hoạch chung — nên màn hình có thể nhanh lúc thử và chậm sau vài phút chạy thật. Bài tập 2 có cách sửa.

### Partial UNIQUE: dùng index để thực thi luật nghiệp vụ

Partial index còn giữ được luật mà `UNIQUE` thường không diễn đạt nổi: **mỗi tài xế có tối đa một chuyến đang chở**. Thí nghiệm cho thấy fixture đã vi phạm luật này trước khi có index:

```sql
CREATE UNIQUE INDEX uq_trips_tai_xe_dang_cho ON trips (driver_id)
WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit');
```

```text
ERROR:  could not create unique index "uq_trips_tai_xe_dang_cho"
DETAIL:  Key (driver_id)=(324a0fdd-f591-47f3-8e0d-3cfc6f3c528c) is duplicated.
```

200 chuyến đang chở nhưng chỉ 197 tài xế: ba tài xế mỗi người hai chuyến (1099751 + 1099860, 1099873 + 1099885, 1099890 + 1099909). Dữ liệu bẩn phải xử lý theo quyết định nghiệp vụ trước (lab huỷ chuyến nhận sau), rồi index mới tạo được — 197 mục, 16 KB. Từ đó, gán chuyến thứ hai cho tài xế đang chở bị chặn `23505`, còn chuyến đã hoàn thành của cùng tài xế không vướng vì không nằm trong index.

## 5. Expression index: index trên kết quả tính

CSKH lọc theo «ngày giờ Việt Nam» bằng biểu thức trên cột:

```sql
SELECT * FROM trips WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = DATE '2026-10-15';
```

```text
 Seq Scan on trips (actual rows=5896.00 loops=1)
   Filter: (((requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh'::text))::date = '2026-10-15'::date)
   Rows Removed by Filter: 994104
   Buffers: shared hit=429 read=19012
```

Index trên `requested_at` lưu giá trị cột, không lưu kết quả biểu thức, nên không dùng được — 19.441 page để lấy 5.896 dòng. **Expression index** (index trên biểu thức) lưu sẵn kết quả tính cho mỗi dòng. Nhưng không phải biểu thức nào cũng được:

```sql
CREATE INDEX ix_trips_ngay_sai ON trips ((requested_at::date));
```

```text
ERROR:  functions in index expression must be marked IMMUTABLE
```

Mã lỗi `42P17`. Hàm trong index phải **IMMUTABLE** — kết quả chỉ phụ thuộc đối số. `requested_at::date` phụ thuộc `TimeZone` của phiên ([PG-007](/articles/postgresql-select-filter-order-limit) đã thấy nó đếm 8 hay 9 tuỳ người chạy); nếu index nhận nó, cùng một dòng sẽ «thuộc» hai ngày tuỳ phiên đã ghi. `AT TIME ZONE` với tên múi giờ cố định thì không phụ thuộc phiên:

```sql
CREATE INDEX ix_trips_ngay_vn ON trips (((requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date));
ANALYZE trips;   -- thống kê cho BIỂU THỨC chỉ có sau ANALYZE
```

```text
 Index Scan using ix_trips_ngay_vn on trips (actual rows=5896.00 loops=1)
   Index Cond: (((requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh'::text))::date = '2026-10-15'::date)
   Buffers: shared read=129
```

129 page thay vì 19.441. Hai điều kiện: câu phải viết **đúng biểu thức** của index — thí nghiệm viết cùng ý bằng `date_trunc('day', requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') = '2026-10-15'` thì ra cùng 5.896 dòng nhưng quay về `Seq Scan`; và tài liệu nhắc biểu thức được tính lại ở mỗi lần INSERT và mỗi UPDATE không HOT.

Thường có cách không cần index mới: khoảng nửa mở trên cột gốc (PG-007, PG-016) dùng ngay index sẵn có — lab ra `Index Scan using ix_trips_requested_city`. Expression index dành cho biểu thức không viết lại được thành điều kiện trên cột (như `lower(email)`).

## 6. Cái giá khi ghi: đo bằng WAL

Mỗi index là một cấu trúc phải cập nhật cùng bảng, và mỗi thay đổi đều ghi vào **WAL** (Write-Ahead Log — nhật ký ghi trước, [PG-022](/articles/page-don-vi-doc-ghi)). Lab chép 50.000 chuyến vào bảng trống `ghi_thu` với 0, 5 (như `trips`) và 9 index (thêm 4 index của bài này), đo bằng `EXPLAIN (ANALYZE, WAL)` ngay sau `CHECKPOINT`:

```text
 Insert on ghi_thu (actual rows=0.00 loops=1)
   WAL: records=453320 fpi=5 bytes=40533215 buffers full=3621
```

| Số index | Bản ghi WAL | Byte WAL | WAL / chuyến | Thời gian (DO-DAC, client) |
|---:|---:|---:|---:|---:|
| 0 | 50.001 | 9.026.397 | ≈ 180 B | 79 ms |
| 5 | 302.054 | 29.826.950 | ≈ 597 B | 603 ms |
| 9 | 453.320 | 40.533.215 | ≈ 811 B | 846 ms |

Mỗi index B-tree thêm **một bản ghi WAL cho mỗi dòng**: 0 → 5 index tăng 252.053 ≈ 5 × 50.000, phần dư là bản ghi tách page. 5 → 9 index chỉ tăng 151.266 ≈ 3 × 50.000 — index thứ tư mới là partial, không chuyến cũ nào thoả predicate nên không ghi gì. Thời gian ghi tăng ~7,6 lần với 5 index (phép đo một máy, 5 lần, có nhiễu ~±20 %). `fpi` (full page image — ảnh nguyên page lần đầu page bị sửa sau checkpoint) ở đây chỉ 1–5 vì bảng mới toàn page mới; `buffers full` (mới ở bản 18: số lần bộ đệm WAL đầy phải đẩy ra) phụ thuộc thời điểm, không so giữa các lần chạy. Kích thước sau 50.000 dòng: bảng 7,6 MB; các index từ 2,4 MB (`customer_id, requested_at`) xuống 344 KB (`status`) và 8 KB (partial — 50.000 chuyến cũ đều đã kết thúc).

UPDATE còn đắt hơn. Sửa giá 10.000 chuyến của `ghi_thu` (`fare` không có index):

```text
 Update on ghi_thu (actual rows=0.00 loops=1)
   WAL: records=101995 fpi=1046 bytes=15901666 buffers full=1346
```

`n_tup_hot_upd` = **4 / 10.000**. Bảng chép dày đặc (fillfactor 100) nên phiên bản mới không vừa page cũ, không được **HOT** ([PG-024](/articles/heap-ctid-va-primary-key)) ⇒ ghi mục mới vào cả 9 index: ~10 bản ghi WAL mỗi dòng. 1.046 ảnh page là do đo ngay sau `CHECKPOINT` — giữa hai checkpoint, cùng thao tác ghi ít WAL hơn; con số này là cận trên để so sánh, không phải mức ổn định.

Bẫy riêng của partial index: cột chỉ nằm trong **predicate** cũng chặn HOT. Thí nghiệm (mục E) trên bảng 10.000 chuyến fillfactor 90: đảo `status` giữa `completed` và `cancelled` cho 500 dòng ⇒ 500 HOT; thêm `ix_hot_dang_chay … WHERE status IN (…đang chạy…)` — index vẫn rỗng, cả giá trị cũ lẫn mới đều nằm ngoài predicate — rồi đảo 500 dòng khác ⇒ **0 HOT** (`n_tup_upd 1000 | n_tup_hot_upd 500`). Với Tới Nơi, đổi trạng thái là loại UPDATE nhiều nhất trong 560 lần/giây — đó là giá thật của index «chuyến đang chạy», lớn hơn nhiều so với 16 KB dung lượng.

## 7. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Có index `(a, b)` mà câu lọc `b` vẫn đọc gần cả index | Cột đầu nhiều giá trị, skip scan không nhảy được | Index bắt đầu bằng `b`, hoặc chấp nhận |
| Partial index chạy khi thử, mất khi lên app | Tham số `$1`, generic plan | Lặp predicate bằng hằng trong câu |
| Expression index không được dùng | Câu viết biểu thức khác (dù cùng nghĩa) | Viết đúng biểu thức, hoặc dùng điều kiện trên cột gốc |
| `42P17` khi tạo expression index | Hàm/ép kiểu phụ thuộc phiên (STABLE) | Biểu thức IMMUTABLE (múi giờ cố định) |
| Tạo partial UNIQUE báo `23505` | Dữ liệu cũ đã vi phạm | Tìm bằng `GROUP BY … HAVING count(*) > 1`, xử lý trước |

## 8. Đánh đổi khi chọn

- **Một index nhiều cột hay nhiều index một cột:** `(city_id, requested_at)` phục vụ «thành phố + ngày», «thành phố», và nhờ skip scan cả «chỉ ngày»; hai index một cột tốn hai lần ghi và phải ghép bằng `BitmapAnd`.
- **Partial:** nhỏ, rẻ khi ghi dòng ngoài predicate, nhưng chỉ phục vụ câu chứng minh được predicate, và chặn HOT cho cột trong predicate.
- **Expression:** đáp ứng đúng câu hỏi khó viết lại, nhưng tính biểu thức ở mỗi lần ghi và chỉ khớp đúng một cách viết.
- **Index mới nói chung:** trả lời bằng số — page đọc giảm bao nhiêu cho câu nào, WAL và HOT mất bao nhiêu cho tải ghi nào. Không có số thì chưa nên thêm.

## 9. Bài tập

**Bài 1.** Sau khi có `ix_trips_ngay_vn`, một nhân viên viết `WHERE date_trunc('day', requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh') = TIMESTAMP '2026-10-15 00:00'`. Kết quả đúng không, có dùng index không, sửa thế nào?

<details><summary><b>Lời giải</b></summary>
<p>Kết quả đúng (5.896 dòng, giống kết quả của cách viết <code>::date</code>) nhưng kế hoạch là <code>Seq Scan</code>: planner chỉ khớp biểu thức index theo dạng cây biểu thức, không suy ra hai cách viết tương đương. Hai cách sửa:</p>
<pre><code class="language-sql">-- đúng biểu thức của index
WHERE (requested_at AT TIME ZONE 'Asia/Ho_Chi_Minh')::date = DATE '2026-10-15'
-- hoặc khoảng nửa mở trên cột gốc, không cần index biểu thức
WHERE requested_at &gt;= TIMESTAMPTZ '2026-10-15 00:00+07'
  AND requested_at &lt;  TIMESTAMPTZ '2026-10-16 00:00+07'</code></pre>
<p>Chạy thật: mục D của <code>p03/thu-nghiem/pg025-bai-tap-skip-scan-partial-unique.sql</code>.</p>
</details>

**Bài 2.** Ứng dụng điều phối dùng prepared statement `SELECT code FROM trips WHERE status = $1 ORDER BY requested_at LIMIT 50`. Sửa câu để kế hoạch chung vẫn dùng `ix_trips_dang_chay`, không đổi kết quả với các trạng thái đang chạy.

<details><summary><b>Lời giải</b></summary>
<p>Thêm nguyên văn predicate của index bằng hằng số; điều kiện này luôn đúng với 6 trạng thái màn điều phối gửi, và planner chứng minh được nó kéo theo predicate mà không cần biết <code>$1</code>:</p>
<pre><code class="language-sql">PREPARE dang_chay_theo_trang_thai(text) AS
  SELECT code FROM trips
  WHERE status = $1
    AND status IN ('requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit')
  ORDER BY requested_at LIMIT 50;</code></pre>
<p>Với <code>plan_cache_mode = force_generic_plan</code>, lab ra <code>Limit → Index Scan using ix_trips_dang_chay</code>, <code>Filter: (status = $1)</code>, trả 50 dòng. Truyền trạng thái ngoài danh sách thì câu trả 0 dòng — đúng ý màn hình. Mục B của tệp thí nghiệm.</p>
</details>

**Bài 3.** Viết ràng buộc «mỗi tài xế tối đa một chuyến đang chở» bằng index, triển khai trên fixture hiện có, và chứng minh nó chặn vi phạm mà không chặn chuyến đã hoàn thành.

<details><summary><b>Lời giải</b></summary>
<p>Tạo trực tiếp thất bại <code>23505</code> vì 3 tài xế đang có hai chuyến. Tìm trước:</p>
<pre><code class="language-sql">SELECT array_agg(code ORDER BY code), array_agg(status ORDER BY code)
FROM trips WHERE status IN ('accepted', 'arriving', 'picked_up', 'in_transit')
GROUP BY driver_id HAVING count(*) &gt; 1;</code></pre>
<p>Ra 3 cặp {1099751,1099860}, {1099873,1099885}, {1099890,1099909}. Sau khi bên nghiệp vụ quyết định xử lý (lab huỷ chuyến nhận sau, 3 dòng), <code>CREATE UNIQUE INDEX uq_trips_tai_xe_dang_cho ON trips (driver_id) WHERE status IN (…4 trạng thái…)</code> thành công, 16.384 byte. Gán chuyến <code>searching</code> cho tài xế đang <code>in_transit</code> ⇒ <code>duplicate key value violates unique constraint "uq_trips_tai_xe_dang_cho"</code>; tài xế đó vẫn có chuyến <code>completed</code> trong bảng mà không vướng. Mục C của tệp thí nghiệm.</p>
</details>

## 10. Đọc tiếp

- [PostgreSQL 18 · Multicolumn Indexes](https://www.postgresql.org/docs/18/indexes-multicolumn.html) — quy tắc cột đầu và skip scan.
- [PostgreSQL 18 · Partial Indexes](https://www.postgresql.org/docs/18/indexes-partial.html) — suy luận predicate, tham số, partial unique.
- [PostgreSQL 18 · Indexes on Expressions](https://www.postgresql.org/docs/18/indexes-expressional.html) và [CREATE INDEX](https://www.postgresql.org/docs/18/sql-createindex.html) (yêu cầu IMMUTABLE).
- [PostgreSQL 18 · Release Notes](https://www.postgresql.org/docs/18/release-18.html) — skip scan, `Index Searches`, WAL `buffers full`.
- [PostgreSQL 18 · Heap-Only Tuples (HOT)](https://www.postgresql.org/docs/18/storage-hot.html).
- Lab: `p03/25-index-nhieu-cot-partial-expression.sql`, kết quả ở `p03/OUTPUT.txt`, thời gian ở `p03/DO-DAC.md`; thí nghiệm (không có trong lab chính) ở `p03/thu-nghiem/pg025-bai-tap-skip-scan-partial-unique.sql`.

Bài tiếp theo: [PG-026: Index-only scan, INCLUDE và visibility map](/articles/postgresql-index-only-scan-visibility-map).
