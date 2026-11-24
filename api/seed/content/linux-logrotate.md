---
slug: linux-logrotate
title: "logrotate: xoay vòng, nén và giữ log của bao-gia theo số ngày, và báo dịch vụ mở file log mới"
summary: "Vì sao file log phải được xoay vòng, logrotate chạy thế nào trên Ubuntu (logrotate.timer, logrotate.conf, logrotate.d), viết cấu hình cho log yêu cầu của bao-gia: daily, rotate 14, compress, delaycompress, notifempty, su, create, postrotate gửi reload; chạy thử bằng -d, ép xoay bằng -f -v, kiểm chứng dịch vụ ghi vào file mới nhờ SIGHUP, và file trạng thái /var/lib/logrotate/status."
category: devops-linux
level: advanced
tags: [linux, lnx-039, logrotate, log, gzip, sighup, systemd-timer, ssh-studio, toi-noi]
language: vi
---

Journal có giới hạn dung lượng ([LNX-037](/articles/linux-journalctl-doc-log)). File log yêu cầu `/var/log/toinoi/bao-gia.log` thì không: mỗi báo giá thêm một dòng, ngày này qua ngày khác. Ở Phần 2, Bình đã chép tay vài file log cũ (`bao-gia.log.1`, `bao-gia.log.2.gz`) sang server cho An tập `zcat`; Phần 4 cho việc đó chạy tự động, đúng cách, bằng **logrotate**.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-039 · Phần 4/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-tar-gzip-nen-luu-tru">LNX-018</a> (gzip), <a href="/articles/linux-systemd-unit-bao-gia">LNX-036</a> (<code>reload</code> gửi SIGHUP), <a href="/articles/linux-cron-systemd-timer">LNX-038</a> (timer). Trạng thái đầu bài: <code>npm run setup -- 39</code>.</p>
</div>

## 1. Vì sao phải xoay vòng

Một file log không ai dọn sẽ có ngày làm đầy đĩa; và khi đĩa đầy, mọi dịch vụ trên server cùng hỏng. File log lớn cũng khó đọc, khó tìm, khó chép đi. **Xoay vòng** (*log rotation*) là định kỳ đổi tên file đang ghi thành bản lưu (`.1`), tạo file mới trống để ghi tiếp, nén các bản cũ, và xoá bản quá cũ.

```text
an@toinoi-app-01:~$ ls -l /var/log/toinoi/
total 60
-rw-r--r-- 1 baogia toinoi 27331 Nov 24 07:05 bao-gia.log
-rw-r--r-- 1 baogia toinoi 23353 Nov 20 11:16 bao-gia.log.1
-rw-r--r-- 1 baogia toinoi  4314 Nov 19 11:07 bao-gia.log.2.gz
an@toinoi-app-01:~$ for i in 1 2 3 4 5; do curl -s -o /dev/null "http://127.0.0.1:8080/bao-gia?dich_vu=bike&km=$i"; done
an@toinoi-app-01:~$ tail -n 2 /var/log/toinoi/bao-gia.log
2026-11-24 07:06:00 INFO  q-100004 dich_vu=bike km=4 gia=18000 0 ms
2026-11-24 07:06:00 INFO  q-100005 dich_vu=bike km=5 gia=21000 0 ms

an@toinoi-app-01:~$ systemctl list-timers logrotate.timer --no-pager | cut -c1-110
NEXT                        LEFT LAST PASSED UNIT            ACTIVATES
Wed 2026-11-25 00:00:00 +07  16h -         - logrotate.timer logrotate.service

1 timers listed.
Pass --all to see loaded but inactive timers, too.
an@toinoi-app-01:~$ grep -v '^#' /etc/logrotate.conf | grep -v '^$'
weekly
su root adm
rotate 4
create
include /etc/logrotate.d
an@toinoi-app-01:~$ ls /etc/logrotate.d/
alternatives  apt  btmp  dpkg  wtmp
an@toinoi-app-01:~$ cat /etc/logrotate.d/apt
/var/log/apt/term.log {
  rotate 12
  monthly
  compress
  missingok
  notifempty
}

/var/log/apt/history.log {
  rotate 12
  monthly
  compress
  missingok
  notifempty
}
```

- Mỗi yêu cầu báo giá thêm một dòng vào `bao-gia.log`.
- **logrotate** đã cài sẵn và chạy mỗi ngày bằng **`logrotate.timer`** ([LNX-038](/articles/linux-cron-systemd-timer)).
- **`/etc/logrotate.conf`** đặt mặc định (`weekly`, `rotate 4`, `create`, `su root adm`) rồi `include /etc/logrotate.d`: mỗi gói thêm một file cấu hình cho log của nó, như `apt`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-logrotate/ssh-studio-01.webp" alt="Terminal của SSH Studio: list-timers logrotate.timer chạy lúc 00:00, logrotate.conf với weekly, su root adm, rotate 4, create, include, ls /etc/logrotate.d và nội dung file apt" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · log của bao-gia và logrotate có sẵn</figcaption></figure>

