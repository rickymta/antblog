---
slug: git-conventional-commits-changelog
title: "Conventional Commits và CHANGELOG: tiêu đề commit có cấu trúc để sinh nhật ký thay đổi"
summary: "Bộ phận vận hành hỏi bản 1.3.0 có gì mới, và câu trả lời nằm rải rác trong 40 tiêu đề commit viết tự do. Bài này áp dụng Conventional Commits cho nhóm Tới Nơi (feat, fix, docs, chore…, phạm vi, dấu ! cho thay đổi phá vỡ), dùng hook commit-msg để kiểm tra, lọc lịch sử theo loại commit, rồi viết script sinh mục CHANGELOG từ git log. Trên GitStudio: ô Tóm tắt commit với bộ đếm 72 ký tự, hook chạy cả khi commit từ giao diện."
category: git
level: beginner
tags: [git, git-051, conventional-commits, changelog, git-hooks, gitstudio, toi-noi]
language: vi
---

Thứ Hai 30/11/2026, chị Hà bên vận hành nhắn: "Bản 1.3.0 tuần này có gì khách thấy được không? Mình cần ghi vào thông báo cho tài xế." Minh mở `git log` và thấy những tiêu đề như "Thêm hook chung của nhóm", "Cập nhật thư viện tiền tệ: dạng ngắn". Cái nào là tính năng mới, cái nào là sửa lỗi, cái nào chỉ là việc nội bộ? Phải đọc từng diff mới biết. Phần 6 bắt đầu bằng việc làm cho tiêu đề commit tự trả lời câu hỏi đó.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-051 · Phần 6/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-commit-stage-hunk-amend">GIT-005</a> (viết commit), <a href="/articles/git-hook-pre-commit">GIT-049</a> (hook). Lab: <code>npm run p06 -- 51</code>; mở bằng GitStudio: <code>npm run setup -- 51</code>. Sau bài này, bạn viết được tiêu đề commit theo Conventional Commits, bắt cả nhóm theo bằng hook, và sinh mục CHANGELOG từ lịch sử.</p>
</div>

## 1. Quy ước: loại, phạm vi, mô tả

[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) là một quy ước viết tiêu đề commit theo khuôn:

```text
<loại>(<phạm vi>): <mô tả>

[thân commit, tuỳ chọn]

[chân commit, tuỳ chọn, ví dụ BREAKING CHANGE: …]
```

- **Loại** cho biết commit thuộc nhóm nào. `feat` là tính năng mới người dùng thấy, `fix` là sửa lỗi người dùng gặp. Những loại còn lại là việc nội bộ: `docs`, `test`, `refactor`, `style`, `perf`, `build`, `ci`, `chore`.
- **Phạm vi** (không bắt buộc) là phần của hệ thống bị đổi. Nhóm Tới Nơi dùng `cuoc`, `phi-cho`, `ban-do`, `tien-te`, `readme`, `release`, `hook`.
- **Dấu `!`** sau loại, hoặc đoạn `BREAKING CHANGE:` trong chân commit, đánh dấu thay đổi phá vỡ tương thích (GIT-052 dùng nó để quyết định số phiên bản).

Quy ước chỉ có giá trị khi cả nhóm cùng theo, nên Minh ghi nó vào `CONTRIBUTING.md`. Trong file có một bảng nói loại nào được đưa vào CHANGELOG:

| Loại | Dùng khi | Vào CHANGELOG |
|---|---|---|
| `feat` | Tính năng mới cho người dùng | Tính năng mới |
| `fix` | Sửa lỗi người dùng gặp | Sửa lỗi |
| `perf` | Nhanh hơn, không đổi hành vi | Hiệu năng |
| `docs`, `test`, `refactor`, `style` | Tài liệu, test, sắp xếp lại code, định dạng | Không |
| `build`, `ci`, `chore` | Script, CI, việc vặt (phát hành, cấu hình) | Không |

Quy ước này không thay câu chữ của bạn: mô tả vẫn là một câu ngắn nói rõ thay đổi (GIT-005), chỉ có thêm một tiền tố mà máy đọc được.

