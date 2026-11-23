---
slug: git-xac-thuc-https-ssh
title: "Xác thực với remote: HTTPS và token, credential helper, SSH key"
summary: "Khi remote là GitHub hay GitLab thật, mỗi lần fetch, pull, push đều phải chứng minh bạn là ai. Bài này phân biệt URL HTTPS và URL SSH, vì sao mật khẩu tài khoản không còn dùng được và personal access token thay thế nó, credential helper lưu token ở đâu, cách tạo SSH key ed25519, ssh-agent, kiểm tra bằng ssh -T, đổi URL bằng remote set-url và viết tắt bằng insteadOf; trong GitStudio là các trang Tuỳ chọn SSH, Thông tin đăng nhập và Tích hợp."
category: git
level: intermediate
tags: [git, git-024, https, ssh, ssh-key, credential-helper, personal-access-token, gitstudio, toi-noi]
language: vi
---

Remote của lab là một thư mục trên máy, nên không ai hỏi mật khẩu. Thứ Hai tới, nhóm Tới Nơi chuyển repository lên GitHub (GIT-025). Từ đó, mỗi lần fetch, pull, push, máy chủ đều hỏi: **bạn là ai, và có quyền với repository này không?**

Git có hai đường để trả lời, chọn theo **dạng URL** của remote: HTTPS (tên người dùng và token) hoặc SSH (cặp khoá). Bài này giúp bạn chọn một, cài cho đúng, và biết sửa khi Git cứ hỏi mật khẩu mãi.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-024 · Phần 3/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-remote-origin-tracking">GIT-021</a>. Lab: <code>npm run p03 -- 24</code> (lab không có máy chủ thật, nên chỉ chạy phần URL và cấu hình). Sau bài này, bạn đọc được URL của remote để biết Git sẽ xác thực kiểu gì, cài được token hoặc SSH key, và chuyển qua lại giữa hai kiểu.</p>
</div>

## 1. Hai dạng URL, hai cách xác thực

```sh
git remote add github https://github.com/toinoi/bao-gia.git
git remote -v
git remote set-url github git@github.com:toinoi/bao-gia.git
git remote get-url github
```

```text
github	https://github.com/toinoi/bao-gia.git (fetch)
github	https://github.com/toinoi/bao-gia.git (push)
origin	<lab>/work/remote/bao-gia.git (fetch)
origin	<lab>/work/remote/bao-gia.git (push)
git@github.com:toinoi/bao-gia.git
```

| | HTTPS | SSH |
|---|---|---|
| URL | `https://github.com/toinoi/bao-gia.git` | `git@github.com:toinoi/bao-gia.git` (hoặc `ssh://git@github.com/toinoi/bao-gia.git`) |
| Bạn đưa ra | Tên người dùng + **token** | **Private key** trên máy, khớp public key đã đăng ký trên GitHub |
| Ai giữ bí mật | Credential helper (mục 3) | File `~/.ssh/id_ed25519`, có thể kèm passphrase, ssh-agent |
| Cổng mạng | 443 (hầu như mạng nào cũng mở) | 22 (mạng công ty đôi khi chặn; GitHub có `ssh.github.com:443`) |
| Hết hạn | Token có hạn dùng | Key dùng tới khi bạn gỡ |

Không có cách nào "an toàn hơn hẳn": cả hai đều tốt nếu bí mật được giữ đúng chỗ. Nhóm Tới Nơi dùng HTTPS cho máy mới (ít bước nhất), SSH cho ai đã quen. `git remote set-url` đổi qua lại bất cứ lúc nào; lịch sử không bị ảnh hưởng.

## 2. HTTPS: token thay cho mật khẩu

GitHub (từ 2021) và GitLab không nhận mật khẩu tài khoản khi dùng Git qua HTTPS. Thay vào đó là **personal access token (PAT)**: một chuỗi dài do bạn tạo trên trang cài đặt, có phạm vi quyền (chỉ đọc, đọc ghi repository…) và hạn dùng. Khi Git hỏi `Password for 'https://…'`, bạn dán token.

Trên GitHub, nên dùng **fine-grained token**: chỉ chọn đúng repository cần dùng, quyền *Contents: Read and write* để push. Lộ token thì thu hồi token đó, không cần đổi mật khẩu tài khoản.

## 3. Credential helper: không phải gõ lại

Gõ token mỗi lần push thì không ai chịu nổi. **Credential helper** là chương trình Git gọi để lấy và lưu thông tin đăng nhập:

| `credential.helper` | Lưu ở đâu | Ghi chú |
|---|---|---|
| `manager` | Git Credential Manager (đi kèm Git for Windows) | Mặc định trên Windows; có đăng nhập qua trình duyệt cho GitHub/GitLab/Azure |
| `wincred` | Windows Credential Manager | Đơn giản, chỉ lưu |
| `store` | `~/.git-credentials`, **văn bản thường** | Tránh dùng: ai đọc được file là có token |
| `cache` | Bộ nhớ, có thời hạn | Linux/macOS |
| `osxkeychain` | Keychain của macOS | |

