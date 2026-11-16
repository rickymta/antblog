---
slug: mssql-bang-khoa-chinh-clustered-index
title: "Bảng, khoá chính và clustered index trong nghiệp vụ chuyến xe"
summary: "Tổng đài Tới Nơi cần chỉ ra đúng một chuyến trong hai chuyến đặt cùng 12:00. Trên SQL Server, chọn khoá còn quyết định bảng được lưu theo thứ tự nào: vì sao trips có khoá chính uniqueidentifier NONCLUSTERED nhưng clustered index lại nằm trên mã chuyến."
category: mssql
level: beginner
tags: [ms-003, mssql, sql-server-2022, primary-key, foreign-key, clustered-index, toi-noi]
language: vi
---

Tổng đài Tới Nơi nhận cuộc gọi: «Tôi đặt đồ ăn lúc 12 giờ trưa hôm qua, chuyến đó bị sao vậy?». Nhân viên mở dữ liệu ngày 02/11 và thấy **hai** chuyến đồ ăn cùng đặt đúng 12:00: chuyến ở Hà Nội đã huỷ, chuyến ở Đà Nẵng đã hoàn thành. Giờ đặt không đủ để chỉ ra chuyến nào; tên khách cũng không đủ, vì trùng tên là chuyện thường.

Câu hỏi «làm sao chỉ ra đúng **một** chuyến, rồi từ đó tới đúng **một** khách, **một** tài xế?» là câu hỏi về khoá. Trên SQL Server còn một câu hỏi thứ hai mà PostgreSQL không đặt ra: **bảng được sắp xếp và lưu theo cột nào?** Câu trả lời nằm ở clustered index, và với bảng `trips` của Tới Nơi, cột đó không phải khoá chính.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-003 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a>, <a href="/articles/mssql-cai-dat-docker-dbstudio">MS-002: Dựng SQL Server 2022 bằng Docker và kết nối bằng DBStudio</a>. Lab: <code>examples/toi-noi-mssql/p01</code> (<code>01-schema.sql</code>, <code>02-fixture-valid.sql</code>, <code>03-fixture-invalid.sql</code>). Sau bài này, bạn chọn được khoá chính và clustered key cho một bảng, và vẽ được quan hệ khách–chuyến–tài xế.</p>
</div>

## 1. Một dòng là gì: bắt đầu từ «một chuyến»

Dữ liệu SQL Server nằm trong **table** (bảng), thuộc một **schema** (nhóm đối tượng có tên) trong một database. Lab đặt mọi bảng nghiệp vụ trong schema `toinoi`, nên tên đầy đủ của bảng chuyến là `ToiNoi.toinoi.trips` (database, schema, bảng; MS-001 giải thích ba tầng này). Mỗi bảng có danh sách **column** (cột) cố định và chứa nhiều **row** (dòng).

Câu hỏi đầu tiên khi thiết kế bảng là: **một dòng của bảng này là gì?** Người thiết kế dữ liệu gọi đó là grain của bảng.

| Bảng trong schema `toinoi` | Một dòng là | Số dòng sau `02-fixture-valid.sql` |
|---|---|---:|
| `cities` | một thành phố | 3 |
| `customers` | một khách | 6 |
| `drivers` | một tài xế | 6 |
| `merchants` | một quán | 3 |
| `trips` | một chuyến (đặt xe, đồ ăn hoặc bưu kiện) | 15 |
| `trip_events` | một lần chuyến đổi trạng thái | 0 (ghi từ MS-006) |

Một dòng `trips` là một chuyến, nên chuyến chỉ có **một** trạng thái hiện tại; chuỗi «requested → searching → accepted…» cần một bảng riêng là `trip_events`. Trộn hai grain trong một bảng là nguồn lỗi mà [MS-008](/articles/mssql-mo-hinh-quan-he-chuan-hoa) sẽ phân tích.

Trong DBStudio, mở cây đối tượng của kết nối: **ToiNoi → toinoi → Tables** liệt kê sáu bảng; mở một bảng sẽ thấy ba nhóm **Columns**, **Indexes**, **Triggers**. Bài này dùng nhiều nhất nhóm Indexes.

## 2. Cột: tên, kiểu và có được để trống

Mỗi cột có tên, **data type** (kiểu dữ liệu) và quy định có được chứa `NULL` (giá trị «không có / chưa biết»). Cuối `01-schema.sql`, lab đọc catalog `sys.columns` để in cấu trúc `trips`. Kết quả trong `OUTPUT.txt`, giữ 9 trong 13 dòng:

```text
stt|cot|kieu|null_|mac_dinh
---|---|----|-----|--------
1|id|uniqueidentifier|NOT NULL|df_trips_id
2|code|bigint|NOT NULL|IDENTITY
3|kind|varchar(10)|NOT NULL|
5|customer_id|uniqueidentifier|NOT NULL|
6|driver_id|uniqueidentifier|NULL|
8|status|varchar(12)|NOT NULL|df_trips_status
9|fare|int|NOT NULL|
10|requested_at|datetimeoffset(3)|NOT NULL|df_trips_requested_at
11|accepted_at|datetimeoffset(3)|NULL|
```

