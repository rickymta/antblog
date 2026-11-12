---
slug: postgresql-thuc-hanh-runbook-phuc-hoi
title: "Thực hành: bàn giao runbook vận hành và phục hồi PostgreSQL"
summary: "Bài thực hành cuối Phần 5: biến toàn bộ lab P05 thành hồ sơ bàn giao — báo cáo nghiệm thu, cổng phục hồi 101 chuyến / 3.045.000 đ / digest / tenant, kết thúc recovery sang timeline 2, bằng chứng cần giữ, bảng xử lý lỗi, vai trò và RPO/RTO phân rã trung thực."
category: postgresql
level: advanced
tags: [postgresql, pg-050, thuc-hanh, runbook, recovery, pitr, timeline, rpo-rto, toi-noi]
language: vi
---

Thứ Bảy, 21:40. Một kỹ sư chạy script dọn dữ liệu thử trên nhầm cửa sổ terminal; `DELETE FROM ops.trips` chạy trên database thật, không có `WHERE`. Người trực tuần này mới vào nhóm ba tuần. Thứ họ có trong tay là một thư mục cấu hình, một cron backup «chạy xanh mỗi đêm» và số điện thoại của trưởng nhóm đang ở trên máy bay.

Họ cần biết: backup nào dùng được, WAL có đủ không, phục hồi tới đâu, ai được chấp nhận mất dữ liệu, kiểm gì trước khi cho API ghi lại, làm gì khi một bước lệch. Danh sách cấu hình không trả lời được câu nào. **Runbook** — tài liệu thao tác từng bước kèm điều kiện dừng — trả lời được, nếu nó đã được chạy thật và có bằng chứng.

Đây là bài thực hành cuối Phần 5: đóng gói mọi phép thử PG-041 … PG-049 của cùng một lab thành **hồ sơ bàn giao**.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-050 · Phần 5/6 · Nâng cao</span>
<p>Đọc trước: toàn bộ PG-041 … PG-049 (danh sách ở mục 10), đặc biệt <a href="/articles/postgresql-pitr-base-backup-wal">PG-048 · PITR với base backup và WAL</a> và <a href="/articles/postgresql-rls-tenant-security">PG-049 · Row-level security</a>. Lab: <code>examples/toi-noi-postgresql/p05</code> — <code>run.mjs</code> (đúng những gì đã chạy), <code>OUTPUT.txt</code> (kết quả thực trên PostgreSQL 18.1), <code>README.md</code> (phạm vi và giới hạn đo), <code>RUNBOOK.md</code> (bản bàn giao). Sau bài này, bạn bàn giao được bằng chứng restore, cổng nghiệm thu và quy trình xử lý lỗi mà người khác chạy lại được.</p>
</div>

## 1. Đề bài

### Bối cảnh

Lab P05 dựng **hai cluster PostgreSQL 18.1 tạm**, chỉ nghe localhost: *source* đóng vai production, *đích* là nơi phục hồi. `ops.trips` có 100 chuyến, hai tenant mỗi bên 50, mỗi chuyến 30.000 đ, tổng 3.000.000 đ. Thứ tự: 041 → 047, rồi **049 trước 048** để base backup chứa policy RLS, cuối cùng 050.

```bash
cd examples/toi-noi-postgresql
npm run p05
```

Yêu cầu và biến `PG_BIN`/`OPENSSL_BIN` ở `p05/README.md`. Chỉ chạy trên máy cá nhân: control plane của lab dùng `trust` trên loopback.

### Sản phẩm bàn giao

| Mã | Sản phẩm | Yêu cầu tối thiểu |
|---|---|---|
| D1 | Báo cáo nghiệm thu | Mỗi khẳng định của PG-041 … 049 gắn với một dòng `PASS` hoặc khối bằng chứng thật |
| D2 | Cổng phục hồi | Điều kiện **trước** khi resume và **trước** khi mở ghi, kiểm bằng máy |
| D3 | Timeline | Giải thích dòng `selected new timeline ID: 2` và tệp `00000002.history` |
| D4 | Bằng chứng cần giữ | Danh sách, nơi lưu, thứ không được lưu |
| D5 | Xử lý lỗi và vai trò | Mỗi cổng thất bại: kiểm gì, làm gì, ai quyết |
| D6 | RPO/RTO | Phân rã thành từng mốc; tách số **đã đo** khỏi số **chưa đo** |

