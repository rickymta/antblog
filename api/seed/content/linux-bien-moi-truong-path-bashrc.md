---
slug: linux-bien-moi-truong-path-bashrc
title: "Biến, biến môi trường và PATH trên Linux: vì sao lệnh «không tìm thấy» và ~/.bashrc để làm gì"
summary: "Mở đầu Phần 2 của series Linux qua SSH Studio: biến của shell và cách đặt dấu nháy, lưu kết quả lệnh vào biến, biến môi trường và export, biến tạm cho một lệnh, PATH quyết định lệnh nào được tìm thấy, thêm ~/bin vào PATH, và hai file khởi động ~/.profile, ~/.bashrc với alias riêng cho dịch vụ bao-gia."
category: devops-linux
level: beginner
tags: [linux, lnx-011, bash, bien-moi-truong, path, bashrc, alias, ssh-studio, toi-noi]
language: vi
---

Phần 1 kết thúc với một thư mục triển khai gọn gàng và một An đã quen gõ lệnh. Nhưng nhìn lại lịch sử lệnh của mình, An thấy cùng một đường dẫn `/var/log/toinoi/bao-gia.log` được gõ đi gõ lại cả chục lần, cùng một pipeline đếm lỗi được chép từ bài này sang bài khác. Bình gợi ý: «Lệnh nào gõ lần thứ ba thì nên có tên.»

Phần 2 biến những dòng lệnh gõ tay thành công cụ: biến, alias, rồi script có tham số, điều kiện, vòng lặp, chạy theo lịch. Bài đầu tiên lo phần nền: shell nhớ giá trị bằng cách nào, vì sao có lệnh gõ được và lệnh báo `command not found`, và đặt cấu hình riêng của bạn ở đâu để mỗi lần đăng nhập đều có.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-011 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: Phần 1, đặc biệt <a href="/articles/linux-shell-dong-lenh-tro-giup">LNX-003</a> và <a href="/articles/linux-chuyen-huong-pipe">LNX-008</a>. Trạng thái đầu bài: <code>npm run setup -- 11</code> trong <code>examples/toi-noi-linux</code>, rồi kết nối lại bằng SSH Studio. Sau bài này, bạn đặt và dùng được biến, biết khi nào cần <code>export</code>, đọc được <code>PATH</code>, có thư mục <code>~/bin</code> cho script của riêng mình và vài alias cho dịch vụ <code>bao-gia</code>.</p>
</div>

## 0. Log của ba ngày

Đầu Phần 2, Bình bật xoay vòng log cho dịch vụ báo giá và chép thêm log của hai ngày trước từ server cũ sang:

```text
an@toinoi-app-01:~$ ls -l /var/log/toinoi
total 60
-rw-r--r-- 1 baogia toinoi 27260 Nov 21 12:17 bao-gia.log
-rw-r--r-- 1 baogia toinoi 23353 Nov 20 11:16 bao-gia.log.1
-rw-r--r-- 1 baogia toinoi  4314 Nov 19 11:07 bao-gia.log.2.gz
```

Đây là cách đặt tên quen thuộc của công cụ xoay vòng log (logrotate, Phần 4): `bao-gia.log` là file đang được ghi (ngày 21/11), `bao-gia.log.1` là file của ngày trước đó (20/11), `bao-gia.log.2.gz` là file cũ hơn nữa (19/11) đã được nén. Trên terminal, `ls` tô đỏ file nén. Cả Phần 2 làm việc với ba file này. (`npm run setup -- 11` đã làm bước chép log cho bạn.)

## 1. Biến của shell

