---
slug: page-don-vi-doc-ghi
title: "Đơn vị đọc ghi nhỏ nhất của cơ sở dữ liệu là gì?"
summary: "Đội Tới Nơi đề xuất «chỉ SELECT fare cho nhẹ» và tin rằng «count(*) luôn phải đọc cả bảng». Đo bằng pageinspect, EXPLAIN BUFFERS và WAL trên bảng trips để thấy vì sao lấy một chuyến cũng phải đọc cả 8 KB."
category: postgresql
level: beginner
tags: [postgresql, pg-022, page, buffer, storage, wal, pageinspect, toi-noi]
language: vi
---

Màn hình theo dõi chuyến của Tới Nơi chỉ cần `fare` và `status`. Trong buổi rà hiệu năng, một bạn đề xuất «đổi `SELECT *` thành `SELECT fare` cho đọc ít đi», bạn khác khẳng định «`count(*)` lúc nào cũng phải đọc hết bảng». Nghe hợp lý, nhưng cả hai đang đếm chi phí theo **dòng** và **cột** — trong khi PostgreSQL không đọc ghi theo đơn vị đó.

Bài này đo đơn vị thật trên bảng `trips` và đặt hai khẳng định ấy về đúng chỗ.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-022 · Phần 3/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-explain-ke-hoach-thuc-thi">PG-021 · Đọc EXPLAIN từ truy vấn tìm chuyến</a>. Lab: <code>examples/toi-noi-postgresql/p03</code> (tệp <code>22-page-buffer.sql</code>, schema <code>bai_cu</code> dựng ở <code>03-bai-cu-20k.sql</code>). Sau bài này, bạn đọc được số page và buffer của một truy vấn, mổ được một page bằng pageinspect, và biết kết luận nào rút ra được từ các con số đó.</p>
</div>

Trước khi đọc tiếp, thử trả lời — chọn một đáp án trong đầu rồi mới mở phần giải thích.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Theo bạn, đơn vị đọc ghi nhỏ nhất của cơ sở dữ liệu là gì?</span>
<p><b>A.</b> Một dòng (row)</p>
<p><b>B.</b> Một bản ghi (record)</p>
<p><b>C.</b> Một ô giá trị — một cột của một dòng</p>
<p><b>D.</b> Cả một bảng</p>
<p><b>E.</b> Một khối byte có kích thước cố định</p>
<details>
<summary><b>Xem đáp án</b></summary>
<p><b>E — page.</b> Ở PostgreSQL mỗi page là 8.192 byte. Muốn lấy một dòng, PostgreSQL đọc nguyên page chứa dòng đó; muốn sửa một dòng, nó sửa trong page rồi về sau ghi nguyên page xuống đĩa. A và B thực ra là cùng một khái niệm, C nhỏ hơn nhưng không ai đọc riêng được, D thì quá lớn.</p>
</details>
</div>

Nếu bạn chọn A hoặc B, bạn có lý: SQL làm việc với dòng, ứng dụng nghĩ theo bản ghi. Nhưng đó là **lớp logic**. Thứ thật sự đi giữa đĩa, bộ nhớ và CPU là **page**.

## 1. Dòng, bản ghi, tuple: ba tên của cùng một thứ

Trong cơ sở dữ liệu quan hệ, **row**, **record**, **bản ghi** và **tuple** là những tên của cùng một đơn vị logic: một chuyến trong bảng `trips`. Ứng dụng của [Tới Nơi](/articles/du-an-mau-toi-noi) đọc ghi theo đơn vị đó — «lấy chuyến 112345», «đổi trạng thái chuyến này sang `accepted`».

Bộ máy lưu trữ thì không nhìn theo dòng. Mỗi bảng và mỗi index là các tệp chia thành những khối bằng nhau gọi là **page** (tài liệu PostgreSQL còn gọi là *block*). Dòng chỉ là thứ nằm **bên trong** page.

```sql
SHOW block_size;
```

```text
 block_size
------------
 8192
```

