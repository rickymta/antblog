---
slug: postgresql-vacuum-autovacuum-bloat
title: "VACUUM, autovacuum và bloat: số chuyến không tăng mà file bảng vẫn phình"
summary: "Bảng trips của Tới Nơi bị sửa 560 lần mỗi giây lúc cao điểm, dung lượng tăng nhanh hơn số chuyến. Lab PostgreSQL 18.1 cho thấy VACUUM dọn 2.500 tuple chết nhưng file vẫn 1.867.776 byte, một snapshot Repeatable Read bỏ quên giữ lại 500 tuple chết, và cách đọc ngưỡng kích hoạt autovacuum."
category: postgresql
level: intermediate
tags: [postgresql, pg-044, vacuum, autovacuum, bloat, mvcc, pgstattuple, maintenance, toi-noi]
language: vi
---

Sáng thứ Hai, kênh vận hành của Tới Nơi nhận cảnh báo dung lượng: file của bảng `trips` lớn thêm khá nhiều trong vài tuần, trong khi số chuyến chỉ tăng đều như mọi tháng. Mỗi chuyến đổi trạng thái hay sửa giá khoảng tám lần; lúc cao điểm bảng nhận khoảng **560 câu `UPDATE` mỗi giây** (hồ sơ Tới Nơi). Trong buổi họp có hai đề xuất: «chạy `VACUUM FULL` ngay cho file nhỏ lại» và «tắt autovacuum đi, nó ăn I/O lúc cao điểm». Rồi có người nhớ ra tối thứ Sáu ai đó mở một phiên báo cáo trong công cụ GUI, chạy một câu `SELECT` rồi để đó cả cuối tuần.

Cả ba chi tiết dẫn về cùng một cơ chế. MVCC (PG-032) không ghi đè dòng mà tạo phiên bản mới; phiên bản cũ phải được dọn sau, và chỉ dọn được khi không còn snapshot nào có thể cần nó. Bài này dùng lab PostgreSQL 18.1 để tách bốn chuyện hay bị gộp làm một: tuple chết, chỗ trống tái sử dụng được, dung lượng trả lại hệ điều hành, và «đường chân trời» (horizon) mà một giao dịch mở lâu giữ lại.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-044 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-pg-stat-activity-quan-sat">PG-043 · Quan sát phiên làm việc và truy vấn chậm</a> và <a href="/articles/postgresql-mvcc-snapshot">PG-032 · MVCC và snapshot</a>; nên lướt <a href="/articles/page-don-vi-doc-ghi">PG-022 · Page</a>, <a href="/articles/heap-ctid-va-primary-key">PG-024 · Heap, ctid và HOT</a>, <a href="/articles/postgresql-index-only-scan-visibility-map">PG-026 · Visibility map</a>. Lab: <code>examples/toi-noi-postgresql/p05</code> (PostgreSQL 18.1 thật), kết quả trong <code>p05/OUTPUT.txt</code>. Sau bài này, bạn đo được vòng đời tuple chết trước và sau VACUUM, tìm được phiên đang chặn việc dọn, và lập chính sách autovacuum theo bảng dựa trên bằng chứng.</p>
</div>

## 1. Dữ liệu và cách chạy

```bash
cd examples/toi-noi-postgresql
npm run p05
```

Runner dựng một cluster PostgreSQL 18.1 tạm, chỉ nghe localhost, chạy mười bài của Phần 5 rồi dọn sạch. Khác lab Phần 3–4 (tắt autovacuum để số đo ổn định), cluster Phần 5 **bật autovacuum** như hệ thống thật. Để kịp quan sát tuple chết trước khi worker dọn, riêng bảng thí nghiệm `ops.churn` được tạo với `autovacuum_enabled = false` và được `RESET` cuối thí nghiệm.

`ops.churn` có 5.000 dòng: `id` và một chuỗi 320 ký tự (`md5` lặp 10 lần) — đủ nhỏ để không bị TOAST (đẩy giá trị lớn ra bảng phụ), nên mọi byte nằm trong heap. Công cụ đo là extension **pgstattuple**: nó đọc **toàn bộ** bảng và đếm chính xác tuple sống, tuple chết và chỗ trống. Kết quả dưới đây chép từ `p05/OUTPUT.txt`, đã bỏ hai khoảng trắng thụt lề mà runner thêm vào.

