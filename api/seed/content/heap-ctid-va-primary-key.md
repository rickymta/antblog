---
slug: heap-ctid-va-primary-key
title: "Clustered index thật và clustered index nhìn thấy trong PostgreSQL"
summary: "Lược đồ trips của Tới Nơi mở đầu bằng id uuid PRIMARY KEY, dễ khiến ta tưởng primary key quyết định chuyến nằm ở đâu trên đĩa. Đi sâu vào heap, ctid, HOT update và CLUSTER trên PostgreSQL 18.1 để thấy dòng thật sự nằm ở đâu."
category: postgresql
level: intermediate
tags: [postgresql, pg-024, heap, ctid, primary-key, hot-update, cluster, mvcc]
language: vi
---

Mở lược đồ bảng `trips` của [Tới Nơi](/articles/du-an-mau-toi-noi), thứ ta thấy đầu tiên là `id uuid PRIMARY KEY`. Rất dễ nghĩ rằng primary key là thứ quyết định một chuyến nằm ở đâu trên đĩa — giống "clustered index" ở vài hệ quản trị khác. Với PostgreSQL thì không. Bảng có hai lớp tách rời:

- **Clustered index thật — lớp vật lý:** tệp **heap** chia thành các **page** 8 KB, mỗi phiên bản dòng có một địa chỉ vật lý gọi là **`ctid`**. Đây là nơi duy nhất cất dữ liệu dòng.
- **Clustered index nhìn thấy — primary key:** `trips_pkey` chỉ là một cây B-tree riêng ánh xạ `id → ctid`. Về cấu trúc, nó ngang hàng với mọi index khác của bảng.

Nhầm hai lớp này dẫn tới những lỗi rất cụ thể: tin rằng `SELECT` không có `ORDER BY` sẽ trả dòng theo `id`, lưu `ctid` làm khoá, hoặc chạy `CLUSTER` lúc cao điểm. Bài này lần lượt xem từng lớp trên chính bảng `trips`.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-024 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-btree-index-co-ban">PG-023 · B-tree index giúp tìm chuyến như thế nào?</a> và <a href="/articles/page-don-vi-doc-ghi">PG-022 · page</a>. Lab: <code>examples/toi-noi-postgresql/p03</code> (tệp <code>24-heap-ctid-hot.sql</code>; thí nghiệm <code>thu-nghiem/pg024-cluster-hang-doi-khoa.sql</code>, <code>thu-nghiem/pg024-ctid-hot-bai-tap.sql</code>). Sau bài này, bạn phân biệt được heap, ctid, primary key và index; giải thích được vì sao ctid đổi sau UPDATE, khi nào UPDATE là HOT, và vì sao PostgreSQL không có clustered index được duy trì.</p>
</div>

## 1. Dữ liệu và cách chạy

Bài dùng bộ dữ liệu thu nhỏ **20.000 chuyến** (30 ngày, 6 thành phố) theo lược đồ lõi của hồ sơ, trong schema riêng `bai_cu`. Bản trước của bài chạy trên một bộ sinh bằng `random()`; lab Phần 3 sinh lại **tất định** và chạy trên **PostgreSQL 18.1** (bản trước ghi nhầm 18.6). Vì thế giá trị `id` và số ô trong `ctid` khác bản trước, còn hình dạng và đa số con số — cost của kế hoạch, `correlation`, tỉ lệ HOT — thì khớp. Bảng 20.000 dòng nhỏ hơn mẫu 30.000 dòng của `ANALYZE` nên thống kê ở đây cũng tất định.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p03 --only 24
node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg024-cluster-hang-doi-khoa.sql
node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg024-ctid-hot-bai-tap.sql
```

Hai tệp thí nghiệm không có trong lab chính, nằm ở `p03/thu-nghiem/`. Kết quả dưới đây chép từ lần chạy thật, chỗ lược bỏ ghi bằng `…`.

## 2. Hai lớp của bảng trips

Đọc 5 dòng đầu **theo thứ tự vật lý** (không `ORDER BY`):

```sql
SELECT ctid, code, status FROM trips LIMIT 5;
```

```text
 ctid  |  code  |  status
