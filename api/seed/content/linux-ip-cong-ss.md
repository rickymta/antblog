---
slug: linux-ip-cong-ss
title: "Địa chỉ IP, cổng và ss: dịch vụ đang nghe ở đâu, 127.0.0.1 khác 0.0.0.0 thế nào"
summary: "Mở đầu Phần 5 của series Linux qua SSH Studio: đọc địa chỉ và tuyến của server bằng ip, cổng và socket là gì, liệt kê cổng đang nghe bằng ss -tlnp, vì sao bao-gia chỉ nghe ở 127.0.0.1 nên máy khác không gọi được, thử một dịch vụ nghe ở 0.0.0.0 từ server thứ hai bằng nc, và đọc các kết nối đang mở."
category: devops-linux
level: beginner
tags: [linux, lnx-041, network, ip, ss, port, socket, loopback, ssh-studio, toi-noi]
language: vi
---

Đội ứng dụng di động hỏi An: «Bọn mình gọi thẳng `bao-gia` ở cổng 8080 của `toinoi-app-01` được không?» Trên server, `curl http://127.0.0.1:8080` chạy tốt từ Phần 4. Nhưng từ một máy khác thì không. Để trả lời được vì sao, và có nên mở hay không, An cần biết server có những địa chỉ nào, dịch vụ nào đang **nghe** ở đâu, và ai thực sự gọi tới được.

Phần 5 nói về server trong mạng: địa chỉ, DNS, đĩa, port forwarding, tường lửa, đồng bộ, theo dõi tài nguyên, và một reverse proxy có TLS. Lab có thêm một server thứ hai, **`toinoi-app-02`**, cùng mạng nội bộ với `toinoi-app-01`, để thử «từ một máy khác».

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-041 · Phần 5/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-systemd-unit-bao-gia">LNX-036</a> (<code>bao-gia</code> chạy bằng systemd). Trạng thái đầu bài: <code>npm run setup -- 41</code>; lab có hai server trong mạng <code>10.20.0.0/24</code>: <code>toinoi-app-01</code> (10.20.0.11, SSH Studio kết nối như mọi khi) và <code>toinoi-app-02</code> (10.20.0.12, không mở cổng nào ra máy bạn). Các lệnh chạy trên app02 có dấu nhắc <code>an@toinoi-app-02</code> trong bản ghi.</p>
</div>

## 1. Địa chỉ của server

