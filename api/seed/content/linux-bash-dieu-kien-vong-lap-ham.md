---
slug: linux-bash-dieu-kien-vong-lap-ham
title: "Điều kiện, vòng lặp và hàm trong bash: cho script đọc nhiều file log, kể cả file nén"
summary: "test và [ ] cho file, số và chuỗi; vì sao so sánh 17 với 9 có thể sai; [[ ]] với mẫu; if, elif, else; vòng for qua danh sách và qua file; while read đọc từng dòng; case rẽ nhánh theo đuôi file; hàm với biến local. Bài dùng tất cả để viết phiên bản 2 của bao-cao-log: nhận nhiều file, đọc được file .gz, bỏ qua file hỏng mà vẫn báo đúng mã thoát."
category: devops-linux
level: intermediate
tags: [linux, lnx-017, bash, if, for, while, case, function, shell-script, ssh-studio, toi-noi]
language: vi
---

Phiên bản đầu của `bao-cao-log` chỉ nhận một file và không đọc được file nén: chạy nó với `bao-gia.log.2.gz` thì `grep` đếm trên dữ liệu nhị phân và cho ra con số vô nghĩa. Bình muốn một lệnh tóm tắt được cả ba ngày, và nếu một file không đọc được thì vẫn báo cáo các file còn lại.

Muốn vậy, script phải **quyết định** (file nén hay không, đọc được hay không) và **lặp** (qua từng file). Bài này đi qua các cấu trúc điều khiển của bash bằng những dòng lệnh chạy được ngay trong terminal, rồi ghép chúng thành phiên bản 2.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-017 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-shell-script-dau-tien">LNX-016</a>. Trạng thái đầu bài: <code>npm run setup -- 17</code>. Sau bài này, bạn viết được điều kiện cho file, số, chuỗi; lặp qua danh sách, qua file và qua từng dòng; tách script thành hàm; và có <code>bao-cao-log</code> phiên bản 2.</p>
</div>

## 1. Điều kiện: `test`, `[ ]` và `[[ ]]`

Trong bash, «điều kiện» chỉ là **một lệnh**: mã thoát 0 là đúng, khác 0 là sai. `test` và `[` là cùng một lệnh: `[ -r file ]` chính là `test -r file`, chỉ khác là `[` đòi dấu `]` ở cuối. Vì `[` là một lệnh, **dấu cách quanh nó là bắt buộc**: `[-r file]` là lệnh tên `[-r`, không tồn tại.

