---
slug: git-bisect-tim-commit-loi
title: "Bisect: tìm commit làm sai cước trong 29 commit bằng tìm kiếm nhị phân"
summary: "Bộ phận vận hành báo chuyến 3,7 km bị tính 30.000 đ thay vì 31.000 đ. Bản 1.1.0 còn đúng, từ đó tới nay có 29 commit. Bài này dùng git bisect: đánh dấu good/bad, tự kiểm tra một bước, rồi để git bisect run chạy script kiểm tra cho tới khi chỉ ra commit lỗi đầu tiên; đọc bisect log, bisect skip, bisect reset, rồi revert commit lỗi. Trên GitStudio là menu Bisect của commit và banner Đang bisect với Tốt, Lỗi, Bỏ qua."
category: git
level: beginner
tags: [git, git-041, git-bisect, debugging, regression, gitstudio, toi-noi]
language: vi
---

Thứ Năm 26/11/2026, 13 giờ. Tổng đài của Tới Nơi nhận phàn nàn ngược chiều: khách được báo **ít tiền hơn** bảng giá. Chuyến ô tô 4 chỗ 3,7 km: 12.000 + 5.000 × 3,7 = 30.500 đ, bảng giá quy định làm tròn **lên** bội số 1.000 đ, tức 31.000 đ. App báo 30.000 đ. Mỗi chuyến như vậy công ty mất 1.000 đ.

Bản phát hành `v1.1.0` còn tính đúng. Từ đó tới `main` có 29 commit, cả của Minh lẫn của Lan, cả merge lẫn revert. Đọc từng commit thì mất cả buổi. **Bisect** tìm ra commit lỗi trong khoảng năm lần thử.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-041 · Phần 5/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/git-lich-su-log-do-thi-commit">GIT-006</a>, <a href="/articles/git-revert-commit-da-chia-se">GIT-033</a>. Lab: <code>npm run p05 -- 41</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 41</code>. Sau bài này, bạn tìm được commit đầu tiên gây lỗi bằng bisect thủ công và tự động, và biết chuẩn bị "test" cho bisect.</p>
</div>

## 1. Ý tưởng: chia đôi lịch sử

Bạn có một commit **lỗi** (`main`) và một commit **tốt** (`v1.1.0`). Lỗi xuất hiện ở đâu đó giữa hai mốc. Git checkout commit ở giữa; bạn thử; nếu commit đó tốt thì lỗi nằm ở nửa sau, nếu lỗi thì ở nửa trước. Mỗi lần thử loại một nửa: 29 commit cần khoảng log₂(29) ≈ 5 lần.

Điều kiện duy nhất: phải có cách **thử** một commit cho ra tốt/lỗi rõ ràng: chạy một test, gọi API với chuyến 3,7 km, hay đơn giản là đọc một dòng code.

## 2. Bắt đầu: đánh dấu hai mốc

```sh
git rev-list --count v1.1.0..main
git grep -n "atLeastMinimum / 1000m" v1.1.0 -- src
git grep -n "atLeastMinimum / 1000m" main -- src
```

```text
29
v1.1.0:src/BaoGia/FareCalculator.cs:24:        return Math.Ceiling(atLeastMinimum / 1000m) * 1000m;
main:src/BaoGia/FareCalculator.cs:24:        return Math.Round(atLeastMinimum / 1000m) * 1000m;
```

Ở đây lab "gian lận" một chút: dòng làm tròn đã đổi từ `Math.Ceiling` sang `Math.Round`, nên `git log -S` (GIT-042) cũng tìm ra ngay. Thực tế lỗi thường không lộ như vậy (một refactor, một thư viện cập nhật, một cấu hình), và bisect là cách chắc chắn nhất.

```sh
git bisect start
git bisect bad main
git bisect good v1.1.0
```

```text
status: waiting for both good and bad commits
status: waiting for good commit(s), bad commit known
Bisecting: 14 revisions left to test after this (roughly 4 steps)
[17b8c3c3a02ecb3df3c72c6da63bbcfdfd93997d] Tăng phí chờ tối đa lên 50.000 đ
```

Git đã checkout commit giữa khoảng (HEAD đang detached, GIT-008) và báo còn khoảng 4 bước.

## 3. Thử bằng tay một bước

```sh
git grep -n "atLeastMinimum / 1000m" -- src
git bisect good
```

```text
src/BaoGia/FareCalculator.cs:24:        return Math.Ceiling(atLeastMinimum / 1000m) * 1000m;
Bisecting: 7 revisions left to test after this (roughly 3 steps)
[5f830ad4cc84b3c58a8157d487c21948f2fec42a] Thêm runbook hoàn tác
```

