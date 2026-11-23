---
slug: linux-quan-ly-nguoi-dung-nhom
title: "Quản lý người dùng và nhóm trên Linux: useradd, mật khẩu, khoá tài khoản và tài khoản dịch vụ"
summary: "Mở đầu Phần 3 của series Linux qua SSH Studio: liệt kê tài khoản trên server, tạo tài khoản cho thành viên mới bằng useradd, đặt mật khẩu tạm và bắt đổi ở lần đăng nhập đầu (làm thật trong SSH Studio), thêm và gỡ nhóm, khoá, đặt hạn và mở khoá tài khoản, và vì sao tài khoản dịch vụ không đăng nhập được."
category: devops-linux
level: beginner
tags: [linux, lnx-021, useradd, usermod, passwd, chage, groups, ssh-studio, toi-noi]
language: vi
---

Nhóm backend của Tới Nơi có thêm người: Cường, bên vận hành, cần vào `toinoi-app-01` để xem log và khởi động lại vài dịch vụ. Bình giao việc tạo tài khoản cho An, kèm ba yêu cầu: Cường phải tự đặt mật khẩu của mình ở lần đăng nhập đầu, Cường thuộc nhóm `toinoi` như mọi người trong nhóm, và nếu Cường nghỉ việc thì phải khoá được tài khoản ngay mà không xoá dữ liệu.

Đến giờ An vẫn đăng nhập bằng mật khẩu, bằng một tài khoản có toàn quyền `sudo`. Phần 3 dọn lại chuyện đó: người dùng và nhóm (bài này), SSH key, host key, `sudo` theo đúng việc, cấu hình `sshd` an toàn, quyền đặc biệt và ACL, agent và passphrase, mã OTP, và cuối cùng là khoá chặt đăng nhập của server.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-021 · Phần 3/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-nguoi-dung-nhom-quyen-file">LNX-007</a> (người dùng, nhóm, quyền). Trạng thái đầu bài: <code>npm run setup -- 21</code> trong <code>examples/toi-noi-linux</code>, rồi kết nối lại bằng SSH Studio. Sau bài này, bạn tạo và quản lý được tài khoản, nhóm, vòng đời của mật khẩu, và biết tài khoản dịch vụ khác tài khoản người dùng ở đâu.</p>
</div>

## 1. Ai có tài khoản trên server?

```text
an@toinoi-app-01:~$ getent passwd | awk -F: '$3 >= 1000 && $3 < 60000 { print $1, $3, $6, $7 }'
an 1000 /home/an /bin/bash
binh 1001 /home/binh /bin/bash
an@toinoi-app-01:~$ getent passwd | wc -l
24
an@toinoi-app-01:~$ awk -F: '$7 !~ /(nologin|false)$/ { print $1, $7 }' /etc/passwd
root /bin/bash
sync /bin/sync
an /bin/bash
binh /bin/bash
```

`/etc/passwd` có 24 dòng, nhưng phần lớn không phải người: đó là **tài khoản hệ thống** mà các gói phần mềm tạo ra (`daemon`, `www-data`, `sshd`…). Trên Ubuntu, quy ước là:

| UID | Loại |
|---|---|
| 0 | `root` |
| 1–999 | Tài khoản hệ thống, tài khoản dịch vụ (`baogia` có UID 990) |
| 1000 trở lên | Người dùng thật |
| 65534 | `nobody` |

Lệnh `awk` đầu tiên lọc người dùng thật theo UID ([LNX-015](/articles/linux-awk-xu-ly-log)). Lệnh thứ ba đặt câu hỏi khác, quan trọng hơn khi soát an ninh: **tài khoản nào có shell đăng nhập được?** Chỉ bốn: `root`, `sync` (một tài khoản lịch sử chạy đúng lệnh `/bin/sync`), `an`, `binh`. Mọi tài khoản còn lại có shell là `nologin` hoặc `false`.

`getent passwd` đọc cùng dữ liệu với `cat /etc/passwd`, nhưng qua cơ chế tra cứu của hệ thống: trên server lấy người dùng từ LDAP hay Active Directory, `getent` thấy cả họ, còn file `/etc/passwd` thì không.

