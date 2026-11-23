---
slug: git-remote-origin-tracking
title: "Remote, origin và nhánh remote-tracking: đưa repository báo giá lên chỗ chung của nhóm"
summary: "Repository báo giá đến giờ chỉ nằm trên máy Minh. Bài này giải thích remote là gì, tạo remote chung, git remote add origin, push lần đầu với -u để đặt upstream, đẩy nhánh phát hành và tag, đọc origin/main, git branch -vv, git ls-remote, git remote show, rồi để Lan clone về; trên GitStudio là mục Remote ở thanh bên và các nút Push, Pull."
category: git
level: beginner
tags: [git, git-021, remote, origin, remote-tracking, upstream, git-push, gitstudio, toi-noi]
language: vi
---

Hai phần đầu, mọi thứ diễn ra trên máy Minh: Lan "có" commit chỉ vì lab dựng giúp. Thứ Sáu 20/11/2026, Lan bắt đầu làm thật trên máy của cô. Repository cần một **chỗ chung** mà cả hai cùng đẩy lên và kéo về: một **remote**. Thường đó là GitHub hay GitLab; lab của series dùng một bare repository trong thư mục `work/remote/` để không cần tài khoản hay mạng. Với Git, hai thứ này cư xử giống hệt nhau.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-021 · Phần 3/6 · Cơ bản</span>
<p>Đọc trước: Phần 2, nhất là <a href="/articles/git-nhanh-la-gi">GIT-011</a> (nhánh là con trỏ). Lab: <code>npm run p03 -- 21</code>; mở trạng thái cuối bài bằng GitStudio: <code>npm run setup -- 21 --end</code>. Sau bài này, bạn thêm được remote, push lần đầu có upstream, đọc được nhánh remote-tracking, và clone được repository chung.</p>
</div>

## 1. Remote là gì

Một **remote** là một repository khác mà repository của bạn biết địa chỉ, kèm một tên ngắn để gọi. Tên quy ước cho remote chính là **`origin`** (khi clone, Git tự đặt). Remote không tự đồng bộ: bạn chủ động **push** (đẩy commit lên) và **fetch/pull** (kéo về).

Ba loại nhánh cần phân biệt từ bài này:

| Loại | Ví dụ | Ở đâu | Ai cập nhật |
|---|---|---|---|
| Nhánh cục bộ | `main` | `.git/refs/heads/` trên máy bạn | Bạn commit |
| Nhánh trên remote | `main` của `origin` | Repository remote | Ai đó push |
| **Nhánh remote-tracking** | `origin/main` | `.git/refs/remotes/origin/` trên máy bạn | **Chỉ** khi bạn fetch/pull/push |

`origin/main` là **bản ghi nhớ** trên máy bạn về vị trí `main` của `origin` ở lần liên lạc gần nhất. Nó không tự cập nhật; GIT-022 xoay quanh chuyện đó.

## 2. Tạo remote chung và gắn vào repository

Trên GitHub/GitLab, bạn tạo repository trống bằng giao diện web. Lab làm cùng việc bằng một **bare repository** (repository chỉ có phần `.git`, không có working tree, đúng như trên máy chủ):

```sh
git remote -v
git init --bare -b main <lab>/work/remote/bao-gia.git
git remote add origin <lab>/work/remote/bao-gia.git
git remote -v
```

```text
Initialized empty Git repository in <lab>/work/remote/bao-gia.git/
origin	<lab>/work/remote/bao-gia.git (fetch)
origin	<lab>/work/remote/bao-gia.git (push)
```

Lệnh đầu không in gì: chưa có remote. `git remote add <tên> <url>` ghi địa chỉ vào `.git/config`. URL có thể là đường dẫn trên máy (lab), `https://github.com/…` hoặc `git@github.com:…` (GIT-024). Mỗi remote có địa chỉ fetch và push, thường giống nhau.

Trên GitStudio: thanh bên → mục **Remote** → nút `+` (**Thêm remote**) mở bảng **Thêm remote** với ba nguồn: **URL**, **GitHub**, **GitLab**. Với URL, điền **Tên** (`origin`), **URL pull**, và **URL push** nếu khác (để trống là giống URL pull); GitStudio thêm remote rồi fetch luôn. Nguồn GitHub/GitLab cho chọn từ danh sách repository của tài khoản đã kết nối (GIT-025).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-remote-origin-tracking/gitstudio-01.webp" alt="Bảng Thêm remote của GitStudio mở từ nút cộng ở mục Remote: ba nguồn URL, GitHub, GitLab; nguồn URL đang chọn với ô Tên là origin và ô URL pull là đường dẫn tới work/remote/bao-gia.git" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · thêm remote <code>origin</code> bằng URL</figcaption></figure>