-------+--------+-----------
 (0,1) | 100000 | completed
 (0,2) | 100001 | completed
 (0,3) | 100002 | completed
 (0,4) | 100003 | completed
 (0,5) | 100004 | cancelled
```

Còn đây là 5 dòng đầu **theo primary key**:

```sql
SELECT ctid, left(id::text, 8) AS id, code FROM trips ORDER BY id LIMIT 5;
```

```text
   ctid   |    id    |  code
----------+----------+--------
 (177,53) | 00011769 | 109521
 (255,10) | 0002832a | 113651
 (180,17) | 000b0dac | 109644
 (218,40) | 000c662d | 111701
 (138,41) | 000d8f74 | 107422
```

Theo thứ tự vật lý, mã chuyến tăng dần từ page 0 — chuyến được chèn theo thời gian. Theo `id`, năm dòng đầu nằm ở **năm page khác nhau**: đọc theo primary key là đọc rải khắp tệp heap. (Không có `ORDER BY` thì PostgreSQL không hứa thứ tự nào; ở đây là thứ tự quét heap, và nó đổi khi bảng bị sửa.)

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 400" role="img" aria-label="Lớp nhìn thấy gồm uq_trips_code, trips_pkey và ba index khác; lớp vật lý là các page heap. Ba mục liền nhau của uq_trips_code trỏ vào cùng page 0, còn ba mục liền nhau của trips_pkey trỏ vào ba page 177, 255 và 180.">
<defs>
<marker id="pg024-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="pg024-ph" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="16" width="880" height="164" rx="12" class="f-bg s-pk dash" stroke-width="1.3"/>
<text x="36" y="40" class="t-b t-pk">LỚP NHÌN THẤY — schema, index, primary key</text>
<rect x="40" y="56" width="260" height="110" rx="8" class="f-surface s-line"/>
<text x="52" y="76" class="t-mono t-b">uq_trips_code</text>
<text x="52" y="92" class="t-small">B-tree (code) · mã chuyến tăng dần</text>
<text x="52" y="116" class="t-mono">100000 → (0,1)</text>
<text x="52" y="134" class="t-mono">100001 → (0,2)</text>
<text x="52" y="152" class="t-mono">100002 → (0,3)</text>
<rect x="320" y="56" width="300" height="110" rx="8" class="f-pk-soft s-pk" stroke-width="1.8"/>
<text x="332" y="76" class="t-mono t-b t-pk">trips_pkey</text>
<text x="332" y="92" class="t-small">B-tree (id) · primary key · uuid kiểu v4</text>
<text x="332" y="116" class="t-mono">00011769… → (177,53)</text>
<text x="332" y="134" class="t-mono">0002832a… → (255,10)</text>
<text x="332" y="152" class="t-mono">000b0dac… → (180,17)</text>
<rect x="640" y="56" width="250" height="110" rx="8" class="f-surface s-line"/>
<text x="652" y="76" class="t-b">3 index còn lại</text>
<text x="652" y="98" class="t-mono t-10">ix_trips_city_requested</text>
<text x="652" y="116" class="t-mono t-10">ix_trips_customer_requested</text>
<text x="652" y="134" class="t-mono t-10">ix_trips_driver_id</text>
<text x="652" y="154" class="t-small">cũng chỉ giữ khoá → ctid</text>
<text x="190" y="214" class="t-b">mọi index chỉ giữ ctid</text>
<rect x="20" y="240" width="880" height="150" rx="12" class="f-bg s-phys" stroke-width="1.6"/>
<rect x="30" y="254" width="200" height="86" rx="8" class="f-surface s-phys"/>
<text x="40" y="272" class="t-small">heap · page 0</text>
<text x="40" y="294" class="t-mono t-10">(0,1) │ 100000</text>
<text x="40" y="312" class="t-mono t-10">(0,2) │ 100001</text>
<text x="40" y="330" class="t-mono t-10">(0,3) │ 100002</text>
<rect x="250" y="254" width="200" height="86" rx="8" class="f-surface s-phys"/>
<text x="260" y="272" class="t-small">heap · page 177</text>
<rect x="254" y="300" width="192" height="19" rx="4" class="f-pk-soft"/>
<text x="260" y="314" class="t-mono t-10 t-b">(177,53) │ 109521</text>
<rect x="470" y="254" width="200" height="86" rx="8" class="f-surface s-phys"/>
<text x="480" y="272" class="t-small">heap · page 180</text>
<rect x="474" y="300" width="192" height="19" rx="4" class="f-pk-soft"/>
<text x="480" y="314" class="t-mono t-10 t-b">(180,17) │ 109644</text>
<rect x="690" y="254" width="200" height="86" rx="8" class="f-surface s-phys"/>
<text x="700" y="272" class="t-small">heap · page 255</text>
<rect x="694" y="300" width="192" height="19" rx="4" class="f-pk-soft"/>
<text x="700" y="314" class="t-mono t-10 t-b">(255,10) │ 113651</text>
<text x="36" y="362" class="t-b t-phys">LỚP VẬT LÝ — heap + ctid: nơi dòng thật sự nằm (cột: ctid │ code)</text>
<text x="36" y="380" class="t-small">Dòng nằm theo thứ tự được chèn, khoảng 53 chuyến mỗi page 8 KB. Không cột nào, kể cả id, quyết định thứ tự này.</text>
<line x1="170" y1="168" x2="170" y2="250" class="s-phys" stroke-width="1.8" marker-end="url(#pg024-ph)"/>
<path d="M380 168 C 380 220, 350 240, 350 298" class="f-none s-pk" stroke-width="1.8" marker-end="url(#pg024-pk)"/>
<path d="M470 168 C 470 222, 800 220, 800 298" class="f-none s-pk" stroke-width="1.8" marker-end="url(#pg024-pk)"/>
<path d="M560 168 C 560 218, 570 240, 570 298" class="f-none s-pk" stroke-width="1.8" marker-end="url(#pg024-pk)"/>
</svg>
</div>
<figcaption><b>Hình 1 · Hai lớp của <code>trips</code>.</b> Ba mục liền nhau của <code>uq_trips_code</code> trỏ vào cùng page 0 vì chuyến được chèn theo thứ tự mã. Ba mục liền nhau của <code>trips_pkey</code> trỏ vào ba page cách xa nhau: primary key có thứ tự trong cây của nó, heap thì không. Số liệu từ <code>24-heap-ctid-hot.sql</code>, mục 1.</figcaption>
</figure>

## 3. ctid trỏ vào đâu

`ctid` là một **cột hệ thống** ẩn có ở mọi bảng, ghi vị trí vật lý của **phiên bản dòng** trong bảng. Nó có dạng `(page, ô)`: `(0,2)` là page 0, ô số 2 trong **mảng line pointer** ở đầu page. Ô đó mới giữ vị trí byte thật của dòng; dữ liệu dòng ghi từ cuối page ngược lên (giải phẫu page ở [PG-022](/articles/page-don-vi-doc-ghi)). Nhờ lớp gián tiếp này, PostgreSQL dồn chỗ trống trong page mà ctid không đổi.

Tra thẳng theo địa chỉ vật lý là **Tid Scan**, bỏ qua mọi index — chỉ dùng để chẩn đoán, lý do ở mục 6 và bài tập 3:

```text
EXPLAIN (COSTS OFF) SELECT * FROM trips WHERE ctid = '(0,1)';

 Tid Scan on trips
   TID Cond: (ctid = '(0,1)'::tid)