## 2. Tạo tài khoản cho Cường

```text
an@toinoi-app-01:~$ sudo useradd --create-home --shell /bin/bash --comment "Le Van Cuong" cuong
an@toinoi-app-01:~$ id cuong
uid=1002(cuong) gid=1002(cuong) groups=1002(cuong)
an@toinoi-app-01:~$ sudo ls -la /home/cuong
total 24
drwxr-x--- 2 cuong cuong 4096 Nov 23 23:36 .
drwxr-xr-x 1 root  root  4096 Nov 23 23:36 ..
-rw-r--r-- 1 cuong cuong  220 Mar 31  2024 .bash_logout
-rw-r--r-- 1 cuong cuong 3771 Mar 31  2024 .bashrc
-rw-r--r-- 1 cuong cuong  807 Mar 31  2024 .profile
an@toinoi-app-01:~$ ls -A /etc/skel
.bash_logout  .bashrc  .profile
```

| Tuỳ chọn | Ý nghĩa |
|---|---|
| `--create-home` (`-m`) | Tạo `/home/cuong`. Thiếu tuỳ chọn này, Ubuntu tạo tài khoản **không có** thư mục riêng |
| `--shell /bin/bash` (`-s`) | Shell đăng nhập. Mặc định của `useradd` trên Ubuntu là `/bin/sh` |
| `--comment` (`-c`) | Mô tả, thường là họ tên |

Thư mục riêng mới không trống: `useradd` chép mọi thứ trong **`/etc/skel`** (*skeleton*) vào đó. Muốn mọi tài khoản mới có sẵn một `.vimrc` hay một alias, đặt file vào `/etc/skel` trước khi tạo tài khoản.

`useradd` cũng tạo một nhóm riêng trùng tên (`cuong`, GID 1002) làm nhóm chính, theo đúng cách Ubuntu làm với `an` và `binh` ([LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file)).

Ubuntu còn có lệnh `adduser`: một script hỏi từng thông tin (họ tên, mật khẩu…) rồi gọi `useradd`. `adduser` tiện khi gõ tay; trong script và tài liệu vận hành, `useradd` với tuỳ chọn đầy đủ thì rõ ràng và lặp lại được.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-quan-ly-nguoi-dung-nhom/ssh-studio-01.webp" alt="Terminal của SSH Studio: liệt kê người dùng thật và các tài khoản có shell đăng nhập, sudo useradd tạo tài khoản cuong, id cuong, thư mục /home/cuong với ba file chép từ /etc/skel" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tạo tài khoản <code>cuong</code> bằng <code>useradd</code></figcaption></figure>

## 3. Mật khẩu tạm, bắt đổi ở lần đăng nhập đầu

```text
an@toinoi-app-01:~$ sudo passwd -S cuong
cuong L 2026-11-23 0 99999 7 -1
an@toinoi-app-01:~$ echo 'cuong:Tam-thoi-2611' | sudo chpasswd
an@toinoi-app-01:~$ sudo passwd -S cuong
cuong P 2026-11-23 0 99999 7 -1
an@toinoi-app-01:~$ sudo chage -d 0 cuong
an@toinoi-app-01:~$ sudo chage -l cuong
Last password change                                    : password must be changed
Password expires                                        : password must be changed
Password inactive                                       : password must be changed
Account expires                                         : never
Minimum number of days between password change          : 0
Maximum number of days between password change          : 99999
Number of days of warning before password expires       : 7
```

