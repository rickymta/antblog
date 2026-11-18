---
slug: postgresql-jsonb-quan-he
title: "JSONB hay cột quan hệ? Bản tin GPS của Tới Nơi và những lỗi không ai báo"
summary: "Mỗi đời điện thoại tài xế gửi kèm một bộ thuộc tính GPS khác nhau. Lab PostgreSQL 18.1 cho thấy @> đếm đúng 147 điểm, gõ sai tên khoá trả 0 dòng mà không lỗi, ép kiểu hỏng chỉ nổ lúc đọc (22P02), CHECK chặn được mảng ở gốc (23514), GIN jsonb_path_ops dùng được nhưng planner vẫn chọn Seq Scan trên dữ liệu nhỏ, và cột generated đưa khoá nóng về thế giới có kiểu."
category: postgresql
level: beginner
tags: [postgresql, pg-053, jsonb, gin, generated-column, check-constraint, schema-design, toi-noi]
language: vi
---

Đội ứng dụng tài xế của Tới Nơi vừa phát hành bản mới. Điện thoại đời cũ chỉ gửi mã tài xế, thời điểm đo và toạ độ. Một dòng máy mới gửi thêm `accuracy_m` (bán kính sai số tính bằng mét), một dòng khác gửi `battery_pct`, còn bản thử nghiệm của đội bản đồ thì gửi cả `speed_kmh` lẫn `heading`. Hồ sơ dự án ước lúc cao điểm có khoảng 6.250 điểm GPS mới mỗi giây ([PG-028](/articles/postgresql-gin-gist-brin-lua-chon)), nên một quyết định mô hình sai sẽ nhân lên rất nhanh.

Trong buổi họp có hai phe. Phe một: «thêm cột cho mọi trường, có kiểu, có ràng buộc». Phe hai: «đẩy hết vào một cột JSONB, khỏi migration mỗi lần thiết bị đổi». Ba tuần sau, một biểu đồ «tỉ lệ điểm có sai số 5 m» hiển thị 0 % suốt mấy ngày mà không ai thấy cảnh báo nào — chỉ vì ai đó viết `accurracy_m` thừa một chữ `r`.

Bài này không chọn phe. Nó dùng lab PostgreSQL 18.1 để tách rõ: những gì **phải** nằm ở cột quan hệ, những gì JSONB làm tốt, lỗi nào JSONB **không** báo cho bạn, và khi nào nên «nâng» một khoá JSONB thành cột thật.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-053 · Phần 6/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-replication-primary-standby">PG-052 · Primary, standby và replication lag</a>; nên ôn <a href="/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid">PG-004 · Kiểu dữ liệu</a>, <a href="/articles/postgresql-create-table-constraints">PG-005 · CREATE TABLE và ràng buộc</a>, <a href="/articles/postgresql-mo-hinh-quan-he-chuan-hoa">PG-008 · Chuẩn hoá</a> và <a href="/articles/postgresql-gin-gist-brin-lua-chon">PG-028 · GIN, GiST và BRIN</a>. Lab: <code>examples/toi-noi-postgresql/p06</code>, chạy <code>npm run p06</code> tại <code>examples/toi-noi-postgresql</code> với PostgreSQL 18 và biến <code>PG17_BIN</code> trỏ tới thư mục bin của PostgreSQL 17 (xem README của lab). Kết quả trong <code>p06/OUTPUT.txt</code>. Sau bài này, bạn chọn được cột quan hệ hay JSONB cho từng trường dựa trên ràng buộc và truy vấn, biết chỗ JSONB im lặng khi dữ liệu sai, và biết cách đưa một khoá hay lọc về cột có kiểu.</p>
</div>

## 1. Dữ liệu và cách chạy

```powershell
cd examples/toi-noi-postgresql
$env:PG17_BIN = 'D:\pg17\pgsql\bin'   # chỉ cần cho PG-059, nhưng runner kiểm tra từ đầu
npm run p06
```

Runner `p06/run.mjs` dựng các cluster PostgreSQL 18.1 tạm, chỉ nghe localhost, chạy mười bài của Phần 6 theo thứ tự cố định rồi xoá sạch. Session của lab đặt `timezone = 'Asia/Ho_Chi_Minh'`, nên giá trị `timestamptz` in ra mang đuôi `+07`.

`p06/fixture.sql` tạo bảng `gps` — cũng là bảng của [PG-051](/articles/postgresql-partition-co-ban):

```sql
CREATE TABLE gps (
  driver_id int NOT NULL,
  seen_at timestamptz NOT NULL,
  attrs jsonb NOT NULL CHECK (jsonb_typeof(attrs) = 'object'),
  PRIMARY KEY (driver_id, seen_at)
) PARTITION BY RANGE (seen_at);
CREATE TABLE gps_oct PARTITION OF gps FOR VALUES FROM ('2026-10-01 UTC') TO ('2026-11-01 UTC');
CREATE TABLE gps_nov PARTITION OF gps FOR VALUES FROM ('2026-11-01 UTC') TO ('2026-12-01 UTC');
INSERT INTO gps
SELECT 1 + n % 20, '2026-10-01 UTC'::timestamptz + n * interval '1 hour',
       jsonb_build_object('source', 'phone', 'accuracy_m', 5 + n % 10)
FROM generate_series(0, 1463) n;
```

