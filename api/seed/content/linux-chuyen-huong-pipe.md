---
slug: linux-chuyen-huong-pipe
title: "Chuyển hướng và pipe trên Linux: stdin, stdout, stderr và cách nối lệnh"
summary: "Mỗi lệnh có ba luồng: stdin, stdout, stderr. Bài dùng log của dịch vụ bao-gia để học > và >>, 2> và 2>&1, /dev/null, <, rồi nối lệnh bằng pipe: grep | wc -l, cut | sort | uniq -c, tee, && và ||. Kết quả là báo cáo số lỗi theo dịch vụ làm bằng một dòng lệnh, xem song song trong chế độ Chia đôi của SSH Studio."
category: devops-linux
level: advanced
tags: [linux, lnx-008, pipe, redirect, stdin, stdout, stderr, grep, sort, uniq, ssh-studio, toi-noi]
language: vi
---

An đã tìm thấy lỗi đầu tiên trong log bằng `less`. Bình hỏi tiếp: «Sáng 21/11 có tất cả bao nhiêu lỗi? Loại dịch vụ nào bị nhiều nhất?»

Đếm bằng mắt 362 dòng thì được, nhưng log thật có hàng triệu dòng. Không có lệnh nào tên là «đếm lỗi theo dịch vụ». Linux có thứ tốt hơn: nhiều lệnh nhỏ, mỗi lệnh làm một việc, và một cách để **nối đầu ra của lệnh này vào đầu vào của lệnh kia**. Bài này dạy cách nối đó. Cuối bài, câu trả lời cho Bình là một dòng lệnh.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-008 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-nguoi-dung-nhom-quyen-file">LNX-007</a>. Trạng thái đầu bài: <code>npm run setup -- 08</code>. Sau bài này, bạn ghi kết quả và lỗi của lệnh ra file đúng ý, nối lệnh bằng pipe để lọc, cắt, sắp xếp và đếm, và nối lệnh có điều kiện theo mã thoát.</p>
</div>

## 0. Đăng nhập lại để nhận nhóm mới

Bài trước kết thúc bằng việc thêm `an` vào nhóm `toinoi`, và phiên đang mở chưa nhận nhóm đó. Trong terminal, gõ `exit`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-chuyen-huong-pipe/ssh-studio-01.webp" alt="Terminal của SSH Studio sau lệnh exit: lệnh id trước đó chưa có nhóm toinoi, shell in logout, phía trên là thanh thông báo Phiên đã kết thúc với nút Kết nối lại, trạng thái phiên chuyển thành Đã ngắt" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>exit</code> kết thúc phiên; thanh thông báo có nút <b>Kết nối lại</b></figcaption></figure>

`exit` kết thúc shell, `sshd` đóng phiên, và SSH Studio báo «Phiên đã kết thúc». Tab vẫn còn, nội dung terminal vẫn còn. Bấm **Kết nối lại**, nhập mật khẩu, và kiểm tra:

```text
an@toinoi-app-01:~$ id
uid=1000(an) gid=1000(an) groups=1000(an),27(sudo),2000(toinoi)
an@toinoi-app-01:~$ cd tai-lieu
```

`2000(toinoi)` đã có trong danh sách nhóm. Phần còn lại của bài làm trong `~/tai-lieu`.

## 1. Ba luồng của mỗi lệnh

Mỗi chương trình khi chạy được nối sẵn với ba luồng dữ liệu, đánh số 0, 1, 2:

| Số | Tên | Mặc định nối với | Dùng cho |
|---|---|---|---|
| 0 | **stdin** (đầu vào chuẩn) | Bàn phím | Dữ liệu chương trình đọc vào |
| 1 | **stdout** (đầu ra chuẩn) | Màn hình | **Kết quả** |
| 2 | **stderr** (đầu ra lỗi chuẩn) | Màn hình | **Thông báo lỗi** và thông tin chẩn đoán |

Kết quả và lỗi đều hiện trên màn hình nên trông như một, nhưng chúng đi hai đường riêng. Việc tách hai đường là chủ ý: nhờ vậy bạn đưa được kết quả vào file hoặc vào lệnh khác mà thông báo lỗi không lẫn vào dữ liệu.

