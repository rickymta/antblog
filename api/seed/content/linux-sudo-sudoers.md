---
slug: linux-sudo-sudoers
title: "sudo và sudoers: cấp quyền theo việc bằng visudo thay vì cho cả nhóm sudo"
summary: "sudo của bạn đến từ đâu, đọc /etc/sudoers và /etc/sudoers.d, cấp cho một người đúng những lệnh cần dùng bằng Cmnd_Alias và NOPASSWD, sửa bằng visudo để không tự khoá sudo vì lỗi cú pháp, kiểm tra bằng sudo -l -U, và đọc nhật ký sudo trong journal."
category: devops-linux
level: intermediate
tags: [linux, lnx-024, sudo, sudoers, visudo, least-privilege, journalctl, ssh-studio, toi-noi]
language: vi
---

Cường làm vận hành ca đêm. Việc của Cường trên `toinoi-app-01` chỉ có ba thứ: xem log của `cron`, xem log của `ssh`, và khởi động lại `cron` khi job báo cáo bị treo. Cách nhanh nhất là `usermod -aG sudo cuong`, và đó cũng là cách sai: thành viên nhóm `sudo` làm được **mọi thứ** với quyền `root`, kể cả đọc `/etc/shadow`, xoá database hay tắt firewall.

Bài này cấp cho Cường đúng ba việc đó, không hơn. Nguyên tắc đằng sau có tên là **quyền tối thiểu** (*least privilege*): mỗi người, mỗi tiến trình chỉ có quyền cần cho việc của mình.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-024 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-nguoi-dung-nhom-quyen-file">LNX-007</a>, <a href="/articles/linux-quan-ly-nguoi-dung-nhom">LNX-021</a> (tạo tài khoản <code>cuong</code>), <a href="/articles/linux-vim-hang-ngay">LNX-012</a>. Trạng thái đầu bài: <code>npm run setup -- 24</code>.</p>
</div>

## 1. sudo của An đến từ đâu?

```text
an@toinoi-app-01:~$ sudo -l
Matching Defaults entries for an on toinoi-app-01:
    env_reset, mail_badpass,
    secure_path=/usr/local/sbin\:/usr/local/bin\:/usr/sbin\:/usr/bin\:/sbin\:/bin\:/snap/bin,
    use_pty

User an may run the following commands on toinoi-app-01:
    (ALL : ALL) ALL
an@toinoi-app-01:~$ sudo grep -v -E '^(#|$)' /etc/sudoers
Defaults        env_reset
Defaults        mail_badpass
Defaults        secure_path="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/snap/bin"
Defaults        use_pty
root    ALL=(ALL:ALL) ALL
%admin ALL=(ALL) ALL
%sudo   ALL=(ALL:ALL) ALL
@includedir /etc/sudoers.d
an@toinoi-app-01:~$ sudo ls -l /etc/sudoers.d
total 4
-r--r----- 1 root root 1068 Jan 30  2024 README
an@toinoi-app-01:~$ getent group sudo
sudo:x:27:an,binh
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sudo-sudoers/ssh-studio-05.webp" alt="Terminal của SSH Studio: sudo -l của an cho (ALL : ALL) ALL cùng các Defaults, grep bỏ chú thích của /etc/sudoers thấy %sudo ALL=(ALL:ALL) ALL và @includedir /etc/sudoers.d" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sudo -l</code> và phần hiệu lực của <code>/etc/sudoers</code></figcaption></figure>

- **`sudo -l`** liệt kê những gì bạn được chạy: `(ALL : ALL) ALL` nghĩa là mọi lệnh, dưới danh nghĩa mọi người dùng và mọi nhóm.
- Trong `/etc/sudoers`, dòng **`%sudo ALL=(ALL:ALL) ALL`** cấp quyền đó cho cả nhóm `sudo` (dấu `%` chỉ nhóm). An và Bình có quyền vì nằm trong nhóm này.
- **`Defaults`** đặt hành vi chung. `env_reset` xoá biến môi trường của bạn trước khi chạy lệnh, `secure_path` đặt lại `PATH` an toàn, `use_pty` chạy lệnh trong một terminal riêng để chương trình độc hại không chen được vào terminal của bạn.
- Dòng cuối **`@includedir /etc/sudoers.d`** đọc thêm mọi file trong thư mục đó. Quy ước tốt: **không sửa `/etc/sudoers`**, mỗi quy tắc mới là một file trong `/etc/sudoers.d/`. Gói phần mềm cập nhật `/etc/sudoers` cũng không đè lên quy tắc của bạn.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sudo-sudoers/ssh-studio-01.webp" alt="Terminal của SSH Studio: sudo ls -l /etc/sudoers.d chỉ có README, getent group sudo cho sudo:x:27:an,binh" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · quyền sudo của An đến từ nhóm <code>sudo</code></figcaption></figure>

### Cú pháp một dòng sudoers

```text
cuong    ALL   =  (root)   NOPASSWD:  /usr/bin/systemctl restart cron
 ai     máy nào  chạy dưới   không hỏi    lệnh nào (đường dẫn đầy đủ,
                 danh nghĩa  mật khẩu     kèm tham số nếu muốn giới hạn)
