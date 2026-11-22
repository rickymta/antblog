---
slug: git-diff-so-sanh-thay-doi
title: "Diff: xem đúng thay đổi trước khi commit, kể cả đổi tên file và xuống dòng CRLF"
summary: "Lan mở appsettings.json trên Windows, lưu lại, và Git báo cả 14 dòng đều thay đổi dù cô không sửa chữ nào. Bài này phân biệt git diff, git diff --staged, git diff HEAD và diff giữa hai commit, đọc diff theo từ, nhận ra file đổi tên, dùng các chế độ hunk, inline, split của GitStudio, rồi chặn lỗi CRLF/LF bằng .gitattributes."
category: git
level: intermediate
tags: [git, git-007, git-diff, staged, rename, crlf, gitattributes, gitstudio, toi-noi]
language: vi
---

Ngày 10/11/2026, Lan mở `src/BaoGia/appsettings.json` bằng một trình soạn thảo trên Windows, đọc qua rồi bấm lưu, không sửa gì. GitStudio báo file thay đổi; `git diff --stat` báo `14 insertions(+), 14 deletions(-)`, tức *mọi dòng* đều khác. Nếu cô commit, lịch sử sẽ ghi Lan là người "viết" toàn bộ file bảng giá, và `git blame` (GIT-009) không còn chỉ ra được ai đổi đơn giá mỗi km nữa.

Thủ phạm là ký tự xuống dòng: trình soạn thảo đã đổi `LF` thành `CRLF`. Để nhận ra những chuyện như vậy *trước* khi commit, cần đọc diff cho đúng. Bài này đi qua các loại diff, rồi chặn hẳn lỗi xuống dòng bằng `.gitattributes`.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-007 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-lich-su-log-do-thi-commit">GIT-006</a>. Lab: <code>npm run p01 -- 07</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 07</code>. Sau bài này, bạn biết mỗi lệnh diff so hai vùng nào, đọc được diff theo từ và diff có đổi tên, dùng được các chế độ xem của GitStudio, và chuẩn hoá xuống dòng bằng <code>.gitattributes</code>.</p>
</div>

## 1. Diff so cái gì với cái gì

Ba vùng của GIT-004 cho ra ba cặp so sánh, mỗi cặp một lệnh:

| Lệnh | So | Trả lời |
|---|---|---|
| `git diff` | staging area ↔ working tree | Có gì tôi đã sửa mà **chưa stage**? |
| `git diff --staged` (= `--cached`) | HEAD ↔ staging area | Commit tới **sẽ chứa** gì? |
| `git diff HEAD` | HEAD ↔ working tree | Tổng cộng tôi đã sửa gì từ commit cuối? |
| `git diff A B` | commit A ↔ commit B | Hai phiên bản khác nhau thế nào? |

Lab tạo hai thay đổi: thêm một dòng vào `README.md` (đã stage) và đổi đơn giá mỗi km của xe máy trong `appsettings.json` (chưa stage):

```sh
git status --short
```

```text
M  README.md
 M src/BaoGia/appsettings.json
```

```sh
git diff
```

```diff
diff --git a/src/BaoGia/appsettings.json b/src/BaoGia/appsettings.json
index ce82839..0c37630 100644
--- a/src/BaoGia/appsettings.json
+++ b/src/BaoGia/appsettings.json
@@ -7,7 +7,7 @@
     },
     "Bike": {
       "BaseFare": 8000,
-      "PerKm": 2500,
+      "PerKm": 3000,
       "MinimumFare": 12000
     }
   }
```

`git diff` **không** hiện thay đổi của `README.md`, vì thay đổi đó đã nằm trong staging area. Người mới hay hoảng ở đây: "thay đổi của tôi đâu mất?". Nó ở `--staged`:

```sh
git diff --staged
```