```text
an@toinoi-app-01:~$ hostname
toinoi-app-01
an@toinoi-app-01:~$ ip -br addr
lo               UNKNOWN        127.0.0.1/8 ::1/128
eth0@if91        UP             10.20.0.11/24
an@toinoi-app-01:~$ ip route
default via 10.20.0.1 dev eth0
10.20.0.0/24 dev eth0 proto kernel scope link src 10.20.0.11
an@toinoi-app-01:~$ hostname -I
10.20.0.11
an@toinoi-app-01:~$ grep toinoi /etc/hosts
10.20.0.11      toinoi-app-01
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ip-cong-ss/ssh-studio-04.webp" alt="Terminal của SSH Studio: hostname in toinoi-app-01, ip -br addr cho lo 127.0.0.1/8 và eth0 UP 10.20.0.11/24" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tên và địa chỉ IP của server</figcaption></figure>

- **`ip -br addr`** (`-br`: dạng gọn) liệt kê **giao diện mạng** và địa chỉ của chúng. `lo` là giao diện **loopback**: địa chỉ `127.0.0.1` (IPv4) và `::1` (IPv6) chỉ dẫn về chính máy này, không bao giờ ra mạng. `eth0` là card mạng nối vào mạng nội bộ, địa chỉ `10.20.0.11/24` (`/24`: cùng mạng với mọi địa chỉ `10.20.0.x`).
- **`ip route`**: gói tin đi đâu. Đích trong `10.20.0.0/24` đi thẳng qua `eth0`; mọi đích khác đi qua **gateway** `10.20.0.1` (`default via`).
- **`hostname -I`**: các địa chỉ của máy, không tính loopback.
- **`/etc/hosts`**: tên `toinoi-app-01` ứng với `10.20.0.11` ngay trên máy này ([LNX-042](/articles/linux-dns-ping-curl) bàn về phân giải tên).

Trên một máy ảo đám mây, `eth0` (hay `ens5`, `enp0s3`) thường có một địa chỉ riêng như vậy; địa chỉ công khai nằm ở thiết bị NAT của nhà cung cấp, server không thấy trực tiếp.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ip-cong-ss/ssh-studio-01.webp" alt="Terminal của SSH Studio: ip route cho default via 10.20.0.1 dev eth0 và mạng 10.20.0.0/24, hostname -I in 10.20.0.11, grep toinoi /etc/hosts cho 10.20.0.11 toinoi-app-01" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đường đi mặc định và tên máy trong <code>/etc/hosts</code></figcaption></figure>

## 2. Cổng và dịch vụ đang nghe

Một dịch vụ mạng **nghe** (*listen*) ở một **cổng** (*port*, số 1–65535) trên một **địa chỉ**. Cặp địa chỉ:cổng cùng giao thức (TCP hay UDP) gọi là **socket**. Cổng dưới 1024 chỉ `root` mở được (22 cho SSH, 80 HTTP, 443 HTTPS).

```text
an@toinoi-app-01:~$ ss -tln
State      Recv-Q     Send-Q         Local Address:Port           Peer Address:Port     Process
LISTEN     0          5                  127.0.0.1:8080                0.0.0.0:*
LISTEN     0          4096              127.0.0.11:38807               0.0.0.0:*
LISTEN     0          128                  0.0.0.0:22                  0.0.0.0:*
LISTEN     0          128                     [::]:22                     [::]:*
an@toinoi-app-01:~$ sudo ss -tlnp
State   Recv-Q  Send-Q   Local Address:Port    Peer Address:Port Process
LISTEN  0       5            127.0.0.1:8080         0.0.0.0:*     users:(("bao-gia",pid=4281,fd=4))
LISTEN  0       4096        127.0.0.11:38807        0.0.0.0:*
LISTEN  0       128            0.0.0.0:22           0.0.0.0:*     users:(("sshd",pid=1833,fd=3))
LISTEN  0       128               [::]:22              [::]:*     users:(("sshd",pid=1833,fd=4))
an@toinoi-app-01:~$ curl -s http://127.0.0.1:8080/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 4281, "moi_truong": "production", "so_dich_vu": 4}
an@toinoi-app-01:~$ curl -sS --connect-timeout 3 http://10.20.0.11:8080/suc-khoe
curl: (7) Failed to connect to 10.20.0.11 port 8080 after 0 ms: Couldn't connect to server
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ip-cong-ss/ssh-studio-05.webp" alt="Terminal của SSH Studio: ss -tln liệt kê các cổng LISTEN, sudo ss -tlnp thêm cột Process với sshd và bao-gia, curl 127.0.0.1:8080 trả ok, curl 10.20.0.11:8080 báo Failed to connect" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · bao-gia chỉ nghe trên 127.0.0.1, gọi qua IP của eth0 thì bị từ chối</figcaption></figure>

**`ss`** (*socket statistics*) là công cụ hiện đại thay cho `netstat`:

| Tuỳ chọn | Ý nghĩa |
|---|---|
| `-t` / `-u` | TCP / UDP |
| `-l` | Chỉ socket đang nghe |
| `-n` | In số, không đổi cổng thành tên (`22` thay vì `ssh`) |
| `-p` | Kèm tiến trình (cần `sudo` để thấy tiến trình của người khác) |

Đọc cột **Local Address:Port**:

- `0.0.0.0:22` và `[::]:22`: `sshd` nghe ở **mọi địa chỉ** IPv4 và IPv6. Ai tới được bất kỳ địa chỉ nào của server đều kết nối được tới cổng 22.
- `127.0.0.1:8080`: `bao-gia` chỉ nghe ở **loopback**. Chỉ tiến trình trên chính server gọi được nó. Gọi qua `10.20.0.11:8080`, kể cả từ chính server, bị từ chối: không có gì nghe ở địa chỉ đó.
- `127.0.0.11:…`: DNS nội bộ của Docker, dấu vết của lab chạy trong container.

