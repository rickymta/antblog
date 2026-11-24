---
slug: linux-xem-sua-file-van-ban
title: "Xem và sửa file văn bản trên Linux: cat, less, tail, vim và trình sửa file của SSH Studio"
summary: "Xem file ngắn bằng cat, file dài bằng wc, head, tail và less có tìm kiếm; theo dõi log bằng tail -f. Sửa file theo hai cách: vim với ba chế độ Normal, Insert, Command-line trong terminal, và trình sửa file của SSH Studio qua SFTP. Bài dùng log 362 dòng và bảng giá của dịch vụ bao-gia, kiểm tra thay đổi bằng diff."
category: devops-linux
level: intermediate
tags: [linux, lnx-006, cat, less, tail, vim, diff, ssh-studio, sftp, toi-noi]
language: vi
---

Bình nhờ An xem vì sao sáng 21/11 có yêu cầu báo giá bị lỗi. Log của dịch vụ nằm ở `/var/log/toinoi/bao-gia.log`. An gõ `cat` với file đó, và 362 dòng chạy vụt qua màn hình. An chỉ kịp thấy vài dòng cuối.

Trên server, gần như mọi thứ là file văn bản: log, cấu hình, script, dữ liệu CSV. Đọc và sửa chúng là việc bạn làm nhiều nhất. Bài này có hai nửa: **xem** (chọn đúng lệnh theo độ dài của file) và **sửa** (bằng `vim` trong terminal, và bằng trình sửa file của SSH Studio).

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-006 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-tao-sao-chep-di-chuyen-xoa-file">LNX-005</a>. Trạng thái đầu bài: <code>npm run setup -- 06</code>. Sau bài này, bạn xem được file dài mà không ngợp, tìm được chuỗi trong <code>less</code>, sửa và lưu file bằng <code>vim</code> mà không kẹt, sửa file bằng trình sửa file của SSH Studio, và so hai file bằng <code>diff</code>. Series chỉ dùng <code>vim</code> làm trình soạn thảo trong terminal.</p>
</div>

## 1. File ngắn: `cat`

```text
an@toinoi-app-01:~$ cat -n /opt/toinoi/bao-gia/bang-gia.csv
     1  dich_vu,ten,gia_mo_cua,gia_moi_km,gia_toi_thieu
     2  bike,Xe máy,8000,2500,12000
     3  car4,Ô tô 4 chỗ,12000,4500,25000
     4  car7,Ô tô 7 chỗ,15000,5500,30000
     5  delivery,Giao hàng,10000,3000,15000
```

`cat` in toàn bộ file. `-n` đánh số dòng, tiện khi cần nói với đồng nghiệp «xem dòng 3». `cat` hợp với file vài chục dòng. Với file dài hơn một màn hình, `cat` là lựa chọn sai: bạn chỉ thấy phần cuối, và nếu file có vài trăm nghìn dòng thì terminal bận in trong nhiều giây.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-xem-sua-file-van-ban/ssh-studio-07.webp" alt="Terminal của SSH Studio: cat huong-dan.txt in lời nhắn ngắn của Bình về server toinoi-app-01 và thư mục /opt/toinoi/bao-gia" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>cat</code> hợp với file ngắn vừa một màn hình</figcaption></figure>

Đây là bảng giá của dịch vụ báo giá: mỗi dòng là một loại dịch vụ với giá mở cửa, giá mỗi km và giá tối thiểu. Phần sau của bài sẽ sửa một bản sao của file này.

## 2. File dài: đếm trước, rồi xem đầu và cuối

