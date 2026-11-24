---
slug: linux-tin-hieu-kill
title: "Tín hiệu và dừng tiến trình: kill, SIGTERM, SIGKILL, pkill và vì sao kill -9 là lựa chọn cuối"
summary: "Tín hiệu là gì, danh sách kill -l, gửi SIGTERM để một chương trình tự dọn dẹp rồi thoát, vì sao không dừng được tiến trình của người khác nếu không có sudo, một chương trình bỏ qua SIGTERM và cách đọc SigIgn trong /proc, SIGKILL và cái giá của nó (file khoá nằm lại), pkill, killall và dừng theo tên."
category: devops-linux
level: beginner
tags: [linux, lnx-032, signal, kill, sigterm, sigkill, pkill, killall, ssh-studio, toi-noi]
language: vi
---

[LNX-031](/articles/linux-tien-trinh-ps-top) tìm ra thủ phạm: `tinh-lai-bao-cao` của Bình ăn trọn một nhân CPU từ tối qua. Bình đang nghỉ phép, nhắn: «Dừng giúp mình, không cần nữa. Cả cái `xuat-du-lieu` cũng tắt luôn.» An biết lệnh `kill`, và cũng nghe nói «cứ `kill -9` cho chắc». Bài này giải thích vì sao lời khuyên đó sai, và dừng một tiến trình đúng cách là thế nào.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-032 · Phần 4/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-tien-trinh-ps-top">LNX-031</a>. Trạng thái đầu bài: <code>npm run setup -- 32</code> (hai script của Bình đang chạy).</p>
</div>

## 1. Tín hiệu là gì

**Tín hiệu** (*signal*) là một thông báo ngắn kernel giao cho một tiến trình: «hãy dừng», «terminal đã đóng», «có người bấm Ctrl+C». Mỗi tín hiệu có số và tên. Khi nhận, tiến trình có thể làm theo mặc định (thường là thoát), **bắt** tín hiệu để tự xử lý (dọn dẹp rồi thoát, nạp lại cấu hình), hoặc **bỏ qua**. Hai tín hiệu không bắt và không bỏ qua được: `SIGKILL` và `SIGSTOP`.

```text
an@toinoi-app-01:~$ kill -l | head -n 4
 1) SIGHUP       2) SIGINT       3) SIGQUIT      4) SIGILL       5) SIGTRAP
 6) SIGABRT      7) SIGBUS       8) SIGFPE       9) SIGKILL     10) SIGUSR1
11) SIGSEGV     12) SIGUSR2     13) SIGPIPE     14) SIGALRM     15) SIGTERM
16) SIGSTKFLT   17) SIGCHLD     18) SIGCONT     19) SIGSTOP     20) SIGTSTP
an@toinoi-app-01:~$ kill -l 15 9 1
TERM
KILL
HUP
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tin-hieu-kill/ssh-studio-04.webp" alt="Terminal của SSH Studio: kill -l | head -n 4 liệt kê tín hiệu 1 SIGHUP đến 20 SIGTSTP, kill -l 15 9 1 in TERM, KILL, HUP" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · danh sách tín hiệu và số hiệu của chúng</figcaption></figure>

Những tín hiệu bạn sẽ gặp hằng ngày:

| Tín hiệu | Số | Khi nào | Mặc định |
|---|---|---|---|
| `SIGTERM` | 15 | `kill PID` không ghi gì thêm; `systemctl stop` | Thoát, chương trình được phép dọn dẹp trước |
| `SIGINT` | 2 | Bấm Ctrl+C trong terminal | Thoát |
| `SIGHUP` | 1 | Terminal hoặc phiên SSH đóng; dịch vụ thì quy ước «nạp lại cấu hình» | Thoát |
| `SIGKILL` | 9 | `kill -9` | Kernel giết ngay, chương trình không biết gì |
| `SIGSTOP` / `SIGCONT` | 19 / 18 | Tạm dừng / chạy tiếp (Ctrl+Z gửi `SIGTSTP`, gần giống) | Dừng / chạy tiếp |

## 2. SIGTERM: xin dừng một cách lịch sự

```text
an@toinoi-app-01:~$ pgrep -a -u binh
2422 bash /usr/local/bin/tinh-lai-bao-cao
2427 bash /usr/local/bin/xuat-du-lieu
2429 sleep 5
an@toinoi-app-01:~$ kill $(pgrep -u binh -f tinh-lai)
-bash: kill: (2422) - Operation not permitted
an@toinoi-app-01:~$ sudo kill $(pgrep -u binh -f tinh-lai)
an@toinoi-app-01:~$ pgrep -a -u binh
2427 bash /usr/local/bin/xuat-du-lieu
2429 sleep 5
an@toinoi-app-01:~$ cat /tmp/tinh-lai-bao-cao.log
2026-11-24 07:05:04 bat dau, pid 2422, file tam /tmp/tinh-lai-bao-cao.2422.tmp
2026-11-24 07:05:07 nhan SIGTERM: xoa /tmp/tinh-lai-bao-cao.2422.tmp, thoat
an@toinoi-app-01:~$ ls /tmp/tinh-lai-bao-cao.*.tmp
ls: cannot access '/tmp/tinh-lai-bao-cao.*.tmp': No such file or directory
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tin-hieu-kill/ssh-studio-05.webp" alt="Terminal của SSH Studio: pgrep -a -u binh thấy tinh-lai-bao-cao và xuat-du-lieu, kill không sudo báo Operation not permitted, sudo kill hỏi mật khẩu của an rồi gửi SIGTERM" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · dừng tiến trình của người khác cần <code>sudo</code></figcaption></figure>