Tức 1.464 điểm theo giờ của 20 tài xế, 01/10–30/11/2026 (UTC), chia hai partition tháng: `gps_oct` 744 dòng, `gps_nov` 720 dòng. Mỗi `attrs` có hai khoá: `source` luôn là chuỗi `"phone"`, `accuracy_m` là số nguyên chạy vòng 5…14. Vì `n % 10 = 0` xảy ra với n = 0, 10, …, 1460, đúng **147** điểm có `accuracy_m = 5` — con số bạn sẽ gặp nhiều lần dưới đây.

Dữ liệu này cố ý nhỏ và đồng đều. README của lab ghi rõ giới hạn: plan trên dữ liệu nhỏ, cache ấm, một máy; ở dữ liệu lớn planner có thể chọn khác. Các khối kết quả dưới đây chép từ `OUTPUT.txt`, đã bỏ hai khoảng trắng thụt lề mà runner thêm vào và cắt bớt dòng gạch ngang dài của tiêu đề plan.

## 2. JSONB là gì, và nó khác `json` ở đâu

PostgreSQL có hai kiểu cho dữ liệu JSON. Theo tài liệu PostgreSQL 18 (mục *JSON Types*):

- **`json`** lưu nguyên văn chuỗi bạn gửi vào: giữ khoảng trắng, giữ thứ tự khoá, giữ cả khoá trùng. Mỗi lần xử lý phải phân tích (parse) lại.
- **`jsonb`** («JSON binary») phân rã chuỗi thành cấu trúc nhị phân lúc ghi. Ghi chậm hơn một chút vì phải chuyển đổi, nhưng đọc và xử lý nhanh hơn vì không parse lại, và **hỗ trợ index**. Đổi lại, nó không giữ khoảng trắng, không giữ thứ tự khoá, và khi trùng khoá chỉ giữ giá trị **cuối cùng**. Số JSON được lưu bằng kiểu `numeric` của PostgreSQL.

Tài liệu khuyên phần lớn ứng dụng nên dùng `jsonb`, trừ khi cần giữ nguyên văn (ví dụ lưu payload gốc để đối chứng pháp lý). Trong bài, «JSONB» nghĩa là kiểu `jsonb`.

Điều quan trọng hơn là thứ JSONB **không** có. Một cột `jsonb` chấp nhận mọi tài liệu JSON hợp lệ: object, mảng, chuỗi, số, `true`, `null`. PostgreSQL không biết `accuracy_m` phải là số, phải nằm trong khoảng 0–100, hay phải có mặt. Đó là ý nghĩa thực tế của «schema-less» (không lược đồ): không phải là không có lược đồ, mà là **lược đồ nằm trong code của từng người đọc**, và database không kiểm nó giúp bạn.

## 3. Cột quan hệ giữ những gì không được phép sai

Nhìn lại định nghĩa bảng. Ba trường quyết định danh tính và thứ tự của một điểm GPS — `driver_id`, `seen_at` — nằm ở cột có kiểu, `NOT NULL`, và là khoá chính. Nhờ vậy:

- Hai điểm cùng tài xế cùng thời điểm bị từ chối (khoá chính, [PG-005](/articles/postgresql-create-table-constraints)).
- `seen_at` là `timestamptz`, không phải chuỗi tuỳ định dạng — so sánh khoảng thời gian, partition theo tháng và pruning ([PG-054](/articles/postgresql-partition-pruning-retention)) đều dựa vào điều đó.
- Nếu sau này cần khoá ngoại tới bảng `drivers`, chỉ cột quan hệ mới làm được: PostgreSQL không có khoá ngoại trỏ từ một giá trị nằm trong JSONB.

Còn `attrs` thì sao? Lab đặt đúng một ràng buộc: `CHECK (jsonb_typeof(attrs) = 'object')`. `jsonb_typeof` trả về tên kiểu JSON ở gốc tài liệu (`object`, `array`, `string`, `number`, `boolean`, `null`). Runner thử chèn một mảng:

```sql
INSERT INTO gps VALUES (1, '2026-11-15 00:30 UTC', '[5]');
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-jsonb-quan-he/dbstudio-01.webp" alt="DBStudio chạy truy vấn của bước 053 trong p06/run.mjs và hiển thị thông báo lỗi" width="1066" height="500" loading="lazy"></div><figcaption>DBStudio · bước 053 của <code>p06/run.mjs</code> · lỗi</figcaption></figure>

SQLSTATE **`23514`** là `check_violation`; ứng dụng nên bắt theo mã này, không theo câu chữ. Lỗi báo trên **`gps_nov`**, không phải `gps`: dòng được định tuyến vào partition tháng 11 trước, rồi ràng buộc kế thừa từ bảng cha được kiểm tại đó, vẫn mang tên `gps_attrs_check` mà PostgreSQL tự đặt ở bảng cha. (`00:30 UTC` được in thành `07:30:00+07` theo múi giờ session — cùng một thời điểm, xem [PG-016](/articles/postgresql-date-time-timezone).)

Ràng buộc này chỉ bảo đảm «gốc là object». Nó **không** nói gì về `accuracy_m`. Một object `{"accuracy_m": "rất tốt"}` vẫn lọt qua.

## 4. Đọc JSONB: `->`, `->>`, `@>` và `?`

Bốn toán tử dùng nhiều nhất (tài liệu PostgreSQL 18, *JSON Functions and Operators*):

| Toán tử | Nghĩa | Kiểu kết quả |
|---|---|---|
| `attrs -> 'accuracy_m'` | lấy giá trị của khoá | `jsonb` |
| `attrs ->> 'accuracy_m'` | lấy giá trị của khoá dưới dạng văn bản | `text` |
| `attrs @> '{"accuracy_m":5}'` | **chứa** (containment): vế trái có chứa cặp khoá–giá trị này không | `boolean` |
| `attrs ? 'accuracy_m'` | khoá có tồn tại ở cấp cao nhất không | `boolean` |

Lab kiểm kiểu trả về của hai toán tử đầu:

```sql
SELECT pg_typeof(attrs->>'accuracy_m') || ' / ' || pg_typeof(attrs->'accuracy_m') FROM gps LIMIT 1;
```

```text
PASS 053 ->> returns text, -> returns jsonb: text / jsonb
```

Sự khác biệt này quyết định phép so sánh. `attrs->>'accuracy_m'` là chuỗi `'5'`; so sánh với `'10'` theo thứ tự chuỗi thì `'10' < '5'`. Muốn so sánh số, bạn phải ép kiểu: `(attrs->>'accuracy_m')::int`. `->` giữ giá trị ở dạng `jsonb` — hữu ích khi đi sâu tiếp (`attrs -> 'gps' -> 'lat'`) hoặc khi cần biết kiểu JSON thật bằng `jsonb_typeof`.

Containment là cách «đúng chất JSONB» để lọc bằng:

```sql
SELECT count(*) FROM gps WHERE attrs @> '{"accuracy_m":5}';
```

```text
PASS 053 attrs @> accuracy 5: 147
```

Đúng 147 như phép tính ở mục 1. Containment so sánh theo **kiểu JSON**: số `5` và chuỗi `"5"` là hai giá trị khác nhau, nên `attrs @> '{"accuracy_m":"5"}'` sẽ không khớp các điểm trong fixture. Lab **không chạy** biến thể chuỗi; bài tập 1 yêu cầu bạn tự kiểm.

Một chi tiết hay bị bỏ qua: `->>` trả SQL `NULL` cả khi khoá **không có** lẫn khi khoá có giá trị JSON `null`. Hai tình huống đó có nghĩa nghiệp vụ khác nhau — «máy này không đo sai số» và «máy có đo nhưng lần này không có kết quả». Muốn tách chúng, dùng `attrs ? 'accuracy_m'` (có khoá hay không) kết hợp `jsonb_typeof(attrs -> 'accuracy_m')` (giá trị là `null` hay `number`).

## 5. Schema-less: lỗi dời từ lúc ghi sang lúc đọc

Đây là phần quan trọng nhất của bài. Với cột có kiểu, dữ liệu sai bị chặn **lúc ghi**: người gửi nhận lỗi ngay, dữ liệu bẩn không vào được bảng. Với khoá trong JSONB, không có ai chặn lúc ghi. Lỗi — nếu có — xuất hiện **lúc đọc**, ở một người đọc khác, có khi vài tuần sau. Lab cho thấy hai dạng.

**Dạng 1: im lặng hoàn toàn.** Truy vấn gõ sai tên khoá:

```sql
SELECT count(*) FROM gps WHERE (attrs->>'accurracy_m')::int = 5;
```

```text
PASS 053 typo in key is silent: 0
```

Không lỗi, không cảnh báo, chỉ là **0**. `attrs->>'accurracy_m'` trả `NULL` cho mọi dòng vì khoá ấy không tồn tại; `NULL::int` vẫn là `NULL`; `NULL = 5` không đúng, nên không dòng nào qua bộ lọc ([PG-011](/articles/postgresql-null-case-coalesce)). Với một cột thật, `accurracy_m` sẽ gây lỗi `column does not exist` ngay khi chạy lần đầu. Đây chính là biểu đồ 0 % ở đầu bài. Viết bằng `@>` cũng không cứu được: `attrs @> '{"accurracy_m":5}'` cũng chỉ trả 0 dòng (suy ra từ định nghĩa containment, lab không chạy riêng biến thể này).

**Dạng 2: nổ lúc truy vấn.** Truy vấn ép kiểu một khoá chứa chuỗi:

```sql
SELECT count(*) FROM gps WHERE (attrs->>'source')::int = 1;
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-jsonb-quan-he/dbstudio-02.webp" alt="DBStudio chạy truy vấn của bước 053 trong p06/run.mjs và hiển thị thông báo lỗi" width="996" height="544" loading="lazy"></div><figcaption>DBStudio · bước 053 của <code>p06/run.mjs</code> · lỗi</figcaption></figure>

