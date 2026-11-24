---
slug: git-su-co-thuong-gap
title: "Sự cố Git thường gặp: index.lock, detached HEAD, push nhầm main và các thông báo lỗi khó hiểu"
summary: "Sổ tay xử lý sự cố của nhóm Tới Nơi, mỗi sự cố theo cùng khuôn: triệu chứng, nguyên nhân, cách sửa an toàn, cách phòng. Làm thật trong lab: index.lock còn sót, commit trong lúc detached HEAD rồi chuyển đi, push nhầm một thay đổi chưa duyệt lên main (revert, không force push). Kèm bảng tra các thông báo lỗi hay gặp: dubious ownership, unrelated histories, non-fast-forward, LF/CRLF, filename too long, repository hỏng. Và GitStudio báo những sự cố đó ra sao."
category: git
level: advanced
tags: [git, git-059, troubleshooting, index-lock, detached-head, git-revert, gitstudio, toi-noi]
language: vi
---

Chiều 04/12/2026, trong một giờ Minh gặp ba sự cố. GitStudio báo không stage được vì "index.lock: File exists". Một commit thử trên bản 1.3.1 "biến mất" sau khi chuyển về `main`. Rồi Minh push lên `main` một thay đổi lẽ ra phải qua pull request. Sự cố Git hiếm khi làm mất dữ liệu nếu xử lý đúng. Mất dữ liệu thường đến từ cách sửa vội: xoá thư mục `.git`, `reset --hard`, `push --force`. Bài này là sổ tay để tra khi gặp chuyện, mỗi mục cùng một khuôn.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-059 · Phần 6/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-reflog-tim-lai-commit">GIT-034</a> (reflog), <a href="/articles/git-revert-commit-da-chia-se">GIT-033</a> (revert), <a href="/articles/git-thuc-hanh-don-lich-su-hoan-tac">GIT-040</a> (runbook hoàn tác). Lab: <code>npm run p06 -- 59</code>; mở bằng GitStudio: <code>npm run setup -- 59</code>. Quy tắc chung trước mọi cách sửa: dừng lại, đọc kỹ thông báo, chạy <code>git status</code>, và sao lưu cả thư mục repository nếu định làm gì không hoàn tác được.</p>
</div>

## 1. index.lock còn sót

**Triệu chứng.** Mọi lệnh ghi (add, commit, checkout, pull) đều báo:

```sh
git add docs/ghi-chu.md
```

```text
fatal: Unable to create '<lab>/work/bao-gia/.git/index.lock': File exists.

Another git process seems to be running in this repository, e.g.
an editor opened by 'git commit'. Please make sure all processes
are terminated then try again. If it still fails, a git process
may have crashed in this repository earlier:
remove the file manually to continue.
```

**Nguyên nhân.** Trước khi ghi index (GIT-004), Git tạo `.git/index.lock` để không có hai tiến trình ghi cùng lúc, ghi xong thì đổi tên file đó thành `index`. File còn sót nghĩa là có một tiến trình git **đang chạy** (IDE, GitStudio fetch nền, terminal khác, `git commit` đang chờ trình soạn thảo), hoặc một tiến trình đã **bị tắt ngang** (tắt máy, kill, ổ mạng mất kết nối).

**Sửa.** Kiểm tra trước, xoá sau:

```sh
Get-Process git -ErrorAction SilentlyContinue
Remove-Item .git/index.lock
git add docs/ghi-chu.md
```

Đóng IDE, GitStudio, các terminal khác; nếu `Get-Process git` vẫn còn tiến trình, đợi hoặc tắt chúng. Chỉ khi chắc không còn ai ghi mới xoá file khoá. Xoá khi tiến trình khác đang ghi có thể làm hỏng index. Index hỏng thì sửa được (`Remove-Item .git/index; git reset`, mất phần đã stage nhưng không mất commit), nhưng phiền hơn nhiều. Cùng cách xử lý cho `HEAD.lock`, `refs/heads/<nhánh>.lock`, `config.lock`.

**Phòng.** Không để repository trong thư mục đồng bộ (OneDrive, Dropbox): chương trình đồng bộ giữ file và gây khoá. Không mở cùng repository bằng hai công cụ cùng ghi một lúc.

## 2. Commit trong lúc detached HEAD rồi chuyển đi