```text
an@toinoi-app-01:~$ [ -r /var/log/toinoi/bao-gia.log ] && echo "đọc được"
đọc được
an@toinoi-app-01:~$ [ -w /var/log/toinoi/bao-gia.log ] || echo "không ghi được"
không ghi được
an@toinoi-app-01:~$ so_loi=$(grep -c ' ERROR ' /var/log/toinoi/bao-gia.log)
an@toinoi-app-01:~$ [ "$so_loi" -gt 10 ] && echo "$so_loi lỗi: nhiều hơn 10"
17 lỗi: nhiều hơn 10
an@toinoi-app-01:~$ [ "$so_loi" -gt 9 ] && echo "so sánh số: 17 > 9"
so sánh số: 17 > 9
an@toinoi-app-01:~$ [ "$so_loi" \> 9 ] || echo "so sánh chuỗi: \"17\" đứng trước \"9\""
so sánh chuỗi: "17" đứng trước "9"
an@toinoi-app-01:~$ [[ bao-gia.log.2.gz == *.gz ]] && echo "file nén"
file nén
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bash-dieu-kien-vong-lap-ham/ssh-studio-05.webp" alt="Terminal của SSH Studio: [ -r … ] &amp;&amp; echo đọc được, [ -w … ] || echo không ghi được, so_loi=$(grep -c …), [ &quot;$so_loi&quot; -gt 10 ] in 17 lỗi: nhiều hơn 10" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · kiểm tra file và so sánh số với <code>[ ]</code></figcaption></figure>

| Kiểm tra | Đúng khi |
|---|---|
| `-e f` / `-f f` / `-d f` | Tồn tại / là file thường / là thư mục |
| `-r f` / `-w f` / `-x f` | Bạn đọc / ghi / chạy được |
| `-s f` | File có kích thước lớn hơn 0 |
| `-z "$s"` / `-n "$s"` | Chuỗi rỗng / khác rỗng |
| `"$a" = "$b"` / `!=` | Hai chuỗi bằng / khác nhau |
| `$a -eq $b` | Hai **số** bằng nhau; `-ne`, `-lt`, `-le`, `-gt`, `-ge` |
| `! ĐK` | Đảo điều kiện |

Log thuộc về `baogia` với quyền `644`, nên `an` đọc được nhưng không ghi được: hai dòng đầu xác nhận điều đó.

### Số hay chuỗi?

Hai dòng giữa của khối trên đáng đọc kỹ. `-gt` so **số**: 17 lớn hơn 9. `\>` so **chuỗi** theo thứ tự từ điển: `"17"` đứng trước `"9"` vì ký tự `1` đứng trước ký tự `9`, nên `17 > 9` là **sai**. (Dấu `\` cần có vì `>` trần là chuyển hướng.) Lỗi này giống hệt `sort` thiếu `-n` ở [LNX-013](/articles/linux-cut-sort-uniq-tr). Quy tắc: so số dùng `-eq`, `-lt`, `-gt`; so chuỗi dùng `=`, `!=`.

### `[[ ]]`

`[[ … ]]` là cú pháp riêng của bash (không phải một lệnh), an toàn hơn và mạnh hơn `[ ]`:

- Biến bên trong không cần nháy để tránh tách từ (vẫn nên có).
- `==` so với **mẫu** kiểu wildcard: `[[ $f == *.gz ]]`. `=~` so với biểu thức chính quy: `[[ $ma =~ ^q-[0-9]+$ ]]`.
- Viết được `&&`, `||` bên trong: `[[ -r $f && -s $f ]]`.

Dùng `[[ ]]` trong script bash; dùng `[ ]` khi cần script chạy được với `sh`.

### `if`, `elif`, `else`

```text
an@toinoi-app-01:~$ if [ "$so_loi" -eq 0 ]; then echo "không có lỗi"; elif [ "$so_loi" -lt 20 ]; then echo "ít lỗi"; else echo "nhiều lỗi"; fi
ít lỗi
```

Gõ trên một dòng thì các phần ngăn nhau bằng `;`. Trong script, viết nhiều dòng cho dễ đọc:

```bash
if [ "$so_loi" -eq 0 ]; then
  echo "không có lỗi"
elif [ "$so_loi" -lt 20 ]; then
  echo "ít lỗi"
else
  echo "nhiều lỗi"
fi
```

Sau `if` có thể là **bất kỳ lệnh nào**, không chỉ `[ ]`: `if grep -q ' ERROR ' "$f"; then …` đọc là «nếu có dòng lỗi». Đây cũng là cách viết đúng thay cho `if [ $(grep -c …) -gt 0 ]`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bash-dieu-kien-vong-lap-ham/ssh-studio-01.webp" alt="Terminal của SSH Studio: -gt so sánh số đúng 17 > 9, \> so sánh chuỗi cho thấy &quot;17&quot; đứng trước &quot;9&quot;, [[ … == *.gz ]] khớp mẫu file nén, if elif else in ít lỗi" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · so sánh số và so sánh chuỗi khác nhau; <code>[[ ]]</code> với mẫu; <code>if</code></figcaption></figure>

## 2. Vòng lặp

### `for` qua một danh sách

```text
an@toinoi-app-01:~$ for dv in bike car4 car7 delivery; do echo "$dv: $(grep -c "dich_vu=$dv " /var/log/toinoi/bao-gia.log) yêu cầu"; done
bike: 83 yêu cầu
car4: 99 yêu cầu
car7: 94 yêu cầu
delivery: 84 yêu cầu
an@toinoi-app-01:~$ for f in /var/log/toinoi/bao-gia.log*; do echo "$f"; done
/var/log/toinoi/bao-gia.log
/var/log/toinoi/bao-gia.log.1
/var/log/toinoi/bao-gia.log.2.gz
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bash-dieu-kien-vong-lap-ham/ssh-studio-06.webp" alt="Terminal của SSH Studio: for dv in bike car4 car7 delivery đếm yêu cầu của từng dịch vụ, for f in /var/log/toinoi/bao-gia.log* liệt kê ba file log" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>for</code> lặp qua danh sách và qua file</figcaption></figure>

