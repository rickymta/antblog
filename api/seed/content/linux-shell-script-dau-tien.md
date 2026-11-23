---
slug: linux-shell-script-dau-tien
title: "Shell script đầu tiên: shebang, tham số, mã thoát và gỡ lỗi bằng bash -x"
summary: "Gói pipeline đếm lỗi thành script bao-cao-log trong ~/bin: dòng shebang, quyền chạy, set -euo pipefail, tham số $1 với giá trị bắt buộc, kiểm tra file và trả mã thoát có nghĩa. Bài gặp thật một lỗi kinh điển: grep -c không tìm thấy gì làm script dừng im lặng vì set -e, rồi tìm ra bằng bash -x và sửa bằng sed."
category: devops-linux
level: intermediate
tags: [linux, lnx-016, bash, shell-script, shebang, exit-code, set-e, ssh-studio, toi-noi]
language: vi
---

Mỗi sáng, An gõ ba lệnh để biết log hôm qua có gì: đếm dòng, đếm lỗi, đếm yêu cầu chậm. Bình muốn mọi người trong nhóm làm được việc đó mà không cần nhớ lệnh. Câu trả lời là một **script**: một file văn bản chứa các lệnh, có tên, chạy như một lệnh thật.

Một script ba dòng thì ai cũng viết được. Một script mà người khác dám dùng thì cần thêm vài thứ: báo lỗi rõ khi dùng sai, trả mã thoát đúng để script khác dựa vào, và không lặng lẽ chạy tiếp khi một bước đã hỏng. Bài này viết phiên bản đầu của `bao-cao-log`, chạy nó trong vài tình huống, và gặp một lỗi mà hầu như ai viết bash cũng gặp một lần.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-016 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-bien-moi-truong-path-bashrc">LNX-011</a> (biến, <code>~/bin</code>, <code>PATH</code>), mã thoát ở <a href="/articles/linux-shell-dong-lenh-tro-giup">LNX-003</a>. Trạng thái đầu bài: <code>npm run setup -- 16</code>. Sau bài này, bạn có script <code>~/bin/bao-cao-log</code>, hiểu từng dòng của nó, chạy được với hoặc không có quyền thực thi, và biết dùng <code>bash -n</code>, <code>bash -x</code> để tìm lỗi.</p>
</div>

## 1. Viết script bằng vim

```sh
vim bin/bao-cao-log
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-script-dau-tien/ssh-studio-01.webp" alt="vim mở ~/bin/bao-cao-log phiên bản đầu: dòng shebang #!/usr/bin/env bash, chú thích cách dùng, set -euo pipefail, gán file từ $1 với thông báo khi thiếu, kiểm tra file đọc được, ba lệnh đếm bằng wc và grep -c, và bốn dòng echo in kết quả" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>~/bin/bao-cao-log</code> phiên bản đầu trong vim</figcaption></figure>

```bash
#!/usr/bin/env bash
# bao-cao-log: tóm tắt một file log của dịch vụ bao-gia.
# Dùng: bao-cao-log FILE
set -euo pipefail

file=${1:?Dùng: bao-cao-log FILE}

if [ ! -r "$file" ]; then
  echo "bao-cao-log: không đọc được $file" >&2
  exit 2
fi

tong=$(wc -l < "$file")
loi=$(grep -c ' ERROR ' "$file")
cham=$(grep -c ' WARN ' "$file")

echo "File   : $file"
echo "Số dòng: $tong"
echo "Lỗi    : $loi"
echo "Chậm   : $cham"
```

Đi từng phần:

| Dòng | Ý nghĩa |
|---|---|
| `#!/usr/bin/env bash` | **Shebang**: dòng đầu tiên, bắt đầu bằng `#!`, cho kernel biết dùng chương trình nào để chạy file. `/usr/bin/env bash` tìm `bash` trong `PATH`; cũng hay gặp `#!/bin/bash` |
| `# …` | Chú thích. Hai dòng đầu nói script làm gì và cách dùng: thứ người khác đọc đầu tiên |
| `set -euo pipefail` | Ba chế độ an toàn (mục 4) |
| `file=${1:?…}` | `$1` là **tham số thứ nhất**. `${1:?thông báo}`: nếu `$1` rỗng hoặc không có, in thông báo và dừng script |
| `if [ ! -r "$file" ]; then … fi` | Nếu file **không** đọc được thì báo lỗi và thoát. `[ … ]` và `if` là nội dung của [LNX-017](/articles/linux-bash-dieu-kien-vong-lap-ham) |
| `echo … >&2` | In ra **stderr**: thông báo lỗi đi đường lỗi ([LNX-008](/articles/linux-chuyen-huong-pipe)) |
| `exit 2` | Kết thúc với mã thoát 2 |
| `tong=$(…)` | Lưu kết quả lệnh vào biến ([LNX-011](/articles/linux-bien-moi-truong-path-bashrc)) |

