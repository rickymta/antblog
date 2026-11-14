---
slug: postgresql-index-only-scan-visibility-map
title: "Index-only scan, INCLUDE và visibility map"
summary: "Màn lịch sử chuyến của khách doanh nghiệp Tới Nơi đọc 50 page buổi sáng, chiều sau đợt sửa giá đọc 18.540 page dù kế hoạch vẫn là Index Only Scan. Bài này quan sát Heap Fetches, visibility map, VACUUM và cái giá của INCLUDE."
category: postgresql
level: intermediate
tags: [postgresql, pg-026, index-only-scan, visibility-map, vacuum, covering-index, include, toi-noi]
language: vi
---

Khách doanh nghiệp B1 của Tới Nơi — một công ty gửi bưu kiện ở TP. Hồ Chí Minh — có 9.365 chuyến trong tháng 10. Màn «lịch sử chuyến tháng này» của họ chỉ lấy giờ đặt, chạy bằng `Index Only Scan` và đọc **50 page**. Chiều hôm đó, kế toán chạy một lệnh điều chỉnh giá cho toàn bộ chuyến tháng 10 của B1. Ngay sau đó, cùng câu, cùng kế hoạch `Index Only Scan`, đọc **18.540 page**.

Không ai đổi index, không ai đổi câu SQL. Thứ đã đổi nằm ngoài index: một bản đồ nhỏ cho biết page nào của bảng «sạch». Bài này giải thích vì sao index-only scan vẫn phải ghé bảng, VACUUM liên quan gì, và khi nào nên nhét thêm cột vào index bằng `INCLUDE`.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-026 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-index-nhieu-cot-partial-expression">PG-025 · Index nhiều cột, partial và expression</a>, <a href="/articles/heap-ctid-va-primary-key">PG-024 · Heap, ctid và HOT</a>. Lab: <code>examples/toi-noi-postgresql/p03</code> (tệp <code>26-index-only-scan.sql</code>; thí nghiệm <code>thu-nghiem/pg026-snapshot-cu-chan-vacuum.sql</code> và <code>thu-nghiem/pg026-heap-fetch-hot-partial-include.sql</code>). Sau bài này, bạn quan sát được heap fetch và biết điều kiện để covering index thật sự hiệu quả.</p>
</div>

## 1. Dữ liệu và cách chạy

PostgreSQL **18.1 thật**, bảng `trips` 1.000.000 chuyến = 19.441 page heap, vừa `VACUUM (ANALYZE)` lúc nạp nền; cluster lab **tắt autovacuum** để mọi bước tái lập được. Index dùng trong bài: `ix_trips_customer_requested (customer_id, requested_at DESC)`.

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p03 --only 26
node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg026-heap-fetch-hot-partial-include.sql
```

Số page, `Heap Fetches`, kích thước là tất định. Thời gian lấy từ `p03/DO-DAC.md`: một máy i7-10700, NVMe, cache nóng, trung vị 7 lần, nhiễu ~±20 % — chỉ để so tương đối.

## 2. Index Only Scan cần hai điều kiện

Câu của màn B1 chỉ cần `requested_at`, cột có sẵn trong index:

```sql
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT requested_at FROM trips
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';
```

```text
 Index Only Scan using ix_trips_customer_requested on trips (actual rows=9365.00 loops=1)
   Index Cond: ((customer_id = '239ec4d4-…'::uuid) AND (requested_at >= '2026-10-01 00:00:00+07'::timestamp with time zone))
   Heap Fetches: 0
   Index Searches: 1
   Buffers: shared hit=1 read=49
```

**Index Only Scan** (quét chỉ index) trả kết quả từ mục index mà không đọc heap. Điều kiện thứ nhất hiển nhiên: câu chỉ dùng cột có trong index. Thêm `fare` — cột không có trong index:

```text
 Bitmap Heap Scan on trips (actual rows=9365.00 loops=1)
   Heap Blocks: exact=3335
   Buffers: shared hit=49 read=3335
