---
slug: postgresql-upgrade-migration-cutover
title: "Nâng PostgreSQL 17 lên 18: hai đường diễn tập, cổng go/no-go và giới hạn quay lại"
summary: "Tới Nơi cần lên PostgreSQL 18 trong một cửa sổ bảo trì đêm. Bài diễn tập trên 17.7 và 18.1 thật cả hai đường: pg_dump/pg_restore và pg_upgrade. pg_upgrade --check từ chối ngay vì PostgreSQL 18 bật data checksums mặc định; dump/restore đúng dữ liệu nhưng mất statistics; copy mode giữ cluster 17 để quay về — cho tới khi bản 18 nhận lần ghi đầu tiên. Từ đó dựng kế hoạch cutover có cổng go/no-go, đối soát và điểm không quay lại."
category: postgresql
level: advanced
tags: [postgresql, pg-059, upgrade, pg-upgrade, pg-dump, cutover, rollback, data-checksums, toi-noi]
language: vi
---

Database đặt xe của Tới Nơi đang chạy PostgreSQL 17. Đội hạ tầng muốn lên 18 trước mùa cao điểm Tết, trong một cửa sổ bảo trì từ 02:00 tới 02:30 sáng — khung giờ ít chuyến nhất. Trong buổi họp chuẩn bị, bốn câu hỏi được đặt lên bàn:

1. Đi đường nào: dump rồi restore, hay `pg_upgrade`?
2. Làm sao biết dữ liệu bên 18 **giống hệt** bên 17, chứ không chỉ «server lên được, dashboard mở được»?
3. Chuyến đầu tiên đặt sau nâng cấp có nhận mã trùng không? Truy vấn tìm tài xế có đột ngột chậm không?
4. Nếu 02:25 phát hiện sai, quay lại bằng cách nào — và tới lúc nào thì **không quay lại được nữa**?

Bài này trả lời bằng một lần diễn tập thật: một cluster PostgreSQL 17.7 làm nguồn, nâng lên 18.1 bằng cả hai đường. Lần diễn tập vấp ngay một lỗi mà đọc tài liệu lướt sẽ không thấy, và cho ra số liệu để viết cổng go/no-go thay vì cảm giác.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-059 · Phần 6/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-pg-dump-restore">PG-046 · Backup logic với pg_dump</a>, <a href="/articles/postgresql-statistics-analyze-cardinality">PG-027 · Statistics, ANALYZE và ước lượng cardinality</a>, <a href="/articles/postgresql-logical-replication-cdc">PG-055 · Logical replication và CDC</a> và <a href="/articles/postgresql-ha-failover-read-consistency">PG-058 · Failover, split-brain và tính nhất quán khi đọc</a>. Lab: <code>examples/toi-noi-postgresql/p06</code>, PostgreSQL 18.1 và 17.7 thật; trong <code>examples/toi-noi-postgresql</code> đặt <code>PG17_BIN</code> trỏ tới thư mục bin của PostgreSQL 17 (chỉ cần binary, không cần cài service) rồi chạy <code>npm run p06</code> — xem README của lab. Mọi cluster đều tạm, chỉ nghe localhost và bị xoá khi xong. Sau bài này, bạn diễn tập được một lần nâng major version trên bản sao, đối soát kết quả và viết được kế hoạch cutover có điểm quay lại rõ ràng.</p>
</div>

## 1. Vì sao major version cần «di chuyển dữ liệu»

PostgreSQL đánh số phiên bản dạng `17.7`: số đầu là **major version**, số sau là **minor version**. Nâng minor (17.6 → 17.7) chỉ là thay binary rồi khởi động lại: định dạng dữ liệu trên đĩa không đổi. Nâng major (17 → 18) thì khác. Theo tài liệu «Upgrading a PostgreSQL Cluster», các bản major có thể đổi cấu trúc bảng hệ thống (catalog) và định dạng lưu trữ bên trong, nên binary 18 không mở thẳng được data directory của 17.

Tài liệu nêu ba đường:

