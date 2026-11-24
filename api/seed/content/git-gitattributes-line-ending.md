---
slug: git-gitattributes-line-ending
title: ".gitattributes nâng cao: xuống dòng, diff theo ngôn ngữ, file nhị phân và merge=union"
summary: "GIT-007 dùng .gitattributes để chuẩn hoá xuống dòng LF. Bài này đi tiếp: đọc thuộc tính bằng git check-attr và git ls-files --eol, khai báo diff=csharp để tiêu đề hunk là tên hàm, đánh dấu bảng tính là nhị phân, merge=union cho CHANGELOG, chuẩn hoá lại file cũ bằng git add --renormalize, và vì sao .gitattributes thắng core.autocrlf. GitStudio đọc cùng các thuộc tính đó khi hiển thị diff."
category: git
level: beginner
tags: [git, git-043, gitattributes, line-ending, diff-driver, merge-union, gitstudio, toi-noi]
language: vi
---

Sáng 27/11/2026, Lan review diff của Minh và thấy tiêu đề mỗi hunk luôn là `public static class FareCalculator`, dù thay đổi nằm sâu trong hàm `Quote`. Cùng lúc, bộ phận kế toán muốn đưa bảng giá dạng Excel vào repository, và hai người cùng thêm dòng vào CHANGELOG thì lần nào cũng xung đột. Cả ba chuyện đều giải quyết bằng vài dòng trong `.gitattributes`.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-043 · Phần 5/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-diff-so-sanh-thay-doi">GIT-007</a> (CRLF và .gitattributes lần đầu). Lab: <code>npm run p05 -- 43</code>; mở bằng GitStudio: <code>npm run setup -- 43</code>. Sau bài này, bạn đọc được thuộc tính Git gán cho một file, và cấu hình được xuống dòng, diff, merge theo loại file cho cả nhóm.</p>
</div>

## 1. .gitattributes là gì, ai đọc nó

Mỗi dòng: **mẫu đường dẫn** rồi các **thuộc tính**. Git đọc file từ thư mục gốc xuống thư mục con (file ở thư mục sâu hơn ghi đè), dòng sau thắng dòng trước. Vì nằm trong repository nên mọi người clone về đều dùng chung, khác với `core.autocrlf` chỉ là cấu hình máy.

```sh
git check-attr -a -- src/BaoGia/FareCalculator.cs build.cmd docs/logo.png
git ls-files --eol -- README.md src/BaoGia/FareCalculator.cs
```

```text
src/BaoGia/FareCalculator.cs: text: auto
src/BaoGia/FareCalculator.cs: eol: lf
build.cmd: text: set
build.cmd: eol: crlf
docs/logo.png: binary: set
docs/logo.png: diff: unset
docs/logo.png: merge: unset
docs/logo.png: text: unset
i/lf    w/lf    attr/text=auto eol=lf 	README.md
i/lf    w/lf    attr/text=auto eol=lf 	src/BaoGia/FareCalculator.cs
```

`check-attr` cho biết thuộc tính áp lên một đường dẫn, kể cả file chưa tồn tại (`build.cmd`, `docs/logo.png`), rất tiện để thử mẫu. `ls-files --eol` cho biết xuống dòng thật: `i/` trong index (repository), `w/` trong working tree. `binary` là viết tắt của `-diff -merge -text`.

## 2. Diff theo ngôn ngữ: `diff=csharp`

Mặc định, tiêu đề hunk (phần sau `@@ … @@`) là dòng gần nhất phía trên **không thụt lề**:

```text
@@ -18,3 +18,3 @@ public static class FareCalculator
-        // Chưa gồm phí cầu đường, khuyến mãi.
+        // Chưa gồm phí cầu đường, khuyến mãi, phụ phí đêm.
```

Git có sẵn "diff driver" cho nhiều ngôn ngữ (csharp, java, python, golang, rust, html, markdown…), biết nhận ra dòng khai báo hàm. Thêm `*.cs diff=csharp`:

```text
@@ -18,3 +18,3 @@ public static decimal Quote(FareRule rule, decimal distanceKm, decimal peakMulti
-        // Chưa gồm phí cầu đường, khuyến mãi.
+        // Chưa gồm phí cầu đường, khuyến mãi, phụ phí đêm.
```

Lợi ích không chỉ để đọc: `git log -L :Quote:src/BaoGia/FareCalculator.cs` (lịch sử của riêng một hàm, GIT-009) dựa vào cùng cơ chế để biết hàm bắt đầu và kết thúc ở đâu.

## 3. Khối thêm vào của nhóm Tới Nơi

```text
# Diff hiểu cấu trúc C# (tiêu đề hunk là tên hàm)
*.cs diff=csharp

# Bảng tính: nhị phân
*.xlsx binary

# CHANGELOG: hai nhánh cùng thêm dòng thì giữ cả hai, không báo xung đột
CHANGELOG.md merge=union
```

```sh
git check-attr diff merge -- src/BaoGia/FareCalculator.cs CHANGELOG.md docs/bang-gia.xlsx
```

```text
src/BaoGia/FareCalculator.cs: diff: csharp
src/BaoGia/FareCalculator.cs: merge: unspecified
CHANGELOG.md: diff: unspecified
CHANGELOG.md: merge: union
docs/bang-gia.xlsx: diff: unset
docs/bang-gia.xlsx: merge: unset
```

- **`binary`** cho file Excel, Word, PDF, ảnh: Git không thử đổi xuống dòng (đổi là hỏng file), không in diff dạng chữ, không thử merge từng dòng. Khi hai nhánh cùng sửa, Git báo xung đột cả file và bạn chọn một bản.
- **`merge=union`** (kiểu merge union): khi hai bên cùng thêm dòng vào cùng một chỗ, Git giữ **cả hai** thay vì đặt marker xung đột. Hợp với file chỉ thêm dòng như CHANGELOG, danh sách thành viên; **không** dùng cho code (thứ tự dòng không đảm bảo, có thể ra code sai mà không báo).
- File lớn nên đi qua Git LFS (GIT-045); LFS cũng ghi quy tắc vào chính file này.