`22P02` là `invalid_text_representation`. Lab cố ý ép một khoá toàn chuỗi, nhưng tình huống thật thường là: tuần này mọi máy gửi `accuracy_m` là số, tuần sau một bản firmware gửi `"accuracy_m": "unknown"`. Không có gì chặn lúc ghi, và **mọi** truy vấn ép kiểu quét qua dòng đó bắt đầu thất bại — kể cả dashboard chẳng liên quan gì đến bản firmware kia. Người phát hiện lỗi không phải người gây ra lỗi.

Đó là cái giá thật của schema-less. Không phải «JSONB chậm», mà là **trách nhiệm kiểm dữ liệu bị dời từ database sang mọi người đọc**.

Bạn có thể kéo một phần trách nhiệm về lúc ghi bằng CHECK sâu hơn. Ví dụ **không chạy trong lab**:

```sql
-- Nếu có accuracy_m thì phải là số và nằm trong 0..1000
ALTER TABLE gps ADD CONSTRAINT gps_accuracy_is_number CHECK (
  NOT attrs ? 'accuracy_m'
  OR (jsonb_typeof(attrs -> 'accuracy_m') = 'number'
      AND (attrs ->> 'accuracy_m')::numeric BETWEEN 0 AND 1000)
);
```

Lưu ý cách viết: CHECK coi kết quả `NULL` là **đạt**, nên nếu bỏ vế `NOT attrs ? 'accuracy_m'`, dòng thiếu khoá vẫn qua vì `jsonb_typeof(NULL)` là `NULL` — đúng ý trong ví dụ này, nhưng là cái bẫy nếu bạn muốn bắt buộc có khoá. Mỗi CHECK như thế là một mảnh lược đồ bạn tự viết lại bằng tay; khi số CHECK trên `attrs` nhiều lên, đó là dấu hiệu khoá ấy nên là cột (mục 8).

## 6. Index GIN cho JSONB: dùng được không bằng nhanh hơn

Lab tạo index **GIN** (Generalized Inverted Index — index ngược, trỏ từ từng thành phần bên trong giá trị về các dòng chứa nó; xem [PG-028](/articles/postgresql-gin-gist-brin-lua-chon)) với operator class `jsonb_path_ops`:

```sql
CREATE INDEX gps_attrs_gin ON gps USING gin (attrs jsonb_path_ops);
```

Theo tài liệu PostgreSQL 18 (mục *jsonb Indexing*), JSONB có hai operator class cho GIN:

- **`jsonb_ops`** (mặc định): phục vụ `@>`, `?`, `?|`, `?&`, `@?`, `@@`. Tạo mục index cho từng khoá và từng giá trị.
- **`jsonb_path_ops`**: chỉ phục vụ `@>`, `@?`, `@@` — **không** phục vụ `?`. Mỗi mục là giá trị băm cùng đường khoá dẫn tới nó; tài liệu nói index này thường nhỏ hơn và tìm nhanh hơn `jsonb_ops` cho các phép nó hỗ trợ.

Lab chỉ tạo `jsonb_path_ops`, nên bài **không có số** so sánh kích thước hai loại ở quy mô lớn. PG-028 có một phép so trên dữ liệu quán ăn; đừng chuyển con số đó sang bảng GPS.

Vì `gps` là bảng phân vùng, `CREATE INDEX` trên bảng cha tạo một index thật trên **mỗi** partition, PostgreSQL tự đặt tên `gps_oct_attrs_idx` và `gps_nov_attrs_idx`. Bảng cha chỉ giữ index «khung», không có dữ liệu.

Planner có dùng index không?