```

## 4. Tìm theo primary key vẫn phải xuống heap

Primary key không có "đường tắt" nào so với index thường. Tra theo mã chuyến hay theo `id` đều đi cây B-tree lấy ctid rồi đọc page heap (đường đi chi tiết ở [PG-023](/articles/postgresql-btree-index-co-ban)):

```text
EXPLAIN SELECT * FROM trips WHERE code = 150000;

 Index Scan using uq_trips_code on trips  (cost=0.29..8.30 rows=1 width=125)
   Index Cond: (code = 150000)

EXPLAIN (COSTS OFF) SELECT * FROM trips WHERE id = 'ab31fe65-42a9-425f-8abd-48b358eb8098';

 Index Scan using trips_pkey on trips
   Index Cond: (id = 'ab31fe65-42a9-425f-8abd-48b358eb8098'::uuid)
```

Cost `0.29..8.30 rows=1 width=125` khớp từng chữ số với bản trước của bài. Nhiều dòng khớp (chuyến của một thành phố trong một ngày) thì planner chọn **Bitmap Heap Scan**: gom ctid từ index, sắp theo page, đọc mỗi page heap một lần. Lab thay `now()` của bản trước bằng một mốc thời gian cố định:

```text
EXPLAIN SELECT * FROM trips
WHERE city_id = 1 AND requested_at >= TIMESTAMPTZ '2026-10-31 01:00+07' - interval '1 day';

 Bitmap Heap Scan on trips  (cost=5.43..247.17 rows=111 width=125)
   Recheck Cond: ((city_id = 1) AND (requested_at >= …))
   ->  Bitmap Index Scan on ix_trips_city_requested  (cost=0.00..5.40 rows=111 width=0)
         Index Cond: ((city_id = 1) AND (requested_at >= …))
