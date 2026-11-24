---
slug: linux-cut-sort-uniq-tr
title: "Trích cột và dòng trên Linux: cut, sort, uniq, tr, wc và log đã nén"
summary: "Đọc log của ba ngày, trong đó một file đã nén gzip, bằng zcat -f; vì sao cut lấy sai cột khi có dấu cách kép và tr -s sửa thế nào; đếm theo nhiều khoá bằng sort | uniq -c; sắp xếp theo cột số bằng sort -t -k; dàn bảng bằng column; wc đếm byte khác đếm ký tự ra sao với tiếng Việt; tìm dòng chỉ có ở một file bằng uniq -u."
category: devops-linux
level: beginner
tags: [linux, lnx-013, cut, sort, uniq, tr, wc, zcat, column, ssh-studio, toi-noi]
language: vi
---

Bình gửi An một câu hỏi qua chat: «Sự cố sáng 20/11 có phải chỉ là lỗi dịch vụ bản đồ như mọi ngày không? So giúp mình ba ngày.»

[LNX-008](/articles/linux-chuyen-huong-pipe) đã có công thức đếm theo nhóm cho một file log. Câu hỏi của Bình đòi thêm ba thứ: đọc một file **đã nén**, gộp **nhiều ngày**, và đếm theo **hai khoá** cùng lúc (ngày và loại lỗi). Bài này đào sâu các công cụ xử lý cột và dòng, và chỉ ra một cái bẫy trong chính định dạng log của `bao-gia` mà Phần 1 đã khéo tránh.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-013 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/linux-chuyen-huong-pipe">LNX-008</a>. Trạng thái đầu bài: <code>npm run setup -- 13</code>. Sau bài này, bạn đọc được log nén mà không giải nén ra đĩa, cắt cột đúng dù dữ liệu có khoảng trắng không đều, đếm và sắp xếp theo nhiều khoá, và trả lời được câu hỏi của Bình.</p>
</div>

## 1. Ba ngày log, một file đã nén

```text
an@toinoi-app-01:~$ cd /var/log/toinoi
an@toinoi-app-01:/var/log/toinoi$ ls -l
total 60
-rw-r--r-- 1 baogia toinoi 27260 Nov 21 12:17 bao-gia.log
-rw-r--r-- 1 baogia toinoi 23353 Nov 20 11:16 bao-gia.log.1
-rw-r--r-- 1 baogia toinoi  4314 Nov 19 11:07 bao-gia.log.2.gz
an@toinoi-app-01:/var/log/toinoi$ file bao-gia.log.2.gz
bao-gia.log.2.gz: gzip compressed data, max compression, from Unix, original size modulo 2^32 22916
an@toinoi-app-01:/var/log/toinoi$ zcat bao-gia.log.2.gz | head -n 3
2026-11-19 06:00:00 INFO  bao-gia 1.4.1 khoi dong, cong 8080, moi truong production
2026-11-19 06:00:00 INFO  nap bang gia: 4 loai dich vu tu /opt/toinoi/bao-gia/bang-gia.csv
2026-11-19 06:00:43 INFO  q-099401 dich_vu=delivery km=9.1 gia=38000 179 ms
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cut-sort-uniq-tr/ssh-studio-05.webp" alt="Terminal của SSH Studio: cd /var/log/toinoi, ls -l ba file log, file nhận ra bao-gia.log.2.gz là gzip compressed data, zcat bao-gia.log.2.gz | head -n 3" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · một file log đã nén: <code>file</code> nhận ra, <code>zcat</code> đọc</figcaption></figure>

`file` cho biết `bao-gia.log.2.gz` là dữ liệu nén gzip, không phải văn bản: `cat` hay `grep` thẳng lên nó chỉ ra ký tự rác. **`zcat`** giải nén ra stdout, file trên đĩa giữ nguyên. Gia đình của nó: `zless` để xem từng trang, `zgrep` để tìm.

```text
an@toinoi-app-01:/var/log/toinoi$ wc -l bao-gia.log.1 bao-gia.log
  305 bao-gia.log.1
  362 bao-gia.log
  667 total
an@toinoi-app-01:/var/log/toinoi$ zcat bao-gia.log.2.gz | wc -l
302
an@toinoi-app-01:/var/log/toinoi$ zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | wc -l
969
an@toinoi-app-01:/var/log/toinoi$ zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | cut -d' ' -f1 | uniq -c
    302 2026-11-19
    305 2026-11-20
    362 2026-11-21
