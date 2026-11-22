---
slug: linux-nguoi-dung-nhom-quyen-file
title: "Người dùng, nhóm và quyền file trên Linux: chmod, chown và sudo"
summary: "Đọc chín ký tự rwx của ls -l theo ba lớp chủ sở hữu, nhóm, người khác; quyền trên thư mục khác quyền trên file thế nào; chmod dạng chữ và dạng số; umask; vì sao chown cần root và sudo cho mượn quyền ra sao; vì sao thêm người dùng vào nhóm phải đăng nhập lại. Kèm hộp thoại Đổi quyền của SSH Studio."
category: devops-linux
level: intermediate
tags: [linux, lnx-007, permissions, chmod, chown, sudo, umask, ssh-studio, toi-noi]
language: vi
---

Ở bài trước, An chép cả thư mục `/opt/toinoi/bao-gia` và nhận một dòng lỗi: `cp: cannot open '/opt/toinoi/bao-gia/bi-mat.env' for reading: Permission denied`. Ba file kia chép được, file này thì không. Cùng một thư mục, cùng một người, cùng một lệnh.

`Permission denied` là thông báo lỗi bạn sẽ gặp nhiều nhất trên Linux, và cũng là thông báo hay bị «sửa» sai nhất, bằng cách gõ `sudo` trước mọi lệnh hoặc `chmod 777` lên mọi thứ. Cả hai cách đều làm lỗi biến mất và đều tạo ra vấn đề lớn hơn. Bài này giải thích hệ thống quyền đủ kỹ để bạn đọc một dòng `ls -l` và biết chính xác ai làm được gì.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-007 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-xem-sua-file-van-ban">LNX-006</a>. Trạng thái đầu bài: <code>npm run setup -- 07</code>. Sau bài này, bạn giải thích được mọi <code>Permission denied</code> trong lab, đặt quyền bằng <code>chmod</code> theo cả hai dạng, biết khi nào cần <code>sudo</code>, và hiểu vì sao thay đổi nhóm chỉ có hiệu lực ở phiên đăng nhập sau.</p>
</div>

## 1. Người dùng và nhóm

```text
an@toinoi-app-01:~$ whoami
an
an@toinoi-app-01:~$ id
uid=1000(an) gid=1000(an) groups=1000(an),27(sudo)
an@toinoi-app-01:~$ id binh
uid=1001(binh) gid=1001(binh) groups=1001(binh),27(sudo),2000(toinoi)
an@toinoi-app-01:~$ getent passwd an binh baogia
an:x:1000:1000:Nguyen Van An:/home/an:/bin/bash
binh:x:1001:1001:Tran Thi Binh:/home/binh:/bin/bash
baogia:x:990:2000::/opt/toinoi/bao-gia:/usr/sbin/nologin
an@toinoi-app-01:~$ getent group toinoi sudo
toinoi:x:2000:binh
sudo:x:27:an,binh
```

Mỗi tiến trình trên Linux chạy dưới danh nghĩa một **người dùng** (user) và một tập **nhóm** (group). Kernel không làm việc với tên; nó làm việc với số: **UID** cho người dùng, **GID** cho nhóm. `id` in cả hai:

- `uid=1000(an)`: bạn là người dùng số 1000, tên `an`.
- `gid=1000(an)`: **nhóm chính** của bạn. Ubuntu tạo cho mỗi người một nhóm riêng trùng tên. File bạn tạo ra mặc định thuộc nhóm này.
- `groups=1000(an),27(sudo)`: mọi nhóm bạn thuộc về. `sudo` là nhóm được phép dùng lệnh `sudo`.

`binh` thuộc thêm nhóm `toinoi`; `an` thì chưa. Chi tiết này sẽ quan trọng ở cuối bài.

`getent passwd` đọc cơ sở dữ liệu người dùng (file `/etc/passwd`). Mỗi dòng có bảy trường cách nhau bằng dấu hai chấm:

```text
baogia : x : 990 : 2000 :  : /opt/toinoi/bao-gia : /usr/sbin/nologin
tên     mk   UID   GID   mô tả   thư mục riêng      shell
```

`baogia` là **tài khoản dịch vụ**: UID nhỏ hơn 1000, shell là `/usr/sbin/nologin` nên không ai đăng nhập bằng nó được. Nó tồn tại để dịch vụ `bao-gia` chạy dưới một danh nghĩa riêng, chỉ có quyền trên file của chính dịch vụ. Chữ `x` ở trường thứ hai nghĩa là mật khẩu (đã băm) nằm ở file khác, `/etc/shadow`.

