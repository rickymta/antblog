---
slug: linux-tim-kiem-find-grep
title: "Tìm kiếm trên Linux: find, grep và tìm trong terminal của SSH Studio"
summary: "find tìm file theo tên, loại, kích thước, thời điểm sửa, quyền và chủ sở hữu, rồi chạy lệnh trên kết quả bằng -exec. grep tìm trong nội dung: đếm, số dòng, ngữ cảnh trước sau, đảo điều kiện, biểu thức chính quy mở rộng, tìm đệ quy trong thư mục. Bài kèm ô tìm kiếm của terminal SSH Studio và cách chọn đúng công cụ."
category: devops-linux
level: advanced
tags: [linux, lnx-009, find, grep, regex, ssh-studio, toi-noi]
language: vi
---

Một khách hàng khiếu nại về báo giá mang mã `q-100057`. Bộ phận chăm sóc khách hàng chuyển mã đó cho An và hỏi: lúc ấy hệ thống đã trả lời gì? Cùng ngày, Bình hỏi một câu khác hẳn: trên server có những file nào nhắc tới `bang-gia.csv`, vì nhóm sắp đổi tên file này.

Hai câu hỏi, hai loại tìm kiếm. Câu thứ nhất là **tìm trong nội dung**: dòng nào trong file chứa chuỗi này. Câu thứ hai gồm cả **tìm file**: file nào, ở đâu, thoả điều kiện gì. Linux có một lệnh cho mỗi loại, `grep` và `find`, và người mới hay nhầm lệnh này với lệnh kia. Bài này dạy cả hai, rồi ghép chúng lại.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-009 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-chuyen-huong-pipe">LNX-008</a>. Trạng thái đầu bài: <code>npm run setup -- 09</code>. Sau bài này, bạn tìm được file theo mọi thuộc tính mà <code>ls -l</code> hiển thị, tìm được chuỗi trong một file hoặc cả cây thư mục kèm ngữ cảnh, và biết khi nào dùng <code>find</code>, <code>grep</code> hay ô tìm kiếm của terminal.</p>
</div>

## 1. `find`: tìm file theo thuộc tính

Cú pháp của `find` khác các lệnh bạn đã gặp:

```text
find   Ở_ĐÂU   ĐIỀU_KIỆN…   [HÀNH_ĐỘNG]
```

`find` đi qua **mọi** thư mục con của nơi bắt đầu, kiểm tra từng mục với các điều kiện, và in đường dẫn của những mục thoả mãn.

### Theo tên

```text
an@toinoi-app-01:~$ find /opt/toinoi -name '*.csv'
/opt/toinoi/bao-gia/bang-gia.csv
an@toinoi-app-01:~$ find ~ -name 'bang-gia*'
/home/an/tai-lieu/bang-gia.csv
/home/an/du-an/bao-gia/cau-hinh/bang-gia.csv.bak
/home/an/du-an/bao-gia/bang-gia.csv
an@toinoi-app-01:~$ find ~ -iname '*.TXT'
/home/an/tai-lieu/lich-truc.txt
/home/an/tai-lieu/ghi-chu.txt
/home/an/tai-lieu/loi-theo-dich-vu.txt
/home/an/bao-cao-03.txt
/home/an/huong-dan.txt
```

- **`-name 'mẫu'`** so với **tên file** (không phải cả đường dẫn), dùng wildcard như của shell.
- **`-iname`** không phân biệt hoa thường.

**Luôn đặt mẫu trong dấu nháy đơn.** Không có nháy, shell sẽ mở rộng `*.csv` theo các file trong thư mục **hiện tại** trước khi `find` kịp chạy ([LNX-005](/articles/linux-tao-sao-chep-di-chuyen-xoa-file)), và `find` nhận một thứ hoàn toàn khác với ý bạn. Đây là lỗi `find` phổ biến nhất.

Thứ tự kết quả là thứ tự `find` gặp các mục trên đĩa, không phải thứ tự chữ cái. Cần xếp thì nối `| sort`.

### Theo loại, kích thước, thời điểm

