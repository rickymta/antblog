---
slug: linux-cay-thu-muc-duong-dan
title: "Cây thư mục Linux: đường dẫn, cd, ls và khung Files của SSH Studio"
summary: "Linux chỉ có một cây thư mục bắt đầu từ /. Bài giải thích /etc, /var, /opt, /srv, /home dùng để làm gì, phân biệt đường dẫn tuyệt đối và tương đối, đi lại bằng cd, đọc từng cột của ls -l, nhìn cây bằng tree, nhận ra symlink và loại file bằng file và stat, rồi duyệt cùng cây đó trong khung Files."
category: devops-linux
level: intermediate
tags: [linux, lnx-004, filesystem, fhs, cd, ls, symlink, ssh-studio, sftp, toi-noi]
language: vi
---

Tin nhắn của Bình có hai đường dẫn: `/opt/toinoi/bao-gia` và `/var/log/toinoi`. An quen với Windows, nơi chương trình nằm trong `C:\Program Files` và mỗi ổ đĩa là một chữ cái, nên có hai câu hỏi: vì sao chương trình ở `/opt` còn log lại ở `/var`, và ổ đĩa đâu?

Trên Linux không có chữ cái ổ đĩa. Mọi thứ nằm trong **một cây duy nhất** bắt đầu từ `/`, và vị trí của một file trong cây nói lên nó là loại dữ liệu gì. Biết quy ước đó, bạn vào một server lạ vẫn đoán được cấu hình ở đâu, log ở đâu, dữ liệu ở đâu.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-004 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-shell-dong-lenh-tro-giup">LNX-003</a>. Trạng thái đầu bài: <code>npm run setup -- 04</code>. Sau bài này, bạn đi lại được trong cây thư mục bằng đường dẫn tuyệt đối và tương đối, đọc được mọi cột của <code>ls -l</code>, biết các thư mục cấp một dùng để làm gì và duyệt được cùng cây đó trong khung Files.</p>
</div>

## 1. Một cây, một gốc