```

- `wc -l` với nhiều file in từng file và dòng `total`.
- **`zcat -f`** (*force*): file nén thì giải nén, file thường thì in nguyên. Một lệnh đọc được cả ba file, đúng thứ tự thời gian: 19, 20, 21/11.
- `cut -d' ' -f1 | uniq -c` đếm số dòng mỗi ngày. Không cần `sort` vì log đã được ghi theo thứ tự thời gian, nên các dòng cùng ngày vốn đã nằm liền nhau. `uniq` chỉ cần thế.

Từ đây, mẫu `zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | …` là cách đọc «toàn bộ log» của bài. Bạn sẽ thấy nó ở đầu hầu hết các pipeline.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cut-sort-uniq-tr/ssh-studio-01.webp" alt="Terminal của SSH Studio trong /var/log/toinoi: wc -l hai file chưa nén, zcat … | wc -l đếm file nén 302 dòng, zcat -f ghép ba ngày 969 dòng, cut -f1 | uniq -c đếm số dòng theo ngày" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>zcat -f</code> đọc cả file nén lẫn chưa nén; đếm dòng theo ngày</figcaption></figure>

## 2. Cái bẫy của dấu cách kép

Nhìn kỹ log: sau `INFO` và `WARN` có **hai** dấu cách, sau `ERROR` chỉ có một. Định dạng này giữ cho cột mã yêu cầu thẳng hàng khi đọc bằng mắt, và làm `cut` lấy sai cột:

```text
an@toinoi-app-01:/var/log/toinoi$ head -n 4 bao-gia.log | cut -d' ' -f3,4
INFO
INFO
INFO
WARN
an@toinoi-app-01:/var/log/toinoi$ head -n 4 bao-gia.log | tr -s ' ' | cut -d' ' -f3,4
INFO bao-gia
INFO nap
INFO q-100001
WARN q-100002
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cut-sort-uniq-tr/ssh-studio-06.webp" alt="Terminal của SSH Studio: head -n 4 | cut -d' ' -f3,4 chỉ in INFO vì dấu cách kép, thêm tr -s ' ' thì ra đúng INFO bao-gia, INFO q-100001, WARN q-100002" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>tr -s ' '</code> gộp dấu cách kép trước khi <code>cut</code></figcaption></figure>

`cut -d' '` coi **mỗi** dấu cách là một ranh giới. Giữa hai dấu cách liền nhau là một trường rỗng, nên trường 4 của dòng `INFO` là chuỗi rỗng, còn mã yêu cầu bị đẩy sang trường 5. Trên dòng `ERROR`, mã yêu cầu lại ở trường 4. Cùng một lệnh `cut`, hai kết quả khác nhau tuỳ mức log.

**`tr -s ' '`** (*squeeze*) gộp mọi chuỗi dấu cách liền nhau thành một. Sau đó trường nào cũng ở đúng chỗ của nó trên mọi dòng.

Đây là lý do LNX-008 dùng `cut -c21-25` (cắt theo **vị trí ký tự**) để lấy mức log, và dùng `cut -d' ' -f5` chỉ trên các dòng `ERROR`. Hai cách đó đúng, nhưng chỉ đúng nhờ biết trước định dạng. Gặp dữ liệu có khoảng trắng không đều, hãy `tr -s ' '` trước `cut`, hoặc dùng `awk` ([LNX-015](/articles/linux-awk-xu-ly-log)), công cụ tự tách theo mọi khoảng trắng.

### Mức log theo từng ngày: cut nhiều khoảng

```text
an@toinoi-app-01:/var/log/toinoi$ zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | cut -c1-10,21-25 --output-delimiter=' ' | sort | uniq -c
     18 2026-11-19 ERROR
    259 2026-11-19 INFO
     25 2026-11-19 WARN
     26 2026-11-20 ERROR
    260 2026-11-20 INFO
     19 2026-11-20 WARN
     17 2026-11-21 ERROR
    321 2026-11-21 INFO
     24 2026-11-21 WARN
```

`cut -c1-10,21-25` lấy hai khoảng ký tự: ngày và mức log. Mặc định `cut` dán các khoảng liền nhau (`2026-11-19INFO`); `--output-delimiter=' '` chèn dấu cách giữa chúng. `sort | uniq -c` đếm mỗi cặp ngày–mức. Ngày 20/11 có 26 lỗi, nhiều hơn hẳn hai ngày còn lại.

### tr: đổi từng ký tự