- **`null_`**: `driver_id` cho phép `NULL` vì chuyến vừa đặt chưa có tài xế; `customer_id` thì không, vì không chuyến nào thiếu khách.
- **`mac_dinh`**: tên của ràng buộc DEFAULT (giá trị tự điền khi câu INSERT không nhắc tới cột). Trên SQL Server, DEFAULT là một đối tượng có tên; lab đặt tên `df_…` cho mọi DEFAULT, lý do ở [MS-005](/articles/mssql-create-table-constraints).
- **`IDENTITY`**: `code` do SQL Server tự cấp số tăng dần.
- Vì sao chọn `uniqueidentifier`, `int` cho tiền, `datetimeoffset(3)` cho thời điểm là chủ đề của [MS-004](/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode).

## 3. Nhận diện một dòng: khoá và primary key

Một **key** (khoá) là tập cột mà giá trị không bao giờ trùng giữa hai dòng, nên biết giá trị khoá là chỉ ra đúng một dòng. Một bảng có thể có vài khoá ứng viên: với `customers`, cả `id` lẫn `phone` đều không trùng.

Ta chọn **một** khoá ứng viên làm **primary key** (khoá chính): không trùng, không `NULL`, mỗi bảng tối đa một khoá chính. Các khoá còn lại khai báo bằng ràng buộc `UNIQUE`. SQL Server thực thi cả hai bằng một unique index, và database thật sự **chặn** dữ liệu vi phạm. Hai câu trong `03-fixture-invalid.sql` thêm thành phố trùng `id` và khách trùng số điện thoại (kết quả trong `OUTPUT.txt`):

```text
PASS U1 trùng id thành phố — lỗi 2627: Violation of PRIMARY KEY constraint 'pk_cities'. Cannot insert duplicate key in object 'toinoi.cities'. The duplicate key value is (1).
PASS U2 trùng số điện thoại — lỗi 2627: Violation of UNIQUE KEY constraint 'uq_customers_phone'. Cannot insert duplicate key in object 'toinoi.customers'. The duplicate key value is (0900000001).
```

Cả hai mang **số hiệu lỗi** 2627: khoá chính và UNIQUE dùng chung cơ chế «không trùng». Tên ràng buộc (`pk_cities`, `uq_customers_phone`) nằm ngay trong thông báo, nhờ lab đặt tên theo tiền tố `pk_`, `uq_`, `fk_`, `ck_`, `df_`.

**Vì sao không lấy số điện thoại làm khoá chính?** `phone` là **natural key** (khoá tự nhiên, có nghĩa ngoài đời): khách đổi số, nhà mạng cấp lại số cũ cho người khác, và mọi chuyến trỏ tới số đó phải sửa theo. Tới Nơi dùng **surrogate key** (khoá thay thế, không mang nghĩa nghiệp vụ) `id uniqueidentifier`, giữ `phone` là `UNIQUE`. Đây là đánh đổi chứ không phải luật: `cities` dùng `id smallint` do quản trị viên đặt, vì danh mục nhỏ và gần như không đổi.

## 4. Clustered index: bảng chính là một cây B-tree

Đến đây, câu chuyện giống mọi database quan hệ. Điểm riêng của SQL Server bắt đầu từ cách lưu bảng.

SQL Server lưu dữ liệu trong các **page** (trang) 8 KB. Một bảng được lưu theo một trong hai cách:

- **Heap** (đống): bảng **không có** clustered index. Các dòng nằm ở trang nào còn chỗ, không theo thứ tự nào. Index phụ trỏ tới dòng bằng **RID** (row identifier: số file, số trang, vị trí trong trang).
- **Clustered index**: một cây **B-tree** (cây cân bằng nhiều nhánh) mà **tầng lá chứa chính các dòng dữ liệu**, sắp theo **clustered key** (các cột khoá của clustered index). Tầng trên chỉ chứa khoá để dẫn đường xuống trang lá. Vì các dòng chỉ xếp được theo một thứ tự, **mỗi bảng có tối đa một clustered index**. Clustered index *là* bảng, không phải một bản sao bên cạnh bảng.

Mọi index khác là **nonclustered index**: một B-tree riêng mà tầng lá chứa khoá của index cùng một **row locator** để quay về dòng. Với bảng có clustered index, row locator chính là **clustered key**; với heap, đó là RID.

Ràng buộc PRIMARY KEY mặc định tạo **clustered** index nếu bảng chưa có clustered index nào. Thử trên một database nháp (`Scratch_ms003`, không thuộc lab), ba bảng chỉ khác nhau ở khoá chính:

```sql
CREATE TABLE dbo.t_heap            (id int NOT NULL, ghi_chu nvarchar(50));
CREATE TABLE dbo.t_pk_mac_dinh     (id int NOT NULL PRIMARY KEY, ghi_chu nvarchar(50));
CREATE TABLE dbo.t_pk_nonclustered (id int NOT NULL PRIMARY KEY NONCLUSTERED, ghi_chu nvarchar(50));

SELECT OBJECT_NAME(object_id) AS bang, index_id, ISNULL(name, N'(không tên)') AS index_name, type_desc
FROM sys.indexes
WHERE object_id IN (OBJECT_ID('dbo.t_heap'), OBJECT_ID('dbo.t_pk_mac_dinh'), OBJECT_ID('dbo.t_pk_nonclustered'))
ORDER BY bang, index_id;
```

