---
slug: git-hook-pre-commit
title: "Hook: chặn commit sai trước khi rời máy với pre-commit, commit-msg, pre-push"
summary: "Vụ sai cước (GIT-041) và dòng in debug (GIT-036) đều có thể bị chặn ngay trên máy người viết. Bài này giải thích hook phía máy, viết ba hook bằng sh cho nhóm Tới Nơi (pre-commit chặn Console.WriteLine trong thay đổi đã stage, commit-msg chặn tiêu đề WIP hay quá ngắn, pre-push kiểm tra quy tắc làm tròn rồi gọi git lfs pre-push), chia sẻ qua thư mục .githooks với core.hooksPath và quyền 100755, khi nào --no-verify, và việc hook vẫn chạy khi commit, push từ GitStudio."
category: git
level: advanced
tags: [git, git-049, git-hooks, pre-commit, commit-msg, pre-push, gitstudio, toi-noi]
language: vi
---

Hai lỗi của tuần này có chung một điểm: chúng **rời khỏi máy** người viết. Dòng `Console.WriteLine` lọt vào nhánh tính năng (GIT-036), và phép làm tròn bị đổi lọt lên `main` (GIT-041). Review và CI bắt được, nhưng muộn. **Hook** của Git là script chạy ở những thời điểm cố định trên chính máy bạn: ngay trước khi commit, khi viết thông điệp, trước khi push. Hook thoát mã khác 0 là chặn thao tác.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-049 · Phần 5/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-chien-luoc-nhanh">GIT-027</a> (hook phía máy chủ pre-receive), <a href="/articles/git-lfs-file-lon">GIT-045</a>. Lab: <code>npm run p05 -- 49</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 49</code>. Sau bài này, bạn viết, chia sẻ được hook cho cả nhóm và biết giới hạn của chúng.</p>
</div>

## 1. Hook phía máy

| Hook | Chạy khi | Nhận | Chặn được |
|---|---|---|---|
| `pre-commit` | Trước khi tạo commit (chưa có thông điệp) | Không tham số; đọc staging area | Commit |
| `commit-msg` | Sau khi có thông điệp | `$1` = file chứa thông điệp (sửa được) | Commit |
| `pre-push` | Trước khi gửi lên remote | `$1` tên remote, `$2` URL, stdin các ref | Push |
| `post-checkout`, `post-merge`, `post-commit` | Sau thao tác | | Không (chỉ để thông báo, cập nhật) |

Hook nằm mặc định trong `.git/hooks` (có sẵn các file `.sample`). Thư mục `.git` **không** được commit, nên hook ở đó là của riêng máy bạn. LFS đã đặt vài hook ở đây (GIT-045): `post-checkout`, `post-commit`, `post-merge`, `pre-push`.

Trên Windows, Git for Windows chạy hook bằng `sh` đi kèm (dòng `#!/bin/sh` đầu file), nên viết hook bằng sh chạy được trên mọi máy của nhóm.

## 2. Ba hook của nhóm Tới Nơi

**pre-commit** chặn dòng debug trong **phần thay đổi đã stage** (không phải trong file trên đĩa: thứ được commit là staging area):

```sh
#!/bin/sh
# Chặn commit còn dòng in debug (Console.WriteLine) trong phần thay đổi C# đã stage.
if git diff --cached -U0 -- '*.cs' | grep -q '^+.*Console\.WriteLine'; then
  echo "pre-commit: còn Console.WriteLine trong thay đổi .cs đã stage. Bỏ dòng debug rồi commit lại." >&2
  exit 1
fi
exit 0
```

**commit-msg** đòi tiêu đề không bắt đầu bằng WIP và dài ít nhất 10 byte (`head -n 1 "$1"` lấy tiêu đề, `case` bắt `WIP*`, `${#subject}` đếm độ dài; file đầy đủ trong lab ở `lib/history5.mjs`).

**pre-push** kiểm tra quy tắc làm tròn (thực tế: chạy `dotnet test`), rồi **chuyển cho Git LFS**:

```sh
#!/bin/sh
if ! grep -q 'Math.Ceiling(atLeastMinimum' src/BaoGia/FareCalculator.cs; then
  echo "pre-push: FareCalculator không còn làm tròn lên bội số 1.000 đ." >&2
  exit 1
fi
command -v git-lfs >/dev/null 2>&1 || exit 0
exec git lfs pre-push "$@"
```

Dòng cuối quan trọng: khi nhóm chuyển hook sang thư mục riêng (mục 3), hook `pre-push` của LFS trong `.git/hooks` không còn chạy; nếu hook chung không gọi `git lfs pre-push`, file LFS không được đẩy lên và người khác nhận con trỏ trỏ vào hư không.

## 3. Chia sẻ hook: thư mục .githooks và core.hooksPath

```sh
git add .githooks
git update-index --chmod=+x .githooks/pre-commit .githooks/commit-msg .githooks/pre-push
git ls-files -s .githooks
git commit -q -m "Thêm hook chung của nhóm"
git config core.hooksPath .githooks
```

```text
100755 ef66397a33d165330a97c36c1a2cfe276ec1072a 0	.githooks/commit-msg
100755 f5c3dbaeb5c7840b867547f999f70206e7a32307 0	.githooks/pre-commit
100755 e616ec5596a21186c0fe689565c3e197553ed08c 0	.githooks/pre-push
```

Hook nằm trong repository, review và cập nhật như code. Mỗi người chạy **một lần** `git config core.hooksPath .githooks`: Git không tự bật hook từ repository vì lý do an toàn (clone một repository không được tự chạy script của người khác). `update-index --chmod=+x` ghi mode `100755` vào index để trên Linux, macOS hook có quyền chạy.

Nhóm .NET hay JavaScript thường dùng công cụ như Husky.Net, Husky, pre-commit (Python) hay lefthook để cài hook; bên dưới đều là cơ chế này.

## 4. Hook làm việc

```sh
git add src/BaoGia/NightSurcharge.cs
git commit -m "Ghi log khi tính phụ phí đêm"
git commit --allow-empty -m "WIP phụ phí"
git commit --allow-empty -m "sửa"
```

```text
pre-commit: còn Console.WriteLine trong thay đổi .cs đã stage. Bỏ dòng debug rồi commit lại.
commit-msg: không commit WIP; viết tiêu đề nói rõ thay đổi.
commit-msg: tiêu đề quá ngắn, cần ít nhất 10 ký tự.
```

Không commit nào được tạo. Push bình thường đi qua (hook kiểm tra xong im lặng):

```text
To <lab>/work/remote/bao-gia.git
   1f0a787..21b047b  main -> main
```

`--no-verify` (`-n`) bỏ qua `pre-commit` và `commit-msg`; với push, bỏ qua `pre-push`. Lab dùng nó để tạo một commit "WIP" rồi bỏ ngay bằng reset. Dùng khi biết rõ vì sao (hook hỏng, commit tạm trên nhánh riêng); nếu phải dùng thường xuyên thì hook đang sai.

## 5. Giới hạn

- Hook phía máy là **tiện ích**, không phải **kiểm soát**: ai cũng `--no-verify` hay bỏ `core.hooksPath` được. Luật bắt buộc đặt ở máy chủ (pre-receive, branch protection, GIT-027) và CI (Phần 6).
- Hook chậm làm mọi người khó chịu và tìm cách tắt. `pre-commit` nên chạy trong vài giây (lint, kiểm tra nhanh); test đầy đủ để ở `pre-push` hoặc CI.
- Hook chạy với thư mục hiện tại là gốc repository, môi trường của người gọi. Trên máy CI hay GitStudio, PATH có thể khác terminal của bạn.

## 6. Trên GitStudio