8.192 byte được chọn lúc biên dịch PostgreSQL — không đổi bằng cấu hình khi đang chạy, và mọi bảng, mọi index trong cluster dùng chung.

Mọi số trong bài là kết quả chạy thật trên **PostgreSQL 18.1** bằng lab Phần 3 (`node pg/run-pg.mjs p03 --only 22`). Phần lớn bài dùng schema `bai_cu`: bộ sinh **20.000 chuyến** của hồ sơ Tới Nơi, thay `random()` và `now()` bằng bản tất định để ai chạy cũng ra cùng số. Mục 6 đo lại trên bảng 1 triệu chuyến của Phần 3.

## 2. Bên trong một page của trips

`pg_class` ghi số page (`relpages`) và số dòng ước tính (`reltuples`) của mỗi bảng, index — do `VACUUM`/`ANALYZE` cập nhật:

```sql
SELECT relname, relpages, pg_relation_size(oid) AS bytes,
       round((reltuples / relpages)::numeric, 1) AS rows_per_page
FROM pg_class
WHERE relnamespace = 'bai_cu'::regnamespace
  AND relname IN ('trips', 'trips_pkey', 'uq_trips_code', 'ix_trips_city_requested')
ORDER BY relname;
```

```text
         relname         | relpages |  bytes  | rows_per_page
-------------------------+----------+---------+---------------
 ix_trips_city_requested |       83 |  679936 |         241.0
 trips                   |      374 | 3063808 |          53.5
 trips_pkey              |      102 |  835584 |         196.1
 uq_trips_code           |       57 |  466944 |         350.9
```

20.000 chuyến nằm trong **374 page** (374 × 8.192 = 3.063.808 byte — kích thước tệp luôn là bội số của page), trung bình 53,5 chuyến một page. Index chứa nhiều mục hơn trên một page vì mỗi mục chỉ có khoá và vị trí dòng (`ctid`).

Dòng `relnamespace` không phải trang trí. Database lab có hai bảng tên `trips` (`toinoi.trips` 1 triệu chuyến và `bai_cu.trips`); bỏ điều kiện schema thì cùng câu trả **7 dòng** lẫn của hai bảng. Truy vấn catalog theo tên mà không lọc schema là cách nhanh nhất để đọc nhầm số của bảng khác.

Mở page đầu tiên bằng extension `pageinspect`:

```sql
CREATE EXTENSION pageinspect;
SELECT lower, upper, upper - lower AS free_bytes, (lower - 24) / 4 AS so_line_pointer
FROM page_header(get_raw_page('bai_cu.trips', 0));
```

```text
 lower | upper | free_bytes | so_line_pointer
-------+-------+------------+-----------------
   236 |   328 |         92 |              53
```