`bao-gia` nghe ở `127.0.0.1` là chủ ý (từ chính mã của nó): dịch vụ không mã hoá, không xác thực, nên không được ra mạng. [LNX-049](/articles/linux-nginx-reverse-proxy-tls) đặt nginx có TLS phía trước.

## 3. 127.0.0.1 hay 0.0.0.0: thử từ máy khác

An mở tạm một dịch vụ khác, nghe ở **mọi địa chỉ**: trình chia sẻ thư mục có sẵn của Python, phục vụ thư mục log (chạy bằng `nohup` như [LNX-034](/articles/linux-job-nen-nohup-tmux)):

```text
an@toinoi-app-01:~$ cd /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:/srv/toinoi/bao-gia/shared/logs$ nohup python3 -m http.server 8081 --bind 0.0.0.0 > /dev/null 2>&1 &
an@toinoi-app-01:/srv/toinoi/bao-gia/shared/logs$ cd ~
an@toinoi-app-01:~$ sudo ss -tlnp | grep -E ':(22|8080|8081) '
LISTEN 0      5          127.0.0.1:8080       0.0.0.0:*    users:(("bao-gia",pid=4281,fd=4))
LISTEN 0      5            0.0.0.0:8081       0.0.0.0:*    users:(("python3",pid=4344,fd=3))
LISTEN 0      128          0.0.0.0:22         0.0.0.0:*    users:(("sshd",pid=1833,fd=3))
LISTEN 0      128             [::]:22            [::]:*    users:(("sshd",pid=1833,fd=4))
```

Cột cuối của `sudo ss -tlnp` cho biết tiến trình nào giữ socket: `sshd`, `bao-gia`, `python3`. Đây là cách nhanh nhất trả lời «cái gì đang chiếm cổng 8081?».

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ip-cong-ss/ssh-studio-02.webp" alt="Terminal của SSH Studio: chạy python3 -m http.server 8081 --bind 0.0.0.0 bằng nohup trong thư mục logs, sudo ss -tlnp cho 0.0.0.0:22 sshd, 0.0.0.0:8081 python3, 127.0.0.1:8080 bao-gia, [::]:22 sshd" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · 8081 nghe trên mọi địa chỉ, 8080 chỉ nghe trên 127.0.0.1</figcaption></figure>

Từ `toinoi-app-02`, thử ba cổng bằng **`nc -zv`** (`-z`: chỉ thử kết nối, không gửi gì; `-v`: in kết quả):

```text
an@toinoi-app-02:~$ hostname
toinoi-app-02
an@toinoi-app-02:~$ nc -zv -w 3 toinoi-app-01 22
Connection to toinoi-app-01 (10.20.0.11) 22 port [tcp/ssh] succeeded!
an@toinoi-app-02:~$ nc -zv -w 3 toinoi-app-01 8080
nc: connect to toinoi-app-01 (10.20.0.11) port 8080 (tcp) failed: Connection refused
an@toinoi-app-02:~$ nc -zv -w 3 toinoi-app-01 8081
Connection to toinoi-app-01 (10.20.0.11) 8081 port [tcp/tproxy] succeeded!
an@toinoi-app-02:~$ curl -s http://toinoi-app-01:8081/ | grep -o 'href="[^"]*"' | head -n 3
href="bao-cao-2026-11-15.txt"
href="bao-cao-2026-11-21.txt"
href="bao-cao-2026-11-24.txt"
```

| Cổng | Nghe ở | Từ app02 |
|---|---|---|
| 22 | `0.0.0.0` | `succeeded` |
| 8080 | `127.0.0.1` | `Connection refused`: app01 trả lời ngay «không có ai nghe ở đây» |
| 8081 | `0.0.0.0` | `succeeded`, và đọc được danh sách file log |

