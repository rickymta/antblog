---
slug: git-init-clone-mo-repository
title: "Tạo repository: init, clone và mở trong GitStudio"
summary: "Thư mục dịch vụ báo giá trên máy Minh chưa phải repository. Bài này tạo repository bằng git init với nhánh khởi tạo main, nhìn vào thư mục .git, tạo commit đầu tiên, clone sang thư mục khác để thấy remote origin, và làm cùng các bước đó bằng nút Tạo, Mở, Clone của GitStudio."
category: git
level: beginner
tags: [git, git-003, git-init, git-clone, repository, origin, gitstudio, toi-noi]
language: vi
---

Thư mục `bao-gia` trên máy Minh có `README.md` và vài file C#, nhưng Git chưa biết gì về nó. Gõ `git status` trong thư mục đó, Git trả lời `fatal: not a git repository (or any of the parent directories): .git`. Một thư mục chỉ trở thành repository khi có thư mục `.git` bên trong, và có hai cách để có `.git`: **tạo mới** (`git init`) hoặc **sao chép một repository có sẵn** (`git clone`).

Bài này làm cả hai, trên giao diện GitStudio và bằng lệnh, rồi nhìn vào `.git` để biết lịch sử nằm ở đâu.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-003 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-cai-dat-cau-hinh-gitstudio">GIT-002</a> (đã đặt danh tính). Lab: <code>npm run p01 -- 03</code>. Sau bài này, bạn tạo được repository với nhánh khởi tạo <code>main</code>, biết thư mục <code>.git</code> chứa gì, tạo được commit đầu tiên, clone một repository và đọc được remote <code>origin</code>.</p>
</div>

## 1. Repository = working tree + `.git`

| Phần | Là gì | Có bị commit không |
|---|---|---|
| **Working tree** | Các file bạn thấy và sửa trong thư mục dự án | Chỉ những file bạn chọn |
| **`.git`** | Lịch sử (object), nhánh và tag (ref), staging area (index), cấu hình riêng | Không, đây chính là "kho" |

Xoá thư mục `.git` là xoá toàn bộ lịch sử; working tree vẫn còn nhưng chỉ là thư mục thường. Ngược lại, chép thư mục `.git` sang chỗ khác là chép theo cả lịch sử. Đừng bao giờ sửa tay file trong `.git` khi chưa hiểu nó (GIT-008 sẽ mở ra xem, chỉ đọc).

## 2. Tạo repository bằng lệnh

```sh
git init -b main bao-gia
cd bao-gia
```

```text
Initialized empty Git repository in <lab>/work/bao-gia/.git/
```

`<lab>` là chỗ lab thay cho đường dẫn tuyệt đối trên máy bạn. `-b main` đặt tên nhánh khởi tạo; đã đặt `init.defaultBranch main` ở GIT-002 thì `git init bao-gia` cũng ra nhánh `main`. Không có tên thư mục, `git init` biến chính thư mục hiện tại thành repository: đó là cách dùng khi thư mục dự án đã có file từ trước.

Xem `.git` vừa được tạo (trên PowerShell; Git Bash dùng `ls -A .git`):

```powershell
Get-ChildItem -Force -Name .git
```

```text
config
description
HEAD
hooks
info
objects
refs
```

| Mục | Vai trò |
|---|---|
| `HEAD` | Đang đứng ở nhánh nào (lúc này: `ref: refs/heads/main`) |
| `config` | Cấu hình cấp `local` của repository (GIT-002) |
| `objects/` | Nơi lưu mọi commit, cây thư mục và nội dung file (GIT-008) |
| `refs/` | Nhánh (`refs/heads/`) và tag (`refs/tags/`) |
| `hooks/` | Script mẫu chạy trước/sau các thao tác git (Phần 5) |
| `info/`, `description` | Luật bỏ qua riêng máy (`info/exclude`), mô tả dùng cho máy chủ web của Git |

Repository mới chưa có commit nào:

```sh
git status
```

```text
On branch main

No commits yet

nothing to commit (create/copy files and use "git add" to track)
```

