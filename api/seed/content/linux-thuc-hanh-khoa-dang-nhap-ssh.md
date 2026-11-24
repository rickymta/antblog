---
slug: linux-thuc-hanh-khoa-dang-nhap-ssh
title: "Thực hành: khoá chặt đăng nhập SSH của server với tài khoản deploy, sudoers, Match User và bộ kiểm tra"
summary: "Bài thực hành cuối Phần 3: tạo tài khoản deploy cho CI chỉ đăng nhập bằng key, cấp sudo đúng hai lệnh, dùng khối Match User trong sshd để deploy không cần OTP nhưng không mở tunnel, kiểm chứng bằng sshd -T -C và bằng những lần đăng nhập thật của deploy, an, root, cuong, rồi chạy bộ kiểm tra 22 mục của lab."
category: devops-linux
level: advanced
tags: [linux, lnx-030, ssh, sshd, match, sudoers, deploy, hardening, thuc-hanh, ssh-studio, toi-noi]
language: vi
---

Đầu tuần sau, dịch vụ báo giá sẽ được triển khai tự động: hệ thống CI build xong thì SSH vào `toinoi-app-01`, chép bản mới, khởi động lại dịch vụ. CI không có điện thoại để nhận mã OTP, cũng không nên có quyền `root`. Bình giao cho An bài cuối của tuần: tạo tài khoản cho CI, cấp đúng quyền nó cần, và chứng minh rằng mọi lớp bảo vệ của cả tuần vẫn đứng vững.

Bài này không có kiến thức mới lớn. Nó ghép người dùng ([LNX-021](/articles/linux-quan-ly-nguoi-dung-nhom)), key ([LNX-022](/articles/linux-ssh-key-ssh-studio)), sudoers ([LNX-024](/articles/linux-sudo-sudoers)), cấu hình sshd ([LNX-025](/articles/linux-sshd-config-an-toan)) và MFA ([LNX-029](/articles/linux-ssh-mfa-totp)) lại, thêm một công cụ: khối **`Match`** của sshd.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-030 · Phần 3/6 · Nâng cao · Thực hành</span>
<p>Trạng thái đầu bài: <code>npm run setup -- 30</code> (server đã có key, sudoers của Cường, sshd gia cố và MFA của các bài trước). Thử tự làm từng bước theo yêu cầu trước khi đọc lời giải. Cuối bài, <code>npm run kiem-tra:p03</code> chấm toàn bộ Phần 3.</p>
</div>

## Yêu cầu

1. Tài khoản **`deploy`**: thuộc `toinoi` (để ghi vào thư mục của dịch vụ) và `ssh-users`; **không có mật khẩu**; đăng nhập bằng key `deploy@ci` của CI.
2. `deploy` được `sudo` **đúng hai lệnh**: khởi động lại và xem trạng thái dịch vụ `bao-gia`, không hỏi mật khẩu.
3. `deploy` đăng nhập chỉ bằng key (không OTP), và **không** được mở tunnel (port forwarding). Mọi người khác vẫn phải qua key và OTP.
4. Chứng minh bằng log: `deploy` vào được bằng key; `an` vào bằng key và OTP; `root` và `cuong` bị từ chối; mật khẩu không còn được nhận với bất kỳ ai.

## 1. Tài khoản deploy