- **Dump và restore**: xuất dữ liệu ra dạng logic (SQL hoặc archive) bằng `pg_dump`, nạp vào cluster mới. Chậm theo kích thước dữ liệu, nhưng đơn giản và không phụ thuộc định dạng vật lý.
- **`pg_upgrade`**: dựng lại catalog cho bản mới rồi chuyển các file dữ liệu người dùng sang, không phải xuất và nạp lại từng dòng. Nhanh hơn nhiều với database lớn, nhưng có yêu cầu tương thích chặt.
- **Logical replication**: cho cluster 18 làm subscriber của cluster 17 (PG-055), chờ đuổi kịp, rồi chuyển ứng dụng sang. Thời gian ngừng ghi ngắn nhất, vận hành phức tạp nhất. Lab **không chạy** đường này (mục 7).

Lab chạy hai đường đầu, trên cùng một nguồn.

## 2. Nguồn diễn tập trên PostgreSQL 17.7

Runner tạo một cluster bằng binary của PostgreSQL 17 rồi kiểm phiên bản:

```text
PASS 059 source version: 17.7
```

Dữ liệu cố ý nhỏ nhưng có ba thứ hay hỏng khi nâng cấp: cột **identity** (bộ sinh mã), cột **jsonb**, và **statistics** của planner:

```sql
CREATE TABLE tickets (
  id   bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  fare int NOT NULL,
  meta jsonb NOT NULL DEFAULT '{}'
);
INSERT INTO tickets (fare) VALUES (85000), (65000);
ANALYZE tickets;
```

Đáp án để so sau mỗi đường là một **digest**: số dòng, tổng tiền và md5 của chuỗi «mã:giá» theo thứ tự mã — cùng tinh thần đối soát của PG-046:

```sql
SELECT count(*)||'/'||sum(fare)||'/'||md5(string_agg(id||':'||fare, ',' ORDER BY id))
FROM tickets;
```

Hai đường dưới đây đều phải trả lại đúng digest của nguồn. Đáp án được tính một lần trên 17.7 trước khi đụng tới bất cứ thứ gì.

## 3. Đường A: pg_dump bản 18, pg_restore vào cluster 18

```text
pg_dump    -h 127.0.0.1 -p <PG17> -U postgres -Fc -f tickets.dump postgres
pg_restore -h 127.0.0.1 -p <PORT18> -U postgres -d postgres --exit-on-error --single-transaction tickets.dump
```

Chi tiết quan trọng nhất nằm ở chỗ không nhìn thấy: `pg_dump` ở đây là **binary của bản 18**, chạy vào server 17. Tài liệu nâng cấp khuyến nghị dùng `pg_dump` và `pg_dumpall` của **phiên bản mới hơn**, để hưởng các sửa đổi trong công cụ. Trang `pg_dump` nói thêm về tương thích: `pg_dump` dump được từ server cũ hơn chính nó, nhưng không dump được từ server có major version mới hơn, và output không được bảo đảm nạp được vào một major version cũ hơn. Nói cách khác: luôn lấy công cụ của bản đích.

- `-Fc`: **custom format**, archive nén, cho phép `pg_restore` chọn đối tượng và chạy song song. File của lab:

```text
MEASURE 059 dump size bytes: 2495
```

- `--single-transaction`: cả lần restore nằm trong một giao dịch; lỗi giữa chừng thì không để lại database nửa vời. Theo tài liệu, tuỳ chọn này kéo theo `--exit-on-error`; runner ghi cả hai cho rõ ý.

Kết quả, từng dòng:

```text
PASS 059 A dump/restore digest: 2/150000/fd0064873b687f567636614c521ab902
PASS 059 A identity continues after restore: 3
PASS 059 A source untouched (rollback target lacks new write): 2
PASS 059 A statistics after restore (before ANALYZE): 0
```

**Digest khớp**: 2 vé, 150.000 đ, md5 trùng với nguồn. Dữ liệu sang đủ và đúng.

**Identity tiếp tục ở 3**: runner chạy `INSERT INTO tickets (fare) VALUES (50000) RETURNING id` trên bản 18 và nhận `3`. Bản dump mang theo trạng thái của sequence đứng sau cột identity, nên chuyến đặt đầu tiên sau nâng cấp không đụng mã 1 hay 2. Đây là câu hỏi số 3 của buổi họp, và nó chỉ được trả lời bằng một lần **ghi thật** trên đích.

