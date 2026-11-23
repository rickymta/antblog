---
slug: linux-thuc-hanh-script-sao-luu-log
title: "Thực hành Linux: báo cáo và sao lưu log dịch vụ báo giá hằng ngày bằng cron"
summary: "Bài thực hành cuối Phần 2: đưa bao-cao-log, sao-luu-log và tom-tat.awk vào thư mục scripts dùng chung, viết script hang-ngay ghép báo cáo và sao lưu, chạy thử với môi trường trống của cron bằng env -i và gặp hai lỗi thật (HOME chưa gán, cột lệch vì thiếu LANG), lên lịch bằng crontab, xác nhận cron đã chạy, rồi chuyển sang lịch 01:17 mỗi ngày. Bàn giao có bộ chấm 20 yêu cầu."
category: devops-linux
level: advanced
tags: [linux, lnx-020, thuc-hanh, cron, crontab, shell-script, backup, ssh-studio, toi-noi]
language: vi
---

Phần 2 đã cho An ba công cụ: `bao-cao-log` tóm tắt từng file log, `tom-tat.awk` tóm tắt theo ngày, `sao-luu-log` đóng gói log và giữ N bản mới nhất. Cả ba đang nằm trong `~/bin` của An, chạy khi An nhớ gõ.

Bình giao việc cuối phần: «Mỗi đêm, server tự tạo báo cáo log và tự sao lưu. Báo cáo để trong thư mục log của dịch vụ cho cả nhóm đọc, bản sao lưu để ở `/srv/toinoi/sao-luu`. Script đặt ở chỗ dùng chung chứ không phải trong home của bạn.» Nghe như chép file rồi thêm một dòng cron. Thực tế, cron chạy script trong một môi trường rất khác terminal của bạn, và đó là nơi phần lớn script «chạy được trên máy tôi» gãy.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-020 · Phần 2/6 · Nâng cao · Thực hành</span>
<p>Đọc trước: LNX-011 tới <a href="/articles/linux-shell-script-an-toan">LNX-019</a>. Trạng thái đầu bài: <code>npm run setup -- 20</code>, rồi kết nối lại bằng SSH Studio. Sản phẩm: <code>npm run kiem-tra:p02</code> cho <code>20 PASS, 0 FAIL</code>. Hãy tự làm theo đề ở mục 1 trước khi đọc lời giải từ mục 2.</p>
</div>

## 1. Đề bài

1. Thư mục `/srv/toinoi/bao-gia/scripts`, chủ `an`, nhóm `toinoi`, quyền `750`, chứa:
   - `bao-cao-log`, `sao-luu-log` (phiên bản 2 của LNX-017 và LNX-019), quyền `750`;
   - `tom-tat.awk` của LNX-015, quyền `640`;
   - `hang-ngay`, quyền `750`: ghi báo cáo vào `/srv/toinoi/bao-gia/shared/logs/bao-cao-NGÀY.txt` rồi sao lưu log vào `/srv/toinoi/sao-luu`, giữ 7 bản.
2. Cả ba script qua `shellcheck` không cảnh báo.
3. `hang-ngay` chạy đúng trong môi trường của cron.
4. Crontab của `an` chạy `hang-ngay` lúc **01:17 mỗi ngày**, ghi kết quả và lỗi vào `shared/logs/hang-ngay.log`.
5. Có bằng chứng cron đã thực sự chạy `hang-ngay` ít nhất một lần.

## 2. Đưa script vào thư mục dùng chung

```text
an@toinoi-app-01:~$ mkdir -p /srv/toinoi/bao-gia/scripts
an@toinoi-app-01:~$ cp ~/bin/bao-cao-log ~/bin/sao-luu-log ~/bin/tom-tat.awk /srv/toinoi/bao-gia/scripts/
```

Rồi viết script ghép:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-script-sao-luu-log/ssh-studio-01.webp" alt="vim mở /srv/toinoi/bao-gia/scripts/hang-ngay: shebang, chú thích về môi trường của cron, set -euo pipefail, PATH cố định, export LANG=C.UTF-8, biến goc, ngay và bao_cao, khối ngoặc nhọn gom đầu ra của bao-cao-log và tom-tat.awk vào file báo cáo, rồi gọi sao-luu-log với -d /srv/toinoi/sao-luu -g 7" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · script <code>hang-ngay</code> trong vim</figcaption></figure>

