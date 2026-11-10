---
slug: postgresql-gin-gist-brin-lua-chon
title: "Khi nào cân nhắc GIN, GiST và BRIN? Ma trận workload, toán tử và giới hạn"
summary: "Tới Nơi cần tìm quán theo một mẩu tên, lọc thuộc tính jsonb, chặn ca tài xế chồng giờ và đọc GPS vài phút gần nhất — bốn việc B-tree không làm được. Lập ma trận workload × toán tử × loại index kèm giới hạn đo trên PostgreSQL 18."
category: postgresql
level: advanced
tags: [postgresql, pg-028, gin, gist, brin, pg-trgm, jsonb, toi-noi]
language: vi
---

Đội sản phẩm của Tới Nơi gửi sang bốn yêu cầu trong cùng một tuần. Ô tìm quán phải tìm được «tấm bà năm» ở *giữa* tên quán, kể cả khi khách gõ không dấu. Bộ lọc «quán chay» đọc từ cột `attrs jsonb`. Bảng ca làm việc phải **không bao giờ** có hai ca chồng giờ của cùng một tài xế. Màn hình điều phối đọc vị trí GPS của 5 phút gần nhất từ bảng `driver_locations`, nơi hồ sơ dự án ước 6.250 dòng mới mỗi giây lúc cao điểm.

Người viết migration thêm B-tree cho cả bốn, rồi `EXPLAIN` vẫn ra Seq Scan ở ba chỗ, còn chỗ thứ tư không chặn được dữ liệu sai. B-tree không hỏng. Nó chỉ hiểu `<`, `<=`, `=`, `>=`, `>` trên giá trị nguyên khối, trong khi bốn câu hỏi trên dùng những toán tử khác: «chứa mẩu chuỗi», «chứa khoá jsonb», «hai khoảng thời gian giao nhau», «nằm trong dải page này». Bài này chọn đúng loại index cho từng toán tử và đo xem mỗi loại phải trả giá gì.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-028 · Phần 3/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-btree-index-co-ban">PG-023 · B-tree index</a>, <a href="/articles/postgresql-index-nhieu-cot-partial-expression">PG-025 · Index nhiều cột, partial và expression</a>, <a href="/articles/postgresql-statistics-analyze-cardinality">PG-027 · Statistics và ước lượng</a>. Lab: <code>examples/toi-noi-postgresql/p03</code> (tệp <code>28-gin-gist-brin.sql</code>, số đo ở <code>DO-DAC.md</code>) và thí nghiệm <code>p03/thu-nghiem/pg028-gioi-han-tung-loai-index.sql</code>. Sau bài này, bạn lập được ma trận workload → toán tử → loại index → giới hạn, và kiểm từng ô của ma trận bằng EXPLAIN trước khi đưa index vào migration.</p>
</div>

## 1. Dữ liệu, cách chạy và máy đo

```bash
cd examples/toi-noi-postgresql
node pg/run-pg.mjs p03 --only 28                                                         # bài lab
node pg/run-pg.mjs --nen p03/lab.json:p03 p03/thu-nghiem/pg028-gioi-han-tung-loai-index.sql   # thí nghiệm
```

Lab chạy trên **PostgreSQL 18.1 thật**, mỗi tệp trên một bản sao mới của nền: `merchants` 15.000 quán (tên kiểu «Cơm Tấm Bà Năm», địa chỉ hư cấu, `attrs` jsonb), `driver_shifts` 158.175 ca tháng 10 kèm 3 ca chồng giờ cài sẵn, `driver_locations` do tệp bài sinh 2.000.000 điểm GPS của 1.000 tài xế, 5 giây một điểm, chèn theo thứ tự thời gian.