**Nguồn vẫn 2 dòng**: lần ghi vừa rồi (vé 3) chỉ nằm ở bản 18. Nguồn không bị đụng — tốt cho phương án quay lại — nhưng đồng thời là bằng chứng đầu tiên cho giới hạn quay lại ở mục 9: kể từ lần ghi đầu trên bản mới, hai bên đã khác nhau.

**Statistics bằng 0**: `pg_stats` trên bản 18 không có dòng nào cho `tickets`, dù nguồn đã `ANALYZE`. Statistics là dữ liệu planner dùng để ước lượng số dòng (PG-027); trong lab này chúng không đi qua bản dump. Planner của cluster mới sẽ lập kế hoạch bằng giá trị mặc định cho tới khi có `ANALYZE` — với database thật, đó là lúc truy vấn tìm tài xế chọn kế hoạch tệ ngay sau cutover. Cluster lab tắt autovacuum; ở production autovacuum sẽ analyze dần, nhưng «dần» không đủ cho giờ đầu tiên. PostgreSQL 18 có thêm khả năng cho `pg_dump` xuất kèm statistics; lab **không thử** tuỳ chọn đó và mặc định đã cho 0 dòng như trên, nên kế hoạch phải có bước `ANALYZE` (hoặc `vacuumdb --analyze-in-stages`) trước khi mở traffic.

Đường A còn những điều PG-046 đã dạy và không nhắc lại ở đây: `pg_dump` chỉ dump **một database**, không có role và tablespace (lấy bằng `pg_dumpall --globals-only`); password, extension và quyền phải đối soát riêng. Cluster đích của lab là một initdb mặc định của 18, tức **có data checksums** — dump/restore không quan tâm, vì nó nạp dữ liệu qua SQL. Chi tiết này lại là thứ chặn đường B.

## 4. Đường B: pg_upgrade --check từ chối

`pg_upgrade` cần hai cluster: cluster cũ (17) và một cluster mới vừa `initdb` bằng binary 18, cả hai đều **đang dừng**. Bước đầu tiên bao giờ cũng là `--check`: chạy toàn bộ phần kiểm tương thích mà không đổi dữ liệu nào.

```text
pg_upgrade -b <bin17> -B <bin18> -d <data17> -D <data18> -U postgres -p <PG17> -P <PORT18> --check
```

Runner dựng cluster 18 bằng `initdb` với cùng các tuỳ chọn locale như mọi cluster khác của lab, không thêm gì. Kết quả (runner chỉ trích các dòng chứa «Checking», «checksum», «Failure»):

```text
Checking cluster versions                                     ok
old cluster does not use data checksums but the new one does
Failure, exiting
```

**Data checksums** là tổng kiểm tra trên mỗi page dữ liệu, giúp phát hiện page bị hỏng khi đọc từ đĩa. Cluster 17 của lab được `initdb` theo mặc định của bản 17: không có checksums. Còn ghi chú phát hành PostgreSQL 18 ghi rõ: **`initdb` của bản 18 bật data checksums theo mặc định**, và thêm tuỳ chọn `--no-data-checksums` để tắt. `pg_upgrade` chép file dữ liệu nguyên trạng, nên không thể biến page không có checksum thành page có checksum; nó yêu cầu hai cluster cùng thiết lập. Ghi chú phát hành cũng nói thẳng tuỳ chọn mới có ích chính cho trường hợp nâng cấp cluster cũ không có checksums.

Đây là loại lỗi mà kế hoạch viết trên giấy hay bỏ sót: script `initdb` dùng cho bản 17 chạy lại y nguyên với binary 18 vẫn cho ra một cluster khác cấu hình. Hai cách sửa:

