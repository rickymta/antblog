---
slug: postgresql-extension-postgis-pgvector
title: "Đánh giá PostGIS và pgvector theo workload: hai thí nghiệm, điều kiện dùng và điều kiện dừng"
summary: "Dispatch của Tới Nơi muốn tìm tài xế trong 500 m, CSKH muốn tìm ticket tương tự, và có người đề xuất cài luôn PostGIS lẫn pgvector vào cluster chính. Bài dựng hai thí nghiệm độc lập trên PGlite (PostgreSQL 18.6, postgis 3.6.2, vector 0.8.7): ST_DWithin trước/sau GiST, KNN, bẫy SRID 4326 đo bằng độ, top-10 chính xác so với HNSW ở ba mức ef_search qua ba lần dựng index, rồi viết điều kiện dùng, điều kiện dừng và những gì lab không chứng minh."
category: postgresql
level: intermediate
tags: [postgresql, pg-057, extension, postgis, pgvector, gist, hnsw, knn, recall, toi-noi]
language: vi
---

Cuộc họp sáng thứ Hai của Tới Nơi có hai yêu cầu không liên quan đến nhau. Đội điều phối muốn màn hình gán chuyến lấy được «mọi tài xế trong bán kính 500 m quanh điểm đón», thay cho đoạn code kéo toạ độ cả quận về ứng dụng rồi tự tính. Đội chăm sóc khách hàng (CSKH) muốn khi một ticket mới vào, hệ thống gợi ý mười ticket cũ «giống nhất».

Một kỹ sư đề xuất: «Cài PostGIS và pgvector vào cluster chính, một migration là xong.» Cả hai đều là **extension** — gói mở rộng nạp bằng `CREATE EXTENSION`, thêm kiểu dữ liệu, hàm, toán tử và loại index. Nhưng câu hỏi đúng không phải «extension có chạy không», mà là: với **workload** này (khối lượng, kiểu truy vấn, tần suất ghi, hậu quả khi sai), cần chứng minh gì trước khi dùng, và **khi nào thì dừng**.

Bài này dựng hai thí nghiệm tách biệt, mỗi cái có câu hỏi, baseline (đường so sánh không dùng thứ mới), phép kiểm đúng và giới hạn riêng. Không thí nghiệm nào được dùng để biện minh cho thí nghiệm kia.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-057 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-gin-gist-brin-lua-chon">PG-028 · Khi nào cân nhắc GIN, GiST và BRIN</a> (chọn index theo toán tử), <a href="/articles/postgresql-explain-ke-hoach-thuc-thi">PG-021 · Đọc EXPLAIN</a>. Lab: <code>examples/toi-noi-postgresql/p06/extensions</code> — <code>run.mjs</code> (đúng những gì đã chạy), <code>OUTPUT.txt</code> (kết quả thực), <code>package.json</code> (phiên bản cố định). Chạy tại <code>examples/toi-noi-postgresql</code>: <code>npm run p06:extensions:install</code> một lần, rồi <code>npm run p06:extensions</code>. Sau bài này, bạn thiết kế được một thí nghiệm đánh giá extension có baseline, phép kiểm đúng, điều kiện dùng, điều kiện dừng, và nói rõ nó <b>không</b> chứng minh điều gì.</p>
</div>

## 1. Cài extension là nhận một cam kết vận hành

