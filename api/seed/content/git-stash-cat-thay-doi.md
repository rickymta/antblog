---
slug: git-stash-cat-thay-doi
title: "Stash: cất thay đổi dở dang để làm việc khác, rồi lấy lại đúng chỗ"
summary: "Minh đang viết dở test phụ phí giờ cao điểm thì Lan cần anh chạy lại test trên code sạch. Bài này dùng git stash push có tiêu đề, stash list và show, pop khác apply thế nào, -u cho file chưa theo dõi, --staged chỉ cất phần đã stage, và tab Stash, nút Pop, hàng stash trên đồ thị của GitStudio."
category: git
level: intermediate
tags: [git, git-015, git-stash, stash-pop, stash-apply, untracked, gitstudio, toi-noi]
language: vi
---

9:40 sáng 17/11/2026, ngay sau khi merge nhánh giờ cao điểm, Minh viết test cho phụ phí: một test trong `FareCalculatorTests.cs` đã xong một nửa, cộng một file `ghi-chu-test.md` ghi những trường hợp cần thêm. Lan nhắn: "Test `Quote_ChuyenNgan_TraGiaToiThieu` đỏ trên máy mình. Có phải do merge vừa rồi không? Chạy thử trên `main` sạch giúp mình."

Chạy test trên code sạch nghĩa là working tree không được có thay đổi của Minh. Commit nửa test thì không đúng (GIT-005: mỗi commit phải có nghĩa và build được). Xoá đi thì mất công. Đây là việc của **stash**: cất thay đổi chưa commit vào một chỗ riêng, trả working tree về đúng commit hiện tại, rồi lấy lại sau.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-015 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-xung-dot-merge">GIT-014</a>. Lab: <code>npm run p02 -- 15</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 15</code>. Sau bài này, bạn stash có tiêu đề, xem và lấy lại stash, phân biệt pop với apply, stash cả file chưa theo dõi hoặc chỉ phần đã stage, và dùng stash trong GitStudio.</p>
</div>

## 1. Stash là gì

Một **stash** là một commit đặc biệt (thực ra là hai hoặc ba commit nhỏ) lưu trạng thái working tree và staging area, không nằm trên nhánh nào, xếp thành một ngăn xếp `stash@{0}`, `stash@{1}`… (`{0}` là mới nhất). Ref `refs/stash` trỏ vào stash mới nhất.

Mặc định stash cất:

| Loại thay đổi | `git stash` | `git stash -u` | `git stash -a` |
|---|---|---|---|
| File đã theo dõi, chưa stage | Có | Có | Có |
| File đã stage | Có | Có | Có |
| File **untracked** | Không | Có | Có |
| File bị `.gitignore` | Không | Không | Có |

## 2. Cất, xem, lấy lại bằng lệnh

```sh
git status --short
git stash push -m "Test phụ phí giờ cao điểm dở dang"
git status --short
git stash list
git stash show --stat stash@{0}
```

```text
 M tests/BaoGia.Tests/FareCalculatorTests.cs
?? ghi-chu-test.md
Saved working directory and index state On main: Test phụ phí giờ cao điểm dở dang
?? ghi-chu-test.md
stash@{0}: On main: Test phụ phí giờ cao điểm dở dang
 tests/BaoGia.Tests/FareCalculatorTests.cs | 4 ++++
 1 file changed, 4 insertions(+)
```

Luôn đặt tiêu đề (`-m`): ba ngày sau, `stash@{2}: WIP on main: e46c89f Merge branch…` không nói được gì. Sau khi stash, `FareCalculatorTests.cs` trở về như commit `HEAD`; file ghi chú vẫn còn vì là untracked. Trong PowerShell nên viết `'stash@{0}'` trong nháy đơn (dấu `{}` có nghĩa riêng trong PowerShell).

Minh chạy test trên code sạch (test của Lan đỏ vì máy cô chưa pull bản mới, không phải do merge), rồi lấy thay đổi về:

```sh
git stash pop
git status --short
git stash list
```

```text
 M tests/BaoGia.Tests/FareCalculatorTests.cs
?? ghi-chu-test.md
```

`pop` = áp stash vào working tree **rồi xoá** stash đó; `stash list` giờ trống. `apply` = áp nhưng **giữ** stash, hữu ích khi muốn áp cùng thay đổi lên nhiều nhánh hoặc chưa chắc áp có trơn tru.

