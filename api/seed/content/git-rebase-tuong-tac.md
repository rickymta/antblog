---
slug: git-rebase-tuong-tac
title: "Rebase tương tác: squash, fixup, reword, drop và --autosquash để dọn nhánh trước pull request"
summary: "Sau GIT-036, nhánh phụ phí đêm còn ba commit, trong đó một commit sửa giờ đáng lẽ thuộc về commit đầu. Bài này giải thích todo của git rebase -i và sáu lệnh pick, reword, edit, squash, fixup, drop; dùng commit --fixup để ghi một sửa đổi nhắm sẵn vào commit test, rồi rebase -i --autosquash main gộp mọi thứ thành hai commit gọn; cuối cùng là hộp thoại Rebase tương tác từ đây… của GitStudio với các lựa chọn Giữ, Sửa nội dung, Squash, Fixup, Bỏ."
category: git
level: intermediate
tags: [git, git-037, interactive-rebase, squash, fixup, autosquash, gitstudio, toi-noi]
language: vi
---

Sáng 25/11/2026, Minh chuẩn bị mở pull request cho phụ phí đêm. Nhánh đã gọn hơn hôm qua, nhưng người review vẫn sẽ đọc một commit thêm tính năng có lỗi giờ kết thúc, rồi một commit sửa lỗi đó. Lan cũng nhắc: thiếu test cho đúng 22:00. Mục tiêu: **hai** commit, mỗi commit một ý trọn vẹn: code (đã đúng) và test (đủ ca).

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-037 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-sua-commit-message-drop-thu-tu">GIT-036</a>. Lab: <code>npm run p04 -- 37</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 37</code>. Sau bài này, bạn dọn được một nhánh tính năng bằng rebase tương tác, dùng commit fixup! để sửa "về sau" mà vẫn gọn, và biết khi nào không nên làm vậy.</p>
</div>

## 1. Todo và sáu lệnh

`git rebase -i <gốc>` liệt kê các commit từ sau `<gốc>` tới HEAD, **cũ nhất ở trên**, mỗi dòng một lệnh. Git làm lần lượt từ trên xuống:

| Lệnh | Làm gì |
|---|---|
| `pick` | Giữ nguyên commit |
| `reword` | Giữ thay đổi, dừng lại để sửa thông điệp |
| `edit` | Dừng **sau** commit này để bạn amend, tách commit, chạy test…; `git rebase --continue` để đi tiếp |
| `squash` | Gộp vào commit **phía trên**, nối hai thông điệp (mở trình soạn thảo) |
| `fixup` | Gộp vào commit phía trên, **bỏ** thông điệp của commit này |
| `drop` (hoặc xoá dòng) | Bỏ commit |

Đổi thứ tự dòng là đổi thứ tự commit (GIT-036). Xoá hết các dòng là huỷ rebase. Giữa chừng có thể `git rebase --abort` để quay về nguyên trạng.

## 2. Ghi sửa đổi nhắm sẵn vào commit test

```sh
git log --oneline main..HEAD
git commit -a --fixup HEAD
git log --oneline main..HEAD
```

```text
c682a26 Thêm test phụ phí đêm
ec0ae3a Sửa giờ kết thúc phụ phí đêm thành 5h
ba63c45 Thêm phụ phí đêm
[tinh-nang/phu-phi-dem b25820f] fixup! Thêm test phụ phí đêm
 1 file changed, 4 insertions(+)
b25820f fixup! Thêm test phụ phí đêm
c682a26 Thêm test phụ phí đêm
ec0ae3a Sửa giờ kết thúc phụ phí đêm thành 5h
ba63c45 Thêm phụ phí đêm
```

`--fixup <commit>` tạo commit có tiêu đề `fixup! <tiêu đề commit đích>`. Ở đây đích là HEAD (commit test), nhưng có thể là bất kỳ commit nào: `git commit --fixup ba63c45`. Trong lúc review, cách này rất tiện: mỗi sửa đổi theo nhận xét là một commit fixup! riêng, người review thấy rõ đã sửa gì; trước khi merge mới gộp.

