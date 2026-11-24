---
slug: linux-nginx-reverse-proxy-tls
title: "TLS và reverse proxy nginx cho bao-gia: chứng chỉ tự ký, proxy_pass, chuyển HTTP sang HTTPS và kiểm tra bằng openssl"
summary: "Đặt nginx trước bao-gia: cài bằng apt, tạo chứng chỉ tự ký có subjectAltName bằng openssl cho lab, viết site với server 80 chuyển hướng 301 và server 443 có ssl_certificate và proxy_pass tới 127.0.0.1:8080, nginx -t trước reload, vì sao curl từ chối chứng chỉ tự ký và cách tin nó bằng --cacert, xem bắt tay TLS bằng openssl s_client, access log, mở 443 cho mạng nội bộ bằng ufw."
category: devops-linux
level: advanced
tags: [linux, lnx-049, nginx, reverse-proxy, tls, https, openssl, certificate, ssh-studio, toi-noi]
language: vi
---

Đội ứng dụng cuối cùng cũng cần gọi `bao-gia` từ máy khác trong mạng nội bộ. Bình không cho mở thẳng cổng 8080: `bao-gia` không mã hoá, không giới hạn gì, và chỉ biết nghe ở `127.0.0.1` ([LNX-041](/articles/linux-ip-cong-ss)). Cách làm chuẩn là đặt một **reverse proxy** phía trước: nginx nhận kết nối HTTPS ở cổng 443, giải mã TLS, ghi log, rồi chuyển yêu cầu vào `127.0.0.1:8080`.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-049 · Phần 5/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-apt-cai-phan-mem">LNX-033</a> (apt), <a href="/articles/linux-tuong-lua-ufw-nftables">LNX-045</a> (ufw). Trạng thái đầu bài: <code>npm run setup -- 49</code> (cần Internet để cài nginx). Lab dùng <b>chứng chỉ tự ký</b> cho tên <code>bao-gia.toinoi.lab</code>; server có tên miền công khai thì dùng chứng chỉ của một CA như Let's Encrypt (bài nói ở cuối).</p>
</div>

## 1. Cài nginx

```text
an@toinoi-app-01:~$ sudo apt update 2>&1 | tail -n 1
All packages are up to date.
an@toinoi-app-01:~$ sudo apt install -y nginx 2>&1 | grep -E '^Setting up nginx'
Setting up nginx-common (1.24.0-2ubuntu7.21) ...
Setting up nginx (1.24.0-2ubuntu7.21) ...
an@toinoi-app-01:~$ systemctl is-active nginx
active
an@toinoi-app-01:~$ sudo ss -tlnpH | grep nginx | awk '{ split($6, p, "\""); print $4, p[2] }'
0.0.0.0:80 nginx
[::]:80 nginx
an@toinoi-app-01:~$ ps -C nginx -o user=,cmd= | sort | uniq -c
      1 root     nginx: master process /usr/sbin/nginx -g daemon on; master_process on;
     16 www-data nginx: worker process
an@toinoi-app-01:~$ curl -s -o /dev/null -w '%{http_code} %{content_type}\n' http://127.0.0.1/
200 text/html
```

Trên Ubuntu, cài xong là nginx chạy ngay (dịch vụ `nginx.service`, [LNX-035](/articles/linux-systemd-systemctl)) với trang mặc định ở cổng 80. Một tiến trình chủ (*master*) chạy bằng `root` (để mở cổng 80, 443) và các tiến trình con (*worker*) chạy bằng `www-data` làm việc thật: `ps -C nginx` đếm được 16 worker vì mặc định `worker_processes auto` lấy theo số nhân, mà container thấy 16 nhân của máy chạy Docker.

`ss -tlnpH` bỏ dòng tiêu đề; `awk` ([LNX-015](/articles/linux-awk-xu-ly-log)) cắt cột tiến trình `users:(("nginx",pid=…` theo dấu `"` và chỉ in địa chỉ nghe cùng tên chương trình. Không có `awk`, mỗi dòng liệt kê đủ 17 PID của nginx và tràn cả màn hình.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nginx-reverse-proxy-tls/ssh-studio-01.webp" alt="Terminal của SSH Studio: apt update, apt install nginx, systemctl is-active nginx active, ss qua awk thấy nginx nghe 0.0.0.0:80 và [::]:80, ps -C nginx đếm 1 master chạy bằng root và 16 worker chạy bằng www-data, curl 127.0.0.1 trả 200 text/html" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cài <code>nginx</code></figcaption></figure>

