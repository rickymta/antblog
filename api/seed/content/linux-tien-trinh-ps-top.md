---
slug: linux-tien-trinh-ps-top
title: "Tiến trình trên Linux: PID, ps, top, cây tiến trình và tìm ra ai đang ăn CPU"
summary: "Mở đầu Phần 4 của series Linux qua SSH Studio: tiến trình là gì, PID và PPID, /proc, đọc ps aux và ps -ef --forest, tìm tiến trình theo người dùng và theo tên bằng pgrep, đọc %CPU, RSS, trạng thái R/S, dùng top tương tác và top -b, đọc load average, và vì sao free trong container thấy RAM của cả máy."
category: devops-linux
level: beginner
tags: [linux, lnx-031, process, ps, top, pgrep, pid, proc, ssh-studio, toi-noi]
language: vi
---

Sáng thứ Ba, dịch vụ báo giá trả lời chậm hẳn. Không ai sửa gì trên `toinoi-app-01` từ hôm qua. Bình nhắn: «Tối qua mình có chạy tay một script tính lại báo cáo tháng, chắc xong rồi.» An đăng nhập để xem thực sự trên server đang có những gì chạy.

Phần 4 của series nói về những thứ **đang chạy** trên server: tiến trình, tín hiệu, cài phần mềm, job chạy lâu, và cuối cùng là đưa `bao-gia` thành một dịch vụ systemd đúng nghĩa. Bài đầu tiên trả lời câu hỏi cơ bản nhất: trên server đang có gì chạy, của ai, tốn bao nhiêu.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-031 · Phần 4/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-nguoi-dung-nhom-quyen-file">LNX-007</a> (người dùng, UID), <a href="/articles/linux-chuyen-huong-pipe">LNX-008</a> (pipe). Trạng thái đầu bài: <code>npm run setup -- 31</code> trong <code>examples/toi-noi-linux</code>, rồi kết nối bằng SSH Studio (từ Phần 3, server cần key và mã OTP). Lab đưa bản 1.5.0 của <code>bao-gia</code> lên server và cho Bình chạy hai script, để bạn có thứ mà tìm.</p>
</div>

## 1. Tiến trình là gì

Một **tiến trình** (*process*) là một chương trình đang chạy: mã lệnh đã nạp vào bộ nhớ, cùng với những gì kernel giữ cho nó: bộ nhớ riêng, file đang mở, thư mục hiện tại, biến môi trường, và chạy dưới danh nghĩa người dùng nào. Cùng một file `/usr/bin/bash` có thể là hàng chục tiến trình khác nhau.

Mỗi tiến trình có:

| Thuộc tính | Ý nghĩa |
|---|---|
| **PID** | Số hiệu duy nhất lúc nó đang sống |
| **PPID** | PID của tiến trình đã tạo ra nó (cha) |
| **UID** | Chạy dưới danh nghĩa người dùng nào, quyết định nó được đọc ghi gì |
| **Trạng thái** | Đang chạy, đang ngủ chờ việc, đã dừng… |

Shell bạn đang gõ cũng là một tiến trình:

```text
an@toinoi-app-01:~$ echo $$
2398
an@toinoi-app-01:~$ ps
    PID TTY          TIME CMD
   2398 ?        00:00:00 bash
   2437 ?        00:00:00 ps
an@toinoi-app-01:~$ ps -o pid,ppid,user,stat,etime,cmd -p $$
    PID    PPID USER     STAT     ELAPSED CMD
   2398    2392 an       Ss         00:02 bash /run/lab.sh
an@toinoi-app-01:~$ ls /proc/$$ | head -n 8
arch_status
attr
auxv
cgroup
clear_refs
cmdline
comm
coredump_filter
an@toinoi-app-01:~$ readlink /proc/$$/exe
/usr/bin/bash
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tien-trinh-ps-top/ssh-studio-05.webp" alt="Terminal của SSH Studio: echo $$ in PID 2562 của shell, ps chỉ có bash và ps, ps -o cho PPID 2561 và thời gian chạy, ls /proc/$$ liệt kê các file của tiến trình, readlink /proc/$$/exe trỏ tới /usr/bin/bash" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · shell của bạn cũng là một tiến trình, có thư mục riêng trong <code>/proc</code></figcaption></figure>

- **`$$`** là PID của chính shell.
- **`ps`** không tham số liệt kê tiến trình thuộc terminal hiện tại: shell và chính `ps`.
- **`ps -o …`** chọn cột: `PPID` là cha, `STAT` là trạng thái (`S` đang ngủ, `s` là trưởng phiên), `ELAPSED` là đã chạy bao lâu.
- Mỗi tiến trình có một thư mục **`/proc/PID`**: kernel trình bày thông tin của tiến trình dưới dạng file. `exe` trỏ tới chương trình đang chạy, `cwd` tới thư mục hiện tại, `status` là tóm tắt dễ đọc. `ps` và `top` đọc từ đây.

Bản ghi của lab chạy kịch bản không qua SSH nên cột `TTY` là `?` và cha của shell là `su`; trong SSH Studio bạn sẽ thấy `pts/0` và cha là `sshd`, như ảnh dưới.

## 2. Mọi tiến trình trên server

```text
an@toinoi-app-01:~$ ps aux | head -n 6
USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
root           1  0.1  0.0  20936 12424 ?        Ss   07:03   0:00 /sbin/init
root          22  0.1  0.0  59036 23976 ?        S<s  07:03   0:00 /usr/lib/systemd/systemd-journald
message+      63  0.0  0.0   8152  4604 ?        Ss   07:03   0:00 @dbus-daemon --system --address=systemd: --nofork --nopidfile --systemd-activation --syslog-only
root        1832  0.3  0.0  12036  8264 ?        Ss   07:04   0:00 sshd: /usr/sbin/sshd -D [listener] 0 of 10-100 startups
root        1891  0.0  0.0   4248  2784 ?        Ss   07:04   0:00 /usr/sbin/cron -f -P
an@toinoi-app-01:~$ ps aux | wc -l
13
an@toinoi-app-01:~$ ps -ef --forest | grep -v ' \[' | cut -c1-100 | head -n 25
UID          PID    PPID  C STIME TTY          TIME CMD
root        2392       0  0 07:05 pts/0    00:00:00 su - an -c bash /run/lab.sh
an          2398    2392  0 07:05 ?        00:00:00  \_ bash /run/lab.sh
an          2446    2398  0 07:05 ?        00:00:00      \_ ps -ef --forest
an          2447    2398  0 07:05 ?        00:00:00      \_ grep --color=auto -v  \[
an          2448    2398  0 07:05 ?        00:00:00      \_ cut -c1-100
an          2449    2398  0 07:05 ?        00:00:00      \_ head -n 25
root           1       0  0 07:03 ?        00:00:00 /sbin/init
root          22       1  0 07:03 ?        00:00:00 /usr/lib/systemd/systemd-journald
message+      63       1  0 07:03 ?        00:00:00 @dbus-daemon --system --address=systemd: --nofor
root        1891       1  0 07:04 ?        00:00:00 /usr/sbin/cron -f -P
binh        2422       1 99 07:05 ?        00:00:02 bash /usr/local/bin/tinh-lai-bao-cao
binh        2427       1  0 07:05 ?        00:00:00 bash /usr/local/bin/xuat-du-lieu
binh        2429    2427  0 07:05 ?        00:00:00  \_ sleep 5
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tien-trinh-ps-top/ssh-studio-06.webp" alt="Terminal của SSH Studio: ps aux | head -n 6 với các cột USER, PID, %CPU, %MEM, VSZ, RSS, STAT, START, TIME, COMMAND; ps aux | wc -l đếm 14 dòng" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>ps aux</code>: mọi tiến trình trên server</figcaption></figure>

**`ps aux`** là cách xem mọi tiến trình quen thuộc nhất (`a`: của mọi người dùng, `u`: định dạng theo người dùng, `x`: cả tiến trình không gắn terminal):

| Cột | Ý nghĩa |
|---|---|
| `USER`, `PID` | Của ai, số hiệu |
| `%CPU` | Phần trăm một nhân CPU, trung bình từ lúc tiến trình bắt đầu |
| `%MEM`, `RSS` | Bộ nhớ thật đang chiếm (RSS tính bằng KB) |
| `VSZ` | Bộ nhớ ảo đã xin, thường lớn hơn nhiều so với RSS, ít khi cần quan tâm |
| `STAT` | Trạng thái: `R` đang chạy, `S` ngủ chờ việc, `D` chờ đĩa (không ngắt được), `T` dừng, `Z` zombie |
| `START`, `TIME` | Bắt đầu lúc nào, đã dùng bao nhiêu thời gian CPU |
| `COMMAND` | Lệnh, kèm tham số |

Server lab chỉ có hơn chục tiến trình; server thật thường có vài trăm. **`ps -ef --forest`** vẽ **cây tiến trình**: mỗi tiến trình nằm dưới cha của nó. PID 1 (`/sbin/init`, tức systemd) là gốc của mọi tiến trình dịch vụ. Hai script của Bình có cha là PID 1: người chạy chúng đã thoát, và tiến trình «mồ côi» được PID 1 nhận nuôi.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tien-trinh-ps-top/ssh-studio-01.webp" alt="Terminal của SSH Studio: ps -ef --forest cho cây tiến trình: /sbin/init là PID 1, sshd: an@pts/0 sinh ra -bash, bash sinh ra ps, grep, cut, head; hai tiến trình của binh tinh-lai-bao-cao và xuat-du-lieu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cây tiến trình: mỗi tiến trình có một tiến trình cha</figcaption></figure>

## 3. Ai đang ăn CPU?

```text
an@toinoi-app-01:~$ ps aux --sort=-%cpu | head -n 4
USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
binh        2422  100  0.0   4764  3440 ?        Rs   07:05   0:02 bash /usr/local/bin/tinh-lai-bao-cao
an          2398  0.4  0.0   6204  4936 ?        Ss   07:05   0:00 bash /run/lab.sh
root        1832  0.3  0.0  12036  8264 ?        Ss   07:04   0:00 sshd: /usr/sbin/sshd -D [listener] 0 of 10-100 startups
an@toinoi-app-01:~$ pgrep -a -u binh
2422 bash /usr/local/bin/tinh-lai-bao-cao
2427 bash /usr/local/bin/xuat-du-lieu
2429 sleep 5
an@toinoi-app-01:~$ ps -o pid,user,%cpu,%mem,rss,etime,stat,cmd -u binh
    PID USER     %CPU %MEM   RSS     ELAPSED STAT CMD
   2422 binh     99.5  0.0  3440       00:02 Rs   bash /usr/local/bin/tinh-lai-bao-cao
   2427 binh      0.0  0.0  3520       00:02 Ss   bash /usr/local/bin/xuat-du-lieu
   2429 binh      0.0  0.0  1868       00:02 S    sleep 5
an@toinoi-app-01:~$ grep -E '^(Name|State|PPid|Uid|VmRSS|Threads)' /proc/$(pgrep -u binh -f tinh-lai)/status
Name:   bash
State:  R (running)
PPid:   1
Uid:    1001    1001    1001    1001
VmRSS:      3440 kB
Threads:        1
```

- **`--sort=-%cpu`** sắp giảm dần theo CPU: `tinh-lai-bao-cao` của Bình chiếm trọn **100%** một nhân, trạng thái `R` (đang chạy liên tục). Đây là thủ phạm làm `bao-gia` chậm.
- **`pgrep`** tìm PID theo tiêu chí: `-u binh` của Bình, `-f` khớp cả dòng lệnh (không chỉ tên chương trình, ở đây là `bash`), `-a` in kèm dòng lệnh. `pgrep` hợp để dùng trong lệnh khác: `$(pgrep -u binh -f tinh-lai)`.
- **`ps -u binh -o …`** chỉ hiện tiến trình của Bình với đúng các cột cần.
- **`/proc/PID/status`**: `PPid: 1` xác nhận nó mồ côi, `Uid: 1001` là Bình, `VmRSS` là bộ nhớ thật.

`xuat-du-lieu` thì `0.0` CPU và trạng thái `S`: nó ngủ trong `sleep 5`. Một tiến trình ngủ không tốn CPU, nhưng vẫn giữ bộ nhớ và file đang mở.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tien-trinh-ps-top/ssh-studio-02.webp" alt="Terminal của SSH Studio: ps aux --sort=-%cpu với tinh-lai-bao-cao của binh ở đầu 100% CPU trạng thái Rs, pgrep -a -u binh liệt kê ba tiến trình, ps -o pid,user,%cpu,%mem,rss,etime,stat,cmd -u binh, /proc/PID/status với Name bash, State R running, PPid 1, Uid 1001" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tìm ra tiến trình đang ăn CPU</figcaption></figure>

## 4. top: nhìn liên tục

`ps` là ảnh chụp một thời điểm. **`top`** cập nhật mỗi vài giây, sắp theo CPU:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tien-trinh-ps-top/ssh-studio-03.webp" alt="Lệnh top tương tác trong terminal của SSH Studio: dòng tóm tắt uptime, load average, Tasks, %Cpu(s), MiB Mem và Swap, bảng tiến trình với tiến trình bash của binh (tinh-lai-bao-cao) đứng đầu ở 100% CPU" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>top</code> tương tác: cột COMMAND chỉ hiện tên chương trình, bấm <code>c</code> để thấy cả dòng lệnh</figcaption></figure>

Trong `top`, vài phím hay dùng:

| Phím | Tác dụng |
|---|---|
| `P` / `M` | Sắp theo CPU / theo bộ nhớ |
| `u` rồi tên | Chỉ hiện tiến trình của một người dùng |
| `c` | Hiện đủ dòng lệnh thay vì tên chương trình |
| `1` | Tách dòng CPU theo từng nhân |
| `k` rồi PID | Gửi tín hiệu dừng cho tiến trình ([LNX-032](/articles/linux-tin-hieu-kill)) |
| `q` | Thoát |

Trong script, hoặc khi muốn lưu lại kết quả, dùng **chế độ batch**:

```text
an@toinoi-app-01:~$ top -b -n 1 -o %CPU | head -n 12
top - 07:05:06 up 6 min,  0 user,  load average: 0.42, 0.43, 0.23
Tasks:  12 total,   2 running,  10 sleeping,   0 stopped,   0 zombie
%Cpu(s):  6.1 us,  0.0 sy,  0.0 ni, 93.3 id,  0.0 wa,  0.0 hi,  0.6 si,  0.0 st
MiB Mem :  32045.9 total,  28575.5 free,   1623.5 used,   2218.1 buff/cache
MiB Swap:   8192.0 total,   8192.0 free,      0.0 used.  30422.4 avail Mem

    PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM     TIME+ COMMAND
   2422 binh      20   0    4764   3440   3156 R  90.9   0.0   0:02.24 bash
      1 root      20   0   20936  12424   9752 S   0.0   0.0   0:00.17 systemd
     22 root      19  -1   59036  23976  22488 S   0.0   0.1   0:00.15 systemd+
     63 message+  20   0    8152   4604   4128 S   0.0   0.0   0:00.00 dbus-da+
   1832 root      20   0   12036   8264   7120 S   0.0   0.0   0:00.03 sshd
an@toinoi-app-01:~$ uptime
 07:05:06 up 6 min,  0 user,  load average: 0.42, 0.43, 0.23
an@toinoi-app-01:~$ nproc
16
an@toinoi-app-01:~$ free -h
               total        used        free      shared  buff/cache   available
Mem:            31Gi       1.6Gi        27Gi        26Mi       2.2Gi        29Gi
Swap:          8.0Gi          0B       8.0Gi
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tien-trinh-ps-top/ssh-studio-07.webp" alt="Terminal của SSH Studio: top -b -n 1 -o %CPU in phần đầu và các tiến trình, bash của binh 100% CPU; uptime cho load average 0.68, 0.51, 0.31" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>top</code> chế độ batch, chụp một lần để lưu hoặc so sánh</figcaption></figure>

- Dòng đầu của `top` giống **`uptime`**: giờ, đã chạy bao lâu, và **load average** trong 1, 5, 15 phút. Load là số tiến trình trung bình đang chạy hoặc chờ CPU (và chờ đĩa). So với **`nproc`** (số nhân): load nhỏ hơn số nhân là CPU còn rảnh.
- Dòng `%Cpu(s)` tính trên **mọi** nhân: một tiến trình ăn 100% một nhân trên máy 16 nhân chỉ hiện khoảng 6% ở đây. Cột `%CPU` của từng tiến trình thì tính theo một nhân.
- **`free -h`**: `available` là bộ nhớ còn dùng được (kể cả phần cache có thể nhả ra), con số đáng nhìn hơn `free`.

Lab chạy trong Docker, nên `top`, `free`, `nproc` thấy CPU và RAM của **cả máy chạy Docker** (ở đây 16 nhân, 31 GB), không phải giới hạn 512 MB của container. Trên máy ảo hay server thật, các con số này là của chính server.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tien-trinh-ps-top/ssh-studio-04.webp" alt="Terminal của SSH Studio: nproc in 16, free -h cho Mem total 31Gi, used 1.5Gi, available 29Gi, Swap 8.0Gi chưa dùng" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · số nhân CPU và bộ nhớ để đọc đúng load average</figcaption></figure>

## 5. Đối chiếu thao tác

| Câu hỏi | Lệnh |
|---|---|
| PID của shell này | `echo $$` |
| Mọi tiến trình | `ps aux` |
| Cây tiến trình | `ps -ef --forest`, `pstree -p` |
| Tiến trình của một người | `ps -u binh -o pid,%cpu,cmd`, `pgrep -a -u binh` |
| Tìm theo dòng lệnh | `pgrep -a -f tinh-lai` |
| Ăn CPU / RAM nhiều nhất | `ps aux --sort=-%cpu`, `--sort=-rss` |
| Nhìn liên tục | `top` (hoặc `htop`, cài ở [LNX-033](/articles/linux-apt-cai-phan-mem)) |
| Chi tiết một tiến trình | `/proc/PID/status`, `ls -l /proc/PID/cwd` |

## 6. Lỗi thường gặp

**`ps aux | grep tên` thấy cả chính `grep`.** Dòng lệnh của `grep` chứa chữ bạn tìm. Dùng `pgrep -a -f tên`.

**Đọc `%CPU` của `ps` như số tức thời.** Đó là trung bình từ lúc tiến trình bắt đầu. Muốn biết lúc này, xem `top`.

**Hoảng vì `VSZ` lớn.** Bộ nhớ ảo đã xin không phải bộ nhớ đang dùng; nhìn `RSS`.

**Kết luận «máy thiếu RAM» vì cột `free` nhỏ.** Linux dùng RAM rảnh làm cache đĩa. Nhìn `available`.

**Tin con số của `free`, `nproc` trong container.** Chúng thường là của máy chủ; giới hạn thật của container nằm trong cgroup.

## Bài tập

1. Dùng `ps -o pid,ppid,cmd` và đi ngược theo `PPID` từ shell của bạn lên tới PID 1. Bạn đi qua những tiến trình nào?
2. `ls -l /proc/$$/fd`. Các số 0, 1, 2 trỏ tới đâu? Liên hệ với stdin, stdout, stderr ([LNX-008](/articles/linux-chuyen-huong-pipe)).
3. Trong `top`, bấm `u`, gõ `binh`. Rồi bấm `c`. Dòng nào thay đổi?
4. Chạy `sleep 300 &` rồi `ps -o pid,stat,cmd -p $!`. Trạng thái là gì? `$!` là gì?

## Đọc tiếp

[LNX-032 · Tín hiệu và dừng tiến trình](/articles/linux-tin-hieu-kill): dừng `tinh-lai-bao-cao` cho đúng cách, và vì sao `kill -9` là lựa chọn cuối.