## 3. Push lần đầu và upstream

```sh
git push -u origin main
git push origin phat-hanh/1.0
git push origin --tags
```

```text
branch 'main' set up to track 'origin/main'.
To <lab>/work/remote/bao-gia.git
 * [new branch]      main -> main
To <lab>/work/remote/bao-gia.git
 * [new branch]      phat-hanh/1.0 -> phat-hanh/1.0
To <lab>/work/remote/bao-gia.git
 * [new tag]         v1.0.0 -> v1.0.0
 * [new tag]         v1.0.1 -> v1.0.1
 * [new tag]         v1.1.0 -> v1.1.0
```

`git push origin main` đẩy nhánh `main` cục bộ lên nhánh `main` của `origin`. `-u` (`--set-upstream`) còn ghi rằng `main` cục bộ **theo dõi** `origin/main`: từ nay `git push`, `git pull`, `git status` trong nhánh này không cần nói remote nào, nhánh nào. Tag không tự lên (GIT-016), nên đẩy riêng bằng `--tags`.

```sh
git branch -vv
git branch -r
```

```text
* main          61f5ed6 [origin/main] Thêm tài liệu phí chờ
  phat-hanh/1.0 c6e65d7 Thêm ghi chú phát hành 1.0.1
  origin/main
  origin/phat-hanh/1.0
```

`[origin/main]` sau `main` là **upstream** của nó. `phat-hanh/1.0` đã lên remote nhưng chưa có upstream (không dùng `-u`); muốn đặt sau: `git branch -u origin/phat-hanh/1.0 phat-hanh/1.0`, hay trên GitStudio: chuột phải nhánh → **Đặt upstream**. `git branch -r` liệt kê các nhánh remote-tracking.

Trên GitStudio, nút **Push** trên thanh công cụ làm việc này cho nhánh hiện tại; lần đầu GitStudio tự đặt upstream. Mục **Remote** ở thanh bên mở ra thấy `origin` và các nhánh của nó; trên đồ thị, nhãn `origin/main` xuất hiện cạnh nhãn `main`.

## 4. Hỏi trực tiếp remote

`origin/main` là bản ghi nhớ; `git ls-remote` hỏi thẳng remote **ngay lúc này**:

```sh
git ls-remote origin
```

```text
61f5ed6fe9bd350f1159ce8c1276b36c44748e97	HEAD
61f5ed6fe9bd350f1159ce8c1276b36c44748e97	refs/heads/main
c6e65d730f741d9af24e5bc222acef8e501ae995	refs/heads/phat-hanh/1.0
42dff92596716b74170e440625b7647d3b6b7ebf	refs/tags/v1.0.0
89855d5ae9eeadc33e9969090faabe175773a520	refs/tags/v1.0.0^{}
…
```

Với annotated tag có hai dòng: mã hash của tag object, và `^{}` là commit nó trỏ tới (GIT-016).

## 5. Lan clone

```sh
git clone <lab>/work/remote/bao-gia.git ../lan/bao-gia
cd ../lan/bao-gia
git config user.name "Lê Lan"
git config user.email lan@toinoi.test
git branch -a
git remote show origin
```

```text
* main
  remotes/origin/HEAD -> origin/main
  remotes/origin/main
  remotes/origin/phat-hanh/1.0
* remote origin
  Fetch URL: <lab>/work/remote/bao-gia.git
  Push  URL: <lab>/work/remote/bao-gia.git
  HEAD branch: main
  Remote branches:
    main          tracked
    phat-hanh/1.0 tracked
  Local branch configured for 'git pull':
    main merges with remote main
  Local ref configured for 'git push':
    main pushes to main (up to date)
```

