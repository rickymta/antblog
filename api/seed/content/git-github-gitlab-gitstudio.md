---
slug: git-github-gitlab-gitstudio
title: "GitHub và GitLab trong GitStudio: đăng repository, kết nối tài khoản, xem pull request"
summary: "Git không biết gì về pull request, issue hay tài khoản: đó là phần của dịch vụ lưu trữ. Bài này tách phần nào là Git, phần nào là GitHub/GitLab; đăng repository báo giá lên GitHub bằng Đăng lên GitHub…, thêm remote từ GitHub, kết nối tài khoản, mục Pull request ở thanh bên, Launchpad, sao chép liên kết commit; và cách lấy code của một pull request về máy bằng refs/pull/<số>/head."
category: git
level: intermediate
tags: [git, git-025, github, gitlab, pull-request, merge-request, launchpad, gitstudio, toi-noi]
language: vi
---

Sau GIT-024, nhóm Tới Nơi chuyển repository chung lên GitHub. Từ đây sẽ xuất hiện những thứ Git không hề có: **pull request**, review, issue, quyền thành viên, luật bảo vệ nhánh. Bài này vẽ ranh giới giữa Git và dịch vụ lưu trữ, rồi đi qua những gì GitStudio làm được với GitHub và GitLab mà không cần rời ứng dụng.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-025 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-remote-origin-tracking">GIT-021</a>, <a href="/articles/git-xac-thuc-https-ssh">GIT-024</a>. Lab: <code>npm run p03 -- 25</code> (mô phỏng ref của pull request trên remote cục bộ). Sau bài này, bạn biết phần nào là Git, phần nào là GitHub/GitLab, đăng được repository lên GitHub và lấy được code của một pull request về máy.</p>
</div>

## 1. Git và dịch vụ lưu trữ

| Thuộc về Git (chạy được với mọi remote) | Thuộc về GitHub / GitLab |
|---|---|
| Commit, nhánh, tag, merge, rebase | Pull request (GitHub) / merge request (GitLab) |
| Fetch, pull, push qua HTTPS hoặc SSH | Review, comment theo dòng, approve |
| Hook phía máy chủ (`pre-receive`) | Branch protection / protected branches (GIT-027) |
| | Issue, CI (GitHub Actions, GitLab CI), release |
| | Tài khoản, tổ chức, quyền đọc/ghi, fork (GIT-028) |

Hệ quả thực tế: mọi thứ bên trái làm được bằng `git` hay GitStudio với bất kỳ remote nào (kể cả thư mục trong lab). Bên phải cần **API** của dịch vụ, nên GitStudio phải được **kết nối tài khoản** (GIT-024 mục 6) mới hiện pull request hay tạo được repository.

Hai dịch vụ gọi tên khác nhau cho cùng một thứ: GitHub **pull request** `#12`, GitLab **merge request** `!12`. Từ đây series dùng "pull request" cho cả hai.

## 2. Đưa repository lên GitHub

Cách thủ công: tạo repository trống trên github.com (không tick "Add a README", để tránh lịch sử rẽ đôi ngay từ đầu), rồi làm như GIT-021:

```sh
git remote add origin https://github.com/toinoi/bao-gia.git
git push -u origin main
git push origin --tags
```

Trên GitStudio, khi repository chưa có remote nào và tài khoản GitHub đã kết nối, mục **Remote** ở thanh bên có liên kết **Đăng lên GitHub…**. Hộp thoại **Xuất bản lên GitHub** tạo repository dưới tài khoản của bạn, thêm nó làm `origin` và push nhánh hiện tại; có ô **Repository riêng tư** và **Dùng URL SSH cho remote**. Với GitHub CLI: `gh repo create toinoi/bao-gia --private --source . --push`.

