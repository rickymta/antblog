---
slug: postgresql-replication-primary-standby
title: "Primary, standby và replication lag: vì sao khách vừa đặt chuyến lại chưa thấy đơn"
summary: "App trả «đặt chuyến thành công» nhưng màn lịch sử đọc từ replica vẫn trống. Bài dựng standby thật bằng pg_basebackup -R trên PostgreSQL 18.1, đọc pg_stat_replication, tạm dừng replay để thấy một chuyến đã commit trên primary mà standby đã nhận nhưng chưa hiện, rồi đọc-sau-ghi bằng cách chờ tới LSN commit — kèm cách đo độ trễ bằng byte, bằng thời gian và các chiến lược định tuyến đọc."
category: postgresql
level: beginner
tags: [postgresql, pg-052, replication, streaming-replication, hot-standby, replication-lag, read-your-writes, toi-noi]
language: vi
---

19:05 tối thứ Sáu, chị Lan ở Hà Nội đặt một chuyến 30.000 đ trên app Tới Nơi. API đặt chuyến ghi vào PostgreSQL, nhận `COMMIT`, trả về «Đặt chuyến thành công» kèm mã chuyến. Chị bấm sang tab «Chuyến của tôi» — danh sách trống. Kéo xuống làm mới lần nữa thì chuyến hiện ra. Chị nhắn tổng đài hỏi có bị đặt hai lần không.

Không có gì bị mất. Tuần trước, đội backend chuyển các màn hình chỉ đọc — lịch sử chuyến, báo cáo tài xế — sang một **bản sao** của database để giảm tải cho máy chính. Lệnh ghi đi vào máy chính, lệnh đọc ngay sau đó đi vào bản sao, và bản sao chưa kịp có chuyến vừa đặt. «Đã commit» và «mọi bản sao đều đọc thấy» là hai thời điểm khác nhau.

Bài này dựng đúng tình huống đó trên hai cluster PostgreSQL 18.1 thật, cố ý giữ bản sao ở trạng thái «đã nhận nhưng chưa áp», rồi đọc lại đúng cách: chờ bản sao áp tới vị trí commit của chính giao dịch vừa ghi.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-052 · Phần 6/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/postgresql-wal-checkpoint-durability">PG-045 · WAL, checkpoint và độ bền dữ liệu</a> và <a href="/articles/postgresql-pitr-base-backup-wal">PG-048 · PITR với base backup và WAL</a>. Lab: <code>examples/toi-noi-postgresql/p06</code>, PostgreSQL 18.1 thật; trong <code>examples/toi-noi-postgresql</code> chạy <code>npm run p06</code> (cần PostgreSQL 18 và biến <code>PG17_BIN</code> trỏ tới thư mục bin của PostgreSQL 17 cho bài nâng phiên bản, xem <code>p06/README.md</code>). Runner tự tạo và tự xoá mọi cluster tạm. Sau bài này, bạn quan sát được sao chép trong lab, đọc được các vị trí WAL trên primary và standby, diễn giải độ trễ đọc và chọn được đường đọc phù hợp cho từng màn hình.</p>
</div>

## 1. Primary, standby và vì sao cần bản sao

**Primary** là máy chủ PostgreSQL nhận ghi. **Standby** là máy chủ giữ một bản sao của toàn bộ cluster và liên tục cập nhật bản sao đó từ primary. Khi standby cho phép chạy truy vấn chỉ đọc trong lúc vẫn đang cập nhật, ta gọi nó là **hot standby** (mặc định bật qua `hot_standby = on`). Trong các tài liệu và công cụ khác, hot standby thường được gọi là **read replica**.

