---
slug: git-pull-request-code-review
title: "Pull request và code review: từ nhánh tính năng của Lan đến một commit gọn trên main"
summary: "Lan làm tính năng phí chờ trên nhánh riêng, push và mở pull request; Minh review và phát hiện code tính phí từ phút đầu trong khi tài liệu ghi miễn phí 5 phút. Bài này đi trọn vòng: push -u nhánh tính năng, review bằng log main..nhánh và diff main...nhánh (ba chấm), sửa theo review bằng commit mới, ba cách merge pull request (merge commit, squash, rebase), author và committer sau squash, xoá nhánh và vì sao branch -d từ chối sau squash. GitStudio: So sánh, Merge pull request…, Squash merge."
category: git
level: intermediate
tags: [git, git-026, pull-request, code-review, squash-merge, three-dot-diff, gitstudio, toi-noi]
language: vi
---

Thứ Bảy 21/11/2026, Lan bắt tay vào việc đầu tiên đúng nghĩa "làm nhóm": tính **phí chờ tại điểm đón**. Tài liệu `docs/phi-cho.md` (GIT-020) đã ghi luật: miễn phí 5 phút đầu, sau đó 1.000 đ mỗi phút, tối đa 30.000 đ. Lan không commit thẳng lên `main`; nhóm thống nhất mọi thay đổi đi qua **pull request** để có người thứ hai đọc trước khi vào `main`.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-026 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-github-gitlab-gitstudio">GIT-025</a>, <a href="/articles/git-merge-fast-forward-merge-commit">GIT-013</a> (merge, squash). Lab: <code>npm run p03 -- 26</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 26</code>. Sau bài này, bạn mở được pull request, review đúng phần thay đổi, sửa theo review, chọn được cách merge và dọn nhánh sau đó.</p>
</div>

## 1. Vòng đời một pull request

1. Tạo nhánh tính năng từ `main` mới nhất, commit.
2. Push nhánh với upstream, mở pull request (nhánh tính năng → `main`).
3. Người review đọc **phần thay đổi của nhánh**, bình luận.
4. Tác giả sửa bằng commit mới trên **cùng nhánh**, push; pull request tự cập nhật.
5. Được duyệt: merge (một trong ba cách ở mục 5).
6. Xoá nhánh trên remote và trên máy.

## 2. Lan: nhánh, commit, push

```sh
git switch -c tinh-nang/tinh-phi-cho
git add src tests
git commit -m "Tính phí chờ theo phút, tối đa 30.000 đ"
git push -u origin tinh-nang/tinh-phi-cho
```

```text
Switched to a new branch 'tinh-nang/tinh-phi-cho'
[tinh-nang/tinh-phi-cho 6559212] Tính phí chờ theo phút, tối đa 30.000 đ
 2 files changed, 26 insertions(+)
 create mode 100644 src/BaoGia/WaitingFee.cs
 create mode 100644 tests/BaoGia.Tests/WaitingFeeTests.cs
branch 'tinh-nang/tinh-phi-cho' set up to track 'origin/tinh-nang/tinh-phi-cho'.
To <lab>/work/remote/bao-gia.git
 * [new branch]      tinh-nang/tinh-phi-cho -> tinh-nang/tinh-phi-cho
```

Trên GitHub, sau lần push này có ngay nút "Compare & pull request". Trên GitStudio: **Pull Request** trên thanh công cụ, hoặc chuột phải nhánh → **Tạo pull request…** (GIT-025). Một pull request tốt có tiêu đề nói **kết quả** ("Tính phí chờ tại điểm đón"), mô tả trả lời: làm gì, vì sao, kiểm tra thế nào.

## 3. Minh review: đúng phần cần đọc

```sh
git fetch
git log --oneline main..origin/tinh-nang/tinh-phi-cho
git diff --stat main...origin/tinh-nang/tinh-phi-cho
```

```text
From <lab>/work/remote/bao-gia
 * [new branch]      tinh-nang/tinh-phi-cho -> origin/tinh-nang/tinh-phi-cho
6559212 Tính phí chờ theo phút, tối đa 30.000 đ
 src/BaoGia/WaitingFee.cs              | 13 +++++++++++++
 tests/BaoGia.Tests/WaitingFeeTests.cs | 13 +++++++++++++
 2 files changed, 26 insertions(+)
```

