---
slug: linux-sed-sua-van-ban
title: "sed trên Linux: in theo mẫu, thay chuỗi và sửa file cấu hình tại chỗ"
summary: "sed đọc văn bản từng dòng và áp lệnh lên những dòng chọn được. Bài dùng log và cấu hình của bao-gia: in theo số dòng và theo mẫu, tìm lần nâng cấp 1.4.1 lên 1.4.2, thay chuỗi một lần và mọi lần, vì sao dấu chấm trong mẫu khớp mọi ký tự, nhóm trong biểu thức chính quy để đổi định dạng dòng log, xoá dòng, sửa file tại chỗ có bản sao lưu và chỉ sửa trong một khối."
category: devops-linux
level: intermediate
tags: [linux, lnx-014, sed, regex, ssh-studio, toi-noi]
language: vi
---

Nhóm triển khai sắp chuyển `bao-gia` sang thư mục `/srv/toinoi/bao-gia` mà An dựng ở Phần 1. File cấu hình có những đường dẫn trỏ về `/opt/toinoi/bao-gia`, mức log cần hạ xuống `Warning`, và trên server thật có mười hai bản cấu hình như vậy, một cho mỗi môi trường. Mở từng file bằng vim thì được, nhưng mười hai lần sửa tay là mười hai cơ hội gõ nhầm.

`sed` (*stream editor*) là vim không có màn hình: bạn viết lệnh sửa một lần, `sed` áp nó lên từng dòng của bao nhiêu file cũng được, và kết quả lặp lại y hệt mỗi lần chạy. Lệnh `:s` mà bạn đã dùng trong vim ở [bài trước](/articles/linux-vim-hang-ngay) là cùng một ngôn ngữ.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-014 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-vim-hang-ngay">LNX-012</a>, <a href="/articles/linux-cut-sort-uniq-tr">LNX-013</a>, phần biểu thức chính quy của <a href="/articles/linux-tim-kiem-find-grep">LNX-009</a>. Trạng thái đầu bài: <code>npm run setup -- 14</code>. Sau bài này, bạn in được đúng đoạn cần xem trong một file dài, thay chuỗi có kiểm soát, đổi định dạng dữ liệu bằng nhóm regex và sửa file cấu hình tại chỗ mà vẫn có đường lui.</p>
</div>

## 1. sed làm việc thế nào

```text
sed [-n] 'ĐỊA_CHỈ LỆNH' FILE…
```

`sed` đọc từng dòng vào một vùng nhớ, áp các lệnh lên dòng đó nếu dòng khớp **địa chỉ**, rồi in vùng nhớ ra stdout và chuyển sang dòng kế. Mặc định file gốc **không bị đụng tới**: `sed` chỉ in kết quả. Không có địa chỉ thì lệnh áp lên mọi dòng.

| Địa chỉ | Chọn |
|---|---|
| `5` | Dòng 5 |
| `$` | Dòng cuối |
| `1,3` | Dòng 1 tới 3 |
| `/mẫu/` | Các dòng khớp biểu thức chính quy |
| `/A/,/B/` | Từ dòng khớp A tới dòng khớp B kế tiếp |
| `/A/,+2` | Dòng khớp A và hai dòng sau nó (GNU sed) |

Ba lệnh dùng nhiều nhất: `p` (in), `d` (xoá khỏi kết quả), `s` (thay).

## 2. In theo dòng và theo mẫu

