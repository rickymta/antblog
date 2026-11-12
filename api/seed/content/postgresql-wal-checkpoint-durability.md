---
slug: postgresql-wal-checkpoint-durability
title: "WAL, checkpoint và độ bền dữ liệu: khoản thanh toán có sống sót sau cú mất điện?"
summary: "API của Tới Nơi vừa báo thanh toán thành công thì máy chủ database mất điện. COMMIT đã hứa điều gì? Đi qua WAL, flush khi commit, checkpoint, điểm redo và full-page write, rồi đọc phép đo nhỏ trên PostgreSQL 18.1: cùng một UPDATE 100 dòng ghi 29.952 byte WAL ngay sau checkpoint và 7.544 byte khi lặp lại."
category: postgresql
level: intermediate
tags: [postgresql, pg-045, wal, checkpoint, durability, full-page-writes, synchronous-commit, toi-noi]
language: vi
---

21:47 tối thứ Bảy, khách của Tới Nơi trả 85.000 đ cho chuyến xe bằng ví VíXanh. Dịch vụ thanh toán chạy `COMMIT`, nhận `COMMIT` từ PostgreSQL và trả về app «Thanh toán thành công». Hai giây sau, máy chủ ảo chạy database bị tắt đột ngột vì sự cố nguồn ở máy vật lý bên dưới. Mười phút sau database khởi động lại. Câu hỏi đầu tiên trong kênh trực: **khoản thanh toán đó còn không?** Câu hỏi thứ hai, từ một người đang lo chuyện hiệu năng: «Nếu bật `synchronous_commit = off` thì nhanh hơn bao nhiêu, và cho checkpoint chạy mỗi phút có làm dữ liệu an toàn hơn không?»

Trả lời cả hai cần hiểu đường ghi của PostgreSQL: dữ liệu đi từ bộ nhớ xuống đĩa theo thứ tự nào, `COMMIT` chờ cái gì, và checkpoint thực sự làm gì. Bài này nối giao dịch của Phần 4 ([PG-031](/articles/postgresql-transaction-commit-rollback)) với storage, rồi đọc một phép đo nhỏ trong lab — đọc kèm giới hạn của nó.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-045 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/postgresql-vacuum-autovacuum-bloat">PG-044 · VACUUM, autovacuum và bloat</a> và <a href="/articles/page-don-vi-doc-ghi">PG-022 · Page, đơn vị đọc ghi</a>. Lab: <code>examples/toi-noi-postgresql/p05</code> (PostgreSQL 18.1 thật); số đo trong <code>p05/OUTPUT.txt</code>, giới hạn phép đo trong <code>p05/README.md</code>. Sau bài này, bạn mô tả được đường ghi từ <code>UPDATE</code> tới file dữ liệu, giải thích COMMIT bảo đảm gì và không bảo đảm gì, và đọc được số đo WAL/checkpoint mà không suy rộng quá dữ kiện.</p>
</div>

## 1. Dữ liệu và cách chạy

```bash
cd examples/toi-noi-postgresql
npm run p05
```

Cùng cluster tạm với PG-044: PostgreSQL 18.1, `shared_buffers` 128 MB, autovacuum bật. Bảng đo là `ops.trips` — 100 chuyến, mỗi chuyến 30.000 đ. Runner sửa `fare` ba lượt, đo WAL, chạy `CHECKPOINT`, rồi đặt `fare` về 30.000 để PG-046 có baseline rõ ràng. Kết quả chép từ `p05/OUTPUT.txt`, bỏ phần thụt lề của runner.

Lab **không** mô phỏng mất điện, hỏng đĩa hay failover (README nói rõ). Câu hỏi «khoản thanh toán còn không» được trả lời bằng cơ chế theo tài liệu PostgreSQL 18; lab cho thấy những mảnh quan sát được: cấu hình, lượng WAL, bộ đếm checkpoint và log.

## 2. Đường đi của một câu UPDATE

Một số thuật ngữ:

- **Shared buffers**: vùng nhớ chung nơi PostgreSQL giữ bản sao các page 8 KB của bảng và index. Mọi đọc ghi đi qua đây. Page đã sửa trong bộ nhớ mà chưa ghi xuống file gọi là **page bẩn** (dirty page).
- **WAL** (write-ahead log, nhật ký ghi trước): chuỗi bản ghi mô tả mọi thay đổi trên page, lưu thành các file 16 MB trong thư mục `pg_wal`.
- **LSN** (log sequence number): vị trí byte trong dòng WAL, viết dạng `0/4001DF0`. Hiệu hai LSN là số byte WAL sinh ra giữa hai thời điểm. Mỗi page cũng ghi LSN của bản ghi WAL cuối cùng đã sửa nó.

