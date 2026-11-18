---
slug: postgresql-partition-pruning-retention
title: "Partition pruning lúc chạy và vòng đời dữ liệu GPS: tạo trước, DETACH CONCURRENTLY, đối soát rồi mới DROP"
summary: "Hết tháng 11, Tới Nơi phải bỏ GPS tháng 10 mà không làm khựng màn hình điều phối. Lab PostgreSQL 18.1 cho thấy generic plan vẫn tỉa partition lúc chạy (Subplans Removed: 1), bọc khoá phân vùng trong date_trunc làm mất pruning, partition tháng 12 tạo trước nhận dữ liệu, và trong một lần chạy DELETE 744 dòng sinh 59.736 byte WAL so với khoảng 12 KB cho DETACH CONCURRENTLY hay DROP."
category: postgresql
level: intermediate
tags: [postgresql, pg-054, partition-pruning, retention, detach-partition, prepared-statement, plan-cache-mode, wal, toi-noi]
language: vi
---

Ngày 1/12, chính sách lưu giữ của Tới Nơi đến hạn: GPS chi tiết chỉ giữ trên database chính trong một số tháng nhất định, tháng 10 phải rời đi. Ba nhóm cùng có yêu cầu với thao tác này:

- **Đội vận hành** muốn làm giữa ngày, không đợi cửa sổ bảo trì đêm, và không được làm khựng luồng ghi GPS hay màn hình điều phối. Lần trước họ chạy `DELETE` trên bảng thường, câu lệnh kéo dài, WAL tăng vọt, autovacuum bận cả buổi chiều.
- **Nhóm dữ liệu** yêu cầu lưu trữ tháng 10 ra kho lạnh trước khi xoá, và phải **chứng minh** bản lưu trữ đủ 744 điểm trước khi bản trên database biến mất.
- **Đội backend** lo rằng API lịch sử tài xế, chạy prepared statement qua connection pool, sau vài lần gọi sẽ dùng **generic plan** (kế hoạch dùng lại cho mọi giá trị tham số) và PostgreSQL không còn biết khoảng thời gian để tỉa partition. Bảng điều khiển BI thì lọc tháng bằng `date_trunc` và có vẻ chậm dần theo số tháng.

PG-051 đã cho thấy pruning khi điều kiện là hằng số. Bài này đi tiếp: pruning khi giá trị chỉ biết lúc chạy, cách viết làm mất pruning, và vòng đời partition — tạo trước, tách, đối soát, rồi mới xoá.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-054 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-jsonb-quan-he">PG-053 · JSONB và dữ liệu quan hệ</a> và <a href="/articles/postgresql-partition-co-ban">PG-051 · Partition là gì và dùng khi nào</a>; nên lướt <a href="/articles/postgresql-explain-ke-hoach-thuc-thi">PG-021 · Đọc EXPLAIN</a>, <a href="/articles/postgresql-lock-row-table">PG-033 · Khoá dòng, khoá bảng</a>, <a href="/articles/postgresql-vacuum-autovacuum-bloat">PG-044 · VACUUM và bloat</a> và <a href="/articles/postgresql-wal-checkpoint-durability">PG-045 · WAL và checkpoint</a>. Lab: <code>examples/toi-noi-postgresql/p06</code> (PostgreSQL 18.1 thật), kết quả trong <code>p06/OUTPUT.txt</code>. Sau bài này, bạn đọc được dấu hiệu pruning lúc chạy trong plan, viết bộ lọc thời gian giữ được pruning, và chạy một vòng retention có bước kiểm chứng trước khi xoá.</p>
</div>

## 1. Dữ liệu và cách chạy

```bash
cd examples/toi-noi-postgresql
$env:PG17_BIN = 'D:\pg17\pgsql\bin'   # PowerShell; runner kiểm tra biến này ngay từ đầu
npm run p06
```

Runner dựng cluster PostgreSQL 18.1 tạm, chỉ nghe localhost, bật `autovacuum` như hệ thống thật, chạy các bài Phần 6 theo thứ tự rồi xoá cluster. Dữ liệu từ `p06/fixture.sql` giống PG-051: bảng `gps` 1.464 điểm GPS theo giờ của 20 tài xế, phân vùng theo tháng UTC thành `gps_oct` (744 dòng) và `gps_nov` (720 dòng); bảng thường `gps_plain` chứa cùng dữ liệu, cùng khoá chính `(driver_id, seen_at)`. Phiên chạy ở múi giờ `Asia/Ho_Chi_Minh`.

