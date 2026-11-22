---
slug: git-blame-lich-su-file-truy-vet
title: "Truy vết thay đổi: blame, lịch sử file và tìm commit theo nội dung code"
summary: "Bộ phận vận hành hỏi ai đổi đơn giá mỗi km của ô tô 4 chỗ, khi nào và vì sao. Bài này trả lời bằng git blame và Blame của GitStudio, đọc lịch sử một file kể cả khi file đã đổi tên (--follow), tìm commit làm xuất hiện hay biến mất một chuỗi bằng pickaxe -S và -G, theo dõi một khoảng dòng bằng log -L, và tránh bẫy dấu nháy của PowerShell."
category: git
level: advanced
tags: [git, git-009, git-blame, file-history, pickaxe, git-log-follow, git-log-l, gitstudio, toi-noi]
language: vi
---

Quay lại câu hỏi ở GIT-001: "Từ thứ Tư, giá ô tô 4 chỗ tăng 500 đ mỗi km. Ai đổi, đổi theo yêu cầu nào?". Lúc đó nhóm Tới Nơi phải lục tin nhắn. Giờ dịch vụ báo giá đã có lịch sử commit sạch, câu hỏi được trả lời trong vài giây, và còn trả lời được những câu khó hơn: "đơn giá 4.500 đ xuất hiện từ bao giờ, biến mất lúc nào?", "hàm `Quote` đã bị sửa những lần nào?", "tài liệu bảng giá có từ trước khi đổi tên không?".

Bài này là phần "thu hoạch" của Phần 1: mọi nỗ lực viết commit nhỏ, nội dung commit rõ ràng ở các bài trước được đền đáp ở đây.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-009 · Phần 1/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-ben-trong-object-ref-head">GIT-008</a>. Lab: <code>npm run p01 -- 09</code> (repository đủ mười commit). Sau bài này, bạn dùng được blame (lệnh và giao diện), lịch sử file kể cả qua lần đổi tên, pickaxe <code>-S</code>/<code>-G</code>, <code>git log -L</code>, và biết khi nào terminal làm sai lệnh vì dấu nháy.</p>
</div>

## 1. Blame: dòng này đến từ commit nào

`git blame` gắn mỗi dòng của file với **commit gần nhất đã thay đổi dòng đó**. Tên lệnh nghe như "đổ lỗi", nhưng công dụng thật là tìm *ngữ cảnh*: ai, khi nào, trong commit nào, để đọc tiếp nội dung commit và hỏi đúng người.

```sh
git blame -L 3,7 src/BaoGia/appsettings.json
```

```text
7ab5adc3 (Trần Minh 2026-11-02 09:10:00 +0700 3)     "Car4": {
7ab5adc3 (Trần Minh 2026-11-02 09:10:00 +0700 4)       "BaseFare": 12000,
d508151e (Lê Lan    2026-11-04 10:40:00 +0700 5)       "PerKm": 5000,
7ab5adc3 (Trần Minh 2026-11-02 09:10:00 +0700 6)       "MinimumFare": 25000
7ab5adc3 (Trần Minh 2026-11-02 09:10:00 +0700 7)     },
```

`-L 3,7` giới hạn dòng 3 tới 7. Mỗi dòng: mã hash commit, tác giả, thời điểm, số dòng, nội dung. Dòng 5 (`"PerKm": 5000`) đến từ commit `d508151` của Lan; các dòng còn lại từ commit Minh tạo file. Đọc commit đó để biết *vì sao*:

```sh
git show d508151
```

```text
commit d508151e625bcf6af2e0467fc3d7d757b1d1a376
Author: Lê Lan <lan@toinoi.test>
Date:   Wed Nov 4 10:40:00 2026 +0700

    Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ

    Theo bảng giá mới của bộ phận vận hành, áp dụng từ 01/12/2026.

diff --git a/src/BaoGia/appsettings.json b/src/BaoGia/appsettings.json
index 65a696e..ce82839 100644
--- a/src/BaoGia/appsettings.json
+++ b/src/BaoGia/appsettings.json
@@ -2,7 +2,7 @@
   "Fare": {
     "Car4": {
       "BaseFare": 12000,
-      "PerKm": 4500,
+      "PerKm": 5000,
       "MinimumFare": 25000
     },
     "Bike": {
```

Ai: Lê Lan. Khi nào: 04/11/2026 10:40. Vì sao: bảng giá mới của bộ phận vận hành, áp dụng từ 01/12/2026. Trước đó: 4.500 đ. Câu hỏi đầu bài đã có đủ lời đáp, trong một commit.