Hai dấu chấm và ba dấu chấm nghĩa khác nhau, và chọn sai là review nhầm:

| Cú pháp | Với `log` | Với `diff` |
|---|---|---|
| `main..nhánh` | Commit có trong nhánh mà chưa có trong main | So **cây** của `main` với cây của nhánh (giống `diff main nhánh`) |
| `main...nhánh` | Commit ở một trong hai bên mà không ở bên kia | So **điểm rẽ nhánh** (merge-base) với nhánh: đúng phần nhánh đã thay đổi |

Nếu `main` có commit mới sau khi Lan tách nhánh, `git diff main nhánh` sẽ hiện cả những thay đổi đó (theo chiều ngược, như thể nhánh "xoá" chúng). `git diff main...nhánh` chỉ hiện việc của Lan; đây cũng là thứ GitHub hiển thị ở tab "Files changed".

```sh
git diff main...origin/tinh-nang/tinh-phi-cho -- src
```

```text
+/// <summary>Phí chờ tại điểm đón: 1.000 đ mỗi phút, tối đa 30.000 đ.</summary>
+public static class WaitingFee
+{
+    public static decimal Calculate(int waitedMinutes)
+    {
+        if (waitedMinutes < 0)
+            throw new ArgumentOutOfRangeException(nameof(waitedMinutes));
+
+        return Math.Min(waitedMinutes * 1000m, 30000m);
+    }
+}
```

Minh để lại nhận xét: "`docs/phi-cho.md` ghi chờ **quá 5 phút** mới tính, code đang tính từ phút đầu." Review tốt so code với yêu cầu (ở đây là tài liệu), không chỉ soi cú pháp.

Trên GitStudio, chuột phải `origin/tinh-nang/tinh-phi-cho` trên đồ thị → **So sánh main → origin/tinh-nang/tinh-phi-cho**: bảng bên phải liệt kê file thay đổi và "N commit chỉ có ở …". Lưu ý GitStudio so **cây với cây** (giống `git diff main nhánh`, hai chấm). Khi `main` đã đi tiếp, chọn mốc là commit rẽ nhánh: chuột phải commit đó → **Đánh dấu làm mốc so sánh**, rồi chuột phải nhánh → So sánh.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-pull-request-code-review/gitstudio-01.webp" alt="GitStudio mở commit Tính phí chờ theo phút, tối đa 30.000 đ của Lê Lan trên origin/tinh-nang/tinh-phi-cho: bảng bên phải liệt kê WaitingFee.cs và WaitingFeeTests.cs, ở giữa là diff của WaitingFee.cs với dòng return Math.Min(waitedMinutes * 1000m, 30000m)" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · đọc commit của nhánh pull request: dòng 11 tính phí từ phút đầu</figcaption></figure>

## 4. Lan sửa theo review

```sh
git commit -am "Miễn phí 5 phút chờ đầu tiên theo tài liệu"
git push
```

```text
[tinh-nang/tinh-phi-cho d0c8e3b] Miễn phí 5 phút chờ đầu tiên theo tài liệu
 2 files changed, 9 insertions(+), 3 deletions(-)
To <lab>/work/remote/bao-gia.git
   6559212..d0c8e3b  tinh-nang/tinh-phi-cho -> tinh-nang/tinh-phi-cho
```

Sửa bằng **commit mới**, không amend rồi force push: người review chỉ cần đọc phần mới (`git log 6559212..origin/tinh-nang/tinh-phi-cho`), và bình luận cũ trên GitHub vẫn bám đúng dòng. Gộp commit cho gọn là việc của bước merge.

## 5. Ba cách merge một pull request

| Cách (GitHub) | Lệnh tương đương | `main` nhận được | Hợp khi |
|---|---|---|---|
| Create a merge commit | `git merge --no-ff nhánh` | Mọi commit của nhánh + một merge commit | Muốn giữ nguyên từng bước và biết commit nào đi cùng pull request nào |
| Squash and merge | `git merge --squash nhánh` + `git commit` | **Một** commit mới chứa toàn bộ thay đổi | Commit trên nhánh lặt vặt ("sửa theo review"); mỗi pull request là một đơn vị |
| Rebase and merge | `git rebase main nhánh` + fast-forward | Các commit của nhánh, chép lại lên đầu `main` | Commit trên nhánh đã sạch, muốn lịch sử thẳng |