```text
an@toinoi-app-01:~/tai-lieu$ sed -n '1,3p' /var/log/toinoi/bao-gia.log
2026-11-21 06:00:00 INFO  bao-gia 1.4.2 khoi dong, cong 8080, moi truong production
2026-11-21 06:00:00 INFO  nap bang gia: 4 loai dich vu tu /opt/toinoi/bao-gia/bang-gia.csv
2026-11-21 06:01:45 INFO  q-100001 dich_vu=delivery km=23.8 gia=82000 178 ms
an@toinoi-app-01:~/tai-lieu$ sed -n '$p' /var/log/toinoi/bao-gia.log
2026-11-21 12:17:05 INFO  q-100360 dich_vu=car4 km=14.3 gia=77000 59 ms
an@toinoi-app-01:~/tai-lieu$ sed -n '/SIGTERM/,+2p' /var/log/toinoi/bao-gia.log.1
2026-11-20 09:15:41 INFO  nhan tin hieu SIGTERM, dung bao-gia 1.4.1
2026-11-20 09:16:21 INFO  bao-gia 1.4.2 khoi dong, cong 8080, moi truong production
2026-11-20 09:16:21 INFO  nap bang gia: 4 loai dich vu tu /opt/toinoi/bao-gia/bang-gia.csv
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sed-sua-van-ban/ssh-studio-04.webp" alt="Terminal của SSH Studio trong ~/tai-lieu: sed -n '1,3p' in ba dòng đầu log, sed -n '$p' in dòng cuối, sed -n '/SIGTERM/,+2p' in đoạn bao-gia dừng rồi khởi động lại ngày 20/11" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sed -n</code> in theo số dòng và theo mẫu</figcaption></figure>

**`-n`** tắt việc tự in mọi dòng, nên chỉ những dòng được lệnh `p` in ra mới xuất hiện. `sed -n '1,3p'` là `head -n 3`, `sed -n '$p'` là `tail -n 1`; lợi thế của `sed` là in được **khoảng ở giữa** (`sed -n '120,140p'`) và in **theo mẫu**.

Lệnh thứ ba tìm ra lần nâng cấp mà [LNX-013](/articles/linux-cut-sort-uniq-tr) nhắc tới: 09:15:41 ngày 20/11, bản 1.4.1 nhận tín hiệu dừng; 40 giây sau, bản 1.4.2 khởi động và nạp bảng giá. Địa chỉ `/SIGTERM/,+2` in dòng khớp cùng hai dòng sau nó, đủ để thấy chuyện gì xảy ra quanh một sự kiện.

## 3. Thay chuỗi: lệnh `s`

```text
an@toinoi-app-01:~/tai-lieu$ echo "bao-gia 1.4.1 dang chay" | sed 's/1.4.1/1.4.2/'
bao-gia 1.4.2 dang chay
an@toinoi-app-01:~/tai-lieu$ echo "km=2.5 km=3.0" | sed 's/km=/quang_duong=/'
quang_duong=2.5 km=3.0
an@toinoi-app-01:~/tai-lieu$ echo "km=2.5 km=3.0" | sed 's/km=/quang_duong=/g'
quang_duong=2.5 quang_duong=3.0
an@toinoi-app-01:~/tai-lieu$ echo "1x4y1" | sed 's/1.4.1/1.4.2/'
1.4.2
an@toinoi-app-01:~/tai-lieu$ echo "1x4y1" | sed 's/1\.4\.1/1.4.2/'
1x4y1
```

`s/MẪU/THAY/CỜ`:

- Không có cờ: chỉ thay **lần khớp đầu tiên** trên mỗi dòng. `km=2.5 km=3.0` chỉ đổi chữ `km=` đầu.
- Cờ **`g`**: thay mọi lần khớp trên dòng.
- Cờ `2`: chỉ thay lần khớp thứ hai. Cờ `I`: không phân biệt hoa thường.

Hai lệnh cuối là cái bẫy kinh điển. Mẫu của `sed` là **biểu thức chính quy**, và dấu `.` trong regex khớp **một ký tự bất kỳ**. Mẫu `1.4.1` khớp cả `1x4y1`, nên chuỗi chẳng liên quan gì cũng bị «nâng cấp». Viết `1\.4\.1` để dấu chấm chỉ khớp dấu chấm thật. Trên một file cấu hình có địa chỉ IP, phiên bản, tên miền, quên dấu `\` trước dấu chấm là cách thay nhầm mà không ai thấy.

Phần THAY không phải regex, nhưng có hai ký tự đặc biệt: `&` là toàn bộ chuỗi vừa khớp (`s/[0-9]* ms/[&]/` bọc số mili giây trong ngoặc vuông), và `\1`…`\9` là các nhóm (mục 4). Muốn chữ `&` thật, viết `\&`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sed-sua-van-ban/ssh-studio-01.webp" alt="Terminal của SSH Studio: các ví dụ sed s: đổi 1.4.1 thành 1.4.2, đổi km= một lần và với cờ g đổi mọi chỗ, dấu chấm không thoát khớp nhầm 1x4y1, thoát bằng \. thì không" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · lệnh <code>s</code>, cờ <code>g</code> và dấu chấm phải thoát</figcaption></figure>

## 4. Nhóm: đổi định dạng dòng log

Nhóm sản phẩm muốn một danh sách yêu cầu chậm dạng `mã thời_điểm_ISO thời_gian`, để dán vào bảng tính:

```text
an@toinoi-app-01:~/tai-lieu$ grep -m 3 ' WARN ' /var/log/toinoi/bao-gia.log
2026-11-21 06:02:30 WARN  q-100002 dich_vu=bike km=16.9 bao gia cham: 1406 ms
2026-11-21 06:18:02 WARN  q-100017 dich_vu=delivery km=18.6 bao gia cham: 2295 ms
2026-11-21 06:40:06 WARN  q-100037 dich_vu=bike km=2.2 bao gia cham: 1548 ms
an@toinoi-app-01:~/tai-lieu$ grep -m 3 ' WARN ' /var/log/toinoi/bao-gia.log | sed -E 's/^([0-9-]+) ([0-9:]+) +WARN +(q-[0-9]+) .* ([0-9]+) ms$/\3 \1T\2 \4ms/'
q-100002 2026-11-21T06:02:30 1406ms
q-100017 2026-11-21T06:18:02 2295ms
q-100037 2026-11-21T06:40:06 1548ms
```

Đọc biểu thức từ trái sang phải:

| Phần | Khớp với | Nhóm |
|---|---|---|
| `^([0-9-]+)` | Ngày ở đầu dòng | `\1` |
| ` ([0-9:]+)` | Giờ | `\2` |
| ` +WARN +` | Chữ `WARN` với một hoặc nhiều dấu cách hai bên | |
| `(q-[0-9]+)` | Mã yêu cầu | `\3` |
| ` .* ` | Mọi thứ ở giữa | |
| `([0-9]+) ms$` | Số mili giây ở cuối dòng | `\4` |

**`-E`** bật biểu thức chính quy mở rộng, như `grep -E`: ngoặc `( )` tạo nhóm và `+` là «một lần trở lên» mà không cần dấu `\`. Phần thay ghép các nhóm theo thứ tự mới. Dòng nào không khớp mẫu thì đi qua `sed` nguyên vẹn, nên nên lọc trước bằng `grep` như ở trên.

Viết regex như vậy nên làm từng khúc: bắt đầu với `s/^([0-9-]+).*/\1/`, xem kết quả, rồi thêm nhóm kế. Viết cả dòng một lần rồi mới chạy thì lúc sai rất khó biết sai ở đâu.

Không có `-E` (regex cơ bản), nhóm viết là `\( \)`, và `+` phải viết `\+`. Lệnh sau dùng dạng cơ bản để lấy số mili giây của mọi yêu cầu chậm trong ba ngày:

```text
an@toinoi-app-01:~/tai-lieu$ zcat -f /var/log/toinoi/bao-gia.log* | sed -n 's/.*cham: \([0-9]*\) ms$/\1/p' | sort -n | tail -n 3
2671
2671
2685
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sed-sua-van-ban/ssh-studio-05.webp" alt="Terminal của SSH Studio: ba dòng WARN gốc, sed -E với ba nhóm đổi thành q-100002 2026-11-21T06:02:30 1406ms, sed -n …p lấy ba thời gian chậm nhất 2671, 2671, 2685" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · nhóm <code>\(…\)</code> và <code>\1</code>: đổi định dạng dòng log</figcaption></figure>

