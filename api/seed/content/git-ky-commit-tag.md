---
slug: git-ky-commit-tag
title: "Ký commit và tag: chứng minh v1.2.0 đúng là do Minh phát hành, bằng khoá SSH hoặc GPG"
summary: "user.name và user.email ai cũng tự đặt được, nên một commit ghi Trần Minh chưa chắc do Minh tạo. Bài này ký commit và tag phát hành v1.2.0 bằng khoá SSH: gpg.format ssh, user.signingkey, commit -S, tag -s; xác minh bằng log --show-signature, verify-commit, tag -v với file allowed_signers commit cùng repository; so sánh với GPG, cách bật ký mặc định, huy hiệu Verified của GitHub, và trang Ký commit cùng huy hiệu chữ ký trong GitStudio."
category: git
level: advanced
tags: [git, git-048, commit-signing, ssh-signing, gpg, tag, gitstudio, toi-noi]
language: vi
---

Trước khi phát hành 1.2.0, bộ phận an ninh của Tới Nơi hỏi một câu đơn giản: "Làm sao biết commit ghi tên Trần Minh đúng là của Minh?". Câu trả lời thật thà: không biết. `user.name` và `user.email` ai cũng đặt được (GIT-002); GitHub cũng chỉ ghi lại cái tên đó. **Chữ ký số** mới chứng minh được: chỉ người giữ private key ký được, ai có public key cũng kiểm tra được.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-048 · Phần 5/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-xac-thuc-https-ssh">GIT-024</a> (SSH key), <a href="/articles/git-tag-phien-ban">GIT-016</a> (annotated tag). Lab: <code>npm run p05 -- 48</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 48</code>. Lab dùng một khoá SSH demo cố định trong <code>fixtures/</code> để chữ ký và mã commit giống nhau trên mọi máy; đừng dùng khoá đó cho việc thật. Sau bài này, bạn ký và xác minh được commit, tag bằng khoá SSH, và biết khác biệt với GPG.</p>
</div>

## 1. SSH hay GPG

| | SSH (Git 2.34+) | GPG (OpenPGP) |
|---|---|---|
| Khoá | Khoá SSH bạn đã có (GIT-024) hoặc một khoá riêng để ký | Cặp khoá GPG riêng, có ngày hết hạn, thu hồi |
| Cài đặt | `ssh-keygen` có sẵn trong Git for Windows, macOS, Linux | Cần GnuPG, quản lý keyring |
| Xác minh cục bộ | File **allowed_signers** liệt kê ai được tin | Keyring GPG, mạng lưới tin cậy |
| GitHub/GitLab | Hỗ trợ (thêm khoá dạng "Signing key") | Hỗ trợ |
| Hợp với | Hầu hết nhóm | Tổ chức đã có hạ tầng PGP, cần thu hồi/hết hạn chặt |

Nhóm Tới Nơi chọn SSH: ít bước nhất.

## 2. Cấu hình ký bằng khoá SSH

```sh
git config gpg.format ssh
git config user.signingkey <lab>/fixtures/khoa-ky-demo.pub
```

Tên `gpg.format` là lịch sử (Git ban đầu chỉ có GPG). `user.signingkey` là đường dẫn **public key**; `ssh-keygen` tìm private key nằm cạnh nó (cùng tên, bỏ `.pub`) hoặc trong ssh-agent. Trên máy thật: `git config --global user.signingkey ~/.ssh/id_ed25519.pub`.

## 3. Ai được tin: allowed_signers

Ký thì chỉ cần private key; **xác minh** cần biết khoá nào thuộc về ai. Với SSH, đó là file `allowed_signers`: mỗi dòng một email, phạm vi (`namespaces="git"`), và public key. Nhóm commit file này vào repository để ai clone về cũng xác minh được:

```text
minh@toinoi.test namespaces="git" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIETm/+hYtvRupU2iS7zX2tz3qxPXIC8pXk19M39U9MDQ
```

```sh
git config gpg.ssh.allowedSignersFile .allowed_signers
```

Đường dẫn tương đối được tính từ thư mục gốc repository. Thêm người mới = thêm một dòng qua pull request, như mọi thay đổi khác.

