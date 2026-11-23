---
slug: git-fork-upstream
title: "Fork và upstream: đóng góp khi không có quyền push vào repository gốc"
summary: "Với dự án mã nguồn mở, hay khi công ty chỉ cho vài người push vào repository chính, bạn làm việc qua fork. Bài này giải thích fork là bản sao trên máy chủ, đặt upstream cho repository gốc và origin cho fork, đẩy nhánh lên fork, đồng bộ với upstream bằng fetch và rebase trước khi mở pull request, phía người nhận lấy nhánh từ fork về, rồi dọn dẹp. GitStudio: hai remote ở thanh bên, Pull từ upstream, Push lên origin, Sửa remote…"
category: git
level: advanced
tags: [git, git-028, fork, upstream, remote, open-source, pull-request, gitstudio, toi-noi]
language: vi
---

Thứ Hai 23/11/2026, Tới Nơi đổi quy định: chỉ trưởng nhóm (Minh) có quyền ghi vào repository chung; người khác đóng góp qua **fork**. Đây cũng chính là cách bạn đóng góp cho một dự án mã nguồn mở trên GitHub: bạn không có quyền push vào repository của họ, nhưng có thể tạo bản sao của riêng mình, làm việc ở đó, rồi đề nghị họ lấy về.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-028 · Phần 3/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-pull-request-code-review">GIT-026</a>, <a href="/articles/git-ahead-behind-push-bi-tu-choi">GIT-023</a> (force-with-lease). Lab: <code>npm run p03 -- 28</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 28</code>. Sau bài này, bạn thiết lập được cặp remote origin/upstream, giữ fork đồng bộ với repository gốc và gửi được pull request từ fork.</p>
</div>

## 1. Fork là gì

**Fork** là một bản sao repository **trên máy chủ**, thuộc tài khoản của bạn. Nó không phải khái niệm của Git: GitHub/GitLab sao chép repository (và nhớ quan hệ "fork của") để pull request từ fork về repository gốc biết đường đi. Sau khi fork, có **ba** repository:

| Repository | Ở đâu | Tên remote quy ước trên máy bạn | Bạn có quyền |
|---|---|---|---|
| Gốc | `toinoi/bao-gia` | **`upstream`** | Đọc |
| Fork | `lan/bao-gia` | **`origin`** | Đọc, ghi |
| Bản trên máy | máy của Lan | | Tất cả |

Luồng một chiều: kéo từ `upstream`, đẩy lên `origin`, mở pull request từ `origin` vào `upstream`.

Lab mô phỏng nút "Fork" bằng `git clone --bare` repository chung sang một bare repository khác (thuộc "tài khoản" của Lan):

```text
$ git clone --bare <lab>/work/remote/bao-gia.git <lab>/work/remote/lan-bao-gia.git
Cloning into bare repository '<lab>/work/remote/lan-bao-gia.git'...
done.
```

## 2. Đặt hai remote

Lan đã clone repository chung từ GIT-021, nên `origin` đang trỏ vào repository gốc. Đổi tên nó thành `upstream`, thêm fork làm `origin`:

```sh
git remote rename origin upstream
git remote add origin <lab>/work/remote/lan-bao-gia.git
git fetch origin
git remote -v
```

```text
origin	<lab>/work/remote/lan-bao-gia.git (fetch)
origin	<lab>/work/remote/lan-bao-gia.git (push)
upstream	<lab>/work/remote/bao-gia.git (fetch)
upstream	<lab>/work/remote/bao-gia.git (push)
```

`git remote rename` đổi luôn tên các remote-tracking (`origin/main` thành `upstream/main`) và upstream của nhánh `main` (giờ theo dõi `upstream/main`, đúng ý: `git pull` trên `main` kéo từ repository gốc). Bắt đầu từ đầu thì làm ngược lại: clone **fork** (thành `origin`), rồi `git remote add upstream <url gốc>`. Với GitHub CLI, `gh repo fork --clone` làm cả hai.