Commit này còn `Math.Ceiling`: tốt. Git nhảy tới giữa nửa còn lại. Ba lệnh để trả lời: `git bisect good`, `git bisect bad`, và `git bisect skip` khi commit đó **không thử được** (không build được, thiếu dữ liệu…); Git chọn commit khác gần đó.

## 4. Để Git tự chạy: `bisect run`

Nếu việc thử viết được thành một lệnh có **mã thoát** 0 = tốt, khác 0 = lỗi, Git tự làm hết phần còn lại. Lab không có .NET, nên "test" là một script kiểm tra dòng làm tròn:

```sh
#!/bin/sh
# Dùng cho git bisect run: mã thoát 0 = commit tốt, khác 0 = commit lỗi.
grep -q 'Math.Ceiling(atLeastMinimum' src/BaoGia/FareCalculator.cs
```

Nhóm Tới Nơi dùng test thật: `git bisect run dotnet test --filter Quote_LamTronLen`. Mã thoát 125 có nghĩa "bỏ qua commit này" (như `skip`), dùng khi commit không build được.

```sh
git bisect run sh ../../p05/kiem-tra-lam-tron.sh
```

```text
running 'sh' '../../p05/kiem-tra-lam-tron.sh'
Bisecting: 3 revisions left to test after this (roughly 2 steps)
[5f5bc153d7f6e41ce7224252d4f1287b8c99aecb] Thêm test phí chờ tính theo giây
running 'sh' '../../p05/kiem-tra-lam-tron.sh'
Bisecting: 1 revision left to test after this (roughly 1 step)
[7297149815fd49c6079a82511c0da9b6df65c7ea] Ghi phụ phí đêm vào tài liệu bảng giá
running 'sh' '../../p05/kiem-tra-lam-tron.sh'
Bisecting: 0 revisions left to test after this (roughly 0 steps)
[6c0a1e8a36a8269173d819c1701dfc3e5f8deab5] Làm gọn phép làm tròn giá cước
running 'sh' '../../p05/kiem-tra-lam-tron.sh'
6c0a1e8a36a8269173d819c1701dfc3e5f8deab5 is the first bad commit
commit 6c0a1e8a36a8269173d819c1701dfc3e5f8deab5
Author: Trần Minh <minh@toinoi.test>
Date:   Thu Nov 26 09:00:00 2026 +0700

    Làm gọn phép làm tròn giá cước

 src/BaoGia/FareCalculator.cs | 4 ++--
bisect found first bad commit
```

Năm lần thử tổng cộng (một bằng tay, bốn tự động). Thủ phạm: "Làm gọn phép làm tròn giá cước", 9 giờ sáng cùng ngày. `Math.Round(30,5)` trong .NET làm tròn về số **chẵn** gần nhất (30), không phải làm tròn lên: commit "làm gọn" đã đổi nghĩa phép tính.

Script chạy từ thư mục gốc repository, và phải nằm **ngoài** những gì bisect checkout (hoặc có ở mọi commit), nếu không nó biến mất giữa chừng.

## 5. Ghi lại và kết thúc

```sh
git bisect log
git bisect reset
```

```text
git bisect start
# bad: [dbb47b7…] Hướng dẫn chạy riêng test tính cước
git bisect bad dbb47b7e089026561c3aafb56b48c6661ecf2980
# good: [70b18f6…] Thêm test cho phụ phí giờ cao điểm
git bisect good 70b18f6ef606e77c6d57b2ace5cdbd3652a5f7f2
…
# first bad commit: [6c0a1e8…] Làm gọn phép làm tròn giá cước
Previous HEAD position was 6c0a1e8 Làm gọn phép làm tròn giá cước
Switched to branch 'main'
```

`bisect log` là biên bản của cuộc điều tra: lưu vào file (`git bisect log > bisect.txt`) thì có thể chạy lại bằng `git bisect replay bisect.txt`, hoặc đính kèm báo cáo lỗi. `bisect reset` đưa HEAD về nhánh lúc bắt đầu; **luôn** nhớ reset, nếu không repository ở trạng thái bisect dở dang.

Commit lỗi đã push lên `main`, nên sửa bằng revert (GIT-033):

```sh
git revert --no-edit 6c0a1e8
git grep -n "atLeastMinimum / 1000m" -- src
git push -q
```

```text
[main f1d35ab] Revert "Làm gọn phép làm tròn giá cước"
 1 file changed, 2 insertions(+), 2 deletions(-)
src/BaoGia/FareCalculator.cs:24:        return Math.Ceiling(atLeastMinimum / 1000m) * 1000m;
```

GIT-049 sẽ thêm một hook để dòng này không bị đổi lần nữa mà không ai hay.

## 6. Trên GitStudio

