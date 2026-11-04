---
slug: heap-ctid-va-primary-key
title: "Clustered index thật và clustered index nhìn thấy trong PostgreSQL"
summary: "Primary key không quyết định dòng nằm ở đâu. Mổ xẻ heap, ctid và đường đi của một truy vấn trên bảng trips của Tới Nơi — kèm số đo thật về UPDATE, HOT và dòng bị dời page."
category: database
level: intermediate
tags: [postgresql, index, heap, ctid, mvcc]
language: vi
---

Mở lược đồ bảng `trips` của [Tới Nơi](/articles/du-an-mau-toi-noi), thứ ta thấy đầu tiên là `id uuid PRIMARY KEY`. Rất dễ nghĩ rằng primary key là thứ quyết định một chuyến nằm ở đâu trên đĩa — giống "clustered index" ở vài hệ quản trị khác. Với PostgreSQL thì không. Bảng có hai lớp tách rời:

- **Clustered index thật — lớp vật lý:** tệp **heap** chia thành các **page** 8 KB (đơn vị lưu trữ và đọc ghi của PostgreSQL), mỗi dòng có một địa chỉ vật lý gọi là **`ctid`**. Đây là nơi duy nhất cất dữ liệu dòng.
- **Clustered index nhìn thấy — primary key:** `trips_pkey` chỉ là một cây B-tree riêng, ánh xạ `id → ctid`. Về cấu trúc, nó ngang hàng với mọi index khác của bảng.

Bài này đi qua hai lớp đó bằng chính bảng `trips`. Mọi kết quả truy vấn dưới đây là **kết quả chạy thật** trên PostgreSQL 18.6 với bộ dữ liệu mẫu thu nhỏ còn 20.000 chuyến (giá trị `id` và `ctid` ở máy bạn sẽ khác, nhưng hình dạng thì giống).