```

- **Ai**: tên người dùng, hoặc `%nhóm`.
- **Máy nào**: hầu như luôn là `ALL` (dùng khi một file sudoers được chia sẻ cho nhiều máy).
- **`(root)`**: được chạy lệnh dưới danh nghĩa ai. `(ALL)` là bất kỳ ai, kể cả `root`.
- **`NOPASSWD:`**: không hỏi mật khẩu. Cường làm ca đêm, đăng nhập bằng key, và ba lệnh này vô hại, nên bỏ mật khẩu ở đây là hợp lý.
- **Lệnh**: đường dẫn tuyệt đối. Có tham số thì sudo chỉ cho đúng tham số đó; `*` khớp phần còn lại.

`Cmnd_Alias` đặt tên cho một nhóm lệnh, đọc dễ hơn khi danh sách dài.

## 2. Viết quy tắc cho Cường bằng visudo

**Không bao giờ sửa sudoers bằng `vim` trực tiếp.** Một lỗi cú pháp trong `/etc/sudoers` làm `sudo` từ chối chạy với **mọi người**. Nếu không còn ai đăng nhập được `root` bằng cách khác, bạn đã tự khoá mình khỏi quyền quản trị của server.

**`visudo`** mở file trong một bản sao, và chỉ ghi vào chỗ thật khi cú pháp đúng. Trên Ubuntu, `visudo` dùng trình soạn thảo trong biến `EDITOR`/`SUDO_EDITOR`; lab đặt sẵn là `vim`. `-f` chọn file khác `/etc/sudoers`:

```bash
sudo visudo -f /etc/sudoers.d/cuong
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sudo-sudoers/ssh-studio-02.webp" alt="Vim mở /etc/sudoers.d/cuong qua sudo visudo -f trong SSH Studio, nội dung hai dòng chú thích, Cmnd_Alias CUONG_LOG gồm journalctl -u cron * và journalctl -u ssh *, dòng cuong ALL=(root) NOPASSWD: CUONG_LOG, /usr/bin/systemctl restart cron" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sudo visudo -f /etc/sudoers.d/cuong</code> trong vim</figcaption></figure>

Nội dung file:

```text
# Cường (vận hành) được xem log của cron, ssh và khởi động lại cron, không cần mật khẩu (LNX-024).
# Sửa bằng: sudo visudo -f /etc/sudoers.d/cuong
Cmnd_Alias CUONG_LOG = /usr/bin/journalctl -u cron *, /usr/bin/journalctl -u ssh *
cuong ALL=(root) NOPASSWD: CUONG_LOG, /usr/bin/systemctl restart cron
```

Lưu bằng `:wq`, `visudo` kiểm tra rồi mới ghi. Dòng chú thích thứ hai nhắc người sau sửa đúng cách.