- Chuột phải một commit → **Bisect** → **Bắt đầu bisect: commit này lỗi** (hoặc **…commit này tốt**). Banner hiện "Đang bisect: đánh dấu một commit chạy đúng (tốt) để bắt đầu tìm."
- Chuột phải commit mốc còn lại → **Bisect** → **Đánh dấu tốt** / **Đánh dấu lỗi**.
- Từ đó banner ghi "Đang bisect: kiểm tra 17b8c3c. Còn khoảng 14 commit (~4 bước)." với các nút **Tốt**, **Lỗi**, **Bỏ qua**, **Kết thúc bisect**. Bạn thử commit đang checkout (chạy app, chạy test trong terminal tích hợp) rồi bấm.
- Khi xong: "Đã tìm thấy: 6c0a1e8 "Làm gọn phép làm tròn giá cước" là commit lỗi đầu tiên." **Kết thúc bisect** chạy `git bisect reset`.

GitStudio không có nút `bisect run`; gõ lệnh trong terminal tích hợp, banner và đồ thị vẫn cập nhật theo.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-bisect-tim-commit-loi/gitstudio-01.webp" alt="Menu chuột phải của commit trên đồ thị GitStudio, mục Bisect mở ra Bắt đầu bisect: commit này lỗi và Bắt đầu bisect: commit này tốt" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · bắt đầu bisect từ menu commit</figcaption></figure>

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-bisect-tim-commit-loi/gitstudio-02.webp" alt="Banner Đang bisect của GitStudio: kiểm tra 17b8c3c, còn khoảng 14 commit, khoảng 4 bước, với các nút Tốt, Lỗi, Bỏ qua, Kết thúc bisect; thanh bên và đồ thị cho thấy HEAD tách rời ở commit đang kiểm tra" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · banner Đang bisect: thử commit đang checkout rồi bấm Tốt hoặc Lỗi</figcaption></figure>

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Bắt đầu, đánh dấu mốc lỗi | Chuột phải commit → Bisect → Bắt đầu bisect: commit này lỗi | `git bisect start`, `git bisect bad <commit>` |
| Đánh dấu mốc tốt | Bisect → Đánh dấu tốt | `git bisect good <commit>` |
| Trả lời cho commit đang thử | Banner → Tốt / Lỗi / Bỏ qua | `git bisect good` / `bad` / `skip` |
| Tự động bằng test | (terminal) | `git bisect run <lệnh>` |
| Xem biên bản | (terminal) | `git bisect log` |
| Kết thúc | Banner → Kết thúc bisect | `git bisect reset` |

## 8. Lỗi thường gặp

- **Đánh dấu nhầm một bước**: kết quả sai. `git bisect log > b.txt`, sửa dòng sai trong file, `git bisect reset`, rồi `git bisect replay b.txt`.
- **Test phụ thuộc thứ không có ở commit cũ** (file test mới thêm): mọi commit cũ đều "lỗi". Đặt script ngoài repository, hoặc trả 125 khi thiếu.
- **Quên `bisect reset`** rồi commit tiếp trên HEAD detached.
- **Mốc "tốt" thật ra cũng lỗi**: bisect vẫn chỉ ra một commit, nhưng sai. Kiểm tra mốc tốt trước khi bắt đầu.
- **Merge commit**: bisect xử lý được, nhưng nếu lỗi chỉ xuất hiện khi hai nhánh gặp nhau, thủ phạm có thể chính là merge commit; đọc kỹ.

## 9. Bài tập

**Bài 1.** Với 1.000 commit giữa mốc tốt và lỗi, bisect cần khoảng bao nhiêu lần thử?

<details><summary><b>Lời giải</b></summary><p>log₂(1000) ≈ 10 lần.</p></details>

**Bài 2.** Viết script cho `bisect run` trả 125 nếu commit chưa có file `src/BaoGia/FareCalculator.cs`.

<details><summary><b>Lời giải</b></summary><p><code>[ -f src/BaoGia/FareCalculator.cs ] || exit 125</code> ở đầu script, rồi tới lệnh kiểm tra.</p></details>

**Bài 3.** Bisect cũng dùng được để tìm commit **sửa** một lỗi (ví dụ lỗi tự hết từ lúc nào). Dùng từ khoá nào cho rõ nghĩa?

<details><summary><b>Lời giải</b></summary><p><code>git bisect start --term-old=con-loi --term-new=da-sua</code>, rồi <code>git bisect da-sua</code> / <code>con-loi</code> (hoặc <code>old</code>/<code>new</code>).</p></details>

## 10. Đọc tiếp

- [git-bisect](https://git-scm.com/docs/git-bisect), [Pro Git, Debugging with Git: Binary Search](https://git-scm.com/book/en/v2/Git-Tools-Debugging-with-Git).

Bài tiếp theo: [GIT-042 · Tìm kiếm nâng cao trong lịch sử](/articles/git-tim-kiem-lich-su-nang-cao).