```bash
#!/usr/bin/env bash
# hang-ngay: việc chạy mỗi ngày của bao-gia: báo cáo log rồi sao lưu log.
# cron chạy script với môi trường tối giản, nên script tự đặt PATH và chỉ dùng đường dẫn tuyệt đối.
set -euo pipefail
PATH=/usr/local/bin:/usr/bin:/bin
export LANG=C.UTF-8   # để awk đếm đúng chữ có dấu khi căn cột

goc=/srv/toinoi/bao-gia
ngay=${NGAY:-$(date +%F)}
bao_cao=$goc/shared/logs/bao-cao-$ngay.txt

{
  echo "Báo cáo log bao-gia ngày $ngay"
  echo
  "$goc/scripts/bao-cao-log" /var/log/toinoi/bao-gia.log*
  echo
  zcat -f /var/log/toinoi/bao-gia.log* | awk -f "$goc/scripts/tom-tat.awk"
} > "$bao_cao"

NGAY=$ngay "$goc/scripts/sao-luu-log" -d /srv/toinoi/sao-luu -g 7
echo "hang-ngay: xong $ngay, báo cáo ở $bao_cao"
```

- **`{ …; } > file`** gom đầu ra của nhiều lệnh vào một lần chuyển hướng. Gọn hơn và an toàn hơn năm lần `>>`.
- **Đường dẫn tuyệt đối** cho mọi script được gọi, vì cron không có `~/bin` trong `PATH`.
- **`PATH=…` và `LANG=…`** ở đầu: mục 4 giải thích vì sao từng dòng.
- `NGAY=$ngay` truyền cùng ngày cho `sao-luu-log`, để báo cáo và bản sao lưu luôn mang một ngày, kể cả khi chạy qua nửa đêm.

```text
an@toinoi-app-01:~$ cd /srv/toinoi/bao-gia/scripts
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ chgrp toinoi . bao-cao-log sao-luu-log tom-tat.awk hang-ngay
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ chmod 750 . bao-cao-log sao-luu-log hang-ngay
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ chmod 640 tom-tat.awk
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ ls -la
total 24
drwxr-x--- 2 an toinoi 4096 Nov 22 23:21 .
drwxr-xr-x 5 an toinoi 4096 Nov 22 23:21 ..
-rwxr-x--- 1 an toinoi  913 Nov 22 23:21 bao-cao-log
-rwxr-x--- 1 an toinoi  794 Nov 22 23:21 hang-ngay
-rwxr-x--- 1 an toinoi 1515 Nov 22 23:21 sao-luu-log
-rw-r----- 1 an toinoi  686 Nov 22 23:21 tom-tat.awk
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ shellcheck bao-cao-log sao-luu-log hang-ngay && echo "shellcheck: sạch"
shellcheck: sạch
```

`chgrp toinoi .` đổi nhóm của chính thư mục; `chmod 750 .` cho nhóm vào và chạy script, người ngoài không thấy gì. `tom-tat.awk` là dữ liệu cho `awk` đọc, không cần quyền chạy. Không cần `sudo` ở bước nào: `/srv/toinoi/bao-gia` thuộc về `an` từ [LNX-010](/articles/linux-thuc-hanh-thu-muc-trien-khai).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-script-sao-luu-log/ssh-studio-02.webp" alt="Terminal của SSH Studio: tạo thư mục scripts, chép ba file, mở hang-ngay bằng vim, chgrp toinoi và chmod 750, 640, ls -la cho thấy thư mục drwxr-x--- an toinoi và bốn file với quyền đúng, shellcheck ba script sạch" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · thư mục <code>scripts</code> với nhóm, quyền đúng và <code>shellcheck</code> sạch</figcaption></figure>

## 3. Chạy thử trong môi trường của cron

Khi cron chạy một việc, nó **không** đọc `~/.profile`, `~/.bashrc`; không có alias, không có `~/bin` trong `PATH`, không có `LANG`. Môi trường chỉ gồm vài biến tối thiểu: `HOME`, `LOGNAME`, `SHELL=/bin/sh`, `PATH=/usr/bin:/bin`. Script nào dựa vào thứ gì khác đều có thể gãy lúc 01:17, khi không ai nhìn.

