---
slug: git-workspace-launchpad
title: "Workspace và Launchpad: làm việc với nhiều repository cùng lúc"
summary: "Tới Nơi giờ có ba repository Minh phải theo dõi: báo giá, app tài xế và thư viện tiền tệ. Bài này cho thấy làm bằng tay trên nhiều repository tốn công thế nào (status, fetch, pull từng cái), rồi dùng Workspace của GitStudio: gom repository, xem trạng thái trên một trang, Fetch tất cả, Pull tất cả chỉ fast-forward, lọc Chưa commit/Cần pull/Chưa push. Launchpad gom việc dở (WIP), nhánh lệch upstream và pull request của bạn; bài nói rõ phần nào chạy cục bộ, phần nào gọi GitHub."
category: git
level: beginner
tags: [git, git-053, workspace, launchpad, multi-repo, gitstudio, toi-noi]
language: vi
---

Sáng 02/12/2026, Minh có ba repository trên máy. `bao-gia` là dịch vụ báo giá. `app-tai-xe` là app Flutter mà Lan và Minh vừa khởi tạo; hôm qua Minh sửa dở màn hình nhận chuyến. `tien-te` là bản clone thư viện tiền tệ để theo dõi, và Lan vừa push thêm một commit. Mở từng thư mục, chạy `git status`, `git fetch`, rồi nhớ cái nào chưa push: với ba repository đã mệt, với mười repository của một nhóm lớn thì chắc chắn bỏ sót. Bài này làm bằng lệnh trước, rồi giao cho Workspace và Launchpad của GitStudio.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-053 · Phần 6/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-fetch-pull-push">GIT-022</a> (fetch, pull), <a href="/articles/git-ahead-behind-push-bi-tu-choi">GIT-023</a> (ahead/behind). Lab: <code>npm run p06 -- 53</code>; dựng ba repository để mở bằng GitStudio: <code>npm run setup -- 53</code> (thư mục <code>work/</code> có <code>bao-gia</code>, <code>app-tai-xe</code>, <code>tien-te</code>). Sau bài này, bạn theo dõi và đồng bộ được nhiều repository trên một trang.</p>
</div>

## 1. Bằng lệnh: từng repository một

```sh
cd ..
Get-ChildItem -Force -Name .
git -C bao-gia status -sb
git -C app-tai-xe status -sb
git -C tien-te status -sb
```

```text
app-tai-xe
bao-gia
lan
remote
thu-vien
tien-te
## main...origin/main
## main...origin/main
 M lib/main.dart
## main...origin/main [behind 1]
```

`git -C <thư mục>` chạy git như thể đang đứng trong thư mục đó, nên một terminal là đủ cho cả ba. `status -sb` gom nhánh, upstream, ahead/behind (GIT-023) và thay đổi vào vài dòng. Đọc kết quả: `bao-gia` sạch và khớp remote; `app-tai-xe` có việc dở; `tien-te` chậm hơn remote 1 commit, tức là thông tin `[behind 1]` từ lần fetch trước.

Đồng bộ thì fetch từng repository, rồi chỉ pull những repository đang chậm, và chỉ khi pull không tạo merge commit:

```sh
git -C bao-gia fetch -q --tags --prune --all
git -C app-tai-xe fetch -q --tags --prune --all
git -C tien-te fetch -q --tags --prune --all
git -C tien-te rev-list --left-right --count main...origin/main
git -C tien-te pull --ff-only
git -C tien-te log --oneline -3
```

```text
0	1
Updating 02ae86b..31d0d96
Fast-forward
 README.md | 3 +++
 1 file changed, 3 insertions(+)
 create mode 100644 README.md
31d0d96 docs: cách dùng Vnd.Format và Vnd.Short
02ae86b Thêm dạng ngắn 31k
23f146f Định dạng tiền Việt Nam
```

`rev-list --left-right --count main...origin/main` in hai số: số commit chỉ có ở `main`, rồi số commit chỉ có ở `origin/main`. Ở đây `0 1` nghĩa là không có gì chưa push và chậm 1, nên fast-forward an toàn. `pull --ff-only` từ chối nếu hai bên đã rẽ nhánh. Đó là lựa chọn đúng khi đồng bộ hàng loạt: không tự tạo merge commit hay rebase ở repository bạn không nhìn tới.

Việc dở trong `app-tai-xe` là thứ dễ quên nhất:

```sh
git -C app-tai-xe diff --stat
```

```text
 lib/main.dart | 2 ++
 1 file changed, 2 insertions(+)
```