Thời gian chỉ lấy từ `p03/DO-DAC.md`: Intel i7-10700 (16 luồng), 64 GB RAM, NVMe SSD, Windows 11 Pro, PostgreSQL 18.1; `shared_buffers=128MB`, `work_mem=4MB`, `random_page_cost=4`; parallel query, JIT, autovacuum **tắt**. Mỗi phép đo chạy 1 lần khởi động rồi 7 lần (5 lần với phép nặng), lấy **trung vị**, cache **nóng**. Máy có tải nền nên số tuyệt đối dao động khoảng ±20 % giữa các lần chạy lệnh đo. Dữ liệu tổng hợp, một máy: chỉ so sánh tương đối.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Kiểm locale trước khi dùng pg_trgm với tiếng Việt</span>
<p>pg_trgm phân loại «ký tự chữ» theo LC_CTYPE của database. Cluster lab dùng provider builtin <code>C.UTF-8</code> nhưng đặt <code>LC_CTYPE=en-US</code> (Windows), nên <code>show_trgm('Cơm Tấm')</code> cho 8 trigram. Thí nghiệm tạo một database <code>LC_CTYPE=C</code>: cùng chuỗi chỉ còn <code>{"  c","  m","  t"," c "," m "," t "}</code>, mọi chữ có dấu thành dấu cách, và <code>similarity('Cơm Tấm', 'Cám Tôm')</code> bằng <b>1,00</b> (database lab: 0,14). Chạy <code>SELECT show_trgm('Cơm Tấm')</code> trên database của bạn trước khi tin bất kỳ số nào dưới đây. Release notes PostgreSQL 18 cũng khuyên REINDEX index pg_trgm sau pg_upgrade với cluster dùng provider không phải libc.</p>
</div>

## 2. Câu hỏi đúng: toán tử nào, operator class nào?

Index trong PostgreSQL không phục vụ «cột» mà phục vụ **toán tử** trên cột, thông qua **operator class** (lớp toán tử: bộ quy tắc cho biết một loại index hiểu những toán tử nào trên một kiểu dữ liệu). Cùng cột `name`, B-tree mặc định hiểu so sánh thứ tự; `gin_trgm_ops` hiểu `LIKE '%…%'`, `ILIKE`, `%` (độ giống). Muốn chọn index, đọc mệnh đề `WHERE` và gạch chân toán tử trước, rồi mới hỏi «index loại gì».

| Loại | Cấu trúc | Toán tử tiêu biểu (lõi + extension trong bài) | Ý tưởng một câu |
|---|---|---|---|
| B-tree | cây có thứ tự | `<` `<=` `=` `>=` `>` | sắp xếp giá trị nguyên khối |
| **GIN** | inverted index (chỉ mục ngược) | `@>` `?` (jsonb), `LIKE`/`ILIKE`/`%` (pg_trgm), `&&` `@>` (mảng) | mỗi giá trị tách thành nhiều «thành phần», index trỏ từ thành phần về dòng |
| **GiST** | cây cân bằng tổng quát | `&&` `@>` `<@` (range, hình học), `<->` (khoảng cách), `=` (qua btree_gist) | khung để cài nhiều chiến lược tìm, nhất là «giao nhau/chứa» |
| **BRIN** | tóm tắt theo dải page | `<` `<=` `=` `>=` `>` | lưu (min, max) cho mỗi dải page liền nhau, bỏ dải không thể khớp |

SP-GiST và hash không thuộc phạm vi bài.

## 3. GIN + pg_trgm: tìm quán theo mẩu tên

**Trigram** là cụm 3 ký tự liên tiếp của mỗi từ, sau khi đệm hai khoảng trắng đầu và một khoảng trắng cuối từ. «Cơm Tấm» có 8 trigram. Index GIN `gin_trgm_ops` lưu, cho mỗi trigram, danh sách dòng chứa nó. Tìm `'%tấm bà năm%'` là lấy giao các danh sách trigram của mẫu, rồi **Recheck** biểu thức thật trên heap, vì đủ trigram chưa chắc đúng thứ tự.

