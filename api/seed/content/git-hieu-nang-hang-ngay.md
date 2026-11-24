---
slug: git-hieu-nang-hang-ngay
title: "Hiệu năng hằng ngày: alias, cấu hình chung của nhóm và phím tắt GitStudio"
summary: "Sau sáu phần, nhóm Tới Nơi gõ những lệnh giống nhau hàng chục lần mỗi ngày và mỗi người cấu hình Git một kiểu. Bài này đưa cấu hình khuyến nghị vào repository (docs/git/toinoi.gitconfig) và bật bằng include.path, giải thích từng dòng (pull.ff only, push.autoSetupRemote, fetch.prune, rebase.autoStash và updateRefs, rerere, diff histogram, zdiff3, sắp xếp nhánh, tag), viết alias, đọc --show-origin để biết giá trị đến từ đâu, và các phím tắt GitStudio dùng nhiều nhất."
category: git
level: advanced
tags: [git, git-058, git-config, alias, include-path, keyboard-shortcuts, gitstudio, toi-noi]
language: vi
---

Thứ Sáu 04/12/2026, Lan pull và lại có một merge commit thừa "Merge branch 'main' of …". Minh push nhánh mới và lại gặp `fatal: The current branch has no upstream branch`. Cả hai gõ `git log --oneline --graph` hàng chục lần mỗi ngày. Không lỗi nào nghiêm trọng, nhưng cộng lại là hàng giờ mỗi tuần và vài lần lịch sử bẩn. Bài này gom các cấu hình và thói quen đáng giá nhất thành một file trong repository mà cả nhóm cùng dùng.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-058 · Phần 6/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-cai-dat-cau-hinh-gitstudio">GIT-002</a> (cấu hình), <a href="/articles/git-pull-xung-dot-lich-su-roi">GIT-029</a> (pull), <a href="/articles/git-patch-rerere">GIT-039</a> (rerere). Lab: <code>npm run p06 -- 58</code>; mở bằng GitStudio: <code>npm run setup -- 58</code>. Sau bài này, bạn chia sẻ được cấu hình Git của nhóm và biết mỗi giá trị đang có hiệu lực đến từ đâu.</p>
</div>

## 1. Cấu hình của nhóm nằm trong repository

```text
# Cấu hình git khuyến nghị cho nhóm Tới Nơi (GIT-058). Bật cho một repository:
#   git config --local include.path ../docs/git/toinoi.gitconfig
# Đường dẫn tương đối tính từ file chứa nó (.git/config). Cấu hình riêng của bạn vẫn thắng nếu đặt sau.
[alias]
	st = status -sb
	lg = log --oneline --graph --decorate -12
	last = log -1 --stat
	unstage = restore --staged
	amend = commit --amend --no-edit
	nhanh = branch --format='%(committerdate:short) %(refname:short)'
[pull]
	ff = only
[push]
	autoSetupRemote = true
[fetch]
	prune = true
[rebase]
	autoStash = true
	updateRefs = true
[rerere]
	enabled = true
[diff]
	algorithm = histogram
[merge]
	conflictStyle = zdiff3
[branch]
	sort = -committerdate
[tag]
	sort = -version:refname
```

Từng dòng, và vì sao nhóm chọn:

| Khoá | Tác dụng | Bài liên quan |
|---|---|---|
| `pull.ff = only` | `git pull` chỉ fast-forward; khi đã rẽ nhánh thì dừng lại để bạn chọn merge hay rebase. Hết merge commit thừa của Lan | GIT-029 |
| `push.autoSetupRemote = true` | Lần push đầu của nhánh mới tự đặt upstream, không cần `-u` | GIT-022 |
| `fetch.prune = true` | Fetch xoá nhánh remote-tracking đã bị xoá trên remote | GIT-021 |
| `rebase.autoStash = true` | Rebase khi có thay đổi chưa commit: tự stash rồi trả lại | GIT-015 |
| `rebase.updateRefs = true` | Rebase một chồng nhánh: các nhánh ở giữa được dời theo | GIT-038 |
| `rerere.enabled = true` | Ghi nhớ cách giải quyết xung đột, lần sau tự áp dụng | GIT-039 |
| `diff.algorithm = histogram` | Diff dễ đọc hơn khi code bị dời khối | GIT-007 |
| `merge.conflictStyle = zdiff3` | Marker xung đột có thêm khối "bản gốc", biết hai bên đã đổi gì | GIT-013 |
| `branch.sort = -committerdate` | `git branch` liệt kê nhánh mới làm gần đây lên đầu | GIT-011 |
| `tag.sort = -version:refname` | `git tag` sắp theo số phiên bản, mới nhất trên cùng | GIT-052 |

Những thứ **không** nên vào file chung: `user.name`, `user.email`, `core.autocrlf` (tuỳ máy, GIT-002), credential helper, khoá ký. Đó là việc của từng người.

## 2. Bật bằng include.path

```sh
git add docs/git
git commit -q -m "docs: cấu hình git khuyến nghị cho nhóm"
git push -q
git config --local include.path ../docs/git/toinoi.gitconfig
git config --show-origin --get pull.ff
git config --show-scope --get-regexp "^alias\."
```