## 2. Hook commit-msg kiểm tra tiêu đề

Hook `commit-msg` ở GIT-049 chỉ chặn tiêu đề "WIP" và tiêu đề quá ngắn. Bản mới thay bằng một biểu thức chính quy. Nó bỏ qua tiêu đề do Git tự tạo (`Merge …`, `Revert "…"`) và commit `fixup!`, `squash!` của rebase (GIT-037):

```sh
#!/bin/sh
# Tiêu đề commit theo Conventional Commits: <loại>(<phạm vi>): <mô tả>.
# Merge và Revert do git tạo, fixup!/squash! của rebase --autosquash được bỏ qua.
subject=$(head -n 1 "$1")
case "$subject" in
  Merge\ *|Revert\ \"*|fixup!\ *|squash!\ *) exit 0 ;;
esac
if ! printf '%s\n' "$subject" | grep -Eq '^(feat|fix|perf|docs|test|refactor|style|build|ci|chore|revert)(\([a-z0-9-]+\))?!?: .{3,}'; then
  echo "commit-msg: tiêu đề phải có dạng <loại>(<phạm vi>): <mô tả>, ví dụ fix(phi-cho): báo lỗi khi số giây chờ âm" >&2
  echo "commit-msg: loại hợp lệ: feat fix perf docs test refactor style build ci chore revert" >&2
  exit 1
fi
exit 0
```

```sh
git add CONTRIBUTING.md .githooks/commit-msg
git commit -q -m "docs: quy ước commit theo Conventional Commits"
git commit --allow-empty -m "Sửa lỗi phí chờ"
```

```text
commit-msg: tiêu đề phải có dạng <loại>(<phạm vi>): <mô tả>, ví dụ fix(phi-cho): báo lỗi khi số giây chờ âm
commit-msg: loại hợp lệ: feat fix perf docs test refactor style build ci chore revert
```

Commit đầu tiên tự đi qua hook mới: `core.hooksPath` trỏ vào thư mục `.githooks` trong working tree, nên file vừa sửa có hiệu lực ngay. Ai chưa bật `core.hooksPath` thì hook không chạy. Vì vậy GIT-054 cho CI kiểm tra lại cùng quy tắc.

## 3. Một ngày commit theo quy ước

```sh
git log --oneline v1.2.0..main
```

```text
8415f73 docs(readme): ví dụ phụ phí ngày lễ
76636c1 fix(phi-cho): báo lỗi khi số giây chờ âm
1212722 test(cuoc): test phụ phí ngày lễ
096203a feat(cuoc): phụ phí ngày lễ 30%
e11d08e docs: quy ước commit theo Conventional Commits
8ea4486 Thêm báo cáo điều tra sai cước
21b047b Thêm hook chung của nhóm
```

Hai commit cuối có trước khi nhóm áp dụng quy ước; không cần sửa lại lịch sử cũ. Giờ lọc theo loại chỉ cần `--grep` với biểu thức chính quy mở rộng (`-E`, GIT-042):

```sh
git log --format=%s -E --grep="^feat(\(.*\))?!?:" v1.2.0..main
git log --format=%s -E --grep="^fix(\(.*\))?!?:" v1.2.0..main
```

```text
feat(cuoc): phụ phí ngày lễ 30%
fix(phi-cho): báo lỗi khi số giây chờ âm
```

Đó chính là câu trả lời cho chị Hà: một tính năng mới, một sửa lỗi. Những commit còn lại khách không thấy.

## 4. Sinh mục CHANGELOG bằng script

`CHANGELOG.md` là file người đọc được, mới nhất ở trên, mỗi phiên bản một mục. Không nên chép tay từ `git log`: dễ sót và mỗi người viết một kiểu. Script `scripts/changelog.sh` của nhóm làm việc đó trong 20 dòng:

```sh
#!/bin/sh
# Sinh một mục CHANGELOG từ tiêu đề commit theo Conventional Commits.
# Dùng: sh scripts/changelog.sh <phiên bản> <ngày> <tag trước>
ver=$1; day=$2; from=$3
log=$(git log --no-merges --reverse --format='%s' "$from..HEAD")
echo "## $ver ($day)"
for t in feat fix perf; do
  case $t in
    feat) title="Tính năng mới" ;;
    fix) title="Sửa lỗi" ;;
    perf) title="Hiệu năng" ;;
  esac
  lines=$(printf '%s\n' "$log" | grep -E "^$t(\(|!|:)" | sed -E 's/^[a-z]+\(([^)]*)\)!?: /- \1: /; s/^[a-z]+!?: /- /')
  if [ -n "$lines" ]; then
    printf '\n### %s\n%s\n' "$title" "$lines"
  fi
done
```

```sh
git commit -q -m "build: script sinh changelog từ tiêu đề commit"
sh scripts/changelog.sh 1.3.0 2026-11-30 v1.2.0
```

```text
## 1.3.0 (2026-11-30)

### Tính năng mới
- cuoc: phụ phí ngày lễ 30%

### Sửa lỗi
- phi-cho: báo lỗi khi số giây chờ âm
```

> **Trên Windows:** `sh` có sẵn trong Git for Windows nhưng chỉ Git Bash thấy nó; PowerShell báo `sh : The term 'sh' is not recognized`. Chạy script trong Git Bash (đặt terminal tích hợp là Git Bash ở Tuỳ chọn → Công cụ ngoài → Terminal, GIT-009), hoặc gọi thẳng: `& "$env:ProgramFilesGitinsh.exe" scripts/changelog.sh 1.3.0 2026-11-30 v1.2.0`. Các script chấm của lab (`npm run kiem-tra-p06`) tự tìm `sh.exe` này nên chạy được từ PowerShell.

Minh dán mục này dưới phần đầu của `CHANGELOG.md`, đọc lại câu chữ (script chỉ chép tiêu đề, người phát hành có thể viết rõ hơn cho khách), rồi commit:

```sh
git add CHANGELOG.md
git commit -q -m "chore(release): 1.3.0"
git push -q
```

