---
slug: git-reflog-tim-lai-commit
title: "Reflog: tìm lại commit \"đã mất\" sau reset --hard, xoá nhánh hay rebase hỏng"
summary: "Git gần như không bao giờ xoá commit ngay: reflog ghi lại mọi lần HEAD và từng nhánh di chuyển. Bài này cứu ba tình huống trên repository báo giá: reset --hard lùi main hai commit, xoá nhầm nhánh chưa merge, và rebase xong mới thấy sai; đọc HEAD@{n}, nhanh@{n}, ORIG_HEAD, giới hạn của reflog (chỉ trên máy bạn, có hạn, không cứu thay đổi chưa commit), và mục Lịch sử HEAD (reflog) ở thanh bên GitStudio."
category: git
level: intermediate
tags: [git, git-034, reflog, orig-head, git-reset, recovery, gitstudio, toi-noi]
language: vi
---

Cuối buổi sáng 24/11/2026, Minh định dọn nhánh thử thì gõ nhầm `git reset --hard HEAD~2` trên `main`. Hai commit vừa push (tăng phí chờ và revert của nó) biến khỏi `git log`. Vẫn còn trên remote, nhưng anh muốn hiểu cách cứu khi commit **chỉ** nằm trên máy mình. Câu trả lời là **reflog**.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-034 · Phần 4/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-reset-soft-mixed-hard">GIT-032</a>, <a href="/articles/git-ben-trong-object-ref-head">GIT-008</a>. Lab: <code>npm run p04 -- 34</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 34</code>. Sau bài này, bạn tìm lại được commit sau reset, xoá nhánh, rebase, và biết khi nào reflog không cứu được.</p>
</div>

## 1. Reflog là gì

Mỗi lần HEAD hay một nhánh đổi vị trí (commit, checkout, reset, merge, rebase, pull…), Git ghi một dòng vào **reflog** của nó trong `.git/logs/`. Commit "mất" khỏi nhánh vẫn còn trong kho object, và reflog cho biết mã của nó. `HEAD@{n}` = chỗ HEAD đứng **n bước di chuyển trước**; `main@{n}` = chỗ nhánh `main` đứng n bước trước.

Reflog là **cục bộ**: không push, không clone. Mặc định giữ 90 ngày (30 ngày với commit không còn thuộc nhánh nào), sau đó `git gc` mới thật sự xoá.

## 2. Cứu sau `reset --hard`

```sh
git reset --hard HEAD~2
git log --oneline -1
git reflog -3
```

```text
HEAD is now at e93038f Ghi múi giờ áp dụng bảng giá
e93038f Ghi múi giờ áp dụng bảng giá
e93038f HEAD@{0}: reset: moving to HEAD~2
c19b370 HEAD@{1}: revert: Revert "Tăng phí chờ tối đa lên 50.000 đ"
17b8c3c HEAD@{2}: commit: Tăng phí chờ tối đa lên 50.000 đ
```

Đọc từ trên xuống: mới nhất trước. `HEAD@{1}` là chỗ HEAD đứng ngay trước lệnh reset.

```sh
git reset --hard HEAD@{1}
git status -sb
```

```text
HEAD is now at c19b370 Revert "Tăng phí chờ tối đa lên 50.000 đ"
## main...origin/main
```

Ngay sau một lệnh lớn (reset, merge, rebase), `ORIG_HEAD` cũng trỏ về chỗ cũ: `git reset --hard ORIG_HEAD` cho kết quả giống. Nhưng `ORIG_HEAD` bị ghi đè ở lệnh lớn kế tiếp; reflog thì giữ cả lịch sử.

## 3. Cứu nhánh bị xoá nhầm

```sh
git branch -D thu/ghi-chu
git reflog -2
git branch thu/ghi-chu HEAD@{1}
git log --oneline -1 thu/ghi-chu
```

```text
Deleted branch thu/ghi-chu (was 5ff3c7d).
c19b370 HEAD@{0}: checkout: moving from thu/ghi-chu to main
5ff3c7d HEAD@{1}: commit: Ghi ý tưởng giảm phí chờ
5ff3c7d Ghi ý tưởng giảm phí chờ
```

Xoá nhánh thì reflog **của nhánh** mất theo, nhưng reflog của HEAD vẫn nhớ lúc đứng ở đó. Ngay dòng `Deleted branch … (was 5ff3c7d)` cũng cho sẵn mã cần thiết: `git branch thu/ghi-chu 5ff3c7d`.

## 4. Quay về trước một rebase

Nhánh `thu/rebase` có một commit, rebase lên `main`, rồi Minh nhận ra không muốn thế:

```sh
git reflog -3 thu/rebase
git reset --hard thu/rebase@{1}
```

```text
f634cf0 thu/rebase@{0}: rebase (finish): refs/heads/thu/rebase onto c19b370…
c96052c thu/rebase@{1}: commit: Nháp trên nhánh thu/rebase
e93038f thu/rebase@{2}: branch: Created from HEAD~2
HEAD is now at c96052c Nháp trên nhánh thu/rebase
```

Reflog của **riêng nhánh** gọn hơn reflog của HEAD (rebase để lại nhiều dòng trong reflog HEAD, mỗi commit chép lại một dòng). `nhánh@{1}` = vị trí của nhánh trước lần đổi gần nhất.

## 5. Reflog không cứu được gì