Có một người dùng đặc biệt: **`root`**, UID 0. Kernel bỏ qua mọi kiểm tra quyền với `root`.

## 2. Đọc quyền của file

```text
an@toinoi-app-01:~$ ls -l /opt/toinoi/bao-gia
total 16
-rw-r--r-- 1 baogia toinoi 275 Nov 20 09:15 appsettings.json
-rw-r--r-- 1 baogia toinoi 188 Nov 20 09:15 bang-gia.csv
-rw------- 1 baogia toinoi 126 Nov 20 09:20 bi-mat.env
-rw-r--r-- 1 baogia toinoi 403 Nov 20 09:20 kiem-tra-cau-hinh.sh
an@toinoi-app-01:~$ cat /opt/toinoi/bao-gia/bi-mat.env
cat: /opt/toinoi/bao-gia/bi-mat.env: Permission denied
an@toinoi-app-01:~$ ls -l /etc/passwd /etc/shadow
-rw-r--r-- 1 root root   1168 Nov 22 22:26 /etc/passwd
-rw-r----- 1 root shadow  764 Nov 22 22:26 /etc/shadow
an@toinoi-app-01:~$ cat /etc/shadow
cat: /etc/shadow: Permission denied
```

Lấy hai dòng để so:

```text
-rw-r--r-- 1 baogia toinoi 188 Nov 20 09:15 bang-gia.csv
-rw------- 1 baogia toinoi 126 Nov 20 09:20 bi-mat.env
```

Sau ký tự đầu chỉ loại file là **chín ký tự quyền, chia làm ba nhóm ba**:

```text
 rw-      r--      r--
 chủ      nhóm     người khác
 (baogia) (toinoi) (tất cả những ai còn lại)
```

Mỗi nhóm ba có ba vị trí theo thứ tự cố định: `r` (*read*, đọc), `w` (*write*, ghi), `x` (*execute*, chạy). Dấu `-` nghĩa là không có quyền đó.

Khi một tiến trình mở file, kernel chọn **đúng một** trong ba lớp, theo thứ tự:

1. Tiến trình chạy dưới danh nghĩa **chủ sở hữu** của file? Dùng ba ký tự đầu.
2. Không. Tiến trình thuộc **nhóm** của file? Dùng ba ký tự giữa.
3. Không. Dùng ba ký tự cuối, **người khác**.

Áp dụng cho `an` (không phải `baogia`, không thuộc nhóm `toinoi`):

| File | Lớp áp dụng cho `an` | Quyền | Kết quả |
|---|---|---|---|
| `bang-gia.csv` | Người khác | `r--` | Đọc được, không ghi được |
| `bi-mat.env` | Người khác | `---` | Không làm được gì: `Permission denied` |
| `/etc/shadow` (`root:shadow`, `rw-r-----`) | Người khác | `---` | `Permission denied` |

Đó là lời giải cho lỗi của `cp` ở bài trước. `bi-mat.env` chứa khoá gọi dịch vụ bản đồ; chỉ tài khoản `baogia` cần đọc nó, nên chỉ `baogia` được đọc. `/etc/shadow` chứa mật khẩu đã băm của mọi tài khoản; còn `/etc/passwd` ai cũng đọc được, vì nhiều chương trình cần đổi UID thành tên.

Thông báo lỗi cho biết ai từ chối: `cat: …: Permission denied` do `cat` in ra, sau khi kernel từ chối cho nó mở file.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nguoi-dung-nhom-quyen-file/ssh-studio-01.webp" alt="Terminal của SSH Studio với whoami, id, id binh, getent passwd, getent group, ls -l /opt/toinoi/bao-gia và hai lỗi Permission denied khi cat bi-mat.env và cat /etc/shadow" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · danh tính của <code>an</code> và hai file <code>an</code> không được đọc</figcaption></figure>

## 3. Quyền thực thi

An chép script kiểm tra cấu hình về thư mục nháp và thử chạy:

```text
an@toinoi-app-01:~$ cp /opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh /opt/toinoi/bao-gia/bang-gia.csv du-an/bao-gia/
an@toinoi-app-01:~$ cd du-an/bao-gia
an@toinoi-app-01:~/du-an/bao-gia$ ls -l
total 20
-rw-r--r-- 1 an an  188 Nov 22 22:27 bang-gia.csv
drwxrwxr-x 2 an an 4096 Nov 22 22:27 cau-hinh
-rw-r--r-- 1 an an  403 Nov 22 22:27 kiem-tra-cau-hinh.sh
drwxrwxr-x 2 an an 4096 Nov 22 22:27 releases
drwxrwxr-x 2 an an 4096 Nov 22 22:27 shared
an@toinoi-app-01:~/du-an/bao-gia$ ./kiem-tra-cau-hinh.sh
-bash: ./kiem-tra-cau-hinh.sh: Permission denied
an@toinoi-app-01:~/du-an/bao-gia$ chmod u+x kiem-tra-cau-hinh.sh
an@toinoi-app-01:~/du-an/bao-gia$ ls -l kiem-tra-cau-hinh.sh
-rwxr--r-- 1 an an 403 Nov 22 22:27 kiem-tra-cau-hinh.sh
an@toinoi-app-01:~/du-an/bao-gia$ ./kiem-tra-cau-hinh.sh
bao-gia: bang gia co 4 loai dich vu
bao-gia: cau hinh hop le
```

File là của `an`, `an` có `rw-`, nhưng **không có `x`** nên bash từ chối chạy. Một file chỉ chạy được như một chương trình khi người chạy có quyền `x` trên nó. Điều này đúng cả với script lẫn chương trình đã biên dịch, và là lý do một file tải về không tự nhiên chạy được.

`chmod u+x` thêm quyền chạy cho chủ sở hữu. Sau đó `ls -l` hiện `-rwxr--r--`, và trên terminal tên file đổi sang màu xanh lá.

Vì sao phải gõ `./` ở đầu? Khi bạn gõ một tên lệnh không có dấu `/`, bash chỉ tìm trong các thư mục liệt kê ở biến `PATH` (như `/usr/bin`), **không tìm trong thư mục hiện tại**. `./kiem-tra-cau-hinh.sh` là một đường dẫn («file này, ở ngay đây») nên bash chạy đúng file đó. Thiết kế này có chủ ý: nó ngăn việc ai đó đặt một file tên `ls` trong `/tmp` và chờ bạn `cd` vào đó.

## 4. `chmod`: hai cách viết

```text
an@toinoi-app-01:~/du-an/bao-gia$ chmod go-r bang-gia.csv
an@toinoi-app-01:~/du-an/bao-gia$ ls -l bang-gia.csv
-rw------- 1 an an 188 Nov 22 22:27 bang-gia.csv
an@toinoi-app-01:~/du-an/bao-gia$ chmod 640 bang-gia.csv
an@toinoi-app-01:~/du-an/bao-gia$ chmod 750 kiem-tra-cau-hinh.sh
an@toinoi-app-01:~/du-an/bao-gia$ stat -c '%a %A %U:%G %n' bang-gia.csv kiem-tra-cau-hinh.sh
640 -rw-r----- an:an bang-gia.csv
750 -rwxr-x--- an:an kiem-tra-cau-hinh.sh
```

### Dạng chữ: thay đổi tương đối

```text
chmod   u    +    x     file
        ai   làm gì  quyền nào
```

| Phần | Giá trị |
|---|---|
| Ai | `u` chủ (*user*), `g` nhóm, `o` người khác (*other*), `a` tất cả |
| Làm gì | `+` thêm, `-` bớt, `=` đặt đúng bằng |
| Quyền | `r`, `w`, `x` |

`chmod go-r` bớt quyền đọc của nhóm và người khác, các quyền còn lại giữ nguyên. Dạng chữ hợp khi bạn muốn **đổi một thứ**.

### Dạng số: đặt toàn bộ

Mỗi quyền có một giá trị: **`r` = 4, `w` = 2, `x` = 1**. Cộng lại cho từng lớp, bạn được một chữ số từ 0 tới 7; ba lớp cho ba chữ số.

| Số | Quyền | | Số | Quyền |
|---|---|---|---|---|
| 7 | `rwx` | | 3 | `-wx` |
| 6 | `rw-` | | 2 | `-w-` |
| 5 | `r-x` | | 1 | `--x` |
| 4 | `r--` | | 0 | `---` |

`chmod 640` = `rw-` cho chủ, `r--` cho nhóm, `---` cho người khác. `chmod 750` = `rwx`, `r-x`, `---`. Dạng số hợp khi bạn muốn **nói rõ trạng thái cuối cùng**, và là dạng tài liệu vận hành hay dùng.

