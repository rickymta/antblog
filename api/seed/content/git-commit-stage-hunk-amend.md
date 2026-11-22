---
slug: git-commit-stage-hunk-amend
title: "Commit tốt: stage theo hunk và theo dòng, viết nội dung commit, amend commit cuối"
summary: "Minh sửa FareCalculator.cs hai chỗ không liên quan: thông báo lỗi khi quãng đường âm và làm tròn cước lên 1.000 đ. Bài này tách một lần sửa thành hai commit có nghĩa bằng Stage hunk, chọn từng dòng trong GitStudio và git add -p, viết summary và description, rồi amend commit vừa tạo để thêm test bị quên."
category: git
level: intermediate
tags: [git, git-005, commit, hunk, git-add-patch, commit-message, amend, gitstudio, toi-noi]
language: vi
---

Sáng 05/11/2026, Minh mở `FareCalculator.cs` để làm tròn cước lên bội số 1.000 đ theo yêu cầu của bộ phận vận hành. Tiện tay, anh sửa luôn thông báo lỗi khi quãng đường âm cho dễ hiểu hơn. Hai thay đổi nằm trong một file, nhưng là hai chuyện khác nhau: một thay đổi **giá khách phải trả**, một thay đổi **thông báo lỗi**. Gộp chung một commit "Sửa FareCalculator", sau này muốn hoàn tác riêng phần làm tròn (nếu bộ phận vận hành đổi ý) thì phải gỡ tay; người review cũng phải tự đoán dòng nào thuộc việc nào.

Một commit tốt chứa **đúng một thay đổi có nghĩa**, và nội dung commit nói rõ thay đổi đó là gì, vì sao. Bài này tách hai thay đổi trên thành hai commit, rồi sửa commit vừa tạo khi phát hiện quên test.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-005 · Phần 1/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-ba-vung-working-tree-staging-gitignore">GIT-004</a>. Lab: <code>npm run p01 -- 05</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 05</code>. Sau bài này, bạn stage được một phần của file (theo hunk, theo dòng), viết được summary và description đúng mực, và amend được commit cuối.</p>
</div>

## 1. Hunk là gì

**Diff** là phần khác nhau giữa hai phiên bản của file. Git chia diff thành các **hunk**: mỗi hunk là một cụm dòng thay đổi liền nhau, kèm vài dòng không đổi xung quanh làm ngữ cảnh (mặc định 3 dòng mỗi phía). Hai thay đổi cách nhau đủ xa thì thành hai hunk; gần nhau quá thì Git gộp thành một.

Thay đổi của Minh (lab ghi sẵn vào working tree):

```sh
git diff
```

```diff
diff --git a/src/BaoGia/FareCalculator.cs b/src/BaoGia/FareCalculator.cs
index 175bdd3..378b9cb 100644
--- a/src/BaoGia/FareCalculator.cs
+++ b/src/BaoGia/FareCalculator.cs
@@ -9,7 +9,8 @@ public static class FareCalculator
     public static decimal Quote(FareRule rule, decimal distanceKm)
     {
         if (distanceKm < 0)
-            throw new ArgumentOutOfRangeException(nameof(distanceKm));
+            throw new ArgumentOutOfRangeException(
+                nameof(distanceKm), distanceKm, "Quãng đường không được âm.");
 
         // Giá = giá mở cửa + đơn giá mỗi km × số km.
         // Ví dụ ô tô 4 chỗ đi 4 km: 12.000 + 4.500 × 4 = 30.000 đ.
@@ -18,6 +19,7 @@ public static class FareCalculator
         var fare = rule.BaseFare + rule.PerKm * distanceKm;
         var atLeastMinimum = Math.Max(fare, rule.MinimumFare);
 
-        return atLeastMinimum;
+        // Làm tròn lên bội số 1.000 đ để khách không phải trả tiền lẻ.
+        return Math.Ceiling(atLeastMinimum / 1000m) * 1000m;
     }
 }
```

Dòng `@@ -9,7 +9,8 @@` là tiêu đề hunk: ở bản cũ, hunk bắt đầu từ dòng 9 và dài 7 dòng; ở bản mới, bắt đầu từ dòng 9 và dài 8 dòng (thêm một dòng). Phần chữ sau `@@` là ngữ cảnh Git đoán được (tên lớp chứa đoạn thay đổi). Dòng `-` bị bỏ, dòng `+` được thêm, dòng bắt đầu bằng khoảng trắng là ngữ cảnh.

