---
slug: postgresql-btree-index-co-ban
title: "B-tree index giúp tìm chuyến như thế nào?"
summary: "Màn «Chuyến của tôi» của Tới Nơi tìm 40 chuyến của một khách mà phải đọc cả 19.441 page của bảng 1 triệu chuyến. Thêm một B-tree index: 43 page. Đi từ gốc → nhánh → lá → heap của cây và xem vì sao có index mà planner vẫn chọn Seq Scan."
category: postgresql
level: beginner
tags: [postgresql, pg-023, index, btree, seq-scan, bitmap-scan, toi-noi]
language: vi
---

Khách `0910000500` mở màn «Chuyến của tôi» trên app Tới Nơi. Họ mới đi 40 chuyến từ tháng 5, nhưng bảng `trips` đã có **1.000.000** chuyến. Nếu database không có cách nào biết 40 chuyến đó nằm ở đâu, nó chỉ còn một đường: đọc từng chuyến một, giữ lại 40, bỏ 999.960.

Hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi) đã khai sẵn `ix_trips_customer_requested` cho đúng truy vấn này. Bài này cố ý **bỏ** index đó để thấy bảng không có index trông ra sao, rồi tạo lại một **B-tree index** đơn giản trên `customer_id`, đếm số page đọc trước và sau, đi qua từng tầng của cây, và kết thúc bằng câu hỏi hay gặp nhất: có index rồi, vì sao planner vẫn đọc cả bảng?

<div class="tkh-callout">
<span class="tkh-callout-title">PG-023 · Phần 3/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/page-don-vi-doc-ghi">PG-022 · Đơn vị đọc ghi nhỏ nhất của cơ sở dữ liệu là gì?</a> (page, buffer) và <a href="/articles/postgresql-explain-ke-hoach-thuc-thi">PG-021 · Đọc EXPLAIN</a>. Lab: <code>examples/toi-noi-postgresql/p03</code> (tệp <code>23-btree.sql</code>, phép đo ở <code>DO-DAC.md</code>). Sau bài này, bạn so sánh được đường tìm dữ liệu khi có index và khi chưa có index bằng số page đọc, và giải thích vì sao planner vẫn chọn Seq Scan khi điều kiện khớp nhiều dòng.</p>
</div>

## 1. Dữ liệu và cách chạy

Từ Phần 3, lab chạy trên **PostgreSQL 18.1 thật** (Phần 1–2 chạy PGlite 17.5). Bảng `trips` có 1.000.000 chuyến từ 01/05 đến 31/10/2026, chèn theo thứ tự thời gian, chiếm **19.441 page = 152 MB** — lớn hơn `shared_buffers` 128 MB của cluster lab. Mỗi tệp bài chạy trên một database mới chép từ nền, nên lúc bắt đầu `shared_buffers` chưa có page nào của bảng. Các số đo thời gian trong bài lấy từ `p03/DO-DAC.md`: máy i7-10700 (16 luồng), 64 GB RAM, NVMe, Windows 11 Pro; parallel query, JIT, autovacuum tắt để tái lập; trung vị 7 lần (5 lần với câu nặng); máy có tải nền nên số tuyệt đối dao động khoảng ±20 %.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p03 --only 23                                            # bài chính
node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg023-btree-bai-tap.sql
```

Tệp thứ hai (ví dụ lỗi thường gặp, page gốc của cây, lời giải bài tập) không có trong lab chính, nằm ở `p03/thu-nghiem/pg023-btree-bai-tap.sql`. Kết quả dưới đây chép từ lần chạy thật, bỏ dòng `-- [lab]`. Ba khác biệt của PostgreSQL 18 so với bản 17 sẽ gặp ngay: `EXPLAIN ANALYZE` tự kèm dòng `Buffers`, số dòng in hai chữ số thập phân (`rows=40.00`), và dòng mới `Index Searches` (số lần đi xuống cây).

## 2. Chưa có index: đọc cả bảng

```sql
DROP INDEX ix_trips_customer_requested;

EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, requested_at, fare FROM trips WHERE customer_id = :'kh';
```

`:'kh'` là biến psql, gán bằng `\gset` từ `SELECT id AS kh FROM customers WHERE phone = '0910000500'`.

```text
 Seq Scan on trips (actual rows=40.00 loops=1)
   Filter: (customer_id = '3b3bf766-29e0-4cca-8a69-7b8beba432de'::uuid)
   Rows Removed by Filter: 999960
   Buffers: shared read=19441
```

**Seq Scan** (quét tuần tự) đọc heap từ page đầu tới page cuối và thử điều kiện trên từng dòng. `Buffers: shared read=19441` đúng bằng số page của bảng: để trả 40 dòng, PostgreSQL đã chạm **mọi** page. `read` nghĩa là page chưa có trong `shared_buffers` — PostgreSQL phải xin hệ điều hành, và ở máy lab page đó thường đã nằm trong cache của hệ điều hành chứ chưa chắc là đọc đĩa thật.

Thời gian đo trong `DO-DAC.md` (G1-c): trung vị **65,4 ms** cho một lần mở màn «Chuyến của tôi». Không chậm với một người, nhưng mỗi lần là một lượt quét 152 MB và một CPU bận 65 ms; hồ sơ ước tính ≈ 30.000 truy vấn đọc mỗi giây lúc cao điểm (mọi loại câu), nên chỉ cần một phần nhỏ trong đó quét cả bảng là đủ gây nghẽn — và bảng còn lớn thêm 2 triệu chuyến mỗi ngày.

## 3. Thêm một B-tree: 43 page

```sql
CREATE INDEX ix_trips_customer ON trips (customer_id);
```

```text
 Bitmap Heap Scan on trips (actual rows=40.00 loops=1)
   Recheck Cond: (customer_id = '3b3bf766-29e0-4cca-8a69-7b8beba432de'::uuid)
   Heap Blocks: exact=40
   Buffers: shared read=43
   ->  Bitmap Index Scan on ix_trips_customer (actual rows=40.00 loops=1)
         Index Cond: (customer_id = '3b3bf766-29e0-4cca-8a69-7b8beba432de'::uuid)
         Index Searches: 1
         Buffers: shared read=3
```

Từ **19.441** xuống **43** page: 3 page của index (nút con dưới cùng) cộng 40 page heap (`Heap Blocks: exact=40` — 40 chuyến nằm ở 40 page khác nhau, vì khách đặt rải rác trong 6 tháng). Dòng `Buffers` của nút cha là tổng, gồm cả nút con.

Phép đo thời gian của lab dùng index hai cột có sẵn của nền (`customer_id, requested_at DESC`), không phải index một cột vừa tạo, nhưng cùng đường đi — 44 page: trung vị **0,04 ms** khi page đã nằm trong `shared_buffers`, **0,32 ms** khi đẩy bảng và index ra khỏi `shared_buffers` trước mỗi lần (G1-a, G1-b, 7 lần đo). So với 65,4 ms thì chênh nhau ba bậc độ lớn. Giới hạn: một máy, cache hệ điều hành luôn nóng ("lạnh" chỉ là `shared_buffers` rỗng), dữ liệu tổng hợp — tin vào **tỉ lệ page** hơn là con số mili-giây.

Cái giá: một cấu trúc nữa trên đĩa, `ix_trips_customer` = **12,7 MB**, và mỗi lần ghi chuyến phải cập nhật thêm nó (PG-025 đo chi phí ghi).

## 4. Bên trong cây B-tree

**B-tree** (cây cân bằng) là loại index mặc định của `CREATE INDEX`. B-tree lưu trong một tệp riêng, cũng chia page 8 KB như heap, gồm ba loại page:

- **Page lá** (leaf): mỗi mục là một khoá đã sắp thứ tự kèm **ctid** — địa chỉ `(page, ô)` của dòng trong heap. Mọi page cùng tầng nối với page kề hai chiều, nên đọc một khoảng khoá là đi xuống một lần rồi đi ngang.
- **Page nhánh** (internal): mỗi mục là một khoá phân cách kèm số page con.
- **Page gốc** (root): page nhánh trên cùng. Ngoài ra có một **metapage** ở block 0 ghi lại page gốc nằm ở đâu và cây cao mấy tầng.

Extension `pageinspect` và `pgstattuple` cho xem từng phần (cần quyền superuser):

```sql
SELECT level FROM bt_metap('ix_trips_customer');                    -- 2
SELECT tree_level, leaf_pages, internal_pages, avg_leaf_density
FROM pgstatindex('ix_trips_customer');
```

```text
 tree_level | leaf_pages | internal_pages | avg_leaf_density