Khi giao dịch thanh toán `UPDATE` một dòng rồi `COMMIT`:

| Bước | Ở đâu | Việc gì xảy ra |
|---|---|---|
| 1 | Shared buffers | Page chứa dòng được sửa trong bộ nhớ (phiên bản mới theo MVCC, PG-032), trở thành page bẩn |
| 2 | WAL buffers | Một bản ghi WAL mô tả thay đổi được thêm vào; page ghi nhận LSN của bản ghi đó |
| 3 | `COMMIT` | Bản ghi commit được thêm vào WAL; WAL **tới LSN của bản ghi commit** được ghi và **flush** (ép xuống thiết bị lưu trữ bằng `fsync` hoặc tương đương) |
| 4 | Trả về client | Chỉ sau bước 3, client mới nhận `COMMIT` (khi `synchronous_commit = on`) |
| 5 | Về sau | Page bẩn được ghi vào file dữ liệu — bởi background writer, checkpointer, hoặc backend cần chỗ trong buffer |

Quy tắc trung tâm, theo tài liệu (WAL Introduction): thay đổi trên file dữ liệu chỉ được ghi **sau khi** bản ghi WAL mô tả nó đã được flush. Trước khi ghi một page bẩn xuống đĩa, PostgreSQL bảo đảm WAL đã flush ít nhất tới LSN ghi trên page đó. Vì thế, nếu máy sập giữa bước 4 và 5, file dữ liệu chưa có thay đổi nhưng WAL thì có; khi khởi động lại, PostgreSQL **phát lại** (redo) WAL để dựng lại thay đổi.

Vì sao không ghi thẳng page lúc commit? Flush **file WAL** là ghi tuần tự, một lần flush có thể phủ nhiều giao dịch cùng lúc; còn page giao dịch đã chạm nằm rải rác khắp các file. Một page bị sửa nhiều lần (chuyến đổi trạng thái tám lần) cũng chỉ cần ghi xuống file một lần về sau.

## 3. COMMIT đã hứa gì

Với `fsync = on` và `synchronous_commit = on`, khi client nhận `COMMIT`, bản ghi commit đã nằm trên thiết bị lưu trữ bền vững. Sau mất điện, phát lại WAL sẽ dựng lại khoản thanh toán. **Câu trả lời cho câu hỏi đầu bài là «còn»** — với ba điều kiện:

1. Cấu hình đúng như trên. Lab kiểm trước tiên: `PASS 045 durability settings: on/on/on` (`fsync`, `synchronous_commit`, `full_page_writes`).
2. Storage không nói dối. Tài liệu (WAL Reliability) cảnh báo bộ nhớ đệm ghi của ổ đĩa hay bộ điều khiển có thể báo «đã ghi» khi dữ liệu mới nằm trong cache không được bảo vệ bằng pin. Khi đó flush của PostgreSQL không còn nghĩa là bền vững — đây là việc của người vận hành hạ tầng, PostgreSQL không tự kiểm được.
3. Ổ đĩa chứa WAL còn đọc được. Mất ổ là chuyện của backup và replica (PG-046, PG-048), không phải của crash recovery.

COMMIT **không** hứa: file dữ liệu đã có thay đổi (bước 5 xảy ra sau); một replica đã nhận (trừ khi cấu hình standby đồng bộ); hay bản backup đã có giao dịch đó.

**`synchronous_commit = off`** (lab **không chạy**). Server trả `COMMIT` trước khi flush WAL; WAL writer flush sau. Tài liệu PostgreSQL 18 nêu độ trễ tối đa giữa lúc báo thành công và lúc thực sự an toàn là **ba lần `wal_writer_delay`** (mặc định 200 ms). Mất điện trong khoảng đó thì giao dịch đã báo thành công **biến mất** — nhưng database vẫn nhất quán: không có nửa giao dịch, không hỏng dữ liệu. Tham số này đặt được theo từng giao dịch:

```sql
-- Chưa chạy trong lab. Chỉ cho dữ liệu chấp nhận mất vài trăm ms cuối,
-- ví dụ một bản ghi vị trí tài xế sẽ bị ghi đè sau vài giây.
BEGIN;
SET LOCAL synchronous_commit = off;
INSERT INTO driver_locations ...;
COMMIT;
```

Với thanh toán VíXanh thì không: khách đã thấy «thành công», mất giao dịch là mất tiền hoặc đối soát sai. Quyết định này thuộc về từng loại dữ liệu, ghi rõ trong thiết kế, không áp đại trà vì «tăng TPS».

**`fsync = off`** (lab **không chạy**) nguy hiểm hơn hẳn: PostgreSQL không còn ép dữ liệu xuống đĩa, và tài liệu cảnh báo sau sự cố hệ điều hành hoặc mất điện có thể dẫn tới **hỏng dữ liệu không phục hồi được**. Tài liệu chỉ gợi ý tắt nó khi nạp dữ liệu ban đầu có thể làm lại từ đầu. Một benchmark không có crash không chứng minh được rằng tắt nó «miễn phí».

## 4. Checkpoint và điểm redo

Nếu chỉ dựa vào WAL, sau nhiều tháng chạy, khôi phục phải phát lại WAL từ ngày đầu. **Checkpoint** chặn chuyện đó. Theo tài liệu (WAL Configuration), checkpoint là một điểm trong dòng WAL mà tại đó mọi page bẩn đã sửa trước điểm này đều đã được ghi xuống file dữ liệu. Checkpointer ghi page bẩn, flush, rồi ghi một bản ghi checkpoint vào WAL. Bản ghi này trỏ tới **điểm redo** (redo point) — chỗ crash recovery sẽ bắt đầu phát lại. WAL trước điểm redo không còn cần cho crash recovery và có thể được tái chế hoặc xoá, trừ khi còn cần cho archive, replication slot hoặc `wal_keep_size`.

Checkpoint xảy ra khi:

- đủ `checkpoint_timeout` kể từ lần trước (mặc định 5 phút);
- lượng WAL sắp vượt `max_wal_size` (mặc định 1 GB);
- có người chạy lệnh `CHECKPOINT` (lab làm vậy), lúc tắt server, lúc bắt đầu base backup (PG-048 dùng `pg_basebackup --checkpoint=fast`).

Checkpoint theo lịch không xả hết page bẩn một lúc: `checkpoint_completion_target` (mặc định 0,9) trải việc ghi ra khoảng 90 % thời gian tới checkpoint kế tiếp để tránh dội I/O.

Cấu hình WAL của cluster lab:

```text
        name        | setting | unit
--------------------+---------+------
 archive_mode       | on      | NULL
 checkpoint_timeout | 1800    | s
 fsync              | on      | NULL
 full_page_writes   | on      | NULL
 max_wal_size       | 4096    | MB
 synchronous_commit | on      | NULL
 wal_compression    | off     | NULL
 wal_level          | replica | NULL
(8 rows)
```

Lưu ý: `checkpoint_timeout = 1800` giây (30 phút) và `max_wal_size = 4096` MB là **cấu hình chung của lab** (`pg/lib/cluster.mjs`), không phải mặc định PostgreSQL — chúng được nới để checkpoint theo lịch không chen vào phép đo. `archive_mode = on` và `wal_level = replica` phục vụ PITR ở PG-048. `wal_compression = off` là mặc định.

Hai tham số kéo hai hướng. Checkpoint **dày** hơn: ít WAL phải phát lại sau crash, nên khởi động lại nhanh hơn; nhưng ghi page nhiều hơn và — như mục 5 — sinh nhiều WAL hơn. Checkpoint **thưa** hơn: ngược lại, và `pg_wal` cần chỗ hơn. `max_wal_size` là **giới hạn mềm**: tài liệu nói nó có thể bị vượt khi tải nặng, khi `archive_command` lỗi, hoặc khi `wal_keep_size` lớn.

Vậy «cho checkpoint chạy mỗi phút để an toàn hơn» có đúng không? Không. Độ bền của giao dịch đã commit đến từ **WAL đã flush**, không từ checkpoint. Checkpoint dày chỉ rút ngắn thời gian **khôi phục**, đổi lại chi phí I/O và WAL.

## 5. Page rách và full-page write

