---
slug: postgresql-partition-co-ban
title: "Partition là gì và khi nào bảng GPS của Tới Nơi nên chia theo tháng?"
summary: "Bảng GPS của Tới Nơi lớn thêm mỗi giờ, đội vận hành muốn bỏ dữ liệu cũ theo tháng còn điều phối viên tra theo tài xế. Lab PostgreSQL 18.1 đặt cùng 1.464 điểm lên bảng phân vùng và bảng thường: điểm biên 00:00 UTC rơi đúng gps_nov, EXPLAIN chỉ chạm một partition, lọc thiếu khoá phân vùng phải quét mọi tháng, INSERT ngoài biên báo 23514 và UNIQUE thiếu khoá phân vùng báo 0A000."
category: postgresql
level: beginner
tags: [postgresql, pg-051, partitioning, range-partition, partition-key, explain, ddl, toi-noi]
language: vi
---

Mỗi chiếc điện thoại tài xế của Tới Nơi gửi một điểm GPS sau vài giây. Hồ sơ Tới Nơi ghi khoảng 25.000 tài xế online lúc cao điểm, mỗi người một điểm sau mỗi 4 giây, tức khoảng **6.250 điểm mỗi giây**; nếu giữ nhịp đó suốt một giờ là hơn 22 triệu dòng. Bảng `gps` vì thế chỉ có một chiều: lớn lên. Ở buổi họp tháng, hai nhóm cùng đặt câu hỏi về nó, nhưng theo hai hướng khác nhau:

- **Đội vận hành** muốn giữ dữ liệu GPS chi tiết trong một số tháng nhất định rồi bỏ đi. Họ đã thử `DELETE ... WHERE seen_at < ...` trên bản sao và thấy câu lệnh chạy lâu, sinh nhiều WAL, để lại cả núi tuple chết cho VACUUM.
- **Điều phối viên** hỏi «tài xế 1 đã đi đâu trong mười phút vừa qua», hoặc «tài xế này chạy những đâu trong tháng 11». Họ không quan tâm tháng nào bị xoá; họ cần câu trả lời nhanh và đúng.

Có người đề xuất «partition bảng GPS theo tháng». Bài này dùng lab PostgreSQL 18.1 để tách hai thứ hay bị gộp làm một: **partition** (chia một bảng thành nhiều bảng con theo một khoá) và **index** (đường tắt để tìm dòng trong một bảng). Hai công cụ giải hai bài toán khác nhau; chọn khoá phân vùng sai có thể làm truy vấn của điều phối viên chậm đi.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-051 · Phần 6/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-thuc-hanh-runbook-phuc-hoi">PG-050 · Thực hành: bàn giao runbook vận hành và phục hồi</a>; nên lướt <a href="/articles/postgresql-explain-ke-hoach-thuc-thi">PG-021 · Đọc EXPLAIN</a>, <a href="/articles/postgresql-btree-index-co-ban">PG-023 · B-tree index</a> và <a href="/articles/postgresql-date-time-timezone">PG-016 · Ngày giờ và múi giờ</a>. Lab: <code>examples/toi-noi-postgresql/p06</code> (PostgreSQL 18.1 thật), kết quả trong <code>p06/OUTPUT.txt</code>. Sau bài này, bạn phân biệt được partition với index, đọc được plan để biết partition nào bị chạm, và chọn khoá phân vùng theo workload thay vì theo cảm giác «bảng to thì chia».</p>
</div>

## 1. Dữ liệu và cách chạy

```bash
cd examples/toi-noi-postgresql
$env:PG17_BIN = 'D:\pg17\pgsql\bin'   # PowerShell; xem p06/README.md
npm run p06
```

Runner dựng cluster PostgreSQL 18.1 tạm, chỉ nghe localhost, nạp `p06/fixture.sql`, chạy các phép kiểm của mười bài Phần 6 rồi xoá cluster. Biến `PG17_BIN` chỉ cần cho bài nâng phiên bản (PG-059), nhưng runner kiểm tra nó ngay từ đầu, nên phải đặt dù bạn chỉ quan tâm bài này. Cluster đặt `timezone = 'Asia/Ho_Chi_Minh'` như mọi lab trước, nên `timestamptz` hiện ra theo giờ +07.

Fixture tạo hai bảng chứa **cùng một dữ liệu**:

- `gps`: 1.464 điểm GPS theo giờ của 20 tài xế, từ 01/10 đến hết 30/11/2026 theo UTC, phân vùng theo tháng thành `gps_oct` và `gps_nov`. Cột `attrs jsonb` giữ nguồn và độ chính xác của điểm (PG-053 sẽ dùng).
- `gps_plain`: bản sao trên bảng thường, cùng khoá chính `(driver_id, seen_at)`.

Đặt hai bảng cạnh nhau là để so sánh công bằng: cùng dòng, cùng khoá chính, cùng `ANALYZE`, chỉ khác cách tổ chức. Dữ liệu **rất nhỏ** — một bảng vài trăm KB nằm gọn trong bộ nhớ đệm — nên lab không đo tốc độ. Điều nó chứng minh được là **những quan hệ nào bị chạm**, lỗi nào xảy ra, và ràng buộc nào PostgreSQL bắt buộc. Kết quả dưới đây chép từ `p06/OUTPUT.txt`; đã bỏ hai khoảng trắng thụt lề runner thêm vào và rút ngắn dòng gạch ngang dưới tiêu đề `QUERY PLAN`.

## 2. Partition là gì, và nó không phải là gì

**Partitioning** (phân vùng) trong PostgreSQL là chia một bảng logic thành nhiều bảng vật lý nhỏ hơn. Tài liệu PostgreSQL 18 gọi bảng logic là **partitioned table** (bảng phân vùng, ở đây là `gps`) và các bảng con là **partition** (`gps_oct`, `gps_nov`). Ba điều cần nắm ngay:

1. **Bảng cha không chứa dòng nào.** Nó là «vỏ» định nghĩa cột, ràng buộc và quy tắc chia. Ứng dụng vẫn `INSERT INTO gps`, `SELECT ... FROM gps`; PostgreSQL tự định tuyến dòng mới vào đúng partition và gom kết quả khi đọc.
2. **Khoá phân vùng** (partition key) là cột hoặc biểu thức dùng để quyết định dòng thuộc partition nào. Ở đây là `seen_at`.
3. **Phương pháp chia.** PostgreSQL hỗ trợ ba kiểu khai báo: **range** (theo khoảng liên tục, như tháng), **list** (theo danh sách giá trị rời rạc, như mã thành phố) và **hash** (theo giá trị băm, chia đều mà không theo ý nghĩa nghiệp vụ). Lab chỉ chạy range.

Hai kiểu còn lại trông như sau — **không chạy trong lab**, chỉ để nhận diện cú pháp:

```sql
-- list: mỗi thành phố một partition (không chạy trong lab)
CREATE TABLE trips_by_city (city text NOT NULL, id bigint NOT NULL, fare int)
  PARTITION BY LIST (city);
CREATE TABLE trips_hcm PARTITION OF trips_by_city FOR VALUES IN ('HCM');

-- hash: chia 4 phần theo driver_id (không chạy trong lab)
CREATE TABLE gps_hash (driver_id int NOT NULL, seen_at timestamptz NOT NULL)
  PARTITION BY HASH (driver_id);
CREATE TABLE gps_hash_0 PARTITION OF gps_hash FOR VALUES WITH (MODULUS 4, REMAINDER 0);
```

Partition **không** phải là index. Index (PG-023) là cấu trúc phụ trỏ vào dòng để tìm nhanh; bảng vẫn là một khối. Partition chia chính khối dữ liệu, mỗi phần là một bảng có file, index, thống kê riêng, được VACUUM riêng. Lợi ích chính của partition, theo đúng tài liệu PostgreSQL, là **quản lý theo phần**: thêm hay bỏ cả một partition là thao tác trên metadata, không phải xoá từng dòng. Lợi ích hiệu năng đọc có thể có, nhưng chỉ khi truy vấn **mang theo điều kiện trên khoá phân vùng** để PostgreSQL loại bỏ được partition không liên quan. Mục 4 và 5 cho thấy cả hai mặt.

## 3. Bảng GPS chia theo tháng

DDL trong `fixture.sql`:

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

Mỗi giờ một điểm, tài xế xoay vòng `1 + n % 20`. Câu kiểm đầu tiên đếm dòng theo `tableoid` — cột hệ thống cho biết dòng thực sự nằm trong bảng vật lý nào:

```sql
SELECT count(*) FILTER (WHERE tableoid = 'gps_oct'::regclass) || '/' ||
       count(*) FILTER (WHERE tableoid = 'gps_nov'::regclass)
FROM gps;
```

```text
PASS 051 rows per partition oct/nov: 744/720
```

744 = 31 ngày × 24 giờ, 720 = 30 × 24. Dòng không «chảy tràn» sang tháng bên cạnh, và tổng 1.464 khớp fixture.

### Biên nửa mở và câu hỏi «00:00 ngày 1/11 thuộc tháng nào?»

Range partition trong PostgreSQL dùng **khoảng nửa mở** (half-open range): `FROM` là cận dưới **bao gồm**, `TO` là cận trên **không bao gồm**. Vì vậy `gps_oct` nhận `seen_at >= 2026-10-01 00:00 UTC` và `< 2026-11-01 00:00 UTC`; đúng mốc 00:00 UTC ngày 1/11 thuộc `gps_nov`. Hai partition liền nhau chia chung một mốc mà không chồng lấn, không hở. Đây cũng là cách PG-016 khuyên viết bộ lọc ngày: `>=` mốc đầu, `<` mốc sau, không dùng `BETWEEN` hay `23:59:59`.

```sql
SELECT tableoid::regclass FROM gps WHERE seen_at = '2026-11-01 UTC';
```

```text
PASS 051 boundary point 2026-11-01 00:00 UTC lands in: gps_nov
```

Phiên lab chạy ở múi giờ `Asia/Ho_Chi_Minh`, nên PostgreSQL in mốc này thành `2026-11-01 07:00:00+07` — **cùng một thời điểm**. `timestamptz` lưu thời điểm tuyệt đối; múi giờ phiên chỉ ảnh hưởng cách đọc chữ vào và in chữ ra. Biên được viết kèm `UTC` nên không phụ thuộc phiên tạo bảng. Viết `'2026-11-01'` không kèm múi giờ thì PostgreSQL hiểu theo `timezone` của phiên — 00:00 giờ Việt Nam, sớm hơn biên UTC 7 tiếng, và không có lỗi nào báo. «Tháng theo UTC hay theo giờ Việt Nam» là quyết định nghiệp vụ phải ghi vào data dictionary; lab chọn UTC, PG-054 bàn hệ quả.

## 4. Một tài xế, một tháng: plan chạm những bảng nào?

Truy vấn điển hình của điều phối viên: mọi điểm của tài xế 1 trong tháng 11. Lab chạy cùng câu trên hai bảng, cả hai đều trả **36 dòng** (720 điểm chia đều cho 20 tài xế):

```sql
EXPLAIN (ANALYZE, BUFFERS, COSTS OFF, TIMING OFF, SUMMARY OFF)
SELECT * FROM gps
WHERE driver_id = 1 AND seen_at >= '2026-11-01 UTC' AND seen_at < '2026-12-01 UTC';
```

Trên bảng phân vùng `gps`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-partition-co-ban/dbstudio-07.webp" alt="DBStudio chạy truy vấn của bước 051 trong p06/run.mjs và hiển thị kế hoạch thực thi gồm 2 toán tử, có số liệu thực tế" width="1400" height="392" loading="lazy"></div><figcaption>DBStudio · bước 051 của <code>p06/run.mjs</code> · Kế hoạch truy vấn, 2 toán tử · ANALYZE</figcaption></figure>

Trên bảng thường `gps_plain`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-partition-co-ban/dbstudio-08.webp" alt="DBStudio chạy truy vấn của bước 051 trong p06/run.mjs và hiển thị kế hoạch thực thi gồm 2 toán tử, có số liệu thực tế" width="1400" height="392" loading="lazy"></div><figcaption>DBStudio · bước 051 của <code>p06/run.mjs</code> · Kế hoạch truy vấn, 2 toán tử · ANALYZE</figcaption></figure>

Cách đọc (nhắc lại PG-021): **Bitmap Index Scan** dùng index để thu danh sách vị trí dòng thoả điều kiện, rồi **Bitmap Heap Scan** đọc các page chứa những dòng đó. `Buffers: shared hit` là số page 8 KB đọc từ bộ nhớ đệm chung.