```text
bang|index_id|index_name|type_desc
----|--------|----------|---------
t_heap|0|(không tên)|HEAP
t_pk_mac_dinh|1|PK__t_pk_mac__3213E83FA19DCBDD|CLUSTERED
t_pk_nonclustered|0|(không tên)|HEAP
t_pk_nonclustered|2|PK__t_pk_non__3213E83E89A303C2|NONCLUSTERED
```

Trong `sys.indexes`, mỗi bảng có đúng một dòng `index_id` 0 (heap) hoặc 1 (clustered); các index nonclustered có `index_id` từ 2. Bảng thứ ba có khoá chính nhưng vẫn là heap. Hai tên `PK__t_pk_mac__3213E83F…` do SQL Server tự sinh vì câu lệnh không đặt tên, chuyện này để MS-005 xử lý.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 400" role="img" aria-label="Hai cây B-tree của bảng trips: clustered index uq_trips_code có tầng lá là các dòng dữ liệu sắp theo code; nonclustered index pk_trips có tầng lá chứa id kèm code để quay về clustered index.">
<text x="20" y="22" class="t-b">uq_trips_code · CLUSTERED (chính là bảng trips)</text>
<rect x="190" y="40" width="210" height="52" rx="6" class="f-pk-soft s-pk"/>
<text x="295" y="62" text-anchor="middle" class="t-small">trang gốc</text>
<text x="295" y="82" text-anchor="middle" class="t-mono t-10">100000 · 100005 · 100010</text>
<line x1="230" y1="92" x2="100" y2="150" class="s-line"/>
<line x1="295" y1="92" x2="295" y2="150" class="s-line"/>
<line x1="360" y1="92" x2="490" y2="150" class="s-line"/>
<rect x="20" y="150" width="170" height="150" rx="6" class="f-surface s-line"/>
<text x="30" y="172" class="t-small">trang lá: dòng đầy đủ</text>
<text x="30" y="196" class="t-mono t-10">100000 ride completed</text>
<text x="30" y="216" class="t-mono t-10">100001 parcel completed</text>
<text x="30" y="236" class="t-mono t-10">100002 food completed</text>
<text x="30" y="256" class="t-mono t-10">100003 food cancelled</text>
<text x="30" y="276" class="t-mono t-10">100004 food completed</text>
<rect x="210" y="150" width="170" height="150" rx="6" class="f-surface s-line"/>
<text x="220" y="172" class="t-small">trang lá</text>
<text x="220" y="196" class="t-mono t-10">100005 ride completed</text>
<text x="220" y="216" class="t-mono t-10">100006 ride cancelled</text>
<text x="220" y="236" class="t-mono t-10">…</text>
<text x="220" y="256" class="t-mono t-10">100009 food picked_up</text>
<rect x="400" y="150" width="170" height="150" rx="6" class="f-surface s-line"/>
<text x="410" y="172" class="t-small">trang lá</text>
<text x="410" y="196" class="t-mono t-10">100010 ride in_transit</text>
<text x="410" y="216" class="t-mono t-10">…</text>
<text x="410" y="236" class="t-mono t-10">100014 ride requested</text>
<text x="410" y="270" class="t-small t-pk">chuyến mới ghi vào đây →</text>
<text x="20" y="326" class="t-small">Code tăng dần: mọi chuyến mới nằm cuối trang lá cuối cùng.</text>
<text x="640" y="22" class="t-b">pk_trips · NONCLUSTERED</text>
<rect x="660" y="40" width="210" height="52" rx="6" class="f-surface s-line"/>
<text x="765" y="62" text-anchor="middle" class="t-small">trang gốc</text>
<text x="765" y="82" text-anchor="middle" class="t-mono t-10">sắp theo id</text>
<line x1="765" y1="92" x2="765" y2="150" class="s-line"/>
<rect x="620" y="150" width="290" height="110" rx="6" class="f-surface s-line"/>
<text x="630" y="172" class="t-small">trang lá: id → row locator (code)</text>
<text x="630" y="196" class="t-mono t-10">7000…100003 → 100003</text>
<text x="630" y="216" class="t-mono t-10">7000…100004 → 100004</text>
<text x="630" y="236" class="t-mono t-10">…</text>
<path d="M620 216 C 590 216, 600 120, 405 66" class="f-none s-warn dash"/>
<text x="600" y="330" class="t-small t-warn">Tìm theo id: đọc pk_trips lấy code,</text>
<text x="600" y="350" class="t-small t-warn">rồi đi xuống clustered index để lấy dòng.</text>
</svg>
</div>
<figcaption><b>Hình 1 · Bảng trips là một B-tree sắp theo code.</b> Khoá chính id nằm trong một B-tree riêng; tầng lá của nó giữ code làm row locator. Số trang và số tầng trên hình chỉ để minh hoạ: <code>sys.dm_db_index_physical_stats</code> cho thấy mọi chuyến của lab thực tế nằm gọn trong một trang lá ở mỗi index.</figcaption>
</figure>

## 5. Vì sao trips có khoá chính NONCLUSTERED và clustered trên code