```text
an@toinoi-app-01:~$ pwd
/home/an
an@toinoi-app-01:~$ ls /
bin                dev   lib                media  proc  sbin                sys  var
bin.usr-is-merged  etc   lib.usr-is-merged  mnt    root  sbin.usr-is-merged  tmp
boot               home  lib64              opt    run   srv                 usr
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cay-thu-muc-duong-dan/ssh-studio-06.webp" alt="Terminal của SSH Studio: pwd in /home/an, ls / liệt kê các thư mục cấp một bin, boot, dev, etc, home, lib, opt, proc, root, run, srv, sys, tmp, usr, var" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>pwd</code> và gốc <code>/</code> của cây thư mục</figcaption></figure>

`pwd` (*print working directory*) in thư mục bạn đang đứng. `ls /` liệt kê **thư mục gốc** (root directory), ký hiệu là một dấu `/`. Mọi file trên máy đều nằm đâu đó bên dưới nó.

Đĩa thứ hai, ổ USB, ổ mạng không có chữ cái riêng: chúng được **gắn** (mount) vào một thư mục trong cây, và từ đó nội dung của chúng hiện ra như một thư mục thường (Phần 5).

Cách tổ chức các thư mục cấp một theo một chuẩn chung gọi là FHS (*Filesystem Hierarchy Standard*). Những thư mục bạn sẽ dùng nhiều:

| Thư mục | Chứa gì | Trên `toinoi-app-01` |
|---|---|---|
| `/home` | Thư mục riêng của từng người dùng | `/home/an`, `/home/binh` |
| `/etc` | **Cấu hình** của hệ thống và của các dịch vụ, dạng văn bản | `/etc/ssh/sshd_config`, `/etc/passwd` |
| `/var` | Dữ liệu **thay đổi** khi hệ thống chạy: log, hàng đợi, bộ đệm | `/var/log/toinoi/bao-gia.log` |
| `/opt` | Phần mềm cài thêm, không do trình quản lý gói của bản phân phối cài | `/opt/toinoi/bao-gia` |
| `/srv` | Dữ liệu mà server này phục vụ ra ngoài | Bài thực hành [LNX-010](/articles/linux-thuc-hanh-thu-muc-trien-khai) dựng `/srv/toinoi` |
| `/usr` | Chương trình và thư viện của bản phân phối | `/usr/bin/ls` |
| `/tmp` | File tạm; bị xoá khi khởi động lại | |
| `/root` | Thư mục riêng của `root` (không nằm trong `/home`) | |
| `/proc`, `/sys`, `/dev` | Không phải file trên đĩa: kernel bày thông tin về tiến trình, phần cứng và thiết bị ra dưới dạng file | |
| `/run` | Dữ liệu trạng thái của các dịch vụ đang chạy; nằm trong RAM | |

Vậy câu hỏi của An có câu trả lời: chương trình `bao-gia` là phần mềm cài thêm nên nằm ở `/opt`; log của nó là dữ liệu lớn dần theo thời gian nên nằm ở `/var/log`. Tách như vậy có lợi ích thực tế: `/var` đầy đĩa là sự cố thường gặp, và người vận hành thường đặt `/var` trên một phân vùng riêng để log không làm đầy phần còn lại của hệ thống.

Các mục `bin.usr-is-merged`, `lib.usr-is-merged`, `sbin.usr-is-merged` là dấu vết của Ubuntu 24.04 về việc gộp `/bin` vào `/usr/bin`; bạn không cần quan tâm tới chúng. Mục 5 giải thích `/bin` hiện là gì.

## 2. Đường dẫn tuyệt đối và tương đối

Một **đường dẫn tuyệt đối** bắt đầu bằng `/` và mô tả vị trí tính từ gốc: `/var/log/toinoi`. Nó chỉ tới cùng một chỗ, bất kể bạn đang đứng ở đâu.

Một **đường dẫn tương đối** không bắt đầu bằng `/` và được tính từ thư mục hiện tại: nếu đang ở `/var/log` thì `toinoi` nghĩa là `/var/log/toinoi`.

`cd` (*change directory*) đổi thư mục hiện tại:

```text
an@toinoi-app-01:~$ cd /var/log
an@toinoi-app-01:/var/log$ pwd
/var/log
an@toinoi-app-01:/var/log$ ls
README            apt            btmp      faillog  lastlog  toinoi
alternatives.log  bootstrap.log  dpkg.log  journal  private  wtmp
an@toinoi-app-01:/var/log$ cd toinoi
an@toinoi-app-01:/var/log/toinoi$ pwd
/var/log/toinoi
an@toinoi-app-01:/var/log/toinoi$ cd ..
an@toinoi-app-01:/var/log$ pwd
/var/log
an@toinoi-app-01:/var/log$ cd ../../opt/toinoi/bao-gia
an@toinoi-app-01:/opt/toinoi/bao-gia$ pwd
/opt/toinoi/bao-gia
```

Đi theo từng bước:

1. `cd /var/log`: đường dẫn tuyệt đối. Dấu nhắc đổi thành `an@toinoi-app-01:/var/log$`.
2. `cd toinoi`: đường dẫn tương đối, vào thư mục con.
3. `cd ..`: **`..` là thư mục cha**. Từ `/var/log/toinoi` lên `/var/log`.
4. `cd ../../opt/toinoi/bao-gia`: lên hai cấp (tới `/`), rồi xuống `opt/toinoi/bao-gia`. `..` dùng được ở bất cứ đâu trong đường dẫn.

Ngoài `..` còn có `.` là **chính thư mục hiện tại**. `.` có vẻ vô dụng cho tới khi bạn cần nói «ngay ở đây»: `cp /etc/hostname .` chép file về thư mục hiện tại, `./kiem-tra-cau-hinh.sh` chạy script nằm ngay đây ([LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file)).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cay-thu-muc-duong-dan/ssh-studio-01.webp" alt="Terminal của SSH Studio: chuỗi lệnh cd và pwd đi từ /var/log vào toinoi, lên một cấp bằng cd .., rồi tới /opt/toinoi/bao-gia bằng đường dẫn tương đối ../../opt/toinoi/bao-gia" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đi lại bằng đường dẫn tuyệt đối và tương đối</figcaption></figure>

Terminal trong ảnh rộng hơn 100 cột nên `ls /` xếp các mục trên một dòng; bản ghi của lab dùng terminal 100 cột nên chia thành ba dòng. Nội dung như nhau.

### Về nhà và quay lại

```text
an@toinoi-app-01:/opt/toinoi/bao-gia$ cd
an@toinoi-app-01:~$ pwd
/home/an
an@toinoi-app-01:~$ cd -
/opt/toinoi/bao-gia
an@toinoi-app-01:/opt/toinoi/bao-gia$ cd ~
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cay-thu-muc-duong-dan/ssh-studio-05.webp" alt="Terminal của SSH Studio: sau chuỗi cd ở trên, cd về /home/an, cd - quay lại /opt/toinoi/bao-gia, cd ~ về home lần nữa" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>cd</code>, <code>cd -</code> và <code>cd ~</code></figcaption></figure>