**`env -i`** chạy một lệnh với môi trường **trống hoàn toàn**: cách rẻ nhất để thử script trước khi giao cho cron.

```text
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ env -i /usr/bin/env
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ env -i NGAY=2026-11-21 /srv/toinoi/bao-gia/scripts/hang-ngay
/srv/toinoi/bao-gia/scripts/sao-luu-log: line 8: HOME: unbound variable
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ echo $?
1
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ env -i HOME=/home/an NGAY=2026-11-21 /srv/toinoi/bao-gia/scripts/hang-ngay
sao-luu-log: đã ghi /srv/toinoi/sao-luu/log-2026-11-21.tar.gz
hang-ngay: xong 2026-11-21, báo cáo ở /srv/toinoi/bao-gia/shared/logs/bao-cao-2026-11-21.txt
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ cat /srv/toinoi/bao-gia/shared/logs/bao-cao-2026-11-21.txt
Báo cáo log bao-gia ngày 2026-11-21

/var/log/toinoi/bao-gia.log        362 dòng   17 lỗi   24 chậm
/var/log/toinoi/bao-gia.log.1      305 dòng   26 lỗi   19 chậm
/var/log/toinoi/bao-gia.log.2.gz   302 dòng   18 lỗi   25 chậm

ngày        yêu cầu   lỗi  chậm   tb ms
2026-11-19      300    18    25    98.7
2026-11-20      300    26    19   101.2
2026-11-21      360    17    24    95.3
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ ls -l /srv/toinoi/sao-luu
total 16
-rw-r----- 1 an an 15111 Nov 22 23:21 log-2026-11-21.tar.gz
```

Đọc từ trên xuống:

1. `env -i /usr/bin/env` không in gì: môi trường thật sự trống.
2. Lần chạy đầu **hỏng**: `sao-luu-log: line 8: HOME: unbound variable`. Dòng 8 của `sao-luu-log` là `dich=$HOME/sao-luu`, giá trị mặc định của thư mục đích. `hang-ngay` có truyền `-d`, nhưng dòng gán mặc định chạy **trước** `getopts`, và `set -u` không cho dùng biến chưa có. Chính `set -u` đã biến một sự phụ thuộc ngầm thành một lỗi rõ ràng.
3. `env -i` khắt khe hơn cron: cron **có** đặt `HOME`. Lần chạy thứ hai thêm `HOME=/home/an`, gần với cron hơn, và chạy được.
4. File báo cáo: các cột tiếng Việt thẳng hàng.

Điểm 4 không tự nhiên mà có. Bản đầu của `hang-ngay` thiếu dòng `export LANG=C.UTF-8`, và trong môi trường trống, bảng của `tom-tat.awk` ra như sau (chép từ lần chạy thử lúc soạn bài):

```text
ngày      yêu cầu lỗi chậm   tb ms
2026-11-19      300    18    25    98.7
```

Không có `LANG`, `awk` làm việc ở chế độ byte: chữ `ngày` dài 6 byte nên `%-10s` chỉ thêm 4 dấu cách thay vì 6, và các tiêu đề có dấu lệch khỏi cột số. Dữ liệu đúng, chỉ trình bày sai, đúng loại lỗi chỉ lộ ra trong môi trường của cron. Đặt `LANG` ngay trong script thay vì trông vào môi trường của người chạy.

`ls -l /srv/toinoi/sao-luu`: bản sao lưu đầu tiên, quyền `640` như `sao-luu-log` đặt.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-script-sao-luu-log/ssh-studio-03.webp" alt="Terminal của SSH Studio: env -i /usr/bin/env không in gì, chạy hang-ngay với env -i hỏng vì HOME unbound variable và trả mã 1, chạy lại với HOME=/home/an thành công, cat báo cáo ngày 21/11 với ba dòng của bao-cao-log và bảng tom-tat thẳng cột, ls -l thư mục sao-luu có log-2026-11-21.tar.gz quyền rw-r-----" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · thử <code>hang-ngay</code> trong môi trường trống bằng <code>env -i</code></figcaption></figure>