```sql
EXPLAIN (COSTS OFF) SELECT count(*) FROM gps WHERE attrs @> '{"accuracy_m":5}';
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-jsonb-quan-he/dbstudio-06.webp" alt="DBStudio chạy truy vấn của bước 053 trong p06/run.mjs và hiển thị kế hoạch thực thi gồm 4 toán tử" width="1400" height="484" loading="lazy"></div><figcaption>DBStudio · bước 053 của <code>p06/run.mjs</code> · Kế hoạch truy vấn, 4 toán tử</figcaption></figure>

Không. Planner chọn **Seq Scan** (quét tuần tự) trên cả hai partition. Cách giải thích hợp lý — là suy luận, không phải thứ lab đo: mỗi partition chỉ vài trăm dòng nằm trong một số ít page, và điều kiện khớp khoảng 10 % số dòng; đọc thẳng vài page rẻ hơn đi qua index rồi quay lại heap.

Để chứng minh index **dùng được** cho toán tử này, runner tắt seq scan trong đúng một session — chỉ để minh hoạ, không bao giờ làm vậy trong production:

```sql
SET enable_seqscan = off;
EXPLAIN (COSTS OFF) SELECT count(*) FROM gps WHERE attrs @> '{"accuracy_m":5}';
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-jsonb-quan-he/dbstudio-07.webp" alt="DBStudio chạy truy vấn của bước 053 trong p06/run.mjs và hiển thị kế hoạch thực thi gồm 6 toán tử" width="1400" height="576" loading="lazy"></div><figcaption>DBStudio · bước 053 của <code>p06/run.mjs</code> · Kế hoạch truy vấn, 6 toán tử</figcaption></figure>

`Bitmap Index Scan on gps_*_attrs_idx` cho thấy `@>` khớp operator class và index có thể phục vụ truy vấn. `Recheck Cond` xuất hiện vì GIN trả về tập ứng viên; với `jsonb_path_ops`, giá trị băm có thể trùng, nên dòng phải được kiểm lại trên heap. Đây là plan `COSTS OFF`, không có `ANALYZE`: nó **không chứng minh index nhanh hơn**. Muốn biết, bạn cần dữ liệu cỡ thật, `EXPLAIN (ANALYZE, BUFFERS)` và nhiều lần đo.

Lưu ý thêm, như PG-028 đã chỉ ra: GIN JSONB chỉ phục vụ toán tử của operator class. Viết `(attrs->>'accuracy_m')::int = 5` hay `attrs->>'source' = 'phone'` thì index GIN **không bao giờ** được dùng, dù có tắt seq scan.

Cuối cùng là kích thước (đo sau khi thêm cột generated ở mục 8, nên các index đã được dựng lại cùng bảng):

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-jsonb-quan-he/dbstudio-05.webp" alt="DBStudio chạy truy vấn của bước 053 trong p06/run.mjs và hiển thị kết quả 4 dòng" width="996" height="648" loading="lazy"></div><figcaption>DBStudio · bước 053 của <code>p06/run.mjs</code> · 4 dòng</figcaption></figure>

Index GIN mỗi partition 16 kB, nhỏ hơn khoá chính 40 kB. Đừng suy rộng: `attrs` trong fixture chỉ có hai khoá và 11 giá trị phân biệt (`"phone"` và các số 5…14), tức rất ít mục index, mỗi mục trỏ tới nhiều dòng. Dữ liệu thật với nhiều khoá, nhiều giá trị khác nhau cho index lớn hơn hẳn. Thêm vào đó, mỗi index GIN là thêm việc cho mỗi lần ghi — ở 6.250 điểm/giây, đó là chi phí phải đo, không phải đoán.

## 7. Planner biết gì về bên trong JSONB?

Khi bạn `ANALYZE`, PostgreSQL thu thống kê cho từng **cột**: giá trị phổ biến, histogram, tỉ lệ `NULL` ([PG-027](/articles/postgresql-statistics-analyze-cardinality)). Với cột `jsonb`, thống kê đó nói về **toàn bộ tài liệu** như một giá trị, không có thống kê riêng cho từng khoá bên trong. Hệ quả thực tế:

- Với điều kiện kiểu `(attrs->>'accuracy_m')::int < 7`, planner không có phân bố của `accuracy_m`, nên phải dùng ước lượng mặc định. Ước lượng sai số dòng dẫn tới chọn sai kiểu join hay sai thứ tự join ở truy vấn lớn hơn.
- Với `@>`, planner có cơ chế ước lượng dựa trên các giá trị phổ biến của cả cột; khi tài liệu đa dạng (mỗi điểm GPS một bộ toạ độ khác nhau), cơ chế này khó cho ra con số sát.

Có ba cách cho planner thống kê về một khoá, đều **không chạy trong lab**:

1. **Expression index** trên biểu thức, ví dụ `CREATE INDEX ON gps (((attrs->>'accuracy_m')::int));` — sau `ANALYZE`, PostgreSQL thu thống kê cho biểu thức của index ([PG-025](/articles/postgresql-index-nhieu-cot-partial-expression)).
2. **Extended statistics trên biểu thức** (có từ PostgreSQL 14): `CREATE STATISTICS ... ON ((attrs->>'accuracy_m')::int) FROM ...` rồi `ANALYZE`, không cần index.
3. **Cột generated** — cột thật, có thống kê như mọi cột (mục 8).