Page của PostgreSQL là 8 KB, nhưng hệ điều hành và ổ đĩa không bảo đảm ghi nguyên tử 8 KB. Mất điện giữa lúc ghi có thể để lại **page rách** (torn page): nửa cũ, nửa mới. Bản ghi WAL thường chỉ mô tả thay đổi nhỏ («đặt cột fare của tuple số 7 thành 30.001»), giả định page gốc nguyên vẹn; phát lại lên page rách không sửa được nó.

**`full_page_writes = on`** (mặc định) xử lý bằng cách: lần **đầu tiên** một page bị sửa sau mỗi checkpoint, PostgreSQL ghi **toàn bộ nội dung page** vào WAL — gọi là **full-page image** (FPI). Khi khôi phục, bắt đầu từ điểm redo, PostgreSQL chép đè FPI lên page trên đĩa — rách hay không — rồi mới phát lại các thay đổi sau đó. Vì recovery luôn bắt đầu từ điểm redo, chỉ cần FPI cho lần sửa đầu tiên sau checkpoint là đủ phủ mọi page có thể bị rách.

Cái giá: ngay sau mỗi checkpoint, lượng WAL tăng vọt vì hàng loạt page bị sửa lần đầu. Đây là lý do thứ hai khiến checkpoint dày làm tăng WAL. `wal_compression` (lab **không bật**) nén FPI, đổi CPU lấy dung lượng WAL. Tắt `full_page_writes` thì tài liệu xếp rủi ro cùng loại với tắt `fsync`, chỉ nên cân nhắc khi phần cứng hoặc filesystem tự bảo đảm không có page rách.

## 6. Lab: đo WAL quanh checkpoint

### 6.1. Ba lượt UPDATE + CHECKPOINT

Mỗi lượt runner chạy các câu dưới đây, **mỗi câu một tiến trình `psql` riêng**, đo thời gian bằng `performance.now()` của Node:

```sql
SELECT pg_current_wal_insert_lsn();                  -- lưu mốc trước
UPDATE ops.trips SET fare = fare + 1;                -- 100 dòng, đo updateMs
SELECT pg_wal_lsn_diff(pg_current_wal_insert_lsn(), '<LSN_TRUOC>')::bigint;
CHECKPOINT;                                          -- đo checkpointMs
```

Kết quả (dòng `MEASURE 045 warm local 100-row updates, includes psql startup`, đã chuyển sang bảng):

| Lượt | `updateMs` | `checkpointMs` | `walBytes` |
|---|---|---|---|
| 1 | 104 | 168,2 | 22.488 |
| 2 | 204,7 | 117,8 | 32.312 |
| 3 | 105,7 | 106,9 | 21.848 |

Đọc cẩn thận:

- **Mili-giây ở đây không phải độ trễ SQL.** Mỗi số gồm cả thời gian khởi động `psql`, kết nối, chạy và thoát. Lượt 2 gần gấp đôi lượt 1 và 3 cho cùng một câu — một lần đo đơn lẻ trên Windows không đủ nói tại sao. Không dùng bảng này để kết luận throughput hay «checkpoint mất 100 ms».
- **`walBytes` dao động 21.848–32.312** cho cùng một câu lệnh. `pg_current_wal_insert_lsn()` là vị trí của **cả cluster**, nên hiệu LSN gồm mọi WAL sinh ra trong khoảng đó, kể cả việc nền. Cluster bật autovacuum, và mỗi lượt để lại 100 tuple chết trên bảng 100 dòng — vượt ngưỡng 50 + 0,2 × 100 = 70 của PG-044 — nên autovacuum ghé `ops.trips` giữa chừng là một **giả thuyết** hợp lý; lab không ghi log autovacuum để xác nhận.

### 6.2. Bộ đếm checkpoint

Runner đọc `num_requested` của view `pg_stat_checkpointer` trước và sau vòng lặp:

```text
PASS 045 requested checkpoints counted: +3
```

Ba lệnh `CHECKPOINT` được đếm là ba checkpoint **theo yêu cầu**. View `pg_stat_checkpointer` có từ PostgreSQL 17 (trước đó các bộ đếm nằm trong `pg_stat_bgwriter`). Trên hệ thống thật, tỉ lệ giữa `num_timed` (do `checkpoint_timeout`) và `num_requested` (do `max_wal_size`, lệnh tay, backup…) là tín hiệu đầu tiên: nếu checkpoint theo yêu cầu chiếm đa số mà không ai chạy lệnh tay, `max_wal_size` có thể quá nhỏ so với tốc độ sinh WAL.