| Lệnh | Áp thay đổi | Xoá stash |
|---|---|---|
| `git stash pop` | Có | Có (nếu áp không xung đột) |
| `git stash apply` | Có | Không |
| `git stash drop` | Không | Có |
| `git stash clear` | Không | Xoá **tất cả** |

Khi áp stash gặp xung đột (code đã đổi kể từ lúc stash), `pop` vẫn giữ stash lại để bạn không mất gì; giải quyết như GIT-014 rồi `git stash drop` bằng tay.

## 3. Cất cả file chưa theo dõi, hoặc chỉ phần đã stage

```sh
git stash push -u -m "Kèm ghi chú"
git status --short
git stash apply
git stash list
git stash drop
```

```text
Saved working directory and index state On main: Kèm ghi chú
stash@{0}: On main: Kèm ghi chú
```

Với `-u`, `git status --short` sau khi stash không còn dòng nào: cả file ghi chú cũng được cất. Sau `apply`, stash vẫn nằm trong danh sách cho tới khi `drop`.

Git 2.35 trở lên cất được **chỉ phần đã stage**, để lại phần chưa stage trong working tree:

```sh
git add tests/BaoGia.Tests/FareCalculatorTests.cs
git stash push --staged -m "Chỉ phần đã stage"
git status --short
git stash pop --index
git status --short
```

```text
Saved working directory and index state On main: Chỉ phần đã stage
?? ghi-chu-test.md
M  tests/BaoGia.Tests/FareCalculatorTests.cs
?? ghi-chu-test.md
```

`pop --index` khôi phục cả trạng thái staging: file trở lại `M ` (đã stage) thay vì ` M`. Không có `--index`, mọi thứ quay về dạng chưa stage.

## 4. Stash trên GitStudio

Bảng commit có hai tab **Commit** và **Stash**. Tab Stash có ô **Tiêu đề stash** và **Mô tả**, và giống tab Commit, nó làm việc với **phần đã stage**: stage những file muốn cất, nút đổi thành **Stash N file đã stage**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-stash-cat-thay-doi/gitstudio-01.webp" alt="Tab Stash của GitStudio: Tệp đã stage gồm ghi-chu-test.md và FareCalculatorTests.cs, Tiêu đề stash Test phụ phí giờ cao điểm dở dang, Mô tả Cất tạm để chạy lại test trên code sạch theo yêu cầu của Lan, nút Stash 2 file đã stage" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · tab Stash: cất phần đã stage, có tiêu đề và mô tả</figcaption></figure>

Vì file ghi chú đã được stage, nó được cất cùng (tương đương `git stash push --staged`). Sau khi stash, đồ thị có thêm một **hàng stash** (biểu tượng hộp, chữ nghiêng) phía trên commit nó dựa vào, thanh bên trái mục **Stash** có số 1, và nút **Pop** trên thanh công cụ hiện số stash đang có.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-stash-cat-thay-doi/gitstudio-02.webp" alt="Đồ thị GitStudio sau khi stash: hàng trên cùng có biểu tượng stash và chữ nghiêng Test phụ phí giờ cao điểm dở dang Cất tạm để chạy lại test, thanh bên mục Stash có số 1, nút Pop trên thanh công cụ có huy hiệu 1, bảng thay đổi trống" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · stash hiện trên đồ thị và ở thanh bên; nút <b>Pop</b> báo có 1 stash</figcaption></figure>

Tiêu đề và mô tả được ghép thành một dòng trong nội dung stash. Cách khác để stash nhanh: nút **Stash** trên thanh công cụ, hoặc Spotlight (`Ctrl+K`) → **Stash thay đổi** (cất mọi thay đổi, như `git stash`). Lấy lại: nút **Pop**, hoặc chuột phải stash ở thanh bên → **Áp dụng stash** / **Pop stash** / **Xoá stash**. Spotlight tìm được stash theo tiêu đề; `Ctrl+Enter` để áp dụng.

## 5. Khi nào không nên dùng stash

Stash là chỗ cất **tạm vài phút tới vài giờ**. Những dấu hiệu cho thấy nên dùng cách khác:

- **Stash list dài hơn 3 mục.** Không ai nhớ `stash@{4}` là gì. Thay đổi đáng giữ lâu nên thành commit trên một nhánh riêng (commit "WIP" trên nhánh của bạn rồi amend sau cũng được).
- **Cần làm song song lâu trên hai nhánh.** Dùng worktree (GIT-019) thay vì stash qua lại.
- **Stash rồi quên trên máy.** Stash không được push lên remote; đổi máy là mất.

Lab kết thúc bằng commit test giờ cao điểm (`70b18f6`, `(12.000 + 4.500 × 4) × 1,2 = 36.000 đ`) và kiểm `stash list` trống.

## 6. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Spotlight / phím | Lệnh |
|---|---|---|---|
| Cất mọi thay đổi | Nút Stash | Spotlight → Stash thay đổi | `git stash push -m "…"` |
| Cất phần đã stage, có mô tả | Tab Stash → Stash N file đã stage | | `git stash push --staged -m "…"` |
| Cất cả file untracked | Stage file untracked rồi stash | | `git stash push -u` |
| Xem danh sách | Thanh bên → Stash; hàng stash trên đồ thị | Spotlight, gõ tiêu đề | `git stash list` |
| Lấy lại và xoá | Nút Pop; chuột phải → Pop stash | | `git stash pop` |
| Lấy lại, giữ stash | Chuột phải → Áp dụng stash | Spotlight → `Ctrl+Enter` | `git stash apply` |
| Xoá | Chuột phải → Xoá stash | | `git stash drop` |

## 7. Lỗi thường gặp

- **Stash không có tiêu đề.** Sau vài lần không phân biệt nổi. Luôn `-m`.
- **Tưởng file mới đã được cất.** Mặc định stash bỏ qua file untracked; chuyển nhánh có thể mang file đó theo hoặc bị chặn. Dùng `-u` khi cần.
- **`git stash clear` nhầm.** Xoá hết stash, không hỏi. Stash vừa mất vẫn có thể tìm lại bằng `git fsck --unreachable | grep commit` trong một thời gian, nhưng đừng dựa vào đó.
- **Pop lên nhầm nhánh.** Thay đổi áp vào nhánh đang đứng. Kiểm ô **nhánh** trước khi Pop.
- **Pop gặp xung đột rồi tưởng stash đã mất.** Stash vẫn còn (Git chỉ xoá khi áp sạch); giải quyết xung đột rồi `drop`.

## 8. Bài tập

**Bài 1.** Dựng đầu bài, stash bằng `git stash` (không `-u`), rồi `git switch -c thu` và `git status`. File ghi chú ở đâu?

<details><summary><b>Lời giải</b></summary><p>Vẫn nằm trong working tree (untracked) và đi theo sang nhánh mới, vì <code>git stash</code> không cất file untracked. Dùng <code>-u</code> nếu muốn working tree thật sự sạch.</p></details>

**Bài 2.** `git stash show -p stash@{0}` khác `git stash show --stat` thế nào? Làm sao xem cả file untracked đã cất bằng `-u`?

<details><summary><b>Lời giải</b></summary><p><code>-p</code> in diff đầy đủ thay vì thống kê. File untracked nằm trong commit cha thứ ba của stash; Git 2.32+ có <code>git stash show --include-untracked</code> (hoặc <code>-u</code>) để hiện chúng.</p></details>

**Bài 3.** Bạn muốn áp cùng một stash lên `main` và lên `phat-hanh/1.0`. Dùng `pop` hay `apply`? Viết chuỗi lệnh.

<details><summary><b>Lời giải</b></summary><p><code>apply</code>: <code>git switch main</code>, <code>git stash apply</code>, commit; <code>git switch phat-hanh/1.0</code>, <code>git stash apply</code>, commit; cuối cùng <code>git stash drop</code>. Dùng <code>pop</code> ở lần đầu thì stash đã bị xoá trước lần thứ hai.</p></details>

## 9. Đọc tiếp

- [git-stash](https://git-scm.com/docs/git-stash): `push`, `--staged`, `--include-untracked`, `branch`.
- [Pro Git, Stashing and Cleaning](https://git-scm.com/book/en/v2/Git-Tools-Stashing-and-Cleaning).

Bài tiếp theo: [GIT-016 · Tag và phiên bản phát hành](/articles/git-tag-phien-ban).
