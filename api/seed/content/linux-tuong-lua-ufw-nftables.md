---
slug: linux-tuong-lua-ufw-nftables
title: "Tường lửa với ufw và nftables: chặn mặc định, chỉ mở cổng cần thiết, không tự khoá cổng SSH"
summary: "Tường lửa của server làm gì, ufw và nftables liên quan thế nào, viết quy tắc trước khi bật (default deny, limit OpenSSH, allow từ một dải mạng), bật mà phiên SSH đang mở không bị cắt, thử từ toinoi-app-02 để thấy cổng được phép, cổng bị chặn (timeout), xoá quy tắc, và đọc quy tắc thật ufw sinh ra trong nftables."
category: devops-linux
level: intermediate
tags: [linux, lnx-045, firewall, ufw, nftables, iptables, ssh, ssh-studio, toi-noi]
language: vi
---

Ở [LNX-041](/articles/linux-ip-cong-ss), một lệnh `python3 -m http.server` tiện tay là đủ để mọi máy trong mạng đọc được thư mục log. Trên `toinoi-app-01` chưa có gì ngăn chuyện đó: dịch vụ nào nghe ở `0.0.0.0` là ai cũng tới được. Bình yêu cầu: chỉ những cổng cần thiết được mở, mọi thứ khác bị chặn mặc định. Và một điều kiện không thương lượng: An không được tự khoá mình ngoài cửa, server ở xa, không có màn hình.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-045 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-ip-cong-ss">LNX-041</a>, <a href="/articles/linux-dns-ping-curl">LNX-042</a> (timeout khác bị từ chối). Trạng thái đầu bài: <code>npm run setup -- 45</code>. <code>ufw</code> có sẵn trên Ubuntu Server. Thử nghiệm «từ máy khác» chạy trên <code>toinoi-app-02</code>.</p>
</div>

## 1. Tường lửa, ufw và nftables

**Tường lửa** (*firewall*) của server là bộ lọc gói tin trong kernel: mỗi kết nối đến được so với danh sách quy tắc, rồi được cho qua, bị từ chối, hay bị bỏ im lặng. Trên Linux hiện đại, cơ chế trong kernel là **nftables** (thay cho `iptables` cũ). Quy tắc nftables mạnh nhưng dài và dễ sai, nên Ubuntu có **ufw** (*Uncomplicated Firewall*): bạn viết quy tắc ngắn gọn, ufw dịch ra nftables.

```text
an@toinoi-app-01:~$ sudo ufw status
Status: inactive
an@toinoi-app-01:~$ sudo nft list ruleset | wc -l
21
an@toinoi-app-01:~$ sudo ufw app list
Available applications:
  OpenSSH
an@toinoi-app-01:~$ sudo ufw app info OpenSSH
Profile: OpenSSH
Title: Secure shell server, an rshd replacement
Description: OpenSSH is a free implementation of the Secure Shell protocol.

Port:
  22/tcp
```

- `inactive`: tường lửa chưa bật; nftables chỉ có vài quy tắc của Docker.
- **Hồ sơ ứng dụng** (`ufw app list`): gói phần mềm khai báo cổng của mình. `OpenSSH` là cổng `22/tcp`. Dùng tên hồ sơ dễ đọc hơn số cổng, và đúng cả khi gói đổi cổng.

## 2. Viết quy tắc trước, bật sau

```text
an@toinoi-app-01:~$ sudo ufw default deny incoming
Default incoming policy changed to 'deny'
(be sure to update your rules accordingly)
an@toinoi-app-01:~$ sudo ufw default allow outgoing
Default outgoing policy changed to 'allow'
(be sure to update your rules accordingly)
an@toinoi-app-01:~$ sudo ufw limit OpenSSH
Rules updated
Rules updated (v6)
an@toinoi-app-01:~$ sudo ufw allow from 10.20.0.0/24 to any port 8081 proto tcp comment 'thu muc log noi bo'
Rules updated
an@toinoi-app-01:~$ sudo ufw show added
Added user rules (see 'ufw status' for running firewall):
ufw limit OpenSSH
ufw allow from 10.20.0.0/24 to any port 8081 proto tcp comment 'thu muc log noi bo'
```

| Quy tắc | Nghĩa |
|---|---|
| `default deny incoming` | Mọi kết nối **đến** không khớp quy tắc nào thì bị bỏ |
| `default allow outgoing` | Server tự do kết nối **đi** (cài gói, gọi API…) |
| `limit OpenSSH` | Cho phép SSH, nhưng một địa chỉ mở quá 6 kết nối trong 30 giây thì bị chặn tạm: giảm dò mật khẩu |
| `allow from 10.20.0.0/24 to any port 8081 proto tcp` | Chỉ máy trong mạng nội bộ được vào cổng 8081, có ghi chú |