Page mở đầu bằng **page header** 24 byte, tiếp theo là mảng **line pointer** (con trỏ dòng, 4 byte mỗi ô, ghi vị trí và độ dài của một dòng). `lower` là chỗ mảng line pointer kết thúc, `upper` là chỗ dữ liệu dòng bắt đầu; dữ liệu được ghi **từ cuối page ngược lên**. `heap_page_items` cho thấy ô 1 nằm ở byte 8.048 dài 144 byte, ô 53 ở byte 328.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 260" role="img" aria-label="Page 0 của bảng trips dài 8.192 byte: page header 24 byte, 53 line pointer chiếm 212 byte, còn 92 byte trống, 53 chuyến chiếm 7.864 byte từ cuối page ngược lên.">
<defs>
<pattern id="pg022-rows" width="15.21" height="56" patternUnits="userSpaceOnUse" x="73.63">
<rect width="15.21" height="56" class="f-phys-soft"/>
<line x1="15.2" y1="0" x2="15.2" y2="56" class="s-phys" stroke-width="1"/>
</pattern>
</defs>
<text x="40" y="34" class="t-b">page 0 của bảng bai_cu.trips — 8.192 byte</text>
<text x="880" y="34" text-anchor="end" class="t-small">tỉ lệ thật: 1 px ≈ 10 byte</text>
<text x="62" y="66" text-anchor="end" class="t-mono t-small">lower = 236</text>
<text x="76" y="66" class="t-mono t-small">upper = 328</text>
<line x1="64.2" y1="70" x2="64.2" y2="84" class="s-muted" stroke-width="1"/>
<line x1="73.63" y1="70" x2="73.63" y2="84" class="s-muted" stroke-width="1"/>
<rect x="40" y="84" width="840" height="56" rx="4" class="f-surface s-line"/>
<rect x="40" y="84" width="2.46" height="56" class="f-warn"/>
<rect x="42.46" y="84" width="21.74" height="56" class="f-pk"/>
<rect x="64.2" y="84" width="9.43" height="56" class="f-bg"/>
<rect x="73.63" y="84" width="806.37" height="56" fill="url(#pg022-rows)"/>
<rect x="40" y="84" width="840" height="56" rx="4" class="f-none s-muted" stroke-width="1"/>
<text x="477" y="117" text-anchor="middle" class="t-b t-phys">53 chuyến · 7.864 byte · ghi từ cuối page ngược lên</text>
<line x1="40" y1="148" x2="40" y2="154" class="s-muted"/><text x="40" y="168" text-anchor="middle" class="t-mono t-small">0</text>
<line x1="250" y1="148" x2="250" y2="154" class="s-muted"/><text x="250" y="168" text-anchor="middle" class="t-mono t-small">2.048</text>
<line x1="460" y1="148" x2="460" y2="154" class="s-muted"/><text x="460" y="168" text-anchor="middle" class="t-mono t-small">4.096</text>
<line x1="670" y1="148" x2="670" y2="154" class="s-muted"/><text x="670" y="168" text-anchor="middle" class="t-mono t-small">6.144</text>
<line x1="880" y1="148" x2="880" y2="154" class="s-muted"/><text x="880" y="168" text-anchor="middle" class="t-mono t-small">8.192</text>
<rect x="40" y="196" width="14" height="14" rx="2" class="f-warn"/><text x="62" y="208">page header: 24 byte</text>
<rect x="230" y="196" width="14" height="14" rx="2" class="f-pk"/><text x="252" y="208">53 line pointer × 4 byte = 212 byte</text>
<rect x="510" y="196" width="14" height="14" rx="2" class="f-bg s-muted"/><text x="532" y="208">còn trống: 92 byte</text>
<rect x="690" y="196" width="14" height="14" rx="2" class="f-phys-soft s-phys"/><text x="712" y="208">1 chuyến 136–152 byte</text>
<text x="40" y="240" class="t-small">24 + 212 + 92 + 7.864 = 8.192. Chuyến kế tiếp cần ≈ 148 byte + 4 byte line pointer, không vừa 92 byte ⇒ sang page 1.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Một page đầy của <code>trips</code>, vẽ đúng tỉ lệ.</b> Dữ liệu chuyến chiếm gần trọn 8 KB; header và mảng line pointer chỉ là dải mỏng ở đầu. Số byte lấy từ <code>page_header</code> và <code>heap_page_items</code> (tổng độ dài 53 dòng = 7.864 byte, kiểm ở <code>p03/thu-nghiem/pg022-bai-tap.sql</code>).</figcaption>
</figure>

Một chuyến trung bình 147,3 byte (`avg(pg_column_size(t.*))`), gồm khoảng 24 byte header của tuple (23 byte, căn lề) cộng các cột: `id`, `customer_id`, `driver_id`, `merchant_id` là uuid 16 byte mỗi cột, `code` 8, `fare` 4, mỗi mốc thời gian 8, `status` vài byte. Page chỉ còn 92 byte trống thì chuyến tiếp theo phải sang page mới.

## 3. Đọc: lấy một chuyến cũng phải đọc cả page

