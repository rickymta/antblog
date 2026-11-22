---
slug: linux-tao-sao-chep-di-chuyen-xoa-file
title: "Tạo, sao chép, di chuyển và xoá file trên Linux: lệnh và SFTP trong SSH Studio"
summary: "mkdir -p, touch, cp -r, mv, rm -r và rmdir trên server toinoi-app-01, kèm những thông báo lỗi bạn sẽ gặp thật. Bài giải thích wildcard và brace expansion do shell mở rộng trước khi lệnh chạy, vì sao rm không có thùng rác, và cách tải file lên server bằng khung Files của SSH Studio."
category: devops-linux
level: intermediate
tags: [linux, lnx-005, mkdir, cp, mv, rm, wildcard, sftp, ssh-studio, toi-noi]
language: vi
---

An cần một chỗ làm việc trên server: một thư mục để ghi chú, một thư mục nháp để thử cấu hình của `bao-gia` trước khi đụng vào bản đang chạy. An cũng cần đưa một file từ máy mình lên server.

Trên máy cá nhân, đó là vài cú kéo thả. Trên server, đó là năm lệnh: `mkdir`, `cp`, `mv`, `rm`, `touch`. Năm lệnh này đơn giản, nhưng có một khác biệt lớn với trình quản lý file bạn quen dùng: **không có thùng rác và không có nút hoàn tác**. Bài này đi qua từng lệnh cùng những lỗi thật mà lab trả về, rồi làm việc tải file bằng khung Files.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-005 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-cay-thu-muc-duong-dan">LNX-004</a>. Trạng thái đầu bài: <code>npm run setup -- 05</code>. Sau bài này, bạn tạo được cây thư mục, sao chép và di chuyển file, xoá có kiểm soát, chọn nhiều file bằng wildcard, và tải file giữa máy mình với server bằng khung Files.</p>
</div>

## 1. Tạo thư mục và file

```text
an@toinoi-app-01:~$ mkdir tai-lieu
an@toinoi-app-01:~$ mkdir du-an/bao-gia/cau-hinh
mkdir: cannot create directory ‘du-an/bao-gia/cau-hinh’: No such file or directory
an@toinoi-app-01:~$ mkdir -p du-an/bao-gia/cau-hinh
an@toinoi-app-01:~$ tree du-an
du-an
└── bao-gia
    └── cau-hinh

3 directories, 0 files
an@toinoi-app-01:~$ touch tai-lieu/ghi-chu.txt
an@toinoi-app-01:~$ ls -l tai-lieu
total 0
-rw-rw-r-- 1 an an 0 Nov 22 22:27 ghi-chu.txt
```

- `mkdir tai-lieu` tạo một thư mục trong thư mục hiện tại. Lệnh thành công thì **không in gì**. Đây là quy ước chung của Unix: im lặng nghĩa là ổn.
- `mkdir du-an/bao-gia/cau-hinh` thất bại: `mkdir` chỉ tạo cấp cuối cùng, mà `du-an` và `du-an/bao-gia` chưa tồn tại.
- **`mkdir -p`** (*parents*) tạo mọi cấp còn thiếu. `-p` cũng không báo lỗi nếu thư mục đã có, nên nó an toàn để dùng trong script chạy nhiều lần.
- `touch` tạo một file rỗng nếu file chưa có. Nếu file đã có, `touch` chỉ cập nhật thời điểm sửa, không đụng tới nội dung.

File và thư mục mới thuộc về người tạo (`an`), nhóm chính của người đó (`an`). Quyền `-rw-rw-r--` đến từ đâu là nội dung của [LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file).

Về đặt tên: Linux cho phép gần như mọi ký tự trong tên file, kể cả dấu cách và tiếng Việt có dấu. Nhưng dấu cách buộc bạn phải đặt tên trong dấu nháy ở mọi lệnh về sau, nên trên server người ta dùng chữ thường không dấu, nối bằng `-` hoặc `_`.

## 2. Sao chép: `cp`

```text
an@toinoi-app-01:~$ cp /opt/toinoi/bao-gia/bang-gia.csv tai-lieu/
an@toinoi-app-01:~$ cp tai-lieu/bang-gia.csv tai-lieu/bang-gia.csv.bak
an@toinoi-app-01:~$ ls -l tai-lieu
total 8
-rw-r--r-- 1 an an 188 Nov 22 22:27 bang-gia.csv
-rw-r--r-- 1 an an 188 Nov 22 22:27 bang-gia.csv.bak
-rw-rw-r-- 1 an an   0 Nov 22 22:27 ghi-chu.txt
```