`for BIẾN in DANH_SÁCH; do …; done` chạy thân vòng lặp một lần cho mỗi phần tử. Danh sách có thể viết thẳng ra, có thể là wildcard (shell mở rộng thành tên file), có thể là `"$@"` (mọi tham số của script), hoặc `{1..10}`.

Chú ý dấu cách sau `$dv` trong mẫu `"dich_vu=$dv "`: thiếu nó thì `dich_vu=car` sẽ khớp cả `car4` lẫn `car7`.

**Lặp qua file bằng wildcard, đừng bằng `$(ls)`.** `for f in $(ls)` tách tên file theo dấu cách, nên `bao cao.txt` thành hai phần tử. [LNX-019](/articles/linux-shell-script-an-toan) thử chuyện này.

### `while read`: lặp qua từng dòng

```text
an@toinoi-app-01:~$ grep ' ERROR ' /var/log/toinoi/bao-gia.log | head -n 3 | while read -r ngay gio muc ma dv _; do echo "$gio $ma $dv"; done
06:37:58 q-100034 dich_vu=car7
06:41:18 q-100038 dich_vu=car4
07:20:18 q-100076 dich_vu=bike
```

`read` đọc **một dòng** từ stdin và tách nó vào các biến theo khoảng trắng; biến cuối cùng nhận phần còn lại của dòng. Đặt `_` cho phần không dùng tới là quy ước quen thuộc. `-r` giữ nguyên dấu `\` trong dữ liệu; nên luôn có.

`while read` đọc đầu ra của một lệnh từng dòng, kể cả khi mỗi dòng có dấu cách. Một chi tiết: vòng `while` đứng sau `|` chạy trong một shell con, nên biến gán bên trong không còn sau vòng lặp. Cần giữ lại, dùng `while read …; do …; done < file` hoặc `< <(lệnh)`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bash-dieu-kien-vong-lap-ham/ssh-studio-07.webp" alt="Terminal của SSH Studio: grep ERROR | head -n 3 | while read -r ngay gio muc ma dv _ in giờ, mã yêu cầu và dịch vụ của ba lỗi đầu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>while read</code> tách từng dòng thành các biến</figcaption></figure>

### `case`: rẽ nhánh theo mẫu

```text
an@toinoi-app-01:~$ for f in /var/log/toinoi/bao-gia.log*; do case $f in *.gz) echo "$f: nén gzip" ;; *.log) echo "$f: đang ghi" ;; *) echo "$f: đã xoay vòng" ;; esac; done
/var/log/toinoi/bao-gia.log: đang ghi
/var/log/toinoi/bao-gia.log.1: đã xoay vòng
/var/log/toinoi/bao-gia.log.2.gz: nén gzip
```

```bash
case $f in
  *.gz)  echo "$f: nén gzip" ;;
  *.log) echo "$f: đang ghi" ;;
  *)     echo "$f: đã xoay vòng" ;;
esac
```

`case` so một giá trị với lần lượt từng mẫu wildcard và chạy nhánh đầu tiên khớp. `;;` kết thúc một nhánh; `*)` khớp mọi thứ còn lại. Nhiều mẫu cho một nhánh viết bằng `|`: `*.gz|*.tgz)`. Với những lựa chọn theo dạng chuỗi (đuôi file, tuỳ chọn dòng lệnh, tên môi trường), `case` gọn và dễ đọc hơn một chuỗi `if … elif`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bash-dieu-kien-vong-lap-ham/ssh-studio-02.webp" alt="Terminal của SSH Studio: for kết hợp case in mỗi file log là đang ghi, đã xoay vòng hay nén gzip" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>case</code> rẽ nhánh theo mẫu tên file</figcaption></figure>

## 3. Hàm

```bash
ten_ham() {
  local a=$1
  …
}
```

Hàm là một nhóm lệnh có tên, gọi như một lệnh: `ten_ham đối_số`. Bên trong, `$1`, `$2`, `$#`, `"$@"` là tham số **của hàm**. `local` giới hạn biến trong hàm; thiếu `local`, biến của hàm ghi đè biến cùng tên của cả script. Hàm trả về mã thoát của lệnh cuối cùng, hoặc mã đặt bằng `return N`; dữ liệu thì «trả về» bằng cách in ra stdout, để người gọi bắt bằng `$( )`.