Các biến thể: `--fixup=amend:<commit>` (gộp và thay thông điệp, chính là commit `amend!` của GIT-036), `--fixup=reword:<commit>` (chỉ thay thông điệp), `--squash <commit>` (gộp, giữ cả hai thông điệp).

## 3. `rebase -i --autosquash`

```sh
git rebase -i --autosquash main
```

```text
pick ba63c45 # Thêm phụ phí đêm
pick ec0ae3a # Sửa giờ kết thúc phụ phí đêm thành 5h
pick c682a26 # Thêm test phụ phí đêm
fixup b25820f # fixup! Thêm test phụ phí đêm
Successfully rebased and updated refs/heads/tinh-nang/phu-phi-dem.
```

Todo Git đề xuất (in ra ở trên): `--autosquash` đã tự chuyển commit `fixup!` xuống ngay dưới commit test và đổi lệnh thành `fixup`. Minh sửa thêm một dòng trước khi lưu: `pick ec0ae3a` thành `fixup ec0ae3a`, để commit sửa giờ gộp vào commit đầu. Kết quả:

```sh
git log --format="%h %ai %s" main..HEAD
git show --stat --format="%h %s" HEAD~1
```

```text
40d9795 2026-11-24 14:40:00 +0700 Thêm test phụ phí đêm
3158035 2026-11-24 14:00:00 +0700 Thêm phụ phí đêm
3158035 Thêm phụ phí đêm

 docs/phi-cho.md              |  2 +-
 src/BaoGia/NightSurcharge.cs | 17 +++++++++++++++++
 2 files changed, 18 insertions(+), 1 deletion(-)
```

Hai commit, giữ **ngày viết** của commit gốc (14:00, 14:40 hôm trước); ngày commit là lúc rebase. `git diff --stat main` cho thấy tổng thay đổi không đổi: rebase tương tác sắp xếp lại lịch sử, không đổi kết quả cuối.

Muốn `--autosquash` luôn bật: `git config --global rebase.autoSquash true`. Thêm `rebase.autoStash true` để không phải dọn working tree trước.

## 4. Trên GitStudio

Chuột phải commit cũ nhất muốn sửa (ở đây `Thêm phụ phí đêm`) → **Rebase tương tác từ đây…**. Hộp thoại **Rebase tương tác** liệt kê commit từ đó tới đỉnh nhánh, commit cũ nhất ở trên, mỗi dòng có:

- Ô chọn thao tác: **Giữ** (pick), **Sửa nội dung** (reword, gõ nội dung mới ngay trong hộp thoại), **Squash**, **Fixup**, **Bỏ** (drop). Không có `edit`: muốn dừng giữa chừng thì dùng terminal.
- Nút **Lên trên** / **Xuống dưới** để đổi thứ tự.
- Ghi chú: "Squash và fixup gộp commit vào commit phía trên; squash giữ cả hai nội dung, fixup chỉ giữ nội dung commit phía trên." và "N commit; thay đổi chưa commit được stash rồi khôi phục lại."

**Bắt đầu rebase** chạy `git rebase -i --autostash` với todo GitStudio tự viết (thông điệp reword được áp bằng `exec git commit --amend` sau commit tương ứng). Chọn nhiều commit kề nhau rồi chuột phải còn có **Squash N commit**. Cả hộp thoại là một bước trong nhật ký Hoàn tác.

GitStudio không tự xếp các commit `fixup!` như `--autosquash`; bạn chọn Fixup và kéo dòng tới đúng chỗ bằng tay, hoặc chạy `git rebase -i --autosquash main` trong terminal tích hợp.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-rebase-tuong-tac/gitstudio-01.webp" alt="Hộp thoại Rebase tương tác của GitStudio với bốn commit của nhánh phụ phí đêm: Thêm phụ phí đêm chọn Giữ, Sửa giờ kết thúc phụ phí đêm thành 5h chọn Fixup, Thêm test phụ phí đêm chọn Giữ, fixup! Thêm test phụ phí đêm chọn Fixup; nút Lên trên, Xuống dưới ở mỗi dòng và nút Bắt đầu rebase" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Rebase tương tác từ đây…: chọn thao tác cho từng commit</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-rebase-tuong-tac/gitstudio-02.webp" alt="Đồ thị GitStudio sau rebase tương tác: nhánh tinh-nang/phu-phi-dem chỉ còn hai commit Thêm phụ phí đêm và Thêm test phụ phí đêm phía trên main" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · nhánh còn hai commit, sẵn sàng cho pull request</figcaption></figure>