Cuối `01-schema.sql`, lab liệt kê index của `trips` (kết quả trong `OUTPUT.txt`):

```text
index_name|type_desc|la_pk|duy_nhat|cot_khoa
----------|---------|-----|--------|--------
uq_trips_code|CLUSTERED|0|1|code
pk_trips|NONCLUSTERED|1|1|id
ix_trips_city_requested|NONCLUSTERED|0|0|city_id, requested_at
ix_trips_customer_requested|NONCLUSTERED|0|0|customer_id, requested_at
ix_trips_driver_id|NONCLUSTERED|0|0|driver_id
```

Khoá chính là `id` vì ứng dụng và các khoá ngoại trỏ tới nó, nhưng bảng được **xếp theo `code`**. Hai dòng DDL tạo ra kết quả này:

```sql
CONSTRAINT pk_trips PRIMARY KEY NONCLUSTERED (id),
CONSTRAINT uq_trips_code UNIQUE CLUSTERED (code),
```

Lý do nằm ở cách giá trị mới đi vào B-tree. `id` của chuyến sinh bằng `NEWID()`, một GUID ngẫu nhiên: chuyến mới có thể rơi vào **bất kỳ** trang lá nào. Khi trang đó đầy, SQL Server phải **tách trang** (page split): chuyển khoảng một nửa số dòng sang trang mới. Còn `code` là `bigint IDENTITY` tăng dần, nên chuyến mới luôn nằm cuối trang lá cuối cùng.

Để thấy khác biệt, database nháp `Scratch_ms003` có ba bảng cùng cột đệm `char(200)`, mỗi bảng nhận 20.000 câu INSERT từng dòng một (giống ứng dụng ghi từng chuyến), chỉ khác clustered key:

```sql
CREATE TABLE dbo.ck_newid    (id uniqueidentifier NOT NULL DEFAULT NEWID()           PRIMARY KEY CLUSTERED, filler char(200) NOT NULL DEFAULT 'x');
CREATE TABLE dbo.ck_seqid    (id uniqueidentifier NOT NULL DEFAULT NEWSEQUENTIALID() PRIMARY KEY CLUSTERED, filler char(200) NOT NULL DEFAULT 'x');
CREATE TABLE dbo.ck_identity (code bigint IDENTITY(100000,1) NOT NULL               PRIMARY KEY CLUSTERED, filler char(200) NOT NULL DEFAULT 'x');
-- vòng lặp WHILE 20.000 lần, mỗi lần INSERT … DEFAULT VALUES vào cả ba bảng

SELECT OBJECT_NAME(s.object_id) AS bang, s.page_count AS so_page,
       CAST(s.avg_page_space_used_in_percent AS decimal(5,1)) AS page_day_pct,
       CAST(s.avg_fragmentation_in_percent AS decimal(5,1)) AS phan_manh_pct
FROM sys.dm_db_index_physical_stats(DB_ID(), NULL, NULL, NULL, 'DETAILED') AS s
WHERE s.index_level = 0 AND s.index_id = 1 AND OBJECT_NAME(s.object_id) LIKE 'ck[_]%'
ORDER BY bang;
```

```text
bang|so_page|page_day_pct|phan_manh_pct
----|-------|------------|-------------
ck_identity|541|99.1|.4
ck_newid|839|66.2|99.2
ck_seqid|572|97.2|.5
```

Số liệu từ một lần chạy trên container của lab (SQL Server 2022 CU28, Docker Desktop trên Windows 11, giới hạn 2 GB RAM), chỉ đọc tầng lá của clustered index. Cùng 20.000 dòng, bảng clustered theo `NEWID()` cần nhiều hơn khoảng 55% số trang, mỗi trang chỉ đầy hai phần ba, và thứ tự trang trên đĩa gần như đảo lộn hoàn toàn (`phan_manh_pct` là tỷ lệ trang lá nằm sai thứ tự). `NEWSEQUENTIALID()` tăng dần nên gần bằng IDENTITY; phần chênh 31 trang đến từ khoá 16 byte so với 8 byte. Ảnh hưởng tới tốc độ đọc ghi sẽ được đo ở Phần 3; ở đây chỉ kết luận về dung lượng và thứ tự trang.

Tiêu chí chọn clustered key rút ra từ đó:

1. **Hẹp**: clustered key được chép vào tầng lá của **mọi** nonclustered index làm row locator. `trips` có bốn nonclustered index, mỗi index mang thêm 8 byte `code` thay vì 16 byte GUID.
2. **Không trùng**: nếu clustered index không UNIQUE, SQL Server phải tự thêm một giá trị 4 byte vào các dòng trùng khoá để phân biệt. `uq_trips_code` khai báo UNIQUE để tránh việc đó.
3. **Không đổi**: đổi clustered key là chuyển dòng sang vị trí khác và sửa mọi nonclustered index. Mã chuyến không bao giờ đổi.
4. **Tăng dần**: dòng mới nằm cuối, không tách trang giữa bảng.

Còn khoá chính `pk_trips` trên `id` thì sao? Index nonclustered này vẫn nhận GUID ngẫu nhiên, nhưng mỗi dòng của nó chỉ mang `id` và `code` (24 byte dữ liệu) chứ không phải cả dòng chuyến; tách trang ở đây rẻ hơn nhiều so với tách trang của chính bảng.