## 2. Cấu hình cho bao-gia

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-logrotate/ssh-studio-02.webp" alt="Vim mở /etc/logrotate.d/bao-gia qua sudoedit trong SSH Studio: khối cho /var/log/toinoi/bao-gia.log với daily, rotate 14, compress, delaycompress, missingok, notifempty, su baogia toinoi, create 0644 baogia toinoi và postrotate gọi systemctl reload bao-gia.service" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>/etc/logrotate.d/bao-gia</code></figcaption></figure>

```text
# Xoay vòng log yêu cầu của bao-gia (LNX-039): mỗi ngày, giữ 14 bản, nén từ bản thứ hai.
/var/log/toinoi/bao-gia.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    su baogia toinoi
    create 0644 baogia toinoi
    postrotate
        systemctl reload bao-gia.service > /dev/null 2>&1 || true
    endscript
}
```

| Dòng | Ý nghĩa |
|---|---|
| `daily` | Xoay mỗi ngày (còn `weekly`, `monthly`, hoặc `size 50M`) |
| `rotate 14` | Giữ 14 bản cũ; bản thứ 15 bị xoá |
| `compress`, `delaycompress` | Nén bằng gzip, nhưng **chừa bản `.1`** chưa nén: chương trình có thể vẫn đang ghi nốt vào nó, và người đọc log hôm qua đỡ phải `zcat` |
| `missingok`, `notifempty` | Không có file thì bỏ qua; file trống thì không xoay |
| `su baogia toinoi` | Làm việc với file bằng quyền của chủ thư mục log, không phải `root` (an toàn hơn với thư mục không thuộc `root`) |
| `create 0644 baogia toinoi` | File mới tạo ra với quyền và chủ này |
| `postrotate … endscript` | Chạy sau khi xoay: báo `bao-gia` mở file mới |

Vì sao cần `postrotate`? Một chương trình đang ghi giữ **file đang mở**, tức giữ chính file đó chứ không phải cái tên. Đổi tên `bao-gia.log` thành `bao-gia.log.1` thì `bao-gia` vẫn ghi tiếp vào… `bao-gia.log.1`. Phải báo nó đóng file và mở lại theo tên: với `bao-gia`, đó là `SIGHUP`, qua `systemctl reload` ([LNX-036](/articles/linux-systemd-unit-bao-gia)). Chương trình không hỗ trợ mở lại file thì dùng `copytruncate` (chép rồi cắt file gốc về 0), chấp nhận mất vài dòng ghi đúng lúc cắt.

```text
an@toinoi-app-01:~$ sudo logrotate -d /etc/logrotate.d/bao-gia 2>&1 | grep -E 'rotating pattern|log needs|log does not need|considering log|error' | head -n 6
rotating pattern: /var/log/toinoi/bao-gia.log  after 1 days (14 rotations)
considering log /var/log/toinoi/bao-gia.log
  log does not need rotating (log has already been rotated)
```

**`logrotate -d`** (*debug*) chạy thử, không đổi gì, in ra sẽ làm gì. Nó hiểu cấu hình (`after 1 days (14 rotations)`), và nói log **chưa cần xoay**: logrotate chưa từng thấy file này, nên coi «bây giờ» là lần xoay gần nhất; lần xoay thật đầu tiên là ngày mai.

## 3. Xoay ngay và kiểm chứng

**`-f`** ép xoay dù chưa tới hạn; **`-v`** in từng bước:

```text
an@toinoi-app-01:~$ sudo logrotate -f -v /etc/logrotate.d/bao-gia 2>&1 | grep -E 'bao-gia\.log(\.1)? to|^(creating|compressing|running)' | cut -c1-90
compressing log with: /bin/gzip
renaming /var/log/toinoi/bao-gia.log to /var/log/toinoi/bao-gia.log.1
creating new /var/log/toinoi/bao-gia.log mode = 0644 uid = 990 gid = 2000
running postrotate script
an@toinoi-app-01:~$ ls -l /var/log/toinoi/
total 44
-rw-r--r-- 1 baogia toinoi     0 Nov 24 07:06 bao-gia.log
-rw-r--r-- 1 baogia toinoi 27671 Nov 24 07:06 bao-gia.log.1
-rw-r--r-- 1 baogia toinoi  4556 Nov 20 11:16 bao-gia.log.2.gz
-rw-r--r-- 1 baogia toinoi  4314 Nov 19 11:07 bao-gia.log.3.gz
an@toinoi-app-01:~$ journalctl -u bao-gia --no-pager -n 2 -o cat
INFO nhan SIGHUP: mo lai file log, nap lai bang gia (4 loai dich vu)
Reloaded bao-gia.service - bao-gia: dich vu bao gia cua Toi Noi.
an@toinoi-app-01:~$ curl -s -o /dev/null 'http://127.0.0.1:8080/bao-gia?dich_vu=car7&km=3'
an@toinoi-app-01:~$ cat /var/log/toinoi/bao-gia.log
2026-11-24 07:06:00 INFO  q-100006 dich_vu=car7 km=3 gia=32000 0 ms
an@toinoi-app-01:~$ tail -n 1 /var/log/toinoi/bao-gia.log.1
2026-11-24 07:06:00 INFO  q-100005 dich_vu=bike km=5 gia=21000 0 ms
an@toinoi-app-01:~$ zcat /var/log/toinoi/bao-gia.log.2.gz | head -n 1
2026-11-20 06:00:00 INFO  bao-gia 1.4.1 khoi dong, cong 8080, moi truong production
an@toinoi-app-01:~$ sudo cat /var/lib/logrotate/status | grep toinoi
"/var/log/toinoi/bao-gia.log" 2026-11-24-7:6:0
```