```sql
CREATE INDEX ix_merchants_name_trgm ON merchants USING gin (name gin_trgm_ops);
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT name, address FROM merchants WHERE name ILIKE '%tấm bà năm%';
```

```text
 Bitmap Heap Scan on merchants (actual rows=75.00 loops=1)
   Recheck Cond: (name ~~* '%tấm bà năm%'::text)
   Heap Blocks: exact=71
   Buffers: shared hit=93
   ->  Bitmap Index Scan on ix_merchants_name_trgm (actual rows=75.00 loops=1)
         Index Cond: (name ~~* '%tấm bà năm%'::text)
         Index Searches: 1
         Buffers: shared hit=22
```

Trước khi có index: Seq Scan 412 page, `Rows Removed by Filter: 14925`. DO-DAC đo trung vị **5,69 ms** (Seq Scan) và **0,32 ms** (GIN). Tìm địa chỉ `'%đường hoa sứ%quận 3,%'` ra 84 quán qua index GIN trên `address`, với `Rows Removed by Index Recheck: 2`: index trả 86 ứng viên có đủ trigram, hai dòng không khớp thứ tự.

Các giới hạn đã kiểm:

- **Mẫu dưới 3 ký tự**: `'%gà%'` vẫn Seq Scan. Tài liệu pg_trgm ghi mẫu không trích được trigram nào sẽ thoái hoá thành quét toàn index; planner chọn quét bảng.
- **Gõ không dấu**: `'%com tam ba nam%'` ra **0** dòng, vì trigram `tam` khác `tấm`. Cách làm phổ biến là index trên biểu thức bỏ dấu. `unaccent()` chỉ là STABLE (từ điển có thể đổi), nên phải bọc trong một hàm IMMUTABLE tự viết, chấp nhận REINDEX khi đổi từ điển:

```sql
CREATE FUNCTION khong_dau(text) RETURNS text
LANGUAGE sql IMMUTABLE PARALLEL SAFE STRICT
AS $$ SELECT lower(public.unaccent('public.unaccent'::regdictionary, $1)) $$;
CREATE INDEX ix_merchants_name_khong_dau ON merchants USING gin (khong_dau(name) gin_trgm_ops);
```

  Câu `WHERE khong_dau(name) LIKE '%com tam ba nam%'` ra lại 75 quán qua index mới. Gõ sai chính tả `'com tam ba nan'` với toán tử `%` vẫn tìm được «Cơm Tấm Bà Năm», độ giống **0,81**.
- **Sắp theo độ giống**: lab dùng `ORDER BY khong_dau(name) <-> 'com tam ba nan'` trên GIN, kế hoạch là Bitmap → Sort → Limit (sắp mọi ứng viên). Theo tài liệu pg_trgm, chỉ GiST (`gist_trgm_ops`) trả dòng theo thứ tự khoảng cách trực tiếp từ index; GIN không làm được.
- **So sánh thứ tự**: `name > 'Cơm Tấm' ORDER BY name` không dùng index trigram (thí nghiệm, Seq Scan + Sort). Trigram không có khái niệm «trước/sau».
- **Kích thước**: bảng `merchants` 3.375.104 B; GIN địa chỉ **1.425.408 B** (≈ 42 % bảng); GIN tên 655.360 B; GIN tên không dấu 630.784 B. Mỗi chuỗi sinh hàng chục trigram.
- **Hàng chờ ghi** (pending list): mặc định `fastupdate = on`, dòng mới vào một danh sách chưa sắp thay vì vào cây chính. Thí nghiệm chèn 2.000 quán: index mặc định có `pending_pages = 108`, `pending_tuples = 2000` (`pgstatginindex`, extension pgstattuple); index `fastupdate = off` có hàng chờ rỗng. Tìm vẫn đúng (2.000 quán mới), nhưng mỗi lần tìm phải quét thêm hàng chờ. `gin_clean_pending_list()` dọn 108 page; VACUUM và autoanalyze cũng dọn.