## Hai lớp của bảng trips

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 430" role="img" aria-label="Lớp nhìn thấy gồm trips_pkey, uq_trips_code và các index khác; lớp vật lý là các page heap. Ba mục liền nhau của uq_trips_code trỏ vào cùng page 0, còn ba mục liền nhau của trips_pkey trỏ vào ba page 148, 192 và 336.">
<defs>
<marker id="h1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="h1-ph" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="16" width="880" height="164" rx="12" class="f-bg s-pk dash" stroke-width="1.3"/>
<text x="36" y="40" class="t-b t-pk">LỚP NHÌN THẤY — schema, index, primary key</text>
<rect x="40" y="56" width="260" height="110" rx="8" class="f-surface s-line"/>
<text x="52" y="76" class="t-mono t-b">uq_trips_code</text>
<text x="52" y="92" class="t-small">B-tree (code) · số chuyến tăng dần</text>
<text x="52" y="116" class="t-mono">100000 → (0,1)</text>
<text x="52" y="134" class="t-mono">100001 → (0,2)</text>
<text x="52" y="152" class="t-mono">100002 → (0,3)</text>
<rect x="320" y="56" width="300" height="110" rx="8" class="f-pk-soft s-pk" stroke-width="1.8"/>
<text x="332" y="76" class="t-mono t-b t-pk">trips_pkey</text>
<text x="332" y="92" class="t-small">B-tree (id) · primary key · uuid v4</text>
<text x="332" y="116" class="t-mono">000d094a… → (336,33)</text>
<text x="332" y="134" class="t-mono">000e2402… → (148,17)</text>
<text x="332" y="152" class="t-mono">000e6594… → (192,10)</text>
<rect x="640" y="56" width="250" height="110" rx="8" class="f-surface s-line"/>
<text x="652" y="76" class="t-b">3 index còn lại</text>
<text x="652" y="98" class="t-mono t-10">ix_trips_city_requested</text>
<text x="652" y="116" class="t-mono t-10">ix_trips_customer_requested</text>
<text x="652" y="134" class="t-mono t-10">ix_trips_driver_id</text>
<text x="652" y="154" class="t-small">cũng chỉ giữ khoá → ctid</text>
<text x="190" y="214" class="t-b">mọi index chỉ giữ ctid</text>
<rect x="20" y="240" width="880" height="180" rx="12" class="f-bg s-phys" stroke-width="1.6"/>
<rect x="30" y="254" width="200" height="98" rx="8" class="f-surface s-phys"/>
<text x="40" y="272" class="t-small">heap · page 0</text>
<text x="40" y="294" class="t-mono t-10">(0,1) │ 1ff65316 │ 100000</text>
<text x="40" y="314" class="t-mono t-10">(0,2) │ c1d5b5c8 │ 100001</text>
<text x="40" y="334" class="t-mono t-10">(0,3) │ f5404368 │ 100002</text>
<rect x="250" y="254" width="200" height="98" rx="8" class="f-surface s-phys"/>
<text x="260" y="272" class="t-small">heap · page 148</text>
<text x="260" y="294" class="t-mono t-10 t-dead">(148,16) │ … │ 107924</text>
<rect x="254" y="300" width="192" height="19" rx="4" class="f-pk-soft"/>
<text x="260" y="314" class="t-mono t-10 t-b">(148,17) │ 000e2402 │ 107925</text>
<text x="260" y="334" class="t-mono t-10 t-dead">(148,18) │ … │ 107926</text>
<rect x="470" y="254" width="200" height="98" rx="8" class="f-surface s-phys"/>
<text x="480" y="272" class="t-small">heap · page 192</text>
<text x="480" y="294" class="t-mono t-10 t-dead">(192,9) │ … │ 110267</text>
<rect x="474" y="300" width="192" height="19" rx="4" class="f-pk-soft"/>
<text x="480" y="314" class="t-mono t-10 t-b">(192,10) │ 000e6594 │ 110268</text>
<text x="480" y="334" class="t-mono t-10 t-dead">(192,11) │ … │ 110269</text>
<rect x="690" y="254" width="200" height="98" rx="8" class="f-surface s-phys"/>
<text x="700" y="272" class="t-small">heap · page 336</text>
<text x="700" y="294" class="t-mono t-10 t-dead">(336,32) │ … │ 117998</text>
<rect x="694" y="300" width="192" height="19" rx="4" class="f-pk-soft"/>
<text x="700" y="314" class="t-mono t-10 t-b">(336,33) │ 000d094a │ 117999</text>
<text x="700" y="334" class="t-mono t-10 t-dead">(336,34) │ … │ 118000</text>
<text x="36" y="384" class="t-b t-phys">LỚP VẬT LÝ — heap + ctid: nơi dòng thật sự nằm (cột: ctid │ id │ code)</text>
<text x="36" y="404" class="t-small">Dòng nằm theo thứ tự được chèn, mỗi page 8 KB chứa khoảng 53 chuyến. Không cột nào, kể cả id, quyết định thứ tự này.</text>
<line x1="170" y1="168" x2="170" y2="250" class="s-phys" stroke-width="1.8" marker-end="url(#h1-ph)"/>
<path d="M380 168 C 380 220, 438 225, 438 298" class="f-none s-pk" stroke-width="1.8" marker-end="url(#h1-pk)"/>
<path d="M470 168 C 470 222, 658 220, 658 298" class="f-none s-pk" stroke-width="1.8" marker-end="url(#h1-pk)"/>
<path d="M560 168 C 560 218, 878 214, 878 298" class="f-none s-pk" stroke-width="1.8" marker-end="url(#h1-pk)"/>
</svg>
</div>
<figcaption><b>Hình 1 · Hai lớp của <code>trips</code>.</b> Ba mục liền nhau của <code>uq_trips_code</code> trỏ vào cùng page 0, vì chuyến được chèn theo thứ tự số. Ba mục liền nhau của <code>trips_pkey</code> — đã xếp theo <code>id</code> — trỏ vào ba page cách xa nhau: primary key có thứ tự trong cây của nó, heap thì không theo thứ tự đó.</figcaption>
</figure>

Bạn có thể tự kiểm tra hai câu sau trên bộ dữ liệu mẫu. Đọc 5 dòng đầu tiên **theo thứ tự vật lý** (không `ORDER BY`):

