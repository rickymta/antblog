---
slug: postgresql-pitr-base-backup-wal
title: "Khôi phục theo thời điểm với base backup và WAL: dừng đúng trước DELETE nhầm"
summary: "22:40, một script chạy DELETE không có WHERE trên bảng chuyến; bản dump 02:00 thiếu gần 21 giờ dữ liệu. Bài dựng PITR thật trên PostgreSQL 18.1: archive WAL, pg_basebackup, restore point, recovery.signal, dừng ở trạng thái pause để đối soát trên cluster riêng — và nói thẳng RPO, RTO đo được tới đâu."
category: postgresql
level: advanced
tags: [postgresql, pg-048, pitr, recovery, base-backup, wal-archive, rpo-rto, toi-noi]
language: vi
---

22:40 tối thứ Năm, script dọn dữ liệu thử nghiệm chạy nhầm sang database thật của Tới Nơi và thực thi `DELETE FROM ops.trips` — **không có `WHERE`**. PG-046 đã cho thấy bản `pg_dump` lúc 02:00 restore được, đối soát được, nhưng chỉ mang trạng thái lúc 02:00. Mọi chuyến từ sáng tới 22:39 — khách đã đi, tài xế đã chạy, tiền đã thu — không có trong đó.

Nhóm vận hành cần có trạng thái của database **ngay trước câu DELETE**. PostgreSQL làm được điều đó bằng **PITR** (point-in-time recovery — khôi phục theo thời điểm): lấy một bản chụp vật lý của cluster từ trước, rồi phát lại (replay) nhật ký WAL tới đúng điểm mình chọn và dừng ở đó.

Bài này chạy trọn chuỗi đó trên hai cluster PostgreSQL 18.1 tạm, dừng đúng trước DELETE và **đối soát khi còn đang pause**, rồi nói rõ lab chứng minh được gì về RPO và RTO, và chưa chứng minh được gì.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-048 · Phần 5/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-wal-checkpoint-durability">PG-045 · WAL, checkpoint và độ bền dữ liệu</a>, <a href="/articles/postgresql-pg-dump-restore">PG-046 · Backup logic với pg_dump</a> và <a href="/articles/postgresql-connection-pool-timeout">PG-047 · Connection pool và timeout</a>. Lab: <code>examples/toi-noi-postgresql/p05</code>, PostgreSQL 18.1 thật; chạy <code>npm run p05</code> trong <code>examples/toi-noi-postgresql</code>. Mọi lệnh phá dữ liệu chỉ chạy trên cluster tạm mà runner vừa tạo; không chạy câu DELETE mẫu trên database có sẵn. Sau bài này, bạn diễn tập được một lỗi thao tác, restore tới mốc trước lỗi trên cluster riêng và đối chiếu kết quả với mục tiêu RPO/RTO.</p>
</div>

## 1. Ba mảnh ghép của PITR

PG-045 đã giải thích **WAL** (write-ahead log): mọi thay đổi được ghi vào nhật ký trước khi trang dữ liệu được ghi xuống, để sau crash PostgreSQL phát lại nhật ký và về trạng thái nhất quán. PITR dùng đúng cơ chế đó, nhưng phát lại từ một bản backup cũ thay vì từ checkpoint gần nhất. Cần ba thứ:

1. **Base backup**: bản chép vật lý toàn bộ data directory, lấy trong khi cluster vẫn chạy. Bản chép này tự nó *không* nhất quán — file được chép trong lúc đang bị ghi — nhưng PostgreSQL biết phải phát lại WAL từ đâu để nó trở nên nhất quán.
2. **WAL archive liên tục**: mọi **segment** WAL (file nhật ký, mặc định 16 MB, tên dạng `000000010000000000000003`) phát sinh từ lúc bắt đầu base backup tới mốc muốn khôi phục, được chép sang một kho riêng. Thiếu một segment ở giữa là chuỗi bị đứt: không phát lại qua được chỗ đó.
3. **Mục tiêu khôi phục** (recovery target): điểm dừng. Có thể là một **restore point** có tên, một thời điểm, một mã giao dịch, một **LSN** (log sequence number — vị trí byte trong dòng WAL, dạng `0/4001DF0`), hoặc «ngay khi nhất quán».