## 2. Chứng chỉ tự ký cho lab

**TLS** mã hoá kết nối và chứng minh danh tính server bằng một **chứng chỉ**: khoá công khai của server cộng với tên miền, được một bên thứ ba (**CA**) ký. Trình duyệt và `curl` tin một danh sách CA có sẵn. Trong lab không có tên miền thật, nên An tự ký: chứng chỉ hợp lệ về mặt kỹ thuật, nhưng không ai tin nó cho tới khi được dặn phải tin.

```bash
sudo openssl req -x509 -newkey rsa:2048 -nodes -days 365 \
  -subj '/CN=bao-gia.toinoi.lab' -addext 'subjectAltName=DNS:bao-gia.toinoi.lab' \
  -keyout /etc/ssl/private/bao-gia.toinoi.lab.key -out /etc/ssl/certs/bao-gia.toinoi.lab.crt
```

| Tuỳ chọn | Ý nghĩa |
|---|---|
| `req -x509` | Tạo thẳng một chứng chỉ tự ký (không qua CA) |
| `-newkey rsa:2048 -nodes` | Tạo khoá RSA 2048 bit mới, **không** mã hoá khoá (nginx cần đọc lúc khởi động) |
| `-days 365` | Hiệu lực một năm |
| `-subj '/CN=…'` | Tên chung |
| `-addext 'subjectAltName=DNS:…'` | **SAN**: tên miền mà client thực sự kiểm tra. Thiếu SAN, `curl` và trình duyệt hiện đại từ chối dù CN đúng |

```text
an@toinoi-app-01:~$ sudo ls -l /etc/ssl/private/bao-gia.toinoi.lab.key /etc/ssl/certs/bao-gia.toinoi.lab.crt
-rw-r--r-- 1 root root 1180 Nov 24 08:13 /etc/ssl/certs/bao-gia.toinoi.lab.crt
-rw------- 1 root root 1704 Nov 24 08:13 /etc/ssl/private/bao-gia.toinoi.lab.key
an@toinoi-app-01:~$ openssl x509 -in /etc/ssl/certs/bao-gia.toinoi.lab.crt -noout -subject -issuer -dates
subject=CN = bao-gia.toinoi.lab
issuer=CN = bao-gia.toinoi.lab
notBefore=Nov 24 01:13:16 2026 GMT
notAfter=Nov 24 01:13:16 2027 GMT
```

Khoá riêng `600` chỉ `root` đọc, chứng chỉ `644` ai cũng đọc. `subject` và `issuer` trùng nhau: dấu hiệu của chứng chỉ tự ký.

## 3. Cấu hình site

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nginx-reverse-proxy-tls/ssh-studio-02.webp" alt="Vim mở /etc/nginx/sites-available/bao-gia qua sudoedit trong SSH Studio: khối server listen 80 return 301 https, khối server listen 443 ssl với ssl_certificate, ssl_certificate_key, ssl_protocols TLSv1.2 TLSv1.3, access_log và location / proxy_pass http://127.0.0.1:8080 cùng các proxy_set_header" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · site <code>bao-gia</code> của nginx</figcaption></figure>

```nginx
# nginx đứng trước bao-gia (LNX-049): nhận HTTPS ở cổng 443, chuyển vào 127.0.0.1:8080.
server {
    listen 80;
    server_name bao-gia.toinoi.lab;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name bao-gia.toinoi.lab;

    ssl_certificate     /etc/ssl/certs/bao-gia.toinoi.lab.crt;
    ssl_certificate_key /etc/ssl/private/bao-gia.toinoi.lab.key;
    ssl_protocols       TLSv1.2 TLSv1.3;

    access_log /var/log/nginx/bao-gia.access.log;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
    }
}
```

- Khối đầu: mọi yêu cầu HTTP được **chuyển hướng 301** sang HTTPS.
- Khối sau: nghe 443 với TLS, chỉ bật TLS 1.2 và 1.3, log riêng, và **`proxy_pass`** chuyển mọi yêu cầu vào `bao-gia`. Các `X-Forwarded-*` cho ứng dụng biết địa chỉ thật của client và giao thức gốc (với `bao-gia`, mọi yêu cầu giờ đến từ `127.0.0.1`).

Quy ước của Ubuntu: site viết ở `sites-available`, bật bằng symlink trong `sites-enabled`, như `.wants` của systemd ([LNX-035](/articles/linux-systemd-systemctl)).