```text
an@toinoi-app-01:~$ dich_vu=bao-gia
an@toinoi-app-01:~$ echo $dich_vu
bao-gia
an@toinoi-app-01:~$ echo "Dịch vụ: $dich_vu"
Dịch vụ: bao-gia
an@toinoi-app-01:~$ echo 'Dịch vụ: $dich_vu'
Dịch vụ: $dich_vu
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bien-moi-truong-path-bashrc/ssh-studio-05.webp" alt="Terminal của SSH Studio: ls -l /var/log/toinoi với ba file log, dich_vu=bao-gia, echo $dich_vu, nháy kép mở rộng biến còn nháy đơn in nguyên chữ $dich_vu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · gán biến; nháy kép và nháy đơn khác nhau</figcaption></figure>

- **Gán**: `tên=giá trị`, **không có dấu cách** quanh dấu `=`. Tên biến gồm chữ, số và dấu gạch dưới, không bắt đầu bằng số.
- **Dùng**: đặt `$` trước tên. Shell thay `$dich_vu` bằng giá trị trước khi chạy lệnh, như cách nó mở rộng wildcard ([LNX-005](/articles/linux-tao-sao-chep-di-chuyen-xoa-file)).
- **Dấu nháy kép** `"…"` vẫn cho shell thay biến bên trong. **Dấu nháy đơn** `'…'` giữ nguyên mọi ký tự, kể cả `$`.

Quy tắc thực hành: **luôn đặt biến trong nháy kép** (`"$dich_vu"`), trừ khi bạn có lý do không làm vậy. [LNX-019](/articles/linux-shell-script-an-toan) chỉ ra chuyện gì xảy ra khi giá trị có dấu cách mà biến không được đặt trong nháy.

### Tên biến dính vào chữ phía sau

```text
an@toinoi-app-01:~$ phien_ban=1.4.2
an@toinoi-app-01:~$ echo "$dich_vu-$phien_ban.tar.gz"
bao-gia-1.4.2.tar.gz
an@toinoi-app-01:~$ echo "$dich_vu_$phien_ban.tar.gz"
1.4.2.tar.gz
an@toinoi-app-01:~$ echo "${dich_vu}_${phien_ban}.tar.gz"
bao-gia_1.4.2.tar.gz
an@toinoi-app-01:~$ dich_vu = bao-gia
-bash: dich_vu: command not found
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bien-moi-truong-path-bashrc/ssh-studio-06.webp" alt="Terminal của SSH Studio: phien_ban=1.4.2, $dich_vu-$phien_ban đúng, $dich_vu_$phien_ban mất phần đầu, ${dich_vu}_${phien_ban} đúng, dich_vu = bao-gia báo command not found" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>${…}</code> khi tên biến dính chữ; không được có dấu cách quanh <code>=</code></figcaption></figure>

Dòng thứ ba in thiếu `bao-gia`: shell đọc `$dich_vu_` là tên biến `dich_vu_`, vì dấu gạch dưới hợp lệ trong tên. Biến đó không tồn tại, nên được thay bằng chuỗi rỗng, **không báo lỗi gì**. Dấu `-` ở dòng trên không thuộc tên biến nên không gặp chuyện này. Cách chắc chắn: viết `${dich_vu}` để chỉ rõ tên kết thúc ở đâu.

Dòng cuối là lỗi hay gặp nhất của người mới: có dấu cách quanh `=`, shell hiểu `dich_vu` là **tên lệnh**, còn `=` và `bao-gia` là hai đối số.

### Lưu kết quả của một lệnh vào biến

```text
an@toinoi-app-01:~$ so_loi=$(grep -c ERROR /var/log/toinoi/bao-gia.log)
an@toinoi-app-01:~$ ngay=$(head -n 1 /var/log/toinoi/bao-gia.log | cut -d' ' -f1)
an@toinoi-app-01:~$ echo "Ngày $ngay có $so_loi lỗi"
Ngày 2026-11-21 có 17 lỗi
```

`$(lệnh)` là **command substitution**: shell chạy lệnh bên trong, lấy stdout của nó (bỏ dấu xuống dòng ở cuối) và đặt vào chỗ đó. Kết hợp với biến, bạn đặt tên được cho một con số hay một chuỗi lấy từ log, rồi dùng lại nhiều lần. Bạn sẽ còn thấy cách viết cũ hơn bằng dấu backtick `` `lệnh` ``; `$(…)` dễ đọc hơn và lồng được vào nhau.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bien-moi-truong-path-bashrc/ssh-studio-01.webp" alt="Terminal của SSH Studio: so_loi=$(grep -c ERROR …), ngay=$(head -n 1 … | cut -d' ' -f1), echo in Ngày 2026-11-21 có 17 lỗi" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · lưu kết quả của một lệnh vào biến bằng <code>$(…)</code></figcaption></figure>