```

3.384 page thay vì 50; DO-DAC đo **4,46 ms** so với **0,59 ms**. 9.365 chuyến của B1 rải trên 3.335 page heap khác nhau vì heap xếp theo giờ đặt, không theo khách.

Điều kiện thứ hai khó thấy hơn. Index B-tree **không lưu thông tin MVCC**: nó không biết mục đang trỏ tới phiên bản dòng còn sống, đã bị xoá, hay chưa commit. Thông tin đó (xmin/xmax) chỉ nằm trong tuple ở heap — PG-024 đã thấy một UPDATE để lại phiên bản cũ ngay trong page. Vậy làm sao Index Only Scan dám bỏ qua heap? Nhờ visibility map.

## 3. Visibility map: 2 bit cho mỗi page

**Visibility map** (VM — bản đồ hiển thị) là một «fork» riêng của bảng, mỗi page heap có hai bit:

- **all-visible**: mọi tuple trong page đều thấy được với mọi giao dịch hiện tại và tương lai — không còn phiên bản chết, không còn phiên bản chưa ai thấy.
- **all-frozen**: mọi tuple đã được «đóng băng» (Phần 5 nói về đóng băng).

Index Only Scan với mỗi mục index tìm được sẽ tra bit all-visible của page heap tương ứng. Bit bật ⇒ trả dữ liệu từ index luôn. Bit tắt ⇒ phải đọc tuple trong heap để kiểm tra phiên bản — mỗi lần như vậy cộng vào **`Heap Fetches`**. VM rất nhỏ (thí nghiệm, mục A):

```text
 heap_bytes | vm_bytes
------------+----------
  159260672 |     8192
```

159 MB heap, một page VM 8 kB. Extension `pg_visibility` đọc được bản đồ:

```sql
SELECT all_visible, (SELECT relpages FROM pg_class WHERE oid = 'trips'::regclass) AS so_page
FROM pg_visibility_map_summary('trips');
```

```text
 all_visible | so_page
-------------+---------
       19441 |   19441
```

Bảng vừa VACUUM: mọi page đều all-visible. Tài liệu nêu một quy tắc quan trọng: bit VM chỉ do **VACUUM bật**, còn **bất kỳ thao tác sửa dữ liệu nào** trên page cũng **xoá** bit này. Bản đồ «bảo thủ»: bit bật thì chắc chắn đúng; bit tắt thì có thể page vẫn sạch, chỉ là chưa ai kiểm lại.

## 4. Một lệnh UPDATE làm Index Only Scan thoái hoá

Kế toán sửa giá 9.365 chuyến tháng 10 của B1:

```sql
UPDATE trips SET fare = fare + 1000
WHERE customer_id = :'kh_b1' AND requested_at >= TIMESTAMPTZ '2026-10-01 00:00+07';
-- UPDATE 9365
```

Số page all-visible còn **16.103**; bảng lớn thêm 174 page (19.441 → 19.615) để chứa phiên bản mới. `pg_visibility('trips', <page chứa chuyến tháng 10 đầu tiên của B1>)` trả `all_visible = f`, `pd_all_visible = f` (bit cùng nghĩa nằm trong header của chính page). Chạy lại câu của màn B1:

```text
 Index Only Scan using ix_trips_customer_requested on trips (actual rows=9365.00 loops=1)
   Heap Fetches: 18536
   Index Searches: 1
   Buffers: shared hit=18540
```

Vẫn 9.365 dòng, vẫn Index Only Scan, nhưng **18.536 heap fetch**. Vì sao gần gấp đôi số dòng? Index giờ có **hai** mục cho phần lớn chuyến: mục cũ trỏ phiên bản cũ (đã chết) và mục mới trỏ phiên bản mới. Cả hai loại page đều mất bit all-visible, nên với mỗi mục PostgreSQL phải ghé heap — với mục cũ, chỉ để phát hiện phiên bản đã chết và bỏ qua. Thí nghiệm đếm chính xác:

```text
 n_tup_upd | n_tup_hot_upd
-----------+---------------
      9365 |           194