Một điều cần biết khi đọc số: các phép kiểm của PG-054 chạy **sau** PG-053 trên cùng cluster, và PG-053 đã thêm vào `gps` một index GIN trên `attrs` cùng một cột generated `accuracy_m`. Vì vậy lúc bài này chạy, mỗi partition của `gps` có thêm một index và một cột mà `gps_plain` không có. Điều đó không ảnh hưởng tới kết luận về pruning, nhưng làm phép so sánh WAL ở mục 6 kém cân xứng hơn — bài sẽ nhắc lại ở đó.

Kết quả dưới đây chép từ `p06/OUTPUT.txt`; đã bỏ thụt lề runner thêm vào và rút ngắn dòng gạch ngang dưới tiêu đề `QUERY PLAN`.

## 2. Pruning xảy ra vào lúc nào

**Partition pruning** là việc PostgreSQL loại bỏ partition không thể chứa dòng thoả điều kiện, dựa trên biên của partition. Tài liệu PostgreSQL 18 nhấn mạnh hai điều. Thứ nhất, pruning **chỉ** dựa vào ràng buộc ngầm định từ khoá phân vùng, không dựa vào index — có hay không index trên `seen_at` không quyết định partition nào bị bỏ. Thứ hai, pruning có thể xảy ra ở **ba thời điểm**:

1. **Lúc lập kế hoạch** (planning). Giá trị trong điều kiện là hằng số, planner so với biên và bỏ partition ngay. Partition bị bỏ không xuất hiện trong plan. Đây là trường hợp PG-051: plan chỉ nhắc `gps_nov`.
2. **Lúc khởi tạo thực thi** (executor initialization). Giá trị chưa biết khi lập kế hoạch nhưng đã biết khi bắt đầu chạy — điển hình là tham số của prepared statement chạy bằng generic plan. Partition bị bỏ ở bước này cũng không hiện trong `EXPLAIN`, nhưng plan có dòng **`Subplans Removed: N`** cho biết đã bỏ bao nhiêu.
3. **Trong khi thực thi** (execution). Giá trị chỉ có khi đang chạy, ví dụ từ subquery hoặc từ phía ngoài của một nested loop. Phải đọc `loops` trong `EXPLAIN ANALYZE`; nhánh bị bỏ mọi lần hiện `(never executed)`. Lab **không chạy** trường hợp này.

Cả ba đều được bật bởi tham số `enable_partition_pruning` (mặc định `on`). Tắt nó đi để so sánh là thí nghiệm hữu ích — **không chạy trong lab**:

```sql
-- Không chạy trong lab: xem plan khi tắt pruning
SET enable_partition_pruning = off;
EXPLAIN (COSTS OFF) SELECT count(*) FROM gps
WHERE seen_at >= '2026-11-05 UTC' AND seen_at < '2026-11-06 UTC';
RESET enable_partition_pruning;
```

## 3. Prepared statement và generic plan: pruning lúc khởi tạo

**Prepared statement** là câu lệnh được phân tích một lần rồi chạy nhiều lần với tham số khác nhau (`$1`, `$2`). Nhiều driver và connection pool dùng nó ngầm. Khi chạy, PostgreSQL chọn giữa hai loại kế hoạch:

- **Custom plan**: lập lại cho từng lần chạy với giá trị tham số cụ thể. Planner thấy giá trị, nên pruning lúc lập kế hoạch vẫn hoạt động như với hằng số.
- **Generic plan**: lập một lần, không phụ thuộc giá trị, dùng lại cho mọi lần chạy. Tiết kiệm thời gian lập kế hoạch, nhưng planner không biết `$1` là ngày nào.

Tham số `plan_cache_mode` quyết định: `auto` (mặc định), `force_custom_plan`, `force_generic_plan`. Theo tài liệu `PREPARE`, ở `auto` năm lần chạy đầu dùng custom plan, sau đó PostgreSQL chuyển sang generic plan nếu chi phí ước lượng của nó không cao hơn đáng kể trung bình các custom plan. Nỗi lo của đội backend có cơ sở.

Lab ép tình huống đó cho tất định:

```sql
SET plan_cache_mode = force_generic_plan;
PREPARE by_window(timestamptz, timestamptz) AS
  SELECT count(*) FROM gps WHERE seen_at >= $1 AND seen_at < $2;
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF)
  EXECUTE by_window('2026-11-05 UTC', '2026-11-06 UTC');
```

```text
                         QUERY PLAN
-------------------------------------------------------------------
 Aggregate (actual rows=1.00 loops=1)
   Buffers: shared hit=16
   ->  Append (actual rows=24.00 loops=1)
         Buffers: shared hit=16
         Subplans Removed: 1
         ->  Seq Scan on gps_nov gps_1 (actual rows=24.00 loops=1)
               Filter: ((seen_at >= $1) AND (seen_at < $2))
               Rows Removed by Filter: 696
               Buffers: shared hit=16
 Planning:
   Buffers: shared hit=238
(11 rows)
```