**Chuyển hướng** (redirection) là bảo shell nối một trong ba luồng tới chỗ khác **trước khi** chạy lệnh. Lệnh không biết và không cần biết: nó vẫn ghi vào luồng số 1 như mọi khi.

## 2. Ghi kết quả ra file: `>` và `>>`

```text
an@toinoi-app-01:~/tai-lieu$ ls /opt/toinoi/bao-gia > danh-sach.txt
an@toinoi-app-01:~/tai-lieu$ cat danh-sach.txt
appsettings.json
bang-gia.csv
bi-mat.env
kiem-tra-cau-hinh.sh
an@toinoi-app-01:~/tai-lieu$ date +%F >> danh-sach.txt
an@toinoi-app-01:~/tai-lieu$ cat danh-sach.txt
appsettings.json
bang-gia.csv
bi-mat.env
kiem-tra-cau-hinh.sh
2026-11-22
an@toinoi-app-01:~/tai-lieu$ date +%F > danh-sach.txt
an@toinoi-app-01:~/tai-lieu$ cat danh-sach.txt
2026-11-22
```

- **`>`** nối stdout vào file. File chưa có thì được tạo; **file đã có thì bị xoá trắng trước khi lệnh chạy**.
- **`>>`** ghi **nối thêm** vào cuối file.

Lệnh cuối minh hoạ cái bẫy: `date +%F > danh-sach.txt` thay toàn bộ nội dung cũ bằng một dòng ngày tháng. Shell xoá trắng file **trước** khi chạy lệnh, nên `sort file > file` cho ra một file rỗng: lúc `sort` mở file để đọc thì nó đã trống. Muốn sửa file tại chỗ, ghi ra file tạm rồi `mv` đè lên.

Để ý thêm: khi ghi vào file, `ls` in mỗi tên một dòng thay vì xếp cột. `ls` kiểm tra xem stdout của nó là terminal hay không, và chỉ xếp cột, tô màu khi đầu ra là terminal. Nhiều lệnh làm như vậy.

Để phòng ghi đè nhầm, `set -o noclobber` khiến `>` từ chối ghi vào file đã tồn tại (dùng `>|` khi thực sự muốn ghi đè).

## 3. Lỗi đi đường riêng: `2>`, `2>&1`, `/dev/null`

```text
an@toinoi-app-01:~/tai-lieu$ ls /opt/toinoi /khong-co
ls: cannot access '/khong-co': No such file or directory
/opt/toinoi:
bao-gia
an@toinoi-app-01:~/tai-lieu$ ls /opt/toinoi /khong-co > ket-qua.txt
ls: cannot access '/khong-co': No such file or directory
an@toinoi-app-01:~/tai-lieu$ cat ket-qua.txt
/opt/toinoi:
bao-gia
an@toinoi-app-01:~/tai-lieu$ ls /opt/toinoi /khong-co 2> loi.txt
/opt/toinoi:
bao-gia
an@toinoi-app-01:~/tai-lieu$ cat loi.txt
ls: cannot access '/khong-co': No such file or directory
an@toinoi-app-01:~/tai-lieu$ ls /opt/toinoi /khong-co > tat-ca.txt 2>&1
an@toinoi-app-01:~/tai-lieu$ cat tat-ca.txt
ls: cannot access '/khong-co': No such file or directory
/opt/toinoi:
bao-gia
an@toinoi-app-01:~/tai-lieu$ ls /opt/toinoi /khong-co 2> /dev/null
/opt/toinoi:
bao-gia
```

Lệnh `ls /opt/toinoi /khong-co` sinh ra cả hai loại đầu ra: danh sách của `/opt/toinoi` (stdout) và lỗi về `/khong-co` (stderr). Đi qua từng biến thể:

| Viết | stdout đi đâu | stderr đi đâu |
|---|---|---|
| (không gì) | Màn hình | Màn hình |
| `> ket-qua.txt` | File | **Màn hình**: lỗi vẫn hiện, file chỉ có kết quả |
| `2> loi.txt` | Màn hình | File |
| `> tat-ca.txt 2>&1` | File | Cùng chỗ với stdout, tức là file |
| `2> /dev/null` | Màn hình | Bỏ đi |

