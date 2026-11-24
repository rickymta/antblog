---
slug: linux-vim-hang-ngay
title: "vim cho công việc hằng ngày trên server: di chuyển, sửa, tìm và thay, chạy lệnh"
summary: "Đi tiếp từ ba chế độ của vim ở Phần 1: nhân đôi và đổi một dòng bảng giá bằng yy, p, cw, thay chuỗi trên một dòng và trên cả file bằng :s và :%s, dùng dấu phân cách khác cho đường dẫn, chạy diff ngay trong vim bằng :!, hoàn tác bằng u, và một ~/.vimrc tối thiểu cho server. Kèm cách dán nhiều dòng an toàn với :set paste."
category: devops-linux
level: beginner
tags: [linux, lnx-012, vim, vimrc, ssh-studio, toi-noi]
language: vi
---

Ở [LNX-006](/articles/linux-xem-sua-file-van-ban), An biết đủ vim để không bị kẹt: `i` để gõ, `Esc` để về, `:wq` để lưu, `:q!` để bỏ. Đủ để sửa một con số. Nhưng sáng nay nhóm sản phẩm gửi yêu cầu: thêm loại xe 16 chỗ vào bảng giá, đổi tên «Ô tô» thành «Xe» cho gọn, và chuẩn bị một bản cấu hình mới trỏ tới thư mục triển khai. Làm bằng cách đưa con trỏ tới từng chữ rồi xoá từng ký tự thì vừa chậm vừa dễ sót.

Sức mạnh của vim nằm ở chế độ Normal: mỗi phím là một lệnh, và lệnh ghép được với nhau. Bài này đi qua những lệnh dùng nhiều nhất trên server, đều làm thật trên bản sao của bảng giá và file cấu hình của `bao-gia`.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-012 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-xem-sua-file-van-ban">LNX-006</a> (ba chế độ của vim) và <a href="/articles/linux-bien-moi-truong-path-bashrc">LNX-011</a>. Trạng thái đầu bài: <code>npm run setup -- 12</code>. Sau bài này, bạn nhân đôi, xoá, đổi dòng và từ bằng vài phím; thay chuỗi trên cả file; chạy lệnh shell từ trong vim; và có một <code>~/.vimrc</code> hợp với server. Series dùng vim 9.1 có sẵn trong lab.</p>
</div>

## 0. Phiên mới: `~/bin` đã ở trong PATH

```text
an@toinoi-app-01:~$ echo $PATH
/home/an/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
an@toinoi-app-01:~$ type cdbg
cdbg is aliased to `cd /srv/toinoi/bao-gia'
an@toinoi-app-01:~$ mkdir -p ~/vim
an@toinoi-app-01:~$ cp /opt/toinoi/bao-gia/bang-gia.csv /opt/toinoi/bao-gia/appsettings.json ~/vim/
an@toinoi-app-01:~$ cd ~/vim
an@toinoi-app-01:~/vim$ cat -n bang-gia.csv
     1  dich_vu,ten,gia_mo_cua,gia_moi_km,gia_toi_thieu
     2  bike,Xe máy,8000,2500,12000
     3  car4,Ô tô 4 chỗ,12000,4500,25000
     4  car7,Ô tô 7 chỗ,15000,5500,30000
     5  delivery,Giao hàng,10000,3000,15000
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-vim-hang-ngay/ssh-studio-06.webp" alt="Terminal của SSH Studio ở phiên mới: echo $PATH đã có /home/an/bin, type cdbg là alias, chép bang-gia.csv và appsettings.json vào ~/vim, cat -n bảng giá năm dòng" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · phiên mới đã có <code>~/bin</code> trong PATH; chuẩn bị file để sửa</figcaption></figure>

Đây là phiên đăng nhập mới sau bài trước. `~/.profile` thấy thư mục `~/bin` tồn tại nên tự thêm vào `PATH`, và alias `cdbg` trong `~/.bashrc` có sẵn: đúng như [LNX-011](/articles/linux-bien-moi-truong-path-bashrc) đã nói. An chép bảng giá và file cấu hình sang `~/vim` để thử.

## 1. Ngữ pháp của chế độ Normal

Hầu hết lệnh sửa trong vim có dạng **toán tử + chuyển động**:

| Toán tử | | Chuyển động | |
|---|---|---|---|
| `d` | xoá (*delete*) | `w` | tới đầu từ kế tiếp |
| `c` | xoá rồi vào Insert (*change*) | `e` | tới cuối từ |
| `y` | sao chép (*yank*) | `$` | tới cuối dòng |
| | | `0` | tới đầu dòng |

Ghép lại: `dw` xoá tới đầu từ kế, `cw` đổi một từ, `d$` xoá tới cuối dòng, `y$` chép tới cuối dòng. Gõ toán tử hai lần là tác động lên **cả dòng**: `dd` xoá dòng, `yy` chép dòng, `cc` đổi cả dòng. Thêm số ở trước để lặp: `3dd` xoá ba dòng, `5j` xuống năm dòng.

Học ngữ pháp này một lần, bạn đọc được những lệnh chưa gặp bao giờ.

## 2. Thêm loại xe 16 chỗ

Mở bảng giá và bật số dòng:

```text
vim bang-gia.csv
:set number
```

### Tìm, chép, dán

Gõ `/car7` rồi Enter: con trỏ nhảy tới đầu dòng `car7`. Gõ `yy` (chép dòng) rồi `p` (dán bên dưới dòng hiện tại):

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-vim-hang-ngay/ssh-studio-01.webp" alt="vim mở bang-gia.csv với số dòng ở lề trái; dòng car7 xuất hiện hai lần ở dòng 4 và 5 sau khi chép và dán, con trỏ nằm ở đầu dòng 5" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>/car7</code>, <code>yy</code>, <code>p</code>: dòng car7 được nhân đôi</figcaption></figure>

Bản sao nằm ở dòng 5, con trỏ đứng ở đầu dòng đó. `P` (chữ hoa) dán **phía trên**.

### Đổi một từ, thay một chuỗi trên dòng

Con trỏ đang ở chữ `c` của `car7`. Gõ `cw`: vim xoá từ `car7` và vào chế độ Insert. Gõ `van16`, rồi `Esc`. Từ dừng ở dấu phẩy, nên `cw` chỉ đổi `car7`, không đụng vào phần còn lại của dòng.

Phần còn lại của dòng cần đổi ở hai chỗ. Thay vì đi tới từng chữ, dùng lệnh **thay thế** của chế độ Command-line:

```text
:s/7 chỗ/16 chỗ/
:s/15000,5500,30000/25000,8000,45000/
```

`:s/cũ/mới/` thay lần xuất hiện đầu tiên của `cũ` **trên dòng hiện tại**. Đây chính là cú pháp của `sed` ([LNX-014](/articles/linux-sed-sua-van-ban)): học một, dùng được cả hai.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-vim-hang-ngay/ssh-studio-02.webp" alt="vim với bảng giá sáu dòng, dòng 5 đã thành van16,Ô tô 16 chỗ,25000,8000,45000 sau lệnh cw và hai lệnh :s" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · dòng 5 sau <code>cw</code> và hai lệnh <code>:s</code></figcaption></figure>

### Thay trên cả file

```text
:%s/Ô tô/Xe/g
```

`%` là phạm vi «mọi dòng». `g` ở cuối là thay **mọi** lần xuất hiện trên mỗi dòng, không chỉ lần đầu. Vim báo số chỗ đã thay ở dòng cuối màn hình:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-vim-hang-ngay/ssh-studio-03.webp" alt="vim sau lệnh :%s/Ô tô/Xe/g: ba dòng car4, car7, van16 có tên Xe 4 chỗ, Xe 7 chỗ, Xe 16 chỗ; dòng cuối màn hình ghi 3 substitutions on 3 lines" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>:%s/Ô tô/Xe/g</code>: «3 substitutions on 3 lines»</figcaption></figure>

Thêm `c` vào cuối (`:%s/Ô tô/Xe/gc`) để vim hỏi xác nhận từng chỗ (`y` đồng ý, `n` bỏ qua, `a` đồng ý hết, `q` dừng). Nên dùng khi chuỗi cần thay có thể xuất hiện ở chỗ bạn không muốn đổi.

Phạm vi viết được nhiều kiểu: `:2,4s/…/…/` (dòng 2 tới 4), `:.,$s/…/…/` (từ dòng hiện tại tới cuối).

### Hoàn tác

Thử `G` (tới dòng cuối), `dd` (xoá dòng `delivery`), rồi `u`: dòng quay lại. `u` hoàn tác từng bước, `Ctrl+R` làm lại. Vim nhớ nhiều bước hoàn tác, nên đừng ngại thử: chỉ cần chưa `:w`, bạn luôn có `:q!`.