```text
an@toinoi-app-01:~$ sudo useradd --create-home --shell /bin/bash --comment "Trien khai tu dong (CI)" --groups toinoi,ssh-users deploy
an@toinoi-app-01:~$ id deploy
uid=1004(deploy) gid=1004(deploy) groups=1004(deploy),2000(toinoi),2002(ssh-users)
an@toinoi-app-01:~$ sudo passwd -S deploy
deploy L 2026-11-23 0 99999 7 -1
an@toinoi-app-01:~$ sudo install -d -m 700 -o deploy -g deploy /home/deploy/.ssh
an@toinoi-app-01:~$ echo 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICZUGM/RB5spPmpjCeCYkOgP1vCe9YV5XT3kEL8MCG3A deploy@ci' | sudo tee /home/deploy/.ssh/authorized_keys
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICZUGM/RB5spPmpjCeCYkOgP1vCe9YV5XT3kEL8MCG3A deploy@ci
an@toinoi-app-01:~$ sudo chown deploy:deploy /home/deploy/.ssh/authorized_keys
an@toinoi-app-01:~$ sudo chmod 600 /home/deploy/.ssh/authorized_keys
an@toinoi-app-01:~$ sudo ls -la /home/deploy/.ssh
total 12
drwx------ 2 deploy deploy 4096 Nov 23 23:36 .
drwxr-x--- 3 deploy deploy 4096 Nov 23 23:36 ..
-rw------- 1 deploy deploy   91 Nov 23 23:36 authorized_keys
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-khoa-dang-nhap-ssh/ssh-studio-05.webp" alt="Terminal của SSH Studio: useradd tạo deploy trong nhóm toinoi và ssh-users, id deploy, passwd -S deploy cho trạng thái L (không có mật khẩu)" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tài khoản <code>deploy</code> cho CI, không có mật khẩu</figcaption></figure>

- `useradd --groups toinoi,ssh-users` thêm nhóm phụ ngay khi tạo.
- **`passwd -S`** báo **`L`** (*locked*): `useradd` không đặt mật khẩu, nên không ai đăng nhập bằng mật khẩu được. Đó là đúng ý: CI chỉ dùng key.
- **`install -d -m 700 -o deploy -g deploy`** tạo thư mục với quyền và chủ đúng trong một lệnh.
- Ghi file của người khác thì `sudo tee` (nhớ [LNX-008](/articles/linux-chuyen-huong-pipe): `sudo echo … >` không chạy vì phần `>` do shell của An làm). Rồi `chown` và `chmod 600`, vì `StrictModes` ([LNX-022](/articles/linux-ssh-key-ssh-studio)) từ chối file quyền sai.

Trong thực tế, CI tạo cặp key, giữ private key trong kho bí mật của nó, và chỉ đưa public key cho bạn. Private key của lab nằm sẵn trong server để bài kiểm tra đăng nhập được.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-khoa-dang-nhap-ssh/ssh-studio-01.webp" alt="Terminal của SSH Studio: install -d -m 700 tạo /home/deploy/.ssh, echo public key deploy@ci qua sudo tee vào authorized_keys, chown deploy:deploy, chmod 600, ls -la cho .ssh drwx------ và authorized_keys -rw-------" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cài public key của CI cho tài khoản <code>deploy</code></figcaption></figure>

## 2. sudo đúng việc

```bash
sudo visudo -f /etc/sudoers.d/deploy
```

```text
# Tài khoản deploy của CI: chỉ được khởi động lại và xem trạng thái dịch vụ bao-gia (LNX-030).
deploy ALL=(root) NOPASSWD: /usr/bin/systemctl restart bao-gia, /usr/bin/systemctl status bao-gia
```

```text
an@toinoi-app-01:~$ sudo chmod 440 /etc/sudoers.d/deploy
an@toinoi-app-01:~$ sudo visudo -c
/etc/sudoers: parsed OK
/etc/sudoers.d/README: parsed OK
/etc/sudoers.d/cuong: parsed OK
/etc/sudoers.d/deploy: parsed OK
an@toinoi-app-01:~$ sudo -l -U deploy
Matching Defaults entries for deploy on toinoi-app-01:
    env_reset, mail_badpass,
    secure_path=/usr/local/sbin\:/usr/local/bin\:/usr/sbin\:/usr/bin\:/sbin\:/bin\:/snap/bin,
    use_pty

User deploy may run the following commands on toinoi-app-01:
    (root) NOPASSWD: /usr/bin/systemctl restart bao-gia, /usr/bin/systemctl
        status bao-gia
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-khoa-dang-nhap-ssh/ssh-studio-06.webp" alt="Terminal của SSH Studio: visudo -f /etc/sudoers.d/deploy, chmod 440, visudo -c parsed OK cả ba file, sudo -l -U deploy chỉ cho systemctl restart bao-gia và systemctl status bao-gia với NOPASSWD" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>deploy</code> chỉ được khởi động lại và xem trạng thái bao-gia</figcaption></figure>

Như ở [LNX-024](/articles/linux-sudo-sudoers), file mới do `visudo -f` tạo có quyền `0640`, nên `chmod 440` ngay rồi mới `visudo -c`. `NOPASSWD` là bắt buộc ở đây (`deploy` không có mật khẩu để nhập), và an toàn vì danh sách lệnh hẹp, có tham số cố định. `systemctl status` mở pager khi chạy ở terminal; CI chạy không có terminal nên không vấp vào đó.

