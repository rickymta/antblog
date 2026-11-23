---
slug: linux-tar-gzip-nen-luu-tru
title: "Nén và lưu trữ trên Linux: gzip, xz, zstd, tar và script sao lưu đầu tiên"
summary: "Phân biệt nén (gzip, xz, zstd: một file thành một file nhỏ hơn) với đóng gói (tar: nhiều file thành một). Bài so kích thước ba cách nén trên log của bao-gia, tạo, liệt kê và giải nén file tar.gz, kiểm tra bản giải nén giống hệt bản gốc bằng diff -r, lấy riêng một file khỏi gói, tải bản sao lưu về máy bằng khung Files, rồi viết script sao-luu-log đầu tiên."
category: devops-linux
level: advanced
tags: [linux, lnx-018, tar, gzip, xz, zstd, backup, sftp, ssh-studio, toi-noi]
language: vi
---

Log của `bao-gia` lớn dần mỗi ngày, và xoay vòng log thì cuối cùng sẽ xoá các file cũ. Bình muốn giữ log của mọi ngày ở một chỗ khác server: «Mỗi ngày đóng gói log lại thành một file, nén cho nhỏ, rồi chép đi.»

Câu đó chứa hai việc mà người mới hay gộp làm một. **Nén** biến một file thành một file nhỏ hơn. **Đóng gói** gom nhiều file (kèm tên, quyền, chủ sở hữu, thời điểm sửa) thành một file. Trên Linux, hai việc do hai nhóm công cụ làm: `gzip`, `xz`, `zstd` nén; `tar` đóng gói. File `.tar.gz` là cả hai: đóng gói bằng `tar`, rồi nén bằng `gzip`.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-018 · Phần 2/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-bash-dieu-kien-vong-lap-ham">LNX-017</a>. Trạng thái đầu bài: <code>npm run setup -- 18</code>. Sau bài này, bạn chọn được công cụ nén theo nhu cầu, tạo và kiểm tra được một bản sao lưu bằng <code>tar</code>, khôi phục toàn bộ hoặc một file, tải bản sao lưu về máy, và có script <code>sao-luu-log</code> phiên bản đầu.</p>
</div>

## 1. Nén một file: `gzip`

```text
an@toinoi-app-01:~$ mkdir -p ~/sao-luu
an@toinoi-app-01:~$ cd ~/sao-luu

an@toinoi-app-01:~/sao-luu$ cp /var/log/toinoi/bao-gia.log.1 .
an@toinoi-app-01:~/sao-luu$ ls -l
total 24
-rw-r--r-- 1 an an 23353 Nov 22 23:21 bao-gia.log.1
an@toinoi-app-01:~/sao-luu$ gzip bao-gia.log.1
an@toinoi-app-01:~/sao-luu$ ls -l
total 8
-rw-r--r-- 1 an an 4570 Nov 22 23:21 bao-gia.log.1.gz
an@toinoi-app-01:~/sao-luu$ gzip -l bao-gia.log.1.gz
         compressed        uncompressed  ratio uncompressed_name
               4570               23353  80.6% bao-gia.log.1
an@toinoi-app-01:~/sao-luu$ gunzip bao-gia.log.1.gz
an@toinoi-app-01:~/sao-luu$ gzip -k bao-gia.log.1
an@toinoi-app-01:~/sao-luu$ ls -l
total 32
-rw-r--r-- 1 an an 23353 Nov 22 23:21 bao-gia.log.1
-rw-r--r-- 1 an an  4570 Nov 22 23:21 bao-gia.log.1.gz
```

- **`gzip FILE`** nén thành `FILE.gz` và **xoá file gốc**. Log văn bản nén rất tốt: 23 KB còn 4,5 KB.
- **`gzip -l`** cho biết kích thước trước và sau nén, tỉ lệ nén, mà không cần giải nén.
- **`gunzip FILE.gz`** (hoặc `gzip -d`) giải nén và xoá file `.gz`.
- **`gzip -k`** (*keep*) giữ lại file gốc. Khi nén log đang được dịch vụ dùng, hay khi bạn chưa chắc, `-k` an toàn hơn.

Để ý: `gzip` giữ thời điểm sửa của file gốc cho file nén, nên `ls -l` vẫn hiện giờ cũ.

