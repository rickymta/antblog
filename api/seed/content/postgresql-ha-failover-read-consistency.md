---
slug: postgresql-ha-failover-read-consistency
title: "Failover, split-brain và tính nhất quán khi đọc: chuyến đã commit vẫn có thể mất"
summary: "Primary của Tới Nơi chết khi standby đã bị cắt khỏi nó vài giây. Lab PostgreSQL 18.1 cho thấy trọn chuỗi: primary cũ vẫn nhận chuyến 5, crash, 184 byte WAL chưa tới standby, promote lên timeline 2, chuyến 5 mất; primary cũ khởi động lại không bị fence thành writer thứ hai và cùng một chuỗi kết nối target_session_attrs=read-write chọn nhầm nó; fencing và pg_rewind đưa nó về làm standby. Kèm RPO của replication bất đồng bộ, STONITH, xử lý giao dịch bị loại và cổng go/no-go trước khi mở lại ghi."
category: postgresql
level: advanced
tags: [postgresql, pg-058, failover, split-brain, fencing, pg-rewind, target-session-attrs, rpo, toi-noi]
language: vi
---

21:10 tối thứ Sáu, máy chủ chạy primary của Tới Nơi bắt đầu chập chờn: kết nối tới standby đứt từ vài giây trước, nhưng ứng dụng vẫn ghi vào primary bình thường. Một chuyến ở Hà Nội giá 20.000 đ vừa được thanh toán, API nhận `COMMIT`. Ngay sau đó primary sập hẳn. Người trực promote standby lên làm primary mới; app hoạt động lại sau vài phút.

Sáng thứ Bảy có hai báo cáo. Một tài xế hỏi vì sao chuyến 21:10 khách đã trả tiền không có trong sổ. Và đội đối soát thấy một chuyến 99.000 đ ghi lúc 21:40 tồn tại trong log ứng dụng nhưng không có trên primary mới: lúc đó máy cũ được khởi động lại sau khi thay nguồn, tự lên như một primary bình thường, và một worker đã ghi vào nó.

Đây là hai lỗi khác nhau: **mất dữ liệu đã commit** — cái giá có sẵn của replication bất đồng bộ — và **split-brain**: hai server cùng nhận ghi, hai lịch sử tách nhau. Bài này dựng lại cả hai trên PostgreSQL 18.1, đo phần đã mất, đưa máy cũ về làm standby bằng `pg_rewind`, và nói rõ phần nào production cần công cụ mà lab không có.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-058 · Phần 6/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/postgresql-replication-primary-standby">PG-052 · Primary, standby và replication lag</a>, <a href="/articles/postgresql-thuc-hanh-runbook-phuc-hoi">PG-050 · Runbook phục hồi</a> (timeline và tệp history) và <a href="/articles/postgresql-connection-pool-timeout">PG-047 · Connection pool và timeout</a>. Lab: <code>examples/toi-noi-postgresql/p06</code>, PostgreSQL 18.1 thật; trong <code>examples/toi-noi-postgresql</code> chạy <code>npm run p06</code> (cần PostgreSQL 18 và biến <code>PG17_BIN</code>, xem <code>p06/README.md</code>). Crash, promote và rewind chỉ chạm cluster tạm do runner tạo. Sau bài này, bạn lập được kịch bản mất primary, đo được dữ liệu đã mất, fence node cũ trước khi mở writer mới, kết nối lại đúng primary và đưa node cũ về đúng vai.</p>
</div>

## 1. Giới hạn của lab, nói trước

`p06/README.md` ghi rõ, và bài giữ đúng những giới hạn đó:

- Replication **bất đồng bộ**, một standby, không có synchronous standby.
- **Không có công cụ HA** (Patroni, repmgr, pg_auto_failover…), không có virtual IP hay DNS. Mọi quyết định do runner ra theo kịch bản cố định.
- «**Network partition**» được giả lập bằng cách xoá `primary_conninfo` trên standby — chỉ cắt đường WAL, không cắt ứng dụng khỏi primary.
- «**Crash**» là `pg_ctl stop -m immediate`: tiến trình dừng ngay, không checkpoint, như mất điện ở mức tiến trình.
- **Fencing** là dừng tay node cũ.

