---
slug: git-xung-dot-merge
title: "Xung đột merge: đọc marker, dùng công cụ ba khung của GitStudio và giải quyết dòng tính cước"
summary: "Nhánh phụ phí giờ cao điểm và bản vá làm tròn quãng đường cùng sửa dòng var fare trong FareCalculator.cs. Bài này tái hiện xung đột thật khi merge, đọc git status và marker <<<<<<< ======= >>>>>>>, dùng màn hình Xem trước merge ba khung của GitStudio, tránh bẫy chọn cả hai bên ra hai dòng, huỷ merge, giải quyết đúng nghiệp vụ và kiểm lại bằng git show --remerge-diff."
category: git
level: intermediate
tags: [git, git-014, merge-conflict, conflict-marker, merge-tool, merge-abort, gitstudio, toi-noi]
language: vi
---

Sáng 17/11/2026, nhánh phụ phí giờ cao điểm đã xong, Minh merge vào `main`. Git dừng lại: `CONFLICT (content): Merge conflict in src/BaoGia/FareCalculator.cs`. Không phải lỗi của Git hay của ai: hôm qua anh sửa dòng `var fare` để nhân hệ số giờ cao điểm (trên nhánh tính năng), rồi cũng chính anh sửa dòng đó trên `main` để làm tròn quãng đường (GIT-012). Hai thay đổi đều đúng, và Git không thể tự biết kết quả nên là một trong hai, hay là cả hai.

Giải quyết xung đột là **quyết định nghiệp vụ**, không phải thao tác chọn bên. Bài này đi qua toàn bộ quy trình, trên GitStudio và bằng lệnh.

<div class="tkh-callout">
<span class="tkh-callout-title">GIT-014 · Phần 2/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/git-merge-fast-forward-merge-commit">GIT-013</a>. Lab: <code>npm run p02 -- 14</code>; mở trạng thái đầu bài bằng GitStudio: <code>npm run setup -- 14</code>. Sau bài này, bạn đọc được trạng thái merge dở dang và marker xung đột, dùng được công cụ xung đột của GitStudio, huỷ hoặc hoàn tất merge, và kiểm lại cách đã giải quyết.</p>
</div>

## 1. Vì sao có xung đột

Merge ba chiều (three-way merge) so **ba** phiên bản: commit chung gần nhất của hai nhánh (**merge base**), bản của nhánh nhận, bản của nhánh nguồn. Với mỗi đoạn code:

| Merge base → nhánh nhận | Merge base → nhánh nguồn | Kết quả |
|---|---|---|
| Không đổi | Đổi | Lấy bản nhánh nguồn |
| Đổi | Không đổi | Lấy bản nhánh nhận |
| Đổi giống hệt nhau | Đổi giống hệt nhau | Lấy bản chung |
| Đổi **khác nhau** cùng chỗ (hoặc sát nhau) | | **Xung đột**: con người quyết định |

Trước khi merge, nhìn hai nhánh đã đi những đâu:

```sh
git merge-base main tinh-nang/phu-phi-gio-cao-diem
git log --oneline main..tinh-nang/phu-phi-gio-cao-diem
git log --oneline tinh-nang/phu-phi-gio-cao-diem..main
```

```text
89855d5ae9eeadc33e9969090faabe175773a520
e4cd655 Nhân hệ số giờ cao điểm khi tính cước
c46c1dd Thêm hệ số phụ phí giờ cao điểm vào bảng giá
8c05257 Merge branch 'tai-lieu/gio-cao-diem'
3e7f15a Mô tả phụ phí giờ cao điểm trong tài liệu
efe9c33 Ghi cách làm tròn quãng đường vào tài liệu
7993c73 Làm tròn quãng đường tới 0,1 km trước khi tính cước
```

Merge base là cuối Phần 1 (`89855d5`). Commit `e4cd655` (nhánh tính năng) và `7993c73` (main) cùng đụng `FareCalculator.cs`.

## 2. Xem trước merge trên GitStudio