```text
an@toinoi-app-01:~$ wc -l /var/log/toinoi/bao-gia.log
362 /var/log/toinoi/bao-gia.log
an@toinoi-app-01:~$ wc /var/log/toinoi/bao-gia.log
  362  3431 27260 /var/log/toinoi/bao-gia.log
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-xem-sua-file-van-ban/ssh-studio-08.webp" alt="Terminal của SSH Studio: cat -n đánh số năm dòng của bang-gia.csv, wc -l đếm 362 dòng log, wc in dòng, từ, byte, head in 10 dòng đầu của bao-gia.log" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đếm trước bằng <code>wc</code>, rồi xem 10 dòng đầu bằng <code>head</code></figcaption></figure>

**`wc -l`** đếm số dòng. Chạy nó trước khi mở một file lạ là thói quen tốt: nó cho bạn biết nên dùng `cat` hay công cụ khác. Không có tuỳ chọn, `wc` in ba số: dòng, từ, byte.

```text
an@toinoi-app-01:~$ head -n 3 /var/log/toinoi/bao-gia.log
2026-11-21 06:00:00 INFO  bao-gia 1.4.2 khoi dong, cong 8080, moi truong production
2026-11-21 06:00:00 INFO  nap bang gia: 4 loai dich vu tu /opt/toinoi/bao-gia/bang-gia.csv
2026-11-21 06:01:45 INFO  q-100001 dich_vu=delivery km=23.8 gia=82000 178 ms
an@toinoi-app-01:~$ tail -n 3 /var/log/toinoi/bao-gia.log
2026-11-21 12:14:13 INFO  q-100358 dich_vu=bike km=8.8 gia=30000 26 ms
2026-11-21 12:15:08 INFO  q-100359 dich_vu=delivery km=20.3 gia=71000 109 ms
2026-11-21 12:17:05 INFO  q-100360 dich_vu=car4 km=14.3 gia=77000 59 ms
an@toinoi-app-01:~$ tail -n +361 /var/log/toinoi/bao-gia.log
2026-11-21 12:15:08 INFO  q-100359 dich_vu=delivery km=20.3 gia=71000 109 ms
2026-11-21 12:17:05 INFO  q-100360 dich_vu=car4 km=14.3 gia=77000 59 ms
```

| Lệnh | In ra |
|---|---|
| `head FILE` | 10 dòng đầu |
| `head -n 3 FILE` | 3 dòng đầu |
| `tail -n 3 FILE` | 3 dòng cuối |
| `tail -n +361 FILE` | Từ dòng 361 tới hết (để ý dấu `+`) |

Với log, **cuối file là nơi có tin mới nhất**, nên `tail` là lệnh bạn gõ nhiều nhất khi điều tra sự cố. Ba dòng đầu cho biết định dạng của log này: thời điểm, mức (`INFO`, `WARN`, `ERROR`), mã yêu cầu (`q-100001`), rồi các cặp `tên=giá trị`. Dòng đầu tiên cho biết dịch vụ là bản 1.4.2, nghe ở cổng 8080.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-xem-sua-file-van-ban/ssh-studio-01.webp" alt="Terminal của SSH Studio với head -n 3, tail -n 3 và tail -n +361 trên file bao-gia.log, mỗi dòng log gồm thời điểm, mức INFO và thông tin một yêu cầu báo giá" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · đầu và cuối của <code>bao-gia.log</code></figcaption></figure>

### Theo dõi log đang được ghi: `tail -f`

```sh
tail -f /var/log/toinoi/bao-gia.log
```

`-f` (*follow*) in các dòng cuối rồi **không thoát**: mỗi dòng mới được ghi vào file hiện ra ngay. Đây là cách xem một dịch vụ đang làm gì theo thời gian thực. Nhấn `Ctrl+C` để dừng. Log của lab là file tĩnh nên bạn sẽ không thấy dòng mới; Phần 4 dùng `tail -f` và `journalctl -f` với dịch vụ đang chạy.

Trong SSH Studio, `tail -f` hợp với chế độ **Chia đôi** hoặc với một tab phiên thứ hai tới cùng server (nút «Mở kết nối khác» cạnh tab): một bên theo dõi log, một bên gõ lệnh.

## 3. Xem từng trang và tìm kiếm: `less`

```sh
less /var/log/toinoi/bao-gia.log
```

`less` mở file ở chế độ toàn màn hình mà không đọc cả file vào bộ nhớ, nên mở file vài GB vẫn tức thì. Đây cũng là chương trình mà `man` dùng để hiển thị ([LNX-003](/articles/linux-shell-dong-lenh-tro-giup)), nên bạn đã biết phím của nó.

Trong `less`, gõ `/ERROR` rồi Enter:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-xem-sua-file-van-ban/ssh-studio-02.webp" alt="less đang mở bao-gia.log trong terminal SSH Studio sau khi tìm /ERROR: màn hình nhảy tới dòng ERROR đầu tiên lúc 06:37:58, các chữ ERROR được tô sáng giữa những dòng INFO và WARN" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>less</code> sau khi gõ <code>/ERROR</code></figcaption></figure>

`less` nhảy tới dòng đầu tiên có `ERROR` và tô sáng mọi chỗ khớp. Dấu `:` ở dòng cuối là dấu nhắc của `less`.

| Phím | Tác dụng |
|---|---|
| `Space` / `b` | Xuống / lên một trang |
| `g` / `G` | Về đầu / tới cuối file |
| `/chuỗi` | Tìm xuôi. `?chuỗi` tìm ngược |
| `n` / `N` | Kết quả kế tiếp / trước đó |
| `&chuỗi` | Chỉ hiện các dòng có `chuỗi` (Enter với chuỗi rỗng để bỏ lọc) |
| `-N` | Bật/tắt số dòng |
| `-S` | Bật/tắt cắt dòng dài thay vì xuống dòng |
| `F` | Chuyển sang theo dõi như `tail -f`; `Ctrl+C` để quay lại |
| `q` | Thoát |

Với việc Bình nhờ, quy trình tự nhiên là: `less` file log, `G` để tới cuối, `?ERROR` để tìm ngược từ lỗi mới nhất, `n` để đi tiếp về quá khứ. Lỗi đầu tiên của buổi sáng là yêu cầu `q-100034` lúc 06:37:58: dịch vụ không lấy được quãng đường vì gọi dịch vụ bản đồ quá 3000 ms. [LNX-008](/articles/linux-chuyen-huong-pipe) sẽ đếm xem có bao nhiêu lỗi như vậy.

`less` chỉ xem, không sửa được file. Đó là ưu điểm: bạn không thể lỡ tay sửa một file cấu hình khi chỉ định đọc nó.

## 4. Sửa file trong terminal: `vim`

`vim` có trên gần như mọi server Linux (hoặc bản rút gọn của nó, gọi bằng lệnh `vi`). Nó cũng là trình soạn thảo mặc định mà nhiều lệnh tự mở ra: `git commit`, `crontab -e`, `visudo`. Người mới thường sợ `vim` vì một lý do: mở ra rồi gõ chữ không vào, và không biết thoát. Cả hai đều do chưa biết `vim` có **chế độ**.

| Chế độ | Bàn phím làm gì | Vào bằng |
|---|---|---|
| **Normal** | Mỗi phím là một lệnh: di chuyển, xoá, sao chép | `Esc`. Đây là chế độ khi vừa mở `vim` |
| **Insert** | Phím gõ ra chữ, như trình soạn thảo thường | `i` từ Normal |
| **Command-line** | Gõ lệnh ở dòng cuối màn hình: lưu, thoát, tìm thay | `:` từ Normal |

Quy tắc sống còn: **không chắc mình đang ở đâu thì nhấn `Esc`**. `Esc` luôn đưa về Normal, nhấn thừa không sao.

An ghi lại việc sắp làm vào một file ghi chú:

```sh
vim tai-lieu/ghi-chu.txt
```

File chưa tồn tại nên `vim` mở một vùng trống; các dấu `~` ở lề trái đánh dấu những dòng nằm ngoài nội dung file. Nhấn `i`. Dòng cuối màn hình hiện `-- INSERT --`. Gõ hai dòng:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-xem-sua-file-van-ban/ssh-studio-04.webp" alt="vim đang mở tai-lieu/ghi-chu.txt trong terminal SSH Studio ở chế độ Insert: hai dòng Ghi chú của An và ghi chú ngày 22/11 về giá mở cửa xe máy, dòng trạng thái cuối màn hình ghi -- INSERT -- và vị trí con trỏ" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>vim</code> ở chế độ Insert (<code>-- INSERT --</code> ở dòng cuối)</figcaption></figure>

Gõ xong, nhấn `Esc` để về Normal (chữ `-- INSERT --` biến mất), rồi gõ `:wq`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-xem-sua-file-van-ban/ssh-studio-05.webp" alt="vim sau khi nhấn Esc và gõ :wq: dòng lệnh ở cuối màn hình hiển thị :wq, nội dung hai dòng ghi chú vẫn ở trên" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>vim</code> ở chế độ Command-line, sắp lưu và thoát bằng <code>:wq</code></figcaption></figure>

Nhấn Enter. `:w` là ghi (*write*), `:q` là thoát (*quit*), `:wq` là cả hai. Bạn trở lại dấu nhắc của shell.

```text
an@toinoi-app-01:~$ cat tai-lieu/ghi-chu.txt
Ghi chú của An
- 22/11: tăng giá mở cửa xe máy lên 9.000 đ ở bản nháp
```

### Những lệnh vim cần cho Phần 1

| Lệnh | Tác dụng |
|---|---|
| `i` / `a` | Vào Insert trước / sau con trỏ |
| `o` | Mở một dòng mới bên dưới và vào Insert |
| `Esc` | Về Normal |
| `:w` | Lưu |
| `:q` | Thoát (từ chối nếu có thay đổi chưa lưu) |
| `:wq` hoặc `:x` | Lưu và thoát |
| **`:q!`** | **Thoát, bỏ mọi thay đổi.** Lối thoát khi bạn lỡ sửa hỏng |
| `h` `j` `k` `l` hoặc phím mũi tên | Trái, xuống, lên, phải |
| `gg` / `G` | Về đầu / tới cuối file |
| `/chuỗi`, `n`, `N` | Tìm, như trong `less` |
| `x` / `dd` | Xoá một ký tự / cả dòng |
| `u` / `Ctrl+R` | Hoàn tác / làm lại |
| `:set number` | Hiện số dòng |

Chừng đó đủ để sửa một file cấu hình. `vim` còn rất nhiều thứ; Phần 2 có một bài riêng (LNX-012) về dùng `vim` hằng ngày.

Hai tình huống hay gặp:

- **Mở file mình không có quyền ghi.** `vim` hiện `[readonly]` ở dòng trạng thái, và `:w` báo lỗi. Thoát bằng `:q!`, rồi mở lại với quyền phù hợp ([LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file)).
- **Mất kết nối khi đang sửa.** `vim` để lại một file `.ten-file.swp`. Lần sau mở file, `vim` hỏi có khôi phục không; chọn `R` (*recover*), kiểm tra nội dung, lưu, rồi xoá file `.swp`.

Muốn chỉ xem một file bằng `vim` mà chắc chắn không sửa nhầm, mở bằng `view tên-file` (hoặc `vim -R`).

## 5. Sửa file bằng trình sửa file của SSH Studio

An muốn thử tăng giá mở cửa của xe máy từ 8.000 đ lên 9.000 đ, trên **bản sao** `~/tai-lieu/bang-gia.csv` đã chép ở bài trước. File đang chạy ở `/opt/toinoi/bao-gia` thì An không có quyền ghi, và cũng không nên sửa trực tiếp.

Chuyển sang chế độ **Files**, tới `/home/an/tai-lieu` ở khung Remote server, rồi **nhấp đúp** `bang-gia.csv` (hoặc chọn file và bấm nút «Sửa file văn bản» trên thanh công cụ). Sửa `8000` ở dòng 2 thành `9000`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-xem-sua-file-van-ban/ssh-studio-03.webp" alt="Trình sửa file của SSH Studio đang mở bang-gia.csv từ /home/an/tai-lieu: năm dòng CSV có số dòng ở lề, dòng 2 đã sửa thành bike,Xe máy,9000,2500,12000, tiêu đề có dấu chấm báo chưa lưu, dòng trạng thái ghi 6 dòng · 188 B · LF · UTF-8, các nút Tải lại, Đóng, Lưu (Ctrl+S)" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · trình sửa file với thay đổi chưa lưu ở dòng 2</figcaption></figure>

- Dấu chấm sau tên file ở tiêu đề báo có thay đổi chưa lưu.
- Dòng trạng thái ghi `LF · UTF-8`: kiểu xuống dòng và bảng mã của file. SSH Studio **giữ nguyên kiểu xuống dòng** khi lưu. Điều này quan trọng: file văn bản trên Linux xuống dòng bằng `LF`, còn Windows dùng `CRLF`, và một script bị lưu thành `CRLF` sẽ không chạy được trên Linux.
- Bấm **Lưu (Ctrl+S)**, rồi **Đóng**.

Trình sửa file làm gì ở bên dưới? Nó tải nội dung file về qua SFTP, cho bạn sửa trên máy mình, rồi ghi lại lên server khi bạn lưu, giữ nguyên quyền của file. Nếu trong lúc bạn đang sửa mà file trên server bị người khác (hoặc một chương trình) đổi, SSH Studio báo xung đột thay vì ghi đè. Trình sửa file mở được file UTF-8 tối đa 5 MB và không tô màu cú pháp.

| | `vim` | Trình sửa file của SSH Studio |
|---|---|---|
| Chạy ở đâu | Trên server, trong phiên shell | Trên máy bạn; đọc ghi qua SFTP |
| Quyền | Của shell; dùng được với `sudo` | Của tài khoản đăng nhập; **không dùng được `sudo`** |
| File lớn | Được | Tối đa 5 MB |
| Có ở mọi nơi | Có, kể cả khi bạn dùng công cụ khác | Chỉ trong SSH Studio |
| Hợp với | File hệ thống trong `/etc`, sửa khi đang ở sẵn trong terminal | File của chính bạn, sửa nhanh, chép dán từ máy mình |

Dòng «Quyền» là lý do bạn vẫn phải biết `vim`: file cấu hình hệ thống thuộc về `root`, và chỉ sửa được bằng `sudo vim …` hoặc `sudoedit …` trong terminal.

## 6. Kiểm tra thay đổi: `diff`

```text
an@toinoi-app-01:~$ cat tai-lieu/bang-gia.csv
dich_vu,ten,gia_mo_cua,gia_moi_km,gia_toi_thieu
bike,Xe máy,9000,2500,12000
car4,Ô tô 4 chỗ,12000,4500,25000
car7,Ô tô 7 chỗ,15000,5500,30000
delivery,Giao hàng,10000,3000,15000
an@toinoi-app-01:~$ diff /opt/toinoi/bao-gia/bang-gia.csv tai-lieu/bang-gia.csv
2c2
< bike,Xe máy,8000,2500,12000
---
> bike,Xe máy,9000,2500,12000
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-xem-sua-file-van-ban/ssh-studio-09.webp" alt="Terminal của SSH Studio: cat tai-lieu/bang-gia.csv sau khi sửa trong trình sửa file, diff với bản gốc in 2c2: dòng bike 8000 đổi thành 9000" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>diff</code> xác nhận đúng một dòng đã đổi</figcaption></figure>

