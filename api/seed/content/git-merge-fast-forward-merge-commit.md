---
slug: git-merge-fast-forward-merge-commit
title: "Merge trong Git: fast-forward và merge commit, khi nào dùng --no-ff"
summary: "Lan sửa tài liệu bảng giá trên hai nhánh ngắn và cần đưa về main. Bài này phân biệt merge fast-forward (chỉ dời con trỏ) với merge commit có hai commit cha, merge bằng menu chuột phải của GitStudio và bằng git merge, dùng --no-ff để giữ dấu vết nhánh, kiểm nhánh đã merge bằng git branch --merged và xoá nhánh sau khi merge."
category: git
level: beginner
tags: [git, git-013, git-merge, fast-forward, merge-commit, no-ff, gitstudio, toi-noi]
language: vi
---

Chiều 16/11/2026, Lan có hai việc nhỏ với tài liệu bảng giá, mỗi việc trên một nhánh riêng: ghi cách làm tròn quãng đường 0,1 km (bản vá của Minh ở GIT-012), và mô tả phụ phí giờ cao điểm cho bộ phận chăm sóc khách hàng. Cả hai cần về `main`. Cùng là `git merge`, nhưng hai lần merge cho ra hai hình dạng lịch sử khác nhau, và nhóm cần hiểu vì sao.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-013 · Phần 2/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-chuyen-nhanh-thay-doi-chua-commit">GIT-012</a>. Lab: <code>npm run p02 -- 13</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 13</code>. Sau bài này, bạn phân biệt được fast-forward với merge commit, merge được bằng giao diện và lệnh, biết khi nào dùng <code>--no-ff</code>, và dọn nhánh sau khi merge.</p>
</div>

## 1. Merge là gì

**Merge** đưa các thay đổi của một nhánh vào nhánh đang đứng (nhánh HEAD trỏ tới). Bạn luôn **đứng ở nhánh nhận** rồi merge **nhánh nguồn** vào: muốn đưa `sua/x` vào `main` thì checkout `main`, rồi `git merge sua/x`.

Git chọn một trong hai cách, tuỳ hình dạng đồ thị:

| Tình huống | Cách merge | Kết quả |
|---|---|---|
| Nhánh nhận **chưa đi tiếp** kể từ khi nhánh nguồn tách ra | **Fast-forward** | Không có commit mới; nhánh nhận chỉ dời con trỏ tới commit của nhánh nguồn |
| Cả hai nhánh **đều có commit riêng** | **Merge commit** (three-way merge) | Commit mới có **hai commit cha**, gộp thay đổi của hai bên |

## 2. Fast-forward

Lan tạo nhánh, sửa một dòng tài liệu, commit:

```sh
git switch -c sua/tai-lieu-lam-tron
git commit -am "Ghi cách làm tròn quãng đường vào tài liệu"
git switch main
git merge sua/tai-lieu-lam-tron
```

```text
Switched to a new branch 'sua/tai-lieu-lam-tron'
[sua/tai-lieu-lam-tron efe9c33] Ghi cách làm tròn quãng đường vào tài liệu
 1 file changed, 1 insertion(+), 1 deletion(-)
Switched to branch 'main'
Updating 7993c73..efe9c33
Fast-forward
 docs/bang-gia-cuoc.md | 2 +-
 1 file changed, 1 insertion(+), 1 deletion(-)
```

`main` đang ở `7993c73`, nhánh của Lan là `7993c73` + một commit. Không có gì phải "gộp": Git chỉ dời `main` lên `efe9c33` (`Updating 7993c73..efe9c33`, `Fast-forward`). Lịch sử vẫn thẳng, không ai biết đã từng có nhánh:

```sh
git branch -d sua/tai-lieu-lam-tron
git log --oneline --graph -3
```

```text
Deleted branch sua/tai-lieu-lam-tron (was efe9c33).
* efe9c33 Ghi cách làm tròn quãng đường vào tài liệu
* 7993c73 Làm tròn quãng đường tới 0,1 km trước khi tính cước
* 89855d5 Chuẩn hoá xuống dòng LF bằng .gitattributes
```

`git commit -am` = stage mọi file **đã theo dõi** có thay đổi rồi commit (GIT-005).

## 3. Merge commit với `--no-ff`

Nhánh thứ hai, `tai-lieu/gio-cao-diem`, cũng chỉ có một commit và `main` cũng chưa đi tiếp: mặc định Git lại fast-forward. Nhưng nhóm muốn lịch sử ghi lại rằng tài liệu giờ cao điểm là một **đơn vị công việc** riêng, để sau này revert cả cụm hay đọc lịch sử theo tính năng. `--no-ff` ép tạo merge commit:

```sh
git switch -c tai-lieu/gio-cao-diem
git commit -am "Mô tả phụ phí giờ cao điểm trong tài liệu"
git switch main
git merge --no-ff --no-edit tai-lieu/gio-cao-diem
git log --oneline --graph -4
```

```text
[tai-lieu/gio-cao-diem 3e7f15a] Mô tả phụ phí giờ cao điểm trong tài liệu
 1 file changed, 4 insertions(+)
Merge made by the 'ort' strategy.
 docs/bang-gia-cuoc.md | 4 ++++
 1 file changed, 4 insertions(+)
*   8c05257 Merge branch 'tai-lieu/gio-cao-diem'
|\  
| * 3e7f15a Mô tả phụ phí giờ cao điểm trong tài liệu
|/  
* efe9c33 Ghi cách làm tròn quãng đường vào tài liệu
* 7993c73 Làm tròn quãng đường tới 0,1 km trước khi tính cước
```

`ort` là thuật toán merge mặc định của Git hiện đại. `--no-edit` dùng nội dung mặc định `Merge branch '…'` thay vì mở trình soạn thảo. Merge commit trông thế nào bên trong:

```sh
git cat-file -p HEAD
```

```text
tree dc34c543d086ef04e66d59a1fcf1209085c94091
parent efe9c33cf463dead481a14f4c239963fbef6c7f9
parent 3e7f15acc7c9d68fa78d67c8fb4ceced44b27ed7
author Trần Minh <minh@toinoi.test> 1794817800 +0700
committer Trần Minh <minh@toinoi.test> 1794817800 +0700

Merge branch 'tai-lieu/gio-cao-diem'
```

Hai dòng `parent`: cha thứ nhất là `main` lúc merge, cha thứ hai là nhánh được merge vào. Thứ tự này quan trọng khi revert merge commit (Phần 4) hay xem lịch sử theo `--first-parent`.

| | Fast-forward | Merge commit (`--no-ff`) |
|---|---|---|
| Lịch sử | Thẳng, gọn | Có "bong bóng" cho mỗi nhánh |
| Biết commit nào thuộc tính năng nào | Không | Có (giữa merge commit và điểm tách) |
| Hoàn tác cả tính năng | Revert từng commit | Revert một merge commit |
| Hợp với | Sửa nhỏ, một commit | Tính năng nhiều commit |

Nhóm Tới Nơi dùng `--no-ff` cho nhánh `tinh-nang/…` và để fast-forward cho sửa nhỏ. Phần 3 bàn thêm khi merge qua pull request.

## 4. Merge trên GitStudio

Chuột phải nhánh muốn merge vào (ở thanh bên hoặc nhãn trên đồ thị), hoặc chuột phải nhánh nhận: menu có cả **Fast-forward main tới …**, **Merge … vào main**, **Rebase … lên main**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-merge-fast-forward-merge-commit/gitstudio-01.webp" alt="Menu chuột phải của nhánh main trong GitStudio khi đang ở nhánh tính năng: Push, Đặt upstream, Fast-forward main tới tinh-nang/phu-phi-gio-cao-diem, Merge tinh-nang/phu-phi-gio-cao-diem vào main, Rebase tinh-nang/phu-phi-gio-cao-diem lên main, Checkout main, Tạo worktree, Reset, Revert, Đổi tên, Xoá, So sánh, Tạo tag" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · menu của một nhánh: fast-forward, merge, rebase và checkout ở cùng một chỗ</figcaption></figure>

Chọn **Merge … vào main**, GitStudio mở **Xem trước merge**: danh sách file thay đổi, số xung đột (nếu có), và ô chọn kiểu merge ở góc trên với ba lựa chọn: "Fast-forward nếu được, nếu không thì tạo merge commit" (mặc định, như `git merge`), **Luôn tạo merge commit (--no-ff)**, và Squash (dồn mọi thay đổi vào staging area để bạn tự commit thành một commit thường, như `git merge --squash`). Bấm **Merge** để thực hiện. Khi không có xung đột, merge chạy ngay; GIT-014 dùng màn hình này để giải quyết xung đột.

Sau merge `--no-ff`, đồ thị có bong bóng của nhánh tài liệu. Nhãn nhánh đã xoá còn hiện mờ một lúc ở commit cuối của nó:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-merge-fast-forward-merge-commit/gitstudio-02.webp" alt="Đồ thị GitStudio: main ở commit Merge branch tai-lieu/gio-cao-diem, nhánh phụ chứa commit Mô tả phụ phí giờ cao điểm trong tài liệu của Lê Lan nhập lại vào main; bên dưới là commit fast-forward Ghi cách làm tròn quãng đường vào tài liệu và nhánh tính năng giờ cao điểm vẫn tách riêng" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · merge commit tạo "bong bóng"; commit fast-forward nằm thẳng trên <code>main</code></figcaption></figure>