## 2. Vì sao số dòng đứng yên mà file vẫn lớn

Từ các bài trước: file bảng là chuỗi **page** 8 KB (PG-022); mỗi phiên bản dòng — PostgreSQL gọi là **tuple** — nằm trong một page. `UPDATE` chèn phiên bản mới và đánh dấu bản cũ bằng `xmax`; `DELETE` chỉ đặt `xmax` (PG-032). Bản cũ vẫn chiếm chỗ chừng nào còn snapshot có thể thấy nó. Từ đó:

- **Tuple chết** (dead tuple): phiên bản đã bị xoá hoặc thay thế bởi giao dịch đã commit. «Chết» chưa có nghĩa là **dọn được** — nếu còn snapshot cũ hơn giao dịch xoá, nó là tuple «chết nhưng chưa được phép dọn».
- **Chỗ trống tái sử dụng**: byte đã giải phóng và được ghi vào **free space map** (FSM — bản đồ chỗ trống của bảng), để `INSERT`/`UPDATE` sau đặt phiên bản mới vào đó thay vì nối page mới vào cuối file.
- **Bloat** (phình): phần dung lượng vượt quá mức cần cho dữ liệu sống cộng một khoảng trống hợp lý. Một ít chỗ trống là **có lợi**: `UPDATE` cần chỗ để đặt phiên bản mới ngay trong page. Bloat chỉ thành vấn đề khi chỗ trống quá lớn, hoặc tiếp tục tăng vì việc dọn không theo kịp.

Với **HOT update** (PG-024), bước **dọn tỉa trang** (page pruning) có thể thu hồi bản cũ ngay khi page được truy cập lại, nên `trips` không phình tỉ lệ thuận với số `UPDATE`. Nhưng pruning chỉ làm trong một page: không xoá mục index, không cập nhật visibility map, không đóng băng tuple — những việc đó vẫn là của VACUUM.

## 3. VACUUM làm gì và không làm gì

Tài liệu PostgreSQL 18 (Routine Vacuuming) nêu bốn lý do cần VACUUM:

1. **Thu hồi chỗ** của tuple chết đã dọn được, ghi vào FSM, và xoá các mục index trỏ tới chúng.
2. **Cập nhật thống kê** cho planner — khi chạy kèm `ANALYZE`.
3. **Cập nhật visibility map** (PG-026): bật bit all-visible cho page sạch, giúp index-only scan và giúp lần VACUUM sau bỏ qua page đó.
4. **Chống quay vòng mã giao dịch** (transaction ID wraparound) bằng cách đóng băng tuple cũ (mục 8).

`VACUUM` thường (không `FULL`) lấy khoá `SHARE UPDATE EXCLUSIVE`: không chặn `SELECT`, `INSERT`, `UPDATE`, `DELETE`; chỉ xung đột với những thao tác như `ALTER TABLE`, `CREATE INDEX CONCURRENTLY` hoặc một VACUUM khác trên cùng bảng.

Điều VACUUM thường **không** hứa: thu nhỏ file. Tài liệu nói nó chỉ trả dung lượng cho hệ điều hành khi các page **ở cuối bảng** trống hoàn toàn và lấy được khoá độc quyền một cách dễ dàng. Chỗ trống ở giữa file chỉ vào FSM để dùng lại. Viết lại cả bảng cho gọn là việc của `VACUUM FULL` — một công cụ khác hẳn, với cái giá khác hẳn (mục 7).

## 4. Thí nghiệm 1: xoá nửa bảng rồi VACUUM

Runner chạy từng câu như một lệnh riêng, không có giao dịch bao ngoài:

```sql
CREATE EXTENSION pgstattuple;
CREATE TABLE ops.churn(id int, payload text) WITH (autovacuum_enabled = false);
INSERT INTO ops.churn
SELECT n, repeat(md5(n::text), 10) FROM generate_series(1, 5000) n;

DELETE FROM ops.churn WHERE id % 2 = 0;          -- xoá 2.500 dòng id chẵn

SELECT table_len, tuple_count AS live, dead_tuple_count AS dead,
       round(dead_tuple_percent) AS dead_pct,
       free_space, round(free_percent) AS free_pct
FROM pgstattuple('ops.churn');

VACUUM (ANALYZE) ops.churn;
-- chạy lại câu pgstattuple ở trên
```

Sau `DELETE`, trước VACUUM:

```text
 table_len | live | dead | dead_pct | free_space | free_pct
-----------+------+------+----------+------------+----------
   1867776 | 2500 | 2500 |       47 |      82304 |        4
(1 row)
```

Sau `VACUUM (ANALYZE)`:

```text
 table_len | live | dead | dead_pct | free_space | free_pct
-----------+------+------+----------+------------+----------
   1867776 | 2500 |    0 |        0 |     963216 |       52
(1 row)
```

- **`table_len` không đổi: 1.867.776 byte** = đúng 228 page × 8.192 byte (cổng `PASS 044 VACUUM keeps file size: 1867776`).
- **`dead` 2.500 → 0**: mọi tuple chết đều dọn được, vì không có giao dịch nào khác đang mở.
- **`free_pct` 4 % → 52 %**: khoảng 860 KB đổi từ «tuple chết» thành «chỗ trống». Byte vẫn thuộc về file, chỉ đổi vai trò.
- **`dead_pct` 47 %, không phải 50 %**: phần trăm tính trên `table_len`, mà mỗi page còn header, con trỏ dòng (line pointer) và chút chỗ trống sẵn có.

Vì sao file không nhỏ đi? Dòng `id` chẵn nằm **xen kẽ** trên cả 228 page: page nào cũng mất khoảng một nửa, không page cuối nào trống hẳn, nên VACUUM không có gì để cắt. Nếu xoá nửa sau (`id > 2500`), các page cuối có thể trống hẳn và VACUUM có cơ hội cắt đuôi file — lab **không chạy** biến thể đó, nên bài không khẳng định kết quả cụ thể.

Lab cũng **chưa chứng minh** rằng chỗ trống 52 % sẽ được `INSERT` sau dùng lại. Đó là cơ chế FSM theo tài liệu; bài tập 1 yêu cầu bạn tự đo.

## 5. Thí nghiệm 2: một snapshot bỏ quên giữ lại tuple chết

Đây là chuyện «phiên báo cáo bỏ quên cuối tuần». Lab mở phiên **H** (holder) giữ snapshot Repeatable Read, rồi xoá và VACUUM từ phiên chính **M**:

| Bước | Phiên | Câu lệnh | Kết quả |
|---|---|---|---|
| T1 | H | `BEGIN ISOLATION LEVEL REPEATABLE READ; SELECT count(*) FROM ops.churn;` | snapshot chụp ở câu `SELECT`; H ngồi yên |
| T2 | M | `DELETE FROM ops.churn WHERE id <= 1000;` | xoá 500 dòng (id lẻ 1…999 còn lại sau thí nghiệm 1) |
| T3 | M | `VACUUM (VERBOSE) ops.churn;` | chạy xong, không lỗi |
| T4 | M | `pgstattuple`, `pg_stat_activity` | 500 tuple chết vẫn còn |
| T5 | H | `COMMIT;` | snapshot được thả |
| T6 | M | `VACUUM (VERBOSE) ops.churn;` | dọn 500 |

Hai dòng runner giữ lại từ báo cáo `VACUUM VERBOSE` ở T3:

```text
tuples: 0 removed, 2494 remain, 500 are dead but not yet removable
removable cutoff: 774, which was 1 XIDs old when operation ended
```

`pgstattuple` sau T3 đếm đúng **500** tuple chết (cổng `PASS 044 old snapshot keeps dead tuples: 500`). VACUUM không lỗi, không chờ khoá — nó **chạy trọn** và báo thẳng rằng 500 tuple «đã chết nhưng chưa được phép dọn».