Chuột phải nhánh tính năng → **Merge tinh-nang/phu-phi-gio-cao-diem vào main**. Trước khi đụng vào repository, GitStudio mở **Xem trước merge**: "2 tệp thay đổi · 1 xung đột". Bên trái là danh sách: mục **Xung đột** có `FareCalculator.cs`, mục **Thay đổi** có `appsettings.json` (tự merge được). Khung giữa có ba phần:

- **A**: bản của `main` (nhánh nhận).
- **B**: bản của nhánh tính năng.
- **Kết quả** (dưới): file sau merge, chỗ xung đột tô màu, có nút **Sửa tay**.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-xung-dot-merge/gitstudio-01.webp" alt="Màn hình Xem trước merge của GitStudio: tinh-nang/phu-phi-gio-cao-diem vào main, 2 tệp thay đổi, 1 xung đột; khung A commit 8c05257 trên main có dòng var fare với Math.Round, khung B commit e4cd655 trên nhánh tính năng có chú thích giờ cao điểm và dòng var fare nhân peakMultiplier, khung Kết quả bên dưới tô vùng xung đột dòng 18-19; nút Lưu cách giải quyết, Reset và Merge" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · Xem trước merge: A = <code>main</code>, B = nhánh tính năng, Kết quả ở dưới</figcaption></figure>

Mỗi khối thay đổi trong A và B có một ô chọn; **Lấy tất cả** chọn mọi khối của một bên.

### Bẫy: chọn cả hai bên

Hai thay đổi đều đúng, nên phản xạ đầu tiên là đánh dấu cả A lẫn B. Kết quả:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-xung-dot-merge/gitstudio-02.webp" alt="Xem trước merge sau khi đánh dấu cả khối A và khối B: khung Kết quả có dòng 18-19 của A (chú thích cũ và var fare với Math.Round) và dòng 20-22 của B (chú thích giờ cao điểm và var fare nhân peakMultiplier), tức hai dòng var fare liên tiếp" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · chọn cả A và B: Kết quả có <b>hai dòng <code>var fare</code></b>, code không biên dịch được</figcaption></figure>

Kết quả có hai lần khai báo `var fare`. Ghép hai bên theo dòng không ghép được hai **ý**. Đáp án đúng là một dòng mới mà không bên nào có sẵn:

```csharp
var fare = (rule.BaseFare + rule.PerKm * Math.Round(distanceKm, 1, MidpointRounding.AwayFromZero)) * peakMultiplier;
```

Chọn B (để lấy chú thích giờ cao điểm và hệ số), rồi bấm **Sửa tay** trong khung Kết quả để thêm `Math.Round(…)` vào. **Lưu cách giải quyết**, rồi **Merge**.

## 3. Cũng việc đó bằng lệnh

```sh
git merge tinh-nang/phu-phi-gio-cao-diem
git status
```

```text
Auto-merging src/BaoGia/FareCalculator.cs
CONFLICT (content): Merge conflict in src/BaoGia/FareCalculator.cs
Automatic merge failed; fix conflicts and then commit the result.
On branch main
You have unmerged paths.
  (fix conflicts and run "git commit")
  (use "git merge --abort" to abort the merge)

Changes to be committed:
	modified:   src/BaoGia/appsettings.json

Unmerged paths:
  (use "git add <file>..." to mark resolution)
	both modified:   src/BaoGia/FareCalculator.cs
```

Repository đang ở trạng thái **merge dở dang**: `appsettings.json` đã merge xong và nằm trong staging area; `FareCalculator.cs` là **unmerged** (`both modified`). Git ghi cả hai phiên bản vào file, giữa các **marker**:

```sh
git diff
```