## 3. sshd: ngoại lệ cho deploy bằng Match User

Cấu hình từ [LNX-029](/articles/linux-ssh-mfa-totp) bắt mọi người qua `publickey,keyboard-interactive`. CI không trả lời được câu hỏi OTP. Khối **`Match`** đặt cấu hình khác cho một nhóm kết nối thoả điều kiện:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-khoa-dang-nhap-ssh/ssh-studio-02.webp" alt="Vim mở /etc/ssh/sshd_config.d/05-an-toan.conf qua sudoedit trong SSH Studio với các dòng gia cố chung và ở cuối khối Match User deploy gồm AuthenticationMethods publickey và AllowTcpForwarding no" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · khối <code>Match User deploy</code> cuối file cấu hình</figcaption></figure>

```text
# Gia cố sshd cho toinoi-app-01 (LNX-025, LNX-029, LNX-030).
PasswordAuthentication no
KbdInteractiveAuthentication yes
AuthenticationMethods publickey,keyboard-interactive
PermitRootLogin no
AllowGroups ssh-users
MaxAuthTries 3
X11Forwarding no

# Tài khoản deploy của CI: chỉ dùng key, không mở tunnel.
Match User deploy
    AuthenticationMethods publickey
    AllowTcpForwarding no
```

- **`Match User deploy`**: mọi dòng sau nó, cho tới `Match` kế tiếp hoặc **hết file**, chỉ áp dụng cho kết nối của `deploy`. Thụt lề chỉ để dễ đọc; sshd không quan tâm.
- Vì vậy khối `Match` **luôn đặt cuối file**. Một dòng chung đặt sau `Match` sẽ bị hiểu là thuộc khối đó.
- Có thể match theo `User`, `Group`, `Address` (dải IP), `Host`. Ví dụ hay gặp: `Match Address 10.0.0.0/8` cho phép kiểu xác thực khác từ mạng nội bộ.
- Một chi tiết đáng nhớ: file này nằm trong `sshd_config.d` và được `Include` ở đầu `sshd_config`. Một `Match` không đóng lại sẽ «nuốt» các dòng phía sau trong file chính; OpenSSH mới tự kết thúc khối `Match` ở cuối mỗi file được include, nhưng giữ thói quen đặt `Match` cuối cùng vẫn an toàn hơn.

`sshd -T` thông thường in cấu hình chung. Để xem cấu hình **cho một kết nối cụ thể**, thêm **`-C`** với các thông tin của kết nối:

```text
an@toinoi-app-01:~$ sudo sshd -t && echo "cú pháp ổn"
cú pháp ổn
an@toinoi-app-01:~$ sudo sshd -T -C user=deploy,host=ci,addr=10.0.0.5 | grep -E '^(authenticationmethods|allowtcpforwarding|kbdinteractiveauthentication) '
kbdinteractiveauthentication yes
allowtcpforwarding no
authenticationmethods publickey
an@toinoi-app-01:~$ sudo sshd -T -C user=an,host=laptop,addr=10.0.0.9 | grep -E '^(authenticationmethods|allowtcpforwarding|kbdinteractiveauthentication) '
kbdinteractiveauthentication yes
allowtcpforwarding yes
authenticationmethods publickey,keyboard-interactive
an@toinoi-app-01:~$ sudo systemctl reload ssh
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-khoa-dang-nhap-ssh/ssh-studio-07.webp" alt="Terminal của SSH Studio: sudoedit 05-an-toan.conf, sshd -t báo cú pháp ổn, sshd -T -C user=deploy cho allowtcpforwarding no và authenticationmethods publickey" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>Match User deploy</code>: chỉ key, không mở tunnel</figcaption></figure>

Cùng một server, hai kết nối khác nhau: `deploy` chỉ cần `publickey` và không có tunnel; `an` cần `publickey,keyboard-interactive` và vẫn được forwarding.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-khoa-dang-nhap-ssh/ssh-studio-03.webp" alt="Terminal của SSH Studio: sshd -T -C user=an,host=laptop,addr=10.0.0.9 cho kbdinteractiveauthentication yes, allowtcpforwarding yes, authenticationmethods publickey,keyboard-interactive, rồi systemctl reload ssh" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · với An, cấu hình chung vẫn đòi key cộng mã OTP</figcaption></figure>

## 4. Kiểm tra bằng những lần đăng nhập thật

Đọc cấu hình chưa đủ; bằng chứng là log. Lab thử bốn lần đăng nhập: `deploy` bằng key của CI, `an` bằng key và OTP, `root` và `cuong` bằng key của An:

```text
an@toinoi-app-01:~$ sudo journalctl -u ssh --no-pager | grep -E 'Accepted|not allowed because|ROOT LOGIN REFUSED' | tail -n 5
Nov 23 23:36:15 toinoi-app-01 sshd(pam_google_authenticator)[2261]: Accepted google_authenticator for an
Nov 23 23:36:15 toinoi-app-01 sshd[2259]: Accepted keyboard-interactive/pam for an from 127.0.0.1 port 48600 ssh2
Nov 23 23:36:16 toinoi-app-01 sshd[2352]: Accepted publickey for deploy from 127.0.0.1 port 45716 ssh2: ED25519 SHA256:yAG5n1d0W3YL5Q9tRyDrxn3P+LuTYFqC8CnyLarSPKs
Nov 23 23:36:16 toinoi-app-01 sshd[2371]: User root from 127.0.0.1 not allowed because none of user's groups are listed in AllowGroups
Nov 23 23:36:16 toinoi-app-01 sshd[2376]: User cuong from 127.0.0.1 not allowed because none of user's groups are listed in AllowGroups
an@toinoi-app-01:~$ ssh -o BatchMode=yes -o PreferredAuthentications=password an@localhost true
an@localhost: Permission denied (publickey).
```

| Thử | Kết quả trong log | Lớp chặn |
|---|---|---|
| `an`, key + OTP | `Accepted google_authenticator`, `Accepted keyboard-interactive/pam` | Đủ hai lớp |
| `deploy`, key của CI | `Accepted publickey for deploy`, fingerprint `SHA256:yAG5n1d0…` | `Match User deploy` |
| `root` | `not allowed because none of user's groups are listed in AllowGroups` | `AllowGroups` (trước cả `PermitRootLogin`) |
| `cuong` | như `root` | `AllowGroups` |
| Mật khẩu, với bất kỳ ai | `Permission denied (publickey)` | Server chỉ còn mời `publickey` |

`root` bị chặn bởi `AllowGroups` trước khi `PermitRootLogin no` kịp lên tiếng. Hai lớp chồng nhau là có chủ đích: lỡ ai đó thêm `root` vào `ssh-users`, `PermitRootLogin no` vẫn đứng đó.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-khoa-dang-nhap-ssh/ssh-studio-04.webp" alt="Terminal của SSH Studio: journalctl cho Accepted google_authenticator và Accepted keyboard-interactive/pam cho an, Accepted publickey for deploy với fingerprint SHA256:yAG5n1d0…, User root và User cuong not allowed because none of user's groups are listed in AllowGroups; ssh với PreferredAuthentications=password báo Permission denied (publickey)" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · log: mỗi lần đăng nhập bị chặn hoặc cho qua đúng lớp</figcaption></figure>

## 5. Chấm điểm cả Phần 3

Trên máy của bạn (thư mục `examples/toi-noi-linux`):

```bash
npm run kiem-tra:p03
```

Bộ kiểm tra chạy trên server đang chạy (không tạo lại server), có 22 mục, gồm cả việc **đăng nhập thật** bằng từng key (với `an`, nó đợi sang khoảng 30 giây mới để lấy mã OTP chưa dùng):

| Nhóm kiểm tra | Nội dung |
|---|---|
| Tài khoản `deploy` | Thuộc `toinoi` và `ssh-users`, `passwd -S` là `L`, `.ssh` là `700`, `authorized_keys` là `600` và chứa key `deploy@ci` |
| sudo | `visudo -c` sạch; `deploy` chỉ có `restart`/`status bao-gia`, không có `ALL`; file sudoers quyền `440` |
| sshd | `sshd -T` có đủ `passwordauthentication no`, `permitrootlogin no`, `allowgroups ssh-users`, MFA, `maxauthtries 3`; với `deploy`: chỉ `publickey`, không forwarding |
| MFA | PAM của sshd có `pam_google_authenticator`; `~an/.google_authenticator` chỉ `an` đọc được |
| Đăng nhập thật | `deploy` vào bằng key; `an` chỉ có key thì không vào được, có thêm OTP thì vào được; `root`, `cuong` bị từ chối; mật khẩu bị từ chối |