```sql
SELECT ctid, left(id::text, 8) AS id, code, status FROM trips LIMIT 5;
```

```text
 ctid  |    id    |  code  |  status
-------+----------+--------+-----------
 (0,1) | 1ff65316 | 100000 | completed
 (0,2) | c1d5b5c8 | 100001 | completed
 (0,3) | f5404368 | 100002 | completed
 (0,4) | ca18763b | 100003 | completed
 (0,5) | 5a035d87 | 100004 | completed
```

Còn đây là 5 dòng đầu **theo thứ tự primary key**:

```sql
SELECT ctid, left(id::text, 8) AS id, code FROM trips ORDER BY id LIMIT 5;
```

```text
   ctid   |    id    |  code
----------+----------+--------
 (336,33) | 000d094a | 117999
 (148,17) | 000e2402 | 107925
 (192,10) | 000e6594 | 110268
 (219,37) | 00127cb3 | 111740
 (173,51) | 0017f22f | 109294
```

`ctid` nhảy lung tung: đọc theo primary key là đọc rải khắp tệp heap.

## Bên trong một page heap: ctid trỏ vào đâu

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 340" role="img" aria-label="Một page heap 8 KB: page header, mảng line pointer lớn dần từ trên xuống, chỗ trống ở giữa, dữ liệu dòng ghi từ cuối page ngược lên; ctid (0,2) nghĩa là page 0, line pointer số 2.">
<defs>
<marker id="h2-ph" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
<marker id="h2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-warn"/></marker>
</defs>
<rect x="40" y="24" width="460" height="270" rx="8" class="f-surface s-phys" stroke-width="1.8"/>
<rect x="41" y="25" width="458" height="27" rx="7" class="f-bg"/>
<text x="52" y="43" class="t-small">page header · 24 byte</text>
<text x="488" y="43" text-anchor="end" class="t-mono t-small">page 0 · 8 KB</text>
<rect x="48" y="60" width="58" height="28" rx="5" class="f-phys-soft s-phys"/><text x="77" y="79" text-anchor="middle" class="t-mono t-b">lp 1</text>
<rect x="112" y="60" width="58" height="28" rx="5" class="f-warn-soft s-warn" stroke-width="1.6"/><text x="141" y="79" text-anchor="middle" class="t-mono t-b t-warn">lp 2</text>
<rect x="176" y="60" width="58" height="28" rx="5" class="f-phys-soft s-phys"/><text x="205" y="79" text-anchor="middle" class="t-mono t-b">lp 3</text>
<text x="244" y="79" class="t-small">→ mảng line pointer dài ra khi thêm dòng</text>
<rect x="48" y="98" width="444" height="98" rx="5" class="f-none s-line dash"/>
<text x="484" y="118" text-anchor="end" class="t-small">chỗ trống</text>
<rect x="48" y="204" width="140" height="78" rx="5" class="f-phys-soft s-phys"/>
<text x="58" y="224" class="t-b">dòng của lp 3</text>
<text x="58" y="244" class="t-mono">id f5404368…</text>
<text x="58" y="262" class="t-mono">code 100002</text>
<rect x="196" y="204" width="144" height="78" rx="5" class="f-warn-soft s-warn" stroke-width="1.6"/>
<text x="206" y="224" class="t-b t-warn">dòng của lp 2</text>
<text x="206" y="244" class="t-mono">id c1d5b5c8…</text>
<text x="206" y="262" class="t-mono">code 100001</text>
<rect x="348" y="204" width="144" height="78" rx="5" class="f-phys-soft s-phys"/>
<text x="358" y="224" class="t-b">dòng của lp 1</text>
<text x="358" y="244" class="t-mono">id 1ff65316…</text>
<text x="358" y="262" class="t-mono">code 100000</text>
<path d="M77 90 C 90 150, 380 140, 420 202" class="f-none s-phys" stroke-width="1.5" marker-end="url(#h2-ph)"/>
<path d="M141 90 C 150 150, 262 150, 268 202" class="f-none s-warn" stroke-width="1.8" marker-end="url(#h2-w)"/>
<path d="M205 90 C 200 150, 125 150, 118 202" class="f-none s-phys" stroke-width="1.5" marker-end="url(#h2-ph)"/>
<text x="270" y="316" text-anchor="middle" class="t-small">← dữ liệu dòng ghi từ cuối page ngược lên; hai phía gặp nhau là page đầy (≈ 53 chuyến)</text>
<text x="540" y="40" class="t-small">địa chỉ của dòng thứ hai</text>
<text x="540" y="76" class="t-big t-warn">ctid = (0,2)</text>
<text x="540" y="102" class="t-mono">0 → page số 0 của tệp heap</text>
<text x="540" y="120" class="t-mono">2 → ô line pointer số 2</text>
<text x="540" y="140" class="t-small">Line pointer giữ vị trí byte của dòng trong page,</text>
<text x="540" y="156" class="t-small">nên dòng dời chỗ TRONG page thì ctid vẫn giữ.</text>
<rect x="540" y="176" width="350" height="118" rx="8" class="f-bg s-line"/>
<text x="552" y="196" class="t-mono">SELECT ctid, left(id::text, 8), code</text>
<text x="552" y="212" class="t-mono">FROM trips</text>
<text x="552" y="228" class="t-mono">WHERE code IN (100000, 100001);</text>
<line x1="552" y1="238" x2="878" y2="238" class="s-line"/>
<text x="552" y="256" class="t-mono t-small"> ctid  │ id       │ code</text>
<text x="552" y="272" class="t-mono"> (0,1) │ 1ff65316 │ 100000</text>
<text x="552" y="288" class="t-mono t-b t-warn"> (0,2) │ c1d5b5c8 │ 100001</text>
</svg>
</div>
<figcaption><b>Hình 2 · Giải phẫu một page heap.</b> <code>ctid</code> là cột hệ thống ẩn có ở mọi bảng. Số sau dấu phẩy là số thứ tự ô trong mảng line pointer — ô đó mới giữ vị trí byte thật của dòng.</figcaption>
</figure>