**`2>`** là `>` cho luồng số 2. **`2>&1`** đọc là «nối luồng 2 tới nơi luồng 1 đang trỏ». Thứ tự quan trọng: `> file 2>&1` (trỏ 1 vào file, rồi cho 2 theo 1) đúng; `2>&1 > file` thì 2 theo 1 lúc 1 còn là màn hình, nên lỗi vẫn ra màn hình. Bash có cách viết tắt cho trường hợp phổ biến: `&> file`.

**`/dev/null`** là một file đặc biệt: mọi thứ ghi vào đó biến mất. `2> /dev/null` là cách nói «tôi không quan tâm tới lỗi của lệnh này». Dùng có cân nhắc: giấu lỗi cũng là giấu thông tin bạn có thể cần.

Tách lỗi ra file riêng rất có ích với việc chạy theo lịch: `sao-luu.sh > sao-luu.log 2> sao-luu.err`, rồi chỉ cần kiểm tra file `.err` có rỗng không.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-chuyen-huong-pipe/ssh-studio-02.webp" alt="Terminal của SSH Studio với năm biến thể của ls /opt/toinoi /khong-co: không chuyển hướng, > ket-qua.txt, 2> loi.txt, > tat-ca.txt 2>&1 và 2> /dev/null, kèm cat từng file để xem thứ gì được ghi vào đâu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · stdout và stderr đi hai đường</figcaption></figure>

## 4. Luồng vào: `<`

```text
an@toinoi-app-01:~/tai-lieu$ wc -l /var/log/toinoi/bao-gia.log
362 /var/log/toinoi/bao-gia.log
an@toinoi-app-01:~/tai-lieu$ wc -l < /var/log/toinoi/bao-gia.log
362
```

**`<`** nối stdin vào một file. Hai lệnh trên cho cùng con số, khác một chi tiết: lệnh đầu nhận **tên file** làm đối số nên in tên file; lệnh sau chỉ nhận **dữ liệu** qua stdin, không biết dữ liệu từ đâu tới, nên chỉ in số. Dạng sau gọn khi bạn cần đúng một con số để dùng tiếp trong script.

Quy ước chung: phần lớn lệnh xử lý văn bản (`cat`, `grep`, `sort`, `wc`, `head`…) đọc từ file nếu có đối số là tên file, và **đọc từ stdin nếu không có**. Quy ước này là nền tảng của pipe.

## 5. Pipe: nối lệnh với nhau

Dấu **`|`** (pipe) nối **stdout của lệnh bên trái** vào **stdin của lệnh bên phải**. Cả hai lệnh chạy cùng lúc, dữ liệu chảy thẳng từ lệnh này sang lệnh kia, không qua file trung gian.

```text
an@toinoi-app-01:~/tai-lieu$ grep ERROR /var/log/toinoi/bao-gia.log | wc -l
17
an@toinoi-app-01:~/tai-lieu$ grep ERROR /var/log/toinoi/bao-gia.log | head -n 3
2026-11-21 06:37:58 ERROR q-100034 dich_vu=car7 khong lay duoc quang duong: timeout sau 3000 ms goi ban-do
2026-11-21 06:41:18 ERROR q-100038 dich_vu=car4 khong lay duoc quang duong: timeout sau 3000 ms goi ban-do
2026-11-21 07:20:18 ERROR q-100076 dich_vu=bike khong lay duoc quang duong: timeout sau 3000 ms goi ban-do
an@toinoi-app-01:~/tai-lieu$ grep ERROR /var/log/toinoi/bao-gia.log | tail -n 2 | cut -d' ' -f1,2,4
2026-11-21 11:51:32 q-100336
2026-11-21 11:56:57 q-100341
```

`grep ERROR file` in các dòng có chứa `ERROR` ([bài sau](/articles/linux-tim-kiem-find-grep) nói kỹ về `grep`). Nối tiếp:

- `| wc -l`: đếm số dòng đó. **Câu trả lời đầu tiên cho Bình: 17 lỗi.**
- `| head -n 3`: ba lỗi đầu tiên. Cả ba cùng một nguyên nhân: gọi dịch vụ bản đồ quá 3000 ms.
- `| tail -n 2 | cut -d' ' -f1,2,4`: hai lỗi cuối, chỉ lấy ngày, giờ và mã yêu cầu.