Nhóm Tới Nơi chọn squash cho pull request nhỏ. Minh làm tại máy (trên GitHub là nút "Squash and merge"):

```sh
git fetch
git merge --squash origin/tinh-nang/tinh-phi-cho
git commit --author="Lê Lan <lan@toinoi.test>" -m "Tính phí chờ tại điểm đón" -m "Pull request của Lê Lan (2 commit), Trần Minh review: miễn phí 5 phút đầu, tối đa 30.000 đ."
git log -1 --format=fuller --stat
```

```text
Squash commit -- not updating HEAD
 src/BaoGia/WaitingFee.cs              | 16 ++++++++++++++++
 tests/BaoGia.Tests/WaitingFeeTests.cs | 16 ++++++++++++++++
 2 files changed, 32 insertions(+)
[main cb31b2e] Tính phí chờ tại điểm đón
 Author: Lê Lan <lan@toinoi.test>
commit cb31b2e274c4e4503d5a4fdac87e62e90000d87d
Author:     Lê Lan <lan@toinoi.test>
AuthorDate: Sat Nov 21 10:00:00 2026 +0700
Commit:     Trần Minh <minh@toinoi.test>
CommitDate: Sat Nov 21 10:00:00 2026 +0700

    Tính phí chờ tại điểm đón
    
    Pull request của Lê Lan (2 commit), Trần Minh review: miễn phí 5 phút đầu, tối đa 30.000 đ.
```

`--squash` đưa toàn bộ thay đổi vào staging area mà không tạo commit. Mỗi commit có hai người: **author** (người viết thay đổi) và **committer** (người đưa nó vào lịch sử). `--author` giữ công cho Lan; Minh là committer. "Squash and merge" trên GitHub làm đúng như vậy, và tự ghi số pull request vào tiêu đề commit.

```sh
git push
git push origin --delete tinh-nang/tinh-phi-cho
```

```text
   dbdf678..cb31b2e  main -> main
 - [deleted]         tinh-nang/tinh-phi-cho
```

Trên GitStudio có hai đường:

- Pull request trên GitHub: thanh bên → **Pull request** → chuột phải → **Merge pull request…**. Hộp thoại kiểm tra với GitHub (đã duyệt chưa, CI đã qua chưa, có xung đột không), cho chọn **Cách merge** (**Tạo commit merge**, **Squash rồi merge**, **Rebase rồi merge**), tick **Xoá &lt;nhánh&gt; sau khi merge**, rồi **Merge trên GitHub**. Khi có xung đột thì có **Giải quyết xung đột tại đây**: merge tại máy, push, rồi merge pull request.
- Tại máy, như lab: chuột phải `origin/tinh-nang/tinh-phi-cho` → **Squash merge origin/tinh-nang/tinh-phi-cho vào main**, hoặc **Xem trước merge … vào main…** (GIT-013) rồi chọn chế độ **Squash: stage toàn bộ thay đổi, bạn tự commit**. GitStudio stage toàn bộ và nhắc "Thay đổi của … đã được stage: hãy commit để hoàn tất squash merge".

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-pull-request-code-review/gitstudio-02.webp" alt="Xem trước merge origin/tinh-nang/tinh-phi-cho vào main trong GitStudio, ô chế độ đang chọn Squash: stage toàn bộ thay đổi, bạn tự commit" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · merge kiểu squash tại máy</figcaption></figure>

## 6. Dọn nhánh, và vì sao `-d` từ chối

```sh
git switch main
git pull --ff-only --prune
git branch -d tinh-nang/tinh-phi-cho
git branch -D tinh-nang/tinh-phi-cho
```

