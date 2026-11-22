---
slug: git-rebase-co-ban
title: "Rebase cơ bản: chép nhánh tính năng lên đầu main, xử lý xung đột giữa chừng, khi nào không rebase"
summary: "Nhánh phí chờ của Lan tách ra từ v1.1.0, trong khi main đã có revert và một commit sửa đúng dòng README cô cũng sửa. Bài này rebase nhánh lên đầu main để lịch sử thẳng, dừng ở xung đột, đọc trạng thái rebase (detached HEAD, A là main còn B là commit đang chép), giải quyết, rebase --continue, rồi merge fast-forward; kèm so sánh rebase với merge và quy tắc không rebase nhánh đã chia sẻ."
category: git
level: advanced
tags: [git, git-018, git-rebase, rebase-conflict, fast-forward, linear-history, gitstudio, toi-noi]
language: vi
---

Lan làm tính năng phí chờ (tài xế chờ quá 5 phút thì tính thêm) trên nhánh `tinh-nang/phi-cho-doi`, tách ra từ `v1.1.0` sáng 18/11/2026. Tới chiều, `main` đã có thêm hai commit: revert cấu hình giờ cao điểm (GIT-017) và một commit của Minh sửa câu mô tả dịch vụ trong `README.md`, đúng câu Lan cũng sửa.

Merge nhánh của Lan vào `main` sẽ tạo thêm một merge commit, và xung đột README vẫn phải giải quyết. Nhóm muốn lịch sử của tính năng nhỏ này **thẳng**, như thể Lan bắt đầu làm sau commit mới nhất của `main`. Đó là việc của **rebase**.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-018 · Phần 2/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-xung-dot-merge">GIT-014</a>, <a href="/articles/git-cherry-pick-revert">GIT-017</a>. Lab: <code>npm run p02 -- 18</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 18</code>. Sau bài này, bạn rebase được một nhánh, xử lý xung đột giữa chừng, hiểu rebase làm gì với mã hash, và biết khi nào không được rebase.</p>
</div>

## 1. Rebase là chuỗi cherry-pick

`git rebase main` (đứng ở nhánh của Lan) làm ba việc:

1. Tìm các commit có trên nhánh hiện tại mà không có trên `main` (`main..HEAD`).
2. Chuyển HEAD tới đầu `main` (detached HEAD).
3. **Cherry-pick** lần lượt từng commit đó (GIT-017), rồi dời nhánh tới commit cuối cùng.

Kết quả: cùng các thay đổi, nhưng là **commit mới, mã hash mới**, nằm sau `main`. Commit cũ không bị xoá ngay (reflog vẫn giữ), nhưng không còn nhánh nào trỏ tới.

## 2. Tình huống

```sh
git switch -c tinh-nang/phi-cho-doi v1.1.0
git commit -am "Nhắc thời gian chờ trong mô tả dịch vụ"
git add docs/phi-cho.md
git commit -m "Thêm tài liệu phí chờ"
git switch main
git commit -am "Nhắc giờ đặt chuyến trong mô tả dịch vụ"
git log --oneline --graph main tinh-nang/phi-cho-doi -6
```

```text
* d42d397 Nhắc giờ đặt chuyến trong mô tả dịch vụ
* b6829e3 Revert "Thêm hệ số phụ phí giờ cao điểm vào bảng giá"
| * aa8ac44 Thêm tài liệu phí chờ
| * f3496a8 Nhắc thời gian chờ trong mô tả dịch vụ
|/  
* 70b18f6 Thêm test cho phụ phí giờ cao điểm
*   e46c89f Merge branch 'tinh-nang/phu-phi-gio-cao-diem'
```

(Lược bớt dòng kết quả của từng commit.) Dòng 3 của README: Lan viết "nhận quãng đường, **thời gian chờ** và loại dịch vụ", Minh viết "nhận quãng đường, **giờ đặt** và loại dịch vụ".

## 3. Rebase và dừng ở xung đột

```sh
git switch tinh-nang/phi-cho-doi
git rebase main
```

