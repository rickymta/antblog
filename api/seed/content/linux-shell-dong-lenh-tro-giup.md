---
slug: linux-shell-dong-lenh-tro-giup
title: "Shell và dòng lệnh: lệnh, tuỳ chọn, đối số và cách tự tra trợ giúp"
summary: "Đọc dấu nhắc của bash, tách một dòng lệnh thành lệnh, tuỳ chọn và đối số, gộp tuỳ chọn ngắn, phân biệt lệnh dựng sẵn, alias và chương trình bằng type, tra trợ giúp bằng --help và man, đọc mã thoát $?. Bài kèm các phím tắt của bash và của terminal SSH Studio để gõ ít mà không nhầm."
category: devops-linux
level: beginner
tags: [linux, lnx-003, bash, shell, man, exit-code, ssh-studio, toi-noi]
language: vi
---

An chép một lệnh từ tài liệu nội bộ của nhóm: `ls -la /opt/toinoi`. Lệnh chạy được. Hôm sau An cần xem thư mục log và gõ `ls -al /var/log/toinoi`, rồi tự hỏi: `-la` với `-al` có khác nhau không? Có viết `-l -a` được không? `--all` trong một bài viết khác có phải cùng một thứ?

Người chép lệnh thì phải nhớ từng lệnh. Người hiểu cấu trúc dòng lệnh thì đọc được lệnh chưa gặp bao giờ, và biết tra ở đâu khi không chắc. Bài này dạy cấu trúc đó.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-003 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-dung-lab-docker-ket-noi-ssh-studio">LNX-002</a>. Cần server lab đang chạy và một phiên SSH Studio tới <code>toinoi-app-01</code> (<code>npm run setup -- 03</code> nếu muốn bắt đầu từ trạng thái sạch). Sau bài này, bạn đọc được mọi dòng lệnh theo ba phần, tự tra được lệnh lạ bằng <code>--help</code> và <code>man</code>, và biết một lệnh thành công hay thất bại qua mã thoát.</p>
</div>

## 1. Dấu nhắc nói gì?

```text
an@toinoi-app-01:~$
```

| Phần | Ý nghĩa |
|---|---|
| `an` | Người dùng mà shell đang chạy dưới danh nghĩa đó |
| `toinoi-app-01` | Tên máy |
| `~` | Thư mục hiện tại. `~` là cách viết tắt thư mục riêng của bạn, `/home/an` |
| `$` | Bạn là người dùng thường. Shell của `root` kết thúc bằng `#` |

Dấu nhắc đổi theo thư mục bạn đang đứng: vào `/var/log` thì nó thành `an@toinoi-app-01:/var/log$`. Trong tài liệu, người ta thường chỉ viết `$` ở đầu để nói «đây là lệnh gõ ở shell», và `#` để nói «lệnh này cần quyền `root`». Khi chép lệnh, **đừng chép dấu `$`**.

Shell bạn đang dùng là bash:

```text
an@toinoi-app-01:~$ echo $SHELL
/bin/bash
```

`echo` in lại những gì đứng sau nó. `$SHELL` là một **biến môi trường**: shell thay `$SHELL` bằng giá trị của biến trước khi chạy `echo`. Biến môi trường là nội dung của Phần 2; ở đây chỉ cần biết dấu `$` đứng trước một cái tên nghĩa là «lấy giá trị».

## 2. Ba phần của một dòng lệnh

```text
ls      -la        /opt/toinoi
lệnh    tuỳ chọn   đối số
```

- **Lệnh** (command): chương trình cần chạy. Luôn là từ đầu tiên.
- **Tuỳ chọn** (option): thay đổi cách lệnh làm việc. Bắt đầu bằng `-` (dạng ngắn, một chữ) hoặc `--` (dạng dài, cả từ).
- **Đối số** (argument): thứ mà lệnh tác động lên, thường là file hoặc thư mục.

Các phần cách nhau bằng **dấu cách**. Shell tách dòng lệnh theo dấu cách trước khi làm bất cứ việc gì khác, nên `ls-la` là một lệnh không tồn tại, và một tên file có dấu cách phải đặt trong dấu nháy.

Thử với `date`, lệnh in ngày giờ:

```text
an@toinoi-app-01:~$ date
Sun Nov 22 22:27:00 +07 2026
an@toinoi-app-01:~$ date +%F
2026-11-22
an@toinoi-app-01:~$ date --utc +%H:%M
15:27
```

Không có gì theo sau, `date` in ngày giờ theo múi giờ của server (`+07`). Đối số `+%F` yêu cầu định dạng năm-tháng-ngày. Tuỳ chọn dài `--utc` đổi sang giờ UTC. Ngày giờ của bạn sẽ khác kết quả trên.

Giờ tới `ls`, lệnh liệt kê thư mục:

```text
an@toinoi-app-01:~$ ls
huong-dan.txt
an@toinoi-app-01:~$ ls -l
total 4
-rw-r--r-- 1 an an 330 Nov 20 08:30 huong-dan.txt
an@toinoi-app-01:~$ ls -a
.  ..  .bash_logout  .bashrc  .cache  .profile  huong-dan.txt
an@toinoi-app-01:~$ ls -l -a
total 28
drwxr-x--- 1 an   an   4096 Nov 22 22:26 .
drwxr-xr-x 1 root root 4096 Nov 22 22:26 ..
-rw-r--r-- 1 an   an    220 Mar 31  2024 .bash_logout
-rw-r--r-- 1 an   an   3771 Mar 31  2024 .bashrc
drwx------ 2 an   an   4096 Nov 22 22:26 .cache
-rw-r--r-- 1 an   an    807 Mar 31  2024 .profile
-rw-r--r-- 1 an   an    330 Nov 20 08:30 huong-dan.txt
```

| Lệnh | Khác gì |
|---|---|
| `ls` | Chỉ tên, không kể file ẩn |
| `ls -l` | Dạng dài (*long*): mỗi mục một dòng, kèm quyền, chủ sở hữu, kích thước, thời điểm sửa |
| `ls -a` | Tất cả (*all*): cả những mục có tên bắt đầu bằng dấu chấm. Trên Linux, đó là quy ước của «file ẩn» |
| `ls -l -a` | Cả hai |