## 4. Ký và xác minh commit

```sh
git add .allowed_signers
git commit -S -m "Thêm danh sách khoá ký của nhóm"
git log --show-signature -1
git verify-commit HEAD
git verify-commit HEAD~1
```

```text
[main 1f0a787] Thêm danh sách khoá ký của nhóm
 1 file changed, 1 insertion(+)
 create mode 100644 .allowed_signers
commit 1f0a787e22591b78dd1ee8efc78495703af4fefb
Good "git" signature for minh@toinoi.test with ED25519 key SHA256:ktrw7kr64a2V/wcdx7FKqeY6NQwEDtlodCSRHgsirWw
Author: Trần Minh <minh@toinoi.test>
Date:   Sat Nov 28 09:00:00 2026 +0700

    Thêm danh sách khoá ký của nhóm
Good "git" signature for minh@toinoi.test with ED25519 key SHA256:ktrw7kr64a2V/wcdx7FKqeY6NQwEDtlodCSRHgsirWw
```

`-S` ký commit; chữ ký nằm **trong** commit object (trường `gpgsig`), nên là một phần của mã hash: sửa commit (amend, rebase) là mất chữ ký, phải ký lại. `verify-commit` thoát 0 khi chữ ký hợp lệ và khoá có trong allowed_signers. `HEAD~1` không ký: không in gì, thoát mã 1.

Muốn ký **mọi** commit: `git config --global commit.gpgsign true`. Rebase, cherry-pick, amend khi đó cũng tự ký lại.

## 5. Tag phát hành có chữ ký

```sh
git tag -s v1.2.0 -m "Phát hành 1.2.0: phụ phí đêm, phí chờ theo giây"
git tag -v v1.2.0
git push -q origin main v1.2.0
```

```text
object 1f0a787e22591b78dd1ee8efc78495703af4fefb
type commit
tag v1.2.0
tagger Trần Minh <minh@toinoi.test> 1795833000 +0700

Phát hành 1.2.0: phụ phí đêm, phí chờ theo giây
Good "git" signature for minh@toinoi.test with ED25519 key SHA256:ktrw7kr64a2V/wcdx7FKqeY6NQwEDtlodCSRHgsirWw
```

`-s` tạo annotated tag có chữ ký (GIT-016). Tag ký là bằng chứng mạnh nhất cho một bản phát hành: nó gắn chặt tên phiên bản với **đúng một commit**, và commit đó (qua mã hash) gắn với toàn bộ cây file và lịch sử phía trước. `git config --global tag.gpgSign true` để mọi annotated tag đều ký.

## 6. Trên GitHub, GitLab

Thêm public key vào tài khoản dưới dạng **Signing key** (GitHub: Settings → SSH and GPG keys → New SSH key → Key type: Signing key). Commit ký bằng khoá đó, với email khớp email đã xác minh của tài khoản, có huy hiệu **Verified**. Branch protection (GIT-027) có tuỳ chọn **Require signed commits**. Lưu ý: commit do GitHub tạo trên web (merge pull request, squash) được GitHub ký bằng khoá của chính GitHub.

## 7. Trên GitStudio

Tuỳ chọn → **Ký commit**: **Áp dụng cho** Repository này hoặc Mọi repository; **Ký mọi commit** (`commit.gpgsign`), **Ký annotated tag** (`tag.gpgSign`), **Kiểu chữ ký** (Chưa đặt (GPG) / Khoá SSH, tức `gpg.format`), **Khoá ký** (đường dẫn public key với SSH, Key ID với GPG). Ghi chú của chính GitStudio: "Để xác minh chữ ký SSH, git còn cần gpg.ssh.allowedSignersFile."

