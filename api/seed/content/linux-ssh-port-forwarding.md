---
slug: linux-ssh-port-forwarding
title: "Port forwarding trong SSH Studio: Local, Remote, Dynamic, mở dịch vụ nội bộ của server ngay trên máy bạn"
summary: "Đường hầm SSH là gì, tạo Local forward trong tab Ports của SSH Studio để gọi bao-gia (chỉ nghe ở 127.0.0.1 của server) từ máy Windows, phía server thấy gì bằng ss, Remote forward để server gọi ngược về máy bạn, Dynamic forward làm proxy SOCKS5, «đích tính từ phía server» nghĩa là gì, và vì sao tài khoản deploy bị tắt đường hầm."
category: devops-linux
level: intermediate
tags: [linux, lnx-044, ssh, port-forwarding, tunnel, socks5, ssh-studio, toi-noi]
language: vi
---

An muốn xem thử `bao-gia` trên trình duyệt của mình, hay nối Postman vào nó để kiểm thử. Nhưng `bao-gia` chỉ nghe ở `127.0.0.1` của server ([LNX-041](/articles/linux-ip-cong-ss)), có chủ đích, và không ai muốn mở nó ra mạng chỉ để An xem. Lối ra là **port forwarding** qua SSH: kết nối SSH An vốn đã có trở thành một đường hầm mã hoá, đưa một cổng trên máy An tới đúng `127.0.0.1:8080` bên trong server.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-044 · Phần 5/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-ip-cong-ss">LNX-041</a>, <a href="/articles/linux-thuc-hanh-khoa-dang-nhap-ssh">LNX-030</a> (<code>AllowTcpForwarding</code>). Trạng thái đầu bài: <code>npm run setup -- 44</code>. Phần giao diện làm trong tab <b>Ports</b> của SSH Studio. Bản ghi chữ của lab dùng <code>toinoi-app-02</code> đóng vai laptop của An, mở cùng các đường hầm bằng lệnh <code>ssh</code>.</p>
</div>

## 1. Ba kiểu đường hầm

| Kiểu | Ai mở cổng | Kết nối đi tới | Dùng khi |
|---|---|---|---|
| **Local** (`ssh -L`) | Máy bạn | Một đích **nhìn từ server** | Xem dịch vụ nội bộ của server (hay của mạng sau server) trên máy mình |
| **Remote** (`ssh -R`) | Server | Một đích **nhìn từ máy bạn** | Cho server gọi ngược về dịch vụ đang chạy trên máy bạn |
| **Dynamic** (`ssh -D`) | Máy bạn, dạng proxy SOCKS5 | Bất kỳ đích nào server tới được | Trình duyệt hay công cụ đi «như từ server» |

Mọi dữ liệu đi bên trong kết nối SSH: đã mã hoá, đã xác thực bằng key (và mã OTP từ [LNX-029](/articles/linux-ssh-mfa-totp)), không cần mở thêm cổng nào trên tường lửa của server.

```text
an@toinoi-app-01:~$ sudo ss -tlnp | grep -E ':(8080|22) '
LISTEN 0      5          127.0.0.1:8080       0.0.0.0:*    users:(("bao-gia",pid=4281,fd=4))
LISTEN 0      128          0.0.0.0:22         0.0.0.0:*    users:(("sshd",pid=1833,fd=3))
LISTEN 0      128             [::]:22            [::]:*    users:(("sshd",pid=1833,fd=4))
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-port-forwarding/ssh-studio-01.webp" alt="Terminal của SSH Studio: sudo ss -tlnp cho thấy sshd nghe 0.0.0.0:22 và bao-gia chỉ nghe 127.0.0.1:8080" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>bao-gia</code> chỉ nghe ở <code>127.0.0.1</code></figcaption></figure>

## 2. Local forward trong SSH Studio

Trong phiên tới `toinoi-app-01`, mở tab **Ports**. Form có năm ô:

| Ô | Giá trị | Ý nghĩa |
|---|---|---|
| Kiểu | Local | |
| Lắng nghe tại / Cổng | `127.0.0.1` / `18080` | Cổng mở **trên máy bạn** |
| Đích (từ server) / Cổng đích | `127.0.0.1` / `8080` | Địa chỉ **nhìn từ server**: `127.0.0.1` ở đây là chính server |

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-port-forwarding/ssh-studio-02.webp" alt="Tab Ports của SSH Studio với form Kiểu Local, Lắng nghe tại 127.0.0.1, Cổng 18080, Đích (từ server) 127.0.0.1, Cổng đích 8080, ô Lưu vào server bỏ chọn, ghi chú Mở cổng trên máy này; mỗi kết nối được server chuyển tới đích" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tạo Local forward <code>127.0.0.1:18080 → 127.0.0.1:8080</code></figcaption></figure>

Bấm **Bắt đầu**. Giờ trên máy Windows của An, `http://127.0.0.1:18080` chính là `bao-gia`:

```text
PS> curl.exe http://127.0.0.1:18080/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 4286, "moi_truong": "production", "so_dich_vu": 4}

HTTP 200
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-port-forwarding/ssh-studio-03.webp" alt="Tab Ports của SSH Studio với đường hầm Local 127.0.0.1:18080 → 127.0.0.1:8080 trạng thái Đang chạy và cột Kết nối đếm số kết nối đang mở trên tổng số" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đường hầm đang chạy, cột <b>Kết nối</b> đếm số kết nối qua nó</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-port-forwarding/ssh-studio-07.webp" alt="Trang Servers của SSH Studio khi đường hầm Local đang chạy: toinoi-app-01 xác thực SSH key với 1 phiên hoạt động, hồ sơ toinoi-app-01 (cuong) dùng mật khẩu đang báo lỗi xác thực" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · trang Servers: phiên đang mở đường hầm vẫn chỉ là một phiên</figcaption></figure>

Ô **Lưu vào server** (bỏ chọn ở đây) ghi đường hầm vào hồ sơ server: lần kết nối sau SSH Studio tự bật lại. Tiện cho đường hầm dùng hằng ngày, như một cơ sở dữ liệu chỉ nghe ở localhost.

### Phía server thấy gì

```text
an@toinoi-app-01:~$ sudo ss -tnp | grep ':8080' | grep -v -E 'TIME-WAIT'
```

Trong SSH Studio, khi máy Windows đang giữ một kết nối qua đường hầm, `ss` trên server cho thấy **`sshd`** (tiến trình của phiên An) nối tới `127.0.0.1:8080`. Với `bao-gia`, yêu cầu đến từ `127.0.0.1`, như mọi yêu cầu nội bộ. (Bản ghi chữ của lab không có dòng này vì runner không mở đường hầm từ SSH Studio.)

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-port-forwarding/ssh-studio-04.webp" alt="Terminal của SSH Studio: sudo ss -tnp lọc cổng 8080 thấy kết nối ESTAB từ tiến trình sshd tới 127.0.0.1:8080 do đường hầm tạo ra" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · trên server: <code>sshd</code> chuyển kết nối tới <code>127.0.0.1:8080</code></figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-port-forwarding/ssh-studio-08.webp" alt="Hộp thoại Nhật ký hoạt động của SSH Studio: các dòng Đã kết nối an@127.0.0.1 và Phiên đã đóng của toinoi-app-01 theo thời gian, ô lọc Tất cả và Lỗi" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · nhật ký hoạt động ghi lại các lần kết nối và đóng phiên</figcaption></figure>

## 3. Remote và Dynamic

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-port-forwarding/ssh-studio-05.webp" alt="Tab Ports của SSH Studio: đường hầm Local đã dừng nên bảng trống, form đổi sang Kiểu Dynamic SOCKS5 với Lắng nghe tại 127.0.0.1 cổng 1080, ô Lưu vào server được chọn sẵn, ghi chú Proxy SOCKS5 trên máy này; ứng dụng đi ra Internet qua server" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đường hầm đã dừng, form <b>Dynamic</b> (SOCKS5)</figcaption></figure>

Bản ghi chữ của lab mô phỏng laptop của An bằng `toinoi-app-02`: một lệnh `ssh` mở cả ba kiểu cùng lúc tới `toinoi-app-01` (key và mã OTP được cung cấp sẵn, bài không in lệnh đó):

```bash
ssh -N -L 18080:127.0.0.1:8080 -R 9090:127.0.0.1:8000 -D 1080 an@toinoi-app-01
```

```text
an@toinoi-app-02:~$ mkdir -p ~/may-an && echo 'trang thu tu laptop cua An' > ~/may-an/index.html
an@toinoi-app-02:~$ nohup python3 -m http.server 8000 --bind 127.0.0.1 --directory ~/may-an > /dev/null 2>&1 &
an@toinoi-app-02:~$ ss -tln | grep -E ':(18080|1080|8000) '
LISTEN 0      5          127.0.0.1:8000       0.0.0.0:*
LISTEN 0      128        127.0.0.1:1080       0.0.0.0:*
LISTEN 0      128        127.0.0.1:18080      0.0.0.0:*
LISTEN 0      128            [::1]:18080         [::]:*
LISTEN 0      128            [::1]:1080          [::]:*
an@toinoi-app-02:~$ curl -s http://127.0.0.1:18080/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 4281, "moi_truong": "production", "so_dich_vu": 4}
an@toinoi-app-02:~$ curl -s --socks5-hostname 127.0.0.1:1080 http://127.0.0.1:8080/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 4281, "moi_truong": "production", "so_dich_vu": 4}
an@toinoi-app-02:~$ curl -s --socks5-hostname 127.0.0.1:1080 -o /dev/null -w '%{http_code}\n' http://toinoi-app-01:8080/suc-khoe
000