Cơ chế cập nhật trong bài là **physical streaming replication**: primary gửi chính các bản ghi WAL của nó sang standby qua mạng, standby phát lại chúng. PG-045 đã giải thích **WAL** (write-ahead log): mọi thay đổi được ghi vào nhật ký trước khi ghi vào trang dữ liệu, để sau crash có thể phát lại. Streaming replication dùng lại đúng nhật ký đó, chỉ khác là phát lại *trên máy khác* và *không bao giờ dừng*. Vì sao chép từng byte của trang dữ liệu, standby phải cùng phiên bản lớn, cùng kiến trúc với primary và chứa *mọi* database trong cluster. Sao chép một vài bảng sang hệ khác là logical replication, chủ đề của PG-055.

Tới Nơi dùng standby cho hai việc nên tách bạch: **giảm tải đọc** (lịch sử chuyến, báo cáo tài xế, tra cứu của tổng đài — nơi nảy sinh câu chuyện của chị Lan) và **sẵn sàng cao** (primary chết thì đưa standby lên thay, chủ đề của PG-058).

Standby **không phải backup**. Một câu `DELETE` nhầm trên primary sẽ sang standby trong chưa tới một giây; muốn quay ngược thời gian vẫn cần base backup và WAL archive của PG-048.

## 2. Một giao dịch đi qua bốn vị trí WAL

Mỗi điểm trong dòng WAL có một địa chỉ gọi là **LSN** (log sequence number), viết dạng `0/40002B8`: vị trí byte tính từ đầu lịch sử WAL. Hai LSN trừ nhau ra số byte WAL nằm giữa chúng.

Khi chị Lan đặt chuyến, giao dịch đi qua các tiến trình sau:

```text
PRIMARY                                       STANDBY
backend: INSERT + COMMIT
  └─ ghi WAL, flush xuống đĩa ─► trả COMMIT cho API
walsender ── gửi WAL (sent_lsn) ──────────►  walreceiver
                                              ├─ ghi vào file      (write_lsn)
                                              ├─ flush xuống đĩa   (flush_lsn)
                                              └─ startup process phát lại
                                                 vào dữ liệu       (replay_lsn)
                                                 ► SELECT mới thấy chuyến
```

- **walsender** là tiến trình trên primary phục vụ một kết nối replication; mỗi standby có một walsender.
- **walreceiver** là tiến trình trên standby nhận WAL và ghi xuống `pg_wal` của standby.
- **startup process** trên standby đọc WAL đã nhận và áp vào trang dữ liệu — gọi là **replay**.

View `pg_stat_replication` trên primary có một dòng cho mỗi standby, với bốn cột vị trí: `sent_lsn` (đã gửi), `write_lsn` (standby đã ghi vào file), `flush_lsn` (standby đã flush xuống đĩa) và `replay_lsn` (standby đã phát lại vào dữ liệu). Phía standby có hai hàm tương ứng: `pg_last_wal_receive_lsn()` — vị trí cuối cùng đã nhận và đồng bộ xuống đĩa — và `pg_last_wal_replay_lsn()` — vị trí cuối cùng đã phát lại.