Bài không khẳng định mức sai lệch ước lượng trên `gps`: lab không chạy `EXPLAIN ANALYZE` để so số dòng ước lượng với thực tế cho điều kiện trên JSONB. Bài tập 2 làm việc đó.

## 8. Nâng khoá nóng thành cột: generated column

Giả sử điều phối bắt đầu lọc «chỉ dùng điểm có sai số dưới 8 m» trong mọi truy vấn tìm xe. `accuracy_m` đã thành một **khoá nóng** (hot key): được lọc thường xuyên, so sánh khoảng, cần kiểu số. Đây là lúc đưa nó ra cột.

Cách ít xáo trộn nhất là **generated column** (cột sinh): giá trị được PostgreSQL tính từ biểu thức trên các cột khác của cùng dòng, ứng dụng không ghi trực tiếp. Lab chạy:

```sql
ALTER TABLE gps
  ADD COLUMN accuracy_m int GENERATED ALWAYS AS ((attrs->>'accuracy_m')::int) STORED;
SELECT count(*) FROM gps WHERE accuracy_m = 5;
```

```text
PASS 053 generated column agrees with JSONB: 147
```

Cùng 147 dòng như phép `@>` ở mục 4. Giờ `accuracy_m` là cột `int`: so sánh khoảng là so sánh số, có B-tree bình thường nếu cần, có thống kê sau `ANALYZE`, và gõ sai tên cột sẽ nhận lỗi thay vì 0 dòng. Ứng dụng vẫn ghi `attrs` như cũ; không phải sửa đường ghi.

Một hệ quả ít ai để ý: biểu thức ép `::int` chạy **mỗi lần ghi**. Nếu một bản firmware gửi `"accuracy_m": "unknown"`, `INSERT` sẽ thất bại với `22P02` ngay lúc ghi — lỗi đã quay về đúng chỗ, ở người gửi. Lab **không chạy** tình huống này; nó suy ra từ cách cột generated được tính, và bài tập 3 yêu cầu bạn kiểm. Đó có thể là điều bạn muốn, hoặc là sự cố: quyết định trước khi triển khai, kèm kế hoạch xử lý dữ liệu sai.

Ba điều cần biết về PostgreSQL 18 (tài liệu *Generated Columns*):

- **PostgreSQL 18 có hai loại**: `STORED` tính lúc ghi và lưu trên đĩa như cột thường; `VIRTUAL` tính lúc đọc, không chiếm chỗ. **Từ PostgreSQL 18, nếu không ghi rõ thì mặc định là `VIRTUAL`.** Lab ghi rõ `STORED`. Theo tài liệu PostgreSQL 18, cột virtual chưa đánh index được, nên khi mục tiêu là lọc nhanh và có index, hãy ghi `STORED`.
- **Thêm cột STORED vào bảng đã có dữ liệu sẽ viết lại bảng** (table rewrite) để tính giá trị cho mọi dòng hiện có, dưới khoá `ACCESS EXCLUSIVE` ([PG-033](/articles/postgresql-lock-row-table)). Trên `gps` 1.464 dòng không đáng kể; trên bảng GPS hàng tỉ dòng, đó là một kế hoạch migration riêng ([PG-009](/articles/postgresql-schema-migration-tuong-thich)). Với dữ liệu phân vùng, bạn có thể áp dụng cho partition mới trước, nhưng lab không thử cách đó.
- Nếu dữ liệu cũ đã có giá trị sai kiểu, chính câu `ALTER TABLE` sẽ thất bại. Chạy truy vấn kiểm `jsonb_typeof(attrs->'accuracy_m')` trên dữ liệu thật trước.

Bước tiếp theo, nếu cần, là chuyển hẳn: cột thật do ứng dụng ghi, xoá khoá khỏi `attrs`.

## 9. Những gì lab chưa chạy

PostgreSQL có nhiều công cụ JSON hơn những gì lab đo. Các câu dưới đây **không chạy trong lab**, không có output; dùng làm điểm khởi đầu để tự thử:

```sql
-- SQL/JSON path: điểm có sai số dưới 8 m
SELECT count(*) FROM gps WHERE attrs @? '$.accuracy_m ? (@ < 8)';
SELECT jsonb_path_query(attrs, '$.accuracy_m') FROM gps LIMIT 3;

-- JSON_TABLE (có từ PostgreSQL 17): trải tài liệu thành cột có kiểu
SELECT j.* FROM gps,
  JSON_TABLE(attrs, '$' COLUMNS (source text PATH '$.source',
                                 accuracy_m int PATH '$.accuracy_m')) AS j
LIMIT 3;

-- Cập nhật một khoá: vẫn tạo phiên bản dòng mới với toàn bộ tài liệu
UPDATE gps SET attrs = jsonb_set(attrs, '{accuracy_m}', '6')
WHERE driver_id = 1 AND seen_at = '2026-11-15 00:00 UTC';
```