| Lệnh | Tới đâu |
|---|---|
| `cd` (không đối số) | Thư mục riêng của bạn |
| `cd ~` | Như trên. `~` dùng được trong mọi đường dẫn: `~/tai-lieu` là `/home/an/tai-lieu` |
| `cd -` | Thư mục bạn vừa rời khỏi; lệnh in ra đường dẫn đó. Gõ lần nữa để quay lại |

`cd -` rất hữu ích khi bạn phải qua lại giữa hai thư mục xa nhau, ví dụ thư mục cấu hình và thư mục log của một dịch vụ.

### Chọn loại nào?

- Trong **script và tài liệu**: dùng đường dẫn tuyệt đối. Script chạy từ thư mục nào cũng phải đúng.
- Khi **gõ tay**: đường dẫn tương đối ngắn hơn. Nhưng trước một lệnh xoá hoặc ghi đè, hãy chắc bạn biết mình đang đứng ở đâu. Dấu nhắc cho bạn biết điều đó.

## 3. Đọc một thư mục

```text
an@toinoi-app-01:~$ ls /opt/toinoi/bao-gia
appsettings.json  bang-gia.csv  bi-mat.env  kiem-tra-cau-hinh.sh
an@toinoi-app-01:~$ ls -l /opt/toinoi/bao-gia
total 16
-rw-r--r-- 1 baogia toinoi 275 Nov 20 09:15 appsettings.json
-rw-r--r-- 1 baogia toinoi 188 Nov 20 09:15 bang-gia.csv
-rw------- 1 baogia toinoi 126 Nov 20 09:20 bi-mat.env
-rw-r--r-- 1 baogia toinoi 403 Nov 20 09:20 kiem-tra-cau-hinh.sh
an@toinoi-app-01:~$ ls -lh /var/log/toinoi
total 28K
-rw-r--r-- 1 baogia toinoi 27K Nov 21 12:17 bao-gia.log
an@toinoi-app-01:~$ ls -ld /opt/toinoi/bao-gia
drwxr-xr-x 1 baogia toinoi 4096 Nov 20 09:20 /opt/toinoi/bao-gia
```

Mỗi dòng của `ls -l` có bảy phần. Lấy dòng của `bang-gia.csv`:

```text
-rw-r--r--   1   baogia   toinoi   188   Nov 20 09:15   bang-gia.csv
    ①        ②     ③        ④      ⑤        ⑥              ⑦
```