```text
base backup ──► INSERT chuyến 101 ──► restore point ──► DELETE nhầm ──► switch WAL
 0/3000028                             0/4001DF0                         (archive)
                                           ▲
                                   điểm muốn dừng lại
```

Dừng ở trạng thái của base backup thì thiếu chuyến 101. Phát lại quá mốc thì nhận luôn DELETE. Cả hai lỗi đều cho một server khởi động bình thường và trả lời truy vấn bình thường — chỉ đối soát mới phân biệt được.

## 2. Archive WAL: hợp đồng của archive_command

Trên cluster nguồn, runner bật:

```text
wal_level = replica
archive_mode = on
archive_command = '"<node>" "<lab>/p05/wal-copy.mjs" archive "%p" "<ROOT>/archive/%f"'
```

`wal_level = replica` (mặc định) ghi đủ thông tin cho archive và replication. `archive_mode = on` bảo server gọi `archive_command` mỗi khi một segment hoàn tất; `%p` là đường dẫn file cần chép, `%f` là tên file. Theo tài liệu PostgreSQL, lệnh này phải tuân một hợp đồng nghiêm ngặt:

- Trả exit code **0 chỉ khi file đã được lưu an toàn**. Server coi 0 là «đã archive xong», và từ đó được phép xoá hoặc tái sử dụng segment trong `pg_wal`.
- Trả khác 0 khi thất bại. Server giữ segment và thử lại sau.
- Không ghi đè một file đã có trong archive. Ghi đè có thể phá một chuỗi WAL đang tốt bằng dữ liệu của một cluster khác — ví dụ khi hai cluster lỡ cấu hình cùng một kho.

Helper của lab chỉ có chín dòng, đủ để đọc hết:

```js
const [mode, source, target] = process.argv.slice(2);
try {
  if (!['archive', 'restore'].includes(mode) || !source || !target) throw Error('arguments');
  if (mode === 'archive' && existsSync(target)) {
    if (!readFileSync(source).equals(readFileSync(target))) throw Error('archive conflict');
  } else copyFileSync(source, target, mode === 'archive' ? constants.COPYFILE_EXCL : 0);
} catch { process.exitCode = 1; }
```

Khi archive: nếu file đích đã có thì chỉ chấp nhận khi nội dung **giống hệt** (server có thể thử archive lại một segment đã chép xong trước khi kịp ghi nhận), khác nội dung là lỗi; nếu chưa có thì chép với cờ «không ghi đè». Khi restore: chép từ kho về; file không có thì lỗi. Đây là kho trong thư mục tạm của lab — không có fsync riêng, không offsite, không retention, không mã hoá. README ghi rõ không dùng nó làm cơ chế backup production.

## 3. Base backup với pg_basebackup

```text
pg_basebackup -h 127.0.0.1 -p <PORT> -U postgres -D <ROOT>/base \
              -Fp -X stream --checkpoint=fast --label=p05_base
```

- `-Fp`: định dạng plain, kết quả là một data directory dùng được ngay.
- `-X stream`: mở thêm một kết nối replication để **stream WAL phát sinh trong lúc backup** vào thư mục `pg_wal` của bản backup. Nhờ vậy bản backup tự đủ để về trạng thái nhất quán, không phụ thuộc archive cho đoạn WAL đó.
- `--checkpoint=fast`: yêu cầu checkpoint ngay thay vì chờ checkpoint dàn trải; backup bắt đầu sớm hơn, đổi lại một đợt I/O ngắn trên nguồn.
- Không dùng `-R` (cấu hình standby). Kết nối replication dùng dòng `host replication postgres … trust` của lab — chỉ hợp với cluster tạm (PG-042).

Trong bản backup có file `backup_label`. Runner trích các dòng chính:

```text
START WAL LOCATION: 0/3000028 (file 000000010000000000000003)
CHECKPOINT LOCATION: 0/3000080
BACKUP METHOD: streamed
BACKUP FROM: primary
LABEL: p05_base
START TIMELINE: 1
```