Cú pháp là `cp NGUỒN ĐÍCH`. Đích quyết định chuyện gì xảy ra:

| Đích là | Kết quả |
|---|---|
| Một thư mục đã có (`tai-lieu/`) | File được chép **vào** thư mục, giữ tên cũ |
| Một tên chưa có (`tai-lieu/bang-gia.csv.bak`) | File được chép thành tên mới |
| Một file đã có | File đích bị **ghi đè, không hỏi** |

Viết dấu `/` ở cuối khi đích là thư mục (`tai-lieu/`) là thói quen tốt: nếu thư mục không tồn tại, `cp` báo lỗi thay vì lặng lẽ tạo một file tên `tai-lieu`.

Bản sao thuộc về người chép (`an`), không phải chủ của file gốc (`baogia`), và có thời điểm sửa mới. Muốn giữ thời điểm sửa và quyền, dùng `cp -p`; muốn giữ mọi thuộc tính khi chép cả cây, dùng `cp -a`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tao-sao-chep-di-chuyen-xoa-file/ssh-studio-01.webp" alt="Terminal của SSH Studio với các lệnh mkdir, mkdir -p, tree, touch, cp và hai lần ls -l tai-lieu, có lỗi No such file or directory khi mkdir thiếu thư mục cha" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tạo thư mục, tạo file và sao chép</figcaption></figure>

### Sao chép cả thư mục

```text
an@toinoi-app-01:~$ cp /opt/toinoi/bao-gia du-an/ban-sao
cp: -r not specified; omitting directory '/opt/toinoi/bao-gia'
an@toinoi-app-01:~$ cp -r /opt/toinoi/bao-gia du-an/ban-sao
cp: cannot open '/opt/toinoi/bao-gia/bi-mat.env' for reading: Permission denied
an@toinoi-app-01:~$ ls -l du-an/ban-sao
total 12
-rw-r--r-- 1 an an 275 Nov 22 22:27 appsettings.json
-rw-r--r-- 1 an an 188 Nov 22 22:27 bang-gia.csv
-rw-r--r-- 1 an an 403 Nov 22 22:27 kiem-tra-cau-hinh.sh
```

Ba điều xảy ra ở đây:

1. Không có `-r`, `cp` từ chối chép thư mục (`omitting directory`).
2. **`cp -r`** (*recursive*) chép thư mục và mọi thứ bên trong.
3. Một file không chép được: `bi-mat.env` chỉ cho chủ của nó đọc, và `an` không phải chủ. `cp` báo lỗi cho file đó rồi **tiếp tục với các file còn lại**. Kết quả là một bản sao thiếu.

Điểm thứ ba đáng nhớ: một lệnh có thể thành công một phần. Thông báo lỗi là thứ duy nhất cho bạn biết bản sao không đủ, và nếu bạn chép hàng nghìn file thì dòng lỗi đó trôi mất giữa màn hình. Mã thoát (`echo $?`) khác 0 trong trường hợp này; script nên kiểm tra nó.

## 3. Đổi tên và di chuyển: `mv`

```text
an@toinoi-app-01:~$ mv tai-lieu/ghi-chu.txt tai-lieu/nhat-ky.txt
an@toinoi-app-01:~$ mv tai-lieu/bang-gia.csv.bak du-an/bao-gia/cau-hinh/
an@toinoi-app-01:~$ tree tai-lieu du-an
tai-lieu
├── bang-gia.csv
└── nhat-ky.txt
du-an
├── ban-sao
│   ├── appsettings.json
│   ├── bang-gia.csv
│   └── kiem-tra-cau-hinh.sh
└── bao-gia
    └── cau-hinh
        └── bang-gia.csv.bak

5 directories, 6 files
```

Linux không có lệnh «đổi tên» riêng. `mv NGUỒN ĐÍCH` làm cả hai việc, theo cùng quy tắc với `cp`: đích là thư mục đã có thì chuyển vào, đích là tên mới thì đổi tên. `mv` làm việc với thư mục mà không cần `-r`.