an@toinoi-app-01:~$ sudo ss -tlnp | grep ':9090 '
LISTEN 0      128        127.0.0.1:9090       0.0.0.0:*    users:(("sshd",pid=4649,fd=8))
LISTEN 0      128            [::1]:9090          [::]:*    users:(("sshd",pid=4649,fd=7))
an@toinoi-app-01:~$ curl -s http://127.0.0.1:9090/
trang thu tu laptop cua An

an@toinoi-app-01:~$ sudo sshd -T -C user=deploy,host=ci,addr=10.0.0.5 | grep allowtcpforwarding
allowtcpforwarding no
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-ssh-port-forwarding/ssh-studio-06.webp" alt="Terminal của SSH Studio: ss -tlnp thấy sshd nghe 127.0.0.1:9090 và [::1]:9090 do Remote forward mở, curl 127.0.0.1:9090 trả trang thu tu laptop cua An, sshd -T -C user=deploy cho allowtcpforwarding no" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · Remote forward: server mở cổng 9090, kết nối quay về laptop; deploy không được mở đường hầm</figcaption></figure>

- **Local**: `127.0.0.1:18080` trên «laptop» trả về đúng `/suc-khoe` của server.
- **Dynamic**: `curl --socks5-hostname 127.0.0.1:1080` gửi yêu cầu qua proxy; `http://127.0.0.1:8080` được hiểu **từ phía server**, nên tới `bao-gia`. Nhưng `http://toinoi-app-01:8080` thì không (`000`): từ chính server, tên đó là `10.20.0.11`, mà `bao-gia` không nghe ở địa chỉ đó. «Đích tính từ phía server» là chỗ hay nhầm nhất của đường hầm.
- **Remote**: trên server xuất hiện cổng `127.0.0.1:9090` do `sshd` mở. `curl` tới nó trên server nhận được trang chạy trên **laptop**. Mặc định cổng remote chỉ nghe ở loopback của server (`GatewayPorts no`), nên chỉ người trên server dùng được.
- `deploy` có `AllowTcpForwarding no` ([LNX-030](/articles/linux-thuc-hanh-khoa-dang-nhap-ssh)): key của CI bị lộ cũng không biến thành đường hầm vào mạng nội bộ.

## 4. Đối chiếu thao tác

| SSH Studio (tab Ports) | OpenSSH |
|---|---|
| Local, `127.0.0.1:18080` → `127.0.0.1:8080` | `ssh -L 18080:127.0.0.1:8080 server` |
| Remote, server `127.0.0.1:9090` → máy này `127.0.0.1:8000` | `ssh -R 9090:127.0.0.1:8000 server` |
| Dynamic, `127.0.0.1:1080` | `ssh -D 1080 server` |
| Lưu vào server | `LocalForward 18080 127.0.0.1:8080` trong `~/.ssh/config` |
| Chỉ đường hầm, không terminal | `ssh -N` |

## 5. Lỗi thường gặp

**Đích sai phía.** Đích của Local là **nhìn từ server**. Muốn tới `bao-gia` trên chính server: `127.0.0.1`, không phải tên hay IP ngoài của server.

**«Lắng nghe tại» `0.0.0.0`.** Mọi máy trong mạng của bạn dùng được đường hầm của bạn; SSH Studio cảnh báo đỏ khi bạn làm vậy. Giữ `127.0.0.1`.

**Cổng đã có người dùng trên máy bạn.** Chọn cổng khác, hoặc để `0` cho SSH Studio tự chọn.

**Đường hầm dừng khi phiên SSH đứt.** Bật «Lưu vào server» để tự bật lại khi kết nối lại.

**Dùng đường hầm thay cho cấu hình đúng.** Đường hầm hợp cho người vận hành; người dùng cuối cần dịch vụ được đưa ra đúng cách (TLS, proxy, xác thực), như [LNX-049](/articles/linux-nginx-reverse-proxy-tls).

## Bài tập

1. Tạo Local forward `127.0.0.1:18022 → toinoi-app-02:22` trong SSH Studio. Đích này nhìn từ server có tới được không? Bạn vừa dựng được gì (gợi ý: Phần 6)?
2. Bật Dynamic `127.0.0.1:1080`, cấu hình trình duyệt dùng proxy SOCKS5 đó, mở `http://127.0.0.1:8080/suc-khoe`. Vì sao trình duyệt thấy được `bao-gia`?
3. Thêm `LocalForward 18080 127.0.0.1:8080` vào một khối `Host` trong `~/.ssh/config` trên máy bạn (nếu dùng OpenSSH). `ssh -N` mở đường hầm.
4. `sudo sshd -T -C user=an,host=x,addr=10.20.0.12 | grep -E 'allowtcpforwarding|gatewayports'`. Giá trị mặc định là gì?

## Đọc tiếp

[LNX-045 · Tường lửa với ufw và nftables](/articles/linux-tuong-lua-ufw-nftables): chỉ mở cổng cần thiết, và không tự khoá cổng SSH.
