---
slug: linux-cap-nhat-he-thong-an-toan
title: "Cập nhật hệ thống an toàn: bản vá bảo mật, unattended-upgrades, khi nào cần khởi động lại và cách quay lui một gói"
summary: "Xem gói nào cần cập nhật và gói nào là bản vá bảo mật, bật unattended-upgrades với chính sách riêng (không tự khởi động lại, giữ nginx theo lịch triển khai), chạy thử bằng --dry-run, đọc /var/run/reboot-required, và quay lui nginx về bản trước bằng --allow-downgrades kèm apt-mark hold."
category: devops-linux
level: intermediate
tags: [linux, lnx-054, apt, unattended-upgrades, security, patching, ssh-studio, toi-noi]
language: vi
---

Một lỗ hổng OpenSSL được công bố sáng thứ Hai. Ubuntu phát hành bản vá trong ngày; server nào không cài thì vẫn mở cửa cho tới khi có người nhớ ra. Với ba server, An không muốn việc vá phụ thuộc vào trí nhớ. Nhưng Bình cũng dặn: nginx đứng trước `bao-gia`, đừng để nó tự nâng phiên bản giữa giờ cao điểm.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-054 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-apt-cai-phan-mem">LNX-033</a> (apt), <a href="/articles/linux-cron-systemd-timer">LNX-038</a> (timer). Trạng thái đầu bài: <code>npm run setup -- 54</code>. Bài cài gói từ kho của Ubuntu (cần Internet). Danh sách gói cần cập nhật tuỳ thời điểm: ảnh lab vừa dựng thường đã mới, server chạy lâu thường có hàng chục gói.</p>
</div>

## 1. Có gì cần cập nhật

```text
an@toinoi-app-01:~$ sudo apt update 2>&1 | tail -n 1
All packages are up to date.
an@toinoi-app-01:~$ apt list --upgradable 2>/dev/null | tail -n +2 | wc -l
0
an@toinoi-app-01:~$ apt list --upgradable 2>/dev/null | grep -c -- '-security'
0
an@toinoi-app-01:~$ apt list --upgradable 2>/dev/null | head -n 6
Listing...
```

- `apt update` chỉ tải danh sách gói mới nhất, không cài gì ([LNX-033](/articles/linux-apt-cai-phan-mem)).
- `apt list --upgradable` liệt kê gói có bản mới. Gói đến từ kho `…-security` là **bản vá bảo mật**: ưu tiên cao nhất. Các gói khác (`…-updates`) là sửa lỗi thường.
- Lab vừa dựng từ ảnh mới nên danh sách trống. Trên một server chạy vài tháng không vá, đếm được hàng chục dòng.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cap-nhat-he-thong-an-toan/ssh-studio-01.webp" alt="Terminal của SSH Studio: apt update, đếm gói có thể nâng cấp và gói bảo mật, apt list --upgradable" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · gói nào cần cập nhật</figcaption></figure>

## 2. Vá tự động: unattended-upgrades

**unattended-upgrades** là cách chuẩn của Ubuntu để tự cài bản vá. Hai timer của systemd làm việc: `apt-daily.timer` tải danh sách gói, `apt-daily-upgrade.timer` cài.

Bật nó bằng `20auto-upgrades`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cap-nhat-he-thong-an-toan/ssh-studio-02.webp" alt="Vim mở /etc/apt/apt.conf.d/20auto-upgrades qua sudoedit trong SSH Studio: APT::Periodic::Update-Package-Lists &quot;1&quot;, Unattended-Upgrade &quot;1&quot;, AutocleanInterval &quot;7&quot;" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>20auto-upgrades</code>: bật vá hằng ngày</figcaption></figure>

```text
// Vá tự động trên toinoi-app-01 (LNX-054): mỗi ngày cập nhật danh sách gói và cài bản vá.
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
APT::Periodic::AutocleanInterval "7";
```

Chính sách riêng của Tới Nơi ghi vào một file mới, **không sửa** `50unattended-upgrades` của gói (để lần nâng cấp gói sau không đè mất):

```text
// Chính sách vá của Tới Nơi (LNX-054). Ghi đè 50unattended-upgrades, không sửa file gốc của gói.
// Không tự khởi động lại: nhóm vận hành khởi động lại trong khung giờ đã báo (runbook, LNX-060).
Unattended-Upgrade::Automatic-Reboot "false";
// nginx đứng trước bao-gia: cập nhật cùng lịch triển khai, không để máy tự nâng.
Unattended-Upgrade::Package-Blacklist {
    "nginx";
    "nginx-common";
};
Unattended-Upgrade::Remove-Unused-Dependencies "true";
Unattended-Upgrade::SyslogEnable "true";
```

File trong `/etc/apt/apt.conf.d/` được đọc theo thứ tự tên: `52…` sau `50…` nên thắng.