```text
an@toinoi-app-01:~$ find ~/du-an -type d
/home/an/du-an
/home/an/du-an/bao-gia
/home/an/du-an/bao-gia/releases
/home/an/du-an/bao-gia/shared
/home/an/du-an/bao-gia/cau-hinh
an@toinoi-app-01:~$ find /var/log -type f -size +20k
/var/log/dpkg.log
/var/log/bootstrap.log
/var/log/apt/history.log
/var/log/apt/term.log
/var/log/journal/eb838f0e03e34a54af4996cd142d5f22/system.journal
/var/log/journal/eb838f0e03e34a54af4996cd142d5f22/user-1000.journal
/var/log/toinoi/bao-gia.log
find: ‘/var/log/private’: Permission denied
an@toinoi-app-01:~$ find /var/log -type f -size +20k 2>/dev/null
/var/log/dpkg.log
/var/log/bootstrap.log
/var/log/apt/history.log
/var/log/apt/term.log
/var/log/journal/eb838f0e03e34a54af4996cd142d5f22/system.journal
/var/log/journal/eb838f0e03e34a54af4996cd142d5f22/user-1000.journal
/var/log/toinoi/bao-gia.log
an@toinoi-app-01:~$ find /opt/toinoi -type f -newer /opt/toinoi/bao-gia/bang-gia.csv
/opt/toinoi/bao-gia/bi-mat.env
/opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh
an@toinoi-app-01:~$ find ~ -type f -mmin -30 -not -name '.*'
/home/an/tai-lieu/lich-truc.txt
/home/an/tai-lieu/ghi-chu.txt
/home/an/tai-lieu/loi-theo-dich-vu.txt
/home/an/tai-lieu/bang-gia.csv
/home/an/du-an/bao-gia/kiem-tra-cau-hinh.sh
/home/an/du-an/bao-gia/cau-hinh/bang-gia.csv.bak
/home/an/du-an/bao-gia/bang-gia.csv
/home/an/bao-cao-03.txt
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tim-kiem-find-grep/ssh-studio-05.webp" alt="Terminal của SSH Studio: find /var/log -size +20k kèm lỗi Permission denied rồi bản có 2>/dev/null, find -newer bang-gia.csv, find ~ -mmin -30 liệt kê các file vừa tạo trong home" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>find</code> theo kích thước và thời gian sửa</figcaption></figure>

| Điều kiện | Khớp với |
|---|---|
| `-type f` / `-type d` / `-type l` | File thường / thư mục / symlink |
| `-size +20k` | Lớn hơn 20 KB. `-size -1M` nhỏ hơn 1 MB. Đơn vị: `k`, `M`, `G` |
| `-mmin -30` | Nội dung sửa trong vòng 30 phút. `-mtime -7`: trong 7 ngày. `+` là «lâu hơn» |
| `-newer FILE` | Sửa sau `FILE` |
| `-not ĐK` hoặc `! ĐK` | Đảo điều kiện |
| `ĐK1 ĐK2` | Cả hai (mặc định là «và»). `ĐK1 -o ĐK2` là «hoặc» |

Lệnh thứ hai có một dòng lỗi lẫn giữa kết quả: `find: ‘/var/log/private’: Permission denied`. `find` gặp một thư mục mà `an` không được vào, báo lỗi ra stderr, rồi đi tiếp. Lệnh thứ ba thêm `2>/dev/null` để bỏ các dòng đó. Trên server thật, tìm từ `/` với tài khoản thường sinh ra hàng trăm dòng như vậy, nên `2>/dev/null` gần như luôn đi cùng `find`. (Nhớ bài trước: việc này giấu cả những lỗi bạn có thể cần biết.)

Tên thư mục dài trong `/var/log/journal/…` là mã định danh của máy; trên lab của bạn nó khác.

`find … -mmin -30` trả lời một câu hỏi rất thực tế khi điều tra sự cố: **«cái gì vừa thay đổi?»**. Sau một lần triển khai lỗi, `find /etc /opt -type f -mmin -60` cho bạn danh sách nghi phạm.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tim-kiem-find-grep/ssh-studio-01.webp" alt="Terminal của SSH Studio với find theo tên: *.csv trong /opt/toinoi, bang-gia* trong thư mục home, -iname *.TXT, và find -type d liệt kê các thư mục trong du-an" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>find</code> theo tên và theo loại</figcaption></figure>

### Theo quyền và chủ sở hữu

```text
an@toinoi-app-01:~$ find /opt/toinoi -type f -perm 600
/opt/toinoi/bao-gia/bi-mat.env
an@toinoi-app-01:~$ find ~ -type f -perm -u+x
/home/an/du-an/bao-gia/kiem-tra-cau-hinh.sh
an@toinoi-app-01:~$ find /opt /var/log -user baogia 2>/dev/null
/opt/toinoi
/opt/toinoi/bao-gia
/opt/toinoi/bao-gia/appsettings.json
/opt/toinoi/bao-gia/bi-mat.env
/opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh
/opt/toinoi/bao-gia/bang-gia.csv
/var/log/toinoi
/var/log/toinoi/bao-gia.log
```

- **`-perm 600`**: quyền **đúng bằng** 600.
- **`-perm -u+x`**: có **ít nhất** quyền chạy của chủ (dấu `-` đứng trước nghĩa là «gồm các bit này»).
- **`-user`**, **`-group`**: theo chủ sở hữu, theo nhóm.

Đây là công cụ kiểm tra bảo mật: `find /srv -type f -perm -o+w` tìm file mà ai cũng ghi được; `find / -nouser 2>/dev/null` tìm file của tài khoản đã bị xoá.

### Làm gì đó với kết quả: `-exec`

```text
an@toinoi-app-01:~$ find ~ -name '*.csv' -exec ls -l {} \;
-rw-r--r-- 1 an an 188 Nov 22 22:27 /home/an/tai-lieu/bang-gia.csv
-rw-r----- 1 an toinoi 188 Nov 22 22:27 /home/an/du-an/bao-gia/bang-gia.csv
an@toinoi-app-01:~$ find ~ -name '*.csv' -exec wc -l {} +
  5 /home/an/tai-lieu/bang-gia.csv
  5 /home/an/du-an/bao-gia/bang-gia.csv
 10 total