**`cut`** cắt cột từ mỗi dòng. `-d' '` đặt ký tự phân cách là dấu cách, `-f1,2,4` lấy trường 1, 2 và 4. `cut -c21-25` lấy theo vị trí ký tự, từ ký tự 21 tới 25.

Một pipeline đọc từ trái sang phải như một dây chuyền: mỗi lệnh nhận dòng văn bản, làm một việc, đưa dòng văn bản cho lệnh sau. Cách xây một pipeline là **thêm từng khâu một và nhìn kết quả sau mỗi khâu**; đừng viết cả dòng dài rồi mới chạy.

## 6. Đếm theo nhóm: `sort | uniq -c`

```text
an@toinoi-app-01:~/tai-lieu$ cut -c21-25 /var/log/toinoi/bao-gia.log | sort | uniq -c
     17 ERROR
    321 INFO
     24 WARN
an@toinoi-app-01:~/tai-lieu$ grep -o 'dich_vu=[a-z0-9]*' /var/log/toinoi/bao-gia.log | sort | uniq -c | sort -rn
     99 dich_vu=car4
     94 dich_vu=car7
     84 dich_vu=delivery
     83 dich_vu=bike
an@toinoi-app-01:~/tai-lieu$ grep WARN /var/log/toinoi/bao-gia.log | sort -t: -k4 -n | tail -n 3
2026-11-21 09:56:45 WARN  q-100226 dich_vu=delivery km=9.6 bao gia cham: 2615 ms
2026-11-21 09:12:56 WARN  q-100180 dich_vu=delivery km=2.9 bao gia cham: 2637 ms
2026-11-21 11:21:33 WARN  q-100311 dich_vu=bike km=20.1 bao gia cham: 2671 ms
```

Mẫu **`… | sort | uniq -c | sort -rn`** là một trong những mẫu được dùng nhiều nhất trên dòng lệnh. Nó trả lời câu hỏi «mỗi giá trị xuất hiện bao nhiêu lần?»:

1. Một lệnh trích ra giá trị cần đếm, mỗi dòng một giá trị (`cut`, `grep -o`).
2. **`sort`** xếp các dòng, để những dòng giống nhau đứng cạnh nhau.
3. **`uniq -c`** gộp các dòng **liền nhau** giống nhau thành một và ghi số lần ở đầu. `uniq` chỉ so dòng kề nhau, nên thiếu `sort` ở trước thì kết quả sai.
4. **`sort -rn`** xếp lại theo số (`-n`), giảm dần (`-r`), để giá trị nhiều nhất lên đầu.

Lệnh thứ nhất đếm theo mức log: ký tự 21 tới 25 của mỗi dòng là mức (`INFO`, `WARN`, `ERROR`). Kết quả: 321 `INFO`, 24 `WARN`, 17 `ERROR`, cộng lại đúng 362 dòng.

Lệnh thứ hai dùng **`grep -o`**, chỉ in **phần khớp** thay vì cả dòng: mỗi dòng log cho ra một chuỗi dạng `dich_vu=car4`. Đếm chúng cho biết số yêu cầu theo loại dịch vụ.

Lệnh thứ ba tìm ba yêu cầu chậm nhất: lọc dòng `WARN`, rồi `sort -t: -k4 -n` xếp theo trường thứ 4 khi tách bằng dấu hai chấm (phần đứng sau `bao gia cham:`), theo giá trị số. Yêu cầu chậm nhất mất 2671 ms.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-chuyen-huong-pipe/ssh-studio-03.webp" alt="Terminal của SSH Studio với sáu pipeline trên log bao-gia: đếm 17 dòng ERROR, ba lỗi đầu, hai lỗi cuối cắt cột, số dòng theo mức ERROR/INFO/WARN, số yêu cầu theo dịch vụ xếp giảm dần, và ba yêu cầu WARN chậm nhất" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · pipeline lọc, cắt, sắp xếp và đếm trên <code>bao-gia.log</code></figcaption></figure>

## 7. Vừa xem vừa ghi: `tee`

Câu hỏi thứ hai của Bình: loại dịch vụ nào bị lỗi nhiều nhất?

