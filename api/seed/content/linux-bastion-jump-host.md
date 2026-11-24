---
slug: linux-bastion-jump-host
title: "Bastion và jump host: vào server nội bộ qua một bước nhảy bằng SSH Studio và ProxyJump"
summary: "Vì sao server nội bộ không mở SSH ra ngoài, bastion được dựng thế nào (chỉ key, không chuyển tiếp agent), cấu hình toinoi-app-02 đi qua jump host trong SSH Studio, mỗi bước nhảy được đối chiếu host key và xác thực riêng, ProxyJump của OpenSSH tương ứng, và khoá tường lửa của app02 để chỉ bastion và app01 tới được cổng 22."
category: devops-linux
level: beginner
tags: [linux, lnx-052, ssh, bastion, jump-host, proxyjump, ufw, ssh-studio, toi-noi]
language: vi
---

`toinoi-app-02` không có cổng nào mở ra Internet: nó chỉ có địa chỉ `10.20.0.12` trong mạng nội bộ. Đó là chủ ý. Mỗi server mở SSH ra ngoài là một chỗ phải theo dõi, vá và chặn dò mật khẩu. Tới Nơi chọn cách phổ biến: **một** máy duy nhất mở SSH ra ngoài, gọi là **bastion** (hay **jump host**), và mọi server khác chỉ nhận SSH từ trong mạng. An muốn vào app02 thì đi qua bastion.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-052 · Phần 6/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-workspace-launchpad-nhieu-server">LNX-051</a>, <a href="/articles/linux-host-key-known-hosts">LNX-023</a> (host key), <a href="/articles/linux-tuong-lua-ufw-nftables">LNX-045</a> (ufw). Trạng thái đầu bài: <code>npm run setup -- 52</code>.</p>
</div>

## 1. Bastion được dựng thế nào

Trong tab của `toinoi-bastion` (mở ở [LNX-051](/articles/linux-workspace-launchpad-nhieu-server)):

```text
an@toinoi-bastion:~$ cat /etc/ssh/sshd_config.d/10-bastion.conf
PasswordAuthentication no
KbdInteractiveAuthentication no
PermitRootLogin no
AllowAgentForwarding no
X11Forwarding no
AllowTcpForwarding yes
an@toinoi-bastion:~$ sudo sshd -T | grep -E '^(passwordauthentication|kbdinteractiveauthentication|permitrootlogin|allowagentforwarding|allowtcpforwarding|x11forwarding) '
permitrootlogin no
passwordauthentication no
kbdinteractiveauthentication no
x11forwarding no
allowtcpforwarding yes
allowagentforwarding no
an@toinoi-bastion:~$ ip -br addr show eth0
eth0@if144       UP             10.20.0.10/24
an@toinoi-bastion:~$ nc -zv -w 3 toinoi-app-02 22
Connection to toinoi-app-02 (10.20.0.12) 22 port [tcp/ssh] succeeded!
```

| Thiết lập | Vì sao |
|---|---|
| `PasswordAuthentication no`, `KbdInteractiveAuthentication no` | Máy mở ra ngoài: chỉ nhận key, không cho dò mật khẩu |
| `PermitRootLogin no` | Không ai đăng nhập thẳng bằng `root` |
| `AllowAgentForwarding no` | Ai chiếm được bastion cũng không mượn được ssh-agent trên laptop của bạn ([LNX-028](/articles/linux-ssh-agent-passphrase)) |
| `X11Forwarding no` | Không cần giao diện đồ hoạ |
| `AllowTcpForwarding yes` | **Bắt buộc** cho jump host: bastion mở kênh TCP tới máy đích thay cho bạn |

Bastion có hai mặt: cổng 2200 trên máy bạn (trên máy ảo thật là IP công khai) và `10.20.0.10` trong mạng nội bộ. Từ đó, cổng 22 của app02 tới được.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bastion-jump-host/ssh-studio-01.webp" alt="Terminal của SSH Studio trên toinoi-bastion: nội dung 10-bastion.conf, sudo sshd -T cho thấy passwordauthentication no, allowagentforwarding no, allowtcpforwarding yes, ip -br addr với 10.20.0.10/24, nc tới toinoi-app-02 cổng 22 succeeded" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · bastion: chỉ nhận key, được phép mở kênh TCP</figcaption></figure>