Cổng 8081 cho **mọi máy trong mạng** xem thư mục log. Một dòng lệnh tiện tay là đủ để lộ dữ liệu: đó là lý do cần tường lửa ([LNX-045](/articles/linux-tuong-lua-ufw-nftables)).

## 4. Kết nối đang mở

```text
an@toinoi-app-01:~$ ss -tn
State     Recv-Q     Send-Q          Local Address:Port           Peer Address:Port     Process
an@toinoi-app-01:~$ sudo ss -tnp state established | grep -v 127.0.0.11
Recv-Q Send-Q Local Address:Port Peer Address:PortProcess
an@toinoi-app-01:~$ pkill -f 'http.server 8081'
an@toinoi-app-01:~$ sudo ss -tlnp | grep -c ':8081 '
0
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ip-cong-ss/ssh-studio-06.webp" alt="Terminal của SSH Studio: ss -tn cho một kết nối ESTAB từ 10.20.0.1:33338 tới 10.20.0.11:22" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>ss -tn</code>: kết nối SSH của chính phiên đang dùng</figcaption></figure>

**`ss -tn`** (không `-l`) liệt kê kết nối đã thiết lập. Bản ghi của lab không có kết nối nào (runner không đi qua SSH). Trong SSH Studio, bạn thấy chính phiên của mình: `10.20.0.11:22` nối với một cổng cao của địa chỉ gateway, trạng thái `ESTAB`, tiến trình `sshd`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ip-cong-ss/ssh-studio-03.webp" alt="Terminal của SSH Studio: sudo ss -tnp state established cho kết nối SSH 10.20.0.11:22 từ 10.20.0.1 của sshd, pkill -f http.server 8081, ss -tlnp | grep -c :8081 in 0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · kết nối đang mở và dọn máy chủ thử 8081</figcaption></figure>

## 5. Đối chiếu thao tác

| Câu hỏi | Lệnh |
|---|---|
| Server có địa chỉ nào | `ip -br addr`, `hostname -I` |
| Gói tin đi đường nào | `ip route` |
| Dịch vụ nào nghe cổng nào | `sudo ss -tlnp` |
| Cái gì đang chiếm cổng 8081 | `sudo ss -tlnp 'sport = :8081'` |
| Kết nối đang mở | `ss -tn`, `sudo ss -tnp` |
| Từ máy khác, cổng có mở không | `nc -zv -w 3 MÁY CỔNG` |

## 6. Lỗi thường gặp

**Dịch vụ chạy nhưng «không vào được».** Nó nghe ở `127.0.0.1`. Kiểm tra cột Local Address trước khi nghi tường lửa.

**Mở `0.0.0.0` cho tiện.** Bất kỳ ai trong mạng (hay Internet, nếu server có địa chỉ công khai) đều gọi được. Nghe ở `127.0.0.1` và đi qua proxy hoặc đường hầm SSH ([LNX-044](/articles/linux-ssh-port-forwarding)) nếu chỉ mình cần.

**`ss -tlnp` không có tên tiến trình.** Thiếu `sudo`: bạn chỉ thấy tiến trình của chính mình.

**`Address already in use`.** Cổng đã có người nghe; tìm bằng `sudo ss -tlnp` rồi quyết định dừng ai.

## Bài tập

1. Chạy `python3 -m http.server 8090 --bind 127.0.0.1` rồi thử từ app02 bằng `nc -zv`. Kết quả khác 8081 thế nào?
2. `ss -s` in gì? `ss -tan state time-wait` là gì?
3. `ip -s link show eth0`: đọc số byte đã gửi, nhận. Gọi `/suc-khoe` vài lần qua `10.20.0.11` (nếu đã mở) rồi xem lại.
4. Trong SSH Studio, mở thêm một tab phiên. `ss -tn` có thêm dòng nào? Cổng phía client của hai phiên có giống nhau không?

## Đọc tiếp

[LNX-042 · DNS và kiểm tra kết nối](/articles/linux-dns-ping-curl): tên được phân giải thế nào, và phân biệt lỗi DNS, lỗi mạng, lỗi ứng dụng.