Các biến tham số khác: `$2`, `$3`… là tham số tiếp theo; `$#` là số tham số; `"$@"` là mọi tham số, mỗi cái giữ nguyên là một chuỗi riêng; `$0` là tên script.

## 2. Chạy script

```text
an@toinoi-app-01:~$ bao-cao-log /var/log/toinoi/bao-gia.log
-bash: /home/an/bin/bao-cao-log: Permission denied
an@toinoi-app-01:~$ ls -l bin/bao-cao-log
-rw-rw-r-- 1 an an 448 Nov 22 23:21 bin/bao-cao-log
an@toinoi-app-01:~$ chmod u+x bin/bao-cao-log
an@toinoi-app-01:~$ bao-cao-log /var/log/toinoi/bao-gia.log
File   : /var/log/toinoi/bao-gia.log
Số dòng: 362
Lỗi    : 17
Chậm   : 24
an@toinoi-app-01:~$ bash bin/bao-cao-log /var/log/toinoi/bao-gia.log.1
File   : /var/log/toinoi/bao-gia.log.1
Số dòng: 305
Lỗi    : 26
Chậm   : 19
```

1. Gõ tên script: `Permission denied`. Bash tìm thấy file trong `~/bin` (đã nằm trong `PATH`), nhưng file mới tạo có quyền `-rw-rw-r--`, không có `x` ([LNX-007](/articles/linux-nguoi-dung-nhom-quyen-file)).
2. `chmod u+x`: từ đó `bao-cao-log` chạy được như mọi lệnh khác, ở bất kỳ thư mục nào.
3. `bash bin/bao-cao-log`: chạy bằng cách đưa file cho `bash` làm đối số. Cách này không cần quyền `x` và bỏ qua dòng shebang. Hữu ích khi thử nhanh, hoặc khi script nằm trên ổ không cho chạy file.

Kết quả đúng với những gì các bài trước đếm bằng tay: ngày 21/11 có 362 dòng, 17 lỗi, 24 yêu cầu chậm; ngày 20/11 có 26 lỗi.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-script-dau-tien/ssh-studio-02.webp" alt="Terminal của SSH Studio: chạy bao-cao-log bị Permission denied, ls -l cho thấy file chưa có quyền x, chmod u+x, chạy lại in bốn dòng File, Số dòng 362, Lỗi 17, Chậm 24, và chạy bằng bash với bao-gia.log.1 cho 26 lỗi" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · quyền thực thi và hai cách chạy script</figcaption></figure>

## 3. Dùng sai thì sao?

```text
an@toinoi-app-01:~$ bao-cao-log
/home/an/bin/bao-cao-log: line 6: 1: Dùng: bao-cao-log FILE
an@toinoi-app-01:~$ echo $?
1
an@toinoi-app-01:~$ bao-cao-log /khong-co
bao-cao-log: không đọc được /khong-co
an@toinoi-app-01:~$ echo $?
2
an@toinoi-app-01:~$ bao-cao-log /var/log/toinoi/bao-gia.log.1 > /dev/null && echo "báo cáo xong"
báo cáo xong
```

| Tình huống | Script làm gì | Mã thoát |
|---|---|---|
| Không có tham số | `${1:?…}` in tên script, số dòng và thông báo cách dùng | 1 |
| File không đọc được | Thông báo của chính script, ra stderr | 2 |
| Thành công | In kết quả | 0 |

Mã thoát là **hợp đồng** của script với thế giới bên ngoài. `&& echo "báo cáo xong"` chỉ chạy khi `bao-cao-log` trả 0; cron, CI, script khác đều đọc mã thoát theo cùng cách. Quy ước phổ biến: 0 là thành công, 1 là lỗi chung, 2 là dùng sai (thiếu tham số, tham số không hợp lệ). Ghi quy ước của script vào phần chú thích ở đầu file.

## 4. `set -euo pipefail`