`@?` và `@@` nằm trong nhóm toán tử mà `jsonb_path_ops` phục vụ; mức độ index giúp được một biểu thức path cụ thể (như phép so sánh `< 8`) cần kiểm bằng `EXPLAIN`, đừng giả định.

Về `jsonb_set`: theo MVCC ([PG-032](/articles/postgresql-mvcc-snapshot)), `UPDATE` tạo phiên bản dòng mới; sửa một khoá vẫn ghi lại cả giá trị `jsonb`. Khi tài liệu đủ lớn để bị **TOAST** (đẩy giá trị lớn ra bảng phụ, nén và chia mảnh), mỗi lần sửa nhỏ có thể kéo theo việc ghi lại cả giá trị đã TOAST. Bản tin GPS vài chục byte không chạm ngưỡng đó, nhưng một cột «nhét mọi thứ» lớn dần theo thời gian thì có thể. Lab không đo chi phí này.

## 10. Hướng dẫn quyết định

| Câu hỏi về trường dữ liệu | Nếu «có» | Ví dụ ở Tới Nơi |
|---|---|---|
| Cần `NOT NULL`, `UNIQUE`, khoá chính hay khoá ngoại? | Cột quan hệ | `driver_id`, `seen_at` |
| Có đơn vị, kiểu, miền giá trị nghiệp vụ phải đúng (tiền, thời gian)? | Cột quan hệ ([PG-004](/articles/postgresql-kieu-du-lieu-tien-thoi-gian-uuid)) | cước, thời điểm đo |
| Được lọc, sắp xếp, so sánh khoảng thường xuyên, cần thống kê cho planner? | Cột (hoặc generated column) | `accuracy_m` khi điều phối bắt đầu lọc |
| Thưa (chỉ một số thiết bị có), thay đổi theo đời máy, chủ yếu lưu và hiển thị? | JSONB | `battery_pct`, `heading` của bản thử nghiệm |
| Chỉ lọc «bằng» hoặc «có chứa» trên tập khoá không cố định? | JSONB + GIN, đo trước khi tạo | lọc theo `source` |
| Là payload gốc cần giữ nguyên văn để đối chứng? | `json` (hoặc lưu riêng) | bản tin thô từ đối tác |

Quy tắc ngắn gọn: **dữ liệu mà nghiệp vụ dựa vào để quyết định thì cho vào cột; dữ liệu mà thiết bị tự mô tả về mình thì để JSONB; khoá nào trở nên nóng thì nâng thành cột.** Nghĩ về `attrs` như một vùng «chờ phân loại», không phải nơi chứa vĩnh viễn. Ranh giới này cũng là chuyện chuẩn hoá ([PG-008](/articles/postgresql-mo-hinh-quan-he-chuan-hoa)): JSONB không miễn cho bạn khỏi bất thường cập nhật; nó chỉ dời chúng vào trong một giá trị.

## 11. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách xử lý |
|---|---|---|
| Truy vấn trả 0 dòng, không lỗi | Gõ sai tên khoá (lab: `accurracy_m` → 0) | Test có dữ liệu mẫu và kỳ vọng khác 0; khoá nóng nâng thành cột |
| Truy vấn chạy hôm qua, hôm nay `22P02` | Một dòng mới có giá trị sai kiểu ở khoá đang ép kiểu | CHECK theo `jsonb_typeof`, hoặc generated column để chặn lúc ghi |
| So sánh `'10' < '5'` cho kết quả lạ | So sánh `->>` (text) thay vì số | Ép kiểu, hoặc dùng cột có kiểu |
| Tạo GIN mà truy vấn vẫn Seq Scan | Dùng `->>` (không thuộc operator class), dùng `?` với `jsonb_path_ops`, hoặc bảng còn nhỏ (lab) | Viết bằng `@>`; chọn operator class theo toán tử; đo trên dữ liệu thật |
| Lấy `enable_seqscan = off` làm bằng chứng index nhanh | Tham số đó chỉ ép planner tránh seq scan | `EXPLAIN (ANALYZE, BUFFERS)` trên dữ liệu cỡ thật, nhiều lần đo |
| Không phân biệt thiếu khoá với JSON `null` | `->>` trả `NULL` cho cả hai | `attrs ? 'k'` và `jsonb_typeof(attrs -> 'k')` |
| `ADD COLUMN ... GENERATED` làm bảng đứng | Cột STORED viết lại bảng dưới `ACCESS EXCLUSIVE` | Lập kế hoạch migration; thử trên bản sao |

## 12. Đánh đổi

- **Linh hoạt lúc ghi hay an toàn lúc đọc.** JSONB cho phép thiết bị mới gửi trường mới mà không migration; đổi lại mỗi người đọc phải tự xử lý dữ liệu thiếu và sai. Cột có kiểu làm ngược lại.
- **Một index GIN hay nhiều cột + B-tree.** GIN trên `attrs` phục vụ nhiều khoá bằng một index nhưng chỉ cho phép «bằng/chứa» và cộng chi phí cho mỗi lần ghi; cột riêng có B-tree phục vụ cả so sánh khoảng nhưng phải chọn trước khoá nào quan trọng.
- **STORED hay VIRTUAL.** STORED tốn chỗ, viết lại bảng khi thêm, nhưng có index; VIRTUAL không tốn chỗ, thêm nhanh, nhưng tính mỗi lần đọc và (PostgreSQL 18) chưa đánh index được.