```sh
git config --show-scope --get credential.helper
```

Trên máy bình thường lệnh này in kiểu `system	manager`. Lab cô lập cấu hình (không đọc config hệ thống), nên không in gì: Git sẽ hỏi trực tiếp.

Muốn xoá token cũ đã lưu (đổi tài khoản, token hết hạn): mở Windows Credential Manager và xoá mục `git:https://github.com`, hoặc trên GitStudio như mục 6.

## 4. SSH: cặp khoá

```sh
ssh-keygen -t ed25519 -C "minh@toinoi.test"
```

Lệnh tạo hai file trong `~/.ssh/`: `id_ed25519` (**private key**, không bao giờ rời máy, không gửi cho ai) và `id_ed25519.pub` (**public key**, dán lên GitHub → Settings → SSH and GPG keys). Passphrase mã hoá private key trên đĩa; có passphrase thì nạp key vào **ssh-agent** một lần mỗi phiên để không phải gõ lại:

```sh
ssh-add ~/.ssh/id_ed25519
ssh -T git@github.com
```

`ssh -T` thử đăng nhập: GitHub trả lời "Hi <tên>! You've successfully authenticated…". Lần kết nối đầu tiên, SSH hỏi có tin **host key** của máy chủ không; đối chiếu fingerprint với trang tài liệu của GitHub trước khi gõ `yes`, rồi nó được ghi vào `~/.ssh/known_hosts`.

Trên Windows, dịch vụ "OpenSSH Authentication Agent" mặc định tắt; bật trong Services (Startup type: Automatic) trước khi `ssh-add`.

## 5. Viết tắt URL: `insteadOf`

Gõ URL dài mỗi lần clone rất dễ sai. `url.<gốc>.insteadOf <viết tắt>` cho Git tự thay:

```sh
git config url.<lab>/work/remote/.insteadOf toinoi:
git ls-remote --heads toinoi:bao-gia.git
```

```text
dbdf6787a37810fbf1b918f3f9d1e4c8d26b862e	refs/heads/main
c6e65d730f741d9af24e5bc222acef8e501ae995	refs/heads/phat-hanh/1.0
```

Với GitHub thật, một cách dùng phổ biến: giữ URL HTTPS trong mọi repository nhưng âm thầm đi bằng SSH:

```sh
git config --global url."git@github.com:".insteadOf "https://github.com/"
```

## 6. Trên GitStudio

GitStudio gọi đúng `git` trong PATH, nên credential helper, `~/.ssh/config` và ssh-agent hoạt động y như trong terminal. Ba trang trong **Tuỳ chọn** (biểu tượng bánh răng ở góc phải thanh tab):

- **SSH**: phần **Xác thực** chọn **Dùng cấu hình SSH của máy** (giống `git` trong terminal; hợp với key có passphrase đã nạp vào agent) hoặc **Dùng một key cụ thể** (GitStudio truyền key đó cho git với mọi remote SSH). **Kiểm tra SSH tới GitHub** chạy tương đương `ssh -T git@github.com`. **Tạo key mới** tạo cặp ed25519 trong `~/.ssh`; nút **Thêm lên GitHub** trên key gửi public key lên tài khoản đã kết nối. Công tắc **URL remote** chọn URL SSH khi clone hay thêm repository GitHub. Lưu ý từ chính GitStudio: nó không thể nhập passphrase thay cho git, nên key có passphrase phải nạp vào ssh-agent.
- **Thông tin đăng nhập**: chọn **Credential helper** (Git Credential Manager, Kho thông tin đăng nhập Windows, Lưu dạng văn bản thường), xem các host HTTPS đã lưu (không bao giờ hiện mật khẩu), thêm tài khoản, **Cập nhật token…**, và chọn tài khoản cho từng repository khi một host có nhiều tài khoản (tài khoản công ty và cá nhân).
- **Tích hợp → Kết nối GitHub**: **Đăng nhập qua trình duyệt**, **Dùng đăng nhập của Git Credential Manager**, **Dùng đăng nhập của GitHub CLI (gh)** hoặc **Dùng personal access token**. Token lưu trong Windows Credential Manager. Khi repository chưa chọn tài khoản riêng, fetch, pull và push dùng tài khoản GitHub đã kết nối.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-xac-thuc-https-ssh/gitstudio-01.webp" alt="Trang Tuỳ chọn SSH của GitStudio: phần Xác thực với hai lựa chọn Dùng cấu hình SSH của máy và Dùng một key cụ thể, nút Kiểm tra SSH tới GitHub, ô URL remote, danh sách Key trong ~/.ssh có một key id_ed25519 và khung Tạo key mới" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tuỳ chọn → SSH</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-xac-thuc-https-ssh/gitstudio-02.webp" alt="Trang Tuỳ chọn Thông tin đăng nhập của GitStudio: ô Credential helper đang Chưa đặt, mục Thông tin đăng nhập đã lưu có github.com và gitlab.com đều Chưa lưu với nút Thêm tài khoản…, ghi chú khi chưa chọn tài khoản thì dùng tài khoản GitHub đã đăng nhập" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tuỳ chọn → Thông tin đăng nhập: credential helper và tài khoản theo host</figcaption></figure>

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Đổi URL sang SSH | Chuột phải remote → Chuyển sang URL SSH | `git remote set-url origin git@github.com:…` |
| Chọn credential helper | Tuỳ chọn → Thông tin đăng nhập | `git config --global credential.helper manager` |
| Xoá token đã lưu | Thông tin đăng nhập → Gỡ thông tin đăng nhập | Windows Credential Manager; `git credential reject` |
| Tạo SSH key | Tuỳ chọn → SSH → Tạo key mới | `ssh-keygen -t ed25519` |
| Thử kết nối SSH | Kiểm tra SSH tới GitHub | `ssh -T git@github.com` |
| Kết nối tài khoản GitHub | Tuỳ chọn → Tích hợp → Kết nối GitHub | `gh auth login` |