```sh
git switch --detach v1.3.1
git commit -q --allow-empty -m "test(cuoc): thử lại lỗi làm tròn trên 1.3.1"
git switch main
```

```text
HEAD is now at 42b7cf8 chore(release): 1.3.1
Your branch is up to date with 'origin/main'.
Warning: you are leaving 1 commit behind, not connected to
any of your branches:

  68a37d4 test(cuoc): thử lại lỗi làm tròn trên 1.3.1

If you want to keep it by creating a new branch, this may be a good time
to do so with:

 git branch <new-branch-name> 68a37d4

Switched to branch 'main'
```

**Nguyên nhân.** Checkout một tag hay một commit (GIT-016, bisect ở GIT-041) đưa HEAD vào trạng thái **detached** (GIT-008): HEAD trỏ thẳng vào commit, không qua nhánh nào. Commit vẫn tạo được, nhưng không nhánh nào giữ nó. Khi HEAD chuyển đi, commit chỉ còn được reflog nhớ, và `gc` sẽ xoá sau thời hạn của reflog (mặc định 30 ngày với commit không còn ref nào trỏ tới).

**Sửa.** Git đã in sẵn mã commit. Mất dòng đó thì tra reflog (GIT-034):

```sh
git reflog -3 --format="%h %gs"
git branch thu/lam-tron-1.3.1 68a37d4
git log --oneline -1 thu/lam-tron-1.3.1
```

```text
a772579 checkout: moving from 68a37d4517b5390c67421c9706f112c7b5521f78 to main
68a37d4 commit: test(cuoc): thử lại lỗi làm tròn trên 1.3.1
42b7cf8 checkout: moving from main to v1.3.1
68a37d4 test(cuoc): thử lại lỗi làm tròn trên 1.3.1
```

Lab xoá nhánh thử ngay sau đó (`git branch -D`) vì đó chỉ là một lần thử.

**Phòng.** Muốn làm việc trên một bản cũ thì tạo nhánh ngay từ đầu: `git switch -c thu/lam-tron v1.3.1`.

## 3. Push nhầm lên main

```sh
git commit -q -am "chore(ban-do): bật ước lượng quãng đường mặc định"
git push -q
```

Minh định đưa thay đổi này vào nhánh để Lan duyệt, nhưng đang đứng ở `main`, và remote chấp nhận vì đó là fast-forward. Hook pre-receive (GIT-027) chỉ chặn ghi đè lịch sử, không chặn push thẳng; GitHub thì có quy tắc "yêu cầu pull request" cho việc này.

**Sửa.** `main` đã chia sẻ, nên **không reset, không force push** (GIT-033): người khác có thể đã pull, CI có thể đã deploy. Giữ thay đổi trên một nhánh, rồi revert trên `main`:

```sh
git branch tinh-nang/ban-do-mac-dinh
git revert --no-edit HEAD
git push -q
git log --oneline -3
git diff main~2 main --stat
```

```text
[main c52b396] Revert "chore(ban-do): bật ước lượng quãng đường mặc định"
 Date: Fri Dec 4 14:20:00 2026 +0700
 1 file changed, 3 deletions(-)
c52b396 Revert "chore(ban-do): bật ước lượng quãng đường mặc định"
07d9ea8 chore(ban-do): bật ước lượng quãng đường mặc định
a772579 docs: cấu hình git khuyến nghị cho nhóm
```

`git diff main~2 main` không in gì: nội dung `main` đã về đúng như trước khi push nhầm. Nhánh `tinh-nang/ban-do-mac-dinh` giữ commit gốc để mở pull request. Khi merge pull request đó về sau, Git thấy commit đã có trong lịch sử (và đã bị revert), nên thay đổi **không** quay lại. Phải revert chính commit revert (`git revert c52b396`) hoặc cherry-pick lại thay đổi trên một nhánh mới (GIT-033).

**Nếu chưa push.** Commit nhầm vào `main` mà chưa push thì đơn giản hơn: `git branch tinh-nang/x` (giữ commit), rồi `git reset --keep origin/main` (đưa `main` về khớp remote, giữ thay đổi chưa commit nếu có).

**Phòng.** Bật quy tắc nhánh trên GitHub cho `main` (yêu cầu pull request, GIT-027). Để ý nhánh hiện tại trên thanh công cụ của GitStudio hoặc trong prompt terminal trước khi push.