### Blame trên GitStudio

Mở file (từ chi tiết một commit, đánh dấu **Xem tất cả file** rồi bấm file; hoặc tìm file bằng Spotlight), rồi bấm **Blame** trên thanh công cụ của khung xem file. Cột trái hiện tác giả, thời gian và summary của commit cho từng *khối* dòng liên tiếp cùng một commit; bấm vào đó để nhảy tới commit.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-blame-lich-su-file-truy-vet/gitstudio-01.webp" alt="Chế độ Blame của GitStudio cho src/BaoGia/appsettings.json tại commit 89855d5: dòng 1 tới 4 và 6 trở đi gắn với Trần Minh Thêm hàm tính cước theo km, dòng 5 PerKm 5000 gắn với Lê Lan Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Blame: dòng <code>"PerKm": 5000</code> đến từ commit của Lê Lan</figcaption></figure>

Nhãn `89855d5` ở đầu khung cho biết đang blame file *tại commit nào*. Blame một commit cũ cho thấy file trông thế nào và ai viết từng dòng vào thời điểm đó.

### Blame bỏ qua thay đổi không quan trọng

| Tuỳ chọn | Tác dụng |
|---|---|
| `-w` | Bỏ qua thay đổi chỉ về khoảng trắng (đổi thụt lề không "cướp" dòng của người viết gốc) |
| `-M` | Nhận ra dòng **di chuyển** trong cùng file |
| `-C` | Nhận ra dòng được **chép hoặc chuyển** từ file khác trong cùng commit |
| `--ignore-rev <commit>` | Bỏ qua hẳn một commit (ví dụ commit chạy formatter cho toàn bộ dự án) |

```sh
git blame -w -L 11,13 src/BaoGia/FareCalculator.cs
```

```text
7ab5adc3 (Trần Minh 2026-11-02 09:10:00 +0700 11)         if (distanceKm < 0)
b56c4601 (Trần Minh 2026-11-05 09:00:00 +0700 12)             throw new ArgumentOutOfRangeException(
b56c4601 (Trần Minh 2026-11-05 09:00:00 +0700 13)                 nameof(distanceKm), distanceKm, "Quãng đường không được âm.");
```

Dòng điều kiện có từ lúc tạo hàm; câu lệnh `throw` đến từ commit "Báo lỗi rõ hơn" ở GIT-005. Nếu Minh gộp hai thay đổi trong một commit, blame vẫn chỉ ra được commit, nhưng nội dung commit "Sửa FareCalculator" sẽ không giải thích được dòng nào vì sao.

## 2. Lịch sử một file

Blame cho biết lần **cuối** mỗi dòng thay đổi. Muốn xem **mọi** lần một file thay đổi, dùng lịch sử file. Trên GitStudio, bấm **Lịch sử** cạnh **Blame**: cột trái liệt kê các commit đụng tới file, bấm một commit để xem diff của file trong commit đó.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-blame-lich-su-file-truy-vet/gitstudio-02.webp" alt="Chế độ Lịch sử của GitStudio cho appsettings.json: hai commit Tăng cước mỗi km ô tô 4 chỗ của Lê Lan d508151 và Thêm hàm tính cước theo km của Trần Minh 7ab5adc (Thêm mới), dòng Hết lịch sử; bên phải là diff PerKm 4500 thành 5000" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Lịch sử file: tương đương <code>git log -p -- src/BaoGia/appsettings.json</code></figcaption></figure>

Spotlight cũng mở được lịch sử file: `Ctrl+K`, gõ tên file, `Ctrl+Enter`.

### File đã đổi tên: `--follow`

```sh
git log --oneline -- docs/bang-gia-cuoc.md
git log --oneline --follow -- docs/bang-gia-cuoc.md
git log --oneline --follow --name-status -- docs/bang-gia-cuoc.md
```

```text
1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md
1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md
45c0d6b Thêm tài liệu bảng giá
1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md
R084	docs/bang-gia.md	docs/bang-gia-cuoc.md
45c0d6b Thêm tài liệu bảng giá
A	docs/bang-gia.md
```

Không có `--follow`, lịch sử dừng ở commit đổi tên, như thể tài liệu mới ra đời hôm 09/11. Có `--follow`, Git lần theo tên cũ: `R084` nghĩa là đổi tên (Rename) với độ giống 84%, `A` là thêm mới (Added). `--follow` chỉ dùng được với **một** file.

## 3. Pickaxe: tìm commit làm thay đổi một chuỗi