```text
an@toinoi-app-01:~$ grep -E '^\s*"\$\{distro_id\}' /etc/apt/apt.conf.d/50unattended-upgrades
        "${distro_id}:${distro_codename}";
        "${distro_id}:${distro_codename}-security";
        "${distro_id}ESMApps:${distro_codename}-apps-security";
        "${distro_id}ESM:${distro_codename}-infra-security";
an@toinoi-app-01:~$ sudo unattended-upgrade --dry-run -v 2>&1 | tail -n 3
Initial whitelist (not strict):
No packages found that can be upgraded unattended and no pending auto-removals
The list of kept packages can't be calculated in dry-run mode.
an@toinoi-app-01:~$ systemctl list-timers 'apt-daily*' --no-pager | head -n 3
NEXT                        LEFT LAST PASSED UNIT                    ACTIVATES
Tue 2026-11-24 23:26:17 +07  13h -         - apt-daily.timer         apt-daily.service
Wed 2026-11-25 06:45:02 +07  21h -         - apt-daily-upgrade.timer apt-daily-upgrade.service
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cap-nhat-he-thong-an-toan/ssh-studio-07.webp" alt="Terminal của SSH Studio: apt install unattended-upgrades, sudoedit 20auto-upgrades và 52unattended-toinoi, grep 50unattended-upgrades cho các nguồn được vá tự động: distro_codename, -security, ESMApps, ESM" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cài unattended-upgrades và xem nguồn nào được vá tự động</figcaption></figure>

- **Allowed-Origins** mặc định của Ubuntu: chỉ kho chính và kho `-security` (và ESM nếu có đăng ký). Bản sửa lỗi thường (`-updates`) không tự cài, trừ khi bạn thêm.
- `--dry-run -v` chạy thử: in những gì sẽ cài mà không cài. Luôn chạy thử sau khi sửa cấu hình.
- Hai timer có giờ chạy ngẫu nhiên trong ngày, để hàng triệu máy không cùng tải một lúc.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cap-nhat-he-thong-an-toan/ssh-studio-03.webp" alt="Terminal của SSH Studio: unattended-upgrade --dry-run -v báo No packages found that can be upgraded unattended, list-timers 'apt-daily*' cho apt-daily.timer và apt-daily-upgrade.timer chạy hằng ngày" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · chạy thử unattended-upgrade và hai timer của apt</figcaption></figure>

## 3. Có cần khởi động lại không

```text
an@toinoi-app-01:~$ ls /var/run/reboot-required 2>/dev/null || echo "chua can khoi dong lai"
chua can khoi dong lai
an@toinoi-app-01:~$ cat /var/run/reboot-required.pkgs 2>/dev/null || true
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cap-nhat-he-thong-an-toan/ssh-studio-05.webp" alt="Terminal của SSH Studio: ls /var/run/reboot-required không có, in chua can khoi dong lai, cat reboot-required.pkgs không in gì" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · chưa có bản vá nào đòi khởi động lại</figcaption></figure>

Bản vá kernel, glibc, systemd chỉ có tác dụng khi khởi động lại. Ubuntu đánh dấu bằng file **`/var/run/reboot-required`**, và `reboot-required.pkgs` ghi gói nào gây ra. Vì Tới Nơi tắt `Automatic-Reboot`, runbook ([LNX-060](/articles/linux-thuc-hanh-runbook-van-hanh)) có một mục: thấy file này thì khởi động lại trong khung giờ đã báo. Bản vá cho một thư viện mà dịch vụ đang dùng thì khởi động lại riêng dịch vụ đó là đủ (công cụ `needrestart` liệt kê chúng).

Trong lab, kernel là của máy chạy Docker, nên file này hiếm khi xuất hiện.

## 4. Quay lui một bản vá

Giả sử bản nginx mới làm hỏng một cấu hình. `apt-cache madison` liệt kê mọi bản có trong kho:

```text
an@toinoi-app-01:~$ apt-cache madison nginx
     nginx | 1.24.0-2ubuntu7.21 | http://archive.ubuntu.com/ubuntu noble-updates/main amd64 Packages
     nginx | 1.24.0-2ubuntu7.21 | http://security.ubuntu.com/ubuntu noble-security/main amd64 Packages
     nginx | 1.24.0-2ubuntu7 | http://archive.ubuntu.com/ubuntu noble/main amd64 Packages
an@toinoi-app-01:~$ sudo apt install -y --allow-downgrades nginx=1.24.0-2ubuntu7 nginx-common=1.24.0-2ubuntu7 2>&1 | grep -E '^(Setting up|dpkg: warning)'
dpkg: warning: downgrading nginx from 1.24.0-2ubuntu7.21 to 1.24.0-2ubuntu7
dpkg: warning: downgrading nginx-common from 1.24.0-2ubuntu7.21 to 1.24.0-2ubuntu7
Setting up nginx (1.24.0-2ubuntu7) ...
Setting up nginx-common (1.24.0-2ubuntu7) ...
an@toinoi-app-01:~$ nginx -v
nginx version: nginx/1.24.0 (Ubuntu)
an@toinoi-app-01:~$ sudo apt-mark hold nginx nginx-common
nginx set on hold.
nginx-common set on hold.
an@toinoi-app-01:~$ apt list --upgradable 2>/dev/null | grep nginx
nginx-common/noble-updates,noble-security 1.24.0-2ubuntu7.21 all [upgradable from: 1.24.0-2ubuntu7]
nginx/noble-updates,noble-security 1.24.0-2ubuntu7.21 amd64 [upgradable from: 1.24.0-2ubuntu7]
an@toinoi-app-01:~$ sudo nginx -t 2>&1 | tail -n 1
nginx: configuration file /etc/nginx/nginx.conf test is successful
```