Repository đã có trên GitHub thì dùng nút `+` của mục Remote, chọn nguồn **GitHub**: chọn từ danh sách repository của tài khoản thay vì gõ URL. Chưa kết nối thì bảng chỉ hiện "Chưa kết nối GitHub" kèm nút kết nối. Ở nguồn này GitStudio đọc danh sách repository của tài khoản qua API GitHub.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-github-gitlab-gitstudio/gitstudio-01.webp" alt="Hộp thoại Xuất bản lên GitHub của GitStudio mở từ liên kết Đăng lên GitHub… ở mục Remote khi repository chưa có remote: ô Tên repository bao-gia, Mô tả, Repository riêng tư đã chọn, Dùng URL SSH cho remote, nút Huỷ và Xuất bản" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Đăng lên GitHub…: tạo repository, thêm origin, push trong một bước (tên tài khoản trong ảnh đã thay)</figcaption></figure>

## 3. Pull request trong GitStudio

Khi `origin` là GitHub (hoặc GitLab) và tài khoản đã kết nối:

- **Thanh bên → Pull request**: danh sách pull request đang mở của repository. Nhấp đúp để mở trên web; chuột phải có **Sao chép liên kết**, **Checkout &lt;nhánh&gt;** (lấy nhánh của pull request về để chạy thử) và **Merge pull request…** (GIT-026).
- **Thanh công cụ → Pull Request**: tạo pull request từ nhánh hiện tại (nhánh chưa có trên GitHub thì GitStudio hỏi push trước). Hộp thoại có **Nhánh base**, **Tiêu đề**, **Mô tả**, **Tạo dưới dạng bản nháp**.
- **Chuột phải nhánh → Tạo pull request…**, hoặc chuột phải commit → **Tạo pull request lên origin từ &lt;nhánh&gt;**.
- **Launchpad** (`Ctrl+Shift+K`): pull request của tôi, nhờ tôi review, issue được giao, thay đổi chưa commit, gom từ mọi repository đang mở.
- Thanh trạng thái ở đáy cửa sổ hiện số pull request đang mở.
- Chuột phải commit → **Sao chép link commit này trên remote: origin**: liên kết `https://github.com/…/commit/<hash>` để dán vào chat hay issue.

GitLab có đủ các mục tương ứng (merge request), kết nối ở **Tuỳ chọn → Tích hợp → Kết nối GitLab.com** bằng token.

## 4. Pull request ở tầng Git: `refs/pull/<số>/head`

Pull request không phải khái niệm của Git, nhưng GitHub để lộ nó dưới dạng **ref** trên remote: `refs/pull/12/head` là commit cuối của pull request số 12 (GitLab: `refs/merge-requests/12/head`). Nhờ vậy, ai cũng lấy được code của một pull request, kể cả pull request từ fork mà bạn không có nhánh.

Lab mô phỏng: Lan push nhánh `thu/pr`, "GitHub" (remote cục bộ) gắn pull request số 12 cho commit đó.

```sh
git ls-remote origin "refs/pull/*"
git fetch origin pull/12/head:pr-12
git log --oneline -1 pr-12
git config --get remote.origin.fetch
```

```text
b9e865e38d4f0ad92616f97a6748383f54717ce2	refs/pull/12/head
From <lab>/work/remote/bao-gia
 * [new ref]         refs/pull/12/head -> pr-12
b9e865e Thử phí chờ
+refs/heads/*:refs/remotes/origin/*
```

`git fetch origin pull/12/head:pr-12` dùng **refspec** `nguồn:đích`: lấy `refs/pull/12/head` trên remote, ghi vào nhánh cục bộ `pr-12`. Refspec mặc định của `origin` (dòng cuối) chỉ lấy `refs/heads/*`, nên `git fetch` thường không bao giờ kéo `refs/pull/*` về; đó là chủ ý, vì một repository lớn có hàng nghìn pull request.

Muốn lấy luôn mọi pull request mỗi lần fetch, thêm một refspec:

```sh
git config --add remote.origin.fetch "+refs/pull/*/head:refs/remotes/origin/pr/*"
```