`diff CŨ MỚI` in những chỗ khác nhau. Cách đọc:

- `2c2`: dòng 2 của file thứ nhất bị **thay** (*change*) thành dòng 2 của file thứ hai. Chữ `a` là thêm (*add*), `d` là xoá (*delete*).
- Dòng bắt đầu bằng `<` là của file thứ nhất, `>` là của file thứ hai.

`diff` không in gì nghĩa là hai file giống hệt nhau (và mã thoát là 0). `diff -u` cho định dạng «unified» có ngữ cảnh xung quanh, cùng dạng với `git diff`. So sánh trước và sau khi sửa một file cấu hình là thói quen đáng giá: nó bắt được ký tự gõ nhầm trước khi dịch vụ phải khởi động lại.

Đây cũng là lý do nên **chép một bản trước khi sửa** file quan trọng: `cp bang-gia.csv bang-gia.csv.bak`. Có bản gốc thì có `diff`, và có đường lui.

```text
an@toinoi-app-01:~$ file tai-lieu/ghi-chu.txt tai-lieu/bang-gia.csv /usr/bin/ls
tai-lieu/ghi-chu.txt:  Unicode text, UTF-8 text
tai-lieu/bang-gia.csv: CSV Unicode text, UTF-8 text
/usr/bin/ls:           ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, BuildID[sha1]=44900829b39271d878615772bb2a6ffa422f0891, for GNU/Linux 3.2.0, stripped
```