## 2. toinoi-app-02 qua jump host

**Thêm server** như thường, chỉ khác mục **Kết nối qua**: chọn **Jump host**, rồi chọn `toinoi-bastion` trong ô **Jump host (ProxyJump)**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bastion-jump-host/ssh-studio-02.webp" alt="Hộp thoại Thêm server của SSH Studio: Tên hiển thị toinoi-app-02, Địa chỉ server toinoi-app-02, Port 22, Username an, Nhóm Production, SSH Key an-laptop, Kết nối qua Jump host, Jump host (ProxyJump) toinoi-bastion · an@127.0.0.1:2200" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>toinoi-app-02</code> kết nối qua jump host <code>toinoi-bastion</code></figcaption></figure>

Hai chi tiết dễ nhầm:

- **Địa chỉ server là địa chỉ nhìn từ bastion**, không phải từ máy bạn. `toinoi-app-02` (hay `10.20.0.12`) không phân giải được trên laptop của An, nhưng bastion thì được.
- **Xác thực từng bước bằng hồ sơ của bước đó.** SSH Studio đăng nhập bastion bằng hồ sơ `toinoi-bastion` (key `an-laptop`), mở một kênh TCP từ bastion tới `toinoi-app-02:22`, rồi đăng nhập app02 qua kênh đó bằng hồ sơ của app02. Key không bao giờ được chép lên bastion.

Bấm Kết nối. Lần đầu, SSH Studio hỏi host key **của app02**: mỗi bước nhảy được kiểm tra host key riêng, ghi vào Known Hosts theo địa chỉ `[toinoi-app-02]:22`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bastion-jump-host/ssh-studio-03.webp" alt="Hộp thoại Xác nhận danh tính server của SSH Studio cho toinoi-app-02, fingerprint ED25519 SHA256:bbig/ZMau7+NiJi6Wf6YRxtLARwSfq7ZVPTq6EAMm6U" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đối chiếu host key của <code>toinoi-app-02</code>, kể cả khi đi qua bastion</figcaption></figure>

Vì sao vẫn phải đối chiếu khi đã tin bastion? Vì bastion chỉ chuyển byte. Nếu bastion bị chiếm, kẻ tấn công có thể trỏ kênh tới một máy giả; host key của app02 là thứ duy nhất chứng minh đầu kia đúng là app02.

## 3. Nhìn từ hai đầu

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bastion-jump-host/ssh-studio-04.webp" alt="Terminal của SSH Studio trên toinoi-app-02: hostname, who thấy phiên của an từ 10.20.0.10, echo $SSH_CONNECTION bắt đầu bằng 10.20.0.10" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · trên app02: kết nối đến từ bastion</figcaption></figure>

Với app02, An đến từ `10.20.0.10`, tức bastion. `$SSH_CONNECTION` ghi `IP-nguồn cổng-nguồn IP-đích cổng-đích`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bastion-jump-host/ssh-studio-05.webp" alt="Terminal của SSH Studio trên toinoi-bastion: sudo ss -tnp lọc 10.20.0.12:22 thấy một kết nối ESTAB của tiến trình sshd tới app02" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · trên bastion: đường hầm là một kết nối TCP của <code>sshd</code></figcaption></figure>

Trên bastion, kênh tới app02 là một kết nối TCP bình thường do `sshd` (tiến trình phục vụ phiên của An) mở, giống port forwarding Local ở [LNX-044](/articles/linux-ssh-port-forwarding). Không có shell nào của An chạy trên bastion cho phiên này.

## 4. Tương đương bằng OpenSSH

Trong terminal trên máy bạn (OpenSSH có sẵn trên Windows 10+, macOS, Linux):

```bash
ssh -J an@127.0.0.1:2200 an@toinoi-app-02
```

Hoặc ghi một lần vào `~/.ssh/config` của máy bạn:

```text
Host toinoi-bastion
    HostName 127.0.0.1
    Port 2200
    User an
    IdentityFile ~/.ssh/an-laptop

Host toinoi-app-02
    User an
    IdentityFile ~/.ssh/an-laptop
    ProxyJump toinoi-bastion
```