## 4. GIN cho jsonb: hai operator class, hai bộ toán tử

```sql
SELECT count(*) FROM merchants WHERE attrs @> '{"tags": ["chay"]}';   -- 1204
```

| Operator class | Toán tử phục vụ | Lab |
|---|---|---|
| `jsonb_path_ops` | `@>`, `@?`, `@@` | `@> '{"tags": ["chay"]}'` dùng index; `attrs ? 'khuyen_mai'` vẫn **Seq Scan** |
| `jsonb_ops` (mặc định) | thêm `?`, `?|`, `?&` | `?` dùng index; index **to hơn** `jsonb_path_ops` trên cùng dữ liệu |

Tài liệu giải thích: `jsonb_path_ops` chỉ tạo mục cho mỗi giá trị (băm cùng đường khoá dẫn tới nó), còn `jsonb_ops` tạo mục cho từng khoá và từng giá trị riêng.

Bẫy hay gặp nhất nằm ở cách viết điều kiện. Thí nghiệm tìm quán mở cửa lúc 07:00 (2.467 quán) bằng hai cách cùng nghĩa:

```text
-- WHERE attrs ->> 'gio_mo' = '07:00'
 Aggregate
   ->  Seq Scan on merchants
         Filter: ((attrs ->> 'gio_mo'::text) = '07:00'::text)
-- WHERE attrs @> '{"gio_mo": "07:00"}'
 Aggregate
   ->  Bitmap Heap Scan on merchants
         Recheck Cond: (attrs @> '{"gio_mo": "07:00"}'::jsonb)
         ->  Bitmap Index Scan on ix_merchants_attrs
```

`->>` trả `text`, và `text = text` không phải toán tử của operator class jsonb nào, nên index GIN bị bỏ qua. Muốn so sánh khoảng (`attrs ->> 'gio_mo' < '06:00'`), GIN jsonb hoàn toàn bất lực: cần B-tree trên biểu thức `((attrs ->> 'gio_mo'))` như PG-025. Khoá được lọc thường xuyên bằng so sánh thứ tự là dấu hiệu nên tách thành cột riêng.

## 5. GiST: quy tắc «không chồng giờ» giữa nhiều dòng

Hai ca của cùng tài xế không được giao nhau. CHECK chỉ nhìn một dòng; UNIQUE chỉ so «bằng nhau». **Exclusion constraint** (`EXCLUDE`) phát biểu: không tồn tại hai dòng có `driver_id` bằng nhau **và** `shift && shift` (hai khoảng giao nhau). Ràng buộc cần một index hiểu cả hai toán tử: GiST hiểu `&&` trên `tstzrange`, còn `=` trên `uuid` thì phải nhờ extension **btree_gist**.

```sql
ALTER TABLE driver_shifts
  ADD CONSTRAINT ex_driver_shifts_khong_chong EXCLUDE USING gist (driver_id WITH =, shift WITH &&);
```

```text
ERROR:  could not create exclusion constraint "ex_driver_shifts_khong_chong"
DETAIL:  Key (driver_id, shift)=(bfd06b3f-…, ["2026-10-15 13:53:00+07","2026-10-15 23:53:00+07")) conflicts with key (driver_id, shift)=(bfd06b3f-…, ["2026-10-15 15:53:00+07","2026-10-16 01:53:00+07")).
-- [lab] SQLSTATE 23P01 · constraint ex_driver_shifts_khong_chong · table driver_shifts
```

Dữ liệu cũ đã vi phạm. Tìm bằng self-join theo đúng toán tử của ràng buộc (ra 3 cặp của tài xế 20007, 23605, 25401, ca sau nhập lúc 2026-10-20 10:00), xoá 3 ca nhập sau, rồi thêm lại thành công. Ràng buộc tự tạo index `gist` cùng tên. Sau đó ca chồng giờ bị chặn ngay khi INSERT (`23P01` kèm DETAIL chỉ ra ca đang có). Ca **liền sau**, bắt đầu đúng lúc ca trước kết thúc, hợp lệ, vì `tstzrange` mặc định nửa mở `[bắt đầu, kết thúc)`.