## Tìm theo primary key vẫn phải xuống lớp vật lý

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 340" role="img" aria-label="Truy vấn theo id: đi cây B-tree trips_pkey từ gốc xuống lá, lấy ctid (148,17), rồi mở page heap 148 và đọc line pointer số 17 để tới dòng dữ liệu.">
<defs>
<marker id="h3-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
<marker id="h3-ph" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-phys"/></marker>
<marker id="h3-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="12" width="880" height="34" rx="8" class="f-bg s-line"/>
<text x="36" y="34" class="t-mono">SELECT * FROM trips WHERE id = '000e2402-…';</text>
<text x="180" y="70" text-anchor="middle" class="t-b t-pk">① B-tree trips_pkey (lớp nhìn thấy)</text>
<rect x="90" y="80" width="180" height="32" rx="6" class="f-pk-soft s-pk"/>
<text x="180" y="100" text-anchor="middle" class="t-mono">gốc · … │ 5c… │ a8… │ …</text>
<line x1="160" y1="112" x2="140" y2="134" class="s-muted" stroke-width="1.3" marker-end="url(#h3-m)"/>
<rect x="50" y="136" width="200" height="32" rx="6" class="f-pk-soft s-pk"/>
<text x="150" y="156" text-anchor="middle" class="t-mono">trung gian · 000… │ 01… │ …</text>
<line x1="150" y1="168" x2="150" y2="190" class="s-muted" stroke-width="1.3" marker-end="url(#h3-m)"/>
<rect x="40" y="192" width="260" height="86" rx="6" class="f-surface s-pk" stroke-width="1.6"/>
<text x="52" y="210" class="t-small">lá · id → ctid</text>
<text x="52" y="230" class="t-mono t-dead">000d094a… → (336,33)</text>
<rect x="46" y="237" width="248" height="19" rx="4" class="f-warn-soft"/>
<text x="52" y="251" class="t-mono t-b t-warn">000e2402… → (148,17)</text>
<text x="52" y="270" class="t-mono t-dead">000e6594… → (192,10)</text>
<text x="710" y="70" text-anchor="middle" class="t-b t-phys">② heap, page 148 (lớp vật lý)</text>
<rect x="520" y="80" width="380" height="198" rx="8" class="f-surface s-phys" stroke-width="1.8"/>
<rect x="528" y="92" width="56" height="26" rx="5" class="f-phys-soft s-phys"/><text x="556" y="110" text-anchor="middle" class="t-mono">lp 16</text>
<rect x="590" y="92" width="56" height="26" rx="5" class="f-warn-soft s-warn" stroke-width="1.6"/><text x="618" y="110" text-anchor="middle" class="t-mono t-b t-warn">lp 17</text>
<rect x="652" y="92" width="56" height="26" rx="5" class="f-phys-soft s-phys"/><text x="680" y="110" text-anchor="middle" class="t-mono">lp 18</text>
<text x="716" y="110" class="t-small">…</text>
<rect x="528" y="126" width="364" height="58" rx="5" class="f-none s-line dash"/>
<text x="884" y="146" text-anchor="end" class="t-small">chỗ trống</text>
<rect x="528" y="192" width="118" height="74" rx="5" class="f-phys-soft s-phys"/>
<text x="538" y="212" class="t-mono t-b">(148,18)</text>
<text x="538" y="230" class="t-mono">…</text>
<text x="538" y="248" class="t-mono">107926</text>
<rect x="652" y="192" width="118" height="74" rx="5" class="f-warn-soft s-warn" stroke-width="1.6"/>
<text x="662" y="212" class="t-mono t-b t-warn">(148,17)</text>
<text x="662" y="230" class="t-mono">000e2402…</text>
<text x="662" y="248" class="t-mono">107925</text>
<rect x="776" y="192" width="116" height="74" rx="5" class="f-phys-soft s-phys"/>
<text x="786" y="212" class="t-mono t-b">(148,16)</text>
<text x="786" y="230" class="t-mono">…</text>
<text x="786" y="248" class="t-mono">107924</text>
<path d="M296 246 C 420 246, 500 60, 612 90" class="f-none s-pk" stroke-width="1.8" marker-end="url(#h3-pk)"/>
<text x="410" y="266" text-anchor="middle" class="t-small">ctid (148,17): mở page 148,</text>
<text x="410" y="281" text-anchor="middle" class="t-small">đọc ô line pointer 17</text>
<path d="M624 120 C 640 150, 700 160, 708 190" class="f-none s-phys" stroke-width="1.8" marker-end="url(#h3-ph)"/>
<rect x="20" y="294" width="880" height="38" rx="8" class="f-bg s-line"/>
<text x="36" y="310" class="t-mono">Index Scan using trips_pkey on trips</text>
<text x="36" y="325" class="t-mono">  Index Cond: (id = '000e2402-…'::uuid)</text>
<text x="884" y="318" text-anchor="end" class="t-small">"Index Scan" = đọc cây rồi đọc heap</text>
</svg>
</div>
<figcaption><b>Hình 3 · Tra một chuyến theo <code>id</code>.</b> Hai cấu trúc, hai nhóm page phải đọc: cây <code>trips_pkey</code> cho ra <code>ctid</code>, rồi page heap cho ra dòng. Khoá trong nút gốc/trung gian là minh hoạ; dòng kế hoạch truy vấn là kết quả <code>EXPLAIN</code> thật.</figcaption>
</figure>