Không có dòng này, bash tiếp tục chạy sau mọi lỗi. Một script năm bước mà bước hai hỏng vẫn chạy nốt ba bước còn lại trên dữ liệu sai, rồi trả mã thoát của lệnh cuối cùng, thường là 0.

| Chế độ | Tác dụng |
|---|---|
| `-e` | Dừng script ngay khi một lệnh trả mã khác 0 |
| `-u` | Dùng biến chưa gán là lỗi, thay vì lặng lẽ thay bằng chuỗi rỗng ([LNX-019](/articles/linux-shell-script-an-toan) cho thấy vì sao điều này cứu bạn khỏi `rm -rf`) |
| `-o pipefail` | Một pipeline thất bại nếu **bất kỳ** khâu nào thất bại, không chỉ khâu cuối ([LNX-008](/articles/linux-chuyen-huong-pipe)) |

Ba chế độ này là mặc định nên có ở mọi script. Nhưng `-e` có một mặt trái, và script phiên bản đầu vừa dẫm phải.

## 5. Khi `grep` không tìm thấy gì

Để thử, An tạo một file log không có dòng lỗi nào:

```text
an@toinoi-app-01:~$ grep -v ' ERROR ' /var/log/toinoi/bao-gia.log > ~/tai-lieu/khong-loi.log
an@toinoi-app-01:~$ bao-cao-log ~/tai-lieu/khong-loi.log
an@toinoi-app-01:~$ echo $?
1
```

Script không in gì cả, không một thông báo lỗi, và trả mã 1. Với một người dùng không đọc mã nguồn, đó là trường hợp tệ nhất: không biết hỏng ở đâu, không biết có hỏng không.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-script-dau-tien/ssh-studio-03.webp" alt="Terminal của SSH Studio: bao-cao-log không có tham số in thông báo cách dùng và trả mã 1, với /khong-co trả mã 2, dùng với && in báo cáo xong; sau đó với file khong-loi.log script không in gì và trả mã 1" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · mã thoát của từng tình huống, và một lần chạy hỏng mà không báo gì</figcaption></figure>

### Tìm chỗ hỏng

```text
an@toinoi-app-01:~$ bash -n bin/bao-cao-log && echo "cú pháp ổn"
cú pháp ổn
an@toinoi-app-01:~$ bash -x bin/bao-cao-log ~/tai-lieu/khong-loi.log
+ set -euo pipefail
+ file=/home/an/tai-lieu/khong-loi.log
+ '[' '!' -r /home/an/tai-lieu/khong-loi.log ']'
++ wc -l
+ tong=345
++ grep -c ' ERROR ' /home/an/tai-lieu/khong-loi.log
+ loi=0
an@toinoi-app-01:~$ grep -c ' ERROR ' ~/tai-lieu/khong-loi.log
0
an@toinoi-app-01:~$ echo $?
1
```

- **`bash -n`** đọc script mà không chạy, chỉ kiểm tra cú pháp. Cú pháp ổn: lỗi nằm ở lúc chạy.
- **`bash -x`** in từng lệnh **sau khi** đã thay biến, trước khi chạy, với dấu `+` ở đầu (`++` cho lệnh nằm trong `$( )`). Dòng cuối cùng được in là lệnh cuối cùng đã chạy: `loi=0`, ngay sau `grep -c`. Script dừng ở đó.

Lý do nằm ở hai lệnh cuối: `grep -c` in `0` nhưng **trả mã thoát 1** khi không tìm thấy dòng nào. Với `grep`, mã 1 nghĩa là «không khớp», không phải «hỏng»; mã 2 mới là lỗi thật. `set -e` không biết sự khác biệt đó, thấy mã 1 là dừng.

Đây là nhóm lệnh cần cẩn thận với `set -e`: những lệnh dùng mã thoát để trả lời câu hỏi có/không, như `grep`, `diff` (1 nghĩa là «hai file khác nhau»), `test`.

### Sửa bằng `sed`

```text
an@toinoi-app-01:~$ sed -i 's/\(grep -c .*\))$/\1 || true)/' bin/bao-cao-log
an@toinoi-app-01:~$ grep -n 'grep -c' bin/bao-cao-log
14:loi=$(grep -c ' ERROR ' "$file" || true)
15:cham=$(grep -c ' WARN ' "$file" || true)
an@toinoi-app-01:~$ bao-cao-log ~/tai-lieu/khong-loi.log
File   : /home/an/tai-lieu/khong-loi.log
Số dòng: 345
Lỗi    : 0
Chậm   : 24
an@toinoi-app-01:~$ echo $?
0
```