- **START WAL LOCATION**: LSN nơi backup bắt đầu, và segment chứa nó (`…03`). Recovery cần WAL từ đây trở đi.
- **CHECKPOINT LOCATION**: bản ghi checkpoint mà recovery sẽ bắt đầu phát lại từ đó.
- **BACKUP METHOD: streamed**: WAL được stream (`-X stream`).
- **LABEL**: nhãn tự đặt, để biết file này thuộc lần backup nào.
- **START TIMELINE: 1**: lịch sử WAL hiện tại. Mỗi lần kết thúc một đợt archive recovery, PostgreSQL mở một timeline mới; PG-050 xem kỹ.

`backup_label` là thứ báo cho PostgreSQL «đây là bản backup, bắt đầu phát lại từ checkpoint này», chứ không phải «đây là cluster vừa crash». Xoá nó đi để server «khởi động được» là cách làm hỏng dữ liệu mà tài liệu đã cảnh báo rõ.

## 4. Đánh dấu mốc, rồi gây tai nạn có kiểm soát

Sau base backup, runner mô phỏng một ngày làm việc rút gọn rồi tới sự cố:

```sql
INSERT INTO ops.trips VALUES (101, 1, 45000, 'completed');   -- chuyến phát sinh sau backup
SELECT pg_create_restore_point('p05_before_delete');
DELETE FROM ops.trips;                                        -- «script 22:40»
SELECT pg_switch_wal();
```

```text
MEASURE 048 restore point p05_before_delete LSN: 0/4001DF0
PASS 048 source damaged deliberately: 0
```

`pg_create_restore_point` ghi một bản ghi có tên vào WAL và trả về LSN của nó. Trước đó, runner tính digest của bảng (101 chuyến) làm đáp án mong đợi cho bước đối soát.

`pg_switch_wal()` buộc server đóng segment hiện tại và chuyển sang segment mới. Việc này cần thiết vì `archive_command` chỉ chạy khi một segment **hoàn tất**: segment `…04` chứa restore point (ở `0/4001DF0`, tức mới vài KB đầu segment) và câu DELETE; nếu không switch thì nó nằm trong `pg_wal` tới khi đầy 16 MB. Runner đợi tới khi file xuất hiện trong kho **và** `pg_stat_archiver` ghi nhận:

```text
 archived_count |    last_archived_wal     | failed_count
----------------+--------------------------+--------------
              5 | 000000010000000000000004 |            0
(1 row)
```

`failed_count = 0` và `last_archived_wal` đã tới `…04`. Restore point mới chỉ nằm trên nguồn thì chưa đủ: nếu máy nguồn mất trước khi segment chứa nó vào kho, mốc đó không tồn tại với bất kỳ bản phục hồi nào. Lab không liệt kê năm file đã archive là những file nào; con số chỉ dùng để thấy archiver đang chạy và chưa thất bại lần nào.

**Sự cố thật không có restore point.** Không ai gọi `pg_create_restore_point` trước khi script chạy nhầm. Khi đó mục tiêu thường là:

- `recovery_target_time = '…22:39:59+07'`: dừng trước giao dịch commit sau thời điểm đó. Cần biết giờ đủ chính xác và để ý múi giờ.
- `recovery_target_xid` hoặc `recovery_target_lsn`: chính xác hơn, nếu tìm ra mã giao dịch/LSN của câu DELETE — ví dụ đọc WAL bằng `pg_waldump` hoặc từ log ứng dụng có ghi xid.
- `recovery_target_inclusive` (mặc định `on`) quyết định dừng ngay *sau* hay ngay *trước* target với các kiểu time/xid/lsn.

Lab **không chạy** ba kiểu target này (README ghi rõ). Theo tài liệu, mỗi lần chỉ được đặt **một** loại `recovery_target*`; đặt nhiều hơn là lỗi. Restore point có tên là cách chắc nhất khi biết trước mình sắp làm việc nguy hiểm: migration lớn, script dọn dữ liệu, thao tác tay trên production — gọi nó ngay trước khi chạy là một dòng SQL rẻ.

## 5. Dựng đích riêng

Không phục hồi đè lên nguồn, vì nguồn là bằng chứng, và trên đó còn có giao dịch hợp lệ *sau* 22:40 (mục 8). Runner tạo một cluster thứ hai, dừng nó, đổi tên data directory ban đầu sang `unused-initial-data` trong cùng thư mục tạm, chép base backup vào chỗ đó, rồi thêm cấu hình:

```text
port = <PORT_DICH>
archive_mode = off
ssl = off
restore_command = '"<node>" "<lab>/p05/wal-copy.mjs" restore "<ROOT>/archive/%f" "%p"'
recovery_target_name = 'p05_before_delete'
recovery_target_action = 'pause'
```

và tạo file rỗng `recovery.signal` trong data directory.

- **`recovery.signal`**: từ PostgreSQL 12, sự có mặt của file này bảo server khởi động ở chế độ **targeted recovery** — phát lại WAL từ archive tới target rồi kết thúc. (`standby.signal` là chế độ standby, phát lại mãi.) Không có file signal thì `backup_label` chỉ dẫn tới crash recovery với WAL đang có trong `pg_wal`, không đọc archive, không dừng ở target.
- **`restore_command`**: chiều ngược lại của `archive_command` — lấy file `%f` từ kho về `%p`. Theo tài liệu, server sẽ hỏi cả những file **không có** trong kho (ví dụ file `.history` của timeline tương lai, hay segment kế tiếp ở cuối chuỗi), nên lệnh phải trả khác 0 khi không tìm thấy thay vì treo hay tạo file rỗng.
- **`archive_mode = off`** ở đích: cấu hình chép từ nguồn có `archive_command` trỏ vào cùng kho. Để nguyên thì cluster phục hồi, khi mở ghi ở timeline mới, sẽ đẩy WAL của nó vào kho của nguồn. Helper từ chối ghi đè, nhưng tốt nhất là chặn ngay từ cấu hình.
- **Cổng khác**: ứng dụng không thể vô tình kết nối vào bản chưa nghiệm thu.

## 6. Đọc log recovery

Runner khởi động đích và đợi `pg_is_wal_replay_paused()` trả `true`. Log server (runner thay LSN bằng `<LSN>`, thời điểm bằng `<T>`):

```text
LOG:  restored log file "000000010000000000000003" from archive
LOG:  starting point-in-time recovery to "p05_before_delete"
LOG:  redo starts at <LSN>
LOG:  consistent recovery state reached at <LSN>
LOG:  restored log file "000000010000000000000004" from archive
LOG:  recovery stopping at restore point "p05_before_delete", time <T>
LOG:  pausing at the end of recovery
```

Từng dòng:

1. **`restored log file …03 from archive`**: segment chứa điểm bắt đầu backup được lấy **từ kho**, dù `-X stream` đã đặt một bản trong `pg_wal` của backup. Trong archive recovery, server ưu tiên `restore_command`; chỉ khi kho không có mới tìm trong `pg_wal`.
2. **`starting point-in-time recovery to "p05_before_delete"`**: server đã nhận đúng target. Thấy sai tên ở dòng này thì dừng lại ngay.
3. **`redo starts`**: bắt đầu phát lại từ checkpoint trong `backup_label`.
4. **`consistent recovery state reached`**: đã phát lại qua điểm kết thúc của base backup; từ đây dữ liệu nhất quán và, với `hot_standby = on` (mặc định), server nhận truy vấn chỉ đọc.
5. **`restored log file …04`**: segment chứa INSERT 101, restore point và DELETE.
6. **`recovery stopping at restore point`**: gặp bản ghi restore point thì dừng. INSERT 101 commit trước mốc được giữ; DELETE commit sau mốc không được áp.
7. **`pausing at the end of recovery`**: do `recovery_target_action = 'pause'`.

Tài liệu PostgreSQL (từ bản 13) nói nếu WAL hết trước khi gặp target, recovery kết thúc bằng lỗi thay vì mở database ở một trạng thái sớm hơn mà không báo lỗi. Bài tập 2 cho bạn tự kiểm điều này; lab chính không chạy ca thiếu WAL.

## 7. Đối soát khi còn đang pause

`recovery_target_action` có ba giá trị:

| Giá trị | Khi tới target | Dùng khi |
|---|---|---|
| `pause` (mặc định) | Dừng replay, cho truy vấn chỉ đọc, chờ quyết định | Cần kiểm tra trước khi chốt |
| `promote` | Kết thúc recovery, mở ghi ngay | Target đã được kiểm ở lần diễn tập trước |
| `shutdown` | Dừng server sau khi tới target | Tự động hoá: chuẩn bị đích rồi để người khác quyết định |