```diff
diff --git a/README.md b/README.md
index 6fe73a4..12ec0f3 100644
--- a/README.md
+++ b/README.md
@@ -2,3 +2,5 @@
 
 Dịch vụ báo giá cước của Tới Nơi: nhận quãng đường và loại dịch vụ,
 trả về giá dự kiến cho khách trước khi đặt chuyến.
+
+Chạy test: `dotnet test`.
```

```sh
git diff HEAD --stat
```

```text
 README.md                   | 2 ++
 src/BaoGia/appsettings.json | 2 +-
 2 files changed, 3 insertions(+), 1 deletion(-)
```

Dòng `index ce82839..0c37630 100644` cho biết mã hash nội dung file trước và sau (GIT-008), và quyền file.

## 2. Diff theo từ

Với file cấu hình, dòng dài, hay văn bản tiếng Việt, diff theo dòng khó thấy *chữ nào* đổi. `--word-diff` đánh dấu từng từ:

```sh
git diff --word-diff src/BaoGia/appsettings.json
```

```text
@@ -7,7 +7,7 @@
    },
    "Bike": {
      "BaseFare": 8000,
      "PerKm": [-2500,-]{+3000,+}
      "MinimumFare": 12000
    }
  }
```

`[-…-]` là phần bị bỏ, `{+…+}` là phần được thêm.

## 3. Các chế độ xem diff của GitStudio