### 6.3. Thí nghiệm full-page image

Ngay sau lệnh `CHECKPOINT` cuối của vòng lặp, runner chạy **cùng một câu** hai lần liên tiếp, mỗi lần đo hiệu LSN:

```sql
UPDATE ops.trips SET fare = fare + 1;   -- lần 1: page bị sửa lần đầu sau checkpoint
UPDATE ops.trips SET fare = fare + 1;   -- lần 2: ngay sau đó, không có checkpoint xen giữa
```

```text
MEASURE 045 same 100-row UPDATE right after CHECKPOINT vs repeated: walBytes [29952,7544]
```

Lần đầu **29.952 byte**, lần lặp **7.544 byte**. Runner assert lần đầu lớn hơn lần sau. Hiệu 22.408 byte nhỏ hơn ba page 8 KB, khớp với giả thuyết vài page (heap, có thể kèm page khác) được ghi nguyên ảnh ở lần sửa đầu sau checkpoint. FPI có thể nhỏ hơn 8.192 byte vì phần trống giữa page thường không được ghi. Lần lặp 7.544 byte, khoảng 75 byte cho mỗi dòng, gần với chi phí thuần của 100 bản ghi cập nhật.

Lab **không giải mã WAL**, nên đây là quan sát phù hợp với cơ chế, chưa phải phép đếm FPI. Bài tập 2 dùng `pg_walinspect` để đếm thật.

Bài học thực tế: ước lượng dung lượng WAL hoặc archive bằng «kích thước cột × số UPDATE» sẽ sai. Cùng một câu có thể sinh WAL gấp bốn lần tuỳ vị trí so với checkpoint, chưa kể index và việc nền.

### 6.4. Log checkpoint

Cluster bật `log_checkpoints`. Runner trích ba dòng `checkpoint complete` **đầu tiên** của log server và cắt bỏ phần đuôi (số file sync, thời gian dài nhất/trung bình, distance, estimate, LSN):

```text
LOG:  checkpoint complete: wrote 71 buffers (0.4%), wrote 3 SLRU buffers; 0 WAL file(s) added, 0 removed, 0 recycled; write=0.021 s, sync=0.026 s, total=0.096 s
LOG:  checkpoint complete: wrote 316 buffers (1.9%), wrote 3 SLRU buffers; 0 WAL file(s) added, 0 removed, 1 recycled; write=0.019 s, sync=0.027 s, total=0.062 s
LOG:  checkpoint complete: wrote 4 buffers (0.0%), wrote 1 SLRU buffers; 0 WAL file(s) added, 0 removed, 0 recycled; write=0.002 s, sync=0.002 s, total=0.008 s
```

- `wrote N buffers (x%)`: số page bẩn checkpoint đã ghi, phần trăm so với `shared_buffers` (128 MB ≈ 16.384 buffer).
- `SLRU buffers`: page của các cấu trúc nhỏ như trạng thái commit của giao dịch.
- `WAL file(s) added, removed, recycled`: file WAL tạo mới, xoá, hoặc đổi tên để dùng lại sau khi đã qua điểm redo.
- `write`, `sync`, `total`: thời gian ghi, flush và tổng của checkpoint.

Runner không gắn nhãn dòng nào ứng với lệnh nào, và timestamp đã bị che. Phần PG-042 có khởi động lại server, nên dòng đầu nhiều khả năng là checkpoint lúc tắt server ở bước đó, không thuộc vòng đo. Đọc ba dòng này như mẫu định dạng và độ lớn: checkpoint trên dữ liệu lab nhỏ ghi vài buffer tới vài trăm buffer, trong vài chục mili-giây.

## 7. Lab chứng minh gì và không chứng minh gì

README của lab ghi rõ: fixture 100 dòng, cache ấm, không tải đồng thời, mỗi số đo gồm chi phí mở `psql`, WAL đo ở cấp cluster, và **không có nhóm đối chứng** «không checkpoint». Vì vậy:

- **Chứng minh được:** ba tham số độ bền đang bật; lệnh `CHECKPOINT` được đếm trong `pg_stat_checkpointer`; trong lần chạy này, `UPDATE` đầu tiên sau checkpoint sinh nhiều WAL hơn hẳn chính câu đó lặp lại — phù hợp với cơ chế full-page write.
- **Không chứng minh được:** checkpoint làm production chậm bao nhiêu; `checkpoint_timeout` hay `max_wal_size` nào tối ưu; `synchronous_commit = off` nhanh hơn bao nhiêu; dữ liệu sống sót sau mất điện.

Muốn quyết định tuning (chưa làm trong lab): giữ nguyên dữ liệu, phân bố cập nhật, số client và storage; chạy qua nhiều chu kỳ checkpoint; thu throughput, percentile độ trễ, tốc độ WAL, số FPI và thời gian checkpoint; mỗi lần đổi **một** tham số với giả thuyết cụ thể, lặp nhiều lần. View `pg_stat_wal` cho `wal_records`, `wal_fpi`, `wal_bytes` toàn cluster. Log cảnh báo «checkpoints are occurring too frequently» (ngưỡng `checkpoint_warning`, mặc định 30 giây) là dấu hiệu `max_wal_size` quá nhỏ.

## 8. Checkpoint không phải backup

Checkpoint chỉ dọn đường cho crash recovery trên **chính** cluster đó. Nó không tạo bản sao độc lập để mang sang máy khác, không bảo vệ khỏi `DELETE` nhầm, mất ổ hay mã hoá tống tiền. Backup logic là PG-046; base backup cộng chuỗi WAL archive để khôi phục theo thời điểm là [PG-048](/articles/postgresql-pitr-base-backup-wal).

Ngược lại, chính chuỗi WAL là nguyên liệu của PITR và replication, nên WAL bị **giữ lại** khi archive lỗi hoặc replication slot không được tiêu thụ — bất kể checkpoint. `pg_wal` khi đó phình vượt `max_wal_size`, và nếu đầy đĩa, server không ghi WAL được nữa. Checkpoint thành công **không** chứng minh WAL đã tới kho backup; muốn biết phải xem `pg_stat_archiver` (PG-048).

## 9. Lỗi thường gặp

| Hiểu sai / triệu chứng | Thực tế | Cách xử lý |
|---|---|---|
| «Commit xong là dữ liệu đã nằm trong file bảng» | Commit chỉ chờ WAL flush; page ghi sau | Hiểu độ bền đến từ WAL + redo |
| Checkpoint dày cho an toàn | Độ bền không phụ thuộc checkpoint; dày hơn chỉ rút ngắn recovery, tăng I/O và FPI | Chọn theo mục tiêu thời gian khôi phục, đo trước/sau |
| Bật `synchronous_commit = off` toàn cluster cho nhanh | Có thể mất giao dịch đã báo thành công | Chỉ dùng theo giao dịch cho dữ liệu chấp nhận mất |
| Tắt `fsync` vì benchmark không có crash | Rủi ro hỏng dữ liệu không phục hồi | Giữ `on`; benchmark không crash không chứng minh an toàn |
| Dự toán WAL = kích thước cột × số UPDATE | FPI làm cùng câu sinh WAL gấp nhiều lần (lab: 29.952 vs 7.544) | Đo `pg_stat_wal`, `pg_walinspect` trên workload thật |
| `pg_wal` vượt `max_wal_size`, đĩa sắp đầy | Archive lỗi hoặc replication slot bị bỏ rơi giữ WAL | Kiểm `pg_stat_archiver`, `pg_replication_slots` trước khi đổi checkpoint |

## 10. Đánh đổi

- **Độ bền và độ trễ commit.** `synchronous_commit = on` buộc mỗi commit chờ một lần flush; `off` bỏ chờ, đổi lấy cửa sổ mất dữ liệu ngắn. Ranh giới nên đặt theo loại dữ liệu: thanh toán `on`, vị trí tài xế có thể cân nhắc `off` theo giao dịch.
- **Tần suất checkpoint.** Dày: khôi phục nhanh, nhiều FPI và I/O. Thưa: ít FPI, I/O mượt, nhưng khôi phục lâu hơn và `pg_wal` lớn hơn.
- **`wal_compression`.** Giảm WAL và băng thông archive/replica, tốn CPU. Đáng thử khi FPI chiếm phần lớn WAL — phải đo.

## 11. Bài tập