## 5. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Mở rebase tương tác | Chuột phải commit → Rebase tương tác từ đây… | `git rebase -i <commit>~1` |
| Gộp, giữ cả hai thông điệp | Squash | `squash` |
| Gộp, bỏ thông điệp | Fixup | `fixup` |
| Sửa thông điệp | Sửa nội dung | `reword` |
| Dừng để sửa giữa chừng | (terminal) | `edit`, rồi `git rebase --continue` |
| Ghi sửa đổi cho commit cũ | | `git commit --fixup <commit>` |
| Tự xếp fixup! | | `git rebase -i --autosquash <gốc>` |
| Gộp vài commit kề nhau | Chọn nhiều → Squash N commit | `squash`/`fixup` |

## 6. Lỗi thường gặp

- **Rebase tương tác trên nhánh đã có người dựa vào**: như mọi viết lại lịch sử (GIT-036 mục 1).
- **Squash commit đầu tiên của danh sách**: không có commit phía trên để gộp vào, Git báo lỗi ("cannot 'squash' without a previous commit"). GitStudio chặn trước: "Commit đầu tiên được giữ không có commit nào phía trước để gộp vào."
- **Gộp tất cả thành một commit khổng lồ**: mất khả năng review và revert từng phần. Mỗi commit nên là một bước có nghĩa (code; test; tài liệu…).
- **Quên chạy test sau khi dọn**: thứ tự mới có thể làm một commit giữa chừng không build được, gây khó cho `git bisect` (Phần 5).
- **`--autosquash` không bắt được fixup!** khi tiêu đề commit đích đã bị đổi sau đó; dùng mã commit (`--fixup <mã>`) khi tạo, Git ghi tiêu đề tại thời điểm đó.

## 7. Bài tập

**Bài 1.** Từ trạng thái đầu bài, viết todo để có **một** commit duy nhất với thông điệp "Thêm phụ phí đêm 22h–5h kèm test".

<details><summary><b>Lời giải</b></summary><p><code>reword ba63c45</code>, <code>fixup ec0ae3a</code>, <code>fixup c682a26</code>, <code>fixup b25820f</code>. Git dừng ở reword để sửa thông điệp (hoặc dùng <code>squash</code> cho dòng thứ hai và gộp thông điệp trong trình soạn thảo).</p></details>

**Bài 2.** Vì sao lab chọn `fixup` chứ không `squash` cho commit sửa giờ?

<details><summary><b>Lời giải</b></summary><p>Thông điệp "Sửa giờ kết thúc phụ phí đêm thành 5h" chỉ có ý nghĩa với lịch sử nháp; sau khi gộp, commit đầu đã đúng ngay từ đầu, nên không cần nhắc lỗi đã sửa.</p></details>

**Bài 3.** Rebase tương tác dừng giữa chừng vì xung đột và bạn không chắc nữa. Lệnh nào đưa nhánh về đúng như trước khi bắt đầu?

<details><summary><b>Lời giải</b></summary><p><code>git rebase --abort</code> (GitStudio: banner Đang rebase → Huỷ thao tác). Nếu đã xong mà mới hối hận: <code>git reset --hard ORIG_HEAD</code> hoặc <code>nhánh@{1}</code> (GIT-034).</p></details>

## 8. Đọc tiếp

- [git-rebase: Interactive mode, `--autosquash`](https://git-scm.com/docs/git-rebase), [git-commit: `--fixup`, `--squash`](https://git-scm.com/docs/git-commit).
- [Pro Git, Rewriting History: Squashing Commits](https://git-scm.com/book/en/v2/Git-Tools-Rewriting-History).

Bài tiếp theo: [GIT-038 · Rebase --onto và tách nhánh lỡ tạo sai gốc](/articles/git-rebase-onto).