Blame chỉ thấy *những dòng còn tồn tại*. Muốn biết đơn giá cũ `4500` xuất hiện và biến mất khi nào, dù dòng đó không còn trong file, dùng **pickaxe** (cuốc chim):

```sh
git log -S'4500,' --oneline
git log -G'Math.Ceiling' --oneline
```

```text
d508151 Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ
7ab5adc Thêm hàm tính cước theo km
0d57e4b Làm tròn cước lên bội số 1.000 đ
```

| Tuỳ chọn | Tìm commit mà… | Dùng khi |
|---|---|---|
| `-S<chuỗi>` | Làm **thay đổi số lần xuất hiện** của chuỗi (thêm vào hoặc xoá đi) | Tìm lúc một hằng số, tên hàm xuất hiện/biến mất |
| `-G<regex>` | Có dòng thêm/bỏ **khớp** biểu thức chính quy | Tìm cả những lần sửa dòng chứa chuỗi mà số lần xuất hiện không đổi |

`4500,` xuất hiện ở commit tạo file (`7ab5adc`) và biến mất ở commit của Lan (`d508151`). Thêm `-p` để xem luôn diff của từng commit tìm được.

Trên GitStudio, cùng việc này là `Ctrl+F` trên đồ thị → phạm vi **Đoạn code thay đổi (toàn bộ lịch sử)**: GitStudio chạy `git log -S<chuỗi>` với `--fixed-strings` và không phân biệt hoa thường.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-blame-lich-su-file-truy-vet/gitstudio-03.webp" alt="Terminal tích hợp của GitStudio chạy git log -S'4500,' --oneline in d508151 và 7ab5adc, rồi git log --oneline --follow --name-status -- docs/bang-gia-cuoc.md in R084 docs/bang-gia.md docs/bang-gia-cuoc.md và A docs/bang-gia.md" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · pickaxe và <code>--follow</code> trong terminal tích hợp</figcaption></figure>

### Bẫy: dấu nháy kép trong PowerShell

Ý định ban đầu của bài là tìm chính xác chuỗi `"PerKm": 4500` (có cả dấu nháy kép), và lệnh tự nhiên nhất là:

```powershell
git log -S'"PerKm": 4500' --oneline
```

Trong Git Bash, lệnh cho ra hai commit như trên. Trong **Windows PowerShell 5.1** (shell mặc định của terminal tích hợp trên Windows), nó chỉ ra **một** commit: `b48e0a1 Thêm test cho hàm tính cước`. PowerShell 5.1 bỏ mất các dấu `"` bên trong đối số khi chuyển cho chương trình bên ngoài, nên git nhận được `-SPerKm: 4500`, và chuỗi đó có trong file test (`PerKm: 4500m`). Lệnh không báo lỗi gì; nó tìm *sai chuỗi*. Cách tránh:

- Chọn chuỗi tìm kiếm không có dấu nháy kép, như `4500,` ở trên.
- Hoặc chạy trong Git Bash: chọn **Git Bash** ở **Tuỳ chọn → Công cụ ngoài → Terminal**, terminal tích hợp mở mới sẽ dùng Git Bash. (Ngoài GitStudio, PowerShell 7.3 trở lên cũng đã sửa cách truyền đối số này.)
- Hoặc tìm bằng `Ctrl+F` của GitStudio: chuỗi được truyền thẳng cho git, không qua shell nào.

Bài học rộng hơn: khi kết quả của một lệnh tìm kiếm "trông hợp lý nhưng hơi lạ", kiểm lại bằng một cách thứ hai.

## 4. Lịch sử một khoảng dòng: `git log -L`

Hàm `Quote` đã bị sửa những lần nào? `-L <đầu>,<cuối>:<file>` theo dõi một khoảng dòng qua lịch sử, kể cả khi khoảng đó dịch lên xuống vì code phía trên thay đổi:

```sh
git log -L 10,25:src/BaoGia/FareCalculator.cs --format="%h %an %s" --no-patch
```

```text
0d57e4b Trần Minh Làm tròn cước lên bội số 1.000 đ
b56c460 Trần Minh Báo lỗi rõ hơn khi quãng đường âm
7ab5adc Trần Minh Thêm hàm tính cước theo km
```

Bỏ `--no-patch` để xem diff của khoảng dòng đó trong từng commit. Git còn hiểu `-L :Quote:src/BaoGia/FareCalculator.cs` (theo tên hàm) nếu nhận ra được ranh giới hàm cho ngôn ngữ đó; với C#, khai báo `*.cs diff=csharp` trong `.gitattributes` giúp Git nhận đúng.

## 5. Chọn công cụ nào