- **Thay đổi chưa commit** bị `reset --hard`, `restore`, `checkout --`, Huỷ bỏ thay đổi trong GitStudio: chưa từng là object có tên trong reflog. (Riêng file đã `git add` thì blob còn trong kho, tìm bằng `git fsck --lost-found`, rất vất vả.)
- **Commit trên máy khác**: reflog của Lan không ở trên máy Minh.
- **Sau `git gc` khi đã hết hạn**, hay sau `git reflog expire --expire=now --all` + `gc --prune=now`.
- **Clone mới**: reflog trống.

## 6. Trên GitStudio

Thanh bên → **Lịch sử HEAD (reflog)**: mỗi dòng là một lần HEAD di chuyển, ghi loại (RESET, REVERT, COMMIT, PULL, MERGE, CHECKOUT, REBASE…), tiêu đề commit và thời gian. Commit không còn nhánh hay tag nào trỏ tới được đánh dấu **mất**: đó chính là những commit cần cứu. Chuột phải một dòng: **Tạo nhánh tại đây** (cứu nhánh đã xoá), **Checkout commit này**, **Cherry-pick commit**, **Reset main về commit này** (Soft / Mixed / Hard), **Sao chép SHA commit**, **Copy HEAD@{n}** (mục này chưa được dịch).

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-reflog-tim-lai-commit/gitstudio-01.webp" alt="Thanh bên GitStudio, mục Lịch sử HEAD (reflog) mở rộng sau khi reset --hard: dòng reset: moving to HEAD~2 ở trên cùng, bên dưới là revert và commit Tăng phí chờ tối đa lên 50.000 đ; đồ thị chỉ còn main ở commit Ghi múi giờ áp dụng bảng giá" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Lịch sử HEAD (reflog) ngay sau reset --hard nhầm</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-reflog-tim-lai-commit/gitstudio-02.webp" alt="Menu chuột phải của dòng reflog revert trong GitStudio: Tạo nhánh tại đây, Checkout commit này, Cherry-pick commit, Reset main về commit này, Sao chép SHA commit, Copy HEAD@{1}" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · chuột phải một dòng reflog để cứu commit</figcaption></figure>

Với các thao tác GitStudio tự làm, Hoàn tác (Ctrl+Z) thường nhanh hơn (GIT-035). Reflog là lưới an toàn cho mọi thứ khác: lệnh gõ trong terminal, thao tác của phiên trước, công cụ khác.

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Xem HEAD đã đi đâu | Thanh bên → Lịch sử HEAD (reflog) | `git reflog` |
| Xem một nhánh đã đi đâu | | `git reflog <nhánh>` |
| Quay về vị trí cũ | Chuột phải dòng → Reset … về commit này | `git reset --hard HEAD@{n}` |
| Cứu nhánh đã xoá | Chuột phải dòng → Tạo nhánh tại đây | `git branch <tên> HEAD@{n}` |
| Ngay sau reset/merge/rebase | Hoàn tác | `git reset --hard ORIG_HEAD` |

## 8. Lỗi thường gặp

- **Đọc nhầm thứ tự**: `HEAD@{1}` là một **bước di chuyển** trước, không phải một commit trước (`HEAD~1`).
- **Cứu xong lại chạy lệnh lớn trước khi tạo nhánh**: HEAD@{n} dịch đi. Tạo nhánh ngay bằng mã đầy đủ.
- **Trông vào reflog để giữ thử nghiệm lâu dài**: hết hạn sau 30–90 ngày. Muốn giữ thì tạo nhánh hoặc tag.
- **PowerShell và `@{…}`**: trong PowerShell, `@{1}` có thể bị hiểu là hashtable trong một số ngữ cảnh; đặt trong nháy: `git reset --hard 'HEAD@{1}'`.

## 9. Bài tập

**Bài 1.** Viết lệnh xem `main` đứng ở đâu lúc 9 giờ sáng hôm qua.

<details><summary><b>Lời giải</b></summary><p><code>git rev-parse 'main@{yesterday 09:00}'</code> hoặc <code>git log -1 'main@{2026-11-23 09:00}'</code>. Reflog nhận cả mốc thời gian.</p></details>

**Bài 2.** Sau một rebase, reflog của HEAD có thêm bao nhiêu dòng nếu nhánh có 3 commit?

<details><summary><b>Lời giải</b></summary><p>Khoảng 5: <code>rebase (start)</code>, ba dòng <code>rebase (pick)</code>, <code>rebase (finish)</code>. Reflog của nhánh chỉ thêm một dòng <code>rebase (finish)</code>.</p></details>

**Bài 3.** Liệt kê các commit không còn nhánh nào trỏ tới nhưng reflog vẫn nhớ.

<details><summary><b>Lời giải</b></summary><p><code>git log --oneline --walk-reflogs</code> cho mọi dòng reflog; để lọc commit không thuộc nhánh nào: <code>git log --oneline --reflog --not --branches --tags</code>.</p></details>

## 10. Đọc tiếp

- [git-reflog](https://git-scm.com/docs/git-reflog), [gitrevisions: `@{n}`, `@{date}`](https://git-scm.com/docs/gitrevisions).
- [Pro Git, Data Recovery](https://git-scm.com/book/en/v2/Git-Internals-Maintenance-and-Data-Recovery).

Bài tiếp theo: [GIT-035 · Undo/Redo của GitStudio và giới hạn của nó](/articles/git-undo-redo-gitstudio).