```text
Auto-merging README.md
CONFLICT (content): Merge conflict in README.md
Rebasing (1/2)error: could not apply f3496a8... Nhắc thời gian chờ trong mô tả dịch vụ
hint: Resolve all conflicts manually, mark them as resolved with
hint: "git add/rm <conflicted_files>", then run "git rebase --continue".
hint: You can instead skip this commit: run "git rebase --skip".
hint: To abort and get back to the state before "git rebase", run "git rebase --abort".
hint: Disable this message with "git config set advice.mergeConflict false"
Could not apply f3496a8... # Nhắc thời gian chờ trong mô tả dịch vụ
```

`Rebasing (1/2)`: dừng ở commit thứ nhất trên hai. `git status` cho biết đang ở đâu trong quá trình:

```sh
git status
```

```text
interactive rebase in progress; onto d42d397
Last command done (1 command done):
   pick f3496a8 # Nhắc thời gian chờ trong mô tả dịch vụ
Next command to do (1 remaining command):
   pick aa8ac44 # Thêm tài liệu phí chờ
  (use "git rebase --edit-todo" to view and edit)
You are currently rebasing branch 'tinh-nang/phi-cho-doi' on 'd42d397'.
  (fix conflicts and then run "git rebase --continue")
  (use "git rebase --skip" to skip this patch)
  (use "git rebase --abort" to check out the original branch)

Unmerged paths:
  (use "git restore --staged <file>..." to unstage)
  (use "git add <file>..." to mark resolution)
	both modified:   README.md
```

(Git hiện dòng "interactive rebase" cả khi bạn không gõ `-i`: rebase hiện đại dùng chung bộ máy với rebase tương tác, Phần 4.)

Trên GitStudio: banner **Đang rebase. Giải quyết 1 tệp bị xung đột, stage chúng, rồi tiếp tục.**, ô nhánh trên thanh công cụ ghi **detached @ d42d397** (HEAD đang ở đầu `main`, chưa thuộc nhánh nào), README.md trong mục **Xung đột**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-rebase-co-ban/gitstudio-01.webp" alt="GitStudio trong lúc rebase: banner vàng Đang rebase với nút Huỷ thao tác và Tiếp tục, ô nhánh ghi detached @ d42d397, đồ thị có main ở Nhắc giờ đặt chuyến và nhánh tinh-nang/phi-cho-doi hai commit tách từ v1.1.0, bảng phải có mục Xung đột chứa README.md" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · rebase dừng giữa chừng: HEAD tách rời ở đầu <code>main</code></figcaption></figure>

## 4. Ours và theirs bị đảo trong rebase

```sh
git diff
```

```diff
diff --cc README.md
--- a/README.md
+++ b/README.md
@@@ -1,4 -1,4 +1,8 @@@
  # bao-gia
  
++<<<<<<< HEAD
 +Dịch vụ báo giá cước của Tới Nơi: nhận quãng đường, giờ đặt và loại dịch vụ,
++=======
+ Dịch vụ báo giá cước của Tới Nơi: nhận quãng đường, thời gian chờ và loại dịch vụ,
++>>>>>>> f3496a8 (Nhắc thời gian chờ trong mô tả dịch vụ)
  trả về giá dự kiến cho khách trước khi đặt chuyến.
```

Phần `HEAD` ("ours") là bản của **`main`** (nơi đang chép lên), phần dưới ("theirs") là **commit của Lan** đang được chép. Ngược với merge (GIT-014), nơi "ours" là nhánh của bạn. Trong rebase, "Lấy bản của mình" nghĩa là lấy bản của `main` và **bỏ** thay đổi của chính commit đang chép: chỗ rất hay nhầm.

Công cụ xung đột của GitStudio ghi rõ nguồn gốc nên đỡ nhầm hơn: khung **A** là "Commit d42d397 trên main", khung **B** là "Commit f3496a8 trên tinh-nang/phi-cho-doi~1".

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-rebase-co-ban/gitstudio-02.webp" alt="Công cụ xung đột của GitStudio cho README.md khi rebase: khung A Commit d42d397 trên main, khung B Commit f3496a8 trên tinh-nang/phi-cho-doi~1, mỗi khung có dòng 3 mô tả dịch vụ khác nhau, khung Kết quả bên dưới, nút Lưu và đánh dấu đã giải quyết" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · khi rebase, A là <code>main</code> (đích), B là commit đang được chép</figcaption></figure>