## 13. Bài tập

**Bài 1. Containment phân biệt kiểu.** Trên fixture của lab, đếm `attrs @> '{"accuracy_m":"5"}'` và `attrs @> '{"accuracy_m":5}'`, rồi chèn một điểm mới có `accuracy_m` là chuỗi `"5"` và đếm lại cả hai.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Bảng bốn con số (hai câu × trước/sau khi chèn) kèm <code>jsonb_typeof(attrs -&gt; 'accuracy_m')</code> của điểm mới. Kết luận «containment so theo kiểu JSON» chỉ hợp lệ nếu câu dùng số vẫn là 147 sau khi chèn, còn câu dùng chuỗi tăng đúng 1. Kèm thêm <code>(attrs-&gt;&gt;'accuracy_m')::int = 5</code> để thấy <code>-&gt;&gt;</code> xoá mất khác biệt kiểu (đếm 148). Ghi phiên bản PostgreSQL bạn chạy; lab chưa chạy bài này.</p>
</details>

**Bài 2. Ước lượng của planner trước và sau khi nâng cột.** So sánh `EXPLAIN (ANALYZE)` của `WHERE (attrs->>'accuracy_m')::int < 7` với `WHERE accuracy_m < 7` (sau khi có cột generated và đã `ANALYZE`).

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Với mỗi câu: số dòng planner ước lượng (<code>rows=</code> trong plan có COSTS) và số dòng thật (<code>actual rows</code>), cùng trên một dữ liệu và sau cùng một lần <code>ANALYZE</code>. Số thật phải bằng nhau (theo fixture: 294 điểm có sai số 5 hoặc 6 — tự kiểm bằng <code>count(*)</code>). Kết luận về độ chính xác ước lượng chỉ dựa trên chênh lệch bạn đo được; nếu thêm expression index hoặc <code>CREATE STATISTICS</code>, ghi rõ ước lượng thay đổi thế nào sau <code>ANALYZE</code>.</p>
</details>

**Bài 3. Đưa lỗi về lúc ghi.** Thử chèn một điểm có `"accuracy_m": "unknown"` vào bảng **trước** và **sau** khi có cột generated `accuracy_m ... STORED`; sau đó viết CHECK ở mục 5 thay cho cột generated và thử lại với ba trường hợp: thiếu khoá, chuỗi, số.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Một bảng: trường hợp × cơ chế (không có gì / generated column / CHECK) × kết quả (thành công hoặc SQLSTATE kèm tên ràng buộc). Phải chỉ ra điểm khác nhau giữa hai cơ chế với trường hợp thiếu khoá (generated column cho <code>NULL</code>, CHECK có thể cho qua hoặc chặn tuỳ cách viết), và một truy vấn ép kiểu ở mục 5 trước đó chạy được đã thất bại thế nào khi dữ liệu sai lọt vào. Kết luận dựa trên output của bạn, không chép từ bài.</p>
</details>

## 14. Đọc tiếp

- [PostgreSQL 18 · JSON Types](https://www.postgresql.org/docs/18/datatype-json.html): `json` và `jsonb`, containment, mục *jsonb Indexing* về `jsonb_ops` và `jsonb_path_ops`.
- [PostgreSQL 18 · JSON Functions and Operators](https://www.postgresql.org/docs/18/functions-json.html): `->`, `->>`, `@>`, `?`, `jsonb_typeof`, SQL/JSON path, `JSON_TABLE`.
- [PostgreSQL 18 · GIN Indexes](https://www.postgresql.org/docs/18/gin.html): cấu trúc và chi phí cập nhật của GIN.
- [PostgreSQL 18 · Generated Columns](https://www.postgresql.org/docs/18/ddl-generated-columns.html): `STORED`, `VIRTUAL` và giới hạn.
- Trong chuỗi: [PG-028 · GIN, GiST và BRIN](/articles/postgresql-gin-gist-brin-lua-chon), [PG-025 · Expression index](/articles/postgresql-index-nhieu-cot-partial-expression), [PG-027 · Statistics và ANALYZE](/articles/postgresql-statistics-analyze-cardinality).
- Lab: `examples/toi-noi-postgresql/p06/run.mjs` (khối PG-053), `p06/fixture.sql`, kết quả `p06/OUTPUT.txt`, giới hạn trong `p06/README.md`.

Bài tiếp theo: [PG-054 · Partition pruning và retention cho lịch sử GPS](/articles/postgresql-partition-pruning-retention).