GitStudio gọi `git` trong PATH, nên **mọi hook đều chạy** khi bạn commit, push, merge, checkout từ giao diện, kể cả hook trong `core.hooksPath`. Hook chặn thì thao tác thất bại và GitStudio hiện thông báo lỗi kèm những gì hook in ra stderr; Nhật ký hoạt động giữ đủ đầu ra. Nội dung commit vẫn nằm nguyên trong ô soạn thảo để bạn sửa và thử lại.

GitStudio không có nút tương đương `--no-verify` và không có trình quản lý hook: tạo, sửa file trong `.githooks` như file thường, đặt `core.hooksPath` bằng terminal tích hợp.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-hook-pre-commit/gitstudio-01.webp" alt="GitStudio báo commit thất bại: thông báo lỗi chứa dòng pre-commit: còn Console.WriteLine trong thay đổi .cs đã stage; panel thay đổi vẫn còn NightSurcharge.cs đã stage và nội dung commit" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · hook pre-commit chặn commit từ giao diện</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-hook-pre-commit/gitstudio-02.webp" alt="Commit Thêm hook chung của nhóm trong GitStudio: ba file .githooks/commit-msg, pre-commit, pre-push được thêm, diff của pre-commit" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · hook chung là file trong repository, review như code</figcaption></figure>

## 7. Đối chiếu

| Việc | GitStudio | Lệnh |
|---|---|---|
| Xem hook đang dùng | | `git config core.hooksPath`, thư mục `.git/hooks` |
| Bật hook chung | (terminal) | `git config core.hooksPath .githooks` |
| Cho hook quyền chạy | | `git update-index --chmod=+x <file>` |
| Bỏ qua hook một lần | (không có) | `git commit --no-verify`, `git push --no-verify` |
| Xem hook in gì | Thông báo lỗi; Nhật ký hoạt động | đầu ra của lệnh |

## 8. Lỗi thường gặp

- **Hook kiểm tra file trên đĩa thay vì staging area**: chặn nhầm (file còn dòng debug chưa stage) hoặc bỏ lọt.
- **Hook không chạy trên máy đồng nghiệp**: chưa đặt `core.hooksPath`, thiếu quyền chạy (Linux, macOS), hoặc file có xuống dòng CRLF làm `#!/bin/sh` hỏng; thêm `.githooks/* text eol=lf` vào `.gitattributes`.
- **Đặt `core.hooksPath` rồi LFS không đẩy file**: hook chung thiếu `git lfs pre-push`.
- **Hook gọi lệnh chỉ có trong terminal của bạn** (alias, PATH riêng): chạy từ GitStudio hay CI thì lỗi.

## 9. Bài tập

**Bài 1.** Viết `prepare-commit-msg` thêm tên nhánh (ví dụ `[phu-phi-dem]`) vào đầu thông điệp.

<details><summary><b>Lời giải</b></summary><p><code>b=$(git rev-parse --abbrev-ref HEAD | sed 's#.*/##')</code>, rồi <code>sed -i "1s/^/[$b] /" "$1"</code>. Bỏ qua khi <code>$2</code> là <code>merge</code> hay <code>squash</code>.</p></details>

**Bài 2.** Làm sao để hook không hỏng khi đồng nghiệp dùng Windows và trình soạn thảo lưu CRLF?

<details><summary><b>Lời giải</b></summary><p>Thêm <code>.githooks/* text eol=lf</code> vào <code>.gitattributes</code> (GIT-043) để file luôn checkout với LF.</p></details>

**Bài 3.** Vì sao Git không tự bật hook từ repository khi clone?

<details><summary><b>Lời giải</b></summary><p>Hook là mã chạy trên máy bạn với quyền của bạn. Nếu clone tự bật hook, chỉ cần clone một repository độc hại là bị chạy mã tuỳ ý. Bật hook phải là hành động có chủ đích (<code>core.hooksPath</code>).</p></details>

## 10. Đọc tiếp

- [githooks](https://git-scm.com/docs/githooks), [Pro Git, Git Hooks](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks).

Bài tiếp theo: [GIT-050 · Thực hành: điều tra lỗi cước và bảo trì repository](/articles/git-thuc-hanh-dieu-tra-bao-tri).