`ProxyJump a,b` nối chuỗi nhiều bước. SSH Studio cũng nối chuỗi được: jump host có thể có jump host riêng (tối đa 8 bước, lặp vòng thì báo lỗi).

Đừng dùng `ssh -A` (chuyển tiếp agent) để «nhảy» bằng cách đăng nhập bastion rồi `ssh` tiếp từ đó: agent của bạn khi đó dùng được từ bastion. `ProxyJump` không cần điều đó, và bastion này cũng đã tắt nó.

## 5. Khoá cửa sau: app02 chỉ nhận SSH từ bastion

Bastion chỉ có ý nghĩa khi nó là đường **duy nhất**. Trong tab của app02:

```text
an@toinoi-app-02:~$ sudo ufw allow from 10.20.0.10 to any port 22 proto tcp comment 'SSH qua bastion'
Rules updated
an@toinoi-app-02:~$ sudo ufw allow from 10.20.0.11 to any port 22 proto tcp comment 'rsync tu app01'
Rules updated
an@toinoi-app-02:~$ sudo ufw default deny incoming
Default incoming policy changed to 'deny'
(be sure to update your rules accordingly)
an@toinoi-app-02:~$ sudo ufw --force enable
Firewall is active and enabled on system startup
an@toinoi-app-02:~$ sudo ufw status
Status: active

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       10.20.0.10                 # SSH qua bastion
22/tcp                     ALLOW       10.20.0.11                 # rsync tu app01
```

- Thêm quy tắc cho phép **trước** khi bật tường lửa: phiên của An đang đi qua bastion (`10.20.0.10`), nên bật ufw không cắt phiên này ([LNX-045](/articles/linux-tuong-lua-ufw-nftables)).
- `10.20.0.11` (app01) vẫn được vào, vì rsync sao lưu ở [LNX-046](/articles/linux-dong-bo-rsync-ssh-studio) chạy từ app01.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bastion-jump-host/ssh-studio-06.webp" alt="Terminal của SSH Studio trên toinoi-app-02: ufw allow từ 10.20.0.10 và 10.20.0.11 tới cổng 22, default deny incoming, ufw enable, ufw status với hai quy tắc ALLOW" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · app02 chỉ nhận SSH từ bastion và app01</figcaption></figure>

```text
an@toinoi-app-01:~$ ssh app02 hostname
toinoi-app-02
an@toinoi-app-01:~$ nc -zv -w 3 toinoi-app-02 22
Connection to toinoi-app-02 (10.20.0.12) 22 port [tcp/ssh] succeeded!
```

## 6. Lỗi thường gặp

**Điền IP công khai hay `127.0.0.1` cho server đích.** Địa chỉ được bastion phân giải và kết nối: dùng tên hoặc IP nội bộ.

**Chép private key lên bastion để «ssh tiếp».** Key nằm trên bastion là key của mọi ai chiếm được bastion. Dùng jump host.

**Tắt `AllowTcpForwarding` trên bastion.** Jump host không mở được kênh: lỗi «không mở được kết nối tới…» ([LNX-059](/articles/linux-su-co-khong-ssh-duoc)).

**Bastion là đường duy nhất, và nó chết.** Giữ console của nhà cung cấp làm đường dự phòng, và theo dõi bastion như một server quan trọng.

## Bài tập

1. Trên bastion: `sudo journalctl -u ssh -n 5 -o cat`. Dòng nào ghi lại kết nối của An, và có ghi đích `toinoi-app-02` không?
2. Thử từ app01: `nc -zv -w 3 toinoi-app-02 22` và từ app02: `nc -zv -w 3 toinoi-app-01 22`. Tường lửa của app02 chặn chiều nào?
3. Đọc `man sshd_config` mục `PermitOpen`. Viết dòng cho bastion để kênh TCP chỉ được mở tới cổng 22 của `10.20.0.0/24`.
4. Thêm một hồ sơ `toinoi-app-01 (qua bastion)` với địa chỉ `toinoi-app-01`, port 22, jump host bastion. Vì sao kết nối được dù cổng 2222 không dính dáng?

## Đọc tiếp

[LNX-053 · Sao lưu và khôi phục cấu hình SSH Studio, chuyển máy](/articles/linux-sao-luu-khoi-phuc-ssh-studio).