```

**`-exec LỆNH {} \;`** chạy `LỆNH` cho **từng** file tìm được; `{}` được thay bằng đường dẫn của file. `\;` đánh dấu chỗ kết thúc của lệnh (dấu `\` để shell không coi `;` là dấu ngắt lệnh của chính nó).

**`-exec LỆNH {} +`** gom **nhiều** file vào một lần chạy lệnh. Dòng `10 total` trong kết quả cho thấy `wc` được gọi một lần với cả hai file. Dạng `+` nhanh hơn nhiều khi có hàng nghìn file, và nên là lựa chọn mặc định.

`find` còn có hành động **`-delete`**, xoá mọi thứ khớp. Đây là một lệnh `rm -r` có điều kiện, nên áp dụng quy tắc cũ: **chạy `find` không có `-delete` trước, đọc danh sách, rồi mới thêm `-delete`**. Và đặt `-delete` ở cuối: `find` xử lý điều kiện từ trái sang phải, `-delete` đứng trước `-name` sẽ xoá mọi thứ.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tim-kiem-find-grep/ssh-studio-02.webp" alt="Terminal của SSH Studio với find theo quyền -perm 600 và -perm -u+x, find theo chủ sở hữu baogia, find -exec ls -l cho từng file csv và find -exec wc -l gom hai file trong một lần chạy" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>find</code> theo quyền, chủ sở hữu và <code>-exec</code></figcaption></figure>

## 2. `grep`: tìm trong nội dung

`grep MẪU FILE…` in các **dòng** có chứa mẫu. Bài trước đã dùng nó như một khâu lọc trong pipeline; đây là các tuỳ chọn làm nên sức mạnh của nó.

```text
an@toinoi-app-01:~$ grep -c ERROR /var/log/toinoi/bao-gia.log
17
an@toinoi-app-01:~$ grep -n 'q-100057' /var/log/toinoi/bao-gia.log
59:2026-11-21 06:59:30 INFO  q-100057 dich_vu=car7 km=15.8 gia=102000 95 ms
an@toinoi-app-01:~$ grep -n -B1 -A1 'q-100057' /var/log/toinoi/bao-gia.log
58-2026-11-21 06:57:37 INFO  q-100056 dich_vu=bike km=8.3 gia=29000 114 ms
59:2026-11-21 06:59:30 INFO  q-100057 dich_vu=car7 km=15.8 gia=102000 95 ms
60-2026-11-21 07:01:08 INFO  q-100058 dich_vu=delivery km=13.4 gia=51000 53 ms
an@toinoi-app-01:~$ grep -i 'KHOI DONG' /var/log/toinoi/bao-gia.log
2026-11-21 06:00:00 INFO  bao-gia 1.4.2 khoi dong, cong 8080, moi truong production
an@toinoi-app-01:~$ grep -v INFO /var/log/toinoi/bao-gia.log | head -n 4
2026-11-21 06:02:30 WARN  q-100002 dich_vu=bike km=16.9 bao gia cham: 1406 ms
2026-11-21 06:18:02 WARN  q-100017 dich_vu=delivery km=18.6 bao gia cham: 2295 ms
2026-11-21 06:37:58 ERROR q-100034 dich_vu=car7 khong lay duoc quang duong: timeout sau 3000 ms goi ban-do
2026-11-21 06:40:06 WARN  q-100037 dich_vu=bike km=2.2 bao gia cham: 1548 ms
an@toinoi-app-01:~$ grep -E 'ERROR|WARN' /var/log/toinoi/bao-gia.log | wc -l
41
an@toinoi-app-01:~$ grep -E 'cham: 2[0-9]{3} ms' /var/log/toinoi/bao-gia.log
2026-11-21 06:18:02 WARN  q-100017 dich_vu=delivery km=18.6 bao gia cham: 2295 ms
2026-11-21 08:41:21 WARN  q-100151 dich_vu=car7 km=16.5 bao gia cham: 2187 ms
2026-11-21 09:12:56 WARN  q-100180 dich_vu=delivery km=2.9 bao gia cham: 2637 ms
2026-11-21 09:56:45 WARN  q-100226 dich_vu=delivery km=9.6 bao gia cham: 2615 ms
2026-11-21 11:09:42 WARN  q-100296 dich_vu=bike km=10.6 bao gia cham: 2386 ms
2026-11-21 11:21:33 WARN  q-100311 dich_vu=bike km=20.1 bao gia cham: 2671 ms
2026-11-21 11:27:22 WARN  q-100315 dich_vu=car7 km=15.7 bao gia cham: 2570 ms
an@toinoi-app-01:~$ grep -w 8080 /opt/toinoi/bao-gia/appsettings.json
    "Port": 8080,
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tim-kiem-find-grep/ssh-studio-07.webp" alt="Terminal của SSH Studio: grep -c ERROR in 17, grep -n tìm q-100057 ở dòng 59, grep -n -B1 -A1 in thêm một dòng trước và sau, grep -i KHOI DONG không phân biệt hoa thường" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>grep -c</code>, <code>-n</code>, ngữ cảnh <code>-B</code>/<code>-A</code> và <code>-i</code></figcaption></figure>