## 5. Nhánh nào đã merge

```sh
git branch -d tai-lieu/gio-cao-diem
git branch --merged main
git branch --no-merged main
```

```text
Deleted branch tai-lieu/gio-cao-diem (was 3e7f15a).
* main
  tinh-nang/phu-phi-gio-cao-diem
```

`--merged main`: nhánh có commit cuối đã nằm trong lịch sử của `main` (xoá an toàn). `--no-merged`: nhánh còn commit chưa vào `main`. Nhánh tính năng giờ cao điểm vẫn chưa merge; đó là việc của GIT-014. Thói quen tốt: merge xong thì xoá nhánh ngay, để danh sách nhánh chỉ còn việc đang làm.

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Merge nhánh vào nhánh hiện tại | Chuột phải nhánh → Merge … vào main → Merge | `git merge <nhánh>` |
| Chỉ cho phép fast-forward | Chuột phải main → Fast-forward main tới … | `git merge --ff-only <nhánh>` |
| Luôn tạo merge commit | Xem trước merge → Luôn tạo merge commit (--no-ff) | `git merge --no-ff <nhánh>` |
| Gộp thành một commit thường | Xem trước merge → Squash | `git merge --squash <nhánh>` rồi `git commit` |
| Xem trước thay đổi | Xem trước merge | `git diff main...<nhánh>` |
| Nhánh đã/chưa merge | | `git branch --merged` / `--no-merged` |
| Xoá nhánh đã merge | Chuột phải → Xoá | `git branch -d <nhánh>` |

## 7. Lỗi thường gặp

- **Đứng nhầm nhánh khi merge.** `git merge main` khi đang ở nhánh tính năng sẽ đưa `main` *vào nhánh tính năng*, không phải ngược lại. Kiểm ô **nhánh** trước khi merge.
- **Lịch sử đầy merge commit "Merge branch 'main' into …".** Mỗi lần cập nhật nhánh tính năng bằng merge `main` vào nó lại tạo một merge commit. Có thể chấp nhận, hoặc dùng rebase (GIT-018) khi nhánh chưa chia sẻ.
- **Merge commit có nội dung mặc định khi lẽ ra cần giải thích.** Bỏ `--no-edit` để viết lý do, nhất là khi merge có xung đột.
- **Xoá nhánh bằng `-D` cho nhanh.** `-D` bỏ qua kiểm tra đã merge; dùng `-d` để Git nhắc khi còn commit chưa merge.

## 8. Bài tập

**Bài 1.** Ngay sau mục 2, chạy `git reflog -2`. Hai dòng ghi gì và vì sao không có commit mới?

<details><summary><b>Lời giải</b></summary><p>Dòng trên là <code>merge sua/tai-lieu-lam-tron: Fast-forward</code> ở <code>efe9c33</code>, dòng dưới là lần checkout <code>main</code> ở <code>7993c73</code>. Fast-forward chỉ dời <code>main</code> (và HEAD) tới commit có sẵn của nhánh nguồn, không tạo commit.</p></details>

**Bài 2.** `git log --oneline --first-parent -3` sau mục 3 in ra gì? Khác `git log --oneline -3` thế nào?

<details><summary><b>Lời giải</b></summary><p><code>8c05257</code>, <code>efe9c33</code>, <code>7993c73</code>: chỉ đi theo cha thứ nhất, bỏ qua commit bên trong nhánh tài liệu. Không có <code>--first-parent</code>, <code>3e7f15a</code> cũng xuất hiện. <code>--first-parent</code> cho cái nhìn "main đã nhận những việc gì".</p></details>

**Bài 3.** Thử `git merge --ff-only tinh-nang/phu-phi-gio-cao-diem` trên `main`. Kết quả và lý do?

<details><summary><b>Lời giải</b></summary><p>Thất bại với <code>fatal: Not possible to fast-forward, aborting.</code>, kèm gợi ý <code>Diverging branches can't be fast-forwarded</code> và hai hướng <code>git merge --no-ff</code> hoặc <code>git rebase</code>: <code>main</code> đã có commit riêng (bản vá, tài liệu) mà nhánh tính năng không có, nên không thể chỉ dời con trỏ. Cần merge commit (GIT-014) hoặc rebase nhánh tính năng lên <code>main</code> trước (GIT-018).</p></details>

## 9. Đọc tiếp

- [git-merge](https://git-scm.com/docs/git-merge): `--ff`, `--no-ff`, `--ff-only`, chiến lược `ort`.
- [Pro Git, Basic Branching and Merging](https://git-scm.com/book/en/v2/Git-Branching-Basic-Branching-and-Merging).

Bài tiếp theo: [GIT-014 · Xung đột merge và công cụ giải quyết xung đột](/articles/git-xung-dot-merge).