[PG-021](/articles/postgresql-explain-ke-hoach-thuc-thi) đã dùng dòng `Buffers` của `EXPLAIN ANALYZE` — mỗi **buffer** là đúng một page 8 KB trong bộ nhớ đệm `shared_buffers`. Ở PostgreSQL 18, `EXPLAIN ANALYZE` tự in dòng này:

```sql
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT fare FROM trips WHERE ctid = '(100,7)';
```

```text
 Tid Scan on trips (actual rows=1.00 loops=1)
   TID Cond: (ctid = '(100,7)'::tid)
   Buffers: shared hit=1
```

Một dòng, một cột 4 byte — vẫn **một page 8.192 byte**. Lặp lại với các kiểu truy vấn khác (mỗi câu chạy hai lần, ghi lần hai khi cache đã ấm):

| Truy vấn trên `bai_cu.trips` | Dòng trả về | Page đã đọc |
|---|---:|---:|
| `WHERE ctid = '(100,7)'`, chỉ lấy `fare` | 1 | 1 |
| `WHERE code = 112345`, lấy mọi cột | 1 | 3 — gốc index, lá index, 1 page heap |
| `WHERE code = 112346`, chỉ lấy `fare` | 1 | 3 — lấy ít cột không làm đọc ít page đi |
| `WHERE code BETWEEN 112000 AND 112052` | 53 | 4 — 2 page index + 2 page heap (224, 225) |
| 53 chuyến rải rác, mỗi chuyến cách 377 mã | 53 | 159 — `Index Searches: 53`, 53 page heap khác nhau |
| `SELECT count(fare) FROM trips` | 20.000 | 374 — `Seq Scan` cả bảng |
| `SELECT count(*) FROM trips` | 20.000 | **51** — `Index Only Scan` trên `ix_trips_driver_id`, `Heap Fetches: 0` |

Hai dòng giữa bảng là điểm mấu chốt: **cùng 53 chuyến**, một cách đọc tốn 4 page, cách kia 159 page — gấp gần 40 lần. Khác nhau không ở số dòng mà ở chỗ các dòng **nằm trên bao nhiêu page**: 53 chuyến liền mã nằm gọn trên page 224 và 225, còn 53 chuyến rải rác nằm trên 53 page khác nhau, mỗi chuyến tốn trọn 3 page (2 index + 1 heap).

Hai dòng cuối trả lời khẳng định thứ hai ở đầu bài. Bảng vừa `VACUUM` nên mọi page được đánh dấu «mọi dòng đều nhìn thấy được» trong visibility map; khi đó `count(*)` không cần ghé heap, PostgreSQL đếm trên index **nhỏ nhất** — `ix_trips_driver_id` chỉ 51 page vì gộp các khoá trùng. Muốn thấy đọc hết 374 page thì đếm một cột không có index (`count(fare)`), hoặc đếm khi bảng vừa bị sửa nhiều mà chưa `VACUUM` (bài tập 3). Cơ chế này là chủ đề của [PG-026](/articles/postgresql-index-only-scan-visibility-map).

Ở Tới Nơi, chuyến được chèn theo thời gian, nên heap gần như thẳng hàng với `requested_at` và `code`. Báo cáo «các chuyến trong 1 giờ qua» đọc một dải page liền nhau; «mọi chuyến của khách X» thì rải khắp bảng vì khách đặt ở nhiều thời điểm. Mức «thẳng hàng» này đo được bằng hệ số `correlation` — [PG-024](/articles/heap-ctid-va-primary-key) đi tiếp ý đó.

<div class="tkh-callout">
<span class="tkh-callout-title">«Chỉ SELECT vài cột cho nhanh» — đúng một nửa</span>
<p>Lấy ít cột giúp giảm dữ liệu gửi qua mạng và giảm việc giải nén giá trị lớn, nhưng với dòng nhỏ như <code>trips</code> thì <b>không giảm số page phải đọc</b>: <code>fare</code> nằm chung page với mọi cột khác của chuyến đó (3 page cho cả <code>SELECT *</code> lẫn <code>SELECT fare</code>). Muốn đọc ít page hơn thì phải đọc ít dòng hơn, làm các dòng cần đọc nằm gần nhau hơn, hoặc trả lời được chỉ từ index.</p>
</div>