**Bảng nhỏ thì clustered theo `id`.** Truy vấn `sys.indexes` với `index_id <= 1` trên database `ToiNoi` cho thấy năm bảng còn lại đều clustered theo khoá chính:

```text
bang|index_name|type_desc|la_pk
----|----------|---------|-----
cities|pk_cities|CLUSTERED|1
customers|pk_customers|CLUSTERED|1
drivers|pk_drivers|CLUSTERED|1
merchants|pk_merchants|CLUSTERED|1
trip_events|pk_trip_events|CLUSTERED|1
trips|uq_trips_code|CLUSTERED|0
```

`cities` có khoá `smallint`; `trip_events` có khoá `bigint IDENTITY`; `customers`, `drivers`, `merchants` có khoá `uniqueidentifier` với DEFAULT `NEWSEQUENTIALID()`, giá trị tăng dần theo thứ tự chèn nên không gây tách trang như `NEWID()`. Đây là các bảng danh mục ít ghi, nên một khoá chính kiêm clustered key là đủ đơn giản. Không bảng nào của lab là heap.

## 6. Hai cách gọi một chuyến: `id` và `code`

| Cột | Ràng buộc | Ví dụ | Ai dùng |
|---|---|---|---|
| `id` | `pk_trips` (NONCLUSTERED) | `70000000-0000-4000-8000-000000100004` | ứng dụng, bảng khác qua khoá ngoại |
| `code` | `uq_trips_code` (CLUSTERED) | `100004` | khách, tổng đài, tài xế: đọc được qua điện thoại |

Tổng đài chỉ cần hỏi mã chuyến rồi «đi theo khoá». Ba truy vấn dưới đây chạy trên database `ToiNoi` sau khi chạy lab; mỗi truy vấn lọc bằng một khoá nên trả **đúng một dòng**:

```sql
SELECT code, customer_id, driver_id FROM toinoi.trips WHERE code = 100004;
SELECT phone, full_name FROM toinoi.customers WHERE id = 'c0000000-0000-4000-8000-000000000005';
SELECT code, full_name, city_id FROM toinoi.drivers WHERE id = 'd0000000-0000-4000-8000-000000008005';
```

```text
code|customer_id|driver_id
----|-----------|---------
100004|C0000000-0000-4000-8000-000000000005|D0000000-0000-4000-8000-000000008005
phone|full_name
-----|---------
0900000005|Võ Minh Đức
code|full_name|city_id
----|---------|-------
8005|Lý Thị Oanh|3
```

Chuyến 100004 là của khách Võ Minh Đức, tài xế Lý Thị Oanh (mã 8005) ở Đà Nẵng. SQL Server in `uniqueidentifier` bằng chữ in hoa, nhưng so sánh không phân biệt hoa thường vì giá trị là 16 byte chứ không phải chuỗi. Trong DBStudio, đặt con trỏ vào một câu rồi bấm **Ctrl+Enter** để chạy batch chứa nó; ba câu trong cùng một batch cho ra ba result set riêng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">GUID trong fixture được viết cho dễ đọc</span>
<p>Fixture ghi sẵn GUID có đuôi trùng mã (<code>…100004</code> ↔ 100004) chỉ để kết quả ổn định. Trong hệ thống thật, <code>id</code> sinh bằng <code>NEWID()</code> và không liên quan tới <code>code</code>. <code>code</code> cũng không liên tục: chuyến tạo ở <code>04-lifecycle.sql</code> nhận mã 100032 chứ không phải 100015 (MS-005 giải thích), nên mã dùng để gọi tên chuyến, không dùng để đếm.</p>
</div>

## 7. Foreign key: nối chuyến với khách, tài xế, quán

Để database bảo đảm `trips.customer_id` **trỏ tới một khách có thật**, lab khai báo **foreign key** (khoá ngoại) `fk_trips_customer FOREIGN KEY (customer_id) REFERENCES toinoi.customers (id)`. `trips` là bảng con, `customers` là bảng cha. Khoá ngoại phải trỏ tới khoá chính hoặc cột có ràng buộc UNIQUE của bảng cha.

Khoá ngoại chặn cả hai chiều. Ba câu trong `03-fixture-invalid.sql` thêm chuyến cho khách không tồn tại, thêm tài xế ở thành phố không tồn tại, và xoá khách đang có chuyến:

```text
PASS F1 chuyến của khách không tồn tại — lỗi 547: The INSERT statement conflicted with the FOREIGN KEY constraint "fk_trips_customer". The conflict occurred in database "ToiNoi", table "toinoi.customers", column 'id'.
PASS F2 tài xế ở thành phố không tồn tại — lỗi 547: The INSERT statement conflicted with the FOREIGN KEY constraint "fk_drivers_city". The conflict occurred in database "ToiNoi", table "toinoi.cities", column 'id'.
PASS F3 xoá khách đang có chuyến — lỗi 547: The DELETE statement conflicted with the REFERENCE constraint "fk_trips_customer". The conflict occurred in database "ToiNoi", table "toinoi.trips", column 'customer_id'.
```