**`ufw show added`** liệt kê quy tắc đã thêm **khi tường lửa còn tắt**. Đây là thói quen cứu mạng: **luôn có quy tắc cho SSH trước khi bật**. Bật `default deny` mà chưa cho SSH là tự khoá mình ngoài cửa ngay lập tức.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tuong-lua-ufw-nftables/ssh-studio-01.webp" alt="Terminal của SSH Studio: ufw status inactive, nft list ruleset 21 dòng, ufw app list có OpenSSH, ufw app info OpenSSH cổng 22/tcp, default deny incoming, default allow outgoing, limit OpenSSH, allow từ 10.20.0.0/24 tới cổng 8081 kèm comment, ufw show added liệt kê hai quy tắc" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · viết quy tắc trước khi bật</figcaption></figure>

## 3. Bật tường lửa

```text
an@toinoi-app-01:~$ sudo ufw --force enable
Firewall is active and enabled on system startup
an@toinoi-app-01:~$ sudo ufw status verbose
Status: active
Logging: on (low)
Default: deny (incoming), allow (outgoing), deny (routed)
New profiles: skip

To                         Action      From
--                         ------      ----
22/tcp (OpenSSH)           LIMIT IN    Anywhere
8081/tcp                   ALLOW IN    10.20.0.0/24               # thu muc log noi bo
22/tcp (OpenSSH (v6))      LIMIT IN    Anywhere (v6)

an@toinoi-app-01:~$ sudo ufw status numbered
Status: active

     To                         Action      From
     --                         ------      ----
[ 1] OpenSSH                    LIMIT IN    Anywhere
[ 2] 8081/tcp                   ALLOW IN    10.20.0.0/24               # thu muc log noi bo
[ 3] OpenSSH (v6)               LIMIT IN    Anywhere (v6)
```

- Gõ tay, `sudo ufw enable` hỏi `Command may disrupt existing ssh connections. Proceed (y|n)?`. Bản ghi dùng `--force` để không hỏi; đừng dùng nó cho tới khi đã chắc có quy tắc SSH.
- **Phiên SSH Studio đang mở không bị cắt**: tường lửa của ufw cho qua mọi kết nối đã thiết lập. Dù vậy, **mở thêm một phiên mới để thử** trước khi đóng phiên cũ, như với `sshd` ([LNX-025](/articles/linux-sshd-config-an-toan)).
- `status verbose` cho cả mặc định (`deny (incoming), allow (outgoing), deny (routed)`) và nhật ký (`Logging: on (low)`): gói bị chặn được ghi vào log của kernel.
- `status numbered` đánh số quy tắc, dùng để xoá hay chèn theo vị trí.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tuong-lua-ufw-nftables/ssh-studio-02.webp" alt="Terminal của SSH Studio: sudo ufw --force enable báo Firewall is active and enabled on system startup, status verbose với Default deny incoming allow outgoing, 22/tcp OpenSSH LIMIT IN Anywhere, 8081/tcp ALLOW IN 10.20.0.0/24 kèm ghi chú, status numbered đánh số [1] [2] [3]; phiên SSH vẫn tiếp tục" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · bật <code>ufw</code>, phiên đang mở không bị cắt</figcaption></figure>

## 4. Thử từ máy khác

Hai dịch vụ thử: 8081 (có quy tắc cho mạng nội bộ) và 8082 (không có quy tắc). Từ `toinoi-app-02`:

```text
an@toinoi-app-01:~$ cd /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:/srv/toinoi/bao-gia/shared/logs$ nohup python3 -m http.server 8081 --bind 0.0.0.0 > /dev/null 2>&1 &
an@toinoi-app-01:/srv/toinoi/bao-gia/shared/logs$ nohup python3 -m http.server 8082 --bind 0.0.0.0 > /dev/null 2>&1 &
an@toinoi-app-01:/srv/toinoi/bao-gia/shared/logs$ cd ~
an@toinoi-app-02:~$ nc -zv -w 3 toinoi-app-01 22
Connection to toinoi-app-01 (10.20.0.11) 22 port [tcp/ssh] succeeded!
an@toinoi-app-02:~$ nc -zv -w 3 toinoi-app-01 8081
Connection to toinoi-app-01 (10.20.0.11) 8081 port [tcp/tproxy] succeeded!
an@toinoi-app-02:~$ nc -zv -w 3 toinoi-app-01 8082
nc: connect to toinoi-app-01 (10.20.0.11) port 8082 (tcp) timed out: Operation now in progress
```

- 22 và 8081: `succeeded`.
- 8082: **`timed out`**, không phải `refused`. Dịch vụ vẫn đang chạy, nhưng tường lửa **bỏ im lặng** gói tin (`deny`), nên máy kia chờ tới hết giờ. So với [LNX-042](/articles/linux-dns-ping-curl): timeout là dấu hiệu điển hình của tường lửa. (`ufw reject` thì trả lời «từ chối» ngay; `deny` khiến kẻ dò cổng phải đợi.)

## 5. Sửa, xoá, và nhìn xuống nftables