- **`kill PID`** gửi `SIGTERM`. An không dừng được tiến trình của Bình: `Operation not permitted`. Người dùng thường chỉ gửi tín hiệu được cho tiến trình của chính mình; tiến trình của người khác cần `sudo` (hoặc chính người đó).
- Với `sudo kill`, `tinh-lai-bao-cao` biến mất khỏi danh sách.
- Log của nó cho thấy chuyện gì đã xảy ra: script **bắt** `SIGTERM` bằng `trap` ([LNX-019](/articles/linux-shell-script-an-toan)), xoá file tạm rồi mới thoát. File tạm đã không còn.

Đó là lý do `SIGTERM` là lựa chọn đầu tiên: nó cho chương trình cơ hội ghi nốt dữ liệu, đóng kết nối, xoá file tạm, nhả khoá. Dịch vụ viết tốt (cơ sở dữ liệu, web server, `bao-gia` ở [LNX-036](/articles/linux-systemd-unit-bao-gia)) đều làm vậy.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tin-hieu-kill/ssh-studio-01.webp" alt="Terminal của SSH Studio: pgrep -a -u binh chỉ còn xuat-du-lieu, log của tinh-lai-bao-cao ghi nhận SIGTERM, xoá file tạm rồi thoát, ls không còn file .tmp nào" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · SIGTERM: chương trình kịp dọn file tạm trước khi thoát</figcaption></figure>

## 3. Một chương trình bỏ qua SIGTERM