## 5. Giải quyết, đi tiếp, rồi fast-forward

Giữ cả hai ý: "nhận quãng đường, giờ đặt, thời gian chờ và loại dịch vụ,".

```sh
git add README.md
git rebase --continue
git log --oneline --graph -4
git log -2 --format="%h %an | viết %ad | commit %cd | %s" --date=format:%d/%m_%H:%M
```

```text
[detached HEAD 95720a8] Nhắc thời gian chờ trong mô tả dịch vụ
 1 file changed, 1 insertion(+), 1 deletion(-)
Rebasing (2/2)Successfully rebased and updated refs/heads/tinh-nang/phi-cho-doi.
* 61f5ed6 Thêm tài liệu phí chờ
* 95720a8 Nhắc thời gian chờ trong mô tả dịch vụ
* d42d397 Nhắc giờ đặt chuyến trong mô tả dịch vụ
* b6829e3 Revert "Thêm hệ số phụ phí giờ cao điểm vào bảng giá"
61f5ed6 Lê Lan | viết 18/11_11:00 | commit 18/11_16:00 | Thêm tài liệu phí chờ
95720a8 Lê Lan | viết 18/11_10:00 | commit 18/11_16:00 | Nhắc thời gian chờ trong mô tả dịch vụ
```

Commit thứ hai (tài liệu phí chờ) áp không xung đột. Hai commit của Lan giờ có mã hash mới (`f3496a8` → `95720a8`, `aa8ac44` → `61f5ed6`), giữ ngày viết, đổi ngày commit. Nhánh nằm thẳng sau `main`, nên merge chỉ cần fast-forward:

```sh
git switch main
git merge --ff-only tinh-nang/phi-cho-doi
git branch -d tinh-nang/phi-cho-doi
```