## 4. Lên lịch bằng cron

### Cú pháp crontab

```text
┌───────── phút        (0–59)
│ ┌─────── giờ         (0–23)
│ │ ┌───── ngày        (1–31)
│ │ │ ┌─── tháng       (1–12)
│ │ │ │ ┌─ thứ         (0–7, 0 và 7 đều là Chủ nhật)
│ │ │ │ │
17 1 * * *  lệnh
```

| Biểu thức | Nghĩa |
|---|---|
| `17 1 * * *` | 01:17 mỗi ngày |
| `*/15 * * * *` | Mỗi 15 phút |
| `0 9 * * 1-5` | 09:00 thứ Hai tới thứ Sáu |
| `30 2 1 * *` | 02:30 ngày 1 hằng tháng |
| `* * * * *` | Mỗi phút: chỉ dùng để thử |

Chọn phút lẻ như 17 thay cho 0: rất nhiều việc trên một server (và trên mọi server trong cùng hệ thống) được hẹn đúng 00:00 hay 01:00, và chạy cùng lúc thì tranh tài nguyên.

### Cài crontab từ một file

`crontab -e` mở trình soạn thảo để sửa trực tiếp. Lần đầu trên Ubuntu, nó hỏi chọn trình soạn thảo; muốn đi thẳng vào vim, đặt `EDITOR=vim` ([LNX-011](/articles/linux-bien-moi-truong-path-bashrc)). Cách bài này dùng là viết crontab vào một file rồi cài file đó: file có thể đưa vào Git, xem lại, `diff` được.

```text
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ crontab -l
no crontab for an
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ vim ~/hang-ngay.cron
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ crontab ~/hang-ngay.cron
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ crontab -l
MAILTO=""
* * * * * /srv/toinoi/bao-gia/scripts/hang-ngay >> /srv/toinoi/bao-gia/shared/logs/hang-ngay.log 2>&1
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ cat /srv/toinoi/bao-gia/shared/logs/hang-ngay.log
sao-luu-log: đã ghi /srv/toinoi/sao-luu/log-2026-11-22.tar.gz
hang-ngay: xong 2026-11-22, báo cáo ở /srv/toinoi/bao-gia/shared/logs/bao-cao-2026-11-22.txt
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ ls /srv/toinoi/sao-luu
log-2026-11-21.tar.gz  log-2026-11-22.tar.gz
```

- `crontab -l` lúc đầu: `no crontab for an`.
- File `~/hang-ngay.cron` có ba dòng: một chú thích, `MAILTO=""` (cron mặc định gửi mọi đầu ra qua thư nội bộ; tắt đi vì đầu ra đã được ghi vào file), và lịch **mỗi phút** để thử.
- `crontab FILE` thay toàn bộ crontab của bạn bằng nội dung file. `crontab -l` kiểm tra lại.
- **`>> …/hang-ngay.log 2>&1`**: nối cả kết quả lẫn lỗi vào một file log. Không có dòng này, khi script hỏng lúc 01:17 bạn sẽ không bao giờ biết vì sao.

Sau khoảng một phút (lab chờ 65 giây), `hang-ngay.log` có dòng `hang-ngay: xong …`: **cron đã thực sự chạy script**, bằng môi trường của chính nó, và thành công. Thư mục sao lưu có thêm bản của ngày hôm nay. Ngày trong kết quả là ngày chạy lab; của bạn sẽ khác, và tuỳ lúc bạn chờ, log có thể có một hoặc hai lần chạy.

Muốn xem cron đã khởi chạy những gì, kể cả khi script không ghi log: `journalctl -u cron` (Phần 4).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-script-sao-luu-log/ssh-studio-04.webp" alt="Terminal của SSH Studio: crontab -l báo no crontab for an, vim tạo hang-ngay.cron, crontab cài file, crontab -l in ba dòng với lịch mỗi phút, sau khi chờ thì cat hang-ngay.log cho thấy cron đã chạy hang-ngay và ghi bản sao lưu ngày hôm nay, ls thư mục sao-luu có hai bản" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cron chạy thử mỗi phút và để lại bằng chứng trong <code>hang-ngay.log</code></figcaption></figure>