```text
an@toinoi-app-01:~$ sudo kill -TERM $(pgrep -u binh -f xuat-du-lieu)
an@toinoi-app-01:~$ ps -o pid,stat,etime,cmd -u binh
    PID STAT     ELAPSED CMD
   2427 Ss         00:04 bash /usr/local/bin/xuat-du-lieu
   2429 S          00:04 sleep 5
an@toinoi-app-01:~$ grep -E '^Sig(Ign|Cgt)' /proc/$(pgrep -u binh -f xuat-du-lieu)/status
SigIgn: 0000000000004007
SigCgt: 0000000000010000
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tin-hieu-kill/ssh-studio-06.webp" alt="Terminal của SSH Studio: sudo kill -TERM xuat-du-lieu, ps -o vẫn thấy tiến trình chạy, grep SigIgn và SigCgt trong /proc/PID/status cho mặt nạ tín hiệu bị bỏ qua" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · chương trình bỏ qua SIGTERM: xem <code>SigIgn</code> trong <code>/proc</code></figcaption></figure>

`xuat-du-lieu` vẫn sống sau `SIGTERM`. Lý do nằm trong `/proc/PID/status`:

- **`SigIgn`** là mặt nạ bit các tín hiệu bị **bỏ qua**, mỗi bit một tín hiệu (bit 0 là tín hiệu 1). `0x4007` = bit 0, 1, 2 và 14: `SIGHUP`, `SIGINT`, `SIGQUIT`, `SIGTERM`. Script đã viết `trap '' TERM INT HUP`.
- **`SigCgt`** là các tín hiệu được **bắt** (có hàm xử lý riêng). `0x10000` = bit 16, tức `SIGCHLD`: bash luôn bắt tín hiệu này để biết tiến trình con kết thúc.

Bỏ qua `SIGTERM` hầu như luôn là lỗi của người viết chương trình. Nhưng trên server thật, bạn sẽ gặp cả những chương trình treo, không phản hồi tín hiệu nào.

## 4. SIGKILL: lựa chọn cuối

```text
an@toinoi-app-01:~$ sudo kill -KILL $(pgrep -u binh -f xuat-du-lieu)
an@toinoi-app-01:~$ pgrep -a -u binh || echo "binh khong con tien trinh nao"
binh khong con tien trinh nao
an@toinoi-app-01:~$ ls -l /tmp/xuat-du-lieu.lock
-rw-r--r-- 1 binh binh 5 Nov 24 07:05 /tmp/xuat-du-lieu.lock
an@toinoi-app-01:~$ cat /tmp/xuat-du-lieu.lock
2427
an@toinoi-app-01:~$ sudo rm /tmp/xuat-du-lieu.lock
```

`SIGKILL` không đến tay chương trình: kernel gỡ nó khỏi bộ nhớ ngay. Không `trap` nào chạy, nên **mọi việc dọn dẹp bị bỏ qua**. Ở đây file khoá `/tmp/xuat-du-lieu.lock` (ghi PID đã chết) nằm lại. Lần chạy sau, nếu script kiểm tra file khoá, nó sẽ tưởng có một bản khác đang chạy và từ chối chạy. Với cơ sở dữ liệu hay chương trình đang ghi file, `kill -9` có thể để lại dữ liệu ghi dở.

Trong ảnh dưới (chụp trong SSH Studio), `pgrep` còn thấy một `sleep 5`: đó là tiến trình con mà script đang đợi. `SIGKILL` chỉ giết đúng tiến trình nhận nó; con của nó thành mồ côi và chạy nốt (ở đây vài giây). Với dịch vụ systemd thì không có chuyện này: `systemctl stop` gửi tín hiệu cho cả cgroup của dịch vụ ([LNX-035](/articles/linux-systemd-systemctl)).

Thứ tự đúng khi cần dừng một tiến trình:

1. `kill PID` (SIGTERM), đợi vài giây.
2. Vẫn còn: xem nó đang làm gì (`ps -o stat`, log), thử lại.
3. Chỉ khi chắc chắn nó treo: `kill -9 PID`, rồi tự dọn những gì nó để lại.

`systemctl stop` cũng làm đúng như vậy: gửi `SIGTERM`, đợi (mặc định 90 giây), rồi mới `SIGKILL`.

Một trường hợp ngay cả `SIGKILL` cũng phải đợi: tiến trình ở trạng thái `D` (chờ đĩa hoặc mạng ở mức kernel, ví dụ ổ mạng bị treo). Tín hiệu chỉ được xử lý khi nó thoát khỏi trạng thái đó.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tin-hieu-kill/ssh-studio-02.webp" alt="Terminal của SSH Studio: sudo kill -KILL xuat-du-lieu, pgrep báo binh khong con tien trinh nao, file khoá /tmp/xuat-du-lieu.lock vẫn còn với PID 2606, sudo rm xoá khoá" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · SIGKILL dừng ngay nhưng để lại file khoá</figcaption></figure>

## 5. pkill, killall: dừng theo tên

```text
an@toinoi-app-01:~$ sleep 600 &
an@toinoi-app-01:~$ sleep 601 &
an@toinoi-app-01:~$ jobs -l
[1]-  2522 Running                 sleep 600 &
[2]+  2523 Running                 sleep 601 &
an@toinoi-app-01:~$ pkill -e -f 'sleep 60[01]'
sleep killed (pid 2522)
sleep killed (pid 2523)
an@toinoi-app-01:~$ jobs
an@toinoi-app-01:~$ sleep 602 &
an@toinoi-app-01:~$ killall -v sleep
Killed sleep(2533) with signal 15
```

- **`pkill`** dùng cùng tiêu chí với `pgrep` (`-u`, `-f`…) rồi gửi tín hiệu; `-e` in ra những gì đã dừng. Luôn chạy `pgrep -a` với cùng tiêu chí trước để xem mình sắp dừng những gì.
- **`killall TÊN`** dừng mọi tiến trình có **tên chương trình** khớp chính xác; `-v` in ra. Trên Linux an toàn, nhưng trên vài hệ Unix khác `killall` nghĩa là «giết tất cả», nên nhiều người quen dùng `pkill`.
- **`kill %1`** (theo số job) chỉ dùng được với job của shell hiện tại ([LNX-034](/articles/linux-job-nen-nohup-tmux)).

Tín hiệu khác gửi bằng tên: `kill -HUP PID`, `pkill -USR1 -f tên`. Với dịch vụ systemd, dùng `systemctl kill -s HUP tên` hoặc `systemctl reload tên` ([LNX-036](/articles/linux-systemd-unit-bao-gia)).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tin-hieu-kill/ssh-studio-03.webp" alt="Terminal của SSH Studio: chạy sleep 600 và sleep 601 nền, jobs -l, pkill -e -f dừng cả hai và in sleep killed, sleep 602 nền rồi killall -v sleep in Killed sleep with signal 15" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>pkill</code> và <code>killall</code></figcaption></figure>

## 6. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Xin dừng | `kill PID` (= `kill -TERM PID`) |
| Dừng tiến trình của người khác | `sudo kill PID` |
| Dừng theo tiêu chí | `pkill -u binh -f tinh-lai` (xem trước bằng `pgrep -a`) |
| Dừng theo tên chương trình | `killall -v tên` |
| Nạp lại cấu hình (quy ước) | `kill -HUP PID` |
| Ép dừng | `kill -KILL PID` (= `kill -9`), lựa chọn cuối |
| Xem tín hiệu bị bỏ qua / bắt | `grep Sig /proc/PID/status` |
| Trong `top` | `k`, nhập PID, nhập tín hiệu |

## 7. Lỗi thường gặp

**`kill -9` theo thói quen.** Mất dọn dẹp, để lại file khoá, dữ liệu ghi dở. Bắt đầu bằng `kill` thường.

**`pkill -f` khớp quá rộng.** `pkill -f bao` dừng cả `bao-gia` lẫn `bao-cao`. Xem `pgrep -a -f` trước.

**Dừng dịch vụ systemd bằng `kill`.** systemd thấy tiến trình chết và (với `Restart=`) khởi động lại ngay. Dùng `systemctl stop` ([LNX-035](/articles/linux-systemd-systemctl)).

**`kill` báo `No such process`.** Tiến trình đã thoát, hoặc PID gõ sai. PID được tái sử dụng: đừng giữ PID trong file quá lâu rồi `kill` mù.

## Bài tập

1. Chạy `sleep 1000 &`, gửi `kill -STOP %1`, xem `ps -o stat -p $!`, rồi `kill -CONT %1`. Trạng thái đổi thế nào?
2. Viết một script `trap 'echo nhan HUP' HUP; while :; do sleep 1; done`, chạy nền, gửi `kill -HUP`. Script có thoát không? `SigCgt` của nó là gì?
3. Tính `SigIgn` (dạng hex) của một tiến trình bỏ qua `SIGINT` và `SIGTERM`. Kiểm tra bằng `trap '' INT TERM` trong một script.
4. Vì sao không nên dùng `kill -9 -1` khi đang đăng nhập bằng `root`? (Đọc `man 2 kill` về PID `-1`, nhưng **đừng chạy thử**.)

## Đọc tiếp

[LNX-033 · Cài phần mềm bằng apt](/articles/linux-apt-cai-phan-mem): cài `jq`, `tmux` và những công cụ còn thiếu, tìm gói chứa một file, ghim phiên bản.