## 2. Biến môi trường

```text
an@toinoi-app-01:~$ printenv HOME USER SHELL LANG
/home/an
an
/bin/bash
C.UTF-8
an@toinoi-app-01:~$ bash -c 'echo "Shell con thấy: [$dich_vu]"'
Shell con thấy: []
an@toinoi-app-01:~$ export dich_vu
an@toinoi-app-01:~$ bash -c 'echo "Shell con thấy: [$dich_vu]"'
Shell con thấy: [bao-gia]
```

Biến bạn vừa gán chỉ sống trong **shell hiện tại**. Mỗi lệnh bạn chạy là một tiến trình con, và tiến trình con chỉ nhận được những biến đã được đánh dấu là **biến môi trường** (environment variable).

- `printenv` in các biến môi trường mà shell đang có. `HOME`, `USER`, `SHELL`, `LANG` do quá trình đăng nhập đặt sẵn.
- `bash -c '…'` chạy một shell con. Lần đầu, shell con không thấy `dich_vu`.
- **`export dich_vu`** biến nó thành biến môi trường; từ đó mọi tiến trình con đều nhận được một **bản sao**.

«Bản sao» là chữ quan trọng: tiến trình con sửa biến của nó không làm thay đổi biến của shell cha. Đây là lý do một script không đổi được thư mục hiện tại hay biến của shell đã chạy nó.

Một vài biến môi trường bạn sẽ gặp thường xuyên:

| Biến | Ý nghĩa |
|---|---|
| `HOME` | Thư mục riêng; `~` được thay bằng giá trị này |
| `USER`, `LOGNAME` | Tên người dùng |
| `PATH` | Danh sách thư mục để tìm lệnh (mục 3) |
| `LANG` | Ngôn ngữ và bảng mã. `C.UTF-8` là bảng mã UTF-8 không kèm bản dịch: thông báo lỗi bằng tiếng Anh, chữ có dấu hiển thị đúng |
| `TZ` | Múi giờ, nếu muốn khác múi giờ của hệ thống |
| `EDITOR` | Trình soạn thảo mà các lệnh như `crontab -e`, `git commit` mở ra |

### Biến cho đúng một lệnh

```text
an@toinoi-app-01:~$ date -d '2026-11-21 06:37:58' '+%F %T %Z'
2026-11-21 06:37:58 +07
an@toinoi-app-01:~$ TZ=UTC date -d '2026-11-21 06:37:58 +07' '+%F %T %Z'
2026-11-20 23:37:58 UTC
an@toinoi-app-01:~$ echo $TZ
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bien-moi-truong-path-bashrc/ssh-studio-07.webp" alt="Terminal của SSH Studio: printenv HOME USER SHELL LANG, shell con không thấy dich_vu cho tới khi export, date theo múi giờ +07 và TZ=UTC date chỉ cho riêng lệnh đó, echo $TZ rỗng" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · biến môi trường: <code>export</code> và biến cho riêng một lệnh</figcaption></figure>

Viết `TÊN=giá_trị lệnh` trên cùng một dòng thì biến chỉ có trong môi trường của **lệnh đó**. Ở đây `TZ=UTC` khiến `date` đổi 06:37:58 giờ Việt Nam (lỗi đầu tiên của ngày 21/11) sang giờ UTC, còn shell của bạn không bị ảnh hưởng: `echo $TZ` in một dòng trống. Cách viết này rất hay dùng để chạy thử một lệnh với cấu hình khác, ví dụ `LANG=C sort` để sắp xếp theo byte.

## 3. PATH: vì sao lệnh được tìm thấy

```text
an@toinoi-app-01:~$ echo $PATH
/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
an@toinoi-app-01:~$ type -a ls
ls is aliased to `ls --color=auto'
ls is /usr/bin/ls
ls is /bin/ls
```

