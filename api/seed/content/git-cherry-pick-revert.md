---
slug: git-cherry-pick-revert
title: "Cherry-pick và revert: mang bản vá sang nhánh phát hành, hoàn tác một commit đã chia sẻ"
summary: "Server production vẫn chạy bản 1.0 nhưng cần bản vá làm tròn quãng đường, còn bộ phận vận hành lại hoãn phụ phí giờ cao điểm. Bài này tạo nhánh phát hành từ tag v1.0.0, cherry-pick -x bản vá kèm ghi nguồn rồi gắn v1.0.1, và revert commit cấu hình giờ cao điểm trên main; làm bằng menu commit của GitStudio và bằng lệnh, kèm khi nào không nên dùng hai thao tác này."
category: git
level: intermediate
tags: [git, git-017, cherry-pick, git-revert, release-branch, hotfix, gitstudio, toi-noi]
language: vi
---

Sáng 18/11/2026, hai tin đến cùng lúc. Một: server production của một thành phố vẫn chạy **v1.0.0** và chưa nâng lên v1.1.0 được trong tuần này, nhưng khách ở đó cũng bị giá nhảy khi xe đứng yên. Cần bản vá làm tròn quãng đường (`7993c73`, GIT-012) cho riêng bản 1.0, **không kèm** phụ phí giờ cao điểm. Hai: bộ phận vận hành hoãn phụ phí giờ cao điểm tới tháng sau; `main` phải bỏ cấu hình giờ cao điểm, nhưng lịch sử thì vẫn cần giữ.

Việc thứ nhất là **cherry-pick**: chép một commit sang nhánh khác. Việc thứ hai là **revert**: tạo commit mới làm ngược lại một commit cũ.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-017 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-tag-phien-ban">GIT-016</a>. Lab: <code>npm run p02 -- 17</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 17</code>. Sau bài này, bạn tạo được nhánh phát hành từ tag, cherry-pick có ghi nguồn, revert một commit, và biết hai thao tác này làm gì với mã hash và lịch sử.</p>
</div>

## 1. Cherry-pick và revert là gì

| | Cherry-pick | Revert |
|---|---|---|
| Đầu vào | Một commit ở nơi khác | Một commit đã có trong lịch sử |
| Kết quả | Commit **mới** trên nhánh hiện tại, áp **cùng thay đổi** | Commit **mới** áp thay đổi **ngược lại** |
| Commit gốc | Giữ nguyên | Giữ nguyên |
| Mã hash | Khác commit gốc (cha khác, thời điểm khác) | Mới |
| Viết lại lịch sử? | Không | Không |

Cả hai đều chỉ **thêm** commit, nên dùng an toàn trên nhánh đã chia sẻ, khác với amend hay reset (Phần 4).

## 2. Nhánh phát hành từ tag

Bản vá phải dựa trên code của v1.0.0, nên tạo nhánh từ tag đó:

```sh
git switch -c phat-hanh/1.0 v1.0.0
```

```text
Switched to a new branch 'phat-hanh/1.0'
```

Không tạo nhánh mà checkout thẳng tag (`git switch --detach v1.0.0`) thì commit mới không thuộc nhánh nào (detached HEAD).

## 3. Cherry-pick có ghi nguồn

```sh
git cherry-pick -x 7993c73
git log -1 --format="%h %s%n%n%b"
git log -1 --format="tác giả: %an %ad%nngười commit: %cn %cd" --date=iso
```

```text
[phat-hanh/1.0 b047783] Làm tròn quãng đường tới 0,1 km trước khi tính cước
 Date: Mon Nov 16 11:15:00 2026 +0700
 2 files changed, 5 insertions(+), 1 deletion(-)
b047783 Làm tròn quãng đường tới 0,1 km trước khi tính cước

GPS trả quãng đường tới mét; khách thấy giá nhảy khi xe đứng yên.

(cherry picked from commit 7993c73f77906dbc35746cc946496c2e42025c69)
tác giả: Trần Minh 2026-11-16 11:15:00 +0700
người commit: Trần Minh 2026-11-18 09:00:00 +0700
```

Ba điều đáng chú ý:

- **Mã hash mới** (`b047783` ≠ `7993c73`): cùng thay đổi, nhưng commit cha khác (`89855d5` thay vì commit trên `main`) và thời điểm commit khác.
- **Tác giả và ngày viết giữ nguyên** (16/11 11:15), **người commit và ngày commit là lúc cherry-pick** (18/11 09:00). Đây là chỗ hai danh tính author/committer (GIT-002) có ý nghĩa thật.
- **`-x`** thêm dòng `(cherry picked from commit …)`. Ai đọc nhánh phát hành cũng lần ra được bản gốc trên `main`. Luôn dùng `-x` khi cherry-pick từ nhánh đã chia sẻ.

