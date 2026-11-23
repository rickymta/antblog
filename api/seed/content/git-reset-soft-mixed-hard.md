---
slug: git-reset-soft-mixed-hard
title: "Reset soft, mixed, hard: lùi nhánh và chọn đúng chỗ để thay đổi nằm lại"
summary: "git reset dời nhánh hiện tại tới một commit khác; ba chế độ chỉ khác nhau ở chỗ thay đổi của các commit bị bỏ đi đâu: --soft giữ trong staging area, --mixed giữ trong working tree, --hard bỏ luôn. Bài này chạy cả ba trên một nhánh thử đã push, đọc git status -sb sau mỗi bước, cho thấy vì sao push bị từ chối sau khi reset nhánh đã chia sẻ, và menu Reset về commit này (Soft, Mixed, Hard) của GitStudio."
category: git
level: beginner
tags: [git, git-032, git-reset, soft, mixed, hard, gitstudio, toi-noi]
language: vi
---

GIT-031 hoàn tác thứ chưa commit. Lần này Minh đã commit, thậm chí đã push, rồi mới muốn làm lại: gộp hai commit nhỏ, viết lại commit cuối, hay bỏ hẳn một thử nghiệm. Công cụ là `git reset`, lệnh mạnh nhất và dễ dùng sai nhất của Git.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-032 · Phần 4/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-restore-bo-thay-doi">GIT-031</a>, <a href="/articles/git-ben-trong-object-ref-head">GIT-008</a> (nhánh là con trỏ). Lab: <code>npm run p04 -- 32</code>. Sau bài này, bạn chọn đúng chế độ reset, biết chính xác thay đổi nằm lại ở đâu, và biết vì sao không reset nhánh đã chia sẻ.</p>
</div>

## 1. Reset làm gì

Nhánh là một con trỏ tới commit (GIT-011). `git reset <commit>` **dời con trỏ của nhánh hiện tại** tới commit đó. Các commit phía sau không bị xoá ngay (reflog còn giữ, GIT-034), chỉ không còn thuộc nhánh. Câu hỏi duy nhất là: thay đổi của những commit đó **nằm lại ở đâu**?

| Chế độ | HEAD và nhánh | Staging area | Working tree | Dùng khi |
|---|---|---|---|---|
| `--soft` | Lùi | Giữ: thay đổi bị bỏ nằm sẵn ở đây | Giữ | Gộp vài commit cuối thành một, viết lại commit |
| `--mixed` (mặc định) | Lùi | Đưa về commit đích | Giữ: thay đổi nằm ở đây, chưa stage | Chia lại commit, chọn lại file nào vào commit nào |
| `--hard` | Lùi | Đưa về commit đích | **Đưa về commit đích: mất thay đổi** | Bỏ hẳn thử nghiệm |

## 2. Thử trên nhánh `thu/reset`

Lab tạo nhánh thử với ba commit, mỗi commit thêm một dòng vào `docs/thu-reset.md`, rồi push:

```text
a08674d Thử reset: dòng 3
d1cbbdf Thử reset: dòng 2
1ab8585 Thử reset: dòng 1
e93038f Ghi múi giờ áp dụng bảng giá
```

### `--soft`

```sh
git reset --soft HEAD~1
git status -sb
git log --oneline -1
```

```text
## thu/reset...origin/thu/reset [behind 1]
M  docs/thu-reset.md
d1cbbdf Thử reset: dòng 2
```

Nhánh lùi về "dòng 2"; dòng 3 nằm sẵn trong staging area (`M` cột trái), chỉ cần `git commit` là có lại commit, với nội dung commit mới. `[behind 1]`: remote vẫn còn commit "dòng 3" mà nhánh cục bộ không còn.

### `--mixed`

Minh commit lại ("dòng 3 (làm lại)") rồi reset không ghi chế độ, tức `--mixed`:

```sh
git reset HEAD~1
git status -sb
```

```text
Unstaged changes after reset:
M	docs/thu-reset.md
## thu/reset...origin/thu/reset [behind 1]
 M docs/thu-reset.md
```

Thay đổi vẫn trong file, nhưng chưa stage (`M` cột phải). Đây là cách tách một commit lớn thành nhiều commit nhỏ: reset mixed, rồi `git add -p` từng phần (GIT-005).

### `--hard`

```sh
git reset --hard HEAD
git status -sb
git show HEAD:docs/thu-reset.md
```

```text
HEAD is now at d1cbbdf Thử reset: dòng 2
## thu/reset...origin/thu/reset [behind 1]
Dòng 1
Dòng 2
```

`reset --hard HEAD` (không lùi) là cách nhanh nhất để bỏ mọi thay đổi chưa commit của file đã track. Thay đổi chưa commit bị `--hard` xoá thì **không** có trong reflog, mất hẳn.

## 3. Reset nhánh đã push

```sh
git push
```

```text
 ! [rejected]        thu/reset -> thu/reset (non-fast-forward)
error: failed to push some refs to '<lab>/work/remote/bao-gia.git'
hint: Updates were rejected because the tip of your current branch is behind
hint: its remote counterpart. …
```