Hai hunk ứng đúng hai việc: hunk 1 là thông báo lỗi, hunk 2 là làm tròn.

## 2. Stage theo hunk trên GitStudio

Chọn hàng **// WIP**, bấm `FareCalculator.cs` trong **Tệp chưa stage**. Khung giữa chuyển sang xem diff, mỗi hunk có hai nút **Huỷ bỏ hunk** và **Stage hunk** ở góc phải tiêu đề hunk.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-commit-stage-hunk-amend/gitstudio-01.webp" alt="Khung diff của GitStudio cho src/BaoGia/FareCalculator.cs ở chế độ Chưa stage, hai hunk @@ -9,7 +9,8 @@ và @@ -18,6 +19,7 @@, mỗi hunk có nút Huỷ bỏ hunk và Stage hunk; hunk 1 đổi câu lệnh throw, hunk 2 đổi return thành Math.Ceiling" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · hai hunk của một file, mỗi hunk stage được riêng</figcaption></figure>

Bấm **Stage hunk** của hunk 1. Hunk đó biến khỏi khung "Chưa stage"; file giờ có mặt ở **cả hai** danh sách: phần thông báo lỗi ở **Tệp đã stage**, phần làm tròn vẫn ở **Tệp chưa stage**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-commit-stage-hunk-amend/gitstudio-02.webp" alt="Sau khi stage hunk 1: khung diff chỉ còn hunk làm tròn @@ -19,6 +19,7 @@, FareCalculator.cs xuất hiện ở cả Tệp chưa stage và Tệp đã stage" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · hunk 1 đã stage, hunk làm tròn vẫn chờ</figcaption></figure>

Nút **Huỷ bỏ hunk** thì ngược lại: bỏ hẳn thay đổi đó khỏi working tree. Thao tác này không hoàn tác được bằng git (thay đổi chưa từng được commit), nên GitStudio hỏi xác nhận trước.

## 3. Stage theo dòng

Khi hai thay đổi nằm sát nhau trong cùng một hunk, stage theo hunk không đủ. Bấm vào **cột số dòng** của một dòng `+` hoặc `-` để chọn dòng đó; `Shift` + bấm để chọn cả khoảng. Tiêu đề hunk đổi thành **Bỏ chọn**, **Huỷ bỏ N dòng**, **Stage N dòng**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-commit-stage-hunk-amend/gitstudio-03.webp" alt="Khung diff của GitStudio với một dòng thêm được chọn bằng cách bấm cột số dòng, tiêu đề hunk hiện các nút Bỏ chọn, Huỷ bỏ 1 dòng và Stage 1 dòng" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · bấm cột số dòng để chọn từng dòng; <code>Shift</code> + bấm để chọn khoảng</figcaption></figure>

Stage riêng dòng chú thích `// Làm tròn…` mà không stage dòng `return` sẽ cho ra một commit chỉ có chú thích: hợp lệ với Git, nhưng vô nghĩa với người đọc. Stage theo dòng là công cụ để *tách cho đúng*, không phải để cắt vụn. Ở đây Minh bấm **Bỏ chọn** và giữ nguyên hunk làm tròn cho commit thứ hai.

## 4. Cũng việc đó bằng lệnh: `git add -p`

`git add -p` (patch) đi qua từng hunk và hỏi bạn làm gì với nó. Lab trả lời `y` (stage) cho hunk 1 và `n` (bỏ qua) cho hunk 2:

```sh
git add -p src/BaoGia/FareCalculator.cs
```

```text
@@ -9,7 +9,8 @@ public static class FareCalculator
 …
(1/2) Stage this hunk [y,n,q,a,d,j,J,g,/,e,p,?]? y
@@ -18,6 +19,7 @@ public static class FareCalculator
 …
(2/2) Stage this hunk [y,n,q,a,d,K,g,/,e,p,?]? n
```

(Đã lược phần diff lặp lại; `OUTPUT.txt` của lab có bản đầy đủ. Khi chạy tay, `y`/`n` là chữ bạn gõ.)