Theo tài liệu, nếu `hot_standby` tắt thì `pause` hoạt động như `shutdown`. Lab dùng `pause` vì đây là lúc rẻ nhất để phát hiện sai: chưa có timeline mới, sai target thì làm lại từ base backup.

Trong lúc pause, runner chạy cùng truy vấn digest của PG-046 và so với đáp án đã tính ở nguồn trước DELETE:

```text
PASS 048 recovered data including post-backup insert: 101/3045000/817b908d149b86ec12f316ce544dc676
```

**101 chuyến, 3.045.000 đ** = 100 chuyến × 30.000 đ trong base backup cộng chuyến 101 giá 45.000 đ phát sinh sau backup. Con số này chứng minh hai điều cùng lúc: replay đã đi **qua** base backup (nếu không, chỉ có 100 chuyến), và đã dừng **trước** DELETE (nếu không, 0 chuyến).

Kiểm thứ hai là quyền: chính sách RLS của PG-049 được tạo *trước* base backup nên phải còn nguyên sau phục hồi vật lý:

```text
PASS 048 RLS survives PITR: 51
```

`tn_app` với `app.tenant_id = '1'` thấy 51 chuyến — 50 chuyến ban đầu của tenant 1 cộng chuyến 101. Khác với PG-046, phục hồi vật lý chép cả cluster nên role, owner, ACL và policy đều đi kèm mà không cần globals riêng.

Kiểm thứ ba là đích chưa ghi được:

```text
PASS 048 paused target is still read-only: true
```

Runner thử `INSERT` và yêu cầu nó thất bại với SQLSTATE `25006` (giao dịch chỉ đọc); `OUTPUT.txt` không in thông báo lỗi.

## 8. Sau khi nghiệm thu: mở ghi hay chép dữ liệu về

Đến đây có hai đường, và đây là quyết định của người phụ trách sự cố chứ không phải của script.

**Mở ghi trên bản phục hồi.** Gọi `pg_wal_replay_resume()`: khi đang pause tại target, lệnh này kết thúc recovery. Server chọn timeline mới (`2`), ghi file `00000002.history` nói timeline 2 rẽ ra từ timeline 1 tại `0/4001DF0` — đúng LSN của restore point — và mở ghi. Runner làm bước này rồi chèn chuyến 102 để chứng minh ghi được; chi tiết timeline và vì sao nó ngăn hai lịch sử trộn vào nhau nằm ở <a href="/articles/postgresql-thuc-hanh-runbook-phuc-hoi">PG-050</a>. Đổi lại, mọi giao dịch hợp lệ trên nguồn **sau** 22:40 — chuyến khách đặt lúc 22:45 chẳng hạn — không có trên bản này và phải được bù.

**Giữ nguồn, chép lại phần bị xoá.** Để đích ở trạng thái pause, chỉ đọc, rồi chép các dòng bị xoá từ đích về nguồn (bằng `\copy` hay `postgres_fdw`) và đối soát lại trên nguồn. Giữ được giao dịch sau 22:40, nhưng phải xử lý xung đột khoá và các bảng khác đã tham chiếu tới chuyến bị xoá. Lab **không chạy** đường này.

Runbook phải ghi ai quyết định, dựa trên số liệu nào, và cách bù phần dữ liệu bị bỏ lại.

## 9. RPO và RTO: lab đo được gì, production phải đo gì

**RPO** (recovery point objective) là lượng dữ liệu tối đa được phép mất, tính bằng thời gian. **RTO** (recovery time objective) là thời gian tối đa từ lúc sự cố tới lúc dịch vụ hoạt động lại. Cả hai là **mục tiêu nghiệp vụ**; diễn tập phải cho ra số để so với mục tiêu.

Số duy nhất về thời gian mà lab ghi:

```text
MEASURE 048 start-to-verified-paused-recovery ms: 4284.1 (excludes base-copy and incident detection)
```

