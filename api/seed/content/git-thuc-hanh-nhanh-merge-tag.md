---
slug: git-thuc-hanh-nhanh-merge-tag
title: "Thực hành Phần 2: phụ phí giờ cao điểm từ nhánh tới bản phát hành, chấm bằng script 11 tiêu chí"
summary: "Bài thực hành cuối Phần 2: tự đi trọn quy trình của nhóm Tới Nơi trên GitStudio, từ nhánh tính năng, bản vá gấp, merge fast-forward và --no-ff, xung đột thật ở dòng tính cước, stash, tag v1.0.0/v1.1.0, nhánh phát hành với cherry-pick -x và v1.0.1, revert, rebase, tới worktree, rồi chấm bằng kiem-tra.mjs. Bài cũng chữa một repository còn merge dở dang, stash bị quên, nhánh sót và tag lightweight."
category: git
level: advanced
tags: [git, git-020, thuc-hanh, branch, merge, conflict, tag, cherry-pick, gitstudio, toi-noi]
language: vi
---

Trưởng nhóm backend giao việc cuối tuần cho thành viên mới: "Làm lại toàn bộ đợt phát hành giờ cao điểm vừa rồi trên repository của em, bằng GitStudio. Không cần giống từng commit, nhưng repository cuối cùng phải sạch: không merge dở, không stash quên, nhánh nào merge rồi thì xoá, phiên bản phải là annotated tag, bản vá cho 1.0 phải ghi rõ nguồn." Script `p02/kiem-tra.mjs` là người chấm.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-020 · Phần 2/6 · Nâng cao · Thực hành</span>
<p>Đọc trước: GIT-011 tới <a href="/articles/git-worktree">GIT-019</a>. Lab: <code>npm run p02 -- 20</code>; script chấm: <code>node p02/kiem-tra.mjs &lt;đường dẫn repository&gt;</code> (hoặc <code>npm run kiem-tra-p02 -- &lt;đường dẫn&gt;</code>). Điểm xuất phát: repository cuối Phần 1 (<code>npm run setup -- 10 --end</code>, hoặc repository bạn tự làm ở GIT-010).</p>
</div>

## 1. Đề bài

| # | Việc | Kỹ năng | Bài |
|---|---|---|---|
| 1 | Nhánh `tinh-nang/phu-phi-gio-cao-diem`: cấu hình `Peak`, rồi tham số `peakMultiplier` | Tạo nhánh, commit trên nhánh | GIT-011 |
| 2 | Đang dở thì sửa gấp trên `main`: làm tròn quãng đường 0,1 km, kèm test | Chuyển nhánh khi còn thay đổi | GIT-012 |
| 3 | Hai nhánh tài liệu: một merge fast-forward, một merge `--no-ff` | Merge | GIT-013 |
| 4 | Merge nhánh tính năng vào `main`, giải quyết xung đột dòng `var fare` giữ **cả hai** ý | Xung đột | GIT-014 |
| 5 | Viết test giờ cao điểm, giữa chừng stash để chạy test trên code sạch | Stash | GIT-015 |
| 6 | Annotated tag `v1.0.0` (cuối Phần 1) và `v1.1.0` (HEAD) | Tag | GIT-016 |
| 7 | Nhánh `phat-hanh/1.0` từ `v1.0.0`, cherry-pick `-x` bản vá, tag `v1.0.1`; revert cấu hình giờ cao điểm trên `main` | Cherry-pick, revert | GIT-017 |
| 8 | Nhánh phí chờ tách từ `v1.1.0`, `main` sửa cùng dòng README; rebase lên `main`, fast-forward | Rebase | GIT-018 |
| 9 | Ghi chú phát hành trên `phat-hanh/1.0`, làm trong worktree phụ rồi gỡ worktree | Worktree | GIT-019 |

Làm bằng GitStudio là chính; terminal tích hợp để kiểm và cho những việc chưa có nút.

## 2. Mười một tiêu chí

`p02/kiem-tra.mjs` chỉ đọc repository:

| Tiêu chí | Kiểm bằng | Vì sao |
|---|---|---|
| Không có merge/rebase/cherry-pick/revert dở dang | `MERGE_HEAD`, `rebase-merge`, `CHERRY_PICK_HEAD`… trong `.git` | Thao tác dở làm commit sau thành sai loại |
| Working tree sạch | `git status --porcelain` | Bàn giao là đã commit hết |
| Không còn dấu xung đột trong file đã commit | `git grep` marker `<<<<<<< `, `=======`, `>>>>>>> ` | Marker lọt vào lịch sử là build hỏng |
| `main` có ít nhất một merge commit | `git rev-list --merges` | Có dấu vết tính năng (merge `--no-ff` hoặc merge có xung đột) |
| Có tag **annotated** `v1.0.0`, `v1.1.0`, `v1.0.1` | `git cat-file -t` ra `tag` | Phiên bản phải có người gắn, thời điểm, lời nhắn |
| `v1.1.0` nằm trên lịch sử của `main` | `git merge-base --is-ancestor` | Bản phát hành đến từ nhánh chính |
| `phat-hanh/1.0` có commit cherry-pick ghi nguồn | `(cherry picked from commit` trong `v1.0.0..phat-hanh/1.0` | Lần ra được bản gốc của bản vá |
| `v1.0.1` trên nhánh phát hành, không trên `main` | `merge-base --is-ancestor` hai chiều | Bản vá cho dòng 1.0 không lẫn vào dòng chính |
| Nhánh đã merge vào `main` đã được xoá | `git branch --merged main` (trừ `main`, `phat-hanh/*`) | Danh sách nhánh chỉ còn việc đang làm |
| Không còn stash bị bỏ quên | `git stash list` | Stash là chỗ cất tạm |
| Không còn worktree phụ | `git worktree list --porcelain` | Worktree treo giữ nhánh và tốn chỗ |

## 3. Đáp án mẫu

Lab dựng lịch sử chuẩn của Phần 2 và chấm:

```sh
git log --oneline --graph --decorate --all
```

```text
* c6e65d7 (phat-hanh/1.0) Thêm ghi chú phát hành 1.0.1
* b047783 (tag: v1.0.1) Làm tròn quãng đường tới 0,1 km trước khi tính cước
| * 61f5ed6 (HEAD -> main) Thêm tài liệu phí chờ
| * 95720a8 Nhắc thời gian chờ trong mô tả dịch vụ
| * d42d397 Nhắc giờ đặt chuyến trong mô tả dịch vụ
| * b6829e3 Revert "Thêm hệ số phụ phí giờ cao điểm vào bảng giá"
| * 70b18f6 (tag: v1.1.0) Thêm test cho phụ phí giờ cao điểm
| *   e46c89f Merge branch 'tinh-nang/phu-phi-gio-cao-diem'
| |\  
| | * e4cd655 Nhân hệ số giờ cao điểm khi tính cước
| | * c46c1dd Thêm hệ số phụ phí giờ cao điểm vào bảng giá
| |/  
|/|   
| *   8c05257 Merge branch 'tai-lieu/gio-cao-diem'
| |\  
| | * 3e7f15a Mô tả phụ phí giờ cao điểm trong tài liệu
| |/  
| * efe9c33 Ghi cách làm tròn quãng đường vào tài liệu
| * 7993c73 Làm tròn quãng đường tới 0,1 km trước khi tính cước
|/  
* 89855d5 (tag: v1.0.0) Chuẩn hoá xuống dòng LF bằng .gitattributes
…
```

(Lược mười commit Phần 1 phía dưới.) Đọc đồ thị từ dưới lên là đọc lại cả Phần 2: bản vá thẳng trên `main`, fast-forward tài liệu, bong bóng `--no-ff`, merge có xung đột của nhánh tính năng, tag, revert, hai commit phí chờ nằm thẳng nhờ rebase, và nhánh phát hành rẽ từ `v1.0.0`.

Chạy script trong terminal tích hợp của GitStudio:

```powershell
node ..\..\p02\kiem-tra.mjs .
```