| Phím | Nghĩa |
|---|---|
| `y` / `n` | Stage / không stage hunk này |
| `s` | Chia nhỏ hunk (khi hunk gồm nhiều cụm cách nhau bởi dòng không đổi) |
| `e` | Sửa tay hunk trong trình soạn thảo: cách stage theo dòng bằng lệnh |
| `q` | Thoát, giữ những gì đã stage |
| `?` | Trợ giúp |

Kiểm tra kết quả:

```sh
git status --short
git diff --staged --stat
```

```text
MM src/BaoGia/FareCalculator.cs
 src/BaoGia/FareCalculator.cs | 3 ++-
 1 file changed, 2 insertions(+), 1 deletion(-)
```

`MM`: đã stage một phần (cột 1), còn phần chưa stage (cột 2). `git diff --staged` là thứ *sẽ vào commit*: chỉ 2 dòng thêm, 1 dòng bỏ của hunk thông báo lỗi.

## 5. Commit thứ nhất

```sh
git commit -m "Báo lỗi rõ hơn khi quãng đường âm"
```

```text
[main b56c460] Báo lỗi rõ hơn khi quãng đường âm
 1 file changed, 2 insertions(+), 1 deletion(-)
```

Trên GitStudio: đóng diff (`Esc`), gõ summary vào ô **Tóm tắt commit**, bấm **Commit thay đổi của 1 tệp**. Hunk làm tròn vẫn nằm lại ở **Tệp chưa stage**.

## 6. Commit thứ hai, có description

Thay đổi giá cần ghi lý do. Nội dung commit có hai phần:

- **Summary** (dòng đầu): đổi gì, một dòng, nên dưới 72 ký tự. Bộ đếm ở góc phải ô **Tóm tắt commit** của GitStudio đếm ngược từ 72. Viết như một mệnh lệnh ngắn, bắt đầu bằng động từ: "Thêm…", "Sửa…", "Làm tròn…", "Bỏ…". Không kết thúc bằng dấu chấm.
- **Description** (sau một dòng trống): vì sao đổi, ảnh hưởng gì, liên kết tới yêu cầu. Ô **Mô tả** của GitStudio. Không cần lặp lại *cái gì*, diff đã nói.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-commit-stage-hunk-amend/gitstudio-04.webp" alt="Ô commit của GitStudio: Tóm tắt commit Làm tròn cước lên bội số 1.000 đ với bộ đếm 40, Mô tả Khách trả tiền mặt không phải trả tiền lẻ, tài xế không phải thối; đồ thị đã có commit Báo lỗi rõ hơn khi quãng đường âm; nút Hoàn tác trên thanh công cụ đang sáng" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · summary và description; nút <b>Hoàn tác</b> trên thanh công cụ sáng lên sau commit đầu</figcaption></figure>

Bằng lệnh, mỗi `-m` là một đoạn; đoạn thứ hai trở đi thành description:

```sh
git add src/BaoGia/FareCalculator.cs
git commit -m "Làm tròn cước lên bội số 1.000 đ" -m "Khách trả tiền mặt không phải trả tiền lẻ, tài xế không phải thối."
```

```text
[main bed94fc] Làm tròn cước lên bội số 1.000 đ
 1 file changed, 2 insertions(+), 1 deletion(-)
```

Nội dung dài, nhiều dòng thì gõ `git commit` không có `-m`: Git mở trình soạn thảo đã cấu hình (GIT-002), dòng đầu là summary, để trống một dòng, rồi viết description.

| Nội dung commit tệ | Vì sao tệ | Tốt hơn |
|---|---|---|
| `update` | Không nói gì | `Làm tròn cước lên bội số 1.000 đ` |
| `Sửa FareCalculator` | Nói file, không nói việc | `Báo lỗi rõ hơn khi quãng đường âm` |
| `Sửa bug Lan báo hôm qua` | Người đọc sau này không biết "hôm qua" là gì | `Không tính phụ phí khi quãng đường bằng 0` |
| Summary 120 ký tự | Bị cắt ở mọi giao diện | Summary ngắn, chi tiết đưa xuống description |

## 7. Amend: sửa commit cuối