Mẫu `-n 's/…/…/p'` rất đáng nhớ: chỉ in những dòng **đã được thay thành công**. Một lệnh `sed` vừa lọc vừa trích, thay cho cặp `grep | sed`.

### Xoá dòng

```text
an@toinoi-app-01:~/tai-lieu$ sed -e '/^#/d' -e '/^$/d' /opt/toinoi/bao-gia/kiem-tra-cau-hinh.sh
set -eu
thu_muc="$(cd "$(dirname "$0")" && pwd)"
bang_gia="$thu_muc/bang-gia.csv"
if [ ! -r "$bang_gia" ]; then
  echo "LOI: khong doc duoc $bang_gia" >&2
  exit 1
fi
so_dich_vu=$(($(wc -l < "$bang_gia") - 1))
echo "bao-gia: bang gia co $so_dich_vu loai dich vu"
echo "bao-gia: cau hinh hop le"
```

`d` xoá dòng khỏi kết quả. `-e` cho phép viết nhiều lệnh; lệnh được áp theo thứ tự lên từng dòng. `/^#/d` bỏ dòng chú thích (kể cả dòng shebang `#!/usr/bin/env bash`, vì nó cũng bắt đầu bằng `#`), `/^$/d` bỏ dòng trống. Cách nhanh để đọc phần lõi của một file cấu hình nhiều chú thích như `/etc/ssh/sshd_config`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sed-sua-van-ban/ssh-studio-02.webp" alt="Terminal của SSH Studio: sed -e /^#/d -e /^$/d in kiem-tra-cau-hinh.sh không còn dòng chú thích và dòng trống" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · xoá dòng theo mẫu với lệnh <code>d</code></figcaption></figure>