## 8. Lỗi thường gặp

- **`remote: Support for password authentication was removed`.** Bạn gõ mật khẩu tài khoản; dùng token.
- **`Authentication failed` dù token đúng.** Credential helper đang trả token cũ đã hết hạn hoặc của tài khoản khác. Xoá mục đã lưu rồi thử lại.
- **`Permission denied (publickey)`.** SSH không đưa ra được key nào GitHub biết: key chưa thêm lên GitHub, chưa nạp vào agent, hoặc `~/.ssh/config` trỏ sai file. `ssh -vT git@github.com` cho biết nó đã thử những key nào.
- **`Host key verification failed`.** `known_hosts` chưa có (hoặc có bản cũ) host key của máy chủ. Đừng tắt kiểm tra; đối chiếu fingerprint chính thức rồi cập nhật.
- **Repository không tồn tại (`Repository not found`)** dù URL đúng. Với repository riêng tư, GitHub trả lỗi này cho tài khoản không có quyền, kể cả khi bạn đang dùng nhầm tài khoản.
- **Dán token vào URL** (`https://token@github.com/…`). Token nằm trong `.git/config` dạng chữ, lộ qua `git remote -v` và ảnh chụp màn hình.

## 9. Bài tập

**Bài 1.** Một remote có URL `ssh://git@ssh.github.com:443/toinoi/bao-gia.git`. Git sẽ xác thực kiểu gì, và vì sao lại có `:443`?

<details><summary><b>Lời giải</b></summary><p>SSH, bằng key. GitHub nhận SSH ở cổng 443 trên host <code>ssh.github.com</code> cho mạng chặn cổng 22. Dạng <code>ssh://</code> cho phép ghi cổng; dạng rút gọn <code>git@host:đường-dẫn</code> thì không.</p></details>

**Bài 2.** Viết cấu hình để mọi URL `https://github.com/toinoi/` trong mọi repository được đi qua SSH, nhưng repository của tổ chức khác vẫn dùng HTTPS.

<details><summary><b>Lời giải</b></summary><p><code>git config --global url."git@github.com:toinoi/".insteadOf "https://github.com/toinoi/"</code>. Git chọn quy tắc <code>insteadOf</code> có tiền tố khớp dài nhất.</p></details>

**Bài 3.** Vì sao không nên chọn "Lưu dạng văn bản thường" trên máy dùng chung?

<details><summary><b>Lời giải</b></summary><p>Helper <code>store</code> ghi token vào <code>~/.git-credentials</code> không mã hoá; ai đọc được thư mục người dùng là dùng được token để đọc và ghi mọi repository mà token có quyền. GCM và Windows Credential Manager mã hoá theo tài khoản Windows.</p></details>

## 10. Đọc tiếp

- [gitcredentials](https://git-scm.com/docs/gitcredentials), [Git Credential Manager](https://github.com/git-ecosystem/git-credential-manager).
- [GitHub Docs: Connecting to GitHub with SSH](https://docs.github.com/en/authentication/connecting-to-github-with-ssh), [Managing your personal access tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens).
- [git-config: `url.<base>.insteadOf`](https://git-scm.com/docs/git-config#Documentation/git-config.txt-urlltbasegtinsteadOf).

Bài tiếp theo: [GIT-025 · GitHub và GitLab trong GitStudio](/articles/git-github-gitlab-gitstudio).