```text
an@toinoi-app-01:~/tai-lieu$ grep ERROR /var/log/toinoi/bao-gia.log | cut -d' ' -f5 | sort | uniq -c | tee loi-theo-dich-vu.txt
      5 dich_vu=bike
      6 dich_vu=car4
      4 dich_vu=car7
      2 dich_vu=delivery
an@toinoi-app-01:~/tai-lieu$ ls -l
total 32
-rw-r--r-- 1 an an 188 Nov 22 22:27 bang-gia.csv
-rw-rw-r-- 1 an an  11 Nov 22 22:27 danh-sach.txt
-rw-rw-r-- 1 an an  87 Nov 22 22:27 ghi-chu.txt
-rw-rw-r-- 1 an an  21 Nov 22 22:27 ket-qua.txt
-rw-rw-r-- 1 an an  60 Nov 22 22:27 lich-truc.txt
-rw-rw-r-- 1 an an  88 Nov 22 22:27 loi-theo-dich-vu.txt
-rw-rw-r-- 1 an an  57 Nov 22 22:27 loi.txt
-rw-rw-r-- 1 an an  78 Nov 22 22:27 tat-ca.txt
```

Bốn khâu: lọc dòng lỗi, lấy trường thứ 5 (`dich_vu=…`), xếp, đếm. `car4` có 6 lỗi, `bike` 5, `car7` 4, `delivery` 2. Lỗi trải đều qua các loại dịch vụ, khớp với nguyên nhân chung là dịch vụ bản đồ chậm chứ không phải lỗi của riêng một bảng giá.

**`tee`** đứng ở cuối như một ống chữ T: nó chép stdin ra **cả** màn hình **lẫn** file. Bạn thấy kết quả ngay và có sẵn file để gửi cho Bình. `tee -a` ghi nối thêm.

`tee` còn giải quyết một lỗi kinh điển với `sudo`: `sudo echo "…" > /etc/file` thất bại, vì phần `> /etc/file` do shell của **bạn** (không phải `root`) thực hiện. Viết `echo "…" | sudo tee /etc/file` thì chính `tee` chạy dưới quyền `root` mở file.

Chuyển sang chế độ **Chia đôi** và trỏ khung Remote server vào `/home/an/tai-lieu`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-chuyen-huong-pipe/ssh-studio-04.webp" alt="SSH Studio ở chế độ Chia đôi: nửa trên là terminal với kết quả ls -l liệt kê tám file trong tai-lieu, nửa dưới là khung Files với khung Remote server ở /home/an/tai-lieu hiển thị các file bang-gia.csv, danh-sach.txt, ghi-chu.txt, ket-qua.txt, lich-truc.txt" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · chế độ <b>Chia đôi</b>: các file do chuyển hướng và <code>tee</code> tạo ra hiện trong khung Files</figcaption></figure>

Các file `danh-sach.txt`, `ket-qua.txt`, `loi.txt`, `tat-ca.txt`, `loi-theo-dich-vu.txt` đều do các lệnh trong bài tạo ra. Từ đây, chọn `loi-theo-dich-vu.txt` và bấm **Download** là có báo cáo trên máy mình.

## 8. Pipe chỉ mang stdout

```text
an@toinoi-app-01:~/tai-lieu$ ls /khong-co | wc -l
ls: cannot access '/khong-co': No such file or directory
0
an@toinoi-app-01:~/tai-lieu$ ls /khong-co 2>&1 | wc -l
1
```

Lệnh đầu in lỗi ra màn hình và `wc -l` đếm được **0** dòng: lỗi đi theo stderr, không vào pipe. Lệnh sau gộp stderr vào stdout trước khi vào pipe, nên `wc -l` đếm được 1 dòng, là chính dòng lỗi.

Đây là nguồn của nhiều kết quả sai âm thầm: một lệnh ở đầu pipeline thất bại, lỗi hiện lướt qua màn hình (hoặc bị `2>/dev/null` nuốt), và các khâu sau vui vẻ xử lý một đầu vào rỗng rồi báo «0». Con số 0 đó trông như một câu trả lời.

Thêm một điểm: mã thoát `$?` của một pipeline là mã thoát của **lệnh cuối cùng**. `ls /khong-co | wc -l` có `$?` bằng 0, vì `wc` thành công. Trong script, `set -o pipefail` khiến pipeline thất bại nếu bất kỳ khâu nào thất bại (Phần 2).