Con số đó là khoảng 4,3 giây, tính từ lúc gọi start cluster đích tới lúc đối soát xong ở trạng thái pause, đo trên cluster lab 100 chuyến, lấy hai segment WAL từ kho, chạy trên máy dev Windows, mỗi bước kiểm tra tốn một lần mở `psql`. README ghi rõ đây **không phải RTO đầu cuối**, và runner không assert số millisecond — lần chạy khác cho số khác.

| Thành phần | Lab | Production phải đo |
|---|---|---|
| Phát hiện sự cố | Không có — runner tự gây | Từ 22:40 tới lúc có người nhận cảnh báo |
| Quyết định dừng ghi, chọn target | Không có — target đã biết | Tìm xid/LSN/thời điểm của câu DELETE |
| Chuẩn bị máy đích | Cluster tạm trên cùng máy | Cấp máy, disk, mạng, quyền |
| Chép base backup | Không tính vào số đo | Kích thước thật / băng thông từ kho |
| Lấy WAL và replay | 2 segment | Toàn bộ WAL từ base backup tới target — có thể hàng trăm GB nếu backup cũ |
| Đối soát | Một bảng, 3 cổng | Toàn bộ bảng nghiệp vụ, đối soát với hệ ngoài |
| Chuyển traffic, bù dữ liệu | Không làm | DNS/pool, cache, job nền, giao dịch sau target |

Về RPO, lab chỉ chứng minh: **với target đã chọn và WAL đã nằm trong kho**, không mất dòng nào trước target. Lab không chứng minh RPO của sự cố mất máy. Ở production, RPO bị giới hạn bởi **độ trễ archive**: WAL còn trong `pg_wal` của máy nguồn mà chưa vào kho sẽ mất cùng máy. Vì archive chỉ chạy khi segment đầy, một hệ thống ghi ít có thể mất nhiều phút dữ liệu. Các cách thu hẹp, **không chạy trong lab**:

- `archive_timeout`: ép switch segment sau một khoảng thời gian, đổi lại nhiều file archive gần rỗng.
- `pg_receivewal`: stream WAL liên tục sang máy khác qua giao thức replication thay vì chờ segment đầy.
- Replication đồng bộ: giao dịch chỉ commit khi bản sao đã nhận WAL — đổi lại là độ trễ commit tăng.

Riêng với DELETE nhầm, replication không cứu được: standby nhận câu DELETE gần như ngay lập tức. Chỉ backup + WAL mới quay ngược được thời gian. Chỉ số cần cảnh báo: `pg_stat_archiver.failed_count` tăng, `last_archived_wal` đứng yên, tuổi của base backup gần nhất.

## 10. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Có base backup nhưng không về được trạng thái 22:39 | Không có archive WAL, hoặc archive đứt giữa chừng | Bật archive trước khi cần; cảnh báo `failed_count` và segment đứng yên |
| `pg_wal` phình to, disk đầy | `archive_command` lỗi liên tục, server giữ mọi segment chưa archive | Sửa nguyên nhân; không xoá tay file trong `pg_wal` |
| Server khởi động nhưng thiếu dữ liệu sau backup | Quên `recovery.signal`, hoặc `restore_command` sai nên chỉ có WAL trong `pg_wal` | Kiểm dòng «starting point-in-time recovery to …» và «restored log file … from archive» |
| Bản phục hồi có cả câu DELETE | Target quá muộn, hoặc không đặt target | Đặt target trước lỗi; dùng `pause` để kiểm trước khi kết thúc recovery |
| Xoá `backup_label` để «khởi động được» | Hiểu nhầm đó là file khoá | Không xoá; tìm nguyên nhân thật trong log |
| Gọi số giây diễn tập là RTO | Chỉ đo đoạn replay | Đo đủ chuỗi phát hiện → quyết định → chép → replay → đối soát → chuyển traffic |

## 11. Đánh đổi: backup logic hay vật lý

| Tiêu chí | `pg_dump` (PG-046) | Base backup + WAL |
|---|---|---|
| Thời điểm khôi phục | Lúc bắt đầu dump | Bất kỳ điểm nào có WAL liên tục |
| Phạm vi | Database / schema / bảng | Cả cluster, kể cả role và policy |
| Restore một phần | Dễ | Phải dựng cả cluster rồi lấy phần cần |
| Phiên bản / nền tảng đích | Có thể khác, mới hơn | Cùng phiên bản lớn, cùng kiến trúc |
| Chi phí lưu trữ | Một file mỗi lần dump | Base backup + toàn bộ WAL giữa các lần |
| Vận hành | Một lệnh | archive, retention, kiểm chuỗi WAL, diễn tập |

