---
slug: linux-thuc-hanh-mang-giam-sat
title: "Thực hành: bàn giao mạng và giám sát cho server, tường lửa tối thiểu, HTTPS nội bộ và báo cáo tài nguyên định kỳ"
summary: "Bài thực hành cuối Phần 5: kiểm chứng tường lửa chỉ mở SSH (có giới hạn) và HTTPS cho mạng nội bộ bằng kết nối thật từ toinoi-app-02, viết script báo cáo tài nguyên (tải, bộ nhớ dùng được, dung lượng hai ổ, số cổng nghe, trạng thái dịch vụ), chạy nó 15 phút một lần bằng systemd timer ghi vào log, bảng kiểm tra bàn giao, và bộ chấm 20 mục."
category: devops-linux
level: advanced
tags: [linux, lnx-050, firewall, monitoring, systemd-timer, nginx, thuc-hanh, ssh-studio, toi-noi]
language: vi
---

Phần 5 đã dựng xong từng mảnh: tường lửa, nginx có TLS, ổ dữ liệu riêng, bản sao lưu ở server thứ hai. Bình chốt việc bàn giao cho nhóm vận hành bằng hai câu hỏi: «Từ ngoài nhìn vào, server mở những gì?» và «Khi có chuyện, lấy đâu ra số liệu của mấy tiếng trước?» Bài này trả lời câu đầu bằng kết nối thật từ máy khác, câu sau bằng một báo cáo tài nguyên chạy định kỳ.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-050 · Phần 5/6 · Nâng cao · Thực hành</span>
<p>Trạng thái đầu bài: <code>npm run setup -- 50</code>. Thử tự làm theo yêu cầu trước khi đọc lời giải. Cuối bài, <code>npm run kiem-tra:p05</code> chấm toàn bộ Phần 5, có kết nối thật từ <code>toinoi-app-02</code>.</p>
</div>

## Yêu cầu

1. Từ mạng nội bộ chỉ tới được SSH (có giới hạn tần suất) và HTTPS; HTTP 80 và cổng 8080 của `bao-gia` không tới được.
2. Một script `/usr/local/bin/bao-cao-tai-nguyen` in ảnh chụp tài nguyên: tải so với số nhân, bộ nhớ dùng được, dung lượng `/` và `/du-lieu`, số cổng đang nghe, trạng thái `bao-gia` và `nginx`.
3. Script chạy 15 phút một lần bằng systemd timer, chạy bằng `baogia`, nối kết quả vào `/var/log/toinoi/tai-nguyen.log`.
4. Bảng kiểm tra: ổ dữ liệu gắn theo UUID, nginx hợp lệ, HTTPS trả lời, bản sao lưu có trên app02.

## 1. Tường lửa: nhìn từ ngoài vào

```text
an@toinoi-app-01:~$ sudo ufw status verbose
Status: active
Logging: on (low)
Default: deny (incoming), allow (outgoing), deny (routed)
New profiles: skip

To                         Action      From
--                         ------      ----
22/tcp (OpenSSH)           LIMIT IN    Anywhere
443/tcp                    ALLOW IN    10.20.0.0/24               # https bao-gia noi bo
22/tcp (OpenSSH (v6))      LIMIT IN    Anywhere (v6)

an@toinoi-app-01:~$ sudo ss -tlnpH | grep -v 127.0.0 | awk '{ split($6, p, "\""); print $4, p[2] }'
0.0.0.0:443 nginx
0.0.0.0:80 nginx
0.0.0.0:22 sshd
[::]:22 sshd
an@toinoi-app-02:~$ nc -zv -w 3 toinoi-app-01 22
Connection to toinoi-app-01 (10.20.0.11) 22 port [tcp/ssh] succeeded!
an@toinoi-app-02:~$ nc -zv -w 3 toinoi-app-01 443
Connection to toinoi-app-01 (10.20.0.11) 443 port [tcp/https] succeeded!
an@toinoi-app-02:~$ nc -zv -w 3 toinoi-app-01 80
nc: connect to toinoi-app-01 (10.20.0.11) port 80 (tcp) timed out: Operation now in progress
```