Cả ba mang số hiệu 547. Câu thứ ba bị chặn do hành vi mặc định `NO ACTION`: từ chối xoá dòng cha khi còn dòng con. `ON DELETE CASCADE` sẽ xoá luôn dòng con, và với dữ liệu chuyến và tiền thì đó thường là điều ta **không** muốn.

**NULL trong cột khoá ngoại.** `driver_id` cho phép `NULL`; dòng có `NULL` ở cột tham chiếu không bị khoá ngoại kiểm tra. Fixture có ba chuyến như vậy (100006 huỷ khi chưa ai nhận, 100013 `searching`, 100014 `requested`). Quy tắc «từ `accepted` trở đi phải có tài xế» vì thế không do khoá ngoại quyết định; lab giao nó cho CHECK `ck_trips_assigned_has_driver` (MS-005).

**Khoá ngoại chỉ kiểm «có tồn tại», không kiểm «có hợp lý».** Gán tài xế ở Hà Nội cho một chuyến ở TP.HCM vẫn thoả khoá ngoại, vì tài xế có thật. Đó là quy tắc nhiều bảng mà khoá ngoại đơn không diễn đạt được; lab bắt nó bằng kiểm tra K1 của `07-checks.sql`.

**SQL Server không tự tạo index cho cột khoá ngoại.** Khoá chính và UNIQUE có index đi kèm, khoá ngoại thì không. `07-checks.sql` liệt kê khoá ngoại nào có index bắt đầu bằng cột tham chiếu (kết quả trong `OUTPUT.txt`):

```text
khoa_ngoai|cot|co_index
----------|---|--------
fk_drivers_city|city_id|KHÔNG
fk_merchants_city|city_id|KHÔNG
fk_trip_events_trip|trip_id|có
fk_trips_city|city_id|có
fk_trips_customer|customer_id|có
fk_trips_driver|driver_id|có
fk_trips_merchant|merchant_id|KHÔNG
```

Bốn chữ «có» đến từ các index lab chủ động tạo (`ix_trips_driver_id`, `ix_trip_events_trip`…). Không có index, mỗi lần xoá một dòng cha (một khách, một quán) SQL Server phải quét bảng con để tìm dòng tham chiếu. Ba khoá ngoại còn lại chưa có index là quyết định có chủ đích của P01: Phần 3 sẽ đo lợi ích và cái giá của index thay vì thêm theo phản xạ.

## 8. Sơ đồ quan hệ khách–chuyến–tài xế

Gom các khoá ngoại lại, ta có sơ đồ của P01. Mỗi đường nối đi từ bảng cha (đầu «1») tới bảng con (đầu «n»); nét đứt là khoá ngoại được phép `NULL`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 520" role="img" aria-label="Sơ đồ quan hệ lab P01 trên SQL Server: cities một-nhiều với trips, drivers, merchants; customers một-nhiều với trips; drivers và merchants nối trips bằng khoá ngoại có thể NULL; trips một-nhiều với trip_events. trips có khoá chính id nonclustered và clustered index trên code.">
<rect x="355" y="16" width="210" height="64" rx="8" class="f-surface s-line"/>
<text x="460" y="42" text-anchor="middle" class="t-mono t-b">cities</text>
<text x="460" y="64" text-anchor="middle" class="t-mono t-10 t-pk">PK id smallint (CL)</text>
<rect x="20" y="180" width="220" height="96" rx="8" class="f-surface s-line"/>
<text x="36" y="206" class="t-mono t-b">customers</text>
<text x="36" y="232" class="t-mono t-10 t-pk">PK id guid (CL)</text>
<text x="36" y="254" class="t-mono t-10">UQ phone</text>
<rect x="340" y="150" width="240" height="210" rx="8" class="f-pk-soft s-pk" stroke-width="1.8"/>
<text x="356" y="176" class="t-mono t-b t-pk">trips</text>
<text x="356" y="200" class="t-mono t-10 t-pk">PK id guid (NONCLUSTERED)</text>
<text x="356" y="220" class="t-mono t-10">UQ code bigint (CLUSTERED)</text>
<text x="356" y="244" class="t-mono t-10">FK customer_id</text>
<text x="356" y="264" class="t-mono t-10">FK driver_id (NULL được)</text>
<text x="356" y="284" class="t-mono t-10">FK merchant_id (NULL được)</text>
<text x="356" y="304" class="t-mono t-10">FK city_id</text>
<text x="356" y="330" class="t-small">kind, status, fare,</text>
<text x="356" y="348" class="t-small">requested_at, accepted_at…</text>
<rect x="680" y="150" width="220" height="96" rx="8" class="f-surface s-line"/>
<text x="696" y="176" class="t-mono t-b">drivers</text>
<text x="696" y="200" class="t-mono t-10 t-pk">PK id guid (CL)</text>
<text x="696" y="220" class="t-mono t-10">UQ code · FK city_id</text>
<rect x="680" y="284" width="220" height="76" rx="8" class="f-surface s-line"/>
<text x="696" y="310" class="t-mono t-b">merchants</text>
<text x="696" y="334" class="t-mono t-10 t-pk">PK id guid (CL)</text>
<text x="696" y="352" class="t-mono t-10">FK city_id</text>
<rect x="340" y="420" width="240" height="80" rx="8" class="f-surface s-line"/>
<text x="356" y="446" class="t-mono t-b">trip_events</text>
<text x="356" y="470" class="t-mono t-10 t-pk">PK id bigint (CL)</text>
<text x="356" y="488" class="t-mono t-10">FK trip_id</text>
<line x1="460" y1="80" x2="460" y2="150" class="s-line" stroke-width="1.6"/>
<text x="468" y="98" class="t-small">1</text>
<text x="468" y="144" class="t-small">n</text>
<path d="M565 64 H790 V150" class="f-none s-line" stroke-width="1.6"/>
<text x="574" y="58" class="t-small">1</text>
<text x="798" y="144" class="t-small">n</text>
<path d="M565 40 H910 V322 H900" class="f-none s-line" stroke-width="1.6"/>
<text x="574" y="34" class="t-small">1</text>
<text x="886" y="316" class="t-small">n</text>
<line x1="240" y1="228" x2="340" y2="228" class="s-line" stroke-width="1.6"/>
<text x="248" y="220" class="t-small">1</text>
<text x="326" y="220" class="t-small">n</text>
<line x1="680" y1="198" x2="580" y2="198" class="s-warn dash" stroke-width="1.6"/>
<text x="642" y="190" class="t-small t-warn">0..1</text>
<text x="588" y="190" class="t-small">n</text>
<line x1="680" y1="322" x2="580" y2="322" class="s-warn dash" stroke-width="1.6"/>
<text x="642" y="314" class="t-small t-warn">0..1</text>
<text x="588" y="314" class="t-small">n</text>
<line x1="460" y1="360" x2="460" y2="420" class="s-line" stroke-width="1.6"/>
<text x="468" y="378" class="t-small">1</text>
<text x="468" y="414" class="t-small">n</text>
</svg>
</div>
<figcaption><b>Hình 2 · Quan hệ giữa sáu bảng của lab P01.</b> CL: clustered index. Khoá ngoại luôn nằm ở phía «n». Nét đứt: chuyến có thể chưa có tài xế, và chỉ đơn đồ ăn mới có quán.</figcaption>
</figure>