Gắn tag bản vá:

```sh
git tag -a v1.0.1 -m "Phiên bản 1.0.1: bản vá làm tròn quãng đường"
git log --oneline --graph --decorate -3
```

```text
* b047783 (HEAD -> phat-hanh/1.0, tag: v1.0.1) Làm tròn quãng đường tới 0,1 km trước khi tính cước
* 89855d5 (tag: v1.0.0) Chuẩn hoá xuống dòng LF bằng .gitattributes
* 1c6e78c Đổi tên tài liệu thành bang-gia-cuoc.md
```

Cherry-pick có thể gặp xung đột nếu đoạn code đã khác giữa hai nhánh; cách giải quyết y như GIT-014, rồi `git cherry-pick --continue` (hoặc `--abort`).

## 4. Trên GitStudio

Checkout nhánh nhận (`phat-hanh/1.0`), rồi chuột phải commit muốn chép trên đồ thị → **Cherry-pick commit**. Cùng menu có **Revert commit**, **Tạo nhánh tại đây**, **Tạo annotated tag tại đây**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-cherry-pick-revert/gitstudio-01.webp" alt="Đồ thị GitStudio đang ở nhánh phat-hanh/1.0 (nhãn có dấu tích ở commit v1.0.0); chuột phải commit Làm tròn quãng đường tới 0,1 km trên main mở menu: Checkout, Tạo worktree từ, Tạo nhánh tại đây, Bisect, Reset, Sửa nội dung commit, Revert commit, Cherry-pick commit, Bỏ commit, Áp dụng patch, So sánh, Tạo tag" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · menu commit: <b>Cherry-pick commit</b> và <b>Revert commit</b></figcaption></figure>

Chọn nhiều commit (`Ctrl` + bấm) thì bảng bên phải có nút **Cherry-pick N commit** để chép cả loạt theo đúng thứ tự.

Lưu ý với GitStudio 1.0.1: **Cherry-pick commit** chạy `git cherry-pick <commit>` **không có `-x`**, nên commit mới không có dòng `(cherry picked from commit …)`. Với nhánh phát hành, nơi cần lần ra bản gốc, hãy cherry-pick bằng terminal tích hợp (`git cherry-pick -x …`), hoặc sau khi cherry-pick trên giao diện thì sửa nội dung commit (chuột phải → **Sửa nội dung commit**) để thêm dòng nguồn. Tương tự, **Revert commit** chạy `git revert --no-edit`: nội dung là mẫu tiếng Anh mặc định, muốn ghi lý do thì sửa nội dung commit ngay sau đó.

## 5. Revert trên `main`

Bỏ cấu hình giờ cao điểm mà không xoá lịch sử: revert commit `c46c1dd` (thêm khối `Peak` vào `appsettings.json`).

```sh
git switch main
git revert --no-edit c46c1dd
git log -1 --format="%h %s%n%n%b"
git show --stat --format= HEAD
```

```text
[main b6829e3] Revert "Thêm hệ số phụ phí giờ cao điểm vào bảng giá"
 Date: Wed Nov 18 14:00:00 2026 +0700
 1 file changed, 4 deletions(-)
b6829e3 Revert "Thêm hệ số phụ phí giờ cao điểm vào bảng giá"

This reverts commit c46c1dda9d78f686ef8c9b9f55b99e4150089523.
 src/BaoGia/appsettings.json | 4 ----
 1 file changed, 4 deletions(-)
```

Commit revert xoá đúng 4 dòng mà `c46c1dd` đã thêm. Code tính cước vẫn nhận tham số `peakMultiplier` với mặc định 1, nên không có cấu hình thì không có phụ phí. Nội dung mặc định (`Revert "…"`, `This reverts commit …`) bằng tiếng Anh; bỏ `--no-edit` để Git mở trình soạn thảo và **thêm lý do**, ví dụ "Bộ phận vận hành hoãn phụ phí giờ cao điểm tới 01/12/2026."

Trên GitStudio: chuột phải commit → **Revert commit**. Kết quả:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-cherry-pick-revert/gitstudio-02.webp" alt="Đồ thị GitStudio sau GIT-017: main ở commit Revert Thêm hệ số phụ phí giờ cao điểm vào bảng giá, nhánh phat-hanh/1.0 kèm tag v1.0.1 ở commit cherry-pick Làm tròn quãng đường xuất phát từ v1.0.0; bảng chi tiết hiện mô tả với dòng cherry picked from commit 7993c73…" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · nhánh phát hành 1.0 với bản vá cherry-pick, <code>main</code> với commit revert</figcaption></figure>