## 4. Bảng tra thông báo lỗi

| Thông báo (rút gọn) | Nghĩa | Cách sửa an toàn |
|---|---|---|
| `detected dubious ownership in repository at '…'` | Thư mục thuộc người dùng Windows khác (ổ ngoài, copy từ máy khác, chạy bằng quyền admin). Git từ chối để tránh chạy cấu hình của người lạ | Nếu tin thư mục đó: `git config --global --add safe.directory D:/duong/dan`. Không dùng `*` |
| `refusing to merge unrelated histories` | Hai nhánh không có commit chung (repository khởi tạo riêng, gộp repository) | Chắc chắn là cố ý thì `git merge --allow-unrelated-histories` (GIT-057); không thì bạn đang pull nhầm remote |
| `! [rejected] main -> main (non-fast-forward)` / `fetch first` | Remote có commit bạn chưa có | `git pull` (merge hoặc rebase, GIT-029), rồi push. Không `--force` lên nhánh dùng chung (GIT-023) |
| `remote: Từ chối: …` / `pre-receive hook declined` | Quy tắc phía máy chủ (GIT-027) | Đọc dòng `remote:`; quy tắc đúng thì sửa cách làm, không tìm đường vòng |
| `LF will be replaced by CRLF` | Cảnh báo xuống dòng (GIT-007, GIT-043) | Thường bỏ qua được; muốn hết hẳn thì khai báo `.gitattributes` và `git add --renormalize .` |
| `Filename too long` (Windows) | Đường dẫn vượt 260 ký tự | `git config --global core.longpaths true` (GitStudio: Tuỳ chọn → Chung → Đường dẫn dài) |
| `Your branch and 'origin/main' have diverged` | Cả hai bên có commit riêng | Chọn merge hay rebase (GIT-029); `pull.ff only` (GIT-058) dừng ở đây để bạn chọn |
| `CONFLICT (content): Merge conflict in …` | Xung đột (GIT-014) | Giải quyết, `git add`, tiếp tục; hoặc `git merge --abort` / `rebase --abort` |
| `error: object file … is empty`, `fatal: bad object` | Repository hỏng: thường do tắt máy khi đang ghi, ổ đĩa lỗi, thư mục đồng bộ | `git fsck --full` (GIT-047) để biết phạm vi; sao lưu thư mục; clone lại từ remote rồi chép thay đổi chưa push sang |
| `Permission denied (publickey)` | SSH không có khoá đúng (GIT-024) | `ssh -T git@github.com` để kiểm tra; thêm khoá vào agent |
| `fatal: not a git repository` | Đứng sai thư mục, hoặc `.git` bị xoá | `git rev-parse --show-toplevel` từ thư mục đúng; `.git` mất thì clone lại |

Hai việc **không** làm khi gặp lỗi lạ: xoá `.git` để "làm lại cho sạch" (mất mọi commit chưa push, stash, nhánh cục bộ), và chép lệnh `--force` từ mạng khi chưa hiểu lệnh đó làm gì. Gần như mọi thứ đã commit đều cứu được bằng reflog (GIT-034), còn thứ chưa từng commit thì không.

## 5. Trên GitStudio