Điểm mấu chốt nằm ở dòng cuối của sơ đồ. Theo [tài liệu Hot Standby](https://www.postgresql.org/docs/18/hot-standby.html), thay đổi của một giao dịch chỉ hiện ra cho các snapshot mới trên standby **sau khi bản ghi commit của nó được replay**. Nhận về rồi, ghi xuống đĩa rồi, vẫn chưa đủ.

Và vì replication **mặc định bất đồng bộ** ([warm-standby](https://www.postgresql.org/docs/18/warm-standby.html)), primary trả `COMMIT` cho API ngay khi WAL đã bền trên đĩa *của chính nó*, không chờ standby nhận hay replay. Tài liệu mô tả độ trễ giữa commit trên primary và lúc thay đổi hiện trên standby thường dưới một giây nếu standby đủ mạnh. «Thường dưới một giây» vẫn đủ dài để một người bấm sang tab khác.

## 3. Dựng standby bằng pg_basebackup -R

Runner tạo primary (cổng `<PRIMARY>`), nạp fixture, chạy các bài trước của phần 6, rồi dựng standby. Lệnh lấy bản sao, đúng như trong `run.mjs`:

```text
pg_basebackup -h 127.0.0.1 -p <PRIMARY> -U postgres -D <standby-data> \
              -R -X stream --checkpoint=fast
```

- `-D`: thư mục đích, sẽ thành data directory của standby. Runner tạo một cluster tạm để lấy cổng và thư mục, dừng nó, đổi tên data directory ban đầu sang chỗ khác rồi để `pg_basebackup` ghi vào đúng vị trí cũ.
- `-X stream`: stream WAL phát sinh trong lúc chép, để bản sao tự đủ WAL tới điểm nhất quán (PG-048 đã giải thích).
- `--checkpoint=fast`: yêu cầu checkpoint ngay để bắt đầu chép sớm.
- `-R`: phần mới so với PG-048. Theo [tài liệu pg_basebackup](https://www.postgresql.org/docs/18/app-pgbasebackup.html), tuỳ chọn này tạo file `standby.signal` và thêm thiết lập kết nối tới primary vào `postgresql.auto.conf` của thư mục đích.

Runner kiểm hai thứ `-R` tạo ra:

```text
PASS 052 standby.signal written by -R: true
--- 052 postgresql.auto.conf from pg_basebackup -R
  primary_conninfo = 'user=postgres host=127.0.0.1 port=<PRIMARY> client_encoding=UTF8
```

- **`standby.signal`**: file rỗng. Theo tài liệu, server khởi động ở **chế độ standby** nếu file này có trong data directory: phát lại WAL mãi mãi, không tự kết thúc recovery. (PG-048 dùng `recovery.signal` — phát lại tới một mục tiêu rồi dừng. Hai file, hai chế độ.)
- **`primary_conninfo`**: chuỗi kết nối walreceiver dùng để gọi primary. Runner đã **lược bớt** dòng này trước khi in: bỏ `passfile` và một loạt tuỳ chọn SSL/GSS mà `pg_basebackup` ghi kèm. Bộ lọc xoá luôn tuỳ chọn cuối dòng cùng dấu nháy đóng, nên dòng trên trông như thiếu `'` — trong file thật thì không thiếu. Chuỗi không có `application_name`, nên ở phía primary walreceiver sẽ hiện với tên mặc định `walreceiver`.

Một chi tiết hay gây lỗi: `pg_basebackup` chép **toàn bộ** data directory, kể cả `postgresql.conf` của primary. Trên cùng một máy, standby sẽ cố mở đúng cổng của primary, nên runner ghi thêm `port = <STANDBY>` vào cuối `postgresql.conf` của standby trước khi khởi động. Trên hai máy riêng, mọi thiết lập khác của primary — `wal_keep_size`, `archive_command`… — cũng sang theo và cần được duyệt lại.

Kết nối replication cần role có thuộc tính `REPLICATION` và một dòng `host replication …` trong `pg_hba.conf` của primary. Lab dùng `trust` trên loopback như các phần trước — chỉ hợp với cluster tạm. Production dùng role riêng, SCRAM và TLS như PG-042.

## 4. Xác nhận standby đang sao chép

Sau khi khởi động standby, runner chờ tới khi primary thấy một walsender ở trạng thái `streaming`, rồi kiểm từ hai phía:

```text
PASS 052 standby in recovery: t
--- 052 pg_stat_replication on primary
   application_name |   state   | sync_state | caught_up 
  ------------------+-----------+------------+-----------
   walreceiver      | streaming | async      | t
  (1 row)
```

- **`pg_is_in_recovery()` = `t`** trên standby: server đang ở chế độ phát lại WAL. Đây là câu hỏi nhanh nhất để biết mình đang nối vào primary hay standby — primary trả `f`.
- **`state = streaming`**: theo [tài liệu thống kê](https://www.postgresql.org/docs/18/monitoring-stats.html), walsender đang stream thay đổi sau khi standby đã bắt kịp primary. Trước đó bạn có thể thấy `startup` hoặc `catchup`.
- **`sync_state = async`**: commit trên primary không chờ standby này — mặc định khi `synchronous_standby_names` rỗng.
- **`caught_up`** không phải cột của view: runner tự tính `sent_lsn = replay_lsn`. Lúc chụp, primary không có ghi mới nên hai vị trí trùng nhau; trên hệ thống đang chạy, `t` chỉ là một khoảnh khắc.

Khi tự quan sát, câu truy vấn đầy đủ hơn trên primary là:

```sql
SELECT application_name, state, sync_state,
       sent_lsn, write_lsn, flush_lsn, replay_lsn,
       pg_wal_lsn_diff(pg_current_wal_lsn(), replay_lsn) AS replay_lag_bytes,
       write_lag, flush_lag, replay_lag
FROM pg_stat_replication;
```

## 5. Standby từ chối ghi

Ứng dụng cấu hình sai, gửi lệnh ghi sang standby, sẽ nhận:

```text
--- 052 write on standby
  ERROR:  25006: cannot execute INSERT in a read-only transaction
```

SQLSTATE **`25006`** là `read_only_sql_transaction`. Theo tài liệu Hot Standby, giao dịch trên standby không được cấp mã giao dịch và không thể ghi WAL; `transaction_read_only` luôn là true. Không chỉ DML bị từ chối: DDL (kể cả bảng tạm), `SELECT … FOR UPDATE`, `nextval()`, `LISTEN`/`NOTIFY` cũng vậy. Khi chuyển một màn hình sang replica, hãy kiểm các hàm «chỉ đọc» có gọi `nextval()` hay ghi bảng log. Ngược lại, nếu pool ghi bỗng nhận `25006`, nó đang nối vào một server trong recovery — có thể vì định tuyến chưa đổi sau failover (PG-058).

## 6. Dựng lại lỗi «vừa đặt chưa thấy»

Trên hai cluster cùng một máy, standby bắt kịp gần như tức thì; muốn thấy khoảng trễ phải giữ nó lại. Runner dùng các hàm điều khiển recovery của [tài liệu chức năng quản trị](https://www.postgresql.org/docs/18/functions-admin.html):

```sql
-- Trên standby
SELECT pg_wal_replay_pause();
-- chờ tới khi pg_get_wal_replay_pause_state() = 'paused'

-- Trên primary: chuyến của chị Lan
INSERT INTO fares VALUES (4, 'HN', 30000);
SELECT pg_current_wal_flush_lsn();          -- ghi lại làm «LSN commit»

-- Trên standby: chờ tới khi đã NHẬN tới LSN đó
SELECT pg_last_wal_receive_lsn() >= '<LSN commit>';
SELECT count(*) FROM fares WHERE id = 4;
```

Runner chờ đúng trạng thái `paused` chứ không chỉ gọi hàm, vì tài liệu nói rõ `pg_wal_replay_pause()` chỉ là *yêu cầu* tạm dừng; muốn chắc phải xem `pg_get_wal_replay_pause_state()`. Kết quả:

```text
PASS 052 standby received WAL but row not visible: 0
MEASURE 052 standby received-but-not-replayed bytes: 696
--- 052 pg_stat_replication while replay paused
   application_name | flushed_on_standby | replayed 
  ------------------+--------------------+----------
   walreceiver      | t                  | f
  (1 row)
```

Câu chuyện của chị Lan, nhìn từ bên trong:

1. **Primary đã commit** chuyến 4 — API đã có thể trả «thành công».
2. **Standby đã nhận và flush** WAL chứa commit đó: `flush_lsn >= LSN commit` là `t`. Dữ liệu đã nằm an toàn trên đĩa của standby.
3. **Standby chưa replay**: `replay_lsn >= LSN commit` là `f`, và `SELECT` trên standby đếm được **0** dòng cho chuyến 4.

Khoảng cách giữa vị trí đã nhận và đã phát lại trên standby là **696 byte**: WAL của chuyến 4 (INSERT, commit) và có thể vài bản ghi nền; lab không phân rã con số này, lần chạy khác có thể khác. Điều đáng nhớ: dữ liệu có thể đã *ở trên đĩa* standby mà vẫn *chưa nhìn thấy được* qua SQL.

**Tạm dừng replay là cách dựng lỗi có kiểm soát**, không phải phép đo độ trễ thật; lab không đo standby thường chậm bao nhiêu mili giây. Và trong lúc pause, mọi truy vấn mới trên standby thấy cùng một snapshot — để quên một standby đang pause là để mọi màn hình đọc từ nó đứng yên ở quá khứ.

Ngoài đời, replay chậm vì những lý do không cần ai gọi pause: mạng chậm, standby yếu hơn primary, một đợt ghi lớn, hoặc **xung đột recovery**. Khi một truy vấn dài trên standby đang đọc những dòng mà WAL sắp dọn đi, startup process sẽ chờ truy vấn đó tối đa `max_standby_streaming_delay` rồi mới huỷ nó. Trong lúc chờ, replay đứng — và chuyến của chị Lan cũng đứng. Một báo cáo tài xế chạy lâu trên cùng replica với màn lịch sử chuyến là công thức cho đúng lỗi đầu bài. Lab **không chạy** kịch bản xung đột này.

## 7. Độ trễ đo bằng byte và đo bằng thời gian

«Replica trễ bao nhiêu?» có hai cách trả lời, và chúng trả lời hai câu hỏi khác nhau.

**Theo byte.** Hiệu hai LSN cho biết còn bao nhiêu WAL chưa đi qua một khâu. Tài liệu warm-standby gợi ý so `pg_current_wal_lsn()` trên primary với `pg_last_wal_receive_lsn()` trên standby. Mỗi khoảng cách chỉ ra một khâu khác nhau:

| Khoảng cách | Đo ở | Gợi ý nguyên nhân |
|---|---|---|
| `pg_current_wal_lsn()` − `sent_lsn` | primary | Primary bận, walsender gửi không kịp |
| `sent_lsn` − `flush_lsn` | primary | Mạng hoặc đĩa standby chậm |
| `flush_lsn` − `replay_lsn` | primary | Replay chậm: tải nặng, xung đột recovery, đang pause |
| `pg_last_wal_receive_lsn()` − `pg_last_wal_replay_lsn()` | standby | Như dòng trên, nhìn từ standby (lab: 696 byte) |

Byte tốt để **tìm khâu chậm** và cảnh báo «standby tụt quá xa». Nhưng 696 byte là nhanh hay chậm? Không trả lời được nếu không biết standby replay bao nhiêu byte mỗi giây *lúc đó*: vài trăm byte khi replay đang bị một truy vấn chặn có thể là cả chục giây.

**Theo thời gian.** Câu hỏi của khách là «bao lâu nữa tôi thấy chuyến», nên đơn vị đúng là giây. PostgreSQL cho hai nguồn, lab **không đo** cả hai:

- Cột `write_lag`, `flush_lag`, `replay_lag` trong `pg_stat_replication`. Tài liệu nói với standby bất đồng bộ, `replay_lag` xấp xỉ độ trễ trước khi giao dịch mới hiện ra cho truy vấn. Khi hệ thống rảnh và đã bắt kịp, giá trị gần nhất còn hiện một lúc rồi thành `NULL`.
- `now() - pg_last_xact_replay_timestamp()` trên standby: thời điểm (theo đồng hồ primary) của giao dịch cuối cùng đã replay. Cẩn thận: nếu primary không có giao dịch nào trong năm phút, phép trừ này tăng lên năm phút dù standby chẳng trễ chút nào. Nó cũng phụ thuộc đồng hồ hai máy khớp nhau.

Giám sát tốt dùng cả hai. Nhưng **không con số trung bình nào sửa được lỗi của chị Lan**: độ trễ trung bình thấp vẫn có những lúc vọt lên, và người đọc đúng lúc đó vẫn thấy danh sách trống. Muốn đảm bảo, phải chờ theo LSN của chính giao dịch đó.

## 8. Đọc-sau-ghi bằng LSN

**Read-your-writes** (đọc-thấy-ghi-của-mình) là bảo đảm: sau khi một người dùng ghi thành công, các lần đọc *của chính người đó* phải thấy thay đổi. Nó yếu hơn «mọi người thấy ngay lập tức» và rẻ hơn nhiều. Lab kiểm bảo đảm này bằng cách chờ standby replay tới LSN commit rồi mới đọc:

```sql
-- Trên standby, lặp tới khi true (runner chờ tối đa 20 giây)
SELECT pg_last_wal_replay_lsn() >= '<LSN commit>';
-- rồi mới đọc
SELECT count(*) FROM fares WHERE id = 4;
```

```text
PASS 052 read-your-writes after waiting for commit LSN: 1
```

Cùng câu `SELECT`, cùng standby, lúc trước trả 0, giờ trả 1. Đưa vào API của Tới Nơi:

1. API đặt chuyến commit trên primary, rồi lấy vị trí WAL trên **cùng kết nối** đó. Lab dùng `pg_current_wal_flush_lsn()` với `synchronous_commit` mặc định (`on`): khi `COMMIT` trả về, bản ghi commit đã được flush, nên vị trí flush lúc này đã ở sau nó. Nếu một giao dịch tắt `synchronous_commit`, vị trí flush có thể *chưa* vượt qua commit; khi đó dùng `pg_current_wal_insert_lsn()` cho chắc.
2. API trả LSN đó về client cùng mã chuyến, ví dụ trong một cookie hay header, hoặc giữ trong session phía server.
3. Lần đọc tiếp theo của người đó mang theo LSN. Lớp truy cập dữ liệu hỏi replica `pg_last_wal_replay_lsn() >= $1`. Đạt thì đọc từ replica. Chưa đạt thì chờ một hạn ngắn (vài trăm mili giây, do bạn đặt) rồi **chuyển sang đọc primary**, không bắt khách chờ vô hạn.
4. Nếu hàm trả `NULL` — server không ở recovery, ví dụ replica vừa được promote — đừng so sánh, đọc primary.

Trong tài liệu chức năng quản trị của bản 18, ta không thấy hàm dựng sẵn nào «chờ tới LSN»; lab tự lặp truy vấn, và ứng dụng cũng phải tự lặp hoặc tự quyết định thử lại.

Một giới hạn quan trọng: LSN chỉ so sánh được **trong cùng một lịch sử WAL**. Sau failover, primary mới chạy trên timeline khác, và một LSN cấp từ primary cũ có thể bị vượt qua về con số mà dữ liệu tương ứng không bao giờ có mặt. PG-058 cho thấy đúng trường hợp chuyến đã commit bị mất khi failover; token LSN cũ phải bị vô hiệu cùng lúc.

## 9. Chiến lược định tuyến đọc

LSN token không phải cách duy nhất. Với từng màn hình của Tới Nơi, câu hỏi đúng là: **màn hình này chịu được dữ liệu cũ bao lâu?**

| Chiến lược | Cách làm | Hợp với | Cái giá |
|---|---|---|---|
| Trả luôn trong phản hồi ghi | API đặt chuyến trả đủ thông tin chuyến; app chèn vào danh sách tại chỗ | Màn «vừa đặt xong» | Hai nguồn hiển thị phải nhất quán về định dạng |
| Đọc primary trong cửa sổ ngắn sau ghi | Ghi nhớ «người này vừa ghi lúc T», đọc primary tới T + vài giây | Lịch sử chuyến, hồ sơ cá nhân | Cửa sổ là đoán; replica trễ lâu hơn cửa sổ thì vẫn lỗi |
| LSN token (mục 8) | Đọc replica khi đã replay tới LSN, không thì primary | Mọi màn sau thao tác ghi | Phải truyền token qua mọi tầng; vô hiệu khi failover |
| Luôn đọc primary | Không dùng replica cho luồng này | Thanh toán, số dư ví, trạng thái chuyến đang chạy | Tải dồn về primary |
| Replica, chấp nhận cũ | Đọc bất kỳ replica nào | Báo cáo ngày, thống kê, gợi ý | Phải định nghĩa và giám sát độ tươi |
| Synchronous `remote_apply` | Commit chờ standby replay xong | Ít luồng cần «mọi người thấy ngay» | Mọi commit chậm theo standby; standby chết thì commit treo |

Dòng cuối cấu hình như sau — **không chạy trong lab**, không có output:

```text
# primary, postgresql.conf — KHÔNG CHẠY TRONG LAB
synchronous_standby_names = 'FIRST 1 (standby_hn)'
synchronous_commit = remote_apply
# standby: thêm application_name=standby_hn vào primary_conninfo
```

Theo tài liệu warm-standby, `remote_apply` khiến mỗi commit chờ tới khi các standby đồng bộ báo đã replay giao dịch. Đổi lại, commit chậm thêm khứ hồi mạng và thời gian replay, và nếu standby đồng bộ duy nhất ngừng trả lời thì commit đứng chờ. Đó là quyết định kiến trúc, không phải công tắc để sửa một màn hình.

libpq cũng có `target_session_attrs` (`read-only`, `standby`, `prefer-standby`…) để chuỗi kết nối nhiều host tự chọn replica. Nó chọn *loại* server, không biết gì về độ trễ; PG-058 cho thấy nó hành xử thế nào khi failover.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Vừa ghi xong, đọc lại không thấy, làm mới thì thấy | Đọc từ replica bất đồng bộ chưa replay | Một trong các chiến lược ở mục 9; không «sửa» bằng `sleep` cố định |
| Khách bấm đặt lại vì không thấy chuyến, sinh đơn trùng | Coi đọc cũ là bằng chứng chưa ghi | Khoá idempotency cho lệnh tạo chuyến (PG-039); đọc cũ không bao giờ là lý do tạo lần hai |
| `25006 cannot execute … in a read-only transaction` | Lệnh ghi (hoặc `nextval()`, `FOR UPDATE`) đi vào standby | Tách pool ghi/đọc; rà các hàm «chỉ đọc» có ghi ngầm |
| Standby không lên được vì trùng cổng | `pg_basebackup` chép cả `postgresql.conf` của primary | Đặt lại `port` và duyệt các thiết lập chép sang |
| `replay_lag` trung bình thấp nhưng khách vẫn phàn nàn | Trung bình che đỉnh; xung đột recovery làm replay đứng từng lúc | Giám sát cả byte lẫn đỉnh; tách truy vấn dài sang replica riêng |
| Báo động lag tăng đều khi đêm vắng | `now() - pg_last_xact_replay_timestamp()` lúc primary không có giao dịch | Kết hợp với lag theo byte |

## 11. Đánh đổi

| Câu hỏi | Lựa chọn A | Lựa chọn B |
|---|---|---|
| Có dùng replica để đọc không? | Không: đơn giản, luôn nhất quán, primary chịu hết tải | Có: chia tải, nhưng mọi luồng đọc phải khai báo độ tươi |
| Replica phục vụ HA hay báo cáo? | Một replica cho cả hai: rẻ, nhưng truy vấn dài làm replay chậm | Hai replica: replica HA giữ `max_standby_streaming_delay` ngắn, replica báo cáo chịu trễ dài |
| Đảm bảo đọc-sau-ghi ở đâu? | Ở ứng dụng (LSN token, cửa sổ đọc primary) | Ở database (`remote_apply`): đơn giản cho ứng dụng, đắt cho mọi commit |

## 12. Bài tập

**Bài 1.** Trên bản chép riêng của lab, thay bước pause bằng một đợt ghi lớn trên primary (ví dụ chèn vài trăm nghìn dòng vào một bảng tạm thường), và trong lúc đó đo `pg_wal_lsn_diff(sent_lsn, replay_lsn)` cùng `replay_lag` mỗi 100 ms.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Một chuỗi số đo thật kèm thời điểm, cấu hình máy và số dòng đã chèn. Chỉ ra lúc nào lag theo byte tăng mà <code>replay_lag</code> chưa đổi (hoặc ngược lại) và giải thích vì sao hai cột trả lời hai câu hỏi khác nhau. Không suy ra «replica của Tới Nơi trễ X ms» từ một máy dev; ghi rõ giới hạn kết luận.</p>
</details>

**Bài 2.** Viết hàm đọc lịch sử chuyến nhận một LSN token: thử replica, chờ tối đa 300 ms, quá hạn thì đọc primary.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Ba ca kiểm thử có output thật: replica đã replay tới token (đọc replica), replica đang pause (rơi về primary sau đúng hạn chờ, vẫn thấy chuyến vừa đặt), replica trả <code>NULL</code> cho <code>pg_last_wal_replay_lsn()</code> (đọc primary, không so sánh). Nêu LSN được lấy ở đâu, trên kết nối nào, sau <code>COMMIT</code> hay trước, và vì sao vị trí đó không đứng trước bản ghi commit.</p>
</details>

**Bài 3.** Lập bảng định tuyến đọc cho năm màn hình của app Tới Nơi: lịch sử chuyến, chi tiết chuyến đang chạy, số dư ví, báo cáo thu nhập tuần của tài xế, tra cứu của tổng đài.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Mỗi màn có độ tươi chấp nhận được bằng số hoặc «phải thấy ghi của chính mình», chiến lược ở mục 9, và hành vi khi replica tụt quá ngưỡng. Ít nhất một màn phải luôn đọc primary, kèm lý do nghiệp vụ. Chỉ ra màn nào không được dùng chung replica với truy vấn báo cáo dài và vì sao.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 18 · Log-Shipping Standby Servers](https://www.postgresql.org/docs/18/warm-standby.html): `standby.signal`, streaming replication, giám sát độ trễ, replication đồng bộ.
- [PostgreSQL 18 · Hot Standby](https://www.postgresql.org/docs/18/hot-standby.html): lệnh được và không được chạy trên standby, xung đột recovery, `max_standby_streaming_delay`.
- [PostgreSQL 18 · pg_basebackup](https://www.postgresql.org/docs/18/app-pgbasebackup.html): `-R`, `-X stream`, `--checkpoint`.
- [PostgreSQL 18 · System Administration Functions](https://www.postgresql.org/docs/18/functions-admin.html): `pg_last_wal_receive_lsn`, `pg_last_wal_replay_lsn`, `pg_wal_replay_pause`, `pg_get_wal_replay_pause_state`, `pg_wal_lsn_diff`.
- [PostgreSQL 18 · pg_stat_replication](https://www.postgresql.org/docs/18/monitoring-stats.html): các cột vị trí, `sync_state`, `write_lag`/`flush_lag`/`replay_lag`.
- [PostgreSQL 18 · High Availability, Load Balancing, and Replication](https://www.postgresql.org/docs/18/high-availability.html): so sánh các giải pháp sao chép.
- Lab: `examples/toi-noi-postgresql/p06/run.mjs` (khối PG-052), kết quả `p06/OUTPUT.txt`, dữ liệu và giới hạn trong `p06/README.md`.

Bài trước: [PG-051 · Partition là gì và dùng khi nào?](/articles/postgresql-partition-co-ban). Bài tiếp theo: [PG-053 · JSONB và dữ liệu quan hệ](/articles/postgresql-jsonb-quan-he).