Primary key không có "đường tắt" nào so với index thường. Tra theo số chuyến đi y hệt, chỉ khác cây:

```text
EXPLAIN SELECT * FROM trips WHERE code = 150000;

 Index Scan using uq_trips_code on trips  (cost=0.29..8.30 rows=1 width=125)
   Index Cond: (code = 150000)
```

Hai biến thể bạn sẽ gặp trong `EXPLAIN`:

- **Nhiều dòng khớp** (vd. chuyến của một thành phố trong ngày) ⇒ `Bitmap Heap Scan`: gom hết `ctid` từ index, sắp theo số page, rồi đọc mỗi page heap một lần thay vì nhảy qua lại.
- **Tra thẳng theo địa chỉ vật lý** (`WHERE ctid = '(0,1)'`) ⇒ `Tid Scan`, bỏ qua mọi index. Chỉ dùng để chẩn đoán — lý do ở phần sau.

```text
EXPLAIN SELECT * FROM trips WHERE city_id = 1 AND requested_at >= now() - interval '1 day';

 Bitmap Heap Scan on trips  (cost=5.43..247.45 rows=111 width=125)
   Recheck Cond: ((city_id = 1) AND (requested_at >= (now() - '1 day'::interval)))
   ->  Bitmap Index Scan on ix_trips_city_requested  (cost=0.00..5.40 rows=111 width=0)
         Index Cond: ((city_id = 1) AND (requested_at >= (now() - '1 day'::interval)))
```