```text
an@toinoi-app-01:~$ sudo ufw status numbered | grep 8081
[ 2] 8081/tcp                   ALLOW IN    10.20.0.0/24               # thu muc log noi bo
an@toinoi-app-01:~$ sudo ufw delete allow from 10.20.0.0/24 to any port 8081 proto tcp
Rule deleted
an@toinoi-app-01:~$ sudo ufw status numbered
Status: active

     To                         Action      From
     --                         ------      ----
[ 1] OpenSSH                    LIMIT IN    Anywhere
[ 2] OpenSSH (v6)               LIMIT IN    Anywhere (v6)

an@toinoi-app-01:~$ pkill -f 'http.server 808[12]'

an@toinoi-app-01:~$ sudo nft list ruleset | wc -l
412
an@toinoi-app-01:~$ sudo nft list chain ip filter ufw-user-input
table ip filter {
        chain ufw-user-input {
                tcp dport 22 ct state new xt match "recent"  counter packets 0 bytes 0
                tcp dport 22 ct state new xt match "recent"  counter packets 0 bytes 0 jump ufw-user-limit
                tcp dport 22  counter packets 0 bytes 0 jump ufw-user-limit-accept
        }
}
```

- **`ufw delete <quy tắc>`** xoá bằng chính câu lệnh đã thêm (chắc chắn hơn xoá theo số, vì số đổi khi danh sách đổi). `ufw delete 2` cũng được, sau khi xem `status numbered`.
- Từ 21 dòng, nftables giờ có hơn 400 dòng: các chuỗi (*chain*) `ufw-before-input`, `ufw-user-input`, `ufw-after-input`… mà ufw dựng. Chuỗi `ufw-user-input` chứa đúng quy tắc của An: ba dòng cho cổng 22 với module `recent` (cơ chế của `limit`).
- Đừng sửa thẳng các bảng đó (`do not touch!`): ufw sẽ ghi đè. Cần quy tắc phức tạp hơn ufw cho phép thì viết thẳng nftables trong `/etc/nftables.conf` và **không** dùng ufw song song.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tuong-lua-ufw-nftables/ssh-studio-03.webp" alt="Terminal của SSH Studio: chạy hai http.server 8081 và 8082, từ app02 nc tới 22 và 8081 succeeded còn 8082 timed out, ufw delete quy tắc 8081, status numbered còn hai quy tắc OpenSSH, nft list ruleset hơn 400 dòng với cảnh báo do not touch, chain ufw-user-input với các dòng tcp dport 22 jump ufw-user-limit" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · thử từ máy khác, xoá quy tắc, và quy tắc thật trong nftables</figcaption></figure>

Trên máy ảo đám mây còn có **tường lửa của nhà cung cấp** (security group, firewall rule của VPC) chặn trước khi gói tới server. Hai lớp bổ sung cho nhau; khi «không vào được», kiểm tra cả hai.

## 6. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Trạng thái | `sudo ufw status verbose`, `status numbered` |
| Mặc định | `sudo ufw default deny incoming` |
| Cho SSH, có giới hạn | `sudo ufw limit OpenSSH` |
| Cho một cổng từ một dải mạng | `sudo ufw allow from 10.20.0.0/24 to any port 443 proto tcp` |
| Xem quy tắc trước khi bật | `sudo ufw show added` |
| Bật / tắt | `sudo ufw enable`, `disable` |
| Xoá | `sudo ufw delete <quy tắc>` hoặc `delete N` |
| Quy tắc thật | `sudo nft list ruleset` |

## 7. Lỗi thường gặp

**Bật trước, thêm SSH sau.** Khoá mình ngoài cửa. Luôn `show added` trước `enable`.

**`allow 8081` không giới hạn nguồn.** Mở cho cả thế giới nếu server có địa chỉ công khai. Thêm `from`.

**Tưởng ufw chặn được dịch vụ nghe ở `127.0.0.1` từ chính server.** Loopback luôn được cho qua; giới hạn cục bộ là việc của cấu hình dịch vụ.

**Dùng song song ufw, iptables thủ công và nftables thủ công.** Quy tắc giẫm lên nhau. Chọn một.

**Quên tường lửa của nhà cung cấp đám mây.** Mở ufw rồi mà vẫn timeout: kiểm tra security group.

## Bài tập

1. `sudo ufw allow from 10.20.0.12 to any port 8082 proto tcp` (chỉ một máy). Thử từ app02. Rồi xoá quy tắc.
2. Đổi một quy tắc sang `reject` thay vì `deny` và thử lại bằng `nc`. Thông báo khác thế nào?
3. Mở 7 kết nối SSH thật nhanh từ app02 tới app01 (`for i in $(seq 7); do nc -z -w 1 toinoi-app-01 22; done`). Kết nối thứ mấy bị chặn? Đợi 30 giây rồi thử lại.
4. `sudo ufw status verbose` có `deny (routed)`. «routed» là gì, và khi nào nó quan trọng (gợi ý: Docker)?

## Đọc tiếp

[LNX-046 · Đồng bộ thư mục: SSH Studio và rsync](/articles/linux-dong-bo-rsync-ssh-studio).