- Sau khi xoay: `bao-gia.log` mới trống, của `baogia:toinoi`, quyền `0644`; bản hôm nay thành `.1` (chưa nén, vì `delaycompress`); bản `.1` cũ (20/11) được nén thành `.2.gz`; `.2.gz` cũ (19/11) thành `.3.gz`.
- Journal cho thấy `postrotate` đã chạy: `bao-gia` nhận `SIGHUP`, mở lại file log.
- Bằng chứng quyết định: một yêu cầu mới ghi vào **`bao-gia.log` mới** (`q-100006`), còn `bao-gia.log.1` dừng ở `q-100005`. Không có `postrotate`, dòng `q-100006` sẽ nằm trong `.1`.
- **`/var/lib/logrotate/status`** ghi lần xoay cuối của mỗi file; logrotate dựa vào đó để biết «đã một ngày chưa».

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-logrotate/ssh-studio-03.webp" alt="Terminal của SSH Studio: logrotate -f -v in compressing, renaming bao-gia.log to bao-gia.log.1, creating new, running postrotate script; ls -l với bao-gia.log 0 byte, .1, .2.gz, .3.gz; journal nhan SIGHUP mo lai file log; yêu cầu mới q-100006 nằm trong file mới còn .1 kết thúc ở q-100005; zcat .2.gz; dòng trạng thái trong /var/lib/logrotate/status" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · xoay ngay và kiểm chứng file mới</figcaption></figure>

## 4. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Cấu hình chung / theo gói | `/etc/logrotate.conf`, `/etc/logrotate.d/TÊN` |
| Chạy thử, không đổi gì | `sudo logrotate -d /etc/logrotate.d/TÊN` |
| Ép xoay ngay | `sudo logrotate -f -v /etc/logrotate.d/TÊN` |
| Lần chạy tự động | `systemctl list-timers logrotate.timer`, `journalctl -u logrotate` |
| Lần xoay cuối của mỗi file | `/var/lib/logrotate/status` |
| Đọc bản đã nén | `zcat`, `zgrep`, `zless` |

## 5. Lỗi thường gặp

**Xoay mà không báo chương trình.** File mới trống mãi, log tiếp tục chảy vào `.1`, rồi vào file đã bị nén hay xoá. Luôn có `postrotate` (hoặc `copytruncate`).

**`compress` mà không `delaycompress`.** Chương trình vẫn đang ghi vào bản vừa đổi tên đúng lúc nó bị nén: mất dòng.

**Thư mục log ghi được bởi nhóm không phải `root`.** logrotate từ chối với lỗi «insecure permissions» trừ khi có `su`.

**Viết file cấu hình rồi không thử `-d`.** Một lỗi cú pháp làm logrotate bỏ qua cả file đó; bạn chỉ biết khi đĩa đầy.

**Dùng `rotate` mà không tính dung lượng.** 14 bản × kích thước một ngày phải vừa với đĩa; thêm `maxsize` nếu lưu lượng có thể tăng đột biến.

## Bài tập

1. Đổi `daily` thành `size 1k`, gửi vài chục yêu cầu, chạy `sudo logrotate -v /etc/logrotate.d/bao-gia` (không `-f`). Nó có xoay không? Đặt lại `daily`.
2. Thêm `dateext` và `dateformat -%Y%m%d`. Tên các bản lưu thay đổi thế nào? Ưu, nhược điểm so với `.1`, `.2.gz`?
3. Bỏ `postrotate`, ép xoay, gửi một yêu cầu. Dòng mới nằm ở file nào? `ls -l /proc/$(systemctl show bao-gia -p MainPID --value)/fd` cho thấy gì? (Cần `sudo`.) Đặt lại cấu hình.
4. Đọc `cat /usr/lib/systemd/system/logrotate.timer`. Logrotate chạy lúc nào, và có chạy bù không?

## Đọc tiếp

[LNX-040 · Thực hành: đưa bao-gia thành dịch vụ systemd hoàn chỉnh](/articles/linux-thuc-hanh-dich-vu-systemd): siết dịch vụ bằng drop-in, kiểm tra bàn giao, và bộ chấm có khởi động lại cả server.