Giống `cp`, `mv` **ghi đè file đích mà không hỏi**. Hai tuỳ chọn để phòng thân: `-i` (hỏi trước khi ghi đè) và `-n` (không bao giờ ghi đè).

Trong cùng một hệ thống file, `mv` chỉ đổi tên trong thư mục chứ không chép dữ liệu, nên di chuyển một file 50 GB cũng xong tức thì. Khác hệ thống file (sang một đĩa khác), `mv` phải chép rồi xoá.

## 4. Xoá: `rm` và `rmdir`

```text
an@toinoi-app-01:~$ rm tai-lieu/nhat-ky.txt
an@toinoi-app-01:~$ rmdir du-an/ban-sao
rmdir: failed to remove 'du-an/ban-sao': Directory not empty
an@toinoi-app-01:~$ rm du-an/ban-sao
rm: cannot remove 'du-an/ban-sao': Is a directory
an@toinoi-app-01:~$ rm -r du-an/ban-sao
an@toinoi-app-01:~$ tree du-an
du-an
└── bao-gia
    └── cau-hinh
        └── bang-gia.csv.bak

3 directories, 1 file
```

| Lệnh | Xoá được gì |
|---|---|
| `rm FILE` | File |
| `rmdir THƯ_MỤC` | Thư mục **rỗng**. Báo lỗi nếu còn gì bên trong |
| `rm -r THƯ_MỤC` | Thư mục và **mọi thứ bên trong** |

**`rm` không đưa file vào thùng rác. File bị xoá là mất.** Không có lệnh khôi phục, và trên server thường không có ai ngồi cạnh để hỏi. Vài thói quen làm giảm rủi ro:

- **`ls` trước, `rm` sau.** Chạy `ls` với đúng đối số bạn định đưa cho `rm`, đọc danh sách, rồi nhấn `↑`, sửa `ls` thành `rm`.
- **Dùng `rmdir` khi bạn tin thư mục đã rỗng.** Nếu nó không rỗng, lỗi `Directory not empty` là một lời cảnh báo có ích.
- **`rm -i`** hỏi trước từng file. `rm -I` hỏi một lần khi xoá nhiều hơn ba file hoặc xoá đệ quy.
- **Tránh `-f`** (*force*) trừ khi bạn hiểu rõ mình đang tắt đi cái gì: `-f` bỏ qua mọi câu hỏi và mọi lỗi «không tồn tại». `rm -rf` cộng với một đường dẫn sai là cách kinh điển để mất dữ liệu.
- **Cẩn thận với đường dẫn có biến hoặc dấu cách.** `rm -r du-an /ban-sao` (thừa một dấu cách) xoá `du-an` rồi thử xoá `/ban-sao`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tao-sao-chep-di-chuyen-xoa-file/ssh-studio-02.webp" alt="Terminal của SSH Studio với các lệnh mv, tree tai-lieu du-an, rm, lỗi Directory not empty của rmdir, lỗi Is a directory của rm, rm -r và tree du-an sau khi xoá" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đổi tên, di chuyển và xoá, kèm hai lỗi của <code>rmdir</code> và <code>rm</code></figcaption></figure>

## 5. Nhiều file một lúc: wildcard và ngoặc nhọn

```text
an@toinoi-app-01:~$ touch bao-cao-01.txt bao-cao-02.txt bao-cao-03.txt nhap.tmp
an@toinoi-app-01:~$ ls
bao-cao-01.txt  bao-cao-02.txt  bao-cao-03.txt  du-an  huong-dan.txt  nhap.tmp  tai-lieu
an@toinoi-app-01:~$ ls bao-cao-*
bao-cao-01.txt  bao-cao-02.txt  bao-cao-03.txt
an@toinoi-app-01:~$ echo rm bao-cao-0[12].txt
rm bao-cao-01.txt bao-cao-02.txt
an@toinoi-app-01:~$ rm bao-cao-0[12].txt nhap.tmp
an@toinoi-app-01:~$ ls
bao-cao-03.txt  du-an  huong-dan.txt  tai-lieu
an@toinoi-app-01:~$ mkdir -p du-an/bao-gia/{releases,shared}
an@toinoi-app-01:~$ ls du-an/bao-gia
cau-hinh  releases  shared
```