Một vòng lặp PowerShell (`Get-ChildItem -Directory | ForEach-Object { git -C $_ status -sb }`) làm được phần đọc, nhưng bạn vẫn phải tự đọc từng dòng để biết cái nào cần xử lý. Workspace làm phần đọc đó thay bạn.

## 2. Workspace của GitStudio

Workspace là một nhóm repository bạn hay làm cùng nhau. Mỗi workspace có **bộ tab riêng** và một **trang tổng quan** hiện trạng thái mọi repository. Tạo workspace từ trang chào (mục **Workspace** → **Mới**) hoặc từ nút workspace trên thanh tab (**Workspace mới…**). Hộp thoại **Workspace mới** gồm **Tên**, **Màu** và cách lấy repository: **Quét thư mục** (tìm repository sâu tới 3 cấp, bỏ qua `node_modules`, `bin`, `obj`…), **Các tab đang mở**, hoặc **Bắt đầu trống**. Minh đặt tên "Tới Nơi", quét thư mục `work/` và bỏ chọn hai repository của lab (`lan/bao-gia`, `thu-vien/tien-te`), vì đó là bản của Lan.

Trang tổng quan có các cột **Repository**, **Nhánh**, **Đồng bộ**, **Thay đổi**, **Hoạt động gần nhất**, **PR**. Bộ lọc nhanh gồm **Tất cả**, **Chưa commit**, **Cần pull**, **Chưa push**, **Cần xử lý**; mục cuối gom các repository có lỗi, có xung đột hoặc đang merge, rebase dở. Hai nút ở đầu trang làm đúng việc của mục 1:

- **Fetch tất cả**: `git fetch --tags --prune --all` cho mọi repository trong danh sách, tối đa 4 repository chạy song song.
- **Pull tất cả**: chỉ fast-forward những repository đang chậm hơn upstream (`git pull --ff-only`), **không bao giờ tạo merge commit**. Repository đã rẽ nhánh được để nguyên cho bạn xử lý.

Kết quả báo bằng một thông báo kiểu "Fetch tất cả: xong 3 repository", hoặc kèm số repository lỗi. Menu chuột phải của mỗi dòng có **Mở**, **Fetch**, **Pull (fast-forward)**, **Mở terminal tại đây**, **Hiện trong trình quản lý tệp**, **Gỡ khỏi workspace**. Workspace có thể chia thư mục con, với **Fetch cả thư mục** và **Pull cả thư mục (fast-forward)**. Danh sách workspace lưu trong `%APPDATA%\GitStudio\workspaces.json`. Xoá workspace không đụng tới repository trên ổ đĩa.

Cột **PR** và khung **Pull request** chỉ xuất hiện khi repository có remote trên github.com. Repository của lab dùng remote `file://` nên phần đó trống.

## 3. Launchpad: việc của bạn ở mọi repository

**Launchpad** (nút trên thanh tab, `Ctrl+Shift+K`) gom "những gì đang chờ bạn". Các tab của nó lấy dữ liệu từ hai nguồn khác nhau:

| Tab | Lấy từ đâu | Cho thấy |
|---|---|---|
| **WIP** | `git status` của các repository trong phạm vi, chạy trên máy | Repository có thay đổi chưa commit, số file sửa/thêm/xoá, xung đột |
| **Nhánh** | Nhánh đang checkout so với upstream, chạy trên máy | Repository chậm (`chậm N`), chưa push (`N chưa push`), chưa có upstream; nút **Pull** chỉ fast-forward |
| **Pull request của tôi**, **Nhờ tôi review**, **Issue của tôi** | API của GitHub (github.com) và GitLab (gitlab.com) bằng tài khoản đã kết nối | Pull request và issue bạn tạo, được giao, được nhờ review, được nhắc tới, trên **mọi** repository của tài khoản, kèm trạng thái kiểm tra CI, xung đột, phê duyệt |

Ô **phạm vi** ("Các repository mà WIP và Nhánh quét") chọn **Workspace "Tới Nơi"** hay **Mọi workspace**. Ô này chỉ ảnh hưởng tới tab WIP và Nhánh; tab pull request và issue luôn theo tài khoản, không lọc theo workspace. Khi chưa kết nối GitHub hay GitLab, các tab đó hiện lời mời kết nối; WIP và Nhánh vẫn chạy bình thường với repository chỉ có remote nội bộ như của lab. Phím tắt trong Launchpad: `J`/`K` di chuyển, `Enter` mở, `P` ghim, `S` tạm ẩn một ngày, `/` tìm, `R` làm mới.

