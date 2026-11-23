---
slug: linux-shell-script-an-toan
title: "Shell script an toàn: dấu nháy, set -u, trap, mktemp, flock và shellcheck"
summary: "Năm thói quen phân biệt script dùng được với script nguy hiểm: đặt biến trong nháy kép và lặp file bằng wildcard; set -u bắt một lỗi gõ tên biến suýt biến thành rm -rf /*; trap và mktemp dọn file tạm kể cả khi script hỏng; flock ngăn hai lần chạy chồng nhau; shellcheck tìm lỗi trước khi chạy. Bài viết lại sao-luu-log với tuỳ chọn, giữ N bản mới nhất và ghi file nguyên tử."
category: devops-linux
level: advanced
tags: [linux, lnx-019, bash, shellcheck, trap, mktemp, flock, quoting, shell-script, ssh-studio, toi-noi]
language: vi
---

Một dòng như `rm -rf "$THU_MUC/"*` trông vô hại. Nhưng nếu một ngày `THU_MUC` rỗng (biến gõ sai tên, một bước trước đó hỏng, script được gọi từ một môi trường khác), dòng đó thành `rm -rf /*`: xoá mọi thứ người chạy script có quyền xoá. Cộng đồng Linux đã chứng kiến không ít sự cố đúng kiểu này, kể cả trong phần mềm được dùng rộng rãi. Không có lỗi logic phức tạp nào ở đây, chỉ có một biến rỗng và một script không được viết để phòng chuyện đó.

`sao-luu-log` của An sắp được cron chạy mỗi đêm, không ai nhìn. Bài này đi qua năm thói quen khiến một script chạy không người trông vẫn an toàn, mỗi thói quen gắn với một lỗi làm thật trên lab, rồi viết lại `sao-luu-log` theo đúng những thói quen đó.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-019 · Phần 2/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/linux-tar-gzip-nen-luu-tru">LNX-018</a>. Trạng thái đầu bài: <code>npm run setup -- 19</code>. Sau bài này, bạn biết vì sao mọi biến phải nằm trong nháy kép, biết dùng <code>set -u</code>, <code>trap</code>, <code>mktemp</code>, <code>flock</code>, đọc được cảnh báo của <code>shellcheck</code>, và có <code>sao-luu-log</code> phiên bản 2. Mọi thí nghiệm nguy hiểm trong bài chỉ <b>in</b> lệnh, không chạy.</p>
</div>

## 1. Dấu nháy và tên file có dấu cách

```text
an@toinoi-app-01:~$ mkdir -p ~/thu-an-toan
an@toinoi-app-01:~$ cd ~/thu-an-toan

an@toinoi-app-01:~/thu-an-toan$ touch "bao cao 21-11.txt" bao-cao-20-11.txt
an@toinoi-app-01:~/thu-an-toan$ ls
'bao cao 21-11.txt'   bao-cao-20-11.txt
an@toinoi-app-01:~/thu-an-toan$ f="bao cao 21-11.txt"
an@toinoi-app-01:~/thu-an-toan$ ls -l $f
ls: cannot access 'bao': No such file or directory
ls: cannot access 'cao': No such file or directory
ls: cannot access '21-11.txt': No such file or directory
an@toinoi-app-01:~/thu-an-toan$ ls -l "$f"
-rw-rw-r-- 1 an an 0 Nov 22 23:21 'bao cao 21-11.txt'
an@toinoi-app-01:~/thu-an-toan$ for x in $(ls); do echo "[$x]"; done
[bao]
[cao]
[21-11.txt]
[bao-cao-20-11.txt]
an@toinoi-app-01:~/thu-an-toan$ for x in *; do echo "[$x]"; done
[bao cao 21-11.txt]
[bao-cao-20-11.txt]
```

Khi một biến không nằm trong nháy kép, shell làm thêm hai việc với giá trị của nó: **tách từ** theo khoảng trắng, rồi **mở rộng wildcard** trên từng từ. `ls -l $f` với `f="bao cao 21-11.txt"` thành `ls -l bao cao 21-11.txt`: ba đối số, ba file không tồn tại. `"$f"` giữ nguyên là một đối số.

`$(ls)` bị tách từ theo cùng cách, nên `for x in $(ls)` chia tên file làm ba. `for x in *` thì shell tự liệt kê file, mỗi tên là một phần tử, dấu cách hay ký tự lạ đều không sao.