`removable cutoff` chính là **horizon**: VACUUM chỉ được dọn tuple bị xoá bởi giao dịch có xid nhỏ hơn 774. H chỉ đọc nên không có xid riêng (PG-032), nhưng snapshot của nó có `xmin` — mọi giao dịch từ mốc đó trở đi có thể «chưa xảy ra» trong mắt H. `DELETE` ở T2 nhận xid từ mốc đó trở đi, nên các phiên bản nó xoá phải được giữ cho H. «1 XIDs old»: lúc VACUUM kết thúc, horizon chỉ cách xid kế tiếp đúng một — chỉ có giao dịch `DELETE` chen giữa.

Câu ở T4 hỏi `pg_stat_activity` về H:

```text
   s    |        state        | has_xmin |              query
--------+---------------------+----------+---------------------------------
 holder | idle in transaction | t        | SELECT count(*) FROM ops.churn;
(1 row)
```

`idle in transaction` và `backend_xmin` khác NULL (`has_xmin = t`) — hai dấu hiệu PG-043 đã dạy đọc. Cột `query` là câu **cuối cùng** phiên đã chạy, không phải câu đang chạy. Sau khi H `COMMIT`, VACUUM lần hai:

```text
tuples: 500 removed, 1592 remain, 0 are dead but not yet removable
```

và `pgstattuple` báo 0 tuple chết (`PASS 044 dead tuples after holder commits: 0`).

Lưu ý: `remain` (2494, rồi 1592) **không** khớp số dòng sống thật — sau T2 bảng có 2.000 dòng sống. Theo hiểu biết của tôi về mã nguồn, đây là **ước lượng** cho cả bảng: VACUUM bỏ qua page đã all-visible và ngoại suy phần không quét từ thống kê cũ. Muốn số chính xác, dùng `pgstattuple` như lab.

Ba bài học vận hành:

1. **«VACUUM chạy xong» không có nghĩa «đã dọn».** Đọc `removed` và `dead but not yet removable`, đừng chỉ nhìn exit code.
2. **Một phiên bỏ quên đủ chặn cả database.** Horizon tính theo snapshot cũ nhất trong database, nên giữ tuple chết của **mọi** bảng, không chỉ bảng phiên kia đã đọc. Với 560 `UPDATE`/giây, mỗi phút phiên kia còn mở là hàng chục nghìn phiên bản không dọn được.
3. **Mức cô lập quyết định ai giữ snapshot.** Ở Read Committed, snapshot sống trong từng câu; phiên Read Committed chỉ đọc rồi ngồi yên thường không giữ `backend_xmin`. Nhưng nếu nó đã **ghi** (có xid), xid đó giữ horizon. Lab dùng Repeatable Read để giữ snapshot một cách tất định.

Tài liệu còn kể các nguồn giữ horizon khác: replication slot có `xmin` không được tiêu thụ, giao dịch đã `PREPARE TRANSACTION` mà chưa chốt, standby bật `hot_standby_feedback` đang chạy truy vấn dài. Bộ câu kiểm sau **không chạy trong lab**, dùng làm khởi điểm cho runbook:

```sql
SELECT pid, usename, state, age(backend_xmin) AS xmin_age,
       now() - xact_start AS xact_age, left(query, 60) AS last_query
FROM pg_stat_activity
WHERE backend_xmin IS NOT NULL
ORDER BY age(backend_xmin) DESC LIMIT 5;

SELECT slot_name, active, xmin, catalog_xmin FROM pg_replication_slots;
SELECT gid, prepared, owner FROM pg_prepared_xacts;
```

Phòng ngừa: `idle_in_transaction_session_timeout` (lab 043 đã chứng minh phiên bị cắt), đặt theo role cho tài khoản báo cáo và công cụ GUI.

## 6. Autovacuum: ai chạy VACUUM, và khi nào

**Autovacuum** gồm một tiến trình launcher thức dậy định kỳ và các worker (mặc định tối đa 3, `autovacuum_max_workers`) đi VACUUM/ANALYZE bảng vượt ngưỡng. Lab đọc cấu hình cluster:

```text
              name               |  setting
---------------------------------+-----------
 autovacuum                      | on
 autovacuum_naptime              | 60
 autovacuum_vacuum_max_threshold | 100000000
 autovacuum_vacuum_scale_factor  | 0.2
 autovacuum_vacuum_threshold     | 50
(5 rows)
```

Lab đặt `autovacuum = on`; bốn giá trị còn lại là mặc định PostgreSQL 18. `autovacuum_naptime = 60` giây là chu kỳ launcher ghé mỗi database.

**Ngưỡng theo tuple chết.** Tài liệu PostgreSQL 18:

```text
ngưỡng vacuum = min( autovacuum_vacuum_max_threshold,
                     autovacuum_vacuum_threshold
                     + autovacuum_vacuum_scale_factor × reltuples )
```

`reltuples` là số tuple ước lượng trong `pg_class`. Bảng được VACUUM khi số tuple chết tích luỹ từ lần VACUUM trước (thống kê tích luỹ, cột `n_dead_tup`) vượt ngưỡng. **`autovacuum_vacuum_max_threshold` mới có từ PostgreSQL 18**; trước đó công thức không có trần.

Tính thử cho `trips` **20 triệu dòng** (con số minh hoạ, không phải dữ liệu lab): `min(100.000.000, 50 + 0,2 × 20.000.000)` = **4.000.050** tuple chết. Trần chỉ bắt đầu có tác dụng khi `reltuples` vượt khoảng 500 triệu. Nếu mỗi `UPDATE` để lại một tuple chết mà pruning chưa thu hồi, 560 `UPDATE`/giây cần khoảng **7.100 giây — gần hai giờ** cao điểm mới chạm ngưỡng. HOT pruning dọn bớt một phần, nên đây là cận trên thô, nhưng đủ thấy tỉ lệ 20 % trên bảng lớn nghĩa là «chờ rất lâu».

**Ngưỡng theo dòng chèn.** Bảng chỉ chèn như `trip_events` (~16 triệu dòng/ngày) hầu như không có tuple chết, nhưng vẫn cần VACUUM để cập nhật visibility map và đóng băng. Từ PostgreSQL 13 có thêm ngưỡng chèn `autovacuum_vacuum_insert_threshold` (mặc định 1000) và `autovacuum_vacuum_insert_scale_factor` (0,2); tài liệu PostgreSQL 18 ghi phần tỉ lệ được nhân với tỉ lệ page **chưa đóng băng** của bảng.

**Ngưỡng ANALYZE** tách riêng: `autovacuum_analyze_threshold` (50) + `autovacuum_analyze_scale_factor` (0,1) × `reltuples`, so với số dòng đã đổi từ lần ANALYZE trước (`n_mod_since_analyze`). [PG-027](/articles/postgresql-statistics-analyze-cardinality) đã cho thấy thống kê cũ làm planner ước lượng sai thế nào.

**Ghi đè theo bảng.** Mọi tham số trên đều đặt riêng được qua storage parameter. Lab dùng đúng một: `autovacuum_enabled = false` lúc tạo `ops.churn`, rồi `ALTER TABLE ops.churn RESET (autovacuum_enabled);`. Tài liệu ghi rõ: kể cả khi tắt theo bảng, autovacuum **vẫn** chạy khi cần chống quay vòng. Tắt theo bảng là công cụ thí nghiệm, không phải cách giảm tải.

Với `trips`, hướng hợp lý là **hạ tỉ lệ cho riêng bảng sửa nhiều** thay vì đụng cấu hình toàn cluster. Ví dụ **chưa chạy trong lab**, số chỉ để minh hoạ:

```sql
-- Ngưỡng ≈ 50 + 0,01 × 20.000.000 = 200.050 tuple chết
ALTER TABLE trips SET (autovacuum_vacuum_scale_factor = 0.01,
                       autovacuum_analyze_scale_factor = 0.02);
SELECT relname, reloptions FROM pg_class WHERE relname = 'trips';
```