------------+------------+----------------+------------------
          2 |       1611 |             10 |             90.3
```

Gốc ở tầng 2 ⇒ cây có **3 tầng**: 1 gốc → 9 nhánh → 1.611 lá (10 page "internal" là gốc + 9 nhánh). Page lá đầy trung bình 90,3 %. Tra một khách là đi đúng một page mỗi tầng — đó là 3 page index ở mục 3. Page gốc trông thế này (thí nghiệm, mục 1):

```text
 itemoffset |  ctid   |          khoa
------------+---------+-------------------------
          1 | (3,0)   |
          2 | (208,1) | 21 59 c5 70 88 5a 4d a9
          3 | (400,1) | 39 83 b2 21 12 5c 41 39
```

Mục 1 không có khoá ("âm vô cùng"): mọi uuid nhỏ hơn `2159c570…` đi xuống page 3 của index. Ở page nhánh, phần đầu của `ctid` là **số page con trong index**, không phải địa chỉ heap. Xuống tới lá mới gặp ctid thật:

```text
 itemoffset |        khoa_dau         |   htid    | so_ctid_gop
------------+-------------------------+-----------+-------------
          2 | 00 00 30 bd 09 f1 4c 73 | (658,41)  |          11
          3 | 00 00 9b 1f 52 28 48 75 | (1326,11) |        NULL
          4 | 00 00 d1 30 2b f1 43 52 | (4812,8)  |           4