`gzip -1` nén nhanh nhất, `gzip -9` nén nhỏ nhất; mặc định là `-6`. Với log, `-6` thường đủ.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tar-gzip-nen-luu-tru/ssh-studio-01.webp" alt="Terminal của SSH Studio trong ~/sao-luu: chép bao-gia.log.1, gzip nén từ 23353 byte còn 4570 byte và xoá bản gốc, gzip -l cho tỉ lệ 80,6%, gunzip giải nén, gzip -k giữ cả hai file" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>gzip</code>, <code>gzip -l</code>, <code>gunzip</code> và <code>gzip -k</code></figcaption></figure>

## 2. gzip, xz hay zstd?

```text
an@toinoi-app-01:~/sao-luu$ xz -k bao-gia.log.1
an@toinoi-app-01:~/sao-luu$ zstd -q -k bao-gia.log.1
an@toinoi-app-01:~/sao-luu$ ls -l bao-gia.log.1*
-rw-r--r-- 1 an an 23353 Nov 22 23:21 bao-gia.log.1
-rw-r--r-- 1 an an  4570 Nov 22 23:21 bao-gia.log.1.gz
-rw-r--r-- 1 an an  3620 Nov 22 23:21 bao-gia.log.1.xz
-rw-r--r-- 1 an an  4817 Nov 22 23:21 bao-gia.log.1.zst
an@toinoi-app-01:~/sao-luu$ xzcat bao-gia.log.1.xz | wc -l
305
an@toinoi-app-01:~/sao-luu$ zstdcat bao-gia.log.1.zst | wc -l
305
an@toinoi-app-01:~/sao-luu$ rm bao-gia.log.1.gz bao-gia.log.1.xz bao-gia.log.1.zst
```

| Công cụ | Đuôi | Trên file này | Đặc điểm |
|---|---|---|---|
| `gzip` | `.gz` | 4570 byte | Có ở mọi nơi, nhanh, đủ tốt. Mặc định an toàn |
| `xz` | `.xz` | 3620 byte | Nhỏ nhất, nhưng nén chậm hơn nhiều và tốn RAM. Hợp với bản lưu trữ lâu dài, ít khi mở |
| `zstd` | `.zst` | 4817 byte | Nén và giải nén rất nhanh. Ở mức mặc định (`-3`) kích thước tương đương gzip; mức cao (`-19`) gần bằng xz |

Với file nhỏ như log của lab, khác biệt không đáng kể. Với log hàng chục GB mỗi ngày, `zstd` thường là lựa chọn tốt nhất cho việc nén hằng ngày vì tốc độ, còn `xz` dành cho bản lưu trữ dài hạn. Mỗi công cụ có lệnh đọc thẳng file nén: `zcat`, `xzcat`, `zstdcat`; và `-k` để giữ file gốc (`zstd` mặc định đã giữ).

Chọn gì thì chọn, **người khôi phục phải có công cụ đó**. Một bản sao lưu `.zst` trên một máy cũ không có `zstd` là một bản sao lưu không mở được đúng lúc cần. Khi không chắc, dùng gzip.

## 3. Đóng gói: `tar`

```text
an@toinoi-app-01:~/sao-luu$ tar -cf log.tar -C /var/log/toinoi bao-gia.log bao-gia.log.1 bao-gia.log.2.gz
an@toinoi-app-01:~/sao-luu$ tar -tvf log.tar
-rw-r--r-- baogia/toinoi 27260 2026-11-21 12:17 bao-gia.log
-rw-r--r-- baogia/toinoi 23353 2026-11-20 11:16 bao-gia.log.1
-rw-r--r-- baogia/toinoi  4314 2026-11-19 11:07 bao-gia.log.2.gz
an@toinoi-app-01:~/sao-luu$ tar -czf log-ba-ngay.tar.gz -C /var/log/toinoi .
an@toinoi-app-01:~/sao-luu$ tar --zstd -cf log-ba-ngay.tar.zst -C /var/log/toinoi .
an@toinoi-app-01:~/sao-luu$ tar -tzf log-ba-ngay.tar.gz
./
./bao-gia.log
./bao-gia.log.2.gz
./bao-gia.log.1
an@toinoi-app-01:~/sao-luu$ ls -l
total 116
-rw-r--r-- 1 an an 23353 Nov 22 23:21 bao-gia.log.1
-rw-rw-r-- 1 an an 15111 Nov 22 23:21 log-ba-ngay.tar.gz
-rw-rw-r-- 1 an an 15622 Nov 22 23:21 log-ba-ngay.tar.zst
-rw-rw-r-- 1 an an 61440 Nov 22 23:21 log.tar
```