```text
an@toinoi-app-01:~$ sudo ls -l /etc/sudoers.d
total 8
-r--r----- 1 root root 1068 Jan 30  2024 README
-rw-r----- 1 root root  331 Nov 23 23:36 cuong
an@toinoi-app-01:~$ sudo visudo -c
/etc/sudoers: parsed OK
/etc/sudoers.d/README: parsed OK
/etc/sudoers.d/cuong: bad permissions, should be mode 0440
an@toinoi-app-01:~$ sudo chmod 440 /etc/sudoers.d/cuong
an@toinoi-app-01:~$ sudo ls -l /etc/sudoers.d
total 8
-r--r----- 1 root root 1068 Jan 30  2024 README
-r--r----- 1 root root  331 Nov 23 23:36 cuong
an@toinoi-app-01:~$ sudo visudo -c
/etc/sudoers: parsed OK
/etc/sudoers.d/README: parsed OK
/etc/sudoers.d/cuong: parsed OK
an@toinoi-app-01:~$ sudo -l -U cuong
Matching Defaults entries for cuong on toinoi-app-01:
    env_reset, mail_badpass,
    secure_path=/usr/local/sbin\:/usr/local/bin\:/usr/sbin\:/usr/bin\:/sbin\:/bin\:/snap/bin,
    use_pty

User cuong may run the following commands on toinoi-app-01:
    (root) NOPASSWD: /usr/bin/journalctl -u cron *, /usr/bin/journalctl -u ssh
        *, /usr/bin/systemctl restart cron
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sudo-sudoers/ssh-studio-06.webp" alt="Terminal của SSH Studio: visudo -c báo /etc/sudoers.d/cuong bad permissions, should be mode 0440, chmod 440, visudo -c parsed OK, sudo -l -U cuong liệt kê journalctl -u cron *, journalctl -u ssh *, systemctl restart cron với NOPASSWD" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · sửa quyền file <code>sudoers.d/cuong</code> và xem Cường được phép gì</figcaption></figure>

- **`visudo -c`** kiểm tra toàn bộ sudoers, không sửa gì. Cú pháp của file mới ổn, nhưng nó báo **`bad permissions, should be mode 0440`**: `visudo -f` tạo file mới với quyền `0640` (`-rw-r-----`), còn quy ước của sudo cho file sudoers là chỉ đọc, `0440`. Bản sudo hiện tại vẫn đọc file `0640` (nó chỉ từ chối file mà người khác ngoài `root` ghi được), nhưng đừng để cảnh báo nằm đó: một cảnh báo quen mắt sẽ che mất cảnh báo thật lần sau.
- **`sudo chmod 440`**, chạy lại `visudo -c`: sạch. Với `/etc/sudoers` gốc thì `visudo` giữ nguyên quyền cũ, chỉ file mới tạo bằng `-f` mới cần bước này.
- **`sudo -l -U cuong`** cho An xem Cường được làm gì, không cần đăng nhập bằng tài khoản của Cường.

## 3. Thử bằng tài khoản của Cường

`sudo -u cuong` chạy lệnh dưới danh nghĩa Cường; lệnh đó lại gọi `sudo -n` (không bao giờ hỏi mật khẩu, thiếu quyền thì báo lỗi ngay):

```text
an@toinoi-app-01:~$ sudo -u cuong sudo -n /usr/bin/systemctl restart cron && echo "khởi động lại cron: được"
khởi động lại cron: được
an@toinoi-app-01:~$ sudo -u cuong sudo -n /usr/bin/journalctl -u cron -n 2 --no-pager
Nov 23 23:36:10 toinoi-app-01 cron[1891]: (CRON) INFO (pidfile fd = 3)
Nov 23 23:36:10 toinoi-app-01 cron[1891]: (CRON) INFO (Skipping @reboot jobs -- not system startup)
an@toinoi-app-01:~$ sudo -u cuong sudo -n /usr/bin/cat /etc/shadow
sudo: a password is required
an@toinoi-app-01:~$ sudo -u cuong sudo -n /usr/bin/systemctl stop cron
sudo: a password is required
```

- Hai lệnh trong danh sách chạy ngay, không hỏi mật khẩu.
- `cat /etc/shadow` và `systemctl stop cron` (không phải `restart`) đều bị từ chối. `sudo` muốn hỏi mật khẩu vì không có quy tắc `NOPASSWD` nào khớp; có nhập mật khẩu của Cường thì kết quả vẫn là «Sorry, user cuong is not allowed to execute…», vì Cường không có quy tắc nào khác.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sudo-sudoers/ssh-studio-03.webp" alt="Terminal của SSH Studio: ls -l /etc/sudoers.d có file cuong quyền rw-r-----, visudo -c báo bad permissions should be mode 0440, sau sudo chmod 440 file cuong thành r--r----- và visudo -c báo parsed OK cho cả ba file, sudo -l -U cuong liệt kê NOPASSWD cho journalctl -u cron *, journalctl -u ssh * và systemctl restart cron, Cường khởi động lại cron được, xem log cron được, cat /etc/shadow và systemctl stop cron bị từ chối" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · Cường làm được đúng ba việc, không hơn</figcaption></figure>

### Cạm bẫy: lệnh cho phép thoát ra shell

Khi cấp quyền theo lệnh, hãy nghĩ xem lệnh đó có cho người dùng «thoát» ra một shell `root` không. Một số ví dụ kinh điển:

| Lệnh cấp qua sudo | Vì sao nguy hiểm |
|---|---|
| `vim`, `less`, `man` | Gõ `:!bash` hoặc `!bash` là có shell `root` |
| `find` | `find . -exec /bin/bash \;` |
| `tar`, `zip`, `rsync` | Có tuỳ chọn chạy lệnh tuỳ ý |
| `bash script.sh` mà người dùng sửa được `script.sh` | Họ sửa script rồi chạy |
| `journalctl` không giới hạn | Mở log trong `less`, từ đó `!bash` |

Quy tắc của Cường cũng chạm tới điểm này: `journalctl -u cron` mở `less` khi output dài. Các bản systemd mới (từ 247, Ubuntu 24.04 dùng 255) tự nhận ra mình đang chạy qua `sudo` và mở `less` ở chế độ an toàn (`LESSSECURE=1`), tắt lệnh `!`. Đó là may mắn của riêng `journalctl`, không phải quy luật chung: trước khi cấp một lệnh, hãy tự kiểm tra nó. Muốn cho ai sửa một file hệ thống, dùng **`sudoedit`** ([LNX-025](/articles/linux-sshd-config-an-toan)), không cấp `sudo vim`.

## 4. visudo bắt lỗi cú pháp

Một bản nháp quy tắc thiếu dấu `:` sau `NOPASSWD`, lỗi rất dễ gõ:

```text
an@toinoi-app-01:~$ cat ~/tai-lieu/sudoers-loi
cuong ALL=(root) NOPASSWD /usr/bin/systemctl restart cron
an@toinoi-app-01:~$ sudo visudo -c -f ~/tai-lieu/sudoers-loi
/home/an/tai-lieu/sudoers-loi:1:58: syntax error
cuong ALL=(root) NOPASSWD /usr/bin/systemctl restart cron
                                                         ^
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sudo-sudoers/ssh-studio-07.webp" alt="Terminal của SSH Studio: cat sudoers-loi thiếu dấu hai chấm sau NOPASSWD, visudo -c -f báo syntax error ở dòng 1 cột 58 kèm dấu ^ chỉ vị trí" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>visudo -c -f</code> bắt lỗi trước khi file được dùng</figcaption></figure>