Dòng `total` ở đầu là tổng dung lượng đĩa các mục chiếm, tính theo khối 1 KB. `.` là chính thư mục này và `..` là thư mục cha; bài sau dùng tới chúng. Ý nghĩa từng cột của `ls -l` là nội dung của [LNX-004](/articles/linux-cay-thu-muc-duong-dan) và [LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-dong-lenh-tro-giup/ssh-studio-01.webp" alt="Terminal của SSH Studio với kết quả của date, date +%F, date --utc, ls, ls -l, ls -a và ls -l -a trong thư mục home của an" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cùng một lệnh với các tuỳ chọn khác nhau</figcaption></figure>

Trong ảnh, `.cache` có màu xanh: trên terminal, `ls` tô màu theo loại mục, và xanh dương là thư mục. Khối kết quả dạng chữ trong bài không giữ màu.

### Gộp tuỳ chọn ngắn

Nhiều tuỳ chọn ngắn viết liền được sau một dấu `-`, và thứ tự không quan trọng:

```text
an@toinoi-app-01:~$ ls -la
total 28
drwxr-x--- 1 an   an   4096 Nov 22 22:26 .
drwxr-xr-x 1 root root 4096 Nov 22 22:26 ..
-rw-r--r-- 1 an   an    220 Mar 31  2024 .bash_logout
-rw-r--r-- 1 an   an   3771 Mar 31  2024 .bashrc
drwx------ 2 an   an   4096 Nov 22 22:26 .cache
-rw-r--r-- 1 an   an    807 Mar 31  2024 .profile
-rw-r--r-- 1 an   an    330 Nov 20 08:30 huong-dan.txt
an@toinoi-app-01:~$ ls -la /opt/toinoi
total 12
drwxr-xr-x 1 baogia toinoi 4096 Nov 20 09:20 .
drwxr-xr-x 1 root   root   4096 Nov 22 22:26 ..
drwxr-xr-x 1 baogia toinoi 4096 Nov 20 09:20 bao-gia
```

`ls -l -a`, `ls -la` và `ls -al` là một. Dạng dài của `-a` là `--all`; tuỳ chọn dài không gộp được, phải viết từng cái. Đó là câu trả lời cho ba câu hỏi của An.

Lệnh cuối có thêm đối số `/opt/toinoi`: liệt kê thư mục đó thay vì thư mục hiện tại. Phần lớn lệnh làm việc với file đều theo cách này: không có đối số thì dùng thư mục hiện tại.

Hai điều cần nhớ:

- **Linux phân biệt chữ hoa và chữ thường** ở mọi nơi: `ls -a` khác `ls -A`, `Bang-gia.csv` khác `bang-gia.csv`, lệnh `LS` không tồn tại.
- Quy ước `-x` / `--tu-day-du` rất phổ biến nhưng không phải luật. Vài lệnh lâu đời có cú pháp riêng (`find -name`, `tar xf`, `ps aux`). Khi không chắc, tra.

## 3. Lệnh đó là gì và nằm ở đâu?

```text
an@toinoi-app-01:~$ type ls
ls is aliased to `ls --color=auto'
an@toinoi-app-01:~$ type cd
cd is a shell builtin
an@toinoi-app-01:~$ type date
date is hashed (/usr/bin/date)
```

`type` cho biết shell hiểu một cái tên là gì. Có ba câu trả lời khác nhau ở đây:

- **`cd is a shell builtin`**: `cd` là lệnh dựng sẵn trong bash, không phải một chương trình riêng. Nó phải như vậy, vì đổi thư mục là thay đổi trạng thái của chính shell.
- **`date is hashed (/usr/bin/date)`**: `date` là một chương trình, file thực thi nằm ở `/usr/bin/date`. «hashed» nghĩa là bash đã tìm thấy nó một lần và nhớ vị trí. Với lệnh chưa chạy lần nào trong phiên, bạn sẽ thấy `date is /usr/bin/date`.
- **``ls is aliased to `ls --color=auto'``**: `ls` là một **alias**, tức tên tắt do shell thay bằng một dòng lệnh khác. Ubuntu đặt sẵn alias này trong file `~/.bashrc` để `ls` tô màu. Mỗi lần bạn gõ `ls -l`, bash thực ra chạy `ls --color=auto -l`.

Biết điều này giải thích được một hiện tượng khó hiểu: cùng một lệnh `ls` cho kết quả hơi khác nhau khi bạn gõ tay và khi nó chạy trong một script, vì script không có alias. [LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file) có một ví dụ cụ thể.

## 4. Tra trợ giúp

### `--help`: tóm tắt nhanh

Gần như mọi lệnh đều nhận `--help` và in cách dùng:

```text
an@toinoi-app-01:~$ whoami --help
Usage: whoami [OPTION]...
Print the user name associated with the current effective user ID.
Same as id -un.

      --help        display this help and exit
      --version     output version information and exit

GNU coreutils online help: <https://www.gnu.org/software/coreutils/>
Report any translation bugs to <https://translationproject.org/team/>
Full documentation <https://www.gnu.org/software/coreutils/whoami>
or available locally via: info '(coreutils) whoami invocation'
```

Dòng `Usage:` là cú pháp. Ngoặc vuông `[OPTION]` nghĩa là không bắt buộc; dấu ba chấm nghĩa là lặp lại được. `ls --help` dài hơn nhiều, hơn trăm dòng; bài [LNX-006](/articles/linux-xem-sua-file-van-ban) chỉ cách đọc một kết quả dài từng trang.

Lệnh dựng sẵn của bash như `cd` không có `--help`; dùng `help cd`.

### `man`: tài liệu đầy đủ

```text
an@toinoi-app-01:~$ man -f ls
ls (1)               - list directory contents
an@toinoi-app-01:~$ man -f date passwd
date (1)             - print or set the system date and time
passwd (1)           - change user password
passwd (1ssl)        - OpenSSL application commands
passwd (5)           - the password file
```

`man -f` in một dòng mô tả cho mỗi trang tài liệu có tên đó. Số trong ngoặc là **mục** (section) của tài liệu: mục 1 là lệnh cho người dùng, mục 5 là định dạng file, mục 8 là lệnh quản trị. `passwd` có ở cả mục 1 (lệnh đổi mật khẩu) và mục 5 (định dạng file `/etc/passwd`). Mặc định `man passwd` mở mục 1; muốn mục 5, gõ `man 5 passwd`.

Mở trang tài liệu của `ls`:

```sh
man ls
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-dong-lenh-tro-giup/ssh-studio-02.webp" alt="Trang man của lệnh ls hiển thị trong terminal SSH Studio, gồm các mục NAME, SYNOPSIS, DESCRIPTION và danh sách tuỳ chọn; dòng cuối ghi Manual page ls(1) line 1" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>man ls</code> mở trong trình xem từng trang</figcaption></figure>

`man` chiếm cả màn hình và chờ phím. Các phím cần biết:

| Phím | Tác dụng |
|---|---|
| `Space` / `b` | Xuống / lên một trang |
| `↓` `↑` hoặc `j` `k` | Xuống / lên một dòng |
| `/chuỗi` rồi Enter | Tìm `chuỗi`; `n` tới kết quả kế, `N` về kết quả trước |
| `g` / `G` | Về đầu / tới cuối |
| `q` | Thoát |

Cấu trúc của mọi trang `man` giống nhau: `NAME` (lệnh làm gì, trong một dòng), `SYNOPSIS` (cú pháp), `DESCRIPTION` (từng tuỳ chọn), và thường có `EXAMPLES` ở gần cuối. Mẹo: gõ `/` rồi tên tuỳ chọn, ví dụ `/--all`, để nhảy thẳng tới chỗ cần.

Ảnh Ubuntu dùng cho Docker vốn không có trang `man` (để ảnh nhỏ). Lab của series cài lại, vì trên server thật bạn có chúng và cần chúng.

## 5. Khi gõ sai: thông báo lỗi và mã thoát

```text
an@toinoi-app-01:~$ lss
-bash: lss: command not found
an@toinoi-app-01:~$ ls /khong-co
ls: cannot access '/khong-co': No such file or directory
an@toinoi-app-01:~$ echo $?
2
an@toinoi-app-01:~$ ls /opt
toinoi
an@toinoi-app-01:~$ echo $?
0
```

Hai lỗi đầu khác nhau về bản chất:

- **`-bash: lss: command not found`**: lỗi do **bash** báo. Bash không tìm thấy lệnh nào tên `lss`. Không chương trình nào được chạy.
- **`ls: cannot access '/khong-co': No such file or directory`**: lỗi do **`ls`** báo. Lệnh tồn tại và đã chạy, nhưng đối số trỏ tới thứ không có.

Thói quen đáng tập: đọc chữ đứng trước dấu hai chấm đầu tiên của thông báo lỗi. Đó là tên của chương trình đang phàn nàn, và nó cho bạn biết phải sửa ở đâu.

`$?` là **mã thoát** (exit code) của lệnh vừa chạy. Mọi lệnh kết thúc đều trả về một số: **0 là thành công, khác 0 là thất bại**. `ls /khong-co` trả về 2; `ls /opt` trả về 0. Con số cụ thể khác nhau tuỳ lệnh (trang `man` của lệnh thường có mục EXIT STATUS), nhưng quy ước «0 là ổn» thì không đổi.

Bạn không nhìn thấy mã thoát trừ khi hỏi, nhưng script và các công cụ tự động dựa hoàn toàn vào nó để biết bước trước có thành công không. [LNX-008](/articles/linux-chuyen-huong-pipe) dùng mã thoát để nối lệnh có điều kiện.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-dong-lenh-tro-giup/ssh-studio-03.webp" alt="Terminal của SSH Studio với kết quả man -f ls, man -f date passwd, lỗi command not found của lss, lỗi No such file or directory của ls /khong-co và hai lần echo $? in 2 rồi 0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · hai loại lỗi và mã thoát <code>$?</code></figcaption></figure>

## 6. Gõ ít hơn, nhầm ít hơn

### Phím của bash

| Phím | Tác dụng |
|---|---|
| `Tab` | Tự hoàn thành tên lệnh, tên file, đường dẫn. Gõ `cat hu` rồi `Tab` thành `cat huong-dan.txt`. Nhấn `Tab` hai lần để xem các khả năng |
| `↑` `↓` | Đi qua các lệnh đã gõ |
| `Ctrl+R` | Tìm ngược trong lịch sử lệnh: gõ vài chữ, bash đưa ra lệnh gần nhất có chứa chúng |
| `Ctrl+A` / `Ctrl+E` | Về đầu / tới cuối dòng |
| `Ctrl+W` | Xoá từ đứng trước con trỏ |
| `Ctrl+U` | Xoá từ con trỏ về đầu dòng |
| `Ctrl+C` | Huỷ lệnh đang gõ, hoặc ngắt chương trình đang chạy |
| `Ctrl+L` | Xoá màn hình (như lệnh `clear`) |
| `Ctrl+D` | Kết thúc phiên (như lệnh `exit`) khi dòng lệnh đang trống |

`Tab` quan trọng hơn mức «tiện»: nếu `Tab` không hoàn thành được một đường dẫn, nhiều khả năng bạn đã gõ sai từ trước đó. Người làm lâu trên server gõ `Tab` liên tục để kiểm tra chính mình.

Lệnh `history` in toàn bộ lịch sử lệnh của bạn, kèm số thứ tự. Bash ghi lịch sử vào file `~/.bash_history` khi phiên kết thúc.

### Phím của SSH Studio

Trong terminal, `Ctrl+C` thuộc về shell (ngắt chương trình), nên sao chép và dán dùng tổ hợp khác:

| Phím | Tác dụng |
|---|---|
| `Ctrl+Shift+C` / `Ctrl+Shift+V` | Sao chép vùng chọn / dán. Chuột phải cũng dán |
| `Ctrl+Shift+F` | Tìm trong nội dung terminal ([LNX-009](/articles/linux-tim-kiem-find-grep)) |
| `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | Phóng to / thu nhỏ / đặt lại cỡ chữ terminal |
| `Ctrl+Tab`, `Ctrl+1…9` | Chuyển giữa các tab phiên |
| `F1` hoặc `Ctrl+/` | Mở bảng phím tắt |

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-dong-lenh-tro-giup/ssh-studio-04.webp" alt="Hộp thoại Trợ giúp của SSH Studio ở tab Phím tắt, liệt kê các phím tắt chung và phím tắt cho phiên kèm ô lọc" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · bảng phím tắt mở bằng <code>F1</code></figcaption></figure>

Cẩn thận khi dán lệnh nhiều dòng chép từ trang web: mỗi dấu xuống dòng là một lần Enter, lệnh chạy ngay khi dán. Dán vào một trình soạn thảo để đọc trước, hoặc dán từng dòng.

## 7. Lỗi thường gặp

**Chép cả dấu `$` ở đầu lệnh.** Bash báo `$: command not found`. Dấu `$` trong tài liệu là dấu nhắc, không phải một phần của lệnh.

**Thiếu dấu cách.** `ls-l` là tên một lệnh không tồn tại. `ls -l/opt` thì bị `ls` hiểu thành các tuỳ chọn `-l`, `-/`, `-o`, `-p`, `-t` và báo tuỳ chọn không hợp lệ.

**Nhầm chữ hoa với chữ thường.** `cat Huong-dan.txt` báo không có file, dù `huong-dan.txt` nằm ngay đó.

**Dùng `Ctrl+C` để sao chép.** Trong terminal, `Ctrl+C` ngắt chương trình đang chạy. Dùng `Ctrl+Shift+C`, hoặc bật «copy-on-select» trong trang Cài đặt của SSH Studio.

**Màn hình «treo» sau khi gõ `man`.** `man` đang chờ phím. Nhấn `q`.

## Bài tập

1. Viết lại `ls -l -a -h /var/log` thành dạng gộp tuỳ chọn. `-h` làm gì? Tra bằng `ls --help` hoặc `man ls`, đừng đoán.
2. Chạy `type echo`, `type man`, `type ll`. Giải thích từng kết quả. (`ll` có kết quả khác khi bạn gõ tay trong phiên SSH Studio so với bản ghi của lab; vì sao?)
3. Mở `man date`, tìm chuỗi `%F` bằng phím `/`. `%F` tương đương với chuỗi định dạng nào?
4. Chạy `cat /khong-co` rồi `echo $?`. Mã thoát của `cat` khi không có file là bao nhiêu? Chương trình nào in thông báo lỗi?
5. Gõ `cat /opt/toi` rồi nhấn `Tab` nhiều lần cho tới khi ra đường dẫn đầy đủ của `bang-gia.csv`. Bạn phải gõ thêm những chữ nào giữa các lần `Tab`?

## Đọc tiếp

[LNX-004 · Cây thư mục Linux](/articles/linux-cay-thu-muc-duong-dan): `/opt`, `/var`, `/etc` là gì, và cách đi lại giữa chúng.