| Tuỳ chọn | Tác dụng |
|---|---|
| `-c` | Chỉ **đếm** số dòng khớp (thay cho `\| wc -l`) |
| `-n` | In kèm **số dòng** |
| `-B1 -A1` | In thêm 1 dòng **trước** (*before*) và 1 dòng **sau** (*after*). `-C2` là 2 dòng cả hai phía |
| `-i` | Không phân biệt hoa thường |
| `-v` | **Đảo**: in các dòng **không** khớp |
| `-E` | Mẫu là biểu thức chính quy mở rộng |
| `-w` | Khớp **cả từ**: `8080` không khớp bên trong `18080` |
| `-o` | Chỉ in phần khớp, không in cả dòng |
| `-q` | Không in gì, chỉ trả mã thoát |

### Trả lời câu hỏi về `q-100057`

`grep -n 'q-100057'` cho đúng một dòng, ở dòng 59 của file: lúc 06:59:30, dịch vụ đã báo giá 102.000 đ cho 15,8 km xe 7 chỗ, mất 95 ms. Không có lỗi.

`-B1 -A1` cho thêm ngữ cảnh. Trong kết quả, dòng khớp có dấu `:` sau số dòng, dòng ngữ cảnh có dấu `-`. Khi điều tra sự cố, thứ bạn cần hiếm khi chỉ là dòng lỗi; bạn cần biết ngay trước đó chuyện gì đã xảy ra.