## 4. Buffer: page nằm trong bộ nhớ

`shared_buffers` là vùng nhớ chung chia thành các ô vừa đúng một page; lab dùng 128 MB (giá trị mặc định) = **16.384 buffer**. Extension `pg_buffercache` cho xem từng ô đang chứa page nào, và PostgreSQL 18 thêm hàm `pg_buffercache_evict_relation()` để đẩy mọi page của một bảng ra khỏi `shared_buffers`. Lab đẩy `bai_cu.trips` ra (đếm lại: 0 buffer), rồi đọc cả bảng bằng `count(fare)`:

```text
 Aggregate (actual rows=1.00 loops=1)
   Buffers: shared read=374
   ->  Seq Scan on trips (actual rows=20000.00 loops=1)
         Buffers: shared read=374
```

```sql
SELECT count(*) AS so_buffer, min(usagecount) AS usagecount
FROM pg_buffercache
WHERE relfilenode = pg_relation_filenode('bai_cu.trips')
  AND reldatabase = (SELECT oid FROM pg_database WHERE datname = current_database())
  AND relforknumber = 0;
```

```text
 so_buffer | usagecount
-----------+------------
       374 |          1
```

374 page đọc vào thành đúng 374 buffer — không có «nửa page» trong bộ nhớ. Lần chạy sau, cùng câu ra `shared hit=374`.

Giới hạn khi đọc con số: `read` nghĩa là PostgreSQL phải xin page từ hệ điều hành, **không nhất thiết** từ đĩa — page có thể đang nằm trong cache của hệ điều hành, và PostgreSQL không phân biệt được. Lab cũng không xoá được cache hệ điều hành, nên mọi «lạnh» trong Phần 3 chỉ là lạnh với `shared_buffers`. Điều rút ra được từ `Buffers` là **số page phải chạm** — thước đo khối lượng việc; thời gian đọc thật cần đo riêng.

## 5. Ghi: cũng theo page, và lần đầu đắt hơn hẳn

Khi `UPDATE` một chuyến, PostgreSQL sửa trong buffer, đánh dấu page đó «bẩn», về sau checkpoint (hoặc tiến trình nền) ghi **nguyên page 8 KB** xuống tệp dữ liệu. Không có cách ghi riêng 4 byte của `fare`.

Còn một lớp ghi nữa: **WAL** (write-ahead log), nhật ký mọi thay đổi được ghi trước để khôi phục khi sập. Lab sửa giá hai chuyến **cùng nằm trên page 373** (page cuối) ngay sau một `CHECKPOINT`, lưu vị trí WAL trước, giữa, sau vào biến psql, rồi đọc từng bản ghi bằng extension `pg_walinspect`:

```sql
CHECKPOINT;
SELECT pg_current_wal_insert_lsn() AS lsn_truoc \gset
UPDATE trips SET fare = fare + 1000 WHERE code = 119998;
SELECT pg_current_wal_insert_lsn() AS lsn_giua \gset
UPDATE trips SET fare = fare + 1000 WHERE code = 119999;
SELECT pg_current_wal_insert_lsn() AS lsn_sau \gset

SELECT CASE WHEN start_lsn < :'lsn_giua' THEN 1 ELSE 2 END AS cau,
       resource_manager, record_type, record_length, fpi_length
FROM pg_get_wal_records_info(:'lsn_truoc', :'lsn_sau')
WHERE resource_manager <> 'Standby'
ORDER BY start_lsn;
```

```text
 cau | resource_manager | record_type | record_length | fpi_length
-----+------------------+-------------+---------------+------------
   1 | Heap             | HOT_UPDATE  |          6573 |       6508
   1 | Transaction      | COMMIT      |            34 |          0
   2 | Heap             | HOT_UPDATE  |            80 |          0
   2 | Transaction      | COMMIT      |            34 |          0
```