```

194 lần là **HOT update** (phiên bản mới nằm cùng page, không tạo mục index mới — PG-024). Vậy số mục index cần kiểm = 9.365 mục cũ + 9.171 mục mới = **18.536** — khớp từng đơn vị với `Heap Fetches`.

### Lần chạy thứ hai ít hơn — và vì sao

Chạy lại lần nữa, **chưa** VACUUM (thí nghiệm, mục A):

```text
   Heap Fetches: 12339
   Buffers: shared hit=6483
```

Lần đọc đầu đã gặp các phiên bản cũ chết với mọi giao dịch, và PostgreSQL đánh dấu luôn mục index trỏ tới chúng là đã chết (cờ LP_DEAD trên mục index) để lần sau bỏ qua mà không ghé heap. `bt_page_items` của `pageinspect` đếm được: 0 mục `dead` trước lần đọc đầu, **6.197** sau đó — và 18.536 − 6.197 = 12.339. Bit VM vẫn không đổi; chỉ index được «tỉa» một phần. Hệ quả cho đo đạc: con số **2,71 ms** của DO-DAC cho bước này là trung vị các lần chạy *sau* lần khởi động (cột Buffers ở đó là 6.483, không phải 18.540) — so với 0,59 ms của bảng sạch, chậm ~4,6 lần.

`count(*)` cũng chịu chung số phận. Trên bảng sạch, lab đếm 1 triệu chuyến bằng Index Only Scan trên index nhỏ nhất `ix_trips_driver_id`, **860 page**. Sau lệnh UPDATE:

```text
   ->  Index Only Scan using ix_trips_driver_id on trips (actual rows=1000000.00 loops=1)
         Heap Fetches: 180881
         Buffers: shared hit=171807 read=58
```

3.512 page mất bit, ~51 chuyến/page ⇒ 180.881 lần ghé heap, hơn 170.000 lần chạm buffer — nhiều hơn cả quét tuần tự 19.615 page. Planner ước chi phí Index Only Scan theo tỉ lệ page all-visible lưu ở `pg_class.relallvisible`; số này chỉ do những lệnh như VACUUM, ANALYZE cập nhật, nên lúc này planner vẫn «tưởng» bảng sạch.

## 5. VACUUM đặt lại bản đồ

```sql
VACUUM trips;
```

```text
 all_visible | so_page
-------------+---------
       19615 |   19615