Panel commit hiện huy hiệu chữ ký: **Đã xác minh** ("Được ký bằng khoá tin cậy"), **Đã ký** (hợp lệ nhưng khoá chưa đánh dấu tin cậy), **Đã ký (chưa xác minh)** (thiếu khoá hoặc chưa đặt allowedSignersFile), **Chữ ký sai**, **Chữ ký hết hạn**, **Khoá đã bị thu hồi**; rê chuột lên huy hiệu để xem **Người ký** và **Khoá** (dấu vân tay SHA256). Tuỳ chọn → Hồ sơ → **Ký commit** là công tắc nhanh cho `commit.gpgsign` toàn cục.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-ky-commit-tag/gitstudio-01.webp" alt="Trang Tuỳ chọn Ký commit của GitStudio áp dụng cho repository bao-gia: Kiểu chữ ký Khoá SSH, Khoá ký là đường dẫn tới khoa-ky-demo.pub, các ô Ký mọi commit và Ký annotated tag" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Tuỳ chọn → Ký commit</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-ky-commit-tag/gitstudio-02.webp" alt="Commit Thêm danh sách khoá ký của nhóm trong GitStudio: cạnh mã commit có huy hiệu Đã xác minh, đồ thị có tag v1.2.0 ở commit này, panel bên phải liệt kê file .allowed_signers được thêm" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · huy hiệu chữ ký trên commit</figcaption></figure>

## 8. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Chọn kiểu và khoá | Tuỳ chọn → Ký commit | `git config gpg.format ssh`, `user.signingkey` |
| Tin khoá của nhóm | (file trong repository) | `gpg.ssh.allowedSignersFile` |
| Ký một commit | (bật Ký mọi commit) | `git commit -S` |
| Ký mọi commit | Ký mọi commit | `git config commit.gpgsign true` |
| Ký tag | Ký annotated tag | `git tag -s`, `tag.gpgSign` |
| Xác minh | Huy hiệu trên panel commit | `git verify-commit`, `git tag -v`, `log --show-signature` |

## 9. Lỗi thường gặp

- **"Signed (unverified)"** trong GitStudio, `No principal matched` trong terminal: thiếu `gpg.ssh.allowedSignersFile` hoặc email trong file không khớp email commit.
- **Khoá có passphrase mà không có ssh-agent**: Git (và GitStudio) không nhập passphrase hộ được (GIT-024). Nạp khoá vào agent.
- **Ký rồi rebase/amend khi chưa bật `commit.gpgsign`**: commit mới không có chữ ký.
- **Tin chữ ký mà không tin khoá**: chữ ký chỉ chứng minh "người giữ khoá này ký"; allowed_signers phải được review như code.
- **Dùng chung một khoá cho cả nhóm**: mất ý nghĩa; mỗi người một khoá.

## 10. Bài tập

**Bài 1.** Xem chữ ký của tag `v1.2.0` mà không in nội dung tag.

<details><summary><b>Lời giải</b></summary><p><code>git verify-tag v1.2.0</code> (kết quả trên stderr), hoặc <code>git tag -v v1.2.0 2&gt;&amp;1 | Select-String Good</code>.</p></details>

**Bài 2.** Liệt kê 10 commit gần nhất kèm trạng thái chữ ký (G tốt, N không ký…).

<details><summary><b>Lời giải</b></summary><p><code>git log -10 --format="%h %G? %s"</code>: <code>G</code> hợp lệ, <code>N</code> không ký, <code>B</code> sai, <code>U</code> hợp lệ nhưng khoá chưa tin…</p></details>

**Bài 3.** Lan muốn ký commit của cô. Cần những bước nào trên máy Lan và trong repository?

<details><summary><b>Lời giải</b></summary><p>Trên máy Lan: tạo khoá (hoặc dùng khoá SSH sẵn có), đặt <code>gpg.format ssh</code>, <code>user.signingkey</code>, <code>commit.gpgsign true</code>. Trong repository: thêm dòng <code>lan@toinoi.test namespaces="git" ssh-ed25519 …</code> vào <code>.allowed_signers</code> qua pull request. Trên GitHub: thêm khoá dạng Signing key.</p></details>

## 11. Đọc tiếp

- [git-config: `gpg.format`, `gpg.ssh.allowedSignersFile`, `commit.gpgSign`](https://git-scm.com/docs/git-config), [git-verify-commit](https://git-scm.com/docs/git-verify-commit).
- [GitHub Docs: About commit signature verification](https://docs.github.com/en/authentication/managing-commit-signature-verification/about-commit-signature-verification).

Bài tiếp theo: [GIT-049 · Hook: chặn commit sai trước khi rời máy](/articles/git-hook-pre-commit).