Đọc từng dòng:

- `Filter: ((seen_at >= $1) AND (seen_at < $2))` — điều kiện vẫn mang **tham số**, không phải ngày cụ thể. Đây đúng là generic plan.
- Nút `Append` lẽ ra có hai nhánh (`gps_oct`, `gps_nov`) vì plan không biết trước khoảng thời gian. Nhưng chỉ còn `gps_nov`, kèm **`Subplans Removed: 1`**: lúc khởi tạo thực thi, PostgreSQL đã có giá trị `$1`, `$2`, so với biên và bỏ `gps_oct`. Runner assert đúng dòng này.
- `actual rows=24.00`: một ngày, 24 điểm. `Rows Removed by Filter: 696` = 720 − 24: trong `gps_nov`, PostgreSQL vẫn đọc hết rồi lọc.

Dòng cuối là bài học quan trọng: **pruning chọn partition, không chọn dòng trong partition**. Khoá chính là `(driver_id, seen_at)`; điều kiện chỉ có `seen_at` không dùng hiệu quả được index có cột đầu là `driver_id` (PG-025), nên bên trong `gps_nov` là `Seq Scan`. Với một partition tháng chứa hàng tỉ điểm, «chỉ còn một partition» vẫn có thể là quét hàng tỉ dòng. Nếu báo cáo theo thời gian không kèm tài xế là workload quan trọng, cần thêm index phù hợp (BRIN hoặc B-tree trên `seen_at`) — lab không thử, nên bài không khẳng định lựa chọn nào tốt hơn.

Tài liệu PostgreSQL còn ghi một chi tiết vận hành: partition bị bỏ ở giai đoạn khởi tạo **vẫn bị khoá** khi bắt đầu thực thi. Generic plan có pruning lúc chạy vẫn phải lấy khoá trên mọi partition trong plan; với hàng nghìn partition, đó là chi phí đáng kể. Lab chỉ có hai partition nên không quan sát được điều này.

Kết luận cho đội backend: generic plan **không** làm mất pruning, nó chuyển pruning sang lúc khởi tạo thực thi. Kiểm chứng bằng dòng `Subplans Removed` trong `EXPLAIN (ANALYZE) EXECUTE ...` với `force_generic_plan` trong một phiên thử.

## 4. Bọc khoá phân vùng trong hàm: pruning biến mất

Bảng điều khiển BI lọc «tháng 11» bằng cách so tháng của `seen_at`:

```sql
EXPLAIN (COSTS OFF) SELECT count(*) FROM gps
WHERE date_trunc('month', seen_at AT TIME ZONE 'UTC') = '2026-11-01';
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/postgresql-partition-pruning-retention/dbstudio-01.webp" alt="DBStudio chạy truy vấn của bước 054 trong p06/run.mjs và hiển thị kế hoạch thực thi gồm 4 toán tử" width="1400" height="484" loading="lazy"></div><figcaption>DBStudio · bước 054 của <code>p06/run.mjs</code> · Kế hoạch truy vấn, 4 toán tử</figcaption></figure>

Cả `gps_oct` và `gps_nov` đều bị quét, không có `Subplans Removed`. Con người thấy ngay `gps_oct` không thể có kết quả; planner thì không: biên partition nói về **giá trị của `seen_at`**, còn điều kiện nói về **kết quả của một biểu thức** trên `seen_at`, và PostgreSQL không suy ngược từ «tháng của biểu thức bằng X» ra khoảng của `seen_at`. `AT TIME ZONE 'UTC'` còn đổi `timestamptz` thành `timestamp without time zone` (xem hằng số trong `Filter`), thêm một lớp chuyển đổi.

Đây là cùng nguyên lý với index (PG-025): bọc cột trong hàm thì index thường trên cột đó không dùng được. Cách sửa là viết điều kiện **trực tiếp trên khoá phân vùng**, bằng khoảng nửa mở:

```sql
-- Cùng ý nghĩa «tháng 11 theo UTC», giữ được pruning
SELECT count(*) FROM gps
WHERE seen_at >= '2026-11-01 UTC' AND seen_at < '2026-12-01 UTC';
```

Dạng này chính là điều kiện PG-051 đã chứng minh chỉ chạm `gps_nov`. Với BI, chuyển đổi nên làm ở tầng sinh SQL: người dùng chọn tháng, ứng dụng tính hai mốc rồi truyền xuống.

### Tháng theo giờ Việt Nam không trùng partition theo UTC

Tháng 11 của bộ phận tài chính Tới Nơi là tháng theo giờ Việt Nam: từ `2026-11-01 00:00+07` tới `2026-12-01 00:00+07`, tức từ 17:00 UTC ngày 31/10 tới 17:00 UTC ngày 30/11. So với biên partition theo UTC, khoảng này **chạm cả `gps_oct`** (bảy giờ cuối tháng 10 UTC) **lẫn `gps_nov`**. Đó là hành vi đúng, không phải pruning hỏng. Lab không chạy câu này; theo cơ chế ở mục 2, plan phải có hai nhánh — hãy chạy `EXPLAIN` để xác nhận:

```sql
-- Không chạy trong lab: «tháng 11 giờ Việt Nam» vắt qua hai partition UTC
EXPLAIN (COSTS OFF) SELECT count(*) FROM gps
WHERE seen_at >= '2026-11-01 00:00+07' AND seen_at < '2026-12-01 00:00+07';
```

Nếu đa số truy vấn và chính sách retention của bạn tính theo tháng giờ Việt Nam, hãy đặt biên partition theo +07 ngay từ đầu (PG-016). Đổi biên sau khi đã có dữ liệu nghĩa là chuyển dữ liệu giữa các partition.

## 5. Tạo partition trước khi dữ liệu tới

PG-051 cho thấy điểm GPS lúc 00:00 UTC ngày 1/12 bị từ chối với `23514` vì chưa có partition tháng 12. Lab PG-054 tạo partition đó trước, rồi ghi lại đúng điểm ấy:

```sql
CREATE TABLE gps_dec PARTITION OF gps
  FOR VALUES FROM ('2026-12-01 UTC') TO ('2027-01-01 UTC');