Những con số gặp hằng ngày:

| Số | Dùng cho |
|---|---|
| `644` | File thường ai cũng đọc được |
| `640` | File cấu hình chỉ chủ và nhóm đọc |
| `600` | File bí mật: khoá, mật khẩu |
| `755` | Thư mục và chương trình ai cũng dùng được |
| `750` | Thư mục và script chỉ dành cho chủ và nhóm |
| `700` | Thư mục riêng tư |

`stat -c '%a %A %U:%G %n'` in quyền ở cả hai dạng kèm chủ sở hữu và nhóm, tiện để kiểm tra nhiều file một lúc.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nguoi-dung-nhom-quyen-file/ssh-studio-02.webp" alt="Terminal của SSH Studio: chạy kiem-tra-cau-hinh.sh bị Permission denied, chmod u+x, chạy lại thành công, rồi chmod go-r, chmod 640, chmod 750 và stat in quyền dạng số và dạng chữ của hai file" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · quyền thực thi và <code>chmod</code> dạng chữ, dạng số</figcaption></figure>

### Hộp thoại Đổi quyền của SSH Studio

Ở chế độ **Files**, tới `/home/an/du-an/bao-gia`, chọn `kiem-tra-cau-hinh.sh` rồi bấm nút ổ khoá (**Đổi quyền (chmod)**) trên thanh công cụ của khung Remote server:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nguoi-dung-nhom-quyen-file/ssh-studio-03.webp" alt="Hộp thoại Đổi quyền của SSH Studio cho file kiem-tra-cau-hinh.sh: lưới ba hàng Chủ sở hữu, Nhóm, Người khác và ba cột Đọc, Ghi, Chạy; Chủ sở hữu đánh dấu cả ba, Nhóm đánh dấu Đọc và Chạy, Người khác không có; ô Dạng số ghi 750, bên dưới là rwxr-x---" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · hộp thoại <b>Đổi quyền</b> hiển thị <code>750</code> thành lưới ba lớp × ba quyền</figcaption></figure>

Lưới này chính là chín ký tự của `ls -l` xếp thành bảng: ba hàng là ba lớp, ba cột là `r`, `w`, `x`. Đánh dấu một ô thì ô «Dạng số» và chuỗi `rwxr-x---` bên dưới đổi theo; gõ số thì các ô đánh dấu đổi theo. Bấm **Áp dụng** tương đương `chmod <số> <file>`. Nếu bạn chưa quen đổi giữa hai dạng, hộp thoại này là chỗ luyện tốt.

Hộp thoại đổi được quyền, không đổi được chủ sở hữu hay nhóm, và chỉ có tác dụng với file mà bạn là chủ: nó đi qua SFTP với quyền của tài khoản đăng nhập.

## 5. Quyền trên thư mục: cùng chữ, khác nghĩa

Trên thư mục, `r`, `w`, `x` có nghĩa khác với trên file, và đây là chỗ gây nhầm nhiều nhất.

| Quyền | Trên **file** | Trên **thư mục** |
|---|---|---|
| `r` | Đọc nội dung | **Liệt kê tên** các mục bên trong |
| `w` | Sửa nội dung | **Tạo, xoá, đổi tên** mục bên trong |
| `x` | Chạy như chương trình | **Đi vào và đi qua**: `cd` vào, và truy cập bất kỳ thứ gì bên trong |

```text
an@toinoi-app-01:~/du-an/bao-gia$ ls -ld cau-hinh
drwxrwxr-x 2 an an 4096 Nov 22 22:27 cau-hinh
an@toinoi-app-01:~/du-an/bao-gia$ chmod a-x cau-hinh
an@toinoi-app-01:~/du-an/bao-gia$ ls cau-hinh
ls: cannot access 'cau-hinh/bang-gia.csv.bak': Permission denied
bang-gia.csv.bak
an@toinoi-app-01:~/du-an/bao-gia$ cat cau-hinh/bang-gia.csv.bak
cat: cau-hinh/bang-gia.csv.bak: Permission denied
an@toinoi-app-01:~/du-an/bao-gia$ cd cau-hinh
-bash: cd: cau-hinh: Permission denied
an@toinoi-app-01:~/du-an/bao-gia$ chmod 755 cau-hinh
an@toinoi-app-01:~/du-an/bao-gia$ chmod a-w cau-hinh
an@toinoi-app-01:~/du-an/bao-gia$ touch cau-hinh/thu.txt
touch: cannot touch 'cau-hinh/thu.txt': Permission denied
an@toinoi-app-01:~/du-an/bao-gia$ rm -f cau-hinh/bang-gia.csv.bak
rm: cannot remove 'cau-hinh/bang-gia.csv.bak': Permission denied
an@toinoi-app-01:~/du-an/bao-gia$ chmod 755 cau-hinh
```