Cùng giả định thô, 200.050 tuple chết ứng với khoảng 6 phút cao điểm: VACUUM chạy thường hơn, mỗi lần ít việc hơn. Hiệu quả phải **đo lại** sau khi đổi: `n_dead_tup`, thời lượng mỗi lần autovacuum, I/O.

**Tốc độ worker.** Autovacuum tự hãm: cộng «điểm chi phí» cho page đọc/ghi, chạm `vacuum_cost_limit` (200) thì ngủ `autovacuum_vacuum_cost_delay` (2 ms). Chạy liên tục mà `n_dead_tup` vẫn tăng thì có thể đang hãm quá tay — lab **không thử** điều chỉnh nhóm này nên bài không đưa số khuyến nghị. Autovacuum (trừ lần chống quay vòng) cũng **tự nhường**: chặn một yêu cầu khoá xung đột như `ALTER TABLE` thì nó bị huỷ.

**Quan sát định kỳ** — câu **chưa chạy trong lab** (lab dùng `pgstattuple` để có số chính xác):

```sql
SELECT relname, n_live_tup, n_dead_tup, n_mod_since_analyze,
       last_autovacuum, last_autoanalyze, autovacuum_count
FROM pg_stat_user_tables ORDER BY n_dead_tup DESC LIMIT 10;

SELECT pid, relid::regclass, phase, heap_blks_total, heap_blks_scanned
FROM pg_stat_progress_vacuum;
```

`n_live_tup`/`n_dead_tup` là **ước lượng** có độ trễ, dùng để xem xu hướng; `n_dead_tup` cao ngay sau đợt xoá lớn chưa đủ để nói autovacuum hỏng. Đặt `log_autovacuum_min_duration` để mỗi lần autovacuum dài hơn ngưỡng ghi log với thông tin như `VACUUM VERBOSE`, kể cả «dead but not yet removable».

## 7. Khi nào thật sự cần lấy lại dung lượng

Trước khi «chạy `VACUUM FULL` ngay», trả lời ba câu:

1. **Có phải bloat thật không?** Bảng sửa nhiều luôn giữ một tỉ lệ chỗ trống cho phiên bản mới; nếu tỉ lệ ổn định, thu nhỏ file chỉ khiến bảng phình lại mức cũ sau vài ngày. Cần xử lý khi chỗ trống **tăng không dừng**, hoặc sau sự kiện một lần (xoá hàng loạt, phiên bỏ quên cả tuần) để lại khoảng trống không bao giờ dùng hết.
2. **Đã gỡ nguyên nhân chưa?** Horizon còn bị giữ hoặc autovacuum không theo kịp thì viết lại bảng xong sẽ phình lại.
3. **Chịu được cái giá không?** Theo tài liệu, `VACUUM FULL` viết bảng sang file mới, giữ khoá `ACCESS EXCLUSIVE` suốt quá trình — **chặn cả `SELECT`** — và cần thêm đĩa cỡ bản sao mới. Trên `trips` lúc cao điểm, đó là sự cố tự gây ra.

Các lựa chọn, **không lựa chọn nào chạy trong lab**:

| Cách | Làm gì | Cái giá chính |
|---|---|---|
| Để yên, sửa autovacuum | Chỗ trống được dùng lại dần | File không nhỏ đi |
| `VACUUM FULL` / `CLUSTER` | Viết lại bảng gọn | `ACCESS EXCLUSIVE` suốt thời gian chạy, cần thêm đĩa tạm |
| Extension `pg_repack` | Viết lại bảng trực tuyến, khoá ngắn ở đầu và cuối | Ngoài lõi PostgreSQL, cần cài và thử riêng |
| `REINDEX CONCURRENTLY` | Dựng lại index phình | Tốn I/O, không xử lý heap |

Đo trước khi quyết: `pgstattuple` (chính xác nhưng quét cả bảng — đừng chạy mỗi phút trên bảng lớn), `pgstattuple_approx` (bỏ qua page all-visible, gần đúng), `pgstatindex` cho B-tree.