Quy tắc, không có ngoại lệ đáng kể: **đặt mọi biến và mọi `$( )` trong nháy kép**: `"$f"`, `"$@"`, `"$(lệnh)"`. Tên file trên server có thể do người dùng đặt, do một ứng dụng sinh ra, và sớm muộn sẽ có dấu cách, dấu nháy, hay ký tự bắt đầu bằng `-`. Để phòng trường hợp cuối, viết `--` trước danh sách file: `rm -- "$f"`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-script-an-toan/ssh-studio-01.webp" alt="Terminal của SSH Studio: touch tạo file bao cao 21-11.txt và bao-cao-20-11.txt, ls -l $f không có nháy báo ba lỗi không tìm thấy bao, cao, 21-11.txt, ls -l &quot;$f&quot; đúng, vòng for x in $(ls) tách thành bốn phần, for x in * cho đúng hai file" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · biến không có nháy bị tách theo dấu cách</figcaption></figure>

## 2. `set -u`: một chữ gõ sai và lệnh `rm -rf`

Script dưới đây dọn thư mục tạm của báo cáo. Nó có một lỗi gõ: tên biến ở dòng cuối thiếu chữ `a`. Để an toàn, dòng đó chỉ **in** lệnh `rm` thay vì chạy:

```bash
#!/usr/bin/env bash
# Dọn thư mục tạm của báo cáo. Bản có lỗi gõ sai tên biến: chỉ IN lệnh rm, không chạy.
thu_muc_tam=$HOME/thu-an-toan/tam
echo rm -rf "$thu_muc_tm"/*
```

```text
an@toinoi-app-01:~/thu-an-toan$ vim xoa-tam-loi
an@toinoi-app-01:~/thu-an-toan$ bash xoa-tam-loi
rm -rf /bin /bin.usr-is-merged /boot /dev /etc /home /lib /lib.usr-is-merged /lib64 /media /mnt /opt /proc /root /run /sbin /sbin.usr-is-merged /srv /sys /tmp /usr /var
an@toinoi-app-01:~/thu-an-toan$ bash -u xoa-tam-loi
xoa-tam-loi: line 4: thu_muc_tm: unbound variable
an@toinoi-app-01:~/thu-an-toan$ echo $?
1
an@toinoi-app-01:~/thu-an-toan$ shellcheck xoa-tam-loi

In xoa-tam-loi line 3:
thu_muc_tam=$HOME/thu-an-toan/tam
^---------^ SC2034 (warning): thu_muc_tam appears unused. Verify use (or export if used externally).


In xoa-tam-loi line 4:
echo rm -rf "$thu_muc_tm"/*
             ^---------^ SC2154 (warning): thu_muc_tm is referenced but not assigned (did you mean 'thu_muc_tam'?).

For more information:
  https://www.shellcheck.net/wiki/SC2034 -- thu_muc_tam appears unused. Verif...
  https://www.shellcheck.net/wiki/SC2154 -- thu_muc_tm is referenced but not ...
```

1. **Chạy thường**: `$thu_muc_tm` chưa từng được gán, bash lặng lẽ thay bằng chuỗi rỗng. `""/*` thành `/*`, và shell mở rộng nó thành mọi thư mục ở gốc. Nếu không có chữ `echo`, đó là `rm -rf /bin /boot /dev /etc /home …`. Với tài khoản `an`, lệnh đó sẽ xoá sạch thư mục riêng và mọi thứ khác `an` có quyền xoá; với `root`, nó xoá hệ thống.
2. **`bash -u`** (tương đương `set -u` trong script): dùng biến chưa gán là lỗi. Script dừng ngay ở dòng 4 với `unbound variable`, trước khi kịp làm gì. Đây là lý do `set -euo pipefail` có chữ `u` ([LNX-016](/articles/linux-shell-script-dau-tien)).
3. **`shellcheck`** đọc script mà không chạy, và chỉ ra cả hai vấn đề: `thu_muc_tam` được gán mà không dùng, `thu_muc_tm` được dùng mà không gán, kèm gợi ý «có phải ý bạn là `thu_muc_tam`?».