```text
an@toinoi-app-01:~$ sudo ln -s /etc/nginx/sites-available/bao-gia /etc/nginx/sites-enabled/bao-gia
an@toinoi-app-01:~$ sudo rm /etc/nginx/sites-enabled/default
an@toinoi-app-01:~$ sudo nginx -t
nginx: the configuration file /etc/nginx/nginx.conf syntax is ok
nginx: configuration file /etc/nginx/nginx.conf test is successful
an@toinoi-app-01:~$ sudo systemctl reload nginx
```

**`nginx -t`** kiểm tra cú pháp và đường dẫn chứng chỉ **trước** khi `reload`: cùng tinh thần `sshd -t`, `visudo -c`. `reload` thay cấu hình không cắt kết nối đang có.

## 4. Thử từ chính server

```text
an@toinoi-app-01:~$ echo '127.0.0.1 bao-gia.toinoi.lab' | sudo tee -a /etc/hosts
127.0.0.1 bao-gia.toinoi.lab
an@toinoi-app-01:~$ curl -sI http://bao-gia.toinoi.lab/suc-khoe | head -n 3
HTTP/1.1 301 Moved Permanently
Server: nginx/1.24.0 (Ubuntu)
Date: Tue, 24 Nov 2026 01:13:17 GMT
an@toinoi-app-01:~$ curl -sS https://bao-gia.toinoi.lab/suc-khoe
curl: (60) SSL certificate problem: self-signed certificate
More details here: https://curl.se/docs/sslcerts.html

curl failed to verify the legitimacy of the server and therefore could not
establish a secure connection to it. To learn more about this situation and
how to fix it, please visit the web page mentioned above.
an@toinoi-app-01:~$ curl -s --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 4281, "moi_truong": "production", "so_dich_vu": 4}
an@toinoi-app-01:~$ echo | openssl s_client -connect bao-gia.toinoi.lab:443 -servername bao-gia.toinoi.lab 2>/dev/null | grep -E '^(subject|issuer)=|Protocol|Cipher is'
subject=CN = bao-gia.toinoi.lab
issuer=CN = bao-gia.toinoi.lab
New, TLSv1.3, Cipher is TLS_AES_256_GCM_SHA384
an@toinoi-app-01:~$ sudo tail -n 2 /var/log/nginx/bao-gia.access.log
127.0.0.1 - - [24/Nov/2026:08:13:17 +0700] "GET /suc-khoe HTTP/1.1" 200 101 "-" "curl/8.5.0"
127.0.0.1 - - [24/Nov/2026:08:13:17 +0700] "" 400 0 "-" "-"
```

- HTTP trả **`301 Moved Permanently`** sang HTTPS.
- `curl https://…` từ chối: **`SSL certificate problem: self-signed certificate`**. Đó là `curl` làm đúng việc: không ai nó tin đã ký chứng chỉ này.
- **`--cacert FILE`**: tin đúng chứng chỉ này (như cài nó vào máy người dùng). Giờ thì thành công, và `/suc-khoe` trả lời qua nginx. Đừng tập thói quen `curl -k` (bỏ kiểm tra): với `-k`, kẻ đứng giữa ([LNX-023](/articles/linux-host-key-known-hosts)) cũng «thành công».
- **`openssl s_client`** xem bắt tay TLS: chứng chỉ server đưa ra (`subject`, `issuer`), giao thức `TLSv1.3`, bộ mã hoá.
- Access log ghi từng yêu cầu. Dòng `"" 400` là kết nối của `openssl s_client` đóng mà không gửi yêu cầu HTTP nào.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nginx-reverse-proxy-tls/ssh-studio-03.webp" alt="Terminal của SSH Studio: thêm bao-gia.toinoi.lab vào /etc/hosts, curl -sI cho 301 Moved Permanently, curl https báo SSL certificate problem self-signed certificate, curl --cacert trả JSON của bao-gia, openssl s_client với subject và issuer CN bao-gia.toinoi.lab, TLSv1.3, access log với GET /suc-khoe 200" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · HTTPS qua nginx, và chứng chỉ tự ký</figcaption></figure>

## 5. Mở cho mạng nội bộ