Index của ràng buộc phục vụ luôn câu điều phối «ai đang trong ca lúc 09:00 ngày 15/10»: `shift @> TIMESTAMPTZ '…'` ra **1.713** ca qua `Index Only Scan using ex_driver_shifts_khong_chong`.

Giới hạn và đối chiếu (thí nghiệm, phần C):

- **B-tree trên `tstzrange`** tạo được (sắp theo cận dưới rồi cận trên) nhưng cả `@>` lẫn `&&` đều Seq Scan. GiST `range_ops` có sẵn trong lõi phục vụ cả hai, không cần extension.
- **PostgreSQL 18** thêm khoá chính/UNIQUE «theo thời gian» (release notes 18: `WITHOUT OVERLAPS` cho cột cuối của `PRIMARY KEY`/`UNIQUE`). `PRIMARY KEY (driver_id, shift WITHOUT OVERLAPS)` tạo index **gist**, ca chồng giờ bị chặn với cùng mã `23P01`; khoảng rỗng bị từ chối bằng `23514`. Với cột `uuid` vẫn cần btree_gist.
- Tài liệu btree_gist: operator class của nó thường **không nhanh hơn** B-tree tương ứng và không tự cưỡng chế uniqueness. Dùng nó để ghép cột thường vào index GiST, không để thay B-tree.

## 6. BRIN: bảng chỉ-chèn theo thời gian

`driver_locations` được ghi theo thời gian và hầu như chỉ được đọc «vài phút gần nhất». **BRIN** (Block Range INdex) không trỏ tới từng dòng. Với mỗi **dải** `pages_per_range` page liền nhau (mặc định 128), nó lưu (min, max) của cột. Khi tìm, dải nào không thể khớp thì bỏ; dải có thể khớp thì đọc **cả dải** và Recheck từng dòng: index **lossy** (mất chính xác).

| Đối tượng (2.000.000 điểm) | Kích thước |
|---|---:|
| Bảng `driver_locations` | 136.536.064 B |
| B-tree `recorded_at` (đã gộp khoá trùng) | 14.655.488 B |
| BRIN `recorded_at` | **24.576 B** |

BRIN nhỏ hơn B-tree khoảng 600 lần. Nhưng nó chỉ có ích khi giá trị **đi cùng thứ tự vật lý**. Lab tạo bản sao xáo trộn thứ tự dòng (`correlation` 1,000 so với 0,0) rồi đọc bản tóm tắt bằng `brin_page_items` (extension pageinspect):

```text
-- bảng THEO THỨ TỰ: các dải nối tiếp, không chồng
      0 | {2026-10-30 22:13:25+07 .. 2026-10-30 22:14:40+07}
    128 | {2026-10-30 22:14:40+07 .. 2026-10-30 22:15:55+07}
-- bảng XÁO TRỘN: dải nào cũng phủ trọn 2 giờ 47 phút
      0 | {2026-10-30 22:13:25+07 .. 2026-10-31 01:00:00+07}
    128 | {2026-10-30 22:13:25+07 .. 2026-10-31 01:00:00+07}
```

Câu «5 phút gần nhất» (61.000 điểm, khoảng 3 % bảng), cache nóng:

| Phương án | Page / chi tiết | Trung vị (DO-DAC) |
|---|---|---:|
| BRIN, bảng theo thứ tự | `Heap Blocks: lossy=539`, Recheck bỏ 3.640 dòng | 5,59 ms |
| Seq Scan, bảng theo thứ tự | 16.667 page | 85,59 ms |
| B-tree, Index Only Scan | 58 page, `Heap Fetches: 61000` (bảng chưa VACUUM) | 3,33 ms |
| Bảng xáo trộn, planner chọn Seq Scan | 16.704 page | 90,11 ms |
| Bảng xáo trộn, **ép** BRIN | `lossy=16704`, Recheck bỏ **1.939.000** dòng | **140,96 ms** |