```

## 5. Heap đang thẳng hàng với cột nào?

Sau `ANALYZE`, PostgreSQL ghi cho mỗi cột hệ số **`correlation`** (tương quan) giữa thứ tự vật lý trong heap và thứ tự giá trị của cột: 1 là thẳng hàng tuyệt đối, 0 là không liên quan.

```sql
SELECT attname, round(correlation::numeric, 3) AS correlation
FROM pg_stats
WHERE schemaname = 'bai_cu' AND tablename = 'trips'
  AND attname IN ('requested_at', 'code', 'city_id', 'fare', 'driver_id', 'id', 'customer_id')
ORDER BY abs(correlation) DESC, attname;
```

```text
   attname    | correlation
--------------+-------------
 code         |       1.000
 requested_at |       1.000
 city_id      |       0.167
 fare         |       0.014
 id           |       0.009
 customer_id  |      -0.007
 driver_id    |      -0.001
```

(Bản trước của bài thiếu điều kiện `schemaname`: database có hai schema cùng tên bảng `trips` thì câu đó trộn thống kê của cả hai.) Heap thẳng hàng tuyệt đối với **thời gian đặt** và **mã chuyến** — đó là thứ tự chèn — và không liên quan tới `id`. Ở Tới Nơi, "clustered index thật" tự nhiên đi theo `requested_at`, còn primary key thì ngẫu nhiên. Đó là tin tốt cho báo cáo theo khoảng thời gian: PG-023 đo được Index Scan trên `requested_at` vẫn được chọn khi lấy tới ~72 % bảng 1 triệu chuyến, và BRIN (PG-028) dựa vào đúng tính chất này.

<div class="tkh-callout">
<span class="tkh-callout-title">uuid v4 hay uuid v7?</span>
<p><code>gen_random_uuid()</code> sinh uuid version 4 ngẫu nhiên nên <code>correlation</code> của <code>id</code> ≈ 0. Heap không bị ảnh hưởng (heap không xếp theo khoá), nhưng mỗi chuyến mới chèn vào một chỗ ngẫu nhiên trong cây <code>trips_pkey</code>: PG-023 đo cây primary key của bảng 1 triệu chuyến chỉ đầy 71 % so với 90 % của cây mã tăng dần. PostgreSQL 18 thêm <code>uuidv7()</code> — uuid có thể sắp xếp theo thời gian — nếu bạn muốn primary key cũng chèn ở mép phải cây như <code>code</code>.</p>
</div>

## 6. id không đổi, ctid thì đổi

PostgreSQL không ghi đè dòng: mỗi `UPDATE` tạo một **phiên bản mới** của dòng (MVCC — điều khiển đồng thời đa phiên bản), và phiên bản mới có địa chỉ mới. Hồ sơ cho biết `trips` nhận ≈ 560 UPDATE/giây lúc cao điểm, nên đây là chi tiết đáng chú ý nhất của bài. Ba UPDATE chạy trong một giao dịch, đếm bằng `pg_stat_xact_user_tables` (thống kê của riêng giao dịch đang chạy):

| Bước | Câu lệnh | ctid | HOT? | upd / hot |
|---|---|---|---|---|
| — | chuyến 119985 đang `searching`, ở page cuối 373 | `(373,32)` | | |
| ① | tài xế 8001 nhận chuyến: `SET driver_id = …, status = 'accepted', …` | `(373,47)` | Không — `driver_id` có index | 1 / 0 |
| ② | sửa giá ngay sau đó: `SET fare = fare + 5000` | `(373,48)` | **Có** — `fare` không có index, page còn chỗ | 2 / 1 |
| ③ | sửa giá chuyến cũ 110000 ở `(186,54)`: `SET fare = fare + 1000` | `(373,49)` | Không — page 186 đầy, phải dời page | 3 / 1 |

Ở lớp nhìn thấy, mỗi chuyến vẫn là cùng một chuyến (`id` không đổi). Ở lớp vật lý, cả ba đều sinh phiên bản mới ở địa chỉ mới; phiên bản cũ nằm lại, đã "chết", chờ dọn. Soi page 373 bằng `pageinspect` (`heap_page_items`) thấy chuỗi phiên bản — `t_ctid` của phiên bản cũ trỏ sang phiên bản mới:

```text
 lp |  t_ctid  | hot_updated