INSERT INTO gps VALUES (1, '2026-12-01 UTC', '{"source":"phone","accuracy_m":5}');
SELECT count(*) FROM gps_dec;
```

```text
PASS 054 December partition prepared ahead receives: 1
```

Partition mới tự có index tương ứng với index của bảng cha (khoá chính, và GIN mà PG-053 đã thêm), không cần tạo tay.

`CREATE TABLE ... PARTITION OF` không miễn phí về khoá. Tài liệu PostgreSQL 18 ghi nó cần **`ACCESS EXCLUSIVE` trên bảng cha** — mức chặn cả `SELECT` (PG-033). Lệnh làm việc rất ít, nhưng phải xếp hàng sau mọi giao dịch đang dùng `gps`, và trong lúc chờ nó chặn mọi truy cập mới xếp sau nó: một báo cáo dài có thể làm luồng ghi GPS đứng lại theo. Vì vậy job tạo partition nên:

- chạy sớm (luôn có sẵn ít nhất một, hai tháng tương lai), không chạy sát nửa đêm cuối tháng;
- đặt `lock_timeout` ngắn và thử lại, để thà thất bại rồi chạy lại còn hơn xếp hàng chặn người khác;
- phát cảnh báo khi số partition tương lai xuống dưới ngưỡng.

Tài liệu gợi ý một đường ít khoá hơn: tạo bảng độc lập, thêm `CHECK` khớp đúng biên, rồi `ATTACH PARTITION` — thao tác chỉ cần `SHARE UPDATE EXCLUSIVE` trên bảng cha, và `CHECK` hợp lệ giúp PostgreSQL bỏ qua bước quét kiểm tra. **Không chạy trong lab**:

```sql
-- Không chạy trong lab
CREATE TABLE gps_jan (LIKE gps INCLUDING ALL);  -- gồm cả cột generated của PG-053
ALTER TABLE gps_jan ADD CONSTRAINT gps_jan_bound
  CHECK (seen_at >= '2027-01-01 UTC' AND seen_at < '2027-02-01 UTC');
SET lock_timeout = '2s';
ALTER TABLE gps ATTACH PARTITION gps_jan
  FOR VALUES FROM ('2027-01-01 UTC') TO ('2027-02-01 UTC');