Thứ tự Minh dùng mỗi sáng: **Fetch tất cả** trên workspace, xem bộ lọc **Cần pull** rồi **Pull tất cả**, rồi Launchpad tab **WIP** để nhớ việc dở hôm qua.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-workspace-launchpad/gitstudio-01.webp" alt="Trang tổng quan workspace Tới Nơi trong GitStudio với ba repository bao-gia, app-tai-xe, tien-te; cột Đồng bộ cho thấy tien-te chậm 1 commit, cột Thay đổi cho thấy app-tai-xe có file chưa commit; nút Fetch tất cả và Pull tất cả ở đầu trang" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Workspace: ba repository trên một trang</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-workspace-launchpad/gitstudio-02.webp" alt="Launchpad của GitStudio ở tab WIP với phạm vi Workspace Tới Nơi: dòng WIP trên app-tai-xe có một file sửa, nút Xem repo" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Launchpad, tab WIP: việc dở ở mọi repository</figcaption></figure>

## 4. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Xem trạng thái nhiều repository | Workspace → trang tổng quan | `git -C <repo> status -sb` từng repository |
| Fetch mọi repository | Fetch tất cả (`Ctrl+L` trong một repository) | `git -C <repo> fetch --tags --prune --all` |
| Pull những repository đang chậm | Pull tất cả (chỉ fast-forward) | `git -C <repo> pull --ff-only` |
| Chậm/nhanh bao nhiêu commit | Cột Đồng bộ; Launchpad → Nhánh | `git rev-list --left-right --count main...origin/main` |
| Việc dở chưa commit | Bộ lọc Chưa commit; Launchpad → WIP | `git -C <repo> status --short` |
| Chuyển nhanh giữa repository | `Ctrl+P` (tab đang mở và workspace) | `cd` |

## 5. Lỗi thường gặp

- **Pull hàng loạt bằng `git pull` thường**: repository nào đã rẽ nhánh sẽ có merge commit (hoặc rebase, tuỳ `pull.rebase`) mà bạn không nhìn thấy. Luôn dùng `--ff-only` khi đồng bộ hàng loạt; Pull tất cả của GitStudio đã làm vậy.
- **Fetch xong tưởng là đã cập nhật**: fetch chỉ cập nhật `origin/main`, working tree vẫn là bản cũ (GIT-022). Bộ lọc **Cần pull** liệt kê đúng những repository còn chậm.
- **Workspace quét cả bản clone thử, bản của người khác**: bỏ chọn khi tạo, hoặc **Gỡ khỏi workspace** sau.
- **Nghĩ Launchpad chỉ hiện repository trong workspace**: chỉ WIP và Nhánh theo phạm vi; tab pull request hiện mọi pull request của tài khoản GitHub.
- **Để việc dở ở nhiều repository qua đêm**: stash có tên hoặc commit vào nhánh riêng (GIT-015) trước khi chuyển sang việc khác.

## 6. Bài tập

**Bài 1.** Trong `app-tai-xe`, Minh commit việc dở nhưng chưa push. Workspace và Launchpad sẽ hiện điều đó ở đâu?

<details><summary><b>Lời giải</b></summary><p>Trang tổng quan: cột Đồng bộ ghi 1 commit chưa push, repository nằm trong bộ lọc <strong>Chưa push</strong>. Launchpad: tab <strong>Nhánh</strong> ghi <code>1 chưa push</code>. Tab WIP không còn hiện vì working tree đã sạch.</p></details>

**Bài 2.** Viết một dòng PowerShell in nhánh và trạng thái đồng bộ của mọi repository trong thư mục hiện tại.

<details><summary><b>Lời giải</b></summary><p><code>Get-ChildItem -Directory | Where-Object { Test-Path "$_/.git" } | ForEach-Object { "== $($_.Name)"; git -C $_.FullName status -sb }</code>. Muốn fetch trước thì thêm <code>git -C $_.FullName fetch -q</code> vào trong vòng lặp.</p></details>

**Bài 3.** Vì sao "Pull tất cả" bỏ qua một repository vừa chậm 2 commit vừa có 1 commit chưa push?

<details><summary><b>Lời giải</b></summary><p>Hai bên đã rẽ nhánh, nên không fast-forward được. Phải chọn merge hay rebase (GIT-029), và đó là quyết định của người làm repository đó, không nên tự động hoá.</p></details>

## 7. Đọc tiếp

- [git -C và cấu hình](https://git-scm.com/docs/git#Documentation/git.txt--Cltpathgt), [git-rev-list --left-right --count](https://git-scm.com/docs/git-rev-list).

Bài tiếp theo: [GIT-054 · Git trong CI: clone nông, tag, tính phiên bản và quyền của token](/articles/git-trong-ci).