1. **Khởi tạo lại cluster 18 không có checksums** — lab làm cách này: dừng cluster mới, đổi tên data directory cũ của nó, `initdb … --no-data-checksums`, rồi chép lại khối cấu hình lab vào `postgresql.conf`.
2. **Bật checksums cho cluster 17 trước khi nâng cấp** bằng `pg_checksums --enable` khi cluster đang dừng. Công cụ phải đọc và ghi lại mọi page, nên thời gian dừng tỉ lệ với kích thước dữ liệu. **Không chạy trong lab.** Cách này hợp khi đội muốn có checksums lâu dài và chấp nhận một cửa sổ bảo trì riêng, tách khỏi đêm nâng cấp.

Bài học cho cổng go/no-go: «`pg_upgrade --check` sạch trên bản sao» là một cổng thật, chạy với **đúng** script initdb và cấu hình sẽ dùng đêm cutover, không phải một bước nghi thức.

## 5. pg_upgrade copy mode: chạy thật

Sau khi initdb lại với `--no-data-checksums`, runner chạy cùng lệnh nhưng bỏ `--check`, không thêm tuỳ chọn chế độ nào — tức **copy mode** mặc định. Các dòng trích từ output (runner lọc theo «Performing Upgrade», «Copying user relation», «Upgrade Complete», «statistics»):

```text
Performing Upgrade
Copying user relation files                                   ok
Upgrade Complete
Some statistics are not transferred by pg_upgrade.
```

Phần còn lại của output `pg_upgrade` không có trong `OUTPUT.txt`. Kiểm sau khi khởi động cluster 18:

```text
PASS 059 B upgraded version: 18
PASS 059 B digest after pg_upgrade: 2/150000/fd0064873b687f567636614c521ab902
PASS 059 B planner statistics carried over: true
PASS 059 B identity continues: 3
PASS 059 B copy mode: old 17 cluster still starts with: 2 rows on 17.7
```

**Digest khớp**, cùng giá trị với nguồn và với đường A.

**Statistics được mang sang**: `pg_stats` trên cluster 18 có dòng cho `tickets` ngay sau nâng cấp, không cần `ANALYZE`. Đây là thay đổi của PostgreSQL 18: ghi chú phát hành nói `pg_upgrade` giờ giữ lại optimizer statistics, nhưng **extended statistics** (thống kê nhiều cột tạo bằng `CREATE STATISTICS`, PG-027) thì không. Dòng «Some statistics are not transferred by pg_upgrade» là lời nhắc đó. Phép kiểm của lab chỉ là «có hơn 0 dòng `pg_stats`» trên một bảng không có extended statistics; nó không so từng giá trị histogram. Trang `pg_upgrade` của bản 18 hướng dẫn chạy `vacuumdb` để bổ sung phần statistics còn thiếu sau khi khởi động cluster mới — dạng lệnh, **không chạy trong lab**:

```text
vacuumdb --all --analyze-in-stages --missing-stats-only
```

`--analyze-in-stages` chạy ANALYZE ba lượt với độ chi tiết tăng dần để planner sớm có số liệu thô; `--missing-stats-only` chỉ xử lý những đối tượng chưa có statistics. Tra lại đúng dòng hướng dẫn mà `pg_upgrade` của bạn in ra.

**Identity tiếp tục ở 3**: `pg_upgrade` chép nguyên file của sequence, nên trạng thái bộ sinh mã đi theo. (Mã 3 xuất hiện lần nữa vì nguồn 17 vẫn chỉ có 2 vé; vé 3 của đường A nằm ở cluster khác.)

**Cluster 17 vẫn khởi động được**: runner start lại cluster 17.7 sau khi nâng cấp và đếm được 2 vé. Theo tài liệu `pg_upgrade`, ở copy mode cluster cũ không bị sửa đổi, nên còn dùng được để quay lại. Điều đó không còn đúng ở các chế độ khác (mục 6).

Lab là một bảng nhỏ, không extension, không đo thời gian. Ở database thật, `--check` còn phải sạch về extension (bản 18 phải có file thư viện của mọi extension đang dùng), collation, tablespace và các kiểu dữ liệu bị loại bỏ; sau nâng cấp có thể cần `ALTER EXTENSION … UPDATE`. Standby vật lý cũng phải được xử lý riêng — tài liệu mô tả quy trình rsync, hoặc dựng lại standby từ đầu. Không mục nào trong số này được lab chạy.