----+----------+-------------
 32 | (373,47) | f
 47 | (373,48) | t
 48 | (373,48) | f
```

Ô 32 là bản gốc (trỏ sang 47, không HOT vì ① đổi `driver_id`), ô 47 là bản sau ① (đã cập nhật kiểu HOT sang 48), ô 48 là bản hiện hành (trỏ chính nó).

**HOT update** (Heap-Only Tuple — phiên bản chỉ nằm trong heap) là lối thoát: nếu (1) không cột nào được index tham chiếu bị đổi giá trị **và** (2) page đang chứa dòng còn chỗ cho phiên bản mới, PostgreSQL đặt phiên bản mới cùng page, nối vào chuỗi, và **không index nào phải thêm mục**. Mục index vẫn trỏ `(373,47)`; khi đọc, PostgreSQL đi theo chuỗi trong page tới `(373,48)`. Bước ③ cho thấy điều kiện (2) quan trọng không kém điều kiện (1): page chèn đầy (mặc định `fillfactor = 100`) thì một UPDATE "vô hại" cũng làm dòng dời page và ghi vào cả 5 index. Bài tập 1 thêm trường hợp ngược: cùng page vẫn chưa chắc là HOT.

Tỉ lệ HOT của bảng sau ba câu là `n_tup_upd 3 · n_tup_hot_upd 1 · hot_pct 33.3`. Câu kiểm tra trên bảng thật (lab gọi `pg_stat_force_next_flush()` trước, vì thống kê tích luỹ được đẩy định kỳ):

```sql
SELECT n_tup_upd, n_tup_hot_upd,
       round(100.0 * n_tup_hot_upd / nullif(n_tup_upd, 0), 1) AS hot_pct
FROM pg_stat_user_tables
WHERE relid = 'bai_cu.trips'::regclass;
```

## 7. fillfactor: chừa chỗ cho HOT

`fillfactor` của bảng là phần trăm mỗi page được lấp khi chèn; phần còn lại để dành cho phiên bản mới của UPDATE. Lab chép 20.000 chuyến ra hai bảng, `fillfactor` 100 và 90, cùng có index `driver_id`, rồi sửa giá 2.000 chuyến cũ rải đều (mỗi chuyến thứ 10):

```text
 relname | so_page | n_tup_upd | n_tup_hot_upd | hot_pct
---------+---------+-----------+---------------+---------
 ff100   |     384 |      2000 |            21 |     1.1
 ff90    |     448 |      2000 |          2000 |   100.0
```

(Ghép từ hai câu của lab; `CREATE TABLE AS` ghi bằng đường chèn hàng loạt nên ra 384 page chứ không đúng 374 như bảng gốc.) Ta tốn thêm 17 % dung lượng heap để đổi lấy 100 % HOT với kiểu UPDATE này. Đánh đổi đó đáng với bảng bị sửa nhiều như `trips`; bảng chỉ chèn như `trip_events` thì giữ 100.

## 8. Primary key chỉ là một B-tree UNIQUE

```text
     index     | amname | indisunique | indisprimary | contype
---------------+--------+-------------+--------------+---------
 trips_pkey    | btree  | t           | t            | p
 uq_trips_code | btree  | t           | f            | u