```diff
diff --cc src/BaoGia/FareCalculator.cs
index 3f8121c,1337a96..0000000
--- a/src/BaoGia/FareCalculator.cs
+++ b/src/BaoGia/FareCalculator.cs
@@@ -15,8 -15,9 +15,14 @@@ public static class FareCalculato
          // Giá = giá mở cửa + đơn giá mỗi km × số km.
          // Ví dụ ô tô 4 chỗ đi 4 km: 12.000 + 4.500 × 4 = 30.000 đ.
          // Chuyến quá ngắn vẫn trả ít nhất giá tối thiểu.
++<<<<<<< HEAD
 +        // Chưa gồm phụ phí giờ cao điểm, phí cầu đường, khuyến mãi.
 +        var fare = rule.BaseFare + rule.PerKm * Math.Round(distanceKm, 1, MidpointRounding.AwayFromZero);
++=======
+         // Giờ cao điểm: nhân hệ số Peak.Multiplier (mặc định 1, ngoài giờ cao điểm).
+         // Chưa gồm phí cầu đường, khuyến mãi.
+         var fare = (rule.BaseFare + rule.PerKm * distanceKm) * peakMultiplier;
++>>>>>>> tinh-nang/phu-phi-gio-cao-diem
          var atLeastMinimum = Math.Max(fare, rule.MinimumFare);
```

| Marker | Nghĩa |
|---|---|
| `<<<<<<< HEAD` | Bắt đầu phần của nhánh đang đứng (**ours**, ở đây là `main`) |
| `=======` | Ranh giới |
| `>>>>>>> tinh-nang/…` | Kết thúc phần của nhánh được merge vào (**theirs**) |

`git diff` lúc merge dở dang là **diff gộp** (`diff --cc`): hai cột dấu `+`/`-`, mỗi cột cho một bên. Chữ ký hàm (`peakMultiplier = 1m`) không xung đột vì chỉ nhánh tính năng đổi nó: Git đã tự lấy.

Trên GitStudio, cùng lúc đó thanh trên cùng hiện banner **Đang merge. Giải quyết 1 tệp bị xung đột, stage chúng, rồi tiếp tục.** với hai nút **Huỷ thao tác** và **Tiếp tục**; bảng bên phải có mục **Xung đột**, nút commit đổi thành "Giải quyết xung đột để commit". Bấm file xung đột để mở cùng công cụ ba khung như phần Xem trước.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-xung-dot-merge/gitstudio-03.webp" alt="GitStudio trong lúc merge dở dang: banner vàng Đang merge với nút Huỷ thao tác và Tiếp tục, hàng WIP chọn, Tệp chưa stage có mục Xung đột chứa src/BaoGia/FareCalculator.cs, Tệp đã stage có appsettings.json, nút Giải quyết xung đột để commit" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · merge dở dang: <code>FareCalculator.cs</code> trong mục Xung đột, <code>appsettings.json</code> đã stage</figcaption></figure>

## 4. Huỷ merge

Chưa sẵn sàng quyết định (cần hỏi Lan, cần chạy test)? Huỷ:

```sh
git merge --abort
git status --short --branch
```

```text
## main
```

Repository trở về đúng trạng thái trước `git merge`. Trên GitStudio là nút **Huỷ thao tác** trên banner. Đừng để repository ở trạng thái merge dở dang qua đêm: commit tiếp theo của bạn sẽ thành merge commit lúc nào không biết.

## 5. Giải quyết và hoàn tất

Merge lại, sửa file thành bản đúng (một dòng `var fare` có cả làm tròn lẫn hệ số, xoá hết marker), rồi đánh dấu đã giải quyết bằng `git add`:

```sh
git add src/BaoGia/FareCalculator.cs
git status --short
git merge --continue
git log --oneline --graph -6
```

```text
M  src/BaoGia/FareCalculator.cs
M  src/BaoGia/appsettings.json
[main e46c89f] Merge branch 'tinh-nang/phu-phi-gio-cao-diem'
*   e46c89f Merge branch 'tinh-nang/phu-phi-gio-cao-diem'
|\  
| * e4cd655 Nhân hệ số giờ cao điểm khi tính cước
| * c46c1dd Thêm hệ số phụ phí giờ cao điểm vào bảng giá
* |   8c05257 Merge branch 'tai-lieu/gio-cao-diem'
|\ \  
| * | 3e7f15a Mô tả phụ phí giờ cao điểm trong tài liệu
|/ /  
* | efe9c33 Ghi cách làm tròn quãng đường vào tài liệu
```