Thêm một lớp phòng thủ cho mọi lệnh xoá dựa trên biến: `rm -rf -- "${thu_muc_tam:?}"/*`. `${bien:?}` dừng script nếu biến rỗng hoặc chưa gán, kể cả khi không có `set -u`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-script-an-toan/ssh-studio-02.webp" alt="Terminal của SSH Studio: bash xoa-tam-loi in ra lệnh rm -rf với toàn bộ thư mục gốc từ /bin tới /var; bash -u dừng với thông báo thu_muc_tm unbound variable và mã thoát 1; shellcheck báo SC2034 thu_muc_tam không dùng và SC2154 thu_muc_tm chưa gán, gợi ý thu_muc_tam" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · một chữ gõ sai thành <code>rm -rf /*</code>; <code>set -u</code> và <code>shellcheck</code> bắt được</figcaption></figure>

## 3. `trap` và `mktemp`: dọn file tạm kể cả khi hỏng

Script hay cần file tạm. Hai câu hỏi: đặt tên gì để không đè lên file của người khác, và dọn thế nào khi script dừng giữa chừng?

```bash
#!/usr/bin/env bash
# Thử trap: file tạm được xoá kể cả khi script dừng giữa chừng vì lỗi.
set -euo pipefail

tam=$(mktemp /tmp/bao-cao.XXXXXX)
trap 'echo "dọn $tam"; rm -f -- "$tam"' EXIT

echo "file tạm: $tam"
grep ERROR /var/log/toinoi/bao-gia.log > "$tam"
echo "đã ghi $(wc -l < "$tam") dòng"
grep ERROR /khong-co >> "$tam"
echo "dòng này không bao giờ được in"
```

```text
an@toinoi-app-01:~/thu-an-toan$ mktemp
/tmp/tmp.XgdqlMMjov
an@toinoi-app-01:~/thu-an-toan$ vim thu-trap
an@toinoi-app-01:~/thu-an-toan$ bash thu-trap
file tạm: /tmp/bao-cao.KdFeW4
đã ghi 17 dòng
grep: /khong-co: No such file or directory
dọn /tmp/bao-cao.KdFeW4
an@toinoi-app-01:~/thu-an-toan$ echo $?
2
an@toinoi-app-01:~/thu-an-toan$ ls /tmp/bao-cao.* 2>/dev/null | wc -l
0
```

- **`mktemp`** tạo một file mới với tên ngẫu nhiên (thay các chữ `X` bằng ký tự ngẫu nhiên), quyền `600`, và in tên đó ra. Không bao giờ trùng với file đã có. `mktemp -d` tạo thư mục. Đừng tự đặt tên kiểu `/tmp/bao-cao.tmp`: hai lần chạy cùng lúc sẽ dùng chung một file, và trên máy nhiều người dùng, người khác có thể tạo sẵn file đó để chờ.
- **`trap 'LỆNH' EXIT`** đăng ký một lệnh chạy khi script **kết thúc vì bất kỳ lý do gì**: chạy hết, `exit`, lỗi với `set -e`, hay `Ctrl+C`. Trong ví dụ, `grep` thứ hai lỗi, `set -e` dừng script, nhưng `trap` vẫn chạy và xoá file tạm. Mã thoát 2 là của `grep`.
- Nháy đơn quanh lệnh của `trap` để `$tam` được thay **lúc trap chạy**, không phải lúc đăng ký.

`/tmp` được dọn khi khởi động lại, nhưng một server có thể chạy nhiều tháng không khởi động lại. Script chạy mỗi đêm mà mỗi đêm để lại một file tạm là một cách chậm rãi làm đầy đĩa.

## 4. `flock`: không cho hai lần chạy chồng nhau

Cron chạy `sao-luu-log` lúc 01:17. Nếu một đêm log lớn bất thường và lần chạy kéo dài hơn một ngày, hoặc ai đó chạy tay đúng lúc cron chạy, hai bản sẽ cùng ghi một file.

```text
an@toinoi-app-01:~/thu-an-toan$ flock -n /tmp/thu.lock echo "lấy được khoá"
lấy được khoá
an@toinoi-app-01:~/thu-an-toan$ flock -n /tmp/thu.lock bash -c 'flock -n /tmp/thu.lock echo "lấy được khoá" || echo "khoá đang bị giữ"'
khoá đang bị giữ
```

**`flock`** đặt một khoá lên một file. Chỉ một tiến trình giữ được khoá tại một thời điểm; **`-n`** (*non-blocking*) khiến tiến trình thứ hai thất bại ngay thay vì chờ. Lệnh thứ hai mô phỏng hai lần chạy: `flock` ngoài giữ khoá trong khi `bash -c` chạy, và `flock` bên trong không lấy được.

