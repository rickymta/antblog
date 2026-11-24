---
slug: linux-awk-xu-ly-log
title: "awk trên Linux: lọc, tính tổng, trung bình và gom theo khoá trên log có cột"
summary: "awk đọc từng dòng, tự tách thành các trường theo khoảng trắng và cho bạn viết điều kiện, phép tính, mảng ngay trên dòng lệnh. Bài dùng log của bao-gia để lọc theo mức log và thời gian, tính thời gian báo giá trung bình, gom theo dịch vụ bằng mảng, đếm hai loại lỗi theo ngày, tính giá cước 10 km từ file CSV với -F và -v, rồi viết chương trình tom-tat.awk tóm tắt cả ba ngày."
category: devops-linux
level: intermediate
tags: [linux, lnx-015, awk, gawk, log, csv, ssh-studio, toi-noi]
language: vi
---

Bình hỏi một câu mà `grep | sort | uniq` không trả lời được: «Thời gian báo giá trung bình của từng loại dịch vụ là bao nhiêu? Xe 7 chỗ có chậm hơn không?»

Đếm thì `uniq -c` làm được. **Cộng** và **chia** thì không. Đây là lúc dùng `awk`: một ngôn ngữ lập trình nhỏ được thiết kế cho đúng loại dữ liệu này, văn bản chia cột. Một dòng `awk` thay được một script Python ngắn, và có sẵn trên mọi server Linux.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-015 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-cut-sort-uniq-tr">LNX-013</a>, <a href="/articles/linux-sed-sua-van-ban">LNX-014</a>. Trạng thái đầu bài: <code>npm run setup -- 15</code>. Sau bài này, bạn lọc dòng theo điều kiện trên từng cột, tính tổng, trung bình, gom nhóm bằng mảng, đọc file CSV, và có chương trình <code>~/bin/tom-tat.awk</code> mà script ở bài thực hành sẽ dùng. Lab dùng GNU awk (<code>gawk</code>).</p>
</div>

## 1. Cấu trúc của awk

```text
awk 'MẪU { HÀNH_ĐỘNG }' FILE…
```

Với mỗi dòng đầu vào, `awk` tách dòng thành các **trường**, kiểm tra **mẫu**, và nếu đúng thì chạy **hành động**. Thiếu mẫu thì hành động chạy cho mọi dòng; thiếu hành động thì mặc định là in cả dòng. Hai mẫu đặc biệt: `BEGIN` chạy trước dòng đầu tiên, `END` chạy sau dòng cuối cùng.

| Biến | Ý nghĩa |
|---|---|
| `$0` | Cả dòng |
| `$1`, `$2`, … | Trường thứ 1, 2, … |
| `NF` | Số trường của dòng; `$NF` là trường cuối, `$(NF-1)` là trường áp chót |
| `NR` | Số thứ tự dòng (đếm qua mọi file) |
| `FS` | Ký tự phân cách trường, đặt bằng `-F` |

## 2. Trường: awk tự tách theo khoảng trắng