Khi bạn gõ một tên lệnh không có dấu `/`, bash lần lượt tìm trong từng thư mục của `PATH`, từ trái sang phải, và chạy file thực thi đầu tiên có tên đó. `type -a` liệt kê mọi kết quả: alias `ls` trước, rồi `/usr/bin/ls`, rồi `/bin/ls`. Hai kết quả cuối là cùng một file, vì `/bin` là symlink tới `usr/bin` ([LNX-004](/articles/linux-cay-thu-muc-duong-dan)).

`command not found` nghĩa là **không thư mục nào trong `PATH` có file thực thi tên đó**. Nó không có nghĩa là chương trình không có trên máy. Thử ngay:

```text
an@toinoi-app-01:~$ mkdir -p ~/bin
an@toinoi-app-01:~$ vim bin/xin-chao
an@toinoi-app-01:~$ chmod u+x bin/xin-chao
an@toinoi-app-01:~$ xin-chao
-bash: xin-chao: command not found
an@toinoi-app-01:~$ ./bin/xin-chao
Xin chào an, đây là máy toinoi-app-01.
an@toinoi-app-01:~$ export PATH="$HOME/bin:$PATH"
an@toinoi-app-01:~$ echo $PATH
/home/an/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
an@toinoi-app-01:~$ xin-chao
Xin chào an, đây là máy toinoi-app-01.
an@toinoi-app-01:~$ which xin-chao
/home/an/bin/xin-chao
```

An tạo thư mục `~/bin` cho các script của riêng mình và viết một script hai dòng bằng vim (ảnh bên dưới; cấu trúc của script là nội dung của [LNX-016](/articles/linux-shell-script-dau-tien)). Đi theo từng bước:

1. `xin-chao` → `command not found`: `~/bin` chưa nằm trong `PATH`.
2. `./bin/xin-chao` chạy được: có dấu `/` thì bash dùng đúng đường dẫn đó, không tìm trong `PATH` ([LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file)).
3. `export PATH="$HOME/bin:$PATH"`: thêm `~/bin` vào **đầu** `PATH`, giữ nguyên phần còn lại. Đặt ở đầu nghĩa là script của bạn thắng nếu trùng tên với lệnh hệ thống; đặt ở cuối (`"$PATH:$HOME/bin"`) thì ngược lại.
4. Từ đó `xin-chao` chạy được ở mọi thư mục, và `which` cho biết nó nằm ở đâu.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bien-moi-truong-path-bashrc/ssh-studio-02.webp" alt="Terminal của SSH Studio: echo $PATH, type -a ls, tạo ~/bin/xin-chao bằng vim, xin-chao báo command not found, ./bin/xin-chao chạy được, export PATH thêm $HOME/bin, xin-chao chạy được, which xin-chao" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>PATH</code> quyết định lệnh được tìm ở đâu</figcaption></figure>

Hai điều không nên làm với `PATH`:

- **`PATH=$HOME/bin`** (quên `:$PATH`): từ đó shell không tìm thấy cả `ls`. Nếu lỡ tay, gõ `PATH=/usr/bin:/bin` để cứu phiên hiện tại, hoặc đăng nhập lại.
- **Thêm `.` (thư mục hiện tại) vào `PATH`.** Lý do bảo mật đã nói ở LNX-007.

## 4. File khởi động: `~/.profile` và `~/.bashrc`

`export PATH=…` gõ tay chỉ có tác dụng trong phiên hiện tại. Đăng nhập lại là mất. Thứ bạn muốn có ở mọi phiên phải được ghi vào file mà bash đọc lúc khởi động. Trên Ubuntu có hai file chính trong thư mục riêng:

| File | Bash đọc khi nào | Đặt gì vào đây |
|---|---|---|
| `~/.profile` | Một lần, khi **đăng nhập** (phiên SSH mới) | Biến môi trường: `PATH`, `EDITOR`, `LANG` |
| `~/.bashrc` | Mỗi khi mở một **shell tương tác** | Alias, hàm, dấu nhắc, tuỳ chọn của bash |