## Heap đang thẳng hàng với cột nào?

PostgreSQL đo sẵn độ "thẳng hàng" giữa thứ tự vật lý và thứ tự từng cột — hệ số `correlation` sau `ANALYZE`:

```sql
SELECT attname, round(correlation::numeric, 3) AS correlation
FROM pg_stats
WHERE tablename = 'trips'
ORDER BY abs(correlation) DESC;
```

```text
   attname    | correlation
--------------+-------------
 requested_at |       1.000
 code         |       1.000
 city_id      |       0.167
 fare         |       0.014
 driver_id    |       0.007
 id           |      -0.005
 customer_id  |       0.000
```

Heap thẳng hàng tuyệt đối với **thời gian đặt** và **số chuyến** — vì đó là thứ tự chuyến được chèn — và hoàn toàn không liên quan tới `id`. Nói cách khác, ở Tới Nơi "clustered index thật" đang tự nhiên đi theo `requested_at`, còn primary key thì ngẫu nhiên. Đó là tin tốt cho báo cáo theo khoảng thời gian (đọc gần như tuần tự) và là lý do index BRIN trên `requested_at` sẽ rất nhỏ — chủ đề của một bài sau.

<div class="tkh-callout">
<span class="tkh-callout-title">uuid v4 hay uuid v7?</span>
<p><code>gen_random_uuid()</code> sinh uuid v4 ngẫu nhiên nên <code>correlation</code> của <code>id</code> ≈ 0. Heap không bị ảnh hưởng (heap không xếp theo khoá), nhưng mỗi chuyến mới chèn vào một chỗ ngẫu nhiên trong cây <code>trips_pkey</code>. PostgreSQL 18 có sẵn <code>uuidv7()</code> — uuid tăng theo thời gian — nếu bạn muốn primary key cũng "thẳng hàng" như <code>code</code>.</p>
</div>

## id không đổi, ctid thì đổi