Điều quan trọng nhất nằm ở dòng đầu: plan của `gps` chỉ nhắc **`gps_nov`**. `gps_oct` hoàn toàn vắng mặt — runner còn assert điều đó (`doesNotMatch(/gps_oct/)`). Việc PostgreSQL loại bỏ partition không thể chứa kết quả gọi là **partition pruning** (tỉa partition). Ở đây pruning xảy ra lúc lập kế hoạch: hai mốc thời gian là hằng số, planner so chúng với biên của từng partition và thấy `gps_oct` không thể có dòng nào thoả. Index được dùng là `gps_nov_pkey` — khoá chính khai báo trên bảng cha được PostgreSQL tạo thành một index riêng trên **mỗi** partition.

Còn con số? `shared hit=11` so với `12`, `Heap Blocks: exact=9` so với `10`. Chênh một page **không chứng minh** partition nhanh hơn. B-tree `(driver_id, seen_at)` của bảng thường cũng nhảy thẳng tới tài xế 1 trong tháng 11 (`Index Searches: 1`) mà không cần biết tháng 10 tồn tại. **Với truy vấn đã có index phù hợp, partition không làm phép tìm kiếm thông minh hơn**: độ sâu B-tree tăng rất chậm theo số dòng (PG-023). Pruning có lợi lớn khi truy vấn phải **quét** một khoảng thời gian rộng — khi đó bỏ một partition là bỏ cả một file.

Dòng `Planning: Buffers: shared hit=186` so với `86` phải đọc cẩn thận. Runner chạy mỗi lệnh bằng một tiến trình `psql` mới, nên con số gồm cả việc một phiên lạnh nạp metadata (catalog) của bảng cha, hai partition và index của chúng — không phải chi phí lập kế hoạch ở trạng thái ổn định. Nhưng nó đúng hướng tài liệu cảnh báo: càng nhiều partition, planner và bộ nhớ mỗi phiên càng nhiều việc (mục 9).

## 5. Khi truy vấn không mang khoá phân vùng

Câu thứ hai của điều phối viên quên điều kiện thời gian: «cho tôi mọi điểm của tài xế 1».

```sql
EXPLAIN (COSTS OFF) SELECT * FROM gps WHERE driver_id = 1;
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-partition-co-ban/dbstudio-09.webp" alt="DBStudio chạy truy vấn của bước 051 trong p06/run.mjs và hiển thị kế hoạch thực thi gồm 5 toán tử" width="1400" height="530" loading="lazy"></div><figcaption>DBStudio · bước 051 của <code>p06/run.mjs</code> · Kế hoạch truy vấn, 5 toán tử</figcaption></figure>

Lần này lab dùng `EXPLAIN` không `ANALYZE` — chỉ xem kế hoạch, không chạy. Nút **Append** nối kết quả của nhiều nhánh con, mỗi nhánh một partition. Điều kiện chỉ có `driver_id`, mà biên partition nói về `seen_at`, nên planner không có căn cứ nào để bỏ partition nào: nó phải vào **cả hai**, mỗi nơi một lần dò index.

Với ba năm dữ liệu chia theo tháng, câu này thành 36 lần dò index trên 36 bảng; với bảng thường, nó là một lần dò. Đây là trường hợp **partition làm truy vấn tệ đi**. Bài học cho Tới Nơi:

- Câu «tài xế đã đi đâu trong mười phút vừa qua» phải viết kèm `seen_at >= now() - interval '10 minutes'`. Có điều kiện đó, PostgreSQL chỉ chạm partition tháng hiện tại (hoặc hai partition, nếu mười phút vắt qua ranh giới tháng).
- Màn hình «lịch sử tài xế» nên bắt buộc chọn khoảng thời gian, hoặc phân trang theo thời gian (PG-018), thay vì «lấy hết rồi lọc ở ứng dụng».
- Nếu workload chính là tra theo tài xế **không** kèm thời gian, khoá phân vùng theo tháng là lựa chọn sai cho workload đó; một index phù hợp trên bảng thường, hoặc chia theo kiểu khác, mới đúng. Chọn khoá phân vùng là chọn theo câu hỏi được hỏi nhiều nhất và thao tác bảo trì quan trọng nhất.

## 6. Dòng không có chỗ để vào

Partition range chỉ nhận dòng nằm trong biên đã khai báo. Lab thử ghi một điểm đúng 00:00 UTC ngày 1/12 khi chưa có partition tháng 12:

```sql
INSERT INTO gps VALUES (1, '2026-12-01 UTC', '{"accuracy_m":5}');
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-partition-co-ban/dbstudio-04.webp" alt="DBStudio chạy truy vấn của bước 051 trong p06/run.mjs và hiển thị thông báo lỗi" width="1057" height="500" loading="lazy"></div><figcaption>DBStudio · bước 051 của <code>p06/run.mjs</code> · lỗi</figcaption></figure>