```text
an@toinoi-app-01:/var/log/toinoi$ head -n 4 bao-gia.log | awk '{ print $3, $4 }'
INFO bao-gia
INFO nap
INFO q-100001
WARN q-100002
an@toinoi-app-01:/var/log/toinoi$ head -n 4 bao-gia.log | awk '{ print NF, $NF }'
12 production
12 /opt/toinoi/bao-gia/bang-gia.csv
9 ms
11 ms
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-awk-xu-ly-log/ssh-studio-05.webp" alt="Terminal của SSH Studio trong /var/log/toinoi: awk '{ print $3, $4 }' in đúng mức và mã yêu cầu dù dòng có dấu cách kép, awk '{ print NF, $NF }' in số trường và trường cuối" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · awk tự tách trường theo khoảng trắng; <code>NF</code> và <code>$NF</code></figcaption></figure>

So với [LNX-013](/articles/linux-cut-sort-uniq-tr): cũng bốn dòng đó, `cut -d' ' -f3,4` cho kết quả lệch vì dấu cách kép, còn `awk` cho kết quả đúng ngay. Mặc định `awk` coi **mọi chuỗi khoảng trắng** là một dấu phân cách, và bỏ khoảng trắng ở đầu dòng. Đây là lý do chính khiến người làm vận hành dùng `awk '{print $5}'` thay cho `cut` khi đọc log và kết quả của các lệnh như `ls -l`, `ps`, `df`.

`print $3, $4`: dấu phẩy giữa hai giá trị in ra một dấu cách. Viết liền `print $3 $4` thì hai giá trị dính vào nhau.

`NF` khác nhau theo loại dòng: dòng khởi động có 12 trường, dòng `INFO` của yêu cầu có 9, dòng `WARN` có 11. Nhưng mọi dòng yêu cầu thành công hoặc chậm đều **kết thúc bằng** `số ms`: `$NF` là `ms`, `$(NF-1)` là số mili giây. Đếm từ cuối dòng là mẹo quen thuộc khi số trường thay đổi.

## 3. Lọc theo điều kiện

```text
an@toinoi-app-01:/var/log/toinoi$ awk '$3 == "ERROR"' bao-gia.log | head -n 2
2026-11-21 06:37:58 ERROR q-100034 dich_vu=car7 khong lay duoc quang duong: timeout sau 3000 ms goi ban-do
2026-11-21 06:41:18 ERROR q-100038 dich_vu=car4 khong lay duoc quang duong: timeout sau 3000 ms goi ban-do
an@toinoi-app-01:/var/log/toinoi$ awk '$3 == "ERROR" { n++ } END { print n, "lỗi" }' bao-gia.log
17 lỗi
an@toinoi-app-01:/var/log/toinoi$ awk '$NF == "ms" && $(NF-1) > 2500 { print $2, $4, $(NF-1) }' bao-gia.log
09:12:56 q-100180 2637
09:56:45 q-100226 2615
11:21:33 q-100311 2671
11:27:22 q-100315 2570
```

- `$3 == "ERROR"`: mẫu là một phép so sánh. Không có hành động, `awk` in cả dòng: một `grep` chỉ khớp ở đúng cột 3, không khớp nhầm chữ `ERROR` nằm ở chỗ khác.
- `{ n++ } END { print n, "lỗi" }`: biến tự khởi tạo bằng 0 (hoặc chuỗi rỗng) lần đầu dùng. Cộng dồn ở mỗi dòng khớp, in ở `END`.
- `$NF == "ms" && $(NF-1) > 2500`: hai điều kiện nối bằng `&&` («và»; `||` là «hoặc», `!` là «không»). `awk` tự hiểu `$(NF-1)` là **số** khi so sánh với số, nên `2637 > 2500` đúng theo giá trị, không theo chữ như `sort` thiếu `-n`.

Lệnh cuối liệt kê bốn yêu cầu chậm hơn 2,5 giây trong ngày 21/11: giờ, mã yêu cầu, thời gian.

So sánh chuỗi dùng `==` với nháy kép quanh chuỗi; so khớp regex dùng `~`: `$5 ~ /car/` (trường 5 chứa `car`), `$0 !~ /INFO/` (dòng không chứa `INFO`). Mẫu chỉ có regex, `awk '/bi khoa/'`, tương đương `grep 'bi khoa'`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-awk-xu-ly-log/ssh-studio-01.webp" alt="Terminal của SSH Studio: awk '$3 == &quot;ERROR&quot;' in hai lỗi đầu, đếm 17 lỗi bằng n++ và END, lọc yêu cầu có thời gian trên 2500 ms in giờ, mã và số ms" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · awk lọc theo điều kiện trên trường</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-awk-xu-ly-log/ssh-studio-06.webp" alt="Terminal của SSH Studio: awk cộng thời gian của các yêu cầu INFO và chia trong END: 319 yêu cầu, trung bình 95,3 ms" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · tổng và trung bình trong khối <code>END</code></figcaption></figure>

## 4. Tổng và trung bình

```text
an@toinoi-app-01:/var/log/toinoi$ awk '$3 == "INFO" && $NF == "ms" { tong += $(NF-1); n++ } END { printf "%d yêu cầu, trung bình %.1f ms\n", n, tong / n }' bao-gia.log
319 yêu cầu, trung bình 95.3 ms
```

Chỉ lấy dòng `INFO` kết thúc bằng `ms`, tức các yêu cầu thành công (bỏ hai dòng khởi động), cộng dồn thời gian vào `tong`, đếm vào `n`, chia ở cuối. 319 yêu cầu thành công trung bình mất 95,3 ms.

`printf` định dạng như trong C: `%d` số nguyên, `%.1f` số thực một chữ số thập phân, `%s` chuỗi, `%-9s` chuỗi căn trái rộng 9 ký tự, `\n` xuống dòng. Khác `print`, `printf` không tự xuống dòng.

## 5. Mảng: gom theo khoá

Câu hỏi của Bình:

```text
an@toinoi-app-01:/var/log/toinoi$ awk '$3 == "INFO" && $NF == "ms" { split($5, a, "="); n[a[2]]++; t[a[2]] += $(NF-1) } END { for (dv in n) printf "%-9s %4d %6.1f ms\n", dv, n[dv], t[dv] / n[dv] }' bao-gia.log | sort
bike        73   97.0 ms
car4        89   91.6 ms
car7        84  100.1 ms
delivery    73   92.4 ms
```

Mảng trong `awk` là **mảng kết hợp**: chỉ số là chuỗi bất kỳ.

1. `split($5, a, "=")` tách `dich_vu=car7` thành `a[1] = "dich_vu"`, `a[2] = "car7"`.
2. `n[a[2]]++` đếm số yêu cầu theo dịch vụ, `t[a[2]] += …` cộng thời gian theo dịch vụ.
3. Ở `END`, `for (dv in n)` duyệt mọi khoá. Thứ tự duyệt **không xác định**, nên nối `| sort`.

Trả lời Bình: xe 7 chỗ chậm nhất, trung bình 100,1 ms, so với 91,6 ms của xe 4 chỗ. Chênh 10% chưa phải chuyện lớn, nhưng giờ bạn đã có một con số thay cho cảm giác.

### Đếm hai loại lỗi theo ngày

```text
an@toinoi-app-01:/var/log/toinoi$ zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | awk '$3 == "ERROR" { loai = ($0 ~ /bi khoa/) ? "bang-gia" : "ban-do"; n[$1 " " loai]++ } END { for (k in n) print k, n[k] }' | sort
2026-11-19 ban-do 18
2026-11-20 ban-do 19
2026-11-20 bang-gia 7
2026-11-21 ban-do 17
```

`điều_kiện ? A : B` chọn A hoặc B. Khoá của mảng ghép hai giá trị: `$1 " " loai` (hai chuỗi đặt cạnh nhau là phép nối chuỗi trong `awk`). Kết quả khớp với những gì [LNX-013](/articles/linux-cut-sort-uniq-tr) tìm ra bằng ba lệnh: lỗi khoá bảng giá chỉ có ở ngày 20/11, còn lỗi bản đồ thì ngày nào cũng có, 17 tới 19 lỗi mỗi ngày.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-awk-xu-ly-log/ssh-studio-02.webp" alt="Terminal của SSH Studio: mảng gom theo dịch vụ in số yêu cầu và thời gian trung bình của bike, car4, car7, delivery; đếm hai loại lỗi bản đồ và bảng giá theo ngày" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · mảng của awk: gom theo khoá</figcaption></figure>

## 6. File CSV: `-F` và `-v`

```text
an@toinoi-app-01:/var/log/toinoi$ awk -F, 'NR > 1 { print $1, $3 }' /opt/toinoi/bao-gia/bang-gia.csv
bike 8000
car4 12000
car7 15000
delivery 10000
an@toinoi-app-01:/var/log/toinoi$ awk -F, -v km=10 'NR > 1 { gia = $3 + $4 * km; if (gia < $5) gia = $5; printf "%-9s %7d đ\n", $1, gia }' /opt/toinoi/bao-gia/bang-gia.csv
bike        33000 đ
car4        57000 đ
car7        70000 đ
delivery    40000 đ
```

- **`-F,`** đặt dấu phẩy làm phân cách trường.
- **`NR > 1`** bỏ dòng tiêu đề.
- **`-v km=10`** truyền giá trị từ shell vào biến `km` của chương trình. Đây là cách đúng để đưa biến shell vào `awk`: `-v km="$so_km"`. Đừng chèn `$so_km` vào giữa chương trình nằm trong nháy đơn.
- `if (…) …` hoạt động như trong C.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-awk-xu-ly-log/ssh-studio-07.webp" alt="Terminal của SSH Studio: awk -F, in mã dịch vụ và giá mở cửa từ bang-gia.csv, awk -F, -v km=10 tính giá cước 10 km cho từng loại với giá tối thiểu" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · CSV với <code>-F,</code>; truyền giá trị từ shell bằng <code>-v</code></figcaption></figure>

Lệnh thứ hai tính lại giá cước 10 km theo đúng công thức của dịch vụ báo giá: giá mở cửa cộng đơn giá mỗi km nhân số km, nhưng không thấp hơn giá tối thiểu. Một dòng lệnh để kiểm tra bảng giá trước khi triển khai.

## 7. Chương trình awk trong file

Chương trình dài hơn một dòng nên nằm trong file, chạy bằng `awk -f`. An viết `tom-tat.awk` vào `~/bin`:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-awk-xu-ly-log/ssh-studio-03.webp" alt="vim mở ~/bin/tom-tat.awk với chú thích ở đầu, khối mẫu $4 ~ /^q-/ đếm tổng, lỗi, chậm và cộng thời gian theo ngày, và khối END in tiêu đề rồi một dòng cho mỗi ngày qua sort" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>~/bin/tom-tat.awk</code> trong vim</figcaption></figure>

```awk
$4 ~ /^q-/ {
  ngay = $1
  tong[ngay]++
  if ($3 == "ERROR") loi[ngay]++
  else if ($3 == "WARN") cham[ngay]++
  else { ms[ngay] += $(NF - 1); ok[ngay]++ }
}