Cùng kiểu thay đổi, cùng một page, nhưng câu đầu ghi **6.607 byte** WAL còn câu sau chỉ **114 byte**. Câu đầu chứa một **full page image** (ảnh nguyên page, cột `fpi_length`) 6.508 byte. Điều kiện `Standby` bỏ bản ghi do tiến trình nền tự ghi định kỳ; trừ thẳng hai vị trí LSN thì ra khoảng 6.640 và 120 byte vì gồm cả phần đệm căn lề.

Lý do: hệ điều hành và ổ đĩa ghi theo khối nhỏ hơn 8 KB. Nếu máy sập đúng lúc PostgreSQL đang ghi một page, page trên đĩa có thể nửa cũ nửa mới — *torn page* — và bản ghi WAL «chỉ phần thay đổi» không đủ để dựng lại nó. Vì vậy, với `full_page_writes = on` (mặc định), lần **đầu tiên** một page bị sửa sau mỗi checkpoint, PostgreSQL chép nguyên nội dung page đó vào WAL; các lần sửa sau trên cùng page, tới checkpoint kế tiếp, chỉ ghi phần thay đổi. Ảnh page nhỏ hơn 8.192 byte vì phần trống giữa `lower` và `upper` không cần chép — page 373 là page cuối, trước khi sửa còn 1.800 byte trống.

Trường hợp thứ ba: sửa giá một chuyến **cũ** (mã 110000) trên page đã đầy, ngay sau checkpoint. Phiên bản mới không vừa page cũ nên phải sang page khác — không còn là HOT update — và mỗi index của `trips` phải thêm một mục trỏ tới vị trí mới:

| Bản ghi WAL | Số bản ghi |
|---|---:|
| Btree `INSERT_LEAF` (5 index) | 5 |
| Heap `LOCK` | 1 |
| Heap `UPDATE` | 1 |
| Transaction `COMMIT` | 1 |
| **Tổng: 8 bản ghi, 7 có ảnh page, 47.691 byte** | |

Bản ghi `Heap LOCK` xuất hiện vì khi phiên bản mới phải sang page khác, PostgreSQL khoá tạm dòng cũ trong lúc nhả khoá page để tìm chỗ. HOT update và chuyện «dời page» là chủ đề của [PG-024](/articles/heap-ctid-va-primary-key).

Với Tới Nơi, khoảng **560 UPDATE mỗi giây** lúc cao điểm (theo hồ sơ dự án) rải trên hàng trăm page khác nhau. Ngay sau mỗi checkpoint (`checkpoint_timeout` mặc định 5 phút), hàng loạt page bị sửa «lần đầu» cùng lúc nên lượng WAL tăng vọt rồi giảm dần — hình răng cưa quen thuộc trên biểu đồ giám sát. Lưu ý: mọi số WAL ở đây đo **ngay sau** `CHECKPOINT`, trường hợp đắt nhất; giữa hai checkpoint, cùng thao tác ghi ít hơn nhiều.

## 6. Từ 20.000 lên 1 triệu chuyến

Lab đo lại cùng các ý trên `toinoi.trips` 1 triệu chuyến (lược đồ Phần 1–2, có thêm `pickup_area_id`):

| Chỉ số | `bai_cu` · 20.000 chuyến | `toinoi` · 1 triệu chuyến |
|---|---:|---:|
| Page của bảng | 374 | 19.441 |
| Chuyến mỗi page · byte mỗi chuyến | 53,5 · 147,3 | 51,4 · 149,2 (mẫu 20.000 chuyến) |
| Số tầng của `uq_trips_code` | 2 (`bt_metap` level 1) | 3 (level 2) |
| Tra một mã chuyến | 3 page | 4 page (lạnh `hit=1 read=3`, nóng `hit=4`) |
| 53 chuyến liền mã | 4 page | 5 page |
| 53 chuyến rải rác | 159 page | 212 page (`hit=102 read=110`) |