Nguyên tắc chung: **không viết vào hồ sơ điều gì không có trong `OUTPUT.txt`**. Bước nào chưa chạy thì ghi «chưa chạy», không ghi «dự kiến đạt».

## 2. Đọc OUTPUT.txt như một báo cáo nghiệm thu

`OUTPUT.txt` có hai loại dòng (chú thích đầu `run.mjs`):

- `PASS …` / `MEASURE …` — **cổng** và số quan sát. Mỗi `PASS` là một `assert` đã đúng; `MEASURE` là số đo, **không** bị assert theo millisecond.
- `--- 0xx nhãn` — **khối bằng chứng**: output psql hoặc trích log server thật. Port, PID, thời điểm, mật khẩu bị che; LSN và tên tệp WAL giữ nguyên.

Assertion thất bại làm runner thoát khác 0 và **không ghi đè** `OUTPUT.txt`; dòng cuối chỉ có khi mọi thứ đạt:

```text
P05 operations lab · PostgreSQL 18.1
Node v22.18.0 · win32/x64
Fixture: 100 trips, 2 tenants, total fare 3000000 VND.
…
PASS P05: all assertions passed; both temporary clusters cleaned up.
```

Ba dòng đầu — phiên bản, nền tảng, fixture — là phần bắt buộc của mọi báo cáo. Lab chưa chạy trên Linux/macOS.

## 3. D1 — Báo cáo nghiệm thu theo bài

<details><summary><b>Lời giải</b></summary>
<table>
<thead><tr><th>Bài</th><th>Khẳng định</th><th>Bằng chứng trong OUTPUT.txt</th></tr></thead>
<tbody>
<tr><td>041</td><td>Báo cáo đọc được, không xoá được; app không sửa schema; app không phải owner; default privileges chỉ cấp cho report</td><td><code>PASS 041 report reads: 100</code>; <code>42501: permission denied for table trips</code>; <code>42501: must be owner of table trips</code>; <code>application is not owner: tn_owner</code>; <code>future table default privileges: t</code> / <code>not granted to app: f</code></td></tr>
<tr><td>042</td><td>Chỉ TLS + SCRAM; xác thực đúng máy chủ</td><td><code>stored verifier is SCRAM: SCRAM-SHA-256</code>; <code>TLS verify-full: t</code>, <code>TLSv1.3</code>; từ chối plaintext, sai mật khẩu, sai hostname; <code>verify-ca accepts wrong hostname: t</code></td></tr>
<tr><td>043</td><td>Tìm đúng phiên chặn; timeout cắt chờ</td><td><code>identifies lock wait: Lock</code>, B <code>blocked_by {&lt;PID A&gt;}</code>; <code>lock_timeout SQLSTATE: 55P03</code>; idle-in-transaction bị server kết thúc; <code>tn_report</code> thấy <code>&lt;insufficient privilege&gt;</code></td></tr>
<tr><td>044</td><td>VACUUM dọn tuple chết, không thu nhỏ tệp; snapshot cũ giữ tuple</td><td>2.500 dead → <code>0</code>; <code>keeps file size: 1867776</code>; <code>old snapshot keeps dead tuples: 500</code> rồi <code>0</code> sau khi phiên giữ commit</td></tr>
<tr><td>045</td><td>Độ bền bật đủ; checkpoint được đếm</td><td><code>durability settings: on/on/on</code>; <code>requested checkpoints counted: +3</code>; WAL ngay sau checkpoint 29.952 byte so với 7.544 byte lần lặp</td></tr>
<tr><td>046</td><td>Restore logic khớp dữ liệu và quyền; bỏ owner/ACL thì mất mô hình quyền</td><td><code>restored data digest: 100/3000000/1d3675e3…</code>, owner <code>tn_owner</code>, report ACL <code>t</code>; bản «bare» cùng digest nhưng owner <code>postgres</code>, ACL <code>f</code></td></tr>
<tr><td>047</td><td>Hàng chờ đo riêng; timeout và giới hạn kết nối hoạt động</td><td>queue waits <code>[0.1,0.1,53.4,53.5,104.7,104.9]</code> ms; <code>57014</code>; <code>too many connections for role "tn_app"</code></td></tr>
<tr><td>049</td><td>Tenant cách ly cả đọc lẫn ghi; biết role nào đi vòng</td><td>0 / 50; hai lỗi <code>new row violates row-level security policy</code>; UPDATE khác tenant chạm 0; SET LOCAL 50/0; report 0; owner 100 → FORCE 0; superuser 100</td></tr>
<tr><td>048</td><td>Phục hồi tới trước DELETE, giữ chuyến sau base backup</td><td>Mục 4</td></tr>
<tr><td>050</td><td>Kết thúc recovery, ghi được, trên timeline mới</td><td>Mục 4–5</td></tr>
</tbody>
</table>
<p>Cột thứ ba trích (có rút gọn) từ <code>OUTPUT.txt</code>. Ô nào người nhận không tìm được bằng tìm chuỗi là ô chưa được nghiệm thu.</p>
</details>