## 6. Các chế độ của pg_upgrade: tốc độ đổi lấy đường lùi

Copy mode chép từng file dữ liệu, nên thời gian và dung lượng đĩa cần thêm tỉ lệ với kích thước database. `pg_upgrade` có các chế độ khác, theo tài liệu PostgreSQL 18; **lab chỉ chạy copy mode**:

| Chế độ | Cách chuyển file | Thời gian | Cluster cũ sau đó |
|---|---|---|---|
| copy (mặc định) | Chép từng file | Tỉ lệ dữ liệu; cần gấp đôi dung lượng | Không bị sửa, khởi động lại được (lab đã kiểm) |
| `--copy-file-range` (từ 17) | Chép bằng `copy_file_range` của hệ điều hành | Tuỳ hệ thống file | Như copy |
| `--clone` | Reflink (Btrfs, XFS có reflink, APFS…) | Gần như tức thì | Không bị ảnh hưởng |
| `--link` | Hard link tới file cũ | Nhanh, gần như không phụ thuộc dữ liệu | **Không dùng được nữa** khi cluster mới đã khởi động |
| `--swap` (mới ở 18) | Chuyển nguyên thư mục dữ liệu sang cluster mới | Thường nhanh nhất, nhất là khi có nhiều quan hệ | **Không dùng được nữa** |

Hai chế độ cuối làm cửa sổ bảo trì ngắn đi nhiều, nhưng xoá đường lùi «khởi động lại cluster cũ». Khi chọn chúng, đường lùi phải là một thứ khác đã được diễn tập: một base backup và WAL của PG-048 lấy ngay trước khi chạy, hoặc một standby được tách ra trước nâng cấp. Ghi điều này vào kế hoạch, kèm thời gian phục hồi đo được, chứ không phải «có backup».

Dạng lệnh để diễn tập trên bản sao, **không chạy trong lab**:

```text
pg_upgrade -b <bin17> -B <bin18> -d <data17> -D <data18> --check --link
pg_upgrade -b <bin17> -B <bin18> -d <data17> -D <data18> --link
```

## 7. Đường thứ ba: logical replication (không chạy trong lab)

Khi cửa sổ 30 phút không đủ để dump/restore hay chép file, đường còn lại là cho cluster 18 bắt kịp cluster 17 **trong khi 17 vẫn phục vụ**, rồi chỉ dừng ghi vài phút để chuyển. Phác thảo, **không chạy trong lab** và không có output:

1. Dựng cluster 18, nạp **schema** bằng `pg_dump --schema-only` của bản 18.
2. Trên 17: `wal_level = logical`, `CREATE PUBLICATION … FOR ALL TABLES`. Trên 18: `CREATE SUBSCRIPTION` và chờ mọi bảng tới trạng thái `r` (PG-055).
3. Đêm cutover: dừng ghi trên 17, chờ subscriber áp dụng tới LSN cuối của 17, đối soát digest từng bảng.
4. **Đặt lại mọi sequence** trên 18: logical replication không sao chép dữ liệu sequence (PG-055, mục 8). Quên bước này thì chuyến đầu tiên trên 18 nhận mã đã tồn tại.
5. Chuyển ứng dụng sang 18.

Mọi giới hạn của PG-055 đi theo: DDL trong thời gian đồng bộ phải đóng băng hoặc chạy hai phía; bảng không có khoá chính cần replica identity; slot trên 17 giữ WAL nếu subscriber chậm. Tài liệu còn có `pg_createsubscriber` (từ bản 17) để biến một standby vật lý thành subscriber logic, đỡ phải initial copy lại từ đầu với database lớn; và từ bản 17, `pg_upgrade` có thể giữ lại logical slot và subscription khi cluster cũ cũng từ bản 17 trở lên. Lab không chạy công cụ nào trong số này. Đổi lại, đường này cho một phương án quay lại mạnh: dựng replication **ngược** từ 18 về 17 sau cutover, để ghi mới trên 18 vẫn chảy về 17 trong thời gian theo dõi.

## 8. Chọn đường theo thời gian ngừng ghi