Số chuyến gấp 50 lần, nhưng tra một mã chỉ thêm **một** page vì cây index thêm một tầng. Thứ tăng theo kích thước là truy vấn đọc nhiều dòng rải rác — mỗi dòng vẫn tốn trọn đường xuống cây cộng một page heap. [PG-023](/articles/postgresql-btree-index-co-ban) mổ cây B-tree này.

## 7. Vì sao không đọc ghi từng dòng?

- **Phần cứng vốn làm việc theo khối.** Ổ đĩa theo sector, SSD theo page nội bộ, hệ điều hành theo page bộ nhớ của nó. Đọc 147 byte hay 8 KB tốn gần như cùng một lần truy cập; cái đắt là **số lần** truy cập, nhất là truy cập rải rác.
- **Bộ nhớ đệm cần đơn vị cố định.** 16.384 ô cùng cỡ thì quản lý, thay thế và khoá từng ô đơn giản.
- **Địa chỉ ngắn gọn.** `ctid = (số page, số ô)` chỉ hiểu được vì tệp chia thành page cùng cỡ: page 148 luôn bắt đầu ở byte 148 × 8.192.
- **Đọc tuần tự rẻ.** Quét liền nhiều page rẻ hơn nhảy qua lại — `count(fare)` đọc 374 page liền nhau vẫn nhanh.

Các hệ quản trị khác cũng làm vậy, chỉ khác cỡ: InnoDB của MySQL mặc định page 16 KB (`innodb_page_size`, chọn lúc khởi tạo), SQL Server page 8 KB, Oracle chọn `DB_BLOCK_SIZE` lúc tạo database, SQLite chỉnh bằng `PRAGMA page_size`. Không hệ nào đọc ghi đĩa theo từng dòng.

## 8. Mang gì về dự án

- **Nghĩ chi phí theo page, không theo dòng.** Trong `EXPLAIN ANALYZE`, mỗi buffer là một page — thước đo khối lượng việc của truy vấn.
- **Dòng càng nhỏ, page càng chứa nhiều.** Chuyến 147 byte ⇒ 53 chuyến/page. Dòng phình lên khoảng 1,5 KB vì nhét thêm cột ghi chú thì cùng truy vấn phải đọc gấp chừng 10 lần số page. Giá trị rất lớn được PostgreSQL nén và tách ra lưu riêng (TOAST) — đừng dựa vào đó để thiết kế bảng «rộng».
- **Dữ liệu hay đọc cùng nhau nên nằm gần nhau.** Truy vấn theo khoảng của cột thẳng hàng với thứ tự chèn (`requested_at`, `code`) rẻ; truy vấn rải rác đắt gấp nhiều lần dù cùng số dòng.
- **Ghi cũng tính theo page.** Lần sửa đầu sau checkpoint ghi cả ảnh page vào WAL; mỗi index phải sửa thêm một ảnh nữa.
- **Giới hạn kết luận.** Số page, số WAL ở bài này tất định trên dữ liệu lab nhưng phụ thuộc độ dài dòng, chỗ trống từng page và thời điểm checkpoint. Trên dữ liệu của bạn, so **hình dạng** (1 vs 3 vs 159 page, lần đầu vs lần sau checkpoint), đừng so từng byte.

## 9. Bài tập

Lời giải chạy được: `p03/thu-nghiem/pg022-bai-tap.sql` (không có trong lab chính; `node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg022-bai-tap.sql`). Các câu dưới giả định `SET search_path TO bai_cu, public;`.

**Bài 1.** Dự đoán `SELECT * FROM trips WHERE code BETWEEN 112000 AND 112999` (1.000 chuyến liền mã) chạm bao nhiêu page, rồi đo.

<details><summary><b>Lời giải</b></summary>
<p>1.000 ÷ 53,5 ≈ 19 page heap (thêm 1 nếu dải bắt đầu giữa page), cộng gốc và vài lá index. Đo: các chuyến nằm trên đúng 19 page heap; <code>Index Scan using uq_trips_code</code> ra <code>actual rows=1000.00</code>, <code>Buffers: shared hit=24</code> = 19 heap + 5 index. Số dòng gấp 19 lần dải 53 chuyến, số page chỉ gấp 6 lần.</p>
</details>