Đọc thành câu: một khách có nhiều chuyến (khách `0900000001` có ba chuyến trong fixture), một tài xế có nhiều chuyến theo thời gian, một chuyến có nhiều sự kiện. Hai điểm hay bị vẽ sai:

- Khoá ngoại luôn ở phía «n»: không có cột `trip_ids` nào trong `customers`.
- «Tài xế – chuyến» là một–nhiều **theo thời gian**. Quy tắc «tại một thời điểm, tài xế chỉ chạy một chuyến» không nằm trong sơ đồ; lab kiểm tra bằng K3 của `07-checks.sql`, còn tranh chấp thật là chủ đề Phần 4.

Không cần tự vẽ: tab **ERD** của DBStudio vẽ các bảng và quan hệ khoá ngoại từ chính database `ToiNoi`; chọn một bảng để xem bảng **DDL** của nó, và xuất sơ đồ ra PNG, SVG hoặc PDF để đưa vào tài liệu thiết kế.

## 9. So với PostgreSQL

Nếu bạn đã đọc [PG-003](/articles/postgresql-bang-dong-cot-primary-foreign-key), phần khoá và khoá ngoại gần như giống hệt. Khác biệt thật nằm ở cách lưu bảng:

| Chủ đề | PostgreSQL | SQL Server |
|---|---|---|
| Bảng được lưu thế nào | Luôn là heap; mọi index (kể cả khoá chính) là cấu trúc phụ | Clustered index *là* bảng; không có clustered index thì là heap |
| Khoá chính tạo gì | Unique index B-tree riêng | Mặc định là clustered index, trừ khi ghi `NONCLUSTERED` |
| Index phụ trỏ về dòng bằng | Vị trí vật lý của dòng | Clustered key (hoặc RID nếu là heap) |
| Index cho cột khoá ngoại | Không tự tạo | Không tự tạo |
| Mã lỗi trùng khoá / khoá ngoại | SQLSTATE `23505` / `23503` | Số hiệu 2627 / 547 |

Hệ quả cho thiết kế: trên PostgreSQL, khoá chính `uuid` ngẫu nhiên chỉ làm rối index khoá chính; trên SQL Server, nếu để mặc định, nó làm rối **chính bảng**. Vì vậy lab SQL Server tách khoá chính và clustered key cho `trips`, còn lab PostgreSQL thì không cần.

## 10. Lỗi thường gặp và đánh đổi

- **Nhận diện bằng thuộc tính không phải khoá** (giờ đặt, họ tên, «dòng thứ mấy»): hai chuyến 12:00 của fixture là ví dụ thật.
- **Lấy natural key dễ đổi làm khoá chính**: số điện thoại, biển số, email nên là `UNIQUE`.
- **Để `PRIMARY KEY` trên GUID `NEWID()` mặc định thành clustered**: bảng lớn dần với trang đầy hai phần ba và thứ tự trang đảo lộn, như bảng `ck_newid` ở mục 5.
- **Clustered key rộng hoặc hay đổi**: chi phí nhân lên ở mọi nonclustered index.
- **Tưởng khoá ngoại tự có index**: xoá dòng cha phải quét bảng con.
- **Dùng `ON DELETE CASCADE` cho dữ liệu có giá trị lịch sử**: xoá một khách mà mất mọi chuyến là sự cố.