PostgreSQL không ghi đè dòng: mỗi `UPDATE` tạo một **phiên bản mới** của dòng (MVCC), và phiên bản mới có địa chỉ mới. Với 560 UPDATE/giây vào `trips` lúc cao điểm, đây là chi tiết đáng giá nhất của bài. Ba câu UPDATE dưới đây chạy trong một giao dịch, đếm bằng `pg_stat_xact_user_tables`:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 330" role="img" aria-label="Ba câu UPDATE thật: nhận tài xế đổi ctid từ (373,14) sang (373,34), không HOT vì driver_id có index; sửa giá ngay sau đó đổi ctid sang (373,35) và là HOT; sửa giá một chuyến cũ ở page 186 đã đầy làm dòng dời sang (373,36), không HOT.">
<defs>
<marker id="h4-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-muted"/></marker>
</defs>
<rect x="20" y="16" width="280" height="276" rx="12" class="f-surface s-line"/>
<text x="36" y="42" class="t-b">① Tài xế 8812 nhận chuyến 119980</text>
<text x="36" y="60" class="t-mono t-10">SET driver_id = …, status = 'accepted'</text>
<rect x="40" y="80" width="240" height="40" rx="8" class="f-bg s-line"/>
<text x="160" y="105" text-anchor="middle" class="t-mono t-b">(373,14) · searching</text>
<line x1="160" y1="122" x2="160" y2="148" class="s-muted" stroke-width="1.5" marker-end="url(#h4-m)"/>
<rect x="40" y="150" width="240" height="40" rx="8" class="f-warn-soft s-warn"/>
<text x="160" y="175" text-anchor="middle" class="t-mono t-b t-warn">(373,34) · accepted</text>
<text x="36" y="214" class="t-small">Cùng page 373 (page cuối còn chỗ),</text>
<text x="36" y="230" class="t-small">nhưng driver_id có index.</text>
<rect x="36" y="246" width="248" height="34" rx="8" class="f-warn-soft s-warn"/>
<text x="160" y="268" text-anchor="middle" class="t-b t-warn">Không HOT · cả 5 index thêm mục</text>
<rect x="320" y="16" width="280" height="276" rx="12" class="f-surface s-line"/>
<text x="336" y="42" class="t-b">② Sửa giá ngay sau đó</text>
<text x="336" y="60" class="t-mono t-10">SET fare = fare + 1000</text>
<rect x="340" y="80" width="240" height="40" rx="8" class="f-bg s-line"/>
<text x="460" y="105" text-anchor="middle" class="t-mono t-b">(373,34) · 137.000 đ</text>
<line x1="460" y1="122" x2="460" y2="148" class="s-muted" stroke-width="1.5" marker-end="url(#h4-m)"/>
<rect x="340" y="150" width="240" height="40" rx="8" class="f-phys-soft s-phys"/>
<text x="460" y="175" text-anchor="middle" class="t-mono t-b t-phys">(373,35) · 138.000 đ</text>
<text x="336" y="214" class="t-small">fare không thuộc index nào,</text>
<text x="336" y="230" class="t-small">page 373 còn chỗ.</text>
<rect x="336" y="246" width="248" height="34" rx="8" class="f-phys-soft s-phys"/>
<text x="460" y="268" text-anchor="middle" class="t-b t-phys">HOT · không index nào phải ghi</text>
<rect x="620" y="16" width="280" height="276" rx="12" class="f-surface s-line"/>
<text x="636" y="42" class="t-b">③ Sửa giá chuyến cũ 110000</text>
<text x="636" y="60" class="t-mono t-10">SET fare = fare + 1000</text>
<rect x="640" y="80" width="240" height="40" rx="8" class="f-bg s-line"/>
<text x="760" y="105" text-anchor="middle" class="t-mono t-b">(186,44) · 116.000 đ</text>
<line x1="760" y1="122" x2="760" y2="148" class="s-muted" stroke-width="1.5" marker-end="url(#h4-m)"/>
<rect x="640" y="150" width="240" height="40" rx="8" class="f-warn-soft s-warn"/>
<text x="760" y="175" text-anchor="middle" class="t-mono t-b t-warn">(373,36) · 117.000 đ</text>
<text x="636" y="214" class="t-small">Page 186 đã đầy ⇒ phiên bản mới</text>
<text x="636" y="230" class="t-small">dời sang page cuối, dù fare không có index.</text>
<rect x="636" y="246" width="248" height="34" rx="8" class="f-warn-soft s-warn"/>
<text x="760" y="268" text-anchor="middle" class="t-b t-warn">Không HOT · cả 5 index thêm mục</text>
<text x="460" y="318" text-anchor="middle" class="t-mono t-small">pg_stat_xact_user_tables sau 3 câu: n_tup_upd = 3, n_tup_hot_upd = 1</text>
</svg>
</div>
<figcaption><b>Hình 4 · Ba câu UPDATE thật trên <code>trips</code>.</b> Ở lớp nhìn thấy, mỗi chuyến vẫn là chính nó (<code>id</code> không đổi). Ở lớp vật lý, cả ba đều sinh phiên bản mới ở địa chỉ mới; phiên bản cũ nằm lại chờ VACUUM dọn. Vì index chỉ giữ <code>ctid</code>, chỉ có trường hợp ② tránh được việc ghi thêm vào cả 5 index.</figcaption>
</figure>

**HOT update** (Heap-Only Tuple) là lối thoát duy nhất: nếu (1) không cột nào có index bị đổi **và** (2) page hiện tại còn chỗ cho phiên bản mới, PostgreSQL đặt phiên bản mới cùng page, nối nó với phiên bản cũ, và không index nào phải ghi thêm. Trường hợp ③ cho thấy điều kiện (2) quan trọng không kém điều kiện (1): bảng chèn đầy page (mặc định `fillfactor = 100`) thì một UPDATE "vô hại" cũng làm dòng dời page.