**Bài 2.** Đẩy `trips` và `uq_trips_code` ra khỏi `shared_buffers`, đọc chuyến 112345, rồi đếm bằng `pg_buffercache` mỗi quan hệ đang chiếm mấy buffer.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">CREATE EXTENSION pg_buffercache WITH SCHEMA public;
SELECT buffers_evicted &gt;= 0 AS ok FROM pg_buffercache_evict_relation('bai_cu.trips');
SELECT buffers_evicted &gt;= 0 AS ok FROM pg_buffercache_evict_relation('bai_cu.uq_trips_code');
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM trips WHERE code = 112345;          -- Buffers: shared read=3
SELECT c.relname, count(*) AS so_buffer
FROM pg_buffercache AS b
JOIN pg_class AS c ON c.relfilenode = b.relfilenode
                  AND c.relnamespace = 'bai_cu'::regnamespace
WHERE b.reldatabase = (SELECT oid FROM pg_database WHERE datname = current_database())
  AND c.relname IN ('trips', 'uq_trips_code')
GROUP BY c.relname ORDER BY c.relname;</code></pre>
<p>Kết quả: <code>trips | 1</code>, <code>uq_trips_code | 2</code>. Một chuyến kéo đúng một page heap (8 KB, chứa thêm khoảng 50 chuyến khác) và hai page index vào bộ nhớ.</p>
</details>

**Bài 3.** Sửa giá 4.000 chuyến (`WHERE code % 5 = 0`) rồi chạy `EXPLAIN ANALYZE SELECT count(*) FROM trips` — trước và sau `VACUUM trips`. Giải thích hai kế hoạch.

<details><summary><b>Lời giải</b></summary>
<p>Trước <code>VACUUM</code>: <code>Seq Scan</code>, <code>Buffers: shared hit=449</code> — bảng đã to từ 374 lên 449 page vì 4.000 phiên bản mới không vừa page cũ, và các page bị sửa mất dấu «mọi dòng nhìn thấy được» nên đếm trên index không còn rẻ. Sau <code>VACUUM</code>: lại <code>Index Only Scan using ix_trips_driver_id</code>, <code>Heap Fetches: 0</code>, 61 page — index cũng to ra (51 → 61) vì mỗi UPDATE không HOT thêm một mục vào mọi index. Kế hoạch phụ thuộc chi phí ước tính nên trên dữ liệu khác điểm chuyển có thể khác; xem <a href="/articles/postgresql-index-only-scan-visibility-map">PG-026</a>.</p>
</details>

## 10. Đọc tiếp

- [PostgreSQL 18 · Database Page Layout](https://www.postgresql.org/docs/18/storage-page-layout.html): page header 24 byte, line pointer 4 byte, `pd_lower`/`pd_upper`, header tuple 23 byte.
- [PostgreSQL 18 · Write Ahead Log](https://www.postgresql.org/docs/18/runtime-config-wal.html): `full_page_writes`, `checkpoint_timeout`, `wal_compression`.
- [PostgreSQL 18 · EXPLAIN](https://www.postgresql.org/docs/18/sql-explain.html): nghĩa của hit, read, dirtied, written trong `BUFFERS`.
- [PostgreSQL 18 · Release Notes](https://www.postgresql.org/docs/18/release-18.html): `BUFFERS` tự bật trong `EXPLAIN ANALYZE`, `pg_buffercache_evict_relation()`.
- Lab: `examples/toi-noi-postgresql/p03/22-page-buffer.sql`, kết quả ở `p03/OUTPUT.txt`, bảng đối chiếu với phiên bản cũ của bài ở `p03/README.md`.

Bài tiếp theo: [PG-023 · B-tree index giúp tìm chuyến như thế nào?](/articles/postgresql-btree-index-co-ban)