Trên dữ liệu đúng thứ tự, BRIN 24 KB đạt cùng bậc tốc độ với B-tree 14 MB. Trên dữ liệu xáo trộn, BRIN tệ hơn cả Seq Scan. Planner biết điều đó nhờ `correlation` (PG-027) nên tự chọn Seq Scan; chỉ khi ép (`enable_seqscan = off`) mới thấy cái giá.

Hai giới hạn nữa từ thí nghiệm (phần D, bảng 500.000 điểm, 3.677 page, câu «5 phút đầu» 60.000 điểm, ép dùng BRIN):

- **Cỡ dải.** `pages_per_range = 128`: `lossy=512`, Recheck bỏ 9.632 dòng. `pages_per_range = 32`: `lossy=448`, bỏ 928 dòng. Cả hai index đều 3 page. Dải nhỏ chính xác hơn, đổi lại nhiều mục tóm tắt hơn (rõ hơn ở bảng lớn).
- **Dải mới chưa tóm tắt.** Nạp thêm 100.000 điểm *mới hơn*: bảng lên 4.412 page. Câu hỏi về 5 phút **đầu** (kết quả vẫn 60.000) giờ đọc `lossy=1212` và bỏ 104.800 dòng, vì 700 page mới thuộc các dải chưa có bản tóm tắt nên luôn bị đọc. `brin_summarize_new_values()` tóm tắt **6** dải, `lossy` về lại 512. Tài liệu BRIN: dải mới chỉ được tóm tắt khi VACUUM, khi gọi hàm tóm tắt, hoặc khi index bật `autosummarize` (mặc định **tắt**).

## 7. Ma trận lựa chọn

Đây là sản phẩm của bài. Mỗi dòng có bằng chứng ở lab hoặc thí nghiệm; ô «không phục vụ» là những câu bạn sẽ viết nhầm.

| Workload Tới Nơi | Toán tử trong WHERE | Index | Không phục vụ / giới hạn | Bằng chứng |
|---|---|---|---|---|
| Tìm quán theo mẩu tên, địa chỉ | `LIKE`/`ILIKE '%…%'`, `%` | GIN `gin_trgm_ops` | mẫu < 3 ký tự; `<`/`>`/sắp xếp; cần index biểu thức để bỏ dấu; phụ thuộc LC_CTYPE; index ≈ 19–42 % bảng | 412 → 71 page heap; 5,69 → 0,32 ms |
| Gợi ý «có phải ý bạn là…» theo độ giống | `%` + `ORDER BY <->` | GiST `gist_trgm_ops` (theo tài liệu) | GIN lọc được `%` nhưng phải Sort mọi ứng viên | kế hoạch Bitmap → Sort → Limit |
| Lọc thuộc tính quán | `@>`, `@?`, `@@` | GIN `jsonb_path_ops` | không phục vụ `?`; `->> =` không dùng GIN | `?` → Seq Scan |
| Có/không có khoá jsonb | `?`, `?\|`, `?&` | GIN `jsonb_ops` | to hơn `jsonb_path_ops` | `?` → Bitmap Index Scan |
| So sánh khoảng trên một khoá jsonb | `->> <` | B-tree biểu thức | GIN jsonb bất lực | `ix_merchants_gio_mo` được dùng |
| Một tài xế không có hai ca chồng giờ | `=` + `&&` | `EXCLUDE USING gist` + btree_gist; PG 18: `WITHOUT OVERLAPS` | dữ liệu cũ vi phạm ⇒ thêm ràng buộc lỗi 23P01; btree_gist không thay B-tree | 3 cặp chồng; 23P01 |
| Ai đang trong ca lúc t | `@>`, `&&` | GiST `range_ops` | B-tree trên range: Seq Scan | 1.713 ca, Index Only Scan |
| GPS vài phút gần nhất | `>=`, `<` | BRIN `recorded_at` | cần thứ tự vật lý; lossy; dải mới chưa tóm tắt | 24 KB, 5,59 ms; xáo trộn 140,96 ms |
| Tra một điểm GPS chính xác | `=` | B-tree (PK) | BRIN đọc cả dải 128 page | — |