## 5. Sửa file tại chỗ, có đường lui

```text
an@toinoi-app-01:~/tai-lieu$ cp /opt/toinoi/bao-gia/appsettings.json appsettings.json
an@toinoi-app-01:~/tai-lieu$ sed -i.bak 's/"Level": "Information"/"Level": "Warning"/' appsettings.json
an@toinoi-app-01:~/tai-lieu$ ls appsettings.json*
appsettings.json  appsettings.json.bak
an@toinoi-app-01:~/tai-lieu$ diff appsettings.json.bak appsettings.json
13c13
<     "Level": "Information"
---
>     "Level": "Warning"
an@toinoi-app-01:~/tai-lieu$ sed -i 's#/opt/toinoi/bao-gia#/srv/toinoi/bao-gia/current#' appsettings.json
an@toinoi-app-01:~/tai-lieu$ grep -n current appsettings.json
8:    "RuleFile": "/srv/toinoi/bao-gia/current/bang-gia.csv",
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sed-sua-van-ban/ssh-studio-06.webp" alt="Terminal của SSH Studio: sed -i.bak đổi Level thành Warning, ls thấy appsettings.json.bak, diff 13c13, sed -i với dấu # làm phân cách đổi đường dẫn, grep -n current thấy dòng 8" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>sed -i.bak</code>: sửa tại chỗ, có bản lưu để lui</figcaption></figure>

**`-i`** (*in-place*) ghi kết quả đè lên chính file. **`-i.bak`** làm như vậy nhưng giữ bản gốc với đuôi `.bak`, rồi `diff` cho biết đúng những gì đã đổi.

Lệnh thứ hai dùng dấu `#` làm phân cách, vì cả mẫu lẫn phần thay đều chứa dấu `/`. Giống `:s` trong vim, ký tự ngay sau `s` là dấu phân cách.

Vài nguyên tắc khi dùng `-i` trên server:

- **Chạy không có `-i` trước**, đọc kết quả (hoặc `| diff file -`), rồi mới thêm `-i`.
- **Giữ bản sao lưu** bằng `-i.bak`, ít nhất ở lần chạy đầu.
- **`-i` thay file bằng một file mới**: chủ sở hữu và quyền được giữ khi bạn là chủ của file, nhưng symlink bị thay bằng file thường. Sửa file đích, đừng sửa qua symlink (`--follow-symlinks` nếu thật cần).
- Trên macOS, `sed -i` cần một đối số (`sed -i ''`). Bài này dùng GNU sed của Ubuntu.

### Chỉ sửa trong một khối

```text
an@toinoi-app-01:~/tai-lieu$ sed -n '/"Fare"/,/}/p' appsettings.json
  "Fare": {
    "RuleFile": "/srv/toinoi/bao-gia/current/bang-gia.csv",
    "RoundTo": 1000
  },
an@toinoi-app-01:~/tai-lieu$ sed -i '/"Fare"/,/}/ s/1000/500/' appsettings.json
an@toinoi-app-01:~/tai-lieu$ diff appsettings.json.bak appsettings.json
8,9c8,9
<     "RuleFile": "/opt/toinoi/bao-gia/bang-gia.csv",
<     "RoundTo": 1000
---
>     "RuleFile": "/srv/toinoi/bao-gia/current/bang-gia.csv",
>     "RoundTo": 500
13c13
<     "Level": "Information"
---
>     "Level": "Warning"
```

