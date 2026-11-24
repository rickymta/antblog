---
slug: linux-sshd-config-an-toan
title: "Cấu hình sshd an toàn: tắt mật khẩu, tắt root, AllowGroups, và áp dụng mà không tự khoá mình ngoài cửa"
summary: "Đọc cấu hình sshd đang có hiệu lực bằng sshd -T, viết file trong sshd_config.d bằng sudoedit, hiểu quy tắc «giá trị đầu tiên thắng» khiến cấu hình không ăn, chỉ cho nhóm ssh-users đăng nhập, kiểm tra bằng sshd -t, reload và luôn giữ một phiên dự phòng; xem SSH Studio đăng nhập bằng key và tài khoản ngoài nhóm bị từ chối."
category: devops-linux
level: intermediate
tags: [linux, lnx-025, sshd, sshd-config, sudoedit, allowgroups, hardening, ssh-studio, toi-noi]
language: vi
---

Từ [LNX-022](/articles/linux-ssh-key-ssh-studio), An đăng nhập bằng key. Nhưng server vẫn **cho phép** mật khẩu: bất kỳ ai trên Internet đều có thể thử đoán mật khẩu của `an`, `binh`, `cuong`, và log của một server SSH công khai thường có hàng nghìn lần thử như vậy mỗi ngày. Bình giao việc cuối của tuần: tắt đăng nhập bằng mật khẩu, chỉ cho những người cần SSH được vào, và làm sao để không ai bị khoá ngoài cửa trong lúc đổi.

Câu cuối quan trọng như hai câu đầu. Sửa sai `sshd` trên một server ở xa, không có console, là cách nhanh nhất để mất quyền truy cập vào chính server đó.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-025 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-ssh-key-ssh-studio">LNX-022</a> (An phải đăng nhập được bằng key trước khi tắt mật khẩu), <a href="/articles/linux-sudo-sudoers">LNX-024</a>. Trạng thái đầu bài: <code>npm run setup -- 25</code>. Sau bài, SSH Studio vào server bằng key; mọi hồ sơ dùng mật khẩu bị từ chối.</p>
</div>

## 1. Cấu hình đang có hiệu lực

`sshd` đọc cấu hình từ `/etc/ssh/sshd_config` và các file nó `Include`. Đọc từng file rồi tự suy ra kết quả rất dễ sai; hỏi thẳng `sshd`:

```text
an@toinoi-app-01:~$ sudo sshd -T | grep -E '^(permitrootlogin|pubkeyauthentication|passwordauthentication|kbdinteractiveauthentication|maxauthtries|allowgroups|x11forwarding) '
maxauthtries 6
permitrootlogin no
pubkeyauthentication yes
passwordauthentication yes
kbdinteractiveauthentication no
x11forwarding yes
an@toinoi-app-01:~$ grep -v -E '^(#|$)' /etc/ssh/sshd_config
Include /etc/ssh/sshd_config.d/*.conf
KbdInteractiveAuthentication no
UsePAM yes
X11Forwarding yes
PrintMotd no
AcceptEnv LANG LC_*
Subsystem       sftp    /usr/lib/openssh/sftp-server
an@toinoi-app-01:~$ ls /etc/ssh/sshd_config.d
10-toinoi-lab.conf
an@toinoi-app-01:~$ cat /etc/ssh/sshd_config.d/10-toinoi-lab.conf
PasswordAuthentication yes
KbdInteractiveAuthentication no
PermitRootLogin no
PrintMotd no
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sshd-config-an-toan/ssh-studio-07.webp" alt="Terminal của SSH Studio: sshd -T lọc các tuỳ chọn: maxauthtries 6, permitrootlogin no, pubkeyauthentication yes, passwordauthentication yes, x11forwarding yes; grep bỏ chú thích của sshd_config thấy Include /etc/ssh/sshd_config.d/*.conf" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cấu hình sshd đang có hiệu lực trước khi sửa</figcaption></figure>

- **`sshd -T`** (*test, extended*) in **toàn bộ** cấu hình có hiệu lực sau khi gộp mọi file và giá trị mặc định, tên tuỳ chọn viết thường. Đây là câu trả lời chắc chắn nhất cho câu hỏi «sshd đang chạy với giá trị nào».
- Dòng đầu của `sshd_config` là `Include /etc/ssh/sshd_config.d/*.conf`: các file trong thư mục đó được đọc **trước** phần còn lại, theo thứ tự tên.
- File `10-toinoi-lab.conf` của lab bật `PasswordAuthentication yes` (để những bài đầu dùng mật khẩu được).
- Hiện tại: mật khẩu được phép, `MaxAuthTries 6`, `X11Forwarding yes`, không giới hạn ai được vào.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sshd-config-an-toan/ssh-studio-01.webp" alt="Terminal của SSH Studio: ls /etc/ssh/sshd_config.d chỉ có 10-toinoi-lab.conf, cat file đó: PasswordAuthentication yes, KbdInteractiveAuthentication no, PermitRootLogin no, PrintMotd no" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · file cấu hình riêng của lab trong <code>sshd_config.d</code></figcaption></figure>

## 2. Nhóm được phép SSH

Không phải ai có tài khoản cũng cần SSH: tài khoản dịch vụ `baogia` ([LNX-021](/articles/linux-quan-ly-nguoi-dung-nhom)) không bao giờ đăng nhập, Cường chỉ cần vào khi trực. Danh sách trắng theo nhóm dễ quản lý hơn từng tên:

```text
an@toinoi-app-01:~$ sudo groupadd ssh-users
an@toinoi-app-01:~$ sudo usermod -aG ssh-users an
an@toinoi-app-01:~$ sudo usermod -aG ssh-users binh
an@toinoi-app-01:~$ getent group ssh-users
ssh-users:x:2002:an,binh
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sshd-config-an-toan/ssh-studio-08.webp" alt="Terminal của SSH Studio: groupadd ssh-users, usermod -aG thêm an và binh, getent group ssh-users cho ssh-users:x:2002:an,binh" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · nhóm <code>ssh-users</code> gồm những người được phép SSH</figcaption></figure>

## 3. Viết file cấu hình bằng sudoedit

Quy ước như với sudoers: **không sửa `sshd_config` gốc**, thêm một file trong `sshd_config.d/`. File của gói phần mềm được cập nhật yên ổn, còn thay đổi của bạn nằm gọn trong một chỗ.

Sửa file hệ thống, dùng **`sudoedit`** thay cho `sudo vim`:

```bash
sudoedit /etc/ssh/sshd_config.d/20-an-toan.conf
```

`sudoedit` chép file ra một bản tạm thuộc về bạn, mở **vim chạy bằng quyền của bạn** (không phải `root`), và khi bạn lưu, chép bản tạm đè lên file thật. Trong vim, `:!bash` chỉ cho bạn shell của chính bạn. Đây cũng là cách cấp quyền sửa một file cụ thể qua sudoers mà không mở cửa sau.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sshd-config-an-toan/ssh-studio-02.webp" alt="Vim mở bản tạm của /etc/ssh/sshd_config.d/20-an-toan.conf qua sudoedit trong SSH Studio: chú thích Gia cố sshd, PasswordAuthentication no, KbdInteractiveAuthentication no, PermitRootLogin no, AllowGroups ssh-users, MaxAuthTries 3, X11Forwarding no" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sudoedit</code> file cấu hình mới</figcaption></figure>

```text
# Gia cố sshd cho toinoi-app-01 (LNX-025).
PasswordAuthentication no
KbdInteractiveAuthentication no
PermitRootLogin no
AllowGroups ssh-users
MaxAuthTries 3
X11Forwarding no
```

| Tuỳ chọn | Tác dụng |
|---|---|
| `PasswordAuthentication no` | Không nhận mật khẩu |
| `KbdInteractiveAuthentication no` | Tắt luôn kiểu hỏi-đáp, cửa sau phổ biến để vẫn hỏi mật khẩu qua PAM |
| `PermitRootLogin no` | `root` không đăng nhập trực tiếp; ai cần quyền thì vào bằng tài khoản riêng rồi `sudo` |
| `AllowGroups ssh-users` | Chỉ thành viên nhóm này được vào, mọi người khác bị từ chối trước cả khi xác thực |
| `MaxAuthTries 3` | Mỗi kết nối chỉ được thử 3 lần |
| `X11Forwarding no` | Tắt chuyển tiếp giao diện đồ hoạ, server không cần |

## 4. «Giá trị đầu tiên thắng»

```text
an@toinoi-app-01:~$ sudo sshd -t && echo "cú pháp ổn"
cú pháp ổn
an@toinoi-app-01:~$ sudo sshd -T | grep -E '^(passwordauthentication|allowgroups|maxauthtries) '
maxauthtries 3
passwordauthentication yes
allowgroups ssh-users
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sshd-config-an-toan/ssh-studio-09.webp" alt="Terminal của SSH Studio: sudoedit 20-an-toan.conf, sshd -t báo cú pháp ổn, nhưng sshd -T vẫn cho passwordauthentication yes cùng maxauthtries 3 và allowgroups ssh-users" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cú pháp ổn nhưng <code>PasswordAuthentication</code> vẫn là <code>yes</code></figcaption></figure>

Cú pháp ổn, `AllowGroups` và `MaxAuthTries` đã đổi, nhưng **`passwordauthentication` vẫn là `yes`**. Lý do: với hầu hết tuỳ chọn, `sshd` **giữ giá trị đầu tiên nó đọc được** và bỏ qua các lần đặt sau. Các file trong `sshd_config.d` được đọc theo thứ tự tên: `10-toinoi-lab.conf` đặt `PasswordAuthentication yes` trước, `20-an-toan.conf` đến sau và thua.

Đây là ngược với trực giác của hầu hết file cấu hình khác (nơi dòng sau ghi đè dòng trước), và là lý do phổ biến nhất của câu hỏi «tôi đã tắt mật khẩu rồi mà vẫn đăng nhập được». Cách sửa: đặt tên để file của bạn được đọc trước.

```text
an@toinoi-app-01:~$ sudo mv /etc/ssh/sshd_config.d/20-an-toan.conf /etc/ssh/sshd_config.d/05-an-toan.conf
an@toinoi-app-01:~$ ls /etc/ssh/sshd_config.d
05-an-toan.conf  10-toinoi-lab.conf
an@toinoi-app-01:~$ sudo sshd -t && echo "cú pháp ổn"
cú pháp ổn
an@toinoi-app-01:~$ sudo sshd -T | grep -E '^(permitrootlogin|passwordauthentication|kbdinteractiveauthentication|maxauthtries|allowgroups|x11forwarding) '
maxauthtries 3
permitrootlogin no
passwordauthentication no
kbdinteractiveauthentication no
x11forwarding no
allowgroups ssh-users
```

Bây giờ mọi giá trị đúng như ý. Quy trình luôn là: **sửa → `sshd -t` (cú pháp) → `sshd -T` (giá trị thật)**. Chưa bước nào trong số này ảnh hưởng tới server đang chạy.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sshd-config-an-toan/ssh-studio-03.webp" alt="Terminal của SSH Studio: đổi tên 20-an-toan.conf thành 05-an-toan.conf, ls sshd_config.d, sshd -t báo cú pháp ổn, sshd -T cho maxauthtries 3, permitrootlogin no, passwordauthentication no, x11forwarding no, allowgroups ssh-users" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đổi tên thành <code>05-</code> để file của mình được đọc trước</figcaption></figure>

## 5. Áp dụng mà không tự khoá mình ngoài cửa

Ba nguyên tắc khi đổi cấu hình SSH của một server ở xa:

1. **Đã kiểm tra bằng `sshd -t`.** `sshd` không khởi động được với cấu hình sai cú pháp. Một restart lúc đó là mất SSH.
2. **`reload` thay vì `restart`.** `systemctl reload ssh` bảo tiến trình `sshd` chính đọc lại cấu hình; các phiên đang mở không bị đụng tới.
3. **Giữ phiên hiện tại, thử bằng một phiên mới.** Phiên đang mở là đường lui: nếu phiên mới không vào được, bạn vẫn còn một terminal để sửa lại.

```bash
sudo systemctl reload ssh
```

Rồi mở thêm một tab phiên trong SSH Studio, **không đóng tab cũ**:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sshd-config-an-toan/ssh-studio-04.webp" alt="Tab phiên thứ hai tới toinoi-app-01 trong SSH Studio, đăng nhập bằng key, lệnh whoami; id -nG in an và danh sách nhóm có ssh-users" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · phiên thứ hai vẫn vào được bằng key, an thuộc nhóm <code>ssh-users</code></figcaption></figure>

Phiên thứ hai vào được, `id -nG` có `ssh-users`. Lúc này mới yên tâm. Thử với hồ sơ của Cường mà SSH Studio vẫn giữ từ LNX-021, dùng mật khẩu, và Cường không thuộc `ssh-users`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sshd-config-an-toan/ssh-studio-05.webp" alt="SSH Studio kết nối bằng hồ sơ toinoi-app-01 (cuong) với mật khẩu và bị từ chối xác thực" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · hồ sơ của <code>cuong</code> dùng mật khẩu bị từ chối</figcaption></figure>

## 6. Server thấy gì

```text
an@toinoi-app-01:~$ ssh -o BatchMode=yes an@localhost true
an@localhost: Permission denied (publickey).
an@toinoi-app-01:~$ ssh -o BatchMode=yes cuong@localhost true
cuong@localhost: Permission denied (publickey).
an@toinoi-app-01:~$ sudo journalctl -u ssh --no-pager | grep -E 'not allowed because|Accepted publickey' | tail -n 3
Nov 23 23:36:08 toinoi-app-01 sshd[1751]: Accepted publickey for an from 127.0.0.1 port 48568 ssh2: ED25519 SHA256:yJ42WJZlgwZsUsCCjvAX237jMd5+ssxlWKtoaNrMTgY
Nov 23 23:36:11 toinoi-app-01 sshd[1999]: User cuong from ::1 not allowed because none of user's groups are listed in AllowGroups
Nov 23 23:36:11 toinoi-app-01 sshd[2004]: Accepted publickey for an from 127.0.0.1 port 48584 ssh2: ED25519 SHA256:yJ42WJZlgwZsUsCCjvAX237jMd5+ssxlWKtoaNrMTgY
```

- Thử SSH ngay trên server (không có key riêng tư của An ở đó), server giờ chỉ đưa ra **một** phương thức: `Permission denied (publickey)`. Không còn `password` trong danh sách.
- Với Cường, log nói rõ lý do: `not allowed because none of user's groups are listed in AllowGroups`. `sshd` từ chối trước cả khi kiểm tra mật khẩu hay key.
- Những lần đăng nhập của An bằng key `SHA256:yJ42WJZl…` vẫn được chấp nhận.

Cường vẫn dùng `sudo` theo quy tắc của LNX-024 được: quy tắc đó không liên quan tới SSH. Khi Cường cần SSH vào để trực, An thêm Cường vào `ssh-users` và cài key của Cường, như [LNX-022](/articles/linux-ssh-key-ssh-studio).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sshd-config-an-toan/ssh-studio-06.webp" alt="Terminal của SSH Studio: systemctl reload ssh, ssh cuong@localhost bị Permission denied (publickey), journalctl ghi User cuong not allowed because none of user's groups are listed in AllowGroups và Accepted publickey for an" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · log của sshd: Cường bị chặn bởi <code>AllowGroups</code>, An vào bằng key</figcaption></figure>

### Nếu đã lỡ khoá mình ngoài cửa

Trên máy ảo đám mây, hầu như nhà cung cấp nào cũng có **console nối tiếp** (serial console) hoặc cơ chế chạy lệnh từ bảng điều khiển: vào bằng đường đó, sửa file, `sshd -t`, `reload`. Biết trước đường lui này ở đâu, trước khi cần dùng nó.

## 7. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Cấu hình đang có hiệu lực | `sudo sshd -T` |
| Kiểm tra cú pháp | `sudo sshd -t` |
| Thử cấu hình cho một người dùng cụ thể | `sudo sshd -T -C user=cuong,host=x,addr=10.0.0.5` |
| Áp dụng, giữ phiên đang mở | `sudo systemctl reload ssh` |
| Sửa file hệ thống an toàn | `sudoedit FILE` |

## 8. Lỗi thường gặp

**Tắt mật khẩu trước khi key chạy.** Thứ tự đúng: key chạy và đã kiểm chứng trong log ([LNX-022](/articles/linux-ssh-key-ssh-studio)), rồi mới tắt mật khẩu.

**Sửa nhưng không thấy tác dụng.** Quy tắc «giá trị đầu tiên thắng» và thứ tự tên file trong `sshd_config.d`. Kiểm tra bằng `sshd -T`, không bằng mắt.

**`AllowGroups` quên chính mình.** Thêm mình vào nhóm **trước** khi reload, và phiên đang mở không có nhóm mới cho tới khi đăng nhập lại (điều đó không ảnh hưởng: `sshd` đọc nhóm từ `/etc/group` lúc đăng nhập).

**`restart` thay vì `reload`.** Trên Ubuntu, các phiên đang mở thường vẫn sống sót qua `restart`, nhưng nếu cấu hình sai, `sshd` không lên lại được. `reload` với cấu hình đã kiểm tra là an toàn hơn.

**Đóng phiên cũ trước khi phiên mới vào được.** Phiên cũ là đường lui duy nhất.

## Bài tập

1. Chạy `sudo sshd -T -C user=cuong,host=localhost,addr=127.0.0.1 | grep -i allowgroups` và với `user=an`. Kết quả có khác nhau không? Khi nào `-C` thật sự hữu ích (gợi ý: khối `Match` ở [LNX-030](/articles/linux-thuc-hanh-khoa-dang-nhap-ssh))?
2. Đổi `MaxAuthTries 3` thành `MaxAuthTries 1` (đúng quy trình), rồi kết nối từ một máy có 3 key trong agent. Chuyện gì xảy ra? Đặt lại 3.
3. Đọc `man sshd_config` mục `LoginGraceTime` và `MaxStartups`. Hai tuỳ chọn này chống lại kiểu tấn công nào?
4. Liệt kê những tài khoản trên `toinoi-app-01` có shell đăng nhập (`/etc/passwd`, [LNX-021](/articles/linux-quan-ly-nguoi-dung-nhom)) nhưng không thuộc `ssh-users`. Ai trong số đó vẫn cần đăng nhập, bằng đường nào?

## Đọc tiếp

[LNX-026 · setuid, setgid và sticky bit](/articles/linux-setuid-setgid-sticky): ba bit quyền đặc biệt, vì sao `passwd` sửa được `/etc/shadow`, và thư mục chung cho cả nhóm.