- Bản `…ubuntu7` trong kho `noble` (bản phát hành gốc) vẫn còn, cạnh bản mới trong `noble-updates`/`noble-security`.
- `apt install GÓI=PHIÊN_BẢN --allow-downgrades` cài bản cũ. nginx và nginx-common phải cùng phiên bản, nên hạ cả hai.
- **`apt-mark hold`** giữ gói ở bản đó: `apt upgrade` và unattended-upgrades đều bỏ qua nó, `apt list --upgradable` vẫn báo có bản mới để bạn không quên.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cap-nhat-he-thong-an-toan/ssh-studio-04.webp" alt="Terminal của SSH Studio: apt-cache madison nginx với ba dòng, apt install nginx=1.24.0-2ubuntu7 --allow-downgrades, nginx -v, apt-mark hold, apt list --upgradable vẫn thấy nginx, nginx -t successful" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · quay lui nginx và giữ ở bản cũ</figcaption></figure>

Khi bản sửa đã ổn, bỏ giữ và nâng lại:

```text
an@toinoi-app-01:~$ sudo apt-mark unhold nginx nginx-common
Canceled hold on nginx.
Canceled hold on nginx-common.
an@toinoi-app-01:~$ sudo apt install -y nginx nginx-common 2>&1 | grep -E '^Setting up nginx '
Setting up nginx (1.24.0-2ubuntu7.21) ...
an@toinoi-app-01:~$ nginx -v
nginx version: nginx/1.24.0 (Ubuntu)
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cap-nhat-he-thong-an-toan/ssh-studio-06.webp" alt="Terminal của SSH Studio: apt-mark unhold nginx nginx-common, apt install nâng nginx lên 1.24.0-2ubuntu7.21, nginx -v" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · bản vá đã ổn: bỏ giữ và nâng lại nginx</figcaption></figure>

Quay lui bằng apt chỉ được khi bản cũ còn trong kho. Kho Ubuntu giữ bản phát hành gốc và bản mới nhất, không giữ mọi bản trung gian. Muốn quay lui tuỳ ý thì cần ảnh chụp đĩa (snapshot) của máy ảo trước khi vá, hoặc kho gói riêng.

## 5. Lịch vá của Tới Nơi

| Loại | Cách | Khi nào |
|---|---|---|
| Bản vá bảo mật | unattended-upgrades | Hằng ngày, tự động |
| nginx | Tay, cùng lịch triển khai `bao-gia` | Thứ Ba, sau khi thử trên app02 |
| Khởi động lại | Tay, khi có `reboot-required` | 22:00–23:00, báo trước |
| Nâng bản phát hành Ubuntu | Kế hoạch riêng | Hai năm một lần (LTS) |

## 6. Lỗi thường gặp

**Sửa `50unattended-upgrades`.** Lần nâng gói sau hỏi đè file hoặc giữ file cũ thiếu thiết lập mới. Ghi vào file riêng.

**Bật `Automatic-Reboot` trên server duy nhất của dịch vụ.** Dịch vụ dừng lúc không ai trực.

**`apt-mark hold` rồi quên.** Gói không bao giờ được vá nữa. Ghi lý do và ngày bỏ giữ vào runbook.

**Không bao giờ đọc log.** Kết quả của unattended-upgrades nằm ở `/var/log/unattended-upgrades/`.

## Bài tập

1. `less /var/log/unattended-upgrades/unattended-upgrades.log` sau một ngày. Gói nào được cài? Gói nào bị bỏ qua vì Package-Blacklist?
2. Thêm `"${distro_id}:${distro_codename}-updates";` vào Allowed-Origins trong `52unattended-toinoi`. Ưu, nhược điểm?
3. `sudo apt install needrestart` rồi chạy `sudo needrestart -b`. Dịch vụ nào đang dùng thư viện cũ?
4. Áp `20auto-upgrades` và `52unattended-toinoi` cho app02 và bastion bằng Ansible ([LNX-056](/articles/linux-nhieu-server-ansible-toi-thieu)).

## Đọc tiếp

[LNX-055 · Gia cố server: checklist tối thiểu và fail2ban](/articles/linux-gia-co-server-fail2ban).