Trên Ubuntu, `~/.profile` cũng gọi `~/.bashrc`, nên phiên SSH nào cũng có cả hai. Và `~/.profile` đã có sẵn đoạn lo việc `~/bin`:

```text
an@toinoi-app-01:~$ grep -n -A2 'HOME/bin' ~/.profile
20:if [ -d "$HOME/bin" ] ; then
21:    PATH="$HOME/bin:$PATH"
22-fi
23-
an@toinoi-app-01:~$ grep -n '^alias l' ~/.bashrc
91:alias ll='ls -alF'
92:alias la='ls -A'
93:alias l='ls -CF'
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bien-moi-truong-path-bashrc/ssh-studio-08.webp" alt="Terminal của SSH Studio: grep -n -A2 HOME/bin trong ~/.profile thấy đoạn thêm ~/bin vào PATH, grep -n ^alias l trong ~/.bashrc thấy ba alias ll, la, l" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>~/.profile</code> thêm <code>~/bin</code> vào PATH, <code>~/.bashrc</code> giữ alias</figcaption></figure>

Đoạn `if [ -d "$HOME/bin" ]` thêm `~/bin` vào `PATH` **nếu thư mục đó tồn tại lúc đăng nhập**. Lúc An đăng nhập, `~/bin` chưa có, nên phải `export` tay. Từ phiên sau thì không cần nữa; bài sau mở đầu bằng việc kiểm tra điều đó.

`~/.bashrc` của Ubuntu có sẵn vài alias: `ll` là `ls -alF`, `la` là `ls -A`. An thêm hai alias cho việc hằng ngày. Mở file bằng vim, nhấn `G` để tới cuối file, `o` để mở dòng mới bên dưới và vào chế độ Insert, rồi gõ:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bien-moi-truong-path-bashrc/ssh-studio-03.webp" alt="vim đang mở ~/.bashrc ở cuối file: các dòng cấu hình bash_completion có sẵn và ba dòng mới thêm, một dòng chú thích Lối tắt cho dịch vụ bao-gia và hai alias cdbg, loibg; dòng lệnh cuối màn hình hiển thị :set nopaste" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · thêm hai alias vào cuối <code>~/.bashrc</code> bằng vim</figcaption></figure>

Dòng `:set nopaste` ở cuối màn hình là dấu vết của một mẹo: trước khi dán hoặc gõ nhiều dòng vào vim, gõ `:set paste` để vim không tự thụt lề hay tự thêm dấu `#` ở dòng mới; gõ xong thì `:set nopaste`. [LNX-012](/articles/linux-vim-hang-ngay) nói thêm về chuyện này. Lưu bằng `:wq`.

```text
an@toinoi-app-01:~$ tail -n 3 ~/.bashrc
alias cdbg='cd /srv/toinoi/bao-gia'
alias loibg='grep -c ERROR /var/log/toinoi/bao-gia.log'
an@toinoi-app-01:~$ loibg
-bash: loibg: command not found
an@toinoi-app-01:~$ source ~/.bashrc
an@toinoi-app-01:~$ loibg
17
an@toinoi-app-01:~$ cdbg
an@toinoi-app-01:/srv/toinoi/bao-gia$ pwd
/srv/toinoi/bao-gia
an@toinoi-app-01:/srv/toinoi/bao-gia$ type cdbg loibg
cdbg is aliased to `cd /srv/toinoi/bao-gia'
loibg is aliased to `grep -c ERROR /var/log/toinoi/bao-gia.log'
```

Sửa `~/.bashrc` xong, shell đang chạy **chưa** biết gì về thay đổi đó (`loibg: command not found`), vì nó đã đọc file từ lúc khởi động. **`source ~/.bashrc`** (viết tắt là `. ~/.bashrc`) đọc lại file và chạy từng dòng **trong shell hiện tại**. Khác với chạy `bash ~/.bashrc`: lệnh đó chạy file trong một shell con, alias sinh ra rồi mất theo shell con.