```

Primary key gồm một index B-tree **UNIQUE**, ràng buộc **NOT NULL** và cờ "primary" (mỗi bảng tối đa một). Về cấu trúc, `trips_pkey` ngang hàng `uq_trips_code`. Khoá ngoại cũng tham chiếu được cột UNIQUE: bảng `ghi_chu_chuyen (trip_code bigint REFERENCES trips (code))` từ chối mã không tồn tại với `23503 … violates foreign key constraint`.

Mỗi index là thêm một cây trỏ xuống heap. Trên bộ 20.000 chuyến, 5 index của `trips` chiếm **3,3 MB**, nhiều hơn chính bảng **2,9 MB** (374 page × 8.192 byte = 2,92 MiB; bản trước ghi 3,4 và 3,0 MB do làm tròn lên) — và mỗi UPDATE không HOT ghi vào cả 5.

## 9. CLUSTER: xếp heap theo index một lần, không duy trì

`CLUSTER trips USING trips_pkey` chép lại heap theo thứ tự `id`. Kết quả trong lab:

```text
-- pg_locks của phiên vừa chạy CLUSTER, trước COMMIT:
 AccessExclusiveLock
 ShareLock
-- sau CLUSTER + ANALYZE:
   attname    | correlation
--------------+-------------
 id           |       1.000
 code         |       0.009
 requested_at |       0.009