ALTER TABLE gps_jan DROP CONSTRAINT gps_jan_bound;  -- tài liệu khuyên bỏ CHECK thừa sau khi gắn
```

Câu kiểm tra số partition tương lai, dùng cho cảnh báo — **không chạy trong lab**:

```sql
SELECT c.relname, pg_get_expr(c.relpartbound, c.oid) AS bound
FROM pg_inherits i JOIN pg_class c ON c.oid = i.inhrelid
WHERE i.inhparent = 'gps'::regclass
ORDER BY 1;
```

Công cụ như extension pg_partman tự động hoá việc tạo và bỏ partition; lab không dùng.

## 6. Bỏ một tháng: DELETE, DETACH, DROP

Lab đo **lượng WAL** mỗi cách sinh ra: đọc `pg_current_wal_insert_lsn()` trước và sau mỗi lệnh, lấy hiệu bằng `pg_wal_lsn_diff`. WAL (PG-045) là nhật ký mọi thay đổi phải đi qua, cũng là thứ replication gửi sang standby và archive phải lưu — chỉ báo trực tiếp của chi phí I/O và băng thông sao chép.

Ba lệnh, chạy lần lượt, mỗi lệnh là một giao dịch riêng:

```sql
-- Cách cũ: xoá tháng 10 khỏi bảng thường
DELETE FROM gps_plain WHERE seen_at < '2026-11-01 UTC';
-- Cách mới, bước 1: tách partition tháng 10 khỏi bảng cha
ALTER TABLE gps DETACH PARTITION gps_oct CONCURRENTLY;
-- (kiểm chứng ở mục 8)
-- Cách mới, bước 2: xoá bảng đã tách
DROP TABLE gps_oct;
```

```text
MEASURE 054 WAL bytes · DELETE 744 rows from gps_plain: 59736 · DETACH CONCURRENTLY: 12296 · DROP detached partition: 12680
PASS 054 gps_plain after DELETE: 720
```

Hai cách cùng một kết quả nghiệp vụ — bảng còn tháng 11 — nhưng:

- **`DELETE` 744 dòng: 59.736 byte**, trung bình khoảng 80 byte cho mỗi dòng xoá (phép chia đơn giản, không phải kích thước bản ghi WAL đo riêng). `DELETE` ghi một bản ghi WAL cho từng tuple bị đánh dấu xoá. Lượng WAL **tỉ lệ với số dòng**: tháng 10 có 744 điểm trong lab, nhưng có thể là hàng tỉ điểm ở hệ thống thật.
- **`DETACH ... CONCURRENTLY`: 12.296 byte** và **`DROP`: 12.680 byte**. Hai thao tác sửa **catalog** (danh mục hệ thống) chứ không chạm từng dòng; `DROP` xoá mục catalog và lên lịch xoá file khi commit. Theo cơ chế, WAL của chúng phụ thuộc số quan hệ và index liên quan, không phụ thuộc số dòng.

Phần không nằm trong con số: `DELETE` để lại **744 tuple chết** trong `gps_plain`; file không nhỏ đi, VACUUM phải dọn sau (PG-044) và cũng sinh WAL — lab không đo phần đó. `DROP` trả cả file về hệ điều hành, không để việc gì cho VACUUM.

Giới hạn của phép đo, theo `p06/README.md`: đây là **quan sát của một lần chạy**, không phải benchmark. WAL đo ở cấp cluster, có thể lẫn công việc nền (autovacuum đang bật), và có thể chứa full-page image nếu một checkpoint vừa xảy ra (PG-045). Hai bảng cũng không giống hệt nhau: `gps_oct` lúc đó mang thêm index GIN và cột generated từ PG-053, `gps_plain` thì không. Runner không đo thời gian, nên bài không khẳng định thao tác nào **nhanh** hơn bao nhiêu. Điều kết luận được là **hình dạng** chi phí: một bên tăng theo số dòng, một bên theo số đối tượng catalog.

## 7. DETACH CONCURRENTLY làm gì, và không được làm ở đâu

`DETACH PARTITION` biến một partition thành **bảng độc lập**: dữ liệu vẫn nguyên, chỉ thôi thuộc về bảng cha. Index đã gắn với index của bảng cha được tách ra, trigger nhân bản từ bảng cha bị gỡ. Tài liệu PostgreSQL 18 mô tả hai dạng:

- **Không có `CONCURRENTLY`**: cần khoá `ACCESS EXCLUSIVE` trên bảng cha — chặn mọi đọc và ghi vào `gps` trong lúc chờ và trong lúc thực hiện. Với luồng GPS 6.250 điểm mỗi giây, xếp hàng sau một báo cáo dài là sự cố.
- **Có `CONCURRENTLY`**: chỉ cần `SHARE UPDATE EXCLUSIVE` trên bảng cha, mức khoá không chặn `SELECT`, `INSERT`, `UPDATE`, `DELETE` thông thường.

Cái giá của `CONCURRENTLY` nằm ở cơ chế. Tài liệu `ALTER TABLE` mô tả nó dùng **hai giao dịch nội bộ**:

1. Giao dịch thứ nhất lấy `SHARE UPDATE EXCLUSIVE` trên cả bảng cha và partition, đánh dấu partition **đang được tách** (detach pending), rồi commit. Sau đó lệnh **chờ mọi giao dịch khác đang dùng bảng phân vùng** kết thúc.
2. Giao dịch thứ hai lấy `SHARE UPDATE EXCLUSIVE` trên bảng cha và `ACCESS EXCLUSIVE` trên **partition** (không phải bảng cha), rồi hoàn tất việc tách. Một ràng buộc `CHECK` sao chép đúng ràng buộc partition được thêm vào bảng vừa tách.

Từ đó suy ra các ràng buộc vận hành, đều theo tài liệu:

- **Không chạy trong transaction block.** Vì lệnh tự commit giữa chừng, `BEGIN; ALTER TABLE ... DETACH PARTITION ... CONCURRENTLY; COMMIT;` sẽ bị từ chối. Công cụ migration tự bọc mọi bước trong một giao dịch phải được cấu hình cho bước này chạy riêng. Lab chạy lệnh ở chế độ autocommit của `psql`.
- **Không được dùng nếu bảng phân vùng có default partition.** Thêm một lý do để cân nhắc kỹ trước khi tạo default partition (PG-051).
- **Mỗi bảng phân vùng chỉ có tối đa một partition đang chờ tách.**
- **Có thể bị gián đoạn.** Nếu lệnh bị huỷ hay phiên đứt sau giao dịch thứ nhất, partition ở trạng thái «đang tách». Hoàn tất bằng `ALTER TABLE gps DETACH PARTITION gps_oct FINALIZE;`. **Không chạy trong lab**; cột `inhdetachpending` của `pg_inherits` cho biết partition nào đang chờ.
- **Phải chờ giao dịch dài.** `CONCURRENTLY` không chặn người khác, nhưng phải đợi mọi giao dịch đang dùng `gps`; một phiên `idle in transaction` bỏ quên (PG-044) làm job retention treo. Lab không có phiên đồng thời lúc tách, nên không quan sát được thời gian chờ.

Ràng buộc `CHECK` được thêm vào có ích cho đường quay lại: nếu tách nhầm, `ATTACH PARTITION` lại bảng đó với đúng biên cũ có thể dùng `CHECK` này để bỏ qua bước quét kiểm tra (mục 5). **Không chạy trong lab**:

```sql
-- Không chạy trong lab: gắn lại partition vừa tách nhầm
ALTER TABLE gps ATTACH PARTITION gps_oct
  FOR VALUES FROM ('2026-10-01 UTC') TO ('2026-11-01 UTC');