Tự xem tỉ lệ HOT trên bảng thật:

```sql
SELECT n_tup_upd, n_tup_hot_upd,
       round(100.0 * n_tup_hot_upd / nullif(n_tup_upd, 0), 1) AS hot_pct
FROM pg_stat_user_tables
WHERE relname = 'trips';
```

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Những lúc mọi ctid cùng đổi</span>
<p><code>VACUUM FULL</code> và <code>CLUSTER</code> chép lại cả bảng sang tệp mới: mọi dòng nhận <code>ctid</code> mới, mọi index được dựng lại, và bảng bị khoá <code>ACCESS EXCLUSIVE</code> suốt thời gian chạy — không ai đọc được, kể cả <code>SELECT</code>. Với Tới Nơi lúc cao điểm, đó là vài phút không ai đặt được chuyến.</p>
</div>

## Có ép được heap đi theo primary key không?

Chỉ một lần mỗi lần chạy: `CLUSTER trips USING trips_pkey` chép lại heap theo thứ tự `id`. Sau đó INSERT/UPDATE lại ghi vào chỗ trống bất kỳ và thứ tự vỡ dần. PostgreSQL chỉ ghi nhớ index đã dùng (cờ `pg_index.indisclustered`) để lần sau gõ `CLUSTER trips` không cần `USING` — cờ này không bắt ai giữ thứ tự. Đó là "clustered index nhìn thấy" theo nghĩa thứ hai: `\d trips` có in chữ `CLUSTER` cạnh tên index, nhưng không có gì được duy trì.

Với `trips` việc này còn phản tác dụng: `id` là uuid ngẫu nhiên, xếp heap theo nó sẽ phá mất sự thẳng hàng tự nhiên với `requested_at` mà báo cáo đang hưởng.

## Bảng đối chiếu hai lớp

| | Clustered index thật — heap + ctid | Clustered index nhìn thấy — primary key |
|---|---|---|
| Là gì | Tệp heap chia page 8 KB; địa chỉ dòng `ctid = (page, line pointer)` | B-tree riêng `trips_pkey`: `id → ctid`, kèm unique + not null |
| Chứa dữ liệu dòng | Có — nơi duy nhất | Không — chỉ khoá và ctid |
| Quyết định thứ tự trên đĩa | Có: dòng nằm theo chỗ trống lúc ghi (ở Tới Nơi ≈ theo `requested_at`) | Không: chỉ có thứ tự trong cây của chính nó |
| Ổn định | Không: đổi sau mỗi UPDATE, `VACUUM FULL`, `CLUSTER` | Có: `id` không bao giờ đổi |
| Ai dùng | Bộ máy PostgreSQL: mọi index, Bitmap Heap Scan, Tid Scan | Ứng dụng: khoá ngoại, API, URL |
| Code có nên dựa vào | Không bao giờ lưu hay so sánh ctid | Luôn dùng `id` làm danh tính |

## Mang gì về dự án

- **Primary key không phải clustered index.** Truy vấn cần thứ tự phải có `ORDER BY`; đừng dựa vào "dữ liệu vốn theo id".
- **`ctid` chỉ để chẩn đoán.** Không lưu nó, không đưa ra API, không dùng làm khoá phân trang.
- **Mỗi index là thêm một cây trỏ xuống heap.** Trên bộ dữ liệu mẫu, 5 index của `trips` đã chiếm 3,4 MB — nhiều hơn chính bảng (3,0 MB) — và mỗi UPDATE không-HOT ghi vào cả 5. Trước khi thêm index thứ 6 vào bảng bị sửa 560 lần/giây, hãy hỏi nó có đáng không.
- **Cột hay bị sửa thì cân nhắc kỹ trước khi đánh index**, và để ý `fillfactor` cho bảng hay UPDATE — bài về MVCC và VACUUM sẽ đo cụ thể.
