---
slug: git-cai-dat-cau-hinh-gitstudio
title: "Cài Git và GitStudio, cấu hình danh tính commit"
summary: "Commit đầu tiên của Minh mang tên máy công ty và một email không ai nhận ra. Bài này cài Git và GitStudio, đặt user.name, user.email và nhánh khởi tạo main ở cấu hình toàn cục, dùng danh tính riêng cho một repository, và kiểm tra bằng git config --show-scope trong terminal tích hợp."
category: git
level: beginner
tags: [git, git-002, git-config, user-name, user-email, gitstudio, cai-dat, toi-noi]
language: vi
---

Commit đầu tiên Minh tạo trên máy mới mang tên tác giả `DESKTOP-7Q2K\minh` và email `minh@DESKTOP-7Q2K.localdomain`. Git không báo lỗi gì: khi chưa được cấu hình, Git tự đoán danh tính từ tên máy và tài khoản hệ điều hành. Ba tuần sau, khi nhóm cần biết ai đổi cấu hình giá, lịch sử ghi một cái tên không ai nhận ra, và GitHub không liên kết được commit đó với tài khoản nào.

Danh tính commit là thứ đầu tiên cần cấu hình, trước cả commit đầu tiên. Bài này cài Git và GitStudio, đặt danh tính ở hai cấp (toàn cục và riêng từng repository) bằng cả giao diện lẫn lệnh, và kiểm tra Git đang dùng giá trị nào.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-002 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-la-gi-va-gitstudio">GIT-001</a>. Sau bài này, bạn có Git và GitStudio chạy được, đặt được <code>user.name</code>, <code>user.email</code>, <code>init.defaultBranch</code> ở cấp toàn cục và cấp repository, và biết đọc <code>git config --show-scope</code> để xem giá trị nào đang thắng.</p>
</div>

## 1. Cài Git

GitStudio không tự mang Git theo: nó gọi lệnh `git` có trong `PATH`. Cài Git trước.