```text
an@toinoi-app-01:~$ sudo ufw allow from 10.20.0.0/24 to any port 443 proto tcp comment 'https bao-gia noi bo'
Rule added
an@toinoi-app-01:~$ sudo ufw status | grep 443
443/tcp                    ALLOW       10.20.0.0/24               # https bao-gia noi bo
an@toinoi-app-02:~$ curl -sk --resolve bao-gia.toinoi.lab:443:10.20.0.11 https://bao-gia.toinoi.lab/suc-khoe
{"trang_thai": "ok", "phien_ban": "1.5.0", "pid": 4281, "moi_truong": "production", "so_dich_vu": 4}
an@toinoi-app-02:~$ curl -sS --connect-timeout 3 http://toinoi-app-01:8080/suc-khoe
curl: (28) Failed to connect to toinoi-app-01 port 8080 after 3002 ms: Timeout was reached
an@toinoi-app-01:~$ sudo tail -n 1 /var/log/nginx/bao-gia.access.log
10.20.0.12 - - [24/Nov/2026:08:13:18 +0700] "GET /suc-khoe HTTP/1.1" 200 101 "-" "curl/8.5.0"
```

- Tường lửa ([LNX-045](/articles/linux-tuong-lua-ufw-nftables)) chỉ mở 443 cho mạng `10.20.0.0/24`. Cổng 80 **không** mở từ ngoài: trong mạng nội bộ chỉ dùng HTTPS.
- Từ `toinoi-app-02`, `--resolve` gán tên cho địa chỉ ngay trong lệnh (thay cho sửa `/etc/hosts`). HTTPS qua nginx trả lời; cổng 8080 vẫn bị chặn (timeout).
- Access log thấy yêu cầu đến từ `10.20.0.12`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nginx-reverse-proxy-tls/ssh-studio-04.webp" alt="Terminal của SSH Studio: ufw allow 443 từ 10.20.0.0/24, ufw status có dòng 443 ALLOW 10.20.0.0/24, tail access log với yêu cầu GET /suc-khoe từ 10.20.0.12 trả 200" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · mở 443 cho mạng nội bộ, yêu cầu từ app02 vào log</figcaption></figure>

### Với tên miền thật

Server có tên miền công khai trỏ về nó thì dùng **Let's Encrypt** (miễn phí, tự động): `sudo apt install certbot python3-certbot-nginx`, rồi `sudo certbot --nginx -d ten-mien`. certbot lấy chứng chỉ, sửa cấu hình nginx và cài timer gia hạn ([LNX-038](/articles/linux-cron-systemd-timer)). Chứng chỉ chỉ sống 90 ngày; gia hạn tự động không chạy là trang web hỏng đúng ngày hết hạn.

## 6. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Kiểm tra cấu hình | `sudo nginx -t` |
| Áp dụng | `sudo systemctl reload nginx` |
| Bật / tắt site | `ln -s` vào `sites-enabled` / xoá symlink |
| Xem chứng chỉ | `openssl x509 -in FILE -noout -subject -issuer -dates -ext subjectAltName` |
| Xem bắt tay TLS | `openssl s_client -connect TÊN:443 -servername TÊN` |
| Tin một chứng chỉ khi thử | `curl --cacert FILE https://…` |
| Log | `/var/log/nginx/*.access.log`, `error.log` |

## 7. Lỗi thường gặp

**`502 Bad Gateway`.** nginx chạy, nhưng dịch vụ phía sau không trả lời: `bao-gia` dừng, hoặc sai cổng trong `proxy_pass`. Xem `error.log` của nginx.

**Thiếu `subjectAltName`.** Client từ chối dù `CN` đúng tên.

**`curl -k` trong script.** Tắt hết kiểm tra danh tính server. Dùng `--cacert`.

**Khoá riêng đọc được bởi người khác.** Phải `600`, thuộc `root`.

**`reload` không qua `nginx -t`.** Cấu hình lỗi, nginx giữ cấu hình cũ, và bạn không biết vì sao thay đổi «không ăn».

## Bài tập

1. Tắt `bao-gia` (`sudo systemctl stop bao-gia`) rồi `curl --cacert … https://bao-gia.toinoi.lab/suc-khoe`. Mã gì? `sudo tail /var/log/nginx/error.log` ghi gì? Bật lại.
2. Thêm `add_header Strict-Transport-Security "max-age=31536000" always;` vào khối 443. `curl -sI` thấy gì? HSTS bảo vệ điều gì?
3. Thêm khối `location /noi-bo/ { allow 10.20.0.0/24; deny all; … }`. Thử từ app01 (qua `127.0.0.1`) và từ app02.
4. Chép chứng chỉ sang app02 (`scp`) rồi gọi bằng `--cacert` thay cho `-k`.

## Đọc tiếp

[LNX-050 · Thực hành: mạng và giám sát cho toinoi-app-01](/articles/linux-thuc-hanh-mang-giam-sat).