- **Lỗi của git hiện nguyên văn bằng tiếng Anh** trong thông báo "… thất bại" và trong **Nhật ký hoạt động** (`Ctrl+Shift+L`), vì GitStudio chạy git với `LC_ALL=C`. Tra bảng ở mục 4 theo câu tiếng Anh đó. GitStudio không tự xoá `index.lock`.
- **Detached HEAD**: trước khi checkout một commit hay tag, hộp thoại **Checkout commit** nói trước rằng repository sẽ vào trạng thái detached HEAD. Thanh công cụ ghi `detached @ <mã>`, thanh bên ghi `HEAD (detached)`. Khi chuyển đi mà còn commit không thuộc nhánh nào, hộp thoại **Có commit không thuộc nhánh nào** nói rõ số commit sẽ chỉ còn tìm được qua reflog. Ô **Trước khi chuyển** mặc định là **Tạo nhánh tại HEAD để giữ lại** (lựa chọn còn lại là **Vẫn chuyển**), rồi bấm **Tiếp tục**.
- **Reflog**: mục **Lịch sử HEAD (reflog)** ở thanh bên đánh dấu commit "mất" (không thuộc nhánh hay tag nào) và cho tạo nhánh từ đó.
- **Push bị từ chối**: hộp thoại **Push bị từ chối** gợi ý pull trước, và có nút **Force push** (with lease). Với nhánh dùng chung, chọn pull.
- **Thao tác dở**: banner **Đang merge / rebase / cherry-pick / revert** ở đầu cửa sổ có **Huỷ thao tác** và **Tiếp tục**.
- **Repository hỏng**: **Bảo trì repository…** → **Kiểm tra toàn vẹn** (GIT-047).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-su-co-thuong-gap/gitstudio-01.webp" alt="GitStudio báo Stage thất bại với thông báo git Unable to create index.lock: File exists, khi file .git/index.lock còn sót" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · index.lock còn sót: lỗi git hiện nguyên văn</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-su-co-thuong-gap/gitstudio-02.webp" alt="Hộp thoại Có commit không thuộc nhánh nào của GitStudio khi chuyển từ detached HEAD tại 68a37d4 về main: ô Trước khi chuyển đang chọn Tạo nhánh tại HEAD để giữ lại, nút Huỷ và Tiếp tục; đồ thị có nhãn HEAD ở commit thử" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · rời detached HEAD còn commit chưa có nhánh</figcaption></figure>

## 6. Lỗi thường gặp khi xử lý sự cố

- **Xoá `index.lock` khi GitStudio hay IDE đang chạy fetch nền**: lỗi quay lại, hoặc index hỏng. Đóng hết trước.
- **`reset --hard origin/main` để "sửa" push nhầm**: `main` trên remote vẫn có commit nhầm; lần push sau bị từ chối, rồi bạn force push xoá luôn commit của người khác.
- **Bỏ qua cảnh báo "leaving 1 commit behind"**: commit chỉ còn trong reflog, sau 30 ngày `gc` có thể xoá.
- **Thêm `safe.directory = *`** cho đỡ phiền: tắt luôn lớp bảo vệ trên mọi thư mục.

## 7. Bài tập

**Bài 1.** `git pull` báo `index.lock: File exists`, còn GitStudio đang mở cùng repository. Các bước an toàn?

<details><summary><b>Lời giải</b></summary><p>Đợi vài giây (có thể GitStudio đang fetch nền), thử lại. Vẫn lỗi thì đóng GitStudio và IDE, kiểm tra <code>Get-Process git</code>, rồi mới <code>Remove-Item .git/index.lock</code> và pull lại.</p></details>

**Bài 2.** Merge pull request `tinh-nang/ban-do-mac-dinh` vào `main` sau khi đã revert ở mục 3, thay đổi có quay lại không? Làm sao để nó quay lại?

<details><summary><b>Lời giải</b></summary><p>Không: commit gốc đã nằm trong <code>main</code> (và đã bị revert), merge không đưa lại thay đổi. Revert commit revert (<code>git revert c52b396</code>) trên nhánh của pull request, hoặc cherry-pick commit gốc lên một nhánh mới từ <code>main</code>.</p></details>

**Bài 3.** Đồng nghiệp copy thư mục repository từ ổ ngoài vào máy, và mọi lệnh git báo `dubious ownership`. Vì sao Git làm vậy?

<details><summary><b>Lời giải</b></summary><p>Thư mục thuộc người dùng khác, và <code>.git/config</code> có thể chứa cấu hình chạy lệnh (<code>core.fsmonitor</code>, hook). Git không muốn chạy chúng thay bạn. Nếu tin thư mục đó, thêm đúng đường dẫn vào <code>safe.directory</code>, hoặc đổi chủ sở hữu thư mục.</p></details>

## 8. Đọc tiếp

- [Pro Git: Data Recovery](https://git-scm.com/book/en/v2/Git-Internals-Maintenance-and-Data-Recovery), [git-config: safe.directory](https://git-scm.com/docs/git-config#Documentation/git-config.txt-safedirectory).

Bài tiếp theo: [GIT-060 · Thực hành Phần 6: quy trình phát hành 1.4.0 của nhóm Tới Nơi](/articles/git-thuc-hanh-quy-trinh-phat-hanh).