## 8. Chi phí ghi và vận hành

- **GIN**: một dòng sinh nhiều khoá, nên chèn chậm. Tài liệu GIN khuyên với lô nạp lớn hãy xoá index rồi tạo lại sau. Hàng chờ `fastupdate` làm ghi nhanh hơn nhưng dồn việc vào lúc dọn: câu ghi đẩy hàng chờ vượt `gin_pending_list_limit` sẽ tự dọn và chậm hẳn. Cần thời gian phản hồi đều thì `fastupdate = off`.
- **GiST** của exclusion constraint chạy trên **mọi** INSERT/UPDATE `driver_shifts`: đó là cái giá của một bất biến mà ứng dụng không tự kiểm an toàn được khi có nhiều phiên ghi đồng thời (Phần 4).
- **BRIN** gần như không tốn khi ghi theo thứ tự, nhưng cần lịch tóm tắt (autovacuum hoặc `autosummarize = on`). Dữ liệu cũ bị UPDATE/xoá rồi chèn lại lung tung sẽ làm dải «loang» dần, giống bản xáo trộn.
- Mỗi index mới còn làm mất HOT update khi cột của nó bị sửa (PG-024, PG-026). `driver_locations` chỉ chèn nên không chịu chi phí này; `merchants` ít sửa nên chịu được index trigram lớn.

## 9. Lỗi thường gặp

- **Chọn index theo kiểu cột thay vì theo toán tử.** «Cột jsonb thì GIN» rồi viết `attrs ->> 'k' = 'v'`: index không bao giờ được dùng.
- **Không kiểm `EXPLAIN` với chính mẫu người dùng gõ.** Mẫu 2 ký tự, mẫu không dấu, mẫu chỉ có số đều có thể rơi về Seq Scan.
- **Dùng hàm STABLE trong index** (`unaccent` trực tiếp): bị từ chối, cùng mã lỗi `42P17` như ví dụ `requested_at::date` ở PG-025. Bọc IMMUTABLE thì phải tự chịu trách nhiệm REINDEX khi từ điển đổi.
- **Tin BRIN vì nó nhỏ.** Kiểm `correlation` trong `pg_stats` trước; gần 0 thì BRIN vô dụng.
- **Thêm EXCLUDE mà không dọn dữ liệu cũ**: migration thất bại giữa chừng với 23P01. Chạy self-join tìm vi phạm trước.
- **Quên rằng kết quả trigram phụ thuộc locale** khi chuyển database sang máy/cluster khác.

## 10. Bài tập

**Bài 1.** Màn hình CSKH lọc «quán mở cửa trước 06:00» bằng `attrs ->> 'gio_mo' < '06:00'`, bảng đã có `CREATE INDEX … USING gin (attrs)`. Kế hoạch ra sao? Sửa thế nào?

<details><summary><b>Lời giải</b></summary>
<p>Seq Scan: index GIN jsonb không hiểu so sánh <code>text &lt; text</code>, kể cả phép <code>=</code> viết bằng <code>-&gt;&gt;</code> cũng không. Viết lại bằng <code>@&gt;</code> chỉ được cho phép bằng (2.467 quán mở lúc 07:00 qua Bitmap Index Scan). Phép so sánh khoảng cần B-tree biểu thức:</p>
<pre><code class="language-sql">CREATE INDEX ix_merchants_gio_mo ON merchants ((attrs -&gt;&gt; 'gio_mo'));
EXPLAIN (COSTS OFF) SELECT count(*) FROM merchants WHERE attrs -&gt;&gt; 'gio_mo' &lt; '06:00';
-- Bitmap Index Scan on ix_merchants_gio_mo</code></pre>
<p>Lời giải chạy thật: <code>p03/thu-nghiem/pg028-gioi-han-tung-loai-index.sql</code>, phần B.</p>
</details>