## 9. Nối lệnh theo mã thoát: `&&` và `||`

```text
an@toinoi-app-01:~/tai-lieu$ grep -q ERROR /var/log/toinoi/bao-gia.log && echo "co loi trong log"
co loi trong log
an@toinoi-app-01:~/tai-lieu$ grep -q FATAL /var/log/toinoi/bao-gia.log || echo "khong co dong FATAL"
khong co dong FATAL
```

| Viết | Nghĩa |
|---|---|
| `A && B` | Chạy B **chỉ khi** A thành công (mã thoát 0) |
| `A \|\| B` | Chạy B **chỉ khi** A thất bại |
| `A ; B` | Chạy A rồi B, bất kể kết quả |
| `A \| B` | Chạy cả hai, nối stdout của A vào stdin của B |

`grep -q` (*quiet*) không in gì, chỉ trả mã thoát: 0 nếu tìm thấy, 1 nếu không. Kết hợp với `&&` và `||`, nó thành một câu điều kiện một dòng.

`&&` là công cụ an toàn hằng ngày: `cd /srv/toinoi/tmp && rm -r *` chỉ xoá khi `cd` thành công. Viết bằng `;` thì nếu `cd` thất bại, `rm -r *` chạy ở thư mục bạn đang đứng.

Đừng nhầm `|` với `||`, và `&&` với `&` (một dấu `&` cho lệnh chạy nền, Phần 4).

## 10. Tóm tắt ký hiệu

| Ký hiệu | Tác dụng |
|---|---|
| `> file` | stdout ghi đè vào file |
| `>> file` | stdout ghi nối vào file |
| `2> file` | stderr vào file |
| `> file 2>&1` hoặc `&> file` | Cả hai vào file |
| `2> /dev/null` | Bỏ lỗi |
| `< file` | stdin lấy từ file |
| `A \| B` | stdout của A thành stdin của B |
| `A \| tee file` | Xem trên màn hình và ghi ra file |
| `A && B`, `A \|\| B` | Chạy B tuỳ theo A thành công hay thất bại |

## 11. Lỗi thường gặp

**`>` thay vì `>>`.** Mất nội dung cũ. Với file quan trọng, dùng `>>` hoặc bật `noclobber`.

**`lenh file > file`.** File bị xoá trắng trước khi lệnh đọc. Ghi ra file tạm.

**`uniq` không có `sort` đứng trước.** Đếm sai, vì `uniq` chỉ gộp dòng liền nhau.

**`sort` không có `-n` trên cột số.** `sort` xếp theo chữ: `10` đứng trước `9`. Thêm `-n`.

**`sudo lenh > /file-cua-root`.** Chuyển hướng do shell của bạn làm. Dùng `| sudo tee`.

**Tin con số 0 ở cuối pipeline.** Kiểm tra khâu đầu có thực sự chạy được không.

**`cat file | grep x`.** Không sai, nhưng thừa: `grep x file` làm đúng việc đó với ít hơn một tiến trình.

## Bài tập

1. Bao nhiêu yêu cầu `WARN` thuộc dịch vụ `delivery`? Viết pipeline có hai lần `grep`, rồi viết lại chỉ dùng một `grep` kèm `wc -l` hoặc `grep -c`.
2. Liệt kê số yêu cầu theo **giờ** (06, 07, …, 12), xếp theo giờ. Gợi ý: ký tự 12 và 13 của mỗi dòng là giờ.
3. Trong số 17 lỗi, giờ nào có nhiều lỗi nhất?
4. Chạy `ls /opt/toinoi /khong-co > a.txt 2> b.txt`, đoán nội dung hai file trước khi `cat`. Rồi thử `2>&1 > c.txt` (thứ tự ngược) và giải thích vì sao lỗi vẫn hiện trên màn hình.
5. Viết một dòng lệnh in `OK` nếu log **không** có dòng `ERROR` nào của dịch vụ `delivery` sau 12 giờ, và in `CO LOI` nếu có.

## Đọc tiếp

[LNX-009 · Tìm kiếm: find, grep và tìm trong terminal](/articles/linux-tim-kiem-find-grep): tìm file theo tên, kích thước, quyền; tìm trong nội dung với ngữ cảnh và biểu thức chính quy.