| Câu hỏi | Công cụ | GitStudio |
|---|---|---|
| Dòng này do ai viết, trong commit nào? | `git blame` | Blame |
| File này đã thay đổi những lần nào? | `git log -- <file>` | Lịch sử; Spotlight → `Ctrl+Enter` |
| … kể cả trước khi đổi tên? | `git log --follow -- <file>` | |
| Chuỗi này xuất hiện/biến mất khi nào? | `git log -S` | `Ctrl+F` → Đoạn code thay đổi |
| Dòng nào khớp biểu thức này từng bị sửa? | `git log -G` | |
| Hàm này đã bị sửa những lần nào? | `git log -L` | |
| Vì sao đổi? | `git show <commit>` (đọc description) | Bấm commit, đọc Mô tả |

## 6. Lỗi thường gặp

- **Blame trúng commit "định dạng lại code".** Mọi dòng đều trỏ về một commit chạy formatter. Dùng `-w`, hoặc liệt kê commit đó trong file `.git-blame-ignore-revs` và đặt `blame.ignoreRevsFile`.
- **Kết luận "người cuối cùng sửa là người gây lỗi".** Blame chỉ cho biết ai *chạm* vào dòng cuối cùng; lỗi có thể do một dòng khác, hoặc do yêu cầu sai. Đọc nội dung commit và diff trước khi kết luận. Tìm commit gây lỗi có công cụ riêng: bisect (Phần 5).
- **Tìm lịch sử file đã đổi tên mà quên `--follow`.** Kết luận sai rằng file "mới có".
- **Dấu nháy bị shell nuốt** (mục 3). Kiểm lại bằng cách thứ hai khi kết quả lạ.
- **`-S` với regex.** `-S` tìm chuỗi cố định (trừ khi thêm `--pickaxe-regex`); muốn regex thì dùng `-G`.

## 7. Bài tập

**Bài 1.** Dùng blame trên `tests/BaoGia.Tests/FareCalculatorTests.cs`. Test `Quote_LamTronLenBoi1000` đến từ commit nào? Vì sao không phải một commit riêng "Thêm test làm tròn"?

<details><summary><b>Lời giải</b></summary><p>Từ <code>0d57e4b</code> "Làm tròn cước lên bội số 1.000 đ". Ở GIT-005, Minh amend test vào commit làm tròn, nên test và thay đổi mà nó kiểm tra nằm cùng một commit: hoàn tác commit đó là hoàn tác cả hai.</p></details>

**Bài 2.** Tìm commit đã thêm chuỗi `"Quãng đường không được âm."` bằng pickaxe, theo cách chạy đúng trong cả PowerShell 5.1 lẫn Git Bash.

<details><summary><b>Lời giải</b></summary><p>Bỏ dấu nháy kép khỏi chuỗi tìm kiếm: <code>git log -S'Quãng đường không được âm' --oneline</code> ra <code>b56c460</code>. Hoặc dùng <code>Ctrl+F</code> → Đoạn code thay đổi trong GitStudio với cả chuỗi có dấu nháy.</p></details>

**Bài 3.** Vì sao `git log -S'5000' --oneline` ra nhiều commit hơn bạn nghĩ? Viết lệnh chỉ ra đúng commit đổi đơn giá ô tô.

<details><summary><b>Lời giải</b></summary><p>Ra ba commit: <code>d508151</code>, <code>b48e0a1</code> và <code>7ab5adc</code>. Chuỗi <code>5000</code> nằm bên trong <code>25000</code> (giá tối thiểu trong <code>appsettings.json</code> và trong file test), nên commit tạo file cấu hình và commit thêm test cũng làm thay đổi số lần xuất hiện. (<code>5.000 đ</code> trong tài liệu thì không khớp vì có dấu chấm.) Thu hẹp cả chuỗi lẫn đường dẫn: <code>git log -S'5000,' --oneline -- src/BaoGia/appsettings.json</code> chỉ ra <code>d508151</code>.</p></details>

## 8. Đọc tiếp

- [git-blame](https://git-scm.com/docs/git-blame): `-w`, `-M`, `-C`, `--ignore-rev`.
- [git-log](https://git-scm.com/docs/git-log): `-S`, `-G`, `--pickaxe-regex`, `-L`, `--follow`.
- [about_Parsing (PowerShell)](https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_parsing): cách PowerShell truyền đối số cho chương trình bên ngoài.

Bài tiếp theo: [GIT-010 · Thực hành: dựng repository dịch vụ báo giá có lịch sử sạch](/articles/git-thuc-hanh-repository-bao-gia).