Với GitHub CLI: `gh pr checkout 12`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-github-gitlab-gitstudio/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio chạy git ls-remote origin refs/pull/* và git fetch origin pull/12/head:pr-12; trên đồ thị xuất hiện nhánh pr-12 ở commit Thử phí chờ của Lê Lan" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · lấy code của pull request số 12 về nhánh <code>pr-12</code></figcaption></figure>

## 5. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh / CLI |
|---|---|---|
| Đăng repository lên GitHub | Remote (chưa có remote) → Đăng lên GitHub… | `gh repo create --source . --push` |
| Thêm remote là repository GitHub có sẵn | Remote → `+` → nguồn GitHub | `git remote add origin <url>` |
| Xem pull request đang mở | Thanh bên → Pull request; Launchpad | `gh pr list` |
| Tạo pull request | Thanh công cụ → Pull Request; chuột phải nhánh | `gh pr create` |
| Lấy code của pull request | Chuột phải pull request → Checkout | `git fetch origin pull/12/head:pr-12`, `gh pr checkout 12` |
| Liên kết tới commit | Chuột phải commit → Sao chép link commit này trên remote | |

## 6. Lỗi thường gặp

- **Tạo repository trên GitHub có sẵn README** rồi push repository cục bộ: bị từ chối `(fetch first)` vì hai lịch sử không chung gốc. Nếu lỡ: `git pull --rebase origin main` (hoặc `--allow-unrelated-histories` khi merge), hoặc xoá repository rồi tạo lại trống.
- **GitStudio không hiện pull request.** Chưa kết nối tài khoản, token thiếu quyền đọc repository, hoặc `origin` không phải URL GitHub/GitLab (ví dụ máy chủ nội bộ).
- **Nghĩ pull request là một nhánh.** Pull request là một yêu cầu trên dịch vụ, *trỏ tới* một nhánh; xoá nhánh thì pull request đóng theo.
- **Đẩy code bí mật lên repository công khai.** "Riêng tư" phải chọn khi tạo; đổi sau thì những gì đã công khai có thể đã bị sao chép.

## 7. Bài tập

**Bài 1.** Viết lệnh lấy merge request `!7` của GitLab vào nhánh cục bộ `mr-7`.

<details><summary><b>Lời giải</b></summary><p><code>git fetch origin merge-requests/7/head:mr-7</code>.</p></details>

**Bài 2.** Sau khi thêm refspec `+refs/pull/*/head:refs/remotes/origin/pr/*`, `git branch -r` sẽ hiện thêm gì sau lần fetch kế tiếp?

<details><summary><b>Lời giải</b></summary><p>Mỗi pull request một nhánh remote-tracking: <code>origin/pr/12</code>, <code>origin/pr/13</code>… Dấu <code>+</code> cho phép cập nhật cả khi pull request bị force push.</p></details>

**Bài 3.** Một đồng nghiệp hỏi: "Repository trong lab không có pull request, vậy bài GIT-026 làm sao review?" Trả lời bằng những gì thuộc về Git ở bảng mục 1.

<details><summary><b>Lời giải</b></summary><p>Review là đọc khác biệt giữa nhánh tính năng và <code>main</code>: <code>git log main..origin/&lt;nhánh&gt;</code>, <code>git diff main...origin/&lt;nhánh&gt;</code>; merge là <code>git merge</code> (hoặc <code>--squash</code>). Pull request chỉ thêm nơi để bình luận, phê duyệt và luật tự động quanh các thao tác Git đó.</p></details>

## 8. Đọc tiếp

- [GitHub Docs: Checking out pull requests locally](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/checking-out-pull-requests-locally).
- [GitLab Docs: Checkout merge requests locally](https://docs.gitlab.com/ee/user/project/merge_requests/merge_request_troubleshooting.html#check-out-merge-requests-locally-through-the-head-ref).
- [Pro Git, The Refspec](https://git-scm.com/book/en/v2/Git-Internals-The-Refspec).

Bài tiếp theo: [GIT-026 · Pull request và code review](/articles/git-pull-request-code-review).
