---
slug: linux-setuid-setgid-sticky
title: "setuid, setgid và sticky bit: ba bit quyền đặc biệt và thư mục dùng chung cho cả nhóm"
summary: "Vì sao passwd sửa được /etc/shadow dù bạn không phải root (setuid), làm sao để file mới trong thư mục nhóm luôn thuộc nhóm đó (setgid), vì sao không ai xoá được file của người khác trong /tmp (sticky bit), cách đọc s, S, t, T trong ls -l và số 4/2/1 đứng đầu chmod, rồi dựng thư mục chung 3770 cho nhóm toinoi."
category: devops-linux
level: intermediate
tags: [linux, lnx-026, setuid, setgid, sticky-bit, chmod, permissions, ssh-studio, toi-noi]
language: vi
---

Ở [LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file), quyền của một file gói trong chín ký tự `rwxrwxrwx`. Nhưng có vài chuyện chín ký tự đó không giải thích được. `/etc/shadow` chỉ `root` ghi được, vậy mà An gõ `passwd` thì mật khẩu mới vẫn được lưu vào đó. Ai cũng ghi được vào `/tmp`, vậy mà An không xoá được file Bình để ở đó. Và thư mục log chung của nhóm `toinoi` thì cứ có file thuộc nhóm `an`, làm Bình không ghi tiếp được.

Ba hiện tượng, ba bit quyền đặc biệt: **setuid**, **setgid**, **sticky**.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-026 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-nguoi-dung-nhom-quyen-file">LNX-007</a> (rwx, chmod dạng số), <a href="/articles/linux-quan-ly-nguoi-dung-nhom">LNX-021</a>. Trạng thái đầu bài: <code>npm run setup -- 26</code>. Bài dùng <code>sudo -u binh</code> để đóng vai Bình ngay trong phiên của An.</p>
</div>

## 1. Bốn chữ số của chmod

Bạn đã quen `chmod 750`: ba chữ số cho chủ, nhóm, người khác. Thật ra có thể có **bốn** chữ số; chữ số đầu (thường bị bỏ qua, nghĩa là `0`) gồm ba bit đặc biệt:

| Bit | Số | Trên file chạy được | Trên thư mục | Hiển thị trong `ls -l` |
|---|---|---|---|---|
| **setuid** | 4 | Chạy với quyền của **chủ file** | (không dùng trên Linux) | `s` ở vị trí `x` của chủ |
| **setgid** | 2 | Chạy với quyền của **nhóm của file** | File mới **thừa kế nhóm** của thư mục | `s` ở vị trí `x` của nhóm |
| **sticky** | 1 | (không dùng) | Chỉ **chủ file** (hoặc chủ thư mục, root) mới xoá/đổi tên được file | `t` ở vị trí `x` của người khác |

Chữ thường (`s`, `t`) nghĩa là bit đặc biệt **và** `x` cùng bật; chữ hoa (`S`, `T`) là bit đặc biệt bật nhưng `x` tắt.

## 2. setuid: passwd và sudo