`visudo -c -f` kiểm tra một file bất kỳ mà không cài nó, và chỉ rõ dòng, cột lỗi. Nếu bạn đang sửa trong `visudo` và lưu một file sai, `visudo` hỏi `What now?`: gõ `e` để sửa tiếp, `x` để thoát mà không ghi gì. **Không chọn `Q`** (ghi file sai, chấp nhận rủi ro).

## 5. sudo ghi lại mọi lần dùng

```text
an@toinoi-app-01:~$ sudo journalctl _COMM=sudo --no-pager | grep -E 'cuong :' | tail -n 3
Nov 23 23:36:10 toinoi-app-01 sudo[1887]:    cuong : TTY=pts/0 ; PWD=/home/an ; USER=root ; COMMAND=/usr/bin/systemctl restart cron
Nov 23 23:36:10 toinoi-app-01 sudo[1893]:    cuong : TTY=pts/0 ; PWD=/home/an ; USER=root ; COMMAND=/usr/bin/journalctl -u cron -n 2 --no-pager
```

Mỗi lần dùng `sudo`, một dòng nhật ký ghi: ai, từ terminal nào, đang ở thư mục nào, chạy dưới danh nghĩa ai, lệnh gì. Đây là lý do thứ hai để không chia sẻ tài khoản `root`: với `sudo`, nhật ký trả lời được «ai đã khởi động lại cron lúc 2 giờ sáng». Lần bị từ chối cũng được ghi, với `command not allowed`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sudo-sudoers/ssh-studio-04.webp" alt="Terminal của SSH Studio: sudo journalctl _COMM=sudo lọc các dòng cuong : TTY=pts/2 ; PWD=/home/an ; USER=root ; COMMAND=/usr/bin/systemctl restart cron và journalctl -u cron" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · mỗi lệnh sudo của Cường đều có dòng trong journal</figcaption></figure>