- **`passwd -S`** in trạng thái mật khẩu. Cột thứ hai là chìa khoá: `L` (*locked*) là chưa có mật khẩu dùng được, `P` là đã có, `NP` là mật khẩu rỗng. Tài khoản vừa tạo bằng `useradd` ở trạng thái `L`: chưa ai đăng nhập bằng mật khẩu được.
- **`chpasswd`** đặt mật khẩu từ stdin theo dạng `tên:mật_khẩu`. Lệnh quen thuộc hơn là `sudo passwd cuong`, hỏi mật khẩu hai lần; `chpasswd` dùng được trong script. Đổi lại, mật khẩu nằm trong lịch sử lệnh, nên chỉ dùng cho mật khẩu **tạm** như ở đây.
- **`chage -d 0`** đặt «lần đổi mật khẩu gần nhất» về ngày 0, nghĩa là mật khẩu đã hết hạn. `chage -l` xác nhận: *password must be changed*.

An gửi mật khẩu tạm cho Cường qua một kênh riêng. Cường thêm server vào SSH Studio của mình (username `cuong`, xác thực Password) và kết nối:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-quan-ly-nguoi-dung-nhom/ssh-studio-02.webp" alt="Tab phiên toinoi-app-01 (cuong) trong SSH Studio: thông báo You are required to change your password immediately, lời chào của server, rồi WARNING: Your password has expired, các dòng Current password, New password, Retype new password, passwd: password updated successfully; phía trên là thanh Phiên đã kết thúc với nút Kết nối lại" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · lần đăng nhập đầu của Cường: buộc đổi mật khẩu rồi đăng nhập lại</figcaption></figure>

`sshd` chấp nhận mật khẩu tạm, nhưng PAM báo mật khẩu đã hết hạn, nên thay vì mở shell, server chạy ngay `passwd`: nhập mật khẩu hiện tại, mật khẩu mới hai lần. Đổi xong, phiên kết thúc với thông báo «You must change your password now and login again!», và Cường đăng nhập lại bằng mật khẩu mới. Mật khẩu tạm chỉ dùng được đúng một lần.

```text
an@toinoi-app-01:~$ sudo passwd -S cuong
cuong P 2026-11-23 0 99999 7 -1
```

## 4. Nhóm

```text
an@toinoi-app-01:~$ sudo groupadd trien-khai
an@toinoi-app-01:~$ sudo usermod -aG toinoi,trien-khai cuong
an@toinoi-app-01:~$ id cuong
uid=1002(cuong) gid=1002(cuong) groups=1002(cuong),2000(toinoi),2001(trien-khai)
an@toinoi-app-01:~$ getent group trien-khai toinoi
trien-khai:x:2001:cuong
toinoi:x:2000:binh,an,cuong
an@toinoi-app-01:~$ sudo gpasswd -d cuong trien-khai
Removing user cuong from group trien-khai
an@toinoi-app-01:~$ groups cuong
cuong : cuong toinoi
```

- **`groupadd`** tạo nhóm. Nhóm mới nhận GID tiếp theo (2001).
- **`usermod -aG`** thêm vào nhóm phụ. Như đã cảnh báo ở LNX-007, **`-a` là bắt buộc**: thiếu nó, danh sách nhóm phụ bị thay toàn bộ.
- **`gpasswd -d`** gỡ một người khỏi một nhóm, không đụng tới các nhóm khác. Đây là cách an toàn hơn `usermod -G` khi chỉ muốn bớt một nhóm.
- `getent group` cho thấy thành viên phụ của nhóm. Nhóm chính của một người không được liệt kê ở đây.

Nhớ rằng thay đổi nhóm chỉ có hiệu lực với **phiên đăng nhập sau** của người đó.

## 5. Khoá, đặt hạn và mở khoá

```text
an@toinoi-app-01:~$ sudo passwd -l cuong
passwd: password changed.
an@toinoi-app-01:~$ sudo passwd -S cuong
cuong L 2026-11-23 0 99999 7 -1
an@toinoi-app-01:~$ sudo chage -E 2026-12-31 cuong
an@toinoi-app-01:~$ sudo chage -l cuong
Last password change                                    : Nov 23, 2026
Password expires                                        : never
Password inactive                                       : never
Account expires                                         : Dec 31, 2026
Minimum number of days between password change          : 0
Maximum number of days between password change          : 99999
Number of days of warning before password expires       : 7
an@toinoi-app-01:~$ sudo chage -E -1 cuong
an@toinoi-app-01:~$ sudo passwd -u cuong
passwd: password changed.
an@toinoi-app-01:~$ sudo passwd -S cuong
cuong P 2026-11-23 0 99999 7 -1
```