```text
an@toinoi-app-01:~$ ls -l /usr/bin/passwd /usr/bin/sudo /etc/shadow
-rw-r----- 1 root shadow    863 Nov 23 23:36 /etc/shadow
-rwsr-xr-x 1 root root    64152 May 30  2024 /usr/bin/passwd
-rwsr-xr-x 1 root root   282032 Oct  6 20:51 /usr/bin/sudo
an@toinoi-app-01:~$ find /usr/bin /usr/sbin -perm -4000 -type f
/usr/bin/newgrp
/usr/bin/gpasswd
/usr/bin/passwd
/usr/bin/chfn
/usr/bin/umount
/usr/bin/su
/usr/bin/chsh
/usr/bin/mount
/usr/bin/sudo
an@toinoi-app-01:~$ stat -c '%a %A %U %n' /usr/bin/passwd
4755 -rwsr-xr-x root /usr/bin/passwd
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-setuid-setgid-sticky/ssh-studio-03.webp" alt="Terminal của SSH Studio: ls -l cho /usr/bin/passwd và /usr/bin/sudo quyền -rwsr-xr-x, /etc/shadow -rw-r-----, find -perm -4000 liệt kê các chương trình setuid, stat in 4755 cho passwd" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · setuid: <code>passwd</code> và <code>sudo</code> chạy với quyền root</figcaption></figure>

- `/etc/shadow` chỉ `root` ghi được (`-rw-r-----`, nhóm `shadow` chỉ đọc).
- `/usr/bin/passwd` có **`rws`** ở phần của chủ và chủ là `root`. Khi An chạy `passwd`, tiến trình chạy với quyền `root`, nên ghi được vào `/etc/shadow`. Chương trình `passwd` tự giới hạn: người thường chỉ đổi được mật khẩu của chính mình, và phải nhập mật khẩu cũ.
- `sudo` cũng là setuid `root`: đó là cách nó «nâng quyền» được ngay từ đầu.
- `stat -c '%a'` in quyền dạng số: **`4755`**, chữ số `4` là setuid.
- `find -perm -4000` liệt kê mọi file setuid. Danh sách trên một server Ubuntu mới khá ngắn và quen thuộc.

**setuid root là cửa sau mạnh nhất có thể có.** Một chương trình setuid `root` có lỗi là một lỗ hổng leo thang đặc quyền. Vì vậy:

- Hệ thống bỏ qua setuid trên **script** (`#!/bin/bash`); chỉ áp dụng cho chương trình biên dịch.
- Đừng bao giờ `chmod u+s` một chương trình chỉ để «khỏi phải sudo». Dùng sudoers ([LNX-024](/articles/linux-sudo-sudoers)).
- Lưu danh sách `find / -perm -4000` của server lúc sạch; một file setuid lạ xuất hiện sau này là dấu hiệu đáng lo.

## 3. setgid trên thư mục: file mới thừa kế nhóm

Thư mục `/srv/toinoi/bao-gia/shared/logs` thuộc nhóm `toinoi`, nhóm có quyền `rwx`. Nhưng file An tạo trong đó lại thuộc nhóm chính của An (`an`), nên các thành viên khác của `toinoi` chỉ đọc được:

```text
an@toinoi-app-01:~$ ls -ld /srv/toinoi/bao-gia/shared/logs
drwxrwx--- 2 an toinoi 4096 Nov 23 23:36 /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ touch /srv/toinoi/bao-gia/shared/logs/thu-truoc.txt
an@toinoi-app-01:~$ chmod g+s /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ ls -ld /srv/toinoi/bao-gia/shared/logs
drwxrws--- 2 an toinoi 4096 Nov 23 23:36 /srv/toinoi/bao-gia/shared/logs
an@toinoi-app-01:~$ touch /srv/toinoi/bao-gia/shared/logs/thu-sau.txt
an@toinoi-app-01:~$ ls -l /srv/toinoi/bao-gia/shared/logs/thu-*.txt
-rw-rw-r-- 1 an toinoi 0 Nov 23 23:36 /srv/toinoi/bao-gia/shared/logs/thu-sau.txt
-rw-rw-r-- 1 an an     0 Nov 23 23:36 /srv/toinoi/bao-gia/shared/logs/thu-truoc.txt
an@toinoi-app-01:~$ stat -c '%a %A %n' /srv/toinoi/bao-gia/shared/logs
2770 drwxrws--- /srv/toinoi/bao-gia/shared/logs
```

- Trước khi bật setgid, `thu-truoc.txt` thuộc nhóm **`an`**.
- `chmod g+s` bật setgid: quyền nhóm hiện thành **`rws`**, dạng số là `2770`.
- Sau đó, `thu-sau.txt` tự động thuộc nhóm **`toinoi`**, như thư mục. File cũ không đổi; muốn sửa file đã có, dùng `chgrp -R toinoi` một lần.
- Thư mục con tạo sau cũng thừa kế cả bit setgid, nên cả cây giữ được quy tắc.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-setuid-setgid-sticky/ssh-studio-01.webp" alt="Terminal của SSH Studio: thư mục logs drwxrwx---, file thu-truoc.txt thuộc nhóm an; sau chmod g+s thành drwxrws---, file thu-sau.txt thuộc nhóm toinoi; stat in 2770" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · setgid trên thư mục: file tạo sau thừa kế nhóm <code>toinoi</code></figcaption></figure>