Lưu và thoát bằng `:wq`, rồi kiểm tra bằng lệnh:

```text
an@toinoi-app-01:~/vim$ cat -n bang-gia.csv
     1  dich_vu,ten,gia_mo_cua,gia_moi_km,gia_toi_thieu
     2  bike,Xe máy,8000,2500,12000
     3  car4,Xe 4 chỗ,12000,4500,25000
     4  car7,Xe 7 chỗ,15000,5500,30000
     5  van16,Xe 16 chỗ,25000,8000,45000
     6  delivery,Giao hàng,10000,3000,15000
an@toinoi-app-01:~/vim$ diff /opt/toinoi/bao-gia/bang-gia.csv bang-gia.csv
3,4c3,5
< car4,Ô tô 4 chỗ,12000,4500,25000
< car7,Ô tô 7 chỗ,15000,5500,30000
---
> car4,Xe 4 chỗ,12000,4500,25000
> car7,Xe 7 chỗ,15000,5500,30000
> van16,Xe 16 chỗ,25000,8000,45000
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-vim-hang-ngay/ssh-studio-07.webp" alt="Terminal của SSH Studio sau khi lưu trong vim: cat -n bảng giá sáu dòng có van16 và chữ Xe thay cho Ô tô, diff với bản gốc in khối 3,4c3,5" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>diff</code> kiểm lại mọi thay đổi vừa làm trong vim</figcaption></figure>

`diff` xác nhận đúng ba thay đổi: hai dòng đổi tên, một dòng mới.

## 3. Sửa cấu hình: đường dẫn, tìm rồi đổi, chạy lệnh từ vim

```text
vim appsettings.json
:%s#/opt/toinoi/bao-gia#/srv/toinoi/bao-gia/current#
/Information
cw  Warning  Esc
/8080
cw  8081  Esc
:w
:!diff /opt/toinoi/bao-gia/appsettings.json %
```

Ba điểm mới:

- **Dấu phân cách khác.** Đường dẫn chứa dấu `/`; viết `:%s/\/opt\/toinoi…` rất khó đọc. Lệnh `s` nhận ký tự bất kỳ đứng ngay sau nó làm dấu phân cách, ở đây là `#`. `sed` cũng vậy.
- **Tìm rồi đổi.** `/Information` đưa con trỏ tới đúng chữ cần sửa, `cw` đổi nó. Cặp «tìm, rồi toán tử» nhanh hơn mọi cách di chuyển bằng phím mũi tên. `n` tới kết quả kế tiếp nếu chữ đó xuất hiện nhiều lần.
- **`:!lệnh` chạy một lệnh shell** mà không rời vim. Trong lệnh, `%` được thay bằng tên file đang mở. Vim tạm nhường màn hình cho kết quả, rồi chờ bạn nhấn Enter để quay lại:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-vim-hang-ngay/ssh-studio-04.webp" alt="Kết quả lệnh :!diff chạy từ trong vim: ba khối thay đổi của appsettings.json gồm Port 8080 thành 8081, RuleFile chuyển sang /srv/toinoi/bao-gia/current và Level Information thành Warning; dòng cuối Press ENTER or type command to continue" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>:!diff … %</code> chạy ngay trong vim</figcaption></figure>

Lưu (`:w`) **trước** khi `:!diff`, vì lệnh ngoài đọc file trên đĩa, không đọc nội dung đang sửa trong vim. Kiểm tra lại sau khi thoát:

```text
an@toinoi-app-01:~/vim$ diff /opt/toinoi/bao-gia/appsettings.json appsettings.json
4c4
<     "Port": 8080,
---
>     "Port": 8081,
8c8
<     "RuleFile": "/opt/toinoi/bao-gia/bang-gia.csv",
---
>     "RuleFile": "/srv/toinoi/bao-gia/current/bang-gia.csv",
13c13
<     "Level": "Information"
---
>     "Level": "Warning"
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-vim-hang-ngay/ssh-studio-08.webp" alt="Terminal của SSH Studio sau khi thoát vim: diff appsettings.json với bản gốc in ba khối 4c4 Port, 8c8 RuleFile, 13c13 Level" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · ba chỗ đã đổi trong <code>appsettings.json</code></figcaption></figure>

## 4. Những lệnh nên thuộc

| Nhóm | Lệnh | Tác dụng |
|---|---|---|
| Di chuyển | `w` `b` `e` | Từ kế / từ trước / cuối từ |
| | `0` `^` `$` | Đầu dòng / ký tự đầu không trống / cuối dòng |
| | `gg` `G` `:42` | Đầu file / cuối file / dòng 42 |
| | `%` | Tới dấu ngoặc tương ứng: hữu ích với JSON |
| Sửa | `x` `dd` `D` | Xoá ký tự / dòng / tới cuối dòng |
| | `cw` `cc` `C` | Đổi từ / dòng / tới cuối dòng |
| | `r` | Thay đúng một ký tự (`r9` đổi ký tự dưới con trỏ thành `9`) |
| | `o` `O` `A` | Dòng mới dưới / trên, Insert ở cuối dòng |
| | `J` | Nối dòng dưới vào dòng hiện tại |
| | `.` | Lặp lại lần sửa vừa rồi |
| Chép dán | `yy` `p` `P` | Chép dòng, dán dưới / trên |
| Tìm thay | `/x` `?x` `n` `N` `*` | Tìm xuôi / ngược, kế / trước, tìm từ dưới con trỏ |
| | `:s/a/b/` `:%s/a/b/g` | Thay trên dòng / cả file |
| Chọn | `v` `V` | Chọn theo ký tự / theo dòng; rồi `d`, `y`, `>` |
| File | `:w` `:q` `:wq` `:q!` | Lưu / thoát / lưu và thoát / bỏ thay đổi |
| | `:e file` | Mở file khác |
| Khác | `u` `Ctrl+R` | Hoàn tác / làm lại |
| | `:!lệnh` | Chạy lệnh shell |

Lệnh `.` đáng nhắc riêng: sửa một chỗ bằng `cw…Esc`, rồi `n` tới chỗ kế, `.` lặp lại đúng lần sửa đó. «Tìm, sửa, `n .`, `n .`» là cách sửa có chọn lọc nhanh nhất mà không cần viết biểu thức thay thế.

## 5. `~/.vimrc` tối thiểu cho server

Mỗi lần mở vim lại gõ `:set number` thì mệt. Cấu hình lâu dài nằm trong file `~/.vimrc`:

```text
an@toinoi-app-01:~/vim$ vim ~/.vimrc
an@toinoi-app-01:~/vim$ cat ~/.vimrc
" Cấu hình vim tối thiểu cho server (series Linux qua SSH Studio, LNX-012)
" Có ~/.vimrc thì vim không tự nạp defaults.vim nữa, nên nạp lại trước.
unlet! skip_defaults_vim
source $VIMRUNTIME/defaults.vim

set number            " hiện số dòng
set hlsearch          " tô sáng mọi kết quả tìm
set ignorecase        " tìm không phân biệt hoa thường…
set smartcase         " …trừ khi chuỗi tìm có chữ hoa
set expandtab         " Tab chèn dấu cách
set shiftwidth=2 tabstop=2
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-vim-hang-ngay/ssh-studio-09.webp" alt="Terminal của SSH Studio: cat ~/.vimrc vừa viết, gồm unlet skip_defaults_vim, source defaults.vim, set number, hlsearch, ignorecase, smartcase, expandtab, shiftwidth=2 tabstop=2, mỗi dòng có chú thích" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>~/.vimrc</code> tối thiểu cho server</figcaption></figure>

- **Hai dòng đầu là bắt buộc.** Khi không có `~/.vimrc`, vim của Ubuntu tự nạp `defaults.vim`: tô màu cú pháp, nhận loại file, hiện vị trí con trỏ. **Có `~/.vimrc` thì vim bỏ qua `defaults.vim`**, và nhiều người thấy vim «tự nhiên xấu đi» sau khi tạo file cấu hình. Hai dòng này nạp lại nó trước.
- `set number`, `hlsearch`: số dòng và tô sáng mọi kết quả tìm (bỏ tô sáng tạm thời bằng `:noh`).
- `ignorecase` + `smartcase`: tìm `error` khớp cả `ERROR`, nhưng tìm `ERROR` chỉ khớp chữ hoa.
- `expandtab`, `shiftwidth=2`: Tab chèn hai dấu cách. Script shell và YAML trên server hầu như luôn thụt lề bằng dấu cách.

