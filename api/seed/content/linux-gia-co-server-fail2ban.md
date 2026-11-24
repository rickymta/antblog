---
slug: linux-gia-co-server-fail2ban
title: "Gia cố server: checklist tối thiểu và fail2ban chặn IP dò đăng nhập SSH"
summary: "Rà một server theo năm câu hỏi (cổng nào mở, tường lửa cho ai, sshd nhận gì, ai có shell và sudo, dịch vụ nào bật sẵn), rồi cài fail2ban: jail sshd đọc journal, chặn bằng ufw, ignoreip để không tự khoá mình và bastion, thử bằng các lần đăng nhập sai từ app02, đọc log và gỡ chặn."
category: devops-linux
level: intermediate
tags: [linux, lnx-055, security, hardening, fail2ban, ufw, sshd, ssh-studio, toi-noi]
language: vi
---

Log của `sshd` trên `toinoi-app-01` có hàng trăm dòng `Invalid user admin`, `Invalid user oracle`… mỗi ngày: máy dò tự động quét cả Internet, thử các tài khoản phổ biến. Phần 3 đã làm cho việc dò mật khẩu vô ích (chỉ key và OTP). Nhưng log đầy rác che mất những dòng đáng đọc, và mỗi lần thử vẫn tốn tài nguyên. An làm hai việc: rà lại toàn bộ server theo một checklist ngắn, và cài **fail2ban** để chặn những IP cứ thử mãi.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-055 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-sshd-config-an-toan">LNX-025</a> (sshd), <a href="/articles/linux-ssh-mfa-totp">LNX-029</a> (OTP), <a href="/articles/linux-tuong-lua-ufw-nftables">LNX-045</a> (ufw). Trạng thái đầu bài: <code>npm run setup -- 55</code>. Bài cài fail2ban từ kho của Ubuntu (cần Internet). <code>toinoi-app-02</code> đóng vai máy dò.</p>
</div>

## 1. Checklist: năm câu hỏi

```text
an@toinoi-app-01:~$ sudo ss -tlnpH | grep -v 127.0.0 | awk '{ split($6, p, "\""); print $4, p[2] }'
0.0.0.0:443 nginx
0.0.0.0:22 sshd
0.0.0.0:80 nginx
[::]:22 sshd
an@toinoi-app-01:~$ sudo ufw status | head -n 7
Status: active

To                         Action      From
--                         ------      ----
OpenSSH                    LIMIT       Anywhere
443/tcp                    ALLOW       10.20.0.0/24               # https bao-gia noi bo
OpenSSH (v6)               LIMIT       Anywhere (v6)
an@toinoi-app-01:~$ sudo sshd -T | grep -E '^(permitrootlogin|passwordauthentication|authenticationmethods|maxauthtries|x11forwarding|allowagentforwarding) '
maxauthtries 3
permitrootlogin no
passwordauthentication no
x11forwarding no
allowagentforwarding yes
authenticationmethods publickey,keyboard-interactive
an@toinoi-app-01:~$ awk -F: '$7 !~ /(nologin|false)$/ { print $1, $7 }' /etc/passwd
root /bin/bash
sync /bin/sync
an /bin/bash
binh /bin/bash
cuong /bin/bash
kiemtoan /bin/bash
deploy /bin/bash
an@toinoi-app-01:~$ getent group sudo
sudo:x:27:an,binh
an@toinoi-app-01:~$ systemctl list-unit-files --type=service --state=enabled --no-pager --no-legend | awk '{ print $1 }' | paste -sd ' '
bao-gia.service cron.service e2scrub_reap.service getty@.service nginx.service ssh.service systemd-pstore.service ufw.service unattended-upgrades.service
```

| Câu hỏi | Lệnh | Trên app01 |
|---|---|---|
| Cổng nào nghe ra ngoài? | `ss -tlnpH` ([LNX-041](/articles/linux-ip-cong-ss)) | 22 (sshd), 80, 443 (nginx); `bao-gia` chỉ loopback |
| Tường lửa cho ai vào? | `ufw status` | SSH có `LIMIT`, 443 chỉ mạng nội bộ; 80 chặn |
| sshd nhận gì? | `sshd -T` | Không root, không mật khẩu, key **và** OTP, 3 lần thử |
| Ai có shell, ai có sudo? | `/etc/passwd`, `getent group sudo` | `deploy`, `kiemtoan`, `cuong` còn shell: có còn cần không? |
| Dịch vụ nào bật khi khởi động? | `systemctl list-unit-files --state=enabled` | Chỉ những gì dùng tới |