| Lệnh | Tác dụng | Không chặn được |
|---|---|---|
| `passwd -l` | Khoá **mật khẩu**: thêm `!` vào đầu mật khẩu đã băm trong `/etc/shadow` | Đăng nhập bằng SSH key |
| `chage -E NGÀY` | Tài khoản **hết hạn** từ ngày đó: mọi cách đăng nhập đều bị từ chối | |
| `usermod -s /usr/sbin/nologin` | Đổi shell: đăng nhập được nhưng không có shell | Truyền file qua SFTP, tunnel |
| `chage -E -1`, `passwd -u` | Bỏ hạn, mở khoá mật khẩu | |

Dòng đầu của bảng là điều hay bị hiểu sai nhất. **`passwd -l` không khoá tài khoản**, nó chỉ làm mật khẩu không dùng được. Người có SSH key trong `~/.ssh/authorized_keys` vẫn vào được. Khi một người rời nhóm, dùng cả hai: `passwd -l` và `chage -E 1` (hết hạn từ ngày 2/1/1970), hoặc `usermod --expiredate 1`. Rồi xoá key của họ ([LNX-022](/articles/linux-ssh-key-ssh-studio)).

Không xoá tài khoản ngay: `userdel -r` xoá cả thư mục riêng, và file họ để lại ở chỗ khác sẽ thuộc về một UID không còn tên, dễ bị một tài khoản mới «thừa kế». Khoá trước, kiểm kê dữ liệu, rồi mới xoá.

Ở cuối mục này, An bỏ hạn và mở khoá lại cho Cường, vì Cường vẫn làm việc ở các bài sau.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-quan-ly-nguoi-dung-nhom/ssh-studio-03.webp" alt="Terminal của SSH Studio: groupadd trien-khai, usermod -aG thêm cuong vào toinoi và trien-khai, gpasswd -d gỡ khỏi trien-khai, passwd -l khoá, chage -E đặt hạn 31/12/2026 và chage -l hiển thị Account expires Dec 31, 2026, rồi bỏ hạn và mở khoá" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · nhóm, khoá và hạn của tài khoản</figcaption></figure>

## 6. Tài khoản dịch vụ

```text
an@toinoi-app-01:~$ getent passwd baogia
baogia:x:990:2000::/opt/toinoi/bao-gia:/usr/sbin/nologin
an@toinoi-app-01:~$ sudo su - baogia
This account is currently not available.
an@toinoi-app-01:~$ echo $?
1
an@toinoi-app-01:~$ sudo -u baogia id
uid=990(baogia) gid=2000(toinoi) groups=2000(toinoi)
an@toinoi-app-01:~$ sudo useradd --system --no-create-home --shell /usr/sbin/nologin --gid toinoi bao-cao
an@toinoi-app-01:~$ getent passwd bao-cao
bao-cao:x:999:2000::/home/bao-cao:/usr/sbin/nologin
an@toinoi-app-01:~$ sudo userdel bao-cao
an@toinoi-app-01:~$ getent passwd bao-cao || echo "đã xoá"
đã xoá
```

`baogia`, tài khoản mà dịch vụ báo giá chạy dưới danh nghĩa đó, khác tài khoản người dùng ở ba điểm:

- **UID dưới 1000** (`--system`): công cụ quản trị và trình đăng nhập đồ hoạ coi nó là tài khoản hệ thống.
- **Shell `/usr/sbin/nologin`**: `su - baogia` chạy shell đó, và nó chỉ in «This account is currently not available.» rồi thoát với mã 1. Không ai đăng nhập vào được, kể cả qua SSH.
- **Không có thư mục riêng** (`--no-create-home`), hoặc thư mục riêng là thư mục của dịch vụ (`/opt/toinoi/bao-gia`).

