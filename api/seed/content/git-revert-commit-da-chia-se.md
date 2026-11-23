---
slug: git-revert-commit-da-chia-se
title: "Revert: hoàn tác commit đã push mà không viết lại lịch sử chung"
summary: "Minh đã push mức phí chờ tối đa 50.000 đ lên main thì bộ phận vận hành báo chưa duyệt. Reset là sai vì main đã chia sẻ; bài này dùng git revert để tạo commit đảo ngược, đọc thông điệp This reverts commit, revert lại một revert (Reapply), và revert một merge commit bằng -m 1. Trên GitStudio là mục Revert commit trong menu commit (chạy --no-edit, tự thêm -m 1 với merge commit)."
category: git
level: beginner
tags: [git, git-033, git-revert, merge-commit, mainline, gitstudio, toi-noi]
language: vi
---

9 giờ sáng 24/11/2026, Minh tăng phí chờ tối đa từ 30.000 đ lên 50.000 đ theo một email của bộ phận kinh doanh, commit và push lên `main`. 9 giờ 25, bộ phận vận hành gọi: mức mới chưa được duyệt, app tài xế đã bắt đầu hiển thị 50.000 đ. Cần quay lại **ngay**, trên một nhánh mà Lan và hệ thống CI đã kéo về.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-033 · Phần 4/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-reset-soft-mixed-hard">GIT-032</a>, <a href="/articles/git-cherry-pick-revert">GIT-017</a> (revert lần đầu). Lab: <code>npm run p04 -- 33</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 33</code>. Sau bài này, bạn hoàn tác được commit thường và merge commit đã chia sẻ mà không phải force push.</p>
</div>

## 1. Revert khác reset thế nào

| | `git reset` | `git revert` |
|---|---|---|
| Lịch sử | Viết lại: nhánh lùi, commit bị bỏ khỏi nhánh | Giữ nguyên, **thêm** một commit đảo ngược |
| Push sau đó | Phải force push | Push bình thường (fast-forward) |
| Người khác đã pull | Bị rối | Pull về là có commit đảo ngược |
| Dấu vết | Không còn trên nhánh | Còn cả commit gốc lẫn commit revert, kèm lý do |
| Dùng cho | Commit chưa chia sẻ | Commit đã chia sẻ |

## 2. Revert commit vừa push

```sh
git commit -am "Tăng phí chờ tối đa lên 50.000 đ"
git push
git revert --no-edit HEAD
git log -2 --format="%h %s%n%b"
```

```text
[main 17b8c3c] Tăng phí chờ tối đa lên 50.000 đ
 1 file changed, 2 insertions(+), 2 deletions(-)
   e93038f..17b8c3c  main -> main
[main c19b370] Revert "Tăng phí chờ tối đa lên 50.000 đ"
 1 file changed, 2 insertions(+), 2 deletions(-)
c19b370 Revert "Tăng phí chờ tối đa lên 50.000 đ"
This reverts commit 17b8c3c3a02ecb3df3c72c6da63bbcfdfd93997d.

17b8c3c Tăng phí chờ tối đa lên 50.000 đ
```

Revert tính diff của commit đích rồi áp **ngược** lên HEAD, tạo commit mới với tiêu đề `Revert "…"` và thân "This reverts commit <mã>". `--no-edit` giữ nguyên thông điệp mặc định; bỏ nó thì Git mở trình soạn thảo, nên ghi thêm **lý do** ("Vận hành chưa duyệt mức 50.000 đ, xem ticket OPS-112"). Rồi `git push` như mọi commit khác:

```text
   17b8c3c..c19b370  main -> main
```

Revert một commit cũ hơn cũng được (`git revert <mã>`), miễn các commit sau không sửa đè cùng dòng; nếu có, revert dừng với xung đột, giải như GIT-014 rồi `git revert --continue` (hoặc `--abort`).

## 3. Revert của revert: Reapply

Khi vận hành duyệt mức mới, không cần commit lại bằng tay: revert chính commit revert.

```text
[main 65d94b6] Reapply "Tăng phí chờ tối đa lên 50.000 đ"
```

Git 2.43 trở đi đặt tên "Reapply" cho trường hợp này (bản cũ hơn ghi `Revert "Revert "…""`). Lab GIT-035 dùng đúng lệnh này.

## 4. Revert merge commit: `-m`

Merge commit có **hai** cha. "Đảo ngược merge" nghĩa là quay về cha nào? Git không đoán:

```sh
git revert --no-edit d263a4d
git revert --no-edit -m 1 d263a4d
```