Minh nhận ra commit làm tròn thiếu test. Thay vì thêm commit "Thêm test làm tròn" (lịch sử lại có hai commit cho một việc), anh **amend**: gộp thay đổi đang stage vào commit cuối cùng, tạo commit mới thay thế nó.

Trên GitStudio: sửa file test, stage nó, đánh dấu **Sửa commit trước (amend)**. Ô nội dung tự điền lại summary và description của commit cuối để bạn sửa nếu muốn; nút đổi thành **Sửa commit trước (amend)**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-commit-stage-hunk-amend/gitstudio-05.webp" alt="GitStudio với ô Sửa commit trước (amend) đã đánh dấu, ô nội dung điền sẵn Làm tròn cước lên bội số 1.000 đ và mô tả cũ, Tệp đã stage có tests/BaoGia.Tests/FareCalculatorTests.cs, nút Sửa commit trước (amend)" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · amend: commit cuối được thay bằng commit mới có thêm file test</figcaption></figure>

Bằng lệnh (`--no-edit`: giữ nguyên nội dung commit):

```sh
git add tests/BaoGia.Tests/FareCalculatorTests.cs
git commit --amend --no-edit
git log --oneline -3
```

```text
[main 0d57e4b] Làm tròn cước lên bội số 1.000 đ
 Date: Thu Nov 5 09:05:00 2026 +0700
 2 files changed, 6 insertions(+), 1 deletion(-)
0d57e4b Làm tròn cước lên bội số 1.000 đ
b56c460 Báo lỗi rõ hơn khi quãng đường âm
d508151 Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ
```

Commit `bed94fc` đã **biến khỏi nhánh**, thay bằng `0d57e4b`. Amend không "sửa" commit; commit trong Git không bao giờ thay đổi được (GIT-008). Amend tạo một commit mới với cùng commit cha, rồi dời nhánh `main` sang commit mới. Commit cũ vẫn còn trong `.git` và trong **reflog** một thời gian (Phần 4), nên lỡ amend nhầm vẫn tìm lại được.

```sh
git show --stat --format=fuller HEAD
```

```text
commit 0d57e4b9d030ae41e97e6fc614a3c9cc19f0969c
Author:     Trần Minh <minh@toinoi.test>
AuthorDate: Thu Nov 5 09:05:00 2026 +0700
Commit:     Trần Minh <minh@toinoi.test>
CommitDate: Thu Nov 5 09:12:00 2026 +0700

    Làm tròn cước lên bội số 1.000 đ

    Khách trả tiền mặt không phải trả tiền lẻ, tài xế không phải thối.

 src/BaoGia/FareCalculator.cs              | 3 ++-
 tests/BaoGia.Tests/FareCalculatorTests.cs | 4 ++++
 2 files changed, 6 insertions(+), 1 deletion(-)
```

`AuthorDate` giữ thời điểm viết ban đầu (09:05), `CommitDate` là lúc amend (09:12). `git log` mặc định chỉ hiện `AuthorDate`.

### Quy tắc vàng của amend

**Chỉ amend commit chưa push.** Commit đã push lên remote có thể đã nằm trên máy người khác; amend tạo commit mới khác mã hash, lịch sử của bạn và của họ tách ra, và bạn phải force push để ghi đè (Phần 3–4). GitStudio hỏi lại "Sửa commit đã push?" khi bạn amend một commit đã có trên remote, kèm lời nhắc nhánh sẽ cần force push.

Thêm một lưới an toàn: ngay sau mỗi thao tác, nút **Hoàn tác** (`Ctrl+Z`) trên thanh công cụ của GitStudio hoàn tác được commit, amend và nhiều thao tác khác. GIT-035 giải thích nó làm gì bên dưới và giới hạn của nó.

## 8. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Phím tắt | Lệnh |
|---|---|---|---|
| Xem diff một file | Bấm file trong Tệp chưa stage | `J`/`K` chuyển file | `git diff <file>` |
| Stage một hunk | Stage hunk | | `git add -p` → `y` |
| Stage vài dòng | Bấm cột số dòng → Stage N dòng | `Shift` + bấm: chọn khoảng | `git add -p` → `e` |
| Bỏ một hunk khỏi working tree | Huỷ bỏ hunk | | `git restore -p <file>` |
| Commit | Tóm tắt commit + Mô tả → Commit | `Ctrl+Enter` | `git commit -m … -m …` |
| Stage tất cả rồi commit | | `Ctrl+Shift+Enter` | `git commit -am …` (chỉ file đã theo dõi) |
| Amend | Sửa commit trước (amend) | | `git commit --amend` |
| Hoàn tác thao tác vừa làm | Hoàn tác | `Ctrl+Z` | tuỳ thao tác (Phần 4) |