Bấm một file trong **Tệp chưa stage** hay **Tệp đã stage**, khung giữa chuyển sang xem diff. Nút **Chưa stage**/**Đã stage** cho biết đang xem cặp so sánh nào (tương ứng `git diff` và `git diff --staged`). Bộ nút góc phải đổi cách xem:

| Nút | Chế độ |
|---|---|
| **Xem theo hunk** | Chỉ các hunk thay đổi kèm ngữ cảnh, có nút Stage/Huỷ bỏ từng hunk (GIT-005) |
| **Xem liền mạch (inline)** | Toàn bộ file, dòng thêm/bỏ đặt xen trong file |
| **Xem hai cột (split)** | Bản cũ bên trái, bản mới bên phải |
| **Bỏ qua khoảng trắng đầu/cuối dòng** | Giống `git diff -w` một phần: ẩn thay đổi chỉ về thụt lề |
| **Tự xuống dòng** | Ngắt dòng dài thay vì cuộn ngang |

Mũi tên lên/xuống nhảy tới thay đổi trước/sau (`1/2` là đang ở thay đổi thứ nhất trên hai). **Xem tệp** hiện nội dung đầy đủ của file; **Sửa tệp này** sửa ngay trong GitStudio, giữ nguyên bảng mã và kiểu xuống dòng của file.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-diff-so-sanh-thay-doi/gitstudio-01.webp" alt="Khung diff theo hunk của GitStudio cho src/BaoGia/appsettings.json ở chế độ Chưa stage: dòng PerKm 2500 bị bỏ, dòng PerKm 3000 được thêm; README.md nằm ở Tệp đã stage" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · diff phần chưa stage, tương đương <code>git diff</code></figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-diff-so-sanh-thay-doi/gitstudio-02.webp" alt="Cùng file appsettings.json ở chế độ Xem hai cột: bên trái bản trong staging area với PerKm 2500 tô đỏ ở dòng 10, bên phải bản trong working tree với PerKm 3000 tô xanh, nút Stage hunk vẫn có" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · chế độ hai cột (split)</figcaption></figure>

Xong phần xem, lab trả mọi thứ về như cũ:

```sh
git restore --staged README.md
git restore README.md src/BaoGia/appsettings.json
git status --short
```

`git restore --staged` đưa file ra khỏi staging area (thay đổi vẫn còn trong working tree); `git restore` không có `--staged` **bỏ hẳn** thay đổi trong working tree. Lệnh thứ hai không hoàn tác được; Phần 4 bàn kỹ.

## 4. Diff giữa hai commit

```sh
git diff HEAD~4 HEAD --stat
git diff HEAD~4 HEAD -- src/BaoGia/FareCalculator.cs
```

```text
 docs/bang-gia-cuoc.md                     | 10 ++++++++++
 src/BaoGia/FareCalculator.cs              |  6 ++++--
 tests/BaoGia.Tests/FareCalculatorTests.cs |  4 ++++
 3 files changed, 18 insertions(+), 2 deletions(-)
```

Lệnh thứ hai in đúng hai hunk của GIT-005 (thông báo lỗi và làm tròn), vì giữa `HEAD~4` (commit đổi giá của Lan) và `HEAD` chỉ có hai commit đó đụng tới `FareCalculator.cs`. Thứ tự quan trọng: `git diff A B` là "đi từ A tới B cần đổi gì"; đảo lại thì `+` thành `-`.

Trên GitStudio: bấm một commit để xem file đã đổi trong commit đó (so với commit cha). Để so hai commit bất kỳ, chọn một commit rồi `Ctrl` + bấm commit thứ hai, bảng bên phải hiện "2 commit đã chọn" và nút **So sánh**; hoặc chuột phải commit thứ nhất → **Đánh dấu làm mốc so sánh**, rồi chuột phải commit thứ hai → **So sánh**. Kết quả liệt kê file khác nhau giữa hai commit, có nút **Đổi chiều** để đảo bên trái/phải.

## 5. Đổi tên file

Git không lưu thao tác "đổi tên". Commit chỉ có ảnh chụp trước và sau; khi so, Git thấy một file biến mất và một file mới xuất hiện, rồi **đoán** đó là đổi tên nếu nội dung đủ giống nhau (mặc định từ 50% trở lên).

```sh
git diff --stat HEAD~1 HEAD
git diff --stat --no-renames HEAD~1 HEAD
```

```text
 docs/{bang-gia.md => bang-gia-cuoc.md} | 2 ++
 1 file changed, 2 insertions(+)
 docs/bang-gia-cuoc.md | 10 ++++++++++
 docs/bang-gia.md      |  8 --------
 2 files changed, 10 insertions(+), 8 deletions(-)
```

Có phát hiện đổi tên: một file, thêm 2 dòng. Tắt đi (`--no-renames`): một file bị xoá 8 dòng, một file mới 10 dòng. Cùng một commit, hai cách nhìn. Commit của Lan vừa đổi tên vừa thêm một dòng "Nguồn số liệu…", vẫn đủ giống (84%, xem `R084` ở GIT-009) để Git nhận ra.

GitStudio hiện đổi tên bằng mũi tên `→` và ghi "1 đổi tên" trong chi tiết commit:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-diff-so-sanh-thay-doi/gitstudio-03.webp" alt="Chi tiết commit 1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md của Lê Lan trong GitStudio, Mô tả Không có mô tả, danh sách file ghi 1 đổi tên với mũi tên trước docs/bang-gia-cuoc.md" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · commit đổi tên file: "1 đổi tên"</figcaption></figure>

Mẹo: đổi tên và sửa nội dung lớn trong **hai commit riêng**. Gộp chung, độ giống có thể dưới 50%, Git coi là xoá + thêm, và lịch sử của file bị đứt ở đó.

## 6. CRLF và LF: vì sao cả file "thay đổi"

Windows kết thúc dòng bằng hai ký tự `CR LF` (`\r\n`), Linux và macOS bằng một ký tự `LF` (`\n`). Repository `bao-gia` lưu LF. Lab giả lập trình soạn thảo của Lan lưu lại file với CRLF:

```sh
git diff --stat
git diff --ignore-cr-at-eol --stat
```

```text
 src/BaoGia/appsettings.json | 28 ++++++++++++++--------------
 1 file changed, 14 insertions(+), 14 deletions(-)
```

Lệnh đầu: 14 dòng bỏ, 14 dòng thêm. Lệnh thứ hai bỏ qua `CR` ở cuối dòng và không in gì: nội dung thật **không đổi**. `--ignore-cr-at-eol` (và nút **Bỏ qua khoảng trắng đầu/cuối dòng** của GitStudio) chỉ giúp *nhìn*; commit vẫn sẽ ghi CRLF vào lịch sử. Phải chặn từ gốc.

## 7. `.gitattributes`: quy định xuống dòng cho cả nhóm

`core.autocrlf` (GIT-002) là cấu hình **trên máy từng người**: mỗi máy một kiểu là sinh chuyện. `.gitattributes` là file **trong repository**, commit cùng code, áp dụng cho mọi người clone về:

```gitattributes
# Mọi file văn bản lưu trong repository với xuống dòng LF
* text=auto eol=lf

# Script Windows cần CRLF khi checkout
*.cmd text eol=crlf
*.bat text eol=crlf

# File nhị phân: không diff, không đổi xuống dòng
*.png binary
*.webp binary
```

| Thuộc tính | Nghĩa |
|---|---|
| `text=auto` | Git tự nhận file văn bản; file văn bản luôn được **lưu trong repository bằng LF** |
| `eol=lf` / `eol=crlf` | Kiểu xuống dòng khi **checkout ra working tree** |
| `binary` | Không chuyển xuống dòng, không diff dạng văn bản |

```sh
git add .gitattributes
git commit -m "Chuẩn hoá xuống dòng LF bằng .gitattributes"
git ls-files --eol
```

```text
[main 89855d5] Chuẩn hoá xuống dòng LF bằng .gitattributes
 1 file changed, 10 insertions(+)
 create mode 100644 .gitattributes
i/lf    w/lf    attr/text=auto eol=lf 	.gitattributes
i/lf    w/lf    attr/text=auto eol=lf 	.gitignore
i/lf    w/lf    attr/text=auto eol=lf 	README.md
i/lf    w/lf    attr/text=auto eol=lf 	docs/bang-gia-cuoc.md
i/lf    w/lf    attr/text=auto eol=lf 	src/BaoGia/FareCalculator.cs
i/lf    w/lf    attr/text=auto eol=lf 	src/BaoGia/appsettings.json
i/lf    w/lf    attr/text=auto eol=lf 	tests/BaoGia.Tests/FareCalculatorTests.cs
```

`i/` là kiểu xuống dòng trong staging area (sẽ vào commit), `w/` trong working tree, `attr/` là thuộc tính đang áp dụng. Giờ thử lại tình huống của Lan:

```sh
git add src/BaoGia/appsettings.json
git status --short
git ls-files --eol src/BaoGia/appsettings.json
```

```text
warning: in the working copy of 'src/BaoGia/appsettings.json', CRLF will be replaced by LF the next time Git touches it
i/lf    w/crlf  attr/text=auto eol=lf 	src/BaoGia/appsettings.json
```

File trên đĩa vẫn CRLF (`w/crlf`), nhưng khi add, Git chuyển về LF (`i/lf`). Nội dung trong staging area giống hệt commit, nên `git status --short` không in gì: **không còn thay đổi giả**. Lời cảnh báo chỉ báo rằng lần sau Git ghi file này ra đĩa, nó sẽ dùng LF theo `eol=lf`.

Repository đã có sẵn file CRLF trong lịch sử thì sau khi thêm `.gitattributes`, chạy `git add --renormalize .` rồi commit một lần để chuẩn hoá (GIT-010 làm việc này).

## 8. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Diff phần chưa stage | Bấm file ở Tệp chưa stage | `git diff` |
| Diff phần đã stage | Bấm file ở Tệp đã stage | `git diff --staged` |
| Diff một commit | Bấm commit → bấm file | `git show <commit> -- <file>` |
| Diff hai commit | `Ctrl` + bấm hai commit → So sánh | `git diff A B` |
| Xem hai cột | Xem hai cột (split) | `git difftool` |
| Bỏ qua thụt lề | Bỏ qua khoảng trắng đầu/cuối dòng | `git diff -w` |
| Diff theo từ | | `git diff --word-diff` |
| Kiểu xuống dòng của file | Nhãn `UTF-8 · LF` trên tiêu đề diff | `git ls-files --eol` |

## 9. Lỗi thường gặp

- **Xem `git diff` trống rồi commit, tưởng không có gì.** `git diff` không hiện phần đã stage; dùng `git diff --staged` hoặc `git status`.
- **Commit cả file chỉ vì đổi xuống dòng hay thụt lề.** Nhìn `--stat`: số dòng thêm/bỏ bằng đúng số dòng của file là dấu hiệu. Bật `.gitattributes`.
- **Đặt `core.autocrlf=true` trên máy này, `false` trên máy kia.** Cùng repository, hai người liên tục "sửa" xuống dòng của nhau. `.gitattributes` thắng `core.autocrlf` cho những file nó khớp.
- **Quên đánh dấu `binary` cho file nhị phân không rõ đuôi.** Git có thể đoán nhầm là văn bản và đổi byte `\n` bên trong, làm hỏng file. Khai báo `binary` cho mọi đuôi nhị phân dự án dùng.
- **So hai commit theo thứ tự ngược** (`git diff HEAD HEAD~4`): mọi `+` thành `-`, đọc tưởng code bị xoá.

## 10. Bài tập

**Bài 1.** Stage một nửa thay đổi của `appsettings.json` bằng GitStudio (sửa hai giá, stage một hunk). Chạy `git diff`, `git diff --staged`, `git diff HEAD`. Lệnh nào hiện cả hai thay đổi?

<details><summary><b>Lời giải</b></summary><p><code>git diff HEAD</code>: nó so commit cuối với working tree, nên gồm cả phần đã stage và chưa stage. <code>git diff</code> chỉ hiện phần chưa stage, <code>--staged</code> chỉ hiện phần đã stage.</p></details>

**Bài 2.** Vì sao `git diff --stat HEAD~2 HEAD` (từ commit làm tròn tới commit đổi tên) **không** nhận ra đổi tên, mà báo `docs/bang-gia-cuoc.md` là file mới?

<details><summary><b>Lời giải</b></summary><p>Ở <code>HEAD~2</code> chưa có <code>docs/bang-gia.md</code> (file được thêm ở <code>HEAD~1</code>). So hai đầu khoảng, Git chỉ thấy một file mới xuất hiện, không có file nào biến mất để ghép cặp. Đổi tên chỉ được phát hiện giữa hai phiên bản mà file cũ tồn tại ở bên trái và file mới ở bên phải.</p></details>

**Bài 3.** Thêm dòng `*.sln text eol=crlf` vào `.gitattributes`. File `.sln` được lưu trong repository bằng gì, và checkout ra bằng gì?

<details><summary><b>Lời giải</b></summary><p>Lưu trong repository bằng LF (mọi file <code>text</code> đều được chuẩn hoá về LF khi add), checkout ra working tree bằng CRLF. Visual Studio đọc được cả hai, nhưng một số công cụ cũ của Windows cần CRLF, nên quy tắc này hay gặp trong repository .NET.</p></details>

## 11. Đọc tiếp

- [git-diff](https://git-scm.com/docs/git-diff): mọi tuỳ chọn, gồm `--word-diff`, `-M` (ngưỡng phát hiện đổi tên).
- [gitattributes](https://git-scm.com/docs/gitattributes): `text`, `eol`, `binary`, diff driver.
- [GitHub: Configuring Git to handle line endings](https://docs.github.com/en/get-started/getting-started-with-git/configuring-git-to-handle-line-endings).

Bài tiếp theo: [GIT-008 · Bên trong `.git`: blob, tree, commit, ref và HEAD](/articles/git-ben-trong-object-ref-head).
