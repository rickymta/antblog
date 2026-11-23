---
slug: git-restore-bo-thay-doi
title: "Bỏ thay đổi trong working tree và staging area: git restore, restore --staged, restore -p và clean"
summary: "Phần 4 bắt đầu từ việc hoàn tác nhỏ nhất: chưa commit gì cả. Bài này phân biệt bỏ thay đổi trong working tree (restore) với bỏ khỏi staging area (restore --staged), bỏ từng hunk bằng restore -p, lấy một file như ở một commit cũ bằng --source, dọn file chưa track bằng clean -n/-f, và các nút Huỷ bỏ thay đổi, Huỷ bỏ hunk, Huỷ bỏ dòng của GitStudio, kèm cảnh báo: những thao tác này không hoàn tác được."
category: git
level: beginner
tags: [git, git-031, git-restore, staging-area, working-tree, git-clean, gitstudio, toi-noi]
language: vi
---

Thứ Ba 24/11/2026. Minh mở dịch vụ báo giá để thử vài ý tưởng: thêm "phí chờ" vào chú thích hàm tính cước, ghi một dòng nháp vào README, đổi số phút miễn phí từ 5 lên 10. Mười phút sau anh đổi ý về gần hết. Chưa có commit nào, nên đây là loại hoàn tác đơn giản nhất. Nhưng cũng là loại **không có đường lùi**: thay đổi chưa commit mà bị bỏ thì Git không giữ bản sao nào.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-031 · Phần 4/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-ba-vung-working-tree-staging-gitignore">GIT-004</a> (ba vùng), <a href="/articles/git-commit-stage-hunk-amend">GIT-005</a> (hunk). Lab: <code>npm run p04 -- 31</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 31</code>. Sau bài này, bạn bỏ được đúng phần thay đổi muốn bỏ (cả file, một hunk, hay chỉ bỏ khỏi staging area) và biết lúc nào nên stash thay vì bỏ.</p>
</div>

## 1. Hoàn tác nhắm vào vùng nào

| Muốn | Vùng bị đổi | Lệnh | Lệnh cũ tương đương |
|---|---|---|---|
| Bỏ sửa đổi trong file | Working tree (lấy lại từ staging area) | `git restore <file>` | `git checkout -- <file>` |
| Bỏ stage, giữ sửa đổi | Staging area (lấy lại từ HEAD) | `git restore --staged <file>` | `git reset HEAD <file>` |
| Bỏ cả hai | Cả hai | `git restore --staged --worktree <file>` | `git checkout HEAD -- <file>` |
| Lấy file như ở commit khác | Working tree | `git restore --source=<commit> <file>` | `git checkout <commit> -- <file>` |
| Xoá file chưa track | Working tree | `git clean -f` | |

`git restore` (Git 2.23) tách phần "khôi phục file" ra khỏi `git checkout` vốn làm quá nhiều việc. Lệnh cũ vẫn chạy và bạn sẽ gặp trong tài liệu cũ, kể cả trong chính GitStudio (mục 5).

## 2. Bỏ thay đổi một file, và bỏ stage

```sh
git status -s
git restore src/BaoGia/FareCalculator.cs
git status -s
```

```text
 M README.md
 M src/BaoGia/FareCalculator.cs
 M README.md
```

Chữ `M` ở cột phải: sửa trong working tree, chưa stage (GIT-004). `restore` chép phiên bản trong staging area đè lên file, nên FareCalculator.cs trở lại như lúc commit.

Lỡ `git add README.md` rồi mới thấy dòng nháp:

```sh
git add README.md
git status -s
git restore --staged README.md
git status -s
```

```text
M  README.md
 M README.md
```

`M` nhảy từ cột trái (đã stage) sang cột phải: thay đổi vẫn còn trong file, chỉ không còn trong lần commit tới. Đây là thao tác an toàn: không mất gì.

## 3. Bỏ từng hunk: `restore -p`

Minh sửa hai chỗ trong WaitingFee.cs: đổi số phút miễn phí và thêm một dòng TODO. Anh muốn bỏ chỗ thứ nhất, giữ chỗ thứ hai. `-p` (patch) hỏi từng hunk, giống `git add -p` ở GIT-005:

```sh
git restore -p src/BaoGia/WaitingFee.cs
```

```text
@@ -3,7 +3,7 @@ namespace ToiNoi.BaoGia;
 /// <summary>Phí chờ tại điểm đón: miễn phí 5 phút đầu, sau đó 1.000 đ mỗi phút, tối đa 30.000 đ.</summary>
 public static class WaitingFee
 {
-    private const int FreeMinutes = 5;
+    private const int FreeMinutes = 10;
 
(1/2) Discard this hunk from worktree [y,n,q,a,d,j,J,g,/,e,p,?]? y
@@ -11,6 +11,7 @@ public static class WaitingFee
         var billable = Math.Max(waitedMinutes - FreeMinutes, 0);
+        // TODO: xem lại mức tối đa
         return Math.Min(billable * 1000m, 30000m);
(2/2) Discard this hunk from worktree [y,n,q,a,d,K,g,/,e,p,?]? n
```

Sau đó `git diff` chỉ còn dòng TODO. `y` bỏ hunk, `n` giữ, `s` chia hunk lớn thành hunk nhỏ hơn, `q` dừng.

## 4. Lấy file như ở commit khác, và dọn file mới

```sh
git restore --source=v1.0.0 src/BaoGia/FareCalculator.cs
git diff --stat
git restore src/BaoGia/FareCalculator.cs
```

```text
 src/BaoGia/FareCalculator.cs | 7 +++----
 1 file changed, 3 insertions(+), 4 deletions(-)
```