Hai thí nghiệm:

**Bỏ `x` (`chmod a-x`).** Thư mục còn `r` nên `ls` vẫn in được **tên** `bang-gia.csv.bak`, nhưng kèm một lỗi: không có `x`, `ls` không lấy được thông tin của file để tô màu. Còn `cat` file bên trong và `cd` vào thì bị từ chối hẳn. Thư mục thiếu `x` giống một cuốn danh bạ bạn đọc được tên nhưng không gọi được cho ai.

Chi tiết về lỗi của `ls`: nó chỉ xuất hiện vì `ls` trong phiên của bạn là alias `ls --color=auto` ([LNX-003](/articles/linux-shell-dong-lenh-tro-giup)). `ls` trần (gõ `\ls cau-hinh`) chỉ cần `r` và in tên mà không phàn nàn.

**Bỏ `w` (`chmod a-w`).** Không tạo được file mới, và **không xoá được file bên trong**, dù file đó là của bạn và bạn có quyền ghi trên nó.

Điều rút ra từ thí nghiệm thứ hai rất quan trọng: **quyền xoá một file nằm ở thư mục chứa nó, không nằm ở file.** Xoá một file là sửa danh sách của thư mục. Chiều ngược lại cũng đúng: nếu bạn có `w` trên thư mục, bạn xoá được file của người khác trong đó, kể cả file bạn không đọc được.

Hệ quả thực tế: để tới được một file, bạn cần `x` trên **mọi** thư mục dọc đường dẫn. Nếu `/srv/toinoi` là `750` và bạn không thuộc nhóm của nó, thì mọi thứ bên dưới đều ngoài tầm với, bất kể quyền của từng file.

## 6. Quyền mặc định của file mới: `umask`

```text
an@toinoi-app-01:~/du-an/bao-gia$ umask
0002
an@toinoi-app-01:~/du-an/bao-gia$ touch moi.txt
an@toinoi-app-01:~/du-an/bao-gia$ mkdir moi
an@toinoi-app-01:~/du-an/bao-gia$ ls -ld moi.txt moi
drwxrwxr-x 2 an an 4096 Nov 22 22:27 moi
-rw-rw-r-- 1 an an    0 Nov 22 22:27 moi.txt
an@toinoi-app-01:~/du-an/bao-gia$ rm -r moi.txt moi
```

File mới có `rw-rw-r--` (664), thư mục mới có `rwxrwxr-x` (775). Con số đó đến từ **`umask`**, mặt nạ các quyền **bị bỏ đi** lúc tạo:

- File mới bắt đầu từ `666` (không bao giờ tự có `x`), thư mục mới từ `777`.
- `umask 0002` bỏ quyền ghi (2) của người khác: `666 − 002 = 664`, `777 − 002 = 775`.

Ubuntu đặt `umask 002` cho người dùng thường vì mỗi người có nhóm riêng, nên cho nhóm quyền ghi không hại gì. Nhiều hệ thống khác (và tài khoản `root`) dùng `022`, cho ra `644` và `755`. Vì vậy **file chứa bí mật phải được `chmod 600` một cách chủ động**; mặc định của hệ thống là «ai cũng đọc được».

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nguoi-dung-nhom-quyen-file/ssh-studio-04.webp" alt="Terminal của SSH Studio: sau chmod a-x cau-hinh thì ls báo lỗi kèm tên file, cat và cd bị Permission denied; sau chmod a-w thì touch và rm trong thư mục bị từ chối; umask in 0002, file và thư mục mới có quyền rw-rw-r-- và rwxrwxr-x" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · quyền trên thư mục và <code>umask</code></figcaption></figure>

## 7. Đổi chủ sở hữu và nhóm; mượn quyền bằng `sudo`

