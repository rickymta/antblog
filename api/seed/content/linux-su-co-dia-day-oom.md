---
slug: linux-su-co-dia-day-oom
title: "Sự cố: đĩa đầy, hết inode, tiến trình bị OOM killer giết, chẩn đoán theo trình tự có bằng chứng"
summary: "Ba sự cố thật trên toinoi-app-01: job sao lưu đêm thất bại vì /du-lieu đầy (journal, df, du, giữ 3 bản mới nhất), không tạo được file dù còn chỗ vì hết inode (df -i, đếm file theo thư mục), và một job báo cáo bị giết vì vượt MemoryMax (Result: oom-kill trong systemctl và journal), mỗi lần đi từ triệu chứng tới bằng chứng rồi mới sửa."
category: devops-linux
level: advanced
tags: [linux, lnx-058, incident, disk-full, inode, oom-killer, journalctl, systemd, ssh-studio, toi-noi]
language: vi
---

Sáng thứ Ba, kênh vận hành báo: «Job sao lưu đêm qua đỏ.» Chưa ai biết vì sao. Kiểu sự cố này dễ dẫn tới sửa theo cảm tính: xoá đại vài file, khởi động lại đại vài dịch vụ, rồi không ai biết thứ gì đã giúp. An làm theo một trình tự cố định: **triệu chứng → bằng chứng → nguyên nhân → sửa → kiểm chứng**, và ghi lại từng bước.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-058 · Phần 6/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-df-du-dung-luong">LNX-043</a> (df, du, inode), <a href="/articles/linux-bo-nho-tai-he-thong">LNX-048</a> (bộ nhớ), <a href="/articles/linux-journalctl-doc-log">LNX-037</a> (journal). Trạng thái đầu bài: <code>npm run setup -- 58</code>. Lab dựng sẵn sự cố: job <code>sao-luu-dem</code> ghi vào <code>/du-lieu</code> cho tới khi đầy.</p>
</div>

## 1. Sự cố 1: job sao lưu thất bại

### Triệu chứng