Khoá tự nhả khi tiến trình giữ nó kết thúc, kể cả khi tiến trình đó chết bất ngờ. Đây là ưu điểm lớn so với cách tự làm bằng «file đánh dấu» (`touch running`, `rm running`): file đánh dấu nằm lại sau một lần script chết, và mọi lần sau đều tưởng có bản khác đang chạy.

Trong script, cách dùng phổ biến là mở file khoá trên một số hiệu file (*file descriptor*) riêng:

```bash
exec 9> "$dich/.sao-luu.lock"
if ! flock -n 9; then
  echo "sao-luu-log: đang có một lần sao lưu khác chạy" >&2
  exit 1
fi
```

`exec 9> file` mở file để ghi trên số hiệu 9 và giữ nó mở tới hết script. `flock -n 9` khoá số hiệu đó.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-script-an-toan/ssh-studio-03.webp" alt="Terminal của SSH Studio: mktemp in tên file ngẫu nhiên trong /tmp, bash thu-trap ghi 17 dòng, grep lỗi với /khong-co, trap in dòng dọn file tạm, mã thoát 2, không còn file bao-cao trong /tmp; flock -n lấy được khoá, và flock lồng nhau báo khoá đang bị giữ" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>trap</code> dọn file tạm khi script hỏng; <code>flock</code> chặn lần chạy thứ hai</figcaption></figure>

## 5. `sao-luu-log` phiên bản 2

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-script-an-toan/ssh-studio-04.webp" alt="vim mở ~/bin/sao-luu-log phiên bản 2: chú thích cách dùng với -d, -g và biến NGAY, set -euo pipefail, giá trị mặc định, vòng while getopts xử lý tuỳ chọn, case kiểm tra -g là số nguyên dương, mkdir và khối exec 9> với flock -n" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sao-luu-log</code> phiên bản 2 trong vim (phần đầu)</figcaption></figure>

```bash
#!/usr/bin/env bash
# sao-luu-log: đóng gói log của bao-gia thành ĐÍCH/log-NGÀY.tar.gz, giữ lại N bản mới nhất.
# Dùng: sao-luu-log [-d THƯ_MỤC_ĐÍCH] [-g SỐ_BẢN_GIỮ]
#       NGAY=2026-11-21 sao-luu-log    đặt ngày trong tên file (mặc định: hôm nay)
set -euo pipefail

nguon=/var/log/toinoi
dich=$HOME/sao-luu
giu=7

while getopts 'd:g:' opt; do
  case $opt in
    d) dich=$OPTARG ;;
    g) giu=$OPTARG ;;
    *) echo "Dùng: sao-luu-log [-d THƯ_MỤC_ĐÍCH] [-g SỐ_BẢN_GIỮ]" >&2; exit 2 ;;
  esac
done

case $giu in
  '' | *[!0-9]* | 0) echo "sao-luu-log: -g cần một số nguyên dương, nhận '$giu'" >&2; exit 2 ;;
esac

ngay=${NGAY:-$(date +%F)}
mkdir -p "$dich"

# Chỉ một lần sao lưu chạy tại một thời điểm.
exec 9> "$dich/.sao-luu.lock"
if ! flock -n 9; then
  echo "sao-luu-log: đang có một lần sao lưu khác chạy" >&2
  exit 1
fi

# Ghi ra file tạm trong cùng thư mục, xong mới đổi tên: không bao giờ để lại file dở dang.
tam=$(mktemp "$dich/.log-$ngay.XXXXXX")
trap 'rm -f -- "$tam"' EXIT

tar -czf "$tam" -C "$nguon" .
chmod 640 "$tam"
mv -- "$tam" "$dich/log-$ngay.tar.gz"
echo "sao-luu-log: đã ghi $dich/log-$ngay.tar.gz"

# Chỉ giữ $giu bản mới nhất. Tên file chứa ngày nên xếp theo tên cũng là xếp theo ngày.
find "$dich" -maxdepth 1 -name 'log-*.tar.gz' | sort | head -n -"$giu" | while read -r cu; do
  rm -f -- "$cu"
  echo "sao-luu-log: đã xoá bản cũ $cu"
done
```

Điểm mới, theo thứ tự trong file:

| Kỹ thuật | Giải quyết |
|---|---|
| `getopts 'd:g:'` | Tuỳ chọn kiểu `-d DIR -g 7`. Dấu `:` sau chữ cái nghĩa là tuỳ chọn đó cần giá trị, nằm trong `$OPTARG` |
| `case $giu in … *[!0-9]* …` | Kiểm tra đầu vào: chỉ chấp nhận số nguyên dương. Đầu vào sai thì dừng với mã 2, trước khi làm gì |
| `${NGAY:-$(date +%F)}` | Giá trị mặc định: dùng `NGAY` nếu có, không thì hôm nay. Nhờ vậy thử được script với ngày cố định |
| `exec 9>` + `flock -n 9` | Không chạy chồng (mục 4) |
| `mktemp` trong **cùng thư mục** + `mv` | Ghi xong mới đổi tên. Trong cùng hệ thống file, `mv` là một thao tác **nguyên tử**: file `log-NGÀY.tar.gz` hoặc chưa có, hoặc đầy đủ, không bao giờ dở dang. `mktemp` trong `/tmp` rồi `mv` sang đĩa khác thì mất tính chất này |
| `trap … EXIT` | `tar` hỏng giữa chừng thì file tạm bị xoá |
| `chmod 640` | Bản sao lưu chứa log, không cho người ngoài nhóm đọc |
| `find … \| sort \| head -n -N` | `head -n -7` in **mọi dòng trừ 7 dòng cuối**: các bản cũ cần xoá |

Script này phải qua `shellcheck` sạch, và `bao-cao-log` của [LNX-017](/articles/linux-bash-dieu-kien-vong-lap-ham) cũng vậy:

```text
an@toinoi-app-01:~/thu-an-toan$ shellcheck ~/bin/sao-luu-log ~/bin/bao-cao-log && echo "shellcheck: không có cảnh báo"
shellcheck: không có cảnh báo
an@toinoi-app-01:~/thu-an-toan$ NGAY=2026-11-19 sao-luu-log -d ~/sao-luu/thu
sao-luu-log: đã ghi /home/an/sao-luu/thu/log-2026-11-19.tar.gz
an@toinoi-app-01:~/thu-an-toan$ NGAY=2026-11-20 sao-luu-log -d ~/sao-luu/thu
sao-luu-log: đã ghi /home/an/sao-luu/thu/log-2026-11-20.tar.gz
an@toinoi-app-01:~/thu-an-toan$ NGAY=2026-11-21 sao-luu-log -d ~/sao-luu/thu -g 2
sao-luu-log: đã ghi /home/an/sao-luu/thu/log-2026-11-21.tar.gz
sao-luu-log: đã xoá bản cũ /home/an/sao-luu/thu/log-2026-11-19.tar.gz
an@toinoi-app-01:~/thu-an-toan$ ls -l ~/sao-luu/thu
total 32
-rw-r----- 1 an an 15111 Nov 22 23:21 log-2026-11-20.tar.gz
-rw-r----- 1 an an 15111 Nov 22 23:21 log-2026-11-21.tar.gz
an@toinoi-app-01:~/thu-an-toan$ flock -n ~/sao-luu/thu/.sao-luu.lock sao-luu-log -d ~/sao-luu/thu
sao-luu-log: đang có một lần sao lưu khác chạy
an@toinoi-app-01:~/thu-an-toan$ echo $?
1
an@toinoi-app-01:~/thu-an-toan$ sao-luu-log -g abc
sao-luu-log: -g cần một số nguyên dương, nhận 'abc'
an@toinoi-app-01:~/thu-an-toan$ echo $?
2
```

- Ba lần chạy với `NGAY` khác nhau tạo ba bản; lần thứ ba có `-g 2` nên xoá bản cũ nhất. Còn lại đúng hai bản, quyền `-rw-r-----`.
- Một `flock` bên ngoài giữ khoá (giả lập một lần chạy khác): script báo và trả 1, không đụng vào file nào.
- `-g abc` bị từ chối với mã 2 trước khi làm bất cứ việc gì.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-shell-script-an-toan/ssh-studio-05.webp" alt="Terminal của SSH Studio: shellcheck không cảnh báo hai script, ba lần chạy sao-luu-log với NGAY 19, 20, 21 tháng 11 và -g 2 xoá bản cũ nhất, ls -l còn hai bản quyền rw-r-----, flock giả lập lần chạy khác khiến script báo đang có lần sao lưu khác và trả 1, -g abc bị từ chối với mã 2" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sao-luu-log</code> phiên bản 2: giữ N bản, khoá, kiểm tra đầu vào</figcaption></figure>