`tar` dùng một chữ cái cho **việc chính** và các chữ khác cho tuỳ chọn:

| Tuỳ chọn | Ý nghĩa |
|---|---|
| `-c` | Tạo (*create*) gói mới |
| `-t` | Liệt kê nội dung |
| `-x` | Giải nén (*extract*) |
| `-f FILE` | Tên file gói. Luôn có, và đứng ngay trước tên file |
| `-v` | In chi tiết từng file |
| `-z` / `-J` / `--zstd` | Nén thêm bằng gzip / xz / zstd |
| `-C THƯ_MỤC` | Chuyển vào thư mục đó trước khi làm việc |

Các lệnh trên:

1. **`tar -cf log.tar -C /var/log/toinoi …`**: gom ba file vào `log.tar`. Nhờ `-C`, đường dẫn trong gói là `bao-gia.log` chứ không phải `/var/log/toinoi/bao-gia.log`. **Đường dẫn trong gói nên là đường dẫn tương đối**: khi giải nén, bạn quyết định đặt chúng vào đâu. (GNU tar cũng tự bỏ dấu `/` đầu và báo `Removing leading '/' from member names`.)
2. **`tar -tvf log.tar`**: liệt kê, kèm quyền, chủ sở hữu, kích thước, thời điểm. `tar` lưu đủ những thông tin đó.
3. **`tar -czf … .`**: gói **cả thư mục** (dấu `.` là thư mục hiện tại sau `-C`) và nén bằng gzip. Kết quả có cả mục `./`, chính thư mục.
4. **`tar --zstd -cf …`**: cùng việc đó, nén bằng zstd.

`log.tar` chưa nén lớn hơn tổng ba file (60 KB): `tar` thêm phần đầu cho mỗi file và làm tròn theo khối 512 byte. Nén lại, cả gói còn 15 KB.

Thứ tự các file trong gói là thứ tự `tar` gặp chúng trên đĩa, không phải thứ tự chữ cái. Lab của bạn có thể ra thứ tự khác.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tar-gzip-nen-luu-tru/ssh-studio-02.webp" alt="Terminal của SSH Studio: so sánh kích thước bao-gia.log.1 nén bằng gzip, xz, zstd; xzcat và zstdcat đếm 305 dòng; tar -cf gom ba file, tar -tvf liệt kê kèm chủ sở hữu baogia/toinoi; tar -czf và tar --zstd gói cả thư mục; ls -l bốn file" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · ba cách nén và đóng gói bằng <code>tar</code></figcaption></figure>

## 4. Giải nén và kiểm tra

Một bản sao lưu chưa từng được thử khôi phục thì chưa phải bản sao lưu.

```text
an@toinoi-app-01:~/sao-luu$ mkdir thu-giai-nen
an@toinoi-app-01:~/sao-luu$ tar -xzf log-ba-ngay.tar.gz -C thu-giai-nen
an@toinoi-app-01:~/sao-luu$ ls -l thu-giai-nen
total 60
-rw-r--r-- 1 an an 27260 Nov 21 12:17 bao-gia.log
-rw-r--r-- 1 an an 23353 Nov 20 11:16 bao-gia.log.1
-rw-r--r-- 1 an an  4314 Nov 19 11:07 bao-gia.log.2.gz
an@toinoi-app-01:~/sao-luu$ diff -r /var/log/toinoi thu-giai-nen && echo "giống hệt bản gốc"
giống hệt bản gốc
an@toinoi-app-01:~/sao-luu$ rm -r thu-giai-nen log.tar bao-gia.log.1
an@toinoi-app-01:~/sao-luu$ mkdir thu-giai-nen
an@toinoi-app-01:~/sao-luu$ tar -xzf log-ba-ngay.tar.gz -C thu-giai-nen ./bao-gia.log.1
an@toinoi-app-01:~/sao-luu$ ls thu-giai-nen
bao-gia.log.1
an@toinoi-app-01:~/sao-luu$ zgrep -c ' ERROR ' /var/log/toinoi/bao-gia.log.2.gz
18
```