| Tiêu chí | Dump/restore | pg_upgrade copy | pg_upgrade link/swap | Logical replication |
|---|---|---|---|---|
| Thời gian ngừng ghi | Dump + restore + index + ANALYZE: tỉ lệ dữ liệu | Chép file: tỉ lệ dữ liệu, không dựng lại index | Ngắn, gần như không phụ thuộc dữ liệu | Vài phút cho bước chuyển |
| Statistics sau nâng | Không có (lab: 0 dòng) | Có, trừ extended (lab: có) | Như copy | Subscriber tự thu thập (autovacuum/ANALYZE) trong lúc đồng bộ |
| Đường lùi | Nguồn nguyên vẹn (lab: 2 dòng) | Cluster cũ khởi động được (lab) | Phải dùng backup | Nguồn nguyên vẹn; có thể replication ngược |
| Yêu cầu tương thích | Ít: chỉ qua SQL | Checksums, extension, locale khớp | Như copy, thêm hệ thống file | Khoá chính, DDL đóng băng, sequence |
| Lab | Đã chạy | Đã chạy | Không | Không |
| Cơ hội dọn dẹp | Bảng và index được viết lại, hết bloat | Giữ nguyên file, giữ nguyên bloat | Như copy | Như dump/restore |

Với Tới Nơi, quyết định phụ thuộc vào số đo mà lab này **chưa có**: dump/restore và copy mode mất bao lâu trên bản sao cỡ thật. README ghi rõ lab không đo downtime. Bước đầu tiên của kế hoạch vì thế là đo.

## 9. Kế hoạch cutover và giới hạn quay lại

Một kế hoạch cutover là một chuỗi bước có giờ, người chịu trách nhiệm và điều kiện dừng. Khung dưới đây dựa trên cổng go/no-go trong `p06/HO-SO-BAN-GIAO.md`.

**Trước đêm cutover (T-14 ngày tới T-1).** Diễn tập trên **bản sao** cỡ thật, dựng từ backup (PG-048) chứ không phải chạy thử trên production. Chạy đúng script initdb, đúng binary, đúng cấu hình sẽ dùng. Đo thời gian từng bước. Chạy bộ truy vấn quan trọng trên bản đã nâng để thấy kế hoạch thực thi có đổi không.

**Cổng go/no-go**, chốt trước khi bắt đầu:

| Cổng | Lab 059 | Production cần thêm |
|---|---|---|
| `pg_upgrade --check` sạch trên bản sao | Phát hiện checksum khác nhau, sửa bằng initdb `--no-data-checksums` | Extension, collation/ICU, tablespace, kiểu dữ liệu bị loại bỏ |
| Digest sau nâng khớp nguồn | Khớp cả hai đường | Đối soát theo từng bảng nghiệp vụ, không chỉ một bảng |
| Sequence/identity tiếp tục đúng | `RETURNING id` = 3 | Mọi sequence, kể cả sequence dùng chung |
| Statistics sẵn sàng | dump/restore: 0 dòng `pg_stats` trước ANALYZE; pg_upgrade 18: có | `vacuumdb` cho phần không được mang sang |
| Còn đường quay lại | Copy mode: cluster 17 vẫn khởi động | Thời hạn giữ cluster cũ; ghi mới sau cutover không tự quay về |

**Đêm cutover.**

1. **02:00 — dừng ghi.** Chặn ở tầng ứng dụng và pool kết nối, rồi xác nhận trên `pg_stat_activity` (<a href="/articles/postgresql-pg-stat-activity-quan-sat">PG-043</a>) không còn phiên ghi. Ghi lại `pg_current_wal_lsn()` làm mốc.
2. **Tính digest nguồn** cho từng bảng nghiệp vụ — đáp án của đêm đó.
3. **Chạy đường đã chọn**, với thời gian dự kiến lấy từ diễn tập. Vượt quá ngưỡng thời gian đã chốt là một điều kiện dừng, không phải lý do «cố thêm chút».
4. **Đối soát**: digest từng bảng khớp; phiên bản là 18; statistics có (hoặc đã chạy ANALYZE); extension đúng phiên bản.
5. **Một lần ghi thật qua đường ứng dụng**: đặt một chuyến thử qua đúng connection string, pool và role mà ứng dụng sẽ dùng; mã trả về phải tiếp nối mã lớn nhất cũ. Đây là bước biến «server lên» thành «hệ thống chạy».
6. **Mở traffic.**

