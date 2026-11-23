---
slug: git-rebase-onto
title: "Rebase --onto: chuyển chuỗi commit sang gốc khác khi lỡ tạo nhánh sai chỗ"
summary: "Minh tạo nhánh sửa lỗi phí chờ trong lúc vẫn đứng ở nhánh phụ phí đêm, nên nhánh sửa lỗi kéo theo hai commit tính năng chưa review. Bài này đọc cú pháp git rebase --onto <gốc mới> <gốc cũ> <nhánh>, chuyển đúng một commit sang main, kiểm tra bằng log main..nhánh, merge fast-forward rồi push; thêm các cách dùng khác của --onto (bỏ một đoạn commit, chuyển nhánh con khi nhánh cha bị squash) và cách làm tương đương trên GitStudio."
category: git
level: advanced
tags: [git, git-038, rebase-onto, branching, gitstudio, toi-noi]
language: vi
---

10 giờ sáng 25/11/2026, Lan báo lỗi: app tài xế gửi thời gian chờ tính bằng **giây**, còn hàm tính phí chờ nhận **phút**. Minh sửa ngay: tạo nhánh `sua/lam-tron-giay-cho`, thêm hàm làm tròn giây lên phút, commit. Rồi anh nhìn lại và thấy mình tạo nhánh trong lúc đang đứng ở `tinh-nang/phu-phi-dem`.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-038 · Phần 4/6 · Nâng cao</span>
<p>Đọc trước: <a href="/articles/git-rebase-tuong-tac">GIT-037</a>, <a href="/articles/git-rebase-co-ban">GIT-018</a>. Lab: <code>npm run p04 -- 38</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 38</code>. Sau bài này, bạn chuyển được một chuỗi commit sang gốc khác và nhận ra tình huống cần <code>--onto</code>.</p>
</div>

## 1. Nhánh mọc sai chỗ

```sh
git branch --show-current
git switch -c sua/lam-tron-giay-cho
git commit -am "Tính phí chờ từ số giây, làm tròn lên phút"
git log --oneline main..sua/lam-tron-giay-cho
```

```text
tinh-nang/phu-phi-dem
Switched to a new branch 'sua/lam-tron-giay-cho'
[sua/lam-tron-giay-cho e2cc862] Tính phí chờ từ số giây, làm tròn lên phút
 1 file changed, 4 insertions(+)
e2cc862 Tính phí chờ từ số giây, làm tròn lên phút
40d9795 Thêm test phụ phí đêm
3158035 Thêm phụ phí đêm
```

`git switch -c` tạo nhánh **tại HEAD** khi không ghi điểm bắt đầu. Nhánh sửa lỗi giờ chứa cả hai commit phụ phí đêm; merge nó vào `main` là đưa luôn tính năng chưa review lên production.

`git rebase main` không giúp được: rebase chép **mọi** commit có trên nhánh mà `main` chưa có, tức cả ba.

## 2. `--onto`: chép từ gốc cũ sang gốc mới

```text
git rebase --onto <gốc mới> <gốc cũ> <nhánh>
```

Lấy các commit trong khoảng `<gốc cũ>..<nhánh>` (có trong nhánh, không có trong gốc cũ), chép lần lượt lên `<gốc mới>`, rồi dời `<nhánh>` tới bản chép cuối cùng.

```sh
git rebase --onto main tinh-nang/phu-phi-dem sua/lam-tron-giay-cho
git log --oneline main..sua/lam-tron-giay-cho
git log --oneline main..tinh-nang/phu-phi-dem
```

```text
Successfully rebased and updated refs/heads/sua/lam-tron-giay-cho.
8a0b649 Tính phí chờ từ số giây, làm tròn lên phút
40d9795 Thêm test phụ phí đêm
3158035 Thêm phụ phí đêm
```

Nhánh sửa lỗi chỉ còn đúng một commit, nằm trên `main` (mã mới `8a0b649` vì cha mới). Nhánh tính năng không bị đụng tới.

```text
  trước                               sau
  main ─ A ─ B  (tinh-nang)           main ─ F'           (sua/…)
              └ F  (sua/…)                 └ A ─ B        (tinh-nang)
```

## 3. Đưa bản sửa vào main

Sửa lỗi nhỏ, đã có người xem: fast-forward, push, xoá nhánh.

```sh
git switch main
git merge --ff-only sua/lam-tron-giay-cho
git push
git branch -d sua/lam-tron-giay-cho
```

```text
Updating c19b370..8a0b649
Fast-forward
 src/BaoGia/WaitingFee.cs | 4 ++++
 1 file changed, 4 insertions(+)
   c19b370..8a0b649  main -> main
Deleted branch sua/lam-tron-giay-cho (was 8a0b649).
```

## 4. Các tình huống khác cần `--onto`