```

## 8. Đối soát trước khi DROP

Giữa `DETACH` và `DROP` là cửa sổ để nhóm dữ liệu làm việc của họ. Lab kiểm hai điều ngay sau khi tách:

```sql
SELECT (SELECT count(*) FROM gps) || '/' || (SELECT count(*) FROM gps_oct);
SELECT relispartition FROM pg_class WHERE oid = 'gps_oct'::regclass;
```

```text
PASS 054 after DETACH: gps / detached gps_oct: 721/744
PASS 054 detached table is a normal table: f
```

- `gps` còn **721** dòng: 720 điểm tháng 11 cộng 1 điểm tháng 12 vừa ghi ở mục 5. Tháng 10 đã rời bảng cha — truy vấn qua `gps` không còn thấy nó.
- `gps_oct` vẫn còn đủ **744** dòng, đọc được như mọi bảng.
- `relispartition = f`: `gps_oct` giờ là bảng thường; `COPY`, `pg_dump -t gps_oct` hay `DROP` trên nó không còn liên quan tới khoá của `gps`.

Quy trình retention đề xuất cho Tới Nơi, mỗi bước có bằng chứng:

1. **Tách**: `DETACH PARTITION ... CONCURRENTLY`, có `lock_timeout` cho giai đoạn lấy khoá và giám sát giao dịch dài.
2. **Đối soát nguồn**: đếm dòng, `min`/`max` của `seen_at`, số tài xế, và một digest (ví dụ `md5` của dữ liệu đã sắp theo khoá chính) trên bảng đã tách.
3. **Lưu trữ**: xuất bảng ra kho lạnh bằng công cụ của đội (`COPY`, `pg_dump -t`, hoặc đẩy sang hệ phân tích).
4. **Đối soát đích**: đọc lại bản lưu trữ, so số dòng và digest với bước 2. Ghi kết quả vào biên bản retention.
5. **Chờ một khoảng an toàn** theo chính sách (để còn gắn lại được nếu chính sách bị áp dụng nhầm), rồi `DROP TABLE`.

Bước 2–4 **lab không chạy** ngoài phép đếm 744 (bài tập 3). `DROP` một bảng đã tách chỉ khoá bảng đó; còn `DROP TABLE` trực tiếp một partition **chưa tách**, theo tài liệu, cần `ACCESS EXCLUSIVE` trên bảng cha — thêm lý do để tách trước, xoá sau.

Về độ hạt: nếu chính sách là «giữ 90 ngày» mà partition theo tháng, chỉ tách khi **toàn bộ** tháng đã quá hạn, nên dữ liệu có thể ở lại thêm gần một tháng. Nếu điều đó vi phạm cam kết, cần partition theo ngày hoặc tuần, với cái giá nhiều quan hệ hơn (PG-051).

## 9. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách xử lý |
|---|---|---|
| Lo generic plan làm mất pruning | Generic plan không biết giá trị tham số lúc lập kế hoạch | Kiểm `Subplans Removed` trong `EXPLAIN (ANALYZE) EXECUTE` (lab: `Subplans Removed: 1`) |
| Truy vấn theo tháng quét mọi partition | Bọc khoá phân vùng trong `date_trunc`/`AT TIME ZONE` (lab: plan có cả `gps_oct` lẫn `gps_nov`) | Viết khoảng nửa mở trực tiếp trên `seen_at` |
| «Chỉ một partition» mà vẫn chậm | Pruning chọn partition, không chọn dòng; bên trong là `Seq Scan` (lab: 696 dòng bị lọc bỏ) | Thêm index phù hợp với điều kiện trong partition, đo trên dữ liệu thật |
| Đầu tháng, INSERT lỗi `23514` | Quên tạo partition tháng mới | Job tạo trước nhiều tháng, cảnh báo khi thiếu |
| Job tạo partition làm đứng luồng ghi | `CREATE TABLE ... PARTITION OF` cần `ACCESS EXCLUSIVE` trên bảng cha, xếp hàng sau giao dịch dài | `lock_timeout` + thử lại, chạy sớm, hoặc `ATTACH` với `CHECK` khớp biên |
| `DETACH ... CONCURRENTLY` bị từ chối | Chạy trong transaction block, hoặc bảng có default partition | Chạy lệnh riêng ở autocommit; xem lại nhu cầu default partition |
| Job retention treo không xong | `CONCURRENTLY` chờ giao dịch đang dùng bảng cha | Tìm phiên giữ giao dịch lâu; dùng `FINALIZE` nếu lệnh đã bị huỷ giữa chừng |
| Retention bằng `DELETE` làm WAL và autovacuum quá tải | WAL tỉ lệ số dòng, để lại tuple chết (lab: 59.736 byte cho 744 dòng) | Chuyển sang partition + `DETACH`/`DROP` |
| `DROP` ngay sau `DETACH`, rồi phát hiện bản lưu trữ thiếu | Không có bước đối soát | Đối soát nguồn và đích, giữ bảng tách một khoảng an toàn |

## 10. Đánh đổi

- **`DETACH CONCURRENTLY` hay `DETACH` thường.** `CONCURRENTLY` không chặn đọc/ghi nhưng không chạy được trong giao dịch, phải chờ mọi giao dịch đang dùng bảng, có thể để lại trạng thái «đang tách» cần `FINALIZE`. `DETACH` thường đơn giản, nguyên tử trong một giao dịch, nhưng cần `ACCESS EXCLUSIVE` trên bảng cha.
- **Generic hay custom plan.** Generic tiết kiệm thời gian lập kế hoạch và vẫn tỉa partition lúc khởi tạo, nhưng vẫn khoá mọi partition trong plan và có thể chọn cách truy cập kém hơn cho giá trị cụ thể. Custom tối ưu theo từng giá trị nhưng lập kế hoạch mỗi lần — với hàng nghìn partition, mỗi lần lập kế hoạch không rẻ.
- **Biên theo UTC hay giờ địa phương.** UTC đơn giản và trung lập; giờ địa phương khớp báo cáo và retention theo ngày nghiệp vụ. Chọn một và ghi vào data dictionary.
- **Giữ bảng đã tách lâu hay ngắn.** Lâu thì còn đường gắn lại, nhưng tốn dung lượng và dễ bị quên. Ngắn thì gọn, nhưng sai sót trong lưu trữ không còn cứu được từ database.

## 11. Bài tập

**Bài 1. Custom plan và generic plan.** Chạy lại câu `PREPARE by_window` của mục 3 hai lần: một lần với `plan_cache_mode = force_custom_plan`, một lần với `force_generic_plan`, cùng tham số `'2026-11-05 UTC'`, `'2026-11-06 UTC'`.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Hai plan thật bạn thu được. Ở custom plan, điều kiện <code>Filter</code> phải chứa giá trị thời gian cụ thể (không còn <code>$1</code>, <code>$2</code>) và plan chỉ có <code>gps_nov</code> mà không có dòng <code>Subplans Removed</code> — pruning đã xảy ra lúc lập kế hoạch. Ở generic plan, <code>Filter</code> mang tham số và có <code>Subplans Removed: 1</code>. Lời giải phải chỉ ra dòng nào trong output cho biết loại plan, không suy từ tên tham số đã đặt.</p>
</details>

**Bài 2. Sửa truy vấn BI.** Viết lại truy vấn `date_trunc` ở mục 4 cho «tháng 11 theo UTC» và cho «tháng 11 theo giờ Việt Nam», rồi chạy `EXPLAIN` cả hai trên `gps` còn đủ `gps_oct` và `gps_nov`.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Plan của bản UTC chỉ nhắc <code>gps_nov</code>; plan của bản giờ Việt Nam nhắc cả hai partition, và lời giải phải giải thích vì sao đó là đúng (khoảng 17:00 UTC 31/10 – 17:00 UTC 30/11 vắt qua biên). Cả hai phải trả cùng số dòng với một phép đếm đối chứng viết bằng <code>date_trunc</code> trên đúng múi giờ tương ứng — chứng minh việc viết lại không đổi kết quả, chỉ đổi cách truy cập.</p>
</details>

**Bài 3. Một vòng retention có biên bản.** Trên lab (hoặc bản sao của bạn), tách `gps_oct` bằng `DETACH PARTITION ... CONCURRENTLY`, xuất ra file bằng `COPY`, nạp file vào một bảng kiểm tra riêng, đối soát, rồi mới `DROP`.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Biên bản có: số dòng, <code>min</code>/<code>max</code> của <code>seen_at</code>, số tài xế và một digest (ví dụ <code>md5(string_agg(... ORDER BY driver_id, seen_at))</code>) tính trên bảng đã tách và trên bảng nạp lại từ file — hai bộ số phải khớp. Kèm <code>relispartition = f</code> trước khi xuất, và số dòng của <code>gps</code> trước/sau khi tách. Thêm một biến thể: mở một phiên giữ giao dịch đọc <code>gps</code> trước khi chạy <code>DETACH ... CONCURRENTLY</code>, ghi lại lệnh chờ ở đâu (<code>pg_stat_activity</code>, <code>pg_locks</code>) và điều gì xảy ra khi phiên kia kết thúc. Không chép số từ bài này.</p>
</details>

## 12. Đọc tiếp

- [PostgreSQL 18 · Partition Pruning](https://www.postgresql.org/docs/18/ddl-partitioning.html#DDL-PARTITION-PRUNING): ba thời điểm pruning, `Subplans Removed`, `(never executed)`.
- [PostgreSQL 18 · Table Partitioning](https://www.postgresql.org/docs/18/ddl-partitioning.html): bảo trì partition, khoá của `CREATE TABLE ... PARTITION OF`, `ATTACH` với `CHECK`, `DROP` và `DETACH`.
- [PostgreSQL 18 · ALTER TABLE](https://www.postgresql.org/docs/18/sql-altertable.html): `DETACH PARTITION [ CONCURRENTLY | FINALIZE ]`, `ATTACH PARTITION`.
- [PostgreSQL 18 · Query Planning: plan_cache_mode](https://www.postgresql.org/docs/18/runtime-config-query.html#GUC-PLAN-CACHE-MODE) và [enable_partition_pruning](https://www.postgresql.org/docs/18/runtime-config-query.html#GUC-ENABLE-PARTITION-PRUNING); [PREPARE](https://www.postgresql.org/docs/18/sql-prepare.html) về custom và generic plan.
- Liên quan trong series: [PG-051 · Partition là gì](/articles/postgresql-partition-co-ban), [PG-025 · Index nhiều cột, partial và expression](/articles/postgresql-index-nhieu-cot-partial-expression), [PG-044 · VACUUM và bloat](/articles/postgresql-vacuum-autovacuum-bloat), [PG-045 · WAL và checkpoint](/articles/postgresql-wal-checkpoint-durability), [PG-016 · Ngày giờ và múi giờ](/articles/postgresql-date-time-timezone).
- Lab: `examples/toi-noi-postgresql/p06/run.mjs` (khối PG-054), kết quả `p06/OUTPUT.txt`, giới hạn trong `p06/README.md`, cổng D2–D3 trong `p06/HO-SO-BAN-GIAO.md`.

Bài tiếp theo: [PG-055 · Logical replication và CDC](/articles/postgresql-logical-replication-cdc).