```text
Updating d42d397..61f5ed6
Fast-forward
 README.md       | 2 +-
 docs/phi-cho.md | 3 +++
 2 files changed, 4 insertions(+), 1 deletion(-)
 create mode 100644 docs/phi-cho.md
Deleted branch tinh-nang/phi-cho-doi (was 61f5ed6).
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-rebase-co-ban/gitstudio-03.webp" alt="Đồ thị GitStudio sau rebase và fast-forward: main thẳng với Thêm tài liệu phí chờ và Nhắc thời gian chờ trong mô tả dịch vụ của Lê Lan nằm ngay sau Nhắc giờ đặt chuyến của Trần Minh, không có merge commit; nhánh phat-hanh/1.0 vẫn tách từ v1.0.0" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · sau rebase, tính năng phí chờ nằm thẳng trên <code>main</code></figcaption></figure>

Trên GitStudio, rebase có ở menu chuột phải của nhánh: **Rebase <nhánh hiện tại> lên main**. GitStudio hỏi xác nhận "Các commit của … sẽ bị viết lại", rồi chạy; gặp xung đột thì hiện banner như trên, giải quyết rồi bấm **Tiếp tục**, hoặc **Huỷ thao tác** (= `git rebase --abort`, trả nhánh về đúng như trước khi rebase).

## 6. Rebase hay merge

| | Merge | Rebase |
|---|---|---|
| Lịch sử | Giữ nguyên hình dạng thật, có merge commit | Thẳng, như làm tuần tự |
| Mã hash commit của nhánh | Giữ nguyên | **Đổi hết** |
| Xung đột | Giải quyết một lần cho cả nhánh | Có thể phải giải quyết ở từng commit |
| An toàn với nhánh đã chia sẻ | Có | **Không** |

**Quy tắc vàng: không rebase commit đã push lên nhánh người khác đang dùng.** Rebase thay commit cũ bằng commit mới; người đã có commit cũ sẽ thấy lịch sử của họ và của bạn tách ra, phải force push và cả nhóm phải đồng bộ lại (Phần 3–4). Rebase nhánh **của riêng bạn**, chưa ai dựa vào, để dọn trước khi merge: đó là cách dùng đúng.

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Rebase nhánh hiện tại lên main | Chuột phải main → Rebase … lên main | `git rebase main` |
| Xem đang ở đâu | Banner Đang rebase, ô nhánh "detached @" | `git status` |
| Giải quyết xung đột | Công cụ xung đột (A = đích, B = commit đang chép) | sửa file, `git add` |
| Đi tiếp | Tiếp tục | `git rebase --continue` |
| Bỏ qua commit đang chép | | `git rebase --skip` |
| Huỷ toàn bộ | Huỷ thao tác | `git rebase --abort` |
| Merge sau rebase | Chuột phải → Fast-forward main tới … | `git merge --ff-only <nhánh>` |

## 8. Lỗi thường gặp

- **Rebase nhánh đã push rồi `git push --force`.** Ghi đè commit của người khác. Nếu buộc phải, dùng `--force-with-lease` và báo nhóm (Phần 3).
- **Nhầm ours/theirs khi rebase.** Xem mục 4; đọc nhãn A/B của GitStudio.
- **`git rebase --skip` khi chưa hiểu.** Bỏ hẳn commit đang chép, thay đổi của nó biến mất khỏi nhánh mới.
- **Quên đang rebase dở.** Commit lúc đó nằm trên detached HEAD. Banner nhắc; `git status` nhắc.
- **Rebase nhánh dài nhiều xung đột lặp lại.** Mỗi commit một lần giải quyết. Nhánh sống lâu thì merge `main` vào định kỳ, hoặc bật `rerere` (Phần 4).

## 9. Bài tập

**Bài 1.** Sau khi rebase và fast-forward, `git reflog -8 --format="%h %gs"` cho thấy những bước nào? Tìm lại mã hash cũ `aa8ac44`.

<details><summary><b>Lời giải</b></summary><p>Từ dưới lên: commit của Minh <code>d42d397</code>; <code>checkout: moving from main to tinh-nang/phi-cho-doi</code> ở <code>aa8ac44</code> (đầu nhánh cũ); <code>rebase (start)</code>, <code>rebase (continue)</code> ở <code>95720a8</code>, <code>rebase (pick)</code> và <code>rebase (finish)</code> ở <code>61f5ed6</code>; rồi checkout <code>main</code> và fast-forward. <code>git branch cuu aa8ac44</code> lấy lại nhánh cũ nếu rebase hỏng.</p></details>

**Bài 2.** Nếu dùng merge thay rebase (đứng ở `main`, `git merge tinh-nang/phi-cho-doi`), có tránh được xung đột README không? Lịch sử khác thế nào?

<details><summary><b>Lời giải</b></summary><p>Không: vẫn xung đột đúng dòng 3, giải quyết một lần. Lịch sử có thêm một merge commit với hai cha; hai commit của Lan giữ mã hash gốc <code>f3496a8</code>, <code>aa8ac44</code>.</p></details>

**Bài 3.** Đứng ở `main`, chạy `git rebase tinh-nang/phi-cho-doi` (đảo vai) ở đầu bài. Chuyện gì xảy ra với `main`?

<details><summary><b>Lời giải</b></summary><p>Các commit của <code>main</code> kể từ <code>v1.1.0</code> (revert, sửa README) bị chép lại lên sau nhánh của Lan: <code>main</code> bị viết lại lịch sử, mã hash đổi. Với nhánh dùng chung như <code>main</code>, đây chính là điều quy tắc vàng cấm.</p></details>

## 10. Đọc tiếp

- [git-rebase](https://git-scm.com/docs/git-rebase): `--onto`, `--autostash`, `--abort`.
- [Pro Git, Rebasing](https://git-scm.com/book/en/v2/Git-Branching-Rebasing): mục "The Perils of Rebasing".

Bài tiếp theo: [GIT-019 · Worktree: làm hai nhánh cùng lúc](/articles/git-worktree).