## 4. D2 — Diễn tập xoá nhầm và các cổng

`RUNBOOK.md` mô tả bảy bước. Dưới đây là từng bước với bằng chứng của lần chạy.

**Bước 1 — Base backup, rồi thêm dữ liệu sau backup.** `pg_basebackup -Fp -X stream --checkpoint=fast --label=p05_base`:

```text
--- 048 backup_label
  START WAL LOCATION: 0/3000028 (file 000000010000000000000003)
  CHECKPOINT LOCATION: 0/3000080
  BACKUP METHOD: streamed
  BACKUP FROM: primary
  LABEL: p05_base
  START TIMELINE: 1
```

Sau backup, source thêm chuyến 101 (tenant 1, 45.000 đ). Chuyến này **không có** trong base backup; nó chỉ sống được nhờ WAL. Runner ghi digest ngay lúc này làm kỳ vọng cho phục hồi.

**Bước 2 — Restore point, rồi gây hỏng.** `pg_create_restore_point('p05_before_delete')` ghi một bản ghi có tên vào WAL; sau đó `DELETE FROM ops.trips`:

```text
MEASURE 048 restore point p05_before_delete LSN: 0/4001DF0
PASS 048 source damaged deliberately: 0
```

LSN (*log sequence number* — vị trí trong dòng WAL) `0/4001DF0` là mốc mà mọi thứ phía sau sẽ xoay quanh.

**Bước 3 — Chờ WAL tới archive.** `pg_switch_wal()` đóng segment hiện tại; runner chờ cả tệp xuất hiện lẫn `pg_stat_archiver` bắt kịp:

```text
--- 048 pg_stat_archiver
   archived_count |    last_archived_wal     | failed_count
  ----------------+--------------------------+--------------
                5 | 000000010000000000000004 |            0
  (1 row)
```

Restore point `0/4001DF0` thuộc segment `…0004`; log recovery ở bước 5 cho thấy đích phải lấy segment này từ archive mới tới được target. Thiếu nó, chuyến 101 không về — vì thế RUNBOOK ghi «không đánh dấu có backup chỉ vì có base».

**Bước 4 — Dựng đích riêng.** Runner tạo cluster mới, đổi tên data directory khởi tạo, chép base backup vào, thêm cấu hình rồi tạo `recovery.signal`. Rút gọn đường dẫn:

```text
port=<cổng riêng>
archive_mode=off
ssl=off
restore_command='"node" ".../wal-copy.mjs" restore ".../archive/%f" "%p"'
recovery_target_name='p05_before_delete'
recovery_target_action='pause'
```

`recovery_target_action = 'pause'` là quyết định quan trọng nhất của runbook: tới target, server **dừng lại và chờ người kiểm**, chưa kết thúc recovery. Source không bị đụng tới; nó là hiện trường.

**Bước 5 — Cổng trước khi resume.** Log của đích:

```text
--- 048 recovery server log
  LOG:  restored log file "000000010000000000000003" from archive
  LOG:  starting point-in-time recovery to "p05_before_delete"
  LOG:  redo starts at <LSN>
  LOG:  consistent recovery state reached at <LSN>
  LOG:  restored log file "000000010000000000000004" from archive
  LOG:  recovery stopping at restore point "p05_before_delete", time <T>
  LOG:  pausing at the end of recovery
```

Rồi các cổng, tất cả phải đạt:

```text
PASS 048 recovered data including post-backup insert: 101/3045000/817b908d149b86ec12f316ce544dc676
PASS 048 RLS survives PITR: 51
MEASURE 048 start-to-verified-paused-recovery ms: 4284.1 (excludes base-copy and incident detection)
PASS 048 paused target is still read-only: true
```