```

Khoá tăng dần (uuid bắt đầu `0000…`). Mục 2 là một khách có 11 chuyến: khoá chỉ lưu **một lần** kèm danh sách 11 ctid — **deduplication** (gộp khoá trùng), có từ PostgreSQL 13 và bật mặc định. Mục 3 là khách có một chuyến nên chỉ có một ctid. Nhờ gộp, index một cột này chỉ 12,7 MB, trong khi index hai cột `(customer_id, requested_at DESC)` của nền là 40,6 MB vì cặp khoá hầu như không trùng. (Mục 1 của page lá là **high key** — cận trên của page, nên truy vấn chỉ lấy mục 2–4.)

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Tra khách 0910000500: đi page gốc (1 page, 9 mục), một page nhánh trong 9, một page lá trong 1.611 — tổng 3 page index — lấy 40 ctid, rồi đọc 40 page heap trong 19.441. Tổng 43 page so với 19.441 page khi Seq Scan.">
<defs>
<marker id="pg023-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="20" y="28" class="t-b t-pk">ix_trips_customer · 12,7 MB · 3 tầng</text>
<rect x="20" y="40" width="300" height="40" rx="8" class="f-pk-soft s-pk"/>
<text x="32" y="58" class="t-b">Gốc · 1 page · 9 mục</text>
<text x="32" y="73" class="t-mono t-10">−∞ → p3 │ 2159c570… → p208 │ …</text>
<rect x="20" y="104" width="300" height="40" rx="8" class="f-pk-soft s-pk"/>
<text x="32" y="122" class="t-b">Nhánh · 9 page</text>
<text x="32" y="137" class="t-mono t-10">khoá phân cách → page lá</text>
<rect x="20" y="168" width="300" height="48" rx="8" class="f-surface s-pk" stroke-width="1.6"/>
<text x="32" y="186" class="t-b">Lá · 1.611 page · đầy 90,3 %</text>
<text x="32" y="204" class="t-mono t-10">3b3bf766… → ctid 40 chuyến · ⇄ lá kề</text>
<line x1="170" y1="80" x2="170" y2="102" class="s-pk" stroke-width="1.6" marker-end="url(#pg023-m)"/>
<line x1="170" y1="144" x2="170" y2="166" class="s-pk" stroke-width="1.6" marker-end="url(#pg023-m)"/>
<text x="340" y="65" class="t-small">1 page</text>
<text x="340" y="129" class="t-small">1 page</text>
<text x="340" y="197" class="t-small">1 page ⇒ 3 page index</text>
<rect x="500" y="40" width="400" height="176" rx="10" class="f-bg s-phys" stroke-width="1.6"/>
<text x="516" y="64" class="t-b t-phys">Heap trips · 19.441 page</text>
<text x="516" y="88" class="t-mono t-10">(658,41) (1326,11) (4812,8) …</text>
<rect x="516" y="104" width="368" height="44" rx="6" class="f-phys-soft s-phys"/>
<text x="528" y="124" class="t-b">40 chuyến nằm ở 40 page khác nhau</text>
<text x="528" y="141" class="t-small">Bitmap Heap Scan: đọc mỗi page một lần</text>
<text x="516" y="176" class="t-small">Seq Scan không có index: đọc cả 19.441 page,</text>
<text x="516" y="194" class="t-small">bỏ 999.960 dòng không khớp.</text>
<path d="M320 192 C 420 192, 420 126, 512 126" class="f-none s-pk" stroke-width="1.6" marker-end="url(#pg023-m)"/>
<rect x="20" y="240" width="880" height="44" rx="8" class="f-warn-soft s-warn"/>
<text x="460" y="267" text-anchor="middle" class="t-b t-warn">3 page index + 40 page heap = 43 page   ·   không index: 19.441 page</text>
</svg>
</div>
<figcaption><b>Hình 1 · Đường tìm 40 chuyến của khách <code>0910000500</code>.</b> Mỗi tầng đọc đúng một page; số mục gốc, số page nhánh/lá, độ đầy và số page đọc lấy từ <code>23-btree.sql</code> và thí nghiệm <code>pg023-btree-bai-tap.sql</code>.</figcaption>
</figure>

Bảng 1 triệu dòng mà cây chỉ cao 3 tầng vì mỗi page nhánh trỏ được tới hàng trăm page con (ở đây trung bình ≈ 180 lá mỗi nhánh). Cây tăng tầng chậm theo số dòng; số page heap phải đọc mới là phần tăng theo số dòng **khớp**.

## 5. Index Scan và Bitmap Heap Scan

Có hai cách dùng ctid lấy từ cây:

| Nút | Cách đọc heap | Khi planner hay chọn |
|---|---|---|
| **Index Scan** | Theo thứ tự khoá trong index: lấy một ctid, đọc page heap, lấy ctid tiếp | Ít dòng, hoặc dòng khớp nằm liền nhau trong heap, hoặc cần kết quả theo thứ tự khoá |
| **Bitmap Index Scan → Bitmap Heap Scan** | Gom hết ctid thành một bitmap theo page, rồi đọc heap theo thứ tự page, mỗi page một lần | Khá nhiều dòng rải rác: tránh nhảy qua lại và đọc trùng page |

Tra một mã chuyến qua `uq_trips_code` (khoá duy nhất) thì planner chọn Index Scan: cây `uq_trips_code` cũng 3 tầng (`bt_metap` level 2), nên chạy lại lần hai đọc đúng **4 page** — gốc, nhánh, lá, một page heap (thí nghiệm, mục 6; lần đầu trong một phiên mới con số lớn hơn, 7 page, rồi không lặp lại). Cùng phép đo đó trên bộ 20.000 chuyến của PG-022 là 3 page vì cây chỉ 2 tầng.

## 6. Có index mà planner vẫn Seq Scan

Index rẻ khi điều kiện chọn **ít** dòng. Lab tạo index trên cước `fare` — giá trị phân bố ngẫu nhiên so với vị trí trong heap — rồi nới dần khoảng giá:

| Khoảng cước | Dòng khớp | % bảng | Kế hoạch planner chọn |
|---|---:|---:|---|
| 150.000 | 3.541 | 0,35 | Bitmap Heap Scan |
| 140.000–150.000 | 39.270 | 3,93 | Bitmap Heap Scan |
| 120.000–150.000 | 111.141 | 11,11 | Bitmap Heap Scan |
| 90.000–150.000 | 266.345 | 26,63 | Bitmap Heap Scan |
| 60.000–150.000 | 555.165 | 55,52 | **Seq Scan** |

Ở mức 11 %, Bitmap đã phải chạm **19.403 / 19.441** page heap: chuyến khớp rải khắp bảng nên gần như page nào cũng có vài chuyến. Lúc đó index chỉ còn giúp bỏ qua 38 page, lại tốn thêm page index. Ở 55,5 %, ép `enable_seqscan = off` thì Bitmap chạm đủ 19.441 page heap **cộng** 475 page index:

```text
 Bitmap Heap Scan on trips (actual rows=555165.00 loops=1)
   Heap Blocks: exact=19441
   Buffers: shared hit=9776 read=10140 written=6
   ->  Bitmap Index Scan on ix_trips_fare (actual rows=555165.00 loops=1)
         Buffers: shared hit=2 read=473