END {
  printf "%-10s %8s %5s %5s %7s\n", "ngày", "yêu cầu", "lỗi", "chậm", "tb ms"
  fflush()
  for (ngay in tong)
    printf "%-10s %8d %5d %5d %7.1f\n", ngay, tong[ngay], loi[ngay], cham[ngay], ms[ngay] / ok[ngay] | "sort"
}
```

- Mẫu `$4 ~ /^q-/` chỉ nhận dòng có mã yêu cầu ở cột 4, tự loại các dòng khởi động, dừng dịch vụ.
- Mỗi ngày là một khoá của năm mảng.
- `printf … | "sort"`: `awk` mở một tiến trình `sort` và đưa các dòng vào đó, để dòng tiêu đề đứng trên cùng còn các ngày được xếp thứ tự. `fflush()` đẩy dòng tiêu đề ra trước khi `sort` in kết quả.
- Dòng bắt đầu bằng `#` là chú thích.

```text
an@toinoi-app-01:/var/log/toinoi$ zcat -f bao-gia.log.2.gz bao-gia.log.1 bao-gia.log | awk -f ~/bin/tom-tat.awk
ngày        yêu cầu   lỗi  chậm   tb ms
2026-11-19      300    18    25    98.7
2026-11-20      300    26    19   101.2
2026-11-21      360    17    24    95.3
```