**Bài 2.** Sau một đêm nạp bù GPS, câu «điểm của ca sáng hôm qua» trên index BRIN chậm hẳn dù kết quả không đổi. `EXPLAIN (ANALYZE)` cho thấy `lossy` tăng. Giải thích và sửa.

<details><summary><b>Lời giải</b></summary>
<p>Page mới nạp nằm ở các dải chưa có bản tóm tắt; BRIN không biết trong đó có gì nên mọi câu hỏi đều phải đọc chúng. Thí nghiệm: 512 page lossy thành 1.212 sau khi nạp 100.000 điểm mới (700 page chưa tóm tắt), bỏ 104.800 dòng khi Recheck.</p>
<pre><code class="language-sql">SELECT brin_summarize_new_values('ix_gps_brin128');   -- 6 dải
-- hoặc lâu dài: CREATE INDEX … USING brin (recorded_at) WITH (autosummarize = on);</code></pre>
<p>Sau khi tóm tắt, <code>lossy</code> về 512. Nếu bản nạp bù chèn điểm <i>cũ</i> vào cuối bảng thì dải mới phủ khoảng thời gian rộng; khi đó tóm tắt không cứu được, phải sắp lại dữ liệu hoặc dùng B-tree.</p>
</details>

**Bài 3.** Thiết kế bảng ca mới cho PostgreSQL 18 mà không viết `EXCLUDE`, rồi chứng minh hai ca 07:00–15:00 và 15:00–19:00 cùng tài xế hợp lệ, còn ca 14:00–16:00 bị chặn.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">CREATE EXTENSION btree_gist;   -- cho cột uuid
CREATE TABLE driver_shifts_v18 (
  driver_id uuid      NOT NULL,
  shift     tstzrange NOT NULL,
  CONSTRAINT pk_driver_shifts_v18 PRIMARY KEY (driver_id, shift WITHOUT OVERLAPS)
);</code></pre>
<p>Index của khoá chính là <code>gist</code>. Hai ca liền nhau chèn được (nửa mở); ca 14:00–16:00 lỗi <code>23P01 … violates exclusion constraint "pk_driver_shifts_v18"</code>; <code>'empty'::tstzrange</code> lỗi <code>23514</code>. Thí nghiệm phần C.</p>
</details>

## 11. Đọc tiếp

- [PostgreSQL 18 · Index Types](https://www.postgresql.org/docs/18/indexes-types.html) và các chương [GIN](https://www.postgresql.org/docs/18/gin.html), [GiST](https://www.postgresql.org/docs/18/gist.html), [BRIN](https://www.postgresql.org/docs/18/brin.html).
- [pg_trgm](https://www.postgresql.org/docs/18/pgtrgm.html), [btree_gist](https://www.postgresql.org/docs/18/btree-gist.html), [jsonb Indexing](https://www.postgresql.org/docs/18/datatype-json.html).
- [CREATE TABLE](https://www.postgresql.org/docs/18/sql-createtable.html) (`WITHOUT OVERLAPS`), [release notes PostgreSQL 18](https://www.postgresql.org/docs/release/18.0/).
- Lab: `examples/toi-noi-postgresql/p03/28-gin-gist-brin.sql`, `p03/OUTPUT.txt`, `p03/DO-DAC.md` (nhóm G5); thí nghiệm `p03/thu-nghiem/pg028-gioi-han-tung-loai-index.sql`.

Bài tiếp theo: [PG-029 · Sort, hash và truy vấn tràn ra đĩa](/articles/postgresql-sort-hash-memory-spill)