Nhánh cục bộ đã lùi, remote vẫn ở "dòng 3": push lúc này không phải fast-forward (GIT-023). Muốn remote lùi theo thì phải force push, nghĩa là **viết lại lịch sử mà người khác có thể đã kéo về**. Quy tắc:

- Commit **chưa push**: reset thoải mái.
- Commit **đã push lên nhánh của riêng bạn**: reset rồi `push --force-with-lease` được (GIT-023), nếu chưa ai dựa vào nhánh.
- Commit **đã push lên `main` hay nhánh chung**: không reset. Dùng **revert** (GIT-033).

## 4. Trên GitStudio

Chuột phải một commit trên đồ thị → **Reset main về commit này** (tên nhánh hiện tại thay cho `main`) → chọn **Soft - giữ toàn bộ thay đổi**, **Mixed - giữ working copy, reset index**, hoặc **Hard - bỏ toàn bộ thay đổi**. Chọn Hard thì GitStudio hỏi thêm: "Các thay đổi chưa commit sẽ bị mất. Tiếp tục?".

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-reset-soft-mixed-hard/gitstudio-01.webp" alt="Menu chuột phải của commit Thử reset: dòng 2 trên đồ thị GitStudio, mục Reset thu/reset về commit này mở ra ba lựa chọn Soft - giữ toàn bộ thay đổi, Mixed - giữ working copy, reset index, Hard - bỏ toàn bộ thay đổi" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Reset về commit này: ba chế độ</figcaption></figure>

Reset trong GitStudio có trong nhật ký **Hoàn tác** (GIT-035): Ctrl+Z đưa nhánh về đỉnh cũ. Nhưng với Hard, Hoàn tác chỉ trả lại các **commit**; thay đổi chưa commit bị xoá thì không lấy lại được.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-reset-soft-mixed-hard/gitstudio-02.webp" alt="Đồ thị GitStudio sau khi reset soft nhánh thu/reset: nhãn thu/reset ở commit Thử reset: dòng 2, nhãn origin/thu/reset ở commit dòng 3 phía trên; panel bên phải có docs/thu-reset.md trong Tệp đã stage" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · sau reset soft: nhánh lùi một commit, thay đổi nằm trong Tệp đã stage</figcaption></figure>

## 5. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Lùi nhánh, giữ thay đổi đã stage | Reset … về commit này → Soft | `git reset --soft <commit>` |
| Lùi nhánh, thay đổi về working tree | → Mixed | `git reset <commit>` |
| Lùi nhánh, bỏ thay đổi | → Hard (hỏi xác nhận) | `git reset --hard <commit>` |
| Đưa nhánh cục bộ về đúng remote | Reset về commit của `origin/…` | `git reset --hard origin/<nhánh>` |
| Hoàn tác vừa reset | Hoàn tác (Ctrl+Z) | `git reset --hard ORIG_HEAD` hoặc reflog |

## 6. Lỗi thường gặp

- **`reset --hard` khi còn thay đổi chưa commit ở file khác**: mất luôn. Chạy `git status` trước; nếu có thay đổi cần giữ thì stash, hoặc dùng `reset --keep` (từ chối nếu sắp đè thay đổi).
- **Reset `main` đã push rồi force push**: đồng nghiệp pull về sẽ rối. Revert.
- **Nhầm `HEAD~1` với `HEAD^1`**: với commit thường giống nhau; với merge commit `^2` là cha thứ hai, `~2` là ông.
- **Reset để "bỏ stage" cả repository** (`git reset`) rồi tưởng mất thay đổi: không mất, chỉ hết stage.

## 7. Bài tập

**Bài 1.** Gộp ba commit cuối của nhánh (chưa push) thành một commit, bằng reset.

<details><summary><b>Lời giải</b></summary><p><code>git reset --soft HEAD~3</code> rồi <code>git commit -m "…"</code>. Mọi thay đổi của ba commit nằm sẵn trong staging area. GIT-037 làm việc này linh hoạt hơn bằng rebase tương tác.</p></details>

**Bài 2.** Sau `git reset --hard HEAD~1` trên nhánh chưa push, làm sao lấy lại commit vừa bỏ?

<details><summary><b>Lời giải</b></summary><p><code>git reset --hard ORIG_HEAD</code> ngay sau đó, hoặc tìm trong <code>git reflog</code> rồi <code>git reset --hard HEAD@{1}</code> (GIT-034).</p></details>

**Bài 3.** Đồng nghiệp vừa force push lên nhánh chung, nhánh cục bộ của bạn `[ahead 3, behind 2]` và bạn không có commit riêng nào. Đưa nhánh về đúng remote.

<details><summary><b>Lời giải</b></summary><p><code>git fetch</code> rồi <code>git reset --hard origin/&lt;nhánh&gt;</code>. Ba commit "ahead" là các commit cũ đã bị viết lại trên remote.</p></details>

## 8. Đọc tiếp

- [git-reset](https://git-scm.com/docs/git-reset), [Pro Git, Reset Demystified](https://git-scm.com/book/en/v2/Git-Tools-Reset-Demystified).

Bài tiếp theo: [GIT-033 · Revert: hoàn tác commit đã chia sẻ](/articles/git-revert-commit-da-chia-se).