## 6. `shellcheck` trong công việc hằng ngày

`shellcheck` (có sẵn trong lab, cài bằng `sudo apt install shellcheck`) bắt phần lớn những lỗi trong bài này trước khi script chạy: biến không có nháy (SC2086), lặp qua `$(ls)` (SC2045), biến chưa gán (SC2154), `rm` với biến có thể rỗng (SC2115), `cd` không kiểm tra lỗi (SC2164). Mỗi cảnh báo có mã, và `https://www.shellcheck.net/wiki/SC2086` giải thích kèm ví dụ.

Chạy `shellcheck` trước khi giao một script cho cron hay cho người khác, giống như chạy test trước khi merge. Khi thật sự muốn bỏ qua một cảnh báo, ghi rõ lý do bằng chú thích ngay trên dòng đó: `# shellcheck disable=SC2034  # biến dùng trong file được source`.

## 7. Danh sách kiểm tra trước khi giao script

| | Kiểm tra |
|---|---|
| 1 | Có shebang và `set -euo pipefail` |
| 2 | Mọi biến, `$( )`, `"$@"` nằm trong nháy kép |
| 3 | Kiểm tra tham số và đầu vào ở đầu script; dùng sai thì thoát với mã 2 |
| 4 | Lỗi ra stderr (`>&2`), mã thoát có nghĩa |
| 5 | File tạm bằng `mktemp`, dọn bằng `trap … EXIT` |
| 6 | Ghi file kết quả nguyên tử: file tạm cùng thư mục rồi `mv` |
| 7 | `flock` nếu script có thể chạy chồng |
| 8 | Lệnh xoá dựa trên biến có `${bien:?}` và `--` |
| 9 | `shellcheck` sạch |
| 10 | Chạy thử với đầu vào sai, file thiếu, quyền thiếu |

## 8. Lỗi thường gặp

**Thiếu nháy quanh biến.** Hỏng với tên có dấu cách; tệ hơn, wildcard trong giá trị bị mở rộng.

**Không có `set -u`.** Một lỗi gõ tên biến thành chuỗi rỗng, và chuỗi rỗng trong đường dẫn có thể thành `/`.

**Tự đặt tên file tạm cố định.** Đụng nhau khi chạy chồng, và là lỗ hổng trên máy nhiều người dùng.

**`trap` với nháy kép.** `trap "rm $tam" EXIT` thay biến lúc đăng ký; nếu `tam` gán sau dòng đó, trap xoá sai thứ.

**File khoá trong `/tmp` dùng chung.** Đặt cạnh dữ liệu của script (như `$dich/.sao-luu.lock`) để hai bản cài khác nhau không chặn nhau.

**Bỏ qua cảnh báo của `shellcheck` vì «chạy vẫn được».** Chạy được với dữ liệu hôm nay, chưa chắc với dữ liệu ngày mai.

## Bài tập

1. Chạy `shellcheck` với `~/bin/xin-chao` của [LNX-011](/articles/linux-bien-moi-truong-path-bashrc). Có cảnh báo không?
2. Viết lại `xoa-tam-loi` cho đúng: sửa tên biến, thêm `set -u`, dùng `${thu_muc_tam:?}` và `--`. Giữ chữ `echo` và chạy thử.
3. Trong `thu-trap`, đổi `EXIT` thành `INT` rồi chạy lại. File tạm có được dọn khi script hỏng không? Vì sao?
4. Chạy `sao-luu-log -d ~/sao-luu/thu` trong một tab phiên, và trong lúc nó chạy (thêm `sleep 10` tạm thời vào script sau dòng `flock`), chạy lại ở tab thứ hai. Tab thứ hai báo gì?
5. `sao-luu-log -x`: `getopts` báo gì, script trả mã nào?

## Đọc tiếp

[LNX-020 · Thực hành: báo cáo và sao lưu log hằng ngày bằng cron](/articles/linux-thuc-hanh-script-sao-luu-log): đưa các script vào thư mục dùng chung, chạy chúng trong môi trường của cron, và bàn giao có bộ chấm.