`--no-merges` bỏ merge commit vì chúng không mô tả thay đổi. `--reverse` xếp cũ trước mới. `$from..HEAD` (GIT-042) là các commit kể từ tag trước. Nhóm lớn hơn thường dùng công cụ có sẵn như [git-cliff](https://git-cliff.org/), conventional-changelog hay release-please: cùng ý tưởng, thêm liên kết tới commit, pull request, và tự đề xuất số phiên bản.

## 5. Trên GitStudio

GitStudio không có bộ chọn loại commit hay mẫu tiêu đề. Ô **Tóm tắt commit** là chữ tự do, có **bộ đếm ký tự còn lại trước mốc khuyến nghị 72**. Bộ đếm chuyển đỏ khi vượt mốc nhưng không chặn commit. Ô **Mô tả** là thân commit; GitStudio ghép tóm tắt, một dòng trống và mô tả thành thông điệp đầy đủ.

Commit từ GitStudio chạy `git commit` bình thường, nên hook `commit-msg` của nhóm vẫn chạy. Tiêu đề sai quy ước thì commit thất bại, kèm thông báo lỗi do hook in ra, và nội dung vẫn còn nguyên trong ô soạn thảo để sửa lại. Tiền tố `feat(cuoc):` chiếm khoảng 10 trong 72 ký tự, nên mô tả cần ngắn gọn hơn.

`commit.template` của Git không có tác dụng trong GitStudio, vì GitStudio truyền thông điệp thẳng cho git chứ không mở trình soạn thảo. Muốn có gợi ý cho cả nhóm, ghi quy ước vào `CONTRIBUTING.md` và dựa vào hook.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-conventional-commits-changelog/gitstudio-01.webp" alt="Ô soạn commit của GitStudio với tóm tắt fix(phi-cho): báo lỗi khi số giây chờ âm, bộ đếm ký tự còn lại và file WaitingFee.cs đã stage" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · tiêu đề theo Conventional Commits trong ô Tóm tắt commit</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-conventional-commits-changelog/gitstudio-02.webp" alt="GitStudio báo Commit thất bại với thông báo của hook commit-msg: tiêu đề phải có dạng loại(phạm vi): mô tả; ô soạn thảo vẫn giữ tiêu đề Sửa lỗi phí chờ" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · hook commit-msg chặn tiêu đề kiểu cũ</figcaption></figure>

## 6. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Viết tiêu đề và thân commit | Tóm tắt commit, Mô tả | `git commit -m "<tiêu đề>" -m "<thân>"` |
| Kiểm tra tiêu đề | Hook chạy khi commit | `.githooks/commit-msg` |
| Lọc commit theo loại | | `git log -E --grep="^feat(\(.*\))?!?:"` |
| Sinh mục CHANGELOG | | `sh scripts/changelog.sh <bản> <ngày> <tag trước>` |
| Sửa tiêu đề commit cuối chưa push | Sửa nội dung trong panel commit | `git commit --amend` |

## 7. Lỗi thường gặp

- **Dùng `feat` cho mọi thứ**: CHANGELOG đầy việc nội bộ, phiên bản tăng MINOR vô lý. Câu hỏi để chọn loại: người dùng có thấy khác không?
- **Một commit vừa `feat` vừa `fix`**: tách làm hai commit (GIT-005 stage từng hunk).
- **Phạm vi mỗi người đặt một kiểu** (`cuoc`, `cước`, `fare`): ghi danh sách phạm vi vào CONTRIBUTING.md; có thể cho hook kiểm tra luôn.
- **Sửa lại tiêu đề của lịch sử đã push cho "đẹp"**: không đáng. Áp dụng từ một mốc, script CHANGELOG chỉ đọc từ tag gần nhất.
- **Hook chỉ chạy trên máy đã bật `core.hooksPath`**: CI phải kiểm tra lại (GIT-054), hoặc dùng quy tắc của nền tảng (GitHub rulesets kiểm tra pull request).

## 8. Bài tập

**Bài 1.** Chọn loại cho ba commit: (a) đổi tên biến `fee` thành `waitingFee` trong `WaitingFee.cs`; (b) sửa phí chờ bị tính khi khách huỷ; (c) nâng phiên bản .NET trong file dự án.

<details><summary><b>Lời giải</b></summary><p>(a) <code>refactor(phi-cho)</code>: không đổi hành vi. (b) <code>fix(phi-cho)</code>. (c) <code>build</code> hoặc <code>chore</code>. Nếu bản .NET mới buộc máy chủ phải cài lại runtime thì có thể là <code>build!</code> kèm <code>BREAKING CHANGE</code>.</p></details>

**Bài 2.** Viết lệnh liệt kê mọi commit có thay đổi phá vỡ (dấu `!` hoặc `BREAKING CHANGE` trong thân) kể từ `v1.2.0`.

<details><summary><b>Lời giải</b></summary><p><code>git log --oneline -E --grep="^[a-z]+(\(.*\))?!:" --grep="^BREAKING CHANGE:" v1.2.0..main</code>. Nhiều <code>--grep</code> mặc định là "hoặc" (GIT-042), đúng ý ở đây.</p></details>

**Bài 3.** Hook `commit-msg` hiện chấp nhận mọi phạm vi gồm chữ thường, số, gạch ngang. Sửa để chỉ chấp nhận phạm vi trong danh sách của nhóm.

<details><summary><b>Lời giải</b></summary><p>Thay <code>(\([a-z0-9-]+\))?</code> bằng <code>(\((cuoc|phi-cho|ban-do|tien-te|readme|release|hook)\))?</code>. Thêm phạm vi mới thì sửa hook trong cùng pull request để reviewer thấy.</p></details>

## 9. Đọc tiếp

- [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/), [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
- [git-cliff](https://git-cliff.org/docs/) để sinh changelog có cấu hình.

Bài tiếp theo: [GIT-052 · Phát hành theo semver: tag có chữ ký, nhánh phát hành và bản vá 1.3.1](/articles/git-phat-hanh-semver).