SQLSTATE `23514` là `check_violation` — cùng mã với vi phạm `CHECK` ở PG-005, vì về bản chất mỗi partition mang một ràng buộc ngầm định theo biên của nó. Dòng `DETAIL` in giá trị khoá phân vùng theo múi giờ phiên (+07), một lần nữa nhắc rằng `07:00+07` chính là `00:00 UTC`. Không có dòng nào được ghi; với ứng dụng, đây là một lỗi ghi như mọi lỗi ràng buộc khác.

Với Tới Nơi, lỗi này nghĩa là **nửa đêm UTC ngày 1/12, mọi điểm GPS đều bị từ chối** nếu không ai tạo partition tháng 12. Có hai cách phòng:

1. **Tạo partition trước** bằng job định kỳ (ví dụ luôn có sẵn ít nhất hai tháng tương lai) và cảnh báo khi số partition tương lai xuống dưới ngưỡng. PG-054 chứng minh partition tạo trước nhận dữ liệu đúng như mong đợi.
2. **Default partition** — partition nhận mọi dòng không khớp partition nào. **Không chạy trong lab**:

```sql
-- Không chạy trong lab
CREATE TABLE gps_default PARTITION OF gps DEFAULT;
```

Default partition biến lỗi thành dữ liệu «lạc chỗ» nằm im lặng. Theo tài liệu `ALTER TABLE`, khi đã có default partition, mỗi lần tạo hay gắn partition mới PostgreSQL phải quét nó để chắc không có dòng thuộc partition mới, trong lúc giữ `ACCESS EXCLUSIVE` trên default partition. Nó còn chặn `DETACH PARTITION ... CONCURRENTLY` (PG-054). Với GPS — dữ liệu đến theo thời gian, tháng sắp tới hoàn toàn đoán trước được — tạo trước kèm cảnh báo minh bạch hơn. Điểm có `seen_at` vô lý (đồng hồ điện thoại sai thành năm 2030) nên bị chặn ở tầng tiếp nhận.

## 7. Ràng buộc duy nhất phải chứa khoá phân vùng

Khoá chính của `gps` là `(driver_id, seen_at)`, không phải một `id` tự tăng. Đó không phải sở thích thiết kế. Lab thử tạo một bảng phân vùng có `UNIQUE` chỉ trên `driver_id`:

```sql
CREATE TABLE gps_bad (driver_id int, seen_at timestamptz, UNIQUE (driver_id))
  PARTITION BY RANGE (seen_at);
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-partition-co-ban/dbstudio-05.webp" alt="DBStudio chạy truy vấn của bước 051 trong p06/run.mjs và hiển thị thông báo lỗi" width="1201" height="566" loading="lazy"></div><figcaption>DBStudio · bước 051 của <code>p06/run.mjs</code> · lỗi</figcaption></figure>

`0A000` là `feature_not_supported`. Tài liệu PostgreSQL 18 giải thích lý do: ràng buộc duy nhất trên bảng phân vùng được thực thi bằng một index riêng trên mỗi partition, và mỗi index chỉ thấy dòng trong partition của nó. PostgreSQL không có «index toàn cục» xuyên partition. Muốn bảo đảm không trùng giữa các partition, chính cấu trúc phân vùng phải bảo đảm điều đó — nghĩa là khoá phân vùng phải nằm trong ràng buộc: hai dòng trùng `(driver_id, seen_at)` chắc chắn có cùng `seen_at`, nên chắc chắn rơi vào cùng partition, nơi index cục bộ bắt được chúng. Tài liệu cũng nói khoá phân vùng không được là biểu thức hay lời gọi hàm nếu muốn có ràng buộc duy nhất.

Bảng GPS hợp với ràng buộc này vì «một tài xế, một thời điểm, một điểm» vốn là định danh tự nhiên. Bảng `trips` nếu chia theo `created_at` thì khoá chính phải thành `(id, created_at)` và mọi foreign key trỏ vào nó phải mang theo `created_at` — thay đổi lan rộng, thường là lý do mạnh để **không** chia bảng đó. Identity vẫn sinh `id` không trùng trong thực tế, nhưng đó là hành vi của bộ sinh số, không phải ràng buộc database kiểm tra.

## 8. Chia bảng không làm dữ liệu nhỏ đi

Runner đo tổng kích thước từng quan hệ bằng `pg_total_relation_size` (gồm heap, index và bảng TOAST đi kèm):

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-partition-co-ban/dbstudio-06.webp" alt="DBStudio chạy truy vấn của bước 051 trong p06/run.mjs và hiển thị kết quả 3 dòng" width="996" height="618" loading="lazy"></div><figcaption>DBStudio · bước 051 của <code>p06/run.mjs</code> · 3 dòng</figcaption></figure>

Hai partition cộng lại **320 kB**, lớn hơn **272 kB** của bảng thường chứa cùng dữ liệu. Trên dữ liệu cỡ này, mỗi quan hệ mang chi phí cố định: page đầu index, bảng TOAST riêng, page chưa đầy ở cuối mỗi file. Hai bộ chi phí cố định nặng hơn một bộ. Lab không tách con số này thành từng thành phần, nên bài không khẳng định phần chênh nằm ở đâu chính xác; điều kết luận được là **partition không phải công cụ tiết kiệm dung lượng**. Ở bảng hàng chục GB, chi phí cố định này không đáng kể, nhưng nó cũng không bao giờ âm.

Thứ thực sự thay đổi là **đơn vị quản lý**: «bỏ tháng 10» có thể là bỏ một file thay vì đánh dấu xoá 744 dòng trong file chung. PG-054 đo phép so sánh đó.

## 9. Khi nào partition giúp, khi nào không

Gom lại từ lab và tài liệu PostgreSQL 18 (mục Best Practices của Table Partitioning):

**Partition giúp khi:**

- **Dữ liệu có vòng đời theo khoá.** GPS, log, sự kiện: dữ liệu cũ bị bỏ hay lưu trữ theo thời gian. `DETACH`/`DROP` một partition thay cho `DELETE` hàng triệu dòng — không có tuple chết để VACUUM dọn (PG-044), lượng WAL không tỉ lệ với số dòng (PG-045, đo ở PG-054).
- **Truy vấn quan trọng luôn mang điều kiện trên khoá phân vùng** và phải quét rộng, nên pruning bỏ được phần lớn dữ liệu.
- **Bảo trì theo phần.** VACUUM, ANALYZE, `REINDEX` chạy trên từng partition nhỏ; partition tháng cũ không còn bị ghi nên gần như yên sau một lần VACUUM.

**Partition không giúp, hoặc gây hại khi:**

- **Bảng chưa đủ lớn.** Lab có 1.464 dòng chỉ để minh hoạ. Tài liệu PostgreSQL gợi ý cân nhắc khi bảng vượt bộ nhớ vật lý của máy chủ; dưới mức đó, index tốt trên bảng thường thường là đủ.
- **Truy vấn chính không có khoá phân vùng** — mục 5. Mỗi truy vấn phải đi qua mọi partition.
- **Quá nhiều partition.** Tài liệu nói planner xử lý tốt tới khoảng vài nghìn partition *nếu truy vấn điển hình tỉa được gần hết*; còn lại càng nhiều thì planning càng lâu, bộ nhớ càng nhiều, và mỗi phiên phải nạp metadata của mọi partition nó chạm. Chia GPS theo **giờ** sẽ có hơn 8.700 partition mỗi năm — quá vụn.
- **Ràng buộc duy nhất toàn cục là bắt buộc** mà khoá phân vùng không nằm trong đó — mục 7.
- **Cập nhật khoá phân vùng thường xuyên.** Tài liệu cho biết `UPDATE` làm khoá phân vùng ra khỏi biên sẽ **chuyển dòng** sang partition khác, tức một lần xoá cộng một lần chèn bên dưới. Lab không chạy trường hợp này; với GPS, `seen_at` không bao giờ đổi nên không thành vấn đề.

Những tính năng liên quan **lab không chạy**, nên bài không nêu kết quả: sub-partitioning (partition của partition), partitionwise join/aggregate (`enable_partitionwise_join`, `enable_partitionwise_aggregate`, mặc định tắt), extension pg_partman để tự động tạo/bỏ partition.

**Checklist quyết định cho bảng GPS của Tới Nơi:**

| Câu hỏi | Trả lời cho `gps` | Hệ quả |
|---|---|---|
| Có chính sách retention theo thời gian không? | Có, theo tháng | Ủng hộ range theo `seen_at` |
| Truy vấn chính có điều kiện thời gian không? | Điều phối: có, nếu viết đúng | Bắt buộc kèm khoảng thời gian trong API |
| Khoá duy nhất tự nhiên có chứa `seen_at` không? | `(driver_id, seen_at)` | Không cần đổi mô hình |
| Có ai cập nhật `seen_at` không? | Không | Không có row movement |
| Bao nhiêu partition sau 3 năm? | 36 theo tháng | Trong tầm planner xử lý tốt |
| Dữ liệu đến ngoài biên thì sao? | Có thể, nếu quên tạo tháng mới | Job tạo trước + cảnh báo (PG-054) |

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách xử lý |
|---|---|---|
| Nửa đêm đầu tháng, mọi INSERT GPS lỗi `23514` | Chưa tạo partition cho tháng mới (lab: `no partition of relation "gps" found for row`) | Job tạo trước nhiều tháng, cảnh báo theo số partition tương lai |
| Biên partition lệch 7 tiếng | Viết `'2026-11-01'` không kèm múi giờ, chạy từ phiên +07 | Luôn ghi múi giờ trong biên; kiểm điểm biên như lab |
| Truy vấn theo tài xế chậm dần theo số tháng | Không có điều kiện trên `seen_at`, plan là Append qua mọi partition | Bắt buộc khoảng thời gian; đọc `EXPLAIN` tìm partition bị chạm |
| Không tạo được khoá chính `id` trên bảng phân vùng | Ràng buộc duy nhất phải chứa khoá phân vùng (`0A000`) | Thêm khoá phân vùng vào khoá chính, hoặc không chia bảng đó |
| Kỳ vọng partition giảm dung lượng | Partition chỉ chia dữ liệu, mỗi quan hệ có chi phí cố định (lab: 320 kB so với 272 kB) | Đặt mục tiêu là vòng đời và pruning, không phải dung lượng |
| Kết luận «partition nhanh hơn» từ một plan nhỏ | Chênh 1 page (11 so với 12) không phải phép đo hiệu năng | So sánh ở dữ liệu và tải thật, đọc cả phần Planning |
| Chia theo giờ «cho chắc» | Hàng nghìn partition làm planning và bộ nhớ phiên tăng | Chọn độ hạt theo retention và lượng dữ liệu |

## 11. Đánh đổi

- **Quản lý vòng đời rẻ đổi lấy ràng buộc mô hình.** Bỏ một tháng dữ liệu thành thao tác metadata, nhưng khoá chính và mọi ràng buộc duy nhất phải mang khoá phân vùng, và không có index toàn cục.
- **Pruning đổi lấy kỷ luật viết truy vấn.** Truy vấn có điều kiện trên khoá phân vùng được lợi; truy vấn không có thì trả giá theo số partition. Partition làm sai lầm trong cách viết truy vấn trở nên đắt hơn.
- **Độ hạt mịn hay thô.** Partition theo ngày cho retention chính xác hơn và từng phần nhỏ hơn, nhưng nhiều quan hệ hơn để planner và job bảo trì quản lý. Theo tháng ít quan hệ hơn nhưng retention chỉ chính xác tới tháng.
- **Thêm việc vận hành.** Phải có job tạo partition, cảnh báo khi thiếu, quy trình tách và lưu trữ partition cũ.

## 12. Bài tập

**Bài 1. Chứng minh khoảng mười phút chỉ chạm một partition.** Viết câu «điểm của tài xế 1 trong mười phút trước `2026-11-15 10:00 UTC`» bằng khoảng nửa mở, chạy `EXPLAIN` trên `gps`. Lặp lại với mười phút trước `2026-11-01 00:05 UTC`.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Hai plan thật bạn thu được: plan thứ nhất chỉ nhắc <code>gps_nov</code>; plan thứ hai vắt qua ranh giới tháng nên phải nhắc cả <code>gps_oct</code> và <code>gps_nov</code> (một nút <code>Append</code>). Điều kiện phải viết bằng hằng số <code>timestamptz</code> có múi giờ rõ ràng, <code>&gt;=</code> mốc đầu và <code>&lt;</code> mốc cuối. Nếu dùng <code>now()</code> thay cho hằng số, hãy ghi lại plan khác đi thế nào — đó là chủ đề của PG-054, đừng đoán trước.</p>
</details>

**Bài 2. Hỏng biên vì múi giờ.** Trong một phiên `SET timezone = 'Asia/Ho_Chi_Minh'`, tạo một bảng phân vùng thử với biên viết `FROM ('2026-11-01') TO ('2026-12-01')` không kèm múi giờ, chèn điểm `'2026-10-31 18:00 UTC'` và kiểm tra nó rơi vào partition nào.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Kết quả <code>tableoid::regclass</code> của điểm đó trong bảng thử, cùng với biên thật PostgreSQL đã lưu (xem bằng <code>pg_get_expr(relpartbound, oid)</code> trên <code>pg_class</code>). Lời giải phải giải thích bằng chính output: biên được hiểu theo múi giờ phiên lúc tạo, nên điểm 18:00 UTC ngày 31/10 (01:00 giờ Việt Nam ngày 1/11) thuộc partition tháng 11 của bảng thử dù trong <code>gps</code> nó thuộc <code>gps_oct</code>. Dọn bảng thử sau khi làm.</p>
</details>

**Bài 3. Có nên chia bảng `trips`?** Áp checklist ở mục 9 cho bảng `trips` của Tới Nơi (khoá chính `id`, nhiều bảng khác tham chiếu tới nó, chuyến được tra theo `id` và theo khách).

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Một bảng trả lời từng câu hỏi của checklist kèm bằng chứng: các truy vấn chính (có hay không điều kiện trên khoá phân vùng đề xuất), danh sách foreign key trỏ vào <code>trips</code> phải đổi nếu khoá chính thêm cột thời gian, và chính sách retention thật. Nếu đề xuất chia, phải viết DDL khoá chính mới và chạy thử để thấy PostgreSQL chấp nhận (không gặp <code>0A000</code>); nếu đề xuất không chia, phải nêu phương án thay thế (index, lưu trữ chuyến cũ sang bảng khác) và điều kiện nào sẽ khiến bạn xét lại.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 18 · Table Partitioning](https://www.postgresql.org/docs/18/ddl-partitioning.html): phân vùng khai báo, giới hạn ràng buộc duy nhất, default partition, [Best Practices](https://www.postgresql.org/docs/18/ddl-partitioning.html#DDL-PARTITIONING-DECLARATIVE-BEST-PRACTICES) về số partition.
- [PostgreSQL 18 · Partition Pruning](https://www.postgresql.org/docs/18/ddl-partitioning.html#DDL-PARTITION-PRUNING).
- [PostgreSQL 18 · CREATE TABLE](https://www.postgresql.org/docs/18/sql-createtable.html): cú pháp `PARTITION BY` và `PARTITION OF`.
- [PostgreSQL 18 · Appendix A. Error Codes](https://www.postgresql.org/docs/18/errcodes-appendix.html): `23514`, `0A000`.
- Liên quan trong series: [PG-021 · Đọc EXPLAIN](/articles/postgresql-explain-ke-hoach-thuc-thi), [PG-023 · B-tree index](/articles/postgresql-btree-index-co-ban), [PG-025 · Index nhiều cột, partial và expression](/articles/postgresql-index-nhieu-cot-partial-expression), [PG-044 · VACUUM và bloat](/articles/postgresql-vacuum-autovacuum-bloat).
- Lab: `examples/toi-noi-postgresql/p06/run.mjs` (khối PG-051), `p06/fixture.sql`, kết quả `p06/OUTPUT.txt`, giới hạn trong `p06/README.md`.

Bài tiếp theo: [PG-052 · Primary, standby và replication lag](/articles/postgresql-replication-primary-standby). PG-054 sẽ quay lại bảng `gps` để đo pruning lúc chạy và vòng đời partition.