PostGIS hay pgvector là thư viện mã máy chạy **bên trong tiến trình server**, cùng bộ nhớ và quyền với PostgreSQL. `CREATE EXTENSION` kéo theo những việc phải làm suốt đời cluster ([PostgreSQL 18 · Packaging Related Objects into an Extension](https://www.postgresql.org/docs/18/extend-extensions.html)):

- **Có mặt trên mọi node.** Standby vật lý replay WAL chứa dữ liệu `geography` hay index HNSW nên cần cùng thư viện; subscriber logical replication (PG-055) cần kiểu dữ liệu tồn tại trước khi apply.
- **Theo vòng đời nâng cấp.** `pg_upgrade` đòi cluster mới có thư viện extension cho bản mới (PG-059); extension có phiên bản riêng, nâng bằng `ALTER EXTENSION … UPDATE`.
- **Backup chỉ mang định nghĩa.** `pg_dump` ghi câu `CREATE EXTENSION`, không ghi thư viện; nơi restore (PG-046) phải cài sẵn gói.
- **Nền tảng có hỗ trợ không.** Dịch vụ PostgreSQL được quản lý chỉ cho cài những extension và phiên bản nhà cung cấp chọn.

Khuôn mẫu dùng cho cả hai thí nghiệm:

| Thành phần | Câu hỏi phải trả lời trước khi chạy |
|---|---|
| Câu hỏi nghiệp vụ | Màn hình nào, ai dùng, kết quả sai thì hậu quả gì |
| Phép kiểm đúng | Lấy gì làm đáp án: đếm tay, baseline chính xác, nhãn do người gán |
| Baseline | Cách làm không cần thứ mới, trên cùng dữ liệu |
| Ứng viên | Index hoặc tham số cần thử, mỗi lần đổi một thứ |
| Điều kiện dùng | Ngưỡng phải đạt, đo trên môi trường giống production |
| Điều kiện dừng | Dấu hiệu nào thì bỏ, hoặc chuyển sang kiến trúc khác |

## 2. Lab: vì sao chạy trên PGlite, và điều đó không chứng minh gì

Bản PostgreSQL 18 cho Windows dùng ở các phần trước **không kèm** PostGIS lẫn pgvector (chú thích đầu `run.mjs`, README P06). Lab vì vậy chạy trên **PGlite** — PostgreSQL biên dịch sang WebAssembly (WASM), chạy trong tiến trình Node, đơn tiến trình, có gói hai extension dựng sẵn. Phiên bản cố định trong `package.json` (`@electric-sql/pglite` 0.5.16, `pglite-postgis` 0.2.16, `pglite-pgvector` 0.0.17). Dòng đầu `OUTPUT.txt`:

```text
PG-057 extension lab · PGlite PostgreSQL 18.6 · postgis 3.6.2 · vector 0.8.7 · Node v22.18.0
```

Dữ liệu tất định, không cần mạng; runner chỉ ghi `OUTPUT.txt` khi mọi `assert` đạt.

PGlite **chứng minh được** ngữ nghĩa (mét hay độ, khoảng cách xếp thế nào), **hình dạng kế hoạch** (planner dùng index bằng điều kiện gì) và **độ đúng so với baseline**. Nó **không chứng minh** hiệu năng server (lab không in một con số millisecond nào, có chủ ý), bộ nhớ khi build và truy vấn index, hành vi khi nhiều phiên đồng thời, chi phí ghi, thời gian build, lượng WAL, hay việc cài gói trên hạ tầng thật. README P06 ghi rõ PGlite «không phải server», và recall trên dữ liệu tổng hợp «không có ý nghĩa cho embedding thật».

## 3. Thí nghiệm 1: tài xế trong 500 m quanh điểm đón

### 3.1. Dữ liệu và kiểu `geography`

`run.mjs` đặt 5.000 tài xế trên một **lưới tất định** quanh Quận 1: 100 cột × 50 hàng, bước 0,002 độ.

```sql
CREATE TABLE drivers (id int PRIMARY KEY, location geography(Point, 4326) NOT NULL);
INSERT INTO drivers
SELECT n, ST_SetSRID(ST_MakePoint(106.60 + (n % 100) * 0.002, 10.70 + (n / 100) * 0.002), 4326)::geography
FROM generate_series(0, 4999) n;
```

`ST_MakePoint` nhận **kinh độ trước, vĩ độ sau**. **SRID** (*spatial reference identifier*) 4326 là hệ WGS 84 mà GPS điện thoại trả về. Kiểu **`geography`** coi toạ độ là điểm trên bề mặt Trái Đất, các hàm đo trả **mét** ([PostGIS · Geography](https://postgis.net/docs/using_postgis_dbmanagement.html#PostGIS_Geography)); kiểu `geometry` tính trên mặt phẳng theo đơn vị của SRID (mục 3.5). Điểm đón `POINT(106.70 10.77)` trùng đúng nút lưới của tài xế 3550.

### 3.2. Baseline: không có index không gian

```sql
SELECT count(*) FROM drivers
WHERE ST_DWithin(location, 'SRID=4326;POINT(106.70 10.77)'::geography, 500);
```

**`ST_DWithin(a, b, d)`** đúng khi khoảng cách không vượt `d`; với `geography`, `d` tính bằng mét và mặc định đo trên elipxoit (`use_spheroid = true`) ([PostGIS · ST_DWithin](https://postgis.net/docs/ST_DWithin.html)). Kế hoạch khi chưa có index (`EXPLAIN (COSTS OFF)`):

```text
--- 057 EXPLAIN ST_DWithin without spatial index
  Aggregate
    ->  Seq Scan on drivers
          Filter: st_dwithin(location, '0101000020E6100000CDCCCCCCCCAC5A400AD7A3703D8A2540'::geography, '500'::double precision, true)
```

Chuỗi hex là điểm đón ở dạng nhị phân; tham số cuối `true` là `use_spheroid`. Seq Scan tính khoảng cách cho cả 5.000 dòng và ra 21 tài xế, con số được giữ làm đáp án trước khi tạo index.

Con số 21 kiểm được bằng tay, đó là lý do dùng lưới. Ở vĩ độ 10,77°, 0,002 độ kinh tuyến dài khoảng 219 m, 0,002 độ vĩ tuyến khoảng 221 m (tính xấp xỉ, không phải output lab):

| Dịch (cột, hàng) | Khoảng cách xấp xỉ | Số điểm | Trong 500 m? |
|---|---|---|---|
| (0, 0) | 0 m | 1 | có |
| (±1, 0) / (0, ±1) | 219 / 221 m | 4 | có |
| (±1, ±1) | ≈ 311 m | 4 | có |
| (±2, 0) / (0, ±2) | ≈ 438 / 442 m | 4 | có |
| (±2, ±1) / (±1, ±2) | ≈ 491 / 493 m | 8 | có |
| (±2, ±2), (±3, 0) … | ≥ 620 m | — | không |

1 + 4 + 4 + 4 + 8 = 21. Có đáp án độc lập với PostgreSQL thì khi index làm đổi con số, ta biết ngay ai sai.

### 3.3. GiST: lọc bằng hộp bao, rồi kiểm lại chính xác

```sql
CREATE INDEX drivers_location_gist ON drivers USING gist (location);
```

**GiST** (*Generalized Search Tree*) là khung index mà PG-028 đã dùng cho exclusion constraint. PostGIS cài vào đó một chiến lược không gian: mỗi mục index lưu **hộp bao** (*bounding box*) của hình, không lưu hình đầy đủ ([PostgreSQL 18 · GiST](https://www.postgresql.org/docs/18/gist.html)).

```text
PASS 057 drivers within 500 m of pickup (seq scan = GiST): 21
--- 057 EXPLAIN ST_DWithin with GiST
  Aggregate
    ->  Bitmap Heap Scan on drivers
          Filter: st_dwithin(location, '0101000020E6100000CDCCCCCCCCAC5A400AD7A3703D8A2540'::geography, '500'::double precision, true)
          ->  Bitmap Index Scan on drivers_location_gist
                Index Cond: (location && _st_expand('0101000020E6100000CDCCCCCCCCAC5A400AD7A3703D8A2540'::geography, '500'::double precision))
```

Đọc từ dưới lên là hai tầng lọc:

1. **`Index Cond: location && _st_expand(…, 500)`** — ta không viết `&&`; PostGIS tự thêm. `_st_expand` nới hộp bao của điểm đón ra 500 m mỗi phía, `&&` hỏi hộp bao của tài xế có **giao** hộp đó không. Tài liệu `ST_DWithin` mô tả đúng hành vi này: lời gọi hàm kèm một phép so hộp bao dùng được index.
2. **`Filter: st_dwithin(…)`** — hộp vuông bao một hình tròn, nên tài xế ở **góc** hộp lọt qua tầng 1 dù cách hơn 500 m. Mọi ứng viên được đọc từ heap và tính lại khoảng cách thật. Index ở đây là **lossy**: loại nhanh phần chắc chắn sai, kết luận cuối do hàm chính xác đưa ra.

Dòng `PASS` là `assert` rằng số đếm trước và sau index bằng nhau; index đổi kết quả là lỗi, không phải tối ưu. Lab chỉ chứng minh planner **chọn** đường index trên 5.000 dòng, không đo nó nhanh hơn bao nhiêu.

### 3.4. KNN: ba tài xế gần nhất

Màn hình gán chuyến thường cần «k người gần nhất». Toán tử **`<->`** trả khoảng cách và dùng trong `ORDER BY … LIMIT k`, gọi là truy vấn **KNN** (*k-nearest neighbours*); GiST có thể trả dòng theo thứ tự khoảng cách mà không sắp cả bảng ([PostGIS · `<->`](https://postgis.net/docs/geometry_distance_knn.html), [PostgreSQL 18 · Indexes and ORDER BY](https://www.postgresql.org/docs/18/indexes-ordering.html)).

```sql
SELECT id, round(ST_Distance(location, :pickup))::int AS m
FROM drivers ORDER BY location <-> :pickup LIMIT 3;
```

```text
--- 057 three nearest drivers (KNN <->)
  id 3550 · 0 m
  id 3551 · 219 m
  id 3549 · 219 m
```

Tài xế 3550 đứng đúng điểm đón; 3551 và 3549 là hai nút kề phương đông–tây (219 m), gần hơn hai nút bắc–nam (221 m). Trước khi đưa vào production:

- **Hai dòng hoà nhau** ở 219 m; thứ tự giữa chúng không được bảo đảm. Nếu chuyến gửi cho người đứng đầu danh sách, thêm khoá phụ `ORDER BY location <-> :pickup, id` (cùng bài học khoá sắp xếp trùng của PG-018).
- **Lab không in kế hoạch câu KNN**, nên bài không khẳng định nó đi qua index. Tự thử thì `EXPLAIN` và tìm `Order By` ở nút index.
- Theo tài liệu PostGIS, với `geography`, `<->` tính trên mặt **cầu**, còn `ST_Distance` mặc định trên **elipxoit**. Dùng `<->` để xếp hạng, `ST_Distance` để hiển thị; đừng so hai số ở chữ số cuối.

### 3.5. Bẫy: `geometry` SRID 4326 đo bằng độ

Cùng câu hỏi, chỉ ép sang `geometry`:

```sql
SELECT count(*) FROM drivers
WHERE ST_DWithin(location::geometry, 'SRID=4326;POINT(106.70 10.77)'::geometry, 500);
```

```text
PASS 057 geometry in SRID 4326 measures degrees: radius 500 matches 5000 of 5000 drivers
```

Với `geometry`, đơn vị là đơn vị của hệ toạ độ; SRID 4326 tính bằng **độ**, nên `500` là 500 độ. Cả lưới chỉ rộng khoảng 0,2 độ, mọi tài xế đều «trong bán kính». Không lỗi, không cảnh báo; chỉ có chuyến bị gửi cho tài xế ở quận khác. Lab `assert` số 5000 để bẫy hiện ra mỗi lần chạy.

Cách sửa: giữ `geography`, hoặc chiếu sang hệ phẳng đo bằng mét. TP.HCM nằm trong múi UTM 48N (EPSG:32648). Đoạn dưới **không chạy trong lab**:

```sql
-- không chạy trong lab: geometry chiếu sang UTM 48N, đơn vị mét
CREATE INDEX drivers_location_utm
  ON drivers USING gist ((ST_Transform(location::geometry, 32648)));
SELECT count(*) FROM drivers
WHERE ST_DWithin(ST_Transform(location::geometry, 32648),
                 ST_Transform('SRID=4326;POINT(106.70 10.77)'::geometry, 32648), 500);
```

Phép chiếu phẳng chỉ chính xác trong vùng của nó; dữ liệu trải nhiều múi thì `geography` đơn giản hơn ([PostGIS · ST_Transform](https://postgis.net/docs/ST_Transform.html)). Lựa chọn nào cũng cần phép kiểm đáp án như mục 3.2.

### 3.6. Điều bán kính không trả lời

Thí nghiệm chứng minh: trên dữ liệu đã biết đáp án, `ST_DWithin` với `geography` trả đúng 21, GiST không đổi kết quả, và bẫy độ/mét có thật. Nó **không** trả lời:

- **Đường chim bay không phải thời gian tới.** Đường một chiều, cầu, điểm quay đầu quyết định ETA; bán kính chỉ lọc ứng viên.
- **GPS có sai số.** Fixture GPS chính của P06 có `accuracy_m` 5–14 m (README). Tài xế ở 495 m với sai số 14 m có thể ở trong hoặc ngoài; quy tắc ở biên do nghiệp vụ quyết.
- **Vị trí cũ và chi phí ghi.** Hồ sơ Tới Nơi ước ≈ 6.250 điểm vị trí mỗi giây lúc cao điểm. Câu hỏi thật là «ai **đang** ở gần», và mỗi điểm chèn vào bảng có GiST đều cập nhật index. Lab không đo chi phí đó.
- **Phương án khác** như lưới lục giác H3, hoặc kho vị trí nóng riêng và chỉ ghi lịch sử vào PostgreSQL. Lab không thử.

## 4. Thí nghiệm 2: ticket tương tự với pgvector

### 4.1. Vector, khoảng cách và một dữ liệu không có nghĩa

Tìm ticket giống nhau thường bắt đầu bằng một mô hình ngôn ngữ biến mỗi đoạn văn thành **embedding** — vector số thực vài trăm tới vài nghìn chiều, sao cho hai đoạn gần nghĩa thì vector gần nhau. pgvector thêm kiểu `vector(n)` và các toán tử khoảng cách ([pgvector README](https://github.com/pgvector/pgvector)):

| Toán tử | Khoảng cách | Operator class cho index |
|---|---|---|
| `<->` | L2 (Euclid) | `vector_l2_ops` |
| `<#>` | tích vô hướng, lấy âm | `vector_ip_ops` |
| `<=>` | cosine distance | `vector_cosine_ops` |
| `<+>` | L1 | `vector_l1_ops` |

Lab **không** dùng mô hình ngôn ngữ. Nó sinh 2.000 vector 16 chiều (id 1–2000) bằng `sin(n * 0.37 + d * 1.71 + (n % 7) * d)`: tất định nhưng **không mang nghĩa**. Câu hỏi của thí nghiệm vì vậy hẹp: **index gần đúng tìm lại được bao nhiêu phần của kết quả chính xác**.

Vector truy vấn là 20 vector sinh từ cùng công thức với `n = 5001 + i·37` (i = 0…19) — cùng «phân bố» với dữ liệu nhưng **không phải dòng nào trong bảng**, giống một ticket mới chưa được lưu. Nhờ vậy câu truy vấn không cần điều kiện `WHERE` để loại chính nó (mục 4.4 giải thích vì sao điều này quan trọng).

### 4.2. Baseline: tìm chính xác

```sql
SELECT id FROM tickets
ORDER BY embedding <=> (SELECT array_agg(sin(5001 * 0.37 + d * 1.71 + (5001 % 7) * d)::real ORDER BY d)
                        FROM generate_series(1, 16) d)::vector
LIMIT 10;
```

```text
--- 057 EXPLAIN cosine top-10 without vector index
  Limit
    InitPlan 1
      ->  Aggregate
            ->  Sort
                  Sort Key: d.d
                  ->  Function Scan on generate_series d
    ->  Sort
          Sort Key: ((tickets.embedding <=> ((InitPlan 1).col1)::vector))
          ->  Seq Scan on tickets
```

`InitPlan 1` dựng vector truy vấn một lần (gom 16 giá trị theo thứ tự `d`); Seq Scan tính cosine distance cho cả 2.000 dòng; Sort chọn 10 dòng nhỏ nhất. Đây là **tìm kiếm chính xác**: đúng tuyệt đối, chi phí tăng tuyến tính theo số dòng, và là đáp án cho mọi phép đo sau.

### 4.3. HNSW: đồ thị nhiều tầng, đổi độ chính xác lấy tốc độ

**HNSW** (*Hierarchical Navigable Small World*) dựng một đồ thị nhiều tầng: tầng trên thưa để nhảy xa, tầng dưới dày để dò kỹ. Truy vấn đi theo cạnh tới các nút gần dần, giữ một danh sách ứng viên, dừng khi không tìm được nút gần hơn. Đây là index **gần đúng** (*approximate nearest neighbour*, ANN): nhanh vì không xét mọi dòng, và vì thế có thể bỏ sót. Theo README pgvector: `m` là số cạnh tối đa mỗi nút mỗi tầng (mặc định 16), `ef_construction` là độ rộng danh sách ứng viên khi dựng (mặc định 64), `hnsw.ef_search` là độ rộng khi truy vấn (mặc định 40), đặt theo phiên.

Lab cố tình dựng index **nghèo** để sự gần đúng hiện ra trên 2.000 dòng:

```sql
CREATE INDEX tickets_embedding_hnsw ON tickets
  USING hnsw (embedding vector_cosine_ops) WITH (m = 4, ef_construction = 16);
```

```text
--- 057 EXPLAIN cosine top-10 with HNSW
  Limit
    InitPlan 1
      ->  Aggregate
            ->  Sort
                  Sort Key: d.d
                  ->  Function Scan on generate_series d
    ->  Index Scan using tickets_embedding_hnsw on tickets
          Order By: (embedding <=> ((InitPlan 1).col1)::vector)
```

Sort và Seq Scan biến mất, thay bằng `Index Scan … Order By`: index trả dòng theo thứ tự khoảng cách, `Limit` dừng sau 10. Operator class `vector_cosine_ops` khớp `<=>`; viết `<->` (L2) thì index này không được dùng — đúng nguyên tắc chọn index theo toán tử của PG-028.

### 4.4. Recall@10 ở ba mức ef_search

**Recall@10** là tỉ lệ trong 10 kết quả chính xác mà index tìm lại được. Runner chạy 20 vector truy vấn, so top-10 của HNSW với top-10 chính xác đã lưu, và chia tổng số trúng cho 200. Vì thuật toán HNSW chọn tầng của mỗi nút **ngẫu nhiên**, cùng dữ liệu tất định vẫn có thể cho đồ thị khác nhau; runner vì vậy xoá và dựng lại index **3 lần**, đo recall của từng lần ở ba mức `ef_search`:

```text
MEASURE 057 HNSW (m=4, ef_construction=16) recall@10 vs exact, 20 queries not in table, 3 index builds: [{"ef_search":10,"recall_per_build":[0.995,0.995,0.945],"min":0.945,"max":0.995},{"ef_search":40,"recall_per_build":[0.995,0.995,0.945],"min":0.945,"max":0.995},{"ef_search":200,"recall_per_build":[0.995,0.995,0.98],"min":0.98,"max":0.995}]
PASS 057 every HNSW query returned 10 rows (min rows seen: 10)
PASS PG-057: both experiments completed.
```

Dòng recall là `MEASURE`, không phải cổng. Runner chỉ `assert` hai điều: mọi truy vấn HNSW trả đủ 10 dòng, và **trong cùng một lần build**, recall ở `ef_search = 200` không thấp hơn ở `10` — đồ thị đã cố định thì tìm rộng hơn không được tìm ra ít láng giềng đúng hơn. Đọc bảng số:

| Build | ef_search 10 | 40 | 200 |
|---|---|---|---|
| 1 | 0,995 | 0,995 | 0,995 |
| 2 | 0,995 | 0,995 | 0,995 |
| 3 | 0,945 | 0,945 | 0,98 |

**Phân tán giữa các lần build lớn hơn tác dụng của tham số.** Một lần sót là 0,005. Build 1 và 2 sót 1/200 ở mọi mức; build 3 sót 11/200 ở `ef_search` 10 và 40, và tăng lên 200 chỉ kéo về 4/200. Cùng dữ liệu, cùng tham số, chỉ khác lần dựng, mà chênh 0,05. Nếu chỉ dựng một lần và trúng build 3, ta sẽ kết luận khác hẳn. Những lần chạy khác của lab trong lúc phát triển cho độ phân tán lớn hơn nhiều; con số thay đổi theo lần build và theo máy, nên bài chỉ dẫn số của lần chạy đã ghi. Bài học: báo recall là báo **min/max qua nhiều lần build**, kèm số truy vấn, không phải một con số.

**`ef_search` 10 và 40 cho kết quả giống hệt** ở cả ba build. Trên 2.000 dòng với đồ thị nghèo (`m = 4`), thứ hạn chế recall là cấu trúc đồ thị, không phải độ rộng tìm kiếm; chỉ build 3 hưởng lợi khi lên 200. Không suy rộng điều này cho bảng triệu dòng.

**Recall cao không nói gì về chất lượng gợi ý.** Recall so index với tìm chính xác; tìm chính xác vẫn gợi ý ticket vô dụng nếu embedding kém. Câu hỏi CSKH cần một **tập đánh giá có nhãn**: vài trăm ticket nhân viên đã đánh dấu «ticket nào thực sự giúp được».

### 4.5. Những gì pgvector còn đòi mà lab không chạy

Một bẫy lab hiện tại đã tránh nhưng production sẽ gặp: **lọc sau khi quét index**. README pgvector ghi rằng với index gần đúng, điều kiện `WHERE` được áp **sau** khi quét, và `hnsw.ef_search` giới hạn số ứng viên index trả về. Câu `WHERE city = 'HCM' … LIMIT 10` với `ef_search = 40` và bộ lọc chỉ khớp một phần nhỏ dòng có thể trả **ít hơn 10 dòng** mà không báo lỗi. Đó là lý do vector truy vấn của lab nằm ngoài bảng thay vì lọc bỏ chính nó bằng `WHERE id <> q`, và là lý do runner `assert` đủ 10 dòng. Từ 0.8.0, pgvector có iterative index scan (`hnsw.iterative_scan`) để quét tiếp khi bộ lọc loại bớt.

Các đoạn dưới **không chạy trong lab** và không có output:

```sql
-- không chạy trong lab: embedding thật, số chiều cố định theo mô hình
CREATE TABLE ticket_embeddings (
  ticket_id bigint PRIMARY KEY,
  model text NOT NULL,            -- đổi mô hình = tính lại toàn bộ
  embedding vector(1536) NOT NULL
);
-- không chạy trong lab: halfvec giảm bộ nhớ index; IVFFlat dựng sau khi có dữ liệu
CREATE INDEX ON ticket_embeddings USING hnsw ((embedding::halfvec(1536)) halfvec_cosine_ops);
CREATE INDEX ON ticket_embeddings USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
-- không chạy trong lab: tìm có lọc, quét lặp khi bộ lọc loại nhiều
SET hnsw.iterative_scan = relaxed_order;
SELECT t.id FROM tickets t JOIN ticket_embeddings e ON e.ticket_id = t.id
WHERE t.city = 'HCM' ORDER BY e.embedding <=> :q LIMIT 10;
```

Mỗi dòng là một câu hỏi chưa có đáp án: sinh embedding ở đâu (lúc tạo ticket, hay hàng đợi bất đồng bộ như outbox của PG-039); tính lại dữ liệu cũ thế nào khi đổi mô hình; HNSW tốn bao nhiêu bộ nhớ khi build (`maintenance_work_mem`) và khi truy vấn; mỗi INSERT chậm đi bao nhiêu; IVFFlat có hợp hơn cho bảng ít ghi không; `halfvec` hay quantization làm mất bao nhiêu recall.

## 5. Từ thí nghiệm tới quyết định

Thí nghiệm 1 có đáp án tuyệt đối (21) và một bẫy cụ thể. Thí nghiệm 2 chỉ cho thấy cơ chế đánh đổi recall bằng tham số. Quyết định vì vậy viết thành **điều kiện**:

| | PostGIS cho «tài xế gần điểm đón» | pgvector cho «ticket tương tự» |
|---|---|---|
| Điều kiện dùng | Gói cùng phiên bản trên primary, standby và nơi restore; bán kính trả đúng đáp án mẫu; EXPLAIN dùng index trên dữ liệu cỡ production; chi phí ghi của GiST trong ngân sách của bảng vị trí | Có tập đánh giá có nhãn; tìm chính xác đạt chất lượng nghiệp vụ trước khi có index; HNSW giữ recall ≥ ngưỡng thống nhất ở `ef_search` đủ cho truy vấn có lọc; bộ nhớ index trong ngân sách |
| Điều kiện dừng | Còn đường nào dùng `geometry` 4326 với bán kính mét; nền tảng production không hỗ trợ gói; cập nhật index làm chậm luồng ghi vị trí | Chất lượng theo nhãn kém dù recall cao (lỗi ở embedding, không ở index); index tranh bộ nhớ với workload giao dịch; truy vấn vector làm chậm truy vấn chuyến cùng cluster |
| Nếu dừng | Lọc theo ô lưới (cột số nguyên, B-tree) rồi tính khoảng cách ở ứng dụng; hoặc kho vị trí nóng riêng | Tìm chính xác trên tập đã lọc nhỏ; hoặc nơi tìm vector riêng, đồng bộ qua CDC (PG-055) |

Hai cột độc lập: PostGIS đạt không làm pgvector đạt. Còn một quyết định hay bị quên: **đặt ở cluster nào**. Bảng vị trí và bảng ticket có hồ sơ tải khác bảng chuyến; gộp vào cluster chuyến là gộp rủi ro của chúng vào đường ghi quan trọng nhất của Tới Nơi.

## 6. Lỗi thường gặp

- **`geometry` SRID 4326 với bán kính mét.** Không lỗi, không cảnh báo; lab trả 5000/5000.
- **Đảo vĩ độ, kinh độ** trong `ST_MakePoint`. Kiểm bằng một điểm đã biết trước khi nạp.
- **Tin index vì EXPLAIN có chữ «Index».** Index lossy cần tầng kiểm lại; điều phải kiểm là số dòng không đổi so với baseline.
- **`ORDER BY … LIMIT` không có khoá phụ** khi khoảng cách hoà nhau.
- **Toán tử không khớp operator class**: index `vector_cosine_ops` không phục vụ `<->`.
- **Lọc sau ANN với `ef_search` nhỏ** rồi ngạc nhiên vì nhận ít hơn `LIMIT` dòng.
- **Báo recall không kèm số truy vấn, dữ liệu, số lần build.**
- **Lấy PGlite làm số hiệu năng server**, hoặc quên extension khi restore, nâng cấp, dựng standby.

## 7. Đánh đổi

- **`geography` hay `geometry` chiếu phẳng.** `geography` đúng toàn cầu, đơn vị mét, ít hàm và tính nặng hơn; chiếu phẳng nhanh, nhiều hàm, nhưng chỉ đúng trong vùng chiếu và dễ rơi vào bẫy đơn vị.
- **Bán kính hay KNN.** Bán kính có ngữ nghĩa rõ, số dòng đổi theo mật độ; KNN luôn trả k dòng nhưng có thể là người rất xa khi khu vực vắng. Thường ghép cả hai.
- **Exact hay ANN.** Exact đúng tuyệt đối, chi phí tuyến tính; ANN nhanh, tốn bộ nhớ và chi phí ghi, đổi một phần recall chỉnh được bằng `ef_search`. Sau khi lọc còn vài chục nghìn dòng, exact có thể đủ — giả thuyết phải đo trên server.
- **Cùng cluster hay tách riêng.** Cùng cluster có transaction, join, một chỗ backup; tách riêng cô lập tài nguyên, đổi lại độ trễ đồng bộ và thêm một hệ thống để vận hành.

## 8. Bài tập

**Bài 1 — Vùng biên.** Thêm tài xế cách điểm đón 495, 499, 500, 501 và 505 m. Kiểm `ST_DWithin` coi 500 m là trong hay ngoài, và nêu ảnh hưởng của sai số GPS 14 m.

<details><summary><b>Một lời giải đúng phải chứng minh</b></summary>
<p>Toạ độ từng tài xế được kiểm lại bằng <code>ST_Distance</code> trước khi dùng; kết quả <code>ST_DWithin</code> so với đáp án tính trước, cả khi có và không có GiST, và số dòng không đổi. Lời giải nêu một quy tắc nghiệp vụ cho vùng sai số (ví dụ lọc ở 500 m cộng sai số rồi xếp hạng), không coi 500 m là ranh giới tuyệt đối. Chạy trên bản sao lab, không sửa <code>OUTPUT.txt</code>.</p>
</details>

**Bài 2 — Độ phân tán và bẫy bộ lọc.** Trên bản sao `run.mjs`: (a) tăng lên 10 lần build và 100 vector truy vấn ngoài bảng, so min/max với `OUTPUT.txt`; (b) thêm một truy vấn có bộ lọc chỉ khớp khoảng 10% dòng (ví dụ `WHERE id % 10 = 0`) và ghi số dòng trả về ở `ef_search` 10, 40, 200, có và không có `hnsw.iterative_scan`.

<details><summary><b>Một lời giải đúng phải chứng minh</b></summary>
<p>Phần (a) báo recall của <b>từng</b> lần build ở từng mức <code>ef_search</code> cùng min/max, nêu số truy vấn và số lần build, và kết luận về độ phân tán chứ không về một trung bình; mọi truy vấn vẫn trả đủ 10 dòng. Phần (b) cho thấy bằng output số dòng trả về khi có bộ lọc (bao nhiêu truy vấn nhận ít hơn <code>LIMIT</code>), đo recall so với tìm chính xác <b>có cùng bộ lọc</b>, và ghi rõ chế độ <code>hnsw.iterative_scan</code> đã dùng. Không kết luận gì về thời gian, không sửa <code>OUTPUT.txt</code> của lab.</p>
</details>

**Bài 3 — Hồ sơ đánh giá một trang** cho đề xuất «pgvector trên cluster chuyến», theo khuôn mẫu ở mục 1.

<details><summary><b>Một lời giải đúng phải chứng minh</b></summary>
<p>Mỗi điều kiện dùng có ngưỡng đo được và nơi đo (server giống production, không phải PGlite); mỗi điều kiện dừng có dấu hiệu quan sát được và người quyết. Hồ sơ tách ba loại khẳng định: đã chứng minh trong lab (kế hoạch dùng index, recall của ba lần build trên dữ liệu tổng hợp), chưa chứng minh (hiệu năng, bộ nhớ, đồng thời, chi phí ghi, chất lượng theo nhãn), và giả định cần xác nhận (nền tảng hỗ trợ gói, đường nâng cấp). Không con số nào thiếu nguồn.</p>
</details>

## 9. Đọc tiếp

- [PostGIS · ST_DWithin](https://postgis.net/docs/ST_DWithin.html), [`<->` (KNN)](https://postgis.net/docs/geometry_distance_knn.html), [Geography](https://postgis.net/docs/using_postgis_dbmanagement.html#PostGIS_Geography), [ST_Transform](https://postgis.net/docs/ST_Transform.html).
- [pgvector README](https://github.com/pgvector/pgvector): toán tử, HNSW (`m`, `ef_construction`, `hnsw.ef_search`), IVFFlat, lọc và iterative index scan, `halfvec`.
- [PostgreSQL 18 · Packaging Related Objects into an Extension](https://www.postgresql.org/docs/18/extend-extensions.html), [CREATE EXTENSION](https://www.postgresql.org/docs/18/sql-createextension.html), [GiST Indexes](https://www.postgresql.org/docs/18/gist.html), [Indexes and ORDER BY](https://www.postgresql.org/docs/18/indexes-ordering.html).
- [PGlite](https://pglite.dev/docs/): PostgreSQL trong WASM và các extension dựng sẵn.
- Trong series: [PG-028 · GIN, GiST và BRIN theo workload](/articles/postgresql-gin-gist-brin-lua-chon), [PG-030 · Tối ưu truy vấn bằng bằng chứng](/articles/postgresql-thuc-hanh-toi-uu-truy-van) (chi phí ghi của index), [PG-059 · Nâng phiên bản và cutover](/articles/postgresql-upgrade-migration-cutover).
- Lab: `examples/toi-noi-postgresql/p06/extensions/run.mjs`, `p06/extensions/OUTPUT.txt`, `p06/README.md` (mục Giới hạn).

Bài trước: [PG-056 · Materialized view và dữ liệu báo cáo](/articles/postgresql-materialized-view-reporting). Bài tiếp theo: [PG-058 · Failover, split-brain và tính nhất quán khi đọc](/articles/postgresql-ha-failover-read-consistency)