Trên GitStudio, mục **Remote** ở thanh bên hiện cả `origin` và `upstream`, mỗi remote một cây nhánh. Chuột phải remote có **Fetch**, **Pull từ upstream**, **Push &lt;nhánh&gt; lên origin**, **Sửa remote…** (đổi tên, URL pull, URL push), **Sao chép URL**, **Xoá**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-fork-upstream/gitstudio-01.webp" alt="GitStudio mở repository của Lan: mục Remote ở thanh bên có origin (fork) và upstream (repository chung); menu chuột phải của upstream mở với Fetch upstream, Pull từ upstream, Sửa remote…, Sao chép URL, Xoá upstream" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · hai remote: <code>origin</code> là fork, <code>upstream</code> là repository gốc</figcaption></figure>

## 3. Làm việc và đẩy lên fork

```sh
git switch -c sua/tai-lieu-phi-cho
git commit -am "Nói rõ phí chờ tính thêm mỗi phút"
git push -u origin sua/tai-lieu-phi-cho
```

```text
[sua/tai-lieu-phi-cho 8e97cd9] Nói rõ phí chờ tính thêm mỗi phút
 1 file changed, 1 insertion(+), 1 deletion(-)
branch 'sua/tai-lieu-phi-cho' set up to track 'origin/sua/tai-lieu-phi-cho'.
To <lab>/work/remote/lan-bao-gia.git
 * [new branch]      sua/tai-lieu-phi-cho -> sua/tai-lieu-phi-cho
```

Luôn làm trên **nhánh riêng**, không trên `main` của fork. `main` của fork chỉ để phản chiếu `main` của repository gốc.

## 4. Đồng bộ với upstream trước khi mở pull request

Trong lúc đó Minh push `22a7c58 Hướng dẫn báo lỗi qua issue` lên repository gốc. Nhánh của Lan rẽ từ commit cũ. Trước khi mở pull request, cô đưa nhánh lên đầu `upstream/main`:

```sh
git fetch upstream
git rebase upstream/main
git push --force-with-lease
```

```text
From <lab>/work/remote/bao-gia
   cb31b2e..22a7c58  main       -> upstream/main
Successfully rebased and updated refs/heads/sua/tai-lieu-phi-cho.
To <lab>/work/remote/lan-bao-gia.git
 + 8e97cd9...951c43d sua/tai-lieu-phi-cho -> sua/tai-lieu-phi-cho (forced update)
```

Đây là đúng trường hợp GIT-023 cho phép force push: nhánh của riêng Lan, trên fork của riêng Lan, sau khi chính cô viết lại lịch sử. Người nhận pull request được một nhánh nối thẳng từ `main` mới nhất, không xung đột.

Trên GitHub, nút "Sync fork" làm phần cập nhật `main` của fork; nhánh tính năng vẫn phải rebase (hoặc merge) như trên.

## 5. Phía người nhận

Pull request từ fork merge được ngay trên GitHub. Khi cần chạy thử tại máy, Minh lấy nhánh từ fork về: thêm fork làm remote tạm (hoặc dùng `refs/pull/<số>/head`, GIT-025):

```sh
git remote add lan-fork <lab>/work/remote/lan-bao-gia.git
git fetch lan-fork
git merge --no-ff --no-edit lan-fork/sua/tai-lieu-phi-cho
git push
git remote remove lan-fork
git log --oneline --graph -4
```

```text
 * [new branch]      sua/tai-lieu-phi-cho -> lan-fork/sua/tai-lieu-phi-cho
Merge made by the 'ort' strategy.
*   d263a4d Merge remote-tracking branch 'lan-fork/sua/tai-lieu-phi-cho'
|\  
| * 951c43d Nói rõ phí chờ tính thêm mỗi phút
|/  
* 22a7c58 Hướng dẫn báo lỗi qua issue
* cb31b2e Tính phí chờ tại điểm đón
```

Lần này Minh chọn merge commit (`--no-ff`) để lịch sử ghi rõ đóng góp đến từ fork. Merge commit vẫn giữ nguyên `951c43d` của Lan, nên phía Lan `git branch -d` chạy được (khác với squash ở GIT-026).

## 6. Lan dọn dẹp

```sh
git switch main
git pull --ff-only upstream main
git push origin main
git push origin --delete sua/tai-lieu-phi-cho
git branch -d sua/tai-lieu-phi-cho
```

```text
Updating cb31b2e..d263a4d
Fast-forward
 README.md       | 2 ++
 docs/phi-cho.md | 2 +-
Deleted branch sua/tai-lieu-phi-cho (was 951c43d).
```

