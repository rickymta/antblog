---
slug: linux-dns-ping-curl
title: "DNS và kiểm tra kết nối: getent, dig, ping, curl và phân biệt lỗi DNS, lỗi mạng, lỗi ứng dụng"
summary: "Tên máy được phân giải ở đâu (/etc/hosts, nsswitch, resolv.conf), getent khác dig thế nào, ping kiểm tra máy kia có trả lời không, và bốn thông báo lỗi khác nhau của curl: không phân giải được tên, bị từ chối, hết thời gian chờ, ứng dụng trả lỗi HTTP; đọc curl -v và đo thời gian từng bước của một yêu cầu."
category: devops-linux
level: beginner
tags: [linux, lnx-042, dns, dig, ping, curl, troubleshooting, network, ssh-studio, toi-noi]
language: vi
---

«`bao-gia` hỏng rồi!», kênh của nhóm báo lúc 9 giờ sáng. Một tin như vậy có thể là bốn chuyện rất khác nhau: tên miền không phân giải được, máy chủ không tới được, dịch vụ không chạy, hoặc dịch vụ chạy nhưng trả lỗi. Mỗi chuyện có người sửa khác nhau (nhóm hạ tầng, nhóm mạng, người trực dịch vụ, đội phát triển). Bài này giúp An chỉ ra đúng chuyện nào trong vòng một phút.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-042 · Phần 5/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-ip-cong-ss">LNX-041</a>. Trạng thái đầu bài: <code>npm run setup -- 42</code>.</p>
</div>

## 1. Tên được phân giải ở đâu

Khi bạn gõ `curl http://toinoi-app-02`, chương trình cần đổi tên thành địa chỉ IP. Trên Linux, thứ tự hỏi nằm trong `/etc/nsswitch.conf`:

```text
an@toinoi-app-01:~$ grep -v '^#' /etc/resolv.conf | grep -v '^$'
nameserver 127.0.0.11
options ndots:0
an@toinoi-app-01:~$ grep -E '^(hosts):' /etc/nsswitch.conf
hosts:          files dns
an@toinoi-app-01:~$ getent hosts toinoi-app-02
10.20.0.12      toinoi-app-02
an@toinoi-app-01:~$ dig +short toinoi-app-02
10.20.0.12
an@toinoi-app-01:~$ dig toinoi-app-02 | grep -A1 'ANSWER SECTION'
;; ANSWER SECTION:
toinoi-app-02.          600     IN      A       10.20.0.12
```

- **`hosts: files dns`**: hỏi file `/etc/hosts` trước, không có thì hỏi DNS.
- **`/etc/resolv.conf`**: máy chủ DNS cần hỏi. Ở lab là `127.0.0.11`, DNS nội bộ của Docker, biết tên các container. Trên máy ảo đám mây thường là DNS của nhà cung cấp; trên Ubuntu cài bình thường là `127.0.0.53` của `systemd-resolved`.
- **`getent hosts TÊN`** phân giải **như chương trình thật** (theo `nsswitch`): đây là câu trả lời mà `curl`, `ssh`, `bao-gia` sẽ nhận được.
- **`dig`** hỏi **thẳng máy chủ DNS**, chi tiết hơn (`ANSWER SECTION`, thời gian sống `600` giây của bản ghi), `+short` chỉ in địa chỉ.

## 2. /etc/hosts đi trước DNS, nhưng dig không đọc nó

```text
an@toinoi-app-01:~$ echo '127.0.0.1 bao-gia.noi-bo' | sudo tee -a /etc/hosts
127.0.0.1 bao-gia.noi-bo
an@toinoi-app-01:~$ getent hosts bao-gia.noi-bo
127.0.0.1       bao-gia.noi-bo
an@toinoi-app-01:~$ dig +short bao-gia.noi-bo
an@toinoi-app-01:~$ curl -s http://bao-gia.noi-bo:8080/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 4281, "moi_truong": "production", "so_dich_vu": 4}
```

An đặt tên `bao-gia.noi-bo` cho chính server trong `/etc/hosts`. `getent` và `curl` dùng được tên mới ngay. **`dig` không trả gì**: nó không đọc `/etc/hosts`. Đây là nguồn nhầm lẫn kinh điển: «`dig` báo không có, sao `curl` vẫn chạy?», hoặc ngược lại. Kiểm tra cái chương trình thật dùng thì dùng `getent`; kiểm tra bản ghi trên DNS thì dùng `dig`.