```text
error: commit d263a4dc7c76e7fc2a4624bf796c9b2f76471bb6 is a merge but no -m option was given.
fatal: revert failed
[thu/revert-merge d032b05] Revert "Merge remote-tracking branch 'lan-fork/sua/tai-lieu-phi-cho'"
 1 file changed, 1 insertion(+), 1 deletion(-)
```

`-m 1` (mainline 1) = giữ phía **cha thứ nhất**, tức nhánh đang đứng lúc merge (`main`), bỏ mọi thay đổi do nhánh được merge mang vào. Gần như luôn là `-m 1`.

Cẩn thận một hệ quả: sau khi revert merge, Git vẫn coi các commit của nhánh kia **đã merge**. Merge lại nhánh đó lần nữa sẽ không mang lại gì. Muốn đưa lại thay đổi thì revert chính commit revert (Reapply), không merge lại.

## 5. Trên GitStudio

Chuột phải commit → **Revert commit**. GitStudio chạy `git revert --no-edit <mã>`, với merge commit tự thêm `-m 1`. Không có bước nhập lý do; muốn ghi lý do thì sau đó dùng **Sửa nội dung commit** (GIT-036) trước khi push, hoặc revert trong terminal không có `--no-edit`.

Nếu revert gặp xung đột, banner **Đang revert** hiện với **Huỷ thao tác** / **Tiếp tục**, như merge và rebase. Revert nằm trong nhật ký Hoàn tác: Ctrl+Z ngay sau đó chạy `reset --keep` về commit trước (chỉ nên làm khi chưa push).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-revert-commit-da-chia-se/gitstudio-01.webp" alt="Menu chuột phải của commit Tăng phí chờ tối đa lên 50.000 đ trên đồ thị GitStudio, mục Revert commit đang được trỏ" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Revert commit từ menu commit</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-revert-commit-da-chia-se/gitstudio-02.webp" alt="GitStudio mở commit Revert Tăng phí chờ tối đa lên 50.000 đ: panel bên phải hiện mô tả This reverts commit 17b8c3c…, ở giữa là diff của WaitingFee.cs đổi 50.000 đ và 50000m trở lại 30.000 đ và 30000m" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · commit revert là một commit bình thường: diff đảo ngược commit gốc</figcaption></figure>

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Revert một commit | Chuột phải commit → Revert commit | `git revert --no-edit <mã>` |
| Revert kèm lý do | Revert rồi Sửa nội dung commit | `git revert <mã>` (mở trình soạn thảo) |
| Revert merge commit | Revert commit (tự `-m 1`) | `git revert -m 1 <mã>` |
| Revert nhiều commit, một commit chung | | `git revert --no-commit A^..B` rồi `git commit` |
| Xung đột giữa chừng | Banner → Tiếp tục / Huỷ thao tác | `git revert --continue` / `--abort` |

## 7. Lỗi thường gặp

- **Reset main đã push rồi force push** thay vì revert. Xem GIT-032 mục 3.
- **Merge lại nhánh sau khi đã revert merge của nó**: "Already up to date", thay đổi không quay lại. Revert commit revert.
- **Revert để "dọn" lịch sử nhánh của riêng mình chưa push**: được, nhưng để lại hai commit thừa; reset hoặc rebase tương tác (GIT-037) gọn hơn.
- **Không ghi lý do**: sáu tháng sau không ai biết vì sao mức 50.000 đ bị bỏ.

## 8. Bài tập

**Bài 1.** Revert gộp hai commit `A` và `B` (B mới hơn) thành một commit duy nhất.

<details><summary><b>Lời giải</b></summary><p><code>git revert --no-commit B A</code> (mới trước, cũ sau), kiểm tra <code>git diff --staged</code>, rồi <code>git commit -m "Revert A và B: …"</code>.</p></details>

**Bài 2.** Merge commit `M` có cha 1 là `main`, cha 2 là nhánh tính năng. `git revert -m 2 M` làm gì?

<details><summary><b>Lời giải</b></summary><p>Giữ phía nhánh tính năng, bỏ mọi thay đổi <code>main</code> có mà nhánh tính năng không có tại lúc merge. Hầu như không bao giờ đúng ý.</p></details>

**Bài 3.** Kiểm tra trên `main` có commit nào đã bị revert chưa, bằng một lệnh log.

<details><summary><b>Lời giải</b></summary><p><code>git log --oneline --grep="This reverts commit"</code>.</p></details>

## 9. Đọc tiếp

- [git-revert](https://git-scm.com/docs/git-revert), [Reverting a faulty merge (howto của Git)](https://github.com/git/git/blob/master/Documentation/howto/revert-a-faulty-merge.txt).

Bài tiếp theo: [GIT-034 · Reflog: tìm lại commit "đã mất"](/articles/git-reflog-tim-lai-commit).