- `ufw` chỉ còn hai quy tắc: SSH có `LIMIT`, HTTPS cho `10.20.0.0/24`.
- `ss` (bỏ các dòng loopback, rút gọn bằng `awk` như ở [LNX-049](/articles/linux-nginx-reverse-proxy-tls)) cho thấy những gì nghe ở địa chỉ ngoài: `sshd` (22), `nginx` (80, 443). `bao-gia` không có trong danh sách: nó chỉ nghe ở `127.0.0.1`.
- Từ `toinoi-app-02`: 22 và 443 mở. **80 bị chặn (timeout)** dù nginx đang nghe ở đó: tường lửa làm đúng việc của nó. Hai lớp: dịch vụ nghe ở đâu ([LNX-041](/articles/linux-ip-cong-ss)), và tường lửa cho ai vào ([LNX-045](/articles/linux-tuong-lua-ufw-nftables)).

Kiểm tra «từ ngoài nhìn vào» luôn phải làm **từ một máy khác**: từ chính server, loopback đi vòng qua tường lửa và mọi thứ trông như mở.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-mang-giam-sat/ssh-studio-01.webp" alt="Terminal của SSH Studio: sudo ufw status verbose với 22/tcp OpenSSH LIMIT IN Anywhere và 443/tcp ALLOW IN 10.20.0.0/24, sudo ss -tlnpH bỏ loopback, rút gọn bằng awk, thấy nginx ở 443 và 80, sshd ở 22" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tường lửa và các cổng nghe ra ngoài</figcaption></figure>

## 2. Báo cáo tài nguyên định kỳ

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-mang-giam-sat/ssh-studio-02.webp" alt="Vim mở /usr/local/bin/bao-cao-tai-nguyen qua sudoedit trong SSH Studio: script bash với set -euo pipefail, in thời điểm và hostname, tải từ /proc/loadavg so với nproc, bộ nhớ dùng được từ free -m, dung lượng / và /du-lieu từ df --output, số cổng đang nghe, trạng thái bao-gia và nginx" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · script <code>bao-cao-tai-nguyen</code></figcaption></figure>

```bash
#!/usr/bin/env bash
# Ảnh chụp tài nguyên của toinoi-app-01 (LNX-050): một khối mỗi lần chạy, timer gọi 15 phút một lần.
set -euo pipefail
echo "=== $(date '+%F %T') $(hostname)"
echo "tai: $(cut -d' ' -f1-3 /proc/loadavg) / $(nproc) nhan"
free -m | awk '/^Mem:/ { printf "bo nho: con dung duoc %d/%d MB\n", $7, $2 }'
df -h --output=target,pcent,avail / /du-lieu 2>/dev/null | tail -n +2 | awk '{ printf "dia %s: dung %s, con %s\n", $1, $2, $3 }'
echo "cong dang nghe: $(ss -tlnH | wc -l)"
echo "bao-gia: $(systemctl is-active bao-gia), nginx: $(systemctl is-active nginx)"
```

Mỗi dòng là kiến thức của một bài: `/proc/loadavg` và `nproc` ([LNX-031](/articles/linux-tien-trinh-ps-top)), cột 7 của `free` là `available` ([LNX-048](/articles/linux-bo-nho-tai-he-thong)), `df --output` chọn cột ([LNX-043](/articles/linux-df-du-dung-luong)), `ss -tlnH` bỏ dòng tiêu đề, `systemctl is-active` ([LNX-035](/articles/linux-systemd-systemctl)), `awk` để định dạng ([LNX-015](/articles/linux-awk-xu-ly-log)).

Service và timer, theo mẫu của [LNX-038](/articles/linux-cron-systemd-timer):