`lệnh || true`: nếu `lệnh` trả mã khác 0 thì chạy `true`, lệnh luôn trả 0. Cả biểu thức thành công, `set -e` không dừng, còn biến vẫn nhận `0` mà `grep -c` đã in. An dùng `sed` của [LNX-014](/articles/linux-sed-sua-van-ban) để sửa cả hai dòng một lần: nhóm `\(grep -c .*\)` giữ nguyên phần lệnh, rồi chèn `|| true` trước dấu `)` cuối dòng. `grep -n` kiểm tra lại, rồi chạy thử: 0 lỗi, mã thoát 0.

Viết `|| true` có chủ đích và chỉ ở chỗ bạn biết mã khác 0 là bình thường. Rải `|| true` khắp script là tắt `set -e` mà không nói ra.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-script-dau-tien/ssh-studio-04.webp" alt="Terminal của SSH Studio: bash -n báo cú pháp ổn, bash -x in từng bước và dừng sau loi=0, grep -c in 0 nhưng trả mã 1, sed thêm phần hoặc true vào hai dòng grep -c, grep -n xác nhận, và bao-cao-log chạy lại in Lỗi 0 với mã thoát 0" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>bash -x</code> tìm ra chỗ dừng, <code>sed</code> sửa hai dòng</figcaption></figure>

## 6. Script nằm ở đâu, đặt tên thế nào

| Nơi | Dùng cho |
|---|---|
| `~/bin` | Công cụ của riêng bạn |
| `/usr/local/bin` | Công cụ dùng chung cho mọi người trên máy (cần `sudo`) |
| `/srv/<dịch vụ>/scripts` | Script vận hành của một dịch vụ, đi cùng thư mục triển khai: bài thực hành LNX-020 đặt script ở đây |

Đặt tên bằng chữ thường, nối bằng `-`, **không có đuôi** `.sh` nếu script được gọi như một lệnh: người dùng không cần biết `bao-cao-log` viết bằng bash hay Python. Giữ đuôi `.sh` cho file được `source` hoặc chạy bằng `bash file.sh`.

## 7. Lỗi thường gặp

**`Permission denied`.** Thiếu `chmod u+x`.

**`bad interpreter: No such file or directory`.** Shebang trỏ tới chương trình không tồn tại, hoặc file có kiểu xuống dòng Windows (`CRLF`): kernel tìm chương trình tên `bash\r`. Kiểm tra bằng `file` ([LNX-006](/articles/linux-xem-sua-file-van-ban)).

**Script chạy được bằng `bash file` nhưng không chạy được bằng `./file`.** Thiếu quyền `x` hoặc shebang sai.

**Script dừng im lặng.** Gần như luôn là `set -e` gặp một lệnh trả mã khác 0 «bình thường». Chạy với `bash -x`.

**Thông báo lỗi ra stdout.** Người dùng chuyển hướng kết quả vào file thì lỗi cũng chui vào file. Dùng `>&2`.

**Kiểm tra tham số sau khi đã dùng nó.** Kiểm tra trước, ở đầu script.

## Bài tập

1. Thêm dòng `echo "Tỉ lệ lỗi: $(( loi * 100 / tong ))%"` vào cuối script. `$(( … ))` làm gì? Vì sao kết quả là số nguyên?
2. Chạy `bao-cao-log /var/log/toinoi/bao-gia.log.2.gz`. Kết quả có đúng không? (Bài sau sửa chuyện này.)
3. Thêm `set -x` ngay sau dòng `set -euo pipefail`, chạy lại, rồi bỏ đi. So với `bash -x`.
4. Viết script `~/bin/loi-hom-nay` in các dòng `ERROR` của `bao-gia.log` và trả mã 1 nếu có lỗi, 0 nếu không. Dùng nó trong `loi-hom-nay > /dev/null || echo "có lỗi"`.
5. Đổi shebang thành `#!/bin/sh` và chạy `bao-cao-log`. Lỗi gì xuất hiện? (Gợi ý: `pipefail` là tính năng của bash.)

## Đọc tiếp

[LNX-017 · Điều kiện, vòng lặp và hàm trong bash](/articles/linux-bash-dieu-kien-vong-lap-ham): cho `bao-cao-log` đọc nhiều file, kể cả file nén.