`sudo tee -a` là cách ghi vào file của `root` từ một pipe: `sudo echo … >> /etc/hosts` không chạy, vì phần `>>` do shell của bạn làm, không có quyền `root` ([LNX-008](/articles/linux-chuyen-huong-pipe)).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dns-ping-curl/ssh-studio-01.webp" alt="Terminal của SSH Studio: resolv.conf với nameserver 127.0.0.11, nsswitch hosts files dns, getent hosts và dig +short toinoi-app-02 ra 10.20.0.12, ANSWER SECTION của dig, thêm bao-gia.noi-bo vào /etc/hosts bằng sudo tee -a, getent thấy 127.0.0.1 còn dig +short không trả gì, curl qua tên mới trả JSON" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>getent</code> theo <code>/etc/hosts</code>, <code>dig</code> thì không</figcaption></figure>

## 3. ping: máy kia có trả lời không

```text
an@toinoi-app-01:~$ ping -c 3 toinoi-app-02
PING toinoi-app-02 (10.20.0.12) 56(84) bytes of data.
64 bytes from toinoi-app-02.toinoi-linux-lab_lan (10.20.0.12): icmp_seq=1 ttl=64 time=0.078 ms
64 bytes from toinoi-app-02.toinoi-linux-lab_lan (10.20.0.12): icmp_seq=2 ttl=64 time=0.063 ms
64 bytes from toinoi-app-02.toinoi-linux-lab_lan (10.20.0.12): icmp_seq=3 ttl=64 time=0.062 ms

--- toinoi-app-02 ping statistics ---
3 packets transmitted, 3 received, 0% packet loss, time 2026ms
rtt min/avg/max/mdev = 0.062/0.067/0.078/0.007 ms
an@toinoi-app-01:~$ ping -c 2 -W 1 192.0.2.10
PING 192.0.2.10 (192.0.2.10) 56(84) bytes of data.

--- 192.0.2.10 ping statistics ---
2 packets transmitted, 0 received, 100% packet loss, time 1026ms
```

**`ping`** gửi gói ICMP «echo request» và đo thời gian phản hồi. `toinoi-app-02` trả lời cả 3 gói, mỗi gói dưới 0,1 ms (cùng máy chủ Docker). `192.0.2.10` thuộc dải dành riêng cho tài liệu, không ai dùng: 100% gói mất.

Ping thành công nghĩa là **máy** tới được, không nói gì về **dịch vụ**. Ngược lại, nhiều server và tường lửa chặn ICMP: ping thất bại chưa chắc máy đã chết. Kiểm tra dịch vụ thì kiểm tra đúng cổng (`nc -zv`, `curl`).

## 4. Bốn kiểu lỗi khác nhau

```text
an@toinoi-app-01:~$ curl -sS http://khong-co.toinoi.invalid/
curl: (6) Could not resolve host: khong-co.toinoi.invalid
an@toinoi-app-01:~$ curl -sS http://toinoi-app-02:8080/
curl: (7) Failed to connect to toinoi-app-02 port 8080 after 0 ms: Couldn't connect to server
an@toinoi-app-01:~$ curl -sS --connect-timeout 3 http://192.0.2.10:8080/
curl: (28) Failed to connect to 192.0.2.10 port 8080 after 3002 ms: Timeout was reached
an@toinoi-app-01:~$ curl -s -o /dev/null -w '%{http_code}\n' 'http://bao-gia.noi-bo:8080/bao-gia?dich_vu=xe-tai&km=1'
400
```

| Thông báo | Nghĩa | Ai xem |
|---|---|---|
| `(6) Could not resolve host` | **DNS**: tên không phân giải được | DNS, `/etc/hosts`, gõ sai tên |
| `(7) Failed to connect … Couldn't connect` sau **0 ms** | **Bị từ chối**: máy tới được, nhưng không ai nghe ở cổng đó | Dịch vụ không chạy, sai cổng, nghe ở `127.0.0.1` |
| `(28) … Timeout was reached` sau **3002 ms** | **Không có phản hồi**: gói tin đi mà không về | Máy tắt, sai địa chỉ, **tường lửa chặn im lặng** |
| HTTP `400`, `404`, `500` | **Ứng dụng** đã nhận yêu cầu và trả lỗi | Log của ứng dụng, đội phát triển |

Mã lỗi `(6)`, `(7)`, `(28)` là mã thoát của `curl` (`echo $?`), dùng được trong script. `-w '%{http_code}'` in mã HTTP; `-o /dev/null` bỏ phần thân. Mã `400` ở đây là lỗi **hợp lệ** của `bao-gia`: dịch vụ chạy tốt, chỉ là yêu cầu sai (`xe-tai` không có trong bảng giá, như [LNX-037](/articles/linux-journalctl-doc-log)).

Thời gian ghi trong thông báo cũng là manh mối: bị từ chối thì trả lời ngay, còn tường lửa chặn thì phải đợi tới hết `--connect-timeout`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dns-ping-curl/ssh-studio-02.webp" alt="Terminal của SSH Studio: ping toinoi-app-02 ba gói trả lời 0% packet loss, ping 192.0.2.10 mất 100%, curl tên .invalid báo Could not resolve host, curl toinoi-app-02:8080 báo Couldn't connect sau 0 ms, curl 192.0.2.10 báo Timeout sau 3002 ms, mã HTTP 400 cho dịch vụ không hợp lệ" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>ping</code> và bốn kiểu lỗi của <code>curl</code></figcaption></figure>