```

Heap thẳng hàng với `id` và **mất** sự thẳng hàng tự nhiên với thời gian đặt, thứ mà các báo cáo đang tận dụng — với `trips`, xếp theo uuid ngẫu nhiên là phản tác dụng. Mọi ctid đều đổi. PostgreSQL chỉ **ghi nhớ** index đã dùng (cờ `pg_index.indisclustered = t`) để lần sau gõ `CLUSTER trips` không cần `USING`; `\d trips` in `"trips_pkey" PRIMARY KEY, btree (id) CLUSTER` (thí nghiệm `pg024-ctid-hot-bai-tap.sql`). Đó là "clustered index nhìn thấy" theo nghĩa thứ hai: có chữ CLUSTER, nhưng không có gì được duy trì. Lab chèn tiếp 500 chuyến mới (id ngẫu nhiên): tất cả nằm ở **page 373–380**, cuối heap, không chen vào đúng vị trí theo id; `correlation` của `id` tụt về khoảng 0,95 sau một lần `ANALYZE` (số lấy mẫu, dao động giữa các lần chạy). Tài liệu nói thẳng: "Clustering is a one-time operation".

`CLUSTER` giữ khoá **ACCESS EXCLUSIVE** suốt thời gian chạy. Thí nghiệm `pg024-cluster-hang-doi-khoa.sql` cho thấy hệ quả còn tệ hơn "không ai đọc được trong lúc chạy":

| Bước | Phiên | Lệnh | Kết quả |
|---|---|---|---|
| T1 | A | `BEGIN; SELECT count(*) FROM bai_cu.trips;` | 20000, giữ AccessShareLock tới khi COMMIT |
| T2 | B | `CLUSTER bai_cu.trips USING trips_pkey;` | **chờ** — bị A chặn |
| T3 | C | `SELECT count(*) FROM bai_cu.trips;` | **chờ** — bị **B** chặn, dù tương thích với khoá của A |
| T4 | D | đọc `pg_locks` | `lab-A AccessShareLock t` · `lab-B AccessExclusiveLock f` · `lab-C AccessShareLock f` |
| T5 | A | `COMMIT;` | B chạy xong `CLUSTER`, rồi C mới trả 20000 |

Các yêu cầu khoá xếp thành hàng đợi: một lệnh cần ACCESS EXCLUSIVE **đang chờ** đã chặn mọi `SELECT` đến sau nó. Với Tới Nơi lúc cao điểm, chỉ cần một giao dịch đọc chạy lâu là `CLUSTER` khiến không ai mở được lịch sử chuyến cho tới khi giao dịch đó xong và `CLUSTER` chạy hết. Khoá và hàng đợi khoá là chủ đề của Phần 4.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Những lúc mọi ctid cùng đổi</span>
<p><code>VACUUM FULL</code> và <code>CLUSTER</code> chép toàn bộ bảng sang tệp mới: mọi dòng nhận <code>ctid</code> mới, index dựng lại, cần thêm chỗ đĩa cho bản sao, và bảng bị khoá <code>ACCESS EXCLUSIVE</code> suốt thời gian chạy — không đọc, không ghi. UPDATE thường cũng đổi ctid của từng dòng. Vì vậy tài liệu PostgreSQL khuyên không dùng <code>ctid</code> làm định danh dòng mà dùng primary key.</p>
</div>

## 10. Bảng đối chiếu hai lớp

| | Clustered index thật — heap + ctid | Clustered index nhìn thấy — primary key |
|---|---|---|
| Là gì | Tệp heap chia page 8 KB; địa chỉ phiên bản dòng `ctid = (page, ô)` | B-tree riêng `trips_pkey`: `id → ctid`, kèm UNIQUE + NOT NULL |
| Chứa dữ liệu dòng | Có — nơi duy nhất | Không — chỉ khoá và ctid |
| Quyết định thứ tự trên đĩa | Có: dòng nằm theo chỗ trống lúc ghi (ở Tới Nơi ≈ theo `requested_at`) | Không: chỉ có thứ tự trong cây của chính nó |
| Ổn định | Không: đổi sau UPDATE, `VACUUM FULL`, `CLUSTER` | Có: `id` không đổi |
| Ai dùng | Bộ máy PostgreSQL: mọi index, Bitmap Heap Scan, Tid Scan | Ứng dụng: khoá ngoại, API, URL |
| Có nên dựa vào trong code không | Không lưu, không so sánh ctid | Luôn dùng `id` làm danh tính |

## 11. Hiểu sai thường gặp

| Hiểu sai | Thực tế (đã kiểm trong lab) | Hệ quả nếu tin |
|---|---|---|
| "Primary key là clustered index" | `trips_pkey` là một B-tree như `uq_trips_code`; heap theo thứ tự chèn | Bỏ `ORDER BY` vì "dữ liệu vốn theo id" ⇒ kết quả sai thứ tự |
| "`CLUSTER` làm bảng luôn xếp theo index" | Một lần; 500 dòng mới nằm ở page 373–380 | Chạy lại `CLUSTER` định kỳ, mỗi lần khoá cả bảng |
| "ctid là id nhanh nhất" | ctid đổi sau mỗi UPDATE, `VACUUM FULL`, `CLUSTER` | Sửa trượt hoặc không tìm thấy dòng (bài tập 3) |
| "UPDATE cột không có index thì luôn rẻ" | Bước ③: page đầy ⇒ dời page ⇒ ghi cả 5 index | Bỏ qua `fillfactor` cho bảng sửa nhiều |
| "UPDATE cùng page là HOT" | Đổi giá trị cột có index thì không HOT dù cùng page (bài tập 1) | Đo HOT sai, thêm index vào cột hay sửa |

## 12. Mang gì về dự án

- **Primary key không phải clustered index.** Truy vấn cần thứ tự phải có `ORDER BY`.
- **`ctid` chỉ để chẩn đoán.** Không lưu nó, không đưa ra API, không dùng làm khoá phân trang ([PG-018](/articles/postgresql-pagination-keyset) dùng khoá nghiệp vụ).
- **Mỗi index là thêm một cây phải ghi.** Trước khi thêm index thứ 6 vào bảng bị sửa ≈ 560 lần/giây, hỏi nó có đáng không — PG-025 đo chi phí ghi.
- **Bảng sửa nhiều: cân nhắc `fillfactor`** và theo dõi tỉ lệ `n_tup_hot_upd / n_tup_upd`. Dọn phiên bản cũ (VACUUM) là chủ đề Phần 5.

## 13. Bài tập

**Bài 1.** Chuyến 119990 đang ở page 373 (page còn chỗ). Dự đoán HOT hay không cho ba UPDATE: (a) đổi `status`; (b) `SET requested_at = requested_at`; (c) `SET requested_at = requested_at + interval '1 minute'`.

<details><summary><b>Lời giải</b></summary>
<p>(a) HOT — <code>status</code> không có index trong <code>bai_cu</code>. (b) Vẫn HOT — <code>requested_at</code> có index nhưng giá trị không đổi. (c) KHÔNG HOT — giá trị cột có index (<code>ix_trips_city_requested</code>, <code>ix_trips_customer_requested</code>) đổi, dù phiên bản mới vẫn ở page 373. Đếm trong một giao dịch: <code>1 | 1</code> → <code>2 | 2</code> → <code>3 | 2</code>. Lab đo trên một kết nối mới: ở phiên vừa commit giao dịch ①②③, <code>pg_stat_xact_user_tables</code> còn cộng dồn số đếm chưa đẩy (ra <code>4 | 2</code> thay vì <code>1 | 1</code>). Tệp <code>p03/thu-nghiem/pg024-ctid-hot-bai-tap.sql</code>.</p>
</details>

**Bài 2.** Sau ba UPDATE ①②③, chạy `VACUUM (INDEX_CLEANUP ON) trips`. Ô 32, 47, 48 của page 373 thành gì? Tra theo `id` của chuyến 119985 còn tới đúng dòng không?

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT lp, lp_flags, lp_off
FROM heap_page_items(get_raw_page('bai_cu.trips', 373)) WHERE lp IN (32, 47, 48);</code></pre>
<p><code>lp_flags</code>: ô 32 = 0 (UNUSED — bản gốc đã dọn cùng mục index trỏ vào nó), ô 47 = 2 (REDIRECT, <code>lp_off</code> = 48 — mục index tạo ở ① vẫn trỏ (373,47), heap chuyển tiếp sang 48), ô 48 = 1 (NORMAL, bản hiện hành). Tra theo <code>id</code> trả <code>(373,48)</code>. Page 186 còn 53 ô: ô 54 (bản cũ của ③) là ô cuối nên bị cắt khỏi mảng line pointer. Dùng <code>INDEX_CLEANUP ON</code> vì khi dòng chết ít, VACUUM có thể bỏ bước dọn index (PG-026).</p>
</details>

**Bài 3.** Một service lưu `ctid` `(373,32)` của chuyến 119985 lúc đọc, rồi sau bước ① gửi `UPDATE … WHERE ctid = '(373,32)'`. Chuyện gì xảy ra?

<details><summary><b>Lời giải</b></summary>
<p><code>SELECT count(*) FROM trips WHERE ctid = '(373,32)'</code> trả <b>0</b>: phiên bản ở địa chỉ đó đã chết, nên UPDATE cùng điều kiện cũng không khớp dòng nào — sửa trượt mà không có lỗi. Tìm theo khoá: <code>119985 | (373,48) | accepted</code>. Tệ hơn, sau VACUUM ô 32 trống (bài 2) và có thể được dùng lại cho một dòng KHÁC. Dùng <code>id</code> hoặc mã nghiệp vụ làm định danh; chống ghi đè đồng thời là việc của khoá dòng (Phần 4).</p>
</details>

## 14. Đọc tiếp

- [PostgreSQL 18 · System Columns](https://www.postgresql.org/docs/18/ddl-system-columns.html): định nghĩa `ctid` và cảnh báo không dùng làm định danh.
- [PostgreSQL 18 · Heap-Only Tuples (HOT)](https://www.postgresql.org/docs/18/storage-hot.html): điều kiện HOT, `fillfactor`, theo dõi HOT.
- [PostgreSQL 18 · CLUSTER](https://www.postgresql.org/docs/18/sql-cluster.html) và [VACUUM](https://www.postgresql.org/docs/18/sql-vacuum.html) (`FULL`): khoá, bản sao bảng, chỉ sắp xếp một lần.
- [PostgreSQL 18 · UUID Functions](https://www.postgresql.org/docs/18/functions-uuid.html) (`uuidv7()`, thêm ở bản 18 theo [release notes](https://www.postgresql.org/docs/release/18.0/)) · [pageinspect](https://www.postgresql.org/docs/18/pageinspect.html) (`heap_page_items`).
- Lab: `p03/24-heap-ctid-hot.sql` (kết quả ở `p03/OUTPUT.txt`, bảng đối chiếu với bản trước ở `p03/README.md`), thí nghiệm `p03/thu-nghiem/pg024-*.sql`.

Bài tiếp theo: [PG-025 · Index nhiều cột, partial và expression](/articles/postgresql-index-nhieu-cot-partial-expression).