**Bài 1. Phép thử dừng đột ngột.** Trên cluster lab riêng (không phải máy có dữ liệu thật): ghi lại `pg_control_checkpoint()`, commit một khoản thanh toán, ghi nhận client đã nhận `COMMIT`, rồi dừng server bằng `pg_ctl stop -m immediate` (dừng không checkpoint, mô phỏng sập tiến trình chứ không mô phỏng mất điện phần cứng). Khởi động lại và kiểm tra.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Ba bằng chứng nối với nhau: (1) client nhận <code>COMMIT</code> trước khi dừng; (2) log khởi động lại cho thấy server đã chạy crash recovery và bắt đầu redo từ một LSN — so LSN đó với <code>redo_lsn</code> của <code>pg_control_checkpoint()</code> đã ghi trước khi dừng; (3) dòng thanh toán có mặt sau khi khởi động. Phải nói rõ giới hạn: <code>-m immediate</code> không kiểm cache ghi của ổ đĩa, nên đây không phải bằng chứng cho mất điện thật. Lab P05 chưa chạy phép thử này; ghi output của chính bạn.</p>
</details>

**Bài 2. Đếm full-page image thật.** Lặp lại thí nghiệm mục 6.3, nhưng ghi LSN ở bốn mốc và dùng extension `pg_walinspect` (có từ PostgreSQL 15) để thống kê bản ghi giữa từng cặp mốc.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Với mỗi lần <code>UPDATE</code>: tổng byte WAL, số bản ghi theo loại và số FPI (ví dụ qua <code>pg_get_wal_stats(start_lsn, end_lsn, true)</code>). Kết luận «chênh lệch do FPI» chỉ hợp lệ nếu lần đầu sau checkpoint có FPI còn lần lặp có ít hoặc không có, và phần byte FPI giải thích được phần lớn chênh lệch. Liệt kê cả bản ghi không đến từ câu <code>UPDATE</code> (autovacuum, việc nền) nếu có — chúng là lý do số của lab dao động.</p>
</details>

**Bài 3. Báo cáo tần suất checkpoint có đối chứng.** Trên bản sao lab lớn hơn, so sánh `checkpoint_timeout` 5 phút và 30 phút với cùng workload ghi.

<details><summary><b>Lời giải phải chứng minh</b></summary>
<p>Ghi đủ cấu hình (phần cứng, phiên bản, <code>shared_buffers</code>, <code>max_wal_size</code>), số dòng, số client, thời gian chạy đủ nhiều chu kỳ checkpoint, số lần lặp mỗi cấu hình. Thu <code>wal_bytes</code>/<code>wal_fpi</code> từ <code>pg_stat_wal</code>, <code>num_timed</code>/<code>num_requested</code>, throughput và p95/p99 độ trễ đo trong một kết nối giữ nguyên, và — nếu làm Bài 1 — thời gian recovery. Chênh lệch nào nằm trong biên dao động giữa các lần lặp thì không được quy cho checkpoint. Báo cáo kết thúc bằng phần «giới hạn kết luận».</p>
</details>

## 12. Đọc tiếp

- [PostgreSQL 18 · Write-Ahead Logging (WAL)](https://www.postgresql.org/docs/18/wal-intro.html) và [Reliability](https://www.postgresql.org/docs/18/wal-reliability.html): WAL trước dữ liệu, cache ghi của ổ đĩa, page rách.
- [PostgreSQL 18 · WAL Configuration](https://www.postgresql.org/docs/18/wal-configuration.html): checkpoint, điểm redo, `max_wal_size`, `checkpoint_completion_target`.
- [PostgreSQL 18 · Write Ahead Log settings](https://www.postgresql.org/docs/18/runtime-config-wal.html): `fsync`, `synchronous_commit`, `full_page_writes`, `wal_compression`.
- [PostgreSQL 18 · Cumulative Statistics](https://www.postgresql.org/docs/18/monitoring-stats.html): `pg_stat_checkpointer`, `pg_stat_wal`, `pg_stat_archiver`.
- [PostgreSQL 18 · pg_walinspect](https://www.postgresql.org/docs/18/pgwalinspect.html).
- Lab: `examples/toi-noi-postgresql/p05/run.mjs` (khối PG-045), `p05/OUTPUT.txt`, giới hạn phép đo trong `p05/README.md`.

Bài tiếp theo: [PG-046 · Backup logic và khôi phục với pg_dump](/articles/postgresql-pg-dump-restore).