## 9. Lỗi thường gặp

- **Commit cả hunk chưa xem.** Bấm "Stage tất cả" khi file có thay đổi thử nghiệm (log tạm, đoạn code comment). Xem diff trước khi stage; `git diff --staged` trước khi commit.
- **Commit một nửa thay đổi làm build hỏng.** Stage theo hunk/dòng dễ tạo ra commit không biên dịch được (thiếu khai báo nằm ở hunk khác). Mỗi commit nên build và chạy test được; nếu không chắc, stash phần còn lại (Phần 2) rồi build thử.
- **Amend commit đã push** rồi push bình thường: bị từ chối (`non-fast-forward`). Đừng force push lên nhánh dùng chung khi chưa thống nhất với nhóm.
- **Quên `--no-edit`** khi amend bằng lệnh: Git mở trình soạn thảo với nội dung cũ; lưu và đóng là xong, không sao.
- **`git commit -am` tưởng là "commit tất cả".** `-a` chỉ stage file *đã theo dõi* có thay đổi; file mới (untracked) không được thêm.

## 10. Bài tập

**Bài 1.** Dựng lại trạng thái đầu bài (`npm run setup -- 05`). Trong GitStudio, thay vì stage hunk 1, hãy chọn dòng `-` và hai dòng `+` của hunk 1 bằng cột số dòng rồi **Stage 3 dòng**. Kết quả có khác stage hunk không?

<details><summary><b>Lời giải</b></summary><p>Không khác: hunk 1 chỉ có đúng ba dòng thay đổi đó, nên stage cả ba dòng tương đương stage cả hunk. <code>git diff --staged</code> giống hệt. Chỉ khác khi bạn chọn một phần các dòng thay đổi trong hunk.</p></details>

**Bài 2.** Viết lại summary cho các commit sau: `fix`, `Cập nhật appsettings.json theo yêu cầu`, `Thêm test, sửa lỗi làm tròn, đổi tên biến fare thành baseFare`.

<details><summary><b>Lời giải</b></summary><p>Ví dụ: <code>Không làm tròn khi cước đã là bội số 1.000 đ</code> (nói rõ lỗi nào); <code>Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ</code> (nói đổi gì, lý do để ở description); commit thứ ba nên tách thành ba commit, mỗi commit một việc.</p></details>

**Bài 3.** Sau khi amend, chạy `git reflog -3`. Tìm mã hash của commit trước khi amend và giải thích vì sao commit đó không còn trong `git log`.

<details><summary><b>Lời giải</b></summary><p>Reflog ghi mọi lần <code>HEAD</code> di chuyển: <code>commit (amend)</code> ở <code>0d57e4b</code>, ngay dưới là <code>commit</code> ở mã hash trước amend (trong lab là <code>bed94fc</code>). <code>git log</code> chỉ đi theo chuỗi commit cha từ nhánh hiện tại; commit cũ không còn nhánh nào trỏ tới và không là cha của commit nào trên nhánh, nên không hiện. Lab kiểm tra điều này ở dòng <code>PASS commit trước amend vẫn còn trong reflog</code>.</p></details>

## 11. Đọc tiếp

- [git-add, mục Interactive mode](https://git-scm.com/docs/git-add#_interactive_mode): đầy đủ các phím của `-p`.
- [git-commit](https://git-scm.com/docs/git-commit): `--amend`, `--reset-author`, `-a`.
- [How to Write a Git Commit Message](https://cbea.ms/git-commit/): bảy quy tắc viết nội dung commit (tiếng Anh, áp dụng tốt cho tiếng Việt).

Bài tiếp theo: [GIT-006 · Đọc lịch sử: log, đồ thị commit và tìm commit](/articles/git-lich-su-log-do-thi-commit).