**Giới hạn quay lại.** Lab đường A cho thấy chính xác ranh giới này: sau một lần ghi trên bản 18, nguồn vẫn chỉ có 2 dòng. Mọi thứ trước bước 5 có thể quay về 17 mà không mất gì — chỉ cần trỏ ứng dụng lại và mở ghi, vì nguồn không bị đụng (đường A, hoặc copy mode ở đường B). Từ lần ghi đầu tiên trên 18, quay về 17 nghĩa là **bỏ** những chuyến đã đặt trên 18, trừ khi có cơ chế đưa chúng về. Khi đó các lựa chọn là:

- **Sửa tiến** trên 18: thường là lựa chọn đúng nếu lỗi không nằm ở dữ liệu.
- **Dừng ghi, đối soát phần chênh, chép phần ghi mới về 17** có kiểm tra: chậm, dễ sai, cần quy trình đã viết sẵn.
- **Replication ngược 18 → 17** dựng sẵn từ lúc cutover (mục 7): đường lùi mạnh nhất, cũng là đường tốn công nhất; lab không chạy.

Kế hoạch phải ghi rõ **điểm không quay lại** (thường là bước 6), **người quyết định** ở mỗi cổng, và **thời hạn giữ cluster 17** — giữ nó chạy mãi chỉ tạo ảo giác có đường lùi khi dữ liệu hai bên đã lệch nhau từ lâu.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| `pg_upgrade --check` báo checksum không khớp | Cluster 18 initdb mặc định bật checksums, cluster 17 không có | `initdb --no-data-checksums` cho cluster mới, hoặc bật checksums cho cluster cũ trong một cửa sổ riêng |
| Sau cutover, truy vấn quen thuộc đổi kế hoạch, chậm hẳn | Statistics không đi qua dump, hoặc extended statistics không đi qua `pg_upgrade` | ANALYZE trước khi mở traffic; kiểm kế hoạch của truy vấn quan trọng trên bản diễn tập |
| Chuyến đầu tiên báo trùng khoá chính | Sequence không được đặt lại (đường logical replication) | Đặt lại mọi sequence trước bước ghi thử |
| Dump bằng `pg_dump` cũ nạp lỗi hoặc thiếu đối tượng | Dùng công cụ của bản nguồn | Dùng `pg_dump`/`pg_dumpall` của bản đích |
| Restore xong nhưng role không đăng nhập được | `pg_dump` không mang role và password | `pg_dumpall --globals-only`, đặt lại password từ kho bí mật (PG-046) |
| Định quay lại 17 sau khi đã dùng `--link` | Link mode làm cluster cũ không dùng được khi cluster mới đã khởi động | Chọn đường lùi bằng backup trước khi chọn link mode |
| Quay về 17 sau một giờ chạy trên 18 | Không có điểm không quay lại trong kế hoạch | Chốt điểm không quay lại; sau điểm đó chỉ sửa tiến hoặc dùng replication ngược đã chuẩn bị |

## 11. Đánh đổi

Không có đường nâng cấp «an toàn nhất» chung. Dump/restore ít yêu cầu nhất và tiện dọn bloat, nhưng thời gian ngừng tỉ lệ với dữ liệu và mất statistics. `pg_upgrade` copy mode nhanh hơn, giữ statistics (ở 18), giữ cluster cũ làm đường lùi, đổi lại đòi tương thích chặt và gấp đôi dung lượng đĩa. Link và swap rút cửa sổ xuống ngắn nhất nhưng bỏ đường lùi tức thì. Logical replication cho thời gian ngừng ghi ngắn nhất và có thể có đường lùi mạnh nhất, đổi lại là cả một hệ thống phải vận hành trong nhiều ngày. Lựa chọn đúng là lựa chọn mà đội **đã diễn tập trên bản sao cỡ thật và đo được thời gian**, trong cửa sổ mà nghiệp vụ chấp nhận.