## 4. `bao-cao-log` phiên bản 2

An mở lại `~/bin/bao-cao-log`, xoá hết nội dung cũ bằng `:%d`, rồi viết phiên bản mới:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bash-dieu-kien-vong-lap-ham/ssh-studio-03.webp" alt="vim mở ~/bin/bao-cao-log phiên bản 2: hàm doc_log dùng case để chọn zcat hoặc cat, hàm tom_tat với biến local và printf, kiểm tra số tham số, vòng for qua các file với continue khi không đọc được, và exit với mã thoát tổng hợp" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>bao-cao-log</code> phiên bản 2: hai hàm và một vòng lặp</figcaption></figure>

```bash
#!/usr/bin/env bash
# bao-cao-log: tóm tắt log của dịch vụ bao-gia, mỗi file một dòng.
# Dùng: bao-cao-log FILE...    (đọc được cả file .gz)
# Mã thoát: 0 nếu đọc được mọi file, 1 nếu bỏ qua file nào đó, 2 nếu dùng sai.
set -euo pipefail

doc_log() {
  case $1 in
    *.gz) zcat -- "$1" ;;
    *) cat -- "$1" ;;
  esac
}

tom_tat() {
  local file=$1 tong loi cham
  tong=$(doc_log "$file" | wc -l)
  loi=$(doc_log "$file" | grep -c ' ERROR ' || true)
  cham=$(doc_log "$file" | grep -c ' WARN ' || true)
  printf '%-33s %4d dòng %4d lỗi %4d chậm\n' "$file" "$tong" "$loi" "$cham"
}

if [ "$#" -eq 0 ]; then
  echo "Dùng: bao-cao-log FILE..." >&2
  exit 2
fi

ma_thoat=0
for file in "$@"; do
  if [ ! -r "$file" ]; then
    echo "bao-cao-log: bỏ qua $file (không đọc được)" >&2
    ma_thoat=1
    continue
  fi
  tom_tat "$file"
done
exit "$ma_thoat"
```

Những gì mới so với phiên bản 1:

- **`doc_log`** che giấu sự khác biệt giữa file nén và file thường: phần còn lại của script chỉ việc đọc stdout của nó. `--` báo hết tuỳ chọn, phòng khi tên file bắt đầu bằng `-`.
- **`tom_tat`** in **một dòng** cho mỗi file bằng `printf`, dễ đọc khi có nhiều file và dễ xử lý tiếp bằng `awk`. `%-33s` là chuỗi căn trái rộng 33 ký tự.
- **`for file in "$@"`** lặp qua mọi tham số. Nháy kép quanh `"$@"` giữ nguyên tên file có dấu cách.
- **`continue`** bỏ qua phần còn lại của vòng lặp và sang file kế. File hỏng không làm dừng cả báo cáo.
- **`ma_thoat`** nhớ rằng đã có file bị bỏ qua, để cuối cùng trả 1 thay vì 0. Một script bỏ qua lỗi mà vẫn trả 0 là đang nói dối người gọi nó.

Vì `set -e` không dừng script khi lệnh trong điều kiện của `if` thất bại, `if [ ! -r "$file" ]` an toàn. Cũng vì thế `|| true` vẫn cần cho `grep -c` (xem [LNX-016](/articles/linux-shell-script-dau-tien)).