```text
an@toinoi-app-01:~$ systemctl --failed --no-legend
● sao-luu-dem.service loaded failed failed Sao luu dem bao-gia vao /du-lieu
an@toinoi-app-01:~$ journalctl -u sao-luu-dem --no-pager -o cat | grep -E 'tar:|gzip:|Failed' | head -n 4
gzip: stdout: No space left on device
/usr/bin/tar: /du-lieu/sao-luu/bao-gia-2026-11-24.tar.gz: Wrote only 6144 of 10240 bytes
/usr/bin/tar: Child returned status 1
/usr/bin/tar: Error is not recoverable: exiting now
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-dia-day-oom/ssh-studio-05.webp" alt="Terminal của SSH Studio: systemctl --failed cho sao-luu-dem.service failed, journalctl lọc gzip: stdout: No space left on device và tar: Wrote only 6144 of 10240 bytes" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · triệu chứng: job sao lưu đêm thất bại vì hết chỗ</figcaption></figure>

`systemctl --failed` liệt kê unit đang ở trạng thái lỗi: nơi đầu tiên nhìn khi «có gì đó đỏ». Journal của unit cho câu trả lời: **`No space left on device`**, `tar` chỉ ghi được một phần file.

### Bằng chứng

```text
an@toinoi-app-01:~$ df -h /du-lieu
Filesystem      Size  Used Avail Use% Mounted on
/dev/loop2      488M  452M  4.0K 100% /du-lieu
an@toinoi-app-01:~$ sudo du -xh --max-depth=1 /du-lieu | sort -h
16K     /du-lieu/lost+found
452M    /du-lieu
452M    /du-lieu/sao-luu
an@toinoi-app-01:~$ ls -lh /du-lieu/sao-luu
total 452M
-rw-r--r-- 1 an toinoi 75M Nov 18 02:00 bao-gia-2026-11-18.tar.gz
-rw-r--r-- 1 an toinoi 75M Nov 19 02:00 bao-gia-2026-11-19.tar.gz
-rw-r--r-- 1 an toinoi 75M Nov 20 02:00 bao-gia-2026-11-20.tar.gz
-rw-r--r-- 1 an toinoi 75M Nov 21 02:00 bao-gia-2026-11-21.tar.gz
-rw-r--r-- 1 an toinoi 75M Nov 22 02:00 bao-gia-2026-11-22.tar.gz
-rw-r--r-- 1 an toinoi 77M Nov 23 02:00 bao-gia-2026-11-23.tar.gz
-rw-r--r-- 1 an an       0 Nov 24 09:41 bao-gia-2026-11-24.tar.gz
```

- `df -h`: `/du-lieu` dùng 100%.
- `du -xh --max-depth=1`: toàn bộ nằm ở `sao-luu/` (`-x`: không đi sang ổ khác).
- `ls -lh`: một bản sao lưu mỗi đêm, không ai xoá bản cũ. File 0 byte của đêm qua là bản dở dang.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-dia-day-oom/ssh-studio-01.webp" alt="Terminal của SSH Studio: df -h /du-lieu cho 452M dùng, 4.0K trống, 100%; du --max-depth=1 chỉ ra /du-lieu/sao-luu 452M; ls -lh liệt kê các bản sao lưu 75M từ 18 đến 23/11 và bản 24/11 rỗng" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đĩa nào đầy, cái gì chiếm chỗ: các bản sao lưu cũ</figcaption></figure>

### Sửa và kiểm chứng

Nguyên nhân gốc không phải «đĩa nhỏ» mà là **không có chính sách giữ bản**. Chính sách của Tới Nơi: giữ 3 bản mới nhất.

```text
an@toinoi-app-01:~$ ls -t /du-lieu/sao-luu/*.tar.gz | tail -n +4
/du-lieu/sao-luu/bao-gia-2026-11-21.tar.gz
/du-lieu/sao-luu/bao-gia-2026-11-20.tar.gz
/du-lieu/sao-luu/bao-gia-2026-11-19.tar.gz
/du-lieu/sao-luu/bao-gia-2026-11-18.tar.gz
an@toinoi-app-01:~$ ls -t /du-lieu/sao-luu/*.tar.gz | tail -n +4 | xargs rm -v
removed '/du-lieu/sao-luu/bao-gia-2026-11-21.tar.gz'
removed '/du-lieu/sao-luu/bao-gia-2026-11-20.tar.gz'
removed '/du-lieu/sao-luu/bao-gia-2026-11-19.tar.gz'
removed '/du-lieu/sao-luu/bao-gia-2026-11-18.tar.gz'
an@toinoi-app-01:~$ sudo systemctl reset-failed sao-luu-dem
an@toinoi-app-01:~$ sudo systemctl start sao-luu-dem
an@toinoi-app-01:~$ systemctl is-failed sao-luu-dem; df -h /du-lieu
inactive
Filesystem      Size  Used Avail Use% Mounted on
/dev/loop2      488M  152M  300M  34% /du-lieu
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-dia-day-oom/ssh-studio-06.webp" alt="Terminal của SSH Studio: ls -t | tail -n +4 liệt kê bốn bản cũ nhất 18 đến 21/11, xargs rm -v xoá cả bốn" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · giữ 3 bản mới nhất, xoá phần còn lại</figcaption></figure>

- Chạy `ls -t … | tail -n +4` một mình trước để **xem** sẽ xoá gì, rồi mới thêm `| xargs rm -v` (`-v` in từng file đã xoá: bằng chứng cho báo cáo).
- `systemctl reset-failed` xoá trạng thái lỗi cũ, rồi chạy lại job, xác nhận `inactive` (oneshot chạy xong) chứ không còn `failed`.
- File dở dang của đêm qua đã được job ghi đè bằng bản đầy đủ.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-dia-day-oom/ssh-studio-02.webp" alt="Terminal của SSH Studio: systemctl reset-failed sao-luu-dem, start lại job, is-failed in inactive, df -h /du-lieu còn 34%" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · chạy lại job sao lưu sau khi dọn chỗ</figcaption></figure>

Việc phải làm tiếp (vào runbook): đưa bước dọn vào chính job sao lưu (`ExecStartPost=`), và để `kiem-tra-van-hanh` ([LNX-060](/articles/linux-thuc-hanh-runbook-van-hanh)) cảnh báo từ 80%.

## 2. Sự cố 2: còn chỗ mà không tạo được file

Vài giờ sau, một job khác báo lỗi tạo file trên `/du-lieu`.

```text
an@toinoi-app-01:~$ touch /du-lieu/bao-cao-moi.txt
touch: cannot touch '/du-lieu/bao-cao-moi.txt': No space left on device
an@toinoi-app-01:~$ df -h /du-lieu
Filesystem      Size  Used Avail Use% Mounted on
/dev/loop2      488M  153M  299M  34% /du-lieu
an@toinoi-app-01:~$ df -i /du-lieu
Filesystem     Inodes IUsed IFree IUse% Mounted on
/dev/loop2      32768 32768     0  100% /du-lieu
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-dia-day-oom/ssh-studio-07.webp" alt="Terminal của SSH Studio: touch /du-lieu/bao-cao-moi.txt báo No space left on device, df -h cho 34% dùng, df -i cho 32768 inode đã dùng hết 100%" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · còn chỗ mà không tạo được file: hết inode</figcaption></figure>

`No space left on device`, trong khi `df -h` nói còn gần 300 MB. Thông báo lỗi giống hệt sự cố 1, nhưng nguyên nhân khác: **hết inode** ([LNX-043](/articles/linux-df-du-dung-luong)). Mỗi file cần một inode; ổ ext4 512 MB này chỉ có 32 768 inode, và tất cả đã dùng hết.

```text
an@toinoi-app-01:~$ sudo find /du-lieu -xdev -type f | cut -d/ -f1-3 | sort | uniq -c | sort -n | tail -n 3
      1 /du-lieu/thu.txt
      3 /du-lieu/sao-luu
  32751 /du-lieu/tam
an@toinoi-app-01:~$ find /du-lieu/tam -type f -name 'phien-*' -delete
an@toinoi-app-01:~$ df -i /du-lieu
Filesystem     Inodes IUsed IFree IUse% Mounted on
/dev/loop2      32768    17 32751    1% /du-lieu
```

- Đếm file theo thư mục cấp hai: `/du-lieu/tam` có hơn 32 000 file `phien-*`: file tạm của một tiến trình không bao giờ dọn.
- `find … -delete` xoá ngay trong lúc duyệt, không gom danh sách khổng lồ vào một dòng lệnh (`rm phien-*` sẽ báo `Argument list too long`).
- `df -i` sau đó: còn 1%.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-dia-day-oom/ssh-studio-03.webp" alt="Terminal của SSH Studio: find /du-lieu | cut | uniq -c chỉ ra /du-lieu/tam có 32751 file, find -name 'phien-*' -delete, df -i trở về 1%" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tìm thư mục có nhiều file nhất và xoá file phiên tạm</figcaption></figure>

**Bài học**: thấy `No space left on device` thì luôn chạy **cả** `df -h` và `df -i`.

## 3. Sự cố 3: tiến trình bị giết

Job báo cáo tháng chạy được vài giây rồi biến mất, không để lại thông báo lỗi nào của chính nó. Lab tái hiện bằng một job Python chạy dưới `systemd-run` với trần bộ nhớ 64 MB (như `MemoryMax=` của `bao-gia` ở [LNX-040](/articles/linux-thuc-hanh-dich-vu-systemd)):

```text
an@toinoi-app-01:~$ sudo systemd-run --unit bao-cao-thang -p MemoryMax=64M -p MemorySwapMax=0 /usr/bin/python3 -c "a = [bytearray(10**6) for _ in range(500)]"
Running as unit: bao-cao-thang.service; invocation ID: 1311bd64b5bb4f828bd10e10a0414ce0
an@toinoi-app-01:~$ sleep 3; systemctl is-failed bao-cao-thang
failed
an@toinoi-app-01:~$ systemctl status bao-cao-thang --no-pager | grep -E 'Active|Main PID|oom'
     Active: failed (Result: oom-kill) since Tue 2026-11-24 09:41:52 +07; 2s ago
   Main PID: 9209 (code=killed, signal=KILL)
Nov 24 09:41:52 toinoi-app-01 systemd[1]: bao-cao-thang.service: Failed with result 'oom-kill'.
an@toinoi-app-01:~$ journalctl -u bao-cao-thang --no-pager -o cat | tail -n 4
Started bao-cao-thang.service - /usr/bin/python3 -c "a = [bytearray(10**6) for _ in range(500)]".
bao-cao-thang.service: A process of this unit has been killed by the OOM killer.
bao-cao-thang.service: Main process exited, code=killed, status=9/KILL
bao-cao-thang.service: Failed with result 'oom-kill'.
```

- `systemctl status`: **`Result: oom-kill`**, tiến trình chết bởi `signal=KILL`.
- Journal của unit: **`A process of this unit has been killed by the OOM killer.`**
- Tiến trình không kịp in gì: SIGKILL không bắt được ([LNX-032](/articles/linux-tin-hieu-kill)). Vì vậy log của chính chương trình im lặng, và bằng chứng chỉ nằm ở systemd và kernel.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-su-co-dia-day-oom/ssh-studio-04.webp" alt="Terminal của SSH Studio: systemd-run bao-cao-thang với MemoryMax 64M, is-failed failed, status với Active failed Result oom-kill và Main PID code=killed signal=KILL, journal với A process of this unit has been killed by the OOM killer" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>Result: oom-kill</code>: bị giết vì vượt trần bộ nhớ</figcaption></figure>

**OOM killer** (*out-of-memory killer*) là cơ chế của kernel: khi một cgroup (ở đây là unit có `MemoryMax=`) hay cả máy hết bộ nhớ, kernel chọn một tiến trình để giết, giải phóng bộ nhớ. Trần của từng unit có ích chính ở đây: job báo cáo chết một mình, thay vì ngốn hết RAM rồi kéo `bao-gia` và `sshd` chết theo.

Hướng xử lý, theo thứ tự: job có thật sự cần nhiều bộ nhớ thế không (đọc cả file vào RAM thay vì đọc từng dòng?), rồi mới nâng `MemoryMax=`. Trên máy không đặt trần, tìm dấu vết ở `journalctl -k | grep -i 'out of memory'`.

## 4. Trình tự chung

| Bước | Câu hỏi | Lệnh |
|---|---|---|
| Triệu chứng | Cái gì hỏng, từ lúc nào? | `systemctl --failed`, `journalctl -u UNIT` |
| Bằng chứng | Tài nguyên nào cạn? | `df -h`, `df -i`, `free -h`, `systemctl status` (Result) |
| Nguyên nhân | Ai/cái gì làm cạn? | `du -xh`, đếm file, `ps --sort=-rss`, journal |
| Sửa | Sửa nguyên nhân, không chỉ triệu chứng | Chính sách giữ bản, dọn file tạm, sửa job |
| Kiểm chứng | Đã hết thật chưa? | Chạy lại job, `df`, `is-failed` |

## 5. Lỗi thường gặp

**Xoá file log đang được ghi để «lấy chỗ».** Chỗ không được trả cho tới khi tiến trình đóng file ([LNX-043](/articles/linux-df-du-dung-luong)): `truncate` hoặc khởi động lại tiến trình.

**Chỉ chạy `df -h`.** Hết inode trông giống hệt đầy đĩa.

**Tưởng chương trình «tự thoát».** Không có log của chính nó thường là SIGKILL; tìm `oom-kill` trong systemd.

**Sửa xong không ghi lại.** Lần sau người khác (hay chính bạn) lại mất cả buổi.

## Bài tập

1. Thêm `ExecStartPost=` vào `sao-luu-dem.service` để giữ 3 bản mới nhất sau mỗi lần sao lưu. Vì sao dùng `ExecStartPost` chứ không phải `ExecStartPre`?
2. Tạo lại 20 000 file trong `/du-lieu/tam`, rồi thử `rm /du-lieu/tam/phien-*`. Lỗi gì? Vì sao `find -delete` không gặp?
3. Chạy lại job OOM với `-p MemoryMax=600M`. Nó có chạy xong không? `systemctl show bao-cao-thang -p MemoryPeak` báo bao nhiêu?
4. Viết báo cáo sự cố năm dòng cho sự cố 1: thời điểm, ảnh hưởng, nguyên nhân gốc, cách sửa, việc phòng ngừa.

## Đọc tiếp

[LNX-059 · Sự cố: không SSH được vào server](/articles/linux-su-co-khong-ssh-duoc).