Tất cả `PASS` là xong Phần 3. Mục nào `FAIL` có kèm dòng chi tiết (giá trị thực tế đọc được trên server) để bạn lần ra chỗ sai.

## 6. Danh sách kiểm tra gia cố SSH

Dùng lại cho mọi server mới của nhóm:

- [ ] Mỗi người một tài khoản, không dùng chung; tài khoản dịch vụ không có shell ([LNX-021](/articles/linux-quan-ly-nguoi-dung-nhom)).
- [ ] Đăng nhập bằng key, key có passphrase ([LNX-022](/articles/linux-ssh-key-ssh-studio), [LNX-028](/articles/linux-ssh-agent-passphrase)).
- [ ] Ghi lại fingerprint host key; dựng lại server thì giữ host key ([LNX-023](/articles/linux-host-key-known-hosts)).
- [ ] sudo theo việc, trong `/etc/sudoers.d/`, sửa bằng `visudo` ([LNX-024](/articles/linux-sudo-sudoers)).
- [ ] `PasswordAuthentication no`, `PermitRootLogin no`, `AllowGroups`, kiểm bằng `sshd -T` ([LNX-025](/articles/linux-sshd-config-an-toan)).
- [ ] Thư mục chung có setgid; không có file setuid lạ ([LNX-026](/articles/linux-setuid-setgid-sticky)).
- [ ] ACL chỉ cho ngoại lệ, có ngày gỡ ([LNX-027](/articles/linux-acl-setfacl)).
- [ ] MFA cho người; ngoại lệ cho máy bằng `Match`, quyền hẹp ([LNX-029](/articles/linux-ssh-mfa-totp), bài này).
- [ ] Mọi thay đổi sshd: `sshd -t`, `reload`, giữ phiên dự phòng, thử bằng phiên mới.

## 7. Lỗi thường gặp

**Đặt dòng chung sau khối `Match`.** Dòng đó chỉ áp cho người dùng của khối. Luôn đặt `Match` cuối file.

**Cho tài khoản CI quyền `sudo` rộng «cho tiện».** Kho bí mật của CI bị lộ thì server mất trắng. Hai lệnh là đủ.

**Quên `AllowTcpForwarding no` cho tài khoản máy.** Key của CI bị lộ thì có thể dùng server như cầu nối vào mạng nội bộ.

**Chỉ đọc cấu hình mà không thử đăng nhập thật.** `sshd -T` cho biết ý định; log cho biết điều đã xảy ra.

## Bài tập

1. Thêm `ForceCommand` vào khối `Match User deploy` để `deploy` chỉ chạy được đúng một script triển khai (ví dụ `/usr/local/bin/trien-khai-bao-gia`). Đọc `man sshd_config`: `SSH_ORIGINAL_COMMAND` là gì?
2. Thêm `Match Address 127.0.0.1` cho phép `an` đăng nhập từ chính server chỉ bằng key. Kiểm tra bằng `sshd -T -C`. Có nên làm vậy trên server thật không?
3. Trong `authorized_keys` của `deploy`, thêm `from="10.0.0.0/8",restrict` trước key. `restrict` tắt những gì? So với `AllowTcpForwarding no` trong `Match`, cách nào bạn chọn, vì sao?
4. Sau khi bộ kiểm tra đạt, cố ý làm hỏng một mục (ví dụ `sudo chmod 644 /home/deploy/.ssh/authorized_keys`), rồi chạy lại `npm run kiem-tra:p03`. Những mục nào `FAIL` (gợi ý: không chỉ một)? Sửa lại và chạy lần nữa.

## Đọc tiếp

Phần 3 khép lại với một server mà ai được vào, vào bằng gì, làm được gì đều đã rõ. **Phần 4** chuyển sang việc của server khi đã có người vào: tiến trình, dịch vụ `systemd`, log và lịch chạy. Bắt đầu với [LNX-031 · Tiến trình: ps, top, PID và cây tiến trình](/articles/linux-tien-trinh-ps-top).