Muốn chạy một lệnh dưới danh nghĩa tài khoản dịch vụ để kiểm tra quyền, dùng `sudo -u baogia LỆNH`: lệnh chạy với UID của `baogia` mà không cần shell của nó.

Mỗi dịch vụ một tài khoản riêng là nguyên tắc «quyền tối thiểu» ở mức hệ điều hành: nếu dịch vụ báo giá bị chiếm quyền, kẻ tấn công chỉ có quyền của `baogia` trên file của `baogia`. Phần 4 cho `bao-gia` chạy bằng systemd dưới đúng tài khoản này.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-quan-ly-nguoi-dung-nhom/ssh-studio-04.webp" alt="Terminal của SSH Studio: getent passwd baogia với shell nologin, sudo su - baogia báo This account is currently not available với mã thoát 1, sudo -u baogia id, tạo rồi xoá tài khoản hệ thống bao-cao" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tài khoản dịch vụ không có shell đăng nhập</figcaption></figure>

## 7. Các file đứng sau

| File | Chứa | Ai đọc được |
|---|---|---|
| `/etc/passwd` | Tên, UID, GID chính, mô tả, thư mục riêng, shell | Mọi người |
| `/etc/shadow` | Mật khẩu đã băm, các mốc đổi mật khẩu, hạn tài khoản | `root`, nhóm `shadow` |
| `/etc/group` | Tên nhóm, GID, thành viên phụ | Mọi người |
| `/etc/gshadow` | Mật khẩu nhóm (hầu như không dùng) | `root` |
| `/etc/login.defs` | Mặc định: khoảng UID, chính sách mật khẩu | Mọi người |

**Đừng sửa các file này bằng vim.** Một dòng hỏng trong `/etc/passwd` có thể khoá mọi người ngoài server. Dùng các lệnh `useradd`, `usermod`, `chage`, `gpasswd`: chúng khoá file trong lúc ghi và kiểm tra định dạng. Nếu thật sự cần sửa tay, có `vipw` và `vigr`, tương tự `visudo` của [LNX-024](/articles/linux-sudo-sudoers).

## 8. Lỗi thường gặp

**`useradd` không có `-m`.** Tài khoản không có thư mục riêng; đăng nhập báo «Could not chdir to home directory».

**`usermod -G` không có `-a`.** Mất mọi nhóm phụ cũ.

**Tưởng `passwd -l` khoá được tài khoản.** SSH key vẫn vào được. Dùng `chage -E`.

**Đặt mật khẩu bằng `echo … | chpasswd` cho mật khẩu thật.** Lộ trong lịch sử lệnh. Dùng `passwd` tương tác, hoặc tốt hơn, không dùng mật khẩu cho SSH (bài sau).

**Xoá tài khoản bằng `userdel -r` khi còn chưa kiểm kê.** Mất dữ liệu, và UID cũ có thể bị cấp lại.

## Bài tập

1. Tạo tài khoản `thuctap` với thư mục riêng, shell bash, thuộc nhóm `toinoi`, mật khẩu tạm buộc đổi ở lần đầu. Kiểm tra bằng `id`, `chage -l`.
2. Đặt cho `thuctap` hạn tài khoản là ngày cuối tháng sau. `passwd -S` có thay đổi không? Cột nào của `chage -l` thay đổi?
3. Viết một lệnh `awk` liệt kê các nhóm có từ hai thành viên phụ trở lên trong `/etc/group`.
4. Thêm một file `.vimrc` vào `/etc/skel` (cần `sudo`), tạo một tài khoản mới và kiểm tra file đó có trong thư mục riêng của tài khoản mới không. Tài khoản đã có trước đó có nhận file không?
5. Khoá và đặt hết hạn cho `thuctap` như khi một người nghỉ việc. Thử `sudo -u thuctap whoami`: lệnh có chạy không? Vì sao điều đó không mâu thuẫn với việc tài khoản đã bị khoá?

## Đọc tiếp

[LNX-022 · SSH key: tạo trong SSH Studio và cài lên server](/articles/linux-ssh-key-ssh-studio): bỏ mật khẩu, đăng nhập bằng key.