Cũng cần `umask` cho phép nhóm ghi (`002`, như Ubuntu đặt cho người dùng có nhóm riêng) thì file mới mới là `rw-rw-r--`. setgid lo phần **nhóm**, umask lo phần **quyền**.

## 4. Sticky bit: /tmp

```text
an@toinoi-app-01:~$ ls -ld /tmp
drwxrwxrwt 6 root root 160 Nov 23 23:35 /tmp
an@toinoi-app-01:~$ sudo -u binh touch /tmp/cua-binh.txt
an@toinoi-app-01:~$ ls -l /tmp/cua-binh.txt
-rw-rw-r-- 1 binh binh 0 Nov 23 23:36 /tmp/cua-binh.txt
an@toinoi-app-01:~$ rm -f /tmp/cua-binh.txt
rm: cannot remove '/tmp/cua-binh.txt': Operation not permitted
an@toinoi-app-01:~$ sudo -u binh rm /tmp/cua-binh.txt
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-setuid-setgid-sticky/ssh-studio-04.webp" alt="Terminal của SSH Studio: /tmp có quyền drwxrwxrwt, binh tạo cua-binh.txt, an chạy rm -f bị Operation not permitted, binh tự xoá được" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · sticky bit của <code>/tmp</code></figcaption></figure>

- `/tmp` có quyền `drwxrwxrwt`: ai cũng ghi được (`rwx` cho người khác), cộng **`t`** cuối cùng là sticky bit.
- Không có sticky bit, quyền ghi trên thư mục nghĩa là xoá được **mọi** file trong đó, kể cả file của người khác (xoá file là sửa thư mục, không phải sửa file).
- Có sticky bit, An không xoá được `cua-binh.txt` dù có quyền ghi vào `/tmp`: `Operation not permitted`. Bình (chủ file) thì xoá được.

## 5. Thư mục dùng chung của nhóm: 3770

Ghép hai bit lại cho một thư mục mà cả nhóm `toinoi` cùng đặt file vào, file nào cũng thuộc nhóm, nhưng không ai xoá nhầm file của người khác:

```text
an@toinoi-app-01:~$ sudo mkdir /srv/toinoi/chung
an@toinoi-app-01:~$ sudo chown root:toinoi /srv/toinoi/chung
an@toinoi-app-01:~$ sudo chmod 3770 /srv/toinoi/chung
an@toinoi-app-01:~$ ls -ld /srv/toinoi/chung
drwxrws--T 2 root toinoi 4096 Nov 23 23:36 /srv/toinoi/chung
an@toinoi-app-01:~$ touch /srv/toinoi/chung/cua-an.txt
an@toinoi-app-01:~$ sudo -u binh touch /srv/toinoi/chung/cua-binh.txt
an@toinoi-app-01:~$ ls -l /srv/toinoi/chung
total 0
-rw-rw-r-- 1 an   toinoi 0 Nov 23 23:36 cua-an.txt
-rw-rw-r-- 1 binh toinoi 0 Nov 23 23:36 cua-binh.txt
an@toinoi-app-01:~$ sudo -u binh rm -f /srv/toinoi/chung/cua-an.txt
rm: cannot remove '/srv/toinoi/chung/cua-an.txt': Operation not permitted
an@toinoi-app-01:~$ sudo -u binh rm /srv/toinoi/chung/cua-binh.txt
an@toinoi-app-01:~$ ls /srv/toinoi/chung
cua-an.txt
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-setuid-setgid-sticky/ssh-studio-05.webp" alt="Terminal của SSH Studio: mkdir /srv/toinoi/chung, chown root:toinoi, chmod 3770, ls -ld hiện drwxrws--T, an và binh mỗi người tạo một file, cả hai thuộc nhóm toinoi" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · thư mục chung <code>3770</code>: setgid cộng sticky</figcaption></figure>