```text
an@toinoi-app-01:/var/log/toinoi$ head -n 1 bao-gia.log | tr ' ' '\n' | head -n 4
2026-11-21
06:00:00
INFO

an@toinoi-app-01:/var/log/toinoi$ echo "dich_vu=car4" | tr 'a-z' 'A-Z'
DICH_VU=CAR4
```

`tr TẬP1 TẬP2` thay mỗi ký tự trong tập thứ nhất bằng ký tự ở cùng vị trí trong tập thứ hai: dấu cách thành xuống dòng, chữ thường thành chữ hoa. Dòng trống thứ tư trong kết quả là trường rỗng giữa hai dấu cách sau `INFO`. `tr -d 'abc'` xoá các ký tự `a`, `b`, `c`.

`tr` làm việc với **từng ký tự**, không phải với chuỗi. `tr 'abc' 'xyz'` không thay chuỗi `abc` bằng `xyz`; nó đổi mọi `a` thành `x`, mọi `b` thành `y`, mọi `c` thành `z`. Thay chuỗi là việc của `sed` ([LNX-014](/articles/linux-sed-sua-van-ban)). `tr` cũng không xử lý tốt chữ có dấu, vì một chữ như `ỗ` là nhiều byte.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cut-sort-uniq-tr/ssh-studio-02.webp" alt="Terminal của SSH Studio: cut -c1-10,21-25 | sort | uniq -c đếm mức log theo từng ngày, tr đổi dấu cách thành xuống dòng, tr a-z A-Z đổi chữ hoa" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · mức log theo ngày; <code>tr</code> đổi ký tự</figcaption></figure>

## 3. Trả lời câu hỏi của Bình

```text
an@toinoi-app-01:/var/log/toinoi$ zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | grep ' ERROR ' | cut -d' ' -f6- | sort | uniq -c | sort -rn
     54 khong lay duoc quang duong: timeout sau 3000 ms goi ban-do
      7 khong doc duoc bang gia: file dang bi khoa
an@toinoi-app-01:/var/log/toinoi$ zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | grep 'bi khoa' | cut -d' ' -f1,2 | head -n 3
2026-11-20 09:07:12
2026-11-20 09:09:21
2026-11-20 09:12:04
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cut-sort-uniq-tr/ssh-studio-07.webp" alt="Terminal của SSH Studio: hai loại thông báo lỗi trong ba ngày, 54 lỗi không lấy được quãng đường và 7 lỗi bảng giá bị khoá, ba lần bảng giá bị khoá đầu tiên đều sáng 20/11" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · hai loại lỗi và thời điểm lỗi khoá bảng giá</figcaption></figure>

`cut -d' ' -f6-` lấy **từ trường 6 tới hết dòng**: phần thông báo lỗi. (Trên dòng `ERROR` chỉ có dấu cách đơn nên `cut -d' '` dùng được.) Đếm các thông báo giống nhau cho ra câu trả lời: ba ngày có **hai loại lỗi**. 54 lỗi là timeout khi gọi dịch vụ bản đồ, loại quen thuộc. 7 lỗi còn lại là loại mới: «không đọc được bảng giá: file đang bị khoá», và cả 7 đều nằm trong khoảng 09:05 đến 09:15 sáng 20/11.

Đó là lúc bảng giá được sửa (file `bang-gia.csv` có thời điểm sửa 09:15 ngày 20/11, [LNX-004](/articles/linux-cay-thu-muc-duong-dan)) và dịch vụ được nâng cấp từ 1.4.1 lên 1.4.2 ([LNX-014](/articles/linux-sed-sua-van-ban) sẽ tìm ra dòng đó). Sự cố sáng 20/11 không phải lỗi bản đồ: nó là cái giá của việc sửa bảng giá trong khi dịch vụ đang chạy.

### Sắp xếp theo cột số

```text
an@toinoi-app-01:/var/log/toinoi$ zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | grep ' WARN ' | tr -s ' ' | sort -t' ' -k10,10n | tail -n 3
2026-11-19 10:22:34 WARN q-099661 dich_vu=bike km=20.1 bao gia cham: 2671 ms
2026-11-21 11:21:33 WARN q-100311 dich_vu=bike km=20.1 bao gia cham: 2671 ms
2026-11-19 07:19:28 WARN q-099483 dich_vu=car7 km=16.8 bao gia cham: 2685 ms
```

| Tuỳ chọn | Ý nghĩa |
|---|---|
| `-t' '` | Ký tự phân cách cột (mặc định: chuỗi khoảng trắng bất kỳ) |
| `-k10,10` | Sắp theo cột 10 **và chỉ cột 10**. Viết `-k10` thì khoá kéo dài từ cột 10 tới hết dòng |
| `n` sau số cột | So theo giá trị số, không theo chữ |
| `-r` | Đảo thứ tự |
| `-u` | Bỏ các dòng trùng khoá (như `sort \| uniq`) |

Sau `tr -s ' '`, cột 10 của dòng `WARN` là số mili giây. Yêu cầu chậm nhất của ba ngày mất 2685 ms, vào sáng 19/11. Có hai yêu cầu cùng 2671 ms; `sort` giữ chúng theo thứ tự so sánh cả dòng.

Không có `n`, `sort` so chữ: `900` đứng sau `2685` vì ký tự `9` lớn hơn `2`. Thiếu `n` trên cột số là lỗi sắp xếp hay gặp nhất.

### Dàn thành bảng

```text
an@toinoi-app-01:/var/log/toinoi$ zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | grep ' ERROR ' | cut -d' ' -f1,5 | sort | uniq -c | column -t
4   2026-11-19  dich_vu=bike
5   2026-11-19  dich_vu=car4
5   2026-11-19  dich_vu=car7
4   2026-11-19  dich_vu=delivery
8   2026-11-20  dich_vu=bike
1   2026-11-20  dich_vu=car4
5   2026-11-20  dich_vu=car7
12  2026-11-20  dich_vu=delivery
5   2026-11-21  dich_vu=bike
6   2026-11-21  dich_vu=car4
4   2026-11-21  dich_vu=car7
2   2026-11-21  dich_vu=delivery
```

**`column -t`** dàn các cột cho thẳng hàng. Bảng này cho thấy lỗi ngày 20/11 dồn vào `delivery` (12 lỗi) và `bike` (8 lỗi), khác hẳn hai ngày còn lại.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cut-sort-uniq-tr/ssh-studio-03.webp" alt="Terminal của SSH Studio: ba yêu cầu WARN chậm nhất của ba ngày nhờ tr -s và sort -k10,10n, bảng số lỗi theo ngày và dịch vụ dàn cột bằng column -t" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sort</code> theo cột số và bảng lỗi theo ngày, dịch vụ</figcaption></figure>

## 4. Bảng CSV, byte và ký tự

```text
an@toinoi-app-01:/var/log/toinoi$ column -t -s, /opt/toinoi/bao-gia/bang-gia.csv
dich_vu   ten         gia_mo_cua  gia_moi_km  gia_toi_thieu
bike      Xe máy      8000        2500        12000
car4      Ô tô 4 chỗ  12000       4500        25000
car7      Ô tô 7 chỗ  15000       5500        30000
delivery  Giao hàng   10000       3000        15000
an@toinoi-app-01:/var/log/toinoi$ wc -l -c /opt/toinoi/bao-gia/bang-gia.csv
  5 188 /opt/toinoi/bao-gia/bang-gia.csv