- **`tar -xzf GÓI -C THƯ_MỤC`**: giải nén vào một thư mục **khác**. Luôn giải nén vào thư mục riêng để thử: `tar -x` sẽ ghi đè file trùng tên ở chỗ giải nén mà không hỏi.
- Thời điểm sửa được giữ nguyên (21, 20, 19/11). Chủ sở hữu thì không: file thuộc về `an`, người giải nén. Chỉ `root` khôi phục được chủ sở hữu gốc.
- **`diff -r`** so hai thư mục từng file một. Không in gì và mã thoát 0: bản giải nén giống hệt bản gốc.
- **Lấy riêng một file**: ghi tên file **đúng như trong gói**, ở đây là `./bao-gia.log.1` (có `./` vì gói được tạo từ `.`). Không chắc tên, chạy `tar -tzf` trước.
- **`zgrep`** tìm thẳng trong file nén.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tar-gzip-nen-luu-tru/ssh-studio-03.webp" alt="Terminal của SSH Studio: tar -xzf giải nén vào thu-giai-nen với thời điểm sửa giữ nguyên, diff -r báo giống hệt bản gốc, giải nén riêng ./bao-gia.log.1, và zgrep đếm 18 lỗi trong bao-gia.log.2.gz" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · giải nén, so bằng <code>diff -r</code>, lấy riêng một file</figcaption></figure>

## 5. Tải bản sao lưu về máy

Bản sao lưu nằm trên chính server thì không giúp được gì khi server hỏng. Cách nhanh nhất để đưa nó ra ngoài là khung **Files** của SSH Studio: chuyển sang chế độ Files, trỏ khung Remote server vào `/home/an/sao-luu`, chọn `log-ba-ngay.tar.gz`, bấm **Download**:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tar-gzip-nen-luu-tru/ssh-studio-04.webp" alt="Chế độ Files của SSH Studio: khung Remote server ở /home/an/sao-luu với log-ba-ngay.tar.gz đang được chọn, khung Máy của bạn đã có log-ba-ngay.tar.gz 14,8 KB, bảng Truyền file ở dưới báo tải về hoàn tất" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tải bản sao lưu về máy bằng <b>Download</b></figcaption></figure>

Tải về xong, bạn có thể mở file `.tar.gz` bằng công cụ giải nén của máy mình để kiểm tra. Một file 14,8 KB như ở đây tải tức thì; với bản sao lưu lớn, bảng Truyền file hiện tiến độ và tiếp tục được nếu mất kết nối giữa chừng.

Tải về bằng tay phù hợp cho việc thỉnh thoảng. Sao lưu đều đặn thì cần tự động: script dưới đây, cron ở [LNX-020](/articles/linux-thuc-hanh-script-sao-luu-log), và các cách đẩy bản sao lưu sang máy khác bằng `rsync` ở Phần 5.

## 6. Script sao lưu đầu tiên

```bash
#!/usr/bin/env bash
# sao-luu-log: đóng gói log của bao-gia vào ~/sao-luu/log-NGÀY.tar.gz
set -euo pipefail

nguon=/var/log/toinoi
dich=$HOME/sao-luu
file=$dich/log-$(date +%F).tar.gz

mkdir -p "$dich"
tar -czf "$file" -C "$nguon" .
echo "Đã sao lưu $(tar -tzf "$file" | grep -vc '/$') file vào $file"
```

```text
an@toinoi-app-01:~/sao-luu$ vim ~/bin/sao-luu-log
an@toinoi-app-01:~/sao-luu$ chmod u+x ~/bin/sao-luu-log
an@toinoi-app-01:~/sao-luu$ sao-luu-log
Đã sao lưu 3 file vào /home/an/sao-luu/log-2026-11-22.tar.gz
an@toinoi-app-01:~/sao-luu$ ls ~/sao-luu
log-2026-11-22.tar.gz  log-ba-ngay.tar.gz  log-ba-ngay.tar.zst  thu-giai-nen
```

