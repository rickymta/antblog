---
slug: git-undo-redo-gitstudio
title: "Hoàn tác và Làm lại của GitStudio: thao tác nào quay lại được, bên dưới là lệnh git nào"
summary: "Nút Hoàn tác (Ctrl+Z) của GitStudio trông như Undo của trình soạn thảo, nhưng bên dưới là các lệnh git cụ thể: reset --soft cho commit, reset --keep cho merge, pull, revert, cherry-pick, sửa và bỏ commit, rebase; xoá lại nhánh vừa tạo, tạo lại nhánh vừa xoá. Bài này liệt kê thao tác nào có trong nhật ký Hoàn tác, chạy đúng các lệnh đó trong lab, cho thấy vì sao reset --keep từ chối khi có thay đổi chưa commit, và những gì Hoàn tác không cứu được."
category: git
level: intermediate
tags: [git, git-035, undo, redo, reset-keep, gitstudio, toi-noi]
language: vi
---

Lan quen Ctrl+Z trong mọi ứng dụng, nên trong GitStudio cô cũng bấm Ctrl+Z sau mỗi lần lỡ tay. Phần lớn lần đó GitStudio quay lại đúng như mong đợi. Nhưng có lần cô bấm **Huỷ bỏ thay đổi** trên một file rồi Ctrl+Z, và không có gì quay lại. Bài này mở nắp nút Hoàn tác để biết chính xác nó làm gì.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-035 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-reset-soft-mixed-hard">GIT-032</a>, <a href="/articles/git-reflog-tim-lai-commit">GIT-034</a>. Lab: <code>npm run p04 -- 35</code> (chạy đúng các lệnh git mà Hoàn tác/Làm lại dùng). Sau bài này, bạn biết thao tác nào trong GitStudio hoàn tác được, cái giá của nó, và lúc nào phải dùng reflog.</p>
</div>

## 1. Cơ chế: nhật ký Hoàn tác

Mỗi khi bạn làm một thao tác "có thể đảo" trong GitStudio, nó ghi lại vị trí nhánh **trước** và **sau**, cùng cách đi qua lại giữa hai vị trí. **Hoàn tác** (nút trên thanh công cụ, `Ctrl+Z`) chạy lệnh đi ngược; **Làm lại** (`Ctrl+Y` hoặc `Ctrl+Shift+Z`) chạy lệnh đi xuôi. Di chuột lên nút để xem tên thao tác: "Hoàn tác revert", "Làm lại sửa thông điệp commit"…

Nhật ký có giới hạn rõ ràng:

- **Theo từng repository, chỉ trong phiên đang mở.** Đổi tab rồi quay lại vẫn còn; đóng GitStudio là mất. Thao tác làm trong terminal (kể cả terminal tích hợp) không vào nhật ký.
- Giữ tối đa 50 bước. Làm thao tác mới thì danh sách Làm lại bị xoá.
- Trước khi ghi, GitStudio đọc vị trí HEAD thật từ git (không dùng hình đang hiển thị), để không lùi qua một commit vừa tạo ngoài GitStudio.

## 2. Thao tác nào hoàn tác được

| Thao tác trong GitStudio | Hoàn tác chạy | Làm lại chạy |
|---|---|---|
| Commit, Amend | `git reset --soft <đỉnh cũ>`: thay đổi quay về Tệp đã stage | `git reset --soft <đỉnh mới>` |
| Merge, Pull, Cherry-pick, Revert, Sửa nội dung commit, Bỏ commit, Di chuyển commit, Rebase tương tác | `git reset --keep <đỉnh cũ>` | `git reset --keep <đỉnh mới>` |
| Reset (Soft/Mixed) | `git reset --soft <đỉnh cũ>` | Reset lại đúng chế độ |
| Reset Hard | `git reset --hard <đỉnh cũ>` (commit quay lại, **thay đổi chưa commit đã mất thì không**) | `git reset --hard <đỉnh mới>` |
| Tạo nhánh | Xoá nhánh (và checkout lại nhánh cũ nếu đã chuyển) | Tạo lại |
| Xoá nhánh | Tạo lại nhánh ở commit cũ | Xoá lại |
| Checkout | Checkout lại nhánh/commit trước | Checkout lại |

Với merge, pull, revert… GitStudio chỉ ghi khi nhánh **thật sự di chuyển** trên **cùng một nhánh**; merge gặp xung đột hay pull "already up to date" thì không có gì để hoàn tác.

**Không** có trong nhật ký: Huỷ bỏ thay đổi (file, hunk, dòng), Stage/Unstage, Stash và Pop/Apply/Drop stash, Push, Fetch, tạo/xoá tag, thao tác trên remote. Push là ranh giới quan trọng nhất: Hoàn tác một commit **đã push** chỉ lùi nhánh cục bộ; remote vẫn giữ commit đó và lần push sau sẽ bị từ chối (GIT-032 mục 3).

## 3. Commit: reset --soft

```sh
git commit -q -am "Ghi chú thử"
git reset --soft HEAD~1
git status -s
```

```text
M  README.md
```

Commit biến mất khỏi nhánh, thay đổi nằm nguyên trong staging area: sửa lại rồi commit lần nữa là xong. Giống hệt GIT-032.

## 4. Merge, pull, revert…: reset --keep

`--keep` giống `--hard` ở chỗ đưa working tree về commit đích, nhưng **từ chối** nếu phải đè lên một file đang có thay đổi chưa commit. Lab tạo commit Reapply (revert của revert, GIT-033), rồi sửa đúng file mà commit đó đổi:

```sh
git revert --no-edit HEAD
git reset --keep HEAD~1
git restore src/BaoGia/WaitingFee.cs
git reset --keep HEAD~1
git status -sb
```

```text
[main 65d94b6] Reapply "Tăng phí chờ tối đa lên 50.000 đ"
 1 file changed, 2 insertions(+), 2 deletions(-)
error: Entry 'src/BaoGia/WaitingFee.cs' not uptodate. Cannot merge.
fatal: Could not reset index file to revision 'HEAD~1'.
## main...origin/main
```

Trong GitStudio, lỗi này hiện thành thông báo "Hoàn tác revert thất bại" kèm nguyên văn lỗi git; nhánh không đổi gì. Commit hoặc stash thay đổi đang dở, rồi Hoàn tác lại. Thay đổi ở file **không** bị đụng tới thì `--keep` giữ nguyên, không cần dọn.

## 5. Huỷ bỏ thay đổi: không có đường lùi

```sh
git checkout -- README.md
git status -s
git stash list
```

(không in gì)

Đây chính là lệnh GitStudio chạy khi bạn bấm **Huỷ bỏ thay đổi** (`git clean -f` cho file mới). Không có commit, stash hay object nào giữ nội dung vừa bỏ, nên GitStudio không đưa nó vào nhật ký và luôn hỏi xác nhận "Thao tác này không thể hoàn tác". Đó là lần Ctrl+Z "không làm gì" của Lan.

## 6. Trên GitStudio

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-undo-redo-gitstudio/gitstudio-01.webp" alt="GitStudio ngay sau khi Revert commit trong menu: commit Reapply Tăng phí chờ tối đa lên 50.000 đ ở đầu main, origin/main ở commit bên dưới; nút Hoàn tác trên thanh công cụ được bật, mục Lịch sử HEAD (reflog) có dòng REVERT mới" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · sau revert, nút Hoàn tác sẵn sàng (di chuột lên để thấy "Hoàn tác revert")</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-undo-redo-gitstudio/gitstudio-02.webp" alt="GitStudio sau khi bấm Hoàn tác: main trở lại commit Revert Tăng phí chờ tối đa lên 50.000 đ, trùng origin/main; nút Làm lại được bật; trong Lịch sử HEAD (reflog) có dòng RESET mới và commit Reapply được đánh dấu mất" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · sau Hoàn tác: nhánh về đỉnh cũ, Làm lại sẵn sàng</figcaption></figure>

## 7. Khi nào dùng gì

| Tình huống | Dùng |
|---|---|
| Vừa làm một thao tác trong GitStudio, chưa push | Hoàn tác (Ctrl+Z) |
| Thao tác gõ trong terminal, hoặc từ phiên trước | Reflog (GIT-034) |
| Commit đã push lên nhánh chung | Revert (GIT-033) |
| Thay đổi chưa commit, còn phân vân | Stash trước khi thử (GIT-015) |
| Thay đổi chưa commit đã bị Huỷ bỏ | Không cứu được bằng Git |

## 8. Lỗi thường gặp

- **Hoàn tác một commit đã push rồi commit tiếp và push**: bị từ chối (non-fast-forward). Hoặc Làm lại để về như cũ, hoặc revert thay vì Hoàn tác.
- **Tưởng Hoàn tác lấy lại thay đổi bị Reset Hard**: chỉ lấy lại commit.
- **Đóng GitStudio rồi mới muốn Hoàn tác**: nhật ký đã mất; reflog vẫn còn.
- **Hoàn tác thất bại vì thay đổi chưa commit**: đọc thông báo, stash rồi thử lại; đừng chuyển sang Reset Hard.

## 9. Bài tập

**Bài 1.** Bạn Merge một nhánh trong GitStudio, có xung đột, bấm Huỷ thao tác trên banner Đang merge. Nhật ký Hoàn tác có thêm gì không?

<details><summary><b>Lời giải</b></summary><p>Không. Merge dừng ở xung đột thì nhánh chưa di chuyển, GitStudio không ghi gì; Huỷ thao tác (<code>git merge --abort</code>) cũng không phải thao tác có trong nhật ký.</p></details>

**Bài 2.** Viết lệnh terminal tương đương với việc bấm Hoàn tác ngay sau khi Pull tạo merge commit.

<details><summary><b>Lời giải</b></summary><p><code>git reset --keep ORIG_HEAD</code> (hoặc <code>git reset --keep HEAD@{1}</code>). GIT-029 dùng <code>--hard</code>, còn GitStudio dùng <code>--keep</code> để không đè thay đổi chưa commit.</p></details>

**Bài 3.** Vì sao Hoàn tác commit dùng `--soft` mà Hoàn tác merge lại dùng `--keep`?

<details><summary><b>Lời giải</b></summary><p>Hoàn tác commit nên trả lại thay đổi để sửa rồi commit lại: <code>--soft</code> giữ chúng trong staging area. Hoàn tác merge nên đưa file về như trước merge; để lại thay đổi của nhánh kia trong staging area sẽ gây nhầm, nên dùng <code>--keep</code>.</p></details>

## 10. Đọc tiếp

- [git-reset: `--keep`](https://git-scm.com/docs/git-reset#Documentation/git-reset.txt---keep).
- Mã nguồn GitStudio: `src/features/actions.ts` (các hàm `journaled`, `journaledHead`).

Bài tiếp theo: [GIT-036 · Sửa commit: sửa nội dung, bỏ commit, đổi thứ tự](/articles/git-sua-commit-message-drop-thu-tu).