```text
an@toinoi-app-01:~/du-an/bao-gia$ chown binh bang-gia.csv
chown: changing ownership of 'bang-gia.csv': Operation not permitted
an@toinoi-app-01:~/du-an/bao-gia$ chgrp toinoi bang-gia.csv
chgrp: changing group of 'bang-gia.csv': Operation not permitted
an@toinoi-app-01:~/du-an/bao-gia$ sudo chown an:toinoi bang-gia.csv
an@toinoi-app-01:~/du-an/bao-gia$ ls -l bang-gia.csv
-rw-r----- 1 an toinoi 188 Nov 22 22:27 bang-gia.csv
an@toinoi-app-01:~/du-an/bao-gia$ sudo cat /opt/toinoi/bao-gia/bi-mat.env
BANDO_API_KEY=lab-khong-phai-khoa-that-0000
an@toinoi-app-01:~/du-an/bao-gia$ sudo -l
Matching Defaults entries for an on toinoi-app-01:
    env_reset, mail_badpass,
    secure_path=/usr/local/sbin\:/usr/local/bin\:/usr/sbin\:/usr/bin\:/sbin\:/bin\:/snap/bin,
    use_pty

User an may run the following commands on toinoi-app-01:
    (ALL : ALL) ALL
```

**`chown`** đổi chủ sở hữu, **`chgrp`** đổi nhóm; `chown an:toinoi` đổi cả hai. Hai lệnh đầu thất bại với `Operation not permitted`:

- Người dùng thường **không được cho file của mình cho người khác**. Chỉ `root` đổi được chủ sở hữu.
- Người dùng thường chỉ đổi được nhóm của file sang **nhóm mà mình thuộc về**. `an` chưa thuộc `toinoi`.

Để ý chữ: `Operation not permitted` khác `Permission denied`. Lỗi sau là «quyền trên file không cho phép»; lỗi trước là «thao tác này cần đặc quyền mà bạn không có», và `chmod` không giải quyết được.

### `sudo`

**`sudo LỆNH`** chạy **một lệnh** với quyền `root`. Lần đầu trong phiên, `sudo` hỏi mật khẩu **của chính bạn** (không phải của `root`), không hiện ký tự nào khi bạn gõ:

```text
[sudo] password for an:
```

Sau đó `sudo` nhớ trong khoảng 15 phút. (Bản ghi của lab không có dòng hỏi mật khẩu vì runner xác thực sẵn; trong phiên của bạn thì có, như trong ảnh bên dưới.)

`sudo -l` liệt kê những gì bạn được phép: `(ALL : ALL) ALL` nghĩa là `an` được chạy mọi lệnh dưới danh nghĩa mọi người dùng, vì `an` thuộc nhóm `sudo`. Trên server thật, quyền này thường hẹp hơn nhiều (Phần 3).

`sudo cat …/bi-mat.env` đọc được file mà mục 2 từ chối: `root` không bị kiểm tra quyền. Chính vì thế, vài nguyên tắc:

- **`sudo` cho từng lệnh, khi cần.** Đừng mở shell `root` (`sudo -i`) rồi làm việc cả ngày trong đó.
- **Gặp `Permission denied`, hỏi «tại sao» trước khi gõ `sudo`.** Thường câu trả lời là bạn đang ở sai thư mục, hoặc file đó không dành cho bạn.
- **File tạo bằng `sudo` thuộc về `root`.** `sudo cp` rồi quên `chown` là nguồn của rất nhiều `Permission denied` về sau.
- Mọi lệnh `sudo` được ghi vào log hệ thống kèm tên người gõ. Đó là một phần lý do server cấm đăng nhập thẳng bằng `root`: với `sudo`, luôn biết ai đã làm gì.

`chmod 777` («cho tất cả mọi quyền») không phải cách sửa lỗi quyền. Nó cho mọi tài khoản trên máy, kể cả một dịch vụ bị chiếm quyền, sửa và xoá file đó. Hãy xác định **ai** cần **quyền gì**, rồi cấp đúng chừng đó qua chủ sở hữu hoặc nhóm.

## 8. Thêm An vào nhóm `toinoi`

```text
an@toinoi-app-01:~/du-an/bao-gia$ sudo usermod -aG toinoi an
an@toinoi-app-01:~/du-an/bao-gia$ id
uid=1000(an) gid=1000(an) groups=1000(an),27(sudo)
an@toinoi-app-01:~/du-an/bao-gia$ id an
uid=1000(an) gid=1000(an) groups=1000(an),27(sudo),2000(toinoi)
```