`--source` lấy nội dung file ở bất kỳ commit, nhánh hay tag nào, ghi vào working tree; HEAD không đổi. Dùng để so sánh nhanh, hay để "cứu" một file bị sửa hỏng mà không đụng tới phần còn lại.

File **mới** (chưa từng `git add`) thì `restore` không đụng tới, vì Git không có phiên bản nào của nó để khôi phục. Dọn bằng `git clean`, luôn xem trước bằng `-n`:

```sh
git status -s
git clean -n
git clean -f
```

```text
?? ghi-chu-tam.txt
Would remove ghi-chu-tam.txt
Removing ghi-chu-tam.txt
```

`-d` để xoá cả thư mục chưa track, `-x` để xoá cả file bị `.gitignore` (bin/, obj/): cẩn thận với `-x`, nó xoá luôn file cấu hình cục bộ như `appsettings.Development.json`.

## 5. Trên GitStudio

Trong panel bên phải (khi chọn dòng thay đổi chưa commit ở đầu đồ thị):

- Chuột phải một file → **Huỷ bỏ thay đổi** (nhiều file: **Huỷ bỏ thay đổi của N tệp**). GitStudio hỏi "…? Thao tác này không thể hoàn tác." rồi chạy `git checkout -- <file>` cho file đã track và `git clean -f` cho file mới.
- Nút thùng rác trên đầu panel: **Huỷ bỏ tất cả thay đổi**.
- Mở diff của một file: mỗi hunk có nút **Huỷ bỏ hunk**; chọn vài dòng rồi **Huỷ bỏ N dòng**. GitStudio tạo một patch nhỏ từ phần đã chọn rồi chạy `git apply --reverse`, đúng như `restore -p` nhưng chọn được tới từng dòng.
- Bỏ stage: nút **Unstage** trên file trong danh sách "Tệp đã stage" (`git reset HEAD -- <file>`).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-restore-bo-thay-doi/gitstudio-01.webp" alt="Panel thay đổi chưa commit của GitStudio với README.md và FareCalculator.cs trong danh sách Tệp chưa stage; menu chuột phải của FareCalculator.cs mở, có mục Huỷ bỏ thay đổi" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Huỷ bỏ thay đổi của một file</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-restore-bo-thay-doi/gitstudio-02.webp" alt="Diff của WaitingFee.cs trong GitStudio với hai hunk: FreeMinutes đổi từ 5 thành 10 và một dòng TODO; mỗi hunk có nút Huỷ bỏ hunk" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · bỏ từng hunk trong diff</figcaption></figure>

**Hoàn tác (Ctrl+Z) của GitStudio không lấy lại được những gì đã Huỷ bỏ** (GIT-035): không có commit nào chứa thay đổi đó. Khi còn phân vân, **stash** (GIT-015) thay vì huỷ: stash có thể pop lại, huỷ thì không.

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Bỏ thay đổi một file | Chuột phải file → Huỷ bỏ thay đổi | `git restore <file>` |
| Bỏ mọi thay đổi | Huỷ bỏ tất cả thay đổi | `git restore .` + `git clean -fd` |
| Bỏ một hunk / vài dòng | Diff → Huỷ bỏ hunk / Huỷ bỏ N dòng | `git restore -p <file>` |
| Bỏ stage | Unstage | `git restore --staged <file>` |
| Lấy file ở commit cũ | | `git restore --source=<commit> <file>` |
| Xoá file chưa track | Huỷ bỏ thay đổi trên file mới | `git clean -n`, rồi `git clean -f` |

## 7. Lỗi thường gặp

- **`git restore .` mà quên có file mới**: file mới vẫn còn, `git status` vẫn chưa sạch. Thêm `git clean`.
- **`git clean -fdx` trong repository có cấu hình cục bộ**: mất file `.env`, `appsettings.Development.json`. Luôn `-n` trước.
- **Nhầm `restore` với `reset --hard`**: `restore` chỉ đụng file; `reset` dời cả nhánh (GIT-032).
- **Bỏ thay đổi rồi mới nhớ còn cần**: không cứu được bằng Git. Trình soạn thảo (VS Code, Rider) đôi khi còn Local History; tập thói quen stash.

## 8. Bài tập

**Bài 1.** Sửa README.md, `git add`, rồi sửa thêm một dòng nữa. `git restore README.md` bỏ phần nào? `git restore --staged README.md` thì sao?

<details><summary><b>Lời giải</b></summary><p><code>restore</code> lấy bản trong staging area đè lên file: mất dòng sửa sau, giữ dòng đã stage. <code>restore --staged</code> chỉ đưa staging area về HEAD; file trong working tree vẫn có cả hai dòng.</p></details>

**Bài 2.** Viết lệnh xem trước file nào sẽ bị xoá nếu dọn cả thư mục chưa track lẫn file bị ignore.

<details><summary><b>Lời giải</b></summary><p><code>git clean -n -d -x</code>.</p></details>

**Bài 3.** Lấy lại `docs/phi-cho.md` như ở commit trước commit gần nhất mà không đổi HEAD.

<details><summary><b>Lời giải</b></summary><p><code>git restore --source=HEAD~1 docs/phi-cho.md</code>. Thêm <code>--staged --worktree</code> để ghi cả vào staging area.</p></details>

## 9. Đọc tiếp

- [git-restore](https://git-scm.com/docs/git-restore), [git-clean](https://git-scm.com/docs/git-clean).
- [Pro Git, Undoing Things](https://git-scm.com/book/en/v2/Git-Basics-Undoing-Things).

Bài tiếp theo: [GIT-032 · Reset soft, mixed, hard](/articles/git-reset-soft-mixed-hard).