- `$(date +%F)` đặt ngày hôm nay vào tên file, nên mỗi ngày một file. Ngày trong kết quả trên là ngày chạy lab; của bạn sẽ khác.
- `mkdir -p` không báo lỗi nếu thư mục đã có, nên script chạy lại bao nhiêu lần cũng được.
- Dòng cuối kiểm tra ngay gói vừa tạo: liệt kê lại và đếm các mục không phải thư mục (`grep -v '/$'` bỏ dòng kết thúc bằng `/`).

Script này chạy được, nhưng có ba điểm yếu mà [bài sau](/articles/linux-shell-script-an-toan) sẽ sửa: nếu `tar` hỏng giữa chừng thì một file dở dang mang tên hợp lệ nằm lại; nếu hai lần chạy chồng nhau thì cả hai cùng ghi một file; và các bản sao lưu cứ tích dần mà không bao giờ được dọn.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-tar-gzip-nen-luu-tru/ssh-studio-05.webp" alt="Terminal của SSH Studio: tạo ~/bin/sao-luu-log bằng vim, chmod u+x, chạy sao-luu-log báo đã sao lưu 3 file vào log-NGÀY.tar.gz, và ls ~/sao-luu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sao-luu-log</code> phiên bản đầu</figcaption></figure>

## 7. Tóm tắt

| Việc | Lệnh |
|---|---|
| Nén / giải nén một file | `gzip -k f`, `gunzip f.gz` |
| Đọc file nén | `zcat`, `zless`, `zgrep`; `xzcat`; `zstdcat` |
| Tạo gói nén | `tar -czf goi.tar.gz -C thu-muc .` |
| Liệt kê | `tar -tzf goi.tar.gz` (thêm `-v` để có chi tiết) |
| Giải nén toàn bộ | `tar -xzf goi.tar.gz -C thu-muc-dich` |
| Giải nén một file | `tar -xzf goi.tar.gz -C dich ./ten-file` |
| Kiểm tra | `diff -r goc ban-giai-nen`, `tar -tzf … > /dev/null && echo ok` |

## 8. Lỗi thường gặp

**`tar -cf` với đường dẫn tuyệt đối.** Gói chứa `var/log/toinoi/…`; khi giải nén ra một đống thư mục lồng nhau. Dùng `-C`.

**Đảo thứ tự `-f`.** `tar -cfz goi.tgz …` coi `z` là tên file. `-f` phải đứng ngay trước tên gói.

**Giải nén thẳng vào thư mục đang dùng.** File trùng tên bị ghi đè. Giải nén vào thư mục thử trước.

**Nén file đang được ghi.** Dịch vụ vẫn ghi vào file cũ đã bị `gzip` xoá. Nén file đã xoay vòng (`.1`), không nén file đang ghi; logrotate lo chuyện này (Phần 4).

**Chỉ sao lưu mà không thử khôi phục.** Một file hỏng chỉ lộ ra khi bạn cần nó nhất.

**`tar` báo `file changed as we read it`.** Một file thay đổi trong lúc đóng gói, thường là log đang được ghi. Gói vẫn được tạo, nhưng file đó có thể không trọn vẹn; mã thoát là 1.

## Bài tập

1. So kích thước `bao-gia.log` khi nén bằng `gzip -1`, `gzip -9`, `xz -9`, `zstd -19`. Đo thời gian bằng `time`.
2. Tạo `log.tar.xz` bằng `tar -cJf`. Liệt kê nó bằng `tar -tJf`. Thử `tar -tf` không có `J`: GNU tar có tự nhận ra không?
3. Giải nén `log-ba-ngay.tar.zst` vào một thư mục mới và so với bản gốc bằng `diff -r`.
4. Sửa `sao-luu-log` để in thêm kích thước gói vừa tạo (`du -h`).
5. Chạy `sao-luu-log` hai lần liền. File của lần đầu bị gì? Đây là một trong ba điểm yếu ở mục 6.

## Đọc tiếp

[LNX-019 · Script an toàn](/articles/linux-shell-script-an-toan): dấu nháy, `set -u`, `trap`, `mktemp`, `flock` và `shellcheck`, rồi viết lại `sao-luu-log`.