## 5. curl -v: từng bước của một yêu cầu

```text
an@toinoi-app-01:~$ curl -sv http://bao-gia.noi-bo:8080/suc-khoe 2>&1 | grep -E '^(\*|>|<) ' | grep -v -E 'Date:|Content-Length'
* Host bao-gia.noi-bo:8080 was resolved.
* IPv6: (none)
* IPv4: 127.0.0.1
*   Trying 127.0.0.1:8080...
* Connected to bao-gia.noi-bo (127.0.0.1) port 8080
> GET /suc-khoe HTTP/1.1
> Host: bao-gia.noi-bo:8080
> User-Agent: curl/8.5.0
> Accept: */*
>
* HTTP 1.0, assume close after body
< HTTP/1.0 200 OK
< Server: bao-gia/1.5.0 Python/3.12.3
< Content-Type: application/json; charset=utf-8
<
* Closing connection
an@toinoi-app-01:~$ curl -s -o /dev/null -w 'dns %{time_namelookup}s  ket noi %{time_connect}s  tong %{time_total}s\n' http://bao-gia.noi-bo:8080/suc-khoe
dns 0.000282s  ket noi 0.000374s  tong 0.000907s
```

**`curl -v`** in từng bước: dòng `*` là việc `curl` làm (phân giải tên ra `127.0.0.1`, kết nối tới cổng 8080), `>` là những gì gửi đi (dòng yêu cầu và header), `<` là những gì nhận về (`200 OK`, header `Server: bao-gia/1.5.0`). Lỗi xảy ra ở bước nào là thấy ngay.

**`-w`** với các biến thời gian chia nhỏ một yêu cầu chậm: phân giải tên mất bao lâu, kết nối bao lâu, tổng bao lâu. «Chậm vì DNS» và «chậm vì dịch vụ» trông giống nhau với người dùng, nhưng con số này tách chúng ra.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-dns-ping-curl/ssh-studio-03.webp" alt="Terminal của SSH Studio: curl -sv với các dòng resolved 127.0.0.1, Trying, Connected, yêu cầu GET /suc-khoe và header gửi đi, phản hồi HTTP/1.0 200 OK với Server bao-gia/1.5.0 Python, rồi curl -w in thời gian dns, kết nối và tổng" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>curl -v</code> và thời gian từng bước</figcaption></figure>

## 6. Quy trình một phút

1. `getent hosts TÊN`: tên có ra địa chỉ không? Không → DNS.
2. `nc -zv -w 3 TÊN CỔNG`: cổng có mở không? `refused` → dịch vụ; `timed out` → mạng hoặc tường lửa.
3. `curl -sv URL`: dịch vụ trả gì? Mã 4xx/5xx → ứng dụng, đọc log ([LNX-037](/articles/linux-journalctl-doc-log)).
4. Làm lại các bước đó **từ đúng nơi gặp lỗi**: từ server thì khác từ máy người dùng (DNS khác, tường lửa khác).

## 7. Lỗi thường gặp

**Dùng `dig` để kiểm tra tên có trong `/etc/hosts`.** `dig` không đọc file đó. Dùng `getent`.

**Kết luận «mạng hỏng» vì ping không trả lời.** Nhiều nơi chặn ICMP. Thử đúng cổng.

**Sửa `/etc/resolv.conf` bằng tay.** Trên Ubuntu nó thường do `systemd-resolved` hoặc netplan quản lý, sửa tay sẽ bị ghi đè. Sửa ở cấu hình gốc.

**Quên rằng DNS có bộ nhớ đệm.** Đổi bản ghi rồi mà máy vẫn nhận địa chỉ cũ tới hết thời gian sống (TTL).

## Bài tập

1. `dig +short ubuntu.com` và `dig +short ubuntu.com @1.1.1.1` (cần Internet). Kết quả có khác không? Vì sao có thể khác?
2. `curl -sS -o /dev/null -w '%{http_code} %{redirect_url}\n' http://ubuntu.com`. Mã 301 nghĩa là gì?
3. Xoá dòng `bao-gia.noi-bo` khỏi `/etc/hosts` bằng `sudo sed -i '/bao-gia.noi-bo/d' /etc/hosts` ([LNX-014](/articles/linux-sed-sua-van-ban)), rồi chạy lại `curl`. Lỗi nào?
4. Viết script `kiem-tra-url URL` in một dòng: `DNS`, `TU CHOI`, `HET GIO` hay mã HTTP, dựa trên mã thoát của `curl`.

## Đọc tiếp

[LNX-043 · Dung lượng đĩa: df, du và tìm thứ chiếm chỗ](/articles/linux-df-du-dung-luong).