```

Thời gian đo (`DO-DAC.md`, G2, trung vị 5 lần, cache nóng):

| Phép đo | Planner chọn | Kế hoạch bị ép |
|---|---:|---:|
| `fare` 11 % | Bitmap **67,1 ms** | Seq Scan 87,6 ms |
| `fare` 55 % | Seq Scan **95,5 ms** | Bitmap 75,5 ms |
| `requested_at` 60 ngày (~36 %) | Index Scan **37,4 ms** | Seq Scan 97,8 ms |

Dòng giữa đáng chú ý: ở 55 %, Bitmap bị ép lại **nhanh hơn** kế hoạch planner chọn trên máy này. Planner không đo thời gian; nó so **chi phí ước tính** theo mô hình mặc định coi đọc page ngẫu nhiên đắt gấp 4 lần đọc tuần tự (`random_page_cost = 4`), trong khi máy lab dùng NVMe và cache hệ điều hành nóng. Đây là một quan sát trên một máy, không phải lời khuyên chỉnh tham số — PG-027 và PG-030 quay lại chuyện ước lượng.

Dòng cuối cho thấy điều quan trọng hơn: **cột thẳng hàng với heap** đổi hẳn kết quả. Chuyến được chèn theo giờ đặt, nên các chuyến của một khoảng thời gian nằm liền một dải page (hệ số `correlation` của `requested_at` = 1,000). Với index trên `requested_at`, planner chọn Index Scan cho 1, 7, 30, 60 và cả **120 ngày (~72 % bảng)**; tới 180 ngày mới chuyển Seq Scan. Cả tháng 10 — 184.396 chuyến — đọc bằng Index Scan mà chỉ chạm **4.212** page (`hit=3707 read=505`).

Nên nhớ hai điều: **không có ngưỡng phần trăm cố định** kiểu "trên 10–20 % thì Seq Scan" (ở đây cột ngẫu nhiên đổi kế hoạch trong khoảng từ 26,6 % đến 55,5 %, cột thẳng hàng vượt 72 %), và cái quyết định là **số page heap phải chạm**, không phải số dòng. Vì sao heap thẳng hàng với thời gian mà không thẳng hàng với khoá chính là chủ đề của PG-024.

## 7. Cây cũng có hình dạng: chèn tăng dần hay ngẫu nhiên

Cả `pk_trips` (khoá uuid ngẫu nhiên) và `uq_trips_code` (mã tăng dần) đều có 1.000.000 mục, nhưng:

```text
     index     | leaf_pages | avg_leaf_density