```ini
# /etc/systemd/system/bao-cao-tai-nguyen.service
[Service]
Type=oneshot
User=baogia
Group=toinoi
ExecStart=/usr/local/bin/bao-cao-tai-nguyen
StandardOutput=append:/var/log/toinoi/tai-nguyen.log

# /etc/systemd/system/bao-cao-tai-nguyen.timer
[Timer]
OnCalendar=*:0/15
Persistent=true
```

- **`StandardOutput=append:`** cho systemd nối stdout vào file, không cần chuyển hướng trong script.
- **`OnCalendar=*:0/15`**: mọi giờ, từ phút 0, cứ 15 phút (systemd chuẩn hoá thành `*-*-* *:00/15:00`).
- Script chỉ đọc, nên chạy bằng tài khoản dịch vụ `baogia`, không cần `root`.

```text
an@toinoi-app-01:~$ sudo chmod 755 /usr/local/bin/bao-cao-tai-nguyen
an@toinoi-app-01:~$ bao-cao-tai-nguyen
=== 2026-11-24 08:13:26 toinoi-app-01
tai: 0.63 0.50 0.35 / 16 nhan
bo nho: con dung duoc 30404/32045 MB
dia /: dung 1%, con 947G
dia /du-lieu: dung 1%, con 452M
cong dang nghe: 6
bao-gia: active, nginx: active
an@toinoi-app-01:~$ sudoedit /etc/systemd/system/bao-cao-tai-nguyen.service
an@toinoi-app-01:~$ sudoedit /etc/systemd/system/bao-cao-tai-nguyen.timer
an@toinoi-app-01:~$ sudo systemctl daemon-reload
an@toinoi-app-01:~$ sudo systemctl enable --now bao-cao-tai-nguyen.timer
Created symlink /etc/systemd/system/timers.target.wants/bao-cao-tai-nguyen.timer → /etc/systemd/system/bao-cao-tai-nguyen.timer.
an@toinoi-app-01:~$ sudo systemctl start bao-cao-tai-nguyen.service
an@toinoi-app-01:~$ cat /var/log/toinoi/tai-nguyen.log
=== 2026-11-24 08:13:26 toinoi-app-01
tai: 0.63 0.50 0.35 / 16 nhan
bo nho: con dung duoc 30401/32045 MB
dia /: dung 1%, con 947G
dia /du-lieu: dung 1%, con 452M
cong dang nghe: 6
bao-gia: active, nginx: active
an@toinoi-app-01:~$ systemctl list-timers bao-cao-tai-nguyen.timer --no-pager | head -n 2
NEXT                            LEFT LAST PASSED UNIT                     ACTIVATES
Tue 2026-11-24 08:15:00 +07 1min 33s -         - bao-cao-tai-nguyen.timer bao-cao-tai-nguyen.service
```

Log này là thứ nhóm vận hành mở khi có sự cố: «lúc 2 giờ sáng tải và bộ nhớ thế nào?» Nó cũng cần được xoay vòng: bài tập 2.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-mang-giam-sat/ssh-studio-03.webp" alt="Terminal của SSH Studio: chạy bao-cao-tai-nguyen in khối === với tải, bộ nhớ dùng được, dung lượng / và /du-lieu, số cổng nghe, bao-gia active và nginx active; daemon-reload, enable --now timer, start service, cat /var/log/toinoi/tai-nguyen.log có khối do service vừa ghi, list-timers với lần chạy tới ở phút chia hết cho 15" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · báo cáo tài nguyên chạy bằng timer</figcaption></figure>

## 3. Bảng kiểm tra bàn giao