### Chuyển sang lịch thật

```text
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ sed -i -e 's/^[*] [*] [*] [*] [*]/17 1 * * */' -e 's/Đang chạy thử mỗi phút/Chạy lúc 01:17 mỗi ngày/' ~/hang-ngay.cron
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ crontab ~/hang-ngay.cron
an@toinoi-app-01:/srv/toinoi/bao-gia/scripts$ crontab -l
MAILTO=""
17 1 * * * /srv/toinoi/bao-gia/scripts/hang-ngay >> /srv/toinoi/bao-gia/shared/logs/hang-ngay.log 2>&1
```

`sed` của [LNX-014](/articles/linux-sed-sua-van-ban) sửa cả lịch lẫn chú thích. `[*]` là một lớp ký tự chỉ chứa dấu `*`, cách viết dấu sao thật mà không cần `\`. Cài lại và kiểm tra: từ giờ `hang-ngay` chạy lúc 01:17 mỗi đêm.

Đừng quên bước này. Một việc «chạy thử mỗi phút» bị bỏ quên sẽ tạo 1.440 báo cáo mỗi ngày (và `sao-luu-log` sẽ giữ đúng 7 bản, nhưng ghi đĩa liên tục).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-thuc-hanh-script-sao-luu-log/ssh-studio-05.webp" alt="Terminal của SSH Studio: sed đổi lịch mỗi phút thành 17 1 * * * và sửa chú thích, crontab cài lại file, crontab -l in lịch 01:17 mỗi ngày" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · lịch thật: 01:17 mỗi ngày</figcaption></figure>

## 5. Chấm tự động

Trên máy của bạn, trong `examples/toi-noi-linux`:

```sh
npm run kiem-tra:p02
```

```text
PASS  scripts có quyền và chủ sở hữu 750 an:toinoi
PASS  scripts/bao-cao-log có quyền và chủ sở hữu 750 an:toinoi
PASS  scripts/sao-luu-log có quyền và chủ sở hữu 750 an:toinoi
PASS  scripts/hang-ngay có quyền và chủ sở hữu 750 an:toinoi
PASS  scripts/tom-tat.awk có quyền và chủ sở hữu 640 an:toinoi
PASS  shellcheck không cảnh báo gì với ba script
PASS  bao-cao-log đếm đúng 362 dòng, 17 lỗi, 24 chậm cho bao-gia.log
PASS  bao-cao-log đọc được file .gz (302 dòng, 18 lỗi)
PASS  bao-cao-log trả mã 1 và báo lỗi ra stderr khi có file không đọc được
PASS  bao-cao-log không có đối số thì trả mã 2
PASS  tom-tat.awk cho ba ngày, ngày 21/11 có 360 yêu cầu, tb 95.3 ms
PASS  sao-luu-log -g 2 chỉ giữ hai bản mới nhất
PASS  sao-luu-log không để lại file tạm
PASS  bản sao lưu có quyền 640
PASS  bản sao lưu chứa đủ ba file log
PASS  sao-luu-log báo đã ghi file
PASS  sao-luu-log trả mã 1 khi đang có bản khác giữ khoá
PASS  sao-luu-log -g abc bị từ chối với mã 2
PASS  crontab của an chạy hang-ngay lúc 01:17 mỗi ngày
PASS  cron đã chạy hang-ngay ít nhất một lần thành công