| Cổng | Kỳ vọng | Vì sao |
|---|---|---|
| G1 Số dòng | 101 | 100 gốc + chuyến 101 sau base backup |
| G2 Tổng tiền | 3.045.000 đ | 3.000.000 đ + 45.000 đ |
| G3 Digest | `817b908d149b86ec12f316ce544dc676` | md5 của `id:tenant_id:fare:status` theo `id`; bắt được cả lỗi «đúng số dòng, sai nội dung» |
| G4 Tenant | `tn_app` với tenant 1 thấy 51 | Policy và role về cùng dữ liệu (PG-049); đếm bằng superuser không chứng minh điều này |
| G5 Chỉ đọc | INSERT thử bị từ chối, SQLSTATE `25006` | Đích còn đang recovery; chưa ai ghi được vào nó |

G1–G3 là một chuỗi do câu `digest` sinh ra; kỳ vọng được tính trên source **trước** khi phá, không gõ tay. Nếu một cổng lệch, RUNBOOK nói rõ: «Không resume hoặc đổi target âm thầm».

**Bước 6 — Resume và mở ghi.** Chỉ sau G1–G5, runner gọi `pg_wal_replay_resume()`. Với target action `pause`, resume nghĩa là **kết thúc recovery** ([Recovery Target](https://www.postgresql.org/docs/18/runtime-config-wal.html#RUNTIME-CONFIG-WAL-RECOVERY-TARGET)), không phải replay tiếp:

```text
--- 050 recovery server log after resume
  LOG:  selected new timeline ID: 2
  LOG:  archive recovery complete
  LOG:  database system is ready to accept connections
PASS 050 writable restored cluster: 102
PASS 050 promoted onto new timeline: 00000002
PASS 050 timeline history file: true
--- 050 00000002.history
  1	0/4001DF0	at restore point "p05_before_delete"
```

Chuyến 102 (tenant 2, 40.000 đ) được INSERT thành công: đích đã ghi được. Tên tệp WAL hiện tại bắt đầu bằng `00000002`.

**Bước 7 — Ghi và dọn.** Runner dọn cả hai cluster trong `finally`, kể cả khi assertion thất bại. Lab **không** cắt routing của ứng dụng — không có ứng dụng nào để cắt.

## 5. D3 — Timeline: vì sao không ghi tiếp trên timeline 1

<details><summary><b>Lời giải</b></summary>
<p>Tên tệp WAL gồm 24 ký tự hex: 8 ký tự đầu là timeline, 16 ký tự sau là vị trí segment. <code>000000010000000000000004</code> là segment 4 của timeline 1. Source vẫn đang sống trên timeline 1 và đã viết DELETE vào chính segment đó, sau LSN <code>0/4001DF0</code>.</p>
<p>Đích dừng ở <code>0/4001DF0</code> rồi bắt đầu nhận ghi mới (chuyến 102). Nếu nó viết tiếp trên timeline 1, WAL mới sẽ mang <b>cùng tên tệp</b> với WAL của source — hai lịch sử khác nhau cùng gọi là «segment 4 của timeline 1». Trong archive dùng chung, một bên sẽ ghi đè hoặc bị từ chối (helper của lab từ chối ghi đè tệp khác nội dung), và lần phục hồi sau có thể replay lẫn DELETE của source.</p>
<p>PostgreSQL tránh điều đó bằng cách mở <b>timeline</b> mới mỗi khi một archive recovery kết thúc (<a href="https://www.postgresql.org/docs/18/continuous-archiving.html#BACKUP-TIMELINES">Timelines</a>): ID mới là một số chưa có trong archive, WAL mới mang tiền tố <code>00000002</code>, không đụng tên với timeline 1. Tệp <code>00000002.history</code> ghi gia phả: dòng <code>1	0/4001DF0	at restore point "p05_before_delete"</code> đọc là «timeline 2 tách khỏi timeline 1 tại LSN 0/4001DF0, lý do: dừng ở restore point».</p>
<p>Lần phục hồi sau dùng chính tệp này. Mặc định <code>recovery_target_timeline = 'latest'</code>: server tìm history mới nhất trong archive, replay timeline 1 <b>chỉ tới</b> <code>0/4001DF0</code>, rồi chuyển sang timeline 2. Không có tệp history, server không biết nhánh 2 tồn tại hay tách ở đâu.</p>
<p>Điều lab <b>chưa</b> làm: đích chạy <code>archive_mode=off</code>, nên <code>00000002.history</code> và WAL timeline 2 chỉ nằm trong <code>pg_wal</code> của đích và bị xoá khi dọn. Trong production, cluster vừa phục hồi phải archive ngay từ lúc mở ghi và nên có base backup mới sớm. Hai lần phục hồi từ cùng archive mà không lần nào archive tệp history có thể cùng chọn timeline 2 — đúng loại nhầm lẫn timeline sinh ra để tránh. Lab không thử kịch bản này.</p>
</details>

## 6. D4 — Bằng chứng cần giữ

<details><summary><b>Lời giải</b></summary>
<table>
<thead><tr><th>Bằng chứng</th><th>Giữ vì</th><th>Lab có</th></tr></thead>
<tbody>
<tr><td>Phiên bản PostgreSQL, OS, Node, lệnh chạy</td><td>Tái lập và so sánh lần sau</td><td>3 dòng đầu <code>OUTPUT.txt</code></td></tr>
<tr><td><code>backup_label</code> của base dùng để phục hồi</td><td>Chứng minh base nào, bắt đầu ở WAL nào, timeline nào</td><td>Khối <code>048 backup_label</code></td></tr>
<tr><td>Target: tên, LSN, cách chọn</td><td>Ranh giới dữ liệu giữ/mất</td><td><code>p05_before_delete</code>, <code>0/4001DF0</code></td></tr>
<tr><td>Ảnh chụp <code>pg_stat_archiver</code></td><td>WAL cần thiết đã ra archive, <code>failed_count</code> = 0</td><td>Khối <code>048 pg_stat_archiver</code></td></tr>
<tr><td>Kỳ vọng tính trước sự cố và kết quả trên đích</td><td>G1–G4</td><td>Dòng <code>PASS 048 …</code></td></tr>
<tr><td>Log recovery: dừng ở target, pause, timeline mới</td><td>Chứng minh dừng đúng chỗ, mở ghi đúng cách</td><td>Khối <code>048/050 … server log</code></td></tr>
<tr><td>Tệp <code>.history</code></td><td>Gia phả cho lần phục hồi sau</td><td>Khối <code>050 00000002.history</code></td></tr>
<tr><td>Nhật ký quyết định: ai, lúc nào, cho phép resume và mở ghi</td><td>Trách nhiệm và dòng thời gian</td><td><b>Không</b> — runner tự resume</td></tr>
<tr><td>Danh sách giao dịch hợp lệ sau target cần bù</td><td>RPO thật</td><td><b>Không</b> — lab không có giao dịch hợp lệ sau target</td></tr>
</tbody>
</table>
<p><b>Không</b> lưu vào repo hay hồ sơ: tệp dump, WAL, base backup, chứng chỉ, khoá, mật khẩu (README P05). Bằng chứng là output kiểm tra; dữ liệu là thứ được bảo vệ bằng quy trình riêng.</p>
</details>

## 7. D5 — Khi cổng thất bại, và ai quyết định

<details><summary><b>Lời giải</b></summary>
<p>Bảng xử lý lỗi của <code>RUNBOOK.md</code> (trích các dòng liên quan phục hồi và quyền, nguyên văn):</p>
<table>
<thead><tr><th>Dấu hiệu</th><th>Kiểm bằng chứng</th><th>Hành động</th></tr></thead>
<tbody>
<tr><td>Restore logic lệch</td><td>pg_restore exit, count/sum/digest, owner/ACL</td><td>Giữ đích riêng để chẩn đoán; chưa nghiệm thu</td></tr>
<tr><td>Không có archive</td><td>archive_command, pg_stat_archiver, file segment</td><td>Sửa nguyên nhân; không đánh dấu có backup chỉ vì có base</td></tr>
<tr><td>Recovery không đạt target</td><td>recovery signal, target, restore_command, chuỗi WAL</td><td>Không resume hoặc đổi target âm thầm</td></tr>
<tr><td>Tenant đọc sai</td><td>current role/context, policy, FORCE/BYPASSRLS</td><td>Sửa mô hình quyền và context; kiểm lại cả INSERT</td></tr>
<tr><td>Permission không như dự kiến</td><td>Role hiện tại, owner, membership, ACL</td><td>Sửa GRANT/default privileges; không cấp ALL để bỏ lỗi</td></tr>
</tbody>
</table>
<p>Bổ sung theo cổng của mục 4 (đề xuất, lab không mô phỏng các lỗi này):</p>
<ul>
<li><b>Không tới pause</b> (runner chờ 30 giây): tìm <code>restored log file</code> cuối trong log đích; thiếu segment thì quay về bước 3, không hạ target cho «vừa» với WAL đang có.</li>
<li><b>G1–G3 lệch</b>: không resume; thiếu dữ liệu thường là target quá sớm hoặc thiếu WAL. Dựng đích mới với target khác — cluster đã kết thúc recovery không «lùi» được.</li>
<li><b>G4 lệch</b>: kiểm <code>pg_policies</code>, <code>relforcerowsecurity</code>, <code>rolbypassrls</code>; không mở API.</li>
<li><b>G5 ghi được</b>: đích đã thoát recovery trước khi kiểm; kết luận về target vô hiệu, dựng lại.</li>
</ul>
<p>Vai trò tối thiểu (lab chạy một mình, nên đây là đề xuất):</p>
<table>
<thead><tr><th>Vai trò</th><th>Quyết định / việc</th></tr></thead>
<tbody>
<tr><td>Chỉ huy sự cố</td><td>Tuyên bố sự cố, dừng ghi, giữ nhật ký quyết định</td></tr>
<tr><td>Người thực thi DB</td><td>Bước 1–6; không tự quyết resume</td></tr>
<tr><td>Người kiểm thứ hai</td><td>Chạy lại G1–G5 độc lập trước resume</td></tr>
<tr><td>Chủ nghiệp vụ</td><td>Chấp nhận target, mức dữ liệu mất, kế hoạch nhập bù</td></tr>
<tr><td>Chủ ứng dụng</td><td>Routing sang đích, xác nhận API sau mở ghi</td></tr>
<tr><td>Bảo mật</td><td>Credential, TLS, RLS trên đích trước khi mở ra ngoài</td></tr>
</tbody>
</table>
<p>Điểm cốt lõi: người bấm <code>pg_wal_replay_resume()</code> không phải người quyết định nó. Resume là bước một chiều.</p>
</details>

## 8. D6 — RPO và RTO, đo được gì và chưa đo gì

**RPO** (*recovery point objective*) là lượng dữ liệu tối đa chấp nhận mất; **RTO** (*recovery time objective*) là thời gian tối đa từ sự cố tới khi dịch vụ dùng lại được. Cả hai là mục tiêu nghiệp vụ; diễn tập cho ra số **quan sát** để so với mục tiêu.

<details><summary><b>Lời giải</b></summary>
<p><b>RPO của lần diễn tập:</b> mất mọi thứ sau <code>0/4001DF0</code> trên timeline 1. Trong lab, sau mốc đó chỉ có DELETE — đúng thứ ta muốn mất. Ở production, target hiếm khi là restore point đặt sẵn ngay trước sự cố; thường là thời điểm hoặc LSN tìm được sau khi điều tra, và mọi giao dịch hợp lệ giữa target với lúc dừng ghi sẽ mất, phải nhập bù. Lab không thử <code>recovery_target_time</code> (README).</p>
<table>
<thead><tr><th>Mốc RTO</th><th>Lab</th></tr></thead>
<tbody>
<tr><td>Phát hiện sự cố</td><td>Không đo</td></tr>
<tr><td>Quyết định, dừng ghi, chọn target</td><td>Không đo</td></tr>
<tr><td>Chuẩn bị máy, chép base backup</td><td>Không đo (loại trừ khỏi số đo)</td></tr>
<tr><td>Khởi động đích → replay → pause → G1–G4</td><td><b>4.284,1 ms</b> (một lần chạy, fixture 100 dòng, 2 segment WAL, cùng máy)</td></tr>
<tr><td>Kiểm G5, chờ người duyệt resume</td><td>Không đo</td></tr>
<tr><td>Kết thúc recovery, INSERT thử</td><td>Không đo</td></tr>
<tr><td>Routing, xác nhận nghiệp vụ</td><td>Không làm</td></tr>
</tbody>
</table>
<p>Số 4.284,1 ms dao động giữa các lần chạy (lab không assert millisecond; một lần chạy trước của cùng lab ghi 4.175,1 ms). Nó cho biết bước kỹ thuật ở quy mô đồ chơi tốn vài giây. Nó <b>không</b> cho biết RTO: base backup production lớn hơn hàng nghìn lần, WAL cần replay tỉ lệ với khối lượng ghi từ base backup tới target, và các mốc «không đo» thường chiếm phần lớn thời gian thật. Ghi số vào hồ sơ kèm đúng câu «excludes base-copy and incident detection».</p>
</details>

## 9. Tiêu chí tự chấm

Mỗi tiêu chí 10 điểm; từ 80 điểm là đạt; tiêu chí 1, 3 và 7 bắt buộc.

| # | Tiêu chí | Đạt khi |
|---|---|---|
| 1 | Bằng chứng thật | Mọi ô D1 tìm được bằng tìm chuỗi trong `OUTPUT.txt` |
| 2 | Kỳ vọng tính trước | Digest kỳ vọng sinh trên source trước khi phá, không gõ tay |
| 3 | Cổng trước resume | G1–G5 đạt trước `pg_wal_replay_resume()`; nói được vì sao resume là một chiều |
| 4 | Quyền cũng là dữ liệu | Có G4 chạy bằng role runtime, không chỉ đếm bằng superuser |
| 5 | WAL, không chỉ base | Chỉ ra chuyến 101 sống nhờ segment `…0004` trong archive |
| 6 | Timeline | Đọc được tệp history; giải thích va chạm tên tệp nếu không có timeline mới |
| 7 | RPO/RTO trung thực | Tách số đo được khỏi mốc chưa đo; không gọi 4,3 s là RTO |
| 8 | Xử lý lỗi | Mỗi cổng có hành động khi lệch; không «đổi target cho khớp» |
| 9 | Vai trò | Người thực thi khác người quyết resume và chấp nhận mất dữ liệu |
| 10 | Giới hạn | Liệt kê thứ lab chưa chứng nhận (mục 11) |

## 10. Tổng kết Phần 5

| Bài | Đóng góp vào hồ sơ bàn giao |
|---|---|
| [PG-041 · Role, GRANT và quyền tối thiểu](/articles/postgresql-role-grant-least-privilege) | Owner/app/report/monitor tách nhau; default privileges; ACL làm bằng chứng |
| [PG-042 · Xác thực và bảo vệ kết nối](/articles/postgresql-ket-noi-auth-tls) | SCRAM, `hostssl`, `verify-full`; các lần từ chối có log |
| [PG-043 · Quan sát phiên làm việc](/articles/postgresql-pg-stat-activity-quan-sat) | Tìm blocker bằng `pg_blocking_pids`; `lock_timeout`, idle timeout |
| [PG-044 · VACUUM, autovacuum và bloat](/articles/postgresql-vacuum-autovacuum-bloat) | Tuple chết, horizon bị giữ bởi snapshot cũ |
| [PG-045 · WAL, checkpoint và độ bền](/articles/postgresql-wal-checkpoint-durability) | Cấu hình bền vững; full-page image sau checkpoint |
| [PG-046 · pg_dump và khôi phục](/articles/postgresql-pg-dump-restore) | Restore logic khớp digest; role nằm ở globals |
| [PG-047 · Connection pool và timeout](/articles/postgresql-connection-pool-timeout) | Hàng chờ đo riêng; `statement_timeout`; giới hạn kết nối theo role |
| [PG-048 · PITR với base backup và WAL](/articles/postgresql-pitr-base-backup-wal) | Base + archive + restore point + pause |
| [PG-049 · Row-level security](/articles/postgresql-rls-tenant-security) | Ma trận tenant; policy sống qua PITR (G4) |

Sản phẩm của phần đúng như khung: runbook phân quyền, giám sát, bảo trì và restore đã đối soát. **Chưa** có: standby và failover, partition, replication logic, nâng phiên bản — Phần 6.

## 11. Hồ sơ production còn thiếu

Câu kết của `RUNBOOK.md`: «Thiết kế storage offsite, encryption, retention, role mapping, kiểm extension và phục hồi định kỳ; các mục này chưa được lab local chứng nhận.» Thành việc cụ thể trước khi dùng cho production:

- **Archive và backup ngoài vùng lỗi**, mã hoá, có retention, quyền xoá tách khỏi quyền ghi. Archive của lab nằm cùng thư mục tạm với source.
- **Đích archive ngay khi mở ghi**, có base backup mới (mục 5).
- **Role mapping và extension**: phục hồi vật lý mang role theo; phục hồi logic sang cluster khác thì không.
- **Phục hồi định kỳ có lịch**, người chạy, nơi lưu output. Backup chưa từng restore thì chưa phải backup.
- **Mốc thời gian và quyết định** dừng/mở ghi, giao dịch cần bù; **phương án quay lại** khi đích đã nhận ghi mới — trỏ connection string về source cũ tạo hai lịch sử lệch nhau.
- **Thứ lab không mô phỏng** (README): mất điện, hỏng đĩa, failover, offsite, tải lớn.

## 12. Bẫy hay gặp khi tự làm

- **Resume vì «server đã lên».** Server nhận kết nối chỉ đọc từ lúc consistent; điều đó không nói gì về target.
- **Đếm bằng superuser rồi mở API.** G4 bắt lỗi mà G1 không thấy.
- **Phục hồi đè lên source.** Mất hiện trường và đường lui.
- **Gõ số kỳ vọng sau khi thấy kết quả.** Kỳ vọng phải có trước sự cố.
- **Sửa `OUTPUT.txt` cho «đúng»** khi cổng thất bại — runner không ghi đè output khi lỗi chính là để ngăn điều này.

## 13. Bài tập mở rộng

Không có lời giải chạy sẵn; mỗi bài nêu điều một lời giải đúng phải chứng minh. Chỉ chạy trên bản sao lab.

**Bài 1 — Bàn giao cho người thứ hai.** Một người chạy lab; người khác chỉ dùng `README.md` và `RUNBOOK.md` để tái lập, rồi đưa một lỗi vào bản sao: sai hostname TLS, thiếu một segment WAL, `tn_report` mất quyền, hoặc tenant context không được đặt. Đạt khi người thứ nhất, chỉ nhìn output, chỉ ra đúng **cổng** thất bại và nguyên nhân; runner thoát khác 0 và `OUTPUT.txt` không bị ghi đè; không assertion nào bị tắt.

**Bài 2 — Target theo thời gian.** Thay `recovery_target_name` bằng `recovery_target_time`. Đạt khi chứng minh thời điểm đã chọn nằm sau commit của chuyến 101 và trước commit của DELETE (dựa trên bằng chứng thật, không ước đoán), G1–G5 vẫn đạt, và giải thích vai trò của `recovery_target_inclusive`.

**Bài 3 — Phục hồi lần hai.** Sau khi đích lên timeline 2 và **có** archive tệp history cùng WAL timeline 2, phục hồi lần nữa từ base cũ với `recovery_target_timeline = 'latest'`. Đạt khi chỉ ra từ log đích mới đã đi theo nhánh nào, có thấy chuyến 102 hay không, và vì sao DELETE của source không bị replay.

**Bài 4 — Hồ sơ production một trang.** Viết bảng RPO/RTO cho Tới Nơi với ba cột: mục tiêu nghiệp vụ, số đã đo (kèm nguồn), số chưa đo (kèm cách sẽ đo). Đạt khi không ô «đã đo» nào thiếu nguồn và con số 4.284,1 ms chỉ xuất hiện ở đúng một mốc.

## 14. Đọc tiếp

- [PostgreSQL 18 · Continuous Archiving and Point-in-Time Recovery](https://www.postgresql.org/docs/18/continuous-archiving.html): base backup, archive, timeline và tệp history.
- [PostgreSQL 18 · Write Ahead Log — Archive Recovery, Recovery Target](https://www.postgresql.org/docs/18/runtime-config-wal.html): `restore_command`, `recovery_target_name`, `recovery_target_action`, `recovery_target_timeline`.
- [PostgreSQL 18 · System Administration Functions](https://www.postgresql.org/docs/18/functions-admin.html): `pg_create_restore_point`, `pg_switch_wal`, `pg_wal_replay_resume`, `pg_is_wal_replay_paused`.
- [PostgreSQL 18 · pg_basebackup](https://www.postgresql.org/docs/18/app-pgbasebackup.html) và [pg_stat_archiver](https://www.postgresql.org/docs/18/monitoring-stats.html).
- Lab: `examples/toi-noi-postgresql/p05/RUNBOOK.md`, `p05/OUTPUT.txt`, `p05/run.mjs`, `p05/README.md` (phương pháp đo), `p05/wal-copy.mjs` (archive/restore helper).

Bài tiếp theo: Phần 6 · Mở rộng, tích hợp và vận hành đầu cuối sắp ra mắt. Đọc lại từ đầu Phần 5: [PG-041: Role, GRANT và quyền tối thiểu](/articles/postgresql-role-grant-least-privilege)