`file` xác nhận cả hai file vừa sửa là văn bản UTF-8. Nếu một file sửa trên Windows rồi tải lên có kiểu xuống dòng `CRLF`, `file` sẽ ghi thêm `with CRLF line terminators`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-xem-sua-file-van-ban/ssh-studio-06.webp" alt="Terminal của SSH Studio với cat bang-gia.csv đã sửa, kết quả diff 2c2 giữa bản gốc và bản nháp, lệnh vim, cat ghi-chu.txt và kết quả lệnh file trên ba file" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · kiểm tra hai file vừa sửa bằng <code>cat</code>, <code>diff</code> và <code>file</code></figcaption></figure>

## 7. Chọn lệnh nào?

| Bạn muốn | Dùng |
|---|---|
| Xem file vài chục dòng | `cat`, `cat -n` |
| Biết file dài bao nhiêu | `wc -l` |
| Xem phần đầu / phần cuối | `head -n N` / `tail -n N` |
| Theo dõi file đang được ghi | `tail -f`, hoặc `less` rồi `F` |
| Đọc và tìm trong file dài | `less` |
| Sửa file trong terminal, kể cả file cần `sudo` | `vim` |
| Sửa nhanh file của mình, có chép dán từ máy local | Trình sửa file của SSH Studio |
| So hai file | `diff`, `diff -u` |