- **`3770`** = setgid (2) + sticky (1), rồi `rwx` cho chủ và nhóm, không gì cho người khác.
- `ls -ld` hiện **`drwxrws--T`**: `s` (setgid và nhóm có `x`), `T` hoa (sticky bật nhưng người khác không có `x`). Chữ `T` hoa ở đây là đúng ý, không phải lỗi.
- File của An và của Bình đều thuộc nhóm `toinoi`.
- Bình không xoá được file của An; xoá file của chính mình thì được.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-setuid-setgid-sticky/ssh-studio-02.webp" alt="Terminal của SSH Studio: binh không xoá được cua-an.txt trong /srv/toinoi/chung (Operation not permitted), xoá được cua-binh.txt của chính mình, ls chỉ còn cua-an.txt" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · sticky bit trong thư mục chung: chỉ chủ file mới xoá được</figcaption></figure>

## 6. Đối chiếu thao tác

| Việc | Lệnh |
|---|---|
| Bật setgid cho thư mục nhóm | `chmod g+s DIR` hoặc `chmod 2770 DIR` |
| Bật sticky | `chmod +t DIR` hoặc thêm `1` ở đầu |
| Thư mục chung cho nhóm | `chown root:NHÓM DIR && chmod 3770 DIR` |
| Xem quyền dạng số | `stat -c '%a %A %n' FILE` |
| Tìm file setuid / setgid | `find / -xdev -perm -4000` / `-perm -2000` |
| Gỡ bit | `chmod u-s`, `chmod g-s`, `chmod -t` |

## 7. Lỗi thường gặp

**`chmod 770` làm mất sticky bit.** Với thư mục, GNU `chmod` dạng số **giữ** setuid/setgid (phải viết `00770` mới xoá được), nhưng **không giữ** sticky: `chmod 770` trên thư mục `3770` cho ra `2770`. Thói quen «chỉnh lại quyền cho chắc» có thể âm thầm gỡ sticky. Dùng dạng chữ (`chmod g+w`, `chmod o-rwx`) khi chỉ muốn đổi một phần, và kiểm tra bằng `stat -c %a`.

**Bật setuid cho script.** Linux bỏ qua; và nếu không bỏ qua thì đó là lỗ hổng.

**Tưởng setgid làm file mới có quyền nhóm ghi.** setgid chỉ đổi **nhóm**; quyền ghi do `umask` quyết định ([LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file)).

**Chép file vào thư mục setgid bằng `cp -a` hoặc `mv`.** `mv` giữ nguyên nhóm cũ của file (nó chỉ đổi tên, không tạo file mới); `cp -a` cố giữ nhóm gốc. Kiểm tra lại bằng `ls -l`.

## Bài tập

1. Chạy `find /usr -perm -2000 -type f`. Những chương trình setgid nào có mặt? Nhóm của chúng là gì, và vì sao chúng cần nhóm đó?
2. Tạo `mkdir /srv/toinoi/chung/bao-cao` bằng tài khoản của Bình. Thư mục con thuộc nhóm nào, có bit setgid không? Có sticky bit không?
3. Lần lượt thử `chmod 770`, `chmod 0770`, `chmod 00770` trên `/srv/toinoi/chung` (đặt lại `3770` trước mỗi lần), xem `stat -c %a`. Bit nào mất ở mỗi lần? Cuối cùng đặt lại `3770`.
4. Vì sao `mv` một file của An từ `~` vào `/srv/toinoi/chung` để lại file thuộc nhóm `an`? Viết lệnh sửa.

## Đọc tiếp

[LNX-027 · ACL: setfacl và getfacl](/articles/linux-acl-setfacl): khi một chủ, một nhóm và «người khác» không đủ để mô tả ai được làm gì.