```

Page của B1: `t | t`. Câu của màn B1 trở về `Heap Fetches: 0`, **96 page** (index đã to thêm vì các mục mới), DO-DAC **0,64 ms**. `VACUUM (VERBOSE)` cho thấy nó làm gì (thí nghiệm, trích):

```text
index scan needed: 3309 pages from table (16.87% of total) had 9171 dead item identifiers removed
visibility map: 3512 pages set all-visible, 0 pages set all-frozen (0 were all-visible)
```

VACUUM xoá mục index trỏ tới 9.171 dòng chết (các phiên bản cũ không HOT), giải phóng khe của chúng trong heap, rồi bật lại bit cho 3.512 page. Trên server thật việc này do **autovacuum** làm; lab tắt nó để tái lập. Bảng sửa nhiều mà autovacuum chạy thưa thì Index Only Scan thoái hoá dần thành «Index Scan đội lốt» — cách chỉnh autovacuum thuộc Phần 5.

## 6. Bẫy: VACUUM xong mà Heap Fetches vẫn không về 0

Thí nghiệm `pg026-snapshot-cu-chan-vacuum.sql` (hai phiên) dựng tình huống thường gặp:

| Bước | Phiên A | Phiên B | `Heap Fetches` (1.000 chuyến 500000–500999) |
|---|---|---|---|
| T1 | `BEGIN ISOLATION LEVEL REPEATABLE READ; SELECT …` (chụp snapshot) | | |
| T2 | | `UPDATE … fare = fare + 1000` (1.000 dòng), `VACUUM` | **1.997** — A còn có thể cần phiên bản cũ, VACUUM không dọn được |
| T3 | Vẫn thấy tổng cước cũ | | |
| T4 | `COMMIT` | `VACUUM` | **1.000** |
| T5 | | `VACUUM (INDEX_CLEANUP ON)` | **0** |

T2 là chuyện quen: một giao dịch mở lâu (kể cả «idle in transaction») giữ snapshot cũ, VACUUM không xoá được phiên bản chết, bit VM không bật lại được. Phần 4 sẽ quay lại.

T4 mới là bẫy: giao dịch đã đóng, VACUUM đã chạy, mà vẫn 1.000. `VACUUM (VERBOSE)` ở tình huống tương tự (thí nghiệm, mục B) nói thẳng:

```text
index scan bypassed: 22 pages from table (0.11% of total) have 997 dead item identifiers
```

Từ bản 14, `INDEX_CLEANUP` mặc định là `AUTO`: khi số dòng chết quá ít, VACUUM **bỏ qua bước dọn index** vì chi phí quét mọi index lớn hơn lợi ích. Ngưỡng trong mã nguồn bản 18 (`vacuumlazy.c`): số page có dòng chết dưới **2 %** số page của bảng, và danh sách dòng chết dưới 32 MB. Khi bỏ qua, phiên bản chết bị dọn khỏi heap nhưng **khe trỏ** (line pointer) của nó phải ở lại trạng thái `LP_DEAD`, vì mục index vẫn trỏ vào đó. Page còn khe `LP_DEAD` thì chưa được đánh all-visible ⇒ mọi mục index trỏ vào 22 page đó vẫn bị ghé heap: 997 mục trỏ khe chết, cộng 3 dòng được HOT update ngay trên các page ấy — đúng 1.000. `VACUUM (INDEX_CLEANUP ON)` ép dọn index, khe được giải phóng, page all-visible, `Heap Fetches: 0`.

Đối lập với mục 5: ở đó dòng chết nằm trên 3.309 page = 16,87 % bảng, vượt ngưỡng, nên VACUUM dọn index bình thường. Đừng kết luận «VACUUM luôn đưa Heap Fetches về 0»; hãy đọc `VERBOSE`.

## 7. INCLUDE: covering index và cái giá của nó

Màn B1 muốn hiện thêm cước và trạng thái. Thay vì Bitmap Heap Scan 3.384 page, có thể làm index **phủ** (covering index) bằng mệnh đề `INCLUDE`:

```sql
CREATE INDEX ix_trips_khach_bao_gom ON trips (customer_id, requested_at DESC) INCLUDE (fare, status);
```

Cột trong `INCLUDE` là cột **không phải khoá** (non-key): chỉ «đi kèm» ở mục lá, không dùng để tìm hay sắp, không tính vào tính duy nhất.

```text
 Index Only Scan using ix_trips_khach_bao_gom on trips (actual rows=9365.00 loops=1)
   Heap Fetches: 0
   Buffers: shared hit=1 read=69