Địa chỉ dạng khoảng `/"Fare"/,/}/` chọn từ dòng có `"Fare"` tới dấu `}` đầu tiên sau nó. Ghép với lệnh `s`, việc thay `1000` thành `500` chỉ xảy ra trong khối `Fare`, dù con số `1000` có xuất hiện ở chỗ khác trong file. Luôn in khối đó bằng `-n …p` trước để chắc địa chỉ chọn đúng.

Cách này hợp với những file có cấu trúc đơn giản. Với JSON hay YAML phức tạp (khối lồng nhau, dấu `}` trong chuỗi), hãy dùng công cụ hiểu cấu trúc đó, như `jq` cho JSON. `sed` hiểu dòng, không hiểu JSON.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-sed-sua-van-ban/ssh-studio-03.webp" alt="Terminal của SSH Studio: sed -n in khối Fare của appsettings.json, sed -i chỉ đổi 1000 thành 500 trong khối đó, diff với bản .bak còn ba khối thay đổi" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · chỉ sửa trong một khối địa chỉ <code>/Fare/,/}/</code></figcaption></figure>

## 6. sed trong vim, vim trong sed

| Trong vim | Với sed |
|---|---|
| `:s/a/b/` | `sed 's/a/b/'` (mỗi dòng, lần đầu) |
| `:%s/a/b/g` | `sed 's/a/b/g'` |
| `:2,4s/a/b/` | `sed '2,4s/a/b/'` |
| `:g/^#/d` | `sed '/^#/d'` |
| `:%s#/opt#/srv#` | `sed 's#/opt#/srv#'` |

Sửa một file, nhìn thấy kết quả ngay: vim. Sửa nhiều file, hoặc cần lặp lại chính xác (trong script, trong tài liệu vận hành): sed.

## 7. Lỗi thường gặp

**Quên `-n` khi dùng `p`.** Mọi dòng bị in hai lần: một lần do `p`, một lần do in tự động.

**Dấu chấm không có `\`.** Khớp mọi ký tự. Xem mục 3.

**`sed -i` không có bản sao lưu ở lần đầu.** Một lỗi trong biểu thức là mất nội dung.

**Mẫu có dấu `/`.** Đổi dấu phân cách.

**Dùng nháy kép quanh biểu thức `sed`.** Shell thay `$` bên trong: `sed "s/$/;/"` vẫn chạy nhưng `"s/$HOME/…/"` thì khác hẳn. Dùng nháy đơn, trừ khi bạn chủ ý muốn chèn biến.

**`sed -E` và `sed` cơ bản lẫn lộn.** `+`, `?`, `( )` chỉ có nghĩa đặc biệt khi có `-E`. Không có `-E`, viết `\+`, `\?`, `\( \)`.

## Bài tập

1. In các dòng log từ 07:00:00 tới 07:10:00 ngày 21/11 bằng một địa chỉ dạng khoảng.
2. Đổi mọi `dich_vu=delivery` thành `dich_vu=giao-hang` trong kết quả của `bao-gia.log` (không sửa file), rồi đếm số dòng đã đổi bằng `sed -n 's/…/…/p' | wc -l`.
3. Viết một lệnh `sed -E` biến dòng `INFO … dich_vu=car4 km=14.3 gia=77000 59 ms` thành `car4;14.3;77000`. Áp lên cả file và lưu ra `~/tai-lieu/gia.csv`.
4. Từ `~/tai-lieu/appsettings.json.bak`, tạo `appsettings.staging.json` có `"Environment": "staging"` và cổng `8090` chỉ bằng một lệnh `sed` có hai `-e`.
5. Chạy `sed -n '/^\s*"Port"/p' /opt/toinoi/bao-gia/appsettings.json`. `\s` khớp với gì?

## Đọc tiếp

[LNX-015 · awk: xử lý log có cột](/articles/linux-awk-xu-ly-log): khi cần tính toán (tổng, trung bình, gom theo khoá), `awk` làm được điều mà `cut`, `sort`, `sed` ghép lại cũng khó làm.