`git merge --continue` tạo merge commit (giống `git commit` lúc này). Nếu Git mở trình soạn thảo, nội dung mặc định có thêm vài dòng `# Conflicts:` bắt đầu bằng `#`: đó là chú thích, Git bỏ đi khi lưu. Cẩn thận với `git commit --no-edit` ở bước này: không qua trình soạn thảo thì các dòng chú thích đó có thể ở lại trong nội dung commit. Trên GitStudio, bấm **Tiếp tục** trên banner.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/git-xung-dot-merge/gitstudio-04.webp" alt="Đồ thị GitStudio sau merge: main ở merge commit Merge branch tinh-nang/phu-phi-gio-cao-diem, nhánh tính năng hai commit giờ cao điểm nhập lại, bên dưới là merge commit của nhánh tài liệu" width="2000" height="1125" loading="lazy"></div><figcaption>GitStudio · sau khi giải quyết: merge commit <code>e46c89f</code> nối hai nhánh</figcaption></figure>

## 6. Kiểm lại cách đã giải quyết

Merge commit thường không hiện diff khi `git show` (diff gộp chỉ hiện chỗ khác cả hai cha). Git 2.36 trở lên có `--remerge-diff`: tái hiện merge tự động (kèm marker) rồi so với bản bạn đã commit, tức là đúng "bạn đã giải quyết thế nào":

```sh
git show --remerge-diff --format=%h%x20%s HEAD
```

```diff
e46c89f Merge branch 'tinh-nang/phu-phi-gio-cao-diem'

diff --git a/src/BaoGia/FareCalculator.cs b/src/BaoGia/FareCalculator.cs
remerge CONFLICT (content): Merge conflict in src/BaoGia/FareCalculator.cs
--- a/src/BaoGia/FareCalculator.cs
+++ b/src/BaoGia/FareCalculator.cs
@@ -15,14 +15,9 @@ public static class FareCalculator
-<<<<<<< 8c05257 (Merge branch 'tai-lieu/gio-cao-diem')
-        // Chưa gồm phụ phí giờ cao điểm, phí cầu đường, khuyến mãi.
-        var fare = rule.BaseFare + rule.PerKm * Math.Round(distanceKm, 1, MidpointRounding.AwayFromZero);
-=======
         // Giờ cao điểm: nhân hệ số Peak.Multiplier (mặc định 1, ngoài giờ cao điểm).
         // Chưa gồm phí cầu đường, khuyến mãi.
-        var fare = (rule.BaseFare + rule.PerKm * distanceKm) * peakMultiplier;
->>>>>>> e4cd655 (Nhân hệ số giờ cao điểm khi tính cước)
+        var fare = (rule.BaseFare + rule.PerKm * Math.Round(distanceKm, 1, MidpointRounding.AwayFromZero)) * peakMultiplier;
```

(Đã lược vài dòng ngữ cảnh.) Người review đọc đúng chỗ này để duyệt merge có xung đột. Cuối cùng xoá nhánh đã merge: `git branch -d tinh-nang/phu-phi-gio-cao-diem`.

Sau merge **luôn build và chạy test**. "Hết marker" chưa có nghĩa là "đúng": code có thể biên dịch được mà vẫn sai nghiệp vụ (ví dụ nhân hệ số trước khi làm tròn quãng đường hay sau đều cho cùng kết quả ở đây, nhưng không phải lúc nào cũng vậy).

## 7. Đối chiếu giao diện và lệnh