an@toinoi-app-01:/var/log/toinoi$ wc -m /opt/toinoi/bao-gia/bang-gia.csv
178 /opt/toinoi/bao-gia/bang-gia.csv
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cut-sort-uniq-tr/ssh-studio-08.webp" alt="Terminal của SSH Studio: column -t -s, dàn bang-gia.csv thành bảng thẳng cột, wc -l -c in 5 dòng 188 byte, wc -m in 178 ký tự" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>column</code> cho CSV; byte và ký tự khác nhau với chữ có dấu</figcaption></figure>

- `column -t -s,` dàn file CSV thành bảng, dùng dấu phẩy làm phân cách. Cách nhanh nhất để đọc một file CSV trên server.
- `wc -c` đếm **byte**: 188. `wc -m` đếm **ký tự**: 178. Mười byte chênh lệch là các chữ có dấu: trong UTF-8, `á` hay `ỗ` chiếm hai hoặc ba byte. `wc -m` đúng khi `LANG` là UTF-8 ([LNX-011](/articles/linux-bien-moi-truong-path-bashrc)); với `LANG=C`, nó đếm byte như `-c`.

Khác biệt byte và ký tự giải thích vì sao `cut -c` có thể cắt đôi một chữ có dấu, và vì sao các cột chứa tiếng Việt đôi khi lệch khi căn bằng `printf`. Log của `bao-gia` cố ý viết không dấu.

### uniq -u: dòng chỉ có ở một bên