Clone tự làm mọi việc ở mục 2–3: đặt `origin`, tạo `main` theo dõi `origin/main`, tải mọi nhánh dưới dạng remote-tracking (`origin/phat-hanh/1.0`) và mọi tag. Lan chỉ có **một** nhánh cục bộ; muốn làm việc trên nhánh phát hành, cô `git switch phat-hanh/1.0` và Git tự tạo nhánh cục bộ theo dõi `origin/phat-hanh/1.0`. `origin/HEAD` cho biết nhánh mặc định của remote. `git remote show origin` liên lạc với remote và tóm tắt toàn bộ quan hệ giữa nhánh cục bộ và nhánh trên remote.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-remote-origin-tracking/gitstudio-02.webp" alt="GitStudio mở repository của Minh sau khi push: mục Remote mở rộng có origin với main và phat-hanh/1.0, trên đồ thị nhãn main có biểu tượng đám mây cho biết đã có trên remote" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · <code>origin</code> và các nhánh remote-tracking ở thanh bên</figcaption></figure>

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Xem remote | Thanh bên → Remote | `git remote -v` |
| Thêm remote | Remote → `+` → nguồn URL | `git remote add <tên> <url>` |
| Sửa URL / xoá remote | Chuột phải remote → Sửa remote… / Xoá | `git remote set-url`, `git remote remove` |
| Push lần đầu, đặt upstream | Push | `git push -u origin <nhánh>` |
| Đặt upstream sau | Chuột phải nhánh → Đặt upstream | `git branch -u origin/<nhánh>` |
| Push tag | Chuột phải tag → Push | `git push origin --tags` |
| Clone | Tab mới → Clone (GIT-003) | `git clone <url>` |
| Hỏi remote ngay lúc này | | `git ls-remote origin`, `git remote show origin` |

## 7. Lỗi thường gặp

- **`fatal: The current branch … has no upstream branch`.** Push nhánh mới mà quên `-u`. Git in sẵn lệnh cần chạy; hoặc đặt `git config --global push.autoSetupRemote true` để lần push đầu tự đặt upstream.
- **`remote origin already exists`.** Repository đã có `origin` (thường do clone). Dùng `git remote set-url origin <url>` để đổi địa chỉ.
- **Tưởng `origin/main` là "main trên máy chủ".** Nó là bản ghi nhớ ở lần fetch gần nhất; muốn biết hiện tại phải fetch hoặc `ls-remote`.
- **Quên push tag.** Bản phát hành có trên máy bạn mà không ai khác thấy.
- **Push nhánh cục bộ có tên khác nhánh trên remote** (`git push origin sua-loi:main`) mà không để ý: cú pháp `nguồn:đích` đẩy vào nhánh đích.

## 8. Bài tập

**Bài 1.** Sau mục 3, mở `.git/config` của Minh. Tìm các khối `[remote "origin"]` và `[branch "main"]`; dòng nào do `-u` ghi?

<details><summary><b>Lời giải</b></summary><p><code>[remote "origin"]</code> có <code>url</code> và <code>fetch = +refs/heads/*:refs/remotes/origin/*</code> (refspec: nhánh trên remote vào remote-tracking nào). <code>[branch "main"]</code> có <code>remote = origin</code> và <code>merge = refs/heads/main</code>: hai dòng này do <code>-u</code> ghi.</p></details>

**Bài 2.** Ở bản clone của Lan, chạy `git switch phat-hanh/1.0` rồi `git branch -vv`. Upstream của nhánh mới là gì?

<details><summary><b>Lời giải</b></summary><p><code>origin/phat-hanh/1.0</code>. Khi tên nhánh cần chuyển không có cục bộ nhưng có đúng một nhánh remote-tracking cùng tên, <code>git switch</code> tự tạo nhánh cục bộ theo dõi nó (chế độ "guess" mặc định).</p></details>

**Bài 3.** Vì sao remote trên máy chủ thường là bare repository?

<details><summary><b>Lời giải</b></summary><p>Không có working tree thì không ai "đang đứng" ở nhánh nào trên máy chủ, nên push vào bất kỳ nhánh nào cũng không xung đột với file đang mở. Git từ chối push vào nhánh đang checkout của một repository có working tree (mặc định <code>receive.denyCurrentBranch</code>).</p></details>

## 9. Đọc tiếp

- [git-remote](https://git-scm.com/docs/git-remote), [git-push](https://git-scm.com/docs/git-push), [git-ls-remote](https://git-scm.com/docs/git-ls-remote).
- [Pro Git, Working with Remotes](https://git-scm.com/book/en/v2/Git-Basics-Working-with-Remotes), [Remote Branches](https://git-scm.com/book/en/v2/Git-Branching-Remote-Branches).

Bài tiếp theo: [GIT-022 · Fetch, pull và push](/articles/git-fetch-pull-push).