Revert một **merge commit** cần chỉ định cha nào là "dòng chính" (`git revert -m 1 <merge>`), và có hệ quả khi merge lại nhánh đó sau này. Phần 4 bàn riêng.

## 6. Khi nào không nên

- **Cherry-pick để đồng bộ hai nhánh dài hạn.** Mỗi lần chép lại tạo commit trùng nội dung khác mã hash; khi merge hai nhánh, Git thường nhận ra thay đổi đã có, nhưng lịch sử thành rối và đôi khi xung đột lại. Đồng bộ thường xuyên thì merge.
- **Cherry-pick ngược hướng.** Quy ước an toàn: sửa trên `main` trước, rồi cherry-pick xuống nhánh phát hành cũ. Sửa trên nhánh phát hành rồi quên chép lên `main` là lỗi quay lại ở bản sau.
- **Revert để "dọn" commit chưa push.** Commit chưa chia sẻ thì sửa hẳn (amend, reset, rebase tương tác ở Phần 4) cho lịch sử gọn; revert dành cho commit **đã** chia sẻ.

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Nhánh từ tag | Chuột phải tag/commit → Tạo nhánh tại đây | `git switch -c <nhánh> <tag>` |
| Cherry-pick một commit | Chuột phải commit → Cherry-pick commit (không có `-x`) | `git cherry-pick -x <commit>` |
| Cherry-pick nhiều commit | `Ctrl` + bấm → Cherry-pick N commit | `git cherry-pick -x A B C` |
| Revert | Chuột phải commit → Revert commit | `git revert <commit>` |
| Xung đột giữa chừng | Banner → Tiếp tục / Huỷ thao tác | `--continue` / `--abort` |

## 8. Lỗi thường gặp

- **Quên `-x`.** Không ai biết commit trên nhánh phát hành là bản sao của commit nào trên `main`.
- **Cherry-pick một commit phụ thuộc commit trước nó.** Commit dùng hàm được thêm ở commit khác: áp được nhưng build hỏng, hoặc xung đột. Cherry-pick theo cụm, đúng thứ tự.
- **Revert nhầm commit.** Revert cái revert (`git revert <commit-revert>`) để lấy lại; lịch sử ghi đủ cả hai bước.
- **Để nguyên nội dung revert mặc định.** Lịch sử ghi "Revert …" mà không có lý do; người sau không biết khi nào được bật lại.

## 9. Bài tập

**Bài 1.** Sau khi cherry-pick, chạy `git log --oneline --cherry-pick --right-only main...phat-hanh/1.0`. Commit `b047783` có xuất hiện không? Vì sao?

<details><summary><b>Lời giải</b></summary><p>Không. <code>--cherry-pick</code> bỏ những commit có cùng thay đổi (cùng "patch-id") ở cả hai phía; <code>b047783</code> có cùng thay đổi với <code>7993c73</code> trên <code>main</code>. Đây là cách kiểm "bản vá nào trên nhánh phát hành chưa có trên main".</p></details>

**Bài 2.** Bộ phận vận hành bật lại phụ phí giờ cao điểm vào 01/12. Làm thế nào bằng một lệnh?

<details><summary><b>Lời giải</b></summary><p><code>git revert b6829e3</code>: revert chính commit revert, khối <code>Peak</code> quay lại. Lịch sử ghi rõ: thêm (16/11), hoãn (18/11), bật lại (01/12).</p></details>

**Bài 3.** Vì sao cherry-pick giữ nguyên ngày tác giả nhưng đổi ngày commit? Ích lợi gì khi điều tra sự cố?

<details><summary><b>Lời giải</b></summary><p>Tác giả là người viết thay đổi và lúc viết; người commit là người ghi nó vào nhánh này và lúc ghi. Khi điều tra, ngày tác giả cho biết bản vá được viết khi nào (16/11), ngày commit cho biết nó vào nhánh phát hành khi nào (18/11), tức khoảng thời gian production chưa có bản vá.</p></details>

## 10. Đọc tiếp

- [git-cherry-pick](https://git-scm.com/docs/git-cherry-pick): `-x`, `--continue`, nhiều commit.
- [git-revert](https://git-scm.com/docs/git-revert): `-m` cho merge commit.
- [Pro Git, Maintaining a Project](https://git-scm.com/book/en/v2/Distributed-Git-Maintaining-a-Project).

Bài tiếp theo: [GIT-018 · Rebase cơ bản: viết lại nhánh lên đầu main](/articles/git-rebase-co-ban).