| Hệ điều hành | Cách cài | Ghi chú |
|---|---|---|
| Windows | [Git for Windows](https://git-scm.com/download/win), hoặc `winget install --id Git.Git -e` | Có kèm Git Bash và Git Credential Manager |
| macOS | `xcode-select --install` hoặc `brew install git` | Bản của Xcode thường cũ hơn bản Homebrew |
| Linux | `sudo apt install git` (Debian/Ubuntu), `sudo dnf install git` (Fedora) | |

Trình cài Git for Windows hỏi khá nhiều câu; giữ mặc định là được, trừ hai chỗ nên để ý:

- **Default editor**: chọn trình soạn thảo bạn dùng được (VS Code, Notepad++). Git mở trình soạn thảo này khi cần bạn viết nội dung commit mà không có `-m`. Mặc định là Vim, người mới hay bị "kẹt" trong đó.
- **Line endings**: mặc định là *Checkout Windows-style, commit Unix-style* (`core.autocrlf=true`). Series này không phụ thuộc lựa chọn ở đây: repository `bao-gia` tự quy định xuống dòng bằng `.gitattributes` (GIT-007), và đó là cách nên làm cho mọi repository làm việc nhóm.

Kiểm tra:

```sh
git --version
```

Máy viết bài: `git version 2.50.1.windows.1`. Series dùng `git switch`, `git restore` (có từ Git 2.23) và `init.defaultBranch` (Git 2.28), nên cần Git **2.28 trở lên**.

## 2. Cài và mở GitStudio

Tải bộ cài cho hệ điều hành của bạn ở trang [Releases của GitStudio](https://github.com/rickymta/git-studio/releases): `…_windows-x64-setup.exe` hoặc `.msi` cho Windows, `.dmg` cho macOS (Apple Silicon hoặc Intel), `.deb`, `.rpm`, `.AppImage` cho Linux. Bộ cài chưa ký số, nên Windows SmartScreen hoặc macOS Gatekeeper có thể hỏi xác nhận ở lần chạy đầu; mỗi bản phát hành có `SHA256SUMS.txt` để bạn kiểm tra file tải về.

Lần mở đầu tiên, GitStudio hỏi ngôn ngữ giao diện. Series dùng **Tiếng Việt**; đổi lại sau ở **Tuỳ chọn → Giao diện**. Nút mặt trăng ở góc trên bên phải đổi theme sáng/tối.

Mở **Tuỳ chọn** (biểu tượng bánh răng góc trên bên phải) → **Chung**, mục **Tệp thực thi Git** cho biết GitStudio đang dùng bản Git nào. Nếu mục này báo không tìm thấy Git, cài Git rồi mở lại GitStudio.

## 3. Danh tính commit: tên và email

Mỗi commit ghi hai danh tính, mỗi danh tính gồm tên và email:

- **Author** (người viết): người tạo ra thay đổi.
- **Committer** (người commit): người ghi thay đổi vào lịch sử. Thường trùng author; khác nhau khi một người áp patch hay sửa commit của người khác (GIT-008 cho thấy hai trường này trong object commit).

Git lấy cả hai từ `user.name` và `user.email`. Đây **không phải** thông tin đăng nhập: Git không kiểm tra bạn có đúng là chủ email đó hay không. Đăng nhập GitHub/GitLab là chuyện khác (Phần 3). Hệ quả: ai cũng có thể commit dưới tên người khác; muốn chứng minh commit là của mình thì phải ký commit (Phần 5).

### Ba cấp cấu hình

| Cấp | File | Áp dụng cho | Lệnh |
|---|---|---|---|
| `system` | `etc/gitconfig` trong thư mục cài Git | Mọi người dùng trên máy | `git config --system` |
| `global` | `~/.gitconfig` (Windows: `C:\Users\<tên>\.gitconfig`) | Mọi repository của bạn | `git config --global` |
| `local` | `.git/config` trong repository | Chỉ repository đó | `git config` (mặc định) |

Cấp hẹp hơn thắng cấp rộng hơn: `local` thắng `global`, `global` thắng `system`.

## 4. Đặt danh tính toàn cục trên giao diện

**Tuỳ chọn → Hồ sơ** hiện hai ô **Tên tác giả** (`user.name`) và **Email tác giả** (`user.email`) của cấu hình Git toàn cục, cùng ô **Ký commit** (`commit.gpgsign`, Phần 5). Sửa và rời ô là GitStudio ghi vào `~/.gitconfig`.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-cai-dat-cau-hinh-gitstudio/gitstudio-01.webp" alt="Trang Tuỳ chọn, mục Hồ sơ của GitStudio: Tên tác giả Trần Minh, Email tác giả minh@toinoi.test, ghi chú user.name và user.email trong cấu hình Git toàn cục, ô Ký commit chưa đánh dấu" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tuỳ chọn → Hồ sơ: danh tính trong cấu hình Git toàn cục</figcaption></figure>

Nếu bạn định đẩy code lên GitHub, dùng email mà tài khoản GitHub biết (hoặc địa chỉ `…@users.noreply.github.com` nếu không muốn lộ email thật), để GitHub gắn commit với tài khoản của bạn.

Khi chưa đặt tên/email, GitStudio chặn ngay ở ô commit với thông báo "Chưa đặt tên/email người dùng Git. Hãy đặt trong Tuỳ chọn → Hồ sơ trước khi commit.", không để Git tự đoán như trên máy của Minh.

## 5. Cũng việc đó bằng lệnh

Mở terminal tích hợp (`` Ctrl+` ``, hoặc `Ctrl+K` rồi chọn thao tác **Terminal**) và gõ:

```sh
git config --global user.name "Trần Minh"
git config --global user.email minh@toinoi.test
git config --global init.defaultBranch main
git config --global --get user.name
git config --global --list
```

Kết quả của lab (cấu hình "toàn cục" của lab nằm trong `work/home/.gitconfig` để không đụng tới máy bạn):

```text
Trần Minh
user.name=Trần Minh
user.email=minh@toinoi.test
init.defaultbranch=main
```

`init.defaultBranch main` đặt tên nhánh đầu tiên khi tạo repository mới. Không đặt, Git 2.50 vẫn dùng `master` và in một đoạn gợi ý dài mỗi lần `git init`. GitStudio cũng có ô riêng cho việc này ở **Tuỳ chọn → Chung → Tên nhánh mặc định**, và hỏi lại tên nhánh khi bạn tạo repository (GIT-003).

Tên khoá in ra viết thường (`init.defaultbranch`): tên khoá trong cấu hình Git không phân biệt hoa thường, `git config --list` luôn in chữ thường.

## 6. Danh tính riêng cho một repository

Minh có một repository cá nhân và muốn commit ở đó bằng email riêng, không bằng email công ty. Đặt `user.email` ở cấp `local`:

```sh
git init -q du-an-ca-nhan
cd du-an-ca-nhan
git config user.email minh.ca-nhan@example.com
git config --show-scope --get user.email
git config --show-scope --get user.name
git config --show-origin --get user.email
```

```text
local	minh.ca-nhan@example.com
global	Trần Minh
file:.git/config	minh.ca-nhan@example.com
```

`--show-scope` cho biết giá trị lấy từ cấp nào; `--show-origin` cho biết từ file nào. Email lấy từ `local`, tên vẫn lấy từ `global` vì repository này không đặt tên riêng. Commit thử để chắc chắn:

```sh
git add ghi-chu.md
git commit -q -m "Ghi chú đầu tiên"
git log -1 --format="%an <%ae>"
```

```text
Trần Minh <minh.ca-nhan@example.com>
```

Trên giao diện: **Tuỳ chọn → Tuỳ chọn riêng cho repository → Repo: <tên>**. Bật **Dùng danh tính khác cho repository này** rồi điền **Tên tác giả**, **Email tác giả**. Dòng ghi chú bên dưới nhắc danh tính toàn cục để bạn so. Cùng trang này có credential helper, ký tự xuống dòng (`core.autocrlf`) và ký commit cho riêng repository; mọi thứ được lưu trong `.git/config` của repository đó.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-cai-dat-cau-hinh-gitstudio/gitstudio-03.webp" alt="Trang Tuỳ chọn repository của bao-gia: Danh tính commit có ô Dùng danh tính khác cho repository này đã bật, Danh tính toàn cục Trần Minh minh@toinoi.test, Tên tác giả và Email tác giả của repository, Credential helper, Ký tự xuống dòng và Ký commit theo cấu hình toàn cục" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tuỳ chọn repository: thiết lập lưu trong <code>.git/config</code></figcaption></figure>

Repository `bao-gia` của lab đặt sẵn danh tính `local` là Trần Minh, nên trong terminal tích hợp bạn thấy cả hai cấp:

```sh
git config --show-scope --get-regexp "^user\."
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-cai-dat-cau-hinh-gitstudio/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio chạy git config --show-scope --get-regexp ^user, in global user.name Trần Minh, global user.email minh@toinoi.test, local user.name Trần Minh, local user.email minh@toinoi.test" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · <code>git config --show-scope</code>: cùng khoá ở hai cấp, cấp <code>local</code> thắng</figcaption></figure>

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Xem bản Git đang dùng | Tuỳ chọn → Chung → Tệp thực thi Git | `git --version` |
| Danh tính toàn cục | Tuỳ chọn → Hồ sơ | `git config --global user.name/user.email` |
| Danh tính riêng repository | Tuỳ chọn → Repo: <tên> → Danh tính commit | `git config user.name/user.email` |
| Nhánh khởi tạo | Tuỳ chọn → Chung → Tên nhánh mặc định | `git config --global init.defaultBranch main` |
| Xem giá trị đang thắng | Ghi chú dưới ô | `git config --show-scope --get <khoá>` |
| Mở terminal | Nút Terminal, `` Ctrl+` ``, Spotlight → Terminal | |

## 8. Lỗi thường gặp

- **`Author identity unknown … Please tell me who you are`.** Chưa có `user.email`/`user.name` ở cấp nào và Git không đoán được. Đặt ở cấp toàn cục như mục 4–5.
- **Commit mang email sai, đã lỡ commit vài lần.** Sửa cấu hình chỉ có tác dụng với commit *sau đó*. Commit cuối cùng sửa được bằng `git commit --amend --reset-author --no-edit`; nhiều commit cũ cần viết lại lịch sử (Phần 4), và chỉ nên làm khi chưa push.
- **Đặt `git config user.email …` ở ngoài mọi repository.** Lệnh báo `fatal: not in a git directory`: không có `--global` thì Git hiểu là cấp `local`, mà ở ngoài repository thì không có `.git/config`.
- **Gõ tên có dấu trong PowerShell bị thành `Tr?n Minh`.** Do bảng mã của cửa sổ console cũ. Terminal tích hợp của GitStudio và Windows Terminal dùng UTF-8; nếu vẫn lỗi, đặt danh tính qua **Tuỳ chọn → Hồ sơ**.
- **Dùng email cá nhân trên máy công ty (hoặc ngược lại).** Đặt email công ty ở `global`, email cá nhân ở `local` của từng repository cá nhân, như mục 6.

## 9. Bài tập

**Bài 1.** Chạy `git config --show-origin --get user.name` trong repository `du-an-ca-nhan` của lab. File nào được in ra, và vì sao không phải `.git/config`?

<details><summary><b>Lời giải</b></summary><p>In ra file cấu hình toàn cục (trong lab là <code>work/home/.gitconfig</code>, trên máy bạn là <code>~/.gitconfig</code>). Repository này chỉ đặt <code>user.email</code> ở cấp <code>local</code>, còn <code>user.name</code> vẫn lấy từ cấp <code>global</code>.</p></details>

**Bài 2.** Lan muốn mọi repository trong thư mục `D:\toinoi\` dùng email công ty, còn các repository khác dùng email cá nhân, mà không phải cấu hình từng repository. Tìm tính năng `includeIf` trong tài liệu `git config` và viết đoạn cấu hình.

<details><summary><b>Lời giải</b></summary><p>Trong <code>~/.gitconfig</code>: đặt <code>[user] email = lan@example.com</code>, rồi thêm <code>[includeIf "gitdir/i:D:/toinoi/"]</code> với dòng <code>path = ~/.gitconfig-toinoi</code>. File <code>~/.gitconfig-toinoi</code> chứa <code>[user] email = lan@toinoi.test</code>. Git chỉ nạp file này khi repository nằm dưới <code>D:/toinoi/</code>; <code>gitdir/i</code> so không phân biệt hoa thường, hợp với Windows.</p></details>

**Bài 3.** Vì sao "đặt `user.email` là email của sếp" không làm bạn có quyền push vào repository của sếp trên GitHub?

<details><summary><b>Lời giải</b></summary><p><code>user.email</code> chỉ là chữ được ghi vào commit, không phải thông tin đăng nhập. Quyền push do GitHub kiểm tra qua token hoặc SSH key của bạn (Phần 3). Ngược lại, commit mang email của người khác vẫn được tạo và đẩy lên được nếu bạn có quyền push; vì vậy mới cần ký commit để chứng minh danh tính (Phần 5).</p></details>

## 10. Đọc tiếp

- [git-config](https://git-scm.com/docs/git-config): các cấp cấu hình, `--show-scope`, `--show-origin`, `includeIf`.
- [Pro Git, First-Time Git Setup](https://git-scm.com/book/en/v2/Getting-Started-First-Time-Git-Setup).
- [GitHub: Setting your commit email address](https://docs.github.com/en/account-and-profile/setting-up-and-managing-your-personal-account-on-github/managing-email-preferences/setting-your-commit-email-address).

Bài tiếp theo: [GIT-003 · Tạo repository: init, clone và mở trong GitStudio](/articles/git-init-clone-mo-repository).