Một bảng tóm tắt ba ngày. Số liệu khớp với các bài trước (ngày 21/11: 360 yêu cầu, 17 lỗi, 24 chậm, trung bình 95,3 ms), và ngày 20/11 nổi bật với 26 lỗi. Bài thực hành [LNX-020](/articles/linux-thuc-hanh-script-sao-luu-log) dùng lại chính chương trình này trong báo cáo hằng ngày.

Cột tiêu đề tiếng Việt thẳng hàng vì `gawk` đếm **ký tự** khi `LANG` là UTF-8. Với `LANG=C`, nó đếm byte và chữ có dấu làm lệch cột. Điều này sẽ quay lại ở bài thực hành, khi script chạy dưới cron.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-awk-xu-ly-log/ssh-studio-04.webp" alt="Terminal của SSH Studio: zcat -f ba file log | awk -f ~/bin/tom-tat.awk in bảng mỗi ngày một dòng: số yêu cầu, lỗi, chậm và thời gian trung bình" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · <code>awk -f</code>: chương trình trong file, chạy trên ba ngày log</figcaption></figure>

## 8. Khi nào dùng công cụ nào

| Việc | Công cụ hợp nhất |
|---|---|
| Lọc dòng chứa một chuỗi | `grep` |
| Thay chuỗi | `sed` |
| Lấy một cột, khoảng trắng đều | `cut` |
| Lấy cột khi khoảng trắng không đều | `awk '{print $N}'` |
| Đếm số lần xuất hiện | `sort \| uniq -c` |
| Điều kiện trên một cột cụ thể | `awk '$N == …'` |
| Tổng, trung bình, nhỏ nhất, lớn nhất | `awk` |
| Gom nhóm có tính toán | `awk` với mảng |
| Logic dài hơn 10 dòng, cần thư viện | Một ngôn ngữ lập trình thật |