```text
an@toinoi-app-01:~$ bao-cao-log /var/log/toinoi/bao-gia.log*
/var/log/toinoi/bao-gia.log        362 dòng   17 lỗi   24 chậm
/var/log/toinoi/bao-gia.log.1      305 dòng   26 lỗi   19 chậm
/var/log/toinoi/bao-gia.log.2.gz   302 dòng   18 lỗi   25 chậm
an@toinoi-app-01:~$ bao-cao-log /var/log/toinoi/bao-gia.log /khong-co /var/log/toinoi/bao-gia.log.1
/var/log/toinoi/bao-gia.log        362 dòng   17 lỗi   24 chậm
bao-cao-log: bỏ qua /khong-co (không đọc được)
/var/log/toinoi/bao-gia.log.1      305 dòng   26 lỗi   19 chậm
an@toinoi-app-01:~$ echo $?
1
an@toinoi-app-01:~$ bao-cao-log
Dùng: bao-cao-log FILE...
an@toinoi-app-01:~$ echo $?
2
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bash-dieu-kien-vong-lap-ham/ssh-studio-08.webp" alt="Terminal của SSH Studio: vim bin/bao-cao-log phiên bản 2, bao-cao-log /var/log/toinoi/bao-gia.log* in ba dòng thẳng cột cho cả file chưa nén và file .gz" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · phiên bản 2 đọc được cả file nén, mỗi file một dòng</figcaption></figure>

- Ba file, ba dòng, file nén được đọc đúng: 302 dòng của ngày 19/11.
- Một file không tồn tại ở giữa: thông báo ra stderr, hai file còn lại vẫn có báo cáo, mã thoát 1.
- Không có tham số: thông báo cách dùng, mã thoát 2, đúng như ghi ở đầu file.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-bash-dieu-kien-vong-lap-ham/ssh-studio-04.webp" alt="Terminal của SSH Studio: bao-cao-log với một file không tồn tại vẫn in hai file đọc được, báo bỏ qua /khong-co và trả mã 1; không tham số in cách dùng và trả mã 2" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · mã thoát của phiên bản 2: 1 khi bỏ qua file, 2 khi dùng sai</figcaption></figure>

## 5. Tóm tắt cú pháp

| Cấu trúc | Mẫu |
|---|---|
| Điều kiện | `if LỆNH; then …; elif LỆNH; then …; else …; fi` |
| Kiểm tra | `[ -r "$f" ]`, `[ "$n" -gt 0 ]`, `[[ $f == *.gz ]]` |
| Lặp danh sách | `for x in a b c; do …; done` |
| Lặp file | `for f in /duong/dan/*.log; do …; done` |
| Lặp tham số | `for x in "$@"; do …; done` |
| Lặp từng dòng | `while read -r a b _; do …; done < file` |
| Rẽ nhánh theo mẫu | `case $x in a\|b) … ;; *) … ;; esac` |
| Hàm | `ten() { local x=$1; …; }` |
| Điều khiển vòng lặp | `continue`, `break` |
| Số học | `$(( a + b ))`, `(( n++ ))` |

## 6. Lỗi thường gặp

**Thiếu dấu cách trong `[ ]`.** `[$n -gt 0]` là lỗi. Viết `[ "$n" -gt 0 ]`.

**So số bằng `=` hoặc `>`.** `[ 17 \> 9 ]` sai. Dùng `-gt`.

**Biến không có nháy trong `[ ]`.** `[ -r $f ]` với `$f` rỗng thành `[ -r ]`, một phép kiểm tra **luôn đúng**. Luôn viết `"$f"`.

**`for f in $(ls)`.** Hỏng với tên có dấu cách. Dùng wildcard.

**Biến gán trong `… | while read` biến mất sau vòng lặp.** Vòng lặp chạy trong shell con. Dùng `done < <(lệnh)`.

**Quên `local` trong hàm.** Hàm ghi đè biến của script.

## Bài tập

1. Viết một vòng `for` in số dòng của mỗi file trong `/var/log/toinoi`, dùng `doc_log` (định nghĩa hàm đó ngay trong terminal trước).
2. Viết một dòng `while read` in mã yêu cầu của mọi dòng `WARN` có thời gian trên 2500 ms, dùng `(( ms > 2500 ))` để so.
3. Thêm vào `bao-cao-log` một nhánh `case` cho file `.zst` (đọc bằng `zstdcat`). Thử với file tạo bằng `zstd -k`.
4. Sửa `bao-cao-log` để in thêm một dòng tổng ở cuối khi có từ hai file trở lên.
5. Viết hàm `la_so` trả 0 nếu đối số là số nguyên dương, 1 nếu không. Gợi ý: `case $1 in ''|*[!0-9]*) return 1 ;; esac`.

## Đọc tiếp

[LNX-018 · Nén và lưu trữ: gzip, xz, zstd và tar](/articles/linux-tar-gzip-nen-luu-tru): đóng gói log để sao lưu, và script sao lưu đầu tiên.