Sổ cước `fares` đi xuyên runner. Khi PG-052 kết thúc, sổ có 4 chuyến, tổng 235.000 đ, và standby đang theo kịp primary. Phần 058 thêm ba chuyến, mỗi chuyến đóng một vai:

| Chuyến | Ghi vào | Lúc | Số phận |
|---|---|---|---|
| 5 (HN, 20.000) | Primary cũ | Standby đã bị cắt | Mất khi failover |
| 6 (HCM, 15.000) | Primary mới | Sau promote | Ở lại |
| 7 (HN, 99.000) | Primary cũ | Khởi động lại không fence | Bị `pg_rewind` loại |

Sổ đúng cuối cùng theo README là 5 chuyến / 250.000 đ.

## 2. Cắt standby: commit bất đồng bộ không chờ ai

Runner cắt đường WAL bằng cách xoá chuỗi kết nối của standby:

```sql
-- Trên standby
ALTER SYSTEM SET primary_conninfo = '';
SELECT pg_reload_conf();
-- chờ tới khi pg_stat_wal_receiver không còn dòng nào
```

Theo [tài liệu cấu hình replication](https://www.postgresql.org/docs/18/runtime-config-replication.html), đổi `primary_conninfo` khi walreceiver đang chạy sẽ khiến nó bị yêu cầu dừng và khởi động lại với giá trị mới — trừ khi giá trị mới là chuỗi rỗng. Chuỗi rỗng nghĩa là walreceiver dừng hẳn và standby không gọi primary nữa. Standby vẫn ở recovery, vẫn trả lời truy vấn đọc, chỉ không nhận thêm WAL.

```text
PASS 058 standby cut off from primary (network partition simulated)
PASS 058 old primary accepted write while standby cut off: 1
```

Primary nhận chuyến 5 và trả `COMMIT` như không có chuyện gì. Đây không phải lỗi; đó là định nghĩa của replication bất đồng bộ. Với `synchronous_standby_names` rỗng, `synchronous_commit = on` chỉ có nghĩa «WAL đã flush xuống đĩa **của primary**» (PG-045). Primary không biết, và không cần biết, standby đã nhận hay chưa.

Tài liệu [warm-standby](https://www.postgresql.org/docs/18/warm-standby.html) nói thẳng: replication mặc định bất đồng bộ, nếu primary crash thì một số giao dịch đã commit có thể chưa được sao chép sang standby, gây mất dữ liệu, và lượng mất tỉ lệ với độ trễ replication lúc failover. Trong lab, độ trễ đó bị kéo dài có chủ đích: standby đã bị cắt, nên mọi thứ ghi sau đó đều nằm ngoài nó.

## 3. Crash và đo phần WAL chưa tới standby

Trước khi crash, runner đọc `pg_current_wal_flush_lsn()` trên primary — vị trí WAL cuối cùng đã bền trên đĩa của nó. Rồi `pg_ctl stop -m immediate`. Sau đó đọc `pg_last_wal_receive_lsn()` trên standby — vị trí cuối cùng standby đã nhận và flush:

```text
MEASURE 058 WAL not received by standby at failover: 184 bytes (primary flush 0/4000370, standby received 0/40002B8)
```

`pg_wal_lsn_diff('0/4000370', '0/40002B8')` = **184 byte**. Trong 184 byte đó có bản ghi INSERT và bản ghi commit của chuyến 5. Con số nhỏ vì lab chỉ ghi một dòng; ở production nó là mọi thứ ghi trong suốt khoảng standby tụt lại — có thể là hàng nghìn giao dịch.

Đây là định nghĩa thực hành của **RPO** (recovery point objective — lượng dữ liệu tối đa được phép mất) cho replication bất đồng bộ: RPO thực tế của một lần failover là **phần WAL primary đã flush mà standby được promote chưa nhận**. Không có cấu hình nào của chế độ bất đồng bộ làm con số này bằng 0; chỉ thu nhỏ được nó bằng cách giữ standby bắt kịp và giám sát độ trễ (PG-052).

Lưu ý cách đo: runner đọc flush LSN của primary *trước* khi giết nó; sự cố thật không lịch sự như vậy. Nếu đĩa primary cũ còn đọc được, khởi động nó **cô lập** (mục 8) để đọc; nếu mất hẳn, phần đã mất phải dựng lại từ log ứng dụng, cổng thanh toán. Cổng go/no-go số 2 trong `HO-SO-BAN-GIAO.md` đòi ghi đúng hai con số này.

## 4. Promote: standby thành primary trên timeline 2

```sql
SELECT pg_promote(true, 30);
```

```text
PASS 058 pg_promote: t
PASS 058 committed row lost by async failover: 0
PASS 058 new primary timeline: 00000002
```

Theo [tài liệu chức năng quản trị](https://www.postgresql.org/docs/18/functions-admin.html), `pg_promote(wait, wait_seconds)` đưa standby lên primary; với `wait = true`, hàm chờ tới khi promote xong hoặc hết `wait_seconds` giây, trả `true` nếu thành công. Cách tương đương ngoài SQL là `pg_ctl promote`.

Ba dòng output nói ba điều:

1. **Promote thành công** — standby kết thúc recovery và nhận ghi.
2. **Chuyến 5 không có trên primary mới**: một giao dịch đã trả `COMMIT` cho ứng dụng đã biến mất khỏi hệ thống đang phục vụ. Không có lỗi, không có cảnh báo nào trong SQL; chỉ đối soát mới phát hiện.
3. **Timeline mới**: runner lấy tám ký tự đầu của `pg_walfile_name(pg_current_wal_lsn())`, ra `00000002`. Như PG-050 đã giải thích, kết thúc recovery mở timeline mới để WAL của nhánh mới không trùng tên với lịch sử cũ. Ở đây điều đó rất cụ thể: timeline 1 trên đĩa primary cũ có chuyến 5, timeline 2 thì không; từ `0/40002B8`, hai lịch sử đã khác nhau.

## 5. Ứng dụng tìm primary mới bằng target_session_attrs

Lab không có VIP hay DNS. Thay vào đó, runner dùng chuỗi kết nối nhiều host của libpq:

```text
host=127.0.0.1,127.0.0.1 port=<OLD>,<NEW> dbname=postgres user=postgres
target_session_attrs=read-write connect_timeout=3
```

Primary cũ đứng **đầu** danh sách. Lúc này nó đang chết:

```text
PASS 058 target_session_attrs=read-write picks: standby (new primary)
```

Theo [tài liệu libpq](https://www.postgresql.org/docs/18/libpq-connect.html), các host được thử **theo thứ tự**, và `target_session_attrs` quyết định một kết nối thành công có được chấp nhận không. Với `read-write`, phiên phải nhận giao dịch đọc-ghi theo mặc định: server không ở hot standby và `default_transaction_read_only` là `off`. Host đầu từ chối kết nối, libpq thử host thứ hai — standby vừa promote, giờ không còn ở recovery — và chấp nhận nó.

| Giá trị | Chấp nhận server khi |
|---|---|
| `any` (mặc định) | Kết nối được là đủ |
| `read-write` | Không ở hot standby và `default_transaction_read_only = off` |
| `read-only` | Ngược lại với `read-write` |
| `primary` | Không ở hot standby |
| `standby` | Đang ở hot standby |
| `prefer-standby` | Ưu tiên standby, không có thì như `any` |

Cơ chế này hữu ích nhưng hẹp. Nó chỉ chạy **lúc mở kết nối**: pool đang giữ kết nối tới primary cũ sẽ chỉ nhận ra khi kết nối đó lỗi (PG-047). Nó cũng phụ thuộc `connect_timeout` khi host đầu không từ chối mà im lặng — chẳng hạn khi máy treo hoặc mạng rơi gói. Và như mục sau cho thấy, nó **không hề biết** server nào là primary «hợp pháp».

## 6. Split-brain: primary cũ quay lại mà không bị fence

Runner ghi chuyến 6 lên primary mới. Rồi khởi động lại primary cũ — đúng như chuyện máy được thay nguồn và tự bật lên:

```text
PASS 058 old primary restarted, not in recovery: f
--- 058 split-brain: two writable primaries
  old primary (timeline 00000001): 1,2,3,4,5,7
  new primary (timeline 00000002): 1,2,3,4,6
PASS 058 same connection string now picks: old primary
```

(Nhãn dòng đầu hơi khó đọc: giá trị `f` là kết quả của `pg_is_in_recovery()`, tức primary cũ **không** ở recovery — nó là một primary bình thường.)

Chuyện gì đã xảy ra:

- Primary cũ khởi động, chạy **crash recovery** từ WAL trên đĩa của chính nó. Chuyến 5 đã flush ở đó, nên nó quay lại. Server không có cách nào biết ở đâu đó đã có một primary mới; nó không có `standby.signal`, nên lên làm primary trên timeline 1.
- Một worker ghi chuyến 7 vào nó, và nó nhận.
- Giờ có **hai writer**: timeline 1 có `1,2,3,4,5,7`, timeline 2 có `1,2,3,4,6`. Đây là **split-brain** — mỗi bên tin mình là primary, mỗi bên tích luỹ những giao dịch bên kia không có.
- Tệ nhất: **cùng chuỗi kết nối** vừa chọn đúng primary mới ở mục 5, giờ chọn **primary cũ**. Primary cũ đứng đầu danh sách, nhận kết nối, không ở hot standby, `default_transaction_read_only = off` — đạt mọi điều kiện của `read-write`. libpq không so timeline, không hỏi ai được promote sau.

Đây là lý do cổng go/no-go số 1 trong hồ sơ bàn giao viết: lab 058 cho thấy node cũ khởi động lại vẫn nhận ghi, và `target_session_attrs=read-write` sẽ chọn nó nếu đứng đầu danh sách host. Đổi thứ tự host không sửa được gì — lần failover sau, vai trò lại đảo.

## 7. Fencing phải đi trước promote

**Fencing** là bảo đảm node cũ **không thể** nhận ghi nữa, trước khi node mới được phép nhận ghi. Tài liệu [Failover](https://www.postgresql.org/docs/18/warm-standby-failover.html) của PostgreSQL nói: nếu primary cũ khởi động lại sau khi standby đã thành primary mới, phải có cơ chế báo cho nó biết nó không còn là primary — thường gọi là **STONITH** (Shoot The Other Node In The Head) — để tránh tình huống cả hai hệ thống cùng nghĩ mình là primary, dẫn tới nhầm lẫn và cuối cùng là mất dữ liệu. Cùng trang đó nói PostgreSQL **không cung cấp** phần mềm hệ thống để phát hiện lỗi trên primary và báo cho standby; đó là việc của công cụ bên ngoài.

Thứ tự đúng:

```text
1. Phát hiện primary có vấn đề
2. FENCE primary cũ   ── chắc chắn nó không thể nhận ghi, kể cả khi tự bật lại
3. Ghi nhận LSN (mục 3), quyết định chấp nhận mất phần chưa tới standby
4. Promote standby
5. Chuyển đường ghi của ứng dụng (DNS/VIP/proxy/chuỗi kết nối)
6. Đối soát, rồi mới mở lại ghi (mục 10)
```

Đảo bước 2 và 4 là mở cửa cho đúng tình huống của mục 6. Các mức fencing, từ mạnh tới yếu — **không chạy trong lab**:

- **Tắt nguồn qua kênh quản trị độc lập**: IPMI/iLO, API của nhà cung cấp cloud để stop/terminate máy ảo. Đây là STONITH theo nghĩa đen, và nó phải thành công *được xác nhận* trước khi promote.
- **Cắt mạng**: security group, firewall, gỡ khỏi switch — chặn mọi client, không chỉ ứng dụng chính.
- **Gỡ khỏi định tuyến**: chuyển VIP, đổi DNS, rút khỏi proxy. Chỉ đủ khi *mọi* client đi qua đường đó; worker cấu hình cứng IP sẽ vẫn tới được node cũ.
- **Chặn tự khởi động**: tắt dịch vụ trong systemd hay tương đương, để máy bật lại không tự lên làm primary — đúng lỗi của câu chuyện đầu bài.

Dừng tiến trình PostgreSQL, như lab làm, là mức yếu nhất: máy reboot, dịch vụ tự bật, và bạn quay lại mục 6. Công cụ HA như Patroni hay repmgr lo phát hiện, bầu chọn, fencing và chuyển định tuyến; lab **không dùng và không kiểm chứng** chúng. Khi đánh giá, hãy hỏi rõ cơ chế fencing của chúng, và chúng làm gì khi không liên lạc được với node cũ.

## 8. Giao dịch bị bỏ lại: lấy ra trước, rồi mới rewind

Lúc này primary cũ chứa hai chuyến không có trên primary mới: chuyến 5 (đã commit trước crash, khách đã trả tiền) và chuyến 7 (ghi vào trong lúc split-brain). Bước tiếp theo — `pg_rewind` — sẽ xoá chúng vĩnh viễn khỏi node đó. **Trước khi rewind là lần cuối cùng bạn đọc được chúng từ database.**

Lab chỉ in mã chuyến rồi rewind; nó **không trích xuất** dữ liệu. Ở production, quy trình nên là:

1. Giữ node cũ trong trạng thái đã fence: không client nào tới được.
2. Khởi động nó cô lập — cổng khác, chỉ quản trị viên kết nối được — hoặc chép nguyên data directory sang chỗ khác để khởi động bản sao.
3. Trích các dòng chỉ có trên timeline cũ, theo **khoá nghiệp vụ**: mã chuyến, khoá idempotency, mã giao dịch thanh toán. So với primary mới.
4. Đối chiếu với nguồn độc lập: log ứng dụng, cổng thanh toán, outbox (PG-039). Chuyến 5 có tiền của khách — phải được ghi lại hoặc hoàn tiền, không được biến mất lặng lẽ.
5. Nhập bù qua **đúng đường ứng dụng** với cùng khoá idempotency, để không sinh trùng nếu một phần đã được ghi lại. Không chép thẳng file hay WAL giữa hai timeline.

Mã chuyến tự tăng là chỗ dễ va chạm: chuyến 6 trên timeline 2 và chuyến 7 trên timeline 1 có thể đã lấy cùng một giá trị sequence ở hệ thống thật. Lab đặt mã tay nên không gặp; khoá nghiệp vụ độc lập với sequence là thứ giúp đối soát được.

## 9. pg_rewind: đưa node cũ về làm standby của timeline 2

Có hai cách đưa node cũ trở lại: dựng lại từ đầu bằng `pg_basebackup` (chép cả cluster), hoặc `pg_rewind` — chỉ chép những block đã khác. Tài liệu Failover nhắc `pg_rewind` để tăng tốc việc này trên cluster lớn. Runner dùng nó:

```text
# 1. Fence: dừng node cũ (lab: pg_ctl stop -m fast)
# 2. Rewind theo primary mới đang chạy
pg_rewind --target-pgdata <old-data> \
          --source-server 'host=127.0.0.1 port=<NEW> dbname=postgres user=postgres' \
          --write-recovery-conf --progress
```

Output, runner chỉ giữ các dòng chính:

```text
--- 058 pg_rewind
  pg_rewind: servers diverged at WAL location 0/40002B8 on timeline 1
  pg_rewind: rewinding from last common checkpoint at 0/3000080 on timeline 1
  pg_rewind: Done!
```

- **`diverged at 0/40002B8 on timeline 1`**: điểm hai lịch sử tách nhau — trùng khớp với vị trí standby đã nhận ở mục 3. Mọi thứ sau đó trên node cũ (chuyến 5, chuyến 7) là nhánh chết.
- **`last common checkpoint at 0/3000080`**: checkpoint cuối cùng hai bên còn chung, trước điểm tách. Theo [tài liệu pg_rewind](https://www.postgresql.org/docs/18/app-pgrewind.html), công cụ quét WAL của node đích từ checkpoint này, ghi lại mọi block bị chạm sau điểm tách, chép các block đó từ nguồn sang, chép nguyên các file khác, rồi tạo `backup_label` để khi khởi động, node phát lại WAL của nguồn từ đó.
- **`Done!`**: rewind xong. Node chưa nhất quán cho tới khi được khởi động và replay xong.

Sau khi khởi động lại node cũ, runner chờ nó thấy chuyến 6 — chỉ có trên timeline 2:

```text
PASS 058 rewound old primary is standby: t
PASS 058 rows 5 and 7 discarded by rewind: 0
PASS 058 ledger on new primary: 5|250000
```

Node cũ giờ ở recovery, theo primary mới, và không còn chuyến 5 hay 7. Sổ trên primary mới: **5 chuyến / 250.000 đ** = 85.000 + 65.000 + 55.000 + 30.000 + 15.000, khớp README.

`pg_rewind` có những điều kiện mà lab phải đáp ứng — và production phải kiểm trước khi cần:

| Điều kiện | Theo tài liệu | Lab |
|---|---|---|
| `wal_log_hints = on` **hoặc** data checksums | Bắt buộc trên node đích | PostgreSQL 18 bật data checksums mặc định khi `initdb` (lab 059 cũng thấy điều này qua `pg_upgrade --check`); lab không bật `wal_log_hints` |
| `full_page_writes = on` | Bắt buộc, mặc định bật | Mặc định |
| Node đích đã shutdown sạch | Nếu không, mặc định `pg_rewind` tự chạy single-user mode để hoàn tất crash recovery rồi dừng (tắt bằng `--no-ensure-shutdown`) | Node đã được khởi động lại sau crash rồi dừng `fast` — đã sạch |
| WAL trên node đích còn tới điểm tách | Thiếu thì chép từ archive vào `pg_wal` hoặc dùng `-c`/`--restore-target-wal` | Runner đặt `wal_keep_size = 512MB` trên primary để giữ đủ WAL; lab không thử trường hợp thiếu |

Hai cảnh báo từ tài liệu mà lab gặp trực tiếp:

- **`pg_rewind` chép nguyên file cấu hình từ nguồn.** Tài liệu cảnh báo có thể phải sửa cấu hình trước khi khởi động lại đích. Trong lab, `postgresql.conf` của primary mới chứa dòng `port` riêng của nó (runner thêm vào ở PG-052), nên sau rewind node cũ sẽ mở nhầm cổng; runner phải ghi lại `port` của node cũ. Ở production, mọi thiết lập khác nhau giữa hai máy — đường dẫn, `archive_command`, bộ nhớ — đều cần soát lại.
- **`--write-recovery-conf`** tạo `standby.signal` và thêm thiết lập kết nối vào `postgresql.auto.conf`. Tài liệu cũng cảnh báo: khởi động lại sau rewind mà **không** cấu hình recovery, node có thể lại phân nhánh khỏi primary. Thiếu `standby.signal` là quay về đúng split-brain.

Và nếu `pg_rewind` thất bại giữa chừng, tài liệu nói data directory đích nhiều khả năng không phục hồi được; nên dựng lại bằng base backup mới.

## 10. Đọc nhất quán sau failover

Tiêu đề bài có «tính nhất quán khi đọc», và failover phá vỡ vài giả định của PG-052 theo cách khó thấy:

- **LSN token qua hai timeline.** Nếu API đã phát cho khách token LSN của chuyến 5 (khoảng `0/4000370` trên timeline 1), sau failover primary mới sẽ sớm vượt qua con số đó trên timeline 2. Phép so `pg_last_wal_replay_lsn() >= token` trên một replica của timeline 2 trả `true`, nhưng chuyến 5 **không bao giờ** có ở đó. Token phải mang kèm timeline hoặc một «thế hệ» primary, và mọi token cũ phải bị vô hiệu khi failover. Đây là suy luận từ cơ chế; lab không chạy kịch bản token.
- **Replica còn theo primary cũ.** Trong khoảng split-brain, một replica nối vào node cũ trả về dữ liệu của timeline 1 — có chuyến 7, thiếu chuyến 6. Fencing phải bao gồm cả các replica của node cũ.
- **Node vừa rewind.** Trong lúc replay sau rewind, nó có thể trả lời truy vấn đọc trước khi bắt kịp. Chỉ đưa nó vào pool đọc sau khi `replay_lsn` của nó đã gần primary mới.
- **Projection lệch.** Lab 060 cho thấy báo cáo materialized view trên primary mới vẫn là 200.000 so với sổ 250.000 tới khi REFRESH — dữ liệu dẫn xuất không tự đúng sau failover.

## 11. Cổng go/no-go trước khi mở lại ghi

`HO-SO-BAN-GIAO.md` của P06 đặt sáu cổng. Lab 058 cung cấp bằng chứng cho từng cổng:

| # | Cổng | Bằng chứng trong lab |
|---|---|---|
| 1 | **Một writer duy nhất**: node cũ đã bị fence trước khi promote | Mục 6: không fence thì cùng chuỗi kết nối chọn node cũ |
| 2 | **Biết đã mất gì**: flush LSN cuối của node cũ và receive LSN của node được promote | 184 byte, gồm chuyến 5 đã commit |
| 3 | **Sổ gốc đúng**: đối soát số chuyến, tổng tiền, digest với nguồn độc lập | 5 chuyến / 250.000 đ |
| 4 | **Projection đúng hoặc được đánh dấu cũ** | Báo cáo 200.000 so với 250.000 tới khi REFRESH (060) |
| 5 | **Đường ghi thật**: thử ghi qua đúng connection string, pool, quyền mà ứng dụng dùng | Chuyến 6 ghi qua psql trực tiếp — lab chỉ chứng minh một phần |
| 6 | **Node cũ có số phận rõ ràng**: rewind thành standby hoặc dựng lại | `pg_rewind`, node cũ ở recovery, theo timeline 2 |

Hồ sơ cũng liệt kê những gì lab **chưa chứng minh**, và không được cam kết cho tới khi đo: RTO đầu cuối (phát hiện, quyết định, fencing, chuyển DNS/VIP, pool kết nối lại); synchronous replication và RPO = 0; mất đồng thời hai node; replication slot trên standby được promote.

## 12. Lỗi thường gặp

| Triệu chứng | Nguyên nhân | Cách sửa |
|---|---|---|
| Khách có biên lai nhưng chuyến không có trong sổ sau failover | Replication bất đồng bộ; commit chưa tới standby | Đo phần WAL chưa tới (mục 3); đối soát với cổng thanh toán; quyết định RPO có chủ đích |
| Hai node cùng nhận ghi | Promote trước khi fence; máy cũ tự bật lại | Fence có xác nhận trước promote; tắt tự khởi động |
| Ứng dụng ghi vào node cũ dù đã failover | Chuỗi nhiều host chọn host đầu đạt `read-write` | Fencing; một nguồn định tuyến có thẩm quyền; không coi `target_session_attrs` là cơ chế HA |
| `pg_rewind` báo cần `wal_log_hints` hoặc checksums | Cluster tạo không có checksums và không bật `wal_log_hints` | Bật từ trước (đổi cần restart); không thì dựng lại bằng base backup |
| `pg_rewind` thiếu WAL | Node đích đã xoá segment cũ hơn điểm tách | `wal_keep_size`/archive; dùng `--restore-target-wal` |
| Node sau rewind không lên hoặc mở nhầm cổng | Cấu hình bị chép từ nguồn | Soát `postgresql.conf` và `postgresql.auto.conf` trước khi khởi động |
| Mất luôn bằng chứng về chuyến bị loại | Rewind ngay, không trích dữ liệu trước | Trích theo khoá nghiệp vụ trên bản cô lập (mục 8) |

## 13. Đánh đổi

**Bất đồng bộ hay đồng bộ.** Lab chạy bất đồng bộ: commit nhanh, standby chết không ảnh hưởng ghi, RPO > 0 như vừa thấy. Cấu hình đồng bộ — **không chạy trong lab**, không có output:

```text
# primary — KHÔNG CHẠY TRONG LAB
synchronous_standby_names = 'ANY 1 (standby_a, standby_b)'
synchronous_commit = on          # hoặc remote_write / remote_apply
```

Theo tài liệu warm-standby, `on` chờ standby ghi bản ghi commit xuống lưu trữ bền; `remote_write` chỉ chờ standby nhận và giao cho hệ điều hành; `remote_apply` chờ standby replay xong. Cái giá: mỗi commit thêm một khứ hồi mạng, và nếu không đủ standby đồng bộ trả lời thì commit đứng chờ — vì thế hay dùng hai standby với `ANY 1`. Đồng bộ cũng không thay được fencing: nó giải quyết «mất gì», không giải quyết «hai writer».

**Failover tay hay tự động.** Tay: chậm, nhưng người quyết định thấy ngữ cảnh — chấp nhận mất 184 byte, hay chờ thêm để cứu primary. Tự động: RTO ngắn, nhưng một công cụ fencing sai gây split-brain nhanh hơn bất kỳ con người nào.

**`pg_rewind` hay dựng lại.** Rewind nhanh trên cluster lớn nhưng có điều kiện và chép đè cấu hình. Dựng lại bằng `pg_basebackup` chậm hơn nhưng không phụ thuộc lịch sử node cũ; khi rewind thất bại hoặc nghi ngờ đĩa node cũ, dựng lại.

## 14. Bài tập

**Bài 1.** Trên bản chép riêng của lab, thêm bước trích các dòng chỉ có trên timeline 1 trước khi rewind.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Node cũ được khởi động ở chế độ cô lập (cổng khác, không nằm trong chuỗi kết nối của ứng dụng) và bằng chứng rằng chuỗi <code>target_session_attrs=read-write</code> của ứng dụng không chọn được nó. Danh sách chuyến 5 và 7 lấy bằng truy vấn thật, so với primary mới theo khoá, kèm output. Sau đó rewind vẫn thành công và sổ vẫn 5 / 250.000. Giải thích bạn sẽ nhập bù chuyến nào, qua đường nào, với khoá idempotency nào — và vì sao không nhập bù chuyến 7 mà không điều tra.</p>
</details>

**Bài 2.** Thiết kế cơ chế fencing cho Tới Nơi trên hạ tầng bạn đang dùng.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Thứ tự bước như mục 7, mỗi bước có lệnh hoặc API cụ thể và cách xác nhận đã thành công. Trả lời rõ: nếu không liên lạc được với node cũ thì làm gì (không được «coi như đã chết»); làm sao máy cũ không tự lên làm primary khi bật lại; worker cấu hình cứng IP có bị chặn không. Có một lần diễn tập trên môi trường riêng với log thật, không chép output của bài này.</p>
</details>

**Bài 3.** Thêm timeline vào LSN token của PG-052 và chứng minh token cũ bị từ chối sau failover.

<details><summary><b>Lời giải phải chứng minh được</b></summary>
<p>Token chứa timeline (ví dụ lấy từ tám ký tự đầu của <code>pg_walfile_name</code> như lab, hoặc một số thế hệ do ứng dụng quản lý). Một ca thử: token cấp từ primary cũ, sau failover và sau khi primary mới đã vượt LSN đó về con số — chứng minh phép so chỉ theo LSN sẽ trả sai, còn phép so có timeline buộc đọc primary và cho thấy chuyến không tồn tại. Kết quả chạy thật cho cả hai phép so.</p>
</details>

## 15. Đọc tiếp

- [PostgreSQL 18 · Failover](https://www.postgresql.org/docs/18/warm-standby-failover.html): STONITH, witness server, `pg_promote`, vai trò của `pg_rewind`.
- [PostgreSQL 18 · High Availability, Load Balancing, and Replication](https://www.postgresql.org/docs/18/high-availability.html) và [Log-Shipping Standby Servers](https://www.postgresql.org/docs/18/warm-standby.html): bất đồng bộ, đồng bộ, mất dữ liệu khi failover.
- [PostgreSQL 18 · pg_rewind](https://www.postgresql.org/docs/18/app-pgrewind.html): điều kiện, cách hoạt động, cảnh báo về cấu hình và WAL.
- [PostgreSQL 18 · Hot Standby](https://www.postgresql.org/docs/18/hot-standby.html) và [System Administration Functions](https://www.postgresql.org/docs/18/functions-admin.html): `pg_promote`, `pg_last_wal_receive_lsn`, `pg_current_wal_flush_lsn`.
- [PostgreSQL 18 · libpq connection parameters](https://www.postgresql.org/docs/18/libpq-connect.html): nhiều host, `target_session_attrs`, `load_balance_hosts`.
- Lab: `examples/toi-noi-postgresql/p06/run.mjs` (khối PG-058), `p06/OUTPUT.txt`, giới hạn trong `p06/README.md`, cổng go/no-go trong `p06/HO-SO-BAN-GIAO.md`.

Bài trước: [PG-057 · Đánh giá PostGIS và pgvector theo workload](/articles/postgresql-extension-postgis-pgvector). Bài tiếp theo: [PG-059 · Nâng phiên bản và cutover có phương án quay lại](/articles/postgresql-upgrade-migration-cutover).