| | Cột | Ý nghĩa |
|---|---|---|
| ① | Loại và quyền | Ký tự đầu là loại: `-` file thường, `d` thư mục, `l` symlink. Chín ký tự sau là quyền ([LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file)) |
| ② | Số liên kết | Số tên cùng trỏ tới nội dung này. Với file thường gần như luôn là 1 |
| ③ | Chủ sở hữu | Người dùng sở hữu file |
| ④ | Nhóm | Nhóm sở hữu file |
| ⑤ | Kích thước | Tính bằng byte. Thêm `-h` để có đơn vị dễ đọc |
| ⑥ | Thời điểm sửa | Lần cuối **nội dung** thay đổi. File cũ hơn sáu tháng hiện năm thay cho giờ |
| ⑦ | Tên | |

Hai tuỳ chọn hay đi cùng `-l`:

- **`-h`** (*human-readable*): `27K` thay cho `27260`.
- **`-d`** (*directory*): in thông tin của **chính thư mục** thay vì nội dung của nó. Không có `-d`, bạn không xem được quyền của bản thân thư mục `/opt/toinoi/bao-gia`.

Muốn sắp xếp: `ls -lt` theo thời điểm sửa (mới nhất trước), `ls -lS` theo kích thước, thêm `-r` để đảo thứ tự. `ls -ltr` là cách quen thuộc để thấy file vừa thay đổi nằm ở cuối, ngay trên dấu nhắc.

### Nhìn cả cây

```text
an@toinoi-app-01:~$ tree /opt/toinoi
/opt/toinoi
└── bao-gia
    ├── appsettings.json
    ├── bang-gia.csv
    ├── bi-mat.env
    └── kiem-tra-cau-hinh.sh

2 directories, 4 files
```

`tree` vẽ thư mục và mọi thứ bên dưới nó. Lab cài sẵn `tree`; trên nhiều server lệnh này không có sẵn và phải cài (`sudo apt install tree`). `tree -L 2` giới hạn hai cấp, `tree -d` chỉ hiện thư mục. Đừng chạy `tree /`: cây của cả hệ thống có hàng trăm nghìn dòng.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cay-thu-muc-duong-dan/ssh-studio-02.webp" alt="Terminal của SSH Studio với ls, ls -l và ls -ld trên /opt/toinoi/bao-gia, ls -lh trên /var/log/toinoi và cây thư mục do tree /opt/toinoi vẽ" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>ls -l</code>, <code>ls -lh</code>, <code>ls -ld</code> và <code>tree</code></figcaption></figure>

## 4. Cùng cây đó trong khung Files

Chuyển phiên sang chế độ **Files**. Khung bên phải, **Remote server**, mở ở thư mục riêng của bạn. Gõ `/` vào ô đường dẫn rồi Enter:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cay-thu-muc-duong-dan/ssh-studio-03.webp" alt="Chế độ Files của SSH Studio: khung Máy của bạn bên trái trỏ vào thư mục may-cua-ban của lab, khung Remote server bên phải hiển thị thư mục gốc của server với các thư mục boot, dev, etc, home, media, mnt, opt, proc, root, run, srv, sys" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · khung <b>Remote server</b> ở thư mục gốc <code>/</code></figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cay-thu-muc-duong-dan/ssh-studio-07.webp" alt="SSH Studio ở chế độ Chia đôi: terminal phía trên còn kết quả file và stat của bang-gia.csv, khung Files phía dưới mở /opt/toinoi/bao-gia với bốn file appsettings.json, bang-gia.csv, bi-mat.env, kiem-tra-cau-hinh.sh" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <b>Chia đôi</b>: gõ lệnh ở trên, nhìn cùng thư mục ở khung Files bên dưới</figcaption></figure>

| Trên khung Files | Lệnh tương ứng |
|---|---|
| Gõ đường dẫn vào ô đường dẫn rồi Enter | `cd <đường dẫn>` rồi `ls -la` |
| Nhấp đúp một thư mục | `cd <tên>` |
| Nút mũi tên lên (Thư mục cha) | `cd ..` |
| Nút Làm mới | `ls` lại |
| Nhấp tiêu đề cột Tên / Kích thước / Sửa đổi | `ls` với sắp xếp tương ứng |