```text
 - [deleted]         (none)     -> origin/tinh-nang/tinh-phi-cho
   dbdf678..cb31b2e  main       -> origin/main
error: the branch 'tinh-nang/tinh-phi-cho' is not fully merged
hint: If you are sure you want to delete it, run 'git branch -D tinh-nang/tinh-phi-cho'
Deleted branch tinh-nang/tinh-phi-cho (was d0c8e3b).
```

`--prune` xoá `origin/tinh-nang/tinh-phi-cho` vì nhánh trên remote đã bị xoá (GIT-022). `git branch -d` chỉ xoá nhánh mà mọi commit đã nằm trong nhánh hiện tại; squash tạo commit **mới** `cb31b2e`, còn `6559212` và `d0c8e3b` không nằm trên `main`. Git không biết nội dung đã vào bằng đường khác, nên đòi `-D`. Trước khi `-D`, kiểm tra nhanh: `git diff main tinh-nang/tinh-phi-cho` rỗng là nội dung đã có đủ trên `main`.

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Push nhánh tính năng | Push (lần đầu tự đặt upstream) | `git push -u origin <nhánh>` |
| Mở pull request | Pull Request; chuột phải nhánh → Tạo pull request… | `gh pr create` |
| Xem commit của nhánh | Chọn nhánh trên đồ thị | `git log main..origin/<nhánh>` |
| Xem thay đổi của nhánh | Chuột phải nhánh → So sánh | `git diff main...origin/<nhánh>` |
| Merge pull request | Pull request → Merge pull request… | `gh pr merge --squash` |
| Squash tại máy | Xem trước merge → Squash | `git merge --squash` + `git commit` |
| Dọn | Tự động prune; chuột phải nhánh → Xoá | `git fetch --prune`, `git branch -D` |

## 8. Lỗi thường gặp

- **Review bằng `git diff main nhánh`** khi `main` đã đi tiếp: thấy cả thay đổi của người khác. Dùng ba chấm.
- **Pull request quá lớn.** 2.000 dòng thì không ai review thật. Chia nhỏ theo bước có thể merge độc lập.
- **Amend rồi force push sau khi đã có review**: bình luận cũ mất ngữ cảnh, người review phải đọc lại từ đầu.
- **Tiếp tục commit trên nhánh đã squash merge.** Lần merge sau, các commit cũ lại xuất hiện và dễ xung đột. Sau squash, tạo nhánh mới từ `main`.
- **Quên `--author` khi squash tại máy:** commit ghi tên người merge, mất công của người viết.

## 9. Bài tập

**Bài 1.** Sau khi Minh merge, Lan quên xoá nhánh và commit thêm một dòng lên `tinh-nang/tinh-phi-cho`. `git log --oneline main..tinh-nang/tinh-phi-cho` in mấy commit? Vì sao?

<details><summary><b>Lời giải</b></summary><p>Ba: <code>6559212</code>, <code>d0c8e3b</code> và commit mới. Squash không làm hai commit cũ trở thành tổ tiên của <code>main</code>. Đó là lý do phải xoá nhánh sau squash.</p></details>

**Bài 2.** Viết lệnh xem riêng phần Lan sửa sau review, so với lần Minh đọc trước đó (`6559212`).

<details><summary><b>Lời giải</b></summary><p><code>git diff 6559212 origin/tinh-nang/tinh-phi-cho</code>, hoặc <code>git log -p 6559212..origin/tinh-nang/tinh-phi-cho</code>.</p></details>

**Bài 3.** Merge bằng "Rebase and merge" thì `git branch -d` ở máy Lan có từ chối không?

<details><summary><b>Lời giải</b></summary><p>Có, nếu rebase tạo commit mới (khi <code>main</code> đã đi tiếp, hoặc GitHub luôn ghi lại committer nên luôn tạo commit mới). Chỉ "Create a merge commit" giữ nguyên commit của nhánh, khi đó <code>-d</code> chạy được.</p></details>

## 10. Đọc tiếp

- [GitHub Docs: About pull request merges](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/about-pull-request-merges).
- [git-diff](https://git-scm.com/docs/git-diff): mục `git diff A...B`.
- [Google Engineering Practices: Code Review](https://google.github.io/eng-practices/review/).

Bài tiếp theo: [GIT-027 · Chiến lược nhánh cho nhóm](/articles/git-chien-luoc-nhanh).