```

70 page, DO-DAC **0,75 ms** so với 4,46 ms. Cái giá:

| Mục | Không INCLUDE | INCLUDE (fare, status) |
|---|---:|---:|
| Kích thước index | 41.000.960 B | 58.892.288 B (+44 %) |
| HOT khi sửa `fare` (bảng 10.000 chuyến, fillfactor 90, 500 lần sửa) | 500 / 500 | **0 / 500** |

Dòng thứ hai là cái giá lớn nhất: cột trong `INCLUDE` vẫn là cột «có trong index», nên đổi giá trị của nó thì UPDATE không còn HOT — mỗi lần sửa giá hay đổi trạng thái phải ghi mục mới vào **mọi** index của bảng (PG-025 đã đo WAL của việc đó). Mà `fare` và `status` chính là hai cột bị sửa nhiều nhất trong 560 UPDATE/giây của Tới Nơi. Tài liệu PostgreSQL cũng khuyên: thêm cột payload chỉ có ý nghĩa khi bảng thay đổi đủ chậm để Index Only Scan ít phải ghé heap — với cột bị sửa liên tục, bit VM tắt liên tục và lợi ích tan biến. Thêm hai lưu ý từ tài liệu: B-tree không dùng deduplication cho index có cột non-key, và mục index vượt kích thước tối đa thì lệnh ghi thất bại — đừng `INCLUDE` cột dài.

Partial index cũng phủ được câu hỏi mà không chứa cột của predicate (thí nghiệm, mục C): `SELECT requested_at FROM trips WHERE status IN (…6 trạng thái…)` chạy `Index Only Scan using ix_trips_dang_chay`, 300 dòng, `Heap Fetches: 0` — mọi mục trong index đều thoả predicate nên không cần kiểm lại `status`.

## 8. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách kiểm / sửa |
|---|---|---|
| Index Only Scan nhưng chậm, `Heap Fetches` lớn | Page mất bit all-visible sau ghi | `pg_visibility_map_summary`; VACUUM / chỉnh autovacuum |
| VACUUM xong vẫn `Heap Fetches` = số dòng vừa sửa | Bỏ qua dọn index (ít dòng chết) | `VACUUM (VERBOSE)` tìm «index scan bypassed»; `INDEX_CLEANUP ON` nếu cần |
| VACUUM không giảm được gì | Giao dịch mở lâu giữ snapshot | `pg_stat_activity` tìm `idle in transaction` lâu |
| Lần chạy thứ hai nhanh hơn lần đầu, số đo khó so | Lần đầu đánh dấu mục index chết | Ghi rõ lần đo; đọc `Heap Fetches` cùng thời gian |
| Thêm `INCLUDE` xong, UPDATE chậm và bảng phình | Mất HOT trên cột hay sửa | `n_tup_hot_upd` trước/sau; chỉ INCLUDE cột ít đổi |
| `count(*)` lúc nhanh lúc chậm | Index Only Scan phụ thuộc VM | Xem `Heap Fetches` của nút đếm |

## 9. Đánh đổi

- **Index Only Scan phát huy tác dụng tốt nhất với bảng «tĩnh».** Bảng lịch sử, chuyến đã kết thúc, dữ liệu theo tháng cũ hưởng trọn; phần đang nóng của bảng thì không.
- **INCLUDE đổi chi phí đọc lấy chi phí ghi**: +44 % dung lượng, mất HOT cho cột đi kèm. Chọn cột ít đổi (`requested_at`, `kind`, `city_id`), tránh `status`, `fare`, `updated_at`.
- **VACUUM nhiều hơn** giữ VM sạch nhưng tốn I/O; `INDEX_CLEANUP ON` thường xuyên trên bảng lớn có thể đắt hơn lợi ích — PostgreSQL đặt mặc định `AUTO` là có lý do.

## 10. Bài tập

**Bài 1.** Đội sản phẩm đề xuất `INCLUDE (fare, status)` cho màn lịch sử chuyến của mọi khách để tránh 3.384 page. Đánh giá bằng số của lab và hồ sơ tải Tới Nơi.

<details><summary><b>Lời giải</b></summary>
<p>Lợi: câu B1 tháng 10 từ 3.384 page / 4,46 ms xuống 70 page / 0,75 ms — nhưng chỉ khi page heap còn all-visible. Hại: index 41,0 → 58,9 MB; mọi UPDATE đổi <code>fare</code> hoặc <code>status</code> mất HOT (lab: 500/500 → 0/500), mà đổi trạng thái là phần lớn của ~560 UPDATE/giây. Chuyến đang chạy là đúng những dòng bị sửa, nên page của chúng thường không all-visible — Index Only Scan vẫn ghé heap đúng ở phần khách hay xem nhất. Kết luận hợp lý: không INCLUDE <code>status</code>/<code>fare</code> trên <code>trips</code>; chấp nhận Bitmap Heap Scan, hoặc tách phần lịch sử đã kết thúc (ít đổi) để index phủ phát huy — Phần 6 nói về partition.</p>
</details>

**Bài 2.** So sánh `(customer_id, requested_at DESC, fare, status)` với `(customer_id, requested_at DESC) INCLUDE (fare, status)`: kích thước nào lớn hơn? Khác nhau ở chỗ nào?

<details><summary><b>Lời giải</b></summary>
<p>Thí nghiệm (mục D): cả hai đúng <b>58.892.288</b> byte. Deduplication không dùng cho index có INCLUDE, nhưng ở đây <code>(customer_id, requested_at)</code> gần như không trùng nên nhiều khả năng nó cũng chẳng gộp được gì ở index 4 cột khoá. Khác biệt chức năng: cột khoá dùng được để tìm và sắp (vd. <code>ORDER BY requested_at DESC, fare</code>) và phải có toán tử so sánh B-tree; cột INCLUDE không tham gia tìm/sắp nhưng cho phép khai <code>UNIQUE</code> chỉ trên phần khoá. Cả hai đều mất HOT khi đổi <code>fare</code>/<code>status</code>.</p>
<pre><code class="language-sql">CREATE UNIQUE INDEX uq_trips_code_kem_status ON trips (code) INCLUDE (status);
-- SELECT code, status … WHERE code BETWEEN 500000 AND 500099
-- ⇒ Index Only Scan using uq_trips_code_kem_status, Heap Fetches: 0</code></pre>
</details>

**Bài 3.** Sau khi sửa giá 1.000 chuyến liền nhau và chạy `VACUUM trips`, `Index Only Scan` trên `uq_trips_code` vẫn báo `Heap Fetches: 1000`. Chẩn đoán và sửa, không đoán mò.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">VACUUM (VERBOSE) trips;
-- index scan bypassed: 22 pages from table (0.11% of total) have 997 dead item identifiers
VACUUM (INDEX_CLEANUP ON) trips;
-- Heap Fetches: 0</code></pre>
<p>Dòng chết chỉ nằm trên 22 page (dưới ngưỡng 2 %), VACUUM bỏ qua dọn index, khe <code>LP_DEAD</code> còn lại khiến page chưa all-visible. Trước đó nên loại trừ nguyên nhân giao dịch mở lâu (VACUUM VERBOSE báo <code>dead but not yet removable</code> &gt; 0). Ép <code>INDEX_CLEANUP ON</code> là tuỳ chọn; để mặc định thì lần VACUUM sau, khi dòng chết tích đủ, sẽ dọn. Mục B của <code>p03/thu-nghiem/pg026-heap-fetch-hot-partial-include.sql</code>.</p>
</details>

## 11. Đọc tiếp

- [PostgreSQL 18 · Index-Only Scans and Covering Indexes](https://www.postgresql.org/docs/18/indexes-index-only-scans.html).
- [PostgreSQL 18 · Visibility Map](https://www.postgresql.org/docs/18/storage-vm.html) và [pg_visibility](https://www.postgresql.org/docs/18/pgvisibility.html).
- [PostgreSQL 18 · VACUUM](https://www.postgresql.org/docs/18/sql-vacuum.html) — `INDEX_CLEANUP`, `DISABLE_PAGE_SKIPPING`.
- [PostgreSQL 18 · CREATE INDEX](https://www.postgresql.org/docs/18/sql-createindex.html) — mệnh đề `INCLUDE`.
- [PostgreSQL 14 · Release Notes](https://www.postgresql.org/docs/14/release-14.html) — VACUUM bỏ qua dọn index khi ít dòng chết; [vacuumlazy.c (REL_18_1)](https://github.com/postgres/postgres/blob/REL_18_1/src/backend/access/heap/vacuumlazy.c) — ngưỡng 2 %.
- Lab: `p03/26-index-only-scan.sql`, `p03/OUTPUT.txt`, `p03/DO-DAC.md` (G4); thí nghiệm (không có trong lab chính) `p03/thu-nghiem/pg026-heap-fetch-hot-partial-include.sql`, cùng thí nghiệm hai phiên có sẵn `p03/thu-nghiem/pg026-snapshot-cu-chan-vacuum.sql`.

Bài tiếp theo: [PG-027: Statistics, ANALYZE và ước lượng cardinality](/articles/postgresql-statistics-analyze-cardinality).