`usermod -aG toinoi an` thêm `an` vào nhóm `toinoi`. **`-a` (*append*) là bắt buộc**: `usermod -G toinoi an` không có `-a` sẽ **thay** toàn bộ danh sách nhóm phụ, tức là đưa `an` ra khỏi nhóm `sudo`.

Rồi hai lệnh cho hai kết quả khác nhau:

- `id an` đọc cơ sở dữ liệu: `an` **đã** thuộc `toinoi`.
- `id` (không đối số) in danh tính của **shell đang chạy**: **chưa** có `toinoi`.

Danh sách nhóm của một tiến trình được gán **lúc đăng nhập** và không đổi trong suốt phiên. Thay đổi trong `/etc/group` chỉ áp dụng cho phiên đăng nhập sau. Vì vậy sau khi được thêm vào một nhóm, bạn phải **đăng xuất và đăng nhập lại**: gõ `exit`, rồi bấm **Kết nối lại** trong SSH Studio. Bài sau mở đầu bằng đúng việc đó.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nguoi-dung-nhom-quyen-file/ssh-studio-05.webp" alt="Terminal của SSH Studio: chown và chgrp bị Operation not permitted, sudo chown hỏi mật khẩu rồi thành công, sudo cat đọc được bi-mat.env, sudo -l liệt kê quyền, sau sudo usermod -aG toinoi an thì id chưa có nhóm toinoi còn id an đã có" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sudo</code> hỏi mật khẩu; <code>id</code> và <code>id an</code> cho hai kết quả khác nhau</figcaption></figure>

## 9. Đối chiếu thao tác

| Việc | Lệnh | Trên SSH Studio |
|---|---|---|
| Xem quyền | `ls -l`, `stat` | Chọn file → nút ổ khoá mở hộp thoại Đổi quyền |
| Đổi quyền | `chmod` | Hộp thoại **Đổi quyền** → Áp dụng |
| Đổi chủ sở hữu / nhóm | `sudo chown`, `chgrp` | Không có; dùng terminal |
| Chạy với quyền `root` | `sudo` | Không có trong khung Files; dùng terminal |
| Áp dụng nhóm mới | `exit` rồi đăng nhập lại | `exit` → **Kết nối lại** |

## 10. Lỗi thường gặp

**`sudo` trước mọi thứ.** Lỗi biến mất, nhưng file sinh ra thuộc về `root` và lần sau bạn lại cần `sudo`. Tìm nguyên nhân trước.

**`chmod -R 777`.** Xem mục 7. Thêm vào đó, `-R` đặt `x` lên mọi file thường, biến file dữ liệu thành «chương trình».

**Quên `-a` trong `usermod -aG`.** Mất nhóm `sudo`, và có thể mất luôn đường tự sửa.

**Thêm vào nhóm rồi thử ngay.** Phải đăng nhập lại.

**Sửa quyền của file trong khi vấn đề nằm ở thư mục cha.** Chạy `ls -ld` trên từng cấp của đường dẫn, hoặc `namei -l /duong/dan/toi/file` để in quyền của mọi cấp một lần.

**Không xoá được file dù là chủ của file.** Quyền xoá nằm ở thư mục chứa nó (mục 5).

## Bài tập

1. Đổi `rw-r-----`, `rwxr-xr-x`, `rw-------` sang dạng số. Đổi `664`, `700`, `750` sang dạng chữ. Kiểm tra bằng hộp thoại Đổi quyền.
2. `ls -ld /home/an /home/binh`. Vì sao `ls /home/binh` bị từ chối? Lớp quyền nào áp dụng cho `an`?
3. Tạo `~/thu/bi-mat.txt`, đặt quyền để chỉ bạn đọc và ghi được. Kiểm tra bằng `sudo -u binh cat ~an/thu/bi-mat.txt`. (Cẩn thận: `binh` còn cần gì nữa để tới được file đó? Xem lại mục 5.)
4. `chmod u-w` một thư mục của bạn rồi thử xoá một file bên trong bằng `rm` và bằng nút Xoá của khung Files. Hai bên báo gì?
5. Dự đoán rồi kiểm tra: với `umask 027`, file mới và thư mục mới có quyền gì? (Gõ `umask 027`, tạo thử, rồi `umask 002` để trả lại.)

## Đọc tiếp

[LNX-008 · Chuyển hướng và pipe](/articles/linux-chuyen-huong-pipe): đăng nhập lại để nhận nhóm mới, rồi bắt đầu nối các lệnh với nhau.