Đánh đổi: `uniqueidentifier` cho phép ứng dụng tự sinh id trước khi ghi và không lộ số lượng chuyến, nhưng tốn 16 byte và khó đọc, nên cần thêm `code`. Tách khoá chính khỏi clustered key cho `trips` thêm một index phải bảo trì (`pk_trips`) và một bước tra cứu khi tìm theo `id` (Hình 1); đổi lại, bảng lớn nhất được ghi tuần tự.

## 11. Bài tập

**Bài 1.** Tới Nơi thêm bảng `ratings`: khách chấm 1–5 sao sau chuyến, mỗi chuyến **tối đa một** đánh giá, khoảng một triệu đánh giá mỗi ngày. Chọn khoá chính và clustered key. Làm sao chặn hai đánh giá cho cùng chuyến?

<details><summary><b>Lời giải</b></summary><p>Một dòng là một đánh giá của một chuyến. <code>trip_id uniqueidentifier NOT NULL</code> có khoá ngoại tới <code>toinoi.trips (id)</code> và <code>UNIQUE (trip_id)</code> để chặn đánh giá thứ hai. Nếu lấy <code>trip_id</code> làm khoá chính mặc định, bảng sẽ clustered theo GUID <code>NEWID()</code> của chuyến, tức là ghi ngẫu nhiên vào giữa bảng như <code>ck_newid</code>. Phương án hợp với SQL Server: <code>id bigint IDENTITY</code> làm khoá chính clustered (hoặc giữ <code>trip_id</code> làm khoá chính <code>NONCLUSTERED</code> và clustered trên <code>id</code>, giống <code>trips</code>), cộng <code>UNIQUE (trip_id)</code>. Không thêm <code>customer_id</code>: suy ra được qua chuyến, thêm vào là lặp dữ liệu (MS-008).</p></details>

**Bài 2.** Không chạy lệnh, hãy đoán số hiệu lỗi và tên ràng buộc trên fixture: (a) thêm tài xế mới mã 8002; (b) đặt <code>merchant_id</code> của chuyến 100002 thành GUID quán không tồn tại; (c) xoá thành phố có <code>id</code> 3.

<details><summary><b>Lời giải</b></summary><p>(a) 2627, <code>uq_drivers_code</code>. (b) 547, <code>fk_trips_merchant</code> (thông báo «The UPDATE statement conflicted with the FOREIGN KEY constraint»). (c) 547 với thông báo «REFERENCE constraint»: Đà Nẵng còn tài xế, quán và chuyến trỏ tới, nên câu bị một trong các khoá ngoại <code>fk_drivers_city</code>, <code>fk_merchants_city</code>, <code>fk_trips_city</code> chặn. Đừng viết code dựa vào việc khoá ngoại nào được báo trước.</p></details>

**Bài 3.** Trong DBStudio, mở **ToiNoi → toinoi → Tables → trips → Indexes**. Có bao nhiêu index? Nếu xoá ràng buộc `uq_trips_code`, bảng `trips` trở thành gì, và `pk_trips` trỏ về dòng bằng gì?

<details><summary><b>Lời giải</b></summary><p>Năm index, đúng như kết quả <code>sys.indexes</code> ở mục 5. Xoá <code>uq_trips_code</code> là xoá clustered index duy nhất, nên <code>trips</code> trở thành heap (<code>index_id</code> 0) và mọi nonclustered index, gồm <code>pk_trips</code>, phải chuyển row locator từ <code>code</code> sang RID. SQL Server phải dựng lại toàn bộ các index đó, nên trên bảng lớn đây là thao tác nặng, cần kế hoạch như một migration (MS-009).</p></details>

## 12. Đọc tiếp

- [MS-004: Kiểu dữ liệu cho tiền, thời gian, chuỗi tiếng Việt và định danh](/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode): vì sao `id` là `uniqueidentifier`, `fare` là `int`, thời điểm là `datetimeoffset(3)`.
- [MS-005: CREATE TABLE, ràng buộc và IDENTITY](/articles/mssql-create-table-constraints): viết các khoá thành DDL có tên.
- Microsoft Learn (SQL Server 2022): [Clustered and nonclustered indexes described](https://learn.microsoft.com/sql/relational-databases/indexes/clustered-and-nonclustered-indexes-described), [Heaps (tables without clustered indexes)](https://learn.microsoft.com/sql/relational-databases/indexes/heaps-tables-without-clustered-indexes), [Index architecture and design guide](https://learn.microsoft.com/sql/relational-databases/sql-server-index-design-guide), [Primary and foreign key constraints](https://learn.microsoft.com/sql/relational-databases/tables/primary-and-foreign-key-constraints), [sys.indexes](https://learn.microsoft.com/sql/relational-databases/system-catalog-views/sys-indexes-transact-sql).

Bài tiếp theo: [MS-004 · Kiểu dữ liệu cho tiền, thời gian, chuỗi tiếng Việt và định danh](/articles/mssql-kieu-du-lieu-tien-thoi-gian-unicode)