## 8. Lỗi thường gặp

**`cat` một file nhị phân.** Terminal đầy ký tự rác, có khi dấu nhắc cũng hỏng. Gõ `reset` rồi Enter (dù không thấy chữ mình gõ) để đưa terminal về bình thường. Lần sau chạy `file` trước.

**Gõ chữ trong vim khi đang ở Normal.** Mỗi phím là một lệnh, nên file bị xoá, bị dán lung tung. Nhấn `Esc`, rồi `u` vài lần để hoàn tác, hoặc `:q!` để bỏ hết và mở lại.

**Không thoát được vim.** `Esc`, rồi `:q!` và Enter. Nếu dòng cuối hiện `recording @q`, bạn lỡ nhấn `q` ở Normal; nhấn `q` lần nữa để dừng.

**Nhấn `Ctrl+S` trong vim rồi terminal «đứng».** `Ctrl+S` là lệnh tạm dừng hiển thị của terminal. Nhấn `Ctrl+Q` để chạy tiếp. Trong `vim`, lưu là `:w`.

**Sửa file bằng trình sửa file trong khi `vim` cũng đang mở file đó.** Hai bên ghi đè lên nhau. Mỗi lúc chỉ sửa một file ở một nơi.

**Sửa thẳng file đang chạy.** Chép một bản, sửa bản sao, `diff`, rồi mới thay.