`main` của Lan và của fork giờ trùng với repository gốc. Nhánh tiếp theo lại tách từ đây.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-fork-upstream/gitstudio-02.webp" alt="Đồ thị GitStudio của Lan sau khi dọn: main, origin/main và upstream/main cùng ở merge commit Merge remote-tracking branch lan-fork/sua/tai-lieu-phi-cho, bên dưới là commit Nói rõ phí chờ tính thêm mỗi phút" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · <code>main</code>, <code>origin/main</code> (fork) và <code>upstream/main</code> trùng nhau sau khi đồng bộ</figcaption></figure>

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Đổi tên remote | Chuột phải remote → Sửa remote… | `git remote rename origin upstream` |
| Thêm fork | Remote → `+` (nguồn URL hoặc GitHub) | `git remote add origin <url fork>` |
| Kéo từ repository gốc | Chuột phải upstream → Pull từ upstream | `git pull upstream main` |
| Đẩy nhánh lên fork | Chuột phải origin → Push &lt;nhánh&gt; lên origin | `git push -u origin <nhánh>` |
| Đưa nhánh lên đầu gốc | Kéo nhánh lên `upstream/main` → Rebase | `git rebase upstream/main` |
| Lấy nhánh từ fork người khác | Thêm remote tạm, Fetch | `git remote add`, `git fetch`, hoặc `gh pr checkout` |

## 8. Lỗi thường gặp

- **Làm việc trên `main` của fork.** Sau vài pull request, `main` của fork rẽ khỏi gốc và mỗi lần đồng bộ là một lần xung đột. Luôn dùng nhánh riêng.
- **Đẩy nhầm lên `upstream`.** Nếu có quyền ghi (như Minh), lệnh `git push upstream` sẽ thành công. Đặt `git remote set-url --push upstream no-push` để vô hiệu hoá push tới đó.
- **Pull request đầy commit lạ.** Nhánh được tách từ `main` cũ của fork chưa đồng bộ. Rebase lên `upstream/main` rồi force-with-lease.
- **Fork của repository riêng tư** có quyền theo tổ chức; khi bị rút quyền vào repository gốc, fork có thể bị xoá theo.

## 9. Bài tập

**Bài 1.** Ở máy Lan, `git rev-parse --abbrev-ref main@{upstream}` in gì sau mục 2? Sau `git branch -u origin/main main`?

<details><summary><b>Lời giải</b></summary><p>Sau mục 2: <code>upstream/main</code> (vì <code>remote rename</code> cập nhật cấu hình nhánh). Sau lệnh thứ hai: <code>origin/main</code>; khi đó <code>git pull</code> trên <code>main</code> kéo từ fork, thường không phải điều bạn muốn.</p></details>

**Bài 2.** Viết các lệnh để `main` của fork bắt kịp repository gốc mà không cần checkout `main` (đang ở nhánh khác).

<details><summary><b>Lời giải</b></summary><p><code>git fetch upstream</code> rồi <code>git push origin upstream/main:main</code>. Refspec <code>nguồn:đích</code> đẩy thẳng commit của <code>upstream/main</code> lên nhánh <code>main</code> của fork.</p></details>

**Bài 3.** Vì sao bước 4 dùng rebase mà không merge `upstream/main` vào nhánh?

<details><summary><b>Lời giải</b></summary><p>Nhánh của riêng Lan, chưa ai dựa vào; rebase cho lịch sử thẳng và pull request chỉ gồm commit của cô. Merge cũng đúng, nhưng thêm một merge commit "Merge branch 'main' of …" vào pull request. Nhiều dự án yêu cầu rebase; đọc CONTRIBUTING của dự án.</p></details>

## 10. Đọc tiếp

- [GitHub Docs: Working with forks](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/working-with-forks), [Syncing a fork](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/working-with-forks/syncing-a-fork).
- [Pro Git, Contributing to a Project: Forked Public Project](https://git-scm.com/book/en/v2/Distributed-Git-Contributing-to-a-Project).

Bài tiếp theo: [GIT-029 · Pull có xung đột và lịch sử rối](/articles/git-pull-xung-dot-lich-su-roi).