Checklist không tự sửa gì: nó tạo ra danh sách việc. Ở đây có hai việc rõ ràng. Tài khoản thử của `cuong` (Phần 3) nên khoá (`sudo usermod -L -s /usr/sbin/nologin cuong`). Còn `allowagentforwarding yes` thì nên tắt, như trên bastion ([LNX-052](/articles/linux-bastion-jump-host)).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-gia-co-server-fail2ban/ssh-studio-01.webp" alt="Terminal của SSH Studio: ss rút gọn bằng awk, ufw status, sshd -T với maxauthtries 3, permitrootlogin no, passwordauthentication no, authenticationmethods publickey,keyboard-interactive, danh sách tài khoản có shell, nhóm sudo, các dịch vụ bật sẵn" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · checklist năm câu hỏi trên <code>toinoi-app-01</code></figcaption></figure>

## 2. fail2ban

**fail2ban** đọc log, đếm số lần thất bại của mỗi IP trong một khoảng thời gian, và khi vượt ngưỡng thì thêm quy tắc chặn IP đó vào tường lửa một thời gian. Mỗi bộ «log nào, mẫu nào, chặn bằng gì» gọi là một **jail**.

Ubuntu cài sẵn jail `sshd` (file `jail.d/defaults-debian.conf`). Cấu hình riêng của Tới Nơi ghi vào file `.local` riêng, không sửa `jail.conf` của gói:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-gia-co-server-fail2ban/ssh-studio-02.webp" alt="Vim mở /etc/fail2ban/jail.d/toinoi.local qua sudoedit trong SSH Studio: ignoreip 127.0.0.1/8 ::1 10.20.0.1 10.20.0.10, banaction ufw, jail sshd enabled, backend systemd, maxretry 3, findtime 10m, bantime 30m" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · jail <code>sshd</code> của Tới Nơi</figcaption></figure>

```ini
# fail2ban trên toinoi-app-01 (LNX-055): chặn IP đăng nhập SSH sai nhiều lần, chặn bằng ufw.
[DEFAULT]
# Không bao giờ chặn: chính máy này, đường vào của An (cổng 2222 đi qua 10.20.0.1) và bastion.
ignoreip = 127.0.0.1/8 ::1 10.20.0.1 10.20.0.10
banaction = ufw

[sshd]
enabled = true
backend = systemd
maxretry = 3
findtime = 10m
bantime = 30m
```

| Thiết lập | Nghĩa |
|---|---|
| `maxretry = 3`, `findtime = 10m` | 3 lần thất bại trong 10 phút thì chặn |
| `bantime = 30m` | Chặn 30 phút rồi tự gỡ (máy dò thường bỏ đi) |
| `backend = systemd` | Đọc journal của sshd ([LNX-037](/articles/linux-journalctl-doc-log)), không cần file log |
| `banaction = ufw` | Chặn bằng một quy tắc ufw đặt lên đầu, cùng chỗ với các quy tắc khác |
| **`ignoreip`** | Danh sách **không bao giờ** chặn |

`ignoreip` là dòng quan trọng nhất. Mọi kết nối từ máy An đến app01 đều đi ra từ `10.20.0.1` (cổng 2222 do Docker chuyển vào, xem [LNX-051](/articles/linux-workspace-launchpad-nhieu-server)). An gõ sai OTP ba lần là tự khoá mình 30 phút. Trên máy ảo thật, đó là IP văn phòng hay IP của bastion. Ở đây có cả bastion `10.20.0.10`.

```text
an@toinoi-app-01:~$ sudo apt install -y fail2ban 2>&1 | grep -E '^Setting up fail2ban'
Setting up fail2ban (1.0.2-3ubuntu0.1) ...
an@toinoi-app-01:~$ sudoedit /etc/fail2ban/jail.d/toinoi.local
an@toinoi-app-01:~$ sudo systemctl restart fail2ban
an@toinoi-app-01:~$ sleep 3; sudo fail2ban-client status sshd
Status for the jail: sshd
|- Filter
|  |- Currently failed: 0
|  |- Total failed:     0
|  `- Journal matches:  _SYSTEMD_UNIT=sshd.service + _COMM=sshd
`- Actions
   |- Currently banned: 0
   |- Total banned:     0
   `- Banned IP list:
```

## 3. Thử: một máy dò từ app02

Từ `toinoi-app-02`, thử đăng nhập bằng bốn tài khoản không tồn tại:

```text
an@toinoi-app-02:~$ for u in admin test oracle ubuntu; do ssh -o BatchMode=yes -o ConnectTimeout=3 -o StrictHostKeyChecking=no $u@toinoi-app-01 true 2>&1 | tail -n 1; done
admin@toinoi-app-01: Permission denied (publickey).
test@toinoi-app-01: Permission denied (publickey).
oracle@toinoi-app-01: Permission denied (publickey).
ubuntu@toinoi-app-01: Permission denied (publickey).
an@toinoi-app-02:~$ sleep 2; nc -zv -w 3 toinoi-app-01 22
nc: connect to toinoi-app-01 (10.20.0.11) port 22 (tcp) failed: Connection refused
```

Sau lần thất bại thứ ba, app02 bị chặn: kết nối tới cổng 22 bị từ chối ngay (quy tắc ufw của fail2ban là `REJECT`). Trên app01:

```text
an@toinoi-app-01:~$ sudo fail2ban-client status sshd
Status for the jail: sshd
|- Filter
|  |- Currently failed: 1
|  |- Total failed:     4
|  `- Journal matches:  _SYSTEMD_UNIT=sshd.service + _COMM=sshd
`- Actions
   |- Currently banned: 1
   |- Total banned:     1
   `- Banned IP list:   10.20.0.12
an@toinoi-app-01:~$ sudo ufw status | head -n 5
Status: active

To                         Action      From
--                         ------      ----
Anywhere                   REJECT      10.20.0.12                 # by Fail2Ban after 3 attempts against sshd
an@toinoi-app-01:~$ sudo grep -E 'Found|Ban' /var/log/fail2ban.log | tail -n 5
2026-11-24 09:40:38,733 fail2ban.filter         [7653]: INFO    [sshd] Found 10.20.0.12 - 2026-11-24 09:40:38
2026-11-24 09:40:38,733 fail2ban.filter         [7653]: INFO    [sshd] Found 10.20.0.12 - 2026-11-24 09:40:38
2026-11-24 09:40:38,734 fail2ban.filter         [7653]: INFO    [sshd] Found 10.20.0.12 - 2026-11-24 09:40:38
2026-11-24 09:40:38,965 fail2ban.actions        [7653]: NOTICE  [sshd] Ban 10.20.0.12
2026-11-24 09:40:39,157 fail2ban.filter         [7653]: INFO    [sshd] Found 10.20.0.12 - 2026-11-24 09:40:38
```

- `Currently failed` / `Total failed`: số lần thất bại đang đếm và tổng.
- `Banned IP list`: IP đang bị chặn.
- `ufw status` có quy tắc `REJECT` cho `10.20.0.12` đứng **đầu** danh sách (thứ tự quy tắc ufw có nghĩa: quy tắc đầu tiên khớp thì thắng).
- `/var/log/fail2ban.log`: mỗi lần `Found` (thấy một lần thất bại) và `Ban`/`Unban`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-gia-co-server-fail2ban/ssh-studio-03.webp" alt="Terminal của SSH Studio: fail2ban-client status sshd với Total failed 4, Currently banned 1, Banned IP list 10.20.0.12, ufw status với quy tắc REJECT từ 10.20.0.12 ở đầu, các dòng Found và Ban trong fail2ban.log" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · fail2ban đã chặn <code>10.20.0.12</code> bằng ufw</figcaption></figure>

## 4. Gỡ chặn

```text
an@toinoi-app-01:~$ sudo fail2ban-client set sshd unbanip 10.20.0.12
1
an@toinoi-app-01:~$ sudo fail2ban-client status sshd | grep -E 'Currently banned|Banned IP'
   |- Currently banned: 0
   `- Banned IP list:
```

Người thật bị chặn (đồng nghiệp gõ sai OTP từ quán cà phê) thì gỡ như trên, rồi cân nhắc thêm IP của họ vào `ignoreip`, hoặc tốt hơn: cho họ đi qua bastion.

## 5. fail2ban làm được gì, không làm được gì

- **Làm được**: giảm rác trong log, giảm tải, làm chậm việc dò ở mức IP.
- **Không làm được**: thay cho xác thực tốt. Mạng botnet thử từ hàng nghìn IP, mỗi IP vài lần, không chạm ngưỡng. Thứ giữ server an toàn vẫn là chỉ key và OTP ([LNX-030](/articles/linux-thuc-hanh-khoa-dang-nhap-ssh)) và ít cổng mở.

## 6. Lỗi thường gặp

**Quên `ignoreip`, tự khoá mình.** Giữ phiên SSH đang mở khi thử, và có console dự phòng.

**Sửa `jail.conf`.** Lần nâng gói sau sẽ đè. Dùng `jail.d/*.local`.

**`backend = auto` trên Ubuntu không có `/var/log/auth.log`.** Jail không thấy gì. Với journal, dùng `backend = systemd`.

**Tưởng `fail2ban-client status` không lỗi là jail đang chạy đúng.** Thử thật như mục 3.

## Bài tập

1. Đặt `bantime.increment = true` trong `[DEFAULT]`. Chặn lần hai của cùng một IP kéo dài bao lâu? (`man jail.conf`)
2. Khoá tài khoản `cuong` theo checklist. Kiểm tra bằng `sudo passwd -S cuong` và `getent passwd cuong`.
3. Tắt `AllowAgentForwarding` trên app01 bằng một file trong `/etc/ssh/sshd_config.d/`, kiểm tra bằng `sshd -t` rồi `sshd -T`.
4. Viết jail cho nginx: chặn IP gọi `/bao-gia` quá 30 lần trong 1 phút (filter `nginx-limit-req` hoặc tự viết `failregex`).

## Đọc tiếp

[LNX-056 · Chạy một lệnh trên nhiều server: vòng lặp SSH và Ansible tối thiểu](/articles/linux-nhieu-server-ansible-toi-thieu).