Sau đó `loibg` in số lỗi của hôm nay, `cdbg` đưa bạn tới thư mục triển khai, và `type` xác nhận cả hai là alias.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bien-moi-truong-path-bashrc/ssh-studio-04.webp" alt="Terminal của SSH Studio: grep đoạn HOME/bin trong ~/.profile, các alias có sẵn trong ~/.bashrc, tail ba dòng mới thêm, loibg báo command not found, source ~/.bashrc, loibg in 17, cdbg chuyển tới /srv/toinoi/bao-gia và type xác nhận hai alias" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>source ~/.bashrc</code> nạp alias mới vào shell đang chạy</figcaption></figure>

### Alias hay script?

Alias chỉ là phép thay chữ ở đầu dòng lệnh. Nó hợp với lối tắt ngắn (`cdbg`, `ll`). Khi bạn cần tham số, điều kiện hay nhiều bước, hãy viết script trong `~/bin`. Một lý do nữa: alias chỉ có trong shell tương tác, nên script và cron không thấy chúng. Từ [LNX-016](/articles/linux-shell-script-dau-tien) trở đi, các công cụ của An đều là script.

## 5. Đối chiếu nhanh

| Bạn muốn | Viết |
|---|---|
| Gán biến | `ten=gia_tri` (không dấu cách) |
| Dùng biến an toàn | `"$ten"`, hoặc `"${ten}_them"` khi có chữ đứng sát sau |
| Lưu kết quả lệnh | `ten=$(lenh)` |
| Cho tiến trình con thấy biến | `export ten` |
| Biến cho đúng một lệnh | `TEN=gia_tri lenh` |
| Thêm thư mục vào PATH | `export PATH="$HOME/bin:$PATH"` |
| Giữ cho mọi phiên | Biến môi trường vào `~/.profile`; alias vào `~/.bashrc` |
| Áp dụng ngay `~/.bashrc` vừa sửa | `source ~/.bashrc` |

## 6. Lỗi thường gặp

**Dấu cách quanh `=`.** `ten = gia_tri` là chạy lệnh tên `ten`. Bash báo `command not found`.

**Tên biến dính chữ phía sau.** `"$ngay_log"` khi bạn muốn `"${ngay}_log"`. Không có lỗi nào được báo; chỉ có kết quả sai.

**Biến trong nháy đơn.** `echo '$HOME'` in đúng chữ `$HOME`.

**Ghi đè PATH.** Luôn giữ `:$PATH` khi thêm thư mục.

**Sửa `~/.bashrc` rồi chờ thấy thay đổi.** `source ~/.bashrc`, hoặc mở phiên mới.

**Đặt `export PATH=…` vào `~/.bashrc`.** Chạy được, nhưng mỗi shell lồng nhau lại thêm `~/bin` một lần nữa, và `PATH` dài dần. Biến môi trường thuộc về `~/.profile`.

## Bài tập

1. Gán `file_log=/var/log/toinoi/bao-gia.log`, rồi dùng biến đó để in ba dòng cuối của log và số dòng `WARN`.
2. Giải thích vì sao `echo "$USER_home"` in dòng trống còn `echo "${USER}_home"` in `an_home`.
3. Chạy `bash -c 'cd /tmp; pwd'; pwd`. Hai lệnh `pwd` in gì? Vì sao script không đổi được thư mục của shell đã chạy nó?
4. Thêm vào `~/.bashrc` một alias `taillog` in 20 dòng cuối của log. Mở một tab phiên mới tới cùng server (nút «Mở kết nối khác» cạnh tab phiên): alias có sẵn không, có cần `source` không?
5. Trong một phiên, gõ `PATH=/tmp`, rồi thử `ls`. Lỗi là gì? Cứu phiên đó mà không đăng nhập lại.

## Đọc tiếp

[LNX-012 · vim cho công việc hằng ngày trên server](/articles/linux-vim-hang-ngay): sửa bảng giá và file cấu hình nhanh hơn, với những lệnh vim dùng nhiều nhất.