## 6. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Xem mình được làm gì | `sudo -l` |
| Xem người khác được làm gì | `sudo -l -U TÊN` |
| Thêm quy tắc mới | `sudo visudo -f /etc/sudoers.d/TÊN`, rồi `sudo chmod 440` file đó |
| Kiểm tra cú pháp toàn bộ | `sudo visudo -c` |
| Kiểm tra một file trước khi cài | `sudo visudo -c -f FILE` |
| Thu hồi | `sudo rm /etc/sudoers.d/TÊN` |
| Xem nhật ký sudo | `sudo journalctl _COMM=sudo` |

## 7. Lỗi thường gặp

**Thêm người vào nhóm `sudo` cho nhanh.** Đó là quyền `root` đầy đủ. Hãy hỏi «người này cần chạy những lệnh nào?».

**Sửa `/etc/sudoers` bằng `vim` hay `echo >>`.** Một lỗi cú pháp và `sudo` ngừng hoạt động cho mọi người. Luôn dùng `visudo`.

**Đặt tên file trong `sudoers.d` có dấu chấm hoặc kết thúc bằng `~`.** `sudo` bỏ qua những file đó (để tránh đọc file sao lưu của trình soạn thảo). `cuong.conf` sẽ **không** có tác dụng; `cuong` thì có.

**Dùng đường dẫn tương đối hoặc thiếu tham số.** `cuong ALL=(root) NOPASSWD: systemctl` sai cú pháp; `/usr/bin/systemctl` không tham số thì cho **mọi** lệnh `systemctl`.

**Cấp một lệnh có thể mở shell** (bảng ở mục 3).

## Bài tập

1. Viết quy tắc cho `binh` được chạy `apt update` và `apt upgrade` không cần mật khẩu, kiểm tra bằng `visudo -c` và `sudo -l -U binh`. Vì sao `apt install *` lại là một quyền rất rộng?
2. Thu hồi quyền của Cường (xoá file), kiểm tra bằng `sudo -l -U cuong`. Sau bài tập, đặt lại bằng `npm run setup -- 25` (lab tạo lại file đúng).
3. Đọc `man sudoers`, mục `SUDOEDIT` và `NOEXEC`. `NOEXEC:` trước một lệnh có tác dụng gì với cạm bẫy «thoát ra shell»? Nó có chặn được mọi trường hợp không?
4. Trong nhật ký sudo, tìm dòng của chính bạn khi chạy `sudo visudo`. Dòng đó có cho biết bạn đã sửa gì không? Bạn sẽ dùng gì để biết nội dung đã đổi?

## Đọc tiếp

[LNX-025 · Cấu hình sshd an toàn](/articles/linux-sshd-config-an-toan): tắt đăng nhập bằng mật khẩu, tắt `root`, chỉ cho nhóm `ssh-users` vào, mà không tự khoá mình ngoài cửa.