Mở thử file log và tìm một mã yêu cầu:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-vim-hang-ngay/ssh-studio-05.webp" alt="vim mở /var/log/toinoi/bao-gia.log với số dòng ở lề trái nhờ ~/.vimrc, mã q-100057 ở dòng 59 được tô sáng sau khi tìm /q-100057" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>~/.vimrc</code> có hiệu lực: số dòng và tô sáng kết quả tìm</figcaption></figure>

Log thuộc về `baogia`, `an` chỉ đọc được, nên vim mở ở chế độ chỉ đọc (`[readonly]`). Muốn xem mà chắc chắn không sửa nhầm, mở bằng `view tên-file`.

Giữ `~/.vimrc` ngắn. Trên server, bạn thường phải dùng vim của người khác (`root`, tài khoản dịch vụ, một máy mới dựng), nên thói quen dựa vào vim gần như mặc định có lợi hơn một cấu hình dài với hàng chục plugin.

## 6. Dán nhiều dòng vào vim

Khi dán một đoạn script vào vim đang ở chế độ Insert, bạn có thể thấy các dòng thụt lề dần như bậc thang, hoặc mỗi dòng sau một dòng chú thích tự có thêm `#` ở đầu. Vim đang coi nội dung dán như bạn gõ từng phím, nên áp dụng tự thụt lề và tự nối chú thích.

- **`:set paste`** trước khi vào Insert để tắt các tính năng đó, dán xong thì `:set nopaste`. Các ảnh về script trong Phần 2 đều được làm theo cách này, nên bạn sẽ thấy dòng `:set nopaste` ở cuối màn hình.
- Vim 9 và terminal của SSH Studio hỗ trợ *bracketed paste*: dán bằng `Ctrl+Shift+V` (hoặc chuột phải) thì vim thường tự nhận ra đây là nội dung dán. `:set paste` là cách chắc chắn khi không rõ terminal có hỗ trợ hay không.
- Với file dài, cách khác là tải file lên bằng khung Files ([LNX-005](/articles/linux-tao-sao-chep-di-chuyen-xoa-file)).

## 7. Lỗi thường gặp

**Gõ lệnh khi đang ở Insert.** `:wq` xuất hiện ngay trong file. Nhấn `Esc`, `u` để hoàn tác, rồi gõ lại.

**`E37: No write since last change`.** Bạn `:q` khi còn thay đổi chưa lưu. `:wq` để lưu, `:q!` để bỏ.

**`E45: 'readonly' option is set`.** File không thuộc quyền ghi của bạn. Thoát bằng `:q!`; nếu thực sự cần sửa, mở lại bằng `sudoedit` hoặc `sudo vim` ([LNX-024](/articles/linux-sudo-sudoers) ở Phần 3).

**`:%s` thay nhầm chỗ.** `u` hoàn tác cả lệnh thay thế một lần. Lần sau thêm `c` để xác nhận từng chỗ.

**Dấu `/` trong mẫu thay thế.** Dùng dấu phân cách khác: `:%s#a/b#c/d#`.

**Vim mất màu sau khi tạo `~/.vimrc`.** Thiếu hai dòng nạp `defaults.vim`.

## Bài tập

1. Trong `~/vim/bang-gia.csv`, dùng `*` khi con trỏ đứng trên chữ `Xe` để tìm mọi chỗ có từ đó. Có bao nhiêu kết quả?
2. Đổi giá tối thiểu của cả ba loại xe ô tô thêm `000` ở cuối chỉ bằng một lệnh `:%s` có phạm vi dòng. Rồi `u` để hoàn tác.
3. Mở `~/vim/appsettings.json`, đặt con trỏ trên dấu `{` của khối `"Fare"`, gõ `%`. Con trỏ nhảy tới đâu? Thử `d%` rồi `u`.
4. Xoá dòng `bike` bằng `dd`, đi xuống dòng cuối bằng `G`, dán bằng `p`. Bạn vừa làm thao tác gì với dòng đó?
5. Thêm `set cursorline` vào `~/.vimrc` bằng `o` ở cuối file. Mở lại vim: khác gì?

## Đọc tiếp

[LNX-013 · Trích cột và dòng: cut, sort, uniq, tr, wc](/articles/linux-cut-sort-uniq-tr): quay lại dòng lệnh, đọc log của cả ba ngày.