## 8. Đóng băng và quay vòng mã giao dịch

Mã giao dịch (xid) chỉ có 32 bit và được so sánh theo vòng tròn: mỗi xid thấy khoảng hai tỉ xid «quá khứ» và hai tỉ «tương lai». Tuple giữ `xmin` cũ mãi sẽ có lúc rơi sang «tương lai» và trở nên vô hình. VACUUM ngăn điều đó bằng cách **đóng băng** (freeze) tuple đủ cũ: đánh dấu nó hiển thị với mọi giao dịch. Khi `age(relfrozenxid)` của bảng vượt `autovacuum_freeze_max_age` (mặc định 200 triệu), autovacuum ép một lần VACUUM chống quay vòng — kể cả khi bảng tắt autovacuum, và lần này không tự nhường khoá.

Lab **không** mô phỏng wraparound (cần hàng trăm triệu giao dịch). Câu theo dõi nên có trong dashboard, chưa chạy trong lab:

```sql
SELECT datname, age(datfrozenxid) AS xid_age FROM pg_database ORDER BY 2 DESC;
```

Horizon ở mục 5 chặn đóng băng giống như chặn dọn. Vì vậy giao dịch mở hàng tuần là sự cố, không chỉ là chuyện dung lượng.

## 9. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách xử lý |
|---|---|---|
| VACUUM xong, file không nhỏ đi | VACUUM thường chỉ đưa chỗ trống vào FSM, chỉ cắt page trống ở cuối (lab: 1.867.776 → 1.867.776) | Đo chỗ trống; chỉ viết lại bảng khi bloat là vấn đề thật |
| VACUUM xong mà tuple chết không giảm | Snapshot cũ giữ horizon (lab: `500 are dead but not yet removable`) | Tìm `backend_xmin` cũ nhất, slot, prepared xact; kết thúc nguồn giữ |
| `n_dead_tup` cao, kết luận autovacuum hỏng | Thống kê là ước lượng; có thể chưa tới ngưỡng hoặc bị horizon chặn | Đối chiếu `last_autovacuum`, log autovacuum, `pg_stat_progress_vacuum` |
| Tắt autovacuum để giảm I/O | Dồn việc sang tương lai; thống kê cũ; rủi ro wraparound | Hạ ngưỡng theo bảng, điều chỉnh hãm tốc có đo đạc |
| `VACUUM FULL` lúc cao điểm | `ACCESS EXCLUSIVE` chặn mọi truy cập bảng | Gỡ nguyên nhân trước; cửa sổ bảo trì hoặc công cụ trực tuyến đã thử |
| `autovacuum_enabled = false` rồi quên | Bảng chỉ còn được dọn khi chống quay vòng | `RESET` như lab; rà `reloptions` định kỳ |

## 10. Đánh đổi

- **Ngưỡng thấp hay cao.** Thấp: VACUUM thường xuyên, mỗi lần ngắn, bảng gọn, nhưng I/O nền đều đặn hơn. Cao: ít lần, mỗi lần dài, tuple chết tích nhiều, index-only scan (PG-026) kém đi giữa hai lần.
- **Hãm worker mạnh hay nhẹ.** Mạnh bảo vệ truy vấn người dùng nhưng có thể không theo kịp; nhẹ dọn nhanh nhưng tranh I/O giờ cao điểm.
- **Giữ chỗ trống hay thu nhỏ file.** Chỗ trống là bộ đệm cho `UPDATE` và HOT; thu nhỏ đổi lấy khoá nặng hoặc công cụ ngoài lõi, rồi bảng lại phình về trạng thái cân bằng.
- **Snapshot dài cho báo cáo nhất quán.** Repeatable Read cho báo cáo một mốc nhất quán ([PG-034](/articles/postgresql-isolation-read-committed-repeatable-read)), nhưng mỗi phút nó mở là một phút horizon đứng yên cho cả database. Báo cáo dài nên chạy ở nơi khác, có timeout rõ ràng.

## 11. Bài tập