```text
ĐẠT      không có merge/rebase/cherry-pick/revert dở dang
ĐẠT      working tree sạch
ĐẠT      không còn dấu xung đột trong file đã commit
ĐẠT      main có ít nhất một merge commit
ĐẠT      có tag annotated v1.0.0, v1.1.0, v1.0.1
ĐẠT      v1.1.0 nằm trên lịch sử của main
ĐẠT      nhánh phat-hanh/1.0 có commit cherry-pick ghi nguồn (-x)
ĐẠT      v1.0.1 nằm trên nhánh phát hành, không nằm trên main
ĐẠT      nhánh đã merge vào main đã được xoá
ĐẠT      không còn stash bị bỏ quên
ĐẠT      không còn worktree phụ
11/11 tiêu chí đạt
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-thuc-hanh-nhanh-merge-tag/gitstudio-01.webp" alt="GitStudio với đồ thị cuối Phần 2: nhánh phat-hanh/1.0 và tag v1.0.1 ở trên cùng, main với các commit phí chờ, revert, tag v1.1.0, hai merge commit, tag v1.0.0; terminal tích hợp chạy node ..\..\p02\kiem-tra.mjs . và in mười một dòng ĐẠT, 11/11 tiêu chí đạt" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · đồ thị cuối Phần 2 và kết quả chấm 11/11</figcaption></figure>

## 4. Gợi ý từng bước trên GitStudio

1. **Nhánh**: nút **Nhánh** (`Ctrl+B`), tên `tinh-nang/phu-phi-gio-cao-diem`, Checkout sau khi tạo.
2. **Sửa gấp**: chuột phải `main` → **Checkout main**. Không chuyển được thì mở **Nhật ký hoạt động** (`Ctrl+Shift+L`) đọc lý do; commit hoặc stash trước.
3. **Merge tài liệu**: chuột phải nhánh tài liệu → **Merge … vào main**; nhánh thứ hai chọn **Luôn tạo merge commit (--no-ff)** trong Xem trước merge.
4. **Xung đột**: Xem trước merge → chọn B, **Sửa tay** dòng `var fare` cho có cả `Math.Round(…)` lẫn `* peakMultiplier` → **Lưu cách giải quyết** → **Merge**. Kiểm bằng `git show --remerge-diff HEAD`.
5. **Stash**: stage file test, tab **Stash**, đặt tiêu đề; xong việc bấm **Pop**.
6. **Tag**: chuột phải commit → **Tạo annotated tag tại đây**.
7. **Phát hành 1.0**: chuột phải tag `v1.0.0` → **Tạo nhánh tại đây** `phat-hanh/1.0`; cherry-pick bản vá bằng terminal tích hợp `git cherry-pick -x <commit>` (nút **Cherry-pick commit** của GitStudio 1.0.1 không thêm dòng nguồn, nên tiêu chí 7 sẽ trượt nếu chỉ dùng nút). Revert: chuột phải commit cấu hình → **Revert commit**.
8. **Rebase**: đứng ở nhánh phí chờ, chuột phải `main` → **Rebase … lên main**; xung đột thì nhớ A là `main`, B là commit đang chép.
9. **Worktree**: thanh bên → Worktree → **Worktree mới…**; xong việc gỡ worktree.

Sau mỗi bước, liếc đồ thị: hình dạng có giống mô tả của bài tương ứng không.

## 5. Chữa một repository "hỏng"

Lab làm hỏng có chủ đích repository đáp án: tạo lại nhánh tính năng đã merge, stash một ghi chú rồi quên, gắn lại `v1.0.1` thành lightweight tag, và bỏ dở một merge có xung đột:

```text
CHƯA ĐẠT không có merge/rebase/cherry-pick/revert dở dang — MERGE_HEAD
CHƯA ĐẠT working tree sạch — UU src/BaoGia/FareCalculator.cs
CHƯA ĐẠT không còn dấu xung đột trong file đã commit — src/BaoGia/FareCalculator.cs:9:<<<<<<< HEAD, src/BaoGia/FareCalculator.cs:11:=======
ĐẠT      main có ít nhất một merge commit
CHƯA ĐẠT có tag annotated v1.0.0, v1.1.0, v1.0.1 — v1.0.1: commit
ĐẠT      v1.1.0 nằm trên lịch sử của main
ĐẠT      nhánh phat-hanh/1.0 có commit cherry-pick ghi nguồn (-x)
ĐẠT      v1.0.1 nằm trên nhánh phát hành, không nằm trên main
CHƯA ĐẠT nhánh đã merge vào main đã được xoá — tinh-nang/phu-phi-gio-cao-diem
CHƯA ĐẠT không còn stash bị bỏ quên — stash@{0}: On main: Ghi chú tạm
ĐẠT      không còn worktree phụ
5/11 tiêu chí đạt
```

Ba dòng đầu cùng một nguyên nhân: merge bỏ dở (`UU` = cả hai bên đều sửa, chưa giải quyết). Sửa theo thứ tự:

```sh
git merge --abort
git branch -D thu-xung-dot
git branch -d tinh-nang/phu-phi-gio-cao-diem
git stash drop
git tag -d v1.0.1
git tag -a v1.0.1 -m "Phiên bản 1.0.1: bản vá làm tròn quãng đường" phat-hanh/1.0~1
```

```text
Deleted branch thu-xung-dot (was d6ba037).
Deleted branch tinh-nang/phu-phi-gio-cao-diem (was b6829e3).
Deleted tag 'v1.0.1' (was b047783)
```

| Lệnh | Sửa lỗi |
|---|---|
| `git merge --abort` | Huỷ merge dở; working tree hết marker, hết `UU` |
| `git branch -D thu-xung-dot` | Xoá nhánh thử (`-D` vì nhánh có commit chưa merge, và chủ đích bỏ) |
| `git branch -d tinh-nang/…` | Nhánh đã merge: `-d` đủ, Git xác nhận an toàn |
| `git stash drop` | Bỏ stash quên (xem `git stash show -p` trước nếu chưa chắc nội dung) |
| `git tag -d` rồi `git tag -a` | Thay lightweight tag bằng annotated tag |

Chạy lại script: 11/11. Lab kiểm thêm: tag `v1.0.1` tạo lại trùng mã hash với lịch sử chuẩn, vì cùng người gắn, cùng thời điểm (lab cố định thời điểm), cùng lời nhắn, cùng commit. Trên máy bạn, thời điểm khác nên tag object khác mã hash: đó là lý do không được xoá rồi tạo lại tag **đã push** (GIT-016).

## 6. Tự đánh giá

1. Ai giải quyết xung đột ở merge `e46c89f`, và kết quả có giữ được bản vá làm tròn quãng đường không? (`git show --remerge-diff e46c89f`.)
2. Production thành phố đang chạy 1.0 có bản vá chưa, bản vá đó tương ứng commit nào trên `main`? (`git log -1 v1.0.1`.)
3. Muốn bật lại phụ phí giờ cao điểm cần đúng một lệnh gì? (GIT-017, bài tập 2.)

## 7. Lỗi thường gặp khi làm bài

- **Merge nhánh tính năng bằng "Lấy bản của họ" cho nhanh.** Script không bắt được (không còn marker), nhưng test `Quote_LamTronQuangDuong01Km` sẽ đỏ: luôn chạy test sau merge.
- **Gắn tag trước khi merge xong.** `v1.1.0` không nằm trên `main` → tiêu chí 6 trượt.
- **Cherry-pick bằng nút của GitStudio.** Không có dòng nguồn, tiêu chí 7 trượt; làm bằng `git cherry-pick -x`, hoặc sửa nội dung commit để thêm dòng `(cherry picked from commit …)`.
- **Rebase nhánh `main` lên nhánh phí chờ** thay vì ngược lại (GIT-018, bài tập 3).
- **Xoá thư mục worktree bằng Explorer.** Tiêu chí 11 vẫn trượt cho tới khi `git worktree prune`.

## 8. Bài tập mở rộng

**Bài 1.** Thêm tiêu chí: mọi merge commit có xung đột phải có description giải thích cách giải quyết. Gợi ý: `git log --merges --format=%H` rồi `git show --remerge-diff` để biết merge nào từng có xung đột.

<details><summary><b>Lời giải</b></summary><p>Với mỗi merge commit, chạy <code>git show --remerge-diff --format= &lt;hash&gt;</code>; nếu kết quả chứa <code>remerge CONFLICT</code> thì merge đó từng xung đột. Khi đó kiểm <code>git log -1 --format=%b &lt;hash&gt;</code> không rỗng. Merge <code>e46c89f</code> của lab sẽ trượt tiêu chí này (dùng nội dung mặc định), một gợi ý tốt để sửa quy trình của nhóm.</p></details>

**Bài 2.** Viết lệnh liệt kê các bản vá trên `phat-hanh/1.0` chưa có trên `main` (theo nội dung thay đổi, không theo mã hash).

<details><summary><b>Lời giải</b></summary><p><code>git log --oneline --cherry-pick --right-only main...phat-hanh/1.0</code>: bỏ các commit có cùng thay đổi ở hai phía. Ở lab, chỉ còn <code>c6e65d7</code> (ghi chú phát hành) vì bản vá <code>b047783</code> đã có trên <code>main</code> dưới dạng <code>7993c73</code>.</p></details>

## 9. Kết thúc Phần 2

Bạn đã đi trọn vòng đời một tính năng trên một máy: nhánh, merge, xung đột, stash, tag, bản vá cho bản cũ, hoàn tác, rebase, worktree. Phần 3 đưa repository lên **remote**: `origin`, fetch, pull, push, nhánh remote-tracking, push bị từ chối, xác thực, GitHub/GitLab và pull request; Minh và Lan cuối cùng làm việc trên hai máy thật.

## 10. Đọc tiếp

- Mã nguồn script chấm: `examples/toi-noi-git/p02/kiem-tra.mjs`; lịch sử chuẩn: `examples/toi-noi-git/lib/history2.mjs`.
- [Pro Git, chương 3: Git Branching](https://git-scm.com/book/en/v2/Git-Branching-Branches-in-a-Nutshell).

Bài tiếp theo (Phần 3): GIT-021 · Remote, origin và nhánh remote-tracking *(đang viết)*.