---------------+------------+------------------
 pk_trips      |       4848 |            71.19
 uq_trips_code |       2733 |            90.01
```

Mã tăng dần luôn chèn ở mép phải cây: page lá được lấp tới 90 % (fillfactor mặc định của B-tree) rồi mở page mới. Uuid ngẫu nhiên chèn vào giữa cây: page lá đầy thì **tách đôi**, mỗi nửa còn trống, nên cây xốp hơn (71 %). `REINDEX INDEX pk_trips` dựng lại cây từ dữ liệu đã sắp ⇒ **3.832** lá, đầy 89,98 %. Cây xốp nghĩa là nhiều page hơn cho cùng số khoá — tốn chỗ và tốn buffer hơn; dùng lâu ngày cây lại xốp dần theo cách chèn.

## 8. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Có index trên `customer_id` mà vẫn Seq Scan, `Rows Removed by Filter: 999960` | Điều kiện bọc cột: `customer_id::text = '…'` — index lưu giá trị `uuid`, không lưu `text` | Để cột trần, ép kiểu phía hằng số: `customer_id = '…'::uuid` ⇒ 43 page (thí nghiệm, mục 2) |
| `phone LIKE '%0500'` không dùng index | B-tree chỉ dùng được mẫu có tiền tố cố định | `LIKE '091000050%'` ⇒ `Index Only Scan using uq_customers_phone` với `phone >= '091000050' AND phone < '091000051'` (database lab collation `C`; collation khác cần operator class riêng). Tìm giữa chuỗi: GIN trigram, PG-028 |
| Index trên `status` "không chạy" với `'completed'` | 880.042 / 1 triệu dòng khớp — Seq Scan rẻ hơn | Index trên cột ít giá trị chỉ có ích cho giá trị hiếm; xem bài tập 1 và partial index ở PG-025 |
| Kết luận "index nhanh hơn" từ một lần chạy | Lần đầu đọc từ ngoài `shared_buffers`, lần sau đã nóng | Ghi `Buffers`, đo lặp, ghi rõ cache nóng/lạnh như `DO-DAC.md` |
| Tưởng `shared read` là đọc đĩa | `read` chỉ là "không có trong `shared_buffers`" | Cache hệ điều hành vẫn có thể phục vụ; đọc đĩa thật cần đo I/O riêng |

## 9. Đánh đổi

- **Đọc chọn lọc nhanh hơn, ghi chậm hơn:** mỗi index là một cây phải cập nhật khi chèn/sửa (trừ trường hợp HOT ở PG-024). `trips` của hồ sơ có 5 index và bị sửa ≈ 560 lần/giây lúc cao điểm — thêm index phải có truy vấn thật cần nó.
- **Một index cho một kiểu truy cập:** index trên `customer_id` không giúp lọc theo `fare`; index hai cột, partial và expression là PG-025.
- **Dung lượng trên đĩa và trong bộ nhớ:** 12,7 MB nhờ gộp khoá trùng; khoá ít trùng như cặp `(customer_id, requested_at)` thì 40,6 MB. Page index cũng tranh chỗ trong `shared_buffers` với page heap.

## 10. Bài tập

**Bài 1.** Tạo `CREATE INDEX ix_trips_status ON trips (status)`. Dự đoán kế hoạch cho `status = 'completed'` và `status = 'searching'`. Vì sao cùng một index mà hai kế hoạch lại khác nhau?

<details><summary><b>Lời giải</b></summary>
<p><code>completed</code> có 880.042 dòng (88 %) ⇒ <code>Seq Scan on trips</code>. <code>searching</code> chỉ có 50 chuyến (đang tìm tài xế lúc 01:00 ngày 31/10) ⇒ <code>Index Scan using ix_trips_status</code>, 50 dòng, <code>Buffers: shared hit=8 read=2</code> — chỉ 10 page. Planner dùng thống kê tần suất của từng giá trị (PG-027) nên quyết định theo giá trị, không theo cột. Lời giải: <code>p03/thu-nghiem/pg023-btree-bai-tap.sql</code>, mục 4.</p>
</details>

**Bài 2.** Khách doanh nghiệp B1 (`0919990001`) có 51.151 chuyến — 5,1 % bảng. Với `ix_trips_customer`, đoán xem truy vấn chuyến của B1 chạm bao nhiêu page heap.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT code, fare FROM trips WHERE customer_id = :'b1';</code></pre>
<p>Bitmap Heap Scan, <code>Heap Blocks: exact=18139</code> — 93 % số page của bảng — trong khi index chỉ tốn 47 page. Chuyến của B1 chiếm 5 % số dòng nhưng rải đều theo thời gian, nên gần như page nào cũng có. Màn hình chỉ cần 20 chuyến gần nhất thì đừng lấy cả 51.151 dòng: index <code>(customer_id, requested_at DESC)</code> + <code>LIMIT</code> đọc 10 page (PG-021, PG-025).</p>
</details>

**Bài 3.** Bảng 1 triệu chuyến. Tra `WHERE code = 600000` qua `uq_trips_code` cần đọc tối thiểu mấy page? Kiểm tra bằng `bt_metap` và `EXPLAIN (ANALYZE)`.

<details><summary><b>Lời giải</b></summary>
<p><code>bt_metap('uq_trips_code').level</code> = 2 ⇒ 3 tầng ⇒ 3 page index + 1 page heap = <b>4</b>. Chạy lần hai trong cùng phiên: <code>Index Scan using uq_trips_code … Buffers: shared hit=4</code>. Lần đầu của phiên mới ra 7 page (có page chỉ đọc một lần mỗi phiên) — lý do nên đọc <code>Buffers</code> của lần chạy lặp lại. Mục 6 của tệp thí nghiệm.</p>
</details>

## 11. Đọc tiếp

- [PostgreSQL 18 · Index Types — B-Tree](https://www.postgresql.org/docs/18/indexes-types.html): toán tử B-tree hỗ trợ, `LIKE` có tiền tố, `IS NULL`, `ORDER BY`.
- [PostgreSQL 18 · B-Tree Indexes — Implementation](https://www.postgresql.org/docs/18/btree.html#BTREE-IMPLEMENTATION): page lá/nhánh, tách page, deduplication.
- [PostgreSQL 18 · pageinspect](https://www.postgresql.org/docs/18/pageinspect.html) (`bt_metap`, `bt_page_items`, `bt_multi_page_stats`) và [pgstattuple](https://www.postgresql.org/docs/18/pgstattuple.html) (`pgstatindex`).
- [Release notes PostgreSQL 13](https://www.postgresql.org/docs/release/13.0/) (deduplication) · [PostgreSQL 18](https://www.postgresql.org/docs/release/18.0/) (`EXPLAIN ANALYZE` tự kèm `BUFFERS`, số lần tra index mỗi nút).
- Lab: `p03/23-btree.sql`, kết quả đầy đủ ở `p03/OUTPUT.txt`, thời gian ở `p03/DO-DAC.md`, thí nghiệm `p03/thu-nghiem/pg023-btree-bai-tap.sql`.

Bài tiếp theo: [PG-024 · Clustered index thật và clustered index nhìn thấy trong PostgreSQL](/articles/heap-ctid-va-primary-key).