Lab đặt danh tính riêng cho repository này để commit trong bài luôn mang tên Trần Minh:

```sh
git config user.name "Trần Minh"
git config user.email minh@toinoi.test
```

## 3. Commit đầu tiên

Tạo `README.md`:

```markdown
# bao-gia

Dịch vụ báo giá cước của Tới Nơi: nhận quãng đường và loại dịch vụ,
trả về giá dự kiến cho khách trước khi đặt chuyến.
```

```sh
git status --short
git add README.md
git commit -m "Khởi tạo dịch vụ báo giá"
git log --oneline
```

```text
?? README.md
[main (root-commit) cd8da2e] Khởi tạo dịch vụ báo giá
 1 file changed, 4 insertions(+)
 create mode 100644 README.md
cd8da2e Khởi tạo dịch vụ báo giá
```

- `??` nghĩa là file chưa được theo dõi (untracked). GIT-004 giải thích hết các ký hiệu của `git status --short`.
- `git add` đưa file vào **staging area**: danh sách những gì sẽ vào commit tiếp theo.
- `(root-commit)` đánh dấu commit đầu tiên, không có commit cha.
- `create mode 100644`: file mới, quyền file thường (không thực thi được).

Mã hash `cd8da2e` là của lab: lab cố định thời điểm commit là 08:30 ngày 02/11/2026 (+07:00). Commit bạn tự tạo sẽ có mã hash khác dù nội dung giống hệt, vì thời điểm khác.

## 4. Cũng việc đó trên GitStudio

Ở **Tab mới** (`Ctrl+T`, hoặc dấu `+` trên thanh tab), mục **Repository** có ba nút **Mở**, **Clone**, **Tạo**.

1. Bấm **Tạo**, chọn thư mục cho repository mới. GitStudio hỏi **Nhánh khởi tạo**, điền sẵn `main` (lấy từ **Tuỳ chọn → Chung → Tên nhánh mặc định**). Bấm **Tạo**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-init-clone-mo-repository/gitstudio-01.webp" alt="Hộp thoại Tạo repository của GitStudio với đường dẫn thư mục bao-gia-moi, ô Nhánh khởi tạo điền main, nút Huỷ và Tạo; phía sau là trang Tab mới với các nút Mở, Clone, Tạo" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tạo repository: chọn nhánh khởi tạo trước khi chạy <code>git init</code></figcaption></figure>

2. Repository mới mở ra trong một tab, đồ thị báo "Chưa có commit nào". Tạo `README.md` trong thư mục (bằng trình soạn thảo của bạn); GitStudio tự phát hiện thay đổi, hoặc bấm `F5`. Trên đồ thị xuất hiện hàng **// WIP** (work in progress: thay đổi chưa commit), bảng bên phải liệt kê `README.md` ở **Tệp chưa stage**.
3. Bấm **Stage tất cả thay đổi**: file chuyển xuống **Tệp đã stage**. Gõ `Khởi tạo dịch vụ báo giá` vào ô **Tóm tắt commit**; con số bên phải ô là số ký tự còn lại trước mốc khuyến nghị 72 ký tự. Nút dưới cùng đổi thành **Commit thay đổi của 1 tệp**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-init-clone-mo-repository/gitstudio-02.webp" alt="Repository bao-gia-moi trong GitStudio: hàng WIP trên đồ thị, Tệp đã stage có README.md, ô Tóm tắt commit ghi Khởi tạo dịch vụ báo giá với bộ đếm 48, nút Commit thay đổi của 1 tệp" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · README.md đã stage, sẵn sàng cho commit đầu tiên</figcaption></figure>

4. Bấm nút commit (hoặc `Ctrl+Enter` khi đang ở ô nội dung). Đồ thị có một commit, nhãn `main` gắn vào đó.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-init-clone-mo-repository/gitstudio-03.webp" alt="Đồ thị GitStudio của bao-gia-moi sau commit đầu tiên: một hàng Khởi tạo dịch vụ báo giá với nhãn nhánh main và chấm TM; bảng bên phải không còn thay đổi" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · commit đầu tiên trên nhánh <code>main</code></figcaption></figure>