```text
an@toinoi-app-01:/var/log/toinoi$ zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | grep -o 'bao-gia 1\.4\.[0-9] khoi dong' | sort | uniq -c
      2 bao-gia 1.4.1 khoi dong
      2 bao-gia 1.4.2 khoi dong
an@toinoi-app-01:/var/log/toinoi$ cut -d, -f1 /opt/toinoi/bao-gia/bang-gia.csv ~/vim/bang-gia.csv | sort | uniq -u
van16
```

Lệnh đầu đếm số lần dịch vụ khởi động theo phiên bản: 1.4.1 hai lần (19 và 20/11), 1.4.2 hai lần (sau khi nâng cấp ngày 20, và sáng 21).

Lệnh sau ghép cột đầu của hai file, xếp lại, rồi **`uniq -u`** chỉ giữ dòng **không lặp**: mã dịch vụ có ở một file mà không có ở file kia. Kết quả là `van16`, loại xe An vừa thêm ở [LNX-012](/articles/linux-vim-hang-ngay). Ngược lại, `uniq -d` chỉ in các dòng **có lặp**. Hai tuỳ chọn này là cách nhanh để so hai danh sách.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-cut-sort-uniq-tr/ssh-studio-04.webp" alt="Terminal của SSH Studio: grep -o đếm các phiên bản 1.4.1 và 1.4.2 đã khởi động, cut -d, -f1 hai bảng giá | sort | uniq -u tìm ra van16 chỉ có ở một file" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>uniq -u</code>: dòng chỉ xuất hiện một lần</figcaption></figure>

## 5. Tóm tắt

| Công cụ | Dùng khi |
|---|---|
| `zcat`, `zcat -f`, `zgrep`, `zless` | Đọc log nén mà không giải nén ra đĩa |
| `cut -d X -f N` | Cột phân cách bằng **một** ký tự cố định |
| `cut -c A-B` | Cột có độ rộng cố định |
| `tr -s ' '` | Gộp khoảng trắng trước khi `cut` |
| `tr A B`, `tr -d A` | Đổi, xoá từng ký tự |
| `sort -t X -kN,Nn` | Sắp xếp theo một cột số |
| `sort \| uniq -c \| sort -rn` | Đếm và xếp hạng |
| `uniq -d`, `uniq -u` | Dòng trùng / dòng không trùng (sau `sort`) |
| `column -t [-s X]` | Dàn bảng |
| `wc -l -w -c -m` | Đếm dòng, từ, byte, ký tự |

## 6. Lỗi thường gặp

**`cut -d' '` trên dữ liệu có khoảng trắng không đều.** Trường rỗng làm lệch cột. `tr -s ' '` trước, hoặc dùng `awk`.

**`uniq` không có `sort`.** Chỉ đúng khi dữ liệu vốn đã liền nhau như log theo thời gian.

**`sort -k10` thay vì `-k10,10`.** Khoá kéo tới hết dòng; với cột số đứng giữa dòng, kết quả có thể sai mà khó thấy.

**Thiếu `n` khi sắp xếp số.** `10` đứng trước `9`.

**`grep` trên file `.gz`.** Không báo lỗi, chỉ không tìm thấy gì (hoặc báo «binary file matches»). Dùng `zgrep` hoặc `zcat -f … | grep`.

**`tr` để thay chuỗi.** `tr` làm theo từng ký tự. Dùng `sed`.

## Bài tập

1. Mỗi ngày có bao nhiêu yêu cầu `delivery`? Viết một pipeline in kết quả theo dạng `số ngày`.
2. Liệt kê 5 mã yêu cầu có quãng đường dài nhất của cả ba ngày. Gợi ý: dòng `INFO` có trường `km=`; tách bằng `tr -s ' '`, cắt ra trường km, bỏ `km=` bằng `cut -d= -f2`, rồi `sort`.
3. Ngày nào có nhiều yêu cầu chậm (`WARN`) nhất trong khung 06:00 đến 07:00? (Ký tự 12 và 13 là giờ.)
4. `cut -d, -f2 /opt/toinoi/bao-gia/bang-gia.csv | wc -m` và `| wc -c` cho hai số khác nhau. Giải thích.
5. So hai danh sách: các dịch vụ có lỗi ngày 19/11 và các dịch vụ có lỗi ngày 21/11. Dịch vụ nào chỉ có ở một ngày? (Gợi ý: `sort -u` từng danh sách trước rồi mới ghép và `uniq -u`.)

## Đọc tiếp

[LNX-014 · sed: sửa văn bản theo dòng](/articles/linux-sed-sua-van-ban): in theo mẫu, thay chuỗi, sửa file cấu hình tại chỗ có bản sao lưu.