Ở quy mô thật, ít ai tự viết `archive_command` bằng script. Công cụ như pgBackRest hay Barman lo nén, mã hoá, đẩy offsite, retention, kiểm chuỗi WAL và restore song song; PostgreSQL 17 có thêm incremental backup (`pg_basebackup --incremental` cùng `pg_combinebackup`) để base backup không phải chép lại toàn bộ mỗi lần. Lab **không dùng và không đo** các công cụ này; nó chỉ cho thấy cơ chế bên dưới.

## 12. Bài tập

**Bài 1.** Sửa lab trên một bản chép riêng: bỏ `pg_create_restore_point`, dùng `recovery_target_time` để dừng trước DELETE.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Cách lấy thời điểm (ví dụ ghi <code>clock_timestamp()</code> sau khi INSERT 101 commit và trước DELETE), kèm múi giờ. Log recovery có dòng «starting point-in-time recovery to» với thời điểm đó và dòng «recovery stopping …» tương ứng. Digest ở trạng thái pause khớp <code>101/3045000/…</code>. Thêm một lần chạy đặt target sớm hơn INSERT 101 và cho thấy digest thành 100 chuyến — chứng minh bạn hiểu target nằm ở đâu so với commit. Đừng chép thông báo log từ bài này; dán log thật của bạn.</p>
</details>

**Bài 2.** Trên bản chép riêng, xoá segment chứa restore point khỏi kho trước khi khởi động đích.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Log cho thấy segment nào được yêu cầu mà không lấy được, và server có kết thúc ở trạng thái lỗi hay không (đối chiếu với tài liệu PostgreSQL 18 về việc recovery không đạt target). Phân biệt rõ «server đã lên» với «đã replay tới target». Cách khắc phục là tìm lại segment, không phải đổi target sớm hơn cho qua kiểm tra. Nếu buộc phải chọn target sớm hơn, biên bản phải ghi dữ liệu bị mất.</p>
</details>

**Bài 3.** Viết mục RPO/RTO trong runbook của Tới Nơi cho sự cố 22:40.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Một mục tiêu RPO và RTO bằng số, do nghiệp vụ đặt. Một bảng thời gian theo đúng các thành phần ở mục 9, mỗi dòng có số đo từ một lần diễn tập trên dữ liệu cỡ thật hoặc ghi «chưa đo». Cấu hình archive (<code>archive_timeout</code> hoặc stream WAL) giải thích vì sao RPO đạt mục tiêu khi mất máy nguồn. Không lấy 4,3 giây của lab làm RTO.</p>
</details>

## 13. Đọc tiếp

- [PostgreSQL 18 · Continuous Archiving and Point-in-Time Recovery](https://www.postgresql.org/docs/18/continuous-archiving.html): archive_command, base backup, recovery, timeline.
- [PostgreSQL 18 · Write Ahead Log settings](https://www.postgresql.org/docs/18/runtime-config-wal.html): `archive_mode`, `archive_timeout`, `restore_command`, `recovery_target_*`, `recovery_target_action`.
- [PostgreSQL 18 · pg_basebackup](https://www.postgresql.org/docs/18/app-pgbasebackup.html): `-X stream`, `--checkpoint`, `-R`, `--incremental`.
- [PostgreSQL 18 · Backup Control Functions](https://www.postgresql.org/docs/18/functions-admin.html): `pg_create_restore_point`, `pg_switch_wal`, `pg_wal_replay_resume`.
- Lab: `examples/toi-noi-postgresql/p05/run.mjs` (khối PG-048/050), `p05/wal-copy.mjs`, kết quả `p05/OUTPUT.txt`, giới hạn phép đo trong `p05/README.md`, các bước diễn tập trong `p05/RUNBOOK.md`.

Bài trước: [PG-047 · Connection pool, giới hạn kết nối và timeout](/articles/postgresql-connection-pool-timeout). Bài tiếp theo: [PG-049 · Row-level security và cách ly tenant](/articles/postgresql-rls-tenant-security).