Hai điều đáng để ý:

- Khung Files **hiện cả file ẩn**, tương đương `ls -a` (không kể `.` và `..`).
- Khung Files và terminal là hai kênh độc lập trên cùng kết nối. `cd` trong terminal **không** làm khung Files đổi thư mục, và ngược lại. Khung Files cũng không tự cập nhật khi bạn tạo file bằng lệnh; bấm **Làm mới**.

Khung bên trái, **Máy của bạn**, là một thư mục trên máy bạn. Trong series, khung này trỏ vào `examples/toi-noi-linux/may-cua-ban`, nơi lab để sẵn file dùng cho bài sau.

## 5. File ẩn, symlink và loại file

```text
an@toinoi-app-01:~$ ls -a
.  ..  .bash_logout  .bashrc  .cache  .lesshst  .profile  huong-dan.txt
an@toinoi-app-01:~$ ls -ld /bin /sbin /lib
lrwxrwxrwx 1 root root 7 Apr 22  2024 /bin -> usr/bin
lrwxrwxrwx 1 root root 7 Apr 22  2024 /lib -> usr/lib
lrwxrwxrwx 1 root root 8 Apr 22  2024 /sbin -> usr/sbin
```

**File ẩn.** Tên bắt đầu bằng dấu chấm thì `ls` không hiện, trừ khi có `-a`. Đây chỉ là quy ước hiển thị, không phải một tính năng bảo mật. Thư mục riêng dùng file ẩn cho cấu hình của từng người: `.bashrc` (cấu hình bash, nơi đặt alias `ls` ở bài trước), `.profile`, `.bash_logout`. `.cache` và `.lesshst` do các chương trình tạo ra khi chạy (`.lesshst` là lịch sử tìm kiếm của trình xem trang mà `man` dùng).

**Symlink.** Dòng `/bin -> usr/bin` bắt đầu bằng chữ `l`: `/bin` không phải một thư mục thật mà là một **liên kết tượng trưng** (symbolic link, symlink) trỏ tới `usr/bin`. Mở `/bin/ls` thì hệ thống đưa bạn tới `/usr/bin/ls`. Các bản phân phối hiện đại gộp `/bin` vào `/usr/bin` và để lại symlink để những script cũ ghi cứng `/bin/sh` vẫn chạy. Bạn sẽ tự tạo symlink ở bài thực hành.

**Loại file.** Phần mở rộng `.csv`, `.sh` trên Linux chỉ là một phần của tên, để người đọc dễ nhận biết. Hệ thống không dựa vào nó. `file` đọc nội dung để nói cho bạn biết một file thực sự là gì:

```text
an@toinoi-app-01:~$ file /opt/toinoi/bao-gia/bang-gia.csv
/opt/toinoi/bao-gia/bang-gia.csv: CSV Unicode text, UTF-8 text
an@toinoi-app-01:~$ file /opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh
/opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh: Bourne-Again shell script, Unicode text, UTF-8 text executable
an@toinoi-app-01:~$ file /usr/bin/ls
/usr/bin/ls: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, BuildID[sha1]=44900829b39271d878615772bb2a6ffa422f0891, for GNU/Linux 3.2.0, stripped
```

`bang-gia.csv` là văn bản UTF-8. `kiem-tra-cau-hinh.sh` là một shell script. `/usr/bin/ls` là file thực thi dạng ELF, định dạng chương trình đã biên dịch của Linux. Trước khi `cat` một file lạ, chạy `file` trước: `cat` một file nhị phân sẽ đổ ký tự rác ra terminal.

**Mọi thông tin về một file.** `stat` in tất cả những gì hệ thống file biết:

```text
an@toinoi-app-01:~$ stat /opt/toinoi/bao-gia/bang-gia.csv
  File: /opt/toinoi/bao-gia/bang-gia.csv
  Size: 188             Blocks: 8          IO Block: 4096   regular file
Device: 0,120   Inode: 538075      Links: 1
Access: (0644/-rw-r--r--)  Uid: (  990/  baogia)   Gid: ( 2000/  toinoi)
Access: 2026-11-20 09:15:00.000000000 +0700
Modify: 2026-11-20 09:15:00.000000000 +0700
Change: 2026-11-22 22:26:53.451433375 +0700
 Birth: 2026-11-22 22:26:53.451338775 +0700
```

Ba mốc thời gian dễ nhầm:

| Mốc | Đổi khi nào |
|---|---|
| `Modify` (mtime) | **Nội dung** file thay đổi. Đây là cột thời gian của `ls -l` |
| `Change` (ctime) | Nội dung **hoặc thông tin về file** (quyền, chủ sở hữu, tên) thay đổi |
| `Access` (atime) | File được đọc. Nhiều hệ thống cập nhật thưa để đỡ ghi đĩa, đừng dựa vào mốc này |

`Change` không phải «ngày tạo». Trong kết quả trên, `Modify` là ngày 20/11 (lúc file được soạn) còn `Change` là lúc lab được dựng (file được chép vào và đổi chủ sở hữu). Số `Inode`, `Device` và hai mốc `Change`, `Birth` trên máy bạn sẽ khác.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cay-thu-muc-duong-dan/ssh-studio-04.webp" alt="Terminal của SSH Studio với ls -a trong thư mục home, ls -ld cho thấy /bin, /lib, /sbin là symlink tới usr, kết quả lệnh file trên ba file và kết quả stat của bang-gia.csv" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · file ẩn, symlink, <code>file</code> và <code>stat</code></figcaption></figure>

## 6. Lỗi thường gặp

**`cd` vào đường dẫn tương đối từ sai chỗ.** `cd toinoi` chỉ đúng khi bạn đang ở `/var/log`. Thông báo `No such file or directory` sau một lệnh `cd` gần như luôn có nghĩa là thư mục hiện tại không như bạn tưởng. Chạy `pwd`.

**Thừa hoặc thiếu dấu `/` ở đầu.** `cd /tai-lieu` tìm `tai-lieu` ở thư mục gốc; `cd tai-lieu` tìm ở thư mục hiện tại. Một ký tự, hai nơi hoàn toàn khác nhau.

**Tưởng khung Files đi theo terminal.** Hai bên độc lập. Đặt đường dẫn riêng cho từng bên.

**Dùng `ls -l <thư mục>` để xem quyền của thư mục.** Lệnh đó liệt kê nội dung. Thêm `-d`.

**Đọc `Change` thành ngày tạo.** Ngày tạo là `Birth`, và không phải hệ thống file nào cũng ghi mốc này.

## Bài tập

1. Từ `/home/an`, viết một lệnh `cd` dùng đường dẫn **tương đối** để tới `/var/log/toinoi`. Rồi quay về bằng `cd -`.
2. `ls -ld /etc /var/log /tmp`. Ba thư mục này thuộc về ai? Quyền của `/tmp` có ký tự gì khác thường ở cuối? (Phần 3 giải thích.)
3. Chạy `ls -lt /etc | head`. File nào trong `/etc` được sửa gần đây nhất, và vì sao lại là lúc dựng lab? (`| head` chỉ lấy 10 dòng đầu; [LNX-008](/articles/linux-chuyen-huong-pipe) giải thích dấu `|`.)
4. Trong khung Files, đi tới `/var/log/toinoi`. File log lớn bao nhiêu? So với `ls -lh`.
5. `file /etc/hostname /bin /dev/null /proc/version`. Giải thích bốn kết quả.

## Đọc tiếp

[LNX-005 · Tạo, sao chép, di chuyển và xoá file](/articles/linux-tao-sao-chep-di-chuyen-xoa-file): bắt đầu thay đổi cây thư mục, bằng lệnh và bằng khung Files.