### `-v` và `-E`

`grep -v INFO` bỏ hết dòng `INFO`, còn lại `WARN` và `ERROR`: cách nhanh để lọc nhiễu khỏi log.

`-E` bật **biểu thức chính quy mở rộng** (extended regular expression):

- `'ERROR|WARN'`: dấu `|` là «hoặc». Có 41 dòng, bằng 17 + 24 của bài trước.
- `'cham: 2[0-9]{3} ms'`: số `2`, theo sau là đúng ba chữ số. Tức các yêu cầu mất từ 2000 tới 2999 ms. Có 7 dòng.

Vài ký hiệu regex dùng nhiều:

| Ký hiệu | Nghĩa |
|---|---|
| `.` | Một ký tự bất kỳ |
| `[0-9]`, `[a-z]` | Một ký tự trong khoảng |
| `*` / `+` / `?` | Lặp 0 lần trở lên / 1 lần trở lên / 0 hoặc 1 lần |
| `{3}`, `{2,4}` | Lặp đúng 3 lần / từ 2 tới 4 lần |
| `^` / `$` | Đầu dòng / cuối dòng |
| `A\|B` | A hoặc B |
| `\.` | Dấu chấm thật (bỏ nghĩa đặc biệt) |

Dấu `*` trong regex **khác** dấu `*` của wildcard: trong regex nó nghĩa là «lặp ký tự đứng trước», còn «chuỗi bất kỳ» viết là `.*`. Đặt mẫu của `grep` trong **nháy đơn** để shell không đụng vào các ký tự `*`, `|`, `$`.

Khi mẫu là một chuỗi cố định có ký tự đặc biệt (địa chỉ IP, đường dẫn), dùng `grep -F`: mọi ký tự được hiểu đúng theo mặt chữ.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tim-kiem-find-grep/ssh-studio-03.webp" alt="Terminal của SSH Studio: grep -v INFO | head -n 4, grep -E ERROR|WARN | wc -l in 41, grep -E cham: 2[0-9]{3} ms tô đỏ các yêu cầu chậm trên 2 giây, grep -w 8080 trong appsettings.json" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>grep -v</code>, <code>-E</code> với biểu thức chính quy, <code>-w</code> khớp nguyên từ</figcaption></figure>

Trên terminal, `grep` tô đỏ phần khớp (nhờ alias `grep --color=auto` của Ubuntu). Màu này rất có ích khi viết regex: bạn thấy ngay mẫu của mình khớp vào đâu.

### Tìm trong cả thư mục

```text
an@toinoi-app-01:~$ grep -r 'bang-gia.csv' /opt/toinoi
/opt/toinoi/bao-gia/appsettings.json:    "RuleFile": "/opt/toinoi/bao-gia/bang-gia.csv",
grep: /opt/toinoi/bao-gia/bi-mat.env: Permission denied
/opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh:bang_gia="$thu_muc/bang-gia.csv"
an@toinoi-app-01:~$ grep -r 'bang-gia.csv' /opt/toinoi 2>/dev/null
/opt/toinoi/bao-gia/appsettings.json:    "RuleFile": "/opt/toinoi/bao-gia/bang-gia.csv",
/opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh:bang_gia="$thu_muc/bang-gia.csv"
an@toinoi-app-01:~$ grep -rl 'bao-gia' /opt/toinoi ~/du-an 2>/dev/null
/opt/toinoi/bao-gia/appsettings.json
/opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh
/home/an/du-an/bao-gia/kiem-tra-cau-hinh.sh
an@toinoi-app-01:~$ grep -rn --include='*.sh' 'exit' /opt/toinoi
/opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh:9:  exit 1
```

- **`-r`** (*recursive*) tìm trong mọi file dưới thư mục. Kết quả có dạng `đường-dẫn:dòng-khớp`.
- **`-l`** chỉ in **tên file** có chứa mẫu.
- **`--include='*.sh'`** giới hạn theo tên file. `--exclude-dir=node_modules` bỏ qua một thư mục.