## 9. Lỗi thường gặp

**Nháy kép quanh chương trình awk.** Shell thay `$3` bằng đối số thứ ba của shell (thường là rỗng). Luôn dùng nháy đơn.

**Chèn biến shell vào chương trình.** Dùng `-v ten="$bien"`.

**Quên dấu phẩy trong `print`.** `print $1 $2` dính hai giá trị.

**Chia cho 0 ở `END`.** Khi không có dòng nào khớp, `tong / n` báo lỗi `division by zero`. Kiểm tra `if (n > 0)` trước.

**Tin vào thứ tự của `for (k in a)`.** Không có thứ tự; nối `| sort` hoặc dùng `PROCINFO["sorted_in"]` của gawk.

**`awk` trên file có dấu phẩy trong trường (CSV có nháy).** `-F,` không hiểu `"Hà Nội, Việt Nam"`. Với CSV phức tạp, dùng công cụ chuyên cho CSV.

## Bài tập

1. Tìm thời gian báo giá nhỏ nhất và lớn nhất của các yêu cầu thành công ngày 21/11 bằng một lệnh `awk`.
2. Tính tổng doanh thu dự kiến (`gia=`) theo dịch vụ của ngày 21/11. Gợi ý: `split($7, g, "=")`.
3. Với `-v km=…`, viết lệnh in giá cước của 3, 10 và 25 km cho xe máy. (Chạy ba lần, hoặc dùng vòng `for` trong `BEGIN`.)
4. Sửa `tom-tat.awk` để có thêm cột «tỉ lệ lỗi %». Chạy lại trên ba ngày.
5. `ls -l /var/log/toinoi | awk 'NR > 1 { tong += $5 } END { print tong }'` in gì? Kiểm tra bằng `du -b`.

## Đọc tiếp

[LNX-016 · Shell script đầu tiên](/articles/linux-shell-script-dau-tien): gói các lệnh vào một file có tên, có tham số và có mã thoát.