**Bài 1. Chỗ trống có thật sự được dùng lại?** Chạy lại thí nghiệm 1. Sau `VACUUM`, chèn 2.500 dòng cùng kích thước (`id` 5001–7500) rồi đo lại bằng `pgstattuple`.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Bảng số liệu ba thời điểm (sau DELETE, sau VACUUM, sau INSERT): <code>table_len</code>, live, dead, <code>free_pct</code>. Kết luận «chỗ trống được tái sử dụng» chỉ hợp lệ nếu <code>table_len</code> sau INSERT không tăng hoặc tăng ít hơn hẳn so với một bảng đối chứng mới tạo nhận cùng 2.500 dòng, và <code>free_pct</code> giảm tương ứng. Lab hiện tại chưa chạy bước này: ghi số của bạn kèm phiên bản PostgreSQL, đừng chép từ bài.</p>
</details>

**Bài 2. Hai loại «không dọn».** Làm lại thí nghiệm 2 với hai biến thể: (a) H ở Read Committed, chỉ đọc rồi ngồi yên trong giao dịch; (b) H ở Read Committed nhưng đã `UPDATE` một dòng ở bảng **khác** rồi ngồi yên.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Lịch chạy có cột bước, phiên, câu lệnh, kết quả như mục 5; với mỗi biến thể: <code>backend_xmin</code> và <code>backend_xid</code> của H, dòng <code>tuples: …</code> và <code>removable cutoff</code> của VERBOSE trước và sau khi H kết thúc. Phải tách «VACUUM đã chạy nhưng không được phép dọn» khỏi «VACUUM không chạy/lỗi». Dự đoán theo mục 5 là (a) không giữ horizon, (b) có giữ, dù bảng bị ảnh hưởng không phải bảng H đã chạm — nhưng kết luận phải dựa trên output bạn thu được.</p>
</details>

**Bài 3. Chính sách cho hai bảng.** Đề xuất cấu hình autovacuum theo bảng cho `trips` (sửa nhiều) và `trip_events` (chỉ chèn), kèm kế hoạch kiểm chứng.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Với mỗi bảng: <code>reltuples</code> hiện tại; ngưỡng mặc định tính theo công thức mục 6 (kể cả trần <code>autovacuum_vacuum_max_threshold</code>) và ngưỡng đề xuất; tốc độ sinh tuple chết hoặc dòng chèn đo từ <code>pg_stat_user_tables</code> ở hai thời điểm. Chỉ ra ngưỡng nào áp dụng — tuple chết cho <code>trips</code>, ngưỡng chèn cho <code>trip_events</code>. Kèm kế hoạch đo lại (<code>n_dead_tup</code> theo thời gian, số lần và thời lượng autovacuum) và cách hoàn tác (<code>ALTER TABLE … RESET (…)</code>). Thiếu baseline trước thay đổi thì chưa đạt.</p>
</details>

## 12. Đọc tiếp

- [PostgreSQL 18 · Routine Vacuuming](https://www.postgresql.org/docs/18/routine-vacuuming.html): bốn lý do VACUUM, VACUUM FULL, wraparound, công thức ngưỡng autovacuum.
- [PostgreSQL 18 · VACUUM](https://www.postgresql.org/docs/18/sql-vacuum.html) và [Automatic Vacuuming](https://www.postgresql.org/docs/18/runtime-config-autovacuum.html).
- [PostgreSQL 18 · Storage Parameters](https://www.postgresql.org/docs/18/sql-createtable.html#SQL-CREATETABLE-STORAGE-PARAMETERS): ghi đè theo bảng.
- [PostgreSQL 18 · pgstattuple](https://www.postgresql.org/docs/18/pgstattuple.html) và [Progress Reporting](https://www.postgresql.org/docs/18/progress-reporting.html).
- Lab: `examples/toi-noi-postgresql/p05/run.mjs` (khối PG-044), kết quả `p05/OUTPUT.txt`, cổng nghiệm thu `p05/RUNBOOK.md`.

Bài tiếp theo: [PG-045 · WAL, checkpoint và độ bền dữ liệu](/articles/postgresql-wal-checkpoint-durability).