## 12. Bài tập

**Bài 1.** Trên bản chép riêng của lab, thêm một extended statistics (`CREATE STATISTICS … (dependencies) ON …`) cho `tickets` ở cluster 17, chạy `ANALYZE`, rồi nâng cấp bằng `pg_upgrade` copy mode.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Output thật cho thấy trên cluster 18, <code>pg_stats</code> có dòng cho <code>tickets</code> nhưng <code>pg_stats_ext</code> (hoặc <code>pg_statistic_ext_data</code>) chưa có dữ liệu cho đối tượng statistics vừa tạo, trong khi định nghĩa trong <code>pg_statistic_ext</code> vẫn còn. Sau khi chạy lệnh <code>vacuumdb</code> mà <code>pg_upgrade</code> của bạn khuyến nghị (dán đúng dòng nó in), dữ liệu xuất hiện. Đối chiếu với ghi chú phát hành PostgreSQL 18 về statistics và pg_upgrade.</p>
</details>

**Bài 2.** Đo thời gian ngừng ghi của hai đường trên một bản sao lớn hơn (ví dụ bảng `gps_plain` nhân lên vài triệu dòng).

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Máy, kích thước database, số lần đo và cách đo (từ lúc dừng ghi tới lúc đối soát xong). Tách thời gian dump, restore, dựng index, ANALYZE với đường A; thời gian chép file và ANALYZE bổ sung với đường B. Digest khớp ở mọi lần. Kết luận chỉ áp dụng cho kích thước đã đo, không ngoại suy tuyến tính cho production mà không có thêm một điểm đo.</p>
</details>

**Bài 3.** Viết kế hoạch cutover một trang cho Tới Nơi, cửa sổ 02:00–02:30.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Đường được chọn kèm số đo thời gian từ diễn tập (không lấy số từ bài này — lab không đo). Mỗi bước có giờ dự kiến, người thực hiện và điều kiện dừng. Bảng cổng go/no-go có cột «bằng chứng» trỏ tới output thật. Điểm không quay lại được ghi rõ, đường lùi trước và sau điểm đó, thời hạn giữ cluster cũ. Nếu chọn link hoặc swap, có thời gian phục hồi từ backup đã đo.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 18 · Upgrading a PostgreSQL Cluster](https://www.postgresql.org/docs/18/upgrading.html): ba đường nâng cấp, khuyến nghị dùng công cụ của bản mới.
- [PostgreSQL 18 · pg_upgrade](https://www.postgresql.org/docs/18/pgupgrade.html): `--check`, copy/link/clone/copy-file-range/swap, standby, statistics sau nâng cấp.
- [PostgreSQL 18 · pg_dump](https://www.postgresql.org/docs/18/app-pgdump.html): tương thích giữa các phiên bản, custom format, tuỳ chọn statistics.
- [PostgreSQL 18 · Release Notes](https://www.postgresql.org/docs/18/release-18.html): initdb bật data checksums mặc định và `--no-data-checksums`; pg_upgrade giữ optimizer statistics, không giữ extended statistics.
- [PostgreSQL 18 · pg_checksums](https://www.postgresql.org/docs/18/app-pgchecksums.html): bật, tắt và kiểm checksums khi cluster dừng.
- [PostgreSQL 18 · Logical Replication](https://www.postgresql.org/docs/18/logical-replication.html) và [pg_createsubscriber](https://www.postgresql.org/docs/18/app-pgcreatesubscriber.html): đường nâng cấp bằng replication.
- Lab: `examples/toi-noi-postgresql/p06/run.mjs` (khối PG-059), kết quả `p06/OUTPUT.txt`, giới hạn trong `p06/README.md`, cổng go/no-go và việc còn thiếu trong `p06/HO-SO-BAN-GIAO.md`.

Bài trước: [PG-058 · Failover, split-brain và tính nhất quán khi đọc](/articles/postgresql-ha-failover-read-consistency). Bài tiếp theo: [PG-060 · Capstone: vận hành dữ liệu Tới Nơi từ schema tới sự cố](/articles/postgresql-capstone-van-hanh-toi-noi).