- **Bỏ một đoạn commit ở giữa**: `git rebase --onto <commit trước đoạn> <commit cuối đoạn>` chép các commit sau đoạn đó lên commit trước đoạn, tức bỏ đoạn khỏi nhánh hiện tại. Ví dụ `git rebase --onto X~3 X` bỏ X cùng hai commit ngay trước nó. GIT-036 dùng đúng dạng này để bỏ một commit.
- **Nhánh con khi nhánh cha bị squash merge**: Lan làm `tinh-nang/b` trên `tinh-nang/a`; `a` được squash merge vào `main` (GIT-026), các commit của `a` không còn trên `main`. `git rebase main` trên `b` sẽ chép lại cả commit của `a` và xung đột. Đúng là: `git rebase --onto main tinh-nang/a tinh-nang/b` (dùng `a` trước khi xoá, hoặc mã commit cuối của `a`).
- **Chuyển nhánh tính năng sang nhánh phát hành**: `git rebase --onto phat-hanh/1.0 main sua/x` chép các commit của `sua/x` (tính từ `main`) sang nhánh phát hành. Thường cherry-pick (GIT-017) rõ ràng hơn.

## 5. Trên GitStudio

GitStudio không có lệnh `--onto` riêng. Hai cách tương đương trên giao diện:

- **Cherry-pick rồi bỏ nhánh sai**: tạo nhánh mới từ `main` (chuột phải `main` → Tạo nhánh tại đây), chọn commit sửa lỗi trên nhánh sai → **Cherry-pick commit**, xoá nhánh sai. Hợp với một vài commit.
- **Rebase tương tác** trên nhánh sai, chọn **Bỏ** cho các commit tính năng, rồi kéo nhánh lên `main` → Rebase. Hai bước, cùng kết quả.

Với chuỗi dài, gõ `git rebase --onto` trong terminal tích hợp (`Ctrl+\``) là nhanh nhất; đồ thị cập nhật ngay sau đó.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-rebase-onto/gitstudio-01.webp" alt="Đồ thị GitStudio trước khi sửa: nhánh sua/lam-tron-giay-cho ở commit Tính phí chờ từ số giây, làm tròn lên phút, nằm ngay trên hai commit của tinh-nang/phu-phi-dem, tất cả phía trên main" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · nhánh sửa lỗi mọc trên nhánh tính năng</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-rebase-onto/gitstudio-02.webp" alt="Terminal tích hợp của GitStudio chạy git rebase --onto main tinh-nang/phu-phi-dem sua/lam-tron-giay-cho; đồ thị phía trên cho thấy sua/lam-tron-giay-cho tách khỏi nhánh tính năng, nằm trực tiếp trên main" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · sau rebase --onto: nhánh sửa lỗi nằm thẳng trên main</figcaption></figure>

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Chuyển chuỗi commit sang gốc khác | Terminal; hoặc Cherry-pick từng commit sang nhánh mới | `git rebase --onto <gốc mới> <gốc cũ> <nhánh>` |
| Xem nhánh có gì so với main | Chọn nhánh trên đồ thị | `git log --oneline main..<nhánh>` |
| Merge chỉ khi fast-forward được | | `git merge --ff-only <nhánh>` |
| Quay lại nếu sai | Hoàn tác (nếu làm trong GitStudio); reflog | `git reset --hard <nhánh>@{1}` |

## 7. Lỗi thường gặp

- **Nhầm thứ tự hai gốc**: `git rebase --onto tinh-nang main sua/…` chép mọi commit sau `main` (cả tính năng lẫn sửa lỗi) lên nhánh tính năng. Đọc lại: gốc **mới** trước, gốc **cũ** sau.
- **Quên tham số nhánh**: không ghi `<nhánh>` thì Git dùng nhánh hiện tại. Đứng nhầm nhánh là chuyển nhầm.
- **Tránh lỗi ngay từ đầu**: luôn ghi điểm bắt đầu khi tạo nhánh: `git switch -c sua/x main` (GitStudio: chuột phải `main` → Tạo nhánh tại đây).
- **Dùng `--onto` trên nhánh đã push**: như mọi rebase, cần force-with-lease và báo người dùng chung.

## 8. Bài tập

**Bài 1.** Nhánh `x` có năm commit sau `main`; bạn muốn bỏ commit thứ hai và thứ ba (tính từ cũ nhất). Viết một lệnh `--onto`.

<details><summary><b>Lời giải</b></summary><p>Gọi các commit là c1…c5. <code>git rebase --onto c1 c3 x</code>: chép c4, c5 lên c1, bỏ c2, c3.</p></details>

**Bài 2.** Sau GIT-026, `tinh-nang/tinh-phi-cho` bị squash merge và xoá. Lan có `tinh-nang/thong-bao-phi-cho` tạo từ nhánh đó, commit cuối của nhánh cũ là `d0c8e3b`. Lệnh nào?

<details><summary><b>Lời giải</b></summary><p><code>git rebase --onto main d0c8e3b tinh-nang/thong-bao-phi-cho</code>.</p></details>

**Bài 3.** Làm lại mục 2 bằng cherry-pick thay vì `--onto`.

<details><summary><b>Lời giải</b></summary><p><code>git switch -c sua/lam-tron-giay-cho-2 main</code>, <code>git cherry-pick e2cc862</code>, <code>git branch -D sua/lam-tron-giay-cho</code>, rồi đổi tên nhánh mới nếu cần (<code>git branch -m</code>).</p></details>

## 9. Đọc tiếp

- [git-rebase: `--onto`](https://git-scm.com/docs/git-rebase#Documentation/git-rebase.txt---ontoltnewbasegt), [Pro Git, More Interesting Rebases](https://git-scm.com/book/en/v2/Git-Branching-Rebasing).

Bài tiếp theo: [GIT-039 · Patch, rerere và merge lại nhiều lần](/articles/git-patch-rerere).