**Wildcard** (ký tự đại diện) là mẫu khớp với tên file:

| Mẫu | Khớp với |
|---|---|
| `*` | Chuỗi bất kỳ, kể cả rỗng. `bao-cao-*` khớp mọi tên bắt đầu bằng `bao-cao-` |
| `?` | Đúng một ký tự. `bao-cao-0?.txt` |
| `[12]` | Một ký tự trong tập. `bao-cao-0[12].txt` khớp `01` và `02`, không khớp `03` |
| `[0-9]` | Một ký tự trong khoảng |

Điều quan trọng nhất về wildcard: **shell mở rộng mẫu trước khi chạy lệnh**. `rm` không bao giờ thấy dấu `*`; nó nhận một danh sách tên file đã được bash điền sẵn. Lệnh `echo rm bao-cao-0[12].txt` ở trên cho thấy điều đó: `echo` chỉ in lại đối số của nó, và thứ được in ra là `rm bao-cao-01.txt bao-cao-02.txt`.

Từ đó có một kỹ thuật an toàn: **đặt `echo` trước một lệnh có wildcard để xem lệnh thật sẽ là gì**, rồi mới chạy. Nó cũng giải thích vì sao `rm *` nguy hiểm: bạn không nhìn thấy danh sách file trước khi nó bị xoá.

Hai chi tiết: `*` không khớp file ẩn (tên bắt đầu bằng dấu chấm), và nếu mẫu không khớp file nào thì bash đưa nguyên mẫu cho lệnh, khiến lệnh báo «No such file».

**Brace expansion** (mở rộng ngoặc nhọn) trông giống wildcard nhưng khác bản chất: nó **sinh ra** các chuỗi, không cần file nào tồn tại.

```sh
mkdir -p du-an/bao-gia/{releases,shared}
```

Bash viết lại dòng trên thành `mkdir -p du-an/bao-gia/releases du-an/bao-gia/shared`. `{1..5}` sinh dãy số, và ngoặc nhọn lồng nhau được: `{releases/1.4.2,shared/{config,logs}}`. Bài thực hành cuối phần dùng cách này để dựng cả cây thư mục bằng một lệnh.

## 6. Tải file lên bằng khung Files

An có file `lich-truc.txt` trên máy mình và muốn đưa lên `~/tai-lieu` của server. Lab để sẵn file này trong `examples/toi-noi-linux/may-cua-ban`.

Chuyển sang chế độ **Files**:

1. Ở khung **Máy của bạn**, bấm nút chọn thư mục (hoặc gõ đường dẫn) để tới `may-cua-ban`.
2. Ở khung **Remote server**, gõ `/home/an/tai-lieu` vào ô đường dẫn rồi Enter.
3. Chọn `lich-truc.txt` ở khung bên trái. Nút **Upload** sáng lên.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tao-sao-chep-di-chuyen-xoa-file/ssh-studio-03.webp" alt="Chế độ Files của SSH Studio: file lich-truc.txt đang được chọn ở khung Máy của bạn, nút Upload sáng, khung Remote server ở /home/an/tai-lieu có một file bang-gia.csv" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · chọn file ở khung <b>Máy của bạn</b> trước khi bấm <b>Upload</b></figcaption></figure>

4. Bấm **Upload** (hoặc kéo file sang khung bên phải).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tao-sao-chep-di-chuyen-xoa-file/ssh-studio-04.webp" alt="Sau khi upload: khung Remote server có thêm lich-truc.txt 60 B, bảng Truyền file ở dưới ghi lich-truc.txt 60 B / 60 B với trạng thái Hoàn tất" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · file đã nằm trên server, bảng <b>Truyền file</b> báo hoàn tất</figcaption></figure>

Bảng **Truyền file** ở dưới liệt kê các lượt truyền kèm tiến độ; với file lớn bạn thấy thanh tiến độ chạy, huỷ được, và tiếp tục được từ chỗ dừng nếu mất kết nối. Bấm chữ «Truyền file» trên thanh trạng thái để thu bảng lại.

Kiểm tra bằng lệnh:

```text
an@toinoi-app-01:~$ ls -l tai-lieu
total 8
-rw-r--r-- 1 an an 188 Nov 22 22:27 bang-gia.csv
-rw-rw-r-- 1 an an  60 Nov 22 22:27 lich-truc.txt
an@toinoi-app-01:~$ cat tai-lieu/lich-truc.txt
Lịch trực tuần 47 (lab)
Thứ Hai: An
Thứ Ba: Bình
```