```text
an@toinoi-app-01:~$ findmnt -o TARGET,SOURCE,FSTYPE /du-lieu
TARGET   SOURCE     FSTYPE
/du-lieu /dev/loop2 ext4
an@toinoi-app-01:~$ grep du-lieu /etc/fstab
UUID=eb774cd9-4027-4678-9eeb-047a2bfe097a /du-lieu ext4 defaults,nofail 0 2
an@toinoi-app-01:~$ sudo nginx -t 2>&1 | tail -n 1
nginx: configuration file /etc/nginx/nginx.conf test is successful
an@toinoi-app-01:~$ curl -s --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 4281, "moi_truong": "production", "so_dich_vu": 4}
an@toinoi-app-01:~$ ssh app02 'ls sao-luu/releases'
1.4.2
1.5.0
1.6.0
an@toinoi-app-01:~$ systemctl is-active bao-gia nginx ufw bao-cao-tai-nguyen.timer
active
active
active
active
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-mang-giam-sat/ssh-studio-04.webp" alt="Terminal của SSH Studio: findmnt /du-lieu ext4, dòng UUID trong fstab, nginx -t successful, curl --cacert HTTPS trả JSON của bao-gia, ssh app02 liệt kê ba release đã sao lưu, systemctl is-active của bao-gia, nginx, ufw và timer đều active" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · bảng kiểm tra bàn giao</figcaption></figure>

## 4. Chấm cả Phần 5

```bash
npm run kiem-tra:p05
```

| Nhóm | Bộ chấm kiểm tra |
|---|---|
| Tường lửa | `ufw` bật, mặc định chặn kết nối vào, SSH `LIMIT`, 443 chỉ cho `10.20.0.0/24`; **từ app02**: 22 và 443 mở, 80 và 8080 không tới được |
| Dịch vụ | `bao-gia` chỉ nghe `127.0.0.1:8080`; `nginx -t` hợp lệ; HTTPS từ app02 trả `/suc-khoe`; chứng chỉ có SAN đúng tên; khoá riêng chỉ root đọc; HTTP chuyển 301 sang HTTPS |
| Ổ dữ liệu | `/du-lieu` là `ext4`; fstab theo UUID có `nofail` |
| Báo cáo | Timer bật, 15 phút một lần; chạy service thì log có thêm một khối đúng định dạng |
| Sao lưu | app02 có các release (rsync qua SSH) |

Tất cả `PASS` (20 mục) là xong Phần 5.

## 5. Lỗi thường gặp

**Kiểm tra tường lửa từ chính server.** Loopback không qua tường lửa; luôn thử từ một máy khác.

**Báo cáo chạy bằng `root` cho tiện.** Script chỉ đọc thì chạy bằng tài khoản ít quyền.

**Log báo cáo không xoay vòng.** 96 khối mỗi ngày; thêm vào logrotate.

**Báo cáo mà không ai đọc.** Số liệu có giá trị khi có ngưỡng và cảnh báo (Phần 6), hoặc ít nhất khi nhóm biết nó ở đâu.

## Bài tập

1. Thêm vào báo cáo một dòng số kết nối đang thiết lập tới cổng 443 (`ss -tnH state established '( sport = :443 )' | wc -l`).
2. Thêm `/var/log/toinoi/tai-nguyen.log` vào `/etc/logrotate.d/bao-gia` ([LNX-039](/articles/linux-logrotate)): weekly, rotate 8, không cần `postrotate` (vì sao?).
3. Viết `canh-bao-dia`: thoát với mã 1 và in cảnh báo nếu `/` hoặc `/du-lieu` dùng trên 80%. Chạy nó trong service báo cáo bằng `ExecStartPost=`. Journal ghi gì khi nó thất bại?
4. Thêm timer rsync sang app02 lúc 03:00 (bài tập của [LNX-046](/articles/linux-dong-bo-rsync-ssh-studio)), chạy lại `npm run kiem-tra:p05`.

## Đọc tiếp

Phần 5 khép lại với một server có hàng rào rõ ràng, dịch vụ có TLS và số liệu tài nguyên được ghi lại đều đặn. **Phần 6** chuyển sang vận hành nhiều server: workspace nhiều server và bastion trong SSH Studio, chạy lệnh trên nhiều máy, và xử lý sự cố có bằng chứng. Xem lại [lộ trình của series](/articles/linux-la-gi-va-ssh-studio).