## Bài tập

1. `less /var/log/toinoi/bao-gia.log`, tới cuối file bằng `G`, tìm ngược `WARN` bằng `?WARN`. Yêu cầu chậm cuối cùng của buổi sáng là yêu cầu nào, mất bao nhiêu ms?
2. Trong `less`, gõ `&ERROR`. Màn hình còn bao nhiêu dòng? So với con số ở bài sau.
3. Mở `tai-lieu/ghi-chu.txt` bằng `vim`, thêm dòng thứ ba bằng `o`, lưu bằng `:w`, thêm dòng thứ tư, rồi thoát bằng `:q!`. File có mấy dòng? Vì sao?
4. Trong `vim`, mở `tai-lieu/bang-gia.csv`, bật số dòng, tìm `car7`, đổi `5500` thành `6000` (gợi ý: di chuyển tới số đó, `x` để xoá ký tự, `i` để gõ). Lưu rồi `diff` với bản gốc: kết quả có mấy khối?
5. Mở `/opt/toinoi/bao-gia/appsettings.json` bằng trình sửa file của SSH Studio và thử lưu. Chuyện gì xảy ra? Giải thích bằng cột chủ sở hữu trong `ls -l`.

## Đọc tiếp

[LNX-007 · Người dùng, nhóm và quyền file](/articles/linux-nguoi-dung-nhom-quyen-file): vì sao `an` đọc được `bang-gia.csv` mà không đọc được `bi-mat.env`.