## 4. Đổi quy tắc xuống dòng: chuẩn hoá lại

Khi thêm hay sửa quy tắc `text`/`eol`, các file đã commit trước đó **không** tự đổi. `git add --renormalize .` đưa mọi file đã track qua bộ lọc mới và stage những file thay đổi:

```sh
git add --renormalize .
git status -s
```

```text
M  .gitattributes
```

Ở đây không file nào cần đổi (từ GIT-007 mọi file đã là LF), chỉ có `.gitattributes` vừa sửa. Khi có thay đổi, nên commit riêng ("Chuẩn hoá xuống dòng theo .gitattributes") để diff của các commit khác không lẫn hàng trăm dòng đổi CRLF.

Thứ tự ưu tiên khi Git quyết định xuống dòng: thuộc tính `eol`/`text` trong `.gitattributes` > `core.eol` > `core.autocrlf` của từng máy. Đó là lý do nhóm đặt quy tắc trong repository: không phụ thuộc máy ai cài Git thế nào.

## 5. Trên GitStudio

GitStudio gọi `git` nên tôn trọng mọi thuộc tính: diff dùng cùng tiêu đề hunk; file `binary` hiện "Tệp nhị phân" thay vì nội dung (ảnh có chế độ xem **Cạnh nhau**, **Trượt**, **Chồng lớp**); file LFS được nhận ra là con trỏ. Panel file cho thấy cảnh báo xuống dòng khi stage. Tuỳ chọn → Chung → **Ký tự xuống dòng** đặt `core.autocrlf` cho máy (GIT-002), nhưng `.gitattributes` vẫn ưu tiên hơn.

Không có trình sửa `.gitattributes` riêng: mở file bằng **Sửa tệp này** trong diff hoặc trình soạn thảo, commit như mọi file.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-gitattributes-line-ending/gitstudio-01.webp" alt="Diff của FareCalculator.cs trong GitStudio sau khi khai báo diff=csharp: tiêu đề hunk ghi public static decimal Quote thay vì public static class FareCalculator" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · tiêu đề hunk là tên hàm nhờ diff=csharp</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-gitattributes-line-ending/gitstudio-02.webp" alt="Diff của .gitattributes trong GitStudio: thêm các dòng *.cs diff=csharp, *.xlsx binary, CHANGELOG.md merge=union kèm chú thích" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · .gitattributes là một file thường: sửa, review, commit cùng code</figcaption></figure>

## 6. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Xem thuộc tính của một file | | `git check-attr -a -- <file>` |
| Xem xuống dòng thật | Cảnh báo khi stage | `git ls-files --eol` |
| Chuẩn hoá lại sau khi đổi quy tắc | | `git add --renormalize .` |
| Xuống dòng của máy | Tuỳ chọn → Chung → Ký tự xuống dòng | `git config --global core.autocrlf` |
| Diff theo ngôn ngữ | Diff tự áp dụng | `*.cs diff=csharp` |

## 7. Lỗi thường gặp

- **Thêm quy tắc mà quên renormalize**: file cũ vẫn CRLF, diff lần sửa sau đổi toàn bộ file.
- **`* text=auto` mà có file nhị phân lạ** (đuôi Git đoán sai): khai báo rõ `binary`.
- **`merge=union` cho code hoặc JSON**: kết quả có thể thiếu dấu phẩy, trùng khoá, sai thứ tự mà không báo xung đột.
- **Viết `*.CS`**: mẫu phân biệt hoa thường trên Linux; dùng đúng đuôi thật hoặc `*.[cC][sS]`.
- **Sửa `.gitattributes` trong nhánh tính năng**: quy tắc chỉ có hiệu lực ở nhánh đó tới khi merge.

## 8. Bài tập

**Bài 1.** Viết quy tắc để file `*.sh` luôn checkout với LF kể cả trên Windows, và `*.ps1` với CRLF.

<details><summary><b>Lời giải</b></summary><p><code>*.sh text eol=lf</code> và <code>*.ps1 text eol=crlf</code>.</p></details>

**Bài 2.** Kiểm tra quy tắc nào áp lên `docs/thiet-ke/bang-gia-2027.png` sau GIT-045, và vì sao dòng LFS thắng dòng `*.png binary`.

<details><summary><b>Lời giải</b></summary><p><code>git check-attr -a -- docs/thiet-ke/bang-gia-2027.png</code>: <code>filter: lfs</code>, <code>diff: lfs</code>, <code>merge: lfs</code>, <code>text: unset</code>. Dòng LFS nằm sau trong file nên ghi đè các thuộc tính trùng tên.</p></details>

**Bài 3.** Một thư mục `vendor/` chứa mã của bên thứ ba, nhóm không muốn GitHub tính vào thống kê ngôn ngữ. Thuộc tính nào?

<details><summary><b>Lời giải</b></summary><p><code>vendor/** linguist-vendored</code>. Thuộc tính lạ với Git vẫn hợp lệ; công cụ khác (GitHub Linguist) đọc chúng.</p></details>

## 9. Đọc tiếp

- [gitattributes](https://git-scm.com/docs/gitattributes): `text`, `eol`, `diff`, `merge`, built-in diff drivers.
- [Pro Git, Git Attributes](https://git-scm.com/book/en/v2/Customizing-Git-Git-Attributes).

Bài tiếp theo: [GIT-044 · Submodule: dùng chung thư viện tiền tệ](/articles/git-submodule).