| Việc | GitStudio | Lệnh |
|---|---|---|
| Xem trước xung đột trước khi merge | Merge … vào main → Xem trước merge | `git merge-tree --write-tree main <nhánh>` |
| Xem trạng thái merge dở dang | Banner Đang merge, mục Xung đột | `git status` |
| Lấy hẳn một bên cho cả file | Chuột phải file → Lấy bản của mình / Lấy bản của họ | `git checkout --ours/--theirs <file>` |
| Sửa tay | Công cụ xung đột → Sửa tay | trình soạn thảo |
| Đánh dấu đã giải quyết | Lưu cách giải quyết / Đánh dấu đã giải quyết | `git add <file>` |
| Hoàn tất | Tiếp tục | `git merge --continue` |
| Huỷ | Huỷ thao tác | `git merge --abort` |
| Xem đã giải quyết thế nào | | `git show --remerge-diff <merge>` |

## 8. Lỗi thường gặp

- **Commit file còn marker.** `<<<<<<<` vào lịch sử, build hỏng. Tìm trước khi commit: `git diff --check` hoặc `git grep -n '^<<<<<<< '`.
- **"Lấy bản của mình" cho nhanh.** Mất trọn thay đổi của bên kia (ở đây là mất phụ phí giờ cao điểm) mà không báo gì.
- **Chọn cả hai bên trong công cụ xung đột** rồi không kiểm kết quả: hai dòng trùng ý như mục 2.
- **Quên rằng merge chưa xong.** Commit sau đó thành merge commit ngoài ý muốn. Banner của GitStudio nhắc; trong terminal, `git status` nhắc.
- **Giải quyết đúng nhưng không chạy test.** Xung đột "logic" (hai thay đổi không đụng cùng dòng nhưng mâu thuẫn ý nhau) Git không phát hiện được; chỉ test phát hiện.

## 9. Bài tập

**Bài 1.** Dựng đầu bài, merge bằng lệnh, rồi chạy `git checkout --theirs src/BaoGia/FareCalculator.cs` và xem file. Kết quả giống chọn bên nào trong GitStudio? Vì sao đây không phải đáp án đúng?

<details><summary><b>Lời giải</b></summary><p>Giống chọn B (nhánh tính năng): dòng <code>var fare</code> nhân hệ số nhưng không làm tròn quãng đường, tức mất bản vá của <code>main</code>. Test <code>Quote_LamTronQuangDuong01Km</code> (3,12 km → 26.000 đ) sẽ đỏ vì ra 27.000 đ.</p></details>

**Bài 2.** Vì sao `appsettings.json` không xung đột dù nhánh tính năng thêm cấu hình giờ cao điểm vào cuối file?

<details><summary><b>Lời giải</b></summary><p>Từ merge base tới <code>main</code>, <code>appsettings.json</code> không đổi (bản vá chỉ sửa code và test). Chỉ một bên đổi file, nên Git lấy bản của bên đó, không cần hỏi.</p></details>

**Bài 3.** Chạy `git merge-tree --write-tree main tinh-nang/phu-phi-gio-cao-diem` ở đầu bài (Git 2.38+). Lệnh cho biết gì mà không đụng tới working tree?

<details><summary><b>Lời giải</b></summary><p>In mã hash của tree kết quả, ba dòng cho file xung đột (blob ở merge base, bên <code>main</code>, bên nhánh tính năng, đánh số stage 1, 2, 3), rồi thông báo <code>CONFLICT (content): Merge conflict in src/BaoGia/FareCalculator.cs</code>, và thoát mã 1. Đây là cách kiểm "merge có sạch không" trong script hay CI mà không phải checkout.</p></details>

## 10. Đọc tiếp

- [git-merge, mục HOW CONFLICTS ARE PRESENTED](https://git-scm.com/docs/git-merge#_how_conflicts_are_presented): marker, `merge.conflictStyle=zdiff3`.
- [git-show `--remerge-diff`](https://git-scm.com/docs/git-show), [git-merge-tree](https://git-scm.com/docs/git-merge-tree).
- [Pro Git, Advanced Merging](https://git-scm.com/book/en/v2/Git-Tools-Advanced-Merging).

Bài tiếp theo: [GIT-015 · Stash: cất thay đổi dở dang](/articles/git-stash-cat-thay-doi).