Đây là câu trả lời cho Bình: `bang-gia.csv` được nhắc tới trong `appsettings.json` (khoá `RuleFile`) và trong `kiem-tra-cau-hinh.sh`. Đổi tên file thì phải sửa hai chỗ đó.

Nhưng lệnh đầu có một dòng lỗi: `grep: /opt/toinoi/bao-gia/bi-mat.env: Permission denied`. `an` không đọc được file đó, nên **`grep` không biết bên trong có nhắc tới `bang-gia.csv` hay không**. `2>/dev/null` làm lỗi biến mất nhưng không làm câu trả lời đầy đủ hơn. Khi báo cáo cho Bình, câu đúng là «hai file, và một file tôi không đọc được»; muốn chắc thì chạy lại với `sudo`.

## 3. Ghép `find` và `grep`

```text
an@toinoi-app-01:~$ find ~ -name '*.csv' -exec grep -H 'bike' {} +
/home/an/tai-lieu/bang-gia.csv:bike,Xe máy,9000,2500,12000
/home/an/du-an/bao-gia/bang-gia.csv:bike,Xe máy,8000,2500,12000
```

`find` chọn file theo thuộc tính, `grep` tìm trong nội dung của chúng. `-H` buộc `grep` luôn in tên file. Kết quả cho thấy hai bản `bang-gia.csv` của An khác nhau ở giá mở cửa xe máy: bản nháp trong `tai-lieu` đã sửa thành 9000, bản trong `du-an` còn 8000.

`grep -r --include` đủ cho điều kiện theo tên. Cần điều kiện mà `grep` không có (kích thước, thời điểm sửa, chủ sở hữu) thì dùng `find … -exec grep … {} +`. Ví dụ: tìm `timeout` trong các file log sửa trong 24 giờ qua:

```sh
find /var/log -name '*.log' -mtime -1 -exec grep -l 'timeout' {} + 2>/dev/null
```

## 4. Lệnh nằm ở đâu?