20 PASS, 0 FAIL
```

Bộ chấm không chỉ đọc file: nó chạy các script dưới tài khoản `an` trên dữ liệu của lab, thử đầu vào sai, giữ khoá bằng `flock` để giả lập một lần chạy chồng, đếm bản sao lưu còn lại sau `-g 2`, và đọc crontab cùng `hang-ngay.log`. Bộ chấm làm việc trong `/tmp/kiem-tra-p02` và dọn sau khi xong. Dòng `FAIL` kèm giá trị thực tế để bạn biết sửa ở đâu.

Ảnh chụp của Phần 2 được tạo bằng cách cho SSH Studio gõ đúng các lệnh của từng bài, kể cả gõ từng script bằng vim; sau lượt đó, bộ chấm này cho 20 PASS và nội dung mọi file gõ bằng vim trùng từng byte với file mẫu của lab.

## 6. Lỗi thường gặp trong bài này

**Script chạy tay được, cron thì không.** Gần như luôn là môi trường: `PATH`, `HOME`, `LANG`, alias, thư mục hiện tại (cron chạy ở thư mục riêng của bạn). Thử bằng `env -i HOME=$HOME script`, và đọc file log mà dòng cron chuyển hướng tới.

**Không chuyển hướng đầu ra trong dòng cron.** Lỗi biến mất (hoặc nằm trong hộp thư nội bộ không ai đọc).

**`%` trong dòng cron.** Trong crontab, `%` có nghĩa đặc biệt (xuống dòng). `date +%F` viết thẳng trong dòng cron phải thành `date +\%F`; dễ nhất là để logic trong script, như `hang-ngay`.

**`crontab FILE` ghi đè crontab cũ.** Nếu đã có việc khác, `crontab -l > crontab.cu` trước, gộp, rồi mới cài.

**Quên đổi lịch thử mỗi phút về lịch thật.** Mục 4.

**Script trong `~/bin`.** Người khác trong nhóm không chạy được, và nếu tài khoản của bạn bị khoá thì việc hằng ngày dừng theo. Thư mục dùng chung có nhóm đúng là chỗ của script vận hành.

## 7. Bạn đã có gì sau Phần 2

| Kỹ năng | Bài |
|---|---|
| Biến, `export`, `PATH`, `~/.profile`, `~/.bashrc`, alias | LNX-011 |
| Sửa nhanh bằng vim: toán tử, `:s`, `:%s`, `:!`, `~/.vimrc` | LNX-012 |
| Log nén, `cut`, `tr`, `sort -k`, `uniq -c`, `column` | LNX-013 |
| `sed`: địa chỉ, thay thế, nhóm, `-i.bak` | LNX-014 |
| `awk`: điều kiện, tổng, trung bình, mảng, chương trình trong file | LNX-015 |
| Script: shebang, tham số, mã thoát, `set -euo pipefail`, `bash -x` | LNX-016 |
| `if`, `for`, `while read`, `case`, hàm | LNX-017 |
| `gzip`, `xz`, `zstd`, `tar`, kiểm tra bản sao lưu | LNX-018 |
| Nháy kép, `set -u`, `trap`, `mktemp`, `flock`, `shellcheck` | LNX-019 |
| Thư mục script dùng chung, môi trường của cron, `crontab` | LNX-020 |

Phần 3 chuyển sang an toàn của chính server: quản lý người dùng, đăng nhập bằng SSH key tạo trong SSH Studio, tắt mật khẩu, cấp `sudo` theo đúng việc, và một tài khoản triển khai riêng thay cho tài khoản của An. Bắt đầu với [LNX-021 · Quản lý người dùng và nhóm](/articles/linux-quan-ly-nguoi-dung-nhom).

## Bài tập mở rộng

1. Thêm vào `hang-ngay` một dòng in tổng số lỗi của ba ngày vào cuối báo cáo, dùng `awk` trên đầu ra của `bao-cao-log`.
2. Thêm lịch thứ hai vào `~/hang-ngay.cron`: chạy `bao-cao-log /var/log/toinoi/bao-gia.log` lúc 12:00 các ngày làm việc, ghi vào `shared/logs/giua-ngay.log`. Cài lại và kiểm tra bằng `crontab -l`.
3. `binh` thuộc nhóm `toinoi`. Kiểm tra bằng `sudo -u binh` xem Bình có chạy được `hang-ngay` không. Báo cáo mới thuộc về ai?
4. Làm `hang-ngay` thất bại có chủ đích (đổi tạm tên `tom-tat.awk`), chạy bằng `env -i HOME=/home/an`, xem mã thoát. Bản sao lưu có được tạo không? Vì sao điều đó đúng hoặc sai với ý bạn?
5. Xoá crontab bằng `crontab -r`, xoá `/srv/toinoi/bao-gia/scripts` và làm lại cả bài chỉ dựa vào đề ở mục 1 và `npm run kiem-tra:p02`.