```text
file:.git/../docs/git/toinoi.gitconfig	only
local	alias.st status -sb
local	alias.lg log --oneline --graph --decorate -12
local	alias.last log -1 --stat
local	alias.unstage restore --staged
local	alias.amend commit --amend --no-edit
local	alias.nhanh branch --format='%(committerdate:short) %(refname:short)'
```

Git **không tự đọc** cấu hình nằm trong repository. Đó là quyết định an toàn: clone một repository không được làm thay đổi hành vi Git trên máy bạn (cấu hình có thể chạy lệnh, ví dụ `core.sshCommand`, `core.fsmonitor`). Mỗi người bật một lần bằng `include.path`. Đường dẫn tương đối tính từ file chứa dòng `include` (`.git/config`), nên là `../docs/git/…`. Khi file trong repository đổi qua pull, cấu hình đổi theo, nên mọi thay đổi của nó phải qua review (CODEOWNERS ở GIT-056).

`--show-origin` cho biết giá trị đến từ file nào, `--show-scope` cho biết phạm vi (system, global, local). Đây là hai cờ đầu tiên cần dùng khi "máy tôi chạy khác máy bạn". Muốn áp dụng cho mọi repository trong một thư mục, ví dụ mọi repository công việc, đặt trong `~/.gitconfig`:

```text
[includeIf "gitdir:D:/ToiNoi/"]
	path = D:/ToiNoi/toinoi.gitconfig
```

## 3. Alias

```sh
git st
git lg
git nhanh
git tag -l "v1.*"
```

```text
## main...origin/main
* a772579 (HEAD -> main, origin/main, origin/HEAD) docs: cấu hình git khuyến nghị cho nhóm
*   e35b76c refactor(tien-te): gộp thư viện tiền tệ bằng subtree, giữ lịch sử
|\  
| * 31d0d96 docs: cách dùng Vnd.Format và Vnd.Short
| * 02ae86b Thêm dạng ngắn 31k
| * 23f146f Định dạng tiền Việt Nam
* 470f477 refactor(tien-te): bỏ submodule lib/tien-te
…
2026-12-04 main
2026-12-01 phat-hanh/1.3
2026-11-19 phat-hanh/1.0
v1.3.1
v1.3.0
v1.2.0
v1.1.0
v1.0.1
v1.0.0
```

Alias là tên ngắn cho một lệnh git kèm tuỳ chọn. `git nhanh` dùng `branch.sort` nên nhánh mới làm gần đây nằm trên cùng; `git tag -l` sắp theo phiên bản nhờ `tag.sort`. Alias bắt đầu bằng `!` chạy lệnh shell (`!git fetch -q && git st`), mạnh nhưng phụ thuộc shell của từng máy: tránh trong file chung của nhóm có người dùng Windows. Đừng đặt alias trùng tên lệnh có sẵn: Git bỏ qua alias đó.

## 4. Cấu hình của bạn vẫn thắng

```sh
git config --local pull.rebase true
git config --show-origin --get-regexp "^pull\."
git config --local --unset pull.rebase
```

```text
file:.git/config	pull.rebase true
file:.git/../docs/git/toinoi.gitconfig	pull.ff only
```

Git đọc lần lượt system → global → local, và trong mỗi file theo thứ tự từ trên xuống. File được include đọc ngay tại chỗ dòng `include`. Giá trị đọc sau thắng. Ai quen `pull --rebase` (GIT-029) đặt `pull.rebase true` của riêng mình; khi đó `pull.ff only` không còn tác dụng, vì rebase không tạo merge commit. Nhóm không cấm, chỉ đặt mặc định an toàn.

## 5. Phím tắt GitStudio dùng nhiều nhất

Cửa sổ **Phím tắt** (`Ctrl+/`, hoặc biểu tượng bàn phím ở thanh trạng thái) liệt kê đủ và có ô lọc. Những phím dùng hằng ngày:

| Phím | Việc |
|---|---|
| `Ctrl+K`, `Ctrl+F` | Spotlight: tìm nhánh, tag, file, commit (cả nội dung commit trong toàn bộ lịch sử), stash, thao tác (GIT-006) |
| `Ctrl+P` | Đi tới repository (tab đang mở và workspace, GIT-053) |
| `Ctrl+L` | Fetch tất cả |
| `Ctrl+B` | Tạo nhánh |
| `S` / `U`, `Ctrl+Shift+S` / `Ctrl+Shift+U` | Stage / unstage tệp đang xem, tất cả tệp |
| `Ctrl+Shift+M`, `Ctrl+Enter` | Tới ô nội dung commit, commit |
| `Ctrl+Z`, `Ctrl+Y` | Hoàn tác, làm lại (GIT-035) |
| ``Ctrl+` `` | Bật/tắt terminal trong app |
| `Ctrl+Shift+H` | Đi tới HEAD trên đồ thị |
| `Ctrl+Shift+K`, `Ctrl+Shift+L` | Launchpad, Nhật ký hoạt động |

Trong cửa sổ Phím tắt, `Ctrl+F` ghi là "Tìm commit": phím này mở Spotlight, nơi tìm commit cùng với mọi thứ khác.

GitStudio chỉ chỉnh một số khoá cấu hình cố định: tên và email, xuống dòng, nhánh mặc định, ký commit, credential helper (Tuỳ chọn → Chung, Hồ sơ, Ký commit, và Tuỳ chọn riêng cho repository). Merge tool, diff tool, trình soạn thảo, terminal nằm ở Tuỳ chọn → **Công cụ ngoài**. Không có trình sửa alias hay sửa cấu hình tự do. Nhưng GitStudio gọi `git` trong PATH, nên mọi cấu hình ở mục 1 đều có hiệu lực khi bạn pull, push, rebase từ giao diện. Riêng nút Pull có lựa chọn riêng (GIT-022).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-hieu-nang-hang-ngay/gitstudio-01.webp" alt="Cửa sổ Phím tắt của GitStudio (Ctrl+/) với ô Lọc phím tắt; nhóm Thao tác repository liệt kê Tạo nhánh Ctrl+B, Fetch tất cả Ctrl+L, Commit Ctrl+Enter, Stage S, Unstage U, Hoàn tác Ctrl+Z, bật/tắt terminal Ctrl+`, bên dưới là nhóm Điều hướng" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Phím tắt (Ctrl+/)</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-hieu-nang-hang-ngay/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio: git config --show-origin cho pull.ff trỏ tới docs/git/toinoi.gitconfig, rồi các alias git st, git lg -8 và git nhanh của nhóm" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · terminal: alias và cấu hình của nhóm có hiệu lực</figcaption></figure>

## 6. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Xem giá trị đến từ đâu | | `git config --show-origin --get <khoá>` |
| Bật cấu hình của nhóm | | `git config --local include.path ../docs/git/toinoi.gitconfig` |
| Tên, email | Tuỳ chọn → Hồ sơ | `git config --global user.name` |
| Cách pull | Tuỳ chọn → Chung → Cách pull; nút Pull ▾ | `git config pull.ff only` / `pull.rebase true` |
| Merge tool | Tuỳ chọn → Công cụ ngoài | `git config merge.tool` |
| Danh sách phím tắt | `Ctrl+/` | |

## 7. Lỗi thường gặp

- **Đặt cấu hình nhóm vào `--global` của từng người bằng tay**: mỗi máy lệch một chút sau vài tháng. File trong repository cộng với include giữ cho mọi người giống nhau.
- **`include.path` sai đường dẫn tương đối**: Git lặng lẽ bỏ qua file không tồn tại. Kiểm tra bằng `--show-origin`.
- **Bật `pull.rebase` cho cả nhóm mà nhiều người chưa quen rebase**: xung đột giữa chừng rebase khó hơn merge. Bắt đầu bằng `pull.ff only`.
- **Alias shell `!…` dùng cú pháp bash** trong nhóm có người dùng PowerShell: Git for Windows chạy alias bằng sh đi kèm nên vẫn chạy, nhưng lệnh gọi bên trong có thể không có.

## 8. Bài tập

**Bài 1.** Thêm alias `dongbo` chạy fetch rồi in trạng thái.

<details><summary><b>Lời giải</b></summary><p><code>git config --local alias.dongbo "!git fetch -q --prune && git status -sb"</code>. Dấu <code>!</code> chạy bằng shell; trong file chung của nhóm thì ghi chú rằng alias này cần sh (có sẵn trong Git for Windows).</p></details>

**Bài 2.** Lan bật `pull.rebase true` ở `--global`, và repository có `include.path` trỏ tới file nhóm có `pull.ff only`. Pull của Lan dùng rebase hay chỉ fast-forward?

<details><summary><b>Lời giải</b></summary><p>Rebase. Hai khoá khác nhau cùng có hiệu lực: <code>pull.rebase true</code> làm pull dùng rebase, và rebase không tạo merge commit, nên <code>pull.ff only</code> không chặn gì. Kiểm tra bằng <code>git config --show-origin --get-regexp "^pull\."</code>.</p></details>

**Bài 3.** Vì sao Git không có cơ chế tự áp dụng `.gitconfig` nằm trong repository như `.gitattributes`?

<details><summary><b>Lời giải</b></summary><p>Cấu hình có thể khiến Git chạy chương trình tuỳ ý (<code>core.sshCommand</code>, <code>core.fsmonitor</code>, alias shell, filter). Tự áp dụng thì chỉ clone một repository lạ là đã chạy mã của người khác. <code>.gitattributes</code> chỉ chọn trong những bộ lọc bạn đã cấu hình. Hook cũng vậy (GIT-049).</p></details>

## 9. Đọc tiếp

- [git-config: includes, conditional includes](https://git-scm.com/docs/git-config#_includes), [Pro Git: Git Configuration](https://git-scm.com/book/en/v2/Customizing-Git-Git-Configuration).

Bài tiếp theo: [GIT-059 · Sự cố Git thường gặp: index.lock, detached HEAD, push nhầm main và các thông báo lỗi khó hiểu](/articles/git-su-co-thuong-gap).