Bên dưới, GitStudio chạy đúng các lệnh `git init -b main`, `git add` và `git commit` như mục 2–3. Mở terminal tích hợp (`` Ctrl+` ``) và gõ `git log --oneline` để thấy commit vừa tạo; mã hash sẽ khác `cd8da2e` vì thời điểm khác.

**Mở** một thư mục đã là repository (có `.git`) thì dùng nút **Mở**, hoặc chuột phải thư mục trong Explorer/Finder → **Open with GitStudio**. Thư mục chưa có `.git` (dự án có sẵn file nhưng chưa dùng Git) thì dùng **Tạo** và chọn chính thư mục đó: giống `git init` không tham số, file có sẵn hiện ra ở **Tệp chưa stage**.

## 5. Clone: sao chép cả repository

`git clone` tạo một repository mới từ một repository có sẵn, gồm **toàn bộ lịch sử**, không chỉ file mới nhất. Nguồn thường là một URL (`https://github.com/...`, `git@github.com:...`), nhưng một đường dẫn trên máy cũng được. Lab clone repository vừa tạo sang thư mục cạnh bên:

```sh
cd ..
git clone bao-gia bao-gia-ban-sao
cd bao-gia-ban-sao
git log --oneline
git remote -v
git status
```

```text
Cloning into 'bao-gia-ban-sao'...
done.
cd8da2e Khởi tạo dịch vụ báo giá
origin	<lab>/work/bao-gia (fetch)
origin	<lab>/work/bao-gia (push)
On branch main
Your branch is up to date with 'origin/main'.

nothing to commit, working tree clean
```

Clone làm ba việc:

1. Tạo thư mục và `.git` mới, chép mọi commit sang (cùng mã hash `cd8da2e`: cùng nội dung là cùng mã hash).
2. Ghi nhớ nguồn dưới tên **`origin`**: một **remote**, tức một repository khác mà repository này biết địa chỉ để fetch/push.
3. Tạo nhánh `main` cục bộ **theo dõi** `origin/main`. Dòng `Your branch is up to date with 'origin/main'` so nhánh của bạn với bản sao gần nhất của `origin/main`. Phần 3 dành trọn cho remote.

### Clone trên GitStudio

Ở **Tab mới**, bấm **Clone**. Hộp thoại có ba nguồn: **Clone bằng URL**, **GitHub.com**, **GitLab.com** (hai mục sau cần đăng nhập, Phần 3). Với **Clone bằng URL**: chọn **Clone vào thư mục** (thư mục cha), dán **URL** (ở đây là đường dẫn của repository `bao-gia`), điền tên thư mục, rồi bấm **Clone repository!**. Hai tuỳ chọn **Shallow clone** và **Sparse checkout** dành cho repository lớn (Phần 5); để trống.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-init-clone-mo-repository/gitstudio-04.webp" alt="Hộp thoại Clone một repository của GitStudio, mục Clone bằng URL: Clone vào thư mục là work của lab, URL là đường dẫn repository bao-gia, tên thư mục bao-gia-ban-sao, hai tuỳ chọn Shallow clone và Sparse checkout chưa chọn" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Clone bằng URL; nguồn có thể là một đường dẫn trên máy</figcaption></figure>

Repository vừa clone mở trong tab mới; thanh bên trái có mục **Remote** với `origin`.

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Spotlight / phím | Lệnh |
|---|---|---|---|
| Tạo repository | Tab mới → Tạo → Nhánh khởi tạo | `Ctrl+T` mở tab mới | `git init -b main <thư mục>` |
| Mở repository có sẵn | Tab mới → Mở; Explorer → Open with GitStudio | `Ctrl+P` chuyển giữa repository đang mở | `cd <thư mục>` |
| Commit đầu tiên | Stage tất cả thay đổi → Tóm tắt commit → Commit | `Ctrl+Enter` trong ô nội dung | `git add`, `git commit -m` |
| Clone | Tab mới → Clone → Clone bằng URL | | `git clone <url> <thư mục>` |
| Xem remote | Thanh bên trái → Remote | | `git remote -v` |

## 7. Lỗi thường gặp

- **`git init` trong thư mục cha nhầm (ví dụ cả ổ `D:\` hay thư mục người dùng).** Mọi thư mục con bỗng thành một phần của repository khổng lồ, GitStudio và `git status` chậm hẳn. Xoá thư mục `.git` vừa tạo nhầm (chỉ khi chắc chắn nó mới tạo, chưa có lịch sử gì đáng giữ) rồi `git init` đúng chỗ.
- **Tạo repository lồng trong repository khác.** `git status` ở repository ngoài báo thư mục con là untracked, không thấy file bên trong. Repository lồng nhau có chủ đích là submodule (Phần 5); còn lại thì đừng lồng.
- **`fatal: destination path 'bao-gia-ban-sao' already exists and is not an empty directory.`** Clone không ghi đè thư mục có sẵn; chọn tên khác hoặc xoá thư mục cũ.
- **Clone qua HTTPS hỏi mật khẩu rồi báo `Authentication failed`.** GitHub và GitLab không nhận mật khẩu tài khoản cho Git qua HTTPS; cần token hoặc SSH key (Phần 3).
- **Commit đầu tiên chứa cả `bin/`, `obj/`.** Chưa có `.gitignore` mà lại bấm "Stage tất cả". Bài sau xử lý chuyện này trước khi nó xảy ra.

## 8. Bài tập

**Bài 1.** Trong bản clone `bao-gia-ban-sao`, chạy `git log --oneline origin/main`. Kết quả so với `git log --oneline` thế nào, và `origin/main` là gì?

<details><summary><b>Lời giải</b></summary><p>Giống hệt: một commit <code>cd8da2e</code>. <code>origin/main</code> là nhánh <b>remote-tracking</b>, bản ghi nhớ trên máy về vị trí nhánh <code>main</code> của <code>origin</code> ở lần fetch/clone gần nhất. Nó không tự cập nhật khi repository nguồn có commit mới; phải fetch (Phần 3).</p></details>

**Bài 2.** Tạo thêm một commit trong repository gốc `bao-gia`, rồi chạy `git status` trong `bao-gia-ban-sao`. Vì sao vẫn báo `up to date`?

<details><summary><b>Lời giải</b></summary><p><code>git status</code> chỉ so nhánh cục bộ với <code>origin/main</code> đang ghi nhớ trên máy, không liên lạc với nguồn. Chạy <code>git fetch</code> trước thì <code>git status</code> mới báo <code>Your branch is behind 'origin/main' by 1 commit</code>.</p></details>

**Bài 3.** Một đồng nghiệp gửi bạn file zip của repository, trong đó có thư mục `.git`. Giải nén ra và mở bằng GitStudio thì được gì so với `git clone`?

<details><summary><b>Lời giải</b></summary><p>Có đủ lịch sử và nhánh như trên máy đồng nghiệp, kể cả cấu hình <code>local</code>, remote của họ, các nhánh dở dang và stash; có khi còn mang theo file bí mật chưa commit. Clone chỉ lấy lịch sử đã commit, đặt <code>origin</code> trỏ về nguồn và không mang cấu hình riêng của người khác. Với repository dùng chung, clone là cách đúng.</p></details>

## 9. Đọc tiếp

- [git-init](https://git-scm.com/docs/git-init), [git-clone](https://git-scm.com/docs/git-clone).
- [Pro Git, Getting a Git Repository](https://git-scm.com/book/en/v2/Git-Basics-Getting-a-Git-Repository).
- [gitrepository-layout](https://git-scm.com/docs/gitrepository-layout): ý nghĩa từng mục trong `.git`.

Bài tiếp theo: [GIT-004 · Working tree, staging area và repository; file bị bỏ qua](/articles/git-ba-vung-working-tree-staging-gitignore).