```text
an@toinoi-app-01:~$ which ls vim tree
/usr/bin/ls
/usr/bin/vim
/usr/bin/tree
an@toinoi-app-01:~$ whereis ls
ls: /usr/bin/ls /usr/share/man/man1/ls.1.gz
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tim-kiem-find-grep/ssh-studio-06.webp" alt="Terminal của SSH Studio: grep -r bang-gia.csv trong /opt/toinoi kèm lỗi Permission denied, grep -rl, grep -rn --include=*.sh, find -exec grep -H bike, which ls vim tree, whereis ls" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>grep -r</code> trong cả thư mục, ghép với <code>find</code>, và <code>which</code>/<code>whereis</code></figcaption></figure>

`which` in đường dẫn của chương trình sẽ chạy khi bạn gõ tên lệnh (tìm trong `PATH`). `whereis` in thêm vị trí trang `man`. Để biết một cái tên là alias, lệnh dựng sẵn hay chương trình, `type` ([LNX-003](/articles/linux-shell-dong-lenh-tro-giup)) vẫn là lệnh trả lời đầy đủ nhất.

## 5. Tìm trong terminal của SSH Studio

Có một loại tìm kiếm thứ ba: tìm trong **những gì đã hiện trên màn hình**. Bạn vừa chạy vài lệnh, kết quả dài, và muốn tìm một chữ trong đó mà không chạy lại. Trong terminal của SSH Studio, nhấn **`Ctrl+Shift+F`**:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tim-kiem-find-grep/ssh-studio-04.webp" alt="Terminal của SSH Studio với ô tìm kiếm ở góc trên bên phải đang tìm delivery, hiển thị 2/4 cùng các nút Aa, .*, lên, xuống, đóng; các chữ delivery trong kết quả grep được tô sáng" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · ô tìm kiếm của terminal tô sáng mọi chỗ khớp trên màn hình</figcaption></figure>

Ô tìm kiếm hiện ở góc trên bên phải của terminal. Gõ `delivery`: mọi chỗ khớp trong nội dung terminal (kể cả phần đã cuộn lên) được tô sáng, kèm số thứ tự kết quả đang đứng trên tổng số.

| Nút / phím | Tác dụng |
|---|---|
| `Enter` / `Shift+Enter` | Kết quả sau / trước |
| **Aa** | Phân biệt hoa thường |
| **.\*** | Coi chuỗi tìm là biểu thức chính quy |
| `Esc` | Đóng |

Tìm trong terminal chạy trên máy bạn, không gửi gì tới server, và chỉ thấy những gì còn trong vùng cuộn (scrollback) của terminal. Nó không thay được `grep`: nếu câu trả lời nằm trong file chứ không nằm trên màn hình, hãy `grep` file.

## 6. Chọn công cụ nào?

| Câu hỏi | Công cụ |
|---|---|
| File tên là gì đó nằm ở đâu? | `find -name` |
| File nào vừa thay đổi / lớn bất thường / có quyền sai? | `find -mmin`, `-size`, `-perm` |
| Dòng nào trong file này chứa chuỗi X? | `grep` |
| File nào trong thư mục này nhắc tới X? | `grep -r`, `grep -rl` |
| Điều kiện về file **và** về nội dung | `find … -exec grep … {} +` |
| Chữ X nằm đâu trong kết quả tôi vừa thấy? | `Ctrl+Shift+F` trong terminal |
| Tìm trong file đang mở bằng `less`, `vim`, `man` | `/chuỗi` |
| Lệnh X là gì, ở đâu? | `type`, `which` |

## 7. Lỗi thường gặp

**Quên nháy quanh mẫu của `find -name`.** `find . -name *.csv` chạy đúng khi thư mục hiện tại không có file `.csv` nào và sai một cách khó hiểu khi có. Luôn viết `'*.csv'`.

**Dùng `grep` để tìm tên file, hoặc `find` để tìm nội dung.** `grep bang-gia.csv /opt` không tìm file tên đó. `find` không đọc nội dung file.

**`find /` không có `2>/dev/null`.** Kết quả chìm trong lỗi quyền. (Và có `2>/dev/null` thì nhớ rằng bạn chỉ thấy phần mình được phép thấy.)

**Dấu chấm trong regex.** `grep '1.4.2'` khớp cả `1x4y2`. Viết `'1\.4\.2'` hoặc dùng `grep -F`.

**`grep` ra quá nhiều vì khớp chuỗi con.** Tìm `car` ra cả `car4`, `car7`. Dùng `-w`, hoặc mẫu chặt hơn như `'dich_vu=car4 '`.

**`-delete` đặt sai chỗ.** Xem mục 1.

## Bài tập

1. Tìm mọi file trong `/etc` có tên bắt đầu bằng `ssh`. Có bao nhiêu? (Nối `| wc -l`.)
2. Tìm các file trong thư mục riêng của bạn lớn hơn 100 byte và nhỏ hơn 1 KB (`-size` dùng đơn vị `c` cho byte). In kèm kích thước bằng `-exec ls -l {} +`.
3. Khách hàng báo mã `q-100226`. Yêu cầu đó thuộc dịch vụ nào, mất bao lâu? In kèm 2 dòng trước và sau.
4. Đếm số yêu cầu `INFO` có giá từ 100.000 đ trở lên. Gợi ý: `gia=` theo sau là sáu chữ số.
5. Viết lệnh liệt kê **tên** các file dưới `/opt/toinoi` và `/etc/ssh` có chứa chữ `Port` (không phân biệt hoa thường), bỏ qua lỗi quyền. Rồi nói rõ: câu trả lời của bạn có thể thiếu gì?
6. Chạy `grep -n ERROR /var/log/toinoi/bao-gia.log`, rồi dùng `Ctrl+Shift+F` với nút `.*` để tìm `car[47]`. Có bao nhiêu chỗ khớp?

## Đọc tiếp

[LNX-010 · Thực hành: chuẩn bị thư mục triển khai dịch vụ báo giá](/articles/linux-thuc-hanh-thu-muc-trien-khai): dùng mọi thứ của Phần 1 để bàn giao một thư mục có bộ chấm tự động.