File có trên server, thuộc về `an`, vì SFTP chạy dưới đúng tài khoản bạn đăng nhập. SSH Studio giữ thời điểm sửa của file gốc khi tải lên, nên nếu làm theo bằng khung Files, cột thời gian của `lich-truc.txt` trên máy bạn sẽ là lúc file được tạo ở máy local, không phải lúc upload.

Các thao tác khác trên khung Files và lệnh tương ứng:

| Trên khung Files | Lệnh trên server | Ghi chú |
|---|---|---|
| Nút **Thư mục** | `mkdir` | Tạo thư mục mới trong thư mục đang xem |
| Nút bút chì (Đổi tên) | `mv cũ mới` | |
| Nút thùng rác (Xoá) | `rm` / `rm -r` | Xoá thư mục không rỗng phải xác nhận số mục. Vẫn **không có thùng rác** |
| **Upload** / kéo sang phải | (không có lệnh trên server; từ máy bạn là `scp` hoặc `sftp put`) | Trùng tên thì SSH Studio hỏi: Ghi đè / Giữ cả hai / Bỏ qua |
| **Download** / kéo sang trái | `scp` hoặc `sftp get` từ máy bạn | |
| `Ctrl`/`Shift` + nhấp | Wildcard hoặc liệt kê nhiều tên | Chọn nhiều mục |

Khung Files **không sao chép được file từ chỗ này sang chỗ khác trên cùng server**: SFTP không có thao tác đó. Việc ấy làm bằng `cp` trong terminal.

Một khác biệt về an toàn đáng ghi nhận: khi trùng tên, khung Files **hỏi** bạn, còn `cp` và `mv` thì ghi đè luôn.

## 7. Lỗi thường gặp

**`cp` vào thư mục chưa tồn tại.** `cp a.txt sao-luu` khi không có thư mục `sao-luu` sẽ tạo một **file** tên `sao-luu`. Viết `sao-luu/` để nhận lỗi thay vì kết quả sai.

**Thừa dấu cách trong `rm -r`.** Đọc lại lệnh trước khi Enter. Dùng `Tab` để bash tự điền đường dẫn; `Tab` không điền thừa dấu cách.

**Tưởng `cp -r` đã chép đủ.** Đọc lỗi và kiểm tra mã thoát. Với dữ liệu quan trọng, so số file hai bên (`find … | wc -l`, [LNX-009](/articles/linux-tim-kiem-find-grep)).

**Wildcard không khớp file ẩn.** `cp * /noi-khac/` bỏ sót `.env`, `.gitignore`. Chép cả thư mục bằng `cp -r thu-muc/. dich/` hoặc `cp -a`.

**Tạo file trong terminal nhưng khung Files chưa thấy.** Bấm **Làm mới**.

## Bài tập

1. Bằng **một** lệnh `mkdir`, tạo `~/thu/2026/{thang-10,thang-11}/{log,bao-cao}`. Kiểm tra bằng `tree ~/thu`. Trước khi chạy, dùng `echo` để xem bash mở rộng thành gì.
2. Chép `/opt/toinoi/bao-gia/appsettings.json` vào `~/thu` với tên `appsettings.nhap.json`, giữ nguyên thời điểm sửa. Kiểm tra bằng `ls -l`.
3. Tạo `a1.txt`, `a2.txt`, `a10.txt`, `b1.txt`. Mẫu `a?.txt` khớp những file nào? Còn `a*.txt`? `[ab]1.txt`? Trả lời trước, rồi kiểm tra bằng `echo`.
4. Tải `tai-lieu/bang-gia.csv` từ server về thư mục `may-cua-ban` bằng khung Files. Rồi upload lại chính file đó: SSH Studio hỏi gì?
5. Xoá `~/thu` theo cách an toàn nhất bạn nghĩ ra. Bạn đã chạy những lệnh nào trước `rm`?

## Đọc tiếp

[LNX-006 · Xem và sửa file văn bản](/articles/linux-xem-sua-file-van-ban): đọc file log dài, sửa file bằng `vim` và bằng trình sửa file của SSH Studio.
